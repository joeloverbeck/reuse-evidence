mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use support::{Fixture, files_beneath};

impl Fixture {
    fn new(name: &str) -> Self {
        Self::from_label(name)
    }

    fn outside_repository(name: &str) -> Self {
        [
            std::env::temp_dir(),
            PathBuf::from("/tmp"),
            PathBuf::from("/dev/shm"),
        ]
        .into_iter()
        .find(|candidate| {
            candidate.is_dir()
                && candidate
                    .ancestors()
                    .all(|ancestor| !ancestor.join(".git").exists())
        })
        .map(|candidate| Self::beneath(&candidate, name))
        .expect("the test environment should provide a temporary directory outside Git")
    }

    fn repository(&self, name: &str) -> PathBuf {
        self.git_repository(name)
    }
}

fn run_in(repository: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(arguments)
        .current_dir(repository)
        .output()
        .expect("compiled reuse-evidence binary should run")
}

fn run_in_with_configuration(
    repository: &Path,
    arguments: &[&str],
    configuration_home: &Path,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(arguments)
        .current_dir(repository)
        .env("XDG_CONFIG_HOME", configuration_home)
        .env("XDG_STATE_HOME", configuration_home.join("state"))
        .output()
        .expect("compiled reuse-evidence binary should run")
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

const CASE_STORAGE_CASE_ID: &str = "00000000-0000-4000-8000-000000000121";

fn assert_populated_case_storage_refuses_independently_of_visibility_and_configuration(
    fixture: &Fixture,
) {
    let populated = fixture.repository("populated-steward");
    let case_directory = populated
        .join("reuse-evidence/cases")
        .join(CASE_STORAGE_CASE_ID);
    fs::create_dir_all(&case_directory).expect("case directory should be creatable");
    fs::write(
        case_directory.join(support::CASE_OPENED_AT_1),
        b"preserved recorded event bytes\n",
    )
    .expect("recorded event fixture should be writable");
    let configuration_home = fixture.root.join("configured");
    fs::create_dir_all(configuration_home.join("reuse-evidence"))
        .expect("portfolio configuration directory should be creatable");
    fs::write(
        configuration_home.join("reuse-evidence/config.toml"),
        format!("portfolio_roots = [\"{}\"]\n", fixture.root.display()),
    )
    .expect("portfolio configuration should be writable");
    let before_populated_refusals = files_beneath(&populated);
    let expected_refusal = format!(
        "refusal: steward-local case directory `{}` is not empty in an unenrolled repository; fresh enrollment would mint a new repository identity and orphan every case recorded under the previous identity\nresolution: restore the committed `reuse-evidence.toml` marker that records those cases' steward identity, or set the case directory aside if this repository should not steward them\n",
        populated.join("reuse-evidence/cases").display()
    );
    let public = run_in(
        &populated,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "public",
        ],
    );
    let private = run_in(
        &populated,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    let configured = run_in_with_configuration(
        &populated,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "public",
        ],
        &configuration_home,
    );
    for refusal in [&public, &private, &configured] {
        assert_eq!(refusal.status.code(), Some(3), "{refusal:?}");
        assert!(refusal.stdout.is_empty(), "{refusal:?}");
        assert_eq!(
            String::from_utf8(refusal.stderr.clone()).expect("stderr should be UTF-8"),
            expected_refusal
        );
    }
    assert_eq!(public.stderr, private.stderr);
    assert_eq!(public.stderr, configured.stderr);
    assert!(!populated.join("reuse-evidence.toml").exists());
    assert_eq!(
        files_beneath(&populated),
        before_populated_refusals,
        "every populated-storage refusal must preserve every repository byte"
    );
}

