//! In-process coverage for installing this project's own skill packages.
//!
//! GitHub #34, GitHub #45, and ADR 0016 assign installer behavior to the public
//! library interface. The process boundary receives one separate test for argv
//! dispatch, exit status, and stream routing.

mod support;

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use reuse_evidence::{ExitMeaning, install_skills};
use support::TempRoot;

const CAPTURE_SKILL: &str = ".claude/skills/reuse-evidence-capture/SKILL.md";
const REVIEW_SKILL: &str = ".claude/skills/reuse-evidence-review/SKILL.md";
const REVIEW_ANALYSIS: &str = ".claude/skills/reuse-evidence-review/references/review-analysis.md";
const DISCOVERY_LINK: &str = ".agents/skills/reuse-evidence-capture";
const DISCOVERY_TARGET: &str = "../../.claude/skills/reuse-evidence-capture";

#[test]
fn a_missing_target_root_refuses_without_writing() {
    let fixture = TempRoot::new("skill-install-missing-root");
    let missing = fixture.join("missing-repository");
    let before = support::snapshot(&fixture);

    let Err(failure) = install_skills(&missing, false) else {
        panic!("a missing target repository root must refuse");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: target repository root `{}` does not exist\nresolution: create the target repository directory, then rerun `install-skills --root <ROOT>`",
            missing.display()
        )
    );
    assert_eq!(
        support::snapshot(&fixture),
        before,
        "the missing-root refusal must preserve every existing byte"
    );
}

#[test]
fn a_fresh_install_writes_every_shipped_package_and_relative_discovery_link_only() {
    let fixture = TempRoot::new("skill-install-fresh");
    let repository = support::git_repository(&fixture, "target");
    let shipped_files = shipped_files_at(Path::new(env!("CARGO_MANIFEST_DIR")));
    let upstream_file = repository.join(".claude/skills/skill-evidence-capture/SKILL.md");
    fs::create_dir_all(
        upstream_file
            .parent()
            .expect("the upstream skill file should have a parent"),
    )
    .expect("the upstream package directory should be creatable");
    fs::write(&upstream_file, b"upstream package sentinel\n")
        .expect("the upstream package sentinel should be writable");
    let upstream_link = repository.join(".agents/skills/skill-evidence-capture");
    fs::create_dir_all(
        upstream_link
            .parent()
            .expect("the upstream discovery link should have a parent"),
    )
    .expect("the discovery directory should be creatable");
    create_directory_symlink(
        Path::new("../../.claude/skills/skill-evidence-capture"),
        &upstream_link,
    );

    let outcome = install_skills(&repository, false)
        .expect("a fresh target repository should accept the shipped package");

    for (relative_path, shipped_bytes) in &shipped_files {
        let installed = repository.join(relative_path);
        assert!(
            installed.is_file(),
            "installer should write {relative_path}"
        );
        assert_eq!(
            fs::read(&installed).expect("an installed skill file should be readable"),
            *shipped_bytes,
            "installed {relative_path} should be byte-identical to the shipped source"
        );
    }
    assert_eq!(
        shipped_files_at(&repository),
        shipped_files,
        "the installed reserved-prefix file set must exactly match the live source tree"
    );
    let mut expected = String::from("installed reuse-evidence skill packages\nwritten:\n");
    for (relative_path, _) in &shipped_files {
        writeln!(&mut expected, "- {relative_path}")
            .expect("writing expected receipt text to a String cannot fail");
    }
    assert_fresh_discovery_link_and_extend_receipt(&repository, &mut expected);
    assert_eq!(outcome.to_string(), expected);

    assert_eq!(
        fs::read(&upstream_file).expect("the upstream sentinel should remain readable"),
        b"upstream package sentinel\n",
        "the project-owned installer must not touch an upstream package name"
    );
    #[cfg(unix)]
    assert_eq!(
        fs::read_link(&upstream_link).expect("the upstream discovery link should remain"),
        PathBuf::from("../../.claude/skills/skill-evidence-capture"),
        "the project-owned installer must not touch an upstream discovery link"
    );
    assert!(
        !repository.join("reuse-evidence.toml").exists(),
        "installing a skill must not enroll its target repository"
    );
    assert!(
        !repository.join("reuse-evidence").exists(),
        "installing a skill must not create case, cache, lock, or index state"
    );
}

