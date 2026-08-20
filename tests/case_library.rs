//! In-process coverage for case behaviour the terminal contract states only indirectly.
//!
//! `tests/case_cli.rs` keeps the terminal contract: argv dispatch, the exit status each
//! `ExitMeaning` maps to, and which stream carries the text. It reaches that contract through a
//! process, which is the right instrument for it and the wrong one for everything else. What is
//! asserted here is what a process states only indirectly:
//!
//! - A terminal *meaning* is observable at the boundary only through its exit status, and status
//!   `1` covers every non-refusal failure. Asserting `ExitMeaning` directly is exact. Doing so
//!   showed that one unreadable marker meant two different things to two surfaces. ADR 0018
//!   resolved that: every marker fault is a refusal, and both surfaces now name which fault it is.
//! - Two writers competing for one revision needed two child processes and a timed sleep. Two
//!   threads and a barrier state the same race exactly, because the marker lock is held per open
//!   file description and so contends within one process.
//! - `recorded_at` is a parameter of every case command, but the binary always supplies
//!   `RecordedInstant::now()` and exposes no flag, so a test at the boundary can only recover the
//!   value the run happened to choose.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Barrier;

use reuse_evidence::case::{self, RecordedInstant};
use reuse_evidence::portfolio::PortfolioLocation;
use reuse_evidence::{ExitMeaning, TerminalFailure, Visibility, enroll};
use support::{Fixture, snapshot};

const CASE_ID: &str = "00000000-0000-4000-8000-000000000011";
const STEWARD_ID: &str = "00000000-0000-4000-8000-000000000012";
const FIRST_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000013";
const SECOND_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000014";
const THIRD_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000015";
const FOURTH_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000016";

/// A fixed instant inside the four-digit-year range `recorded_at` accepts.
const PINNED_UNIX_SECONDS: i64 = 1_760_000_000;
const PINNED_RECORDED_AT: &str = "2025-10-09T08:53:20Z";

impl Fixture {
    fn new(name: &str) -> Self {
        Self::from_label(&format!("case-library-{name}"))
    }

    fn repository(&self, name: &str, repository_id: &str, visibility: &str) -> PathBuf {
        self.enrolled_repository(name, repository_id, visibility)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("fixture file should be writable");
        path
    }

    fn location(&self) -> PortfolioLocation {
        PortfolioLocation::from_user_directories(vec![self.root.to_path_buf()], None, None)
    }
}

