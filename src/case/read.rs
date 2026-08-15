use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::naming::{self, EventPosition};
use super::replay::{self, CaseRecord, Conditions};
use super::{
    DecisionAuthorization, DecisionContent, ReportedPrivacy, complete_case_privacy,
    find_repository_root, read_steward, validate_case_storage_path,
};
use crate::{ExitMeaning, TerminalFailure, Visibility, portfolio};

/// What each query command tells a reader to do about a steward marker it
/// cannot use. ADR 0018 makes the fault's wording shared and this sentence the
/// command's; `case` holds the five for the recording commands.
const LIST_MARKER_RESOLUTION: &str =
    "restore a supported `reuse-evidence.toml` marker before listing cases";
const SHOW_MARKER_RESOLUTION: &str =
    "restore a supported `reuse-evidence.toml` marker before showing a case";
const BRIEF_MARKER_RESOLUTION: &str =
    "restore a supported `reuse-evidence.toml` marker before projecting an implementation brief";

/// The steward-local case listing, as read.
pub struct ListOutcome {
    pub(super) cases: Vec<CaseRecord>,
    pub(super) portfolio_available: bool,
}

/// One steward-local case, as read.
pub struct ShowOutcome {
    pub(super) case: CaseRecord,
    pub(super) portfolio_available: bool,
}

/// The implementation handoff projected from one accepted reuse decision.
///
/// The decision content is held beside the case, and what it authorizes is resolved before this
/// value exists, so rendering the brief cannot fail on a field the recorded decision omitted.
pub struct BriefOutcome {
    pub(super) case: CaseRecord,
    pub(super) privacy: ReportedPrivacy,
    pub(super) decision: DecisionContent,
    pub(super) authorization: DecisionAuthorization,
}

/// Every case or damaged-case condition found across the selected enrolled portfolio.
pub struct FindOutcome {
    pub(super) cases: Vec<PortfolioCaseRecord>,
}

/// One case together with the enrolled steward that owns its event stream.
pub(super) struct PortfolioCaseRecord {
    pub(super) steward_repository_id: Uuid,
    pub(super) steward_path: PathBuf,
    pub(super) case_id: Uuid,
    pub(super) state: PortfolioCaseState,
}

pub(super) enum PortfolioCaseState {
    Recorded {
        case: Box<CaseRecord>,
        privacy: ReportedPrivacy,
    },
    Damaged {
        detail: String,
    },
}

/// Finds every case stewarded beneath the selected portfolio roots.
///
/// # Errors
///
/// Returns a refusal when no root selection resolves or the enrolled steward
/// directories cannot be enumerated safely. Damage inside an identified case
/// is returned as that case's condition so healthy neighbours remain visible.
pub fn find(location: &portfolio::PortfolioLocation) -> Result<FindOutcome, TerminalFailure> {
    let roots = portfolio::selected_roots(location)?;
    let scan = portfolio::scan(&roots)?;
    let mut cases = Vec::new();
    for enrollment in &scan.enrollments {
        for directory in case_directories(&enrollment.path)? {
            let state = match read_case(
                &enrollment.path,
                &directory.relative_path,
                directory.case_id,
                enrollment.repository_id,
            ) {
                Ok(case) => {
                    let privacy =
                        portfolio_case_privacy(&case, enrollment.visibility, &scan.enrollments);
                    PortfolioCaseState::Recorded {
                        case: Box::new(case),
                        privacy,
                    }
                }
                Err(failure) if failure.meaning() == ExitMeaning::Refusal => {
                    PortfolioCaseState::Damaged {
                        detail: failure.to_string(),
                    }
                }
                Err(failure) => return Err(failure),
            };
            cases.push(PortfolioCaseRecord {
                steward_repository_id: enrollment.repository_id,
                steward_path: enrollment.path.clone(),
                case_id: directory.case_id,
                state,
            });
        }
    }
    cases.sort_by(|left, right| {
        left.case_id
            .cmp(&right.case_id)
            .then_with(|| left.steward_repository_id.cmp(&right.steward_repository_id))
            .then_with(|| left.steward_path.cmp(&right.steward_path))
    });
    Ok(FindOutcome { cases })
}

