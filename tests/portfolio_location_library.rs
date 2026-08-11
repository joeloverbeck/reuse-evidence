//! In-process coverage for the degraded case projection ADR 0015 preserves.
//!
//! Before the portfolio location became a value, "no user-local directory
//! resolves" could only be said by spawning the binary with the platform's
//! variables removed, which is why `run_without_portfolio_configuration` in
//! `tests/case_cli.rs` exists and is used 33 times. Those tests still own the
//! terminal contract. This one asserts the same behaviour across the module
//! interface, and asserts the contrast the ADR refuses to collapse: the same
//! absence that makes `portfolio` refuse leaves `case list` succeeding with
//! unknown conditions.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use reuse_evidence::case::RecordedInstant;
use reuse_evidence::portfolio::PortfolioLocation;
use reuse_evidence::{Visibility, case, enroll, portfolio};

const CASE_ID: &str = "00000000-0000-4000-8000-000000000011";

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "reuse-evidence-portfolio-location-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("portfolio fixture should be creatable");
        Self { root }
    }

    fn repository(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::create_dir_all(path.join(".git")).expect("repository fixture should be creatable");
        fs::write(path.join(".git/HEAD"), b"ref: refs/heads/main\n")
            .expect("repository fixture should contain recognizable Git metadata");
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn files_beneath(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// Opens one case whose participants resolve under an explicit portfolio root,
/// and returns the steward repository.
fn steward_with_one_open_case(fixture: &Fixture) -> PathBuf {
    let steward = fixture.repository("steward");
    let first = fixture.repository("first-participant");
    let second = fixture.repository("second-participant");

    enroll(&steward, "products", Visibility::Public).expect("steward should enroll");
    let first = enroll(&first, "products", Visibility::Public).expect("participant should enroll");
    let second =
        enroll(&second, "products", Visibility::Public).expect("participant should enroll");

    let proposal_path = fixture.root.join("open.toml");
    fs::write(
        &proposal_path,
        format!(
            "case_id = \"{CASE_ID}\"\nresponsibility = \"normalize durable event identities\"\n\n[[occurrences]]\nrepository_id = \"{}\"\nconsumer = \"rust-release-tool\"\nindependence = \"separate release lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\npath = \"src/event.rs\"\n\n[[occurrences]]\nrepository_id = \"{}\"\nconsumer = \"web-deployment-tool\"\nindependence = \"independent npm workspace and owner\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"2222222\"\npath = \"packages/events/src/id.ts\"\n",
            first.repository_id, second.repository_id
        ),
    )
    .expect("open proposal should be writable");

    let configured =
        PortfolioLocation::from_user_directories(vec![fixture.root.clone()], None, None);
    case::open(
        &steward,
        &proposal_path,
        &configured,
        RecordedInstant::now().expect("system clock should be readable"),
        false,
    )
    .expect("opening a case under an explicit root should succeed");

    steward
}

#[test]
fn case_list_reports_unknown_conditions_when_no_user_local_directory_resolves() {
    let fixture = Fixture::new();
    let steward = steward_with_one_open_case(&fixture);
    let before = files_beneath(&fixture.root);

    // Both directories undeterminable is what an unset `XDG_CONFIG_HOME` with no
    // `HOME` means. It reaches the module as a value rather than a process.
    let undeterminable = PortfolioLocation::from_user_directories(Vec::new(), None, None);
    let listed = case::list(&steward, &undeterminable)
        .expect("listing must succeed when no user-local directory resolves");

    let rendered = listed.to_string();
    assert!(
        rendered.contains("privacy_conflicted: unknown\n  stale: unknown\n"),
        "{rendered}"
    );
    assert!(
        rendered.contains("portfolio conditions unavailable:"),
        "{rendered}"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "listing without a user-local directory must write nothing"
    );
}

#[test]
fn case_list_derives_conditions_once_the_same_location_supplies_roots() {
    let fixture = Fixture::new();
    let steward = steward_with_one_open_case(&fixture);

    let configured =
        PortfolioLocation::from_user_directories(vec![fixture.root.clone()], None, None);
    let listed = case::list(&steward, &configured).expect("listing under a root should succeed");

    let rendered = listed.to_string();
    assert!(
        rendered.contains("privacy_conflicted: false\n  stale: false\n"),
        "{rendered}"
    );
    assert!(
        !rendered.contains("portfolio conditions unavailable"),
        "{rendered}"
    );
}

#[test]
fn portfolio_refuses_the_absence_that_case_list_tolerates() {
    let fixture = Fixture::new();
    let steward = steward_with_one_open_case(&fixture);
    let undeterminable = PortfolioLocation::from_user_directories(Vec::new(), None, None);

    // The same value: one command refuses it, the other degrades. ADR 0015
    // keeps that divergence, so both halves are asserted from one location.
    let Err(refusal) = portfolio::report(&undeterminable) else {
        panic!("portfolio must refuse when no configuration directory resolves");
    };
    assert!(
        format!("{refusal}")
            .contains("the user-local configuration directory cannot be determined"),
        "{refusal}"
    );
    assert!(
        case::list(&steward, &undeterminable).is_ok(),
        "case list must still succeed on the same absence"
    );
}
