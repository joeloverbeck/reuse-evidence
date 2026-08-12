#![forbid(unsafe_code)]

//! Evidence-gated reuse decisions for repository portfolios.

#[cfg(feature = "cli")]
pub mod case;
#[cfg(not(feature = "cli"))]
#[allow(dead_code, unused_imports)]
mod case;
pub mod marker;
#[cfg(feature = "cli")]
pub mod portfolio;
#[cfg(not(feature = "cli"))]
#[allow(dead_code)]
mod portfolio;

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The marker filename at an enrolled repository's root.
pub const MARKER_FILE: &str = "reuse-evidence.toml";

/// The process-level meaning returned by every command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitMeaning {
    /// The requested operation completed successfully.
    Success,
    /// The operation failed in a way that does not carry a no-write guarantee.
    UnsafeFailure,
    /// The operation declined safely and wrote nothing.
    Refusal,
}

impl ExitMeaning {
    /// Returns the stable process status for this terminal meaning.
    #[must_use]
    pub const fn status(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::UnsafeFailure => 1,
            Self::Refusal => 3,
        }
    }
}

/// A repository's declared visibility.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// The repository is public.
    Public,
    /// The repository is private.
    Private,
}

impl Visibility {
    /// Parses the two visibility values supported by the current marker schema.
    ///
    /// # Errors
    ///
    /// Returns a refusal when `value` is neither `public` nor `private`.
    pub fn parse(value: &str) -> Result<Self, TerminalFailure> {
        match value {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            _ => Err(TerminalFailure::refusal(
                format!("visibility `{value}` is not supported"),
                "use `public` or `private`",
            )),
        }
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => formatter.write_str("public"),
            Self::Private => formatter.write_str("private"),
        }
    }
}

/// The observable effect of an enrollment command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnrollmentEffect {
    /// A new marker was created.
    Created,
    /// The existing marker was validated and preserved.
    Existing,
    /// The existing marker's visibility was deliberately changed.
    VisibilityChanged,
    /// The deliberate visibility request already matched the marker.
    VisibilityUnchanged,
}

impl EnrollmentEffect {
    /// The receipt heading this effect prints.
    const fn heading(self) -> &'static str {
        match self {
            Self::Created => "enrolled repository",
            Self::Existing => "existing enrollment",
            Self::VisibilityChanged => "changed repository visibility",
            Self::VisibilityUnchanged => "repository visibility unchanged",
        }
    }
}

/// The observable result of enrollment.
#[derive(Debug, Eq, PartialEq)]
pub struct Enrollment {
    /// Whether this invocation created or preserved the marker.
    pub effect: EnrollmentEffect,
    /// The path of the enrolled repository's marker.
    pub marker_path: PathBuf,
    /// The repository's opaque identity.
    pub repository_id: Uuid,
    /// The declared ecosystem identity.
    pub ecosystem_id: String,
    /// The declared repository visibility.
    pub visibility: Visibility,
}

/// The `enroll` and `set-visibility` receipt, in the shape the terminal prints.
///
/// Every other command renders its receipt from a value that implements
/// `Display`, which is what lets ADR 0016's in-process suites assert receipt
/// prose without spawning the binary. These two built theirs with `println!`
/// inside `main`, so their success text was reachable only through a process.
impl fmt::Display for Enrollment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", self.effect.heading())?;
        writeln!(formatter, "marker: {}", self.marker_path.display())?;
        writeln!(formatter, "repository_id: {}", self.repository_id)?;
        writeln!(formatter, "ecosystem_id: {}", self.ecosystem_id)?;
        writeln!(formatter, "visibility: {}", self.visibility)
    }
}

/// A classified failure carrying the terminal contract for an owned command.
#[derive(Debug, Eq, PartialEq)]
pub struct TerminalFailure {
    condition: String,
    kind: TerminalFailureKind,
}

#[derive(Debug, Eq, PartialEq)]
enum TerminalFailureKind {
    UnsafeFailure,
    Refusal { resolution: String },
}

impl TerminalFailure {
    /// Creates a safe refusal with its required resolution.
    #[must_use]
    pub fn refusal(condition: impl Into<String>, resolution: impl Into<String>) -> Self {
        Self {
            condition: condition.into(),
            kind: TerminalFailureKind::Refusal {
                resolution: resolution.into(),
            },
        }
    }

    /// Creates a failure that does not carry a no-write guarantee.
    #[must_use]
    pub fn unsafe_failure(condition: impl Into<String>) -> Self {
        Self {
            condition: condition.into(),
            kind: TerminalFailureKind::UnsafeFailure,
        }
    }