fn pinned() -> RecordedInstant {
    RecordedInstant::from_unix_seconds(PINNED_UNIX_SECONDS)
        .expect("the pinned instant should be recordable")
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

fn three_occurrence_proposal() -> String {
    format!(
        "case_id = \"{CASE_ID}\"\nresponsibility = \"normalize durable event identities\"\n\n[[occurrences]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nindependence = \"separate release lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\npath = \"src/event.rs\"\n\n[[occurrences]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\nindependence = \"independent npm workspace and owner\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"2222222\"\npath = \"packages/events/src/id.ts\"\n\n[[occurrences]]\nrepository_id = \"{THIRD_PARTICIPANT_ID}\"\nconsumer = \"desktop-packager\"\nindependence = \"separate distribution contract\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"3333333\"\npath = \"src/package.rs\"\n"
    )
}

fn change_decision_proposal() -> String {
    format!(
        "identity_verdict = \"same_responsibility\"\naction = \"publish_public_package\"\naccepted_scope = \"the durable event identity contract\"\nnon_responsibilities = [\"case lifecycle storage\"]\ncompatibility_consequences = \"preserve the existing event identity spelling\"\nverification_conditions = [\"all named consumers pass their public contract tests\"]\ninvariant_contract = \"one opaque UUID identifies one immutable event\"\nrequired_consumer_level_tests = [\"each consumer round-trips an event identity\"]\nrollback_or_resplitting_path = \"restore consumer-local implementations without rewriting recorded evidence\"\n\n[[affected_consumers]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\nexpectation = \"migrate after the package publishes\"\n\n[[affected_consumers]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\nexpectation = \"retain its language-specific adapter\"\n\n[[alternatives_rejected]]\nalternative = \"retain intentional duplication\"\nreason = \"coordinated fixes already cross the consumer boundary\"\n\n[[existing_packages_considered]]\npackage = \"uuid\"\nfit = \"supplies identifiers but not the event contract\"\nreason = \"the invariant remains portfolio-owned\"\n\n[[migration_expectations]]\norder = 1\nexpectation = \"publish the invariant contract and its tests\"\n"
    )
}

fn closed_verification_proposal() -> String {
    format!(
        "disposition = \"closed\"\n\n[[condition_results]]\ncondition = \"all named consumers pass their public contract tests\"\noutcome = \"met\"\n\n[[condition_results.evidence]]\nkind = \"commit\"\nreference = \"4444444\"\npath = \"tests/contract.rs\"\n\n[[consumer_results]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\noutcome = \"met\"\n\n[[consumer_results.evidence]]\nkind = \"commit\"\nreference = \"5555555\"\n\n[[consumer_results]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\noutcome = \"accepted_exception\"\nexception = \"the accepted decision retained this language-specific adapter\"\n"
    )
}

fn unsuccessful_verification_proposal(disposition: &str) -> String {
    format!(
        "disposition = \"{disposition}\"\n\n[[condition_results]]\ncondition = \"all named consumers pass their public contract tests\"\noutcome = \"not_met\"\n\n[[condition_results.evidence]]\nkind = \"commit\"\nreference = \"8888888\"\npath = \"tests/failing-contract.rs\"\n\n[[consumer_results]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\noutcome = \"not_met\"\n\n[[consumer_results.evidence]]\nkind = \"commit\"\nreference = \"9999999\"\n\n[[consumer_results]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\noutcome = \"accepted_exception\"\nexception = \"the accepted decision retained this language-specific adapter\"\n"
    )
}

fn fourth_occurrence_proposal() -> String {
    format!(
        "[occurrence]\nrepository_id = \"{FOURTH_PARTICIPANT_ID}\"\nconsumer = \"mobile-packager\"\nindependence = \"separate mobile release contract\"\n\n[[occurrence.evidence]]\nkind = \"commit\"\nreference = \"6666666\"\npath = \"src/mobile.rs\"\n"
    )
}

fn verification_without_condition_result() -> String {
    format!(
        "disposition = \"closed\"\ncondition_results = []\n\n[[consumer_results]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\noutcome = \"met\"\n\n[[consumer_results.evidence]]\nkind = \"commit\"\nreference = \"5555555\"\n\n[[consumer_results]]\nrepository_id = \"{SECOND_PARTICIPANT_ID}\"\nconsumer = \"web-deployment-tool\"\noutcome = \"accepted_exception\"\nexception = \"the accepted decision retained this language-specific adapter\"\n"
    )
}

fn verification_without_consumer_result() -> String {
    "disposition = \"closed\"\nconsumer_results = []\n\n[[condition_results]]\ncondition = \"all named consumers pass their public contract tests\"\noutcome = \"met\"\n\n[[condition_results.evidence]]\nkind = \"commit\"\nreference = \"4444444\"\npath = \"tests/contract.rs\"\n"
        .to_owned()
}

/// Enrolls a private steward and two public participants, then opens one case at the pinned
/// instant. Returns the steward repository.
fn steward_with_one_open_case(fixture: &Fixture) -> PathBuf {
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");

    let proposal = fixture.write("open-case.toml", &two_occurrence_proposal());
    case::open(&steward, &proposal, &fixture.location(), pinned(), false)
        .expect("opening a case under an explicit root should succeed");
    steward
}

/// Enrolls a private steward and three public participants, then opens a review-ready case.
fn steward_with_review_ready_case(fixture: &Fixture) -> PathBuf {
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    fixture.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");

    let opening = fixture.write("open-case.toml", &three_occurrence_proposal());
    case::open(&steward, &opening, &fixture.location(), pinned(), false)
        .expect("opening a three-occurrence case should succeed");
    steward
}

/// Opens a review-ready case and records its accepted decision. Returns revision 2.
fn steward_with_decided_case(fixture: &Fixture) -> PathBuf {
    let steward = steward_with_review_ready_case(fixture);
    let decision = fixture.write("decision.toml", &change_decision_proposal());
    case::decide(
        &steward,
        CASE_ID,
        1,
        &decision,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("recording the accepted decision should succeed");
    steward
}

fn case_directory(steward: &Path) -> PathBuf {
    steward.join("reuse-evidence/cases").join(CASE_ID)
}

/// Replaces `repository`'s marker with a directory, so reading it fails in the I/O layer rather
/// than in the parser.
fn make_marker_unreadable(repository: &Path) {
    let marker = repository.join("reuse-evidence.toml");
    fs::remove_file(&marker).expect("marker fixture should be removable");
    fs::create_dir(&marker).expect("marker path should become an unreadable directory");
}

fn event_files(steward: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = fs::read_dir(case_directory(steward))
        .expect("case directory should be readable")
        .map(|entry| {
            let path = entry.expect("case event should be readable").path();
            let bytes = fs::read(&path).expect("case event bytes should be readable");
            (path, bytes)
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

struct ClosedCaseWithSupersededEvents {
    fixture: Fixture,
    steward: PathBuf,
    early_review: PathBuf,
    append: PathBuf,
    decision: PathBuf,
    parked_verification: PathBuf,
}

fn closed_case_with_superseded_events(name: &str) -> ClosedCaseWithSupersededEvents {
    let fixture = Fixture::new(name);
    let steward = steward_with_one_open_case(&fixture);

    let early_review = fixture.write("early-review.toml", &early_review_override_proposal());
    case::authorize_early_review(
        &steward,
        CASE_ID,
        1,
        &early_review,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the early-review override should publish");
    replace_proposal_with_recorded_event(
        &steward,
        support::EARLY_REVIEW_AUTHORIZED_AT_2,
        &early_review,
    );

    let append = fixture.write("append.toml", &append_occurrence_proposal());
    case::append(
        &steward,
        CASE_ID,
        2,
        &append,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the third occurrence should publish after the override");
    replace_proposal_with_recorded_event(&steward, support::OCCURRENCE_APPENDED_AT_3, &append);

    let decision = fixture.write("decision.toml", &change_decision_proposal());
    case::decide(
        &steward,
        CASE_ID,
        3,
        &decision,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the accepted decision should publish");
    replace_proposal_with_recorded_event(
        &steward,
        support::REUSE_DECISION_ACCEPTED_AT_4,
        &decision,
    );

    let parked_verification = fixture.write(
        "parked-verification.toml",
        &unsuccessful_verification_proposal("parked"),
    );
    case::verify(
        &steward,
        CASE_ID,
        4,
        &parked_verification,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the parked verification should publish");
    replace_proposal_with_recorded_event(
        &steward,
        support::VERIFICATION_RECORDED_AT_5,
        &parked_verification,
    );

    let closed_verification =
        fixture.write("closed-verification.toml", &closed_verification_proposal());
    case::verify(
        &steward,
        CASE_ID,
        5,
        &closed_verification,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the closing verification should publish after the parked verification");

    ClosedCaseWithSupersededEvents {
        fixture,
        steward,
        early_review,
        append,
        decision,
        parked_verification,
    }
}

fn replace_proposal_with_recorded_event(steward: &Path, event_file: &str, proposal: &Path) {
    let event = fs::read(case_directory(steward).join(event_file))
        .expect("the recorded event should be readable for an exact retry");
    fs::write(proposal, event).expect("the recorded event should become the prepared retry");
}

fn refused_verification(
    steward: &Path,
    expected_revision: i64,
    proposal: &Path,
    location: &PortfolioLocation,
) -> TerminalFailure {
    let before = event_files(steward);
    let failure = case::verify(
        steward,
        CASE_ID,
        expected_revision,
        proposal,
        location,
        pinned(),
        false,
    )
    .expect_err("the proposed verification should refuse");
    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        event_files(steward),
        before,
        "a verification refusal must preserve every recorded event byte"
    );
    failure
}

fn assert_refusal_without_writes(
    failure: &TerminalFailure,
    expected: &str,
    fixture: &Fixture,
    before: &[(PathBuf, Vec<u8>)],
    subject: &str,
) {
    assert_eq!(
        failure.meaning(),
        ExitMeaning::Refusal,
        "{subject}: {failure}"
    );
    assert_eq!(failure.meaning().status(), 3, "{subject}: {failure}");
    assert_eq!(failure.to_string(), expected, "{subject}");
    assert_eq!(
        snapshot(&fixture.root).as_slice(),
        before,
        "the refused {subject} must write nothing"
    );
}

#[test]
fn an_opening_proposal_missing_responsibility_names_the_required_field() {
    let fixture = Fixture::new("opening-missing-responsibility");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let proposal = fixture.write(
        "open-case.toml",
        &two_occurrence_proposal().replace(
            "responsibility = \"normalize durable event identities\"\n",
            "",
        ),
    );
    let before = snapshot(&fixture.root);

    let failure = case::open(&steward, &proposal, &fixture.location(), pinned(), false)
        .expect_err("an opening proposal without its responsibility should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: case proposal `{}` is invalid: required field `responsibility` is missing\nresolution: add the required `responsibility` field to the case-opening proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "opening",
    );
}

#[test]
fn an_append_proposal_missing_independence_names_the_required_field() {
    let fixture = Fixture::new("append-missing-independence");
    let steward = steward_with_one_open_case(&fixture);
    let proposal = fixture.write(
        "append.toml",
        &append_occurrence_proposal()
            .replace("independence = \"separate distribution contract\"\n", ""),
    );
    let before = snapshot(&fixture.root);

    let failure = case::append(
        &steward,
        CASE_ID,
        1,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect_err("an append proposal without independence should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: append proposal `{}` is invalid: required field `independence` is missing\nresolution: add the required `independence` field to the occurrence-append proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "append",
    );
}

#[test]
fn an_append_proposal_with_an_unrecognized_evidence_kind_names_the_permitted_value() {
    let fixture = Fixture::new("append-unrecognized-evidence-kind");
    let steward = steward_with_one_open_case(&fixture);
    let proposal = fixture.write(
        "append.toml",
        &append_occurrence_proposal().replace("kind = \"commit\"", "kind = \"source_blob\""),
    );
    let before = snapshot(&fixture.root);

    let failure = case::append(
        &steward,
        CASE_ID,
        1,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect_err("an append proposal with an unrecognized evidence kind should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: append proposal `{}` is invalid: field `kind` value `source_blob` is unrecognized; permitted values: `commit`\nresolution: use one permitted `kind` value in the occurrence-append proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "append",
    );
}

#[test]
fn a_decision_proposal_missing_accepted_scope_names_the_required_field() {
    let fixture = Fixture::new("decision-missing-accepted-scope");
    let steward = steward_with_review_ready_case(&fixture);
    let proposal = fixture.write(
        "decision.toml",
        &change_decision_proposal().replace(
            "accepted_scope = \"the durable event identity contract\"\n",
            "",
        ),
    );
    let before = snapshot(&fixture.root);

    let failure = case::decide(
        &steward,
        CASE_ID,
        1,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect_err("a decision proposal without its accepted scope should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: reuse decision proposal `{}` is invalid: required field `accepted_scope` is missing\nresolution: add the required `accepted_scope` field to the reuse-decision proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "reuse decision",
    );
}

#[test]
fn a_decision_proposal_with_the_wrong_accepted_scope_type_names_the_field() {
    let fixture = Fixture::new("decision-wrong-accepted-scope-type");
    let steward = steward_with_review_ready_case(&fixture);
    let proposal = fixture.write(
        "decision.toml",
        &change_decision_proposal().replace(
            "accepted_scope = \"the durable event identity contract\"",
            "accepted_scope = 42",
        ),
    );
    let before = snapshot(&fixture.root);

    let failure = case::decide(
        &steward,
        CASE_ID,
        1,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect_err("a decision proposal with an integer accepted scope should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: reuse decision proposal `{}` is invalid: field `accepted_scope` has invalid TOML type integer `42`; expected a string\nresolution: provide `accepted_scope` with the expected TOML type in the reuse-decision proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "reuse decision",
    );
}

#[test]
fn decision_vocabulary_and_empty_list_refusals_keep_their_exact_text() {
    let fixture = Fixture::new("decision-existing-refusal-text");
    let steward = steward_with_review_ready_case(&fixture);
    let proposal = fixture.write("decision.toml", &change_decision_proposal());
    let cases = [
        (
            "identity verdict vocabulary",
            change_decision_proposal().replace(
                "identity_verdict = \"same_responsibility\"",
                "identity_verdict = \"looks_similar\"",
            ),
            "refusal: reuse decision `identity_verdict` value `looks_similar` is unrecognized\nresolution: use one permitted `identity_verdict` value: same_responsibility, different_responsibilities, insufficient_evidence, existing_abstraction_is_wrong",
        ),
        (
            "action vocabulary",
            change_decision_proposal().replace(
                "action = \"publish_public_package\"",
                "action = \"extract_automatically\"",
            ),
            "refusal: reuse decision `action` value `extract_automatically` is unrecognized\nresolution: use one permitted `action` value: retain_intentional_duplication, wait_for_more_evidence, use_existing_dependency, extract_or_deepen_locally, create_workspace_package, create_private_cross_repository_package, publish_public_package, centralize_schema_specification_or_fixture_corpus, replace_copies_with_generated_artifacts, contribute_missing_behavior_upstream, split_inline_or_narrow_existing_abstraction",
        ),
        (
            "required empty list",
            change_decision_proposal().replace(
                "non_responsibilities = [\"case lifecycle storage\"]",
                "non_responsibilities = []",
            ),
            "refusal: reuse decision `non_responsibilities` is missing or empty\nresolution: provide non-empty `non_responsibilities` content in the accepted reuse decision",
        ),
    ];

    for (case_name, contents, expected) in cases {
        fs::write(&proposal, contents).expect("the refused decision should be writable");
        let before = snapshot(&fixture.root);
        let failure = case::decide(
            &steward,
            CASE_ID,
            1,
            &proposal,
            &fixture.location(),
            pinned(),
            false,
        )
        .expect_err("the existing decision validation should refuse");

        assert_refusal_without_writes(&failure, expected, &fixture, &before, case_name);
    }
}

#[test]
fn a_verification_proposal_missing_disposition_names_the_required_field() {
    let fixture = Fixture::new("verification-missing-disposition");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write(
        "verification.toml",
        &closed_verification_proposal().replace("disposition = \"closed\"\n", ""),
    );
    let before = snapshot(&fixture.root);

    let failure = case::verify(
        &steward,
        CASE_ID,
        2,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect_err("a verification proposal without its disposition should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: verification proposal `{}` is invalid: required field `disposition` is missing\nresolution: add the required `disposition` field to the verification proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "verification",
    );
}

#[test]
fn an_opening_proposal_with_an_unrecognized_evidence_kind_names_the_field_and_permitted_value() {
    let fixture = Fixture::new("opening-unrecognized-evidence-kind");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let proposal = fixture.write(
        "open-case.toml",
        &two_occurrence_proposal().replacen("kind = \"commit\"", "kind = \"source_blob\"", 1),
    );
    let before = snapshot(&fixture.root);

    let failure = case::open(&steward, &proposal, &fixture.location(), pinned(), false)
        .expect_err("an opening proposal with an unrecognized evidence kind should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: case proposal `{}` is invalid: field `kind` value `source_blob` is unrecognized; permitted values: `commit`\nresolution: use one permitted `kind` value in the case-opening proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "opening",
    );
}

#[test]
fn an_incomplete_recorded_envelope_is_diagnosed_as_a_prepared_opening() {
    let fixture = Fixture::new("prepared-opening-missing-schema-version");
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let proposal = fixture.write("open-case.toml", &two_occurrence_proposal());
    let preview = case::open(&steward, &proposal, &fixture.location(), pinned(), true)
        .expect("the Human proposal should preview as a Prepared event")
        .to_string();
    let prepared = preview
        .split_once("event:\n")
        .expect("the preview receipt should carry exact event bytes")
        .1
        .replace("schema_version = 1\n", "");
    fs::write(&proposal, prepared).expect("the incomplete Prepared proposal should be writable");
    let before = snapshot(&fixture.root);

    let failure = case::open(&steward, &proposal, &fixture.location(), pinned(), false)
        .expect_err("an incomplete recorded envelope should refuse as Prepared");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: prepared case proposal `{}` is invalid: required field `schema_version` is missing\nresolution: use the exact event rendered by `case open --preview`",
            proposal.display()
        ),
        &fixture,
        &before,
        "Prepared opening",
    );
}

#[test]
fn a_verification_proposal_with_an_unrecognized_disposition_names_the_permitted_values() {
    let fixture = Fixture::new("verification-unrecognized-disposition");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write(
        "verification.toml",
        &closed_verification_proposal()
            .replace("disposition = \"closed\"", "disposition = \"complete\""),
    );
    let before = snapshot(&fixture.root);

    let failure = case::verify(
        &steward,
        CASE_ID,
        2,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect_err("a verification proposal with an unrecognized disposition should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: verification proposal `{}` is invalid: field `disposition` value `complete` is unrecognized; permitted values: `closed`, `parked`, `reopened`\nresolution: use one permitted `disposition` value in the verification proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "verification",
    );
}

#[test]
fn review_r1_standards_2_unrecognized_condition_outcome_names_field_and_permitted_values() {
    let fixture = Fixture::new("review-r1-condition-outcome");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write(
        "verification.toml",
        &closed_verification_proposal().replacen(
            "outcome = \"met\"",
            "outcome = \"condition_unknown\"",
            1,
        ),
    );
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());

    assert_eq!(failure.meaning().status(), 3, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: verification proposal `{}` is invalid: field `outcome` value `condition_unknown` is unrecognized; permitted values: `met`, `not_met`, `accepted_exception`\nresolution: use one permitted `outcome` value in the verification proposal",
            proposal.display()
        )
    );
}

#[test]
fn review_r1_standards_2_unrecognized_consumer_outcome_names_field_and_permitted_values() {
    let fixture = Fixture::new("review-r1-consumer-outcome");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write(
        "verification.toml",
        &closed_verification_proposal().replace(
            "consumer = \"rust-release-tool\"\noutcome = \"met\"",
            "consumer = \"rust-release-tool\"\noutcome = \"consumer_unknown\"",
        ),
    );
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());

    assert_eq!(failure.meaning().status(), 3, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: verification proposal `{}` is invalid: field `outcome` value `consumer_unknown` is unrecognized; permitted values: `met`, `not_met`, `accepted_exception`\nresolution: use one permitted `outcome` value in the verification proposal",
            proposal.display()
        )
    );
}

#[test]
fn an_early_review_proposal_with_an_unrecognized_evidence_kind_names_the_permitted_value() {
    let fixture = Fixture::new("early-review-unrecognized-evidence-kind");
    let steward = steward_with_one_open_case(&fixture);
    let proposal = fixture.write(
        "early-review.toml",
        &early_review_override_proposal().replace("kind = \"commit\"", "kind = \"source_blob\""),
    );
    let before = snapshot(&fixture.root);

    let failure = case::authorize_early_review(
        &steward,
        CASE_ID,
        1,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect_err("an early-review proposal with an unrecognized evidence kind should refuse");

    assert_refusal_without_writes(
        &failure,
        &format!(
            "refusal: early-review proposal `{}` is invalid: field `kind` value `source_blob` is unrecognized; permitted values: `commit`\nresolution: use one permitted `kind` value in the early-review proposal",
            proposal.display()
        ),
        &fixture,
        &before,
        "early-review authorization",
    );
}

#[test]
fn early_review_omissions_keep_their_named_semantic_refusals() {
    let fixture = Fixture::new("early-review-omissions");
    let steward = steward_with_one_open_case(&fixture);
    let proposal = fixture.write("early-review.toml", "");
    let cases = [
        (
            "empty early-review proposal",
            "",
            "refusal: early-review override reason is missing\nresolution: provide a concrete reason why waiting for a third occurrence is materially worse",
        ),
        (
            "missing early-review reason",
            "review_appetite = \"one working day\"\n\n[[evidence]]\nkind = \"commit\"\nreference = \"4444444\"\n",
            "refusal: early-review override reason is missing\nresolution: provide a concrete reason why waiting for a third occurrence is materially worse",
        ),
        (
            "missing early-review appetite",
            "reason = \"coordinated fixes are already required\"\n\n[[evidence]]\nkind = \"commit\"\nreference = \"4444444\"\n",
            "refusal: early-review override review appetite is missing\nresolution: bound the review effort before authorizing early review",
        ),
        (
            "missing early-review evidence",
            "reason = \"coordinated fixes are already required\"\nreview_appetite = \"one working day\"\n",
            "refusal: early-review override evidence is missing\nresolution: add one or more recoverable evidence references bearing why waiting is worse",
        ),
    ];

    for (case_name, contents, expected) in cases {
        fs::write(&proposal, contents).expect("the incomplete proposal should be writable");
        let before = snapshot(&fixture.root);
        let failure = case::authorize_early_review(
            &steward,
            CASE_ID,
            1,
            &proposal,
            &fixture.location(),
            pinned(),
            false,
        )
        .expect_err("the incomplete early-review proposal should refuse semantically");

        assert_refusal_without_writes(&failure, expected, &fixture, &before, case_name);
    }
}

#[test]
fn a_superseded_early_review_retry_reports_the_closed_case_and_writes_nothing() {
    let closed = closed_case_with_superseded_events("superseded-early-review-retry");
    let before = event_files(&closed.steward);

    let retry = case::authorize_early_review(
        &closed.steward,
        CASE_ID,
        1,
        &closed.early_review,
        &closed.fixture.location(),
        pinned(),
        false,
    )
    .expect("an exact superseded early-review retry should succeed");

    assert_eq!(
        retry.to_string(),
        format!(
            "early review already authorized\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/{}\nrevision: 2\nstate: closed\nprivacy: private\n",
            support::EARLY_REVIEW_AUTHORIZED_AT_2
        )
    );
    assert_eq!(
        event_files(&closed.steward),
        before,
        "a superseded early-review retry must preserve every event byte"
    );
}

#[test]
fn a_superseded_decision_retry_reports_the_closed_case_and_writes_nothing() {
    let closed = closed_case_with_superseded_events("superseded-decision-retry");
    let before = event_files(&closed.steward);

    let retry = case::decide(
        &closed.steward,
        CASE_ID,
        3,
        &closed.decision,
        &closed.fixture.location(),
        pinned(),
        false,
    )
    .expect("an exact superseded reuse-decision retry should succeed");

    assert_eq!(
        retry.to_string(),
        format!(
            "reuse decision already recorded\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/{}\nrevision: 4\nstate: closed\nprivacy: private\ndecision: authorizes implementation outside the reuse lifecycle; does not perform it\n",
            support::REUSE_DECISION_ACCEPTED_AT_4
        )
    );
    assert_eq!(
        event_files(&closed.steward),
        before,
        "a superseded reuse-decision retry must preserve every event byte"
    );
}

#[test]
fn a_superseded_append_retry_keeps_its_live_closed_receipt_and_writes_nothing() {
    let closed = closed_case_with_superseded_events("superseded-append-retry");
    let before = event_files(&closed.steward);

    let retry = case::append(
        &closed.steward,
        CASE_ID,
        2,
        &closed.append,
        &closed.fixture.location(),
        pinned(),
        false,
    )
    .expect("an exact superseded occurrence-append retry should succeed");

    assert_eq!(
        retry.to_string(),
        format!(
            "occurrence already recorded\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/{}\nrevision: 3\nstate: closed\nprivacy: private\n",
            support::OCCURRENCE_APPENDED_AT_3
        )
    );
    assert_eq!(
        event_files(&closed.steward),
        before,
        "a superseded occurrence-append retry must preserve every event byte"
    );
}

#[test]
fn a_superseded_verification_retry_keeps_its_parked_event_voice_and_writes_nothing() {
    let closed = closed_case_with_superseded_events("superseded-verification-retry");
    let before = event_files(&closed.steward);

    let retry = case::verify(
        &closed.steward,
        CASE_ID,
        4,
        &closed.parked_verification,
        &closed.fixture.location(),
        pinned(),
        false,
    )
    .expect("an exact superseded parked-verification retry should succeed");

    assert_eq!(
        retry.to_string(),
        format!(
            "verification already recorded: parked\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/{}\nrevision: 5\nstate: closed\nprivacy: private\ndisposition: parked\n",
            support::VERIFICATION_RECORDED_AT_5
        )
    );
    assert_eq!(
        event_files(&closed.steward),
        before,
        "a superseded verification retry must preserve every event byte"
    );
}

#[test]
fn a_closed_verification_preview_applies_the_exact_approved_event_bytes() {
    preview_and_record_closed_verification();
}

struct RecordedClosedVerification {
    fixture: Fixture,
    steward: PathBuf,
    proposal: PathBuf,
    previewed_event: String,
    brief_before: String,
}

fn preview_and_record_closed_verification() -> RecordedClosedVerification {
    let fixture = Fixture::new("closed-verification");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let event_path = case_directory(&steward).join(support::VERIFICATION_RECORDED_AT_3);
    let brief_before = case::brief(&steward, CASE_ID, &fixture.location())
        .expect("a decided case should project its brief")
        .to_string();

    let preview = case::verify(
        &steward,
        CASE_ID,
        2,
        &proposal,
        &fixture.location(),
        pinned(),
        true,
    )
    .expect("a complete closing verification should preview");
    let rendered_preview = preview.to_string();
    assert!(
        rendered_preview.starts_with(&format!(
            "verification preview: closed\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/{}\nrevision: 3\nstate: closed\nprivacy: private\ndisposition: closed\nevent:\n",
            support::VERIFICATION_RECORDED_AT_3
        )),
        "{rendered_preview}"
    );
    assert!(!event_path.exists(), "a preview must not publish an event");
    let previewed_event = rendered_preview
        .split_once("event:\n")
        .expect("a preview should delimit its exact event")
        .1
        .to_owned();
    assert!(
        previewed_event.starts_with(concat!(
            "schema_version = 1\n",
            "sequence = 3\n",
            "event_id = \""
        )),
        "{previewed_event}"
    );
    assert!(
        previewed_event.contains(
            "event_type = \"verification_recorded\"\nrecorded_at = \"2025-10-09T08:53:20Z\"\ndisposition = \"closed\"\n"
        ),
        "{previewed_event}"
    );

    fs::write(&proposal, &previewed_event).expect("the approved event should be reusable");
    let recorded = case::verify(
        &steward,
        CASE_ID,
        2,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the approved verification should publish");
    assert_eq!(
        recorded.to_string(),
        format!(
            "recorded verification: closed\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/{}\nrevision: 3\nstate: closed\nprivacy: private\ndisposition: closed\n",
            support::VERIFICATION_RECORDED_AT_3
        )
    );
    assert_eq!(
        fs::read_to_string(&event_path).expect("the verification event should be readable"),
        previewed_event,
        "the approved preview bytes must be the recorded bytes"
    );

    RecordedClosedVerification {
        fixture,
        steward,
        proposal,
        previewed_event,
        brief_before,
    }
}

fn record_closed_verification(name: &str) -> RecordedClosedVerification {
    let fixture = Fixture::new(name);
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let brief_before = case::brief(&steward, CASE_ID, &fixture.location())
        .expect("a decided case should project its brief")
        .to_string();
    case::verify(
        &steward,
        CASE_ID,
        2,
        &proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the complete closing verification should publish");
    let previewed_event =
        fs::read_to_string(case_directory(&steward).join(support::VERIFICATION_RECORDED_AT_3))
            .expect("the recorded verification should be readable");
    fs::write(&proposal, &previewed_event)
        .expect("the exact recorded verification should be reusable as a prepared retry");
    RecordedClosedVerification {
        fixture,
        steward,
        proposal,
        previewed_event,
        brief_before,
    }
}

#[test]
fn an_exact_prepared_verification_retry_is_write_free_before_portfolio_resolution() {
    let recorded = record_closed_verification("exact-verification-retry");
    let steward = recorded.steward.as_path();
    let proposal = recorded.proposal.as_path();
    let recorded_files = event_files(steward);
    let unavailable_portfolio = PortfolioLocation::from_user_directories(Vec::new(), None, None);
    let retried = case::verify(
        steward,
        CASE_ID,
        2,
        proposal,
        &unavailable_portfolio,
        pinned(),
        false,
    )
    .expect("an exact verification retry should not need portfolio resolution");
    assert_eq!(
        retried.to_string(),
        format!(
            "verification already recorded: closed\ncase_id: {CASE_ID}\nfile: reuse-evidence/cases/{CASE_ID}/{}\nrevision: 3\nstate: closed\nprivacy: unknown\nportfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\ndisposition: closed\n",
            support::VERIFICATION_RECORDED_AT_3
        )
    );
    assert_eq!(
        event_files(steward),
        recorded_files,
        "an exact retry must preserve every event byte"
    );
}

#[test]
fn an_occupied_verification_sequence_refuses_a_different_prepared_identity_without_writing() {
    let recorded = record_closed_verification("verification-identity-conflict");
    let fixture = &recorded.fixture;
    let steward = recorded.steward.as_path();
    let proposal = recorded.proposal.as_path();
    let previewed_event = recorded.previewed_event.as_str();
    let recorded_event_id_line = previewed_event
        .lines()
        .find(|line| line.starts_with("event_id = "))
        .expect("the prepared event should record its identity");
    let conflicting_identity = previewed_event.replacen(
        recorded_event_id_line,
        "event_id = \"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa\"",
        1,
    );
    fs::write(proposal, conflicting_identity)
        .expect("the conflicting prepared identity should be writable");
    let failure = refused_verification(steward, 2, proposal, &fixture.location());
    assert!(
        failure
            .to_string()
            .contains("has a revision conflict at sequence 3"),
        "{failure}"
    );
}

#[test]
fn an_occupied_verification_sequence_refuses_prepared_content_drift_without_writing() {
    let recorded = record_closed_verification("verification-content-drift");
    let fixture = &recorded.fixture;
    let steward = recorded.steward.as_path();
    let proposal = recorded.proposal.as_path();
    let previewed_event = recorded.previewed_event.as_str();
    let drifting_content =
        previewed_event.replacen("reference = \"4444444\"", "reference = \"4444445\"", 1);
    fs::write(proposal, drifting_content)
        .expect("the content-drifting prepared event should be writable");
    let failure = refused_verification(steward, 2, proposal, &fixture.location());
    assert!(
        failure.to_string().contains(
            "is already recorded with different content\nresolution: restore the exact previewed verification event before retrying"
        ),
        "{failure}"
    );
}

#[test]
fn closed_case_projections_have_no_readiness_and_retain_current_portfolio_conditions() {
    let recorded = record_closed_verification("closed-case-projections");
    let fixture = &recorded.fixture;
    let steward = recorded.steward.as_path();
    let shown = case::show(steward, CASE_ID, &fixture.location())
        .expect("the closed case should remain readable")
        .to_string();
    assert!(
        shown.contains("revision: 3\noccurrence_count: 3\nstate: closed\n"),
        "{shown}"
    );
    assert!(
        shown.contains("privacy_conflicted: false\nstale: false\n"),
        "a closed case must retain current portfolio conditions: {shown}"
    );
    let listed = case::list(steward, &fixture.location())
        .expect("the closed case should remain listable")
        .to_string();
    assert!(listed.contains("  state: closed\n"), "{listed}");
    assert!(
        !listed.contains("readiness_basis:"),
        "a closed case has no readiness basis: {listed}"
    );
    assert!(
        listed.contains("  privacy_conflicted: false\n  stale: false\n"),
        "a closed case must retain current portfolio conditions: {listed}"
    );
}

#[test]
fn case_show_renders_every_verification_result_evidence_and_exception() {
    let recorded = record_closed_verification("closed-verification-show");
    let shown = case::show(&recorded.steward, CASE_ID, &recorded.fixture.location())
        .expect("the closed case should remain readable")
        .to_string();
    assert!(
        shown.contains(&format!(
            "verifications:\n- disposition: closed\n  condition_results:\n  - condition: all named consumers pass their public contract tests\n    outcome: met\n    evidence:\n    - kind: commit\n      reference: 4444444\n      path: tests/contract.rs\n  consumer_results:\n  - repository_id: {FIRST_PARTICIPANT_ID}\n    consumer: rust-release-tool\n    outcome: met\n    evidence:\n    - kind: commit\n      reference: 5555555\n  - repository_id: {SECOND_PARTICIPANT_ID}\n    consumer: web-deployment-tool\n    outcome: accepted_exception\n    exception: the accepted decision retained this language-specific adapter\n    evidence: []\n"
        )),
        "{shown}"
    );
}

#[test]
fn verification_does_not_change_the_accepted_decision_brief() {
    let recorded = record_closed_verification("verification-brief-stability");
    assert_eq!(
        case::brief(&recorded.steward, CASE_ID, &recorded.fixture.location(),)
            .expect("verification must not change the projected decision")
            .to_string(),
        recorded.brief_before,
        "case brief must remain the accepted-decision projection"
    );
}

#[test]
fn parked_and_reopened_cases_accept_later_verification_and_latest_disposition_drives_state() {
    let (recovery, recovery_steward, recovery_proposal) = prepare_appended_decided_case();
    let parked = record_disposition(
        &recovery,
        &recovery_steward,
        &recovery_proposal,
        3,
        &unsuccessful_verification_proposal("parked"),
        "parked",
    );
    assert!(parked.contains("revision: 4\n"), "{parked}");
    let parked_show = case::show(&recovery_steward, CASE_ID, &recovery.location())
        .expect("the parked state should rebuild from its event stream")
        .to_string();
    assert!(
        parked_show.contains("revision: 4\noccurrence_count: 3\nstate: parked\n")
            && !parked_show.contains("readiness_basis:"),
        "{parked_show}"
    );

    let reopened = record_disposition(
        &recovery,
        &recovery_steward,
        &recovery_proposal,
        4,
        &unsuccessful_verification_proposal("reopened"),
        "reopened",
    );
    assert!(reopened.contains("revision: 5\n"), "{reopened}");
    let reopened_show = case::show(&recovery_steward, CASE_ID, &recovery.location())
        .expect("the reopened state should rebuild from its event stream")
        .to_string();
    assert!(
        reopened_show.contains("revision: 5\noccurrence_count: 3\nstate: reopened\n")
            && !reopened_show.contains("readiness_basis:"),
        "{reopened_show}"
    );

    let recovered = record_disposition(
        &recovery,
        &recovery_steward,
        &recovery_proposal,
        5,
        &closed_verification_proposal(),
        "closed",
    );
    assert!(recovered.contains("revision: 6\n"), "{recovered}");
    assert_recovered_history(&recovery, &recovery_steward);
}

fn prepare_appended_decided_case() -> (Fixture, PathBuf, PathBuf) {
    let recovery = Fixture::new("verification-recovery-loop");
    let recovery_steward = steward_with_one_open_case(&recovery);
    let appended_occurrence = recovery.write("append.toml", &append_occurrence_proposal());
    let appended = case::append(
        &recovery_steward,
        CASE_ID,
        1,
        &appended_occurrence,
        &recovery.location(),
        pinned(),
        false,
    )
    .expect("the third independent occurrence should publish")
    .to_string();
    assert!(
        appended.contains("revision: 2\nstate: review-ready\n"),
        "{appended}"
    );
    let recovery_decision = recovery.write("decision.toml", &change_decision_proposal());
    case::decide(
        &recovery_steward,
        CASE_ID,
        2,
        &recovery_decision,
        &recovery.location(),
        pinned(),
        false,
    )
    .expect("the review-ready case should record its accepted decision");
    let recovery_proposal = recovery.write("verification.toml", "");
    (recovery, recovery_steward, recovery_proposal)
}

fn record_disposition(
    fixture: &Fixture,
    steward: &Path,
    proposal: &Path,
    expected_revision: i64,
    contents: &str,
    disposition: &str,
) -> String {
    fs::write(proposal, contents).expect("the next verification proposal should be writable");
    let receipt = case::verify(
        steward,
        CASE_ID,
        expected_revision,
        proposal,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("a parked or reopened case should accept the next verification")
    .to_string();
    assert!(
        receipt.starts_with(&format!("recorded verification: {disposition}\n"))
            && receipt.contains(&format!("state: {disposition}\n"))
            && receipt.contains(&format!("disposition: {disposition}\n")),
        "{receipt}"
    );
    receipt
}

fn assert_recovered_history(recovery: &Fixture, recovery_steward: &Path) {
    let recovered_show = case::show(recovery_steward, CASE_ID, &recovery.location())
        .expect("the recovered case should rebuild from its full event stream")
        .to_string();
    let parked_at = recovered_show
        .find("- disposition: parked\n")
        .expect("the failed parked verification must remain visible");
    let reopened_at = recovered_show
        .find("- disposition: reopened\n")
        .expect("the failed reopened verification must remain visible");
    let closed_at = recovered_show
        .find("- disposition: closed\n")
        .expect("the final closed verification must be visible");
    assert!(
        parked_at < reopened_at && reopened_at < closed_at,
        "verification history must stay in event order: {recovered_show}"
    );
    assert!(
        recovered_show.contains("revision: 6\noccurrence_count: 3\nstate: closed\n"),
        "the latest verification disposition must determine state: {recovered_show}"
    );
}

#[test]
fn verification_refuses_a_stale_revision_without_writing() {
    let race = Fixture::new("verification-revision-race");
    let race_steward = steward_with_decided_case(&race);
    let race_proposal = race.write("verification.toml", &closed_verification_proposal());
    let stale = refused_verification(&race_steward, 1, &race_proposal, &race.location());
    assert!(
        stale.to_string().contains(&format!(
            "expected revision 1 does not match case `{CASE_ID}` current revision 2"
        )),
        "{stale}"
    );
}

#[test]
fn concurrent_verification_writers_publish_exactly_one_event() {
    let race = Fixture::new("verification-writer-race");
    let race_steward = steward_with_decided_case(&race);
    let race_proposal = race.write("verification.toml", &closed_verification_proposal());
    let race_location = race.location();
    let start = Barrier::new(2);
    let (left, right) = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            start.wait();
            case::verify(
                &race_steward,
                CASE_ID,
                2,
                &race_proposal,
                &race_location,
                pinned(),
                false,
            )
        });
        let right = scope.spawn(|| {
            start.wait();
            case::verify(
                &race_steward,
                CASE_ID,
                2,
                &race_proposal,
                &race_location,
                pinned(),
                false,
            )
        });
        (
            left.join().expect("the left writer should not panic"),
            right.join().expect("the right writer should not panic"),
        )
    });
    assert_eq!(
        [left.is_ok(), right.is_ok()]
            .into_iter()
            .filter(|published| *published)
            .count(),
        1,
        "exactly one verification writer may publish: left={left:?}, right={right:?}"
    );
    let ((Err(loser), _) | (_, Err(loser))) = (&left, &right) else {
        unreachable!("one verification writer must refuse")
    };
    assert_eq!(loser.meaning(), ExitMeaning::Refusal, "{loser}");
    assert_eq!(
        event_files(&race_steward)
            .iter()
            .filter(|(path, _)| {
                path.file_name()
                    .is_some_and(|name| name == support::VERIFICATION_RECORDED_AT_3)
            })
            .count(),
        1,
        "the losing writer must not expose a second sequence-3 event"
    );
}

#[test]
fn a_closed_case_refuses_a_new_later_event_without_writing() {
    assert_closed_refuses_append();
}

fn assert_closed_refuses_append() {
    let fixture = Fixture::new("closed-is-terminal");
    let steward = steward_with_decided_case(&fixture);
    let verification = fixture.write("verification.toml", &closed_verification_proposal());
    case::verify(
        &steward,
        CASE_ID,
        2,
        &verification,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the case should close");
    fixture.repository("fourth-consumer", FOURTH_PARTICIPANT_ID, "public");
    let append = fixture.write("append-fourth.toml", &fourth_occurrence_proposal());
    let before = event_files(&steward);

    let Err(failure) = case::append(
        &steward,
        CASE_ID,
        3,
        &append,
        &fixture.location(),
        pinned(),
        false,
    ) else {
        panic!("a closed case must refuse every new later event");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: case `{CASE_ID}` is closed and terminal in version 0.1\nresolution: leave the closed case unchanged; later pressure requires a separately accepted capability"
        )
    );
    assert_eq!(
        event_files(&steward),
        before,
        "a terminal-state refusal must preserve every recorded event byte"
    );
}

#[test]
fn a_closed_case_refuses_a_new_verification_without_writing() {
    let fixture = Fixture::new("closed-refuses-verification");
    let steward = steward_with_decided_case(&fixture);
    let verification = fixture.write("verification.toml", &closed_verification_proposal());
    case::verify(
        &steward,
        CASE_ID,
        2,
        &verification,
        &fixture.location(),
        pinned(),
        false,
    )
    .expect("the case should close");
    let failure = refused_verification(&steward, 3, &verification, &fixture.location());
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: case `{CASE_ID}` is closed and terminal in version 0.1\nresolution: leave the closed case unchanged; later pressure requires a separately accepted capability"
        )
    );
}

#[test]
fn an_undecided_case_refuses_verification_without_writing() {
    let undecided = Fixture::new("undecided-verification");
    let undecided_steward = steward_with_one_open_case(&undecided);
    let undecided_proposal = undecided.write("verification.toml", &closed_verification_proposal());
    let failure = refused_verification(
        &undecided_steward,
        1,
        &undecided_proposal,
        &undecided.location(),
    );
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: case `{CASE_ID}` has no accepted reuse decision; current state is `watching`\nresolution: record an accepted reuse decision before retrying verification"
        )
    );
}

#[test]
fn an_unknown_case_refuses_verification_without_writing() {
    let undecided = Fixture::new("unknown-case-verification");
    let undecided_steward = steward_with_one_open_case(&undecided);
    let undecided_proposal = undecided.write("verification.toml", &closed_verification_proposal());
    let unknown_case_id = "00000000-0000-4000-8000-000000000099";
    let undecided_before = event_files(&undecided_steward);
    let unknown = case::verify(
        &undecided_steward,
        unknown_case_id,
        1,
        &undecided_proposal,
        &undecided.location(),
        pinned(),
        false,
    )
    .expect_err("an unknown case must refuse");
    assert_eq!(unknown.meaning(), ExitMeaning::Refusal, "{unknown}");
    assert!(
        unknown.to_string().contains(&format!(
            "case identity `{unknown_case_id}` is not stewarded"
        )),
        "{unknown}"
    );
    assert_eq!(event_files(&undecided_steward), undecided_before);
}

#[test]
fn an_unstewarded_case_refuses_verification_without_writing() {
    let undecided = Fixture::new("unstewarded-case-verification");
    let undecided_steward = steward_with_one_open_case(&undecided);
    let undecided_proposal = undecided.write("verification.toml", &closed_verification_proposal());
    let undecided_before = event_files(&undecided_steward);
    let other_steward = undecided.repository("other-steward", FOURTH_PARTICIPANT_ID, "private");
    let unstewarded = case::verify(
        &other_steward,
        CASE_ID,
        1,
        &undecided_proposal,
        &undecided.location(),
        pinned(),
        false,
    )
    .expect_err("a repository that does not steward the case must refuse");
    assert_eq!(unstewarded.meaning(), ExitMeaning::Refusal, "{unstewarded}");
    assert!(
        unstewarded
            .to_string()
            .contains(&format!("case identity `{CASE_ID}` is not stewarded")),
        "{unstewarded}"
    );
    assert_eq!(event_files(&undecided_steward), undecided_before);
}

#[test]
fn a_private_case_under_a_public_steward_refuses_verification_without_writing() {
    let conflicted = Fixture::new("private-case-public-steward-verification");
    let conflicted_steward = steward_with_decided_case(&conflicted);
    support::enrollment_marker(&conflicted_steward, STEWARD_ID, "public");
    let conflicted_proposal =
        conflicted.write("verification.toml", &closed_verification_proposal());
    let failure = refused_verification(
        &conflicted_steward,
        2,
        &conflicted_proposal,
        &conflicted.location(),
    );
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: public steward `{STEWARD_ID}` cannot record verification for private case `{CASE_ID}`\nresolution: run `set-visibility --visibility private` in the steward repository, then preview verification again"
        )
    );
}

#[test]
fn a_missing_condition_result_names_the_unanswered_decision_condition() {
    let fixture = Fixture::new("missing-condition-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write(
        "verification.toml",
        &verification_without_condition_result(),
    );
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: verification for case `{CASE_ID}` is missing condition result 1 for `all named consumers pass their public contract tests`\nresolution: answer every accepted verification condition exactly once in its recorded order"
        )
    );
}

#[test]
fn a_changed_condition_result_refuses_without_writing() {
    let fixture = Fixture::new("changed-condition-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let changed_condition = closed_verification_proposal().replace(
        "all named consumers pass their public contract tests",
        "all named consumers pass changed contract tests",
    );
    fs::write(&proposal, changed_condition).expect("the changed condition should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure.to_string().contains(
            "condition result 1 repeats `all named consumers pass changed contract tests`, but the accepted decision records `all named consumers pass their public contract tests`"
        ),
        "{failure}"
    );
}