#[test]
fn unchanged_installations_and_this_repository_are_write_free_no_ops() {
    let fixture = TempRoot::new("skill-install-no-op");
    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false).expect("the fixture should install once");

    assert_unchanged_install_is_a_no_op("second unchanged run", &repository);
    assert_unchanged_install_is_a_no_op(
        "this repository's embedded source package",
        Path::new(env!("CARGO_MANIFEST_DIR")),
    );
}

#[test]
fn a_non_force_install_names_every_differing_file_and_writes_nothing() {
    let fixture = TempRoot::new("skill-install-conflicts");
    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false).expect("the fixture should install once");
    let first_conflict = CAPTURE_SKILL;
    let second_conflict = REVIEW_ANALYSIS;
    fs::write(repository.join(first_conflict), b"local capture edit\n")
        .expect("the first installed file should be editable");
    fs::write(
        repository.join(second_conflict),
        b"local proposal guidance edit\n",
    )
    .expect("the second installed file should be editable");
    let unrelated = repository.join("notes/keep.txt");
    fs::create_dir_all(
        unrelated
            .parent()
            .expect("the unrelated file should have a parent"),
    )
    .expect("the unrelated directory should be creatable");
    fs::write(&unrelated, b"unrelated repository bytes\n")
        .expect("the unrelated file should be writable");
    let before = support::snapshot(&repository);
    #[cfg(unix)]
    let link_before = fs::read_link(repository.join(DISCOVERY_LINK))
        .expect("the discovery link should be readable before refusal");

    let Err(failure) = install_skills(&repository, false) else {
        panic!("a non-force install over differing shipped files must refuse");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: installed reuse-evidence skill paths differ from the package this binary ships:\n- {first_conflict}\n- {second_conflict}\nresolution: rerun with `install-skills --root <ROOT> --force` to replace every differing path"
        )
    );
    assert_eq!(
        support::snapshot(&repository),
        before,
        "conflict refusal must preserve installed and unrelated file bytes"
    );
    #[cfg(unix)]
    assert_eq!(
        fs::read_link(repository.join(DISCOVERY_LINK))
            .expect("the discovery link should remain readable after refusal"),
        link_before,
        "conflict refusal must preserve the discovery link"
    );
}

#[test]
fn force_replaces_modified_files_and_restores_missing_files_explicitly() {
    let fixture = TempRoot::new("skill-install-force");
    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false).expect("the fixture should install once");
    let modified = CAPTURE_SKILL;
    let missing = REVIEW_ANALYSIS;
    fs::write(repository.join(modified), b"local capture edit\n")
        .expect("the installed file should be editable");
    fs::remove_file(repository.join(missing)).expect("the installed file should be removable");

    let outcome = install_skills(&repository, true)
        .expect("force should replace modified files and restore missing files");

    assert_eq!(
        outcome.to_string(),
        format!(
            "installed reuse-evidence skill packages\nwritten:\n- {missing}\nreplaced:\n- {modified}\n"
        )
    );
    for (relative_path, shipped_bytes) in shipped_files_at(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        assert_eq!(
            fs::read(repository.join(&relative_path))
                .expect("every installed file should be readable after force"),
            shipped_bytes,
            "force should leave {relative_path} byte-identical to the shipped source"
        );
    }
    #[cfg(unix)]
    assert_eq!(
        fs::read_link(repository.join(DISCOVERY_LINK))
            .expect("a correct discovery link should remain readable"),
        PathBuf::from(DISCOVERY_TARGET),
        "force should leave a correct discovery link alone and unreported"
    );
}

#[test]
fn stale_retired_assets_refuse_the_whole_set_even_under_force() {
    let fixture = TempRoot::new("skill-install-stale-retired-package");
    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false).expect("the fixture should install once");
    let differing = REVIEW_SKILL;
    fs::write(repository.join(differing), b"local review edit\n")
        .expect("a shipped review file should be editable");
    let stale_package = ".claude/skills/reuse-evidence-retired";
    let stale_file = ".claude/skills/reuse-evidence-retired/SKILL.md";
    fs::create_dir(repository.join(stale_package))
        .expect("the retired package directory should be creatable");
    fs::write(repository.join(stale_file), b"retired instructions\n")
        .expect("the retired skill should be writable");
    let stale_orphan = ".claude/skills/reuse-evidence-review/references/retired-guidance.md";
    fs::write(repository.join(stale_orphan), b"orphaned review guidance\n")
        .expect("an orphaned file inside a current package should be writable");
    let stale_link = ".agents/skills/reuse-evidence-retired";
    fs::write(repository.join(stale_link), b"retired discovery path\n")
        .expect("the retired discovery path should be writable");
    let before = support::snapshot(&repository);

    let Err(failure) = install_skills(&repository, true) else {
        panic!("force must not replace current files while stale paths remain");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: installed reuse-evidence skill paths differ from the package this binary ships:\n- {differing}\nstale reuse-evidence skill paths are not shipped by this binary:\n- {stale_package}\n- {stale_file}\n- {stale_orphan}\n- {stale_link}\nresolution: remove every stale path or move it outside the reserved `reuse-evidence-` package prefix, then rerun `install-skills --root <ROOT> --force` to replace every differing path"
        )
    );
    assert_eq!(
        support::snapshot(&repository),
        before,
        "stale-path refusal must preserve current, retired, and unrelated bytes"
    );
}

