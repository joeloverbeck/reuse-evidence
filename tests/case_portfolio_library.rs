//! In-process coverage for the cross-portfolio case query.
//!
//! ADR 0016 assigns new case behavior to the public module interface. The
//! process boundary receives one separate test for the terminal facts newly
//! introduced by the CLI command.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use reuse_evidence::ExitMeaning;
use reuse_evidence::case::{self, RecordedInstant};
use reuse_evidence::portfolio::PortfolioLocation;
use support::Fixture;

const CASE_ID: &str = "00000000-0000-4000-8000-000000000021";
const STEWARD_ID: &str = "00000000-0000-4000-8000-000000000022";
const FIRST_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000023";
const SECOND_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000024";
const WORKING_REPOSITORY_ID: &str = "00000000-0000-4000-8000-000000000025";
const HEALTHY_CASE_ID: &str = "00000000-0000-4000-8000-000000000031";
const HEALTHY_STEWARD_ID: &str = "00000000-0000-4000-8000-000000000032";
const PINNED_UNIX_SECONDS: i64 = 1_760_000_000;

impl Fixture {
    fn new(name: &str) -> Self {
        Self::from_label(&format!("case-portfolio-library-{name}"))
    }

    fn repository(&self, name: &str, repository_id: &str, visibility: &str) -> PathBuf {
        self.enrolled_repository(name, repository_id, visibility)
    }

    fn location(&self, state_directory: &Path) -> PortfolioLocation {
        PortfolioLocation::from_user_directories(
            vec![self.root.to_path_buf()],
            Some(&self.root.join("config")),
            Some(state_directory),
        )
    }
}

fn opening_proposal(case_id: &str) -> String {
    format!(
        "case_id = \"{case_id}\"\nresponsibility = \"normalize durable event identities\"\n\n[[occurrences]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust release tool\"\nindependence = \"separate release lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\n\n[[occurrences]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web deployment tool\"\nindependence = \"independent npm workspace and owner\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"2222222\"\n"
    )
}

#[test]
fn portfolio_case_query_finds_a_case_from_another_enrolled_repository_without_writing() {
    let fixture = Fixture::new("cross-repository");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-participant", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-participant", SECOND_PARTICIPANT_ID, "public");
    let working_repository =
        fixture.repository("working-repository", WORKING_REPOSITORY_ID, "public");

    let unmarked = support::git_repository(&fixture.root, "unmarked");
    let ignored_case = unmarked.join("reuse-evidence/cases/not-a-case-identity");
    fs::create_dir_all(&ignored_case).expect("ignored case fixture should be creatable");
    fs::write(
        ignored_case.join("not-an-event"),
        b"not recorded evidence\n",
    )
    .expect("ignored case fixture should be writable");

    let proposal = fixture.root.join("opening.toml");
    fs::write(&proposal, opening_proposal(CASE_ID)).expect("opening proposal should be writable");
    case::open(
        &steward,
        &proposal,
        &fixture.location(&fixture.root.join("unused-state")),
        RecordedInstant::from_unix_seconds(PINNED_UNIX_SECONDS)
            .expect("the pinned instant should be recordable"),
        false,
    )
    .expect("the steward should open the fixture case");

    let state_directory = fixture.root.join("state");
    let state_file = state_directory.join("reuse-evidence/portfolio.toml");
    fs::create_dir_all(
        state_file
            .parent()
            .expect("state file should have a parent"),
    )
    .expect("state directory should be creatable");
    fs::write(&state_file, b"sentinel portfolio state\n")
        .expect("state sentinel should be writable");
    let location = fixture.location(&state_directory);
    let before = support::snapshot(&fixture.root);

    let outcome = case::find(&location).expect("the portfolio case query should succeed");

    assert_eq!(
        outcome.to_string(),
        format!(
            "portfolio cases\n- case_id: {CASE_ID}\n  steward_repository_id: {STEWARD_ID}\n  steward_path: {}\n  responsibility: normalize durable event identities\n  revision: 1\n  state: watching\n  privacy: private\n",
            steward
                .canonicalize()
                .expect("the steward path should be canonical")
                .display()
        )
    );
    assert!(
        working_repository.join("reuse-evidence.toml").is_file(),
        "the query is being exercised with another enrolled repository as its working context"
    );
    assert_eq!(
        support::snapshot(&fixture.root),
        before,
        "the query must preserve every inspected repository byte and user-local state byte"
    );
}

#[test]
fn review_r1_spec_1_portfolio_case_query_reports_current_complete_privacy() {
    let fixture = Fixture::new("current-complete-privacy");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-participant", FIRST_PARTICIPANT_ID, "public");
    let second_participant =
        fixture.repository("second-participant", SECOND_PARTICIPANT_ID, "public");
    let location = fixture.location(&fixture.root.join("state"));
    let proposal = fixture.root.join("opening.toml");
    fs::write(&proposal, opening_proposal(CASE_ID)).expect("opening proposal should be writable");
    case::open(
        &steward,
        &proposal,
        &location,
        RecordedInstant::from_unix_seconds(PINNED_UNIX_SECONDS)
            .expect("the pinned instant should be recordable"),
        false,
    )
    .expect("the public portfolio should open a public case");

    support::enrollment_marker(&second_participant, SECOND_PARTICIPANT_ID, "private");
    let before = support::snapshot(&fixture.root);

    let rendered = case::find(&location)
        .expect("the portfolio case query should derive current privacy")
        .to_string();

    assert!(
        rendered.ends_with("  privacy: private\n"),
        "a currently private participant must make complete case privacy private: {rendered}"
    );
    assert!(
        !rendered.ends_with("  privacy: public\n"),
        "opening-recorded privacy must not masquerade as current complete privacy: {rendered}"
    );
    assert_eq!(
        support::snapshot(&fixture.root),
        before,
        "deriving current complete privacy must remain byte-for-byte write-free"
    );
}

