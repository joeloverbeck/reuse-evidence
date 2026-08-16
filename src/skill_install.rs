//! Installation of this project's own agent-skill packages.

use std::collections::HashSet;
use std::fmt::{self, Write as _};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{TerminalFailure, create_file_atomically, replace_file_atomically};

struct ShippedFile {
    relative_path: &'static str,
    bytes: &'static [u8],
}

#[derive(Debug, Eq, PartialEq)]
struct DiscoveryLink {
    relative_path: String,
    target: String,
}

#[derive(Debug, Eq, PartialEq)]
struct DiscoveryLinkOutcome {
    link: DiscoveryLink,
    action: DiscoveryLinkAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InstalledState {
    Missing,
    Matching,
    Differing,
    Unreplaceable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DiscoveryLinkAction {
    Unchanged,
    Created,
    Replaced,
    Unsupported,
    RemovedUnsupported,
}

const SHIPPED_FILES: &[ShippedFile] = &[
    ShippedFile {
        relative_path: ".claude/skills/reuse-evidence-capture/SKILL.md",
        bytes: include_bytes!("../.claude/skills/reuse-evidence-capture/SKILL.md"),
    },
    ShippedFile {
        relative_path: ".claude/skills/reuse-evidence-capture/agents/openai.yaml",
        bytes: include_bytes!("../.claude/skills/reuse-evidence-capture/agents/openai.yaml"),
    },
    ShippedFile {
        relative_path: ".claude/skills/reuse-evidence-capture/references/prepared-proposals.md",
        bytes: include_bytes!(
            "../.claude/skills/reuse-evidence-capture/references/prepared-proposals.md"
        ),
    },
    ShippedFile {
        relative_path: ".claude/skills/reuse-evidence-review/SKILL.md",
        bytes: include_bytes!("../.claude/skills/reuse-evidence-review/SKILL.md"),
    },
    ShippedFile {
        relative_path: ".claude/skills/reuse-evidence-review/agents/openai.yaml",
        bytes: include_bytes!("../.claude/skills/reuse-evidence-review/agents/openai.yaml"),
    },
    ShippedFile {
        relative_path: ".claude/skills/reuse-evidence-review/references/decision-and-publication.md",
        bytes: include_bytes!(
            "../.claude/skills/reuse-evidence-review/references/decision-and-publication.md"
        ),
    },
    ShippedFile {
        relative_path: ".claude/skills/reuse-evidence-review/references/package-research.md",
        bytes: include_bytes!(
            "../.claude/skills/reuse-evidence-review/references/package-research.md"
        ),
    },
    ShippedFile {
        relative_path: ".claude/skills/reuse-evidence-review/references/review-analysis.md",
        bytes: include_bytes!(
            "../.claude/skills/reuse-evidence-review/references/review-analysis.md"
        ),
    },
];

const SHIPPED_SKILL_PREFIX: &str = ".claude/skills/";
const RESERVED_PACKAGE_PREFIX: &str = "reuse-evidence-";

/// The observable result of installing this project's skill packages.
#[derive(Debug, Eq, PartialEq)]
pub struct SkillInstallOutcome {
    written: Vec<&'static str>,
    replaced: Vec<&'static str>,
    discovery_links: Vec<DiscoveryLinkOutcome>,
}

impl fmt::Display for SkillInstallOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.written.is_empty()
            && self.replaced.is_empty()
            && self.discovery_links.iter().all(|outcome| {
                matches!(
                    outcome.action,
                    DiscoveryLinkAction::Unchanged | DiscoveryLinkAction::Unsupported
                )
            })
        {
            writeln!(formatter, "reuse-evidence skill packages already installed")?;
            writeln!(formatter, "nothing needed writing")?;
        } else {
            writeln!(formatter, "installed reuse-evidence skill packages")?;
            if !self.written.is_empty() {
                writeln!(formatter, "written:")?;
                for path in &self.written {
                    writeln!(formatter, "- {path}")?;
                }
            }
            if !self.replaced.is_empty() {
                writeln!(formatter, "replaced:")?;
                for path in &self.replaced {
                    writeln!(formatter, "- {path}")?;
                }
            }
            let created = self
                .discovery_links
                .iter()
                .filter(|outcome| outcome.action == DiscoveryLinkAction::Created)
                .collect::<Vec<_>>();
            if !created.is_empty() {
                writeln!(formatter, "discovery links created:")?;
                for outcome in created {
                    writeln!(
                        formatter,
                        "- {} -> {}",
                        outcome.link.relative_path, outcome.link.target
                    )?;
                }
            }
            let replaced = self
                .discovery_links
                .iter()
                .filter(|outcome| outcome.action == DiscoveryLinkAction::Replaced)
                .collect::<Vec<_>>();
            if !replaced.is_empty() {
                writeln!(formatter, "discovery links replaced:")?;
                for outcome in replaced {
                    writeln!(
                        formatter,
                        "- {} -> {}",
                        outcome.link.relative_path, outcome.link.target
                    )?;
                }
            }
            let removed = self
                .discovery_links
                .iter()
                .filter(|outcome| outcome.action == DiscoveryLinkAction::RemovedUnsupported)
                .collect::<Vec<_>>();
            if !removed.is_empty() {
                writeln!(formatter, "removed discovery paths:")?;
                for outcome in removed {
                    writeln!(formatter, "- {}", outcome.link.relative_path)?;
                }
            }
        }
        let unsupported = self
            .discovery_links
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome.action,
                    DiscoveryLinkAction::Unsupported | DiscoveryLinkAction::RemovedUnsupported
                )
            })
            .collect::<Vec<_>>();
        if !unsupported.is_empty() {
            writeln!(
                formatter,
                "discovery links not created: symbolic links are not supported on this platform"
            )?;
            for outcome in unsupported {
                writeln!(formatter, "- {}", outcome.link.relative_path)?;
            }
        }
        Ok(())
    }
}