#[test]
fn an_owned_leftover_temporary_is_ignored_and_never_swept() {
    let fixture = TempRoot::new("skill-install-leftover-temporary");
    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false).expect("the fixture should install once");
    let temporary = repository.join(
        ".claude/skills/reuse-evidence-review/references/.review-analysis.md.00000000-0000-4000-8000-000000000000.tmp",
    );
    fs::write(&temporary, b"interrupted staged bytes\n")
        .expect("a leftover installer temporary should be writable");
    let before = support::snapshot(&repository);

    let outcome = install_skills(&repository, false)
        .expect("an installer-owned leftover temporary must not be classified as stale");

    assert_eq!(
        outcome.to_string(),
        "reuse-evidence skill packages already installed\nnothing needed writing\n"
    );
    assert_eq!(
        support::snapshot(&repository),
        before,
        "a later run must neither report nor sweep an interrupted temporary"
    );
}

#[cfg(unix)]
#[test]
fn the_temporary_exemption_requires_a_regular_file_and_the_exact_generated_name() {
    let fixture = TempRoot::new("skill-install-temporary-exemption-boundary");
    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false).expect("the fixture should install once");
    let canonical_symlink = ".claude/skills/reuse-evidence-review/references/.review-analysis.md.00000000-0000-4000-8000-000000000000.tmp";
    std::os::unix::fs::symlink(
        "not-an-installer-temporary",
        repository.join(canonical_symlink),
    )
    .expect("a symlink with the temporary name should be creatable");
    let non_rfc4122_file = ".claude/skills/reuse-evidence-review/references/.review-analysis.md.00000000-0000-4000-0000-000000000000.tmp";
    fs::write(
        repository.join(non_rfc4122_file),
        b"UUID variant the installer cannot generate\n",
    )
    .expect("a non-RFC4122 UUID should be writable");
    let noncanonical_file = ".claude/skills/reuse-evidence-review/references/.review-analysis.md.00000000000040008000000000000000.tmp";
    fs::write(
        repository.join(noncanonical_file),
        b"noncanonical temporary name\n",
    )
    .expect("a noncanonical UUID spelling should be writable");
    let before = support::snapshot(&repository);

    let Err(failure) = install_skills(&repository, false) else {
        panic!("only exact installer-owned staged files may bypass stale-path refusal");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: stale reuse-evidence skill paths are not shipped by this binary:\n- {non_rfc4122_file}\n- {canonical_symlink}\n- {noncanonical_file}\nresolution: remove every stale path or move it outside the reserved `reuse-evidence-` package prefix, then rerun `install-skills --root <ROOT>`"
        )
    );
    assert_eq!(
        support::snapshot(&repository),
        before,
        "the stale-path refusal must preserve both false temporary lookalikes"
    );
}

