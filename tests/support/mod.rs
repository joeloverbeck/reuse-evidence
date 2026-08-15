//! Fixture scaffolding shared by every test suite in this directory.
//!
//! Every suite uses the same invariant fixture core: a uniquely named temporary root removed on
//! drop, directories recognizable as Git repositories, and enrolled-repository markers. Only
//! that core lives here. Portfolio configuration, canonicalization, and the search for a
//! directory outside Git differ per suite by intent and stay with their suite.
//!
//! Integration tests are separate crates, so each one compiles its own copy of this module and
//! uses only part of it.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Recorded case event file names, at the sequence each test places them.
///
/// These are written by hand rather than derived from `case::naming`. Recorded evidence is the
/// hardest compatibility surface in `CONSUMER-CONTRACT.md` §3, and ADR 0009 fixes events as
/// sequence-numbered TOML files. A hand-written name pins that layout independently of the code
/// that produces it; a derived one would agree with a naming defect instead of catching it.
///
/// Deliberately invalid names used by negative tests stay inline at their assertion, because
/// they state a violation of this layout rather than an instance of it.
pub const CASE_OPENED_AT_1: &str = "0001-case-opened.toml";
pub const OCCURRENCE_APPENDED_AT_2: &str = "0002-occurrence-appended.toml";
pub const EARLY_REVIEW_AUTHORIZED_AT_2: &str = "0002-early-review-authorized.toml";
pub const VERIFICATION_RECORDED_AT_3: &str = "0003-verification-recorded.toml";
pub const OCCURRENCE_APPENDED_AT_3: &str = "0003-occurrence-appended.toml";
pub const REUSE_DECISION_ACCEPTED_AT_4: &str = "0004-reuse-decision-accepted.toml";
pub const VERIFICATION_RECORDED_AT_5: &str = "0005-verification-recorded.toml";

/// A uniquely named temporary directory that is removed when the fixture drops.
///
/// The name carries the caller's label, the process id, and a nanosecond nonce so concurrent
/// test binaries and repeated runs cannot collide.
///
/// It dereferences to [`Path`] so a suite can hold one wherever it previously held a `PathBuf`
/// without rewriting its call sites.
pub struct TempRoot {
    pub root: PathBuf,
}

impl Deref for TempRoot {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.root
    }
}

impl AsRef<Path> for TempRoot {
    fn as_ref(&self) -> &Path {
        &self.root
    }
}

impl TempRoot {
    /// Creates the root beneath the platform temporary directory.
    pub fn new(name: &str) -> Self {
        Self::beneath(&std::env::temp_dir(), name)
    }

    /// Creates the root beneath `parent`, for suites that need a specific location.
    pub fn beneath(parent: &Path, name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let root = parent.join(format!(
            "reuse-evidence-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("fixture root should be creatable");
        Self { root }
    }

    pub fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The invariant fixture core shared by every integration-test crate.
pub struct Fixture {
    pub root: TempRoot,
}

impl Fixture {
    /// Creates a fixture rooted beneath the platform temporary directory.
    pub fn from_label(name: &str) -> Self {
        Self {
            root: TempRoot::new(name),
        }
    }

    /// Creates a fixture beneath a suite-selected parent directory.
    pub fn beneath(parent: &Path, name: &str) -> Self {
        Self {
            root: TempRoot::beneath(parent, name),
        }
    }

    /// Creates a Git repository beneath this fixture's temporary root.
    pub fn git_repository(&self, name: &str) -> PathBuf {
        git_repository(&self.root, name)
    }

    /// Creates and enrolls a Git repository beneath this fixture's temporary root.
    pub fn enrolled_repository(
        &self,
        name: &str,
        repository_id: &str,
        visibility: &str,
    ) -> PathBuf {
        let repository = self.git_repository(name);
        enrollment_marker(&repository, repository_id, visibility);
        repository
    }
}

/// Creates `parent/name` as a directory this crate recognizes as a Git repository.
pub fn git_repository(parent: &Path, name: &str) -> PathBuf {
    let repository = parent.join(name);
    git_repository_at(&repository);
    repository
}

/// Makes `repository` itself recognizable as a Git repository.
pub fn git_repository_at(repository: &Path) {
    fs::create_dir_all(repository.join(".git")).expect("repository fixture should be creatable");
    fs::write(
        repository.join(".git").join("HEAD"),
        b"ref: refs/heads/main\n",
    )
    .expect("repository fixture should contain recognizable Git metadata");
}

/// Records one symlink-mode path in a version-2 Git index fixture.
#[cfg(windows)]
pub fn git_index_with_symlink(repository: &Path, relative_path: &str) {
    const FIXED_ENTRY_LENGTH: usize = 62;
    let path = relative_path.as_bytes();
    assert!(
        path.len() < 0x0fff,
        "the fixture path should fit Git index flags"
    );
    let entry_length = FIXED_ENTRY_LENGTH + path.len() + 1;
    let padding = (8 - entry_length % 8) % 8;
    let mut index = Vec::with_capacity(12 + entry_length + padding + 20);
    index.extend_from_slice(b"DIRC");
    index.extend_from_slice(&2_u32.to_be_bytes());
    index.extend_from_slice(&1_u32.to_be_bytes());

    let mut entry = vec![0; FIXED_ENTRY_LENGTH];
    entry[24..28].copy_from_slice(&0o120_000_u32.to_be_bytes());
    let path_length = u16::try_from(path.len()).expect("the fixture path length should fit u16");
    entry[60..62].copy_from_slice(&path_length.to_be_bytes());
    index.extend_from_slice(&entry);
    index.extend_from_slice(path);
    index.push(0);
    index.resize(index.len() + padding, 0);
    index.extend_from_slice(&[0; 20]);
    fs::write(repository.join(".git/index"), index)
        .expect("the Git symlink index fixture should be writable");
}

/// Writes an enrolled repository marker into `repository`.
pub fn enrollment_marker(repository: &Path, repository_id: &str, visibility: &str) {
    fs::write(
        repository.join("reuse-evidence.toml"),
        format!(
            "schema_version = 1\nrepository_id = \"{repository_id}\"\necosystem_id = \"products\"\nvisibility = \"{visibility}\"\n"
        ),
    )
    .expect("repository fixture should be enrolled");
}

/// Captures every regular file beneath `root` by relative path and byte value.
pub fn files_beneath(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
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

/// Captures every regular file beneath `root` as a sorted relative path and
/// byte value, so write-free behavior can be asserted through the filesystem
/// boundary without depending on traversal order.
///
/// A Cargo build directory sitting directly at `root` is the one exclusion. A
/// case that snapshots this repository would otherwise assert byte-equality
/// over output `cargo` itself writes while the test runs, which states nothing
/// about the code under test and costs gigabytes of reads twice over on a warm
/// checkout. The exclusion is anchored to `root`, so a fixture repository named
/// `target` is still captured whole.
pub fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn collect(root: &Path, directory: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        let mut entries = fs::read_dir(directory)
            .expect("snapshot directory should be readable")
            .map(|entry| entry.expect("snapshot entry should be readable"))
            .collect::<Vec<_>>();
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .expect("snapshot entry type should be readable");
            if file_type.is_dir() {
                if directory == root && entry.file_name() == "target" {
                    continue;
                }
                collect(root, &path, files);
            } else if file_type.is_file() {
                files.push((
                    path.strip_prefix(root)
                        .expect("snapshot path should remain beneath its root")
                        .to_path_buf(),
                    fs::read(&path).expect("snapshot file should be readable"),
                ));
            }
        }
    }

    let mut files = Vec::new();
    collect(root, root, &mut files);
    files
}