/// Installs this project's shipped skill packages beneath `target_root`.
///
/// # Errors
///
/// Returns a refusal when the named target root or an installation path cannot
/// be changed safely, including when a non-force run finds local differences.
/// Returns an unsafe failure when a write starts but cannot be completed.
pub fn install_skills(
    target_root: &Path,
    force: bool,
) -> Result<SkillInstallOutcome, TerminalFailure> {
    validate_target_root(target_root)?;
    validate_installation_directories(target_root)?;
    let comparisons = compare_shipped_files(target_root)?;
    let link_comparisons = compare_discovery_links(target_root)?;
    let stale_paths = find_stale_paths(target_root)?;
    refuse_unapproved_or_unreplaceable(&comparisons, &link_comparisons, &stale_paths, force)?;
    let (written, replaced) = write_shipped_files(target_root, comparisons)?;
    let discovery_links = write_discovery_links(target_root, link_comparisons)?;
    Ok(SkillInstallOutcome {
        written,
        replaced,
        discovery_links,
    })
}

fn shipped_package_names() -> Vec<&'static str> {
    let mut names = Vec::new();
    for shipped in SHIPPED_FILES {
        let package_and_file = shipped
            .relative_path
            .strip_prefix(SHIPPED_SKILL_PREFIX)
            .expect("every shipped skill file must live under .claude/skills");
        let package_name = package_and_file
            .split_once('/')
            .map(|(name, _)| name)
            .expect("every shipped skill file must name a file inside its package");
        assert!(
            package_name.starts_with(RESERVED_PACKAGE_PREFIX),
            "every shipped skill package must use the reserved reuse-evidence- prefix"
        );
        if !names.contains(&package_name) {
            names.push(package_name);
        }
    }
    names
}

fn shipped_discovery_links() -> Vec<DiscoveryLink> {
    shipped_package_names()
        .into_iter()
        .map(|package_name| DiscoveryLink {
            relative_path: format!(".agents/skills/{package_name}"),
            target: format!("../../.claude/skills/{package_name}"),
        })
        .collect()
}

fn validate_target_root(target_root: &Path) -> Result<(), TerminalFailure> {
    match fs::metadata(target_root) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(TerminalFailure::refusal(
            format!(
                "target repository root `{}` is not a directory",
                target_root.display()
            ),
            "rerun `install-skills --root <ROOT>` with an existing repository directory",
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(TerminalFailure::refusal(
                format!(
                    "target repository root `{}` does not exist",
                    target_root.display()
                ),
                "create the target repository directory, then rerun `install-skills --root <ROOT>`",
            ))
        }
        Err(error) => Err(TerminalFailure::refusal(
            format!(
                "target repository root `{}` cannot be inspected: {error}",
                target_root.display()
            ),
            "make the target repository root readable, then rerun `install-skills --root <ROOT>`",
        )),
    }
}