fn portfolio_case_privacy(
    case: &CaseRecord,
    steward_visibility: Visibility,
    enrollments: &[portfolio::Enrollment],
) -> ReportedPrivacy {
    let requested = case
        .occurrences
        .iter()
        .map(|occurrence| occurrence.repository_id)
        .collect::<BTreeSet<_>>();
    let mut participant_visibilities = Vec::with_capacity(requested.len());
    for repository_id in requested {
        let mut matches = enrollments
            .iter()
            .filter(|enrollment| enrollment.repository_id == repository_id);
        let Some(enrollment) = matches.next() else {
            return ReportedPrivacy::Derived(Visibility::Private);
        };
        if matches.next().is_some() {
            return ReportedPrivacy::Derived(Visibility::Private);
        }
        participant_visibilities.push(enrollment.visibility);
    }
    ReportedPrivacy::Derived(complete_case_privacy(
        case,
        steward_visibility,
        participant_visibilities,
    ))
}

/// Lists every case stewarded by the enrolled repository containing
/// `working_directory`.
///
/// # Errors
///
/// Returns a refusal when the steward or any recorded case cannot be read
/// safely.
pub fn list(
    working_directory: &Path,
    location: &portfolio::PortfolioLocation,
) -> Result<ListOutcome, TerminalFailure> {
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root, LIST_MARKER_RESOLUTION)?;
    let mut cases = read_cases(&repository_root, steward.repository_id())?;
    let portfolio_available = derive_conditions(&mut cases, steward.visibility(), location)?;
    Ok(ListOutcome {
        cases,
        portfolio_available,
    })
}

/// Shows one case stewarded by the enrolled repository containing
/// `working_directory`.
///
/// # Errors
///
/// Returns a refusal when the case identity, steward, or recorded case cannot
/// be read safely.
pub fn show(
    working_directory: &Path,
    case_id: &str,
    location: &portfolio::PortfolioLocation,
) -> Result<ShowOutcome, TerminalFailure> {
    let case_id = parse_recorded_case_id(case_id)?;
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root, SHOW_MARKER_RESOLUTION)?;
    let relative_case_directory = naming::case_directory(case_id);
    validate_case_storage_path(&repository_root, &relative_case_directory)?;
    let mut case = read_case(
        &repository_root,
        &relative_case_directory,
        case_id,
        steward.repository_id(),
    )?;
    let portfolio_available = derive_conditions(
        std::slice::from_mut(&mut case),
        steward.visibility(),
        location,
    )?;
    Ok(ShowOutcome {
        case,
        portfolio_available,
    })
}

/// Projects the implementation brief for one accepted steward-local decision.
///
/// # Errors
///
/// Returns a refusal when the identity, steward, case, or accepted decision
/// cannot be read safely.
pub fn brief(
    working_directory: &Path,
    case_id: &str,
    location: &portfolio::PortfolioLocation,
) -> Result<BriefOutcome, TerminalFailure> {
    let case_id = parse_recorded_case_id(case_id)?;
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root, BRIEF_MARKER_RESOLUTION)?;
    let relative_case_directory = naming::case_directory(case_id);
    validate_case_storage_path(&repository_root, &relative_case_directory)?;
    let case = read_case_for(
        &repository_root,
        &relative_case_directory,
        case_id,
        steward.repository_id(),
        "run `case list` in this steward repository, then retry `case brief <CASE_ID>` with one of its recorded case identities",
    )?;
    let Some(decision) = case.decision.as_ref() else {
        let state = case.state();
        let resolution = if state.authorizes_review() {
            format!("record an accepted reuse decision, then rerun `case brief {case_id}`")
        } else {
            format!(
                "make the case review-ready, record an accepted reuse decision, then rerun `case brief {case_id}`"
            )
        };
        return Err(TerminalFailure::refusal(
            format!(
                "case `{case_id}` has no accepted reuse decision; current state is `{}`",
                state.label()
            ),
            resolution,
        ));
    };
    let content = decision.content.clone();
    let authorization = content.authorization()?;
    let privacy = super::reported_privacy(&case, &steward, location);
    Ok(BriefOutcome {
        case,
        privacy,
        decision: content,
        authorization,
    })
}

