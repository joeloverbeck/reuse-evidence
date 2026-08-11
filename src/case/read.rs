use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use uuid::Uuid;

use super::naming::{self, EventFileName, EventPosition, EventType, OPENING_SEQUENCE};
use super::{
    CASE_SCHEMA_VERSION, CaseOpenedEvent, EarlyReviewAuthorizedEvent, Occurrence,
    OccurrenceAppendedEvent, REVIEW_ONLY_NOTICE, ReportedPrivacy, ReuseDecisionAcceptedEvent,
    find_repository_root, read_steward, validate_case_storage_path,
};
use crate::{TerminalFailure, Visibility, portfolio};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CaseState {
    Watching,
    ReviewReadyByOccurrenceCount,
    ReviewReadyByEarlyReviewOverride,
    AwaitingVerification,
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
            Self::Watching | Self::AwaitingVerification => None,
            Self::ReviewReadyByOccurrenceCount => Some("occurrence-count"),
            Self::ReviewReadyByEarlyReviewOverride => Some("early-review-override"),
        }
    }

    /// Writes the `state:` line and the readiness lines a review-ready case adds.
    ///
    /// `indent` prefixes every line, because a case listing nests what a case
    /// receipt and `case show` print flush.
    pub(super) fn write_receipt_lines(
        self,
        formatter: &mut Formatter<'_>,
        indent: &str,
    ) -> fmt::Result {
        writeln!(formatter, "{indent}state: {}", self.label())?;
        if let Some(basis) = self.basis() {
            writeln!(formatter, "{indent}readiness_basis: {basis}")?;
        }
        if self.authorizes_review() {
            writeln!(formatter, "{indent}readiness: {REVIEW_ONLY_NOTICE}")?;
        }
        Ok(())
    }
}

pub(super) struct CaseRecord {
    pub(super) case_id: Uuid,
    pub(super) responsibility: String,
    pub(super) revision: i64,
    pub(super) privacy: Visibility,
    pub(super) occurrences: Vec<Occurrence>,
    early_review: Option<EarlyReviewAuthorizedEvent>,
    decision: Option<ReuseDecisionAcceptedEvent>,
    conditions: Conditions,
}

enum CaseEvent {
    Opened(CaseOpenedEvent),
    OccurrenceAppended(OccurrenceAppendedEvent),
    EarlyReviewAuthorized(EarlyReviewAuthorizedEvent),
    ReuseDecisionAccepted(ReuseDecisionAcceptedEvent),
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
    pub(super) fn state(&self) -> CaseState {
        if self.decision.is_some() {
            CaseState::AwaitingVerification
        } else if self.early_review.is_some() {
            CaseState::ReviewReadyByEarlyReviewOverride
        } else {
            CaseState::from_occurrence_count(self.occurrences.len())
        }
    }

    pub(super) fn state_after_appending_occurrence(&self) -> CaseState {
        if self.decision.is_some() {
            CaseState::AwaitingVerification
        } else if self.early_review.is_some() {
            CaseState::ReviewReadyByEarlyReviewOverride
        } else {
            CaseState::from_occurrence_count(self.occurrences.len() + 1)
        }
    }

    pub(super) const fn has_early_review(&self) -> bool {
        self.early_review.is_some()
    }

    pub(super) const fn has_decision(&self) -> bool {
        self.decision.is_some()
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

/// The implementation handoff projected from one accepted reuse decision.
pub struct BriefOutcome {
    case: CaseRecord,
    privacy: ReportedPrivacy,
}

/// Renders the accepted implementation handoff without creating a second artifact.
impl Display for BriefOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let case = &self.case;
        let decision = case
            .decision
            .as_ref()
            .expect("a brief outcome has an accepted reuse decision");
        let content = &decision.content;
        writeln!(formatter, "implementation brief\ncase_id: {}", case.case_id)?;
        self.privacy.write_receipt_line(formatter)?;
        if content.action.authorizes_implementation() {
            write_authorized_implementation(formatter, case, content)
        } else {
            write_no_implementation_decision(formatter, case, content)
        }
    }
}