fn compare_shipped_files(
    target_root: &Path,
) -> Result<Vec<(&'static ShippedFile, InstalledState)>, TerminalFailure> {
    SHIPPED_FILES
        .iter()
        .map(|shipped| {
            compare_file(&target_root.join(shipped.relative_path), shipped.bytes)
                .map(|state| (shipped, state))
        })
        .collect()
}

fn refuse_unapproved_or_unreplaceable(
    comparisons: &[(&ShippedFile, InstalledState)],
    link_comparisons: &[(DiscoveryLink, InstalledState)],
    stale_paths: &[String],
    force: bool,
) -> Result<(), TerminalFailure> {
    let differing = differing_paths(comparisons, link_comparisons);
    let unreplaceable = unreplaceable_paths(comparisons, link_comparisons);
    if !stale_paths.is_empty() {
        let mut condition = if differing.is_empty() {
            String::new()
        } else {
            format!(
                "installed reuse-evidence skill paths differ from the package this binary ships:\n- {}\n",
                differing.join("\n- ")
            )
        };
        write!(
            &mut condition,
            "stale reuse-evidence skill paths are not shipped by this binary:\n- {}",
            stale_paths.join("\n- ")
        )
        .expect("writing refusal text to a String cannot fail");
        if !unreplaceable.is_empty() {
            write!(
                &mut condition,
                "\npaths that cannot be replaced atomically:\n- {}",
                unreplaceable.join("\n- ")
            )
            .expect("writing refusal text to a String cannot fail");
        }
        let mut resolution = "remove every stale path or move it outside the reserved `reuse-evidence-` package prefix".to_owned();
        if !unreplaceable.is_empty() {
            resolution.push_str(", and remove every path listed as unreplaceable");
        }
        if differing.is_empty() {
            resolution.push_str(", then rerun `install-skills --root <ROOT>`");
        } else {
            resolution.push_str(
                ", then rerun `install-skills --root <ROOT> --force` to replace every differing path",
            );
        }
        return Err(TerminalFailure::refusal(condition, resolution));
    }
    if !differing.is_empty() && !force {
        let mut condition = format!(
            "installed reuse-evidence skill paths differ from the package this binary ships:\n- {}",
            differing.join("\n- ")
        );
        let resolution = if unreplaceable.is_empty() {
            "rerun with `install-skills --root <ROOT> --force` to replace every differing path"
                .to_owned()
        } else {
            write!(
                &mut condition,
                "\npaths that cannot be replaced atomically:\n- {}",
                unreplaceable.join("\n- ")
            )
            .expect("writing refusal text to a String cannot fail");
            "remove every path listed as unreplaceable, then rerun `install-skills --root <ROOT> --force` to replace the remaining differing paths".to_owned()
        };
        return Err(TerminalFailure::refusal(condition, resolution));
    }
    refuse_unreplaceable(comparisons, link_comparisons)
}

fn differing_paths(
    comparisons: &[(&ShippedFile, InstalledState)],
    link_comparisons: &[(DiscoveryLink, InstalledState)],
) -> Vec<String> {
    let mut differing = Vec::new();
    for (shipped, state) in comparisons {
        if matches!(
            state,
            InstalledState::Differing | InstalledState::Unreplaceable
        ) {
            differing.push(shipped.relative_path.to_owned());
        }
    }
    for (link, state) in link_comparisons {
        if matches!(
            state,
            InstalledState::Differing | InstalledState::Unreplaceable
        ) {
            differing.push(link.relative_path.clone());
        }
    }
    differing
}

fn refuse_unreplaceable(
    comparisons: &[(&ShippedFile, InstalledState)],
    link_comparisons: &[(DiscoveryLink, InstalledState)],
) -> Result<(), TerminalFailure> {
    let unreplaceable = unreplaceable_paths(comparisons, link_comparisons);
    if unreplaceable.is_empty() {
        return Ok(());
    }
    Err(TerminalFailure::refusal(
        format!(
            "installed reuse-evidence skill paths cannot be replaced atomically:\n- {}",
            unreplaceable.join("\n- ")
        ),
        "remove every listed directory or special path, then rerun `install-skills --root <ROOT> --force`",
    ))
}

