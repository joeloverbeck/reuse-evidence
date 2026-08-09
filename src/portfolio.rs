use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use reuse_evidence::Visibility;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    portfolio_roots: Vec<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Marker {
    schema_version: u64,
    repository_id: Uuid,
    ecosystem_id: String,
    visibility: Visibility,
}

#[derive(Debug)]
struct Enrollment {
    repository_id: Uuid,
    ecosystem_id: String,
    path: PathBuf,
    visibility: Visibility,
}

#[derive(Debug)]
struct UnsupportedMarker {
    path: PathBuf,
    schema_version: i64,
}

#[derive(Debug)]
struct Scan {
    enrollments: Vec<Enrollment>,
    unsupported_markers: Vec<UnsupportedMarker>,
}

enum MarkerInspection {
    Enrollment(Enrollment),
    Unsupported(UnsupportedMarker),
    Ignore,
}

pub(crate) enum PortfolioReport {
    Complete(String),
    IdentityConflict(String),
}

pub(crate) fn report(root_overrides: &[PathBuf]) -> Result<PortfolioReport, String> {
    let roots = selected_roots(root_overrides)?;
    let scan = scan(&roots)?;
    if scan.enrollments.is_empty() && scan.unsupported_markers.is_empty() {
        return Ok(PortfolioReport::Complete(
            "no enrolled repositories found\n".to_owned(),
        ));
    }

    let mut identity_paths = BTreeMap::<Uuid, Vec<PathBuf>>::new();
    for enrollment in &scan.enrollments {
        identity_paths
            .entry(enrollment.repository_id)
            .or_default()
            .push(enrollment.path.clone());
    }
    identity_paths.retain(|_, paths| paths.len() > 1);
    let has_identity_conflicts = !identity_paths.is_empty();

    let mut grouped = BTreeMap::<String, Vec<Enrollment>>::new();
    for enrollment in scan.enrollments {
        grouped
            .entry(enrollment.ecosystem_id.clone())
            .or_default()
            .push(enrollment);
    }

    let mut output = String::new();
    for (ecosystem_id, mut entries) in grouped {
        entries.sort_by(|left, right| {
            left.repository_id
                .cmp(&right.repository_id)
                .then_with(|| left.path.cmp(&right.path))
        });
        writeln!(output, "ecosystem: {ecosystem_id}").expect("writing to a string cannot fail");
        for entry in entries {
            writeln!(output, "- repository_id: {}", entry.repository_id)
                .expect("writing to a string cannot fail");
            writeln!(output, "  path: {}", entry.path.display())
                .expect("writing to a string cannot fail");
            writeln!(output, "  visibility: {}", entry.visibility)
                .expect("writing to a string cannot fail");
        }
    }
    if !identity_paths.is_empty() {
        writeln!(output, "duplicate repository identity conflicts:")
            .expect("writing to a string cannot fail");
        for (repository_id, mut paths) in identity_paths {
            paths.sort();
            writeln!(output, "- repository_id: {repository_id}")
                .expect("writing to a string cannot fail");
            writeln!(output, "  paths:").expect("writing to a string cannot fail");
            for path in paths {
                writeln!(output, "  - {}", path.display())
                    .expect("writing to a string cannot fail");
            }
        }
    }
    if !scan.unsupported_markers.is_empty() {
        writeln!(output, "unsupported marker versions:").expect("writing to a string cannot fail");
        let mut unsupported_markers = scan.unsupported_markers;
        unsupported_markers.sort_by(|left, right| left.path.cmp(&right.path));
        for marker in unsupported_markers {
            writeln!(output, "- marker: {}", marker.path.display())
                .expect("writing to a string cannot fail");
            writeln!(output, "  schema_version: {}", marker.schema_version)
                .expect("writing to a string cannot fail");
        }
    }
    if has_identity_conflicts {
        Ok(PortfolioReport::IdentityConflict(output))
    } else {
        Ok(PortfolioReport::Complete(output))
    }
}

fn selected_roots(root_overrides: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    if !root_overrides.is_empty() {
        return Ok(root_overrides.to_vec());
    }

    let config_path = config_path()?;
    let config_bytes = match fs::read(&config_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(no_roots_message(&config_path));
        }
        Err(error) => {
            return Err(format!(
                "user-local portfolio configuration `{}` cannot be read: {error}\nresolution: make the file readable or rerun with `--root <PATH>`",
                config_path.display()
            ));
        }
    };
    let config_text = std::str::from_utf8(&config_bytes).map_err(|error| {
        format!(
            "user-local portfolio configuration `{}` is not UTF-8: {error}\nresolution: save valid TOML in UTF-8 or rerun with `--root <PATH>`",
            config_path.display()
        )
    })?;
    let config = toml::from_str::<Config>(config_text).map_err(|error| {
        format!(
            "user-local portfolio configuration `{}` is invalid: {error}\nresolution: define `portfolio_roots` as an array of paths or rerun with `--root <PATH>`",
            config_path.display()
        )
    })?;
    if config.portfolio_roots.is_empty() {
        return Err(no_roots_message(&config_path));
    }
    Ok(config.portfolio_roots)
}

