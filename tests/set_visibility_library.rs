use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reuse_evidence::{ExitMeaning, Visibility, set_visibility};

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
            "reuse-evidence-library-visibility-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(root.join(".git")).expect("repository fixture should be creatable");
        fs::write(root.join(".git/HEAD"), b"ref: refs/heads/main\n")
            .expect("repository fixture should contain recognizable Git metadata");
        Self { root }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn library_set_visibility_validates_cases_without_the_cli_surface() {
    let fixture = Fixture::new();
    let marker_path = fixture.root.join("reuse-evidence.toml");
    let marker = b"schema_version = 1\nrepository_id = \"00000000-0000-4000-8000-000000000012\"\necosystem_id = \"products\"\nvisibility = \"private\"\n";
    fs::write(&marker_path, marker).expect("marker fixture should be writable");
    let case_id = "00000000-0000-4000-8000-000000000011";
    let case_directory = fixture.root.join("reuse-evidence/cases").join(case_id);
    fs::create_dir_all(&case_directory).expect("case fixture should be creatable");
    fs::write(
        case_directory.join("0001-case-opened.toml"),
        format!(
            "schema_version = 1\nsequence = 1\nevent_id = \"00000000-0000-4000-8000-000000000099\"\nevent_type = \"case_opened\"\nrecorded_at = \"2026-08-11T00:00:00Z\"\ncase_id = \"{case_id}\"\nresponsibility = \"preserve private evidence\"\nsteward_repository_id = \"00000000-0000-4000-8000-000000000012\"\nprivacy = \"private\"\n\n[[occurrences]]\nrepository_id = \"00000000-0000-4000-8000-000000000013\"\nconsumer = \"first-consumer\"\nindependence = \"separate owner\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"1111111\"\n\n[[occurrences]]\nrepository_id = \"00000000-0000-4000-8000-000000000014\"\nconsumer = \"second-consumer\"\nindependence = \"separate lifecycle\"\n\n[[occurrences.evidence]]\nkind = \"commit\"\nreference = \"2222222\"\n"
        ),
    )
    .expect("opening event fixture should be writable");

    let error = set_visibility(&fixture.root, Visibility::Public)
        .expect_err("a public-ward transition must validate steward-local cases");

    assert_eq!(error.meaning(), ExitMeaning::Refusal);
    assert_eq!(
        error.to_string(),
        format!(
            "refusal: repository `00000000-0000-4000-8000-000000000012` cannot be made public while it stewards private case `{case_id}`\nresolution: keep the repository private while it stewards case `{case_id}`; version 0.1 does not support stewardship transfer"
        )
    );
    assert_eq!(
        fs::read(&marker_path).expect("refused marker should remain readable"),
        marker,
        "the library path must preserve the marker byte-for-byte"
    );
}