    /// Returns this failure's process-level meaning.
    #[must_use]
    pub const fn meaning(&self) -> ExitMeaning {
        match self.kind {
            TerminalFailureKind::UnsafeFailure => ExitMeaning::UnsafeFailure,
            TerminalFailureKind::Refusal { .. } => ExitMeaning::Refusal,
        }
    }
}

impl fmt::Display for TerminalFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TerminalFailureKind::UnsafeFailure => {
                write!(formatter, "unsafe failure: {}", self.condition)
            }
            TerminalFailureKind::Refusal { resolution } => write!(
                formatter,
                "refusal: {}\nresolution: {resolution}",
                self.condition
            ),
        }
    }
}

impl std::error::Error for TerminalFailure {}

/// Enrolls the repository containing `working_directory`.
///
/// The repository root is the nearest ancestor recognized by
/// [`marker::is_repository_root`]. An existing valid marker is preserved.
///
/// # Errors
///
/// Returns a refusal when `working_directory` cannot identify a repository.
/// Returns an unsafe failure when marker encoding or writing fails.
pub fn enroll(
    working_directory: &Path,
    ecosystem_id: &str,
    visibility: Visibility,
) -> Result<Enrollment, TerminalFailure> {
    enroll_with_expected_repository_id(working_directory, ecosystem_id, visibility, None)
}

/// Enrolls a repository while optionally verifying an existing identity.
///
/// The expected identity is only a guard for re-enrollment. It is never used to
/// assign the identity of a fresh enrollment.
///
/// # Errors
///
/// Returns a refusal under the same conditions as [`enroll`], when an existing
/// marker has a different identity, or when an identity is supplied for a fresh
/// enrollment. Fresh enrollment also refuses when steward-local case storage is
/// not empty. Returns an unsafe failure when marker encoding or writing fails.
pub fn enroll_with_expected_repository_id(
    working_directory: &Path,
    ecosystem_id: &str,
    visibility: Visibility,
    expected_repository_id: Option<Uuid>,
) -> Result<Enrollment, TerminalFailure> {
    let repository_root = find_repository_root(working_directory)?;
    let marker_path = repository_root.join(MARKER_FILE);
    let malformed_resolution = format!(
        "restore a complete valid version {} marker before rerunning enrollment",
        marker::SUPPORTED_SCHEMA_VERSION
    );
    if let Some(marker) = marker_for_enrollment(&repository_root, &malformed_resolution)? {
        if marker.visibility() != visibility {
            return Err(TerminalFailure::refusal(
                format!(
                    "existing marker declares visibility `{}`, but enrollment requested `{visibility}`",
                    marker.visibility()
                ),
                format!(
                    "use `set-visibility --visibility {visibility}` for a deliberate visibility change"
                ),
            ));
        }
        if marker.ecosystem_id() != ecosystem_id {
            return Err(TerminalFailure::refusal(
                format!(
                    "existing marker declares ecosystem identity `{}`, but enrollment requested `{ecosystem_id}`",
                    marker.ecosystem_id()
                ),
                format!(
                    "rerun with `--ecosystem-id {}`; changing ecosystem identity is not implemented",
                    marker.ecosystem_id()
                ),
            ));
        }
        if let Some(expected_repository_id) = expected_repository_id
            && marker.repository_id() != expected_repository_id
        {
            return Err(TerminalFailure::refusal(
                format!(
                    "existing marker declares repository identity `{}`, but enrollment expected `{expected_repository_id}`",
                    marker.repository_id()
                ),
                format!(
                    "rerun with `--expected-repository-id {}` or omit the identity guard",
                    marker.repository_id()
                ),
            ));
        }
        return Ok(Enrollment {
            effect: EnrollmentEffect::Existing,
            marker_path,
            repository_id: marker.repository_id(),
            ecosystem_id: marker.ecosystem_id().to_owned(),
            visibility: marker.visibility(),
        });
    }

    if expected_repository_id.is_some() {
        return Err(TerminalFailure::refusal(
            "a repository identity cannot be assigned during fresh enrollment",
            "omit `--expected-repository-id`; fresh enrollment generates the opaque identity",
        ));
    }
    refuse_fresh_enrollment_over_steward_case_storage(&repository_root)?;

    let repository_id = Uuid::new_v4();
    let marker = marker::Marker::new(repository_id, ecosystem_id.to_owned(), visibility);
    let marker_bytes = toml::to_string(&marker).map_err(|error| {
        TerminalFailure::unsafe_failure(format!("could not encode the repository marker: {error}"))
    })?;
    create_file_atomically(&marker_path, marker_bytes.as_bytes())?;

    Ok(Enrollment {
        effect: EnrollmentEffect::Created,
        marker_path,
        repository_id,
        ecosystem_id: ecosystem_id.to_owned(),
        visibility,
    })
}

