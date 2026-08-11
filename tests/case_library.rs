//! In-process coverage for case behaviour the terminal contract states only indirectly.
//!
//! `tests/case_cli.rs` keeps the terminal contract: argv dispatch, the exit status each
//! `ExitMeaning` maps to, and which stream carries the text. It reaches that contract through a
//! process, which is the right instrument for it and the wrong one for everything else. What is
//! asserted here is what a process states only indirectly:
//!
//! - A terminal *meaning* is observable at the boundary only through its exit status, and status
//!   `1` covers every non-refusal failure. Asserting `ExitMeaning` directly is exact. Doing so
//!   showed that `UnsafeFailure` is not merely uncovered by the case suite but unreachable from
//!   it: `read_steward` collapses a genuine marker read failure into the same refusal it gives a
//!   malformed marker, while `marker_for_enrollment` calls that input an unsafe failure.
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
use reuse_evidence::{ExitMeaning, Visibility, enroll};
use support::TempRoot;

const CASE_ID: &str = "00000000-0000-4000-8000-000000000011";
const STEWARD_ID: &str = "00000000-0000-4000-8000-000000000012";
const FIRST_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000013";
const SECOND_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000014";
const THIRD_PARTICIPANT_ID: &str = "00000000-0000-4000-8000-000000000015";

/// A fixed instant inside the four-digit-year range `recorded_at` accepts.
const PINNED_UNIX_SECONDS: i64 = 1_760_000_000;
const PINNED_RECORDED_AT: &str = "2025-10-09T08:53:20Z";

struct Fixture {
    root: TempRoot,
}

impl Fixture {
    fn new(name: &str) -> Self {
        Self {
            root: TempRoot::new(&format!("case-library-{name}")),
        }
    }

    fn repository(&self, name: &str, repository_id: &str, visibility: &str) -> PathBuf {
        let repository = support::git_repository(&self.root, name);
        support::enrollment_marker(&repository, repository_id, visibility);
        repository
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

#[test]
fn one_unreadable_marker_is_an_unsafe_failure_to_enrollment_and_a_refusal_to_the_case_surface() {
    let fixture = Fixture::new("marker-classification");

    // `marker::read` reports four outcomes; its callers assign the meanings. `enroll` separates
    // a genuine read failure from a malformed marker, so it is the only surface from which
    // `ExitMeaning::UnsafeFailure` is reachable with this input.
    let enrolling = fixture.repository("enrolling", STEWARD_ID, "private");
    make_marker_unreadable(&enrolling);
    let Err(enrollment_failure) = enroll(&enrolling, "products", Visibility::Private) else {
        panic!("enrolling over an unreadable marker must fail");
    };
    assert_eq!(
        enrollment_failure.meaning(),
        ExitMeaning::UnsafeFailure,
        "{enrollment_failure}"
    );
    assert_eq!(
        enrollment_failure.meaning().status(),
        1,
        "the unsafe-failure meaning maps to status 1"
    );

    // `case::read_steward` collapses every non-supported outcome into one refusal, so the same
    // input reaches the case surface as a safe refusal. Nothing has been written at this point,
    // so the no-write guarantee a refusal carries is truthful — but the two surfaces disagree
    // about what an unreadable marker means, and this pins that divergence rather than
    // asserting either answer is the intended one.
    let steward = fixture.repository("steward", STEWARD_ID, "private");
    let proposal = fixture.write("open-case.toml", &two_occurrence_proposal());
    make_marker_unreadable(&steward);
    let Err(case_failure) = case::open(&steward, &proposal, &fixture.location(), pinned(), false)
    else {
        panic!("opening a case under an unreadable marker must fail");
    };
    assert_eq!(
        case_failure.meaning(),
        ExitMeaning::Refusal,
        "{case_failure}"
    );
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
