//! Durable case-opening mechanics.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::marker::{self, MarkerRead};
use crate::portfolio;
use crate::{TerminalFailure, Visibility, create_file_atomically};

const CASE_SCHEMA_VERSION: i64 = 1;
const OPENING_SEQUENCE: i64 = 1;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OpenProposalDocument {
    Prepared(CaseOpenedEvent),
    Human(HumanOpenProposalDocument),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HumanOpenProposalDocument {
    case_id: String,
    responsibility: String,
    occurrences: Vec<Occurrence>,
}

#[derive(Debug)]
struct OpenProposal {
    case_id: Uuid,
    responsibility: String,
    occurrences: Vec<Occurrence>,
    prepared: Option<PreparedOpening>,
}

#[derive(Debug)]
struct PreparedOpening {
    steward_repository_id: Uuid,
    privacy: Visibility,
    bytes: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Occurrence {
    repository_id: Uuid,
    consumer: String,
    independence: String,
    evidence: Vec<EvidenceReference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct EvidenceReference {
    kind: EvidenceKind,
    reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EvidenceKind {
    Commit,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CaseOpenedEvent {
    schema_version: i64,
    sequence: i64,
    event_id: Uuid,
    event_type: EventType,
    recorded_at: String,
    case_id: Uuid,
    responsibility: String,
    steward_repository_id: Uuid,
    privacy: Visibility,
    occurrences: Vec<Occurrence>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EventType {
    CaseOpened,
}

/// The complete observable result of opening or previewing a case.
#[derive(Debug)]
pub struct OpenOutcome {
    effect: OpenEffect,
    case_id: Uuid,
    event_path: PathBuf,
    privacy: Visibility,
    event: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpenEffect {
    Preview,
    Created,
    Existing,
}

impl OpenOutcome {
    /// Renders the receipt followed by the exact event bytes.
    #[must_use]
    pub fn render(&self) -> String {
        let heading = match self.effect {
            OpenEffect::Preview => "case open preview",
            OpenEffect::Created => "opened case",
            OpenEffect::Existing => "existing case",
        };
        let mut receipt = format!(
            "{heading}\ncase_id: {}\nfile: {}\nrevision: {OPENING_SEQUENCE}\nprivacy: {}\n",
            self.case_id,
            self.event_path.display(),
            self.privacy
        );
        if self.effect == OpenEffect::Preview {
            receipt.push_str("event:\n");
            receipt.push_str(&self.event);
        }
        receipt
    }
}

/// Opens or previews a case in the enrolled steward repository.
///
/// # Errors
///
/// Returns a classified failure when the steward, proposal, roots, or
/// participant repositories cannot be read safely.
pub fn open(
    working_directory: &Path,
    proposal_path: &Path,
    root_overrides: &[PathBuf],
    preview: bool,
) -> Result<OpenOutcome, TerminalFailure> {
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root)?;
    let proposal = read_proposal(proposal_path)?;
    let event_path = PathBuf::from("reuse-evidence/cases")
        .join(proposal.case_id.to_string())
        .join("0001-case-opened.toml");
    validate_case_storage_path(&repository_root, &event_path)?;
    let absolute_event_path = repository_root.join(&event_path);
    if absolute_event_path.exists() {
        return existing_opening(
            &absolute_event_path,
            event_path,
            &steward,
            &proposal,
            preview,
        );
    }
    let participants = resolve_participants(root_overrides, &proposal)?;
    if steward.visibility() == Visibility::Public
        && let Some(repository_id) = participants
            .iter()
            .find(|(_, visibility)| **visibility == Visibility::Private)
            .map(|(repository_id, _)| *repository_id)
    {
        return Err(TerminalFailure::refusal(
            format!(
                "public steward `{}` cannot hold private participant `{repository_id}`",
                steward.repository_id()
            ),
            "open the case from an enrolled private steward repository",
        ));
    }
    let privacy = if steward.visibility() == Visibility::Private
        || participants
            .values()
            .any(|visibility| *visibility == Visibility::Private)
    {
        Visibility::Private
    } else {
        Visibility::Public
    };
    let case_id = proposal.case_id;
    let event = event_bytes(&proposal, &steward, privacy)?;
    let effect = if preview {
        OpenEffect::Preview
    } else {
        let case_directory = absolute_event_path.parent().ok_or_else(|| {
            TerminalFailure::unsafe_failure("case opening event path has no parent directory")
        })?;
        prepare_case_directory(case_directory, case_id)?;
        create_file_atomically(&absolute_event_path, event.as_bytes())?;
        cleanup_opening_temporaries(case_directory)?;
        OpenEffect::Created
    };
    Ok(OpenOutcome {
        effect,
        case_id,
        event_path,
        privacy,
        event,
    })
}

fn event_bytes(
    proposal: &OpenProposal,
    steward: &marker::Marker,
    privacy: Visibility,
) -> Result<String, TerminalFailure> {
    if let Some(prepared) = &proposal.prepared {
        if prepared.steward_repository_id != steward.repository_id() {
            return Err(TerminalFailure::refusal(
                format!(
                    "prepared opening event names steward `{}`, but the current steward is `{}`",
                    prepared.steward_repository_id,
                    steward.repository_id()
                ),
                "preview the proposal again from the enrolled repository that will steward the case",
            ));
        }
        if prepared.privacy != privacy {
            return Err(TerminalFailure::refusal(
                format!(
                    "prepared opening event declares privacy `{}`, but current participant visibility derives `{privacy}`",
                    prepared.privacy
                ),
                "refresh enrollment visibility and preview the proposal again before opening the case",
            ));
        }
        return Ok(prepared.bytes.clone());
    }

    let event = CaseOpenedEvent {
        schema_version: CASE_SCHEMA_VERSION,
        sequence: OPENING_SEQUENCE,
        event_id: Uuid::new_v4(),
        event_type: EventType::CaseOpened,
        recorded_at: recording_timestamp()?,
        case_id: proposal.case_id,
        responsibility: proposal.responsibility.clone(),
        steward_repository_id: steward.repository_id(),
        privacy,
        occurrences: proposal.occurrences.clone(),
    };
    toml::to_string(&event).map_err(|error| {
        TerminalFailure::unsafe_failure(format!("case opening event could not be encoded: {error}"))
    })
}

fn prepare_case_directory(case_directory: &Path, case_id: Uuid) -> Result<(), TerminalFailure> {
    if case_directory.exists() {
        for entry in fs::read_dir(case_directory).map_err(|error| {
            TerminalFailure::refusal(
                format!(
                    "case directory `{}` cannot be inspected: {error}",
                    case_directory.display()
                ),
                "make the steward-local case directory readable before retrying",
            )
        })? {
            let entry = entry.map_err(|error| {
                TerminalFailure::refusal(
                    format!(
                        "an entry in case directory `{}` cannot be inspected: {error}",
                        case_directory.display()
                    ),
                    "make the steward-local case directory readable before retrying",
                )
            })?;
            if !is_opening_temporary(&entry.file_name()) {
                return Err(TerminalFailure::refusal(
                    format!(
                        "case identity `{case_id}` already has unrecognized content at `{}`",
                        entry.path().display()
                    ),
                    "restore the original case record or choose a new UUID version 4 case identity",
                ));
            }
        }
        return Ok(());
    }
    fs::create_dir_all(case_directory).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "case directory `{}` could not be created: {error}",
            case_directory.display()
        ))
    })
}

fn cleanup_opening_temporaries(case_directory: &Path) -> Result<(), TerminalFailure> {
    for entry in fs::read_dir(case_directory).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "case directory `{}` could not be inspected after publishing its event: {error}",
            case_directory.display()
        ))
    })? {
        let entry = entry.map_err(|error| {
            TerminalFailure::unsafe_failure(format!(
                "a case directory entry could not be inspected after publishing the event: {error}"
            ))
        })?;
        if is_opening_temporary(&entry.file_name()) {
            fs::remove_file(entry.path()).map_err(|error| {
                TerminalFailure::unsafe_failure(format!(
                    "interrupted case staging file `{}` could not be removed: {error}",
                    entry.path().display()
                ))
            })?;
        }
    }
    Ok(())
}

