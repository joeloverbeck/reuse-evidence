mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use support::TempRoot;

const INSTALLED_FILES: &[&str] = &[
    ".claude/skills/method-gap-research-status/SKILL.md",
    ".claude/skills/method-gap-research-status/agents/openai.yaml",
    ".claude/skills/method-gap-research-status/references/lineage-reconstruction.md",
    ".claude/skills/method-gap-research-status/references/selection-rules.md",
    ".claude/skills/skill-evidence-capture/SKILL.md",
    ".claude/skills/skill-evidence-capture/agents/openai.yaml",
    ".claude/skills/skill-evolution-status/SKILL.md",
    ".claude/skills/skill-evolution-status/agents/openai.yaml",
    ".claude/skills/skill-evolution/SKILL.md",
    ".claude/skills/skill-evolution/agents/openai.yaml",
    ".claude/skills/skill-evolution/references/authorized-review.md",
    "schemas/skill-evidence/event.v1.schema.json",
    "schemas/skill-evidence/gate-status.v1.schema.json",
];

const INSTALLED_PACKAGES: &[&str] = &[
    "method-gap-research-status",
    "skill-evidence-capture",
    "skill-evolution",
    "skill-evolution-status",
];

struct Fixture {
    root: TempRoot,
}

impl Fixture {
    fn new(name: &str) -> Self {
        Self {
            root: TempRoot::new(name),
        }
    }
}

fn run_in(working_directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(arguments)
        .current_dir(working_directory)
        .output()
        .expect("compiled reuse-evidence binary should run")
}

#[derive(Debug, Eq, PartialEq)]
enum TreeEntry {
    Directory,
    File(Vec<u8>),
    Symlink(PathBuf),
}

fn tree_beneath(root: &Path) -> std::collections::BTreeMap<PathBuf, TreeEntry> {
    fn visit(
        root: &Path,
        directory: &Path,
        entries: &mut std::collections::BTreeMap<PathBuf, TreeEntry>,
    ) {
        for entry in fs::read_dir(directory).expect("fixture directory should be readable") {
            let entry = entry.expect("fixture entry should be readable");
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .expect("fixture entry should be beneath its root")
                .to_path_buf();
            let metadata =
                fs::symlink_metadata(&path).expect("fixture metadata should be readable");
            if metadata.file_type().is_symlink() {
                entries.insert(
                    relative,
                    TreeEntry::Symlink(
                        fs::read_link(path).expect("fixture symlink target should be readable"),
                    ),
                );
            } else if metadata.is_dir() {
                entries.insert(relative, TreeEntry::Directory);
                visit(root, &path, entries);
            } else {
                entries.insert(
                    relative,
                    TreeEntry::File(fs::read(path).expect("fixture file should be readable")),
                );
            }
        }
    }

    let mut entries = std::collections::BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

#[test]
fn skills_install_uses_reuse_evidence_host() {
    let fixture = Fixture::new("skills-install-host");
    let root = fixture.root.to_string_lossy();

    let output = run_in(
        &fixture.root,
        &["skills", "evidence", "install", "--root", &root],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"schema_version\":1"), "{stdout}");

    for relative_path in INSTALLED_FILES {
        assert!(
            fixture.root.join(relative_path).is_file(),
            "installer should write {relative_path}"
        );
        assert!(
            stdout.contains(relative_path),
            "receipt should name {relative_path}"
        );
    }

    #[cfg(unix)]
    for package in INSTALLED_PACKAGES {
        assert_eq!(
            fs::read_link(fixture.root.join(".agents/skills").join(package))
                .expect("installed package should have a discovery link"),
            PathBuf::from("../../.claude/skills").join(package)
        );
    }

    let capture = fs::read_to_string(
        fixture
            .root
            .join(".claude/skills/skill-evidence-capture/SKILL.md"),
    )
    .expect("installed capture package should be readable");
    assert!(capture.contains("compiled `reuse-evidence skills evidence` commands"));
    assert!(capture.contains("cargo run --locked -p reuse-evidence -- skills evidence record"));

    let event_schema = fs::read_to_string(
        fixture
            .root
            .join("schemas/skill-evidence/event.v1.schema.json"),
    )
    .expect("installed event schema should be readable");
    assert!(event_schema.contains("\"$id\": \"reuse-evidence.skill-evidence.event.v1\""));
}