#[test]
fn an_extra_condition_result_refuses_without_writing() {
    let fixture = Fixture::new("extra-condition-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let extra_condition = format!(
        "{}\n[[condition_results]]\ncondition = \"an unaccepted extra condition\"\noutcome = \"accepted_exception\"\nexception = \"the decision did not ask this question\"\n",
        closed_verification_proposal()
    );
    fs::write(&proposal, extra_condition).expect("the extra condition should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure
            .to_string()
            .contains("records extra condition `an unaccepted extra condition`"),
        "{failure}"
    );
}

#[test]
fn a_duplicate_condition_result_refuses_without_writing() {
    let fixture = Fixture::new("duplicate-condition-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let duplicate_condition = format!(
        "{}\n[[condition_results]]\ncondition = \"all named consumers pass their public contract tests\"\noutcome = \"accepted_exception\"\nexception = \"duplicate answer\"\n",
        closed_verification_proposal()
    );
    fs::write(&proposal, duplicate_condition).expect("the duplicate condition should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure.to_string().contains(
            "records extra condition `all named consumers pass their public contract tests`"
        ),
        "{failure}"
    );
}

#[test]
fn an_accepted_exception_without_a_reason_refuses_without_writing() {
    let fixture = Fixture::new("verification-result-rules");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let exception_without_reason = closed_verification_proposal().replace(
        "exception = \"the accepted decision retained this language-specific adapter\"\n",
        "",
    );
    fs::write(&proposal, exception_without_reason)
        .expect("the reasonless exception should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure
            .to_string()
            .contains("consumer result 2 is an accepted exception without a reason"),
        "{failure}"
    );
}

