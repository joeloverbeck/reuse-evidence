//! In-process coverage for installing this project's own skill packages.
//!
//! GitHub #34 and ADR 0016 assign installer behavior to the public library
//! interface. The process boundary receives one separate test for argv
//! dispatch, exit status, and stream routing.

mod support;

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use reuse_evidence::{ExitMeaning, install_skills};
use support::TempRoot;

const SHIPPED_FILES: &[(&str, &[u8])] = &[
    (
        ".claude/skills/reuse-evidence-capture/SKILL.md",
        include_bytes!("../.claude/skills/reuse-evidence-capture/SKILL.md"),
    ),
    (
        ".claude/skills/reuse-evidence-capture/agents/openai.yaml",
        include_bytes!("../.claude/skills/reuse-evidence-capture/agents/openai.yaml"),
    ),
    (
        ".claude/skills/reuse-evidence-capture/references/prepared-proposals.md",
        include_bytes!("../.claude/skills/reuse-evidence-capture/references/prepared-proposals.md"),
    ),
];

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
fn a_fresh_install_writes_the_shipped_package_and_relative_discovery_link_only() {
    let fixture = TempRoot::new("skill-install-fresh");
    let repository = support::git_repository(&fixture, "target");
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

    for (relative_path, shipped_bytes) in SHIPPED_FILES {
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
    let mut expected = String::from("installed reuse-evidence skill packages\nwritten:\n");
    for (relative_path, _) in SHIPPED_FILES {
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
    let first_conflict = SHIPPED_FILES[0].0;
    let second_conflict = SHIPPED_FILES[2].0;
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
    let modified = SHIPPED_FILES[0].0;
    let missing = SHIPPED_FILES[2].0;
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
    for (relative_path, shipped_bytes) in SHIPPED_FILES {
        assert_eq!(
            fs::read(repository.join(relative_path))
                .expect("every installed file should be readable after force"),
            *shipped_bytes,
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
    let replaceable = SHIPPED_FILES[0].0;
    let unreplaceable = SHIPPED_FILES[2].0;
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
    let replaceable = SHIPPED_FILES[0].0;
    let unreplaceable = SHIPPED_FILES[2].0;
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

fn assert_unchanged_install_is_a_no_op(case: &str, repository: &Path) {
    let before = support::snapshot(repository);
    #[cfg(unix)]
    let link_before = fs::read_link(repository.join(DISCOVERY_LINK)).unwrap_or_else(|error| {
        panic!("{case}: correct discovery link should be readable: {error}")
    });
    #[cfg(windows)]
    let link_before = fs::read_link(repository.join(DISCOVERY_LINK)).ok();

    let outcome = install_skills(repository, false)
        .unwrap_or_else(|error| panic!("{case}: an unchanged install should succeed: {error}"));

    #[cfg(unix)]
    let expected =
        String::from("reuse-evidence skill packages already installed\nnothing needed writing\n");
    #[cfg(windows)]
    let expected = if link_before.is_some() {
        String::from("reuse-evidence skill packages already installed\nnothing needed writing\n")
    } else {
        format!(
            "reuse-evidence skill packages already installed\nnothing needed writing\ndiscovery links not created: symbolic links are not supported on this platform\n- {DISCOVERY_LINK}\n"
        )
    };
    #[cfg(not(any(unix, windows)))]
    let expected = String::from(
        "reuse-evidence skill packages already installed\nnothing needed writing\ndiscovery links not created: symbolic links are not supported on this platform\n- .agents/skills/reuse-evidence-capture\n",
    );
    assert_eq!(outcome.to_string(), expected, "{case}");
    assert_eq!(
        support::snapshot(repository),
        before,
        "{case}: no-op installation must preserve every file byte"
    );
    #[cfg(unix)]
    assert_eq!(
        fs::read_link(repository.join(DISCOVERY_LINK)).unwrap_or_else(|error| panic!(
            "{case}: discovery link should remain readable: {error}"
        )),
        link_before,
        "{case}: a correct discovery link must be left alone"
    );
    #[cfg(windows)]
    if let Some(link_before) = link_before {
        assert_eq!(
            fs::read_link(repository.join(DISCOVERY_LINK)).unwrap_or_else(|error| panic!(
                "{case}: discovery link should remain readable: {error}"
            )),
            link_before,
            "{case}: a correct discovery link must be left alone"
        );
    }
}

#[cfg(unix)]
fn assert_fresh_discovery_link_and_extend_receipt(repository: &Path, expected: &mut String) {
    assert_eq!(
        fs::read_link(repository.join(DISCOVERY_LINK))
            .expect("the installed package should have a discovery link"),
        PathBuf::from(DISCOVERY_TARGET),
        "the discovery target should be relative to its own directory"
    );
    writeln!(
        expected,
        "discovery links created:\n- {DISCOVERY_LINK} -> {DISCOVERY_TARGET}"
    )
    .expect("writing expected receipt text to a String cannot fail");
}

#[cfg(windows)]
fn assert_fresh_discovery_link_and_extend_receipt(repository: &Path, expected: &mut String) {
    match fs::read_link(repository.join(DISCOVERY_LINK)) {
        Ok(target) => {
            assert_eq!(target, PathBuf::from(DISCOVERY_TARGET));
            writeln!(
                expected,
                "discovery links created:\n- {DISCOVERY_LINK} -> {DISCOVERY_TARGET}"
            )
            .expect("writing expected receipt text to a String cannot fail");
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => writeln!(
            expected,
            "discovery links not created: symbolic links are not supported on this platform\n- {DISCOVERY_LINK}"
        )
        .expect("writing expected receipt text to a String cannot fail"),
        Err(error) => panic!("the Windows discovery path should be a link or absent: {error}"),
    }
}

#[cfg(not(any(unix, windows)))]
fn assert_fresh_discovery_link_and_extend_receipt(repository: &Path, expected: &mut String) {
    assert!(
        !repository.join(DISCOVERY_LINK).exists(),
        "a platform without ordinary directory symlinks should use the real files alone"
    );
    writeln!(
        expected,
        "discovery links not created: symbolic links are not supported on this platform\n- {DISCOVERY_LINK}"
    )
    .expect("writing expected receipt text to a String cannot fail");
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
    for (relative, bytes) in SHIPPED_FILES {
        let path = repository.join(relative);
        fs::create_dir_all(path.parent().expect("a shipped file should have a parent"))
            .expect("the shipped file parent should be creatable");
        fs::write(path, bytes).expect("the shipped fixture bytes should be writable");
    }
}