fn no_roots_message(config_path: &Path) -> String {
    format!(
        "no portfolio roots were supplied and `{}` does not configure any\nresolution: add `portfolio_roots = [\"/path/to/root\"]` to that user-local configuration or rerun with `--root <PATH>`",
        config_path.display()
    )
}

fn scan(roots: &[PathBuf]) -> Result<Scan, String> {
    let mut directories = Vec::new();
    for root in roots {
        let root = root.canonicalize().map_err(|error| {
            format!(
                "portfolio root `{}` cannot be inspected: {error}\nresolution: supply an existing readable directory with `--root <PATH>` or update the user-local configuration",
                root.display()
            )
        })?;
        if !root.is_dir() {
            return Err(format!(
                "portfolio root `{}` is not a directory\nresolution: supply a directory with `--root <PATH>` or update the user-local configuration",
                root.display()
            ));
        }
        directories.push(root);
    }

    let mut visited = BTreeSet::new();
    let mut enrollments = Vec::new();
    let mut unsupported_markers = Vec::new();
    while let Some(directory) = directories.pop() {
        if !visited.insert(directory.clone()) {
            continue;
        }
        if directory.join(".git").exists() {
            match inspect_marker(&directory)? {
                MarkerInspection::Enrollment(enrollment) => enrollments.push(enrollment),
                MarkerInspection::Unsupported(marker) => unsupported_markers.push(marker),
                MarkerInspection::Ignore => {}
            }
        }

        let entries = fs::read_dir(&directory).map_err(|error| {
            format!(
                "portfolio directory `{}` cannot be read: {error}\nresolution: make the configured root readable or choose a narrower `--root <PATH>`",
                directory.display()
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "an entry beneath portfolio directory `{}` cannot be read: {error}\nresolution: make the configured root readable or choose a narrower `--root <PATH>`",
                    directory.display()
                )
            })?;
            if entry.file_name() == ".git" {
                continue;
            }
            if entry
                .file_type()
                .map_err(|error| {
                    format!(
                        "portfolio entry `{}` cannot be inspected: {error}\nresolution: make the configured root readable or choose a narrower `--root <PATH>`",
                        entry.path().display()
                    )
                })?
                .is_dir()
            {
                directories.push(entry.path());
            }
        }
    }
    Ok(Scan {
        enrollments,
        unsupported_markers,
    })
}

fn inspect_marker(repository: &Path) -> Result<MarkerInspection, String> {
    let marker_path = repository.join(reuse_evidence::MARKER_FILE);
    let marker_bytes = match fs::read(&marker_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(MarkerInspection::Ignore);
        }
        Err(error) => {
            return Err(format!(
                "repository marker `{}` cannot be read: {error}\nresolution: make the marker readable or choose a narrower `--root <PATH>`",
                marker_path.display()
            ));
        }
    };
    let Ok(marker_text) = std::str::from_utf8(&marker_bytes) else {
        return Ok(MarkerInspection::Ignore);
    };
    let Ok(table) = toml::from_str::<toml::Table>(marker_text) else {
        return Ok(MarkerInspection::Ignore);
    };
    let Some(schema_version) = table
        .get("schema_version")
        .and_then(toml::Value::as_integer)
    else {
        return Ok(MarkerInspection::Ignore);
    };
    if schema_version != 1 {
        return Ok(MarkerInspection::Unsupported(UnsupportedMarker {
            path: marker_path,
            schema_version,
        }));
    }
    let Ok(marker) = toml::Value::Table(table).try_into::<Marker>() else {
        return Ok(MarkerInspection::Ignore);
    };
    if marker.schema_version != 1 {
        return Ok(MarkerInspection::Ignore);
    }

    Ok(MarkerInspection::Enrollment(Enrollment {
        repository_id: marker.repository_id,
        ecosystem_id: marker.ecosystem_id,
        path: repository.to_path_buf(),
        visibility: marker.visibility,
    }))
}

fn config_path() -> Result<PathBuf, String> {
    platform_config_directory()
        .map(|directory| directory.join("reuse-evidence").join("config.toml"))
        .ok_or_else(|| {
            "the user-local configuration directory cannot be determined\nresolution: set `APPDATA` on Windows, or `XDG_CONFIG_HOME` or `HOME` on Unix-like systems; alternatively rerun with `--root <PATH>`".to_owned()
        })
}

#[cfg(target_os = "windows")]
fn platform_config_directory() -> Option<PathBuf> {
    nonempty_environment_path("APPDATA")
}

#[cfg(target_os = "macos")]
fn platform_config_directory() -> Option<PathBuf> {
    nonempty_environment_path("XDG_CONFIG_HOME").or_else(|| {
        nonempty_environment_path("HOME")
            .map(|home| home.join("Library").join("Application Support"))
    })
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn platform_config_directory() -> Option<PathBuf> {
    nonempty_environment_path("XDG_CONFIG_HOME")
        .or_else(|| nonempty_environment_path("HOME").map(|home| home.join(".config")))
}

fn nonempty_environment_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