#[test]
fn skills_non_force_install_names_all_conflicts_without_rewriting_receipts() {
    let fixture = Fixture::new("skills-install-conflicts");
    let root = fixture.root.to_string_lossy();
    let install = run_in(
        &fixture.root,
        &["skills", "evidence", "install", "--root", &root],
    );
    assert_eq!(install.status.code(), Some(0), "{install:?}");

    let first_conflict = ".claude/skills/skill-evolution/SKILL.md";
    let second_conflict = "schemas/skill-evidence/event.v1.schema.json";
    fs::write(fixture.root.join(first_conflict), b"local evolution edit\n")
        .expect("first installed asset should be editable");
    fs::write(fixture.root.join(second_conflict), b"local schema edit\n")
        .expect("second installed asset should be editable");
    let receipt_path = fixture
        .root
        .join("reports/skill-evidence/example/events.jsonl");
    fs::create_dir_all(
        receipt_path
            .parent()
            .expect("receipt path should have a parent"),
    )
    .expect("receipt directory should be creatable");
    fs::write(&receipt_path, b"historical receipt bytes\n")
        .expect("historical receipt should be writable");
    let before = tree_beneath(&fixture.root);

    let output = run_in(
        &fixture.root,
        &["skills", "evidence", "install", "--root", &root],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(
        output.stdout.is_empty(),
        "refusal details belong on stderr: {output:?}"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains(first_conflict), "{stderr}");
    assert!(stderr.contains(second_conflict), "{stderr}");
    assert_eq!(
        tree_beneath(&fixture.root),
        before,
        "non-force refusal must preserve installed files, links, and historical receipts"
    );
}

#[test]
fn skills_unsafe_failure_maps_to_status_one_and_stderr() {
    let fixture = Fixture::new("skills-unsafe-failure");
    let obstructed_root = fixture.root.join("not-a-directory");
    fs::write(&obstructed_root, b"file blocks the installation root\n")
        .expect("obstructing file should be writable");
    let root = obstructed_root.to_string_lossy();

    let output = run_in(
        &fixture.root,
        &["skills", "evidence", "install", "--root", &root],
    );

    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        output.stdout.is_empty(),
        "unsafe failure details belong on stderr: {output:?}"
    );
    assert!(
        !output.stderr.is_empty(),
        "unsafe failure should be explained"
    );
}

#[test]
fn committed_operator_assets_match_published_install() {
    let fixture = Fixture::new("committed-skills-install");
    let root = fixture.root.to_string_lossy();
    let output = run_in(
        &fixture.root,
        &["skills", "evidence", "install", "--root", &root],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative_path in INSTALLED_FILES {
        assert_eq!(
            fs::read(repository_root.join(relative_path))
                .unwrap_or_else(|error| panic!("committed {relative_path} should exist: {error}")),
            fs::read(fixture.root.join(relative_path))
                .expect("freshly installed asset should be readable"),
            "committed {relative_path} should match the published installation"
        );
    }

    #[cfg(unix)]
    for package in INSTALLED_PACKAGES {
        assert_eq!(
            fs::read_link(repository_root.join(".agents/skills").join(package)).unwrap_or_else(
                |error| panic!("committed link for {package} should exist: {error}")
            ),
            fs::read_link(fixture.root.join(".agents/skills").join(package))
                .expect("freshly installed package should have a discovery link"),
            "committed discovery link for {package} should match the published installation"
        );
    }
}

#[test]
fn skills_directory_is_the_host_manifest_skill_evolution_package() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = repository_root.join(".claude/skills/skill-evolution");
    let evidence = repository_root.join("reports/skill-evidence/skill-evolution");
    let before = evidence.exists().then(|| tree_beneath(&evidence));
    let root = repository_root.to_string_lossy();
    let target = target.to_string_lossy();

    let output = run_in(
        repository_root,
        &[
            "skills",
            "evolution",
            "preflight",
            "--target",
            &target,
            "--root",
            &root,
            "--recorded-at",
            "2026-08-09T00:00:00Z",
            "--now-epoch-milliseconds",
            "1786233600000",
            "--session-id",
            "issue-5-self-target",
            "--lock-owner",
            "issue-5-self-target",
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(
        output.stdout.is_empty(),
        "self-target refusal belongs on stderr: {output:?}"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("refused_self_target"), "{stderr}");
    assert_eq!(
        evidence.exists().then(|| tree_beneath(&evidence)),
        before,
        "self-target refusal must not create or change lifecycle evidence"
    );
}

#[test]
fn project_skill_install_dispatches_success_and_refusal_through_the_terminal_contract() {
    let fixture = Fixture::new("project-skill-install-terminal");
    let root = fixture.root.to_string_lossy();

    let installed = run_in(&fixture.root, &["install-skills", "--root", &root]);

    assert_eq!(installed.status.code(), Some(0), "{installed:?}");
    assert!(installed.stderr.is_empty(), "{installed:?}");
    let stdout = String::from_utf8(installed.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.starts_with("installed reuse-evidence skill packages\nwritten:\n"),
        "{stdout}"
    );

    let conflict = ".claude/skills/reuse-evidence-capture/SKILL.md";
    fs::write(fixture.root.join(conflict), b"local capture edit\n")
        .expect("the installed capture package should be editable");
    let refused = run_in(&fixture.root, &["install-skills", "--root", &root]);

    assert_eq!(refused.status.code(), Some(3), "{refused:?}");
    assert!(refused.stdout.is_empty(), "{refused:?}");
    let stderr = String::from_utf8(refused.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.starts_with(
            "refusal: installed reuse-evidence skill paths differ from the package this binary ships:\n"
        ),
        "{stderr}"
    );
    assert!(stderr.contains(conflict), "{stderr}");
    assert!(
        stderr.ends_with(
            "resolution: rerun with `install-skills --root <ROOT> --force` to replace every differing path\n"
        ),
        "{stderr}"
    );
}