#[cfg(unix)]
#[test]
fn incorrect_discovery_links_and_non_links_are_reported_then_force_replaces_them() {
    for obstruction in ["incorrect link", "non-link"] {
        let fixture = TempRoot::new(&format!(
            "skill-install-discovery-obstruction-{}",
            obstruction.replace(' ', "-")
        ));
        let repository = support::git_repository(&fixture, "target");
        install_skills(&repository, false).expect("the fixture should install once");
        let link = repository.join(DISCOVERY_LINK);
        fs::remove_file(&link).expect("the correct discovery link should be removable");
        if obstruction == "incorrect link" {
            create_directory_symlink(Path::new("../../elsewhere"), &link);
        } else {
            fs::write(&link, b"a non-link occupies this discovery path\n")
                .expect("the non-link obstruction should be writable");
        }
        let before = support::snapshot(&repository);
        let link_before = fs::read_link(&link).ok();

        let Err(failure) = install_skills(&repository, false) else {
            panic!("{obstruction}: a non-force run must report the differing discovery path");
        };

        assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{obstruction}");
        assert_eq!(
            failure.to_string(),
            format!(
                "refusal: installed reuse-evidence skill paths differ from the package this binary ships:\n- {DISCOVERY_LINK}\nresolution: rerun with `install-skills --root <ROOT> --force` to replace every differing path"
            ),
            "{obstruction}"
        );
        assert_eq!(
            support::snapshot(&repository),
            before,
            "{obstruction}: refusal must preserve every file byte"
        );
        assert_eq!(
            fs::read_link(&link).ok(),
            link_before,
            "{obstruction}: refusal must preserve the occupying path"
        );

        let outcome = install_skills(&repository, true)
            .unwrap_or_else(|error| panic!("{obstruction}: force should replace it: {error}"));
        assert_eq!(
            outcome.to_string(),
            format!(
                "installed reuse-evidence skill packages\ndiscovery links replaced:\n- {DISCOVERY_LINK} -> {DISCOVERY_TARGET}\n"
            ),
            "{obstruction}"
        );
        assert_eq!(
            fs::read_link(&link).expect("force should leave a discovery symlink"),
            PathBuf::from(DISCOVERY_TARGET),
            "{obstruction}: force should restore the exact relative target"
        );
    }
}

#[cfg(unix)]
#[test]
fn a_symlinked_installation_directory_refuses_before_writing_inside_or_outside_the_target() {
    let fixture = TempRoot::new("skill-install-symlinked-directory");
    let repository = support::git_repository(&fixture, "target");
    let outside = fixture.join("outside-claude");
    fs::create_dir_all(&outside).expect("the outside directory should be creatable");
    let redirected = repository.join(".claude");
    create_directory_symlink(&outside, &redirected);
    let before = support::snapshot(&fixture);

    let Err(failure) = install_skills(&repository, true) else {
        panic!("a symlinked installation directory must refuse even under force");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: skill installation directory `{}` is a symbolic link\nresolution: replace every skill installation directory symlink with a real directory beneath the target repository",
            redirected.display()
        )
    );
    assert_eq!(
        support::snapshot(&fixture),
        before,
        "the directory refusal must not write through the symlink or elsewhere"
    );
}

#[test]
fn force_refuses_unreplaceable_installed_paths_before_replacing_other_files() {
    let fixture = TempRoot::new("skill-install-unreplaceable-path");
    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false).expect("the fixture should install once");
    let replaceable = CAPTURE_SKILL;
    let unreplaceable = REVIEW_ANALYSIS;
    fs::write(repository.join(replaceable), b"local capture edit\n")
        .expect("the replaceable installed file should be editable");
    fs::remove_file(repository.join(unreplaceable))
        .expect("the installed file should be removable");
    fs::create_dir(repository.join(unreplaceable))
        .expect("a directory should be able to occupy the installed-file path");
    let before = support::snapshot(&repository);

    let Err(failure) = install_skills(&repository, true) else {
        panic!("force must refuse a path it cannot replace atomically");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: installed reuse-evidence skill paths cannot be replaced atomically:\n- {unreplaceable}\nresolution: remove every listed directory or special path, then rerun `install-skills --root <ROOT> --force`"
        )
    );
    assert_eq!(
        support::snapshot(&repository),
        before,
        "force refusal must happen before a replaceable file is changed"
    );
}

#[test]
fn non_force_unreplaceable_refusal_names_an_effective_resolution() {
    let fixture = TempRoot::new("skill-install-unreplaceable-non-force");
    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false).expect("the fixture should install once");
    let replaceable = CAPTURE_SKILL;
    let unreplaceable = REVIEW_ANALYSIS;
    fs::write(repository.join(replaceable), b"local capture edit\n")
        .expect("the replaceable installed file should be editable");
    fs::remove_file(repository.join(unreplaceable))
        .expect("the installed file should be removable");
    fs::create_dir(repository.join(unreplaceable))
        .expect("a directory should be able to occupy the installed-file path");
    let before = support::snapshot(&repository);

    let Err(failure) = install_skills(&repository, false) else {
        panic!("a non-force run must refuse every differing installed path");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: installed reuse-evidence skill paths differ from the package this binary ships:\n- {replaceable}\n- {unreplaceable}\npaths that cannot be replaced atomically:\n- {unreplaceable}\nresolution: remove every path listed as unreplaceable, then rerun `install-skills --root <ROOT> --force` to replace the remaining differing paths"
        )
    );
    assert_eq!(
        support::snapshot(&repository),
        before,
        "non-force refusal must remain write-free"
    );
}

