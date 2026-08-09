//! Repository marker parsing and repository-root recognition.

use std::borrow::Cow;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{MARKER_FILE, Visibility};

/// The marker schema version supported by this crate.
pub const SUPPORTED_SCHEMA_VERSION: i64 = 1;

/// A supported enrolled-repository marker.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Marker {
    schema_version: i64,
    repository_id: Uuid,
    ecosystem_id: String,
    visibility: Visibility,
}

impl Marker {
    pub(crate) fn new(repository_id: Uuid, ecosystem_id: String, visibility: Visibility) -> Self {
        Self {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            repository_id,
            ecosystem_id,
            visibility,
        }
    }

    /// Returns the repository's stable opaque identity.
    #[must_use]
    pub const fn repository_id(&self) -> Uuid {
        self.repository_id
    }

    /// Returns the repository's ecosystem identity.
    #[must_use]
    pub fn ecosystem_id(&self) -> &str {
        &self.ecosystem_id
    }

    /// Returns the repository's declared visibility.
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub(crate) fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }
}

/// A marker using a schema version this crate does not support.
#[derive(Debug)]
pub struct UnsupportedMarker {
    path: PathBuf,
    schema_version: i64,
}

impl UnsupportedMarker {
    /// Returns the marker path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the unsupported schema version found in the marker.
    #[must_use]
    pub const fn schema_version(&self) -> i64 {
        self.schema_version
    }
}

/// A present marker whose bytes could not be read as a supported marker.
#[derive(Debug)]
pub struct UnreadableMarker {
    path: PathBuf,
    cause: UnreadableCause,
}

impl UnreadableMarker {
    /// Returns the marker path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns a concise reason suitable for a portfolio report.
    #[must_use]
    pub fn reason(&self) -> Cow<'_, str> {
        match &self.cause {
            UnreadableCause::Read(error) => Cow::Owned(error.to_string()),
            UnreadableCause::Utf8(error) => Cow::Owned(error.to_string()),
            UnreadableCause::Toml(error) => Cow::Borrowed(error.message()),
        }
    }

    pub(crate) const fn is_read_failure(&self) -> bool {
        matches!(self.cause, UnreadableCause::Read(_))
    }
}

impl fmt::Display for UnreadableMarker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.cause {
            UnreadableCause::Read(error) => error.fmt(formatter),
            UnreadableCause::Utf8(error) => error.fmt(formatter),
            UnreadableCause::Toml(error) => error.fmt(formatter),
        }
    }
}

#[derive(Debug)]
enum UnreadableCause {
    Read(std::io::Error),
    Utf8(std::str::Utf8Error),
    Toml(toml::de::Error),
}

/// The three classifications for a marker present at a repository root.
pub enum MarkerRead {
    /// The marker uses the supported schema and has all required fields.
    Supported(Marker),
    /// The marker declares a schema version this crate does not support.
    UnsupportedSchemaVersion(UnsupportedMarker),
    /// The marker cannot be read or decoded as a supported marker.
    Unreadable(UnreadableMarker),
}

/// Returns whether `directory` is a repository root.
///
/// A worktree is recognized by a `.git` file. An ordinary repository is
/// recognized by a `.git` directory containing its `HEAD` file.
#[must_use]
pub fn is_repository_root(directory: &Path) -> bool {
    let git_path = directory.join(".git");
    git_path.is_file() || git_path.join("HEAD").is_file()
}

/// Reads and classifies the marker at `repository_root`.
///
/// A missing marker returns `None` because absence is not a marker-read
/// classification and does not enroll the repository.
#[must_use]
pub fn read(repository_root: &Path) -> Option<MarkerRead> {
    let path = repository_root.join(MARKER_FILE);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            return Some(MarkerRead::Unreadable(UnreadableMarker {
                path,
                cause: UnreadableCause::Read(error),
            }));
        }
    };
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(error) => {
            return Some(MarkerRead::Unreadable(UnreadableMarker {
                path,
                cause: UnreadableCause::Utf8(error),
            }));
        }
    };
    let table = match toml::from_str::<toml::Table>(text) {
        Ok(table) => table,
        Err(error) => {
            return Some(MarkerRead::Unreadable(UnreadableMarker {
                path,
                cause: UnreadableCause::Toml(error),
            }));
        }
    };
    if let Some(schema_version) = table
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        && schema_version != SUPPORTED_SCHEMA_VERSION
    {
        return Some(MarkerRead::UnsupportedSchemaVersion(UnsupportedMarker {
            path,
            schema_version,
        }));
    }
    match toml::Value::Table(table).try_into() {
        Ok(marker) => Some(MarkerRead::Supported(marker)),
        Err(error) => Some(MarkerRead::Unreadable(UnreadableMarker {
            path,
            cause: UnreadableCause::Toml(error),
        })),
    }
}
