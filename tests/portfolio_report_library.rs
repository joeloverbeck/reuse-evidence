//! In-process coverage for what the portfolio report says about its own inputs.
//!
//! `PORTFOLIO-PRIVACY-AND-STEWARDSHIP.md` §9 makes the user-local state file
//! disposable and rebuildable, so an undecodable one cannot refuse a read-only
//! report. It still changes what the report is able to say — every repository
//! appears as new, and a genuine move, visibility change or identity
//! substitution goes unreported for that run — so the report names the
//! condition rather than rebuilding in silence.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use reuse_evidence::portfolio::{self, PortfolioLocation};
use support::TempRoot;

const REPOSITORY_ID: &str = "00000000-0000-4000-8000-000000000021";

struct Fixture {
    root: TempRoot,
}

impl Fixture {
    /// One enrolled repository beneath an explicit root, with a user-local
    /// state directory that no other test shares.
    fn new(name: &str) -> Self {
        let root = TempRoot::new(name);
        let repository = support::git_repository(&root, "enrolled");
        support::enrollment_marker(&repository, REPOSITORY_ID, "public");
        Self { root }
    }

    fn state_directory(&self) -> PathBuf {
        self.root.join("state")
    }

    /// Writes `bytes` where the report expects its previous observation.
    fn write_state(&self, bytes: &[u8]) -> PathBuf {
        let state_path = self
            .state_directory()
            .join("reuse-evidence")
            .join("portfolio.toml");
        fs::create_dir_all(
            state_path
                .parent()
                .expect("the state path always has a parent"),
        )
        .expect("state directory should be creatable");
        fs::write(&state_path, bytes).expect("state fixture should be writable");
        state_path
    }

    fn location(&self) -> PortfolioLocation {
        PortfolioLocation::from_user_directories(
            vec![self.root.to_path_buf()],
            Some(Path::new(self.root.path())),
            Some(&self.state_directory()),
        )
    }
}

#[test]
fn a_malformed_state_file_is_reported_and_rebuilt_rather_than_silently_discarded() {
    let fixture = Fixture::new("portfolio-state-malformed");
    let state_path = fixture.write_state(b"this is not a portfolio state\n");

    let report = portfolio::report(&fixture.location())
        .expect("a disposable state file cannot refuse a read-only report");

    assert!(
        report.contains(&format!(
            "unreadable portfolio state:\n- state: {}\n",
            state_path.display()
        )),
        "{report}"
    );
    assert!(
        report.contains("  consequence: every enrolled repository is reported as new\n"),
        "{report}"
    );
    assert!(report.contains("new repositories:"), "{report}");

    // Rebuilt, so the next run has a previous observation again and says nothing.
    let second = portfolio::report(&fixture.location())
        .expect("the rebuilt state should be readable on the next run");
    assert!(!second.contains("unreadable portfolio state:"), "{second}");
    assert!(!second.contains("new repositories:"), "{second}");
}

#[test]
fn a_non_utf8_state_file_is_reported_with_the_same_shape() {
    let fixture = Fixture::new("portfolio-state-non-utf8");
    let state_path = fixture.write_state(&[0xff, 0xfe, 0x00]);

    let report = portfolio::report(&fixture.location())
        .expect("undecodable bytes cannot refuse a read-only report");

    assert!(
        report.contains(&format!(
            "unreadable portfolio state:\n- state: {}\n",
            state_path.display()
        )),
        "{report}"
    );
}

#[test]
fn a_readable_state_file_reports_no_condition() {
    let fixture = Fixture::new("portfolio-state-readable");

    let first =
        portfolio::report(&fixture.location()).expect("the first run should write fresh state");
    assert!(!first.contains("unreadable portfolio state:"), "{first}");

    let second = portfolio::report(&fixture.location())
        .expect("the second run should read the state the first wrote");
    assert!(!second.contains("unreadable portfolio state:"), "{second}");
}