fn is_opening_temporary(file_name: &std::ffi::OsStr) -> bool {
    let Some(file_name) = file_name.to_str() else {
        return false;
    };
    file_name
        .strip_prefix(".0001-case-opened.toml.")
        .and_then(|suffix| suffix.strip_suffix(".tmp"))
        .is_some_and(|identity| Uuid::parse_str(identity).is_ok())
}

fn validate_case_storage_path(
    repository_root: &Path,
    relative_event_path: &Path,
) -> Result<(), TerminalFailure> {
    let components = relative_event_path.components().collect::<Vec<_>>();
    let mut current = repository_root.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(component) = component else {
            return Err(TerminalFailure::unsafe_failure(
                "internally constructed case event path is not repository-relative",
            ));
        };
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(TerminalFailure::refusal(
                    format!(
                        "case storage path `{}` is a symbolic link",
                        current.display()
                    ),
                    "replace every case storage symlink with a real directory or file inside the steward repository",
                ));
            }
            Ok(metadata) if index + 1 < components.len() && !metadata.is_dir() => {
                return Err(TerminalFailure::refusal(
                    format!(
                        "case storage parent `{}` is not a directory",
                        current.display()
                    ),
                    "replace it with a real directory inside the steward repository",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(TerminalFailure::refusal(
                    format!(
                        "case storage path `{}` cannot be inspected: {error}",
                        current.display()
                    ),
                    "make the steward-local case storage path inspectable before retrying",
                ));
            }
        }
    }
    Ok(())
}