fn write_authorized_implementation(
    formatter: &mut Formatter<'_>,
    case: &CaseRecord,
    content: &super::DecisionContent,
) -> fmt::Result {
    formatter.write_str("implementation: authorized\n")?;
    write_responsibility_identity(formatter, case, content)?;
    formatter.write_str("evidence_bearing_consumers:\n")?;
    for occurrence in &case.occurrences {
        writeln!(
            formatter,
            "- repository_id: {}\n  consumer: {}\n  independence: {}",
            occurrence.repository_id, occurrence.consumer, occurrence.independence
        )?;
        if let Some(affected) = content.affected_consumers.iter().find(|affected| {
            occurrence.repository_id == affected.repository_id
                && occurrence.consumer.trim() == affected.consumer.trim()
        }) {
            writeln!(formatter, "  expectation: {}", affected.expectation)?;
        }
        formatter.write_str("  evidence:\n")?;
        for evidence in &occurrence.evidence {
            writeln!(
                formatter,
                "  - kind: {}\n    reference: {}",
                evidence.kind.label(),
                evidence.reference
            )?;
            if let Some(path) = &evidence.path {
                writeln!(formatter, "    path: {path}")?;
            }
        }
    }
    writeln!(
        formatter,
        "invariant_contract: {}\nnon_responsibilities:",
        content
            .invariant_contract
            .as_deref()
            .expect("an implementation-authorizing decision has an invariant contract")
    )?;
    write_string_list(formatter, &content.non_responsibilities)?;
    write_chosen_home_and_scope(formatter, content)?;
    write_alternatives_rejected(formatter, content)?;
    formatter.write_str("existing_packages_considered:\n")?;
    for package in content
        .existing_packages_considered
        .as_deref()
        .expect("an implementation-authorizing decision records considered packages")
    {
        writeln!(
            formatter,
            "- package: {}\n  fit: {}\n  reason: {}",
            package.package, package.fit, package.reason
        )?;
    }
    formatter.write_str("required_consumer_level_tests:\n")?;
    write_string_list(
        formatter,
        content
            .required_consumer_level_tests
            .as_deref()
            .expect("an implementation-authorizing decision records consumer-level tests"),
    )?;
    writeln!(
        formatter,
        "compatibility_and_release_consequences: {}\nmigration_order:",
        content.compatibility_consequences
    )?;
    for migration in content
        .migration_expectations
        .as_deref()
        .expect("an implementation-authorizing decision records migration order")
    {
        writeln!(
            formatter,
            "- order: {}\n  expectation: {}",
            migration.order, migration.expectation
        )?;
    }
    writeln!(
        formatter,
        "rollback_or_resplitting_strategy: {}\nverification_conditions:",
        content
            .rollback_or_resplitting_path
            .as_deref()
            .expect("an implementation-authorizing decision records its reversal path")
    )?;
    write_string_list(formatter, &content.verification_conditions)
}

fn write_no_implementation_decision(
    formatter: &mut Formatter<'_>,
    case: &CaseRecord,
    content: &super::DecisionContent,
) -> fmt::Result {
    formatter
        .write_str("implementation: not authorized\ndecision: authorizes no implementation\n")?;
    write_responsibility_identity(formatter, case, content)?;
    write_chosen_home_and_scope(formatter, content)?;
    formatter.write_str("non_responsibilities:\n")?;
    write_string_list(formatter, &content.non_responsibilities)?;
    write_alternatives_rejected(formatter, content)?;
    write_compatibility_and_verification(formatter, content)
}

fn write_string_list(formatter: &mut Formatter<'_>, values: &[String]) -> fmt::Result {
    for value in values {
        writeln!(formatter, "- {value}")?;
    }
    Ok(())
}

fn write_responsibility_identity(
    formatter: &mut Formatter<'_>,
    case: &CaseRecord,
    content: &super::DecisionContent,
) -> fmt::Result {
    writeln!(
        formatter,
        "accepted_responsibility_identity:\n  responsibility: {}\n  verdict: {}",
        case.responsibility,
        content.identity_verdict.label()
    )
}

fn write_chosen_home_and_scope(
    formatter: &mut Formatter<'_>,
    content: &super::DecisionContent,
) -> fmt::Result {
    writeln!(
        formatter,
        "chosen_home_and_scope:\n  action: {}\n  scope: {}",
        content.action.label(),
        content.accepted_scope
    )
}

fn write_alternatives_rejected(
    formatter: &mut Formatter<'_>,
    content: &super::DecisionContent,
) -> fmt::Result {
    formatter.write_str("alternatives_rejected:\n")?;
    for alternative in &content.alternatives_rejected {
        writeln!(
            formatter,
            "- alternative: {}\n  reason: {}",
            alternative.alternative, alternative.reason
        )?;
    }
    Ok(())
}

fn write_compatibility_and_verification(
    formatter: &mut Formatter<'_>,
    content: &super::DecisionContent,
) -> fmt::Result {
    writeln!(
        formatter,
        "compatibility_and_release_consequences: {}\nverification_conditions:",
        content.compatibility_consequences
    )?;
    write_string_list(formatter, &content.verification_conditions)
}

