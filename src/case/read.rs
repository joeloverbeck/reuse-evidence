use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use uuid::Uuid;

use super::{
    CASE_SCHEMA_VERSION, CaseOpenedEvent, EventType, OPENING_SEQUENCE, Occurrence,
    OccurrenceAppendedEvent, REVIEW_ONLY_NOTICE, find_repository_root, read_steward,
    validate_case_storage_path,
};
use crate::{TerminalFailure, Visibility, portfolio};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Readiness {
    Watching,
    ReviewReady,
}

impl Readiness {
    pub(super) const fn from_occurrence_count(occurrence_count: usize) -> Self {
        if occurrence_count >= 3 {
            Self::ReviewReady
        } else {
            Self::Watching
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Watching => "watching",
            Self::ReviewReady => "review-ready",
        }
    }
}

pub(super) struct CaseRecord {
    pub(super) case_id: Uuid,
    pub(super) responsibility: String,
    pub(super) revision: i64,
    pub(super) privacy: Visibility,
    pub(super) occurrences: Vec<Occurrence>,
    conditions: Conditions,
}

enum CaseEvent {
    Opened(CaseOpenedEvent),
    OccurrenceAppended(OccurrenceAppendedEvent),
}

#[derive(Deserialize)]
struct EventDiscriminator {
    event_type: EventType,
}

#[derive(Clone, Copy)]
struct Conditions {
    privacy_conflicted: Option<bool>,
    stale: Option<bool>,
}

impl Conditions {
    const UNKNOWN: Self = Self {
        privacy_conflicted: None,
        stale: None,
    };
}

fn render_condition(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

impl CaseRecord {
    fn readiness(&self) -> Readiness {
        Readiness::from_occurrence_count(self.occurrences.len())
    }
}

/// The complete rendered result of listing steward-local cases.
pub struct ListOutcome {
    cases: Vec<CaseRecord>,
    portfolio_available: bool,
}

/// The complete rendered result of showing one steward-local case.
pub struct ShowOutcome {
    case: CaseRecord,
    portfolio_available: bool,
}

impl ShowOutcome {
    /// Renders the case and every recorded occurrence deterministically.
    #[must_use]
    pub fn render(&self) -> String {
        let case = &self.case;
        let mut output = format!(
            "case\ncase_id: {}\nresponsibility: {}\nrevision: {}\noccurrence_count: {}\nstate: {}\n",
            case.case_id,
            case.responsibility,
            case.revision,
            case.occurrences.len(),
            case.readiness().label()
        );
        if case.readiness() == Readiness::ReviewReady {
            writeln!(&mut output, "readiness: {REVIEW_ONLY_NOTICE}")
                .expect("writing to a String cannot fail");
        }
        writeln!(
            &mut output,
            "privacy_conflicted: {}\nstale: {}\noccurrences:",
            render_condition(case.conditions.privacy_conflicted),
            render_condition(case.conditions.stale)
        )
        .expect("writing to a String cannot fail");
        for occurrence in &case.occurrences {
            writeln!(
                &mut output,
                "- repository_id: {}\n  consumer: {}\n  independence: {}\n  evidence:",
                occurrence.repository_id, occurrence.consumer, occurrence.independence
            )
            .expect("writing to a String cannot fail");
            for evidence in &occurrence.evidence {
                let kind = match evidence.kind {
                    super::EvidenceKind::Commit => "commit",
                };
                writeln!(
                    &mut output,
                    "  - kind: {kind}\n    reference: {}",
                    evidence.reference
                )
                .expect("writing to a String cannot fail");
                if let Some(path) = &evidence.path {
                    writeln!(&mut output, "    path: {path}")
                        .expect("writing to a String cannot fail");
                }
            }
        }
        if !self.portfolio_available {
            output.push_str(
                "portfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n",
            );
        }
        output
    }
}

impl ListOutcome {
    /// Renders a deterministic steward-local case listing.
    #[must_use]
    pub fn render(&self) -> String {
        let mut output = String::from("cases\n");
        for case in &self.cases {
            writeln!(
                &mut output,
                "- case_id: {}\n  revision: {}\n  occurrence_count: {}\n  state: {}",
                case.case_id,
                case.revision,
                case.occurrences.len(),
                case.readiness().label()
            )
            .expect("writing to a String cannot fail");
            if case.readiness() == Readiness::ReviewReady {
                writeln!(&mut output, "  readiness: {REVIEW_ONLY_NOTICE}")
                    .expect("writing to a String cannot fail");
            }
            writeln!(
                &mut output,
                "  privacy_conflicted: {}\n  stale: {}",
                render_condition(case.conditions.privacy_conflicted),
                render_condition(case.conditions.stale)
            )
            .expect("writing to a String cannot fail");
        }
        if !self.portfolio_available {
            output.push_str(
                "portfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n",
            );
        }
        output
    }
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
    root_overrides: &[PathBuf],
) -> Result<ListOutcome, TerminalFailure> {
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root)?;
    let mut cases = read_cases(&repository_root, steward.repository_id())?;
    let portfolio_available = derive_conditions(&mut cases, steward.visibility(), root_overrides)?;
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
    root_overrides: &[PathBuf],
) -> Result<ShowOutcome, TerminalFailure> {
    let case_id = Uuid::parse_str(case_id).map_err(|error| {
        TerminalFailure::refusal(
            format!("case identity `{case_id}` is invalid: {error}"),
            "supply the opaque UUID recorded for the stewarded case",
        )
    })?;
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root)?;
    let relative_case_directory = Path::new("reuse-evidence/cases").join(case_id.to_string());
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
        root_overrides,
    )?;
    Ok(ShowOutcome {
        case,
        portfolio_available,
    })
}