#[test]
fn a_met_result_with_an_exception_reason_refuses_without_writing() {
    let fixture = Fixture::new("met-result-with-exception");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let reason_on_met = closed_verification_proposal().replace(
        "outcome = \"met\"\n\n[[condition_results.evidence]]",
        "outcome = \"met\"\nexception = \"not permitted for met\"\n\n[[condition_results.evidence]]",
    );
    fs::write(&proposal, reason_on_met).expect("the invalid met result should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure
            .to_string()
            .contains("condition result 1 outcome `met` carries an exception reason"),
        "{failure}"
    );
}

#[test]
fn a_met_result_without_evidence_refuses_without_writing() {
    let fixture = Fixture::new("met-result-without-evidence");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let met_without_evidence = closed_verification_proposal().replace(
        "\n[[condition_results.evidence]]\nkind = \"commit\"\nreference = \"4444444\"\npath = \"tests/contract.rs\"\n",
        "",
    );
    fs::write(&proposal, met_without_evidence)
        .expect("the evidence-free met result should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure
            .to_string()
            .contains("condition result 1 outcome `met` carries no evidence reference"),
        "{failure}"
    );
}

#[test]
fn a_closed_disposition_with_a_not_met_result_refuses_without_writing() {
    let fixture = Fixture::new("closed-with-not-met-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    fs::write(&proposal, unsuccessful_verification_proposal("closed"))
        .expect("the invalid closing result should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure
            .to_string()
            .contains("disposition `closed` carries a `not_met` result"),
        "{failure}"
    );
}

#[test]
fn reordered_condition_results_refuse_without_writing() {
    let ordered = Fixture::new("condition-result-order");
    let ordered_steward = ordered.repository("steward", STEWARD_ID, "private");
    ordered.repository("first-consumer", FIRST_PARTICIPANT_ID, "public");
    ordered.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    ordered.repository("third-consumer", THIRD_PARTICIPANT_ID, "public");
    let opening = ordered.write("open-case.toml", &three_occurrence_proposal());
    case::open(
        &ordered_steward,
        &opening,
        &ordered.location(),
        pinned(),
        false,
    )
    .expect("the ordered-condition case should open");
    let decision = change_decision_proposal().replace(
        "verification_conditions = [\"all named consumers pass their public contract tests\"]",
        "verification_conditions = [\"all named consumers pass their public contract tests\", \"recorded event identities remain immutable\"]",
    );
    let decision = ordered.write("decision.toml", &decision);
    case::decide(
        &ordered_steward,
        CASE_ID,
        1,
        &decision,
        &ordered.location(),
        pinned(),
        false,
    )
    .expect("the two-condition decision should record");
    let reordered = format!(
        "{}\n[[condition_results]]\ncondition = \"all named consumers pass their public contract tests\"\noutcome = \"accepted_exception\"\nexception = \"duplicate answer\"\n",
        closed_verification_proposal().replace(
            "all named consumers pass their public contract tests",
            "recorded event identities remain immutable",
        )
    );
    let reordered = ordered.write("verification.toml", &reordered);
    let failure = refused_verification(&ordered_steward, 2, &reordered, &ordered.location());
    assert!(
        failure.to_string().contains(
            "condition result 1 repeats `recorded event identities remain immutable`, but the accepted decision records `all named consumers pass their public contract tests`"
        ),
        "{failure}"
    );
}

#[test]
fn a_missing_consumer_result_names_the_unanswered_affected_consumer() {
    let fixture = Fixture::new("missing-consumer-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &verification_without_consumer_result());
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: verification for case `{CASE_ID}` is missing consumer `rust-release-tool` in participant `{FIRST_PARTICIPANT_ID}`\nresolution: answer every affected participant repository and consumer pair exactly once"
        )
    );
}

