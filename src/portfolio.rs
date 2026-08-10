use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::marker::{self, MarkerRead, UnreadableMarker, UnsupportedMarker};
use crate::{TerminalFailure, Visibility};
use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    portfolio_roots: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Enrollment {
    pub(crate) repository_id: Uuid,
    ecosystem_id: String,
    pub(crate) path: PathBuf,
    pub(crate) visibility: Visibility,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PortfolioState {
    repositories: Vec<Enrollment>,
}

#[derive(Debug)]
pub(crate) struct Scan {
    roots: Vec<PathBuf>,
    inspected_repositories: BTreeSet<PathBuf>,
    pub(crate) enrollments: Vec<Enrollment>,
    unsupported_markers: Vec<UnsupportedMarker>,
    unreadable_markers: Vec<UnreadableMarker>,
}

struct PortfolioObservation<'a> {
    roots: &'a [PathBuf],
    inspected_repositories: &'a BTreeSet<PathBuf>,
    enrollments: &'a [Enrollment],
}

enum MarkerInspection {
    Enrollment(Enrollment),
    Unsupported(UnsupportedMarker),
    Unreadable(UnreadableMarker),
    Ignore,
}

pub enum PortfolioReport {
    /// A complete unambiguous portfolio observation.
    Complete(String),
    /// A report that found at least one duplicate stable repository identity.
    IdentityConflict(String),
}

struct PortfolioChanges {
    new_repositories: Vec<Enrollment>,
    moved_repositories: Vec<(Enrollment, PathBuf)>,
    unavailable_repositories: Vec<Enrollment>,
    visibility_changes: Vec<(Enrollment, Visibility)>,
}

struct StateLock {
    path: PathBuf,
}

/// Rescans enrolled repositories and renders the current portfolio report.
///
/// # Errors
///
/// Returns a classified terminal failure when roots, markers, or derived state
/// cannot be inspected safely.
pub fn report(root_overrides: &[PathBuf]) -> Result<PortfolioReport, TerminalFailure> {
    let roots = selected_roots(root_overrides)?;
    let scan = scan(&roots)?;
    let identity_paths = duplicate_identity_paths(&scan.enrollments);
    let has_identity_conflicts = !identity_paths.is_empty();

    if has_identity_conflicts {
        return Ok(PortfolioReport::IdentityConflict(render_report(
            &scan,
            &identity_paths,
            None,
        )));
    }

    let state_path = state_path()?;
    ensure_state_outside_repositories(&state_path, &scan.inspected_repositories)?;
    let _state_lock = acquire_state_lock(&state_path)?;
    let previous_state = load_state(&state_path)?;
    let previous_repositories = previous_state
        .repositories
        .into_iter()
        .map(|repository| (repository.repository_id, repository))
        .collect::<BTreeMap<_, _>>();
    let observation = PortfolioObservation {
        roots: &scan.roots,
        inspected_repositories: &scan.inspected_repositories,
        enrollments: &scan.enrollments,
    };
    let changes = derive_changes(&observation, &previous_repositories);
    let output = render_report(&scan, &identity_paths, Some(&changes));
    save_state(
        &state_path,
        &next_state(&observation, previous_repositories),
    )?;
    Ok(PortfolioReport::Complete(output))
}

fn duplicate_identity_paths(enrollments: &[Enrollment]) -> BTreeMap<Uuid, Vec<PathBuf>> {
    let mut identity_paths = BTreeMap::<Uuid, Vec<PathBuf>>::new();
    for enrollment in enrollments {
        identity_paths
            .entry(enrollment.repository_id)
            .or_default()
            .push(enrollment.path.clone());
    }
    identity_paths.retain(|_, paths| paths.len() > 1);
    identity_paths
}

