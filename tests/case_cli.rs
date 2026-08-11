use std::collections::BTreeMap;
use std::ffi::OsStr;
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
const DIFFERENT_DECISION_EVENT_ID: &str = "00000000-0000-4000-8000-000000000098";

struct Fixture {
    root: PathBuf,
}

struct WriteProtection {
    entries: Vec<(PathBuf, fs::Permissions)>,
}

impl WriteProtection {
    fn for_file_and_parent(path: &Path) -> Self {
        let parent = path.parent().expect("protected event should have a parent");
        let mut entries = Vec::new();
        for protected in [path, parent] {
            let original = fs::metadata(protected)
                .expect("protected path should have metadata")
                .permissions();
            let mut read_only = original.clone();
            read_only.set_readonly(true);
            fs::set_permissions(protected, read_only)
                .expect("existing event retry target should become read-only");
            entries.push((protected.to_path_buf(), original));
        }
        Self { entries }
    }
}

impl Drop for WriteProtection {
    fn drop(&mut self) {
        for (path, permissions) in self.entries.drain(..).rev() {
            let _ = fs::set_permissions(path, permissions);
        }
    }
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

// Constructs a state the CLI now refuses so later-event defenses remain covered.
fn force_marker_visibility(repository: &Path, from: &str, to: &str) {
    let marker_path = repository.join("reuse-evidence.toml");
    let marker = fs::read_to_string(&marker_path).expect("fixture marker should be readable");
    let from_line = format!("visibility = \"{from}\"");
    assert_eq!(
        marker.matches(&from_line).count(),
        1,
        "fixture marker should contain exactly one expected visibility"
    );
    fs::write(
        marker_path,
        marker.replacen(&from_line, &format!("visibility = \"{to}\""), 1),
    )
    .expect("fixture marker visibility should be writable");
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

fn spawn_competing_decision_and_append_writers(
    steward: &Path,
    root: &str,
    decision_proposal: &Path,
    append_proposal: &Path,
) -> (Child, Child) {
    let decision = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args([
            "case",
            "decide",
            CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            decision_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
        ])
        .current_dir(steward)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("decision process should start");
    let append = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args([
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
        ])
        .current_dir(steward)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("append process should start");
    (decision, append)
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

fn case_event_path_at_sequence(repository: &Path, case_id: &str, sequence: i64) -> PathBuf {
    let event_directory = repository.join("reuse-evidence/cases").join(case_id);
    let prefix = format!("{sequence:04}-");
    let mut matching = fs::read_dir(&event_directory)
        .expect("case event directory should be readable")
        .map(|entry| entry.expect("case event entry should be readable").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix))
                && path.extension() == Some(OsStr::new("toml"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matching.len(),
        1,
        "sequence {sequence} should identify exactly one case event"
    );
    matching.pop().expect("one matching event should exist")
}

fn assert_decision_receipt(
    receipt: &str,
    heading: &str,
    case_id: &str,
    revision: i64,
    privacy: &str,
    decision_notice: &str,
) {
    let lines = receipt.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 7, "unexpected decision receipt: {receipt}");
    assert_eq!(lines[0], heading);
    assert_eq!(lines[1], format!("case_id: {case_id}"));
    let file = lines[2]
        .strip_prefix("file: ")
        .expect("decision receipt should report its event file");
    let file_name = file
        .strip_prefix(&format!("reuse-evidence/cases/{case_id}/"))
        .expect("decision event should be reported beneath its case directory");
    assert!(file_name.starts_with(&format!("{revision:04}-")), "{file}");
    assert_eq!(
        Path::new(file_name).extension(),
        Some(OsStr::new("toml")),
        "{file}"
    );
    assert_eq!(lines[3], format!("revision: {revision}"));
    assert_eq!(lines[4], "state: awaiting-verification");
    assert_eq!(lines[5], format!("privacy: {privacy}"));
    assert_eq!(lines[6], format!("decision: {decision_notice}"));
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

fn change_decision_proposal() -> String {
    format!(
        "identity_verdict = \"same_responsibility\"\naction = \"publish_public_package\"\naccepted_scope = \"the durable event identity contract\"\nnon_responsibilities = [\"case lifecycle storage\"]\ncompatibility_consequences = \"preserve the existing event identity spelling\"\nverification_conditions = [\"all named consumers pass their public contract tests\"]\ninvariant_contract = \"one opaque UUID identifies one immutable event\"\nrequired_consumer_level_tests = [\"each consumer round-trips an event identity\"]\nrollback_or_resplitting_path = \"restore consumer-local implementations without rewriting recorded evidence\"\n\n[[affected_consumers]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nexpectation = \"migrate after the package publishes\"\n\n[[affected_consumers]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\nexpectation = \"retain its language-specific adapter\"\n\n[[alternatives_rejected]]\nalternative = \"retain intentional duplication\"\nreason = \"coordinated fixes already cross the consumer boundary\"\n\n[[existing_packages_considered]]\npackage = \"uuid\"\nfit = \"supplies identifiers but not the event contract\"\nreason = \"the invariant remains portfolio-owned\"\n\n[[migration_expectations]]\norder = 1\nexpectation = \"publish the invariant contract and its tests\"\n\n[[migration_expectations]]\norder = 2\nexpectation = \"migrate the Rust consumer before the web adapter\"\n"
    )
}

fn no_change_decision_proposal() -> String {
    format!(
        "identity_verdict = \"different_responsibilities\"\naction = \"retain_intentional_duplication\"\naccepted_scope = \"the two evidenced consumer implementations\"\nnon_responsibilities = [\"future consumers\"]\ncompatibility_consequences = \"each consumer retains its current contract\"\nverification_conditions = [\"both local implementations remain independently tested\"]\n\n[[affected_consumers]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nexpectation = \"retain its local implementation\"\n\n[[alternatives_rejected]]\nalternative = \"publish a public package\"\nreason = \"the consumers change for different reasons\"\n"
    )
}

fn record_overridden_decision_then_append(fixture: &Fixture, steward: &Path) {
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&two_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    for (contents, arguments) in [
        (
            None,
            vec!["case", "open", "--proposal", proposal_path, "--root", root],
        ),
        (
            Some(early_review_override_proposal()),
            vec![
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
        ),
        (
            Some(change_decision_proposal()),
            vec![
                "case",
                "decide",
                CASE_ID,
                "--expected-revision",
                "2",
                "--proposal",
                proposal_path,
                "--root",
                root,
            ],
        ),
        (
            Some(append_occurrence_proposal()),
            vec![
                "case",
                "append",
                CASE_ID,
                "--expected-revision",
                "3",
                "--proposal",
                proposal_path,
                "--root",
                root,
            ],
        ),
    ] {
        if let Some(contents) = contents {
            fs::write(&proposal, contents).expect("case operation proposal should be writable");
        }
        let output = run_in(steward, &arguments);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
    }
}

fn record_three_occurrence_decision(fixture: &Fixture, steward: &Path, decision: &str) {
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, decision).expect("decision proposal should be writable");
    let decided = run_in(
        steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(decided.status.code(), Some(0), "{decided:?}");
}

#[test]
fn implementation_brief_projects_all_change_decision_contents_without_writes() {
    let fixture = Fixture::new("change-implementation-brief");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    record_three_occurrence_decision(&fixture, &steward, &change_decision_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let before = files_beneath(&fixture.root);

    let brief = run_in(&steward, &["case", "brief", SECOND_CASE_ID, "--root", root]);

    assert_eq!(brief.status.code(), Some(0), "{brief:?}");
    assert!(brief.stderr.is_empty(), "{brief:?}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "projecting an implementation brief must preserve every fixture byte"
    );
    assert_eq!(
        String::from_utf8(brief.stdout).expect("stdout should be UTF-8"),
        format!(
            "implementation brief\ncase_id: {SECOND_CASE_ID}\nprivacy: private\nimplementation: authorized\naccepted_responsibility_identity:\n  responsibility: preserve generated artifact identity\n  verdict: same_responsibility\nevidence_bearing_consumers:\n- repository_id: {FIRST_PARTICIPANT_ID}\n  consumer: rust-release-tool\n  independence: separate release lifecycle\n  expectation: migrate after the package publishes\n  evidence:\n  - kind: commit\n    reference: 3333333\n    path: src/artifact.rs\n- repository_id: {SECOND_PARTICIPANT_ID}\n  consumer: web-deployment-tool\n  independence: independent npm workspace and owner\n  expectation: retain its language-specific adapter\n  evidence:\n  - kind: commit\n    reference: 4444444\n    path: packages/artifacts/src/id.ts\n- repository_id: {THIRD_PARTICIPANT_ID}\n  consumer: desktop-packager\n  independence: separate distribution contract\n  evidence:\n  - kind: commit\n    reference: 5555555\n    path: src/package.rs\ninvariant_contract: one opaque UUID identifies one immutable event\nnon_responsibilities:\n- case lifecycle storage\nchosen_home_and_scope:\n  action: publish_public_package\n  scope: the durable event identity contract\nalternatives_rejected:\n- alternative: retain intentional duplication\n  reason: coordinated fixes already cross the consumer boundary\nexisting_packages_considered:\n- package: uuid\n  fit: supplies identifiers but not the event contract\n  reason: the invariant remains portfolio-owned\nrequired_consumer_level_tests:\n- each consumer round-trips an event identity\ncompatibility_and_release_consequences: preserve the existing event identity spelling\nmigration_order:\n- order: 1\n  expectation: publish the invariant contract and its tests\n- order: 2\n  expectation: migrate the Rust consumer before the web adapter\nrollback_or_resplitting_strategy: restore consumer-local implementations without rewriting recorded evidence\nverification_conditions:\n- all named consumers pass their public contract tests\n"
        )
    );
}

#[test]
fn implementation_brief_carries_every_recorded_occurrence_and_evidence_reference() {
    let fixture = Fixture::new("brief-evidence-bearing-consumers");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    record_three_occurrence_decision(&fixture, &steward, &change_decision_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");

    let brief = run_in(&steward, &["case", "brief", SECOND_CASE_ID, "--root", root]);

    assert_eq!(brief.status.code(), Some(0), "{brief:?}");
    let stdout = String::from_utf8(brief.stdout).expect("stdout should be UTF-8");
    for recorded_occurrence in [
        format!(
            "- repository_id: {FIRST_PARTICIPANT_ID}\n  consumer: rust-release-tool\n  independence: separate release lifecycle\n  expectation: migrate after the package publishes\n  evidence:\n  - kind: commit\n    reference: 3333333\n    path: src/artifact.rs\n"
        ),
        format!(
            "- repository_id: {SECOND_PARTICIPANT_ID}\n  consumer: web-deployment-tool\n  independence: independent npm workspace and owner\n  expectation: retain its language-specific adapter\n  evidence:\n  - kind: commit\n    reference: 4444444\n    path: packages/artifacts/src/id.ts\n"
        ),
        format!(
            "- repository_id: {THIRD_PARTICIPANT_ID}\n  consumer: desktop-packager\n  independence: separate distribution contract\n  evidence:\n  - kind: commit\n    reference: 5555555\n    path: src/package.rs\n"
        ),
    ] {
        assert!(stdout.contains(&recorded_occurrence), "{stdout}");
    }
}

#[test]
fn no_change_brief_reports_that_no_implementation_is_authorized_with_success() {
    let fixture = Fixture::new("no-change-implementation-brief");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    record_three_occurrence_decision(&fixture, &steward, &no_change_decision_proposal());
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let before = files_beneath(&fixture.root);

    let brief = run_in(&steward, &["case", "brief", SECOND_CASE_ID, "--root", root]);

    assert_eq!(brief.status.code(), Some(0), "{brief:?}");
    assert!(brief.stderr.is_empty(), "{brief:?}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "projecting a no-change brief must preserve every fixture byte"
    );
    assert_eq!(
        String::from_utf8(brief.stdout).expect("stdout should be UTF-8"),
        format!(
            "implementation brief\ncase_id: {SECOND_CASE_ID}\nprivacy: private\nimplementation: not authorized\ndecision: authorizes no implementation\naccepted_responsibility_identity:\n  responsibility: preserve generated artifact identity\n  verdict: different_responsibilities\nchosen_home_and_scope:\n  action: retain_intentional_duplication\n  scope: the two evidenced consumer implementations\nnon_responsibilities:\n- future consumers\nalternatives_rejected:\n- alternative: publish a public package\n  reason: the consumers change for different reasons\ncompatibility_and_release_consequences: each consumer retains its current contract\nverification_conditions:\n- both local implementations remain independently tested\n"
        )
    );
}

#[test]
fn brief_without_an_accepted_decision_refuses_with_the_current_state_and_writes_nothing() {
    let fixture = Fixture::new("brief-without-decision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
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
    let before = files_beneath(&fixture.root);

    let brief = run_in(&steward, &["case", "brief", SECOND_CASE_ID, "--root", root]);

    assert_eq!(brief.status.code(), Some(3), "{brief:?}");
    assert!(brief.stdout.is_empty(), "{brief:?}");
    assert_eq!(
        String::from_utf8(brief.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case `{SECOND_CASE_ID}` has no accepted reuse decision; current state is `review-ready`\nresolution: record an accepted reuse decision, then rerun `case brief {SECOND_CASE_ID}`\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "refusing a brief without an accepted decision must preserve every fixture byte"
    );
}

#[test]
fn brief_for_an_unknown_case_identity_refuses_and_writes_nothing() {
    let fixture = Fixture::new("brief-unknown-case");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let before = files_beneath(&fixture.root);

    let brief =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "brief", CASE_ID]);

    assert_eq!(brief.status.code(), Some(3), "{brief:?}");
    assert!(brief.stdout.is_empty(), "{brief:?}");
    assert_eq!(
        String::from_utf8(brief.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: case identity `{CASE_ID}` is not stewarded by repository `{STEWARD_ID}`\nresolution: run `case list` in this steward repository, then retry `case brief <CASE_ID>` with one of its recorded case identities\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "refusing an unknown brief target must preserve every fixture byte"
    );
}

#[test]
fn brief_without_portfolio_configuration_succeeds_with_conservative_privacy() {
    let fixture = Fixture::new("brief-without-portfolio");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    record_three_occurrence_decision(&fixture, &steward, &change_decision_proposal());
    let before = files_beneath(&fixture.root);

    let brief =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "brief", SECOND_CASE_ID]);

    assert_eq!(brief.status.code(), Some(0), "{brief:?}");
    assert!(brief.stderr.is_empty(), "{brief:?}");
    let stdout = String::from_utf8(brief.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.starts_with(&format!(
            "implementation brief\ncase_id: {SECOND_CASE_ID}\nprivacy: unknown\nportfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\nimplementation: authorized\n"
        )),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "verification_conditions:\n- all named consumers pass their public contract tests\n"
        ),
        "{stdout}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a brief without portfolio configuration must preserve every fixture byte"
    );
}

#[test]
fn decision_preview_on_occurrence_ready_case_renders_the_event_without_writes() {
    let fixture = Fixture::new("decision-preview");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let open_proposal = fixture.proposal(&three_occurrence_proposal());
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
    let decision_proposal = fixture.root.join("decision.toml");
    fs::write(&decision_proposal, change_decision_proposal())
        .expect("decision proposal should be writable");
    let before = files_beneath(&fixture.root);

    let preview = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            decision_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            root,
            "--preview",
        ],
    );

    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    assert!(preview.stderr.is_empty(), "{preview:?}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "decision preview must preserve every fixture byte"
    );
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (receipt, event) = preview
        .split_once("event:\n")
        .expect("preview should separate its receipt from the exact event");
    assert_decision_receipt(
        receipt,
        "reuse decision preview",
        SECOND_CASE_ID,
        2,
        "private",
        "authorizes implementation outside the reuse lifecycle; does not perform it",
    );
    let parsed = event
        .parse::<toml::Table>()
        .expect("previewed reuse decision should be valid TOML");
    assert_eq!(parsed["schema_version"].as_integer(), Some(1));
    assert_eq!(parsed["sequence"].as_integer(), Some(2));
    assert_eq!(
        parsed["identity_verdict"].as_str(),
        Some("same_responsibility")
    );
    assert_eq!(parsed["action"].as_str(), Some("publish_public_package"));
    assert_eq!(
        parsed["affected_consumers"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(
        parsed["alternatives_rejected"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        parsed["existing_packages_considered"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        parsed["migration_expectations"].as_array().map(Vec::len),
        Some(2)
    );
}

#[test]
fn recording_a_reuse_decision_creates_one_event_and_derives_awaiting_verification() {
    let fixture = Fixture::new("record-decision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
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
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_decision_receipt(
        &stdout,
        "accepted reuse decision",
        SECOND_CASE_ID,
        2,
        "private",
        "authorizes implementation outside the reuse lifecycle; does not perform it",
    );
    let relative_event = case_event_path_at_sequence(&steward, SECOND_CASE_ID, 2)
        .strip_prefix(&fixture.root)
        .expect("decision event should be beneath the fixture root")
        .to_path_buf();
    let mut after = files_beneath(&fixture.root);
    let event = after
        .remove(&relative_event)
        .expect("accepting a decision should create its one event file");
    assert_eq!(after, before, "decision must add only its event file");
    let parsed = String::from_utf8(event)
        .expect("decision event should be UTF-8")
        .parse::<toml::Table>()
        .expect("decision event should be valid TOML");
    assert_eq!(
        parsed["accepted_scope"].as_str(),
        Some("the durable event identity contract")
    );
    assert_eq!(
        parsed["non_responsibilities"][0].as_str(),
        Some("case lifecycle storage")
    );
    assert_eq!(
        parsed["affected_consumers"][0]["expectation"].as_str(),
        Some("migrate after the package publishes")
    );
    assert_eq!(
        parsed["alternatives_rejected"][0]["reason"].as_str(),
        Some("coordinated fixes already cross the consumer boundary")
    );
    assert_eq!(
        parsed["compatibility_consequences"].as_str(),
        Some("preserve the existing event identity spelling")
    );
    assert_eq!(
        parsed["verification_conditions"][0].as_str(),
        Some("all named consumers pass their public contract tests")
    );

    let before_reads = files_beneath(&fixture.root);
    let shown =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", SECOND_CASE_ID]);
    let listed = run_without_portfolio_configuration(&fixture, &steward, &["case", "list"]);
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    assert_eq!(listed.status.code(), Some(0), "{listed:?}");
    let shown = String::from_utf8(shown.stdout).expect("stdout should be UTF-8");
    let listed = String::from_utf8(listed.stdout).expect("stdout should be UTF-8");
    assert!(
        shown.contains("revision: 2\noccurrence_count: 3\nstate: awaiting-verification\n"),
        "{shown}"
    );
    assert!(!shown.contains("readiness_basis:"), "{shown}");
    assert!(
        listed.contains("  state: awaiting-verification\n"),
        "{listed}"
    );
    assert!(!listed.contains("readiness_basis:"), "{listed}");
    assert_eq!(
        files_beneath(&fixture.root),
        before_reads,
        "derived decision state reads must write nothing"
    );
}

#[test]
fn case_read_refuses_invalid_recorded_reuse_decision_content_without_writes() {
    let fixture = Fixture::new("invalid-recorded-decision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let decided = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(decided.status.code(), Some(0), "{decided:?}");
    let event_path = case_event_path_at_sequence(&steward, SECOND_CASE_ID, 2);
    let invalid = fs::read_to_string(&event_path)
        .expect("decision event should be readable")
        .replace(
            "invariant_contract = \"one opaque UUID identifies one immutable event\"\n",
            "",
        );
    fs::write(&event_path, invalid).expect("invalid decision fixture should be writable");
    let before = files_beneath(&fixture.root);

    let shown =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", SECOND_CASE_ID]);

    assert_eq!(shown.status.code(), Some(3), "{shown:?}");
    assert!(shown.stdout.is_empty(), "{shown:?}");
    let stderr = String::from_utf8(shown.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("invariant_contract"), "{stderr}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "refusing invalid recorded decision content must write nothing"
    );
}

#[test]
fn case_read_refuses_decision_naming_an_unrecorded_consumer_without_writes() {
    let fixture = Fixture::new("invalid-recorded-decision-consumer");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let decided = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(decided.status.code(), Some(0), "{decided:?}");
    let event_path = case_event_path_at_sequence(&steward, SECOND_CASE_ID, 2);
    let invalid = fs::read_to_string(&event_path)
        .expect("decision event should be readable")
        .replacen(FIRST_PARTICIPANT_ID, DIFFERENT_DECISION_EVENT_ID, 1);
    fs::write(&event_path, invalid).expect("invalid decision fixture should be writable");
    let before = files_beneath(&fixture.root);

    let shown =
        run_without_portfolio_configuration(&fixture, &steward, &["case", "show", SECOND_CASE_ID]);

    assert_eq!(shown.status.code(), Some(3), "{shown:?}");
    assert!(shown.stdout.is_empty(), "{shown:?}");
    let stderr = String::from_utf8(shown.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("affected consumer"), "{stderr}");
    assert!(stderr.contains("not recorded"), "{stderr}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "refusing an unrecorded affected consumer must write nothing"
    );
}

#[test]
fn case_read_refuses_decision_recorded_before_prefix_became_review_ready_without_writes() {
    let fixture = Fixture::new("decision-before-ready-prefix");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    for (contents, arguments) in [
        (
            None,
            vec!["case", "open", "--proposal", proposal_path, "--root", root],
        ),
        (
            Some(change_decision_proposal()),
            vec![
                "case",
                "decide",
                SECOND_CASE_ID,
                "--expected-revision",
                "1",
                "--proposal",
                proposal_path,
                "--root",
                root,
            ],
        ),
        (
            Some(two_occurrence_proposal()),
            vec!["case", "open", "--proposal", proposal_path, "--root", root],
        ),
        (
            Some(early_review_override_proposal()),
            vec![
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
        ),
        (
            Some(append_occurrence_proposal()),
            vec![
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
        ),
    ] {
        if let Some(contents) = contents {
            fs::write(&proposal, contents).expect("case operation proposal should be writable");
        }
        let output = run_in(&steward, &arguments);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
    }
    let donor_decision_path = case_event_path_at_sequence(&steward, SECOND_CASE_ID, 2);
    let override_path = case_event_path_at_sequence(&steward, CASE_ID, 2);
    let target_decision_path = override_path
        .parent()
        .expect("target case event should have a parent")
        .join(
            donor_decision_path
                .file_name()
                .expect("donor decision should have a file name"),
        );
    fs::remove_file(&override_path).expect("override fixture should be removable");
    fs::copy(donor_decision_path, target_decision_path)
        .expect("CLI-authored decision fixture should be copyable");
    let before = files_beneath(&fixture.root);

    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(shown.status.code(), Some(3), "{shown:?}");
    assert!(shown.stdout.is_empty(), "{shown:?}");
    let stderr = String::from_utf8(shown.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("decision at sequence 2"), "{stderr}");
    assert!(stderr.contains("not review-ready"), "{stderr}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "refusing a decision before its prefix was ready must write nothing"
    );
}

#[test]
fn case_read_refuses_decision_consumer_recorded_only_by_later_append_without_writes() {
    let fixture = Fixture::new("decision-consumer-from-future");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    record_overridden_decision_then_append(&fixture, &steward);
    let decision_path = case_event_path_at_sequence(&steward, CASE_ID, 3);
    let invalid = fs::read_to_string(&decision_path)
        .expect("decision event should be readable")
        .replacen(FIRST_PARTICIPANT_ID, THIRD_PARTICIPANT_ID, 1)
        .replacen("rust-release-tool", "desktop-packager", 1);
    fs::write(&decision_path, invalid).expect("invalid decision fixture should be writable");
    let before = files_beneath(&fixture.root);

    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);

    assert_eq!(shown.status.code(), Some(3), "{shown:?}");
    assert!(shown.stdout.is_empty(), "{shown:?}");
    let stderr = String::from_utf8(shown.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("affected consumer `desktop-packager`"),
        "{stderr}"
    );
    assert!(
        stderr.contains("not recorded before sequence 3"),
        "{stderr}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a later occurrence must not retroactively legitimize a decision"
    );
}

#[test]
fn approved_reuse_decision_preview_is_byte_exact_and_retry_is_idempotent() {
    let fixture = Fixture::new("preview-and-retry-decision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let open_proposal = fixture.proposal(&three_occurrence_proposal());
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
    let decision_proposal = fixture.root.join("decision.toml");
    fs::write(&decision_proposal, change_decision_proposal())
        .expect("decision proposal should be writable");
    let proposal_path = decision_proposal
        .to_str()
        .expect("fixture path should be UTF-8");
    let arguments = [
        "case",
        "decide",
        SECOND_CASE_ID,
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
    assert!(preview.stderr.is_empty(), "{preview:?}");
    assert_eq!(
        files_beneath(&fixture.root),
        before_preview,
        "reuse decision preview must preserve every fixture byte"
    );
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should expose the exact accepted event");
    fs::write(&decision_proposal, event)
        .expect("the exact previewed reuse decision should be approvable");

    let applied = run_in(&steward, &arguments);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    assert_eq!(
        fs::read_to_string(case_event_path_at_sequence(&steward, SECOND_CASE_ID, 2))
            .expect("approved reuse decision should be recorded"),
        event,
        "applying an approved preview must preserve its exact bytes"
    );
    let before_retry = files_beneath(&fixture.root);

    let retry = run_in(&steward, &arguments);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    let stdout = String::from_utf8(retry.stdout).expect("stdout should be UTF-8");
    assert_decision_receipt(
        &stdout,
        "reuse decision already recorded",
        SECOND_CASE_ID,
        2,
        "private",
        "authorizes implementation outside the reuse lifecycle; does not perform it",
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "retrying the same reuse decision event identity must preserve every fixture byte"
    );
}

#[test]
fn exact_reuse_decision_retry_without_portfolio_reports_unknown_privacy_without_writes() {
    let fixture = Fixture::new("decision-retry-without-portfolio");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let preview = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
            "--preview",
        ],
    );
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should expose the exact event");
    fs::write(&proposal, event).expect("approved event should be writable as the proposal");
    let applied = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    let before = files_beneath(&fixture.root);

    let retry = run_without_portfolio_configuration(
        &fixture,
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
        ],
    );

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    let stdout = String::from_utf8(retry.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.starts_with("reuse decision already recorded\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("state: awaiting-verification\n"),
        "{stdout}"
    );
    assert!(stdout.contains("privacy: unknown\n"), "{stdout}");
    assert!(
        stdout.contains(
            "portfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n"
        ),
        "{stdout}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an exact retry without portfolio configuration must write nothing"
    );
}

#[test]
fn exact_reuse_decision_retry_with_unresolvable_participant_reports_unknown_privacy_without_writes()
{
    let fixture = Fixture::new("decision-retry-unresolvable-participant");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let first = fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let preview = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
            "--preview",
        ],
    );
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should expose the exact event");
    fs::write(&proposal, event).expect("approved event should be writable as the proposal");
    let arguments = [
        "case",
        "decide",
        SECOND_CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
        "--root",
        root,
    ];
    let applied = run_in(&steward, &arguments);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    fs::remove_file(first.join("reuse-evidence.toml"))
        .expect("participant enrollment should be removable");
    let before = files_beneath(&fixture.root);

    let retry = run_in(&steward, &arguments);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    let stdout = String::from_utf8(retry.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.starts_with("reuse decision already recorded\n"),
        "{stdout}"
    );
    assert!(stdout.contains("privacy: unknown\n"), "{stdout}");
    assert!(
        stdout.contains(
            "portfolio conditions unavailable: a recorded participant does not resolve to exactly one enrolled repository beneath the selected portfolio roots; restore its enrollment and unique repository identity to derive privacy\n"
        ),
        "{stdout}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an exact retry with an unresolvable participant must write nothing"
    );
}

#[test]
fn change_decision_missing_any_implementation_item_refuses_without_writes() {
    let fixture = Fixture::new("decision-missing-change-items");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let complete = change_decision_proposal();
    let existing_package = "[[existing_packages_considered]]\npackage = \"uuid\"\nfit = \"supplies identifiers but not the event contract\"\nreason = \"the invariant remains portfolio-owned\"\n\n";
    let migration_expectations = "[[migration_expectations]]\norder = 1\nexpectation = \"publish the invariant contract and its tests\"\n\n[[migration_expectations]]\norder = 2\nexpectation = \"migrate the Rust consumer before the web adapter\"\n";
    let cases = [
        (
            "invariant_contract",
            complete.replace(
                "invariant_contract = \"one opaque UUID identifies one immutable event\"\n",
                "",
            ),
        ),
        (
            "existing_packages_considered",
            complete.replace(existing_package, ""),
        ),
        (
            "required_consumer_level_tests",
            complete.replace(
                "required_consumer_level_tests = [\"each consumer round-trips an event identity\"]\n",
                "",
            ),
        ),
        (
            "migration_expectations",
            complete.replace(migration_expectations, ""),
        ),
        (
            "rollback_or_resplitting_path",
            complete.replace(
                "rollback_or_resplitting_path = \"restore consumer-local implementations without rewriting recorded evidence\"\n",
                "",
            ),
        ),
    ];

    for (missing, contents) in cases {
        fs::write(&proposal, contents).expect("incomplete decision proposal should be writable");
        let before = files_beneath(&fixture.root);
        let output = run_in(
            &steward,
            &[
                "case",
                "decide",
                SECOND_CASE_ID,
                "--expected-revision",
                "1",
                "--proposal",
                proposal_path,
                "--root",
                root,
                "--preview",
            ],
        );
        assert_eq!(output.status.code(), Some(3), "{missing}: {output:?}");
        assert!(output.stdout.is_empty(), "{missing}: {output:?}");
        let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        assert!(stderr.contains(missing), "{missing}: {stderr}");
        assert!(stderr.contains("resolution:"), "{missing}: {stderr}");
        assert_eq!(
            files_beneath(&fixture.root),
            before,
            "missing {missing} must leave the complete fixture byte-identical"
        );
    }
}

#[test]
fn no_change_decision_carrying_any_implementation_item_refuses_without_writes() {
    let fixture = Fixture::new("no-change-decision-extra-items");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let base = no_change_decision_proposal();
    let cases = [
        (
            "invariant_contract",
            "invariant_contract = \"nothing is implemented by this decision\"\n",
        ),
        (
            "existing_packages_considered",
            "[[existing_packages_considered]]\npackage = \"uuid\"\nfit = \"not applicable\"\nreason = \"no implementation is authorized\"\n",
        ),
        (
            "required_consumer_level_tests",
            "required_consumer_level_tests = [\"no implementation test\"]\n",
        ),
        (
            "migration_expectations",
            "[[migration_expectations]]\norder = 1\nexpectation = \"do not migrate\"\n",
        ),
        (
            "rollback_or_resplitting_path",
            "rollback_or_resplitting_path = \"nothing to roll back\"\n",
        ),
    ];

    for (carried, extra) in cases {
        let contents = base.replacen(
            "\n[[affected_consumers]]",
            &format!("\n{extra}\n[[affected_consumers]]"),
            1,
        );
        fs::write(&proposal, contents).expect("no-change proposal should be writable");
        let before = files_beneath(&fixture.root);
        let output = run_in(
            &steward,
            &[
                "case",
                "decide",
                SECOND_CASE_ID,
                "--expected-revision",
                "1",
                "--proposal",
                proposal_path,
                "--root",
                root,
                "--preview",
            ],
        );
        assert_eq!(output.status.code(), Some(3), "{carried}: {output:?}");
        assert!(output.stdout.is_empty(), "{carried}: {output:?}");
        let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        assert!(stderr.contains(carried), "{carried}: {stderr}");
        assert!(stderr.contains("resolution:"), "{carried}: {stderr}");
        assert_eq!(
            files_beneath(&fixture.root),
            before,
            "carrying {carried} for a no-change action must preserve every fixture byte"
        );
    }
}

#[test]
fn no_change_decision_records_no_implementation_items_and_authorizes_no_implementation() {
    let fixture = Fixture::new("valid-no-change-decision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, no_change_decision_proposal())
        .expect("no-change decision proposal should be writable");

    let output = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
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
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("state: awaiting-verification\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("decision: authorizes no implementation\n"),
        "{stdout}"
    );
    let event = fs::read_to_string(case_event_path_at_sequence(&steward, SECOND_CASE_ID, 2))
        .expect("no-change decision event should be readable");
    let parsed = event
        .parse::<toml::Table>()
        .expect("no-change decision event should be valid TOML");
    assert_eq!(
        parsed["action"].as_str(),
        Some("retain_intentional_duplication")
    );
    for prohibited in [
        "invariant_contract",
        "existing_packages_considered",
        "required_consumer_level_tests",
        "migration_expectations",
        "rollback_or_resplitting_path",
    ] {
        assert!(
            !parsed.contains_key(prohibited),
            "no-change decision must omit `{prohibited}`: {event}"
        );
    }
}

#[test]
fn reuse_decision_empty_required_content_refuses_without_writes() {
    let fixture = Fixture::new("decision-empty-required-content");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let complete = change_decision_proposal();
    let affected_consumers = format!(
        "[[affected_consumers]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nexpectation = \"migrate after the package publishes\"\n\n[[affected_consumers]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\nexpectation = \"retain its language-specific adapter\"\n\n"
    );
    let rejected_alternative = "[[alternatives_rejected]]\nalternative = \"retain intentional duplication\"\nreason = \"coordinated fixes already cross the consumer boundary\"\n\n";
    let cases = [
        (
            "accepted_scope",
            complete.replace(
                "accepted_scope = \"the durable event identity contract\"",
                "accepted_scope = \"   \"",
            ),
        ),
        (
            "non_responsibilities",
            complete.replace(
                "non_responsibilities = [\"case lifecycle storage\"]",
                "non_responsibilities = []",
            ),
        ),
        (
            "affected_consumers",
            complete.replace(&affected_consumers, "affected_consumers = []\n\n"),
        ),
        (
            "alternatives_rejected",
            complete
                .replace(rejected_alternative, "")
                .replacen(
                    "\n[[affected_consumers]]",
                    "\nalternatives_rejected = []\n\n[[affected_consumers]]",
                    1,
                ),
        ),
        (
            "compatibility_consequences",
            complete.replace(
                "compatibility_consequences = \"preserve the existing event identity spelling\"",
                "compatibility_consequences = \"\"",
            ),
        ),
        (
            "verification_conditions",
            complete.replace(
                "verification_conditions = [\"all named consumers pass their public contract tests\"]",
                "verification_conditions = []",
            ),
        ),
    ];

    for (empty, contents) in cases {
        fs::write(&proposal, contents).expect("invalid decision proposal should be writable");
        let before = files_beneath(&fixture.root);
        let output = run_in(
            &steward,
            &[
                "case",
                "decide",
                SECOND_CASE_ID,
                "--expected-revision",
                "1",
                "--proposal",
                proposal_path,
                "--root",
                root,
                "--preview",
            ],
        );
        assert_eq!(output.status.code(), Some(3), "{empty}: {output:?}");
        assert!(output.stdout.is_empty(), "{empty}: {output:?}");
        let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        assert!(stderr.contains(empty), "{empty}: {stderr}");
        assert!(stderr.contains("resolution:"), "{empty}: {stderr}");
        assert_eq!(
            files_beneath(&fixture.root),
            before,
            "empty {empty} must preserve every fixture byte"
        );
    }
}

#[test]
fn unrecognized_reuse_decision_verdict_or_action_refuses_without_writes() {
    let fixture = Fixture::new("decision-unrecognized-vocabulary");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let complete = change_decision_proposal();
    let cases = [
        (
            "identity_verdict",
            "same_responsibility",
            "shape_looks_similar",
        ),
        ("action", "publish_public_package", "extract_automatically"),
    ];

    for (field, recognized, unrecognized) in cases {
        fs::write(
            &proposal,
            complete.replace(
                &format!("{field} = \"{recognized}\""),
                &format!("{field} = \"{unrecognized}\""),
            ),
        )
        .expect("invalid decision proposal should be writable");
        let before = files_beneath(&fixture.root);
        let output = run_in(
            &steward,
            &[
                "case",
                "decide",
                SECOND_CASE_ID,
                "--expected-revision",
                "1",
                "--proposal",
                proposal_path,
                "--root",
                root,
                "--preview",
            ],
        );
        assert_eq!(output.status.code(), Some(3), "{field}: {output:?}");
        assert!(output.stdout.is_empty(), "{field}: {output:?}");
        let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        assert!(stderr.contains(field), "{field}: {stderr}");
        assert!(stderr.contains(unrecognized), "{field}: {stderr}");
        assert!(stderr.contains("resolution:"), "{field}: {stderr}");
        assert_eq!(
            files_beneath(&fixture.root),
            before,
            "unrecognized {field} must preserve every fixture byte"
        );
    }
}

#[test]
fn every_permitted_reuse_decision_verdict_and_action_previews_without_writes() {
    let fixture = Fixture::new("decision-permitted-vocabulary");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let arguments = [
        "case",
        "decide",
        SECOND_CASE_ID,
        "--expected-revision",
        "1",
        "--proposal",
        proposal_path,
        "--root",
        root,
        "--preview",
    ];

    for verdict in [
        "same_responsibility",
        "different_responsibilities",
        "insufficient_evidence",
        "existing_abstraction_is_wrong",
    ] {
        fs::write(
            &proposal,
            change_decision_proposal().replace("same_responsibility", verdict),
        )
        .expect("decision proposal should be writable");
        let before = files_beneath(&fixture.root);
        let output = run_in(&steward, &arguments);
        assert_eq!(output.status.code(), Some(0), "{verdict}: {output:?}");
        assert_eq!(files_beneath(&fixture.root), before, "{verdict}");
    }

    for action in [
        "use_existing_dependency",
        "extract_or_deepen_locally",
        "create_workspace_package",
        "create_private_cross_repository_package",
        "publish_public_package",
        "centralize_schema_specification_or_fixture_corpus",
        "replace_copies_with_generated_artifacts",
        "contribute_missing_behavior_upstream",
        "split_inline_or_narrow_existing_abstraction",
    ] {
        fs::write(
            &proposal,
            change_decision_proposal().replace("publish_public_package", action),
        )
        .expect("change decision proposal should be writable");
        let before = files_beneath(&fixture.root);
        let output = run_in(&steward, &arguments);
        assert_eq!(output.status.code(), Some(0), "{action}: {output:?}");
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(
            stdout.contains("authorizes implementation outside"),
            "{action}: {stdout}"
        );
        assert_eq!(files_beneath(&fixture.root), before, "{action}");
    }

    for action in ["retain_intentional_duplication", "wait_for_more_evidence"] {
        fs::write(
            &proposal,
            no_change_decision_proposal().replace("retain_intentional_duplication", action),
        )
        .expect("no-change decision proposal should be writable");
        let before = files_beneath(&fixture.root);
        let output = run_in(&steward, &arguments);
        assert_eq!(output.status.code(), Some(0), "{action}: {output:?}");
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(
            stdout.contains("authorizes no implementation"),
            "{action}: {stdout}"
        );
        assert_eq!(files_beneath(&fixture.root), before, "{action}");
    }
}

#[test]
fn early_review_override_on_decided_case_refuses_without_writes() {
    let fixture = Fixture::new("override-decided-case");
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
    let overridden = run_in(
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
    assert_eq!(overridden.status.code(), Some(0), "{overridden:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let decided = run_in(
        &steward,
        &[
            "case",
            "decide",
            CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(decided.status.code(), Some(0), "{decided:?}");
    fs::write(&proposal, early_review_override_proposal())
        .expect("second early-review proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "3",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("already records an accepted reuse decision"),
        "{stderr}"
    );
    assert!(stderr.contains("awaiting verification"), "{stderr}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an override on a decided case must preserve every fixture byte"
    );
}

#[test]
fn watching_case_refuses_decision_until_early_review_override_makes_it_ready() {
    let fixture = Fixture::new("decision-readiness-routes");
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
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let before_refusal = files_beneath(&fixture.root);

    let watching = run_in(
        &steward,
        &[
            "case",
            "decide",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(watching.status.code(), Some(3), "{watching:?}");
    assert!(watching.stdout.is_empty(), "{watching:?}");
    let stderr = String::from_utf8(watching.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("case `00000000-0000-4000-8000-000000000011` is watching"),
        "{stderr}"
    );
    assert!(stderr.contains("third independent occurrence"), "{stderr}");
    assert!(stderr.contains("early-review override"), "{stderr}");
    assert_eq!(
        files_beneath(&fixture.root),
        before_refusal,
        "a decision on a watching case must write nothing"
    );

    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let overridden = run_in(
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
    assert_eq!(overridden.status.code(), Some(0), "{overridden:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let decided = run_in(
        &steward,
        &[
            "case",
            "decide",
            CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(decided.status.code(), Some(0), "{decided:?}");
    let stdout = String::from_utf8(decided.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("state: awaiting-verification\n"),
        "{stdout}"
    );
}

#[test]
fn reuse_decision_on_unknown_case_refuses_and_creates_nothing() {
    let fixture = Fixture::new("decision-unknown-case");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let proposal = fixture.proposal(&change_decision_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
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
            "refusal: case identity `{SECOND_CASE_ID}` is not stewarded by repository `{STEWARD_ID}`\nresolution: run `case list` in this steward repository and retry `case decide` with a recorded review-ready case identity\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an unknown decision target must not create a case or event"
    );
}

#[test]
fn reuse_decision_with_unrecorded_affected_consumer_refuses_without_writes() {
    let fixture = Fixture::new("decision-unrecorded-consumer");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(
        &proposal,
        change_decision_proposal().replacen(
            "consumer = \"rust-release-tool\"",
            "consumer = \"unevidenced-consumer\"",
            1,
        ),
    )
    .expect("decision proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
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
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("unevidenced-consumer"), "{stderr}");
    assert!(stderr.contains("not recorded by case"), "{stderr}");
    assert!(stderr.contains("recorded occurrence"), "{stderr}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "an unevidenced affected consumer must preserve every fixture byte"
    );
}

#[test]
fn reuse_decision_requires_a_declared_current_revision_without_writes() {
    let fixture = Fixture::new("decision-revision-guard");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let cases = [
        (
            "missing",
            vec![
                "case",
                "decide",
                SECOND_CASE_ID,
                "--proposal",
                proposal_path,
                "--root",
                root,
            ],
            "missing required `--expected-revision`",
        ),
        (
            "stale",
            vec![
                "case",
                "decide",
                SECOND_CASE_ID,
                "--expected-revision",
                "2",
                "--proposal",
                proposal_path,
                "--root",
                root,
            ],
            "expected revision 2 does not match",
        ),
    ];

    for (name, arguments, condition) in cases {
        let before = files_beneath(&fixture.root);
        let output = run_in(&steward, &arguments);
        assert_eq!(output.status.code(), Some(3), "{name}: {output:?}");
        assert!(output.stdout.is_empty(), "{name}: {output:?}");
        let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        assert!(stderr.contains(condition), "{name}: {stderr}");
        assert!(stderr.contains("resolution:"), "{name}: {stderr}");
        assert_eq!(
            files_beneath(&fixture.root),
            before,
            "{name} decision revision must preserve every fixture byte"
        );
    }
}

#[test]
fn occupied_decision_sequence_with_different_event_identity_is_a_revision_conflict() {
    let fixture = Fixture::new("decision-identity-conflict");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let arguments = [
        "case",
        "decide",
        SECOND_CASE_ID,
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
        .expect("preview should expose the exact event");
    let parsed = event
        .parse::<toml::Table>()
        .expect("previewed decision should be valid TOML");
    let recorded_event_id = parsed["event_id"]
        .as_str()
        .expect("event identity should be a string");
    fs::write(&proposal, event).expect("approved event should be writable as the proposal");
    let applied = run_in(&steward, &arguments);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    let event_path = case_event_path_at_sequence(&steward, SECOND_CASE_ID, 2);
    let recorded = fs::read(&event_path).expect("recorded decision should be readable");
    fs::write(
        &proposal,
        event.replacen(recorded_event_id, DIFFERENT_DECISION_EVENT_ID, 1),
    )
    .expect("conflicting prepared event should be writable");
    let before = files_beneath(&fixture.root);

    let conflict = run_in(&steward, &arguments);

    assert_eq!(conflict.status.code(), Some(3), "{conflict:?}");
    assert!(conflict.stdout.is_empty(), "{conflict:?}");
    let stderr = String::from_utf8(conflict.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("revision conflict at sequence 2"),
        "{stderr}"
    );
    assert!(stderr.contains(recorded_event_id), "{stderr}");
    assert!(stderr.contains(DIFFERENT_DECISION_EVENT_ID), "{stderr}");
    assert_eq!(
        fs::read(&event_path).expect("recorded decision should remain readable"),
        recorded,
        "a different decision identity must not replace the recorded event"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a conflicting decision identity must preserve every fixture byte"
    );
}

#[test]
fn second_reuse_decision_against_current_revision_refuses_without_writes() {
    let fixture = Fixture::new("second-decision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    fs::write(&proposal, change_decision_proposal())
        .expect("first decision proposal should be writable");
    let decided = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(decided.status.code(), Some(0), "{decided:?}");
    fs::write(&proposal, no_change_decision_proposal())
        .expect("second decision proposal should be writable");
    let before = files_beneath(&fixture.root);

    let second = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(second.status.code(), Some(3), "{second:?}");
    assert!(second.stdout.is_empty(), "{second:?}");
    let stderr = String::from_utf8(second.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("already records an accepted reuse decision"),
        "{stderr}"
    );
    assert!(stderr.contains("superseding it requires"), "{stderr}");
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a second accepted decision must preserve every fixture byte"
    );
}

#[test]
fn set_visibility_refuses_public_transition_for_recorded_private_cases_without_writes() {
    let fixture = Fixture::new("visibility-private-case-guard");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    for (path, proposal) in [
        (
            fixture.root.join("first-case.toml"),
            two_occurrence_proposal(),
        ),
        (
            fixture.root.join("second-case.toml"),
            two_occurrence_proposal().replace(CASE_ID, SECOND_CASE_ID),
        ),
    ] {
        fs::write(&path, proposal).expect("case proposal should be writable");
        let opened = run_in(
            &steward,
            &[
                "case",
                "open",
                "--proposal",
                path.to_str().expect("fixture path should be UTF-8"),
                "--root",
                root,
            ],
        );
        assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    }
    let configured_home = fixture.root.join("configured");
    fs::create_dir_all(configured_home.join("reuse-evidence"))
        .expect("portfolio configuration directory should be creatable");
    fs::write(
        configured_home.join("reuse-evidence/config.toml"),
        format!("portfolio_roots = [\"{}\"]\n", fixture.root.display()),
    )
    .expect("portfolio configuration should be writable");
    let before = files_beneath(&fixture.root);

    let unconfigured = run_without_portfolio_configuration(
        &fixture,
        &steward,
        &["set-visibility", "--visibility", "public"],
    );
    let configured = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(["set-visibility", "--visibility", "public"])
        .current_dir(&steward)
        .env("XDG_CONFIG_HOME", configured_home)
        .env("XDG_STATE_HOME", fixture.root.join("configured-state"))
        .output()
        .expect("compiled reuse-evidence binary should run");

    for output in [&unconfigured, &configured] {
        assert_eq!(output.status.code(), Some(3), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8"),
            format!(
                "refusal: repository `{STEWARD_ID}` cannot be made public while it stewards private case `{CASE_ID}`\nresolution: keep the repository private while it stewards case `{CASE_ID}`; version 0.1 does not support stewardship transfer\n"
            )
        );
    }
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a private-case visibility refusal must preserve every fixture byte"
    );
}

#[test]
fn set_visibility_allows_public_transition_when_recorded_cases_are_public() {
    let fixture = Fixture::new("visibility-public-case");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
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
    force_marker_visibility(&steward, "public", "private");

    let output = run_in(&steward, &["set-visibility", "--visibility", "public"]);

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .expect("stdout should be UTF-8")
            .starts_with("changed repository visibility\n"),
        "the existing success receipt should be preserved"
    );
    assert!(
        fs::read_to_string(steward.join("reuse-evidence.toml"))
            .expect("changed marker should be readable")
            .contains("visibility = \"public\"\n")
    );
}

#[test]
fn set_visibility_private_transition_ignores_malformed_or_unreadable_case_state() {
    for (name, cases_path_is_file) in [("malformed", false), ("unreadable", true)] {
        let fixture = Fixture::new(&format!("visibility-private-direction-{name}"));
        let steward = fixture.repository("steward", STEWARD_ID, "public");
        let cases_path = steward.join("reuse-evidence/cases");
        if cases_path_is_file {
            fs::create_dir_all(
                cases_path
                    .parent()
                    .expect("cases path should have a parent"),
            )
            .expect("case storage parent should be creatable");
            fs::write(&cases_path, "not a directory")
                .expect("unreadable case fixture should be creatable");
        } else {
            fs::create_dir_all(cases_path.join("not-a-case-id"))
                .expect("malformed case fixture should be creatable");
        }

        let output = run_in(&steward, &["set-visibility", "--visibility", "private"]);

        assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
        assert!(output.stderr.is_empty(), "{name}: {output:?}");
        assert!(
            fs::read_to_string(steward.join("reuse-evidence.toml"))
                .expect("changed marker should be readable")
                .contains("visibility = \"private\"\n"),
            "{name}"
        );
    }
}

#[test]
fn set_visibility_public_noop_ignores_malformed_case_state() {
    let fixture = Fixture::new("visibility-public-noop");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fs::create_dir_all(steward.join("reuse-evidence/cases/not-a-case-id"))
        .expect("malformed case fixture should be creatable");
    let before = files_beneath(&fixture.root);

    let output = run_in(&steward, &["set-visibility", "--visibility", "public"]);

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .expect("stdout should be UTF-8")
            .starts_with("repository visibility unchanged\n")
    );
    assert_eq!(files_beneath(&fixture.root), before);
}

#[test]
fn set_visibility_public_transition_refuses_malformed_or_unreadable_case_state_without_writes() {
    for (name, cases_path_is_file, expected_error) in [
        (
            "malformed",
            false,
            "case directory identity `not-a-case-id` is invalid",
        ),
        ("unreadable", true, "steward-local case directory"),
    ] {
        let fixture = Fixture::new(&format!("visibility-public-direction-{name}"));
        let steward = fixture.repository("steward", STEWARD_ID, "private");
        let cases_path = steward.join("reuse-evidence/cases");
        if cases_path_is_file {
            fs::create_dir_all(
                cases_path
                    .parent()
                    .expect("cases path should have a parent"),
            )
            .expect("case storage parent should be creatable");
            fs::write(&cases_path, "not a directory")
                .expect("unreadable case fixture should be creatable");
        } else {
            fs::create_dir_all(cases_path.join("not-a-case-id"))
                .expect("malformed case fixture should be creatable");
        }
        let before = files_beneath(&fixture.root);

        let output = run_in(&steward, &["set-visibility", "--visibility", "public"]);

        assert_eq!(output.status.code(), Some(3), "{name}: {output:?}");
        assert!(output.stdout.is_empty(), "{name}: {output:?}");
        assert!(
            String::from_utf8(output.stderr)
                .expect("stderr should be UTF-8")
                .contains(expected_error),
            "the existing steward-local case-read refusal should propagate: {name}"
        );
        assert_eq!(
            files_beneath(&fixture.root),
            before,
            "a case-read visibility refusal must preserve every fixture byte: {name}"
        );
    }
}

#[test]
fn currently_public_steward_refuses_a_private_case_decision_without_writes() {
    let fixture = Fixture::new("public-steward-private-decision");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let proposal = fixture.proposal(&three_occurrence_proposal());
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
    let root = fixture.root.to_str().expect("fixture path should be UTF-8");
    let opened = run_in(
        &steward,
        &["case", "open", "--proposal", proposal_path, "--root", root],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    force_marker_visibility(&steward, "private", "public");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(
        &steward,
        &[
            "case",
            "decide",
            SECOND_CASE_ID,
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
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("public steward"), "{stderr}");
    assert!(stderr.contains("private case"), "{stderr}");
    assert!(
        stderr.contains("set-visibility --visibility private"),
        "{stderr}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "a public steward must not mutate a private case decision"
    );
}

#[test]
fn appending_an_occurrence_after_a_decision_preserves_awaiting_verification() {
    let fixture = Fixture::new("append-after-decision");
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
    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let overridden = run_in(
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
    assert_eq!(overridden.status.code(), Some(0), "{overridden:?}");
    fs::write(&proposal, change_decision_proposal()).expect("decision proposal should be writable");
    let decided = run_in(
        &steward,
        &[
            "case",
            "decide",
            CASE_ID,
            "--expected-revision",
            "2",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );
    assert_eq!(decided.status.code(), Some(0), "{decided:?}");
    fs::write(&proposal, append_occurrence_proposal()).expect("append proposal should be writable");

    let appended = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "3",
            "--proposal",
            proposal_path,
            "--root",
            root,
        ],
    );

    assert_eq!(appended.status.code(), Some(0), "{appended:?}");
    assert!(appended.stderr.is_empty(), "{appended:?}");
    let stdout = String::from_utf8(appended.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("revision: 4\nstate: awaiting-verification\nprivacy: private\n"),
        "{stdout}"
    );
    assert!(!stdout.contains("readiness_basis:"), "{stdout}");
    assert!(!stdout.contains("authorizes semantic review"), "{stdout}");
    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    let shown = String::from_utf8(shown.stdout).expect("stdout should be UTF-8");
    assert!(
        shown.contains("revision: 4\noccurrence_count: 3\nstate: awaiting-verification\n"),
        "{shown}"
    );
}

#[test]
fn competing_decision_and_append_against_one_revision_publish_exactly_one_event() {
    let fixture = Fixture::new("decision-append-concurrency");
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
    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let overridden = run_in(
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
    assert_eq!(overridden.status.code(), Some(0), "{overridden:?}");
    let decision_proposal = fixture.root.join("decision.toml");
    fs::write(&decision_proposal, change_decision_proposal())
        .expect("decision proposal should be writable");
    let append_proposal = fixture.root.join("append.toml");
    fs::write(&append_proposal, append_occurrence_proposal())
        .expect("append proposal should be writable");
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

    let (mut decision, mut append) = spawn_competing_decision_and_append_writers(
        &steward,
        root,
        &decision_proposal,
        &append_proposal,
    );
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        decision
            .try_wait()
            .expect("decision status should be readable")
            .is_none(),
        "decision must wait while another process holds the case write lock"
    );
    assert!(
        append
            .try_wait()
            .expect("append status should be readable")
            .is_none(),
        "append must wait while another process holds the case write lock"
    );
    drop(opening);

    let decision = decision
        .wait_with_output()
        .expect("decision process should finish");
    let append = append
        .wait_with_output()
        .expect("append process should finish");
    let status_codes = [decision.status.code(), append.status.code()];
    assert_eq!(
        status_codes.iter().filter(|code| **code == Some(0)).count(),
        1,
        "exactly one same-revision writer should publish: decision={decision:?}, append={append:?}"
    );
    assert_eq!(
        status_codes.iter().filter(|code| **code == Some(3)).count(),
        1,
        "the competing writer should refuse: decision={decision:?}, append={append:?}"
    );
    let case_directory = steward.join("reuse-evidence/cases").join(CASE_ID);
    let sequence_three = fs::read_dir(&case_directory)
        .expect("case should remain readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("0003-"))
        .count();
    assert_eq!(sequence_three, 1, "only one sequence-three event may exist");
    let shown = run_without_portfolio_configuration(&fixture, &steward, &["case", "show", CASE_ID]);
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    assert!(
        String::from_utf8(shown.stdout)
            .expect("stdout should be UTF-8")
            .contains("revision: 3\n"),
        "the winning event should be the sole new revision"
    );
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
fn case_read_reports_privacy_conflict_when_public_steward_holds_recorded_private_case() {
    let fixture = Fixture::new("recorded-privacy-conflict");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
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
    let opened_stdout = String::from_utf8(opened.stdout).expect("stdout should be UTF-8");
    assert!(
        opened_stdout.contains("privacy: private\n"),
        "a private steward must record private case privacy: {opened_stdout}"
    );

    force_marker_visibility(&steward, "private", "public");
    let append_proposal = fixture.root.join("append-occurrence.toml");
    fs::write(&append_proposal, append_occurrence_proposal())
        .expect("append proposal should be writable");
    let before_read = files_beneath(&fixture.root);

    let shown = run_in(&steward, &["case", "show", CASE_ID, "--root", root]);
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    assert!(shown.stderr.is_empty(), "{shown:?}");
    let shown_stdout = String::from_utf8(shown.stdout).expect("stdout should be UTF-8");
    assert!(
        shown_stdout.contains("privacy_conflicted: true\nstale: false\n"),
        "{shown_stdout}"
    );

    let listed = run_in(&steward, &["case", "list", "--root", root]);
    assert_eq!(listed.status.code(), Some(0), "{listed:?}");
    let listed_stdout = String::from_utf8(listed.stdout).expect("stdout should be UTF-8");
    assert!(
        listed_stdout.contains("  privacy_conflicted: true\n  stale: false\n"),
        "{listed_stdout}"
    );

    // The write path is the independent authority on the answer: it refuses this exact case for
    // this exact conflict, so a projection reporting no conflict would be a second answer.
    let refused = run_in(
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            append_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--preview",
            "--root",
            root,
        ],
    );
    assert_eq!(refused.status.code(), Some(3), "{refused:?}");
    let refusal = String::from_utf8(refused.stderr).expect("stderr should be UTF-8");
    assert!(
        refusal.contains(&format!(
            "public steward `{STEWARD_ID}` cannot append to private case `{CASE_ID}`"
        )),
        "{refusal}"
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
        "--root",
        root,
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
fn exact_early_review_retry_without_portfolio_reports_unknown_privacy_without_writes() {
    let fixture = Fixture::new("early-review-retry-without-portfolio");
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
    let proposal = fixture.root.join("early-review.toml");
    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
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
        "--preview",
    ];
    let preview = run_in(&steward, &arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain the exact event");
    fs::write(&proposal, event).expect("prepared early-review event should be writable");
    let applied = run_in(&steward, &arguments[..arguments.len() - 1]);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    let before_retry = files_beneath(&fixture.root);
    let event_path = steward
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0002-early-review-authorized.toml");
    let _write_protection = WriteProtection::for_file_and_parent(&event_path);

    let retry = run_without_portfolio_configuration(
        &fixture,
        &steward,
        &[
            "case",
            "override",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
        ],
    );

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "early review already authorized\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-early-review-authorized.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: unknown\nportfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "an exact retry without portfolio configuration must write nothing"
    );
}

#[test]
fn exact_early_review_retry_with_unresolvable_participant_reports_unknown_privacy_without_writes() {
    let fixture = Fixture::new("early-review-retry-unresolvable-participant");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    let second_consumer = fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
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
    let proposal = fixture.root.join("early-review.toml");
    fs::write(&proposal, early_review_override_proposal())
        .expect("early-review proposal should be writable");
    let proposal_path = proposal.to_str().expect("fixture path should be UTF-8");
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
        "--preview",
    ];
    let preview = run_in(&steward, &arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain the exact event");
    fs::write(&proposal, event).expect("prepared early-review event should be writable");
    let applied = run_in(&steward, &arguments[..arguments.len() - 1]);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    fs::remove_file(second_consumer.join("reuse-evidence.toml"))
        .expect("recorded participant should become un-enrolled");
    let before_retry = files_beneath(&fixture.root);
    let event_path = steward
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0002-early-review-authorized.toml");
    let _write_protection = WriteProtection::for_file_and_parent(&event_path);

    let retry = run_in(&steward, &arguments[..arguments.len() - 1]);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "early review already authorized\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-early-review-authorized.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: unknown\nportfolio conditions unavailable: a recorded participant does not resolve to exactly one enrolled repository beneath the selected portfolio roots; restore its enrollment and unique repository identity to derive privacy\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "an exact retry whose participants cannot be resolved must write nothing"
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
    force_marker_visibility(&steward, "private", "public");
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
    force_marker_visibility(&steward, "private", "public");
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
    force_marker_visibility(&steward, "private", "public");

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
fn exact_append_retry_without_portfolio_reports_unknown_privacy_without_writes() {
    let fixture = Fixture::new("append-retry-without-portfolio");
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
        "--preview",
    ];
    let preview = run_in(&steward, &arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain the exact event");
    fs::write(&proposal, event).expect("prepared append event should be writable");
    let applied = run_in(&steward, &arguments[..arguments.len() - 1]);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    let before_retry = files_beneath(&fixture.root);
    let event_path = steward
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0002-occurrence-appended.toml");
    let _write_protection = WriteProtection::for_file_and_parent(&event_path);

    let retry = run_without_portfolio_configuration(
        &fixture,
        &steward,
        &[
            "case",
            "append",
            CASE_ID,
            "--expected-revision",
            "1",
            "--proposal",
            proposal_path,
        ],
    );

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "occurrence already recorded\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-occurrence-appended.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: occurrence-count\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: unknown\nportfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "an exact retry without portfolio configuration must write nothing"
    );
}

#[test]
fn exact_append_retry_with_unresolvable_participant_reports_unknown_privacy_without_writes() {
    let fixture = Fixture::new("append-retry-unresolvable-participant");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    let second_consumer = fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
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
        "--preview",
    ];
    let preview = run_in(&steward, &arguments);
    assert_eq!(preview.status.code(), Some(0), "{preview:?}");
    let preview = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    let (_, event) = preview
        .split_once("event:\n")
        .expect("preview should contain the exact event");
    fs::write(&proposal, event).expect("prepared append event should be writable");
    let applied = run_in(&steward, &arguments[..arguments.len() - 1]);
    assert_eq!(applied.status.code(), Some(0), "{applied:?}");
    fs::remove_file(second_consumer.join("reuse-evidence.toml"))
        .expect("recorded participant should become un-enrolled");
    let before_retry = files_beneath(&fixture.root);
    let event_path = steward
        .join("reuse-evidence/cases")
        .join(CASE_ID)
        .join("0002-occurrence-appended.toml");
    let _write_protection = WriteProtection::for_file_and_parent(&event_path);

    let retry = run_in(&steward, &arguments[..arguments.len() - 1]);

    assert_eq!(retry.status.code(), Some(0), "{retry:?}");
    assert!(retry.stderr.is_empty(), "{retry:?}");
    assert_eq!(
        String::from_utf8(retry.stdout).expect("stdout should be UTF-8"),
        format!(
            "occurrence already recorded\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/0002-occurrence-appended.toml\nrevision: 2\nstate: review-ready\nreadiness_basis: occurrence-count\nreadiness: authorizes semantic review; does not authorize extraction\nprivacy: unknown\nportfolio conditions unavailable: a recorded participant does not resolve to exactly one enrolled repository beneath the selected portfolio roots; restore its enrollment and unique repository identity to derive privacy\n"
        )
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before_retry,
        "an exact retry whose participants cannot be resolved must write nothing"
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
fn review_r1_spec_1_staged_temporary_policy_remains_context_specific() {
    let unopened_fixture = Fixture::new("later-staging-before-open");
    let unopened_steward = unopened_fixture.repository("steward", STEWARD_ID, "private");
    unopened_fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    unopened_fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let unopened_proposal = unopened_fixture.proposal(&two_occurrence_proposal());
    let unopened_case_directory = unopened_steward.join("reuse-evidence/cases").join(CASE_ID);
    fs::create_dir_all(&unopened_case_directory)
        .expect("interrupted case directory should be reproducible");
    fs::write(
        unopened_case_directory
            .join(".0002-occurrence-appended.toml.00000000-0000-4000-8000-000000000088.tmp"),
        "interrupted later event\n",
    )
    .expect("interrupted later-event staging should be reproducible");
    let unopened_before = files_beneath(&unopened_fixture.root);
    let unopened = run_in(
        &unopened_steward,
        &[
            "case",
            "open",
            "--proposal",
            unopened_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            unopened_fixture
                .root
                .to_str()
                .expect("fixture path should be UTF-8"),
        ],
    );

    let opened_fixture = Fixture::new("opening-staging-during-read");
    let opened_steward = opened_fixture.repository("steward", STEWARD_ID, "private");
    opened_fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    opened_fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "private");
    let opened_proposal = opened_fixture.proposal(&two_occurrence_proposal());
    let opened = run_in(
        &opened_steward,
        &[
            "case",
            "open",
            "--proposal",
            opened_proposal
                .to_str()
                .expect("fixture path should be UTF-8"),
            "--root",
            opened_fixture
                .root
                .to_str()
                .expect("fixture path should be UTF-8"),
        ],
    );
    assert_eq!(opened.status.code(), Some(0), "{opened:?}");
    let opened_case_directory = opened_steward.join("reuse-evidence/cases").join(CASE_ID);
    fs::write(
        opened_case_directory
            .join(".0001-case-opened.toml.00000000-0000-4000-8000-000000000088.tmp"),
        "interrupted opening event\n",
    )
    .expect("interrupted opening-event staging should be reproducible");
    let opened_before = files_beneath(&opened_fixture.root);
    let shown = run_without_portfolio_configuration(
        &opened_fixture,
        &opened_steward,
        &["case", "show", CASE_ID],
    );

    assert_eq!(
        (unopened.status.code(), shown.status.code()),
        (Some(3), Some(3)),
        "opening must reject later-event staging and reading must reject opening-event staging: unopened={unopened:?}, shown={shown:?}"
    );
    assert!(unopened.stdout.is_empty(), "{unopened:?}");
    assert!(shown.stdout.is_empty(), "{shown:?}");
    assert!(
        String::from_utf8(unopened.stderr)
            .expect("stderr should be UTF-8")
            .contains("already has unrecognized content"),
    );
    assert!(
        String::from_utf8(shown.stderr)
            .expect("stderr should be UTF-8")
            .contains("contains unrecognized event file"),
    );
    assert_eq!(
        files_beneath(&unopened_fixture.root),
        unopened_before,
        "refusal before opening must preserve later-event staging"
    );
    assert_eq!(
        files_beneath(&opened_fixture.root),
        opened_before,
        "read refusal must preserve opening-event staging"
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

#[test]
fn prepared_append_event_with_an_unrecordable_timestamp_is_refused_without_writes() {
    let fixture = Fixture::new("prepared-append-timestamp");
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
    let recorded_at = event
        .lines()
        .find(|line| line.starts_with("recorded_at = "))
        .expect("the previewed event should record its instant");

    for (substitute, condition) in [
        ("2023-02-29T00:00:00Z", "is not a valid UTC instant"),
        ("2024-01-01T00:00:00", "is not UTC RFC 3339"),
    ] {
        fs::write(
            &proposal,
            event.replace(recorded_at, &format!("recorded_at = \"{substitute}\"")),
        )
        .expect("prepared append event should be writable");
        let before_refusal = files_beneath(&fixture.root);

        let refused = run_in(&steward, &arguments);

        assert_eq!(refused.status.code(), Some(3), "{refused:?}");
        assert!(refused.stdout.is_empty(), "{refused:?}");
        assert_eq!(
            String::from_utf8(refused.stderr).expect("stderr should be UTF-8"),
            format!(
                "refusal: prepared append event timestamp `{substitute}` {condition}\nresolution: use the exact event rendered by `case append --preview`\n"
            )
        );
        assert_eq!(
            files_beneath(&fixture.root),
            before_refusal,
            "a refused prepared append must write nothing"
        );
    }
}