#[cfg(unix)]
#[test]
fn unsupported_link_replacement_reports_the_removed_obstruction() {
    let fixture = TempRoot::new("skill-install-unsupported-link-replacement");
    let probe_target = fixture.join("probe-target");
    let probe_link = fixture.join("probe-link");
    fs::create_dir(&probe_target).expect("the capability probe target should be creatable");
    match std::os::unix::fs::symlink(&probe_target, &probe_link) {
        Ok(()) => {
            fs::remove_file(&probe_link).expect("the capability probe link should be removable");
            return;
        }
        Err(error) if error.kind() == std::io::ErrorKind::Unsupported => {}
        Err(error) => panic!("the capability probe failed unexpectedly: {error}"),
    }

    let repository = support::git_repository(&fixture, "target");
    install_skills(&repository, false)
        .expect("real skill files should install when links are unsupported");
    let link = repository.join(DISCOVERY_LINK);
    fs::write(&link, b"obsolete discovery obstruction\n")
        .expect("the unsupported discovery path should be obstructable");

    let outcome = install_skills(&repository, true)
        .expect("force should remove the owned obstruction without requiring a link");

    assert_eq!(
        outcome.to_string(),
        format!(
            "installed reuse-evidence skill packages\nremoved discovery paths:\n- {DISCOVERY_LINK}\ndiscovery links not created: symbolic links are not supported on this platform\n- {DISCOVERY_LINK}\n"
        )
    );
    assert!(
        fs::symlink_metadata(&link).is_err(),
        "the reported discovery obstruction should be absent"
    );
    let rerun = install_skills(&repository, false)
        .expect("the forced result should be an ordinary unchanged installation");
    assert!(
        rerun.to_string().contains("nothing needed writing"),
        "{rerun}"
    );
}

#[cfg(windows)]
#[test]
fn a_symlink_capable_windows_target_receives_the_relative_discovery_link() {
    let fixture = windows_temp_root("skill-install-windows-link-capability");
    let repository = support::git_repository(&fixture, "target");
    let links_supported = windows_supports_directory_links(&fixture);
    write_matching_shipped_files(&repository);

    let outcome = install_skills(&repository, false)
        .expect("a Windows installation should succeed with or without link capability");

    if links_supported {
        assert_eq!(
            fs::read_link(repository.join(DISCOVERY_LINK))
                .expect("a capable Windows target should receive a discovery link"),
            PathBuf::from(DISCOVERY_TARGET)
        );
        assert!(
            outcome.to_string().contains(&format!(
                "discovery links created:\n- {DISCOVERY_LINK} -> {DISCOVERY_TARGET}"
            )),
            "{outcome}"
        );
    } else {
        assert!(
            outcome.to_string().contains(&format!(
                "discovery links not created: symbolic links are not supported on this platform\n- {DISCOVERY_LINK}"
            )),
            "{outcome}"
        );
    }
    let rerun = install_skills(&repository, false)
        .expect("the fresh result should be an ordinary unchanged installation");
    assert!(
        rerun.to_string().contains("nothing needed writing"),
        "{rerun}"
    );
}

#[cfg(windows)]
#[test]
fn windows_inspects_and_reports_a_non_link_at_the_discovery_path() {
    let fixture = windows_temp_root("skill-install-windows-link-obstruction");
    let repository = support::git_repository(&fixture, "target");
    let links_supported = windows_supports_directory_links(&fixture);
    write_matching_shipped_files(&repository);
    install_skills(&repository, false).expect("the fixture should install once");
    let link = repository.join(DISCOVERY_LINK);
    if link.exists() {
        fs::remove_dir(&link).expect("the discovery link should be removable");
    }
    fs::create_dir_all(
        link.parent()
            .expect("the discovery link should have a parent directory"),
    )
    .expect("the discovery parent should be creatable");
    fs::write(&link, b"not a discovery link\n")
        .expect("a regular file should be able to obstruct discovery");

    let Err(failure) = install_skills(&repository, false) else {
        panic!("a Windows non-link obstruction must be reported before replacement");
    };
    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert!(failure.to_string().contains(DISCOVERY_LINK), "{failure}");

    let outcome = install_skills(&repository, true)
        .expect("force should succeed even when Windows cannot create links");
    if links_supported {
        assert_eq!(
            fs::read_link(&link).expect("force should restore the discovery link"),
            PathBuf::from(DISCOVERY_TARGET)
        );
        assert!(outcome.to_string().contains(DISCOVERY_LINK), "{outcome}");
    } else {
        assert!(
            outcome.to_string().contains(&format!(
                "discovery links not created: symbolic links are not supported on this platform\n- {DISCOVERY_LINK}"
            )),
            "{outcome}"
        );
        assert!(
            fs::symlink_metadata(&link).is_err(),
            "force should remove an obsolete discovery obstruction when links are unavailable"
        );
    }
    let rerun = install_skills(&repository, false)
        .expect("the forced result should be an ordinary unchanged installation");
    assert!(
        rerun.to_string().contains("nothing needed writing"),
        "{rerun}"
    );
}

