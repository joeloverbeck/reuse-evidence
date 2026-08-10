use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CASE_ID: &str = "00000000-0000-4000-8000-000000000011";
const SECOND_CASE_ID: &str = "00000000-0000-4000-8000-000000000021";
const STEWARD_ID: &str = "00000000-0000-4000-8000-000000000012";
const FIRST_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000013";
const SECOND_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000014";
const THIRD_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000015";
const DIFFERENT_APPEND_EVENT_ID: &str = "00000000-0000-4000-8000-000000000099";

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
            "reuse-evidence-case-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("fixture root should be creatable");
        Self { root }
    }

    fn repository(&self, name: &str, repository_id: &str, visibility: &str) -> PathBuf {
        let repository = self.git_repository(name);
        fs::write(
            repository.join("reuse-evidence.toml"),
            format!(
                "schema_version = 1\nrepository_id = \"{repository_id}\"\necosystem_id = \"products\"\nvisibility = \"{visibility}\"\n"
            ),
        )
        .expect("repository fixture should be enrolled");
        repository
    }

    fn git_repository(&self, name: &str) -> PathBuf {
        let repository = self.root.join(name);
        fs::create_dir_all(repository.join(".git"))
            .expect("repository fixture should be creatable");
        fs::write(
            repository.join(".git").join("HEAD"),
            b"ref: refs/heads/main\n",
        )
        .expect("repository fixture should contain recognizable Git metadata");
        repository
    }

    fn proposal(&self, contents: &str) -> PathBuf {
        let path = self.root.join("open-case.toml");
        fs::write(&path, contents).expect("proposal should be writable");
        path
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

fn run_without_portfolio_configuration(
    fixture: &Fixture,
    repository: &Path,
    arguments: &[&str],
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(arguments)
        .current_dir(repository)
        .env("XDG_CONFIG_HOME", fixture.root.join("unconfigured"))
        .env("XDG_STATE_HOME", fixture.root.join("unconfigured-state"))
        .output()
        .expect("compiled reuse-evidence binary should run")
}

fn spawn_competing_later_event_writers(
    steward: &Path,
    root: &str,
    append_proposal: &Path,
    override_proposal: &Path,
) -> (Child, Child) {
    let append = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args([
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            append_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ])
        .current_dir(steward)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("append process should start");
    let override_process = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args([
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            override_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ])
        .current_dir(steward)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("override process should start");
    (append, override_process)
}

fn recover_case_revision(fixture: &Fixture, steward: &Path, case_id: &str) -> String {
    let recovered =
        run_without_portfolio_configuration(fixture, steward, &["case", "show", case_id]);
    assert_eq!(recovered.status.code(), Some(0), "{recovered:?}");
    String::from_utf8(recovered.stdout)
        .expect("stdout should be UTF-8")
        .lines()
        .find_map(|line| line.strip_prefix("revision: "))
        .expect("fresh case read should report a recoverable revision")
        .to_owned()
}

fn assert_repeated_open_append_is_idempotent(
    fixture: &Fixture,
    steward: &Path,
    open_arguments: &[&str],
    append_arguments: &[&str],
) {
    let before = files_beneath(&fixture.root);
    let repeated_open = run_in(steward, open_arguments);
    let repeated_append = run_in(steward, append_arguments);
    assert_eq!(repeated_open.status.code(), Some(0), "{repeated_open:?}");
    assert_eq!(
        repeated_append.status.code(),
        Some(0),
        "{repeated_append:?}"
    );
    assert!(
        String::from_utf8(repeated_open.stdout)
            .expect("stdout should be UTF-8")
            .starts_with("existing case\n")
    );
    assert!(
        String::from_utf8(repeated_append.stdout)
            .expect("stdout should be UTF-8")
            .starts_with("occurrence already recorded\n")
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "repeating the complete open-then-append sequence must add no case or event"
    );
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

fn two_occurrence_proposal() -> String {
    format!(
        "case_id = \"{CASE_ID}\"\nresponsibility = \"normalize durable event identities\"\n\n[[occurrences]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nindependence = \"separate release lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\npath = \"src/event.rs\"\n\n[[occurrences]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\nindependence = \"independent npm workspace and owner\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"2222222\"\npath = \"packages/events/src/id.ts\"\n"
    )
}

fn append_occurrence_proposal() -> String {
    format!(
        "[occurrence]\nrepository_id = \"{THIRD_PARTICIPANT_ID}\"\nconsumer = \"desktop-packager\"\nindependence = \"separate distribution contract\"\n\n[[occurrence.evidence]]\nkind = \"commit\"\nreference = \"3333333\"\npath = \"src/package.rs\"\n"
    )
}

fn early_review_override_proposal() -> String {
    "reason = \"coordinated compatibility fixes are already required\"\nreview_appetite = \"compare the two contracts for at most one working day\"\n\n[[evidence]]\nkind = \"commit\"\nreference = \"4444444\"\npath = \"docs/compatibility.md\"\n"
        .to_owned()
}

fn duplicate_occurrence_append_proposal() -> String {
    format!(
        "[occurrence]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nindependence = \"a repository move is not a new consumer need\"\n\n[[occurrence.evidence]]\nkind = \"commit\"\nreference = \"3333333\"\npath = \"src/event.rs\"\n"
    )
}

fn three_occurrence_proposal() -> String {
    format!(
        "case_id = \"{SECOND_CASE_ID}\"\nresponsibility = \"preserve generated artifact identity\"\n\n[[occurrences]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nindependence = \"separate release lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"3333333\"\npath = \"src/artifact.rs\"\n\n[[occurrences]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\nindependence = \"independent npm workspace and owner\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"4444444\"\npath = \"packages/artifacts/src/id.ts\"\n\n[[occurrences]]\nrepository_id = \"{THIRD_PARTICIPANT_ID}\"\nconsumer = \"desktop-packager\"\nindependence = \"separate distribution contract\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"5555555\"\npath = \"src/package.rs\"\n"
    )
}

#[test]
fn listing_reports_every_stewarded_case_and_derived_state_without_writes_or_portfolio_roots() {
    let fixture = Fixture::new("list-cases");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, three_occurrence_proposal())
        .expect("second case proposal should be writable");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let before = files_beneath(&fixture.root);

    let first = run_without_portfolio_configuration(&fixture, &steward, &["case", "list"]);
    let second = run_without_portfolio_configuration(&fixture, &steward, &["case", "list"]);

    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert!(first.stderr.is_empty(), "{first:?}");
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        first.stdout, second.stdout,
        "unchanged reads must be identical"
    );
    let stdout = String::from_utf8(first.stdout).expect("stdout should be UTF-8");
    assert_eq!(
        stdout,
        format!(
            "cases\n- case_id: {CASE_ID}\n  revision: 1\n  occurrence_count: 2\n  state: watching\n  privacy_conflicted: unknown\n  stale: unknown\n- case_id: {SECOND_CASE_ID}\n  revision: 1\n  occurrence_count: 3\n  state: review-ready\n  readiness_basis: occurrence-count\n  readiness: authorizes semantic review; does not authorize extraction\n  privacy_conflicted: unknown\n  stale: unknown\nportfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n"
        )
    );
    for prohibited in [
        "score",
        "percentage",
        "ranking",
        "duplication",
        "health metric",
    ] {
        assert!(
            !stdout.to_lowercase().contains(prohibited),
            "case listing must not contain `{prohibited}`: {stdout}"
        );
    }
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "listing cases must not create a cache, index, projection, or any other file"
    );
}

#[test]
fn review_r1_spec_1_case_list_succeeds_without_platform_configuration_directory() {
    let fixture = Fixture::new("list-without-platform-configuration");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let before_read = files_beneath(&fixture.root);

    let output = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(["case", "list"])
        .current_dir(&steward)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HOME")
        .env_remove("APPDATA")
        .output()
        .expect("compiled reuse-evidence binary should run");

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("privacy_conflicted: unknown\n  stale: unknown\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("portfolio conditions unavailable:"),
        "{stdout}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "listing without a platform configuration directory must write nothing"
    );
}