fn assert_non_case_entry_refuses_without_classification(fixture: &Fixture) {
    let non_case_entry = fixture.repository("non-case-entry");
    fs::create_dir_all(non_case_entry.join("reuse-evidence/cases"))
        .expect("case storage should be creatable");
    fs::write(
        non_case_entry.join("reuse-evidence/cases/README.txt"),
        b"not a case directory\n",
    )
    .expect("non-case entry should be writable");
    let before_non_case_refusal = files_beneath(&non_case_entry);
    let non_case_refusal = run_in(
        &non_case_entry,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    assert_eq!(
        non_case_refusal.status.code(),
        Some(3),
        "{non_case_refusal:?}"
    );
    assert!(non_case_refusal.stdout.is_empty(), "{non_case_refusal:?}");
    assert_eq!(
        files_beneath(&non_case_entry),
        before_non_case_refusal,
        "the guard must classify no entry before refusing"
    );
}

fn assert_empty_case_storage_enrolls(fixture: &Fixture) {
    let empty = fixture.repository("empty-case-storage");
    let empty_case_storage = empty.join("reuse-evidence/cases");
    fs::create_dir_all(&empty_case_storage).expect("empty case storage should be creatable");
    let empty_enrollment = run_in(
        &empty,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    assert_eq!(
        empty_enrollment.status.code(),
        Some(0),
        "{empty_enrollment:?}"
    );
    assert!(empty_enrollment.stderr.is_empty(), "{empty_enrollment:?}");
    assert!(empty.join("reuse-evidence.toml").is_file());
    assert_eq!(
        fs::read_dir(&empty_case_storage)
            .expect("empty case storage should remain readable")
            .count(),
        0
    );
}

#[cfg(unix)]
fn assert_unreadable_case_storage_refuses_without_writes(fixture: &Fixture) {
    use std::os::unix::fs::PermissionsExt;

    let unreadable = fixture.repository("unreadable-case-storage");
    let unreadable_case_storage = unreadable.join("reuse-evidence/cases");
    fs::create_dir_all(&unreadable_case_storage)
        .expect("unreadable case storage should be creatable");
    let before_unreadable_refusal = files_beneath(&unreadable);
    let original_permissions = fs::metadata(&unreadable_case_storage)
        .expect("case storage metadata should be readable")
        .permissions();
    fs::set_permissions(&unreadable_case_storage, fs::Permissions::from_mode(0o000))
        .expect("case storage should become unreadable");
    let unreadable_refusal = run_in(
        &unreadable,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    fs::set_permissions(&unreadable_case_storage, original_permissions)
        .expect("case storage permissions should be restorable");
    assert_eq!(
        unreadable_refusal.status.code(),
        Some(3),
        "{unreadable_refusal:?}"
    );
    assert!(
        unreadable_refusal.stdout.is_empty(),
        "{unreadable_refusal:?}"
    );
    assert!(!unreadable.join("reuse-evidence.toml").exists());
    assert_eq!(
        files_beneath(&unreadable),
        before_unreadable_refusal,
        "unreadable case storage must refuse without changing repository bytes"
    );
}

fn record_recoverable_case(fixture: &Fixture) -> (PathBuf, Vec<u8>) {
    let steward = fixture.repository("recoverable-steward");
    let first_consumer = fixture.repository("first-consumer");
    let second_consumer = fixture.repository("second-consumer");
    for (repository, visibility) in [
        (&steward, "private"),
        (&first_consumer, "public"),
        (&second_consumer, "public"),
    ] {
        let enrollment = run_in(
            repository,
            &[
                "enroll",
                "--ecosystem-id",
                "products",
                "--visibility",
                visibility,
            ],
        );
        assert_eq!(enrollment.status.code(), Some(0), "{enrollment:?}");
    }
    let steward_marker_path = steward.join("reuse-evidence.toml");
    let steward_marker = fs::read(&steward_marker_path).expect("steward marker should be readable");
    let first_marker = fs::read(first_consumer.join("reuse-evidence.toml"))
        .expect("first consumer marker should be readable");
    let second_marker = fs::read(second_consumer.join("reuse-evidence.toml"))
        .expect("second consumer marker should be readable");
    let proposal = fixture.root.join("case-proposal.toml");
    fs::write(
        &proposal,
        format!(
            "case_id = \"{CASE_STORAGE_CASE_ID}\"\nresponsibility = \"preserve steward identity continuity\"\n\n[[occurrences]]\nrepository_id = \"{}\"\nconsumer = \"first-consumer\"\nindependence = \"independent first lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\npath = \"src/first.rs\"\n\n[[occurrences]]\nrepository_id = \"{}\"\nconsumer = \"second-consumer\"\nindependence = \"independent second lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"2222222\"\npath = \"src/second.rs\"\n",
            marker_repository_id(&first_marker),
            marker_repository_id(&second_marker),
        ),
    )
    .expect("case proposal should be writable");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("proposal path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture root should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let marker_present_reenrollment = run_in(
        &steward,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
            "--expected-repository-id",
            marker_repository_id(&steward_marker),
        ],
    );
    assert_eq!(
        marker_present_reenrollment.status.code(),
        Some(0),
        "{marker_present_reenrollment:?}"
    );
    (steward, steward_marker)
}

fn assert_restoring_original_marker_recovers_stewarded_case(fixture: &Fixture) {
    let (steward, steward_marker) = record_recoverable_case(fixture);
    let steward_marker_path = steward.join("reuse-evidence.toml");
    fs::remove_file(&steward_marker_path).expect("marker loss should be reproducible");
    let before_recovery_refusal = files_beneath(&steward);
    let recovery_refusal = run_in(
        &steward,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "public",
        ],
    );
    assert_eq!(
        recovery_refusal.status.code(),
        Some(3),
        "{recovery_refusal:?}"
    );
    assert!(!steward_marker_path.exists());
    assert_eq!(files_beneath(&steward), before_recovery_refusal);
    fs::write(&steward_marker_path, &steward_marker)
        .expect("the committed steward marker should be restorable");
    let recovered = run_in(
        &steward,
        &[
            "case",
            "list",
            "--root",
            fixture.root.to_str().expect("fixture root should be UTF-8"),
        ],
    );
    assert_eq!(recovered.status.code(), Some(0), "{recovered:?}");
    assert!(recovered.stderr.is_empty(), "{recovered:?}");
    let recovered = String::from_utf8(recovered.stdout).expect("stdout should be UTF-8");
    assert!(
        recovered.contains(&format!("case_id: {CASE_STORAGE_CASE_ID}\n")),
        "{recovered}"
    );
    assert!(recovered.contains("state: watching\n"), "{recovered}");
}

#[test]
fn fresh_enrollment_guards_steward_case_storage_before_assigning_identity() {
    let fixture = Fixture::new("fresh-enrollment-case-storage-guard");
    assert_populated_case_storage_refuses_independently_of_visibility_and_configuration(&fixture);
    assert_non_case_entry_refuses_without_classification(&fixture);
    assert_empty_case_storage_enrolls(&fixture);
    #[cfg(unix)]
    assert_unreadable_case_storage_refuses_without_writes(&fixture);
    assert_restoring_original_marker_recovers_stewarded_case(&fixture);
}

#[test]
fn reenrollment_reports_existing_enrollment_and_preserves_marker_bytes() {
    let fixture = Fixture::new("reenrollment");
    let repository = fixture.repository("consumer");
    let first = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let marker_path = repository.join("reuse-evidence.toml");
    let marker_before = fs::read(&marker_path).expect("fresh enrollment should write its marker");
    let tree_before = files_beneath(&repository);

    let second = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );

    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        String::from_utf8(second.stdout).expect("stdout should be UTF-8"),
        format!(
            "existing enrollment\nmarker: {}\nrepository_id: {}\necosystem_id: products\nvisibility: private\n",
            marker_path.display(),
            marker_repository_id(&marker_before),
        )
    );
    assert_eq!(
        fs::read(&marker_path).expect("existing marker should remain readable"),
        marker_before,
        "re-enrollment must preserve marker bytes exactly"
    );
    assert_eq!(
        files_beneath(&repository),
        tree_before,
        "re-enrollment must leave the repository byte-identical"
    );
}