#[test]
fn an_extra_consumer_result_refuses_without_writing() {
    let fixture = Fixture::new("extra-consumer-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let extra_consumer = format!(
        "{}\n[[consumer_results]]\nrepository_id = \"{THIRD_PARTICIPANT_ID}\"\nconsumer = \"desktop-packager\"\noutcome = \"met\"\n\n[[consumer_results.evidence]]\nkind = \"commit\"\nreference = \"aaaaaaa\"\n",
        closed_verification_proposal()
    );
    fs::write(&proposal, extra_consumer).expect("the extra consumer should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure.to_string().contains(&format!(
            "records extra consumer `desktop-packager` in participant `{THIRD_PARTICIPANT_ID}`"
        )),
        "{failure}"
    );
}

#[test]
fn a_duplicate_consumer_result_refuses_without_writing() {
    let fixture = Fixture::new("duplicate-consumer-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let duplicate_consumer = format!(
        "{}\n[[consumer_results]]\nrepository_id = \"{FIRST_PARTICIPANT_ID}\"\nconsumer = \"rust-release-tool\"\noutcome = \"accepted_exception\"\nexception = \"duplicate answer\"\n",
        closed_verification_proposal()
    );
    fs::write(&proposal, duplicate_consumer).expect("the duplicate consumer should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure.to_string().contains(&format!(
            "records consumer `rust-release-tool` in participant `{FIRST_PARTICIPANT_ID}` more than once"
        )),
        "{failure}"
    );
}