fn unreplaceable_paths(
    comparisons: &[(&ShippedFile, InstalledState)],
    link_comparisons: &[(DiscoveryLink, InstalledState)],
) -> Vec<String> {
    let mut unreplaceable = Vec::new();
    for (shipped, state) in comparisons {
        if *state == InstalledState::Unreplaceable {
            unreplaceable.push(shipped.relative_path.to_owned());
        }
    }
    for (link, state) in link_comparisons {
        if *state == InstalledState::Unreplaceable {
            unreplaceable.push(link.relative_path.clone());
        }
    }
    unreplaceable
}

fn write_shipped_files(
    target_root: &Path,
    comparisons: Vec<(&ShippedFile, InstalledState)>,
) -> Result<(Vec<&'static str>, Vec<&'static str>), TerminalFailure> {
    let mut written = Vec::with_capacity(SHIPPED_FILES.len());
    let mut replaced = Vec::new();
    for (shipped, state) in comparisons {
        let destination = target_root.join(shipped.relative_path);
        match state {
            InstalledState::Matching => {}
            InstalledState::Missing => {
                let Some(parent) = destination.parent() else {
                    return Err(TerminalFailure::unsafe_failure(format!(
                        "shipped skill path `{}` has no parent directory",
                        shipped.relative_path
                    )));
                };
                fs::create_dir_all(parent).map_err(|error| {
                    TerminalFailure::unsafe_failure(format!(
                        "could not create `{}` for `{}`: {error}",
                        parent.display(),
                        shipped.relative_path
                    ))
                })?;
                create_file_atomically(&destination, shipped.bytes)?;
                written.push(shipped.relative_path);
            }
            InstalledState::Differing => {
                replace_file_atomically(&destination, shipped.bytes).map_err(|error| {
                    TerminalFailure::unsafe_failure(format!(
                        "could not replace installed skill file `{}`: {error}",
                        destination.display()
                    ))
                })?;
                replaced.push(shipped.relative_path);
            }
            InstalledState::Unreplaceable => {
                unreachable!("unreplaceable paths refuse before the write pass")
            }
        }
    }
    Ok((written, replaced))
}

fn write_discovery_links(
    target_root: &Path,
    comparisons: Vec<(DiscoveryLink, InstalledState)>,
) -> Result<Vec<DiscoveryLinkOutcome>, TerminalFailure> {
    comparisons
        .into_iter()
        .map(|(link, state)| {
            let action = match state {
                InstalledState::Missing => {
                    if install_discovery_link(target_root, &link)? {
                        DiscoveryLinkAction::Created
                    } else {
                        DiscoveryLinkAction::Unsupported
                    }
                }
                InstalledState::Matching => DiscoveryLinkAction::Unchanged,
                InstalledState::Differing => replace_discovery_link(target_root, &link)?,
                InstalledState::Unreplaceable => {
                    unreachable!("unreplaceable paths refuse before the write pass")
                }
            };
            Ok(DiscoveryLinkOutcome { link, action })
        })
        .collect()
}

fn installation_directories() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    for shipped in SHIPPED_FILES {
        let parent = Path::new(shipped.relative_path)
            .parent()
            .expect("every shipped skill file must have a parent directory");
        push_directory_prefixes(&mut directories, parent);
    }
    #[cfg(any(unix, windows))]
    for link in shipped_discovery_links() {
        let parent = Path::new(&link.relative_path)
            .parent()
            .expect("every discovery link must have a parent directory");
        push_directory_prefixes(&mut directories, parent);
    }
    directories
}

fn push_directory_prefixes(directories: &mut Vec<PathBuf>, path: &Path) {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        prefix.push(component);
        if !directories.contains(&prefix) {
            directories.push(prefix.clone());
        }
    }
}

fn compare_discovery_links(
    target_root: &Path,
) -> Result<Vec<(DiscoveryLink, InstalledState)>, TerminalFailure> {
    shipped_discovery_links()
        .into_iter()
        .map(|link| compare_discovery_link(target_root, &link).map(|state| (link, state)))
        .collect()
}