fn derive_conditions(
    cases: &mut [CaseRecord],
    steward_visibility: Visibility,
    root_overrides: &[PathBuf],
) -> Result<bool, TerminalFailure> {
    let Some(roots) = portfolio::selected_roots_if_configured(root_overrides)? else {
        return Ok(false);
    };
    let scan = portfolio::scan(&roots)?;
    for case in cases {
        let mut privacy_conflicted = false;
        let mut stale = false;
        for occurrence in &case.occurrences {
            let matches = scan
                .enrollments
                .iter()
                .filter(|enrollment| enrollment.repository_id == occurrence.repository_id)
                .collect::<Vec<_>>();
            stale |= matches.len() != 1;
            privacy_conflicted |= steward_visibility == Visibility::Public
                && matches
                    .iter()
                    .any(|enrollment| enrollment.visibility == Visibility::Private);
        }
        case.conditions = Conditions {
            privacy_conflicted: Some(privacy_conflicted),
            stale: Some(stale),
        };
    }
    Ok(true)
}

fn read_cases(
    repository_root: &Path,
    steward_repository_id: Uuid,
) -> Result<Vec<CaseRecord>, TerminalFailure> {
    let relative_cases_root = Path::new("reuse-evidence/cases");
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
        cases.push(read_case(
            repository_root,
            &relative_case_directory,
            case_id,
            steward_repository_id,
        )?);
    }
    cases.sort_by_key(|case| case.case_id);
    Ok(cases)
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
            if is_append_temporary(&entry.file_name()) {
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
    if event_paths.is_empty() {
        return Err(TerminalFailure::refusal(
            format!("case `{case_id}` contains no event files"),
            "restore its opening event before reading the case",
        ));
    }
    validate_event_sequences(&event_paths, case_id)?;

    let mut opening = None;
    let mut revision = 0;
    let mut occurrences = Vec::new();
    for event_path in event_paths {
        let (file_sequence, event) =
            read_case_event(repository_root, &event_path, case_id, steward_repository_id)?;
        revision = revision.max(file_sequence);
        match event {
            CaseEvent::Opened(event) => {
                occurrences.extend(event.occurrences.iter().cloned());
                opening = Some(event);
            }
            CaseEvent::OccurrenceAppended(event) => occurrences.push(event.occurrence),
        }
    }

    let opening = opening.expect("a validated case event set has one opening event");
    validate_unique_occurrences(case_id, &occurrences)?;

    Ok(CaseRecord {
        case_id,
        responsibility: opening.responsibility,
        revision,
        privacy: opening.privacy,
        occurrences,
        conditions: Conditions::UNKNOWN,
    })
}