#[test]
fn a_renamed_consumer_result_refuses_without_writing() {
    let fixture = Fixture::new("renamed-consumer-result");
    let steward = steward_with_decided_case(&fixture);
    let proposal = fixture.write("verification.toml", &closed_verification_proposal());
    let changed_consumer = closed_verification_proposal().replace(
        "consumer = \"rust-release-tool\"",
        "consumer = \"renamed-release-tool\"",
    );
    fs::write(&proposal, changed_consumer).expect("the changed consumer should be writable");
    let failure = refused_verification(&steward, 2, &proposal, &fixture.location());
    assert!(
        failure.to_string().contains(&format!(
            "is missing consumer `rust-release-tool` in participant `{FIRST_PARTICIPANT_ID}`"
        )),
        "{failure}"
    );
}

#[test]
fn one_unreadable_marker_means_the_same_thing_to_enrollment_and_to_the_case_surface() {
    let fixture = Fixture::new("marker-classification");

    // ADR 0018: one input, one meaning. Nothing has been written when a marker is read, so the
    // no-write guarantee a refusal carries is truthful at both sites, and `UnsafeFailure` — which
    // is defined by the absence of that guarantee — would state something untrue about the run.
    let enrolling = fixture.repository("enrolling", STEWARD_ID, "private");
    make_marker_unreadable(&enrolling);
    let Err(enrollment_failure) = enroll(&enrolling, "products", Visibility::Private) else {
        panic!("enrolling over an unreadable marker must fail");
    };

    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let proposal = fixture.write("open-case.toml", &two_occurrence_proposal());
    make_marker_unreadable(&steward);
    let Err(case_failure) = case::open(&steward, &proposal, &fixture.location(), pinned(), false)
    else {
        panic!("opening a case under an unreadable marker must fail");
    };

    assert_eq!(
        enrollment_failure.meaning(),
        ExitMeaning::Refusal,
        "{enrollment_failure}"
    );
    assert_eq!(
        case_failure.meaning(),
        enrollment_failure.meaning(),
        "the two surfaces must agree about one unreadable marker"
    );
    assert_eq!(
        case_failure.meaning().status(),
        3,
        "the refusal meaning maps to status 3"
    );

    // The shared classification names the fault and its cause; only the resolution differs,
    // because it names the command that ran.
    for (failure, expected_resolution) in [
        (
            &enrollment_failure,
            "restore a complete valid version 1 marker before rerunning enrollment",
        ),
        (
            &case_failure,
            "restore a supported `reuse-evidence.toml` marker before opening a case",
        ),
    ] {
        let rendered = failure.to_string();
        assert!(
            rendered.starts_with("refusal: could not read `"),
            "{rendered}"
        );
        assert!(
            rendered.ends_with(&format!("\nresolution: {expected_resolution}")),
            "{rendered}"
        );
    }
    assert!(
        !case_directory(&steward).exists(),
        "the refusal must have written nothing"
    );
}

