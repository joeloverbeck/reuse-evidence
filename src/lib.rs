#![forbid(unsafe_code)]

//! Evidence-gated reuse decisions for repository portfolios.

pub mod marker;

use std::fmt;
use std::fs::{self, OpenOptions};
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
    pub fn parse(value: &str) -> Result<Self, EnrollmentError> {
        match value {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            _ => Err(EnrollmentError::refusal(
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

/// The terminal failure returned by repository enrollment operations.
pub type EnrollmentError = TerminalFailure;

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
) -> Result<Enrollment, EnrollmentError> {
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
/// enrollment. Returns an unsafe failure when marker encoding or writing fails.
pub fn enroll_with_expected_repository_id(
    working_directory: &Path,
    ecosystem_id: &str,
    visibility: Visibility,
    expected_repository_id: Option<Uuid>,
) -> Result<Enrollment, EnrollmentError> {
    let repository_root = find_repository_root(working_directory)?;
    let marker_path = repository_root.join(MARKER_FILE);
    let malformed_resolution = format!(
        "restore a complete valid version {} marker before rerunning enrollment",
        marker::SUPPORTED_SCHEMA_VERSION
    );
    if let Some(marker) = marker_for_enrollment(&repository_root, &malformed_resolution)? {
        if marker.visibility() != visibility {
            return Err(EnrollmentError::refusal(
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
            return Err(EnrollmentError::refusal(
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
            return Err(EnrollmentError::refusal(
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
        return Err(EnrollmentError::refusal(
            "a repository identity cannot be assigned during fresh enrollment",
            "omit `--expected-repository-id`; fresh enrollment generates the opaque identity",
        ));
    }

    let repository_id = Uuid::new_v4();
    let marker = marker::Marker::new(repository_id, ecosystem_id.to_owned(), visibility);
    let marker_bytes = toml::to_string(&marker).map_err(|error| {
        EnrollmentError::unsafe_failure(format!("could not encode the repository marker: {error}"))
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
) -> Result<Enrollment, EnrollmentError> {
    let repository_root = find_repository_root(working_directory)?;
    let marker_path = repository_root.join(MARKER_FILE);
    let malformed_resolution = format!(
        "restore a complete valid version {} marker before changing visibility",
        marker::SUPPORTED_SCHEMA_VERSION
    );
    let Some(mut marker) = marker_for_enrollment(&repository_root, &malformed_resolution)? else {
        return Err(EnrollmentError::refusal(
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
    marker.set_visibility(visibility);
    let replacement = toml::to_string(&marker).map_err(|error| {
        EnrollmentError::unsafe_failure(format!("could not encode the repository marker: {error}"))
    })?;
    replace_file_atomically(&marker_path, replacement.as_bytes())?;

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
) -> Result<Option<marker::Marker>, EnrollmentError> {
    match marker::read(repository_root) {
        Some(marker::MarkerRead::Supported(marker)) => Ok(Some(marker)),
        Some(marker::MarkerRead::UnsupportedSchemaVersion(marker)) => {
            let schema_version = marker.schema_version();
            Err(EnrollmentError::refusal(
                format!("marker schema version `{schema_version}` is not supported"),
                format!(
                    "use a reuse-evidence version that supports marker schema version `{schema_version}`"
                ),
            ))
        }
        Some(marker::MarkerRead::Unreadable(marker)) if marker.is_read_failure() => {
            Err(EnrollmentError::unsafe_failure(format!(
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
) -> EnrollmentError {
    EnrollmentError::refusal(
        format!(
            "existing marker `{}` is malformed: {error}",
            marker_path.display()
        ),
        resolution,
    )
}

fn create_file_atomically(path: &Path, bytes: &[u8]) -> Result<(), EnrollmentError> {
    let temporary_path = temporary_path_for(path);
    let result = (|| {
        write_temporary_file(&temporary_path, bytes)?;
        fs::hard_link(&temporary_path, path)?;
        sync_parent(path)?;
        fs::remove_file(&temporary_path)?;
        sync_parent(path)?;
        Ok::<(), std::io::Error>(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary_path);
        return Err(EnrollmentError::unsafe_failure(format!(
            "could not atomically create `{}`: {error}",
            path.display()
        )));
    }
    Ok(())
}

fn replace_file_atomically(path: &Path, bytes: &[u8]) -> Result<(), EnrollmentError> {
    let temporary_path = temporary_path_for(path);
    let result = (|| {
        write_temporary_file(&temporary_path, bytes)?;
        fs::rename(&temporary_path, path)?;
        sync_parent(path)?;
        Ok::<(), std::io::Error>(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary_path);
        return Err(EnrollmentError::unsafe_failure(format!(
            "could not atomically replace `{}`: {error}",
            path.display()
        )));
    }
    Ok(())
}

fn write_temporary_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn temporary_path_for(path: &Path) -> PathBuf {
    path.with_file_name(format!(".{MARKER_FILE}.{}.tmp", Uuid::new_v4()))
}

fn sync_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn find_repository_root(working_directory: &Path) -> Result<PathBuf, EnrollmentError> {
    let working_directory = working_directory.canonicalize().map_err(|error| {
        EnrollmentError::refusal(
            format!(
                "working directory `{}` cannot be inspected: {error}",
                working_directory.display()
            ),
            "rerun from an existing directory inside the repository",
        )
    })?;
    let repository_root = repository_root(&working_directory).ok_or_else(|| {
        EnrollmentError::refusal(
            format!(
                "`{}` is not inside a repository root",
                working_directory.display()
            ),
            "rerun inside a repository containing `.git`",
        )
    })?;
    Ok(repository_root.to_path_buf())
}

fn repository_root(working_directory: &Path) -> Option<&Path> {
    working_directory
        .ancestors()
        .find(|candidate| marker::is_repository_root(candidate))
}