#[test]
fn showing_a_case_reports_its_complete_record_without_writes() {
    let fixture = Fixture::new("show-case");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let before = files_beneath(&fixture.root);

    let first = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);
    let second =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert!(first.stderr.is_empty(), "{first:?}");
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        first.stdout, second.stdout,
        "unchanged reads must be identical"
    );
    assert_eq!(
        String::from_utf8(first.stdout).expect("stdout should be UTF-8"),
        format!(
            "case\ncase_id: {CASE_ID}\nresponsibility: normalize durable event identities\nrevision: 1\noccurrence_count: 2\nstate: watching\nprivacy_conflicted: unknown\nstale: unknown\noccurrences:\n- repository_id: {FIRST_PARTICIPANT_ID}\n  consumer: rust-release-tool\n  independence: separate release lifecycle\n  evidence:\n  - kind: commit\n    reference: 1111111\n    path: src/event.rs\n- repository_id: {SECOND_PARTICIPANT_ID}\n  consumer: web-deployment-tool\n  independence: independent npm workspace and owner\n  evidence:\n  - kind: commit\n    reference: 2222222\n    path: packages/events/src/id.ts\nportfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "showing a case must not create a cache, index, projection, or any other file"
    );
}

#[test]
fn case_read_recomputes_privacy_conflict_from_current_enrollment() {
    let fixture = Fixture::new("privacy-conflict");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    let second = fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");

    let before_change = run_in(&steward, &["case", "show", CASE_ID, "--root", root]);
    assert_eq!(before_change.status.code(), Some(0), "{before_change:?}");
    let before_change_stdout =
        String::from_utf8(before_change.stdout).expect("stdout should be UTF-8");
    assert!(
        before_change_stdout.contains("privacy_conflicted: false\nstale: false\n"),
        "{before_change_stdout}"
    );

    let visibility_change = run_in(&second, &["set-visibility", "--visibility", "private"]);
    assert_eq!(
        visibility_change.status.code(),
        Some(0),
        "{visibility_change:?}"
    );
    let before_read = files_beneath(&fixture.root);

    let after_change = run_in(&steward, &["case", "show", CASE_ID, "--root", root]);

    assert_eq!(after_change.status.code(), Some(0), "{after_change:?}");
    assert!(after_change.stderr.is_empty(), "{after_change:?}");
    let stdout = String::from_utf8(after_change.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("privacy_conflicted: true\nstale: false\n"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("portfolio conditions unavailable"),
        "an explicit root must make current conditions available: {stdout}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "deriving a privacy conflict must write nothing"
    );
}

#[test]
fn case_read_uses_configured_portfolio_to_report_withdrawn_participant_as_stale() {
    let fixture = Fixture::new("stale-participant");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    let withdrawn = fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let event_path = steward
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0001-case-opened.toml");
    let recorded_event = fs::read(&event_path).expect("opening event should be readable");
    let config_home = fixture.root.join("config-home");
    fs::create_dir_all(config_home.join("reuse-evidence"))
        .expect("fixture configuration directory should be creatable");
    fs::write(
        config_home.join("reuse-evidence/config.toml"),
        format!(
            "portfolio_roots = [{}]\n",
            toml::Value::String(root.to_owned())
        ),
    )
    .expect("fixture portfolio configuration should be writable");

    fs::remove_file(withdrawn.join("reuse-evidence.toml"))
        .expect("fixture participant marker should be removable");
    let before_read = files_beneath(&fixture.root);

    let output = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(["case", "show", CASE_ID])
        .current_dir(&steward)
        .env("XDG_CONFIG_HOME", &config_home)
        .env("XDG_STATE_HOME", fixture.root.join("configured-state"))
        .output()
        .expect("compiled reuse-evidence binary should run");

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("privacy_conflicted: false\nstale: true\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "- repository_id: {SECOND_PARTICIPANT_ID}\n  consumer: web-deployment-tool\n  independence: independent npm workspace and owner\n  evidence:\n  - kind: commit\n    reference: 2222222\n    path: packages/events/src/id.ts\n"
        )),
        "the withdrawn participant's occurrence must remain fully visible: {stdout}"
    );
    assert_eq!(
        fs::read(&event_path).expect("opening event should remain readable"),
        recorded_event,
        "withdrawal and reading must not alter historical occurrence bytes"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "deriving staleness must write nothing"
    );
}

#[test]
fn case_read_refuses_duplicated_sequence_number_without_writes() {
    let fixture = Fixture::new("duplicate-case-sequence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let case_directory = steward.join("reuse-evidence/cases").join(CASE_ID);
    fs::copy(
        case_directory.join("0001-case-opened.toml"),
        case_directory.join("0001-duplicate.toml"),
    )
    .expect("damaged duplicate-sequence fixture should be creatable");
    let before_read = files_beneath(&fixture.root);

    let output =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{CASE_ID}` has duplicated sequence number 1 in files `0001-case-opened.toml`, `0001-duplicate.toml`\nresolution: restore exactly one event file for sequence 1 before reading the case\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "a duplicated sequence refusal must write nothing"
    );
}

#[test]
fn review_r1_case_read_refuses_duplicate_occurrence_across_events_without_writes() {
    let fixture = Fixture::new("duplicate-occurrence-across-events");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");
    let appended = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(appended.status.code(), Some(0), "{appended:?}");
    let append_event = steward
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0002-occurrence-appended.toml");
    let duplicate_event = fs::read_to_string(&append_event)
        .expect("append event should be readable")
        .replace(THIRD_PARTICIPANT_ID, FIRST_PARTICIPANT_ID)
        .replace("desktop-packager", "rust-release-tool");
    fs::write(&append_event, duplicate_event)
        .expect("damaged duplicate-occurrence fixture should be writable");
    let before_read = files_beneath(&fixture.root);

    let output =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{CASE_ID}` records participant `{FIRST_PARTICIPANT_ID}` and consumer `rust-release-tool` more than once\nresolution: restore the authoritative event stream so each participant repository and consumer pair occurs once before reading the case\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "a duplicate occurrence in recorded history must write nothing"
    );
}

#[test]
fn review_r1_spec_2_case_read_refuses_body_sequence_that_disagrees_with_filename() {
    let fixture = Fixture::new("event-body-sequence-mismatch");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let case_directory = steward.join("reuse-evidence/cases").join(CASE_ID);
    let mismatched_event = case_directory.join("0002-case-opened.toml");
    fs::copy(
        case_directory.join("0001-case-opened.toml"),
        &mismatched_event,
    )
    .expect("damaged body-sequence fixture should be creatable");
    let before_read = files_beneath(&fixture.root);

    let output =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case event `{}` records sequence 1 but its filename records sequence 2\nresolution: restore the event under the filename matching its recorded sequence before reading the case\n",
            mismatched_event.display()
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "an event body sequence mismatch refusal must write nothing"
    );
}

#[test]
fn review_r1_standards_1_case_read_refuses_second_case_opened_event() {
    let fixture = Fixture::new("second-case-opened-event");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let case_directory = steward.join("reuse-evidence/cases").join(CASE_ID);
    let second_opening = case_directory.join("0002-case-opened.toml");
    let second_opening_bytes = fs::read_to_string(case_directory.join("0001-case-opened.toml"))
        .expect("opening event should be readable")
        .replacen("sequence = 1", "sequence = 2", 1);
    fs::write(&second_opening, second_opening_bytes)
        .expect("damaged repeated-opening fixture should be writable");
    let before_read = files_beneath(&fixture.root);

    let output =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case event `{}` records `case_opened` at sequence 2\nresolution: restore `case_opened` as the single sequence 1 opening event before reading the case\n",
            second_opening.display()
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "a repeated opening-event refusal must write nothing"
    );
}