fn validate_unique_occurrences(
    case_id: Uuid,
    occurrences: &[Occurrence],
) -> Result<(), TerminalFailure> {
    let mut observed = BTreeSet::new();
    for occurrence in occurrences {
        let consumer = occurrence.consumer.trim();
        if !observed.insert((occurrence.repository_id, consumer.to_owned())) {
            return Err(TerminalFailure::refusal(
                format!(
                    "case `{case_id}` records participant `{}` and consumer `{consumer}` more than once",
                    occurrence.repository_id
                ),
                "restore the authoritative event stream so each participant repository and consumer pair occurs once before reading the case",
            ));
        }
    }
    Ok(())
}

pub(super) fn read_case_for_append(
    repository_root: &Path,
    relative_case_directory: &Path,
    case_id: Uuid,
    steward_repository_id: Uuid,
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
            "run `case list` in this steward repository and retry with a recorded case identity",
        ));
    }
    read_case(
        repository_root,
        relative_case_directory,
        case_id,
        steward_repository_id,
    )
}

fn read_case_event(
    repository_root: &Path,
    event_path: &Path,
    case_id: Uuid,
    steward_repository_id: Uuid,
) -> Result<(i64, CaseEvent), TerminalFailure> {
    let relative_event_path = event_path.strip_prefix(repository_root).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "case event path `{}` is not steward-local: {error}",
            event_path.display()
        ))
    })?;
    validate_case_storage_path(repository_root, relative_event_path)?;
    let event_text = fs::read_to_string(event_path).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "case event `{}` cannot be read: {error}",
                event_path.display()
            ),
            "restore the recorded UTF-8 event before reading the case",
        )
    })?;
    let discriminator = toml::from_str::<EventDiscriminator>(&event_text).map_err(|error| {
        TerminalFailure::refusal(
            format!("case event `{}` is invalid: {error}", event_path.display()),
            "restore a supported recorded event before reading the case",
        )
    })?;
    let file_sequence = event_sequence_from_file_name(
        event_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("validated event paths have UTF-8 sequence filenames"),
    )
    .expect("validated event paths have recognized sequence filenames");
    let file_name = event_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("validated event paths have UTF-8 sequence filenames");
    let (_, file_event_type) = event_file_identity(file_name)
        .expect("validated event paths have recognized sequence filenames");
    match discriminator.event_type {
        EventType::CaseOpened => {
            let event = toml::from_str::<CaseOpenedEvent>(&event_text)
                .map_err(|error| invalid_event(event_path, &error))?;
            validate_body_sequence(event_path, event.sequence, file_sequence)?;
            if event.sequence != OPENING_SEQUENCE {
                return Err(TerminalFailure::refusal(
                    format!(
                        "case event `{}` records `case_opened` at sequence {}",
                        event_path.display(),
                        event.sequence
                    ),
                    "restore `case_opened` as the single sequence 1 opening event before reading the case",
                ));
            }
            if event.schema_version != CASE_SCHEMA_VERSION
                || event.case_id != case_id
                || event.steward_repository_id != steward_repository_id
            {
                return Err(TerminalFailure::refusal(
                    format!(
                        "case event `{}` does not match its steward-local case",
                        event_path.display()
                    ),
                    "restore the event under the case and steward identities it records",
                ));
            }
            validate_file_event_type(
                case_id,
                file_name,
                file_sequence,
                file_event_type,
                "case_opened",
                "case-opened",
            )?;
            super::validate_recorded_opening(&event)?;
            Ok((file_sequence, CaseEvent::Opened(event)))
        }
        EventType::OccurrenceAppended => {
            let event = toml::from_str::<OccurrenceAppendedEvent>(&event_text)
                .map_err(|error| invalid_event(event_path, &error))?;
            validate_body_sequence(event_path, event.sequence, file_sequence)?;
            if event.sequence == OPENING_SEQUENCE {
                return Err(TerminalFailure::refusal(
                    format!(
                        "case event `{}` records `occurrence_appended` at opening sequence 1",
                        event_path.display()
                    ),
                    "restore `case_opened` as sequence 1 and append occurrences only after it",
                ));
            }
            validate_file_event_type(
                case_id,
                file_name,
                file_sequence,
                file_event_type,
                "occurrence_appended",
                "occurrence-appended",
            )?;
            super::validate_recorded_append(&event)?;
            Ok((file_sequence, CaseEvent::OccurrenceAppended(event)))
        }
    }
}

fn invalid_event(event_path: &Path, error: &toml::de::Error) -> TerminalFailure {
    TerminalFailure::refusal(
        format!("case event `{}` is invalid: {error}", event_path.display()),
        "restore a supported recorded event before reading the case",
    )
}