#[test]
fn two_threads_competing_for_one_revision_publish_exactly_one_event() {
    let fixture = Fixture::new("revision-race");
    let steward = steward_with_one_open_case(&fixture);
    let append_proposal = fixture.write("append.toml", &append_occurrence_proposal());
    let override_proposal = fixture.write("override.toml", &early_review_override_proposal());
    let location = fixture.location();

    // Both writers claim revision 1. The marker lock is per open file description, so two
    // threads contend exactly as two processes did — without a spawn or a timed sleep.
    let start = Barrier::new(2);
    let (appended, overridden) = std::thread::scope(|scope| {
        let appending = scope.spawn(|| {
            start.wait();
            case::append(
                &steward,
                CASE_ID,
                1,
                &append_proposal,
                &location,
                pinned(),
                false,
            )
        });
        let overriding = scope.spawn(|| {
            start.wait();
            case::authorize_early_review(
                &steward,
                CASE_ID,
                1,
                &override_proposal,
                &location,
                pinned(),
                false,
            )
        });
        (
            appending.join().expect("append thread should not panic"),
            overriding.join().expect("override thread should not panic"),
        )
    });

    let published = [appended.is_ok(), overridden.is_ok()];
    assert_eq!(
        published.iter().filter(|published| **published).count(),
        1,
        "exactly one same-revision writer may publish: append={appended:?}, override={overridden:?}"
    );

    let ((Err(refusal), _) | (_, Err(refusal))) = (&appended, &overridden) else {
        unreachable!("one writer must have refused")
    };
    assert_eq!(
        refusal.meaning(),
        ExitMeaning::Refusal,
        "the losing writer must refuse safely: {refusal}"
    );

    let at_sequence_two = fs::read_dir(case_directory(&steward))
        .expect("case should remain readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("0002-"))
        .count();
    assert_eq!(
        at_sequence_two, 1,
        "the losing writer must not have left a second event at sequence 2"
    );
}

#[test]
fn a_pinned_clock_records_that_exact_instant_in_the_event() {
    let fixture = Fixture::new("pinned-clock");
    let steward = steward_with_one_open_case(&fixture);

    let opening = fs::read_to_string(case_directory(&steward).join(support::CASE_OPENED_AT_1))
        .expect("opening event should be readable");

    assert!(
        opening.contains(&format!("recorded_at = \"{PINNED_RECORDED_AT}\"")),
        "the event must record the instant the caller supplied, not a clock read: {opening}"
    );
    assert_eq!(
        pinned().to_string(),
        PINNED_RECORDED_AT,
        "the pinned instant renders the accepted RFC 3339 shape"
    );
}

#[test]
fn the_same_pinned_instant_makes_an_idempotent_retry_byte_exact() {
    let fixture = Fixture::new("pinned-retry");
    let steward = steward_with_one_open_case(&fixture);
    let event = case_directory(&steward).join(support::CASE_OPENED_AT_1);
    let first = fs::read(&event).expect("opening event should be readable");

    // At the process boundary a retry can only be compared against an instant recovered from the
    // first run. Supplying the instant makes the byte-for-byte retry comparison ADR 0009 relies
    // on a direct assertion.
    let proposal = fixture.root.join("open-case.toml");
    case::open(&steward, &proposal, &fixture.location(), pinned(), false)
        .expect("an exact retry should succeed idempotently");

    assert_eq!(
        first,
        fs::read(&event).expect("opening event should stay readable"),
        "an exact retry must leave the recorded event byte-identical"
    );
}

#[test]
fn a_public_steward_cannot_open_a_case_and_the_refusal_is_safe() {
    let fixture = Fixture::new("public-steward");
    let steward = fixture.repository("steward", STEWARD_ID, "public");
    fixture.repository("first-consumer", FIRST_PARTICIPANT_ID, "private");
    fixture.repository("second-consumer", SECOND_PARTICIPANT_ID, "public");
    let proposal = fixture.write("open-case.toml", &two_occurrence_proposal());

    // Private dominance is the property FOUNDATIONS §10 makes non-negotiable, and ADR 0004
    // fixes it. Asserting it across the module interface keeps it covered under the
    // library-only build, where no binary exists to spawn.
    let Err(failure) = case::open(&steward, &proposal, &fixture.location(), pinned(), false) else {
        panic!("a public steward must not open a case over a private participant");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal, "{failure}");
    assert!(
        !case_directory(&steward).exists(),
        "a refusal must write nothing"
    );
}

#[test]
fn enrollment_visibility_reaches_the_case_command_as_a_value() {
    let fixture = Fixture::new("enrollment-visibility");
    let steward = support::git_repository(&fixture.root, "steward");
    let enrollment =
        enroll(&steward, "products", Visibility::Private).expect("steward should enroll");

    assert_eq!(enrollment.visibility, Visibility::Private);
    assert!(
        steward.join("reuse-evidence.toml").is_file(),
        "enrollment should have written a marker"
    );
}