/// Deliberately changes an enrolled repository's declared visibility.
///
/// # Errors
///
/// Returns a refusal when the working directory is not inside a repository,
/// the marker is absent or invalid.
/// Returns an unsafe failure when the replacement marker cannot be published.
pub fn set_visibility(
    working_directory: &Path,
    visibility: Visibility,
) -> Result<Enrollment, TerminalFailure> {
    let repository_root = find_repository_root(working_directory)?;
    let marker_path = repository_root.join(MARKER_FILE);
    let malformed_resolution = format!(
        "restore a complete valid version {} marker before changing visibility",
        marker::SUPPORTED_SCHEMA_VERSION
    );
    let Some(mut marker) = marker_for_enrollment(&repository_root, &malformed_resolution)? else {
        return Err(TerminalFailure::refusal(
            format!(
                "repository is not enrolled because `{}` does not exist",
                marker_path.display()
            ),
            "run `enroll` before changing visibility",
        ));
    };
    if marker.visibility() == visibility {
        return Ok(Enrollment {
            effect: EnrollmentEffect::VisibilityUnchanged,
            marker_path,
            repository_id: marker.repository_id(),
            ecosystem_id: marker.ecosystem_id().to_owned(),
            visibility: marker.visibility(),
        });
    }
    let _marker_lock = if visibility == Visibility::Public {
        let marker_lock = lock_repository_marker(&repository_root)?;
        let Some(current_marker) = marker_for_enrollment(&repository_root, &malformed_resolution)?
        else {
            return Err(TerminalFailure::refusal(
                format!(
                    "repository is not enrolled because `{}` does not exist",
                    marker_path.display()
                ),
                "run `enroll` before changing visibility",
            ));
        };
        marker = current_marker;
        if marker.visibility() == visibility {
            return Ok(Enrollment {
                effect: EnrollmentEffect::VisibilityUnchanged,
                marker_path,
                repository_id: marker.repository_id(),
                ecosystem_id: marker.ecosystem_id().to_owned(),
                visibility: marker.visibility(),
            });
        }
        Some(marker_lock)
    } else {
        None
    };
    if visibility == Visibility::Public
        && let Some(case_id) =
            case::private_case_stewarded_by(&repository_root, marker.repository_id())?
    {
        return Err(TerminalFailure::refusal(
            format!(
                "repository `{}` cannot become public while it stewards private case `{case_id}`",
                marker.repository_id()
            ),
            "keep the repository private while it stewards that case; version 0.1 does not support stewardship transfer",
        ));
    }
    marker.set_visibility(visibility);
    let replacement = toml::to_string(&marker).map_err(|error| {
        TerminalFailure::unsafe_failure(format!("could not encode the repository marker: {error}"))
    })?;
    replace_file_atomically(&marker_path, replacement.as_bytes()).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "could not atomically replace `{}`: {error}",
            marker_path.display()
        ))
    })?;

    Ok(Enrollment {
        effect: EnrollmentEffect::VisibilityChanged,
        marker_path,
        repository_id: marker.repository_id(),
        ecosystem_id: marker.ecosystem_id().to_owned(),
        visibility: marker.visibility(),
    })
}

fn marker_for_enrollment(
    repository_root: &Path,
    malformed_resolution: &str,
) -> Result<Option<marker::Marker>, TerminalFailure> {
    match marker::read(repository_root) {
        Some(marker::MarkerRead::Supported(marker)) => Ok(Some(marker)),
        Some(marker::MarkerRead::UnsupportedSchemaVersion(marker)) => {
            let schema_version = marker.schema_version();
            Err(TerminalFailure::refusal(
                format!("marker schema version `{schema_version}` is not supported"),
                format!(
                    "use a reuse-evidence version that supports marker schema version `{schema_version}`"
                ),
            ))
        }
        Some(marker::MarkerRead::Unreadable(marker)) if marker.is_read_failure() => {
            Err(TerminalFailure::unsafe_failure(format!(
                "could not read `{}`: {marker}",
                marker.path().display()
            )))
        }
        Some(marker::MarkerRead::Unreadable(marker)) => Err(malformed_marker_error(
            marker.path(),
            &marker,
            malformed_resolution,
        )),
        None => Ok(None),
    }
}

fn malformed_marker_error(
    marker_path: &Path,
    error: &dyn fmt::Display,
    resolution: &str,
) -> TerminalFailure {
    TerminalFailure::refusal(
        format!(
            "existing marker `{}` is malformed: {error}",
            marker_path.display()
        ),
        resolution,
    )
}

