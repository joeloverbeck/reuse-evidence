//! Durable case recording and inspection mechanics.

mod event;
mod instant;
mod naming;
mod publication;
mod read;

pub use instant::RecordedInstant;
pub(crate) use read::private_case_stewarded_by;
pub use read::{BriefOutcome, ListOutcome, ShowOutcome, brief, list, show};

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};

use naming::{EventFileName, EventPosition, EventType, OPENING_SEQUENCE};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::marker::{self, MarkerRead};
use crate::portfolio;
use crate::{TerminalFailure, Visibility, create_file_atomically};

const CASE_SCHEMA_VERSION: i64 = 1;
const REVIEW_ONLY_NOTICE: &str = "authorizes semantic review; does not authorize extraction";
const IMPLEMENTATION_NOTICE: &str =
    "authorizes implementation outside the reuse lifecycle; does not perform it";
const NO_IMPLEMENTATION_NOTICE: &str = "authorizes no implementation";
const PORTFOLIO_UNAVAILABLE_FOOTER: &str = "portfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n";
const PARTICIPANTS_UNRESOLVED_FOOTER: &str = "portfolio conditions unavailable: a recorded participant does not resolve to exactly one enrolled repository beneath the selected portfolio roots; restore its enrollment and unique repository identity to derive privacy\n";
const APPEND_UNSTEWARDED_RESOLUTION: &str =
    "run `case list` in this steward repository and retry with a recorded case identity";
const EARLY_REVIEW_UNSTEWARDED_RESOLUTION: &str = "run `case list` in this steward repository and retry `case override` with a recorded watching case identity";
const DECISION_UNSTEWARDED_RESOLUTION: &str = "run `case list` in this steward repository and retry `case decide` with a recorded review-ready case identity";

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
/// The three later event types share this carrier under ADR 0013. Which heading it renders and
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

const APPEND_HEADINGS: LaterEventHeadings = LaterEventHeadings {
    preview: "case append preview",
    created: "appended occurrence",
    existing: "occurrence already recorded",
};

const EARLY_REVIEW_HEADINGS: LaterEventHeadings = LaterEventHeadings {
    preview: "early-review override preview",
    created: "authorized early review",
    existing: "early review already authorized",
};

const DECISION_HEADINGS: LaterEventHeadings = LaterEventHeadings {
    preview: "reuse decision preview",
    created: "accepted reuse decision",
    existing: "reuse decision already recorded",
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

impl ReportedPrivacy {
    /// Writes the `privacy:` line, followed by the footer explaining an underivable privacy.
    fn write_receipt_line(self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Derived(privacy) => writeln!(formatter, "privacy: {privacy}"),
            Self::PortfolioUnconfigured => {
                formatter.write_str("privacy: unknown\n")?;
                formatter.write_str(PORTFOLIO_UNAVAILABLE_FOOTER)
            }
            Self::ParticipantsUnresolved => {
                formatter.write_str("privacy: unknown\n")?;
                formatter.write_str(PARTICIPANTS_UNRESOLVED_FOOTER)
            }
        }
    }
}

/// The spine every event-type receipt prints, in order.
///
/// Which fields an event type supplies stays that event type's decision under
/// ADR 0010; the spine fixes only their order and spelling. `state` is absent
/// for an event that reports none, and `preview_event` carries the exact event
/// bytes a preview appends.
struct EventReceipt<'a> {
    heading: &'a str,
    case_id: Uuid,
    event_path: &'a Path,
    revision: i64,
    state: Option<read::CaseState>,
    privacy: ReportedPrivacy,
    notice: Option<&'a str>,
    preview_event: Option<&'a str>,
}

impl Display for EventReceipt<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", self.heading)?;
        writeln!(formatter, "case_id: {}", self.case_id)?;
        writeln!(formatter, "file: {}", self.event_path.display())?;
        writeln!(formatter, "revision: {}", self.revision)?;
        if let Some(state) = self.state {
            state.write_receipt_lines(formatter, "")?;
        }
        self.privacy.write_receipt_line(formatter)?;
        if let Some(notice) = self.notice {
            writeln!(formatter, "decision: {notice}")?;
        }
        if let Some(event) = self.preview_event {
            formatter.write_str("event:\n")?;
            formatter.write_str(event)?;
        }
        Ok(())
    }
}