fn derive_changes(
    observation: &PortfolioObservation<'_>,
    previous_repositories: &BTreeMap<Uuid, Enrollment>,
) -> PortfolioChanges {
    let mut new_repositories = observation
        .enrollments
        .iter()
        .filter(|repository| !previous_repositories.contains_key(&repository.repository_id))
        .cloned()
        .collect::<Vec<_>>();
    new_repositories.sort_by(|left, right| left.repository_id.cmp(&right.repository_id));
    let mut moved_repositories = observation
        .enrollments
        .iter()
        .filter_map(|repository| {
            let previous = previous_repositories.get(&repository.repository_id)?;
            (previous.path != repository.path).then(|| (repository.clone(), previous.path.clone()))
        })
        .collect::<Vec<_>>();
    moved_repositories.sort_by(|left, right| left.0.repository_id.cmp(&right.0.repository_id));
    let mut unavailable_repositories = previous_repositories
        .values()
        .filter(|repository| {
            !observation
                .enrollments
                .iter()
                .any(|current| current.repository_id == repository.repository_id)
        })
        .filter(|repository| {
            observation
                .roots
                .iter()
                .any(|root| repository.path.starts_with(root))
        })
        .filter(|repository| {
            !observation
                .inspected_repositories
                .contains(&repository.path)
        })
        .cloned()
        .collect::<Vec<_>>();
    unavailable_repositories.sort_by(|left, right| left.repository_id.cmp(&right.repository_id));
    let mut visibility_changes = observation
        .enrollments
        .iter()
        .filter_map(|repository| {
            let previous = previous_repositories.get(&repository.repository_id)?;
            (previous.visibility != repository.visibility)
                .then(|| (repository.clone(), previous.visibility))
        })
        .collect::<Vec<_>>();
    visibility_changes.sort_by(|left, right| left.0.repository_id.cmp(&right.0.repository_id));

    PortfolioChanges {
        new_repositories,
        moved_repositories,
        unavailable_repositories,
        visibility_changes,
    }
}

fn render_report(
    scan: &Scan,
    identity_paths: &BTreeMap<Uuid, Vec<PathBuf>>,
    changes: Option<&PortfolioChanges>,
) -> String {
    PortfolioReportView {
        scan,
        identity_paths,
        changes,
    }
    .to_string()
}

struct PortfolioReportView<'a> {
    scan: &'a Scan,
    identity_paths: &'a BTreeMap<Uuid, Vec<PathBuf>>,
    changes: Option<&'a PortfolioChanges>,
}

enum RepositoryEntryView<'a> {
    Enrolled(&'a Enrollment),
    IdentityConflict {
        repository_id: &'a Uuid,
        paths: &'a [PathBuf],
    },
    Moved {
        repository: &'a Enrollment,
        previous_path: &'a Path,
    },
    VisibilityChanged {
        repository: &'a Enrollment,
        previous_visibility: &'a Visibility,
    },
}

impl Display for RepositoryEntryView<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let repository_id = match self {
            Self::Enrolled(repository)
            | Self::Moved { repository, .. }
            | Self::VisibilityChanged { repository, .. } => &repository.repository_id,
            Self::IdentityConflict { repository_id, .. } => repository_id,
        };
        writeln!(formatter, "- repository_id: {repository_id}")?;

        match self {
            Self::Enrolled(repository) => {
                writeln!(formatter, "  path: {}", repository.path.display())?;
                writeln!(formatter, "  visibility: {}", repository.visibility)
            }
            Self::IdentityConflict { paths, .. } => {
                writeln!(formatter, "  paths:")?;
                let mut paths = paths.iter().collect::<Vec<_>>();
                paths.sort();
                for path in paths {
                    writeln!(formatter, "  - {}", path.display())?;
                }
                Ok(())
            }
            Self::Moved {
                repository,
                previous_path,
            } => {
                writeln!(formatter, "  previous_path: {}", previous_path.display())?;
                writeln!(formatter, "  path: {}", repository.path.display())
            }
            Self::VisibilityChanged {
                repository,
                previous_visibility,
            } => {
                writeln!(formatter, "  path: {}", repository.path.display())?;
                writeln!(formatter, "  previous_visibility: {previous_visibility}")?;
                writeln!(formatter, "  visibility: {}", repository.visibility)
            }
        }
    }
}

