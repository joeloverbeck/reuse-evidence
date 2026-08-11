use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::marker::{self, MarkerRead, UnreadableMarker, UnsupportedMarker};
use crate::{TerminalFailure, Visibility};
#[cfg(feature = "cli")]
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
    substituted_repositories: Vec<SubstitutedRepository>,
}

struct SubstitutedRepository {
    repository: Enrollment,
    previous_repository_id: Uuid,
    previous_visibility: Visibility,
}

struct StateLock {
    path: PathBuf,
}

enum RootSelection {
    Selected(Vec<PathBuf>),
    Unconfigured(PathBuf),
}

/// Where one command resolves the portfolio from.
///
/// Resolution reads the process environment and nothing else: no filesystem
/// access, no validation, and no refusal. A user-local directory the platform's
/// variables do not determine is carried as an absence, so each command decides
/// whether that absence refuses or degrades. `portfolio` refuses; the case
/// queries report unknown conditions and succeed.
#[derive(Clone, Debug)]
pub struct PortfolioLocation {
    root_overrides: Vec<PathBuf>,
    config_file: Option<PathBuf>,
    state_file: Option<PathBuf>,
}

impl PortfolioLocation {
    /// Resolves the location for one run from the roots supplied on the command
    /// line and the process environment.
    #[must_use]
    pub fn from_environment(root_overrides: Vec<PathBuf>) -> Self {
        Self {
            root_overrides,
            config_file: CONFIGURATION
                .resolve_from_environment()
                .as_deref()
                .map(|directory| CONFIGURATION.file_beneath(directory)),
            state_file: STATE
                .resolve_from_environment()
                .as_deref()
                .map(|directory| STATE.file_beneath(directory)),
        }
    }

    /// Builds a location from explicit user-local directories, without
    /// consulting the process environment.
    ///
    /// `None` states that the directory is undeterminable, which is what an
    /// unset `XDG_CONFIG_HOME` with no `HOME` means to a command.
    #[must_use]
    pub fn from_user_directories(
        root_overrides: Vec<PathBuf>,
        configuration_directory: Option<&Path>,
        state_directory: Option<&Path>,
    ) -> Self {
        Self {
            root_overrides,
            config_file: configuration_directory
                .map(|directory| CONFIGURATION.file_beneath(directory)),
            state_file: state_directory.map(|directory| STATE.file_beneath(directory)),
        }
    }

    fn root_overrides(&self) -> &[PathBuf] {
        &self.root_overrides
    }
}

/// Rescans enrolled repositories and renders the current portfolio report.
///
/// # Errors
///
/// Returns a classified terminal failure when roots, markers, or derived state
/// cannot be inspected safely.
#[cfg(feature = "cli")]
pub fn report(location: &PortfolioLocation) -> Result<PortfolioReport, TerminalFailure> {
    let roots = selected_roots(location)?;
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

    let state_path = state_path(location)?;
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
    let mut substituted_repositories = previous_repositories
        .values()
        .filter(|previous| {
            !observation
                .enrollments
                .iter()
                .any(|current| current.repository_id == previous.repository_id)
        })
        .filter_map(|previous| {
            let repository = observation.enrollments.iter().find(|current| {
                current.path == previous.path && current.repository_id != previous.repository_id
            })?;
            Some(SubstitutedRepository {
                repository: repository.clone(),
                previous_repository_id: previous.repository_id,
                previous_visibility: previous.visibility,
            })
        })
        .collect::<Vec<_>>();
    substituted_repositories.sort_by(|left, right| {
        left.repository
            .repository_id
            .cmp(&right.repository.repository_id)
            .then_with(|| {
                left.previous_repository_id
                    .cmp(&right.previous_repository_id)
            })
    });
    let mut new_repositories = observation
        .enrollments
        .iter()
        .filter(|repository| !previous_repositories.contains_key(&repository.repository_id))
        .filter(|repository| {
            !substituted_repositories.iter().any(|substitution| {
                substitution.repository.repository_id == repository.repository_id
            })
        })
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
        substituted_repositories,
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
    Substituted(&'a SubstitutedRepository),
}

impl Display for RepositoryEntryView<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let repository_id = match self {
            Self::Enrolled(repository)
            | Self::Moved { repository, .. }
            | Self::VisibilityChanged { repository, .. } => &repository.repository_id,
            Self::Substituted(substitution) => &substitution.repository.repository_id,
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
            Self::Substituted(substitution) => {
                writeln!(
                    formatter,
                    "  path: {}",
                    substitution.repository.path.display()
                )?;
                writeln!(
                    formatter,
                    "  previous_repository_id: {}",
                    substitution.previous_repository_id
                )?;
                writeln!(
                    formatter,
                    "  previous_visibility: {}",
                    substitution.previous_visibility
                )?;
                writeln!(
                    formatter,
                    "  visibility: {}",
                    substitution.repository.visibility
                )
            }
        }
    }
}