fn validate_body_sequence(
    event_path: &Path,
    body_sequence: i64,
    file_sequence: i64,
) -> Result<(), TerminalFailure> {
    if body_sequence != file_sequence {
        return Err(TerminalFailure::refusal(
            format!(
                "case event `{}` records sequence {body_sequence} but its filename records sequence {file_sequence}",
                event_path.display()
            ),
            "restore the event under the filename matching its recorded sequence before reading the case",
        ));
    }
    Ok(())
}

fn validate_file_event_type(
    case_id: Uuid,
    file_name: &str,
    file_sequence: i64,
    file_event_type: &str,
    recorded_event_type: &str,
    file_event_slug: &str,
) -> Result<(), TerminalFailure> {
    if file_event_type != file_event_slug {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{case_id}` event file `{file_name}` does not match its recorded type `{recorded_event_type}`"
            ),
            format!(
                "restore the event as `{file_sequence:04}-{file_event_slug}.toml` before reading the case"
            ),
        ));
    }
    Ok(())
}

fn validate_event_sequences(event_paths: &[PathBuf], case_id: Uuid) -> Result<(), TerminalFailure> {
    let mut sequence_files = BTreeMap::<i64, Vec<String>>::new();
    for event_path in event_paths {
        let file_name = event_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                TerminalFailure::refusal(
                    format!("case `{case_id}` contains an event without a UTF-8 filename"),
                    "restore sequence-numbered UTF-8 TOML event filenames before reading the case",
                )
            })?;
        let sequence = event_sequence_from_file_name(file_name).ok_or_else(|| {
            TerminalFailure::refusal(
                format!("case `{case_id}` contains unrecognized event file `{file_name}`"),
                "restore event filenames in `NNNN-<event-type>.toml` form before reading the case",
            )
        })?;
        sequence_files
            .entry(sequence)
            .or_default()
            .push(file_name.to_owned());
    }
    if let Some((sequence, files)) = sequence_files.iter().find(|(_, files)| files.len() > 1) {
        let files = files
            .iter()
            .map(|file| format!("`{file}`"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(TerminalFailure::refusal(
            format!("case `{case_id}` has duplicated sequence number {sequence} in files {files}"),
            format!(
                "restore exactly one event file for sequence {sequence} before reading the case"
            ),
        ));
    }
    let highest_sequence = *sequence_files
        .last_key_value()
        .expect("a non-empty event path set has a sequence")
        .0;
    for expected in 1..=highest_sequence {
        if !sequence_files.contains_key(&expected) {
            let next_recorded = *sequence_files
                .range((expected + 1)..)
                .next()
                .expect("a missing sequence below the maximum has a successor")
                .0;
            return Err(TerminalFailure::refusal(
                format!(
                    "case `{case_id}` is missing sequence number {expected} before recorded sequence {next_recorded}"
                ),
                format!(
                    "restore event file sequence {expected} so the case stream is contiguous before reading it"
                ),
            ));
        }
    }
    Ok(())
}

fn event_sequence_from_file_name(file_name: &str) -> Option<i64> {
    event_file_identity(file_name).map(|(sequence, _)| sequence)
}

fn is_append_temporary(file_name: &std::ffi::OsStr) -> bool {
    let Some(file_name) = file_name.to_str() else {
        return false;
    };
    let Some(staged_name) = file_name
        .strip_prefix('.')
        .and_then(|name| name.strip_suffix(".tmp"))
    else {
        return false;
    };
    let Some((event_file_name, identity)) = staged_name.rsplit_once('.') else {
        return false;
    };
    Uuid::parse_str(identity).is_ok()
        && event_file_identity(event_file_name).is_some_and(|(sequence, event_type)| {
            sequence > OPENING_SEQUENCE && event_type == "occurrence-appended"
        })
}

fn event_file_identity(file_name: &str) -> Option<(i64, &str)> {
    let (sequence, event_type) = file_name.strip_suffix(".toml")?.split_once('-')?;
    if sequence.len() != 4
        || !sequence.bytes().all(|byte| byte.is_ascii_digit())
        || event_type.is_empty()
    {
        return None;
    }
    sequence
        .parse()
        .ok()
        .filter(|sequence| *sequence > 0)
        .map(|sequence| (sequence, event_type))
}