impl Display for PortfolioReportView<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let mut grouped = BTreeMap::<&str, Vec<&Enrollment>>::new();
        for enrollment in &self.scan.enrollments {
            grouped
                .entry(&enrollment.ecosystem_id)
                .or_default()
                .push(enrollment);
        }

        if self.scan.enrollments.is_empty()
            && self.scan.unsupported_markers.is_empty()
            && self.scan.unreadable_markers.is_empty()
        {
            writeln!(formatter, "no enrolled repositories found")?;
        }
        for (ecosystem_id, mut entries) in grouped {
            entries.sort_by(|left, right| {
                left.repository_id
                    .cmp(&right.repository_id)
                    .then_with(|| left.path.cmp(&right.path))
            });
            writeln!(formatter, "ecosystem: {ecosystem_id}")?;
            for entry in entries {
                write!(formatter, "{}", RepositoryEntryView::Enrolled(entry))?;
            }
        }
        if !self.identity_paths.is_empty() {
            writeln!(formatter, "duplicate repository identity conflicts:")?;
            for (repository_id, paths) in self.identity_paths {
                write!(
                    formatter,
                    "{}",
                    RepositoryEntryView::IdentityConflict {
                        repository_id,
                        paths,
                    }
                )?;
            }
        }
        if !self.scan.unsupported_markers.is_empty() {
            writeln!(formatter, "unsupported marker versions:")?;
            let mut unsupported_markers = self.scan.unsupported_markers.iter().collect::<Vec<_>>();
            unsupported_markers.sort_by(|left, right| left.path().cmp(right.path()));
            for marker in unsupported_markers {
                writeln!(formatter, "- marker: {}", marker.path().display())?;
                writeln!(formatter, "  schema_version: {}", marker.schema_version())?;
            }
        }
        if !self.scan.unreadable_markers.is_empty() {
            writeln!(formatter, "unreadable repository markers:")?;
            let mut unreadable_markers = self.scan.unreadable_markers.iter().collect::<Vec<_>>();
            unreadable_markers.sort_by(|left, right| left.path().cmp(right.path()));
            for marker in unreadable_markers {
                writeln!(formatter, "- marker: {}", marker.path().display())?;
                writeln!(formatter, "  reason: {}", marker.reason())?;
            }
        }
        if let Some(changes) = self.changes {
            if !changes.new_repositories.is_empty() {
                writeln!(formatter, "new repositories:")?;
                for repository in &changes.new_repositories {
                    write!(formatter, "{}", RepositoryEntryView::Enrolled(repository))?;
                }
            }
            if !changes.moved_repositories.is_empty() {
                writeln!(formatter, "moved repositories:")?;
                for (repository, previous_path) in &changes.moved_repositories {
                    write!(
                        formatter,
                        "{}",
                        RepositoryEntryView::Moved {
                            repository,
                            previous_path,
                        }
                    )?;
                }
            }
            if !changes.unavailable_repositories.is_empty() {
                writeln!(formatter, "unavailable repositories:")?;
                for repository in &changes.unavailable_repositories {
                    write!(formatter, "{}", RepositoryEntryView::Enrolled(repository))?;
                }
            }
            if !changes.visibility_changes.is_empty() {
                writeln!(formatter, "visibility changed repositories:")?;
                for (repository, previous_visibility) in &changes.visibility_changes {
                    write!(
                        formatter,
                        "{}",
                        RepositoryEntryView::VisibilityChanged {
                            repository,
                            previous_visibility,
                        }
                    )?;
                }
            }
        }
        Ok(())
    }
}

fn next_state(
    observation: &PortfolioObservation<'_>,
    mut previous_repositories: BTreeMap<Uuid, Enrollment>,
) -> PortfolioState {
    for inspected_repository in observation.inspected_repositories {
        previous_repositories.retain(|_, repository| repository.path != *inspected_repository);
    }
    for repository in observation.enrollments {
        previous_repositories.insert(repository.repository_id, repository.clone());
    }
    PortfolioState {
        repositories: previous_repositories.into_values().collect(),
    }
}

fn ensure_state_outside_repositories(
    state_path: &Path,
    inspected_repositories: &BTreeSet<PathBuf>,
) -> Result<(), TerminalFailure> {
    let resolved_state_path =
        resolve_path_through_existing_ancestor(state_path).map_err(|error| {
            TerminalFailure::refusal(
                format!(
                    "user-local portfolio state path `{}` cannot be resolved: {error}",
                    state_path.display()
                ),
                "configure an accessible platform state directory outside every Git repository",
            )
        })?;
    if let Some(repository) = inspected_repositories
        .iter()
        .find(|repository| resolved_state_path.starts_with(repository))
    {
        return Err(TerminalFailure::refusal(
            format!(
                "user-local portfolio state `{}` would be stored inside inspected repository `{}`",
                state_path.display(),
                repository.display()
            ),
            "configure the platform state directory outside every inspected repository",
        ));
    }
    if let Some(repository) = resolved_state_path
        .ancestors()
        .find(|ancestor| marker::is_repository_root(ancestor))
    {
        return Err(TerminalFailure::refusal(
            format!(
                "user-local portfolio state `{}` would be stored inside Git repository `{}`",
                state_path.display(),
                repository.display()
            ),
            "configure the platform state directory outside every Git repository",
        ));
    }
    Ok(())
}