fn find_stale_paths(target_root: &Path) -> Result<Vec<String>, TerminalFailure> {
    let shipped_paths = shipped_paths();
    let mut stale_paths = Vec::new();
    collect_reserved_entries(
        target_root,
        Path::new(".claude/skills"),
        &shipped_paths,
        &mut stale_paths,
    )?;
    collect_reserved_entries(
        target_root,
        Path::new(".agents/skills"),
        &shipped_paths,
        &mut stale_paths,
    )?;
    Ok(stale_paths)
}

fn shipped_paths() -> HashSet<PathBuf> {
    let mut paths = HashSet::new();
    for shipped in SHIPPED_FILES {
        let relative_path = Path::new(shipped.relative_path);
        paths.insert(relative_path.to_path_buf());
        for ancestor in relative_path
            .parent()
            .expect("every shipped skill file must have a parent directory")
            .ancestors()
        {
            paths.insert(ancestor.to_path_buf());
        }
    }
    for link in shipped_discovery_links() {
        paths.insert(PathBuf::from(link.relative_path));
    }
    paths
}

fn collect_reserved_entries(
    target_root: &Path,
    relative_directory: &Path,
    shipped_paths: &HashSet<PathBuf>,
    stale_paths: &mut Vec<String>,
) -> Result<(), TerminalFailure> {
    let directory = target_root.join(relative_directory);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(reserved_inspection_failure("directory", &directory, &error));
        }
    };
    let mut entries = entries
        .map(|entry| {
            entry.map_err(|error| reserved_inspection_failure("directory", &directory, &error))
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(RESERVED_PACKAGE_PREFIX)
        {
            collect_reserved_path(target_root, &entry.path(), shipped_paths, stale_paths)?;
        }
    }
    Ok(())
}

fn collect_reserved_path(
    target_root: &Path,
    path: &Path,
    shipped_paths: &HashSet<PathBuf>,
    stale_paths: &mut Vec<String>,
) -> Result<(), TerminalFailure> {
    let relative_path = path.strip_prefix(target_root).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "reserved skill path `{}` is not beneath target root `{}`: {error}",
            path.display(),
            target_root.display()
        ))
    })?;
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| reserved_inspection_failure("path", path, &error))?;
    if !shipped_paths.contains(relative_path) && !is_installer_temporary(relative_path, &metadata) {
        stale_paths.push(render_relative_path(relative_path));
    }
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        let mut children = fs::read_dir(path)
            .map_err(|error| reserved_inspection_failure("directory", path, &error))?
            .map(|entry| {
                entry.map_err(|error| reserved_inspection_failure("directory", path, &error))
            })
            .collect::<Result<Vec<_>, _>>()?;
        children.sort_by_key(fs::DirEntry::file_name);
        for child in children {
            collect_reserved_path(target_root, &child.path(), shipped_paths, stale_paths)?;
        }
    }
    Ok(())
}

fn reserved_inspection_failure(
    path_kind: &str,
    path: &Path,
    error: &std::io::Error,
) -> TerminalFailure {
    TerminalFailure::refusal(
        format!(
            "reserved skill {path_kind} `{}` cannot be inspected: {error}",
            path.display()
        ),
        "make every reserved reuse-evidence skill path inspectable, then rerun `install-skills --root <ROOT>`",
    )
}

fn is_installer_temporary(relative_path: &Path, metadata: &fs::Metadata) -> bool {
    if !metadata.is_file() {
        return false;
    }
    let Some(candidate_name) = relative_path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    SHIPPED_FILES.iter().any(|shipped| {
        let destination = Path::new(shipped.relative_path);
        if relative_path.parent() != destination.parent() {
            return false;
        }
        let destination_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .expect("every shipped skill file must have a UTF-8 file name");
        let prefix = format!(".{destination_name}.");
        let Some(identifier) = candidate_name
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix(".tmp"))
        else {
            return false;
        };
        uuid::Uuid::parse_str(identifier).is_ok_and(|id| {
            id.get_version_num() == 4
                && id.get_variant() == uuid::Variant::RFC4122
                && id.to_string() == identifier
        })
    })
}

