#![forbid(unsafe_code)]

//! Evidence-gated reuse decisions for repository portfolios.

use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// The repository is public.
    Public,
    /// The repository is private.
    Private,
}

impl Visibility {
    /// Parses the two visibility values supported by marker schema version 1.
    ///
    /// # Errors
    ///
    /// Returns a refusal when `value` is neither `public` nor `private`.
    pub fn parse(value: &str) -> Result<Self, EnrollmentError> {
        match value {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            _ => Err(EnrollmentError::refusal(format!(
                "visibility `{value}` is not supported\nresolution: use `public` or `private`"
            ))),
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

/// The observable result of a fresh enrollment.
#[derive(Debug, Eq, PartialEq)]
pub struct Enrollment {
    /// The path of the marker that was written.
    pub marker_path: PathBuf,
    /// The newly generated opaque repository identity.
    pub repository_id: Uuid,
    /// The declared ecosystem identity.
    pub ecosystem_id: String,
    /// The declared repository visibility.
    pub visibility: Visibility,
}

/// A classified enrollment failure with terminal-contract meaning.
#[derive(Debug, Eq, PartialEq)]
pub struct EnrollmentError {
    meaning: ExitMeaning,
    message: String,
}

impl EnrollmentError {
    fn refusal(message: String) -> Self {
        Self {
            meaning: ExitMeaning::Refusal,
            message,
        }
    }

    fn unsafe_failure(message: String) -> Self {
        Self {
            meaning: ExitMeaning::UnsafeFailure,
            message,
        }
    }

    /// Returns this failure's process-level meaning.
    #[must_use]
    pub const fn meaning(&self) -> ExitMeaning {
        self.meaning
    }
}

impl fmt::Display for EnrollmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EnrollmentError {}

#[derive(Serialize)]
struct Marker<'a> {
    schema_version: u8,
    repository_id: Uuid,
    ecosystem_id: &'a str,
    visibility: Visibility,
}

/// Enrolls the repository containing `working_directory`.
///
/// The repository root is the nearest ancestor containing a `.git` directory
/// or file. This operation implements fresh enrollment only.
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
    let working_directory = working_directory.canonicalize().map_err(|error| {
        EnrollmentError::refusal(format!(
            "working directory `{}` cannot be inspected: {error}\nresolution: rerun from an existing directory inside the repository",
            working_directory.display()
        ))
    })?;
    let repository_root = repository_root(&working_directory).ok_or_else(|| {
        EnrollmentError::refusal(format!(
            "`{}` is not inside a repository root\nresolution: rerun inside a repository containing `.git`",
            working_directory.display()
        ))
    })?;
    let repository_id = Uuid::new_v4();
    let marker = Marker {
        schema_version: 1,
        repository_id,
        ecosystem_id,
        visibility,
    };
    let marker_bytes = toml::to_string(&marker).map_err(|error| {
        EnrollmentError::unsafe_failure(format!("could not encode the repository marker: {error}"))
    })?;
    let marker_path = repository_root.join(MARKER_FILE);
    let mut marker_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker_path)
        .map_err(|error| {
            EnrollmentError::unsafe_failure(format!(
                "could not create `{}`: {error}",
                marker_path.display()
            ))
        })?;
    marker_file
        .write_all(marker_bytes.as_bytes())
        .and_then(|()| marker_file.sync_all())
        .map_err(|error| {
            EnrollmentError::unsafe_failure(format!(
                "could not finish writing `{}`: {error}",
                marker_path.display()
            ))
        })?;

    Ok(Enrollment {
        marker_path,
        repository_id,
        ecosystem_id: ecosystem_id.to_owned(),
        visibility,
    })
}

fn repository_root(working_directory: &Path) -> Option<&Path> {
    working_directory
        .ancestors()
        .find(|candidate| candidate.join(".git").exists())
}