fn resolve_path_through_existing_ancestor(path: &Path) -> std::io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let mut existing_ancestor = absolute.as_path();
    let mut missing_components = Vec::new();
    while !existing_ancestor.exists() {
        let component = existing_ancestor.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no existing ancestor could be found",
            )
        })?;
        missing_components.push(component.to_os_string());
        existing_ancestor = existing_ancestor.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no existing ancestor could be found",
            )
        })?;
    }
    let mut resolved = existing_ancestor.canonicalize()?;
    for component in missing_components.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn load_state(path: &Path) -> Result<PortfolioState, TerminalFailure> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(PortfolioState::default());
        }
        Err(error) => {
            return Err(TerminalFailure::unsafe_failure(format!(
                "user-local portfolio state `{}` cannot be read: {error}",
                path.display()
            )));
        }
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Ok(PortfolioState::default());
    };
    Ok(toml::from_str(text).unwrap_or_default())
}

fn acquire_state_lock(state_path: &Path) -> Result<StateLock, TerminalFailure> {
    let parent = state_path
        .parent()
        .expect("the user-local state path always has a parent");
    fs::create_dir_all(parent).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "user-local portfolio state directory `{}` cannot be created: {error}",
            parent.display()
        ))
    })?;
    let lock_path = parent.join("portfolio.lock");
    let mut lock_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(TerminalFailure::refusal(
                format!(
                    "another portfolio state update is in progress at `{}`",
                    lock_path.display()
                ),
                "wait for it to finish, or remove the derived lock after confirming no portfolio command is running",
            ));
        }
        Err(error) => {
            return Err(TerminalFailure::unsafe_failure(format!(
                "user-local portfolio state lock `{}` cannot be created: {error}",
                lock_path.display()
            )));
        }
    };
    if let Err(error) =
        writeln!(lock_file, "{}", std::process::id()).and_then(|()| lock_file.sync_all())
    {
        let _ = fs::remove_file(&lock_path);
        return Err(TerminalFailure::unsafe_failure(format!(
            "user-local portfolio state lock `{}` cannot be initialized: {error}",
            lock_path.display()
        )));
    }
    Ok(StateLock { path: lock_path })
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn save_state(path: &Path, state: &PortfolioState) -> Result<(), TerminalFailure> {
    let bytes = toml::to_string(state).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "user-local portfolio state could not be encoded: {error}"
        ))
    })?;
    if fs::read(path).is_ok_and(|current| current == bytes.as_bytes()) {
        return Ok(());
    }
    replace_state_atomically(path, bytes.as_bytes()).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "user-local portfolio state `{}` cannot be published atomically: {error}",
            path.display()
        ))
    })
}

fn replace_state_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut temporary = AtomicWriteFile::open(path)?;
    temporary.write_all(bytes)?;
    temporary.commit()
}

pub(crate) fn selected_roots(root_overrides: &[PathBuf]) -> Result<Vec<PathBuf>, TerminalFailure> {
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
            return Err(TerminalFailure::refusal(
                format!(
                    "user-local portfolio configuration `{}` cannot be read: {error}",
                    config_path.display()
                ),
                "make the file readable or rerun with `--root <PATH>`",
            ));
        }
    };
    let config_text = std::str::from_utf8(&config_bytes).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "user-local portfolio configuration `{}` is not UTF-8: {error}",
                config_path.display()
            ),
            "save valid TOML in UTF-8 or rerun with `--root <PATH>`",
        )
    })?;
    let config = toml::from_str::<Config>(config_text).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "user-local portfolio configuration `{}` is invalid: {error}",
                config_path.display()
            ),
            "define `portfolio_roots` as an array of paths or rerun with `--root <PATH>`",
        )
    })?;
    if config.portfolio_roots.is_empty() {
        return Err(no_roots_message(&config_path));
    }
    Ok(config.portfolio_roots)
}

fn no_roots_message(config_path: &Path) -> TerminalFailure {
    TerminalFailure::refusal(
        format!(
            "no portfolio roots were supplied and `{}` does not configure any",
            config_path.display()
        ),
        "add `portfolio_roots = [\"/path/to/root\"]` to that user-local configuration or rerun with `--root <PATH>`",
    )
}

