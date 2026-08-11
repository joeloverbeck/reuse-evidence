//! Library-only coverage for the private-case visibility guard.
//!
//! The command-line integration tests reach the guard through the built binary and therefore
//! only run under the `cli` feature. This test drives `set_visibility` directly so the
//! supported library-only build keeps the guard covered.

mod support;

use std::fs;

use reuse_evidence::{ExitMeaning, Visibility, set_visibility};
use support::TempRoot;

const STEWARD_ID: &str = "00000000-0000-4000-8000-000000000012";
const CASE_ID: &str = "00000000-0000-4000-8000-000000000011";

struct Fixture {
    root: TempRoot,
}

impl Fixture {
    fn new() -> Self {
        let root = TempRoot::new("library-visibility");
        support::git_repository_at(&root);
        Self { root }
    }
}

#[test]
fn library_set_visibility_validates_cases_without_the_cli_surface() {
    let fixture = Fixture::new();
    let marker_path = fixture.root.join("reuse-evidence.toml");
    let marker = format!(
        "schema_version = 1\nrepository_id = \"{STEWARD_ID}\"\necosystem_id = \"products\"\nvisibility = \"private\"\n"
    );
    fs::write(&marker_path, &marker).expect("marker fixture should be writable");
    let case_directory = fixture.root.join("reuse-evidence/cases").join(CASE_ID);
    fs::create_dir_all(&case_directory).expect("case fixture should be creatable");
    fs::write(
        case_directory.join(support::CASE_OPENED_AT_1),
        format!(
            "schema_version = 1\nsequence = 1\nevent_id = \"00000000-0000-4000-8000-000000000099\"\nevent_type = \"case_opened\"\nrecorded_at = \"2026-08-11T00:00:00Z\"\ncase_id = \"{CASE_ID}\"\nresponsibility = \"preserve private evidence\"\nsteward_repository_id = \"{STEWARD_ID}\"\nprivacy = \"private\"\n\n[[occurrences]]\nrepository_id = \"00000000-0000-4000-8000-000000000013\"\nconsumer = \"first-consumer\"\nindependence = \"separate owner\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\n\n[[occurrences]]\nrepository_id = \"00000000-0000-4000-8000-000000000014\"\nconsumer = \"second-consumer\"\nindependence = \"separate lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"2222222\"\n"
        ),
    )
    .expect("opening event fixture should be writable");

    let error = set_visibility(&fixture.root, Visibility::Public)
        .expect_err("a public-ward transition must validate steward-local cases");

    assert_eq!(error.meaning(), ExitMeaning::Refusal);
    assert_eq!(
        error.to_string(),
        format!(
            "refusal: repository `{STEWARD_ID}` cannot become public while it stewards private case `{CASE_ID}`\nresolution: keep the repository private while it stewards that case; version 0.1 does not support stewardship transfer"
        )
    );
    assert_eq!(
        fs::read_to_string(&marker_path).expect("refused marker should remain readable"),
        marker,
        "the library path must preserve the marker byte-for-byte"
    );
}