#[cfg(windows)]
#[test]
fn windows_error_4390_regular_file_is_a_replaceable_conflict() {
    let fixture = windows_temp_root("skill-install-windows-error-4390");
    let repository = support::git_repository(&fixture, "target");
    write_matching_shipped_files(&repository);
    install_skills(&repository, false).expect("the fixture should install once");
    let link = repository.join(DISCOVERY_LINK);
    if link.exists() {
        fs::remove_dir(&link).expect("the discovery link should be removable");
    }
    fs::write(&link, b"not a reparse point\n")
        .expect("a regular file should be able to occupy the discovery path");

    let Err(failure) = install_skills(&repository, false) else {
        panic!("a Windows regular-file obstruction must be a replaceable conflict");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: installed reuse-evidence skill paths differ from the package this binary ships:\n- {DISCOVERY_LINK}\nresolution: rerun with `install-skills --root <ROOT> --force` to replace every differing path"
        )
    );
    install_skills(&repository, true)
        .expect("force should handle the regular-file obstruction after classifying it");
}

#[cfg(windows)]
#[test]
fn windows_git_symlink_placeholder_is_a_self_install_no_op() {
    let fixture = windows_temp_root("skill-install-windows-git-placeholder");
    let repository = support::git_repository(&fixture, "target");
    support::git_index_with_symlink(&repository, DISCOVERY_LINK);
    write_matching_shipped_files(&repository);
    let placeholder = repository.join(DISCOVERY_LINK);
    fs::create_dir_all(
        placeholder
            .parent()
            .expect("the discovery placeholder should have a parent"),
    )
    .expect("the discovery parent should be creatable");
    fs::write(&placeholder, DISCOVERY_TARGET.as_bytes())
        .expect("the Git symlink placeholder should be writable");
    let before = support::snapshot(&repository);

    let outcome = install_skills(&repository, false)
        .expect("an exact Git symlink placeholder should preserve self-install as a no-op");

    assert_eq!(
        outcome.to_string(),
        "reuse-evidence skill packages already installed\nnothing needed writing\n"
    );
    assert_eq!(support::snapshot(&repository), before);
}

#[cfg(windows)]
#[test]
fn windows_untracked_exact_target_regular_file_is_a_conflict() {
    let fixture = windows_temp_root("skill-install-windows-untracked-exact-target");
    let repository = support::git_repository(&fixture, "target");
    write_matching_shipped_files(&repository);
    let obstruction = repository.join(DISCOVERY_LINK);
    fs::create_dir_all(
        obstruction
            .parent()
            .expect("the discovery obstruction should have a parent"),
    )
    .expect("the discovery parent should be creatable");
    fs::write(&obstruction, DISCOVERY_TARGET.as_bytes())
        .expect("the exact-target regular-file obstruction should be writable");
    let before = support::snapshot(&repository);

    let Err(failure) = install_skills(&repository, false) else {
        panic!("an untracked regular file must not masquerade as a Git symlink placeholder");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: installed reuse-evidence skill paths differ from the package this binary ships:\n- {DISCOVERY_LINK}\nresolution: rerun with `install-skills --root <ROOT> --force` to replace every differing path"
        )
    );
    assert_eq!(
        support::snapshot(&repository),
        before,
        "the non-force refusal must not change the obstruction"
    );
}