pub(crate) fn scan(roots: &[PathBuf]) -> Result<Scan, TerminalFailure> {
    let mut canonical_roots = Vec::new();
    for root in roots {
        let root = root.canonicalize().map_err(|error| {
            TerminalFailure::refusal(
                format!(
                    "portfolio root `{}` cannot be inspected: {error}",
                    root.display()
                ),
                "supply an existing readable directory with `--root <PATH>` or update the user-local configuration",
            )
        })?;
        if !root.is_dir() {
            return Err(TerminalFailure::refusal(
                format!("portfolio root `{}` is not a directory", root.display()),
                "supply a directory with `--root <PATH>` or update the user-local configuration",
            ));
        }
        canonical_roots.push(root);
    }

    let mut directories = canonical_roots.clone();
    let mut visited = BTreeSet::new();
    let mut inspected_repositories = BTreeSet::new();
    let mut enrollments = Vec::new();
    let mut unsupported_markers = Vec::new();
    let mut unreadable_markers = Vec::new();
    while let Some(directory) = directories.pop() {
        if !visited.insert(directory.clone()) {
            continue;
        }
        if marker::is_repository_root(&directory) {
            inspected_repositories.insert(directory.clone());
            match inspect_marker(&directory) {
                MarkerInspection::Enrollment(enrollment) => enrollments.push(enrollment),
                MarkerInspection::Unsupported(marker) => unsupported_markers.push(marker),
                MarkerInspection::Unreadable(marker) => unreadable_markers.push(marker),
                MarkerInspection::Ignore => {}
            }
        }

        let entries = fs::read_dir(&directory).map_err(|error| {
            TerminalFailure::refusal(
                format!(
                    "portfolio directory `{}` cannot be read: {error}",
                    directory.display()
                ),
                "make the configured root readable or choose a narrower `--root <PATH>`",
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                TerminalFailure::refusal(
                    format!(
                        "an entry beneath portfolio directory `{}` cannot be read: {error}",
                        directory.display()
                    ),
                    "make the configured root readable or choose a narrower `--root <PATH>`",
                )
            })?;
            if entry.file_name() == ".git" {
                continue;
            }
            if entry
                .file_type()
                .map_err(|error| {
                    TerminalFailure::refusal(
                        format!(
                            "portfolio entry `{}` cannot be inspected: {error}",
                            entry.path().display()
                        ),
                        "make the configured root readable or choose a narrower `--root <PATH>`",
                    )
                })?
                .is_dir()
            {
                directories.push(entry.path());
            }
        }
    }
    Ok(Scan {
        roots: canonical_roots,
        inspected_repositories,
        enrollments,
        unsupported_markers,
        unreadable_markers,
    })
}

fn inspect_marker(repository: &Path) -> MarkerInspection {
    match marker::read(repository) {
        Some(MarkerRead::Supported(marker)) => MarkerInspection::Enrollment(Enrollment {
            repository_id: marker.repository_id(),
            ecosystem_id: marker.ecosystem_id().to_owned(),
            path: repository.to_path_buf(),
            visibility: marker.visibility(),
        }),
        Some(MarkerRead::UnsupportedSchemaVersion(marker)) => MarkerInspection::Unsupported(marker),
        Some(MarkerRead::Unreadable(marker)) => MarkerInspection::Unreadable(marker),
        None => MarkerInspection::Ignore,
    }
}

fn config_path() -> Result<PathBuf, TerminalFailure> {
    platform_config_directory()
        .map(|directory| directory.join("reuse-evidence").join("config.toml"))
        .ok_or_else(|| TerminalFailure::refusal(
            "the user-local configuration directory cannot be determined",
            "set `APPDATA` on Windows, or `XDG_CONFIG_HOME` or `HOME` on Unix-like systems; alternatively rerun with `--root <PATH>`",
        ))
}

fn state_path() -> Result<PathBuf, TerminalFailure> {
    platform_state_directory()
        .map(|directory| directory.join("reuse-evidence").join("portfolio.toml"))
        .ok_or_else(|| {
            TerminalFailure::refusal(
                "the user-local state directory cannot be determined",
                "set `LOCALAPPDATA` on Windows, or `XDG_STATE_HOME` or `HOME` on Unix-like systems",
            )
        })
}

#[cfg(target_os = "windows")]
fn platform_config_directory() -> Option<PathBuf> {
    nonempty_environment_path("APPDATA")
}

#[cfg(target_os = "windows")]
fn platform_state_directory() -> Option<PathBuf> {
    nonempty_environment_path("LOCALAPPDATA")
}

