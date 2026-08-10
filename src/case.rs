//! Durable case recording and inspection mechanics.

mod publication;
mod read;

pub use read::{ListOutcome, ShowOutcome, list, show};

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
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
const REVIEW_ONLY_NOTICE: &str = "authorizes semantic review; does not authorize extraction";
const PORTFOLIO_UNAVAILABLE_FOOTER: &str = "portfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n";

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OpenProposalDocument {
    Prepared(CaseOpenedEvent),
    Human(HumanOpenProposalDocument),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AppendProposalDocument {
    Prepared(OccurrenceAppendedEvent),
    Human(HumanAppendProposalDocument),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EarlyReviewProposalDocument {
    Prepared(EarlyReviewAuthorizedEvent),
    Human(HumanEarlyReviewProposalDocument),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HumanOpenProposalDocument {
    case_id: String,
    responsibility: String,
    occurrences: Vec<Occurrence>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HumanAppendProposalDocument {
    occurrence: Occurrence,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HumanEarlyReviewProposalDocument {
    reason: Option<String>,
    review_appetite: Option<String>,
    evidence: Option<Vec<EvidenceReference>>,
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

#[derive(Debug)]
struct AppendProposal {
    occurrence: Occurrence,
    prepared: Option<PreparedAppend>,
}

#[derive(Debug)]
struct PreparedAppend {
    sequence: i64,
    event_id: Uuid,
    bytes: String,
}

#[derive(Debug)]
struct EarlyReviewProposal {
    reason: String,
    review_appetite: String,
    evidence: Vec<EvidenceReference>,
    prepared: Option<PreparedEarlyReview>,
}

#[derive(Debug)]
struct PreparedEarlyReview {
    sequence: i64,
    event_id: Uuid,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OccurrenceAppendedEvent {
    schema_version: i64,
    sequence: i64,
    event_id: Uuid,
    event_type: EventType,
    recorded_at: String,
    occurrence: Occurrence,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EarlyReviewAuthorizedEvent {
    schema_version: i64,
    sequence: i64,
    event_id: Uuid,
    event_type: EventType,
    recorded_at: String,
    reason: String,
    review_appetite: String,
    evidence: Vec<EvidenceReference>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EventType {
    CaseOpened,
    OccurrenceAppended,
    EarlyReviewAuthorized,
}

impl publication::RevisionedCase for read::CaseRecord {
    fn revision(&self) -> i64 {
        self.revision
    }
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

/// The complete observable result of appending or previewing an occurrence.
#[derive(Debug)]
pub struct AppendOutcome {
    effect: AppendEffect,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    readiness: read::Readiness,
    privacy: Option<Visibility>,
    event: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppendEffect {
    Preview,
    Created,
    Existing,
}

/// The complete observable result of authorizing or previewing early review.
#[derive(Debug)]
pub struct EarlyReviewOutcome {
    effect: EarlyReviewEffect,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    privacy: Option<Visibility>,
    event: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EarlyReviewEffect {
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

impl AppendOutcome {
    /// Renders the receipt followed by exact event bytes for a preview.
    #[must_use]
    pub fn render(&self) -> String {
        let heading = match self.effect {
            AppendEffect::Preview => "case append preview",
            AppendEffect::Created => "appended occurrence",
            AppendEffect::Existing => "occurrence already recorded",
        };
        let mut receipt = format!(
            "{heading}\ncase_id: {}\nfile: {}\nrevision: {}\nstate: {}\n",
            self.case_id,
            self.event_path.display(),
            self.revision,
            self.readiness.label()
        );
        if let Some(basis) = self.readiness.basis() {
            writeln!(&mut receipt, "readiness_basis: {basis}")
                .expect("writing to a String cannot fail");
        }
        if self.readiness.authorizes_review() {
            receipt.push_str("readiness: ");
            receipt.push_str(REVIEW_ONLY_NOTICE);
            receipt.push('\n');
        }
        if let Some(privacy) = self.privacy {
            writeln!(&mut receipt, "privacy: {privacy}").expect("writing to a String cannot fail");
        } else {
            receipt.push_str("privacy: unknown\n");
            receipt.push_str(PORTFOLIO_UNAVAILABLE_FOOTER);
        }
        if self.effect == AppendEffect::Preview {
            receipt.push_str("event:\n");
            receipt.push_str(&self.event);
        }
        receipt
    }
}

impl EarlyReviewOutcome {
    /// Renders the receipt followed by exact event bytes for a preview.
    #[must_use]
    pub fn render(&self) -> String {
        let heading = match self.effect {
            EarlyReviewEffect::Preview => "early-review override preview",
            EarlyReviewEffect::Created => "authorized early review",
            EarlyReviewEffect::Existing => "early review already authorized",
        };
        let mut receipt = format!(
            "{heading}\ncase_id: {}\nfile: {}\nrevision: {}\nstate: review-ready\nreadiness_basis: early-review-override\nreadiness: {REVIEW_ONLY_NOTICE}\n",
            self.case_id,
            self.event_path.display(),
            self.revision
        );
        if let Some(privacy) = self.privacy {
            writeln!(&mut receipt, "privacy: {privacy}").expect("writing to a String cannot fail");
        } else {
            receipt.push_str("privacy: unknown\n");
            receipt.push_str(PORTFOLIO_UNAVAILABLE_FOOTER);
        }
        if self.effect == EarlyReviewEffect::Preview {
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
    let participants = resolve_participants(root_overrides, &proposal.occurrences)?;
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

/// Appends or previews one later occurrence against an expected case revision.
///
/// # Errors
///
/// Returns a classified failure when the steward, case, proposal, revision, or
/// participant repository cannot be read or validated safely.
pub fn append(
    working_directory: &Path,
    case_id: &str,
    expected_revision: i64,
    proposal_path: &Path,
    root_overrides: &[PathBuf],
    preview: bool,
) -> Result<AppendOutcome, TerminalFailure> {
    let case_id = parse_case_id(case_id)?;
    let publication = publication::Publication::new(expected_revision)?;
    let sequence = publication.sequence();
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root)?;
    let relative_case_directory = Path::new("reuse-evidence/cases").join(case_id.to_string());
    validate_case_storage_path(&repository_root, &relative_case_directory)?;
    let case = read::read_case_for_append(
        &repository_root,
        &relative_case_directory,
        case_id,
        steward.repository_id(),
    )?;
    let proposal = read_append_proposal(proposal_path)?;
    validate_prepared_append_sequence(&proposal, expected_revision, sequence)?;
    let relative_event_path =
        relative_case_directory.join(format!("{sequence:04}-occurrence-appended.toml"));
    validate_case_storage_path(&repository_root, &relative_event_path)?;
    let absolute_event_path = repository_root.join(&relative_event_path);
    let event = append_event_bytes(&proposal, sequence)?;
    let prepared_event_id = proposal.prepared.as_ref().map(|prepared| prepared.event_id);
    if preview {
        if absolute_event_path.exists() {
            let existing =
                publication::existing_event(&case, &absolute_event_path, prepared_event_id, &event)
                    .map_err(|failure| append_existing_event_failure(case_id, failure))?;
            return append_retry_outcome(
                case_id,
                relative_event_path,
                &case,
                existing,
                &steward,
                root_overrides,
            );
        }
        if case.revision != expected_revision {
            return Err(TerminalFailure::refusal(
                format!(
                    "expected revision {expected_revision} does not match case `{case_id}` current revision {}",
                    case.revision
                ),
                format!(
                    "run `case show {case_id}` and retry with `--expected-revision {}`",
                    case.revision
                ),
            ));
        }
        let privacy = validate_new_append(&case, &proposal, &steward, root_overrides)?;
        return Ok(AppendOutcome {
            effect: AppendEffect::Preview,
            case_id,
            event_path: relative_event_path,
            revision: sequence,
            readiness: case.readiness_after_appending_occurrence(),
            privacy: Some(privacy),
            event,
        });
    }

    match publication
        .publish(
            publication::PublicationTarget {
                repository_root: &repository_root,
                relative_case_directory: &relative_case_directory,
                relative_event_path: &relative_event_path,
            },
            publication::PreparedEvent {
                event_id: prepared_event_id,
                bytes: &event,
            },
            || {
                read::read_case_for_append(
                    &repository_root,
                    &relative_case_directory,
                    case_id,
                    steward.repository_id(),
                )
            },
            |_| Ok(()),
            |case, ()| validate_new_append(case, &proposal, &steward, root_overrides),
        )
        .map_err(|failure| append_publication_failure(case_id, failure))?
    {
        publication::PublicationOutcome::Created { case, validation } => Ok(AppendOutcome {
            effect: AppendEffect::Created,
            case_id,
            event_path: relative_event_path,
            revision: sequence,
            readiness: case.readiness_after_appending_occurrence(),
            privacy: Some(validation),
            event,
        }),
        publication::PublicationOutcome::Existing { case, event } => append_retry_outcome(
            case_id,
            relative_event_path,
            &case,
            event,
            &steward,
            root_overrides,
        ),
    }
}

fn append_retry_outcome(
    case_id: Uuid,
    event_path: PathBuf,
    case: &read::CaseRecord,
    event: publication::ExistingEvent,
    steward: &marker::Marker,
    root_overrides: &[PathBuf],
) -> Result<AppendOutcome, TerminalFailure> {
    Ok(AppendOutcome {
        effect: AppendEffect::Existing,
        case_id,
        event_path,
        revision: event.sequence,
        readiness: case.readiness(),
        privacy: append_retry_privacy(case, steward, root_overrides)?,
        event: event.bytes,
    })
}

fn append_publication_failure(
    case_id: Uuid,
    failure: publication::PublicationFailure,
) -> TerminalFailure {
    match failure {
        publication::PublicationFailure::Protocol(failure) => failure,
        publication::PublicationFailure::ExistingEvent(failure) => {
            append_existing_event_failure(case_id, failure)
        }
        publication::PublicationFailure::RevisionConflict {
            expected_revision,
            current_revision,
        } => TerminalFailure::refusal(
            format!(
                "expected revision {expected_revision} does not match case `{case_id}` current revision {current_revision}"
            ),
            format!(
                "run `case show {case_id}` and retry with `--expected-revision {current_revision}`"
            ),
        ),
    }
}

fn append_existing_event_failure(
    case_id: Uuid,
    failure: publication::ExistingEventFailure,
) -> TerminalFailure {
    match failure {
        publication::ExistingEventFailure::Unreadable { path, error } => TerminalFailure::refusal(
            format!(
                "recorded append event `{}` cannot be read: {error}",
                path.display()
            ),
            "restore the recorded event before retrying the append",
        ),
        publication::ExistingEventFailure::Invalid { path, error } => TerminalFailure::refusal(
            format!(
                "recorded append event `{}` is invalid: {error}",
                path.display()
            ),
            "restore the supported recorded event before retrying the append",
        ),
        publication::ExistingEventFailure::IdentityConflict {
            recorded_sequence,
            recorded_event_id,
            prepared_event_id,
            current_revision,
        } => {
            let proposed = prepared_event_id.map_or_else(
                || "a newly prepared event".to_owned(),
                |event_id| format!("event `{event_id}`"),
            );
            TerminalFailure::refusal(
                format!(
                    "case `{case_id}` has a revision conflict at sequence {recorded_sequence}: event `{recorded_event_id}` is recorded instead of {proposed}"
                ),
                format!(
                    "inspect sequence {recorded_sequence}; retry its recorded identity if it is the intended append, or prepare a distinct occurrence against revision {current_revision}"
                ),
            )
        }
        publication::ExistingEventFailure::ContentDrift { recorded_event_id } => {
            TerminalFailure::refusal(
                format!(
                    "append event identity `{recorded_event_id}` is already recorded with different content"
                ),
                "restore the exact previewed append event before retrying",
            )
        }
    }
}

fn validate_prepared_append_sequence(
    proposal: &AppendProposal,
    expected_revision: i64,
    sequence: i64,
) -> Result<(), TerminalFailure> {
    if let Some(prepared) = &proposal.prepared
        && prepared.sequence != sequence
    {
        return Err(TerminalFailure::refusal(
            format!(
                "prepared append event records sequence {}, but expected revision {expected_revision} requires sequence {sequence}",
                prepared.sequence
            ),
            "preview the append again against the current expected revision",
        ));
    }
    Ok(())
}

/// Records or previews a human-authorized early-review override.
///
/// # Errors
///
/// Returns a classified failure when the steward, case, proposal, or revision
/// cannot be read or validated safely.
pub fn authorize_early_review(
    working_directory: &Path,
    case_id: &str,
    expected_revision: i64,
    proposal_path: &Path,
    root_overrides: &[PathBuf],
    preview: bool,
) -> Result<EarlyReviewOutcome, TerminalFailure> {
    let case_id = parse_case_id(case_id)?;
    let publication = publication::Publication::new(expected_revision)?;
    let sequence = publication.sequence();
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root)?;
    let relative_case_directory = Path::new("reuse-evidence/cases").join(case_id.to_string());
    validate_case_storage_path(&repository_root, &relative_case_directory)?;
    let case = read::read_case_for_early_review(
        &repository_root,
        &relative_case_directory,
        case_id,
        steward.repository_id(),
    )?;
    let proposal = read_early_review_proposal(proposal_path)?;
    validate_prepared_early_review_sequence(&proposal, expected_revision, sequence)?;
    let relative_event_path =
        relative_case_directory.join(format!("{sequence:04}-early-review-authorized.toml"));
    validate_case_storage_path(&repository_root, &relative_event_path)?;
    let event = early_review_event_bytes(&proposal, sequence)?;
    if preview {
        if let Some(outcome) = early_review_preview_retry(
            &repository_root.join(&relative_event_path),
            &relative_event_path,
            &case,
            &proposal,
            &event,
            &steward,
            root_overrides,
        )? {
            return Ok(outcome);
        }
        let privacy = derive_complete_case_privacy(&case, &steward, root_overrides)?;
        validate_early_review_privacy(&case, &steward, privacy)?;
        if case.revision != expected_revision {
            return Err(TerminalFailure::refusal(
                format!(
                    "expected revision {expected_revision} does not match case `{case_id}` current revision {}",
                    case.revision
                ),
                format!(
                    "run `case show {case_id}` and retry `case override {case_id}` with `--expected-revision {}` and the approved proposal",
                    case.revision
                ),
            ));
        }
        validate_new_early_review(&case)?;
        return Ok(EarlyReviewOutcome {
            effect: EarlyReviewEffect::Preview,
            case_id,
            event_path: relative_event_path,
            revision: sequence,
            privacy: Some(privacy),
            event,
        });
    }

    match publication
        .publish(
            publication::PublicationTarget {
                repository_root: &repository_root,
                relative_case_directory: &relative_case_directory,
                relative_event_path: &relative_event_path,
            },
            publication::PreparedEvent {
                event_id: proposal.prepared.as_ref().map(|prepared| prepared.event_id),
                bytes: &event,
            },
            || {
                read::read_case_for_early_review(
                    &repository_root,
                    &relative_case_directory,
                    case_id,
                    steward.repository_id(),
                )
            },
            |case| {
                let privacy = derive_complete_case_privacy(case, &steward, root_overrides)?;
                validate_early_review_privacy(case, &steward, privacy)?;
                Ok(privacy)
            },
            |case, privacy| {
                validate_new_early_review(case)?;
                Ok(privacy)
            },
        )
        .map_err(|failure| early_review_publication_failure(case_id, failure))?
    {
        publication::PublicationOutcome::Created { validation, .. } => Ok(
            early_review_created_outcome(case_id, relative_event_path, sequence, validation, event),
        ),
        publication::PublicationOutcome::Existing { case, event } => {
            Ok(early_review_retry_outcome(
                case_id,
                relative_event_path,
                &case,
                event,
                &steward,
                root_overrides,
            ))
        }
    }
}

fn early_review_created_outcome(
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    privacy: Visibility,
    event: String,
) -> EarlyReviewOutcome {
    EarlyReviewOutcome {
        effect: EarlyReviewEffect::Created,
        case_id,
        event_path,
        revision,
        privacy: Some(privacy),
        event,
    }
}

fn early_review_retry_outcome(
    case_id: Uuid,
    event_path: PathBuf,
    case: &read::CaseRecord,
    event: publication::ExistingEvent,
    steward: &marker::Marker,
    root_overrides: &[PathBuf],
) -> EarlyReviewOutcome {
    EarlyReviewOutcome {
        effect: EarlyReviewEffect::Existing,
        case_id,
        event_path,
        revision: event.sequence,
        privacy: early_review_retry_privacy(case, steward, root_overrides),
        event: event.bytes,
    }
}

fn early_review_publication_failure(
    case_id: Uuid,
    failure: publication::PublicationFailure,
) -> TerminalFailure {
    match failure {
        publication::PublicationFailure::Protocol(failure) => failure,
        publication::PublicationFailure::ExistingEvent(failure) => {
            early_review_existing_event_failure(case_id, failure)
        }
        publication::PublicationFailure::RevisionConflict {
            expected_revision,
            current_revision,
        } => TerminalFailure::refusal(
            format!(
                "expected revision {expected_revision} does not match case `{case_id}` current revision {current_revision}"
            ),
            format!(
                "run `case show {case_id}` and retry `case override {case_id}` with `--expected-revision {current_revision}` and the approved proposal"
            ),
        ),
    }
}

fn early_review_existing_event_failure(
    case_id: Uuid,
    failure: publication::ExistingEventFailure,
) -> TerminalFailure {
    match failure {
        publication::ExistingEventFailure::Unreadable { path, error } => TerminalFailure::refusal(
            format!(
                "recorded early-review event `{}` cannot be read: {error}",
                path.display()
            ),
            "restore the recorded event before retrying the early-review override",
        ),
        publication::ExistingEventFailure::Invalid { path, error } => TerminalFailure::refusal(
            format!(
                "recorded early-review event `{}` is invalid: {error}",
                path.display()
            ),
            "restore the supported recorded event before retrying the early-review override",
        ),
        publication::ExistingEventFailure::IdentityConflict {
            recorded_sequence,
            recorded_event_id,
            prepared_event_id,
            current_revision,
        } => {
            let proposed = prepared_event_id.map_or_else(
                || "a newly prepared event".to_owned(),
                |event_id| format!("event `{event_id}`"),
            );
            TerminalFailure::refusal(
                format!(
                    "case `{case_id}` has a revision conflict at sequence {recorded_sequence}: event `{recorded_event_id}` is recorded instead of {proposed}"
                ),
                format!(
                    "inspect sequence {recorded_sequence}; retry its recorded identity if it is the intended early-review override, or prepare a new operation against revision {current_revision}"
                ),
            )
        }
        publication::ExistingEventFailure::ContentDrift { recorded_event_id } => {
            TerminalFailure::refusal(
                format!(
                    "early-review event identity `{recorded_event_id}` is already recorded with different content"
                ),
                "restore the exact previewed early-review event before retrying",
            )
        }
    }
}

fn early_review_preview_retry(
    absolute_event_path: &Path,
    relative_event_path: &Path,
    case: &read::CaseRecord,
    proposal: &EarlyReviewProposal,
    event: &str,
    steward: &marker::Marker,
    root_overrides: &[PathBuf],
) -> Result<Option<EarlyReviewOutcome>, TerminalFailure> {
    if !absolute_event_path.exists() {
        return Ok(None);
    }
    let existing = publication::existing_event(
        case,
        absolute_event_path,
        proposal.prepared.as_ref().map(|prepared| prepared.event_id),
        event,
    )
    .map_err(|failure| early_review_existing_event_failure(case.case_id, failure))?;
    Ok(Some(early_review_retry_outcome(
        case.case_id,
        relative_event_path.to_path_buf(),
        case,
        existing,
        steward,
        root_overrides,
    )))
}

fn validate_prepared_early_review_sequence(
    proposal: &EarlyReviewProposal,
    expected_revision: i64,
    sequence: i64,
) -> Result<(), TerminalFailure> {
    if let Some(prepared) = &proposal.prepared
        && prepared.sequence != sequence
    {
        return Err(TerminalFailure::refusal(
            format!(
                "prepared early-review event records sequence {}, but expected revision {expected_revision} requires sequence {sequence}",
                prepared.sequence
            ),
            "preview the early-review override again against the current expected revision",
        ));
    }
    Ok(())
}

fn validate_early_review_privacy(
    case: &read::CaseRecord,
    steward: &marker::Marker,
    privacy: Visibility,
) -> Result<(), TerminalFailure> {
    if steward.visibility() == Visibility::Public && privacy == Visibility::Private {
        return Err(TerminalFailure::refusal(
            format!(
                "public steward `{}` cannot authorize early review for private case `{}`",
                steward.repository_id(),
                case.case_id
            ),
            "run `set-visibility --visibility private` in the steward repository, then preview the early-review override again",
        ));
    }
    Ok(())
}

fn validate_new_early_review(case: &read::CaseRecord) -> Result<(), TerminalFailure> {
    if case.has_early_review() {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{}` is already review-ready from its recorded early-review override",
                case.case_id
            ),
            "proceed to semantic review; do not record another early-review override",
        ));
    }
    if case.occurrences.len() >= 3 {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{}` is already review-ready from {} recorded occurrences",
                case.case_id,
                case.occurrences.len()
            ),
            "proceed to semantic review; an early-review override cannot change this case's readiness",
        ));
    }
    Ok(())
}

fn validate_new_append(
    case: &read::CaseRecord,
    proposal: &AppendProposal,
    steward: &marker::Marker,
    root_overrides: &[PathBuf],
) -> Result<Visibility, TerminalFailure> {
    if case.occurrences.iter().any(|recorded| {
        recorded.repository_id == proposal.occurrence.repository_id
            && recorded.consumer.trim() == proposal.occurrence.consumer.trim()
    }) {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{}` already records participant `{}` and consumer `{}`",
                case.case_id,
                proposal.occurrence.repository_id,
                proposal.occurrence.consumer.trim()
            ),
            "change either the participant repository or consumer so the pair is distinct, or keep the existing occurrence",
        ));
    }
    let mut occurrences = case.occurrences.clone();
    occurrences.push(proposal.occurrence.clone());
    let participant_visibilities = resolve_participants(root_overrides, &occurrences)?;
    if steward.visibility() == Visibility::Public && case.privacy == Visibility::Private {
        return Err(TerminalFailure::refusal(
            format!(
                "public steward `{}` cannot append to private case `{}`",
                steward.repository_id(),
                case.case_id
            ),
            "run `set-visibility --visibility private` in the steward repository, then preview the append again",
        ));
    }
    let private_participant = occurrences.iter().find(|occurrence| {
        participant_visibilities[&occurrence.repository_id] == Visibility::Private
    });
    if steward.visibility() == Visibility::Public
        && let Some(private_participant) = private_participant
    {
        return Err(TerminalFailure::refusal(
            format!(
                "public steward `{}` cannot append private participant `{}`",
                steward.repository_id(),
                private_participant.repository_id
            ),
            "run `set-visibility --visibility private` in the steward repository, then preview the append again",
        ));
    }
    if case.privacy == Visibility::Private
        || steward.visibility() == Visibility::Private
        || private_participant.is_some()
    {
        Ok(Visibility::Private)
    } else {
        Ok(Visibility::Public)
    }
}

fn derive_complete_case_privacy(
    case: &read::CaseRecord,
    steward: &marker::Marker,
    root_overrides: &[PathBuf],
) -> Result<Visibility, TerminalFailure> {
    let participant_visibilities = resolve_participants(root_overrides, &case.occurrences)?;
    if case.privacy == Visibility::Private
        || steward.visibility() == Visibility::Private
        || participant_visibilities
            .values()
            .any(|visibility| *visibility == Visibility::Private)
    {
        Ok(Visibility::Private)
    } else {
        Ok(Visibility::Public)
    }
}

fn append_retry_privacy(
    case: &read::CaseRecord,
    steward: &marker::Marker,
    root_overrides: &[PathBuf],
) -> Result<Option<Visibility>, TerminalFailure> {
    if portfolio::selected_roots_if_configured(root_overrides)?.is_none() {
        return Ok(None);
    }
    derive_complete_case_privacy(case, steward, root_overrides).map(Some)
}

fn early_review_retry_privacy(
    case: &read::CaseRecord,
    steward: &marker::Marker,
    root_overrides: &[PathBuf],
) -> Option<Visibility> {
    if matches!(
        portfolio::selected_roots_if_configured(root_overrides),
        Ok(None)
    ) {
        return None;
    }
    Some(derive_complete_case_privacy(case, steward, root_overrides).unwrap_or(Visibility::Private))
}

fn append_event_bytes(proposal: &AppendProposal, sequence: i64) -> Result<String, TerminalFailure> {
    if let Some(prepared) = &proposal.prepared {
        return Ok(prepared.bytes.clone());
    }
    let event = OccurrenceAppendedEvent {
        schema_version: CASE_SCHEMA_VERSION,
        sequence,
        event_id: Uuid::new_v4(),
        event_type: EventType::OccurrenceAppended,
        recorded_at: recording_timestamp()?,
        occurrence: proposal.occurrence.clone(),
    };
    toml::to_string(&event).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "occurrence append event could not be encoded: {error}"
        ))
    })
}

fn early_review_event_bytes(
    proposal: &EarlyReviewProposal,
    sequence: i64,
) -> Result<String, TerminalFailure> {
    if let Some(prepared) = &proposal.prepared {
        return Ok(prepared.bytes.clone());
    }
    let event = EarlyReviewAuthorizedEvent {
        schema_version: CASE_SCHEMA_VERSION,
        sequence,
        event_id: Uuid::new_v4(),
        event_type: EventType::EarlyReviewAuthorized,
        recorded_at: recording_timestamp()?,
        reason: proposal.reason.clone(),
        review_appetite: proposal.review_appetite.clone(),
        evidence: proposal.evidence.clone(),
    };
    toml::to_string(&event).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "early-review authorization event could not be encoded: {error}"
        ))
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
    let (case_id, responsibility, occurrences, prepared, content_validated) = match document {
        OpenProposalDocument::Human(document) => {
            let case_id = parse_case_id(&document.case_id)?;
            (
                case_id,
                document.responsibility,
                document.occurrences,
                None,
                false,
            )
        }
        OpenProposalDocument::Prepared(event) => {
            validate_recorded_opening(&event)?;
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
                true,
            )
        }
    };
    let proposal = OpenProposal {
        case_id,
        responsibility,
        occurrences,
        prepared,
    };
    if !content_validated {
        validate_proposal(&proposal)?;
    }
    Ok(proposal)
}

fn read_append_proposal(path: &Path) -> Result<AppendProposal, TerminalFailure> {
    let text = fs::read_to_string(path).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "append proposal `{}` cannot be read: {error}",
                path.display()
            ),
            "supply a readable UTF-8 TOML proposal with `--proposal <PATH>`",
        )
    })?;
    let document = toml::from_str::<AppendProposalDocument>(&text).map_err(|error| {
        TerminalFailure::refusal(
            format!("append proposal `{}` is invalid: {error}", path.display()),
            "provide a complete TOML occurrence-append proposal",
        )
    })?;
    let proposal = match document {
        AppendProposalDocument::Human(document) => AppendProposal {
            occurrence: document.occurrence,
            prepared: None,
        },
        AppendProposalDocument::Prepared(event) => {
            validate_recorded_append(&event)?;
            AppendProposal {
                occurrence: event.occurrence,
                prepared: Some(PreparedAppend {
                    sequence: event.sequence,
                    event_id: event.event_id,
                    bytes: text,
                }),
            }
        }
    };
    validate_occurrence(&proposal.occurrence, 1, "occurrence.evidence")?;
    Ok(proposal)
}

fn read_early_review_proposal(path: &Path) -> Result<EarlyReviewProposal, TerminalFailure> {
    let text = fs::read_to_string(path).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "early-review proposal `{}` cannot be read: {error}",
                path.display()
            ),
            "supply a readable UTF-8 TOML proposal with `--proposal <PATH>`",
        )
    })?;
    let document = toml::from_str::<EarlyReviewProposalDocument>(&text).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "early-review proposal `{}` is invalid: {error}",
                path.display()
            ),
            "provide a complete TOML early-review proposal",
        )
    })?;
    let proposal = match document {
        EarlyReviewProposalDocument::Human(document) => {
            let reason = document.reason.ok_or_else(|| {
                TerminalFailure::refusal(
                    "early-review override reason is missing",
                    "provide a concrete reason why waiting for a third occurrence is materially worse",
                )
            })?;
            let evidence = document.evidence.ok_or_else(|| {
                TerminalFailure::refusal(
                    "early-review override evidence is missing",
                    "add one or more recoverable evidence references bearing why waiting is worse",
                )
            })?;
            let review_appetite = document.review_appetite.ok_or_else(|| {
                TerminalFailure::refusal(
                    "early-review override review appetite is missing",
                    "bound the review effort before authorizing early review",
                )
            })?;
            EarlyReviewProposal {
                reason,
                review_appetite,
                evidence,
                prepared: None,
            }
        }
        EarlyReviewProposalDocument::Prepared(event) => {
            validate_recorded_early_review(&event)?;
            EarlyReviewProposal {
                reason: event.reason,
                review_appetite: event.review_appetite,
                evidence: event.evidence,
                prepared: Some(PreparedEarlyReview {
                    sequence: event.sequence,
                    event_id: event.event_id,
                    bytes: text,
                }),
            }
        }
    };
    validate_early_review_content(
        &proposal.reason,
        &proposal.review_appetite,
        &proposal.evidence,
    )?;
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
    validate_recorded_at(&event.recorded_at, "opening", "case open --preview")?;
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

fn validate_recorded_opening(event: &CaseOpenedEvent) -> Result<(), TerminalFailure> {
    validate_prepared_event(event)?;
    validate_opening_content(&event.responsibility, &event.occurrences)
}

fn validate_proposal(proposal: &OpenProposal) -> Result<(), TerminalFailure> {
    validate_opening_content(&proposal.responsibility, &proposal.occurrences)
}

fn validate_opening_content(
    responsibility: &str,
    occurrences: &[Occurrence],
) -> Result<(), TerminalFailure> {
    require_nonempty("responsibility", responsibility)?;
    if occurrences.len() < 2 {
        return Err(TerminalFailure::refusal(
            format!(
                "case opening requires at least two occurrences, but the proposal contains {}",
                occurrences.len()
            ),
            "add a second independently evidenced occurrence before opening the case",
        ));
    }
    let mut observed_consumers = BTreeSet::new();
    for (index, occurrence) in occurrences.iter().enumerate() {
        validate_occurrence(occurrence, index + 1, "occurrences.evidence")?;
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

fn validate_occurrence(
    occurrence: &Occurrence,
    index: usize,
    evidence_field: &str,
) -> Result<(), TerminalFailure> {
    if occurrence.consumer.trim().is_empty() {
        return Err(TerminalFailure::refusal(
            format!("occurrence {index} consumer is empty"),
            "provide a non-empty consumer label",
        ));
    }
    if occurrence.independence.trim().is_empty() {
        return Err(TerminalFailure::refusal(
            format!("occurrence {index} independence justification is empty"),
            "explain why this occurrence arose from an independent consumer need",
        ));
    }
    if occurrence.evidence.is_empty() {
        return Err(TerminalFailure::refusal(
            format!("occurrence {index} carries no evidence reference"),
            format!("add at least one recoverable `{evidence_field}` reference"),
        ));
    }
    for (evidence_index, evidence) in occurrence.evidence.iter().enumerate() {
        if evidence.reference.trim().is_empty() {
            return Err(TerminalFailure::refusal(
                format!(
                    "occurrence {index} evidence reference {} is empty",
                    evidence_index + 1
                ),
                "provide a recoverable commit reference",
            ));
        }
        if let Some(path) = &evidence.path {
            validate_relative_evidence_path(path)?;
        }
    }
    Ok(())
}

fn validate_recorded_append(event: &OccurrenceAppendedEvent) -> Result<(), TerminalFailure> {
    if event.schema_version != CASE_SCHEMA_VERSION
        || event.sequence <= OPENING_SEQUENCE
        || event.event_type != EventType::OccurrenceAppended
    {
        return Err(TerminalFailure::refusal(
            "prepared append event is not a supported `occurrence_appended` event after revision 1",
            "use the exact event rendered by `case append --preview`",
        ));
    }
    if event.event_id.get_version_num() != 4 {
        return Err(TerminalFailure::refusal(
            format!(
                "prepared append event identity `{}` is not an opaque UUID version 4",
                event.event_id
            ),
            "use the exact event rendered by `case append --preview`",
        ));
    }
    validate_recorded_at(&event.recorded_at, "append", "case append --preview")?;
    validate_occurrence(&event.occurrence, 1, "occurrence.evidence")
}

fn validate_recorded_early_review(
    event: &EarlyReviewAuthorizedEvent,
) -> Result<(), TerminalFailure> {
    if event.schema_version != CASE_SCHEMA_VERSION
        || event.sequence <= OPENING_SEQUENCE
        || event.event_type != EventType::EarlyReviewAuthorized
    {
        return Err(TerminalFailure::refusal(
            "prepared early-review event is not a supported `early_review_authorized` event after revision 1",
            "use the exact event rendered by `case override --preview`",
        ));
    }
    if event.event_id.get_version_num() != 4 {
        return Err(TerminalFailure::refusal(
            format!(
                "prepared early-review event identity `{}` is not an opaque UUID version 4",
                event.event_id
            ),
            "use the exact event rendered by `case override --preview`",
        ));
    }
    validate_recorded_at(
        &event.recorded_at,
        "early-review authorization",
        "case override --preview",
    )?;
    validate_early_review_content(&event.reason, &event.review_appetite, &event.evidence)
}

fn validate_early_review_content(
    reason: &str,
    review_appetite: &str,
    evidence: &[EvidenceReference],
) -> Result<(), TerminalFailure> {
    require_nonempty("reason", reason)?;
    require_nonempty("review_appetite", review_appetite)?;
    if evidence.is_empty() {
        return Err(TerminalFailure::refusal(
            "early-review override requires at least one evidence reference",
            "add one or more recoverable evidence references bearing why waiting is worse",
        ));
    }
    for (index, reference) in evidence.iter().enumerate() {
        if reference.reference.trim().is_empty() {
            return Err(TerminalFailure::refusal(
                format!("early-review evidence reference {} is empty", index + 1),
                "provide a recoverable commit reference bearing why waiting is worse",
            ));
        }
        if let Some(path) = &reference.path {
            validate_relative_evidence_path(path)?;
        }
    }
    Ok(())
}

fn validate_recorded_at(
    value: &str,
    event_name: &str,
    preview_command: &str,
) -> Result<(), TerminalFailure> {
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
            format!("prepared {event_name} event timestamp `{value}` is not UTC RFC 3339"),
            format!("use the exact event rendered by `{preview_command}`"),
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
            format!("prepared {event_name} event timestamp `{value}` is not a valid UTC instant"),
            format!("use the exact event rendered by `{preview_command}`"),
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
    occurrences: &[Occurrence],
) -> Result<BTreeMap<Uuid, Visibility>, TerminalFailure> {
    let roots = portfolio::selected_roots(root_overrides)?;
    let scan = portfolio::scan(&roots)?;
    let mut participants = BTreeMap::new();
    let requested = occurrences
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
