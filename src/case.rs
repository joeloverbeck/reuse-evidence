//! Durable case recording and inspection mechanics.

mod event;
mod instant;
mod naming;
mod publication;
mod read;
mod render;

pub use instant::RecordedInstant;
pub(crate) use naming::cases_root;
pub(crate) use read::private_case_stewarded_by;
pub use read::{BriefOutcome, FindOutcome, ListOutcome, ShowOutcome, brief, find, list, show};

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use naming::{EventFileName, EventPosition, EventType, OPENING_SEQUENCE};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::marker;
use crate::portfolio;
use crate::{TerminalFailure, Visibility, create_file_atomically};

const CASE_SCHEMA_VERSION: i64 = 1;
const IMPLEMENTATION_NOTICE: &str =
    "decision: authorizes implementation outside the reuse lifecycle; does not perform it";
const NO_IMPLEMENTATION_NOTICE: &str = "decision: authorizes no implementation";
const APPEND_UNSTEWARDED_RESOLUTION: &str =
    "run `case list` in this steward repository and retry with a recorded case identity";
const EARLY_REVIEW_UNSTEWARDED_RESOLUTION: &str = "run `case list` in this steward repository and retry `case override` with a recorded watching case identity";
const DECISION_UNSTEWARDED_RESOLUTION: &str = "run `case list` in this steward repository and retry `case decide` with a recorded review-ready case identity";
const VERIFICATION_UNSTEWARDED_RESOLUTION: &str = "run `case list` in this steward repository and retry `case verify` with a recorded awaiting-verification, parked, or reopened case identity";
/// What each command tells a reader to do about a steward marker it cannot use.
///
/// ADR 0018 makes the marker fault's own wording shared and this sentence the
/// command's, because it names the command to retry. `case::read` holds the
/// three for the query commands.
const OPEN_MARKER_RESOLUTION: &str =
    "restore a supported `reuse-evidence.toml` marker before opening a case";
const APPEND_MARKER_RESOLUTION: &str =
    "restore a supported `reuse-evidence.toml` marker before appending an occurrence";
const EARLY_REVIEW_MARKER_RESOLUTION: &str =
    "restore a supported `reuse-evidence.toml` marker before authorizing early review";
const DECISION_MARKER_RESOLUTION: &str =
    "restore a supported `reuse-evidence.toml` marker before recording a reuse decision";