#[cfg(target_os = "macos")]
fn platform_state_directory() -> Option<PathBuf> {
    nonempty_environment_path("XDG_STATE_HOME").or_else(|| {
        nonempty_environment_path("HOME")
            .map(|home| home.join("Library").join("Application Support"))
    })
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn platform_state_directory() -> Option<PathBuf> {
    nonempty_environment_path("XDG_STATE_HOME")
        .or_else(|| nonempty_environment_path("HOME").map(|home| home.join(".local/state")))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn enrollment(repository_id: &str, path: &str, visibility: Visibility) -> Enrollment {
        Enrollment {
            repository_id: Uuid::parse_str(repository_id)
                .expect("test repository identity should be valid"),
            ecosystem_id: "products".to_owned(),
            path: PathBuf::from(path),
            visibility,
        }
    }

    #[test]
    fn derive_changes_reports_each_current_observation_difference() {
        let root = PathBuf::from("/portfolio");
        let new_repository = enrollment(
            "00000000-0000-4000-8000-000000000101",
            "/portfolio/new",
            Visibility::Private,
        );
        let moved_repository = enrollment(
            "00000000-0000-4000-8000-000000000102",
            "/portfolio/moved",
            Visibility::Private,
        );
        let unavailable_repository = enrollment(
            "00000000-0000-4000-8000-000000000103",
            "/portfolio/unavailable",
            Visibility::Private,
        );
        let visibility_changed_repository = enrollment(
            "00000000-0000-4000-8000-000000000104",
            "/portfolio/visibility",
            Visibility::Public,
        );
        let enrollments = vec![
            new_repository.clone(),
            moved_repository.clone(),
            visibility_changed_repository.clone(),
        ];
        let inspected_repositories = enrollments
            .iter()
            .map(|repository| repository.path.clone())
            .collect();
        let roots = vec![root];
        let observation = PortfolioObservation {
            roots: &roots,
            inspected_repositories: &inspected_repositories,
            enrollments: &enrollments,
        };
        let previous_repositories = [
            (
                moved_repository.repository_id,
                enrollment(
                    "00000000-0000-4000-8000-000000000102",
                    "/portfolio/original",
                    Visibility::Private,
                ),
            ),
            (
                unavailable_repository.repository_id,
                unavailable_repository.clone(),
            ),
            (
                visibility_changed_repository.repository_id,
                enrollment(
                    "00000000-0000-4000-8000-000000000104",
                    "/portfolio/visibility",
                    Visibility::Private,
                ),
            ),
        ]
        .into_iter()
        .collect();

        let changes = derive_changes(&observation, &previous_repositories);

        assert_eq!(changes.new_repositories.len(), 1);
        assert_eq!(
            changes.new_repositories[0].repository_id,
            new_repository.repository_id
        );
        assert_eq!(changes.moved_repositories.len(), 1);
        assert_eq!(
            changes.moved_repositories[0].0.repository_id,
            moved_repository.repository_id
        );
        assert_eq!(
            changes.moved_repositories[0].1,
            PathBuf::from("/portfolio/original")
        );
        assert_eq!(changes.unavailable_repositories.len(), 1);
        assert_eq!(
            changes.unavailable_repositories[0].repository_id,
            unavailable_repository.repository_id
        );
        assert_eq!(changes.visibility_changes.len(), 1);
        assert_eq!(
            changes.visibility_changes[0].0.repository_id,
            visibility_changed_repository.repository_id
        );
        assert_eq!(changes.visibility_changes[0].1, Visibility::Private);
    }

    #[test]
    fn next_state_replaces_a_stale_identity_at_an_inspected_path() {
        let repository_path = PathBuf::from("/portfolio/current");
        let current_repository = enrollment(
            "00000000-0000-4000-8000-000000000112",
            "/portfolio/current",
            Visibility::Public,
        );
        let enrollments = vec![current_repository.clone()];
        let inspected_repositories = [repository_path.clone()].into_iter().collect();
        let roots = vec![PathBuf::from("/portfolio")];
        let observation = PortfolioObservation {
            roots: &roots,
            inspected_repositories: &inspected_repositories,
            enrollments: &enrollments,
        };
        let stale_repository = enrollment(
            "00000000-0000-4000-8000-000000000111",
            "/portfolio/current",
            Visibility::Private,
        );
        let previous_repositories = [(stale_repository.repository_id, stale_repository)]
            .into_iter()
            .collect();

        let state = next_state(&observation, previous_repositories);

        assert_eq!(state.repositories.len(), 1);
        assert_eq!(
            state.repositories[0].repository_id,
            current_repository.repository_id
        );
        assert_eq!(state.repositories[0].path, repository_path);
        assert_eq!(state.repositories[0].visibility, Visibility::Public);
    }
}