#[test]
fn review_r2_spec_1_portfolio_case_query_treats_unresolved_visibility_as_private() {
    let fixture = Fixture::new("unresolved-visibility");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-participant", FIRST_PARTICIPANT_ID, "public");
    let second_participant =
        fixture.repository("second-participant", SECOND_PARTICIPANT_ID, "public");
    let location = fixture.location(&fixture.root.join("state"));
    let proposal = fixture.root.join("opening.toml");
    fs::write(&proposal, opening_proposal(CASE_ID)).expect("opening proposal should be writable");
    case::open(
        &steward,
        &proposal,
        &location,
        RecordedInstant::from_unix_seconds(PINNED_UNIX_SECONDS)
            .expect("the pinned instant should be recordable"),
        false,
    )
    .expect("the public portfolio should open a public case");

    fs::remove_file(second_participant.join("reuse-evidence.toml"))
        .expect("the fixture participant should become unresolved");
    let before = support::snapshot(&fixture.root);

    let rendered = case::find(&location)
        .expect("uncertain visibility should produce a conservative result")
        .to_string();

    assert!(
        rendered.ends_with("  privacy: private\n"),
        "unresolved participant visibility must conservatively make the case private: {rendered}"
    );
    assert_eq!(
        support::snapshot(&fixture.root),
        before,
        "conservative privacy derivation must remain byte-for-byte write-free"
    );
}

#[test]
fn portfolio_case_query_refuses_without_a_root_selection() {
    let fixture = Fixture::new("no-roots");
    let configuration_directory = fixture.root.join("config");
    let location = PortfolioLocation::from_user_directories(
        Vec::new(),
        Some(&configuration_directory),
        Some(&fixture.root.join("state")),
    );
    let before = support::snapshot(&fixture.root);

    let Err(failure) = case::find(&location) else {
        panic!("a portfolio-wide query without selected roots must refuse");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal);
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: no portfolio roots were supplied and `{}` does not configure any\nresolution: add `portfolio_roots = [\"/path/to/root\"]` to that user-local configuration or rerun with `--root <PATH>`",
            configuration_directory
                .join("reuse-evidence/config.toml")
                .display()
        )
    );
    assert_eq!(
        support::snapshot(&fixture.root),
        before,
        "the refusal must create no state, cache, projection, or repository file"
    );
}

#[test]
fn portfolio_case_query_reports_damaged_history_without_plausible_case_fields() {
    let fixture = Fixture::new("damaged-history");
    let damaged_steward = fixture.repository("damaged-steward", STEWARD_ID, "private");
    let healthy_steward = fixture.repository("healthy-steward", HEALTHY_STEWARD_ID, "public");
    fixture.repository("first-participant", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-participant", SECOND_PARTICIPANT_ID, "public");
    let location = fixture.location(&fixture.root.join("state"));
    let recorded_at = RecordedInstant::from_unix_seconds(PINNED_UNIX_SECONDS)
        .expect("the pinned instant should be recordable");

    let damaged_proposal = fixture.root.join("damaged-opening.toml");
    fs::write(&damaged_proposal, opening_proposal(CASE_ID))
        .expect("damaged opening proposal should be writable");
    case::open(
        &damaged_steward,
        &damaged_proposal,
        &location,
        recorded_at,
        false,
    )
    .expect("the damaged case should first be opened validly");
    fs::write(
        damaged_steward
            .join("reuse-evidence/cases")
            .join(CASE_ID)
            .join(support::OCCURRENCE_APPENDED_AT_3),
        b"damaged event bytes\n",
    )
    .expect("the fixture should create a gap in recorded history");

    let healthy_proposal = fixture.root.join("healthy-opening.toml");
    fs::write(&healthy_proposal, opening_proposal(HEALTHY_CASE_ID))
        .expect("healthy opening proposal should be writable");
    case::open(
        &healthy_steward,
        &healthy_proposal,
        &location,
        recorded_at,
        false,
    )
    .expect("the healthy case should open");
    let before = support::snapshot(&fixture.root);

    let outcome = case::find(&location)
        .expect("one damaged case should be reported without hiding healthy portfolio cases");
    let rendered = outcome.to_string();

    assert!(
        rendered.contains(&format!(
            "- case_id: {CASE_ID}\n  steward_repository_id: {STEWARD_ID}\n  steward_path: {}\n  condition: damaged-recorded-event-history\n  responsibility: unavailable\n  revision: unavailable\n  state: unavailable\n  privacy: unknown\n  detail:\n    refusal: case `{CASE_ID}` is missing sequence number 2 before recorded sequence 3\n    resolution: restore event file sequence 2 so the case stream is contiguous before reading it\n",
            damaged_steward
                .canonicalize()
                .expect("the damaged steward path should be canonical")
                .display()
        )),
        "damaged history must be explicit and must not look like a derived case: {rendered}"
    );
    assert!(
        rendered.contains(&format!(
            "- case_id: {HEALTHY_CASE_ID}\n  steward_repository_id: {HEALTHY_STEWARD_ID}\n  steward_path: {}\n  responsibility: normalize durable event identities\n  revision: 1\n  state: watching\n  privacy: public\n",
            healthy_steward
                .canonicalize()
                .expect("the healthy steward path should be canonical")
                .display()
        )),
        "a damaged neighbour must not hide a healthy case: {rendered}"
    );
    assert_eq!(
        support::snapshot(&fixture.root),
        before,
        "reporting damaged history must still be byte-for-byte write-free"
    );
}