#[test]
fn enrollment_refuses_visibility_conflict_without_writes() {
    let fixture = Fixture::new("visibility-conflict");
    let repository = fixture.repository("consumer");
    let first = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let before = files_beneath(&repository);

    let output = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "public",
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: existing marker declares visibility `private`, but enrollment requested `public`\nresolution: use `set-visibility --visibility public` for a deliberate visibility change\n"
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "visibility conflict must leave the target tree byte-identical"
    );
}

#[test]
fn enrollment_refuses_ecosystem_conflict_without_writes() {
    let fixture = Fixture::new("ecosystem-conflict");
    let repository = fixture.repository("consumer");
    let first = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let before = files_beneath(&repository);

    let output = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "tools",
            "--visibility",
            "private",
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: existing marker declares ecosystem identity `products`, but enrollment requested `tools`\nresolution: rerun with `--ecosystem-id products`; changing ecosystem identity is not implemented\n"
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "ecosystem conflict must leave the target tree byte-identical"
    );
}

#[test]
fn enrollment_refuses_repository_identity_conflict_without_writes() {
    let fixture = Fixture::new("repository-identity-conflict");
    let repository = fixture.repository("consumer");
    let first = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let marker = fs::read(repository.join("reuse-evidence.toml"))
        .expect("fresh enrollment should write its marker");
    let existing_repository_id = marker_repository_id(&marker);
    let requested_repository_id = "00000000-0000-4000-8000-000000000003";
    assert_ne!(existing_repository_id, requested_repository_id);
    let before = files_beneath(&repository);

    let output = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
            "--expected-repository-id",
            requested_repository_id,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: existing marker declares repository identity `{existing_repository_id}`, but enrollment expected `{requested_repository_id}`\nresolution: rerun with `--expected-repository-id {existing_repository_id}` or omit the identity guard\n"
        )
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "repository identity conflict must leave the target tree byte-identical"
    );
}

