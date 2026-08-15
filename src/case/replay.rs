//! Replay of a recorded case event stream into its derived case state.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use uuid::Uuid;

use super::naming::{self, EventFileName, EventType, OPENING_SEQUENCE};
use super::{
    CaseOpenedEvent, EarlyReviewAuthorizedEvent, Occurrence, OccurrenceAppendedEvent,
    ReuseDecisionAcceptedEvent, VerificationDisposition, VerificationRecordedEvent,
};
use crate::{TerminalFailure, Visibility};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CaseState {
    Watching,
    ReviewReadyByOccurrenceCount,
    ReviewReadyByEarlyReviewOverride,
    AwaitingVerification,
    Closed,
    Parked,
    Reopened,
}

impl CaseState {
    pub(super) const fn from_occurrence_count(occurrence_count: usize) -> Self {
        if occurrence_count >= 3 {
            Self::ReviewReadyByOccurrenceCount
        } else {
            Self::Watching
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Watching => "watching",
            Self::ReviewReadyByOccurrenceCount | Self::ReviewReadyByEarlyReviewOverride => {
                "review-ready"
            }
            Self::AwaitingVerification => "awaiting-verification",
            Self::Closed => "closed",
            Self::Parked => "parked",
            Self::Reopened => "reopened",
        }
    }

    pub(super) const fn authorizes_review(self) -> bool {
        matches!(
            self,
            Self::ReviewReadyByOccurrenceCount | Self::ReviewReadyByEarlyReviewOverride
        )
    }

    pub(super) const fn basis(self) -> Option<&'static str> {
        match self {
            Self::Watching
            | Self::AwaitingVerification
            | Self::Closed
            | Self::Parked
            | Self::Reopened => None,
            Self::ReviewReadyByOccurrenceCount => Some("occurrence-count"),
            Self::ReviewReadyByEarlyReviewOverride => Some("early-review-override"),
        }
    }
}

/// One case folded from its recorded event stream.
///
/// This is derived state, and under ADR 0017 `case::render` is its only consumer besides this
/// module, so its fields are readable across `case` rather than through an accessor apiece. Its
/// invariants are established when it is folded, not by field privacy.
pub(super) struct CaseRecord {
    pub(super) case_id: Uuid,
    pub(super) responsibility: String,
    pub(super) revision: i64,
    pub(super) privacy: Visibility,
    pub(super) occurrences: Vec<Occurrence>,
    pub(super) early_review: Option<EarlyReviewAuthorizedEvent>,
    pub(super) decision: Option<ReuseDecisionAcceptedEvent>,
    pub(super) verifications: Vec<VerificationRecordedEvent>,
    pub(super) conditions: Conditions,
}

#[derive(Clone, Copy)]
pub(super) struct Conditions {
    pub(super) privacy_conflicted: Option<bool>,
    pub(super) stale: Option<bool>,
}

impl Conditions {
    const UNKNOWN: Self = Self {
        privacy_conflicted: None,
        stale: None,
    };
}

impl CaseRecord {
    pub(super) fn state(&self) -> CaseState {
        self.state_with_occurrence_count(self.occurrences.len())
    }

    pub(super) fn state_after_appending_occurrence(&self) -> CaseState {
        self.state_with_occurrence_count(self.occurrences.len() + 1)
    }

    /// The precedence an accepted decision, an early-review override and the
    /// occurrence count take over each other.
    ///
    /// The two callers differ only by whether the occurrence about to be
    /// appended is counted, so the rule they share is stated once here.
    fn state_with_occurrence_count(&self, occurrence_count: usize) -> CaseState {
        if let Some(verification) = self.verifications.last() {
            verification.content.disposition.state()
        } else if self.decision.is_some() {
            CaseState::AwaitingVerification
        } else if self.early_review.is_some() {
            CaseState::ReviewReadyByEarlyReviewOverride
        } else {
            CaseState::from_occurrence_count(occurrence_count)
        }
    }

    pub(super) const fn has_early_review(&self) -> bool {
        self.early_review.is_some()
    }

    pub(super) const fn has_decision(&self) -> bool {
        self.decision.is_some()
    }
}

impl super::publication::RevisionedCase for CaseRecord {
    fn revision(&self) -> i64 {
        self.revision
    }
}

enum CaseEvent {
    Opened(CaseOpenedEvent),
    OccurrenceAppended(OccurrenceAppendedEvent),
    EarlyReviewAuthorized(EarlyReviewAuthorizedEvent),
    ReuseDecisionAccepted(ReuseDecisionAcceptedEvent),
    VerificationRecorded(VerificationRecordedEvent),
}

#[derive(Deserialize)]
struct EventDiscriminator {
    event_type: EventType,
}

