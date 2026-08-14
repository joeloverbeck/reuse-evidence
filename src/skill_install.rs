//! Installation of this project's own agent-skill packages.

use std::fmt::{self, Write as _};
use std::fs;
use std::path::Path;

use crate::{TerminalFailure, create_file_atomically, replace_file_atomically};

struct ShippedFile {
    relative_path: &'static str,
    bytes: &'static [u8],
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
];

const DISCOVERY_LINK: &str = ".agents/skills/reuse-evidence-capture";
const DISCOVERY_TARGET: &str = "../../.claude/skills/reuse-evidence-capture";
const SKILL_DIRECTORIES: &[&str] = &[
    ".claude",
    ".claude/skills",
    ".claude/skills/reuse-evidence-capture",
    ".claude/skills/reuse-evidence-capture/agents",
    ".claude/skills/reuse-evidence-capture/references",
];
#[cfg(any(unix, windows))]
const DISCOVERY_DIRECTORIES: &[&str] = &[".agents", ".agents/skills"];

/// The observable result of installing this project's skill packages.
#[derive(Debug, Eq, PartialEq)]
pub struct SkillInstallOutcome {
    written: Vec<&'static str>,
    replaced: Vec<&'static str>,
    discovery_link: DiscoveryLinkAction,
}

impl fmt::Display for SkillInstallOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.written.is_empty()
            && self.replaced.is_empty()
            && matches!(
                self.discovery_link,
                DiscoveryLinkAction::Unchanged | DiscoveryLinkAction::Unsupported
            )
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
            if self.discovery_link == DiscoveryLinkAction::Created {
                writeln!(formatter, "discovery links created:")?;
                writeln!(formatter, "- {DISCOVERY_LINK} -> {DISCOVERY_TARGET}")?;
            }
            if self.discovery_link == DiscoveryLinkAction::Replaced {
                writeln!(formatter, "discovery links replaced:")?;
                writeln!(formatter, "- {DISCOVERY_LINK} -> {DISCOVERY_TARGET}")?;
            }
        }
        if self.discovery_link == DiscoveryLinkAction::Unsupported {
            writeln!(
                formatter,
                "discovery links not created: symbolic links are not supported on this platform"
            )?;
            writeln!(formatter, "- {DISCOVERY_LINK}")?;
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
    let link_state = compare_discovery_link(target_root)?;
    refuse_unapproved_or_unreplaceable(&comparisons, link_state, force)?;
    let (written, replaced) = write_shipped_files(target_root, comparisons)?;
    let discovery_link = write_discovery_link(target_root, link_state)?;
    Ok(SkillInstallOutcome {
        written,
        replaced,
        discovery_link,
    })
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
    link_state: InstalledState,
    force: bool,
) -> Result<(), TerminalFailure> {
    let mut differing = comparisons
        .iter()
        .filter_map(|(shipped, state)| {
            matches!(
                state,
                InstalledState::Differing | InstalledState::Unreplaceable
            )
            .then_some(shipped.relative_path)
        })
        .collect::<Vec<_>>();
    if matches!(
        link_state,
        InstalledState::Differing | InstalledState::Unreplaceable
    ) {
        differing.push(DISCOVERY_LINK);
    }
    if !differing.is_empty() && !force {
        let mut condition = format!(
            "installed reuse-evidence skill paths differ from the package this binary ships:\n- {}",
            differing.join("\n- ")
        );
        let unreplaceable = unreplaceable_paths(comparisons, link_state);
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
    refuse_unreplaceable(comparisons, link_state)
}

fn refuse_unreplaceable(
    comparisons: &[(&ShippedFile, InstalledState)],
    link_state: InstalledState,
) -> Result<(), TerminalFailure> {
    let unreplaceable = unreplaceable_paths(comparisons, link_state);
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
    link_state: InstalledState,
) -> Vec<&'static str> {
    let mut unreplaceable = comparisons
        .iter()
        .filter_map(|(shipped, state)| {
            (*state == InstalledState::Unreplaceable).then_some(shipped.relative_path)
        })
        .collect::<Vec<_>>();
    if link_state == InstalledState::Unreplaceable {
        unreplaceable.push(DISCOVERY_LINK);
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

fn write_discovery_link(
    target_root: &Path,
    link_state: InstalledState,
) -> Result<DiscoveryLinkAction, TerminalFailure> {
    Ok(match link_state {
        InstalledState::Missing => {
            if install_discovery_link(target_root)? {
                DiscoveryLinkAction::Created
            } else {
                DiscoveryLinkAction::Unsupported
            }
        }
        InstalledState::Matching => DiscoveryLinkAction::Unchanged,
        InstalledState::Differing => {
            if replace_discovery_link(target_root)? {
                DiscoveryLinkAction::Replaced
            } else {
                DiscoveryLinkAction::Unsupported
            }
        }
        InstalledState::Unreplaceable => {
            unreachable!("unreplaceable paths refuse before the write pass")
        }
    })
}

fn validate_installation_directories(target_root: &Path) -> Result<(), TerminalFailure> {
    for relative in SKILL_DIRECTORIES
        .iter()
        .copied()
        .chain(discovery_directories())
    {
        let directory = target_root.join(relative);
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

#[cfg(any(unix, windows))]
fn discovery_directories() -> impl Iterator<Item = &'static str> {
    DISCOVERY_DIRECTORIES.iter().copied()
}

#[cfg(not(any(unix, windows)))]
fn discovery_directories() -> impl Iterator<Item = &'static str> {
    std::iter::empty()
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
fn compare_discovery_link(target_root: &Path) -> Result<InstalledState, TerminalFailure> {
    let link = target_root.join(DISCOVERY_LINK);
    match fs::read_link(&link) {
        Ok(target) if target == Path::new(DISCOVERY_TARGET) => Ok(InstalledState::Matching),
        Ok(_) => Ok(InstalledState::Differing),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(InstalledState::Missing),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => {
            match fs::symlink_metadata(&link) {
                Ok(metadata) if metadata.is_file() => Ok(InstalledState::Differing),
                Ok(_) => Ok(InstalledState::Unreplaceable),
                Err(metadata_error) => Err(TerminalFailure::refusal(
                    format!(
                        "discovery link `{}` cannot be inspected: {metadata_error}",
                        link.display()
                    ),
                    "make the discovery-link path readable, then rerun `install-skills --root <ROOT>`",
                )),
            }
        }
        Err(error) => Err(TerminalFailure::refusal(
            format!(
                "discovery link `{}` cannot be inspected: {error}",
                link.display()
            ),
            "make the discovery-link path readable, then rerun `install-skills --root <ROOT>`",
        )),
    }
}

#[cfg(not(any(unix, windows)))]
fn compare_discovery_link(_target_root: &Path) -> Result<InstalledState, TerminalFailure> {
    Ok(InstalledState::Missing)
}

#[cfg(any(unix, windows))]
fn install_discovery_link(target_root: &Path) -> Result<bool, TerminalFailure> {
    let link = target_root.join(DISCOVERY_LINK);
    let parent = link
        .parent()
        .expect("the discovery link has a parent directory");
    fs::create_dir_all(parent).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "could not create `{}` for discovery links: {error}",
            parent.display()
        ))
    })?;
    match create_directory_symlink(Path::new(DISCOVERY_TARGET), &link) {
        Ok(()) => Ok(true),
        Err(error) if symlinks_unavailable(&error) => Ok(false),
        Err(error) => Err(TerminalFailure::unsafe_failure(format!(
            "could not create discovery link `{}`: {error}",
            link.display()
        ))),
    }
}