fn render_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn validate_installation_directories(target_root: &Path) -> Result<(), TerminalFailure> {
    for relative in installation_directories() {
        let directory = target_root.join(&relative);
        match fs::symlink_metadata(&directory) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(TerminalFailure::refusal(
                    format!(
                        "skill installation directory `{}` is a symbolic link",
                        directory.display()
                    ),
                    "replace every skill installation directory symlink with a real directory beneath the target repository",
                ));
            }
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(TerminalFailure::refusal(
                    format!(
                        "skill installation directory `{}` is occupied by a non-directory path",
                        directory.display()
                    ),
                    "replace the occupying path with a real directory beneath the target repository",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(TerminalFailure::refusal(
                    format!(
                        "skill installation directory `{}` cannot be inspected: {error}",
                        directory.display()
                    ),
                    "make every skill installation directory inspectable, then rerun `install-skills --root <ROOT>`",
                ));
            }
        }
    }
    Ok(())
}

fn compare_file(path: &Path, shipped_bytes: &[u8]) -> Result<InstalledState, TerminalFailure> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(InstalledState::Missing);
        }
        Err(error) => {
            return Err(TerminalFailure::refusal(
                format!(
                    "installed skill path `{}` cannot be inspected: {error}",
                    path.display()
                ),
                "make every installed skill path readable, then rerun `install-skills --root <ROOT>`",
            ));
        }
    };
    if metadata.file_type().is_symlink() {
        return Ok(InstalledState::Differing);
    }
    if !metadata.is_file() {
        return Ok(InstalledState::Unreplaceable);
    }
    let installed = fs::read(path).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "installed skill file `{}` cannot be read: {error}",
                path.display()
            ),
            "make every installed skill file readable, then rerun `install-skills --root <ROOT>`",
        )
    })?;
    Ok(if installed == shipped_bytes {
        InstalledState::Matching
    } else {
        InstalledState::Differing
    })
}

#[cfg(any(unix, windows))]
fn compare_discovery_link(
    target_root: &Path,
    shipped_link: &DiscoveryLink,
) -> Result<InstalledState, TerminalFailure> {
    let link = target_root.join(&shipped_link.relative_path);
    match fs::read_link(&link) {
        Ok(target) if target == Path::new(&shipped_link.target) => Ok(InstalledState::Matching),
        Ok(_) => Ok(InstalledState::Differing),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(InstalledState::Missing),
        Err(error) if read_link_reported_non_link(&error) => match fs::symlink_metadata(&link) {
            Ok(metadata) if metadata.is_file() => {
                #[cfg(unix)]
                {
                    Ok(InstalledState::Differing)
                }
                #[cfg(windows)]
                {
                    compare_discovery_regular_file(target_root, &link, shipped_link)
                }
            }
            Ok(_) => Ok(InstalledState::Unreplaceable),
            Err(metadata_error) => Err(TerminalFailure::refusal(
                format!(
                    "discovery link `{}` cannot be inspected: {metadata_error}",
                    link.display()
                ),
                "make the discovery-link path readable, then rerun `install-skills --root <ROOT>`",
            )),
        },
        Err(error) => Err(TerminalFailure::refusal(
            format!(
                "discovery link `{}` cannot be inspected: {error}",
                link.display()
            ),
            "make the discovery-link path readable, then rerun `install-skills --root <ROOT>`",
        )),
    }
}

#[cfg(windows)]
fn compare_discovery_regular_file(
    target_root: &Path,
    link: &Path,
    shipped_link: &DiscoveryLink,
) -> Result<InstalledState, TerminalFailure> {
    let bytes = fs::read(link).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "discovery-link placeholder `{}` cannot be read: {error}",
                link.display()
            ),
            "make the discovery-link path readable, then rerun `install-skills --root <ROOT>`",
        )
    })?;
    Ok(
        if bytes == shipped_link.target.as_bytes()
            && git_index_records_symlink(target_root, shipped_link.relative_path.as_bytes())
        {
            InstalledState::Matching
        } else {
            InstalledState::Differing
        },
    )
}