fn assert_unchanged_install_is_a_no_op(case: &str, repository: &Path) {
    let before = support::snapshot(repository);
    #[cfg(unix)]
    let links_before = shipped_discovery_links()
        .into_iter()
        .map(|(link, _)| {
            let target = fs::read_link(repository.join(&link)).unwrap_or_else(|error| {
                panic!("{case}: correct discovery link {link} should be readable: {error}")
            });
            (link, target)
        })
        .collect::<Vec<_>>();
    #[cfg(windows)]
    let links_before = shipped_discovery_links()
        .into_iter()
        .map(|(link, _)| {
            let target = fs::read_link(repository.join(&link)).ok();
            (link, target)
        })
        .collect::<Vec<_>>();

    let outcome = install_skills(repository, false)
        .unwrap_or_else(|error| panic!("{case}: an unchanged install should succeed: {error}"));

    #[cfg(unix)]
    let expected =
        String::from("reuse-evidence skill packages already installed\nnothing needed writing\n");
    #[cfg(windows)]
    let unsupported_links = shipped_discovery_links()
        .into_iter()
        .filter_map(|(link, _)| {
            fs::symlink_metadata(repository.join(&link))
                .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
                .then_some(link)
        })
        .collect::<Vec<_>>();
    #[cfg(windows)]
    let expected = if unsupported_links.is_empty() {
        String::from("reuse-evidence skill packages already installed\nnothing needed writing\n")
    } else {
        let mut expected = String::from(
            "reuse-evidence skill packages already installed\nnothing needed writing\ndiscovery links not created: symbolic links are not supported on this platform\n",
        );
        for link in unsupported_links {
            writeln!(&mut expected, "- {link}")
                .expect("writing expected receipt text to a String cannot fail");
        }
        expected
    };
    #[cfg(not(any(unix, windows)))]
    let expected = {
        let mut expected = String::from(
            "reuse-evidence skill packages already installed\nnothing needed writing\ndiscovery links not created: symbolic links are not supported on this platform\n",
        );
        for (link, _) in shipped_discovery_links() {
            writeln!(&mut expected, "- {link}")
                .expect("writing expected receipt text to a String cannot fail");
        }
        expected
    };
    assert_eq!(outcome.to_string(), expected, "{case}");
    assert_eq!(
        support::snapshot(repository),
        before,
        "{case}: no-op installation must preserve every file byte"
    );
    #[cfg(unix)]
    for (link, target_before) in links_before {
        assert_eq!(
            fs::read_link(repository.join(&link)).unwrap_or_else(|error| panic!(
                "{case}: discovery link {link} should remain readable: {error}"
            )),
            target_before,
            "{case}: every correct discovery link must be left alone"
        );
    }
    #[cfg(windows)]
    for (link, target_before) in links_before {
        if let Some(target_before) = target_before {
            assert_eq!(
                fs::read_link(repository.join(&link)).unwrap_or_else(|error| panic!(
                    "{case}: discovery link {link} should remain readable: {error}"
                )),
                target_before,
                "{case}: every correct discovery link must be left alone"
            );
        }
    }
}

#[cfg(unix)]
fn assert_fresh_discovery_link_and_extend_receipt(repository: &Path, expected: &mut String) {
    writeln!(expected, "discovery links created:")
        .expect("writing expected receipt text to a String cannot fail");
    for (link, target) in shipped_discovery_links() {
        assert_eq!(
            fs::read_link(repository.join(&link))
                .expect("each installed package should have a discovery link"),
            PathBuf::from(&target),
            "each discovery target should be relative to its own directory"
        );
        writeln!(expected, "- {link} -> {target}")
            .expect("writing expected receipt text to a String cannot fail");
    }
}