#[cfg(any(unix, windows))]
fn replace_discovery_link(target_root: &Path) -> Result<bool, TerminalFailure> {
    let link = target_root.join(DISCOVERY_LINK);
    let temporary = crate::temporary_path_for(&link);
    if let Err(error) = create_directory_symlink(Path::new(DISCOVERY_TARGET), &temporary) {
        if symlinks_unavailable(&error) {
            remove_discovery_obstruction(&link).map_err(|remove_error| {
                TerminalFailure::unsafe_failure(format!(
                    "could not remove unsupported discovery path `{}`: {remove_error}",
                    link.display()
                ))
            })?;
            return Ok(false);
        }
        return Err(TerminalFailure::unsafe_failure(format!(
            "could not prepare replacement discovery link `{}`: {error}",
            link.display()
        )));
    }
    #[cfg(windows)]
    remove_discovery_obstruction(&link).map_err(|error| {
        let _ = fs::remove_dir(&temporary);
        TerminalFailure::unsafe_failure(format!(
            "could not remove the replaced discovery path `{}`: {error}",
            link.display()
        ))
    })?;
    let result = publish_prepared_discovery_link(&temporary, &link);
    if let Err(error) = result {
        remove_temporary_discovery_link(&temporary);
        return Err(TerminalFailure::unsafe_failure(format!(
            "could not replace discovery link `{}`: {error}",
            link.display()
        )));
    }
    Ok(true)
}

#[cfg(not(any(unix, windows)))]
fn install_discovery_link(_target_root: &Path) -> Result<bool, TerminalFailure> {
    Ok(false)
}

#[cfg(not(any(unix, windows)))]
fn replace_discovery_link(_target_root: &Path) -> Result<bool, TerminalFailure> {
    Ok(false)
}

#[cfg(unix)]
fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
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
fn publish_prepared_discovery_link(temporary: &Path, link: &Path) -> std::io::Result<()> {
    fs::rename(temporary, link).and_then(|()| crate::sync_parent(link))
}

#[cfg(windows)]
fn publish_prepared_discovery_link(temporary: &Path, link: &Path) -> std::io::Result<()> {
    fs::rename(temporary, link)
}