#[test]
fn reenrollment_accepts_matching_repository_identity_guard_without_writes() {
    let fixture = Fixture::new("matching-repository-identity");
    let repository = fixture.repository("consumer");
    let first = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let marker_path = repository.join("reuse-evidence.toml");
    let marker = fs::read(&marker_path).expect("fresh enrollment should write its marker");
    let repository_id = marker_repository_id(&marker);
    let before = files_beneath(&repository);

    let output = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
            "--expected-repository-id",
            repository_id,
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.starts_with("existing enrollment\n"), "{stdout}");
    assert_eq!(
        files_beneath(&repository),
        before,
        "matching identity guard must preserve the repository"
    );
}

#[test]
fn fresh_enrollment_refuses_supplied_repository_identity_without_writes() {
    let fixture = Fixture::new("fresh-repository-identity");
    let repository = fixture.repository("consumer");
    let before = files_beneath(&repository);

    let output = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
            "--expected-repository-id",
            "00000000-0000-4000-8000-000000000003",
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: a repository identity cannot be assigned during fresh enrollment\nresolution: omit `--expected-repository-id`; fresh enrollment generates the opaque identity\n"
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "fresh identity refusal must leave the target tree byte-identical"
    );
}

#[test]
fn set_visibility_changes_only_declared_visibility_and_preserves_identity() {
    let fixture = Fixture::new("set-visibility");
    let repository = fixture.repository("consumer");
    let first = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let marker_path = repository.join("reuse-evidence.toml");
    let marker_before = fs::read(&marker_path).expect("fresh enrollment should write its marker");
    let repository_id = marker_repository_id(&marker_before).to_owned();
    let tree_before = files_beneath(&repository);

    let output = run_in(&repository, &["set-visibility", "--visibility", "public"]);

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        format!(
            "changed repository visibility\nmarker: {}\nrepository_id: {repository_id}\necosystem_id: products\nvisibility: public\n",
            marker_path.display(),
        )
    );

    let marker_after = fs::read_to_string(&marker_path).expect("changed marker should be readable");
    assert_eq!(
        marker_after,
        format!(
            "schema_version = 1\nrepository_id = \"{repository_id}\"\necosystem_id = \"products\"\nvisibility = \"public\"\n"
        )
    );
    let mut expected_tree = tree_before;
    expected_tree.insert(
        PathBuf::from("reuse-evidence.toml"),
        marker_after.into_bytes(),
    );
    assert_eq!(
        files_beneath(&repository),
        expected_tree,
        "visibility change must modify only the marker"
    );
}

#[test]
fn set_visibility_reports_unchanged_visibility_without_writes() {
    let fixture = Fixture::new("set-visibility-unchanged");
    let repository = fixture.repository("consumer");
    let first = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "public",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let marker_path = repository.join("reuse-evidence.toml");
    let marker = fs::read(&marker_path).expect("fresh enrollment should write its marker");
    let repository_id = marker_repository_id(&marker);
    let before = files_beneath(&repository);

    let output = run_in(&repository, &["set-visibility", "--visibility", "public"]);

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        format!(
            "repository visibility unchanged\nmarker: {}\nrepository_id: {repository_id}\necosystem_id: products\nvisibility: public\n",
            marker_path.display(),
        )
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "an unchanged visibility must preserve the repository"
    );
}