/// Renders the case and every recorded occurrence deterministically.
impl Display for ShowOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let case = &self.case;
        writeln!(
            formatter,
            "case\ncase_id: {}\nresponsibility: {}\nrevision: {}\noccurrence_count: {}",
            case.case_id,
            case.responsibility,
            case.revision,
            case.occurrences.len()
        )?;
        case.state().write_receipt_lines(formatter, "")?;
        writeln!(
            formatter,
            "privacy_conflicted: {}\nstale: {}\noccurrences:",
            render_condition(case.conditions.privacy_conflicted),
            render_condition(case.conditions.stale)
        )?;
        for occurrence in &case.occurrences {
            writeln!(
                formatter,
                "- repository_id: {}\n  consumer: {}\n  independence: {}\n  evidence:",
                occurrence.repository_id, occurrence.consumer, occurrence.independence
            )?;
            for evidence in &occurrence.evidence {
                let kind = evidence.kind.label();
                writeln!(
                    formatter,
                    "  - kind: {kind}\n    reference: {}",
                    evidence.reference
                )?;
                if let Some(path) = &evidence.path {
                    writeln!(formatter, "    path: {path}")?;
                }
            }
        }
        if let Some(early_review) = &case.early_review {
            writeln!(
                formatter,
                "early_review:\n  reason: {}\n  review_appetite: {}\n  evidence:",
                early_review.reason, early_review.review_appetite
            )?;
            for evidence in &early_review.evidence {
                let kind = evidence.kind.label();
                writeln!(
                    formatter,
                    "  - kind: {kind}\n    reference: {}",
                    evidence.reference
                )?;
                if let Some(path) = &evidence.path {
                    writeln!(formatter, "    path: {path}")?;
                }
            }
        }
        if !self.portfolio_available {
            formatter.write_str(super::PORTFOLIO_UNAVAILABLE_FOOTER)?;
        }
        Ok(())
    }
}

/// Renders a deterministic steward-local case listing.
impl Display for ListOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("cases\n")?;
        for case in &self.cases {
            writeln!(
                formatter,
                "- case_id: {}\n  revision: {}\n  occurrence_count: {}",
                case.case_id,
                case.revision,
                case.occurrences.len()
            )?;
            case.state().write_receipt_lines(formatter, "  ")?;
            writeln!(
                formatter,
                "  privacy_conflicted: {}\n  stale: {}",
                render_condition(case.conditions.privacy_conflicted),
                render_condition(case.conditions.stale)
            )?;
        }
        if !self.portfolio_available {
            formatter.write_str(super::PORTFOLIO_UNAVAILABLE_FOOTER)?;
        }
        Ok(())
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
    let case_id = parse_recorded_case_id(case_id)?;
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

/// Projects the implementation brief for one accepted steward-local decision.
///
/// # Errors
///
/// Returns a refusal when the identity, steward, case, or accepted decision
/// cannot be read safely.
pub fn brief(
    working_directory: &Path,
    case_id: &str,
    root_overrides: &[PathBuf],
) -> Result<BriefOutcome, TerminalFailure> {
    let case_id = parse_recorded_case_id(case_id)?;
    let repository_root = find_repository_root(working_directory)?;
    let steward = read_steward(&repository_root)?;
    let relative_case_directory = Path::new("reuse-evidence/cases").join(case_id.to_string());
    validate_case_storage_path(&repository_root, &relative_case_directory)?;
    let case = read_case_for(
        &repository_root,
        &relative_case_directory,
        case_id,
        steward.repository_id(),
        "run `case list` in this steward repository, then retry `case brief <CASE_ID>` with one of its recorded case identities",
    )?;
    if case.decision.is_none() {
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
    }
    let privacy = super::reported_privacy(&case, &steward, root_overrides);
    Ok(BriefOutcome { case, privacy })
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
    root_overrides: &[PathBuf],
) -> Result<bool, TerminalFailure> {
    let Some(roots) = portfolio::selected_roots_if_configured(root_overrides)? else {
        return Ok(false);
    };
    let scan = portfolio::scan(&roots)?;
    for case in cases {
        // Recorded case privacy is a term of complete case privacy, not only current participant
        // visibility, so a steward that turned public after opening conflicts with its own case.
        let mut privacy_conflicted =
            steward_visibility == Visibility::Public && case.privacy == Visibility::Private;
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
    let mut early_review = None;
    let mut decision = None;
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
        conditions: Conditions::UNKNOWN,
    })
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
                decision.sequence
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
            validate_file_event_type(case_id, file_name, file_sequence, event.event_type)?;
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
    validate_file_event_type(case_id, file_name, file_sequence, event.event_type)?;
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
    validate_body_sequence(event_path, event.sequence, file_sequence)?;
    if event.sequence == OPENING_SEQUENCE {
        return Err(TerminalFailure::refusal(
            format!(
                "case event `{}` records `early_review_authorized` at opening sequence 1",
                event_path.display()
            ),
            "restore `case_opened` as sequence 1 and authorize early review only after it",
        ));
    }
    validate_file_event_type(case_id, file_name, file_sequence, event.event_type)?;
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
    validate_body_sequence(event_path, event.sequence, file_sequence)?;
    if event.sequence == OPENING_SEQUENCE {
        return Err(TerminalFailure::refusal(
            format!(
                "case event `{}` records `{}` at opening sequence 1",
                event_path.display(),
                event.event_type.body_name()
            ),
            "restore `case_opened` as sequence 1 and accept reuse decisions only after it",
        ));
    }
    validate_file_event_type(case_id, file_name, file_sequence, event.event_type)?;
    super::validate_recorded_decision(&event)?;
    Ok((file_sequence, CaseEvent::ReuseDecisionAccepted(event)))
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