pub(super) fn replay(
    repository_root: &Path,
    case_id: Uuid,
    steward_repository_id: Uuid,
    event_paths: &[PathBuf],
    read_event: impl Fn(&Path) -> Result<String, TerminalFailure>,
) -> Result<CaseRecord, TerminalFailure> {
    if event_paths.is_empty() {
        return Err(TerminalFailure::refusal(
            format!("case `{case_id}` contains no event files"),
            "restore its opening event before reading the case",
        ));
    }
    validate_event_sequences(event_paths, case_id)?;
    let mut opening = None;
    let mut revision = 0;
    let mut occurrences = Vec::new();
    let mut early_review = None;
    let mut decision = None;
    let mut verifications = Vec::new();
    for event_path in event_paths {
        let (file_sequence, event) = read_case_event(
            repository_root,
            event_path,
            case_id,
            steward_repository_id,
            &read_event,
        )?;
        validate_not_extended_after_closure(case_id, &verifications)?;
        revision = revision.max(file_sequence);
        match event {
            CaseEvent::Opened(event) => {
                occurrences.extend(event.occurrences.iter().cloned());
                opening = Some(event);
            }
            CaseEvent::OccurrenceAppended(event) => occurrences.push(event.occurrence),
            CaseEvent::EarlyReviewAuthorized(event) => {
                if early_review.replace(event).is_some() {
                    return Err(TerminalFailure::refusal(
                        format!("case `{case_id}` records more than one early-review override"),
                        "restore exactly one early-review authorization event before reading the case",
                    ));
                }
            }
            CaseEvent::ReuseDecisionAccepted(event) => {
                validate_decision_prefix(case_id, &occurrences, early_review.as_ref(), &event)?;
                if decision.replace(event).is_some() {
                    return Err(TerminalFailure::refusal(
                        format!("case `{case_id}` records more than one accepted reuse decision"),
                        "restore exactly one accepted reuse decision event before reading the case",
                    ));
                }
            }
            CaseEvent::VerificationRecorded(event) => {
                validate_verification_prefix(case_id, decision.as_ref(), &verifications, &event)?;
                verifications.push(event);
            }
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
        early_review,
        decision,
        verifications,
        conditions: Conditions::UNKNOWN,
    })
}

fn validate_not_extended_after_closure(
    case_id: Uuid,
    verifications: &[VerificationRecordedEvent],
) -> Result<(), TerminalFailure> {
    let Some(closed) = verifications
        .last()
        .filter(|verification| verification.content.disposition == VerificationDisposition::Closed)
    else {
        return Ok(());
    };
    Err(TerminalFailure::refusal(
        format!(
            "case `{case_id}` records an event after its closed verification at sequence {}",
            closed.envelope.sequence
        ),
        "remove every event after the closed verification; closed is terminal in version 0.1",
    ))
}

fn validate_verification_prefix(
    case_id: Uuid,
    decision: Option<&ReuseDecisionAcceptedEvent>,
    prior_verifications: &[VerificationRecordedEvent],
    verification: &VerificationRecordedEvent,
) -> Result<(), TerminalFailure> {
    let Some(decision) = decision else {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{case_id}` records verification at sequence {} before an accepted reuse decision",
                verification.envelope.sequence
            ),
            "restore an earlier accepted reuse decision before the verification event",
        ));
    };
    if prior_verifications.last().is_some_and(|prior| {
        !matches!(
            prior.content.disposition,
            VerificationDisposition::Parked | VerificationDisposition::Reopened
        )
    }) {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{case_id}` records verification at sequence {} after a terminal disposition",
                verification.envelope.sequence
            ),
            "restore the event stream so only parked or reopened cases are verified again",
        ));
    }
    super::validate_verification_against_decision(case_id, &verification.content, &decision.content)
}