fn parse_recorded_case_id(case_id: &str) -> Result<Uuid, TerminalFailure> {
    Uuid::parse_str(case_id).map_err(|error| {
        TerminalFailure::refusal(
            format!("case identity `{case_id}` is invalid: {error}"),
            "supply the opaque UUID recorded for the stewarded case",
        )
    })
}

fn derive_conditions(
    cases: &mut [CaseRecord],
    steward_visibility: Visibility,
    location: &portfolio::PortfolioLocation,
) -> Result<bool, TerminalFailure> {
    let Some(roots) = portfolio::selected_roots_if_configured(location)? else {
        return Ok(false);
    };
    let scan = portfolio::scan(&roots)?;
    for case in cases {
        // Recorded case privacy is a term of complete case privacy, not only current participant
        // visibility, so a steward that turned public after opening conflicts with its own case.
        let mut privacy_conflicted =
            steward_visibility == Visibility::Public && case.privacy == Visibility::Private;
        let mut privacy_underivable = false;
        let mut stale = false;
        for occurrence in &case.occurrences {
            let matches = scan
                .enrollments
                .iter()
                .filter(|enrollment| enrollment.repository_id == occurrence.repository_id)
                .collect::<Vec<_>>();
            let participant_unresolved = matches.len() != 1;
            privacy_underivable |= participant_unresolved;
            stale |= participant_unresolved;
            privacy_conflicted |= steward_visibility == Visibility::Public
                && matches
                    .iter()
                    .any(|enrollment| enrollment.visibility == Visibility::Private);
        }
        case.conditions = Conditions {
            privacy_conflicted: if privacy_conflicted {
                Some(true)
            } else if privacy_underivable {
                None
            } else {
                Some(false)
            },
            stale: Some(stale),
        };
    }
    Ok(true)
}