impl Display for PortfolioChanges {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if !self.new_repositories.is_empty() {
            writeln!(formatter, "new repositories:")?;
            for repository in &self.new_repositories {
                write!(formatter, "{}", RepositoryEntryView::Enrolled(repository))?;
            }
        }
        if !self.moved_repositories.is_empty() {
            writeln!(formatter, "moved repositories:")?;
            for (repository, previous_path) in &self.moved_repositories {
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
        if !self.unavailable_repositories.is_empty() {
            writeln!(formatter, "unavailable repositories:")?;
            for repository in &self.unavailable_repositories {
                write!(formatter, "{}", RepositoryEntryView::Enrolled(repository))?;
            }
        }
        if !self.visibility_changes.is_empty() {
            writeln!(formatter, "visibility changed repositories:")?;
            for (repository, previous_visibility) in &self.visibility_changes {
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
        if !self.substituted_repositories.is_empty() {
            writeln!(formatter, "substituted repositories:")?;
            for substitution in &self.substituted_repositories {
                write!(
                    formatter,
                    "{}",
                    RepositoryEntryView::Substituted(substitution)
                )?;
            }
        }
        Ok(())
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
            write!(formatter, "{changes}")?;
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

#[cfg(feature = "cli")]
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

#[cfg(feature = "cli")]
fn replace_state_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut temporary = AtomicWriteFile::open(path)?;
    temporary.write_all(bytes)?;
    temporary.commit()
}

/// Selects the roots to scan, refusing when neither the command line nor the
/// user-local configuration supplies any.
pub(crate) fn selected_roots(
    location: &PortfolioLocation,
) -> Result<Vec<PathBuf>, TerminalFailure> {
    match root_selection(location)? {
        RootSelection::Selected(roots) => Ok(roots),
        RootSelection::Unconfigured(config_path) => Err(no_roots_message(&config_path)),
    }
}

/// Selects the roots to scan, reporting absence rather than refusing when no
/// selection resolves.
///
/// Case queries derive unknown portfolio conditions from `None` and still
/// succeed, which is why this is a separate answer from [`selected_roots`].
pub(crate) fn selected_roots_if_configured(
    location: &PortfolioLocation,
) -> Result<Option<Vec<PathBuf>>, TerminalFailure> {
    if !location.root_overrides().is_empty() {
        return Ok(Some(location.root_overrides().to_vec()));
    }
    let Some(config_path) = location.config_file.clone() else {
        return Ok(None);
    };
    match root_selection_from_config(config_path)? {
        RootSelection::Selected(roots) => Ok(Some(roots)),
        RootSelection::Unconfigured(_) => Ok(None),
    }
}

fn root_selection(location: &PortfolioLocation) -> Result<RootSelection, TerminalFailure> {
    if !location.root_overrides().is_empty() {
        return Ok(RootSelection::Selected(location.root_overrides().to_vec()));
    }

    root_selection_from_config(config_path(location)?)
}

fn root_selection_from_config(config_path: PathBuf) -> Result<RootSelection, TerminalFailure> {
    let config_bytes = match fs::read(&config_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RootSelection::Unconfigured(config_path));
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
        return Ok(RootSelection::Unconfigured(config_path));
    }
    Ok(RootSelection::Selected(config.portfolio_roots))
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

fn config_path(location: &PortfolioLocation) -> Result<PathBuf, TerminalFailure> {
    location.config_file.clone().ok_or_else(|| {
        TerminalFailure::refusal(
            "the user-local configuration directory cannot be determined",
            "set `APPDATA` on Windows, or `XDG_CONFIG_HOME` or `HOME` on Unix-like systems; alternatively rerun with `--root <PATH>`",
        )
    })
}

#[cfg(feature = "cli")]
fn state_path(location: &PortfolioLocation) -> Result<PathBuf, TerminalFailure> {
    location.state_file.clone().ok_or_else(|| {
        TerminalFailure::refusal(
            "the user-local state directory cannot be determined",
            "set `LOCALAPPDATA` on Windows, or `XDG_STATE_HOME` or `HOME` on Unix-like systems",
        )
    })
}

/// One kind of user-local directory and the environment variables that locate it.
struct UserDirectory {
    windows_variable: &'static str,
    xdg_variable: &'static str,
    unix_home_relative: &'static str,
    file_name: &'static str,
}

const CONFIGURATION: UserDirectory = UserDirectory {
    windows_variable: "APPDATA",
    xdg_variable: "XDG_CONFIG_HOME",
    unix_home_relative: ".config",
    file_name: "config.toml",
};

const STATE: UserDirectory = UserDirectory {
    windows_variable: "LOCALAPPDATA",
    xdg_variable: "XDG_STATE_HOME",
    unix_home_relative: ".local/state",
    file_name: "portfolio.toml",
};

/// Where macOS keeps both kinds, in place of the XDG-style home-relative path.
const MACOS_HOME_RELATIVE: &str = "Library/Application Support";

/// Which precedence a host applies to the user-local directories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Platform {
    Windows,
    MacOs,
    Xdg,
}

impl Platform {
    /// The host this build targets.
    ///
    /// Selecting the platform as a value rather than by `#[cfg]` keeps one
    /// statement of the precedence and lets every host's precedence be tested
    /// from any host.
    const CURRENT: Self = if cfg!(target_os = "windows") {
        Self::Windows
    } else if cfg!(target_os = "macos") {
        Self::MacOs
    } else {
        Self::Xdg
    };
}

impl UserDirectory {
    /// Applies `platform`'s precedence to `lookup`, which supplies one
    /// non-empty environment value per variable name.
    ///
    /// This is the crate's only statement of that precedence. It performs no
    /// filesystem access, so an undeterminable directory is `None` rather than
    /// a failure.
    fn select(
        &self,
        platform: Platform,
        lookup: impl Fn(&str) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        let home_relative = match platform {
            Platform::Windows => return lookup(self.windows_variable),
            Platform::MacOs => MACOS_HOME_RELATIVE,
            Platform::Xdg => self.unix_home_relative,
        };
        lookup(self.xdg_variable).or_else(|| lookup("HOME").map(|home| home.join(home_relative)))
    }

    fn resolve_from_environment(&self) -> Option<PathBuf> {
        self.select(Platform::CURRENT, nonempty_environment_path)
    }

    fn file_beneath(&self, directory: &Path) -> PathBuf {
        directory.join("reuse-evidence").join(self.file_name)
    }
}

fn nonempty_environment_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Answers `variables` and nothing else, so a precedence test states exactly
    /// which of them the platform is allowed to consult.
    fn environment(variables: &[(&'static str, &str)]) -> impl Fn(&str) -> Option<PathBuf> + use<> {
        let variables: Vec<(String, PathBuf)> = variables
            .iter()
            .map(|(name, value)| ((*name).to_owned(), PathBuf::from(value)))
            .collect();
        move |name| {
            variables
                .iter()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, value)| value.clone())
        }
    }

    #[test]
    fn xdg_variable_takes_precedence_over_home_for_both_directories() {
        assert_eq!(
            CONFIGURATION.select(
                Platform::Xdg,
                environment(&[("XDG_CONFIG_HOME", "/xdg/config"), ("HOME", "/home/user")])
            ),
            Some(PathBuf::from("/xdg/config"))
        );
        assert_eq!(
            STATE.select(
                Platform::Xdg,
                environment(&[("XDG_STATE_HOME", "/xdg/state"), ("HOME", "/home/user")])
            ),
            Some(PathBuf::from("/xdg/state"))
        );
    }

    #[test]
    fn xdg_platform_falls_back_to_distinct_home_relative_directories() {
        let home = environment(&[("HOME", "/home/user")]);
        assert_eq!(
            CONFIGURATION.select(Platform::Xdg, &home),
            Some(PathBuf::from("/home/user/.config"))
        );
        assert_eq!(
            STATE.select(Platform::Xdg, &home),
            Some(PathBuf::from("/home/user/.local/state"))
        );
    }

    #[test]
    fn macos_falls_back_to_one_application_support_directory_for_both() {
        let home = environment(&[("HOME", "/Users/user")]);
        let expected = Some(PathBuf::from("/Users/user/Library/Application Support"));
        assert_eq!(CONFIGURATION.select(Platform::MacOs, &home), expected);
        assert_eq!(STATE.select(Platform::MacOs, &home), expected);
    }

    #[test]
    fn windows_consults_only_its_own_variables() {
        let windows = environment(&[
            ("APPDATA", "C:/Users/user/AppData/Roaming"),
            ("LOCALAPPDATA", "C:/Users/user/AppData/Local"),
            ("XDG_CONFIG_HOME", "/xdg/config"),
            ("XDG_STATE_HOME", "/xdg/state"),
            ("HOME", "/home/user"),
        ]);
        assert_eq!(
            CONFIGURATION.select(Platform::Windows, &windows),
            Some(PathBuf::from("C:/Users/user/AppData/Roaming"))
        );
        assert_eq!(
            STATE.select(Platform::Windows, &windows),
            Some(PathBuf::from("C:/Users/user/AppData/Local"))
        );
    }

    #[test]
    fn an_undeterminable_directory_is_an_absence_rather_than_a_failure() {
        let empty = environment(&[]);
        for platform in [Platform::Windows, Platform::MacOs, Platform::Xdg] {
            assert_eq!(CONFIGURATION.select(platform, &empty), None, "{platform:?}");
            assert_eq!(STATE.select(platform, &empty), None, "{platform:?}");
        }
    }

    #[test]
    fn explicit_user_directories_name_the_same_files_the_environment_would() {
        let location = PortfolioLocation::from_user_directories(
            Vec::new(),
            Some(Path::new("/config")),
            Some(Path::new("/state")),
        );
        assert_eq!(
            location.config_file,
            Some(PathBuf::from("/config/reuse-evidence/config.toml"))
        );
        assert_eq!(
            location.state_file,
            Some(PathBuf::from("/state/reuse-evidence/portfolio.toml"))
        );

        let undeterminable = PortfolioLocation::from_user_directories(Vec::new(), None, None);
        assert_eq!(undeterminable.config_file, None);
        assert_eq!(undeterminable.state_file, None);
    }

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
    fn substitution_is_derived_before_next_state_replaces_stale_identity() {
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
        let previous_repositories = [(stale_repository.repository_id, stale_repository.clone())]
            .into_iter()
            .collect();

        let changes = derive_changes(&observation, &previous_repositories);

        assert!(changes.new_repositories.is_empty());
        assert_eq!(changes.substituted_repositories.len(), 1);
        assert_eq!(
            changes.substituted_repositories[0].repository.repository_id,
            current_repository.repository_id
        );
        assert_eq!(
            changes.substituted_repositories[0].previous_repository_id,
            stale_repository.repository_id
        );
        assert_eq!(
            changes.substituted_repositories[0].previous_visibility,
            Visibility::Private
        );

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