#[test]
fn review_r2_standards_1_case_read_refuses_invalid_opening_filename_and_content_without_writes() {
    let suffix_fixture = Fixture::new("wrong-opening-event-suffix");
    let suffix_steward = suffix_fixture.repository("steward", STEWARD_ID, "private");
    suffix_fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    suffix_fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let suffix_proposal = suffix_fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &suffix_steward,
        &[
            "case",
            "open",
            "--proposal",
            suffix_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            suffix_fixture
                .root
                .to_str()
                .expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let suffix_case_directory = suffix_steward.join("reuse-evidence/cases").join(CASE_ID);
    fs::rename(
        suffix_case_directory.join("0001-case-opened.toml"),
        suffix_case_directory.join("0001-arbitrary.toml"),
    )
    .expect("wrong event-type suffix fixture should be creatable");
    let suffix_before_read = files_beneath(&suffix_fixture.root);
    let suffix_output = run_without_portfolio_configuration(
        &suffix_fixture,
        &suffix_steward,
        &["case", "show", CASE_ID],
    );

    let content_fixture = Fixture::new("invalid-opening-content");
    let content_steward = content_fixture.repository("steward", STEWARD_ID, "private");
    content_fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    content_fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let content_proposal = content_fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &content_steward,
        &[
            "case",
            "open",
            "--proposal",
            content_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            content_fixture
                .root
                .to_str()
                .expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let content_event = content_steward
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0001-case-opened.toml");
    let invalid_content = fs::read_to_string(&content_event)
        .expect("opening event should be readable")
        .replacen("consumer = \"rust-release-tool\"", "consumer = \"   \"", 1);
    fs::write(&content_event, invalid_content)
        .expect("invalid opening-content fixture should be writable");
    let content_before_read = files_beneath(&content_fixture.root);
    let content_output = run_without_portfolio_configuration(
        &content_fixture,
        &content_steward,
        &["case", "show", CASE_ID],
    );

    assert_eq!(
        (suffix_output.status.code(), content_output.status.code()),
        (Some(3), Some(3)),
        "suffix={suffix_output:?}\ncontent={content_output:?}"
    );
    assert!(suffix_output.stdout.is_empty(), "{suffix_output:?}");
    assert!(content_output.stdout.is_empty(), "{content_output:?}");
    assert_eq!(
        String::from_utf8(suffix_output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{CASE_ID}` event file `0001-arbitrary.toml` does not match its recorded type `case_opened`\nresolution: restore the event as `0001-case-opened.toml` before reading the case\n"
        )
    );
    assert_eq!(
        String::from_utf8(content_output.stderr).expect("stderr should be UTF-8"),
        "refusal: occurrence 1 consumer is empty\nresolution: provide a non-empty consumer label\n"
    );
    assert_eq!(
        files_beneath(&suffix_fixture.root),
        suffix_before_read,
        "a wrong event-type suffix refusal must write nothing"
    );
    assert_eq!(
        files_beneath(&content_fixture.root),
        content_before_read,
        "an invalid opening-content refusal must write nothing"
    );
}

#[test]
fn case_read_refuses_missing_sequence_number_without_writes() {
    let fixture = Fixture::new("missing-case-sequence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let case_directory = steward.join("reuse-evidence/cases").join(CASE_ID);
    fs::rename(
        case_directory.join("0001-case-opened.toml"),
        case_directory.join("0002-case-opened.toml"),
    )
    .expect("damaged missing-sequence fixture should be creatable");
    let before_read = files_beneath(&fixture.root);

    let output =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{CASE_ID}` is missing sequence number 1 before recorded sequence 2\nresolution: restore event file sequence 1 so the case stream is contiguous before reading it\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "a missing sequence refusal must write nothing"
    );
}

fn assert_case_opened_event(fixture: &Fixture, event: &str) {
    assert!(
        !event.contains(fixture.root.to_string_lossy().as_ref()),
        "recorded evidence must not contain absolute fixture paths: {event}"
    );
    let parsed = event
        .parse::<toml::Table>()
        .expect("case opening event should be valid TOML");
    assert_eq!(parsed["schema_version"].as_integer(), Some(1));
    assert_eq!(parsed["sequence"].as_integer(), Some(1));
    assert_eq!(parsed["event_type"].as_str(), Some("case_opened"));
    assert_eq!(parsed["case_id"].as_str(), Some(CASE_ID));
    assert_eq!(
        parsed["responsibility"].as_str(),
        Some("normalize durable event identities")
    );
    assert_eq!(parsed["steward_repository_id"].as_str(), Some(STEWARD_ID));
    assert_eq!(parsed["privacy"].as_str(), Some("private"));
    let event_id = parsed["event_id"]
        .as_str()
        .expect("event identity should be a string");
    assert_eq!(
        uuid::Uuid::parse_str(event_id)
            .expect("event identity should be an opaque UUID")
            .get_version_num(),
        4
    );
    let recorded_at = parsed["recorded_at"]
        .as_str()
        .expect("recorded_at should be an RFC 3339 string");
    assert!(
        recorded_at.len() == 20
            && recorded_at.as_bytes()[4] == b'-'
            && recorded_at.as_bytes()[7] == b'-'
            && recorded_at.as_bytes()[10] == b'T'
            && recorded_at.as_bytes()[13] == b':'
            && recorded_at.as_bytes()[16] == b':'
            && recorded_at.ends_with('Z'),
        "recorded_at should use the accepted UTC RFC 3339 shape: {recorded_at}"
    );
    let occurrences = parsed["occurrences"]
        .as_array()
        .expect("event should contain occurrences");
    assert_eq!(occurrences.len(), 2);
    assert_eq!(
        occurrences[0]["repository_id"].as_str(),
        Some(FIRST_PARTICIPANT_ID)
    );
    assert_eq!(
        occurrences[0]["consumer"].as_str(),
        Some("rust-release-tool")
    );
    assert_eq!(
        occurrences[0]["independence"].as_str(),
        Some("separate release lifecycle")
    );
    assert_eq!(
        occurrences[0]["evidence"][0]["kind"].as_str(),
        Some("commit")
    );
    assert_eq!(
        occurrences[0]["evidence"][0]["reference"].as_str(),
        Some("1111111")
    );
    assert_eq!(
        occurrences[0]["evidence"][0]["path"].as_str(),
        Some("src/event.rs")
    );
    assert_eq!(
        occurrences[1]["repository_id"].as_str(),
        Some(SECOND_PARTICIPANT_ID)
    );
    assert_eq!(
        occurrences[1]["consumer"].as_str(),
        Some("web-deployment-tool")
    );
    assert_eq!(
        occurrences[1]["independence"].as_str(),
        Some("independent npm workspace and owner")
    );
    assert_eq!(
        occurrences[1]["evidence"][0]["path"].as_str(),
        Some("packages/events/src/id.ts")
    );
    assert!(
        !fixture.root.join("rust-consumer/Cargo.toml").exists()
            && !fixture.root.join("typescript-consumer/Cargo.toml").exists(),
        "case participation must not require a Cargo project"
    );
}

#[test]
fn preview_renders_exact_case_opened_event_without_writes() {
    let fixture = Fixture::new("preview");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("rust-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("typescript-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
            "--preview",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "preview must preserve every fixture byte"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let (receipt, event) = stdout
        .split_once("event:\n")
        .expect("preview should separate its receipt from the exact event");
    assert_eq!(
        receipt,
        format!(
            "case open preview\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0001-case-opened.toml\nrevision: 1\nprivacy: private\n"
        )
    );
    assert_case_opened_event(&fixture, event);

    fs::write(&proposal, event).expect("the exact previewed event should be approvable");
    let apply = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(apply.status.code(), Some(0), "{apply:?}");
    assert!(apply.stderr.is_empty(), "{apply:?}");
    assert_eq!(
        fs::read_to_string(
            steward
                .join("reuse-evidence/cases")
                .join(CASE_ID)
                .join("0001-case-opened.toml")
        )
        .expect("approved event should be recorded"),
        event,
        "applying the approved preview must preserve its exact bytes"
    );
}

#[test]
fn case_event_uses_accepted_recorded_at_toml_shape() {
    let fixture = Fixture::new("recorded-at-shape");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
            "--preview",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let (_, event) = stdout
        .split_once("event:\n")
        .expect("preview should contain its exact event");
    assert_case_opened_event(&fixture, event);
    assert!(
        !event.contains("recorded_at_unix_seconds"),
        "the durable schema must use the accepted recorded_at field: {event}"
    );
}

#[test]
fn opening_creates_one_case_event_and_reports_the_exact_consequence() {
    let fixture = Fixture::new("open");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("rust-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("typescript-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        format!(
            "opened case\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0001-case-opened.toml\nrevision: 1\nprivacy: private\n"
        )
    );

    let relative_event = PathBuf::from("steward")
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0001-case-opened.toml");
    let mut after = files_beneath(&fixture.root);
    let event = after
        .remove(&relative_event)
        .expect("opening should create its one event file");
    assert_eq!(after, before, "opening must add only its event file");

    let event = String::from_utf8(event).expect("event should be UTF-8");
    assert!(
        !event.contains(fixture.root.to_string_lossy().as_ref()),
        "recorded evidence must not contain absolute fixture paths: {event}"
    );
    let parsed = event
        .parse::<toml::Table>()
        .expect("recorded event should be valid TOML");
    assert_eq!(parsed["schema_version"].as_integer(), Some(1));
    assert_eq!(parsed["sequence"].as_integer(), Some(1));
    assert_eq!(parsed["event_type"].as_str(), Some("case_opened"));
    assert_eq!(parsed["case_id"].as_str(), Some(CASE_ID));
    assert_eq!(parsed["steward_repository_id"].as_str(), Some(STEWARD_ID));
    assert_eq!(parsed["privacy"].as_str(), Some("private"));
    assert_eq!(
        parsed["occurrences"]
            .as_array()
            .expect("event should contain occurrences")
            .len(),
        2
    );
}

#[test]
fn appending_third_occurrence_creates_one_event_and_derives_review_ready() {
    let fixture = Fixture::new("append-third-occurrence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        format!(
            "appended occurrence\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-occurrence-appended.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: occurrence-count\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );

    let relative_event = PathBuf::from("steward")
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0002-occurrence-appended.toml");
    let mut after = files_beneath(&fixture.root);
    let event = after
        .remove(&relative_event)
        .expect("append should create its one event file");
    assert_eq!(after, before, "append must add only its event file");
    let event = String::from_utf8(event).expect("event should be UTF-8");
    let parsed = event
        .parse::<toml::Table>()
        .expect("appended occurrence event should be valid TOML");
    assert_eq!(parsed["schema_version"].as_integer(), Some(1));
    assert_eq!(parsed["sequence"].as_integer(), Some(2));
    assert_eq!(parsed["event_type"].as_str(), Some("occurrence_appended"));
    assert_eq!(
        uuid::Uuid::parse_str(
            parsed["event_id"]
                .as_str()
                .expect("event identity should be a string")
        )
        .expect("event identity should be an opaque UUID")
        .get_version_num(),
        4
    );
    assert_eq!(
        parsed["occurrence"]["repository_id"].as_str(),
        Some(THIRD_PARTICIPANT_ID)
    );
    assert_eq!(
        parsed["occurrence"]["consumer"].as_str(),
        Some("desktop-packager")
    );
    assert_eq!(
        parsed["occurrence"]["independence"].as_str(),
        Some("separate distribution contract")
    );
    assert_eq!(
        parsed["occurrence"]["evidence"][0]["reference"].as_str(),
        Some("3333333")
    );

    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    let shown = String::from_utf8(shown.stdout).expect("stdout should be UTF-8");
    assert!(shown.contains(
        "revision: 2\noccurrence_count: 3\nstate: review-ready\nreadiness_basis: occurrence-count\n"
    ));
    assert!(
        shown.contains("readiness: authorizes semantic review; does not authorize extraction\n")
    );
    assert!(shown.contains(
        "- repository_id: 00000000-0000-4000-8000-000000000015\n  consumer: desktop-packager\n  independence: separate distribution contract\n"
    ));
}

#[test]
fn recording_early_review_override_creates_one_event_and_derives_override_ready() {
    let fixture = Fixture::new("early-review-override");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let open_proposal_path = open_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let override_proposal = fixture.root.join("early-review.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            override_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        format!(
            "authorized early review\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-early-review-authorized.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );

    let relative_event = PathBuf::from("steward")
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0002-early-review-authorized.toml");
    let mut after = files_beneath(&fixture.root);
    let event = after
        .remove(&relative_event)
        .expect("override should create its one event file");
    assert_eq!(after, before, "override must add only its event file");
    let event = String::from_utf8(event).expect("event should be UTF-8");
    let parsed = event
        .parse::<toml::Table>()
        .expect("early-review event should be valid TOML");
    assert_eq!(parsed["schema_version"].as_integer(), Some(1));
    assert_eq!(parsed["sequence"].as_integer(), Some(2));
    assert_eq!(
        parsed["event_type"].as_str(),
        Some("early_review_authorized")
    );
    assert_eq!(
        parsed["reason"].as_str(),
        Some("coordinated compatibility fixes are already required")
    );
    assert_eq!(
        parsed["review_appetite"].as_str(),
        Some("compare the two contracts for at most one working day")
    );
    assert_eq!(parsed["evidence"].as_array().map(Vec::len), Some(1));
    assert_eq!(parsed["evidence"][0]["kind"].as_str(), Some("commit"));
    assert_eq!(parsed["evidence"][0]["reference"].as_str(), Some("4444444"));

    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    let shown = String::from_utf8(shown.stdout).expect("stdout should be UTF-8");
    assert!(shown.contains(
        "revision: 2\noccurrence_count: 2\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\n"
    ));
    assert!(shown.contains(
        "early_review:\n  reason: coordinated compatibility fixes are already required\n  review_appetite: compare the two contracts for at most one working day\n  evidence:\n  - kind: commit\n    reference: 4444444\n    path: docs/compatibility.md\n"
    ));
}

#[test]
fn approved_early_review_preview_is_byte_exact_and_retry_is_idempotent() {
    let fixture = Fixture::new("preview-and-retry-early-review");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let open_proposal_path = open_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let override_proposal = fixture.root.join("early-review.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let override_proposal_path = override_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let arguments = [
        "case",
        "override",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        override_proposal_path,
        "--root",
        root,
    ];
    let mut preview_arguments = arguments.to_vec();
    preview_arguments.push("--preview");
    let before_preview = files_beneath(&fixture.root);

    let preview = run_in(&steward, &preview_arguments);

    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    assert!(preview.stderr.is_empty(), "{preview:?}");
    assert_eq!(
        files_beneath(&fixture.root),
        before_preview,
        "early-review preview must preserve every fixture byte"
    );
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (receipt, event) = preview
        .split_once("event:\n")
        .expect("preview should separate its receipt from the exact event");
    assert_eq!(
        receipt,
        format!(
            "early-review override preview\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-early-review-authorized.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
    fs::write(&override_proposal, event)
        .expect("the exact previewed early-review event should be approvable");

    let applied = run_in(&steward, &arguments);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    assert_eq!(
        fs::read_to_string(
            steward
                .join("reuse-evidence/cases")
                .join(CASE_ID)
                .join("0002-early-review-authorized.toml")
        )
        .expect("approved early-review event should be recorded"),
        event,
        "applying an approved preview must preserve its exact bytes"
    );
    let before_retry = files_beneath(&fixture.root);

    let retry_arguments = [
        "case",
        "override",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        override_proposal_path,
    ];
    let retry = run_in(&steward, &retry_arguments);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "early review already authorized\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-early-review-authorized.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "retrying the same early-review event identity must preserve every fixture byte"
    );
}

#[test]
fn review_r1_standards_1_concurrent_later_event_writers_publish_one_sequence() {
    let fixture = Fixture::new("r1-cross-event-single-sequence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let append_proposal = fixture.root.join("append.toml");
    fs::write(&append_proposal, append_occurrence_proposal())
        .expect("append proposal should be writable");
    let override_proposal = fixture.root.join("override.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("override proposal should be writable");
    let opening = File::open(
        steward
            .join("reuse-evidence/cases")
            .join(CASE_ID)
            .join("0001-case-opened.toml"),
    )
    .expect("opening event should be readable");
    opening
        .lock()
        .expect("test should be able to hold the case write lock");

    let (mut append, mut override_process) =
        spawn_competing_later_event_writers(&steward, root, &append_proposal, &override_proposal);
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        append
            .try_wait()
            .expect("append status should be readable")
            .is_none(),
        "append must wait while another process holds the case write lock"
    );
    assert!(
        override_process
            .try_wait()
            .expect("override status should be readable")
            .is_none(),
        "override must wait while another process holds the case write lock"
    );
    drop(opening);

    let append = append
        .wait_with_output()
        .expect("append process should finish");
    let override_output = override_process
        .wait_with_output()
        .expect("override process should finish");
    let status_codes = [append.status.code(), override_output.status.code()];
    assert_eq!(
        status_codes.iter().filter(|code| **code == Some(0)).count(),
        1,
        "exactly one same-revision writer should publish: append={append:?}, override={override_output:?}"
    );
    assert_eq!(
        status_codes.iter().filter(|code| **code == Some(3)).count(),
        1,
        "the competing writer should refuse: append={append:?}, override={override_output:?}"
    );
    let case_directory = steward.join("reuse-evidence/cases").join(CASE_ID);
    let sequence_two = fs::read_dir(&case_directory)
        .expect("case should remain readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("0002-"))
        .count();
    assert_eq!(sequence_two, 1, "only one sequence-two event may exist");
}

#[test]
fn review_r1_spec_1_cross_event_loser_refuses_stale_without_duplicate_sequence() {
    let fixture = Fixture::new("r1-cross-event-stale-loser");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let append_proposal = fixture.root.join("append.toml");
    fs::write(&append_proposal, append_occurrence_proposal())
        .expect("append proposal should be writable");
    let override_proposal = fixture.root.join("override.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("override proposal should be writable");
    let opening = File::open(
        steward
            .join("reuse-evidence/cases")
            .join(CASE_ID)
            .join("0001-case-opened.toml"),
    )
    .expect("opening event should be readable");
    opening
        .lock()
        .expect("test should be able to hold the case write lock");

    let (append, override_process) =
        spawn_competing_later_event_writers(&steward, root, &append_proposal, &override_proposal);
    std::thread::sleep(Duration::from_millis(100));
    drop(opening);

    let append = append
        .wait_with_output()
        .expect("append process should finish");
    let override_output = override_process
        .wait_with_output()
        .expect("override process should finish");
    let loser = [&append, &override_output]
        .into_iter()
        .find(|output| output.status.code() == Some(3))
        .expect("one competing writer should refuse");
    assert!(loser.stdout.is_empty(), "{loser:?}");
    let loser_stderr = String::from_utf8(loser.stderr.clone()).expect("stderr should be UTF-8");
    assert!(
        loser_stderr.starts_with(&format!(
            "refusal: expected revision 1 does not match case `{CASE_ID}` current revision 2\n"
        )),
        "{loser_stderr}"
    );
    assert!(
        loser_stderr.contains(&format!("run `case show {CASE_ID}`")),
        "{loser_stderr}"
    );
    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    let shown = String::from_utf8(shown.stdout).expect("stdout should be UTF-8");
    assert!(shown.contains("revision: 2\n"), "{shown}");
}

#[test]
fn review_r1_standards_2_public_steward_refuses_private_case_override_without_writes() {
    let fixture = Fixture::new("r1-public-steward-private-case-override");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let visibility_change = run_in(&steward, &["set-visibility", "--visibility", "public"]);
    assert_eq!(
        visibility_change.status.code(),
        Some(0),
        "{visibility_change:?}"
    );
    let override_proposal = fixture.root.join("override.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("override proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            override_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: public steward `{STEWARD_ID}` cannot authorize early review for private case `{CASE_ID}`\nresolution: run `set-visibility --visibility private` in the steward repository, then preview the early-review override again\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a privacy refusal must preserve every fixture byte"
    );
}

#[test]
fn review_r1_spec_2_append_after_override_preserves_override_readiness_basis() {
    let fixture = Fixture::new("r1-append-after-override-basis");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let override_proposal = fixture.root.join("override.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("override proposal should be writable");
    let override_output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            override_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(
        override_output.status.code(),
        Some(0),
        "{override_output:?}"
    );
    let append_proposal = fixture.root.join("append.toml");
    fs::write(&append_proposal, append_occurrence_proposal())
        .expect("append proposal should be writable");

    let appended = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            append_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );

    assert_eq!(appended.status.code(), Some(0), "{appended:?}");
    assert!(appended.stderr.is_empty(), "{appended:?}");
    assert_eq!(
        String::from_utf8(appended.stdout).expect("stdout should be UTF-8"),
        format!(
            "appended occurrence\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0003-occurrence-appended.toml\nrevision: 3\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    let shown = String::from_utf8(shown.stdout).expect("stdout should be UTF-8");
    assert!(shown.contains(
        "revision: 3\noccurrence_count: 3\nstate: review-ready\nreadiness_basis: early-review-override\n"
    ));
}

#[test]
fn review_r2_standards_1_exact_override_retry_survives_steward_visibility_change() {
    let fixture = Fixture::new("r2-idempotent-override-visibility");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let override_proposal = fixture.root.join("override.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("override proposal should be writable");
    let proposal_path = override_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let arguments = [
        "case",
        "override",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
        "--root",
        root,
    ];
    let mut preview_arguments = arguments.to_vec();
    preview_arguments.push("--preview");
    let preview = run_in(&steward, &preview_arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain exact event bytes");
    fs::write(&override_proposal, event).expect("prepared event should be writable");
    let applied = run_in(&steward, &arguments);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    let visibility_change = run_in(&steward, &["set-visibility", "--visibility", "public"]);
    assert_eq!(
        visibility_change.status.code(),
        Some(0),
        "{visibility_change:?}"
    );
    let before_retry = files_beneath(&fixture.root);

    let retry_arguments = [
        "case",
        "override",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
    ];
    let retry = run_in(&steward, &retry_arguments);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert!(
        String::from_utf8(retry.stdout)
            .expect("stdout should be UTF-8")
            .starts_with("early review already authorized\n"),
        "exact prepared identity should retain the success meaning"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "an exact retry after visibility change must preserve every byte"
    );
}

#[test]
fn review_r2_spec_1_exact_override_retry_reports_recorded_event_after_public_transition() {
    let fixture = Fixture::new("r2-exact-override-retry-public-transition");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let override_proposal = fixture.root.join("override.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("override proposal should be writable");
    let proposal_path = override_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let arguments = [
        "case",
        "override",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
        "--root",
        root,
    ];
    let mut preview_arguments = arguments.to_vec();
    preview_arguments.push("--preview");
    let preview = run_in(&steward, &preview_arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain exact event bytes");
    fs::write(&override_proposal, event).expect("prepared event should be writable");
    let applied = run_in(&steward, &arguments);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    let visibility_change = run_in(&steward, &["set-visibility", "--visibility", "public"]);
    assert_eq!(
        visibility_change.status.code(),
        Some(0),
        "{visibility_change:?}"
    );

    let retry = run_in(&steward, &arguments);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "early review already authorized\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-early-review-authorized.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
    let event_path = steward
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0002-early-review-authorized.toml");
    assert_eq!(
        fs::read_to_string(event_path).expect("recorded event should remain readable"),
        event,
        "the exact recorded event must remain unchanged"
    );
}

#[test]
fn review_r2_spec_2_override_receipts_apply_current_private_steward_consequence() {
    let fixture = Fixture::new("r2-override-current-private-consequence");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let visibility_change = run_in(&steward, &["set-visibility", "--visibility", "private"]);
    assert_eq!(
        visibility_change.status.code(),
        Some(0),
        "{visibility_change:?}"
    );
    let override_proposal = fixture.root.join("override.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("override proposal should be writable");
    let proposal_path = override_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let arguments = [
        "case",
        "override",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
        "--root",
        root,
    ];
    let mut preview_arguments = arguments.to_vec();
    preview_arguments.push("--preview");
    let before_preview = files_beneath(&fixture.root);

    let preview = run_in(&steward, &preview_arguments);

    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    assert_eq!(
        files_beneath(&fixture.root),
        before_preview,
        "preview must preserve every fixture byte"
    );
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    assert!(preview.contains("privacy: private\nevent:\n"), "{preview}");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain exact event bytes");
    fs::write(&override_proposal, event).expect("prepared event should be writable");

    let applied = run_in(&steward, &arguments);

    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    assert!(applied.stderr.is_empty(), "{applied:?}");
    assert_eq!(
        String::from_utf8(applied.stdout).expect("stdout should be UTF-8"),
        format!(
            "authorized early review\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-early-review-authorized.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
}

#[test]
fn review_r3_spec_1_override_refuses_participant_that_became_private_without_writes() {
    let fixture = Fixture::new("r3-override-current-private-participant");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    let second = fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            open_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let visibility_change = run_in(&second, &["set-visibility", "--visibility", "private"]);
    assert_eq!(
        visibility_change.status.code(),
        Some(0),
        "{visibility_change:?}"
    );
    let override_proposal = fixture.root.join("override.toml");
    fs::write(&override_proposal, early_review_override_proposal())
        .expect("override proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            override_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: public steward `{STEWARD_ID}` cannot authorize early review for private case `{CASE_ID}`\nresolution: run `set-visibility --visibility private` in the steward repository, then preview the early-review override again\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "current private-participant refusal must preserve every fixture byte"
    );
}

#[test]
fn early_review_override_without_reason_refuses_without_writes() {
    let fixture = Fixture::new("early-review-without-reason");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(
        &proposal,
        "review_appetite = \"compare the two contracts for at most one working day\"\n\n[[evidence]]\nkind = \"commit\"\nreference = \"4444444\"\n",
    )
    .expect("incomplete early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: early-review override reason is missing\nresolution: provide a concrete reason why waiting for a third occurrence is materially worse\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a missing early-review reason must preserve every fixture byte"
    );
}

#[test]
fn early_review_override_without_evidence_refuses_without_writes() {
    let fixture = Fixture::new("early-review-without-evidence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(
        &proposal,
        "reason = \"coordinated compatibility fixes are already required\"\nreview_appetite = \"compare the two contracts for at most one working day\"\n",
    )
    .expect("incomplete early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: early-review override evidence is missing\nresolution: add one or more recoverable evidence references bearing why waiting is worse\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "missing early-review evidence must preserve every fixture byte"
    );
}

#[test]
fn early_review_override_without_appetite_refuses_without_writes() {
    let fixture = Fixture::new("early-review-without-appetite");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(
        &proposal,
        "reason = \"coordinated compatibility fixes are already required\"\n\n[[evidence]]\nkind = \"commit\"\nreference = \"4444444\"\n",
    )
    .expect("incomplete early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: early-review override review appetite is missing\nresolution: bound the review effort before authorizing early review\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a missing early-review appetite must preserve every fixture byte"
    );
}

#[test]
fn early_review_override_on_occurrence_count_ready_case_refuses_without_writes() {
    let fixture = Fixture::new("early-review-on-count-ready-case");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal().replace(SECOND_CASE_ID, CASE_ID));
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{CASE_ID}` is already review-ready from 3 recorded occurrences\nresolution: proceed to semantic review; an early-review override cannot change this case's readiness\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an occurrence-count-ready case refusal must preserve every fixture byte"
    );
}

#[test]
fn early_review_override_with_stale_revision_refuses_without_writes() {
    let fixture = Fixture::new("early-review-stale-revision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: expected revision 2 does not match case `{CASE_ID}` current revision 1\nresolution: run `case show {CASE_ID}` and retry `case override {CASE_ID}` with `--expected-revision 1` and the approved proposal\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a stale early-review revision must preserve every fixture byte"
    );
}

#[test]
fn early_review_override_on_unknown_case_refuses_and_creates_nothing() {
    let fixture = Fixture::new("early-review-unknown-case");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let proposal = fixture.proposal(&early_review_override_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case identity `{CASE_ID}` is not stewarded by repository `{STEWARD_ID}`\nresolution: run `case list` in this steward repository and retry `case override` with a recorded watching case identity\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an unknown early-review case must create no case directory or other file"
    );
}

#[test]
fn early_review_override_requires_declared_expected_revision_without_writes() {
    let fixture = Fixture::new("early-review-missing-expected-revision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &["case", "override", CASE_ID, "--proposal", proposal_path],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: missing required `--expected-revision`\nresolution: run `case show {CASE_ID}` to recover the current revision, then rerun `case override {CASE_ID} --expected-revision <REVISION>`\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a missing expected revision must preserve every fixture byte"
    );
}

#[test]
fn early_review_override_with_empty_evidence_refuses_without_writes() {
    let fixture = Fixture::new("early-review-empty-evidence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(
        &proposal,
        "reason = \"coordinated compatibility fixes are already required\"\nreview_appetite = \"compare the two contracts for at most one working day\"\nevidence = []\n",
    )
    .expect("empty-evidence early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: early-review override requires at least one evidence reference\nresolution: add one or more recoverable evidence references bearing why waiting is worse\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an empty early-review evidence collection must preserve every fixture byte"
    );
}

#[test]
fn second_early_review_override_refuses_without_writes() {
    let fixture = Fixture::new("second-early-review-override");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let first = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    fs::write(
        &proposal,
        early_review_override_proposal().replace(
            "coordinated compatibility fixes are already required",
            "the published contract is diverging",
        ),
    )
    .expect("second early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{CASE_ID}` is already review-ready from its recorded early-review override\nresolution: proceed to semantic review; do not record another early-review override\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a second early-review override must preserve every fixture byte"
    );
}

#[test]
fn approved_append_preview_is_byte_exact_and_retry_is_idempotent() {
    let fixture = Fixture::new("preview-and-retry-append");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let open_proposal = fixture.proposal(&two_occurrence_proposal());
    let open_proposal_path = open_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let append_proposal = fixture.root.join("append-case.toml");
    fs::write(&append_proposal, append_occurrence_proposal())
        .expect("append proposal should be writable");
    let append_proposal_path = append_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let open_arguments = [
        "case",
        "open",
        "--proposal",
        open_proposal_path,
        "--root",
        root,
    ];
    let opened = run_in(&steward, &open_arguments);
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");

    let recovered_revision = recover_case_revision(&fixture, &steward, CASE_ID);
    assert_eq!(recovered_revision, "1");

    let before_preview = files_beneath(&fixture.root);
    let append_arguments = [
        "case",
        "append",
        CASE_ID,
        "--expected-revision",
        recovered_revision.as_str(),
        "--proposal",
        append_proposal_path,
        "--root",
        root,
    ];
    let mut preview_arguments = append_arguments.to_vec();
    preview_arguments.push("--preview");

    let preview = run_in(&steward, &preview_arguments);

    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    assert!(preview.stderr.is_empty(), "{preview:?}");
    assert_eq!(
        files_beneath(&fixture.root),
        before_preview,
        "append preview must preserve every fixture byte"
    );
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (receipt, event) = preview
        .split_once("event:\n")
        .expect("preview should separate its receipt from the exact event");
    assert_eq!(
        receipt,
        format!(
            "case append preview\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-occurrence-appended.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: occurrence-count\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
    fs::write(&append_proposal, event).expect("the exact previewed append should be approvable");

    let applied = run_in(&steward, &append_arguments);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    assert_eq!(
        fs::read_to_string(
            steward
                .join("reuse-evidence/cases")
                .join(CASE_ID)
                .join("0002-occurrence-appended.toml")
        )
        .expect("approved append event should be recorded"),
        event,
        "applying an approved append preview must preserve its exact bytes"
    );
    let before_retry = files_beneath(&fixture.root);

    let retry = run_in(&steward, &append_arguments);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "occurrence already recorded\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-occurrence-appended.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: occurrence-count\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "retrying the same append event identity must preserve every fixture byte"
    );

    assert_repeated_open_append_is_idempotent(
        &fixture,
        &steward,
        &open_arguments,
        &append_arguments,
    );
}

#[test]
fn review_r3_spec_1_idempotent_append_retry_reports_complete_case_privacy() {
    let fixture = Fixture::new("retry-complete-case-privacy");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    assert!(
        String::from_utf8(opened.stdout)
            .expect("stdout should be UTF-8")
            .ends_with("privacy: public\n")
    );
    let visibility_change = run_in(&steward, &["set-visibility", "--visibility", "private"]);
    assert_eq!(
        visibility_change.status.code(),
        Some(0),
        "{visibility_change:?}"
    );
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");
    let arguments = [
        "case",
        "append",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
        "--root",
        root,
    ];
    let mut preview_arguments = arguments.to_vec();
    preview_arguments.push("--preview");
    let preview = run_in(&steward, &preview_arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain the exact event");
    fs::write(&proposal, event).expect("prepared append event should be writable");

    let applied = run_in(&steward, &arguments);

    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    assert!(applied.stderr.is_empty(), "{applied:?}");
    assert_eq!(
        String::from_utf8(applied.stdout).expect("stdout should be UTF-8"),
        format!(
            "appended occurrence\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-occurrence-appended.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: occurrence-count\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
    let before_retry = files_beneath(&fixture.root);

    let retry = run_in(&steward, &arguments);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "occurrence already recorded\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-occurrence-appended.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: occurrence-count\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: private\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "idempotent retry must preserve the private case byte-for-byte"
    );
}

#[test]
fn review_r1_interrupted_append_staging_leaves_case_readable_and_retryable() {
    let fixture = Fixture::new("interrupted-append-staging");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");
    let arguments = [
        "case",
        "append",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
        "--root",
        root,
    ];
    let mut preview_arguments = arguments.to_vec();
    preview_arguments.push("--preview");
    let preview = run_in(&steward, &preview_arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain the exact event");
    fs::write(&proposal, event).expect("prepared append event should be writable");
    let case_directory = steward.join("reuse-evidence/cases").join(CASE_ID);
    fs::write(
        case_directory
            .join(".0002-occurrence-appended.toml.00000000-0000-4000-8000-000000000088.tmp"),
        event,
    )
    .expect("interrupted append staging should be reproducible");
    let before_read = files_beneath(&fixture.root);

    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    assert!(shown.stderr.is_empty(), "{shown:?}");
    assert!(
        String::from_utf8(shown.stdout)
            .expect("stdout should be UTF-8")
            .contains("revision: 1\noccurrence_count: 2\nstate: watching\n")
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_read,
        "reading through interrupted append staging must write nothing"
    );

    let applied = run_in(&steward, &arguments);

    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    assert!(applied.stderr.is_empty(), "{applied:?}");
    assert_eq!(
        fs::read_to_string(case_directory.join("0002-occurrence-appended.toml"))
            .expect("the retried append should publish its authoritative event"),
        event,
    );
    let recovered = recover_case_revision(&fixture, &steward, CASE_ID);
    assert_eq!(recovered, "2");
}

#[test]
fn append_with_mismatched_expected_revision_refuses_without_writes() {
    let fixture = Fixture::new("append-stale-revision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: expected revision 2 does not match case `{CASE_ID}` current revision 1\nresolution: run `case show {CASE_ID}` and retry with `--expected-revision 1`\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an expected-revision refusal must preserve every fixture byte"
    );
}

#[test]
fn review_r1_append_refuses_revision_beyond_four_digit_event_layout_without_writes() {
    let fixture = Fixture::new("append-four-digit-revision-limit");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "9999",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: expected revision 9999 cannot be appended because the accepted `NNNN` event layout ends at revision 9999\nresolution: preserve the case unchanged and obtain an accepted event-layout amendment before appending another occurrence\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an unsupported successor revision must preserve every fixture byte"
    );
}

#[test]
fn occupied_append_sequence_with_different_event_identity_is_a_revision_conflict() {
    let fixture = Fixture::new("append-event-identity-conflict");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");
    let arguments = [
        "case",
        "append",
        CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
        "--root",
        root,
    ];
    let mut preview_arguments = arguments.to_vec();
    preview_arguments.push("--preview");
    let preview = run_in(&steward, &preview_arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain the exact event");
    let recorded_event_id = event
        .parse::<toml::Table>()
        .expect("previewed event should be TOML")["event_id"]
        .as_str()
        .expect("event identity should be a string")
        .to_owned();
    fs::write(&proposal, event).expect("approved append should be writable");
    let applied = run_in(&steward, &arguments);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    fs::write(
        &proposal,
        event.replacen(&recorded_event_id, DIFFERENT_APPEND_EVENT_ID, 1),
    )
    .expect("conflicting prepared event should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(&steward, &arguments);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{CASE_ID}` has a revision conflict at sequence 2: event `{recorded_event_id}` is recorded instead of event `{DIFFERENT_APPEND_EVENT_ID}`\nresolution: inspect sequence 2; retry its recorded identity if it is the intended append, or prepare a distinct occurrence against revision 2\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a different event identity at an occupied sequence must write nothing"
    );
}

#[test]
fn append_to_unknown_case_identity_refuses_and_creates_nothing() {
    let fixture = Fixture::new("append-unknown-case");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&append_occurrence_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case identity `{CASE_ID}` is not stewarded by repository `{STEWARD_ID}`\nresolution: run `case list` in this steward repository and retry with a recorded case identity\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an unknown case refusal must create no case directory or other file"
    );
}

#[test]
fn duplicate_participant_and_consumer_append_refuses_without_writes() {
    let fixture = Fixture::new("append-duplicate-occurrence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, duplicate_occurrence_append_proposal())
        .expect("duplicate append proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{CASE_ID}` already records participant `{FIRST_PARTICIPANT_ID}` and consumer `rust-release-tool`\nresolution: change either the participant repository or consumer so the pair is distinct, or keep the existing occurrence\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a duplicate occurrence refusal must preserve every fixture byte"
    );
}

#[test]
fn public_steward_refuses_private_appended_participant_without_writes() {
    let fixture = Fixture::new("append-private-participant");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, append_occurrence_proposal())
        .expect("private append proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: public steward `{STEWARD_ID}` cannot append private participant `{THIRD_PARTICIPANT_ID}`\nresolution: run `set-visibility --visibility private` in the steward repository, then preview the append again\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "private dominance refusal must preserve every fixture byte"
    );
}

#[test]
fn review_r1_append_rechecks_recorded_participant_visibility_before_writing() {
    let fixture = Fixture::new("append-recorded-participant-became-private");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    let first_participant = fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(
        first_participant.join("reuse-evidence.toml"),
        format!(
            "schema_version = 1\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\necosystem_id = \"products\"\nvisibility = \"private\"\n"
        ),
    )
    .expect("recorded participant visibility should be changeable in the fixture");
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: public steward `{STEWARD_ID}` cannot append private participant `{FIRST_PARTICIPANT_ID}`\nresolution: run `set-visibility --visibility private` in the steward repository, then preview the append again\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "current private dominance must refuse the append without writes"
    );
}

#[test]
fn appended_occurrence_without_evidence_refuses_without_writes() {
    let fixture = Fixture::new("append-without-evidence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(
        &proposal,
        format!(
            "[occurrence]\nrepository_id = \"{THIRD_PARTICIPANT_ID}\"\nconsumer = \"desktop-packager\"\nindependence = \"separate distribution contract\"\nevidence = []\n"
        ),
    )
    .expect("incomplete append proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: occurrence 1 carries no evidence reference\nresolution: add at least one recoverable `occurrence.evidence` reference\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an incomplete appended occurrence must preserve every fixture byte"
    );
}

#[test]
fn retrying_the_same_proposal_reports_the_existing_case_without_writes() {
    let fixture = Fixture::new("idempotent-open");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let arguments = [
        "case",
        "open",
        "--proposal",
        proposal.to_str().expect("fixture path should be UTF-8"),
        "--root",
        fixture.root.to_str().expect("fixture path should be UTF-8"),
    ];
    let first = run_in(&steward, &arguments);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let before_retry = files_beneath(&fixture.root);

    let retry = run_in(&steward, &arguments);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "existing case\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0001-case-opened.toml\nrevision: 1\nprivacy: private\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "an idempotent retry must preserve the complete tree byte-for-byte"
    );
}

#[test]
fn reusing_a_case_identity_for_different_content_refuses_without_writes() {
    let fixture = Fixture::new("conflicting-open");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let arguments = [
        "case",
        "open",
        "--proposal",
        proposal.to_str().expect("fixture path should be UTF-8"),
        "--root",
        fixture.root.to_str().expect("fixture path should be UTF-8"),
    ];
    let first = run_in(&steward, &arguments);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    fs::write(
        &proposal,
        two_occurrence_proposal().replace(
            "normalize durable event identities",
            "a conflicting responsibility",
        ),
    )
    .expect("conflicting proposal should be writable");
    let before_conflict = files_beneath(&fixture.root);

    let conflict = run_in(&steward, &arguments);

    assert_eq!(conflict.status.code(), Some(3), "{conflict:?}");
    assert!(conflict.stdout.is_empty(), "{conflict:?}");
    assert_eq!(
        String::from_utf8(conflict.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case identity `{CASE_ID}` is already recorded with different proposed content\nresolution: restore the exact original proposal or choose a new UUID version 4 case identity\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_conflict,
        "a conflicting identity retry must preserve the tree byte-for-byte"
    );
}

#[test]
fn opening_with_fewer_than_two_occurrences_refuses_without_writes() {
    let fixture = Fixture::new("one-occurrence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("only-consumer", FIRST_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&format!(
        "case_id = \"{CASE_ID}\"\nresponsibility = \"normalize durable event identities\"\n\n[[occurrences]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nindependence = \"separate release lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\npath = \"src/event.rs\"\n"
    ));
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: case opening requires at least two occurrences, but the proposal contains 1\nresolution: add a second independently evidenced occurrence before opening the case\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an undersized proposal must leave the tree byte-identical"
    );
}

#[test]
fn occurrence_without_evidence_refuses_without_writes() {
    let fixture = Fixture::new("missing-evidence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal_text = two_occurrence_proposal().replace(
        "\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\npath = \"src/event.rs\"\n",
        "\nevidence = []\n",
    );
    let proposal = fixture.proposal(&proposal_text);
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: occurrence 1 carries no evidence reference\nresolution: add at least one recoverable `occurrences.evidence` reference\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "missing evidence must leave the tree byte-identical"
    );
}

#[test]
fn duplicate_participant_and_consumer_refuses_without_writes() {
    let fixture = Fixture::new("duplicate-consumer");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("consumer", FIRST_PARTICIPANT_ID, "public");
    let proposal_text = two_occurrence_proposal()
        .replace(SECOND_PARTICIPANT_ID, FIRST_PARTICIPANT_ID)
        .replace("web-deployment-tool", "rust-release-tool");
    let proposal = fixture.proposal(&proposal_text);
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: multiple occurrences use participant `{FIRST_PARTICIPANT_ID}` and consumer `rust-release-tool`\nresolution: keep one occurrence for each distinct participant repository and reuse consumer\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "duplicate consumer evidence must leave the tree byte-identical"
    );
}

#[test]
fn meaningful_case_identity_refuses_without_writes() {
    let fixture = Fixture::new("meaningful-case-id");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal =
        fixture.proposal(&two_occurrence_proposal().replace(CASE_ID, "combat-resolver-case"));
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.starts_with(
            "refusal: case identity `combat-resolver-case` is not a well-formed opaque UUID:"
        ),
        "{stderr}"
    );
    assert!(
        stderr.ends_with("resolution: use a newly generated UUID version 4 as `case_id`\n"),
        "{stderr}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a meaningful case identity must leave the tree byte-identical"
    );
}

#[test]
fn undiscoverable_participant_refuses_without_writes() {
    let fixture = Fixture::new("undiscoverable-participant");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: participant `{SECOND_PARTICIPANT_ID}` does not resolve to a discoverable enrolled repository\nresolution: enroll the participant beneath a selected portfolio root or correct its repository identity\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an unresolvable participant must leave the tree byte-identical"
    );
}

#[test]
fn duplicated_participant_identity_refuses_without_writes() {
    let fixture = Fixture::new("duplicate-participant-identity");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let first = fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    let duplicate = fixture.repository("copied-consumer", FIRST_PARTICIPANT_ID, "private");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.starts_with(&format!(
            "refusal: participant identity `{FIRST_PARTICIPANT_ID}` is duplicated at:"
        )),
        "{stderr}"
    );
    assert!(
        stderr.contains(first.to_string_lossy().as_ref()),
        "{stderr}"
    );
    assert!(
        stderr.contains(duplicate.to_string_lossy().as_ref()),
        "{stderr}"
    );
    assert!(
        stderr.ends_with(
            "resolution: restore a unique stable repository identity before opening the case\n"
        ),
        "{stderr}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an ambiguous participant identity must leave the tree byte-identical"
    );
}

#[test]
fn public_steward_with_private_participant_refuses_without_writes() {
    let fixture = Fixture::new("private-dominance");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("private-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: public steward `{STEWARD_ID}` cannot hold private participant `{SECOND_PARTICIPANT_ID}`\nresolution: open the case from an enrolled private steward repository\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "private-dominance refusal must leave the tree byte-identical"
    );
}

#[test]
fn opening_from_unenrolled_repository_refuses_without_writes() {
    let fixture = Fixture::new("unenrolled-steward");
    let steward = fixture.git_repository("steward");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: repository is not enrolled because `{}` does not exist\nresolution: run `enroll` before opening a case\n",
            steward.join("reuse-evidence.toml").display()
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an unenrolled steward refusal must leave the tree byte-identical"
    );
}

#[test]
fn absolute_evidence_path_refuses_without_writes() {
    let fixture = Fixture::new("absolute-evidence-path");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let first = fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let absolute_path = first.join("src/event.rs");
    let proposal_text = two_occurrence_proposal().replace(
        "src/event.rs",
        absolute_path
            .to_str()
            .expect("fixture path should be representable as UTF-8"),
    );
    let proposal = fixture.proposal(&proposal_text);
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: evidence path `{}` is not repository-relative\nresolution: use a non-empty path relative to the participant repository without `..`\n",
            absolute_path.display()
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an absolute evidence path must leave the tree byte-identical"
    );
}

#[test]
fn empty_responsibility_refuses_without_writes() {
    let fixture = Fixture::new("empty-responsibility");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal().replace(
        "responsibility = \"normalize durable event identities\"",
        "responsibility = \"   \"",
    ));
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: responsibility is empty\nresolution: provide a non-empty `responsibility` value\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an empty responsibility must leave the tree byte-identical"
    );
}

#[test]
fn empty_consumer_refuses_without_writes() {
    let fixture = Fixture::new("empty-consumer");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(
        &two_occurrence_proposal()
            .replace("consumer = \"rust-release-tool\"", "consumer = \"   \""),
    );
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: occurrence 1 consumer is empty\nresolution: provide a non-empty consumer label\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an empty consumer must leave the tree byte-identical"
    );
}

#[test]
fn empty_independence_justification_refuses_without_writes() {
    let fixture = Fixture::new("empty-independence");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(&two_occurrence_proposal().replace(
        "independence = \"separate release lifecycle\"",
        "independence = \"   \"",
    ));
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: occurrence 1 independence justification is empty\nresolution: explain why this occurrence arose from an independent consumer need\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an empty independence justification must leave the tree byte-identical"
    );
}

#[test]
fn empty_evidence_reference_refuses_without_writes() {
    let fixture = Fixture::new("empty-evidence-reference");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let proposal = fixture.proposal(
        &two_occurrence_proposal().replace("reference = \"1111111\"", "reference = \"   \""),
    );
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: occurrence 1 evidence reference 1 is empty\nresolution: provide a recoverable commit reference\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an empty evidence reference must leave the tree byte-identical"
    );
}

#[cfg(unix)]
#[test]
fn interrupted_write_publishes_no_case_event() {
    let fixture = Fixture::new("interrupted-write");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let large_responsibility = "x".repeat(4_096);
    let proposal = fixture.proposal(
        &two_occurrence_proposal()
            .replace("normalize durable event identities", &large_responsibility),
    );
    let binary = env!("CARGO_BIN_EXE_reuse-evidence");
    let script = "ulimit -f 1; exec \"$1\" case open --proposal \"$2\" --root \"$3\"";

    let mut child = Command::new("sh")
        .args([
            "-c",
            script,
            "reuse-evidence-interruption-test",
            binary,
            proposal.to_str().expect("fixture path should be UTF-8"),
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ])
        .current_dir(&steward)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("limited compiled binary should start");
    let deadline = SystemTime::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            break status;
        }
        assert!(
            SystemTime::now() < deadline,
            "interrupted write should terminate promptly"
        );
        std::thread::yield_now();
    };
    assert!(
        !status.success(),
        "the file-size limit should interrupt the write"
    );

    let case_directory = steward.join("reuse-evidence/cases").join(CASE_ID);
    assert!(
        !case_directory.join("0001-case-opened.toml").exists(),
        "an interrupted write must not publish the authoritative event path"
    );
    if case_directory.exists() {
        for entry in fs::read_dir(&case_directory).expect("case directory should be readable") {
            let entry = entry.expect("case directory entry should be readable");
            assert_ne!(
                entry.file_name(),
                "0001-case-opened.toml",
                "no file that a case reader accepts may survive interruption"
            );
        }
    }

    let retry = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    let mut case_entries = fs::read_dir(&case_directory)
        .expect("recovered case directory should be readable")
        .map(|entry| {
            entry
                .expect("recovered case entry should be readable")
                .file_name()
        })
        .collect::<Vec<_>>();
    case_entries.sort();
    assert_eq!(
        case_entries,
        ["0001-case-opened.toml"],
        "recovery must leave exactly the authoritative event file"
    );
}

#[cfg(unix)]
#[test]
fn symlinked_case_directory_refuses_without_cross_repository_write() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new("symlinked-case-directory");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let redirected = fixture.root.join("redirected-case-records");
    fs::create_dir_all(&redirected).expect("redirect target should be creatable");
    let cases_root = steward.join("reuse-evidence/cases");
    fs::create_dir_all(&cases_root).expect("case root should be creatable");
    let case_directory = cases_root.join(CASE_ID);
    symlink(&redirected, &case_directory).expect("case directory symlink should be creatable");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "open",
            "--proposal",
            proposal.to_str().expect("fixture path should be UTF-8"),
            "--root",
            fixture.root.to_str().expect("fixture path should be UTF-8"),
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case storage path `{}` is a symbolic link\nresolution: replace every case storage symlink with a real directory or file inside the steward repository\n",
            case_directory.display()
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a case storage symlink refusal must preserve every repository byte"
    );
    assert!(
        !redirected.join("0001-case-opened.toml").exists(),
        "the steward command must not write through a symlink"
    );
}