/// Renders the receipt followed by the exact event bytes for a preview.
impl Display for OpenOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let heading = match self.effect {
            OpenEffect::Preview => "case open preview",
            OpenEffect::Created => "opened case",
            OpenEffect::Existing => "existing case",
        };
        EventReceipt {
            heading,
            case_id: self.case_id,
            event_path: &self.event_path,
            revision: OPENING_SEQUENCE,
            state: None,
            // Opening derives privacy once and has no retry path, so its stored
            // privacy stays a `Visibility` the spine widens only in passing.
            privacy: ReportedPrivacy::Derived(self.privacy),
            notice: None,
            preview_event: (self.effect == OpenEffect::Preview).then_some(self.event.as_str()),
        }
        .fmt(formatter)
    }
}

/// Renders one later-event receipt, followed by the exact event bytes for a preview.
impl Display for LaterEventOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        EventReceipt {
            heading: self.headings.heading(self.effect),
            case_id: self.case_id,
            event_path: &self.event_path,
            revision: self.revision,
            state: self.state,
            privacy: self.privacy,
            notice: self.notice,
            preview_event: (self.effect == LaterEventEffect::Preview)
                .then_some(self.event.as_str()),
        }
        .fmt(formatter)
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
    location: &portfolio::PortfolioLocation,
    recorded_at: RecordedInstant,
    preview: bool,
) -> Result<OpenOutcome, TerminalFailure> {
    let repository_root = find_repository_root(working_directory)?;
    let mut steward = read_steward(&repository_root)?;
    let _marker_lock = if preview {
        None
    } else {
        let marker_lock = crate::lock_repository_marker(&repository_root)?;
        steward = read_steward(&repository_root)?;
        Some(marker_lock)
    };
    let proposal = read_proposal(proposal_path)?;
    let relative_case_directory =
        PathBuf::from("reuse-evidence/cases").join(proposal.case_id.to_string());
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
        APPEND_UNSTEWARDED_RESOLUTION,
    )?;
    let case_id = located.case_id;
    let proposal = read_append_proposal(proposal_path)?;
    validate_prepared_append_sequence(&proposal, expected_revision, located.sequence)?;
    let (relative_event_path, absolute_event_path) =
        later_event_paths(&located, EventType::OccurrenceAppended)?;
    let event = append_event_bytes(&proposal, located.sequence, recorded_at)?;
    let prepared_event = publication::PreparedEvent {
        event_id: proposal.prepared.as_ref().map(|prepared| prepared.event_id),
        bytes: &event,
    };
    let eligibility = |case: &read::CaseRecord, ()| {
        validate_new_append(case, &proposal, &located.steward, location)
    };

    if preview {
        let checked = located
            .publication
            .check(
                &located.case,
                &absolute_event_path,
                prepared_event,
                |_| Ok(()),
                eligibility,
            )
            .map_err(|failure| append_publication_failure(case_id, failure))?;
        return Ok(match checked {
            publication::Checked::Existing(existing) => append_retry_outcome(
                case_id,
                relative_event_path,
                &located.case,
                existing,
                &located.steward,
                location,
            ),
            publication::Checked::Fresh(privacy) => append_outcome(
                LaterEventEffect::Preview,
                case_id,
                relative_event_path,
                located.sequence,
                located.case.state_after_appending_occurrence(),
                ReportedPrivacy::Derived(privacy),
                event,
            ),
        });
    }

    match located
        .publication
        .publish(
            publication::PublicationTarget {
                repository_root: &located.repository_root,
                relative_case_directory: &located.relative_case_directory,
                relative_event_path: &relative_event_path,
            },
            prepared_event,
            || {
                read::read_case_for(
                    &located.repository_root,
                    &located.relative_case_directory,
                    case_id,
                    located.steward.repository_id(),
                    APPEND_UNSTEWARDED_RESOLUTION,
                )
            },
            |_| Ok(()),
            eligibility,
        )
        .map_err(|failure| append_publication_failure(case_id, failure))?
    {
        publication::PublicationOutcome::Created { case, validation } => Ok(append_outcome(
            LaterEventEffect::Created,
            case_id,
            relative_event_path,
            located.sequence,
            case.state_after_appending_occurrence(),
            ReportedPrivacy::Derived(validation),
            event,
        )),
        publication::PublicationOutcome::Existing { case, event } => Ok(append_retry_outcome(
            case_id,
            relative_event_path,
            &case,
            event,
            &located.steward,
            location,
        )),
    }
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
    location: &portfolio::PortfolioLocation,
    recorded_at: RecordedInstant,
    preview: bool,
) -> Result<LaterEventOutcome, TerminalFailure> {
    let located = locate_later_event_case(
        working_directory,
        case_id,
        expected_revision,
        EARLY_REVIEW_UNSTEWARDED_RESOLUTION,
    )?;
    let case_id = located.case_id;
    let proposal = read_early_review_proposal(proposal_path)?;
    validate_prepared_early_review_sequence(&proposal, expected_revision, located.sequence)?;
    let (relative_event_path, absolute_event_path) =
        later_event_paths(&located, EventType::EarlyReviewAuthorized)?;
    let event = early_review_event_bytes(&proposal, located.sequence, recorded_at)?;
    let prepared_event = publication::PreparedEvent {
        event_id: proposal.prepared.as_ref().map(|prepared| prepared.event_id),
        bytes: &event,
    };
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

    if preview {
        let checked = located
            .publication
            .check(
                &located.case,
                &absolute_event_path,
                prepared_event,
                case_privacy,
                eligibility,
            )
            .map_err(|failure| early_review_publication_failure(case_id, failure))?;
        return Ok(match checked {
            publication::Checked::Existing(existing) => early_review_retry_outcome(
                case_id,
                relative_event_path,
                &located.case,
                existing,
                &located.steward,
                location,
            ),
            publication::Checked::Fresh(privacy) => early_review_outcome(
                LaterEventEffect::Preview,
                case_id,
                relative_event_path,
                located.sequence,
                ReportedPrivacy::Derived(privacy),
                event,
            ),
        });
    }

    match located
        .publication
        .publish(
            publication::PublicationTarget {
                repository_root: &located.repository_root,
                relative_case_directory: &located.relative_case_directory,
                relative_event_path: &relative_event_path,
            },
            prepared_event,
            || {
                read::read_case_for(
                    &located.repository_root,
                    &located.relative_case_directory,
                    case_id,
                    located.steward.repository_id(),
                    EARLY_REVIEW_UNSTEWARDED_RESOLUTION,
                )
            },
            case_privacy,
            eligibility,
        )
        .map_err(|failure| early_review_publication_failure(case_id, failure))?
    {
        publication::PublicationOutcome::Created { validation, .. } => {
            Ok(early_review_created_outcome(
                case_id,
                relative_event_path,
                located.sequence,
                validation,
                event,
            ))
        }
        publication::PublicationOutcome::Existing { case, event } => {
            Ok(early_review_retry_outcome(
                case_id,
                relative_event_path,
                &case,
                event,
                &located.steward,
                location,
            ))
        }
    }
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
        DECISION_UNSTEWARDED_RESOLUTION,
    )?;
    let case_id = located.case_id;
    let proposal = read_decision_proposal(proposal_path)?;
    validate_prepared_decision_sequence(&proposal, expected_revision, located.sequence)?;
    let (relative_event_path, absolute_event_path) =
        later_event_paths(&located, EventType::ReuseDecisionAccepted)?;
    let event = decision_event_bytes(&proposal, located.sequence, recorded_at)?;
    let prepared_event = publication::PreparedEvent {
        event_id: proposal.prepared.as_ref().map(|prepared| prepared.event_id),
        bytes: &event,
    };
    let eligibility = |case: &read::CaseRecord, ()| -> Result<Visibility, TerminalFailure> {
        validate_new_decision(case, &proposal)?;
        let roots = portfolio::selected_roots(location)?;
        let privacy = derive_complete_case_privacy(case, &located.steward, &roots)?;
        validate_decision_privacy(case, &located.steward, privacy)?;
        Ok(privacy)
    };

    if preview {
        let checked = located
            .publication
            .check(
                &located.case,
                &absolute_event_path,
                prepared_event,
                |_| Ok(()),
                eligibility,
            )
            .map_err(|failure| decision_publication_failure(case_id, failure))?;
        return Ok(match checked {
            publication::Checked::Existing(existing) => decision_retry_outcome(
                case_id,
                relative_event_path,
                &located.case,
                existing,
                &located.steward,
                location,
                proposal.content.action,
            ),
            publication::Checked::Fresh(privacy) => decision_outcome(
                LaterEventEffect::Preview,
                case_id,
                relative_event_path,
                located.sequence,
                ReportedPrivacy::Derived(privacy),
                proposal.content.action,
                event,
            ),
        });
    }

    match located
        .publication
        .publish(
            publication::PublicationTarget {
                repository_root: &located.repository_root,
                relative_case_directory: &located.relative_case_directory,
                relative_event_path: &relative_event_path,
            },
            prepared_event,
            || {
                read::read_case_for(
                    &located.repository_root,
                    &located.relative_case_directory,
                    case_id,
                    located.steward.repository_id(),
                    DECISION_UNSTEWARDED_RESOLUTION,
                )
            },
            |_| Ok(()),
            eligibility,
        )
        .map_err(|failure| decision_publication_failure(case_id, failure))?
    {
        publication::PublicationOutcome::Created { validation, .. } => Ok(decision_outcome(
            LaterEventEffect::Created,
            case_id,
            relative_event_path,
            located.sequence,
            ReportedPrivacy::Derived(validation),
            proposal.content.action,
            event,
        )),
        publication::PublicationOutcome::Existing { case, event } => Ok(decision_retry_outcome(
            case_id,
            relative_event_path,
            &case,
            event,
            &located.steward,
            location,
            proposal.content.action,
        )),
    }
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
    unstewarded_resolution: &str,
) -> Result<LocatedCase, TerminalFailure> {
    let case_id = parse_case_id(case_id)?;
    let publication = publication::Publication::new(expected_revision)?;
    let sequence = publication.sequence();
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root)?;
    let relative_case_directory = Path::new("reuse-evidence/cases").join(case_id.to_string());
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