#[cfg(windows)]
fn git_index_records_symlink(target_root: &Path, tracked_path: &[u8]) -> bool {
    let Some(index_path) = git_index_path(target_root) else {
        return false;
    };
    let Ok(index) = fs::read(index_path) else {
        return false;
    };
    index_records_symlink(&index, tracked_path)
}

#[cfg(windows)]
fn git_index_path(target_root: &Path) -> Option<PathBuf> {
    let dot_git = target_root.join(".git");
    let metadata = fs::metadata(&dot_git).ok()?;
    if metadata.is_dir() {
        return Some(dot_git.join("index"));
    }
    if !metadata.is_file() {
        return None;
    }
    let pointer = fs::read_to_string(dot_git).ok()?;
    let git_dir = pointer.trim().strip_prefix("gitdir: ")?;
    let git_dir = Path::new(git_dir);
    let git_dir = if git_dir.is_absolute() {
        git_dir.to_path_buf()
    } else {
        target_root.join(git_dir)
    };
    Some(git_dir.join("index"))
}

#[cfg(windows)]
fn index_records_symlink(index: &[u8], tracked_path: &[u8]) -> bool {
    const HEADER_LENGTH: usize = 12;
    const FIXED_ENTRY_LENGTH: usize = 62;
    const SHA1_LENGTH: usize = 20;
    const EXTENDED_ENTRY: u16 = 0x4000;
    const SYMLINK_MODE: u32 = 0o120_000;
    const OBJECT_TYPE_MASK: u32 = 0o170_000;

    if index.len() < HEADER_LENGTH + SHA1_LENGTH || &index[..4] != b"DIRC" {
        return false;
    }
    let version = u32::from_be_bytes(index[4..8].try_into().expect("four-byte index version"));
    if !matches!(version, 2 | 3) {
        return false;
    }
    let entry_count = u32::from_be_bytes(index[8..12].try_into().expect("four-byte entry count"));
    let mut cursor = HEADER_LENGTH;
    let mut found = false;
    for _ in 0..entry_count {
        let Some(fixed_end) = cursor.checked_add(FIXED_ENTRY_LENGTH) else {
            return false;
        };
        if fixed_end > index.len().saturating_sub(SHA1_LENGTH) {
            return false;
        }
        let mode = u32::from_be_bytes(
            index[cursor + 24..cursor + 28]
                .try_into()
                .expect("four-byte index mode"),
        );
        let flags = u16::from_be_bytes(
            index[cursor + 60..fixed_end]
                .try_into()
                .expect("two-byte index flags"),
        );
        let path_start = if flags & EXTENDED_ENTRY == 0 {
            fixed_end
        } else if version == 3 {
            let Some(extended_end) = fixed_end.checked_add(2) else {
                return false;
            };
            if extended_end > index.len().saturating_sub(SHA1_LENGTH) {
                return false;
            }
            extended_end
        } else {
            return false;
        };
        let Some(path_length) = index[path_start..].iter().position(|byte| *byte == 0) else {
            return false;
        };
        if &index[path_start..path_start + path_length] == tracked_path
            && mode & OBJECT_TYPE_MASK == SYMLINK_MODE
        {
            found = true;
        }
        let Some(entry_length) = path_start
            .checked_add(path_length + 1)
            .and_then(|end| end.checked_sub(cursor))
        else {
            return false;
        };
        let padding = (8 - entry_length % 8) % 8;
        let Some(next_cursor) = cursor
            .checked_add(entry_length)
            .and_then(|end| end.checked_add(padding))
        else {
            return false;
        };
        if next_cursor > index.len().saturating_sub(SHA1_LENGTH) {
            return false;
        }
        cursor = next_cursor;
    }
    found
}

#[cfg(any(unix, windows))]
fn read_link_reported_non_link(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::InvalidInput {
        return true;
    }
    #[cfg(windows)]
    if error.raw_os_error() == Some(4390) {
        return true;
    }
    false
}

#[cfg(not(any(unix, windows)))]
fn compare_discovery_link(
    _target_root: &Path,
    _shipped_link: &DiscoveryLink,
) -> Result<InstalledState, TerminalFailure> {
    Ok(InstalledState::Missing)
}