fn validate_decision_prefix(
    case_id: Uuid,
    occurrences: &[Occurrence],
    early_review: Option<&EarlyReviewAuthorizedEvent>,
    decision: &ReuseDecisionAcceptedEvent,
) -> Result<(), TerminalFailure> {
    let state = if early_review.is_some() {
        CaseState::ReviewReadyByEarlyReviewOverride
    } else {
        CaseState::from_occurrence_count(occurrences.len())
    };
    if !state.authorizes_review() {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{case_id}` records a decision at sequence {} whose event prefix is not review-ready",
                decision.envelope.sequence
            ),
            "restore a third earlier occurrence or an earlier human-authorized review override before the accepted decision",
        ));
    }
    super::validate_recorded_decision_participants(case_id, occurrences, decision)
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

fn read_case_event(
    repository_root: &Path,
    event_path: &Path,
    case_id: Uuid,
    steward_repository_id: Uuid,
    read_event: &impl Fn(&Path) -> Result<String, TerminalFailure>,
) -> Result<(i64, CaseEvent), TerminalFailure> {
    let relative_event_path = event_path.strip_prefix(repository_root).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "case event path `{}` is not steward-local: {error}",
            event_path.display()
        ))
    })?;
    super::validate_case_storage_path(repository_root, relative_event_path)?;
    let event_text = read_event(event_path)?;
    let discriminator = toml::from_str::<EventDiscriminator>(&event_text).map_err(|error| {
        TerminalFailure::refusal(
            format!("case event `{}` is invalid: {error}", event_path.display()),
            "restore a supported recorded event before reading the case",
        )
    })?;
    let file_sequence = naming::sequence_from_file_name(
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
    match discriminator.event_type {
        EventType::CaseOpened => {
            let event = toml::from_str::<CaseOpenedEvent>(&event_text)
                .map_err(|error| invalid_event(event_path, &error))?;
            validate_body_sequence(event_path, event.envelope.sequence, file_sequence)?;
            if event.envelope.sequence != OPENING_SEQUENCE {
                return Err(TerminalFailure::refusal(
                    format!(
                        "case event `{}` records `case_opened` at sequence {}",
                        event_path.display(),
                        event.envelope.sequence
                    ),
                    "restore `case_opened` as the single sequence 1 opening event before reading the case",
                ));
            }
            if event.envelope.schema_version != super::CASE_SCHEMA_VERSION
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
            validate_file_event_type(case_id, file_name, file_sequence, event.envelope.event_type)?;
            super::validate_recorded_opening(&event)?;
            Ok((file_sequence, CaseEvent::Opened(event)))
        }
        EventType::OccurrenceAppended => read_occurrence_appended_event(
            event_path,
            &event_text,
            case_id,
            file_sequence,
            file_name,
        ),
        EventType::EarlyReviewAuthorized => {
            read_early_review_event(event_path, &event_text, case_id, file_sequence, file_name)
        }
        EventType::ReuseDecisionAccepted => {
            read_reuse_decision_event(event_path, &event_text, case_id, file_sequence, file_name)
        }
        EventType::VerificationRecorded => {
            read_verification_event(event_path, &event_text, case_id, file_sequence, file_name)
        }
    }
}

fn read_occurrence_appended_event(
    event_path: &Path,
    event_text: &str,
    case_id: Uuid,
    file_sequence: i64,
    file_name: &str,
) -> Result<(i64, CaseEvent), TerminalFailure> {
    let event = toml::from_str::<OccurrenceAppendedEvent>(event_text)
        .map_err(|error| invalid_event(event_path, &error))?;
    validate_body_sequence(event_path, event.envelope.sequence, file_sequence)?;
    if event.envelope.sequence == OPENING_SEQUENCE {
        return Err(TerminalFailure::refusal(
            format!(
                "case event `{}` records `occurrence_appended` at opening sequence 1",
                event_path.display()
            ),
            "restore `case_opened` as sequence 1 and append occurrences only after it",
        ));
    }
    validate_file_event_type(case_id, file_name, file_sequence, event.envelope.event_type)?;
    super::validate_recorded_append(&event)?;
    Ok((file_sequence, CaseEvent::OccurrenceAppended(event)))
}

fn read_early_review_event(
    event_path: &Path,
    event_text: &str,
    case_id: Uuid,
    file_sequence: i64,
    file_name: &str,
) -> Result<(i64, CaseEvent), TerminalFailure> {
    let event = toml::from_str::<EarlyReviewAuthorizedEvent>(event_text)
        .map_err(|error| invalid_event(event_path, &error))?;
    validate_body_sequence(event_path, event.envelope.sequence, file_sequence)?;
    if event.envelope.sequence == OPENING_SEQUENCE {
        return Err(TerminalFailure::refusal(
            format!(
                "case event `{}` records `early_review_authorized` at opening sequence 1",
                event_path.display()
            ),
            "restore `case_opened` as sequence 1 and authorize early review only after it",
        ));
    }
    validate_file_event_type(case_id, file_name, file_sequence, event.envelope.event_type)?;
    super::validate_recorded_early_review(&event)?;
    Ok((file_sequence, CaseEvent::EarlyReviewAuthorized(event)))
}

fn read_reuse_decision_event(
    event_path: &Path,
    event_text: &str,
    case_id: Uuid,
    file_sequence: i64,
    file_name: &str,
) -> Result<(i64, CaseEvent), TerminalFailure> {
    let event = toml::from_str::<ReuseDecisionAcceptedEvent>(event_text)
        .map_err(|error| invalid_event(event_path, &error))?;
    validate_body_sequence(event_path, event.envelope.sequence, file_sequence)?;
    if event.envelope.sequence == OPENING_SEQUENCE {
        return Err(TerminalFailure::refusal(
            format!(
                "case event `{}` records `{}` at opening sequence 1",
                event_path.display(),
                event.envelope.event_type.body_name()
            ),
            "restore `case_opened` as sequence 1 and accept reuse decisions only after it",
        ));
    }
    validate_file_event_type(case_id, file_name, file_sequence, event.envelope.event_type)?;
    super::validate_recorded_decision(&event)?;
    Ok((file_sequence, CaseEvent::ReuseDecisionAccepted(event)))
}

fn read_verification_event(
    event_path: &Path,
    event_text: &str,
    case_id: Uuid,
    file_sequence: i64,
    file_name: &str,
) -> Result<(i64, CaseEvent), TerminalFailure> {
    let event = toml::from_str::<VerificationRecordedEvent>(event_text)
        .map_err(|error| invalid_event(event_path, &error))?;
    validate_body_sequence(event_path, event.envelope.sequence, file_sequence)?;
    if event.envelope.sequence == OPENING_SEQUENCE {
        return Err(TerminalFailure::refusal(
            format!(
                "case event `{}` records `{}` at opening sequence 1",
                event_path.display(),
                event.envelope.event_type.body_name()
            ),
            "restore `case_opened` as sequence 1 and record verification only after an accepted reuse decision",
        ));
    }
    validate_file_event_type(case_id, file_name, file_sequence, event.envelope.event_type)?;
    super::validate_recorded_verification(&event)?;
    Ok((file_sequence, CaseEvent::VerificationRecorded(event)))
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
    recorded_event_type: EventType,
) -> Result<(), TerminalFailure> {
    let expected_file_name = EventFileName::new(file_sequence, recorded_event_type)
        .expect("the validated recorded event has an accepted file identity")
        .to_string();
    let file_identity_matches = EventFileName::parse(file_name).is_some_and(|identity| {
        identity.sequence() == file_sequence && identity.event_type() == recorded_event_type
    });
    if !file_identity_matches {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{case_id}` event file `{file_name}` does not match its recorded type `{}`",
                recorded_event_type.body_name()
            ),
            format!("restore the event as `{expected_file_name}` before reading the case"),
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
        let sequence = naming::sequence_from_file_name(file_name).ok_or_else(|| {
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    #[cfg(unix)]
    use std::ffi::OsString;
    #[cfg(unix)]
    use std::os::unix::ffi::OsStringExt;
    use std::path::Path;

    use uuid::Uuid;

    use super::*;
    use crate::case::event::Envelope;
    use crate::case::naming::EventType;
    use crate::case::{
        AffectedConsumer, CaseOpenedEvent, ConditionResult, ConsumerResult, DecisionAction,
        DecisionContent, EarlyReviewAuthorizedEvent, EvidenceKind, EvidenceReference,
        IdentityVerdict, Occurrence, OccurrenceAppendedEvent, RejectedAlternative,
        ReuseDecisionAcceptedEvent, VerificationContent, VerificationDisposition,
        VerificationRecordedEvent, VerificationResult,
    };
    use crate::{ExitMeaning, TerminalFailure, Visibility};

    const CASE_ID: &str = "33333333-3333-4333-8333-333333333333";
    const STEWARD_ID: &str = "11111111-1111-4111-8111-111111111111";
    const FIRST_PARTICIPANT_ID: &str = STEWARD_ID;
    const SECOND_PARTICIPANT_ID: &str = "22222222-2222-4222-8222-222222222222";
    const THIRD_PARTICIPANT_ID: &str = "55555555-5555-4555-8555-555555555555";
    const EVENT_ID: &str = "44444444-4444-4444-8444-444444444444";
    const ACCEPTED_CONDITION: &str = "neither consumer gains a dependency";
    const AFFECTED_CONSUMER: &str = "billing totals";
    const EVENT_DIRECTORY: &str = "cases/33333333-3333-4333-8333-333333333333";

    fn uuid(value: &str) -> Uuid {
        Uuid::parse_str(value).expect("the fixture identity is a valid UUID")
    }

    fn case_id() -> Uuid {
        uuid(CASE_ID)
    }

    fn envelope(sequence: i64, event_type: EventType) -> Envelope {
        Envelope {
            schema_version: super::super::CASE_SCHEMA_VERSION,
            sequence,
            event_id: uuid(EVENT_ID),
            event_type,
            recorded_at: "2026-08-15T06:00:00Z".to_owned(),
        }
    }

    fn occurrence(repository_id: &str, consumer: &str) -> Occurrence {
        Occurrence {
            repository_id: uuid(repository_id),
            consumer: consumer.to_owned(),
            independence: "arose from a separate consumer need".to_owned(),
            evidence: vec![EvidenceReference {
                kind: EvidenceKind::Commit,
                reference: "abc123".to_owned(),
                path: Some("src/lib.rs".to_owned()),
            }],
        }
    }

    fn decision(sequence: i64) -> ReuseDecisionAcceptedEvent {
        ReuseDecisionAcceptedEvent {
            envelope: envelope(sequence, EventType::ReuseDecisionAccepted),
            content: DecisionContent {
                identity_verdict: IdentityVerdict::DifferentResponsibilities,
                action: DecisionAction::RetainIntentionalDuplication,
                accepted_scope: "no shared surface".to_owned(),
                non_responsibilities: vec!["rounding policy".to_owned()],
                affected_consumers: vec![AffectedConsumer {
                    repository_id: uuid(STEWARD_ID),
                    consumer: AFFECTED_CONSUMER.to_owned(),
                    expectation: "stays as it is".to_owned(),
                }],
                alternatives_rejected: vec![RejectedAlternative {
                    alternative: "extract locally".to_owned(),
                    reason: "the two totals change for different reasons".to_owned(),
                }],
                compatibility_consequences: "nothing changes".to_owned(),
                verification_conditions: vec![ACCEPTED_CONDITION.to_owned()],
                invariant_contract: None,
                existing_packages_considered: None,
                required_consumer_level_tests: None,
                migration_expectations: None,
                rollback_or_resplitting_path: None,
            },
        }
    }

    fn early_review(sequence: i64) -> EarlyReviewAuthorizedEvent {
        EarlyReviewAuthorizedEvent {
            envelope: envelope(sequence, EventType::EarlyReviewAuthorized),
            reason: "divergence is already costing release time".to_owned(),
            review_appetite: "one afternoon".to_owned(),
            evidence: Vec::new(),
        }
    }

    fn valid_early_review(sequence: i64) -> EarlyReviewAuthorizedEvent {
        let mut event = early_review(sequence);
        event.evidence.push(EvidenceReference {
            kind: EvidenceKind::Commit,
            reference: "def456".to_owned(),
            path: Some("src/case.rs".to_owned()),
        });
        event
    }

    fn verification(
        sequence: i64,
        disposition: VerificationDisposition,
    ) -> VerificationRecordedEvent {
        VerificationRecordedEvent {
            envelope: envelope(sequence, EventType::VerificationRecorded),
            content: VerificationContent {
                disposition,
                condition_results: vec![ConditionResult {
                    condition: ACCEPTED_CONDITION.to_owned(),
                    outcome: VerificationResult::Met,
                    exception: None,
                    evidence: Vec::new(),
                }],
                consumer_results: vec![ConsumerResult {
                    repository_id: uuid(STEWARD_ID),
                    consumer: AFFECTED_CONSUMER.to_owned(),
                    outcome: VerificationResult::Met,
                    exception: None,
                    evidence: Vec::new(),
                }],
            },
        }
    }

    fn recorded_event_path(file_name: &str) -> PathBuf {
        Path::new("steward")
            .join("reuse-evidence/cases")
            .join(CASE_ID)
            .join(file_name)
    }

    fn event_path(file_name: &str) -> PathBuf {
        Path::new(EVENT_DIRECTORY).join(file_name)
    }

    fn opening(case_id: Uuid, steward_repository_id: Uuid) -> CaseOpenedEvent {
        CaseOpenedEvent {
            envelope: envelope(1, EventType::CaseOpened),
            case_id,
            responsibility: "render billing totals".to_owned(),
            steward_repository_id,
            privacy: Visibility::Public,
            occurrences: vec![
                occurrence(STEWARD_ID, AFFECTED_CONSUMER),
                occurrence(SECOND_PARTICIPANT_ID, "invoice totals"),
            ],
        }
    }

    fn failure<T>(result: Result<T, TerminalFailure>) -> String {
        match result {
            Ok(_) => panic!("the replay must refuse this recorded stream"),
            Err(failure) => failure.to_string(),
        }
    }

    fn refusal(result: Result<(), TerminalFailure>) -> String {
        result
            .expect_err("the validator must refuse this recorded stream")
            .to_string()
    }

    #[test]
    fn a_case_without_event_files_is_refused() {
        assert_eq!(
            failure(replay(
                Path::new("steward"),
                uuid(CASE_ID),
                uuid(STEWARD_ID),
                &[],
                |_| panic!("an empty replay must not call its event reader"),
            )),
            format!(
                "refusal: case `{CASE_ID}` contains no event files\nresolution: restore its opening event before reading the case"
            )
        );
    }

    #[test]
    fn an_event_path_outside_the_steward_is_an_unsafe_failure() {
        let event_path = Path::new("elsewhere")
            .join("reuse-evidence/cases")
            .join(CASE_ID)
            .join("0001-case-opened.toml");
        let Err(failure) = replay(
            Path::new("steward"),
            uuid(CASE_ID),
            uuid(STEWARD_ID),
            std::slice::from_ref(&event_path),
            |_| panic!("steward-local validation must precede event reads"),
        ) else {
            panic!("a non-steward-local event path must fail unsafely");
        };

        assert_eq!(failure.meaning(), ExitMeaning::UnsafeFailure, "{failure}");
        assert_eq!(
            failure.to_string(),
            format!(
                "unsafe failure: case event path `{}` is not steward-local: prefix not found",
                event_path.display()
            )
        );
    }

    #[test]
    fn non_opening_events_at_opening_sequence_are_refused() {
        let fixtures = [
            (
                "0001-occurrence-appended.toml",
                toml::to_string(&OccurrenceAppendedEvent {
                    envelope: envelope(1, EventType::OccurrenceAppended),
                    occurrence: occurrence(SECOND_PARTICIPANT_ID, "invoice totals"),
                })
                .expect("the fixture event serializes"),
                "records `occurrence_appended` at opening sequence 1",
                "restore `case_opened` as sequence 1 and append occurrences only after it",
            ),
            (
                "0001-early-review-authorized.toml",
                toml::to_string(&EarlyReviewAuthorizedEvent {
                    envelope: envelope(1, EventType::EarlyReviewAuthorized),
                    reason: "waiting already costs release time".to_owned(),
                    review_appetite: "one afternoon".to_owned(),
                    evidence: Vec::new(),
                })
                .expect("the fixture event serializes"),
                "records `early_review_authorized` at opening sequence 1",
                "restore `case_opened` as sequence 1 and authorize early review only after it",
            ),
            (
                "0001-reuse-decision-accepted.toml",
                toml::to_string(&decision(1)).expect("the fixture event serializes"),
                "records `reuse_decision_accepted` at opening sequence 1",
                "restore `case_opened` as sequence 1 and accept reuse decisions only after it",
            ),
            (
                "0001-verification-recorded.toml",
                toml::to_string(&verification(1, VerificationDisposition::Closed))
                    .expect("the fixture event serializes"),
                "records `verification_recorded` at opening sequence 1",
                "restore `case_opened` as sequence 1 and record verification only after an accepted reuse decision",
            ),
        ];

        for (file_name, event_text, condition, resolution) in fixtures {
            let event_path = recorded_event_path(file_name);
            assert_eq!(
                failure(replay(
                    Path::new("steward"),
                    uuid(CASE_ID),
                    uuid(STEWARD_ID),
                    std::slice::from_ref(&event_path),
                    |path| {
                        assert_eq!(path, event_path);
                        Ok(event_text.clone())
                    },
                )),
                format!(
                    "refusal: case event `{}` {condition}\nresolution: {resolution}",
                    event_path.display()
                )
            );
        }
    }

    #[test]
    fn an_event_that_does_not_match_its_steward_local_case_is_refused() {
        let event_path = recorded_event_path("0001-case-opened.toml");
        let event_text = toml::to_string(&opening(uuid(SECOND_PARTICIPANT_ID), uuid(STEWARD_ID)))
            .expect("the fixture event serializes");

        assert_eq!(
            failure(replay(
                Path::new("steward"),
                uuid(CASE_ID),
                uuid(STEWARD_ID),
                std::slice::from_ref(&event_path),
                |path| {
                    assert_eq!(path, event_path);
                    Ok(event_text.clone())
                },
            )),
            format!(
                "refusal: case event `{}` does not match its steward-local case\nresolution: restore the event under the case and steward identities it records",
                event_path.display()
            )
        );
    }

    #[test]
    fn more_than_one_early_review_override_is_refused() {
        let event_paths = [
            recorded_event_path("0001-case-opened.toml"),
            recorded_event_path("0002-early-review-authorized.toml"),
            recorded_event_path("0003-early-review-authorized.toml"),
        ];
        let events = BTreeMap::from([
            (
                event_paths[0].clone(),
                toml::to_string(&opening(uuid(CASE_ID), uuid(STEWARD_ID)))
                    .expect("the fixture event serializes"),
            ),
            (
                event_paths[1].clone(),
                toml::to_string(&valid_early_review(2)).expect("the fixture event serializes"),
            ),
            (
                event_paths[2].clone(),
                toml::to_string(&valid_early_review(3)).expect("the fixture event serializes"),
            ),
        ]);

        assert_eq!(
            failure(replay(
                Path::new("steward"),
                uuid(CASE_ID),
                uuid(STEWARD_ID),
                &event_paths,
                |path| {
                    Ok(events
                        .get(path)
                        .expect("the replay requests a fixture event")
                        .clone())
                },
            )),
            format!(
                "refusal: case `{CASE_ID}` records more than one early-review override\nresolution: restore exactly one early-review authorization event before reading the case"
            )
        );
    }

    #[test]
    fn more_than_one_accepted_reuse_decision_is_refused() {
        let event_paths = [
            recorded_event_path("0001-case-opened.toml"),
            recorded_event_path("0002-reuse-decision-accepted.toml"),
            recorded_event_path("0003-reuse-decision-accepted.toml"),
        ];
        let mut opening = opening(uuid(CASE_ID), uuid(STEWARD_ID));
        opening
            .occurrences
            .push(occurrence(THIRD_PARTICIPANT_ID, "statement totals"));
        let events = BTreeMap::from([
            (
                event_paths[0].clone(),
                toml::to_string(&opening).expect("the fixture event serializes"),
            ),
            (
                event_paths[1].clone(),
                toml::to_string(&decision(2)).expect("the fixture event serializes"),
            ),
            (
                event_paths[2].clone(),
                toml::to_string(&decision(3)).expect("the fixture event serializes"),
            ),
        ]);

        assert_eq!(
            failure(replay(
                Path::new("steward"),
                uuid(CASE_ID),
                uuid(STEWARD_ID),
                &event_paths,
                |path| {
                    Ok(events
                        .get(path)
                        .expect("the replay requests a fixture event")
                        .clone())
                },
            )),
            format!(
                "refusal: case `{CASE_ID}` records more than one accepted reuse decision\nresolution: restore exactly one accepted reuse decision event before reading the case"
            )
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_event_without_a_utf8_filename_is_refused() {
        let event_path = Path::new("steward")
            .join("reuse-evidence/cases")
            .join(CASE_ID)
            .join(OsString::from_vec(vec![b'0', b'0', b'0', b'1', b'-', 0xff]));

        assert_eq!(
            failure(replay(
                Path::new("steward"),
                uuid(CASE_ID),
                uuid(STEWARD_ID),
                &[event_path],
                |_| panic!("filename validation must precede event reads"),
            )),
            format!(
                "refusal: case `{CASE_ID}` contains an event without a UTF-8 filename\nresolution: restore sequence-numbered UTF-8 TOML event filenames before reading the case"
            )
        );
    }

    #[test]
    fn a_contiguous_event_sequence_set_is_accepted() {
        let paths = [
            event_path("0001-case-opened.toml"),
            event_path("0002-occurrence-appended.toml"),
            event_path("0003-reuse-decision-accepted.toml"),
        ];

        assert!(validate_event_sequences(&paths, case_id()).is_ok());
    }

    /// The paths arrive sorted, which is the order the refusal names the colliding files in.
    #[test]
    fn two_event_files_sharing_a_sequence_number_are_refused() {
        let paths = [
            event_path("0001-case-opened.toml"),
            event_path("0002-early-review-authorized.toml"),
            event_path("0002-occurrence-appended.toml"),
        ];

        assert_eq!(
            refusal(validate_event_sequences(&paths, case_id())),
            format!(
                "refusal: case `{CASE_ID}` has duplicated sequence number 2 in files `0002-early-review-authorized.toml`, `0002-occurrence-appended.toml`\nresolution: restore exactly one event file for sequence 2 before reading the case"
            )
        );
    }

    #[test]
    fn a_gap_in_the_event_sequence_is_refused() {
        let paths = [
            event_path("0001-case-opened.toml"),
            event_path("0003-reuse-decision-accepted.toml"),
        ];

        assert_eq!(
            refusal(validate_event_sequences(&paths, case_id())),
            format!(
                "refusal: case `{CASE_ID}` is missing sequence number 2 before recorded sequence 3\nresolution: restore event file sequence 2 so the case stream is contiguous before reading it"
            )
        );
    }

    #[test]
    fn an_event_file_outside_the_accepted_grammar_is_refused() {
        let paths = [
            event_path("0001-case-opened.toml"),
            event_path("notes.toml"),
        ];

        assert_eq!(
            refusal(validate_event_sequences(&paths, case_id())),
            format!(
                "refusal: case `{CASE_ID}` contains unrecognized event file `notes.toml`\nresolution: restore event filenames in `NNNN-<event-type>.toml` form before reading the case"
            )
        );
    }

    #[test]
    fn a_file_name_naming_its_recorded_event_type_is_accepted() {
        assert!(
            validate_file_event_type(
                case_id(),
                "0002-occurrence-appended.toml",
                2,
                EventType::OccurrenceAppended,
            )
            .is_ok()
        );
    }

    /// ADR 0011 made the body spelling and the file-name slug one declaration; this is the
    /// refusal that holds a recorded pair to it.
    #[test]
    fn a_file_name_disagreeing_with_its_recorded_event_type_is_refused() {
        assert_eq!(
            refusal(validate_file_event_type(
                case_id(),
                "0002-occurrence-appended.toml",
                2,
                EventType::EarlyReviewAuthorized,
            )),
            format!(
                "refusal: case `{CASE_ID}` event file `0002-occurrence-appended.toml` does not match its recorded type `early_review_authorized`\nresolution: restore the event as `0002-early-review-authorized.toml` before reading the case"
            )
        );
    }

    #[test]
    fn a_body_sequence_matching_its_file_name_is_accepted() {
        assert!(validate_body_sequence(&event_path("0002-occurrence-appended.toml"), 2, 2).is_ok());
    }

    #[test]
    fn a_body_sequence_disagreeing_with_its_file_name_is_refused() {
        let path = event_path("0002-occurrence-appended.toml");

        assert_eq!(
            refusal(validate_body_sequence(&path, 3, 2)),
            format!(
                "refusal: case event `{}` records sequence 3 but its filename records sequence 2\nresolution: restore the event under the filename matching its recorded sequence before reading the case",
                path.display()
            )
        );
    }

    #[test]
    fn a_case_recording_no_verification_permits_a_later_event() {
        assert!(validate_not_extended_after_closure(case_id(), &[]).is_ok());
    }

    /// `CONTEXT.md` makes closed terminal in version 0.1, so nothing may follow it.
    #[test]
    fn an_event_after_a_closed_verification_is_refused() {
        let verifications = [verification(4, VerificationDisposition::Closed)];

        assert_eq!(
            refusal(validate_not_extended_after_closure(
                case_id(),
                &verifications
            )),
            format!(
                "refusal: case `{CASE_ID}` records an event after its closed verification at sequence 4\nresolution: remove every event after the closed verification; closed is terminal in version 0.1"
            )
        );
    }

    /// Parked and reopened both stand against the same decision, so both permit a later event.
    #[test]
    fn an_event_after_a_parked_or_reopened_verification_is_accepted() {
        for disposition in [
            VerificationDisposition::Parked,
            VerificationDisposition::Reopened,
        ] {
            let verifications = [verification(4, disposition)];

            assert!(
                validate_not_extended_after_closure(case_id(), &verifications).is_ok(),
                "a {} verification must permit a later event",
                disposition.label()
            );
        }
    }

    /// ADR 0019 records verification against a standing decision, so a stream that verifies
    /// before one is refused.
    #[test]
    fn a_verification_before_an_accepted_decision_is_refused() {
        let recorded = verification(3, VerificationDisposition::Closed);

        assert_eq!(
            refusal(validate_verification_prefix(
                case_id(),
                None,
                &[],
                &recorded
            )),
            format!(
                "refusal: case `{CASE_ID}` records verification at sequence 3 before an accepted reuse decision\nresolution: restore an earlier accepted reuse decision before the verification event"
            )
        );
    }

    #[test]
    fn the_first_verification_after_an_accepted_decision_is_accepted() {
        let accepted = decision(3);
        let recorded = verification(4, VerificationDisposition::Closed);

        assert!(validate_verification_prefix(case_id(), Some(&accepted), &[], &recorded).is_ok());
    }

    #[test]
    fn a_verification_after_a_closed_verification_is_refused() {
        let accepted = decision(3);
        let prior = [verification(4, VerificationDisposition::Closed)];
        let recorded = verification(5, VerificationDisposition::Reopened);

        assert_eq!(
            refusal(validate_verification_prefix(
                case_id(),
                Some(&accepted),
                &prior,
                &recorded
            )),
            format!(
                "refusal: case `{CASE_ID}` records verification at sequence 5 after a terminal disposition\nresolution: restore the event stream so only parked or reopened cases are verified again"
            )
        );
    }

    #[test]
    fn a_verification_after_a_parked_or_reopened_verification_is_accepted() {
        let accepted = decision(3);
        for disposition in [
            VerificationDisposition::Parked,
            VerificationDisposition::Reopened,
        ] {
            let prior = [verification(4, disposition)];
            let recorded = verification(5, VerificationDisposition::Closed);

            assert!(
                validate_verification_prefix(case_id(), Some(&accepted), &prior, &recorded).is_ok(),
                "a {} verification must permit verifying again",
                disposition.label()
            );
        }
    }

    /// `FOUNDATIONS.md` §6 makes the third occurrence the ordinary review threshold, and a
    /// recorded decision is held to the same prefix a live one is.
    #[test]
    fn a_decision_below_the_third_occurrence_is_refused() {
        let occurrences = [
            occurrence(FIRST_PARTICIPANT_ID, AFFECTED_CONSUMER),
            occurrence(SECOND_PARTICIPANT_ID, "invoice totals"),
        ];
        let accepted = decision(3);

        assert_eq!(
            refusal(validate_decision_prefix(
                case_id(),
                &occurrences,
                None,
                &accepted
            )),
            format!(
                "refusal: case `{CASE_ID}` records a decision at sequence 3 whose event prefix is not review-ready\nresolution: restore a third earlier occurrence or an earlier human-authorized review override before the accepted decision"
            )
        );
    }

    #[test]
    fn a_decision_after_a_third_occurrence_is_accepted() {
        let occurrences = [
            occurrence(FIRST_PARTICIPANT_ID, AFFECTED_CONSUMER),
            occurrence(SECOND_PARTICIPANT_ID, "invoice totals"),
            occurrence(THIRD_PARTICIPANT_ID, "statement totals"),
        ];
        let accepted = decision(4);

        assert!(validate_decision_prefix(case_id(), &occurrences, None, &accepted).is_ok());
    }

    /// The override authorizes review after the second occurrence, which is the other prefix a
    /// recorded decision may stand on.
    #[test]
    fn a_decision_under_an_earlier_review_override_is_accepted() {
        let occurrences = [
            occurrence(FIRST_PARTICIPANT_ID, AFFECTED_CONSUMER),
            occurrence(SECOND_PARTICIPANT_ID, "invoice totals"),
        ];
        let authorized = early_review(3);
        let accepted = decision(4);

        assert!(
            validate_decision_prefix(case_id(), &occurrences, Some(&authorized), &accepted).is_ok()
        );
    }

    #[test]
    fn distinct_participant_and_consumer_pairs_are_accepted() {
        let occurrences = [
            occurrence(FIRST_PARTICIPANT_ID, AFFECTED_CONSUMER),
            occurrence(SECOND_PARTICIPANT_ID, "invoice totals"),
        ];

        assert!(validate_unique_occurrences(case_id(), &occurrences).is_ok());
    }

    /// `FOUNDATIONS.md` §5 counts consumer needs rather than repositories, so one participant
    /// may record two distinct consumers.
    #[test]
    fn one_participant_may_record_two_distinct_consumers() {
        let occurrences = [
            occurrence(FIRST_PARTICIPANT_ID, AFFECTED_CONSUMER),
            occurrence(FIRST_PARTICIPANT_ID, "invoice totals"),
        ];

        assert!(validate_unique_occurrences(case_id(), &occurrences).is_ok());
    }

    #[test]
    fn a_repeated_participant_and_consumer_pair_is_refused() {
        let occurrences = [
            occurrence(FIRST_PARTICIPANT_ID, AFFECTED_CONSUMER),
            occurrence(FIRST_PARTICIPANT_ID, AFFECTED_CONSUMER),
        ];

        assert_eq!(
            refusal(validate_unique_occurrences(case_id(), &occurrences)),
            format!(
                "refusal: case `{CASE_ID}` records participant `{FIRST_PARTICIPANT_ID}` and consumer `{AFFECTED_CONSUMER}` more than once\nresolution: restore the authoritative event stream so each participant repository and consumer pair occurs once before reading the case"
            )
        );
    }
}
