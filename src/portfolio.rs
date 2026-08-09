use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;
use reuse_evidence::marker::{self, MarkerRead, UnreadableMarker, UnsupportedMarker};
use reuse_evidence::{ExitMeaning, Visibility};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    portfolio_roots: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Enrollment {
    repository_id: Uuid,
    ecosystem_id: String,
    path: PathBuf,
    visibility: Visibility,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PortfolioState {
    repositories: Vec<Enrollment>,
}

#[derive(Debug)]
struct Scan {
    roots: Vec<PathBuf>,
    inspected_repositories: BTreeSet<PathBuf>,
    enrollments: Vec<Enrollment>,
    unsupported_markers: Vec<UnsupportedMarker>,
    unreadable_markers: Vec<UnreadableMarker>,
}

enum MarkerInspection {
    Enrollment(Enrollment),
    Unsupported(UnsupportedMarker),
    Unreadable(UnreadableMarker),
    Ignore,
}

pub(crate) enum PortfolioReport {
    Complete(String),
    IdentityConflict(String),
}

pub(crate) struct PortfolioError {
    pub(crate) meaning: ExitMeaning,
    pub(crate) message: String,
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

pub(crate) fn report(root_overrides: &[PathBuf]) -> Result<PortfolioReport, PortfolioError> {
    let roots = selected_roots(root_overrides).map_err(PortfolioError::refusal)?;
    let scan = scan(&roots).map_err(PortfolioError::refusal)?;
    let identity_paths = duplicate_identity_paths(&scan.enrollments);
    let has_identity_conflicts = !identity_paths.is_empty();

    if has_identity_conflicts {
        return Ok(PortfolioReport::IdentityConflict(render_report(
            &scan,
            &identity_paths,
            None,
        )));
    }

    let state_path = state_path().map_err(PortfolioError::refusal)?;
    ensure_state_outside_repositories(&state_path, &scan.inspected_repositories)?;
    let _state_lock = acquire_state_lock(&state_path)?;
    let previous_state = load_state(&state_path)?;
    let previous_repositories = previous_state
        .repositories
        .into_iter()
        .map(|repository| (repository.repository_id, repository))
        .collect::<BTreeMap<_, _>>();
    let changes = derive_changes(&scan, &previous_repositories);
    let output = render_report(&scan, &identity_paths, Some(&changes));
    save_state(&state_path, &next_state(&scan, previous_repositories))?;
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
    scan: &Scan,
    previous_repositories: &BTreeMap<Uuid, Enrollment>,
) -> PortfolioChanges {
    let mut new_repositories = scan
        .enrollments
        .iter()
        .filter(|repository| !previous_repositories.contains_key(&repository.repository_id))
        .cloned()
        .collect::<Vec<_>>();
    new_repositories.sort_by(|left, right| left.repository_id.cmp(&right.repository_id));
    let mut moved_repositories = scan
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
            !scan
                .enrollments
                .iter()
                .any(|current| current.repository_id == repository.repository_id)
        })
        .filter(|repository| {
            scan.roots
                .iter()
                .any(|root| repository.path.starts_with(root))
        })
        .filter(|repository| !scan.inspected_repositories.contains(&repository.path))
        .cloned()
        .collect::<Vec<_>>();
    unavailable_repositories.sort_by(|left, right| left.repository_id.cmp(&right.repository_id));
    let mut visibility_changes = scan
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
    let mut grouped = BTreeMap::<String, Vec<Enrollment>>::new();
    for enrollment in scan.enrollments.iter().cloned() {
        grouped
            .entry(enrollment.ecosystem_id.clone())
            .or_default()
            .push(enrollment);
    }

    let mut output = String::new();
    if scan.enrollments.is_empty()
        && scan.unsupported_markers.is_empty()
        && scan.unreadable_markers.is_empty()
    {
        output.push_str("no enrolled repositories found\n");
    }
    for (ecosystem_id, mut entries) in grouped {
        entries.sort_by(|left, right| {
            left.repository_id
                .cmp(&right.repository_id)
                .then_with(|| left.path.cmp(&right.path))
        });
        writeln!(output, "ecosystem: {ecosystem_id}").expect("writing to a string cannot fail");
        for entry in entries {
            write_repository_entry(&mut output, &entry);
        }
    }
    if !identity_paths.is_empty() {
        writeln!(output, "duplicate repository identity conflicts:")
            .expect("writing to a string cannot fail");
        for (repository_id, paths) in identity_paths {
            let mut paths = paths.clone();
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
        let mut unsupported_markers = scan.unsupported_markers.iter().collect::<Vec<_>>();
        unsupported_markers.sort_by(|left, right| left.path().cmp(right.path()));
        for marker in unsupported_markers {
            writeln!(output, "- marker: {}", marker.path().display())
                .expect("writing to a string cannot fail");
            writeln!(output, "  schema_version: {}", marker.schema_version())
                .expect("writing to a string cannot fail");
        }
    }
    if !scan.unreadable_markers.is_empty() {
        writeln!(output, "unreadable repository markers:")
            .expect("writing to a string cannot fail");
        let mut unreadable_markers = scan.unreadable_markers.iter().collect::<Vec<_>>();
        unreadable_markers.sort_by(|left, right| left.path().cmp(right.path()));
        for marker in unreadable_markers {
            writeln!(output, "- marker: {}", marker.path().display())
                .expect("writing to a string cannot fail");
            writeln!(output, "  reason: {}", marker.reason())
                .expect("writing to a string cannot fail");
        }
    }
    if let Some(changes) = changes {
        append_changes(&mut output, changes);
    }
    output
}

fn append_changes(output: &mut String, changes: &PortfolioChanges) {
    if !changes.new_repositories.is_empty() {
        writeln!(output, "new repositories:").expect("writing to a string cannot fail");
        for repository in &changes.new_repositories {
            write_repository_entry(output, repository);
        }
    }
    if !changes.moved_repositories.is_empty() {
        writeln!(output, "moved repositories:").expect("writing to a string cannot fail");
        for (repository, previous_path) in &changes.moved_repositories {
            writeln!(output, "- repository_id: {}", repository.repository_id)
                .expect("writing to a string cannot fail");
            writeln!(output, "  previous_path: {}", previous_path.display())
                .expect("writing to a string cannot fail");
            writeln!(output, "  path: {}", repository.path.display())
                .expect("writing to a string cannot fail");
        }
    }
    if !changes.unavailable_repositories.is_empty() {
        writeln!(output, "unavailable repositories:").expect("writing to a string cannot fail");
        for repository in &changes.unavailable_repositories {
            write_repository_entry(output, repository);
        }
    }
    if !changes.visibility_changes.is_empty() {
        writeln!(output, "visibility changed repositories:")
            .expect("writing to a string cannot fail");
        for (repository, previous_visibility) in &changes.visibility_changes {
            writeln!(output, "- repository_id: {}", repository.repository_id)
                .expect("writing to a string cannot fail");
            writeln!(output, "  path: {}", repository.path.display())
                .expect("writing to a string cannot fail");
            writeln!(output, "  previous_visibility: {previous_visibility}")
                .expect("writing to a string cannot fail");
            writeln!(output, "  visibility: {}", repository.visibility)
                .expect("writing to a string cannot fail");
        }
    }
}

fn write_repository_entry(output: &mut String, repository: &Enrollment) {
    writeln!(output, "- repository_id: {}", repository.repository_id)
        .expect("writing to a string cannot fail");
    writeln!(output, "  path: {}", repository.path.display())
        .expect("writing to a string cannot fail");
    writeln!(output, "  visibility: {}", repository.visibility)
        .expect("writing to a string cannot fail");
}

fn next_state(
    scan: &Scan,
    mut previous_repositories: BTreeMap<Uuid, Enrollment>,
) -> PortfolioState {
    for inspected_repository in &scan.inspected_repositories {
        previous_repositories.retain(|_, repository| repository.path != *inspected_repository);
    }
    for repository in &scan.enrollments {
        previous_repositories.insert(repository.repository_id, repository.clone());
    }
    PortfolioState {
        repositories: previous_repositories.into_values().collect(),
    }
}

impl PortfolioError {
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
}

fn ensure_state_outside_repositories(
    state_path: &Path,
    inspected_repositories: &BTreeSet<PathBuf>,
) -> Result<(), PortfolioError> {
    let resolved_state_path = resolve_path_through_existing_ancestor(state_path).map_err(|error| {
        PortfolioError::refusal(format!(
            "user-local portfolio state path `{}` cannot be resolved: {error}\nresolution: configure an accessible platform state directory outside every Git repository",
            state_path.display()
        ))
    })?;
    if let Some(repository) = inspected_repositories
        .iter()
        .find(|repository| resolved_state_path.starts_with(repository))
    {
        return Err(PortfolioError::refusal(format!(
            "user-local portfolio state `{}` would be stored inside inspected repository `{}`\nresolution: configure the platform state directory outside every inspected repository",
            state_path.display(),
            repository.display()
        )));
    }
    if let Some(repository) = resolved_state_path
        .ancestors()
        .find(|ancestor| marker::is_repository_root(ancestor))
    {
        return Err(PortfolioError::refusal(format!(
            "user-local portfolio state `{}` would be stored inside Git repository `{}`\nresolution: configure the platform state directory outside every Git repository",
            state_path.display(),
            repository.display()
        )));
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

fn load_state(path: &Path) -> Result<PortfolioState, PortfolioError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(PortfolioState::default());
        }
        Err(error) => {
            return Err(PortfolioError::unsafe_failure(format!(
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

fn acquire_state_lock(state_path: &Path) -> Result<StateLock, PortfolioError> {
    let parent = state_path
        .parent()
        .expect("the user-local state path always has a parent");
    fs::create_dir_all(parent).map_err(|error| {
        PortfolioError::unsafe_failure(format!(
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
            return Err(PortfolioError::refusal(format!(
                "another portfolio state update is in progress at `{}`\nresolution: wait for it to finish, or remove the derived lock after confirming no portfolio command is running",
                lock_path.display()
            )));
        }
        Err(error) => {
            return Err(PortfolioError::unsafe_failure(format!(
                "user-local portfolio state lock `{}` cannot be created: {error}",
                lock_path.display()
            )));
        }
    };
    if let Err(error) =
        writeln!(lock_file, "{}", std::process::id()).and_then(|()| lock_file.sync_all())
    {
        let _ = fs::remove_file(&lock_path);
        return Err(PortfolioError::unsafe_failure(format!(
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

fn save_state(path: &Path, state: &PortfolioState) -> Result<(), PortfolioError> {
    let bytes = toml::to_string(state).map_err(|error| {
        PortfolioError::unsafe_failure(format!(
            "user-local portfolio state could not be encoded: {error}"
        ))
    })?;
    if fs::read(path).is_ok_and(|current| current == bytes.as_bytes()) {
        return Ok(());
    }
    replace_state_atomically(path, bytes.as_bytes()).map_err(|error| {
        PortfolioError::unsafe_failure(format!(
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
    let mut canonical_roots = Vec::new();
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

fn config_path() -> Result<PathBuf, String> {
    platform_config_directory()
        .map(|directory| directory.join("reuse-evidence").join("config.toml"))
        .ok_or_else(|| {
            "the user-local configuration directory cannot be determined\nresolution: set `APPDATA` on Windows, or `XDG_CONFIG_HOME` or `HOME` on Unix-like systems; alternatively rerun with `--root <PATH>`".to_owned()
        })
}

fn state_path() -> Result<PathBuf, String> {
    platform_state_directory()
        .map(|directory| directory.join("reuse-evidence").join("portfolio.toml"))
        .ok_or_else(|| {
            "the user-local state directory cannot be determined\nresolution: set `LOCALAPPDATA` on Windows, or `XDG_STATE_HOME` or `HOME` on Unix-like systems".to_owned()
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