#[test]
fn set_visibility_refuses_unenrolled_repository_without_writes() {
    let fixture = Fixture::new("set-visibility-unenrolled");
    let repository = fixture.repository("consumer");
    fs::write(repository.join("preserved.txt"), b"unchanged\n")
        .expect("fixture content should be writable");
    let before = files_beneath(&repository);

    let output = run_in(&repository, &["set-visibility", "--visibility", "public"]);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.starts_with("refusal: repository is not enrolled"),
        "{stderr}"
    );
    assert!(
        stderr.ends_with("resolution: run `enroll` before changing visibility\n"),
        "{stderr}"
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "unenrolled visibility refusal must leave the target tree byte-identical"
    );
}

#[test]
fn enrollment_refuses_unsupported_marker_version_without_writes() {
    let fixture = Fixture::new("unsupported-marker-version");
    let repository = fixture.repository("consumer");
    fs::write(
        repository.join("reuse-evidence.toml"),
        b"schema_version = 2\nrepository_id = \"95e12811-9156-4469-86a5-c01e1b782cac\"\necosystem_id = \"products\"\nvisibility = \"private\"\nfuture_field = true\n",
    )
    .expect("unsupported marker should be writable");
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

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: marker schema version `2` is not supported\nresolution: use a reuse-evidence version that supports marker schema version `2`\n"
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "unsupported marker refusal must leave the target tree byte-identical"
    );
}

#[test]
fn enrollment_refuses_malformed_marker_without_writes() {
    let fixture = Fixture::new("malformed-marker");
    let repository = fixture.repository("consumer");
    fs::write(
        repository.join("reuse-evidence.toml"),
        b"schema_version = one\nrepository_id = [not valid marker TOML\n",
    )
    .expect("malformed marker should be writable");
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

    assert_invalid_marker_refusal(&repository, &before, output);
}

#[test]
fn enrollment_refuses_non_utf8_marker_without_writes() {
    let fixture = Fixture::new("non-utf8-marker");
    let repository = fixture.repository("consumer");
    fs::write(
        repository.join("reuse-evidence.toml"),
        b"schema_version = 1\nrepository_id = \"\xff\"\n",
    )
    .expect("non-UTF-8 marker should be writable");
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

    assert_invalid_marker_refusal(&repository, &before, output);
}

#[test]
fn enrollment_refuses_truncated_marker_without_writes() {
    let fixture = Fixture::new("truncated-marker");
    let repository = fixture.repository("consumer");
    fs::write(
        repository.join("reuse-evidence.toml"),
        b"schema_version = 1\nrepository_id = \"95e12811-9156-4469-86a5-c01e1b782cac\"\n",
    )
    .expect("truncated marker should be writable");
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

    assert_invalid_marker_refusal(&repository, &before, output);
}

fn assert_invalid_marker_refusal(
    repository: &Path,
    before: &BTreeMap<PathBuf, Vec<u8>>,
    output: Output,
) {
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.starts_with("refusal: existing marker `"), "{stderr}");
    assert!(stderr.contains("` is malformed:"), "{stderr}");
    assert!(
        stderr.ends_with(
            "resolution: restore a complete valid version 1 marker before rerunning enrollment\n"
        ),
        "{stderr}"
    );
    assert_eq!(
        files_beneath(repository),
        *before,
        "invalid marker refusal must leave the target tree byte-identical"
    );
}

fn marker_repository_id(marker: &[u8]) -> &str {
    let marker = std::str::from_utf8(marker).expect("fixture marker should be UTF-8");
    marker
        .lines()
        .find_map(|line| line.strip_prefix("repository_id = \"")?.strip_suffix('"'))
        .expect("fixture marker should contain repository_id")
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
        ("missing set visibility", &["set-visibility"]),
        (
            "invalid set visibility",
            &["set-visibility", "--visibility", "secret"],
        ),
        (
            "invalid expected repository identity",
            &[
                "enroll",
                "--ecosystem-id",
                "products",
                "--visibility",
                "private",
                "--expected-repository-id",
                "not-a-uuid",
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
fn enrollment_outside_repository_refuses_without_writes() {
    let fixture = Fixture::outside_repository("outside-repository");
    fs::write(fixture.root.join("preserved.txt"), b"unchanged\n")
        .expect("fixture content should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &fixture.root,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.starts_with("refusal:"), "{stderr}");
    assert!(stderr.contains("not inside a repository root"), "{stderr}");
    assert!(stderr.contains("\nresolution:"), "{stderr}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "refusal must leave the target tree byte-identical"
    );
}