#[cfg(any(unix, windows))]
fn install_discovery_link(
    target_root: &Path,
    shipped_link: &DiscoveryLink,
) -> Result<bool, TerminalFailure> {
    let link = target_root.join(&shipped_link.relative_path);
    let parent = link
        .parent()
        .expect("the discovery link has a parent directory");
    fs::create_dir_all(parent).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "could not create `{}` for discovery links: {error}",
            parent.display()
        ))
    })?;
    match create_directory_symlink(Path::new(&shipped_link.target), &link) {
        Ok(()) => Ok(true),
        Err(error) if symlinks_unavailable(&error) => Ok(false),
        Err(error) => Err(TerminalFailure::unsafe_failure(format!(
            "could not create discovery link `{}`: {error}",
            link.display()
        ))),
    }
}

#[cfg(any(unix, windows))]
fn replace_discovery_link(
    target_root: &Path,
    shipped_link: &DiscoveryLink,
) -> Result<DiscoveryLinkAction, TerminalFailure> {
    let link = target_root.join(&shipped_link.relative_path);
    let temporary = crate::temporary_path_for(&link);
    if let Err(error) = create_directory_symlink(Path::new(&shipped_link.target), &temporary) {
        if symlinks_unavailable(&error) {
            remove_discovery_obstruction(&link).map_err(|remove_error| {
                TerminalFailure::unsafe_failure(format!(
                    "could not remove unsupported discovery path `{}`: {remove_error}",
                    link.display()
                ))
            })?;
            return Ok(DiscoveryLinkAction::RemovedUnsupported);
        }
        return Err(TerminalFailure::unsafe_failure(format!(
            "could not prepare replacement discovery link `{}`: {error}",
            link.display()
        )));
    }
    let result =
        publish_prepared_discovery_link(&temporary, &link, Path::new(&shipped_link.target));
    if let Err(error) = result {
        remove_temporary_discovery_link(&temporary);
        return Err(TerminalFailure::unsafe_failure(format!(
            "could not replace discovery link `{}`: {error}",
            link.display()
        )));
    }
    Ok(DiscoveryLinkAction::Replaced)
}

#[cfg(not(any(unix, windows)))]
fn install_discovery_link(
    _target_root: &Path,
    _shipped_link: &DiscoveryLink,
) -> Result<bool, TerminalFailure> {
    Ok(false)
}

#[cfg(not(any(unix, windows)))]
fn replace_discovery_link(
    _target_root: &Path,
    _shipped_link: &DiscoveryLink,
) -> Result<DiscoveryLinkAction, TerminalFailure> {
    Ok(DiscoveryLinkAction::Unsupported)
}

#[cfg(unix)]
fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)?;
    match fs::read_link(link) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "the platform reported link creation without creating the discovery path",
        )),
        Err(error) => Err(error),
    }
}

#[cfg(any(unix, windows))]
fn symlinks_unavailable(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(windows)]
    if error.raw_os_error() == Some(1314) {
        return true;
    }
    false
}

#[cfg(unix)]
fn remove_temporary_discovery_link(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(windows)]
fn remove_temporary_discovery_link(path: &Path) {
    let _ = fs::remove_dir(path);
}

#[cfg(windows)]
fn remove_discovery_obstruction(path: &Path) -> std::io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            fs::remove_dir(path).or_else(|_| fs::remove_file(path))
        }
        Ok(_) => fs::remove_file(path),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn remove_discovery_obstruction(path: &Path) -> std::io::Result<()> {
    fs::remove_file(path)
}

#[cfg(unix)]
fn publish_prepared_discovery_link(
    temporary: &Path,
    link: &Path,
    _target: &Path,
) -> std::io::Result<()> {
    fs::rename(temporary, link).and_then(|()| crate::sync_parent(link))
}

#[cfg(windows)]
fn publish_prepared_discovery_link(
    temporary: &Path,
    link: &Path,
    target: &Path,
) -> std::io::Result<()> {
    fs::remove_dir(temporary)
        .or_else(|_| fs::remove_file(temporary))
        .map_err(|error| {
            std::io::Error::new(
                error.kind(),
                format!("could not remove prepared link: {error}"),
            )
        })?;
    remove_discovery_obstruction(link).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!("could not remove existing discovery path: {error}"),
        )
    })?;
    create_directory_symlink(target, link).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!("could not create replacement link: {error}"),
        )
    })
}