fn existing_opening(
    absolute_event_path: &Path,
    event_path: PathBuf,
    steward: &marker::Marker,
    proposal: &OpenProposal,
    preview: bool,
) -> Result<OpenOutcome, TerminalFailure> {
    let event = fs::read_to_string(absolute_event_path).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "existing opening event `{}` cannot be read: {error}",
                absolute_event_path.display()
            ),
            "restore the recorded event before retrying this case identity",
        )
    })?;
    let recorded = toml::from_str::<CaseOpenedEvent>(&event).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "existing opening event `{}` is invalid: {error}",
                absolute_event_path.display()
            ),
            "restore the supported recorded event or choose a new opaque case identity",
        )
    })?;
    let matches = recorded.schema_version == CASE_SCHEMA_VERSION
        && recorded.sequence == OPENING_SEQUENCE
        && recorded.event_type == EventType::CaseOpened
        && recorded.case_id == proposal.case_id
        && recorded.responsibility == proposal.responsibility
        && recorded.steward_repository_id == steward.repository_id()
        && recorded.occurrences == proposal.occurrences
        && proposal
            .prepared
            .as_ref()
            .is_none_or(|prepared| prepared.bytes == event);
    if !matches {
        return Err(TerminalFailure::refusal(
            format!(
                "case identity `{}` is already recorded with different proposed content",
                proposal.case_id
            ),
            "restore the exact original proposal or choose a new UUID version 4 case identity",
        ));
    }
    Ok(OpenOutcome {
        effect: if preview {
            OpenEffect::Preview
        } else {
            OpenEffect::Existing
        },
        case_id: recorded.case_id,
        event_path,
        privacy: recorded.privacy,
        event,
    })
}

fn find_repository_root(working_directory: &Path) -> Result<PathBuf, TerminalFailure> {
    let (working_directory, repository_root) = crate::locate_repository_root(working_directory)
        .map_err(|error| {
            TerminalFailure::refusal(
                format!(
                    "working directory `{}` cannot be inspected: {error}",
                    working_directory.display()
                ),
                "rerun from an existing directory inside the steward repository",
            )
        })?;
    repository_root.ok_or_else(|| {
        TerminalFailure::refusal(
            format!(
                "`{}` is not inside a repository root",
                working_directory.display()
            ),
            "rerun inside an enrolled repository containing `.git`",
        )
    })
}

fn read_steward(repository_root: &Path) -> Result<marker::Marker, TerminalFailure> {
    match marker::read(repository_root) {
        Some(MarkerRead::Supported(marker)) => Ok(marker),
        None => Err(TerminalFailure::refusal(
            format!(
                "repository is not enrolled because `{}` does not exist",
                repository_root.join(crate::MARKER_FILE).display()
            ),
            "run `enroll` before opening a case",
        )),
        Some(_) => Err(TerminalFailure::refusal(
            "the steward repository is not validly enrolled",
            "restore a supported `reuse-evidence.toml` marker before opening a case",
        )),
    }
}