/// Builds one reuse-decision receipt.
///
/// The decision event is the only later event that reports a notice; whether it authorizes
/// implementation is the accepted action's decision, not the receipt's.
fn decision_outcome(
    effect: LaterEventEffect,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    privacy: ReportedPrivacy,
    action: DecisionAction,
    event: String,
) -> LaterEventOutcome {
    LaterEventOutcome {
        effect,
        headings: DECISION_HEADINGS,
        case_id,
        event_path,
        revision,
        state: Some(read::CaseState::AwaitingVerification),
        privacy,
        notice: Some(if action.authorizes_implementation() {
            IMPLEMENTATION_NOTICE
        } else {
            NO_IMPLEMENTATION_NOTICE
        }),
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
        action,
        event.bytes,
    )
}

fn decision_publication_failure(
    case_id: Uuid,
    failure: publication::PublicationFailure,
) -> TerminalFailure {
    match failure {
        publication::PublicationFailure::Protocol(failure) => failure,
        publication::PublicationFailure::ExistingEvent(failure) => {
            decision_existing_event_failure(case_id, failure)
        }
        publication::PublicationFailure::RevisionConflict {
            expected_revision,
            current_revision,
        } => decision_revision_conflict(case_id, expected_revision, current_revision),
    }
}

fn decision_existing_event_failure(
    case_id: Uuid,
    failure: publication::ExistingEventFailure,
) -> TerminalFailure {
    match failure {
        publication::ExistingEventFailure::Unreadable { path, error } => TerminalFailure::refusal(
            format!(
                "recorded reuse decision event `{}` cannot be read: {error}",
                path.display()
            ),
            "restore the recorded event before retrying the reuse decision",
        ),
        publication::ExistingEventFailure::Invalid { path, error } => TerminalFailure::refusal(
            format!(
                "recorded reuse decision event `{}` is invalid: {error}",
                path.display()
            ),
            "restore the supported recorded event before retrying the reuse decision",
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
                    "inspect sequence {recorded_sequence}; retry its recorded identity if it is the intended reuse decision, or prepare a new operation against revision {current_revision}"
                ),
            )
        }
        publication::ExistingEventFailure::ContentDrift { recorded_event_id } => {
            TerminalFailure::refusal(
                format!(
                    "reuse decision event identity `{recorded_event_id}` is already recorded with different content"
                ),
                "restore the exact previewed reuse decision event before retrying",
            )
        }
    }
}