const VERIFICATION_MARKER_RESOLUTION: &str =
    "restore a supported `reuse-evidence.toml` marker before recording verification";

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
#[serde(untagged)]
enum DecisionProposalDocument {
    Prepared(ReuseDecisionAcceptedEvent),
    Human(DecisionContent),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum VerificationProposalDocument {
    Prepared(VerificationRecordedEvent),
    Human(VerificationContent),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DecisionContent {
    identity_verdict: IdentityVerdict,
    action: DecisionAction,
    accepted_scope: String,
    non_responsibilities: Vec<String>,
    affected_consumers: Vec<AffectedConsumer>,
    alternatives_rejected: Vec<RejectedAlternative>,
    compatibility_consequences: String,
    verification_conditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invariant_contract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    existing_packages_considered: Option<Vec<ExistingPackageConsidered>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required_consumer_level_tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    migration_expectations: Option<Vec<MigrationExpectation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rollback_or_resplitting_path: Option<String>,
}

/// The five fields an implementation-authorizing decision must record, resolved.
///
/// `validate_change_decision_content` refuses an implementation-authorizing decision that omits
/// any of them, so recorded content always has all five. This value is what carries that proof
/// past validation.
#[derive(Debug)]
struct AuthorizedImplementation {
    invariant_contract: String,
    existing_packages_considered: Vec<ExistingPackageConsidered>,
    required_consumer_level_tests: Vec<String>,
    migration_expectations: Vec<MigrationExpectation>,
    rollback_or_resplitting_path: String,
}

/// What an accepted decision authorizes, holding whatever that authorization requires.
#[derive(Debug)]
enum DecisionAuthorization {
    Implementation(AuthorizedImplementation),
    NoImplementation,
}

impl DecisionContent {
    /// Resolves what this content authorizes.
    ///
    /// The action and the five change-action fields correspond by validation, not by type, so
    /// something has to bridge them. Doing it here — where a refusal is still expressible — is
    /// what lets `case::render` print a brief without re-asserting that correspondence at six
    /// sites where `Display` could only panic.
    fn authorization(&self) -> Result<DecisionAuthorization, TerminalFailure> {
        if !self.action.authorizes_implementation() {
            return Ok(DecisionAuthorization::NoImplementation);
        }
        Ok(DecisionAuthorization::Implementation(
            AuthorizedImplementation {
                invariant_contract: self
                    .invariant_contract
                    .clone()
                    .ok_or_else(|| missing_change_decision_item("invariant_contract"))?,
                existing_packages_considered: self
                    .existing_packages_considered
                    .clone()
                    .ok_or_else(|| missing_change_decision_item("existing_packages_considered"))?,
                required_consumer_level_tests: self
                    .required_consumer_level_tests
                    .clone()
                    .ok_or_else(|| missing_change_decision_item("required_consumer_level_tests"))?,
                migration_expectations: self
                    .migration_expectations
                    .clone()
                    .ok_or_else(|| missing_change_decision_item("migration_expectations"))?,
                rollback_or_resplitting_path: self
                    .rollback_or_resplitting_path
                    .clone()
                    .ok_or_else(|| missing_change_decision_item("rollback_or_resplitting_path"))?,
            },
        ))
    }
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

#[derive(Debug)]
struct DecisionProposal {
    content: DecisionContent,
    prepared: Option<PreparedDecision>,
}

#[derive(Debug)]
struct PreparedDecision {
    sequence: i64,
    event_id: Uuid,
    bytes: String,
}

#[derive(Debug)]
struct VerificationProposal {
    content: VerificationContent,
    prepared: Option<PreparedVerification>,
}

#[derive(Debug)]
struct PreparedVerification {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VerificationContent {
    disposition: VerificationDisposition,
    condition_results: Vec<ConditionResult>,
    consumer_results: Vec<ConsumerResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ConditionResult {
    condition: String,
    outcome: VerificationResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    exception: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    evidence: Vec<EvidenceReference>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ConsumerResult {
    repository_id: Uuid,
    consumer: String,
    outcome: VerificationResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    exception: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    evidence: Vec<EvidenceReference>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum VerificationResult {
    Met,
    NotMet,
    AcceptedException,
}

impl VerificationResult {
    const fn label(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::NotMet => "not_met",
            Self::AcceptedException => "accepted_exception",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum VerificationDisposition {
    Closed,
    Parked,
    Reopened,
}

impl VerificationDisposition {
    const fn label(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Parked => "parked",
            Self::Reopened => "reopened",
        }
    }

    const fn state(self) -> read::CaseState {
        match self {
            Self::Closed => read::CaseState::Closed,
            Self::Parked => read::CaseState::Parked,
            Self::Reopened => read::CaseState::Reopened,
        }
    }

    const fn notice(self) -> &'static str {
        match self {
            Self::Closed => "disposition: closed",
            Self::Parked => "disposition: parked",
            Self::Reopened => "disposition: reopened",
        }
    }

    const fn headings(self) -> LaterEventHeadings {
        match self {
            Self::Closed => LaterEventHeadings {
                preview: "verification preview: closed",
                created: "recorded verification: closed",
                existing: "verification already recorded: closed",
            },
            Self::Parked => LaterEventHeadings {
                preview: "verification preview: parked",
                created: "recorded verification: parked",
                existing: "verification already recorded: parked",
            },
            Self::Reopened => LaterEventHeadings {
                preview: "verification preview: reopened",
                created: "recorded verification: reopened",
                existing: "verification already recorded: reopened",
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EvidenceKind {
    Commit,
}

impl EvidenceKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Commit => "commit",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum IdentityVerdict {
    SameResponsibility,
    DifferentResponsibilities,
    InsufficientEvidence,
    ExistingAbstractionIsWrong,
}

impl IdentityVerdict {
    const NAMES: &[&str] = &[
        Self::SameResponsibility.label(),
        Self::DifferentResponsibilities.label(),
        Self::InsufficientEvidence.label(),
        Self::ExistingAbstractionIsWrong.label(),
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::SameResponsibility => "same_responsibility",
            Self::DifferentResponsibilities => "different_responsibilities",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::ExistingAbstractionIsWrong => "existing_abstraction_is_wrong",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DecisionAction {
    RetainIntentionalDuplication,
    WaitForMoreEvidence,
    UseExistingDependency,
    ExtractOrDeepenLocally,
    CreateWorkspacePackage,
    CreatePrivateCrossRepositoryPackage,
    PublishPublicPackage,
    CentralizeSchemaSpecificationOrFixtureCorpus,
    ReplaceCopiesWithGeneratedArtifacts,
    ContributeMissingBehaviorUpstream,
    SplitInlineOrNarrowExistingAbstraction,
}

impl DecisionAction {
    const NAMES: &[&str] = &[
        Self::RetainIntentionalDuplication.label(),
        Self::WaitForMoreEvidence.label(),
        Self::UseExistingDependency.label(),
        Self::ExtractOrDeepenLocally.label(),
        Self::CreateWorkspacePackage.label(),
        Self::CreatePrivateCrossRepositoryPackage.label(),
        Self::PublishPublicPackage.label(),
        Self::CentralizeSchemaSpecificationOrFixtureCorpus.label(),
        Self::ReplaceCopiesWithGeneratedArtifacts.label(),
        Self::ContributeMissingBehaviorUpstream.label(),
        Self::SplitInlineOrNarrowExistingAbstraction.label(),
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::RetainIntentionalDuplication => "retain_intentional_duplication",
            Self::WaitForMoreEvidence => "wait_for_more_evidence",
            Self::UseExistingDependency => "use_existing_dependency",
            Self::ExtractOrDeepenLocally => "extract_or_deepen_locally",
            Self::CreateWorkspacePackage => "create_workspace_package",
            Self::CreatePrivateCrossRepositoryPackage => "create_private_cross_repository_package",
            Self::PublishPublicPackage => "publish_public_package",
            Self::CentralizeSchemaSpecificationOrFixtureCorpus => {
                "centralize_schema_specification_or_fixture_corpus"
            }
            Self::ReplaceCopiesWithGeneratedArtifacts => "replace_copies_with_generated_artifacts",
            Self::ContributeMissingBehaviorUpstream => "contribute_missing_behavior_upstream",
            Self::SplitInlineOrNarrowExistingAbstraction => {
                "split_inline_or_narrow_existing_abstraction"
            }
        }
    }

    const fn authorizes_implementation(self) -> bool {
        !matches!(
            self,
            Self::RetainIntentionalDuplication | Self::WaitForMoreEvidence
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct AffectedConsumer {
    repository_id: Uuid,
    consumer: String,
    expectation: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RejectedAlternative {
    alternative: String,
    reason: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ExistingPackageConsidered {
    package: String,
    fit: String,
    reason: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct MigrationExpectation {
    order: i64,
    expectation: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CaseOpenedEvent {
    #[serde(flatten)]
    envelope: event::Envelope,
    case_id: Uuid,
    responsibility: String,
    steward_repository_id: Uuid,
    privacy: Visibility,
    occurrences: Vec<Occurrence>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OccurrenceAppendedEvent {
    #[serde(flatten)]
    envelope: event::Envelope,
    occurrence: Occurrence,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EarlyReviewAuthorizedEvent {
    #[serde(flatten)]
    envelope: event::Envelope,
    reason: String,
    review_appetite: String,
    evidence: Vec<EvidenceReference>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReuseDecisionAcceptedEvent {
    #[serde(flatten)]
    envelope: event::Envelope,
    #[serde(flatten)]
    content: DecisionContent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VerificationRecordedEvent {
    #[serde(flatten)]
    envelope: event::Envelope,
    #[serde(flatten)]
    content: VerificationContent,
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

/// The complete observable result of recording or previewing one later case event.
///
/// The later event types share this carrier under ADR 0013. Which heading it renders and
/// which optional fields it populates stay each event type's decision, as ADR 0010 requires.
/// Opening is not a publication and keeps its own [`OpenOutcome`].
#[derive(Debug)]
pub struct LaterEventOutcome {
    effect: LaterEventEffect,
    headings: LaterEventHeadings,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    state: Option<read::CaseState>,
    privacy: ReportedPrivacy,
    notice: Option<&'static str>,
    event: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LaterEventEffect {
    Preview,
    Created,
    Existing,
}

/// The three headings one later event type prints, selected by what the command did.
#[derive(Clone, Copy, Debug)]
struct LaterEventHeadings {
    preview: &'static str,
    created: &'static str,
    existing: &'static str,
}

impl LaterEventHeadings {
    const fn heading(self, effect: LaterEventEffect) -> &'static str {
        match effect {
            LaterEventEffect::Preview => self.preview,
            LaterEventEffect::Created => self.created,
            LaterEventEffect::Existing => self.existing,
        }
    }
}

/// The words one later event type supplies to the shared publication refusals.
///
/// Every later event type refuses a failed publication in the same shape; only the words
/// naming the event and its retry differ. `case::naming` owns the machine spellings under
/// ADR 0011 — the recorded body string and the file-name slug — so the operator-facing
/// vocabulary lives here beside the headings rather than widening the naming module. ADR 0017
/// moved receipt text to `case::render`; a refusal is not a receipt, and both stay each event
/// type's decision under ADR 0010.
#[derive(Clone, Copy, Debug)]
struct LaterEventRefusals {
    /// Names the recorded event in a condition: `recorded {event} event ...`.
    event: &'static str,
    /// Names the operation in a resolution: `... before retrying the {operation}`.
    operation: &'static str,
    /// Completes an identity-conflict resolution: `or {conflict_resolution} against revision N`.
    conflict_resolution: &'static str,
    /// The command a revision-conflict resolution tells the operator to retry with the
    /// approved proposal, for an operation that carries one. An operation whose retry needs
    /// no proposal supplies none and names no command.
    retry_command: Option<&'static str>,
}

impl LaterEventRefusals {
    /// Renders one failed publication as this event type's refusal.
    ///
    /// A protocol failure already carries its own terminal meaning and passes through.
    fn publication_failure(
        self,
        case_id: Uuid,
        failure: publication::PublicationFailure,
    ) -> TerminalFailure {
        match failure {
            publication::PublicationFailure::Protocol(failure) => failure,
            publication::PublicationFailure::ExistingEvent(failure) => {
                self.existing_event_failure(case_id, failure)
            }
            publication::PublicationFailure::RevisionConflict {
                expected_revision,
                current_revision,
            } => TerminalFailure::refusal(
                format!(
                    "expected revision {expected_revision} does not match case `{case_id}` current revision {current_revision}"
                ),
                self.retry_command.map_or_else(
                    || {
                        format!(
                            "run `case show {case_id}` and retry with `--expected-revision {current_revision}`"
                        )
                    },
                    |command| {
                        format!(
                            "run `case show {case_id}` and retry `{command} {case_id}` with `--expected-revision {current_revision}` and the approved proposal"
                        )
                    },
                ),
            ),
        }
    }

    /// Renders an event already recorded at the target sequence as this event type's refusal.
    fn existing_event_failure(
        self,
        case_id: Uuid,
        failure: publication::ExistingEventFailure,
    ) -> TerminalFailure {
        let Self {
            event,
            operation,
            conflict_resolution,
            ..
        } = self;
        match failure {
            publication::ExistingEventFailure::Unreadable { path, error } => {
                TerminalFailure::refusal(
                    format!(
                        "recorded {event} event `{}` cannot be read: {error}",
                        path.display()
                    ),
                    format!("restore the recorded event before retrying the {operation}"),
                )
            }
            publication::ExistingEventFailure::Invalid { path, error } => TerminalFailure::refusal(
                format!(
                    "recorded {event} event `{}` is invalid: {error}",
                    path.display()
                ),
                format!("restore the supported recorded event before retrying the {operation}"),
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
                        "inspect sequence {recorded_sequence}; retry its recorded identity if it is the intended {operation}, or {conflict_resolution} against revision {current_revision}"
                    ),
                )
            }
            publication::ExistingEventFailure::ContentDrift { recorded_event_id } => {
                TerminalFailure::refusal(
                    format!(
                        "{event} event identity `{recorded_event_id}` is already recorded with different content"
                    ),
                    format!("restore the exact previewed {event} event before retrying"),
                )
            }
        }
    }
}

const APPEND_HEADINGS: LaterEventHeadings = LaterEventHeadings {
    preview: "case append preview",
    created: "appended occurrence",
    existing: "occurrence already recorded",
};

/// Appending retries against a fresh revision rather than a re-approved proposal, so its
/// revision-conflict resolution names no command and asks for a distinct occurrence.
const APPEND_REFUSALS: LaterEventRefusals = LaterEventRefusals {
    event: "append",
    operation: "append",
    conflict_resolution: "prepare a distinct occurrence",
    retry_command: None,
};

const EARLY_REVIEW_HEADINGS: LaterEventHeadings = LaterEventHeadings {
    preview: "early-review override preview",
    created: "authorized early review",
    existing: "early review already authorized",
};

const EARLY_REVIEW_REFUSALS: LaterEventRefusals = LaterEventRefusals {
    event: "early-review",
    operation: "early-review override",
    conflict_resolution: "prepare a new operation",
    retry_command: Some("case override"),
};

const DECISION_HEADINGS: LaterEventHeadings = LaterEventHeadings {
    preview: "reuse decision preview",
    created: "accepted reuse decision",
    existing: "reuse decision already recorded",
};

const DECISION_REFUSALS: LaterEventRefusals = LaterEventRefusals {
    event: "reuse decision",
    operation: "reuse decision",
    conflict_resolution: "prepare a new operation",
    retry_command: Some("case decide"),
};

const VERIFICATION_REFUSALS: LaterEventRefusals = LaterEventRefusals {
    event: "verification",
    operation: "verification",
    conflict_resolution: "prepare a new operation",
    retry_command: Some("case verify"),
};

/// The complete case privacy a receipt reports, or why it could not be derived.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReportedPrivacy {
    /// Derived from the steward and the resolved participants.
    Derived(Visibility),
    /// No usable portfolio root selection is configured or supplied.
    PortfolioUnconfigured,
    /// Roots are selected, but a recorded participant does not resolve to one enrolled repository.
    ParticipantsUnresolved,
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
    location: &portfolio::PortfolioLocation,
    recorded_at: RecordedInstant,
    preview: bool,
) -> Result<OpenOutcome, TerminalFailure> {
    let repository_root = find_repository_root(working_directory)?;
    let mut steward = read_steward(&repository_root, OPEN_MARKER_RESOLUTION)?;
    let _marker_lock = if preview {
        None
    } else {
        let marker_lock = crate::lock_repository_marker(&repository_root)?;
        steward = read_steward(&repository_root, OPEN_MARKER_RESOLUTION)?;
        Some(marker_lock)
    };
    let proposal = read_proposal(proposal_path)?;
    let relative_case_directory = naming::case_directory(proposal.case_id);
    let event_path = case_event_path(
        &relative_case_directory,
        OPENING_SEQUENCE,
        EventType::CaseOpened,
    )?;
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
    let roots = portfolio::selected_roots(location)?;
    let participants = resolve_participants(&roots, &proposal.occurrences)?;
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
    let event = event_bytes(&proposal, &steward, privacy, recorded_at)?;
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
    location: &portfolio::PortfolioLocation,
    recorded_at: RecordedInstant,
    preview: bool,
) -> Result<LaterEventOutcome, TerminalFailure> {
    let located = locate_later_event_case(
        working_directory,
        case_id,
        expected_revision,
        APPEND_MARKER_RESOLUTION,
        APPEND_UNSTEWARDED_RESOLUTION,
    )?;
    let case_id = located.case_id;
    let proposal = read_append_proposal(proposal_path)?;
    validate_prepared_append_sequence(&proposal, expected_revision, located.sequence)?;
    let (relative_event_path, absolute_event_path) =
        later_event_paths(&located, EventType::OccurrenceAppended)?;
    let event = append_event_bytes(&proposal, located.sequence, recorded_at)?;
    let eligibility = |case: &read::CaseRecord, ()| {
        validate_new_append(case, &proposal, &located.steward, location)
    };
    LaterEventExecution {
        located: &located,
        relative_event_path,
        absolute_event_path,
        event_id: proposal.prepared.as_ref().map(|prepared| prepared.event_id),
        event,
        preview,
        refusals: APPEND_REFUSALS,
    }
    .execute(
        |_| Ok(()),
        eligibility,
        |effect, case, privacy, event_path, event| {
            append_outcome(
                effect,
                case_id,
                event_path,
                located.sequence,
                case.state_after_appending_occurrence(),
                ReportedPrivacy::Derived(privacy),
                event,
            )
        },
        |case, existing, event_path| {
            append_retry_outcome(
                case_id,
                event_path,
                case,
                existing,
                &located.steward,
                location,
            )
        },
    )
}

/// Builds one occurrence-append receipt; the append event reports readiness and no notice.
fn append_outcome(
    effect: LaterEventEffect,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    state: read::CaseState,
    privacy: ReportedPrivacy,
    event: String,
) -> LaterEventOutcome {
    LaterEventOutcome {
        effect,
        headings: APPEND_HEADINGS,
        case_id,
        event_path,
        revision,
        state: Some(state),
        privacy,
        notice: None,
        event,
    }
}

fn append_retry_outcome(
    case_id: Uuid,
    event_path: PathBuf,
    case: &read::CaseRecord,
    event: publication::ExistingEvent,
    steward: &marker::Marker,
    location: &portfolio::PortfolioLocation,
) -> LaterEventOutcome {
    append_outcome(
        LaterEventEffect::Existing,
        case_id,
        event_path,
        event.sequence,
        case.state(),
        reported_privacy(case, steward, location),
        event.bytes,
    )
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
    location: &portfolio::PortfolioLocation,
    recorded_at: RecordedInstant,
    preview: bool,
) -> Result<LaterEventOutcome, TerminalFailure> {
    let located = locate_later_event_case(
        working_directory,
        case_id,
        expected_revision,
        EARLY_REVIEW_MARKER_RESOLUTION,
        EARLY_REVIEW_UNSTEWARDED_RESOLUTION,
    )?;
    let case_id = located.case_id;
    let proposal = read_early_review_proposal(proposal_path)?;
    validate_prepared_early_review_sequence(&proposal, expected_revision, located.sequence)?;
    let (relative_event_path, absolute_event_path) =
        later_event_paths(&located, EventType::EarlyReviewAuthorized)?;
    let event = early_review_event_bytes(&proposal, located.sequence, recorded_at)?;
    let case_privacy = |case: &read::CaseRecord| -> Result<Visibility, TerminalFailure> {
        let roots = portfolio::selected_roots(location)?;
        let privacy = derive_complete_case_privacy(case, &located.steward, &roots)?;
        validate_early_review_privacy(case, &located.steward, privacy)?;
        Ok(privacy)
    };
    let eligibility = |case: &read::CaseRecord, privacy| {
        validate_new_early_review(case)?;
        Ok(privacy)
    };
    LaterEventExecution {
        located: &located,
        relative_event_path,
        absolute_event_path,
        event_id: proposal.prepared.as_ref().map(|prepared| prepared.event_id),
        event,
        preview,
        refusals: EARLY_REVIEW_REFUSALS,
    }
    .execute(
        case_privacy,
        eligibility,
        |effect, _, privacy, event_path, event| {
            early_review_outcome(
                effect,
                case_id,
                event_path,
                located.sequence,
                read::CaseState::ReviewReadyByEarlyReviewOverride,
                ReportedPrivacy::Derived(privacy),
                event,
            )
        },
        |case, existing, event_path| {
            early_review_retry_outcome(
                case_id,
                event_path,
                case,
                existing,
                &located.steward,
                location,
            )
        },
    )
}

/// Records or previews the exact reuse decision accepted for a review-ready case.
///
/// # Errors
///
/// Returns a classified failure when the steward, case, proposal, revision,
/// privacy, or affected-consumer set cannot be read or validated safely.
pub fn decide(
    working_directory: &Path,
    case_id: &str,
    expected_revision: i64,
    proposal_path: &Path,
    location: &portfolio::PortfolioLocation,
    recorded_at: RecordedInstant,
    preview: bool,
) -> Result<LaterEventOutcome, TerminalFailure> {
    let located = locate_later_event_case(
        working_directory,
        case_id,
        expected_revision,
        DECISION_MARKER_RESOLUTION,
        DECISION_UNSTEWARDED_RESOLUTION,
    )?;
    let case_id = located.case_id;
    let proposal = read_decision_proposal(proposal_path)?;
    validate_prepared_decision_sequence(&proposal, expected_revision, located.sequence)?;
    let (relative_event_path, absolute_event_path) =
        later_event_paths(&located, EventType::ReuseDecisionAccepted)?;
    let event = decision_event_bytes(&proposal, located.sequence, recorded_at)?;
    let eligibility = |case: &read::CaseRecord, ()| -> Result<Visibility, TerminalFailure> {
        validate_new_decision(case, &proposal)?;
        let roots = portfolio::selected_roots(location)?;
        let privacy = derive_complete_case_privacy(case, &located.steward, &roots)?;
        validate_decision_privacy(case, &located.steward, privacy)?;
        Ok(privacy)
    };
    LaterEventExecution {
        located: &located,
        relative_event_path,
        absolute_event_path,
        event_id: proposal.prepared.as_ref().map(|prepared| prepared.event_id),
        event,
        preview,
        refusals: DECISION_REFUSALS,
    }
    .execute(
        |_| Ok(()),
        eligibility,
        |effect, _, privacy, event_path, event| {
            decision_outcome(
                effect,
                case_id,
                event_path,
                located.sequence,
                ReportedPrivacy::Derived(privacy),
                DecisionReceiptFields {
                    state: read::CaseState::AwaitingVerification,
                    action: proposal.content.action,
                },
                event,
            )
        },
        |case, existing, event_path| {
            decision_retry_outcome(
                case_id,
                event_path,
                case,
                existing,
                &located.steward,
                location,
                proposal.content.action,
            )
        },
    )
}

/// Records or previews verification of the standing accepted reuse decision.
///
/// # Errors
///
/// Returns a classified failure when the steward, case, proposal, revision,
/// privacy, or decision-supplied verification question set cannot be read or
/// validated safely.
pub fn verify(
    working_directory: &Path,
    case_id: &str,
    expected_revision: i64,
    proposal_path: &Path,
    location: &portfolio::PortfolioLocation,
    recorded_at: RecordedInstant,
    preview: bool,
) -> Result<LaterEventOutcome, TerminalFailure> {
    let located = locate_later_event_case(
        working_directory,
        case_id,
        expected_revision,
        VERIFICATION_MARKER_RESOLUTION,
        VERIFICATION_UNSTEWARDED_RESOLUTION,
    )?;
    let case_id = located.case_id;
    let proposal = read_verification_proposal(proposal_path)?;
    validate_prepared_verification_sequence(&proposal, expected_revision, located.sequence)?;
    let (relative_event_path, absolute_event_path) =
        later_event_paths(&located, EventType::VerificationRecorded)?;
    let event = verification_event_bytes(&proposal, located.sequence, recorded_at)?;
    let eligibility = |case: &read::CaseRecord, ()| {
        validate_verification_eligibility(case, &proposal, &located.steward, location)
    };
    LaterEventExecution {
        located: &located,
        relative_event_path,
        absolute_event_path,
        event_id: proposal.prepared.as_ref().map(|prepared| prepared.event_id),
        event,
        preview,
        refusals: VERIFICATION_REFUSALS,
    }
    .execute(
        |_| Ok(()),
        eligibility,
        |effect, _, privacy, event_path, event| {
            fresh_verification_outcome(
                effect,
                case_id,
                event_path,
                located.sequence,
                ReportedPrivacy::Derived(privacy),
                proposal.content.disposition,
                event,
            )
        },
        |case, existing, event_path| {
            verification_retry_outcome(
                case_id,
                event_path,
                case,
                existing,
                &located.steward,
                location,
                proposal.content.disposition,
            )
        },
    )
}

fn fresh_verification_outcome(
    effect: LaterEventEffect,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    privacy: ReportedPrivacy,
    disposition: VerificationDisposition,
    event: String,
) -> LaterEventOutcome {
    LaterEventOutcome {
        effect,
        headings: disposition.headings(),
        case_id,
        event_path,
        revision,
        state: Some(disposition.state()),
        privacy,
        notice: Some(disposition.notice()),
        event,
    }
}

fn verification_retry_outcome(
    case_id: Uuid,
    event_path: PathBuf,
    case: &read::CaseRecord,
    event: publication::ExistingEvent,
    steward: &marker::Marker,
    location: &portfolio::PortfolioLocation,
    disposition: VerificationDisposition,
) -> LaterEventOutcome {
    LaterEventOutcome {
        effect: LaterEventEffect::Existing,
        headings: disposition.headings(),
        case_id,
        event_path,
        revision: event.sequence,
        state: Some(case.state()),
        privacy: reported_privacy(case, steward, location),
        notice: Some(disposition.notice()),
        event: event.bytes,
    }
}

fn validate_verification_eligibility(
    case: &read::CaseRecord,
    proposal: &VerificationProposal,
    steward: &marker::Marker,
    location: &portfolio::PortfolioLocation,
) -> Result<Visibility, TerminalFailure> {
    validate_new_verification(case, proposal)?;
    let roots = portfolio::selected_roots(location)?;
    let privacy = derive_complete_case_privacy(case, steward, &roots)?;
    validate_verification_privacy(case, steward, privacy)?;
    Ok(privacy)
}

fn validate_new_verification(
    case: &read::CaseRecord,
    proposal: &VerificationProposal,
) -> Result<(), TerminalFailure> {
    validate_case_accepts_later_event(case)?;
    let Some(decision) = case.decision.as_ref() else {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{}` has no accepted reuse decision; current state is `{}`",
                case.case_id,
                case.state().label()
            ),
            "record an accepted reuse decision before retrying verification",
        ));
    };
    validate_verification_against_decision(case.case_id, &proposal.content, &decision.content)
}

fn validate_case_accepts_later_event(case: &read::CaseRecord) -> Result<(), TerminalFailure> {
    if case.state() == read::CaseState::Closed {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{}` is closed and terminal in version 0.1",
                case.case_id
            ),
            "leave the closed case unchanged; later pressure requires a separately accepted capability",
        ));
    }
    Ok(())
}

fn validate_verification_privacy(
    case: &read::CaseRecord,
    steward: &marker::Marker,
    privacy: Visibility,
) -> Result<(), TerminalFailure> {
    if steward.visibility() == Visibility::Public && privacy == Visibility::Private {
        return Err(TerminalFailure::refusal(
            format!(
                "public steward `{}` cannot record verification for private case `{}`",
                steward.repository_id(),
                case.case_id
            ),
            "run `set-visibility --visibility private` in the steward repository, then preview verification again",
        ));
    }
    Ok(())
}

fn validate_prepared_verification_sequence(
    proposal: &VerificationProposal,
    expected_revision: i64,
    sequence: i64,
) -> Result<(), TerminalFailure> {
    if let Some(prepared) = &proposal.prepared
        && prepared.sequence != sequence
    {
        return Err(TerminalFailure::refusal(
            format!(
                "prepared verification event records sequence {}, but expected revision {expected_revision} requires sequence {sequence}",
                prepared.sequence
            ),
            "preview verification again against the current expected revision",
        ));
    }
    Ok(())
}

/// The steward, case, and expected sequence a later-event command works against.
struct LocatedCase {
    case_id: Uuid,
    publication: publication::Publication,
    sequence: i64,
    repository_root: PathBuf,
    steward: marker::Marker,
    relative_case_directory: PathBuf,
    case: read::CaseRecord,
    /// The resolution this command's unstewarded-case refusal names, carried so the re-read
    /// under the publication lock cannot answer that refusal differently from this location.
    unstewarded_resolution: &'static str,
}

/// Locates the enrolled steward and the recorded case one later event would extend.
///
/// Every later-event command performs these steps in this order; only the resolution for a case
/// this repository does not steward varies, because it names the command to retry and the case
/// state that command requires.
fn locate_later_event_case(
    working_directory: &Path,
    case_id: &str,
    expected_revision: i64,
    marker_resolution: &'static str,
    unstewarded_resolution: &'static str,
) -> Result<LocatedCase, TerminalFailure> {
    let case_id = parse_case_id(case_id)?;
    let publication = publication::Publication::new(expected_revision)?;
    let sequence = publication.sequence();
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root, marker_resolution)?;
    let relative_case_directory = naming::case_directory(case_id);
    validate_case_storage_path(&repository_root, &relative_case_directory)?;
    let case = read::read_case_for(
        &repository_root,
        &relative_case_directory,
        case_id,
        steward.repository_id(),
        unstewarded_resolution,
    )?;
    Ok(LocatedCase {
        case_id,
        publication,
        sequence,
        repository_root,
        steward,
        relative_case_directory,
        case,
        unstewarded_resolution,
    })
}

/// Builds and validates the repository-relative and absolute paths a typed later event occupies.
fn later_event_paths(
    located: &LocatedCase,
    event_type: EventType,
) -> Result<(PathBuf, PathBuf), TerminalFailure> {
    let relative_event_path = case_event_path(
        &located.relative_case_directory,
        located.sequence,
        event_type,
    )?;
    validate_case_storage_path(&located.repository_root, &relative_event_path)?;
    let absolute_event_path = located.repository_root.join(&relative_event_path);
    Ok((relative_event_path, absolute_event_path))
}

/// Executes the one preview-or-publish protocol shared by every later case event.
///
/// Event-specific proposal parsing, eligibility, privacy, and receipt construction remain at the
/// command boundary. This executor owns only the publication branch shape accepted by ADR 0013:
/// exact prepared-event retries are recognized before eligibility, previews stay write-free, and
/// publication re-reads the case while holding the opening-event lock.
struct LaterEventExecution<'a> {
    located: &'a LocatedCase,
    relative_event_path: PathBuf,
    absolute_event_path: PathBuf,
    event_id: Option<Uuid>,
    event: String,
    preview: bool,
    refusals: LaterEventRefusals,
}

impl LaterEventExecution<'_> {
    fn execute<B, V>(
        self,
        before_revision: impl FnOnce(&read::CaseRecord) -> Result<B, TerminalFailure>,
        after_revision: impl FnOnce(&read::CaseRecord, B) -> Result<V, TerminalFailure>,
        fresh_outcome: impl FnOnce(
            LaterEventEffect,
            &read::CaseRecord,
            V,
            PathBuf,
            String,
        ) -> LaterEventOutcome,
        retry_outcome: impl FnOnce(
            &read::CaseRecord,
            publication::ExistingEvent,
            PathBuf,
        ) -> LaterEventOutcome,
    ) -> Result<LaterEventOutcome, TerminalFailure> {
        let Self {
            located,
            relative_event_path,
            absolute_event_path,
            event_id,
            event,
            preview,
            refusals,
        } = self;
        let case_id = located.case_id;

        if preview {
            let checked = located
                .publication
                .check(
                    &located.case,
                    &absolute_event_path,
                    publication::PreparedEvent {
                        event_id,
                        bytes: &event,
                    },
                    before_revision,
                    after_revision,
                )
                .map_err(|failure| refusals.publication_failure(case_id, failure))?;
            return Ok(match checked {
                publication::Checked::Existing(existing) => {
                    retry_outcome(&located.case, existing, relative_event_path)
                }
                publication::Checked::Fresh(validation) => fresh_outcome(
                    LaterEventEffect::Preview,
                    &located.case,
                    validation,
                    relative_event_path,
                    event,
                ),
            });
        }

        let published = located
            .publication
            .publish(
                publication::PublicationTarget {
                    repository_root: &located.repository_root,
                    relative_case_directory: &located.relative_case_directory,
                    relative_event_path: &relative_event_path,
                },
                publication::PreparedEvent {
                    event_id,
                    bytes: &event,
                },
                || {
                    read::read_case_for(
                        &located.repository_root,
                        &located.relative_case_directory,
                        case_id,
                        located.steward.repository_id(),
                        located.unstewarded_resolution,
                    )
                },
                before_revision,
                after_revision,
            )
            .map_err(|failure| refusals.publication_failure(case_id, failure))?;
        Ok(match published {
            publication::PublicationOutcome::Created { case, validation } => fresh_outcome(
                LaterEventEffect::Created,
                &case,
                validation,
                relative_event_path,
                event,
            ),
            publication::PublicationOutcome::Existing { case, event } => {
                retry_outcome(&case, event, relative_event_path)
            }
        })
    }
}

#[derive(Clone, Copy)]
struct DecisionReceiptFields {
    state: read::CaseState,
    action: DecisionAction,
}

/// Builds one reuse-decision receipt.
///
/// Whether the notice authorizes implementation is the accepted action's decision, not the
/// receipt's. A fresh receipt projects `awaiting-verification`; an exact retry reports the case's
/// live derived state under ADR 0010.
fn decision_outcome(
    effect: LaterEventEffect,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    privacy: ReportedPrivacy,
    fields: DecisionReceiptFields,
    event: String,
) -> LaterEventOutcome {
    LaterEventOutcome {
        effect,
        headings: DECISION_HEADINGS,
        case_id,
        event_path,
        revision,
        state: Some(fields.state),
        privacy,
        notice: Some(decision_notice(fields.action)),
        event,
    }
}

fn decision_retry_outcome(
    case_id: Uuid,
    event_path: PathBuf,
    case: &read::CaseRecord,
    event: publication::ExistingEvent,
    steward: &marker::Marker,
    location: &portfolio::PortfolioLocation,
    action: DecisionAction,
) -> LaterEventOutcome {
    decision_outcome(
        LaterEventEffect::Existing,
        case_id,
        event_path,
        event.sequence,
        reported_privacy(case, steward, location),
        DecisionReceiptFields {
            state: case.state(),
            action,
        },
        event.bytes,
    )
}

const fn decision_notice(action: DecisionAction) -> &'static str {
    if action.authorizes_implementation() {
        IMPLEMENTATION_NOTICE
    } else {
        NO_IMPLEMENTATION_NOTICE
    }
}

fn validate_new_decision(
    case: &read::CaseRecord,
    proposal: &DecisionProposal,
) -> Result<(), TerminalFailure> {
    validate_case_accepts_later_event(case)?;
    if case.has_decision() {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{}` already records an accepted reuse decision",
                case.case_id
            ),
            "leave the recorded decision unchanged; superseding it requires the separately accepted reopen capability",
        ));
    }
    if !case.state().authorizes_review() {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{}` is watching and cannot record an accepted reuse decision",
                case.case_id
            ),
            "append a third independent occurrence or record a human-authorized early-review override before retrying the decision",
        ));
    }
    if let Some(affected) =
        unrecorded_affected_consumer(&case.occurrences, &proposal.content.affected_consumers)
    {
        return Err(TerminalFailure::refusal(
            format!(
                "decision affected consumer `{}` in participant `{}` is not recorded by case `{}`",
                affected.consumer.trim(),
                affected.repository_id,
                case.case_id
            ),
            "name only a participant repository and consumer pair already evidenced by a recorded occurrence",
        ));
    }
    Ok(())
}

fn unrecorded_affected_consumer<'a>(
    occurrences: &[Occurrence],
    affected_consumers: &'a [AffectedConsumer],
) -> Option<&'a AffectedConsumer> {
    affected_consumers.iter().find(|affected| {
        !occurrences.iter().any(|occurrence| {
            occurrence.repository_id == affected.repository_id
                && occurrence.consumer.trim() == affected.consumer.trim()
        })
    })
}

fn validate_recorded_decision_participants(
    case_id: Uuid,
    occurrences: &[Occurrence],
    decision: &ReuseDecisionAcceptedEvent,
) -> Result<(), TerminalFailure> {
    let Some(affected) =
        unrecorded_affected_consumer(occurrences, &decision.content.affected_consumers)
    else {
        return Ok(());
    };
    Err(TerminalFailure::refusal(
        format!(
            "decision affected consumer `{}` in participant `{}` is not recorded before sequence {} in case `{case_id}`",
            affected.consumer.trim(),
            affected.repository_id,
            decision.envelope.sequence
        ),
        "restore the accepted decision so every affected repository and consumer pair names a participant recorded by an earlier event",
    ))
}

fn validate_decision_privacy(
    case: &read::CaseRecord,
    steward: &marker::Marker,
    privacy: Visibility,
) -> Result<(), TerminalFailure> {
    if steward.visibility() == Visibility::Public && privacy == Visibility::Private {
        return Err(TerminalFailure::refusal(
            format!(
                "public steward `{}` cannot record a reuse decision for private case `{}`",
                steward.repository_id(),
                case.case_id
            ),
            "run `set-visibility --visibility private` in the steward repository, then preview the reuse decision again",
        ));
    }
    Ok(())
}

fn validate_prepared_decision_sequence(
    proposal: &DecisionProposal,
    expected_revision: i64,
    sequence: i64,
) -> Result<(), TerminalFailure> {
    if let Some(prepared) = &proposal.prepared
        && prepared.sequence != sequence
    {
        return Err(TerminalFailure::refusal(
            format!(
                "prepared reuse decision event records sequence {}, but expected revision {expected_revision} requires sequence {sequence}",
                prepared.sequence
            ),
            "preview the reuse decision again against the current expected revision",
        ));
    }
    Ok(())
}

/// Builds one early-review receipt.
///
/// A fresh receipt projects early-review readiness rather than deriving it from the occurrence
/// count. An exact retry receives the case's live derived state under ADR 0010. The override
/// authorizes no implementation, so neither path reports a notice.
fn early_review_outcome(
    effect: LaterEventEffect,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    state: read::CaseState,
    privacy: ReportedPrivacy,
    event: String,
) -> LaterEventOutcome {
    LaterEventOutcome {
        effect,
        headings: EARLY_REVIEW_HEADINGS,
        case_id,
        event_path,
        revision,
        state: Some(state),
        privacy,
        notice: None,
        event,
    }
}

fn early_review_retry_outcome(
    case_id: Uuid,
    event_path: PathBuf,
    case: &read::CaseRecord,
    event: publication::ExistingEvent,
    steward: &marker::Marker,
    location: &portfolio::PortfolioLocation,
) -> LaterEventOutcome {
    early_review_outcome(
        LaterEventEffect::Existing,
        case_id,
        event_path,
        event.sequence,
        case.state(),
        reported_privacy(case, steward, location),
        event.bytes,
    )
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
    validate_case_accepts_later_event(case)?;
    if case.has_decision() {
        return Err(TerminalFailure::refusal(
            format!(
                "case `{}` already records an accepted reuse decision",
                case.case_id
            ),
            "leave the case awaiting verification; an early-review override cannot change a decided case",
        ));
    }
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
    location: &portfolio::PortfolioLocation,
) -> Result<Visibility, TerminalFailure> {
    validate_case_accepts_later_event(case)?;
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
    let roots = portfolio::selected_roots(location)?;
    let participant_visibilities = resolve_participants(&roots, &occurrences)?;
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
    roots: &[PathBuf],
) -> Result<Visibility, TerminalFailure> {
    let participant_visibilities = resolve_participants(roots, &case.occurrences)?;
    Ok(complete_case_privacy(
        case,
        steward.visibility(),
        participant_visibilities.values().copied(),
    ))
}

fn complete_case_privacy(
    case: &read::CaseRecord,
    steward_visibility: Visibility,
    participant_visibilities: impl IntoIterator<Item = Visibility>,
) -> Visibility {
    if case.privacy == Visibility::Private
        || steward_visibility == Visibility::Private
        || participant_visibilities
            .into_iter()
            .any(|visibility| visibility == Visibility::Private)
    {
        Visibility::Private
    } else {
        Visibility::Public
    }
}

/// Reports complete case privacy for a no-write result, or why it cannot be derived.
///
/// A no-write result reports underivable privacy rather than refusing. Every later event retry and
/// the implementation-brief projection use the same consequence.
fn reported_privacy(
    case: &read::CaseRecord,
    steward: &marker::Marker,
    location: &portfolio::PortfolioLocation,
) -> ReportedPrivacy {
    // The selection is read once and carried into the derivation. Re-selecting
    // there would read the same user-local configuration a second time and
    // could only differ from this one by a concurrent edit.
    let Ok(Some(roots)) = portfolio::selected_roots_if_configured(location) else {
        return ReportedPrivacy::PortfolioUnconfigured;
    };
    derive_complete_case_privacy(case, steward, &roots).map_or(
        ReportedPrivacy::ParticipantsUnresolved,
        ReportedPrivacy::Derived,
    )
}

fn append_event_bytes(
    proposal: &AppendProposal,
    sequence: i64,
    recorded_at: RecordedInstant,
) -> Result<String, TerminalFailure> {
    if let Some(prepared) = &proposal.prepared {
        return Ok(prepared.bytes.clone());
    }
    let event = OccurrenceAppendedEvent {
        envelope: event::Envelope::new(sequence, EventType::OccurrenceAppended, recorded_at),
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
    recorded_at: RecordedInstant,
) -> Result<String, TerminalFailure> {
    if let Some(prepared) = &proposal.prepared {
        return Ok(prepared.bytes.clone());
    }
    let event = EarlyReviewAuthorizedEvent {
        envelope: event::Envelope::new(sequence, EventType::EarlyReviewAuthorized, recorded_at),
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

fn decision_event_bytes(
    proposal: &DecisionProposal,
    sequence: i64,
    recorded_at: RecordedInstant,
) -> Result<String, TerminalFailure> {
    if let Some(prepared) = &proposal.prepared {
        return Ok(prepared.bytes.clone());
    }
    let event = ReuseDecisionAcceptedEvent {
        envelope: event::Envelope::new(sequence, EventType::ReuseDecisionAccepted, recorded_at),
        content: proposal.content.clone(),
    };
    toml::to_string(&event).map_err(|error| {
        TerminalFailure::unsafe_failure(format!(
            "accepted reuse decision event could not be encoded: {error}"
        ))
    })
}

fn verification_event_bytes(
    proposal: &VerificationProposal,
    sequence: i64,
    recorded_at: RecordedInstant,
) -> Result<String, TerminalFailure> {
    if let Some(prepared) = &proposal.prepared {
        return Ok(prepared.bytes.clone());
    }
    let event = VerificationRecordedEvent {
        envelope: event::Envelope::new(sequence, EventType::VerificationRecorded, recorded_at),
        content: proposal.content.clone(),
    };
    toml::to_string(&event).map_err(|error| {
        TerminalFailure::unsafe_failure(format!("verification event could not be encoded: {error}"))
    })
}

fn event_bytes(
    proposal: &OpenProposal,
    steward: &marker::Marker,
    privacy: Visibility,
    recorded_at: RecordedInstant,
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
        envelope: event::Envelope::new(OPENING_SEQUENCE, EventType::CaseOpened, recorded_at),
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
            if !naming::is_staged_temporary(&entry.file_name(), EventPosition::Opening) {
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
        if naming::is_staged_temporary(&entry.file_name(), EventPosition::Opening) {
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

fn case_event_path(
    relative_case_directory: &Path,
    sequence: i64,
    event_type: EventType,
) -> Result<PathBuf, TerminalFailure> {
    let file_name = EventFileName::new(sequence, event_type).ok_or_else(|| {
        TerminalFailure::unsafe_failure(format!(
            "case event `{}` at sequence {sequence} has no accepted file name",
            event_type.body_name()
        ))
    })?;
    Ok(relative_case_directory.join(file_name.to_string()))
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
    let matches = recorded.envelope.schema_version == CASE_SCHEMA_VERSION
        && recorded.envelope.sequence == OPENING_SEQUENCE
        && recorded.envelope.event_type == EventType::CaseOpened
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

/// Reads the enrolled steward whose cases this command works against.
///
/// Under ADR 0018 a present-but-unusable marker is classified by
/// `crate::read_supported_marker`, which names which fault it is, the marker
/// path and its cause. `marker_resolution` is this command's sentence for
/// fixing it, because it names the command that ran.
fn read_steward(
    repository_root: &Path,
    marker_resolution: &str,
) -> Result<marker::Marker, TerminalFailure> {
    crate::read_supported_marker(repository_root, marker_resolution)?.ok_or_else(|| {
        TerminalFailure::refusal(
            format!(
                "repository is not enrolled because `{}` does not exist",
                repository_root.join(crate::MARKER_FILE).display()
            ),
            "run `enroll` before opening a case",
        )
    })
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

fn validate_decision_content(content: &DecisionContent) -> Result<(), TerminalFailure> {
    validate_common_decision_content(content)?;
    if content.action.authorizes_implementation() {
        validate_change_decision_content(content)
    } else {
        validate_no_change_decision_content(content)
    }
}

fn validate_common_decision_content(content: &DecisionContent) -> Result<(), TerminalFailure> {
    require_nonempty("accepted_scope", &content.accepted_scope)?;
    require_nonempty_string_list("non_responsibilities", &content.non_responsibilities)?;
    if content.affected_consumers.is_empty() {
        return Err(missing_required_decision_item("affected_consumers"));
    }
    let mut affected_consumers = BTreeSet::new();
    for (index, affected) in content.affected_consumers.iter().enumerate() {
        require_nonempty(
            &format!("affected_consumers[{}].consumer", index + 1),
            &affected.consumer,
        )?;
        require_nonempty(
            &format!("affected_consumers[{}].expectation", index + 1),
            &affected.expectation,
        )?;
        if !affected_consumers.insert((affected.repository_id, affected.consumer.trim())) {
            return Err(TerminalFailure::refusal(
                format!(
                    "affected_consumers records participant `{}` and consumer `{}` more than once",
                    affected.repository_id,
                    affected.consumer.trim()
                ),
                "record each affected participant repository and consumer pair exactly once",
            ));
        }
    }
    if content.alternatives_rejected.is_empty() {
        return Err(missing_required_decision_item("alternatives_rejected"));
    }
    for (index, alternative) in content.alternatives_rejected.iter().enumerate() {
        require_nonempty(
            &format!("alternatives_rejected[{}].alternative", index + 1),
            &alternative.alternative,
        )?;
        require_nonempty(
            &format!("alternatives_rejected[{}].reason", index + 1),
            &alternative.reason,
        )?;
    }
    require_nonempty(
        "compatibility_consequences",
        &content.compatibility_consequences,
    )?;
    require_nonempty_string_list("verification_conditions", &content.verification_conditions)?;
    Ok(())
}

fn validate_change_decision_content(content: &DecisionContent) -> Result<(), TerminalFailure> {
    let invariant_contract = content
        .invariant_contract
        .as_deref()
        .ok_or_else(|| missing_change_decision_item("invariant_contract"))?;
    require_nonempty("invariant_contract", invariant_contract)?;
    require_nonempty_change_list(
        "existing_packages_considered",
        content.existing_packages_considered.as_deref(),
    )?;
    require_nonempty_change_list(
        "required_consumer_level_tests",
        content.required_consumer_level_tests.as_deref(),
    )?;
    require_nonempty_change_list(
        "migration_expectations",
        content.migration_expectations.as_deref(),
    )?;
    let rollback = content
        .rollback_or_resplitting_path
        .as_deref()
        .ok_or_else(|| missing_change_decision_item("rollback_or_resplitting_path"))?;
    require_nonempty("rollback_or_resplitting_path", rollback)?;

    let packages = content
        .existing_packages_considered
        .as_deref()
        .expect("change-action package list was required above");
    for (index, package) in packages.iter().enumerate() {
        require_nonempty(
            &format!("existing_packages_considered[{}].package", index + 1),
            &package.package,
        )?;
        require_nonempty(
            &format!("existing_packages_considered[{}].fit", index + 1),
            &package.fit,
        )?;
        require_nonempty(
            &format!("existing_packages_considered[{}].reason", index + 1),
            &package.reason,
        )?;
    }
    require_nonempty_string_list(
        "required_consumer_level_tests",
        content
            .required_consumer_level_tests
            .as_deref()
            .expect("change-action test list was required above"),
    )?;
    validate_migration_expectations(
        content
            .migration_expectations
            .as_deref()
            .expect("change-action migration list was required above"),
    )
}

fn validate_migration_expectations(
    migrations: &[MigrationExpectation],
) -> Result<(), TerminalFailure> {
    let mut observed_orders = BTreeSet::new();
    for (index, migration) in migrations.iter().enumerate() {
        require_nonempty(
            &format!("migration_expectations[{}].expectation", index + 1),
            &migration.expectation,
        )?;
        if migration.order < 1 || !observed_orders.insert(migration.order) {
            return Err(TerminalFailure::refusal(
                format!(
                    "migration_expectations[{}].order `{}` is not a unique positive order",
                    index + 1,
                    migration.order
                ),
                "number migration expectations once each in contiguous order beginning at 1",
            ));
        }
    }
    if observed_orders
        .iter()
        .copied()
        .ne(1..=i64::try_from(migrations.len()).expect("migration count fits in i64"))
    {
        return Err(TerminalFailure::refusal(
            "migration_expectations order is not contiguous from 1",
            "number migration expectations once each in contiguous order beginning at 1",
        ));
    }
    Ok(())
}

fn validate_no_change_decision_content(content: &DecisionContent) -> Result<(), TerminalFailure> {
    if let Some((field, _)) = [
        ("invariant_contract", content.invariant_contract.is_some()),
        (
            "existing_packages_considered",
            content.existing_packages_considered.is_some(),
        ),
        (
            "required_consumer_level_tests",
            content.required_consumer_level_tests.is_some(),
        ),
        (
            "migration_expectations",
            content.migration_expectations.is_some(),
        ),
        (
            "rollback_or_resplitting_path",
            content.rollback_or_resplitting_path.is_some(),
        ),
    ]
    .into_iter()
    .find(|(_, present)| *present)
    {
        return Err(TerminalFailure::refusal(
            format!("reuse decision action authorizes no implementation but carries `{field}`"),
            format!(
                "remove `{field}` and all other implementation-shaped items, or choose an action that authorizes implementation"
            ),
        ));
    }
    Ok(())
}

fn require_nonempty_string_list(field: &str, values: &[String]) -> Result<(), TerminalFailure> {
    if values.is_empty() {
        return Err(missing_required_decision_item(field));
    }
    for (index, value) in values.iter().enumerate() {
        require_nonempty(&format!("{field}[{}]", index + 1), value)?;
    }
    Ok(())
}

fn missing_required_decision_item(field: &str) -> TerminalFailure {
    TerminalFailure::refusal(
        format!("reuse decision `{field}` is missing or empty"),
        format!("provide non-empty `{field}` content in the accepted reuse decision"),
    )
}

fn require_nonempty_change_list<T>(
    field: &str,
    value: Option<&[T]>,
) -> Result<(), TerminalFailure> {
    if value.is_none_or(<[T]>::is_empty) {
        return Err(missing_change_decision_item(field));
    }
    Ok(())
}

fn missing_change_decision_item(field: &str) -> TerminalFailure {
    TerminalFailure::refusal(
        format!(
            "reuse decision action authorizes implementation but `{field}` is missing or empty"
        ),
        format!(
            "provide a non-empty `{field}` value, or choose a no-implementation action and omit all implementation-shaped items"
        ),
    )
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
                    sequence: event.envelope.sequence,
                    event_id: event.envelope.event_id,
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
                    sequence: event.envelope.sequence,
                    event_id: event.envelope.event_id,
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

fn read_decision_proposal(path: &Path) -> Result<DecisionProposal, TerminalFailure> {
    let text = fs::read_to_string(path).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "reuse decision proposal `{}` cannot be read: {error}",
                path.display()
            ),
            "supply a readable UTF-8 TOML proposal with `--proposal <PATH>`",
        )
    })?;
    validate_decision_vocabulary(&text)?;
    let document = toml::from_str::<DecisionProposalDocument>(&text).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "reuse decision proposal `{}` is invalid: {error}",
                path.display()
            ),
            "provide a complete TOML reuse-decision proposal using one permitted identity verdict and action",
        )
    })?;
    let proposal = match document {
        DecisionProposalDocument::Human(content) => DecisionProposal {
            content,
            prepared: None,
        },
        DecisionProposalDocument::Prepared(event) => {
            validate_recorded_decision(&event)?;
            let prepared = PreparedDecision {
                sequence: event.envelope.sequence,
                event_id: event.envelope.event_id,
                bytes: text,
            };
            DecisionProposal {
                content: event.content,
                prepared: Some(prepared),
            }
        }
    };
    validate_decision_content(&proposal.content)?;
    Ok(proposal)
}

fn read_verification_proposal(path: &Path) -> Result<VerificationProposal, TerminalFailure> {
    let text = fs::read_to_string(path).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "verification proposal `{}` cannot be read: {error}",
                path.display()
            ),
            "supply a readable UTF-8 TOML proposal with `--proposal <PATH>`",
        )
    })?;
    let document = toml::from_str::<VerificationProposalDocument>(&text).map_err(|error| {
        TerminalFailure::refusal(
            format!(
                "verification proposal `{}` is invalid: {error}",
                path.display()
            ),
            "provide a complete TOML verification proposal using one permitted disposition and result outcome",
        )
    })?;
    let proposal = match document {
        VerificationProposalDocument::Human(content) => VerificationProposal {
            content,
            prepared: None,
        },
        VerificationProposalDocument::Prepared(event) => {
            validate_recorded_verification(&event)?;
            let prepared = PreparedVerification {
                sequence: event.envelope.sequence,
                event_id: event.envelope.event_id,
                bytes: text,
            };
            VerificationProposal {
                content: event.content,
                prepared: Some(prepared),
            }
        }
    };
    validate_verification_content(&proposal.content)?;
    Ok(proposal)
}

fn validate_verification_content(content: &VerificationContent) -> Result<(), TerminalFailure> {
    for (index, result) in content.condition_results.iter().enumerate() {
        require_nonempty(
            &format!("condition_results[{}].condition", index + 1),
            &result.condition,
        )?;
        validate_verification_result(
            &format!("condition result {}", index + 1),
            result.outcome,
            result.exception.as_deref(),
            &result.evidence,
        )?;
    }
    let mut consumers = BTreeSet::new();
    for (index, result) in content.consumer_results.iter().enumerate() {
        require_nonempty(
            &format!("consumer_results[{}].consumer", index + 1),
            &result.consumer,
        )?;
        if !consumers.insert((result.repository_id, result.consumer.trim())) {
            return Err(TerminalFailure::refusal(
                format!(
                    "verification records consumer `{}` in participant `{}` more than once",
                    result.consumer.trim(),
                    result.repository_id
                ),
                "record each affected participant repository and consumer pair exactly once",
            ));
        }
        validate_verification_result(
            &format!("consumer result {}", index + 1),
            result.outcome,
            result.exception.as_deref(),
            &result.evidence,
        )?;
    }
    let has_not_met = content
        .condition_results
        .iter()
        .map(|result| result.outcome)
        .chain(content.consumer_results.iter().map(|result| result.outcome))
        .any(|outcome| outcome == VerificationResult::NotMet);
    if content.disposition == VerificationDisposition::Closed && has_not_met {
        return Err(TerminalFailure::refusal(
            "verification disposition `closed` carries a `not_met` result",
            "use disposition `parked` or `reopened`, or record only met results and explicit accepted exceptions before closing",
        ));
    }
    Ok(())
}

fn validate_verification_result(
    subject: &str,
    outcome: VerificationResult,
    exception: Option<&str>,
    evidence: &[EvidenceReference],
) -> Result<(), TerminalFailure> {
    match outcome {
        VerificationResult::AcceptedException => {
            let exception = exception.ok_or_else(|| {
                TerminalFailure::refusal(
                    format!("{subject} is an accepted exception without a reason"),
                    "state the explicit human-accepted exception in `exception`",
                )
            })?;
            require_nonempty(&format!("{subject} exception"), exception)?;
        }
        VerificationResult::Met | VerificationResult::NotMet => {
            if exception.is_some() {
                return Err(TerminalFailure::refusal(
                    format!(
                        "{subject} outcome `{}` carries an exception reason",
                        outcome.label()
                    ),
                    "remove `exception`, or use outcome `accepted_exception`",
                ));
            }
            if evidence.is_empty() {
                return Err(TerminalFailure::refusal(
                    format!(
                        "{subject} outcome `{}` carries no evidence reference",
                        outcome.label()
                    ),
                    "add one or more recoverable evidence references bearing the verification result",
                ));
            }
        }
    }
    validate_verification_evidence(subject, evidence)
}

fn validate_verification_evidence(
    subject: &str,
    evidence: &[EvidenceReference],
) -> Result<(), TerminalFailure> {
    for (index, reference) in evidence.iter().enumerate() {
        if reference.reference.trim().is_empty() {
            return Err(TerminalFailure::refusal(
                format!("{subject} evidence reference {} is empty", index + 1),
                "provide a recoverable commit reference bearing the verification result",
            ));
        }
        if let Some(path) = &reference.path {
            validate_relative_evidence_path(path)?;
        }
    }
    Ok(())
}

fn validate_verification_against_decision(
    case_id: Uuid,
    verification: &VerificationContent,
    decision: &DecisionContent,
) -> Result<(), TerminalFailure> {
    for (index, expected) in decision.verification_conditions.iter().enumerate() {
        let Some(recorded) = verification.condition_results.get(index) else {
            return Err(TerminalFailure::refusal(
                format!(
                    "verification for case `{case_id}` is missing condition result {} for `{expected}`",
                    index + 1
                ),
                "answer every accepted verification condition exactly once in its recorded order",
            ));
        };
        if recorded.condition != *expected {
            return Err(TerminalFailure::refusal(
                format!(
                    "verification condition result {} repeats `{}`, but the accepted decision records `{expected}`",
                    index + 1,
                    recorded.condition
                ),
                "repeat every accepted verification condition exactly in its recorded order",
            ));
        }
    }
    if let Some(extra) = verification
        .condition_results
        .get(decision.verification_conditions.len())
    {
        return Err(TerminalFailure::refusal(
            format!(
                "verification for case `{case_id}` records extra condition `{}`",
                extra.condition
            ),
            "answer only the verification conditions recorded by the accepted decision",
        ));
    }

    for affected in &decision.affected_consumers {
        if !verification.consumer_results.iter().any(|result| {
            result.repository_id == affected.repository_id
                && result.consumer.trim() == affected.consumer.trim()
        }) {
            return Err(TerminalFailure::refusal(
                format!(
                    "verification for case `{case_id}` is missing consumer `{}` in participant `{}`",
                    affected.consumer.trim(),
                    affected.repository_id
                ),
                "answer every affected participant repository and consumer pair exactly once",
            ));
        }
    }
    if let Some(extra) = verification.consumer_results.iter().find(|result| {
        !decision.affected_consumers.iter().any(|affected| {
            result.repository_id == affected.repository_id
                && result.consumer.trim() == affected.consumer.trim()
        })
    }) {
        return Err(TerminalFailure::refusal(
            format!(
                "verification for case `{case_id}` records extra consumer `{}` in participant `{}`",
                extra.consumer.trim(),
                extra.repository_id
            ),
            "answer only the affected participant repository and consumer pairs recorded by the accepted decision",
        ));
    }
    Ok(())
}

fn validate_decision_vocabulary(text: &str) -> Result<(), TerminalFailure> {
    let Ok(table) = text.parse::<toml::Table>() else {
        return Ok(());
    };
    for (field, allowed) in [
        ("identity_verdict", IdentityVerdict::NAMES),
        ("action", DecisionAction::NAMES),
    ] {
        let Some(value) = table.get(field).and_then(toml::Value::as_str) else {
            continue;
        };
        if !allowed.contains(&value) {
            return Err(TerminalFailure::refusal(
                format!("reuse decision `{field}` value `{value}` is unrecognized"),
                format!("use one permitted `{field}` value: {}", allowed.join(", ")),
            ));
        }
    }
    Ok(())
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

const OPENING_REFUSAL: event::EnvelopeRefusal<'static> = event::EnvelopeRefusal {
    unsupported: "prepared opening event is not a supported revision 1 `case_opened` event",
    noun: "opening",
    instant_name: "opening",
    preview_command: "case open --preview",
};

const APPEND_REFUSAL: event::EnvelopeRefusal<'static> = event::EnvelopeRefusal {
    unsupported: "prepared append event is not a supported `occurrence_appended` event after revision 1",
    noun: "append",
    instant_name: "append",
    preview_command: "case append --preview",
};

const EARLY_REVIEW_REFUSAL: event::EnvelopeRefusal<'static> = event::EnvelopeRefusal {
    unsupported: "prepared early-review event is not a supported `early_review_authorized` event after revision 1",
    noun: "early-review",
    instant_name: "early-review authorization",
    preview_command: "case override --preview",
};

const DECISION_REFUSAL: event::EnvelopeRefusal<'static> = event::EnvelopeRefusal {
    unsupported: "prepared reuse decision event is not a supported later event",
    noun: "reuse decision",
    instant_name: "reuse decision",
    preview_command: "case decide --preview",
};

const VERIFICATION_REFUSAL: event::EnvelopeRefusal<'static> = event::EnvelopeRefusal {
    unsupported: "prepared verification event is not a supported later event",
    noun: "verification",
    instant_name: "verification",
    preview_command: "case verify --preview",
};

fn validate_recorded_opening(event: &CaseOpenedEvent) -> Result<(), TerminalFailure> {
    event
        .envelope
        .validate(EventType::CaseOpened, &OPENING_REFUSAL)?;
    if event.case_id.get_version_num() != 4 {
        return Err(TerminalFailure::refusal(
            format!(
                "case identity `{}` is not an opaque UUID version 4",
                event.case_id
            ),
            "use a newly generated UUID version 4 as `case_id`",
        ));
    }
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
    event
        .envelope
        .validate(EventType::OccurrenceAppended, &APPEND_REFUSAL)?;
    validate_occurrence(&event.occurrence, 1, "occurrence.evidence")
}

fn validate_recorded_early_review(
    event: &EarlyReviewAuthorizedEvent,
) -> Result<(), TerminalFailure> {
    event
        .envelope
        .validate(EventType::EarlyReviewAuthorized, &EARLY_REVIEW_REFUSAL)?;
    validate_early_review_content(&event.reason, &event.review_appetite, &event.evidence)
}

fn validate_recorded_decision(event: &ReuseDecisionAcceptedEvent) -> Result<(), TerminalFailure> {
    event
        .envelope
        .validate(EventType::ReuseDecisionAccepted, &DECISION_REFUSAL)?;
    validate_decision_content(&event.content)
}

fn validate_recorded_verification(
    event: &VerificationRecordedEvent,
) -> Result<(), TerminalFailure> {
    event
        .envelope
        .validate(EventType::VerificationRecorded, &VERIFICATION_REFUSAL)?;
    validate_verification_content(&event.content)
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

/// Resolves each occurrence's participant repository against `roots`.
///
/// Roots arrive already selected so one command selects them once, rather than
/// each derivation re-reading the user-local configuration.
fn resolve_participants(
    roots: &[PathBuf],
    occurrences: &[Occurrence],
) -> Result<BTreeMap<Uuid, Visibility>, TerminalFailure> {
    let scan = portfolio::scan(roots)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact bytes `CONSUMER-CONTRACT.md` §3 calls the hardest
    /// compatibility surface. Nothing else pins the recorded layout, so a
    /// serde or `toml` change that reorders or renames an envelope field
    /// would otherwise reach recorded evidence unannounced.
    const RECORDED_AT: &str = "2026-08-11T06:00:00Z";
    const EVENT_ID: &str = "22222222-2222-4222-8222-222222222222";
    const CASE_ID: &str = "33333333-3333-4333-8333-333333333333";
    const PARTICIPANT_ID: &str = "11111111-1111-4111-8111-111111111111";

    fn envelope(sequence: i64, event_type: EventType) -> event::Envelope {
        event::Envelope {
            schema_version: CASE_SCHEMA_VERSION,
            sequence,
            event_id: Uuid::parse_str(EVENT_ID).expect("the fixture identity is a valid UUID"),
            event_type,
            recorded_at: RECORDED_AT.to_owned(),
        }
    }

    fn occurrence() -> Occurrence {
        Occurrence {
            repository_id: Uuid::parse_str(PARTICIPANT_ID)
                .expect("the fixture participant is a valid UUID"),
            consumer: "billing totals".to_owned(),
            independence: "arose from a separate consumer need".to_owned(),
            evidence: vec![EvidenceReference {
                kind: EvidenceKind::Commit,
                reference: "abc123".to_owned(),
                path: Some("src/lib.rs".to_owned()),
            }],
        }
    }

    #[test]
    fn every_event_type_records_its_envelope_in_one_order() {
        let opening = toml::to_string(&CaseOpenedEvent {
            envelope: envelope(OPENING_SEQUENCE, EventType::CaseOpened),
            case_id: Uuid::parse_str(CASE_ID).expect("the fixture case is a valid UUID"),
            responsibility: "one responsibility".to_owned(),
            steward_repository_id: Uuid::parse_str(PARTICIPANT_ID)
                .expect("the fixture steward is a valid UUID"),
            privacy: Visibility::Private,
            occurrences: vec![occurrence()],
        })
        .expect("the opening event should serialize");

        assert_eq!(
            opening,
            concat!(
                "schema_version = 1\n",
                "sequence = 1\n",
                "event_id = \"22222222-2222-4222-8222-222222222222\"\n",
                "event_type = \"case_opened\"\n",
                "recorded_at = \"2026-08-11T06:00:00Z\"\n",
                "case_id = \"33333333-3333-4333-8333-333333333333\"\n",
                "responsibility = \"one responsibility\"\n",
                "steward_repository_id = \"11111111-1111-4111-8111-111111111111\"\n",
                "privacy = \"private\"\n",
                "\n",
                "[[occurrences]]\n",
                "repository_id = \"11111111-1111-4111-8111-111111111111\"\n",
                "consumer = \"billing totals\"\n",
                "independence = \"arose from a separate consumer need\"\n",
                "\n",
                "[[occurrences.evidence]]\n",
                "kind = \"commit\"\n",
                "reference = \"abc123\"\n",
                "path = \"src/lib.rs\"\n",
            ),
            "the opening event's recorded layout is a compatibility surface"
        );

        let append = toml::to_string(&OccurrenceAppendedEvent {
            envelope: envelope(2, EventType::OccurrenceAppended),
            occurrence: occurrence(),
        })
        .expect("the append event should serialize");

        assert!(
            append.starts_with(concat!(
                "schema_version = 1\n",
                "sequence = 2\n",
                "event_id = \"22222222-2222-4222-8222-222222222222\"\n",
                "event_type = \"occurrence_appended\"\n",
                "recorded_at = \"2026-08-11T06:00:00Z\"\n",
                "\n",
                "[occurrence]\n",
            )),
            "the append event must open with the envelope in order: {append}"
        );

        let early_review = toml::to_string(&EarlyReviewAuthorizedEvent {
            envelope: envelope(3, EventType::EarlyReviewAuthorized),
            reason: "waiting costs more".to_owned(),
            review_appetite: "one afternoon".to_owned(),
            evidence: vec![EvidenceReference {
                kind: EvidenceKind::Commit,
                reference: "def456".to_owned(),
                path: None,
            }],
        })
        .expect("the early-review event should serialize");

        assert!(
            early_review.starts_with(concat!(
                "schema_version = 1\n",
                "sequence = 3\n",
                "event_id = \"22222222-2222-4222-8222-222222222222\"\n",
                "event_type = \"early_review_authorized\"\n",
                "recorded_at = \"2026-08-11T06:00:00Z\"\n",
                "reason = \"waiting costs more\"\n",
            )),
            "the early-review event must open with the envelope in order: {early_review}"
        );
    }

    /// Two flattened fields in one struct is the shape the decision event
    /// takes once the envelope has a type. It must still emit the envelope
    /// before the content, in the recorded order.
    #[test]
    fn the_decision_event_records_its_envelope_before_its_content() {
        let decision = toml::to_string(&ReuseDecisionAcceptedEvent {
            envelope: envelope(4, EventType::ReuseDecisionAccepted),
            content: DecisionContent {
                identity_verdict: IdentityVerdict::SameResponsibility,
                action: DecisionAction::ExtractOrDeepenLocally,
                accepted_scope: "one scope".to_owned(),
                non_responsibilities: vec!["not this".to_owned()],
                affected_consumers: Vec::new(),
                alternatives_rejected: Vec::new(),
                compatibility_consequences: "none".to_owned(),
                verification_conditions: vec!["one condition".to_owned()],
                invariant_contract: None,
                existing_packages_considered: None,
                required_consumer_level_tests: None,
                migration_expectations: None,
                rollback_or_resplitting_path: None,
            },
        })
        .expect("the decision event should serialize");

        assert!(
            decision.starts_with(concat!(
                "schema_version = 1\n",
                "sequence = 4\n",
                "event_id = \"22222222-2222-4222-8222-222222222222\"\n",
                "event_type = \"reuse_decision_accepted\"\n",
                "recorded_at = \"2026-08-11T06:00:00Z\"\n",
                "identity_verdict = \"same_responsibility\"\n",
                "action = \"extract_or_deepen_locally\"\n",
            )),
            "the decision event must open with the envelope in order: {decision}"
        );
    }

    /// An unknown field in a recorded event is still refused. Flattening the
    /// envelope keeps `deny_unknown_fields` in force but costs the span and
    /// the expected-field list, which ADR 0014 names and requires witnessed.
    #[test]
    fn a_recorded_event_carrying_an_unknown_field_is_refused() {
        let mut recorded = toml::to_string(&OccurrenceAppendedEvent {
            envelope: envelope(2, EventType::OccurrenceAppended),
            occurrence: occurrence(),
        })
        .expect("the append event should serialize");
        recorded.insert_str(0, "surprise = \"extra\"\n");

        let error = toml::from_str::<OccurrenceAppendedEvent>(&recorded)
            .expect_err("an unknown field must be refused");

        assert!(
            error.to_string().contains("unknown field `surprise`"),
            "the refusal must name the unknown field: {error}"
        );
    }

    /// The recorded and prepared identities a publication failure names.
    const RECORDED_EVENT_ID: &str = "44444444-4444-4444-8444-444444444444";
    const PREPARED_EVENT_ID: &str = "55555555-5555-4555-8555-555555555555";
    const EXISTING_EVENT_PATH: &str = "cases/0002-occurrence-appended.toml";

    /// Renders one later-event publication failure the way a command's `map_err` does.
    ///
    /// `CONSUMER-CONTRACT.md` §1 makes the terminal meanings an independently versioned
    /// surface, and ADR 0016 places refusal prose in process rather than at the process
    /// boundary. Every expectation below is a literal transcribed from the terminal text,
    /// so a renderer that changes one byte fails here rather than reaching an operator.
    fn refusal(
        render: fn(Uuid, publication::PublicationFailure) -> TerminalFailure,
        failure: publication::PublicationFailure,
    ) -> String {
        render(fixture_case_id(), failure).to_string()
    }

    fn append_refusal(case_id: Uuid, failure: publication::PublicationFailure) -> TerminalFailure {
        APPEND_REFUSALS.publication_failure(case_id, failure)
    }

    fn early_review_refusal(
        case_id: Uuid,
        failure: publication::PublicationFailure,
    ) -> TerminalFailure {
        EARLY_REVIEW_REFUSALS.publication_failure(case_id, failure)
    }

    fn decision_refusal(
        case_id: Uuid,
        failure: publication::PublicationFailure,
    ) -> TerminalFailure {
        DECISION_REFUSALS.publication_failure(case_id, failure)
    }

    fn fixture_case_id() -> Uuid {
        Uuid::parse_str(CASE_ID).expect("the fixture case is a valid UUID")
    }

    fn recorded_event_id() -> Uuid {
        Uuid::parse_str(RECORDED_EVENT_ID).expect("the recorded identity is a valid UUID")
    }

    fn unreadable_existing_event() -> publication::PublicationFailure {
        publication::PublicationFailure::ExistingEvent(
            publication::ExistingEventFailure::Unreadable {
                path: PathBuf::from(EXISTING_EVENT_PATH),
                error: std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "permission denied",
                ),
            },
        )
    }

    fn invalid_existing_event() -> publication::PublicationFailure {
        publication::PublicationFailure::ExistingEvent(publication::ExistingEventFailure::Invalid {
            path: PathBuf::from(EXISTING_EVENT_PATH),
            error: toml::from_str::<toml::Value>("not = = toml")
                .expect_err("the fixture must fail to parse"),
        })
    }

    fn identity_conflict(prepared_event_id: Option<Uuid>) -> publication::PublicationFailure {
        publication::PublicationFailure::ExistingEvent(
            publication::ExistingEventFailure::IdentityConflict {
                recorded_sequence: 2,
                recorded_event_id: recorded_event_id(),
                prepared_event_id,
                current_revision: 3,
            },
        )
    }

    fn prepared_identity_conflict() -> publication::PublicationFailure {
        identity_conflict(Some(
            Uuid::parse_str(PREPARED_EVENT_ID).expect("the prepared identity is a valid UUID"),
        ))
    }

    fn content_drift() -> publication::PublicationFailure {
        publication::PublicationFailure::ExistingEvent(
            publication::ExistingEventFailure::ContentDrift {
                recorded_event_id: recorded_event_id(),
            },
        )
    }

    const fn revision_conflict() -> publication::PublicationFailure {
        publication::PublicationFailure::RevisionConflict {
            expected_revision: 1,
            current_revision: 3,
        }
    }

    /// Asserts an invalid-event refusal, whose middle is `toml`'s own parse diagnostic.
    fn assert_invalid_event_refusal(rendered: &str, event_noun: &str, operation: &str) {
        let condition =
            format!("refusal: recorded {event_noun} event `{EXISTING_EVENT_PATH}` is invalid: ");
        let resolution = format!(
            "\nresolution: restore the supported recorded event before retrying the {operation}"
        );
        assert!(
            rendered.starts_with(&condition),
            "the refusal must open with `{condition}`: {rendered}"
        );
        assert!(
            rendered.ends_with(&resolution),
            "the refusal must close with `{resolution}`: {rendered}"
        );
    }

    #[test]
    fn an_append_publication_failure_names_the_append() {
        let case_id = fixture_case_id();

        assert_eq!(
            refusal(append_refusal, unreadable_existing_event()),
            format!(
                "refusal: recorded append event `{EXISTING_EVENT_PATH}` cannot be read: permission denied\nresolution: restore the recorded event before retrying the append"
            )
        );
        assert_invalid_event_refusal(
            &refusal(append_refusal, invalid_existing_event()),
            "append",
            "append",
        );
        assert_eq!(
            refusal(append_refusal, prepared_identity_conflict()),
            format!(
                "refusal: case `{case_id}` has a revision conflict at sequence 2: event `{RECORDED_EVENT_ID}` is recorded instead of event `{PREPARED_EVENT_ID}`\nresolution: inspect sequence 2; retry its recorded identity if it is the intended append, or prepare a distinct occurrence against revision 3"
            )
        );
        assert_eq!(
            refusal(append_refusal, content_drift()),
            format!(
                "refusal: append event identity `{RECORDED_EVENT_ID}` is already recorded with different content\nresolution: restore the exact previewed append event before retrying"
            )
        );
        assert_eq!(
            refusal(append_refusal, revision_conflict()),
            format!(
                "refusal: expected revision 1 does not match case `{case_id}` current revision 3\nresolution: run `case show {case_id}` and retry with `--expected-revision 3`"
            )
        );
    }

    #[test]
    fn an_early_review_publication_failure_names_the_override() {
        let case_id = fixture_case_id();

        assert_eq!(
            refusal(early_review_refusal, unreadable_existing_event()),
            format!(
                "refusal: recorded early-review event `{EXISTING_EVENT_PATH}` cannot be read: permission denied\nresolution: restore the recorded event before retrying the early-review override"
            )
        );
        assert_invalid_event_refusal(
            &refusal(early_review_refusal, invalid_existing_event()),
            "early-review",
            "early-review override",
        );
        assert_eq!(
            refusal(early_review_refusal, prepared_identity_conflict()),
            format!(
                "refusal: case `{case_id}` has a revision conflict at sequence 2: event `{RECORDED_EVENT_ID}` is recorded instead of event `{PREPARED_EVENT_ID}`\nresolution: inspect sequence 2; retry its recorded identity if it is the intended early-review override, or prepare a new operation against revision 3"
            )
        );
        assert_eq!(
            refusal(early_review_refusal, content_drift()),
            format!(
                "refusal: early-review event identity `{RECORDED_EVENT_ID}` is already recorded with different content\nresolution: restore the exact previewed early-review event before retrying"
            )
        );
        assert_eq!(
            refusal(early_review_refusal, revision_conflict()),
            format!(
                "refusal: expected revision 1 does not match case `{case_id}` current revision 3\nresolution: run `case show {case_id}` and retry `case override {case_id}` with `--expected-revision 3` and the approved proposal"
            )
        );
    }

    #[test]
    fn a_decision_publication_failure_names_the_reuse_decision() {
        let case_id = fixture_case_id();

        assert_eq!(
            refusal(decision_refusal, unreadable_existing_event()),
            format!(
                "refusal: recorded reuse decision event `{EXISTING_EVENT_PATH}` cannot be read: permission denied\nresolution: restore the recorded event before retrying the reuse decision"
            )
        );
        assert_invalid_event_refusal(
            &refusal(decision_refusal, invalid_existing_event()),
            "reuse decision",
            "reuse decision",
        );
        assert_eq!(
            refusal(decision_refusal, prepared_identity_conflict()),
            format!(
                "refusal: case `{case_id}` has a revision conflict at sequence 2: event `{RECORDED_EVENT_ID}` is recorded instead of event `{PREPARED_EVENT_ID}`\nresolution: inspect sequence 2; retry its recorded identity if it is the intended reuse decision, or prepare a new operation against revision 3"
            )
        );
        assert_eq!(
            refusal(decision_refusal, content_drift()),
            format!(
                "refusal: reuse decision event identity `{RECORDED_EVENT_ID}` is already recorded with different content\nresolution: restore the exact previewed reuse decision event before retrying"
            )
        );
        assert_eq!(
            refusal(decision_refusal, revision_conflict()),
            format!(
                "refusal: expected revision 1 does not match case `{case_id}` current revision 3\nresolution: run `case show {case_id}` and retry `case decide {case_id}` with `--expected-revision 3` and the approved proposal"
            )
        );
    }

    #[test]
    fn an_identity_conflict_without_a_prepared_event_names_no_proposed_identity() {
        let case_id = fixture_case_id();

        assert_eq!(
            refusal(append_refusal, identity_conflict(None)),
            format!(
                "refusal: case `{case_id}` has a revision conflict at sequence 2: event `{RECORDED_EVENT_ID}` is recorded instead of a newly prepared event\nresolution: inspect sequence 2; retry its recorded identity if it is the intended append, or prepare a distinct occurrence against revision 3"
            )
        );
    }

    #[test]
    fn a_protocol_failure_reaches_the_terminal_unchanged() {
        for render in [append_refusal, early_review_refusal, decision_refusal] {
            assert_eq!(
                refusal(
                    render,
                    publication::PublicationFailure::Protocol(TerminalFailure::unsafe_failure(
                        "the case directory could not be locked"
                    ))
                ),
                "unsafe failure: the case directory could not be locked"
            );
        }
    }
}