fn refuse_fresh_enrollment_over_steward_case_storage(
    repository_root: &Path,
) -> Result<(), TerminalFailure> {
    let case_directory = repository_root.join(case::cases_root());
    let metadata = match fs::symlink_metadata(&case_directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(nonempty_case_storage_enrollment_refusal(&case_directory)),
    };
    if metadata.is_dir()
        && fs::read_dir(&case_directory).is_ok_and(|mut entries| entries.next().is_none())
    {
        return Ok(());
    }
    Err(nonempty_case_storage_enrollment_refusal(&case_directory))
}

fn nonempty_case_storage_enrollment_refusal(case_directory: &Path) -> TerminalFailure {
    TerminalFailure::refusal(
        format!(
            "steward-local case directory `{}` is not empty in an unenrolled repository; fresh enrollment would mint a new repository identity and orphan every case recorded under the previous identity",
            case_directory.display()
        ),
        "restore the committed `reuse-evidence.toml` marker that records those cases' steward identity, or set the case directory aside if this repository should not steward them",
    )
}

pub(crate) fn lock_repository_marker(repository_root: &Path) -> Result<File, TerminalFailure> {
    let marker_path = repository_root.join(MARKER_FILE);
    let marker = File::open(&marker_path).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "repository marker `{}` cannot be opened for visibility serialization: {error}",
                marker_path.display()
            ),
            "restore a readable enrolled repository marker before retrying",
        )
    })?;
    marker.lock().map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "repository marker `{}` cannot serialize case opening and visibility changes: {error}",
                marker_path.display()
            ),
            "make the enrolled repository marker lockable before retrying",
        )
    })?;
    Ok(marker)
}

pub(crate) enum CreateFileOutcome {
    Created,
    Occupied,
}

pub(crate) fn create_file_atomically_if_absent(
    path: &Path,
    bytes: &[u8],
) -> Result<CreateFileOutcome, TerminalFailure> {
    let temporary_path = temporary_path_for(path);
    let result = (|| {
        write_temporary_file(&temporary_path, bytes)?;
        if let Err(error) = fs::hard_link(&temporary_path, path) {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                fs::remove_file(&temporary_path)?;
                return Ok(CreateFileOutcome::Occupied);
            }
            return Err(error);
        }
        sync_parent(path)?;
        fs::remove_file(&temporary_path)?;
        sync_parent(path)?;
        Ok(CreateFileOutcome::Created)
    })();
    match result {
        Ok(outcome) => Ok(outcome),
        Err(error) => {
            let _ = fs::remove_file(&temporary_path);
            Err(TerminalFailure::unsafe_failure(format!(
                "could not atomically create `{}`: {error}",
                path.display()
            )))
        }
    }
}

pub(crate) fn create_file_atomically(path: &Path, bytes: &[u8]) -> Result<(), TerminalFailure> {
    match create_file_atomically_if_absent(path, bytes)? {
        CreateFileOutcome::Created => Ok(()),
        CreateFileOutcome::Occupied => Err(TerminalFailure::unsafe_failure(format!(
            "could not atomically create `{}`: target already exists",
            path.display()
        ))),
    }
}

/// Durably replaces `path` with `bytes`, leaving no partial file behind.
///
/// The mechanism is shared; the refusal wording is not. Every caller names the
/// file it was publishing, so this returns the underlying I/O error rather than
/// a classified failure that would have to be worded for all of them.
pub(crate) fn replace_file_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary_path = temporary_path_for(path);
    let result = (|| {
        write_temporary_file(&temporary_path, bytes)?;
        fs::rename(&temporary_path, path)?;
        sync_parent(path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn write_temporary_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn temporary_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(MARKER_FILE);
    path.with_file_name(format!(".{file_name}.{}.tmp", Uuid::new_v4()))
}

fn sync_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn find_repository_root(working_directory: &Path) -> Result<PathBuf, TerminalFailure> {
    let (working_directory, repository_root) =
        locate_repository_root(working_directory).map_err(|error| {
            TerminalFailure::refusal(
                format!(
                    "working directory `{}` cannot be inspected: {error}",
                    working_directory.display()
                ),
                "rerun from an existing directory inside the repository",
            )
        })?;
    repository_root.ok_or_else(|| {
        TerminalFailure::refusal(
            format!(
                "`{}` is not inside a repository root",
                working_directory.display()
            ),
            "rerun inside a repository containing `.git`",
        )
    })
}

pub(crate) fn locate_repository_root(
    working_directory: &Path,
) -> std::io::Result<(PathBuf, Option<PathBuf>)> {
    let working_directory = working_directory.canonicalize()?;
    let repository_root = working_directory
        .ancestors()
        .find(|candidate| marker::is_repository_root(candidate))
        .map(Path::to_path_buf);
    Ok((working_directory, repository_root))
}
