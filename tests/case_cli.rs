use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CASE_ID: &str = "00000000-0000-4000-8000-000000000011";
const STEWARD_ID: &str = "00000000-0000-4000-8000-000000000012";
const FIRST_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000013";
const SECOND_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000014";

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