#[cfg(windows)]
fn assert_fresh_discovery_link_and_extend_receipt(repository: &Path, expected: &mut String) {
    let links_supported = fs::read_link(repository.join(DISCOVERY_LINK)).is_ok();
    if links_supported {
        writeln!(expected, "discovery links created:")
            .expect("writing expected receipt text to a String cannot fail");
    } else {
        writeln!(
            expected,
            "discovery links not created: symbolic links are not supported on this platform"
        )
        .expect("writing expected receipt text to a String cannot fail");
    }
    for (link, target) in shipped_discovery_links() {
        if links_supported {
            assert_eq!(
                fs::read_link(repository.join(&link))
                    .expect("each installed package should have a discovery link"),
                PathBuf::from(&target)
            );
            writeln!(expected, "- {link} -> {target}")
                .expect("writing expected receipt text to a String cannot fail");
        } else {
            assert!(
                !repository.join(&link).exists(),
                "a Windows target without symlink support should use the real files alone"
            );
            writeln!(expected, "- {link}")
                .expect("writing expected receipt text to a String cannot fail");
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn assert_fresh_discovery_link_and_extend_receipt(repository: &Path, expected: &mut String) {
    writeln!(
        expected,
        "discovery links not created: symbolic links are not supported on this platform"
    )
    .expect("writing expected receipt text to a String cannot fail");
    for (link, _) in shipped_discovery_links() {
        assert!(
            !repository.join(&link).exists(),
            "a platform without ordinary directory symlinks should use the real files alone"
        );
        writeln!(expected, "- {link}")
            .expect("writing expected receipt text to a String cannot fail");
    }
}

#[cfg(unix)]
fn create_directory_symlink(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("fixture symlink should be creatable");
}

#[cfg(windows)]
fn create_directory_symlink(target: &Path, link: &Path) {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => {}
        Err(error)
            if error.kind() == std::io::ErrorKind::Unsupported
                || error.raw_os_error() == Some(1314) => {}
        Err(error) => panic!("fixture symlink should be creatable or unsupported: {error}"),
    }
}

#[cfg(not(any(unix, windows)))]
fn create_directory_symlink(_target: &Path, _link: &Path) {}

#[cfg(windows)]
fn windows_supports_directory_links(fixture: &Path) -> bool {
    let target = fixture.join("windows-link-probe-target");
    let link = fixture.join("windows-link-probe");
    fs::create_dir(&target).expect("the Windows link probe target should be creatable");
    match std::os::windows::fs::symlink_dir(&target, &link) {
        Ok(()) => {
            if fs::read_link(&link).is_ok() {
                fs::remove_dir(&link)
                    .or_else(|_| fs::remove_file(&link))
                    .expect("the Windows probe link should be removable");
                true
            } else {
                false
            }
        }
        Err(error)
            if error.kind() == std::io::ErrorKind::Unsupported
                || error.raw_os_error() == Some(1314) =>
        {
            false
        }
        Err(error) => panic!("the Windows link probe failed unexpectedly: {error}"),
    }
}

#[cfg(windows)]
fn windows_temp_root(name: &str) -> TempRoot {
    let current = std::env::current_dir().expect("the Windows test cwd should be readable");
    TempRoot::beneath(&current, name)
}

#[cfg(windows)]
fn write_matching_shipped_files(repository: &Path) {
    for (relative, bytes) in shipped_files_at(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        let path = repository.join(relative);
        fs::create_dir_all(path.parent().expect("a shipped file should have a parent"))
            .expect("the shipped file parent should be creatable");
        fs::write(path, bytes).expect("the shipped fixture bytes should be writable");
    }
}

fn shipped_discovery_links() -> Vec<(String, String)> {
    let mut package_names = Vec::<String>::new();
    for (relative_path, _) in shipped_files_at(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        let package_and_file = relative_path
            .strip_prefix(".claude/skills/")
            .expect("a shipped source file must live under .claude/skills");
        let package_name = package_and_file
            .split_once('/')
            .map(|(name, _)| name)
            .expect("a shipped source file must live inside a package");
        if !package_names.iter().any(|name| name == package_name) {
            package_names.push(package_name.to_owned());
        }
    }
    package_names
        .into_iter()
        .map(|package_name| {
            (
                format!(".agents/skills/{package_name}"),
                format!("../../.claude/skills/{package_name}"),
            )
        })
        .collect()
}

fn shipped_files_at(root: &Path) -> Vec<(String, Vec<u8>)> {
    let skills_root = root.join(".claude/skills");
    let mut packages = fs::read_dir(&skills_root)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", skills_root.display()))
        .map(|entry| entry.expect("a skill package entry should be readable"))
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("reuse-evidence-")
        })
        .collect::<Vec<_>>();
    packages.sort_by_key(fs::DirEntry::file_name);
    let mut files = Vec::new();
    for package in packages {
        collect_shipped_files(root, &package.path(), &mut files);
    }
    files
}

fn collect_shipped_files(root: &Path, path: &Path, files: &mut Vec<(String, Vec<u8>)>) {
    let metadata = fs::symlink_metadata(path)
        .unwrap_or_else(|error| panic!("{} should be inspectable: {error}", path.display()));
    assert!(
        !metadata.file_type().is_symlink(),
        "shipped package source paths must be real files and directories: {}",
        path.display()
    );
    if metadata.is_file() {
        let relative = path
            .strip_prefix(root)
            .expect("a shipped path should remain beneath its root")
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        files.push((
            relative,
            fs::read(path)
                .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display())),
        ));
        return;
    }
    assert!(
        metadata.is_dir(),
        "{} should be a file or directory",
        path.display()
    );
    let mut entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
        .map(|entry| entry.expect("a shipped package entry should be readable"))
        .collect::<Vec<_>>();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        collect_shipped_files(root, &entry.path(), files);
    }
}