fn decision_revision_conflict(
    case_id: Uuid,
    expected_revision: i64,
    current_revision: i64,
) -> TerminalFailure {
    TerminalFailure::refusal(
        format!(
            "expected revision {expected_revision} does not match case `{case_id}` current revision {current_revision}"
        ),
        format!(
            "run `case show {case_id}` and retry `case decide {case_id}` with `--expected-revision {current_revision}` and the approved proposal"
        ),
    )
}

fn validate_new_decision(
    case: &read::CaseRecord,
    proposal: &DecisionProposal,
) -> Result<(), TerminalFailure> {
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
/// Its readiness is stated as a constant rather than derived from the recorded occurrence count,
/// as ADR 0010 records, and the override authorizes no implementation, so it reports no notice.
fn early_review_outcome(
    effect: LaterEventEffect,
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    privacy: ReportedPrivacy,
    event: String,
) -> LaterEventOutcome {
    LaterEventOutcome {
        effect,
        headings: EARLY_REVIEW_HEADINGS,
        case_id,
        event_path,
        revision,
        state: Some(read::CaseState::ReviewReadyByEarlyReviewOverride),
        privacy,
        notice: None,
        event,
    }
}

fn early_review_created_outcome(
    case_id: Uuid,
    event_path: PathBuf,
    revision: i64,
    privacy: Visibility,
    event: String,
) -> LaterEventOutcome {
    early_review_outcome(
        LaterEventEffect::Created,
        case_id,
        event_path,
        revision,
        ReportedPrivacy::Derived(privacy),
        event,
    )
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
        reported_privacy(case, steward, location),
        event.bytes,
    )
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

fn validate_recorded_opening(event: &CaseOpenedEvent) -> Result<(), TerminalFailure> {
    event.envelope.validate(
        EventType::CaseOpened,
        EventPosition::Opening,
        &OPENING_REFUSAL,
    )?;
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
    event.envelope.validate(
        EventType::OccurrenceAppended,
        EventPosition::Later,
        &APPEND_REFUSAL,
    )?;
    validate_occurrence(&event.occurrence, 1, "occurrence.evidence")
}

fn validate_recorded_early_review(
    event: &EarlyReviewAuthorizedEvent,
) -> Result<(), TerminalFailure> {
    event.envelope.validate(
        EventType::EarlyReviewAuthorized,
        EventPosition::Later,
        &EARLY_REVIEW_REFUSAL,
    )?;
    validate_early_review_content(&event.reason, &event.review_appetite, &event.evidence)
}

fn validate_recorded_decision(event: &ReuseDecisionAcceptedEvent) -> Result<(), TerminalFailure> {
    event.envelope.validate(
        EventType::ReuseDecisionAccepted,
        EventPosition::Later,
        &DECISION_REFUSAL,
    )?;
    validate_decision_content(&event.content)
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
}