fn read_cases(
    repository_root: &Path,
    steward_repository_id: Uuid,
) -> Result<Vec<CaseRecord>, TerminalFailure> {
    let mut cases = case_directories(repository_root)?
        .into_iter()
        .map(|directory| {
            read_case(
                repository_root,
                &directory.relative_path,
                directory.case_id,
                steward_repository_id,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    cases.sort_by_key(|case| case.case_id);
    Ok(cases)
}

struct CaseDirectory {
    case_id: Uuid,
    relative_path: PathBuf,
}

fn case_directories(repository_root: &Path) -> Result<Vec<CaseDirectory>, TerminalFailure> {
    let relative_cases_root = naming::cases_root();
    validate_case_storage_path(repository_root, relative_cases_root)?;
    let cases_root = repository_root.join(relative_cases_root);
    let entries = match fs::read_dir(&cases_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(TerminalFailure::refusal(
                format!(
                    "steward-local case directory `{}` cannot be read: {error}",
                    cases_root.display()
                ),
                "make the steward-local case directory readable before listing cases",
            ));
        }
    };
    let mut case_directories = entries
        .map(|entry| {
            entry.map_err(|error| {
                TerminalFailure::refusal(
                    format!(
                        "an entry in steward-local case directory `{}` cannot be read: {error}",
                        cases_root.display()
                    ),
                    "make the steward-local case directory readable before listing cases",
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    case_directories.sort_by_key(std::fs::DirEntry::file_name);

    let mut cases = Vec::new();
    for entry in case_directories {
        let case_id_text = entry.file_name().into_string().map_err(|_| {
            TerminalFailure::refusal(
                format!(
                    "case directory `{}` does not have a UTF-8 opaque identity",
                    entry.path().display()
                ),
                "restore a case directory named by its recorded UUID identity",
            )
        })?;
        let case_id = Uuid::parse_str(&case_id_text).map_err(|error| {
            TerminalFailure::refusal(
                format!("case directory identity `{case_id_text}` is invalid: {error}"),
                "restore a case directory named by its recorded UUID identity",
            )
        })?;
        let relative_case_directory = relative_cases_root.join(&case_id_text);
        validate_case_storage_path(repository_root, &relative_case_directory)?;
        cases.push(CaseDirectory {
            case_id,
            relative_path: relative_case_directory,
        });
    }
    Ok(cases)
}

/// Returns the first opaque identity, in case-reader order, whose recorded privacy is private.
pub(crate) fn private_case_stewarded_by(
    repository_root: &Path,
    steward_repository_id: Uuid,
) -> Result<Option<Uuid>, TerminalFailure> {
    Ok(read_cases(repository_root, steward_repository_id)?
        .into_iter()
        .find(|case| case.privacy == Visibility::Private)
        .map(|case| case.case_id))
}

fn read_case(
    repository_root: &Path,
    relative_case_directory: &Path,
    case_id: Uuid,
    steward_repository_id: Uuid,
) -> Result<CaseRecord, TerminalFailure> {
    let case_directory = repository_root.join(relative_case_directory);
    let entries = fs::read_dir(&case_directory).map_err(|error| {
        TerminalFailure::refusal(
            format!("case `{case_id}` cannot be read: {error}"),
            "make every recorded event in the case readable before retrying",
        )
    })?;
    let mut event_paths = entries
        .map(|entry| {
            let entry = entry.map_err(|error| {
                TerminalFailure::refusal(
                    format!("an event in case `{case_id}` cannot be read: {error}"),
                    "make every recorded event in the case readable before retrying",
                )
            })?;
            if naming::is_staged_temporary(&entry.file_name(), EventPosition::Later) {
                let file_type = entry.file_type().map_err(|error| {
                    TerminalFailure::refusal(
                        format!("an event in case `{case_id}` cannot be read: {error}"),
                        "make every recorded event in the case readable before retrying",
                    )
                })?;
                if file_type.is_file() {
                    return Ok(None);
                }
            }
            Ok(Some(entry.path()))
        })
        .collect::<Result<Vec<_>, TerminalFailure>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    event_paths.sort();
    replay::replay(
        repository_root,
        case_id,
        steward_repository_id,
        &event_paths,
        |event_path| {
            fs::read_to_string(event_path).map_err(|error| {
                TerminalFailure::refusal(
                    format!(
                        "case event `{}` cannot be read: {error}",
                        event_path.display()
                    ),
                    "restore the recorded UTF-8 event before reading the case",
                )
            })
        },
    )
}

/// Reads a case an operation addressed by identity, refusing one this repository does not steward.
///
/// The refusal condition is the same for every operation; only the resolution differs, because it
/// names the command to retry and the case state that command requires.
pub(super) fn read_case_for(
    repository_root: &Path,
    relative_case_directory: &Path,
    case_id: Uuid,
    steward_repository_id: Uuid,
    unstewarded_resolution: &str,
) -> Result<CaseRecord, TerminalFailure> {
    let case_directory = repository_root.join(relative_case_directory);
    if matches!(
        fs::metadata(&case_directory),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound
    ) {
        return Err(TerminalFailure::refusal(
            format!(
                "case identity `{case_id}` is not stewarded by repository `{steward_repository_id}`"
            ),
            unstewarded_resolution,
        ));
    }
    read_case(
        repository_root,
        relative_case_directory,
        case_id,
        steward_repository_id,
    )
}
