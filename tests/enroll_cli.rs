use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "reuse-evidence-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("fixture root should be creatable");
        Self { root }
    }

    fn repository(&self, name: &str) -> PathBuf {
        let repository = self.root.join(name);
        fs::create_dir_all(repository.join(".git"))
            .expect("repository fixture should be creatable");
        repository
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_in(repository: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(arguments)
        .current_dir(repository)
        .output()
        .expect("compiled reuse-evidence binary should run")
}

fn files_beneath(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(directory).expect("fixture directory should be readable") {
            let entry = entry.expect("fixture entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("fixture entry should be beneath its root")
                    .to_path_buf();
                files.insert(
                    relative,
                    fs::read(path).expect("fixture file should be readable"),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn fresh_enrollment_writes_exact_marker_in_npm_repository() {
    let fixture = Fixture::new("fresh-enrollment");
    let repository = fixture.repository("payments-app");
    fs::write(
        repository.join("package.json"),
        b"{\"name\":\"@portfolio/payments-app\",\"private\":true,\"workspaces\":[\"packages/*\"]}\n",
    )
    .expect("npm workspace manifest should be writable");
    let before = files_beneath(&repository);

    let output = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "npm-products",
            "--visibility",
            "private",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");

    let marker_path = repository.join("reuse-evidence.toml");
    let marker = fs::read_to_string(&marker_path).expect("enrollment should write its marker");
    let parsed = marker
        .parse::<toml::Table>()
        .expect("the marker should be valid TOML");
    assert_eq!(
        parsed.len(),
        4,
        "the marker must contain exactly four fields"
    );
    assert_eq!(parsed["schema_version"].as_integer(), Some(1));
    assert_eq!(parsed["ecosystem_id"].as_str(), Some("npm-products"));
    assert_eq!(parsed["visibility"].as_str(), Some("private"));

    let repository_id = parsed["repository_id"]
        .as_str()
        .expect("repository identity should be a string");
    let uuid = uuid::Uuid::parse_str(repository_id).expect("repository identity should be opaque");
    assert_eq!(uuid.get_version_num(), 4);
    assert!(!repository_id.contains("payments-app"));
    assert!(!repository_id.contains("portfolio"));
    assert!(!repository_id.contains(repository.to_string_lossy().as_ref()));

    assert_eq!(
        marker,
        format!(
            "schema_version = 1\nrepository_id = \"{repository_id}\"\necosystem_id = \"npm-products\"\nvisibility = \"private\"\n"
        )
    );
    assert!(!repository.join("Cargo.toml").exists());

    let mut after = files_beneath(&repository);
    assert_eq!(
        after.remove(Path::new("reuse-evidence.toml")),
        Some(marker.into_bytes())
    );
    assert_eq!(after, before, "enrollment must add only its marker");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("reuse-evidence.toml"));
    assert!(stdout.contains(repository_id));
    assert!(stdout.contains("npm-products"));
    assert!(stdout.contains("private"));
}

#[test]
fn missing_ecosystem_refuses_actionably_without_writes() {
    let fixture = Fixture::new("missing-ecosystem");
    let repository = fixture.repository("consumer");
    fs::write(repository.join("notes.txt"), b"preserve these bytes\n")
        .expect("fixture content should be writable");
    let before = files_beneath(&repository);

    let output = run_in(&repository, &["enroll", "--visibility", "public"]);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: missing required `--ecosystem-id`\nresolution: rerun with `--ecosystem-id <IDENTITY>`\n"
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "refusal must leave the target tree byte-identical"
    );
}

#[test]
fn enrollment_from_nested_directory_writes_at_repository_root() {
    let fixture = Fixture::new("nested-enrollment");
    let repository = fixture.repository("workspace");
    let nested_package = repository.join("packages").join("web-client");
    fs::create_dir_all(&nested_package).expect("nested package should be creatable");

    let output = run_in(
        &nested_package,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "public",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert!(repository.join("reuse-evidence.toml").is_file());
    assert!(!nested_package.join("reuse-evidence.toml").exists());
}

#[test]
fn cli_argument_refusals_are_write_free_and_actionable() {
    let fixture = Fixture::new("cli-refusals");
    let repository = fixture.repository("consumer");
    fs::write(repository.join("preserved.bin"), [0, 1, 2, 255])
        .expect("fixture bytes should be writable");
    let before = files_beneath(&repository);
    let cases: &[(&str, &[&str])] = &[
        (
            "missing visibility",
            &["enroll", "--ecosystem-id", "products"],
        ),
        (
            "invalid visibility",
            &[
                "enroll",
                "--ecosystem-id",
                "products",
                "--visibility",
                "secret",
            ],
        ),
        ("missing command", &[]),
        ("unknown command", &["unknown"]),
    ];

    for (case, arguments) in cases {
        let output = run_in(&repository, arguments);
        assert_eq!(output.status.code(), Some(3), "{case}: {output:?}");
        assert!(output.stdout.is_empty(), "{case}: {output:?}");
        let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        assert!(stderr.starts_with("refusal:"), "{case}: {stderr}");
        assert!(
            stderr.contains("\nresolution:"),
            "{case}: refusal should say how to resolve it: {stderr}"
        );
        assert_eq!(
            files_beneath(&repository),
            before,
            "{case}: refusal must leave the target tree byte-identical"
        );
    }
}

#[test]
fn marker_creation_failure_has_distinct_unsafe_failure_status() {
    let fixture = Fixture::new("unsafe-failure");
    let repository = fixture.repository("consumer");
    fs::create_dir(repository.join("reuse-evidence.toml"))
        .expect("a conflicting marker directory should be creatable");
    fs::write(
        repository.join("reuse-evidence.toml").join("preserved.txt"),
        b"unchanged\n",
    )
    .expect("conflict fixture should be writable");
    let before = files_beneath(&repository);

    let output = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );

    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.starts_with("unsafe failure:"), "{stderr}");
    assert_eq!(files_beneath(&repository), before);
}