fn read_proposal(path: &Path) -> Result<OpenProposal, TerminalFailure> {
    let text = fs::read_to_string(path).map_err(|error| {
        TerminalFailure::refusal(
            format!("case proposal `{}` cannot be read: {error}", path.display()),
            "supply a readable UTF-8 TOML proposal with `--proposal <PATH>`",
        )
    })?;
    let document = toml::from_str::<OpenProposalDocument>(&text).map_err(|error| {
        TerminalFailure::refusal(
            format!("case proposal `{}` is invalid: {error}", path.display()),
            "provide a complete TOML case-opening proposal",
        )
    })?;
    let (case_id, responsibility, occurrences, prepared) = match document {
        OpenProposalDocument::Human(document) => {
            let case_id = parse_case_id(&document.case_id)?;
            (case_id, document.responsibility, document.occurrences, None)
        }
        OpenProposalDocument::Prepared(event) => {
            validate_prepared_event(&event)?;
            let prepared = PreparedOpening {
                steward_repository_id: event.steward_repository_id,
                privacy: event.privacy,
                bytes: text,
            };
            (
                event.case_id,
                event.responsibility,
                event.occurrences,
                Some(prepared),
            )
        }
    };
    let proposal = OpenProposal {
        case_id,
        responsibility,
        occurrences,
        prepared,
    };
    validate_proposal(&proposal)?;
    Ok(proposal)
}

fn parse_case_id(value: &str) -> Result<Uuid, TerminalFailure> {
    let case_id = Uuid::parse_str(value).map_err(|error| {
        TerminalFailure::refusal(
            format!("case identity `{value}` is not a well-formed opaque UUID: {error}"),
            "use a newly generated UUID version 4 as `case_id`",
        )
    })?;
    if case_id.get_version_num() != 4 {
        return Err(TerminalFailure::refusal(
            format!("case identity `{case_id}` is not an opaque UUID version 4"),
            "use a newly generated UUID version 4 as `case_id`",
        ));
    }
    Ok(case_id)
}

fn validate_prepared_event(event: &CaseOpenedEvent) -> Result<(), TerminalFailure> {
    if event.schema_version != CASE_SCHEMA_VERSION
        || event.sequence != OPENING_SEQUENCE
        || event.event_type != EventType::CaseOpened
    {
        return Err(TerminalFailure::refusal(
            "prepared opening event is not a supported revision 1 `case_opened` event",
            "use the exact event rendered by `case open --preview`",
        ));
    }
    if event.event_id.get_version_num() != 4 {
        return Err(TerminalFailure::refusal(
            format!(
                "prepared opening event identity `{}` is not an opaque UUID version 4",
                event.event_id
            ),
            "use the exact event rendered by `case open --preview`",
        ));
    }
    validate_recorded_at(&event.recorded_at)?;
    if event.case_id.get_version_num() != 4 {
        return Err(TerminalFailure::refusal(
            format!(
                "case identity `{}` is not an opaque UUID version 4",
                event.case_id
            ),
            "use a newly generated UUID version 4 as `case_id`",
        ));
    }
    Ok(())
}

fn validate_proposal(proposal: &OpenProposal) -> Result<(), TerminalFailure> {
    require_nonempty("responsibility", &proposal.responsibility)?;
    if proposal.occurrences.len() < 2 {
        return Err(TerminalFailure::refusal(
            format!(
                "case opening requires at least two occurrences, but the proposal contains {}",
                proposal.occurrences.len()
            ),
            "add a second independently evidenced occurrence before opening the case",
        ));
    }
    let mut observed_consumers = BTreeSet::new();
    for (index, occurrence) in proposal.occurrences.iter().enumerate() {
        if occurrence.consumer.trim().is_empty() {
            return Err(TerminalFailure::refusal(
                format!("occurrence {} consumer is empty", index + 1),
                "provide a non-empty consumer label",
            ));
        }
        if occurrence.independence.trim().is_empty() {
            return Err(TerminalFailure::refusal(
                format!(
                    "occurrence {} independence justification is empty",
                    index + 1
                ),
                "explain why this occurrence arose from an independent consumer need",
            ));
        }
        if occurrence.evidence.is_empty() {
            return Err(TerminalFailure::refusal(
                format!("occurrence {} carries no evidence reference", index + 1),
                "add at least one recoverable `occurrences.evidence` reference",
            ));
        }
        for (evidence_index, evidence) in occurrence.evidence.iter().enumerate() {
            if evidence.reference.trim().is_empty() {
                return Err(TerminalFailure::refusal(
                    format!(
                        "occurrence {} evidence reference {} is empty",
                        index + 1,
                        evidence_index + 1
                    ),
                    "provide a recoverable commit reference",
                ));
            }
            if let Some(path) = &evidence.path {
                validate_relative_evidence_path(path)?;
            }
        }
        if !observed_consumers.insert((occurrence.repository_id, occurrence.consumer.trim())) {
            return Err(TerminalFailure::refusal(
                format!(
                    "multiple occurrences use participant `{}` and consumer `{}`",
                    occurrence.repository_id,
                    occurrence.consumer.trim()
                ),
                "keep one occurrence for each distinct participant repository and reuse consumer",
            ));
        }
    }
    Ok(())
}

fn validate_recorded_at(value: &str) -> Result<(), TerminalFailure> {
    let bytes = value.as_bytes();
    let shaped = bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        });
    if !shaped {
        return Err(TerminalFailure::refusal(
            format!("prepared opening event timestamp `{value}` is not UTC RFC 3339"),
            "use the exact event rendered by `case open --preview`",
        ));
    }
    let component = |range: std::ops::Range<usize>| {
        value[range]
            .parse::<u32>()
            .expect("validated ASCII digits should parse")
    };
    let year = component(0..4);
    let month = component(5..7);
    let day = component(8..10);
    let hour = component(11..13);
    let minute = component(14..16);
    let second = component(17..19);
    let leap_year =
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => 0,
    };
    if day == 0 || day > days_in_month || hour > 23 || minute > 59 || second > 59 {
        return Err(TerminalFailure::refusal(
            format!("prepared opening event timestamp `{value}` is not a valid UTC instant"),
            "use the exact event rendered by `case open --preview`",
        ));
    }
    Ok(())
}

fn require_nonempty(field: &str, value: &str) -> Result<(), TerminalFailure> {
    if value.trim().is_empty() {
        return Err(TerminalFailure::refusal(
            format!("{field} is empty"),
            format!("provide a non-empty `{field}` value"),
        ));
    }
    Ok(())
}

fn validate_relative_evidence_path(path: &str) -> Result<(), TerminalFailure> {
    let path_value = Path::new(path);
    let invalid = path.is_empty()
        || path_value.is_absolute()
        || path_value.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        });
    if invalid {
        return Err(TerminalFailure::refusal(
            format!("evidence path `{path}` is not repository-relative"),
            "use a non-empty path relative to the participant repository without `..`",
        ));
    }
    Ok(())
}

fn resolve_participants(
    root_overrides: &[PathBuf],
    proposal: &OpenProposal,
) -> Result<BTreeMap<Uuid, Visibility>, TerminalFailure> {
    let roots = portfolio::selected_roots(root_overrides)?;
    let scan = portfolio::scan(&roots)?;
    let mut participants = BTreeMap::new();
    let requested = proposal
        .occurrences
        .iter()
        .map(|occurrence| occurrence.repository_id)
        .collect::<BTreeSet<_>>();
    for repository_id in requested {
        let mut matches = scan
            .enrollments
            .iter()
            .filter(|enrollment| enrollment.repository_id == repository_id)
            .collect::<Vec<_>>();
        if matches.is_empty() {
            return Err(TerminalFailure::refusal(
                format!(
                    "participant `{repository_id}` does not resolve to a discoverable enrolled repository"
                ),
                "enroll the participant beneath a selected portfolio root or correct its repository identity",
            ));
        }
        if matches.len() > 1 {
            matches.sort_by(|left, right| left.path.cmp(&right.path));
            let paths = matches
                .iter()
                .map(|enrollment| enrollment.path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(TerminalFailure::refusal(
                format!("participant identity `{repository_id}` is duplicated at: {paths}"),
                "restore a unique stable repository identity before opening the case",
            ));
        }
        let enrollment = matches[0];
        participants.insert(enrollment.repository_id, enrollment.visibility);
    }
    Ok(participants)
}

fn recording_timestamp() -> Result<String, TerminalFailure> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            TerminalFailure::unsafe_failure(format!(
                "system clock cannot supply the case recording timestamp: {error}"
            ))
        })?
        .as_secs();
    let seconds = i64::try_from(seconds).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "case recording timestamp is outside the supported range: {error}"
        ))
    })?;
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_date_from_unix_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

fn civil_date_from_unix_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_piece = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_piece + 2) / 5 + 1;
    let month = month_piece + if month_piece < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month, day)
}
