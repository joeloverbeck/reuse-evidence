//! Every line a case command prints.
//!
//! ADR 0017 gives case terminal text one owner. This module formats and does nothing else: it
//! reads no file, takes no lock, and derives no state. `case::read` produces the projections and
//! `src/case.rs` produces the outcomes; only their printing lives here.
//!
//! Which fields an event type prints stays that event type's decision under ADR 0010, so the
//! per-type headings and notices stay with their event type. What this module owns is the shape
//! those fields print in.

use std::fmt::{self, Display, Formatter};
use std::path::Path;

use uuid::Uuid;

use super::read::{BriefOutcome, CaseRecord, CaseState, ListOutcome, ShowOutcome};
use super::{
    AuthorizedImplementation, DecisionAuthorization, DecisionContent, LaterEventEffect,
    LaterEventOutcome, OpenEffect, OpenOutcome, ReportedPrivacy,
};

/// The readiness a review-ready case reports, and the boundary it does not cross.
const REVIEW_ONLY_NOTICE: &str = "authorizes semantic review; does not authorize extraction";
const PORTFOLIO_UNAVAILABLE_FOOTER: &str = "portfolio conditions unavailable: configure portfolio roots or supply `--root <PATH>` to derive privacy conflicts and staleness\n";
const PARTICIPANTS_UNRESOLVED_FOOTER: &str = "portfolio conditions unavailable: a recorded participant does not resolve to exactly one enrolled repository beneath the selected portfolio roots; restore its enrollment and unique repository identity to derive privacy\n";

/// Writes the `state:` line and the readiness lines a review-ready case adds.
///
/// `indent` prefixes every line, because a case listing nests what a case receipt and
/// `case show` print flush.
fn write_state_lines(state: CaseState, formatter: &mut Formatter<'_>, indent: &str) -> fmt::Result {
    writeln!(formatter, "{indent}state: {}", state.label())?;
    if let Some(basis) = state.basis() {
        writeln!(formatter, "{indent}readiness_basis: {basis}")?;
    }
    if state.authorizes_review() {
        writeln!(formatter, "{indent}readiness: {REVIEW_ONLY_NOTICE}")?;
    }
    Ok(())
}

/// Writes the `privacy:` line, followed by the footer explaining an underivable privacy.
fn write_privacy_line(privacy: ReportedPrivacy, formatter: &mut Formatter<'_>) -> fmt::Result {
    match privacy {
        ReportedPrivacy::Derived(privacy) => writeln!(formatter, "privacy: {privacy}"),
        ReportedPrivacy::PortfolioUnconfigured => {
            formatter.write_str("privacy: unknown\n")?;
            formatter.write_str(PORTFOLIO_UNAVAILABLE_FOOTER)
        }
        ReportedPrivacy::ParticipantsUnresolved => {
            formatter.write_str("privacy: unknown\n")?;
            formatter.write_str(PARTICIPANTS_UNRESOLVED_FOOTER)
        }
    }
}

fn render_condition(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

/// The spine every event-type receipt prints, in order.
///
/// Which fields an event type supplies stays that event type's decision under ADR 0010; the
/// spine fixes only their order and spelling. `state` is absent for an event that reports none,
/// and `preview_event` carries the exact event bytes a preview appends.
struct EventReceipt<'a> {
    heading: &'a str,
    case_id: Uuid,
    event_path: &'a Path,
    revision: i64,
    state: Option<CaseState>,
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
            write_state_lines(state, formatter, "")?;
        }
        write_privacy_line(self.privacy, formatter)?;
        if let Some(notice) = self.notice {
            writeln!(formatter, "{notice}")?;
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
            revision: super::OPENING_SEQUENCE,
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

/// Renders the accepted implementation handoff without creating a second artifact.
impl Display for BriefOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let case = &self.case;
        let content = &self.decision;
        writeln!(formatter, "implementation brief\ncase_id: {}", case.case_id)?;
        write_privacy_line(self.privacy, formatter)?;
        match &self.authorization {
            DecisionAuthorization::Implementation(authorized) => {
                write_authorized_implementation(formatter, case, content, authorized)
            }
            DecisionAuthorization::NoImplementation => {
                write_no_implementation_decision(formatter, case, content)
            }
        }
    }
}

fn write_authorized_implementation(
    formatter: &mut Formatter<'_>,
    case: &CaseRecord,
    content: &DecisionContent,
    authorized: &AuthorizedImplementation,
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
        authorized.invariant_contract
    )?;
    write_string_list(formatter, &content.non_responsibilities)?;
    write_chosen_home_and_scope(formatter, content)?;
    write_alternatives_rejected(formatter, content)?;
    formatter.write_str("existing_packages_considered:\n")?;
    for package in &authorized.existing_packages_considered {
        writeln!(
            formatter,
            "- package: {}\n  fit: {}\n  reason: {}",
            package.package, package.fit, package.reason
        )?;
    }
    formatter.write_str("required_consumer_level_tests:\n")?;
    write_string_list(formatter, &authorized.required_consumer_level_tests)?;
    writeln!(
        formatter,
        "compatibility_and_release_consequences: {}\nmigration_order:",
        content.compatibility_consequences
    )?;
    for migration in &authorized.migration_expectations {
        writeln!(
            formatter,
            "- order: {}\n  expectation: {}",
            migration.order, migration.expectation
        )?;
    }
    writeln!(
        formatter,
        "rollback_or_resplitting_strategy: {}\nverification_conditions:",
        authorized.rollback_or_resplitting_path
    )?;
    write_string_list(formatter, &content.verification_conditions)
}

fn write_no_implementation_decision(
    formatter: &mut Formatter<'_>,
    case: &CaseRecord,
    content: &DecisionContent,
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
    content: &DecisionContent,
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
    content: &DecisionContent,
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
    content: &DecisionContent,
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
    content: &DecisionContent,
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
        write_state_lines(case.state(), formatter, "")?;
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
        write_verifications(formatter, case)?;
        if !self.portfolio_available {
            formatter.write_str(PORTFOLIO_UNAVAILABLE_FOOTER)?;
        }
        Ok(())
    }
}

fn write_verifications(formatter: &mut Formatter<'_>, case: &CaseRecord) -> fmt::Result {
    if case.verifications.is_empty() {
        return Ok(());
    }
    formatter.write_str("verifications:\n")?;
    for verification in &case.verifications {
        writeln!(
            formatter,
            "- disposition: {}\n  condition_results:",
            verification.content.disposition.label()
        )?;
        for result in &verification.content.condition_results {
            writeln!(
                formatter,
                "  - condition: {}\n    outcome: {}",
                result.condition,
                result.outcome.label()
            )?;
            if let Some(exception) = &result.exception {
                writeln!(formatter, "    exception: {exception}")?;
            }
            write_verification_evidence(formatter, &result.evidence)?;
        }
        formatter.write_str("  consumer_results:\n")?;
        for result in &verification.content.consumer_results {
            writeln!(
                formatter,
                "  - repository_id: {}\n    consumer: {}\n    outcome: {}",
                result.repository_id,
                result.consumer,
                result.outcome.label()
            )?;
            if let Some(exception) = &result.exception {
                writeln!(formatter, "    exception: {exception}")?;
            }
            write_verification_evidence(formatter, &result.evidence)?;
        }
    }
    Ok(())
}

fn write_verification_evidence(
    formatter: &mut Formatter<'_>,
    evidence: &[super::EvidenceReference],
) -> fmt::Result {
    if evidence.is_empty() {
        formatter.write_str("    evidence: []\n")?;
        return Ok(());
    }
    formatter.write_str("    evidence:\n")?;
    for reference in evidence {
        writeln!(
            formatter,
            "    - kind: {}\n      reference: {}",
            reference.kind.label(),
            reference.reference
        )?;
        if let Some(path) = &reference.path {
            writeln!(formatter, "      path: {path}")?;
        }
    }
    Ok(())
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
            write_state_lines(case.state(), formatter, "  ")?;
            writeln!(
                formatter,
                "  privacy_conflicted: {}\n  stale: {}",
                render_condition(case.conditions.privacy_conflicted),
                render_condition(case.conditions.stale)
            )?;
        }
        if !self.portfolio_available {
            formatter.write_str(PORTFOLIO_UNAVAILABLE_FOOTER)?;
        }
        Ok(())
    }
}

/// Every line this module prints, from projections built in memory.
///
/// ADR 0017 names this instrument in its consequences: "every renderer becomes reachable from a
/// hand-built projection value with no repository on disk." The projections' fields are readable
/// across `case`, so this module can write what it reads. ADR 0016 forbids widening them to
/// public API, which is why the instrument lives here rather than in an integration suite.
///
/// These tests pin text rather than propose it. `CONSUMER-CONTRACT.md` §1 versions the terminal
/// surface and ADR 0017 authorizes no change to it, so a diff here is a compatibility question.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::Visibility;
    use crate::case::event::Envelope;
    use crate::case::naming::EventType;
    use crate::case::read::Conditions;
    use crate::case::{
        AffectedConsumer, CASE_SCHEMA_VERSION, ConditionResult, ConsumerResult, DecisionAction,
        EarlyReviewAuthorizedEvent, EvidenceKind, EvidenceReference, ExistingPackageConsidered,
        IdentityVerdict, LaterEventHeadings, MigrationExpectation, Occurrence, RejectedAlternative,
        ReuseDecisionAcceptedEvent, VerificationContent, VerificationDisposition,
        VerificationRecordedEvent, VerificationResult,
    };

    const CASE_ID: &str = "33333333-3333-4333-8333-333333333333";
    const EVENT_ID: &str = "44444444-4444-4444-8444-444444444444";
    const FIRST_PARTICIPANT_ID: &str = "11111111-1111-4111-8111-111111111111";
    const SECOND_PARTICIPANT_ID: &str = "22222222-2222-4222-8222-222222222222";
    const THIRD_PARTICIPANT_ID: &str = "55555555-5555-4555-8555-555555555555";

    fn uuid(value: &str) -> Uuid {
        Uuid::parse_str(value).expect("the fixture identity is a valid UUID")
    }

    /// A recorded envelope no renderer reads.
    ///
    /// The event bodies carry one, but nothing in this module prints an envelope field, so its
    /// values are fixed rather than varied per test.
    fn envelope(sequence: i64, event_type: EventType) -> Envelope {
        Envelope {
            schema_version: CASE_SCHEMA_VERSION,
            sequence,
            event_id: uuid(EVENT_ID),
            event_type,
            recorded_at: "2026-08-12T06:00:00Z".to_owned(),
        }
    }

    fn evidence(reference: &str, path: Option<&str>) -> EvidenceReference {
        EvidenceReference {
            kind: EvidenceKind::Commit,
            reference: reference.to_owned(),
            path: path.map(str::to_owned),
        }
    }

    fn occurrence(repository_id: &str, consumer: &str) -> Occurrence {
        Occurrence {
            repository_id: uuid(repository_id),
            consumer: consumer.to_owned(),
            independence: "arose from a separate consumer need".to_owned(),
            evidence: vec![evidence("abc123", Some("src/lib.rs"))],
        }
    }

    fn conditions(privacy_conflicted: Option<bool>, stale: Option<bool>) -> Conditions {
        Conditions {
            privacy_conflicted,
            stale,
        }
    }

    fn watching_case() -> CaseRecord {
        CaseRecord {
            case_id: uuid(CASE_ID),
            responsibility: "one responsibility".to_owned(),
            revision: 1,
            privacy: Visibility::Private,
            occurrences: vec![
                occurrence(FIRST_PARTICIPANT_ID, "billing totals"),
                occurrence(SECOND_PARTICIPANT_ID, "invoice totals"),
            ],
            early_review: None,
            decision: None,
            verifications: Vec::new(),
            conditions: conditions(Some(false), Some(false)),
        }
    }

    /// A third independent occurrence, which is what makes a case review-ready by count.
    fn review_ready_case() -> CaseRecord {
        let mut case = watching_case();
        case.revision = 2;
        case.occurrences
            .push(occurrence(THIRD_PARTICIPANT_ID, "statement totals"));
        case
    }

    fn early_review_event() -> EarlyReviewAuthorizedEvent {
        EarlyReviewAuthorizedEvent {
            envelope: envelope(2, EventType::EarlyReviewAuthorized),
            reason: "divergence is already costing release time".to_owned(),
            review_appetite: "one afternoon".to_owned(),
            evidence: vec![evidence("def456", Some("docs/cost.md"))],
        }
    }

    fn change_decision() -> DecisionContent {
        DecisionContent {
            identity_verdict: IdentityVerdict::SameResponsibility,
            action: DecisionAction::ExtractOrDeepenLocally,
            accepted_scope: "one local module".to_owned(),
            non_responsibilities: vec!["currency formatting".to_owned()],
            affected_consumers: vec![AffectedConsumer {
                repository_id: uuid(FIRST_PARTICIPANT_ID),
                consumer: "billing totals".to_owned(),
                expectation: "migrates to the deepened module".to_owned(),
            }],
            alternatives_rejected: vec![RejectedAlternative {
                alternative: "publish a public package".to_owned(),
                reason: "no consumer outside this repository".to_owned(),
            }],
            compatibility_consequences: "no recorded evidence changes".to_owned(),
            verification_conditions: vec!["both consumers call one module".to_owned()],
            invariant_contract: Some("totals round half to even".to_owned()),
            existing_packages_considered: Some(vec![ExistingPackageConsidered {
                package: "rust-decimal".to_owned(),
                fit: "partial".to_owned(),
                reason: "does not own the rounding policy".to_owned(),
            }]),
            required_consumer_level_tests: Some(vec!["billing totals round trip".to_owned()]),
            migration_expectations: Some(vec![MigrationExpectation {
                order: 1,
                expectation: "billing totals first".to_owned(),
            }]),
            rollback_or_resplitting_path: Some("inline the module back".to_owned()),
        }
    }

    fn no_change_decision() -> DecisionContent {
        DecisionContent {
            identity_verdict: IdentityVerdict::DifferentResponsibilities,
            action: DecisionAction::RetainIntentionalDuplication,
            accepted_scope: "no shared surface".to_owned(),
            non_responsibilities: vec!["rounding policy".to_owned()],
            affected_consumers: vec![AffectedConsumer {
                repository_id: uuid(FIRST_PARTICIPANT_ID),
                consumer: "billing totals".to_owned(),
                expectation: "stays as it is".to_owned(),
            }],
            alternatives_rejected: vec![RejectedAlternative {
                alternative: "extract locally".to_owned(),
                reason: "the two totals change for different reasons".to_owned(),
            }],
            compatibility_consequences: "nothing changes".to_owned(),
            verification_conditions: vec!["neither consumer gains a dependency".to_owned()],
            invariant_contract: None,
            existing_packages_considered: None,
            required_consumer_level_tests: None,
            migration_expectations: None,
            rollback_or_resplitting_path: None,
        }
    }

    fn decided_case(content: DecisionContent) -> CaseRecord {
        let mut case = review_ready_case();
        case.revision = 3;
        case.decision = Some(ReuseDecisionAcceptedEvent {
            envelope: envelope(3, EventType::ReuseDecisionAccepted),
            content,
        });
        case
    }

    /// A decided case with one recorded verification.
    ///
    /// The disposition rides inside the content, because that is what ADR 0019 recorded: the
    /// evidence and the human's conclusion are one event.
    fn verified_case(content: VerificationContent) -> CaseRecord {
        let mut case = decided_case(change_decision());
        case.revision = 4;
        case.verifications = vec![VerificationRecordedEvent {
            envelope: envelope(4, EventType::VerificationRecorded),
            content,
        }];
        case
    }

    fn verification(
        disposition: VerificationDisposition,
        outcome: VerificationResult,
        exception: Option<&str>,
        condition_evidence: Vec<EvidenceReference>,
    ) -> VerificationContent {
        VerificationContent {
            disposition,
            condition_results: vec![ConditionResult {
                condition: "both consumers call one module".to_owned(),
                outcome,
                exception: exception.map(str::to_owned),
                evidence: condition_evidence,
            }],
            consumer_results: vec![ConsumerResult {
                repository_id: uuid(FIRST_PARTICIPANT_ID),
                consumer: "billing totals".to_owned(),
                outcome,
                exception: exception.map(str::to_owned),
                evidence: Vec::new(),
            }],
        }
    }

    /// The occurrence block `case show` prints for the two-occurrence fixture.
    const WATCHING_OCCURRENCES: &str = concat!(
        "occurrences:\n",
        "- repository_id: 11111111-1111-4111-8111-111111111111\n",
        "  consumer: billing totals\n",
        "  independence: arose from a separate consumer need\n",
        "  evidence:\n",
        "  - kind: commit\n",
        "    reference: abc123\n",
        "    path: src/lib.rs\n",
        "- repository_id: 22222222-2222-4222-8222-222222222222\n",
        "  consumer: invoice totals\n",
        "  independence: arose from a separate consumer need\n",
        "  evidence:\n",
        "  - kind: commit\n",
        "    reference: abc123\n",
        "    path: src/lib.rs\n",
    );

    /// The third occurrence's block, appended to [`WATCHING_OCCURRENCES`].
    const THIRD_OCCURRENCE: &str = concat!(
        "- repository_id: 55555555-5555-4555-8555-555555555555\n",
        "  consumer: statement totals\n",
        "  independence: arose from a separate consumer need\n",
        "  evidence:\n",
        "  - kind: commit\n",
        "    reference: abc123\n",
        "    path: src/lib.rs\n",
    );

    fn show(case: CaseRecord, portfolio_available: bool) -> String {
        ShowOutcome {
            case,
            portfolio_available,
        }
        .to_string()
    }

    #[test]
    fn a_watching_case_reports_no_readiness_lines() {
        assert_eq!(
            show(watching_case(), true),
            format!(
                concat!(
                    "case\n",
                    "case_id: 33333333-3333-4333-8333-333333333333\n",
                    "responsibility: one responsibility\n",
                    "revision: 1\n",
                    "occurrence_count: 2\n",
                    "state: watching\n",
                    "privacy_conflicted: false\n",
                    "stale: false\n",
                    "{}"
                ),
                WATCHING_OCCURRENCES
            ),
            "a watching case reports no readiness basis and no review notice"
        );
    }

    #[test]
    fn a_third_occurrence_names_its_basis_and_the_boundary_review_does_not_cross() {
        assert_eq!(
            show(review_ready_case(), true),
            format!(
                concat!(
                    "case\n",
                    "case_id: 33333333-3333-4333-8333-333333333333\n",
                    "responsibility: one responsibility\n",
                    "revision: 2\n",
                    "occurrence_count: 3\n",
                    "state: review-ready\n",
                    "readiness_basis: occurrence-count\n",
                    "readiness: authorizes semantic review; does not authorize extraction\n",
                    "privacy_conflicted: false\n",
                    "stale: false\n",
                    "{}{}"
                ),
                WATCHING_OCCURRENCES, THIRD_OCCURRENCE
            ),
            "review readiness by count names its basis and refuses extraction authority"
        );
    }

    #[test]
    fn an_early_review_override_names_its_own_basis_and_prints_its_evidence() {
        let mut case = watching_case();
        case.revision = 2;
        case.early_review = Some(early_review_event());

        assert_eq!(
            show(case, true),
            format!(
                concat!(
                    "case\n",
                    "case_id: 33333333-3333-4333-8333-333333333333\n",
                    "responsibility: one responsibility\n",
                    "revision: 2\n",
                    "occurrence_count: 2\n",
                    "state: review-ready\n",
                    "readiness_basis: early-review-override\n",
                    "readiness: authorizes semantic review; does not authorize extraction\n",
                    "privacy_conflicted: false\n",
                    "stale: false\n",
                    "{}",
                    "early_review:\n",
                    "  reason: divergence is already costing release time\n",
                    "  review_appetite: one afternoon\n",
                    "  evidence:\n",
                    "  - kind: commit\n",
                    "    reference: def456\n",
                    "    path: docs/cost.md\n",
                ),
                WATCHING_OCCURRENCES
            ),
            "an override reaches review-ready on two occurrences and states why"
        );
    }

    #[test]
    fn an_accepted_decision_supersedes_readiness_with_awaiting_verification() {
        let rendered = show(decided_case(change_decision()), true);

        assert!(
            rendered.contains("state: awaiting-verification\n"),
            "an accepted decision supersedes review readiness: {rendered}"
        );
        assert!(
            !rendered.contains("readiness"),
            "awaiting verification carries no readiness basis and no review notice: {rendered}"
        );
    }

    #[test]
    fn a_closed_case_reports_its_terminal_state_and_every_met_result() {
        let content = verification(
            VerificationDisposition::Closed,
            VerificationResult::Met,
            None,
            vec![evidence("fed987", None)],
        );

        assert_eq!(
            show(verified_case(content), true),
            format!(
                concat!(
                    "case\n",
                    "case_id: 33333333-3333-4333-8333-333333333333\n",
                    "responsibility: one responsibility\n",
                    "revision: 4\n",
                    "occurrence_count: 3\n",
                    "state: closed\n",
                    "privacy_conflicted: false\n",
                    "stale: false\n",
                    "{}{}",
                    "verifications:\n",
                    "- disposition: closed\n",
                    "  condition_results:\n",
                    "  - condition: both consumers call one module\n",
                    "    outcome: met\n",
                    "    evidence:\n",
                    "    - kind: commit\n",
                    "      reference: fed987\n",
                    "  consumer_results:\n",
                    "  - repository_id: 11111111-1111-4111-8111-111111111111\n",
                    "    consumer: billing totals\n",
                    "    outcome: met\n",
                    "    evidence: []\n",
                ),
                WATCHING_OCCURRENCES, THIRD_OCCURRENCE
            ),
            "a closed case carries no readiness basis and prints an absent evidence list as `[]`"
        );
    }

    #[test]
    fn a_parked_case_reports_its_terminal_state_and_an_unmet_result() {
        let content = verification(
            VerificationDisposition::Parked,
            VerificationResult::NotMet,
            None,
            Vec::new(),
        );
        let rendered = show(verified_case(content), true);

        assert!(
            rendered.contains("state: parked\n"),
            "a parked case reports the disposition its latest verification recorded: {rendered}"
        );
        assert!(
            !rendered.contains("readiness"),
            "a parked case has no readiness basis: {rendered}"
        );
        assert!(
            rendered.contains(concat!(
                "- disposition: parked\n",
                "  condition_results:\n",
                "  - condition: both consumers call one module\n",
                "    outcome: not_met\n",
                "    evidence: []\n",
            )),
            "an unmet condition prints its outcome verbatim: {rendered}"
        );
    }

    #[test]
    fn a_reopened_case_reports_its_terminal_state_and_its_accepted_exception() {
        let content = verification(
            VerificationDisposition::Reopened,
            VerificationResult::AcceptedException,
            Some("the second consumer ships next release"),
            Vec::new(),
        );
        let rendered = show(verified_case(content), true);

        assert!(
            rendered.contains("state: reopened\n"),
            "a reopened case reports the disposition its latest verification recorded: {rendered}"
        );
        assert!(
            rendered.contains(concat!(
                "  - condition: both consumers call one module\n",
                "    outcome: accepted_exception\n",
                "    exception: the second consumer ships next release\n",
                "    evidence: []\n",
            )),
            "an accepted exception states its reason beneath the result: {rendered}"
        );
        assert!(
            rendered.contains(concat!(
                "    outcome: accepted_exception\n",
                "    exception: the second consumer ships next release\n",
                "    evidence: []\n",
            )),
            "a consumer result carries the same exception shape: {rendered}"
        );
    }

    #[test]
    fn underivable_conditions_print_unknown_rather_than_a_default() {
        let mut case = watching_case();
        case.conditions = conditions(Some(true), None);

        let rendered = show(case, true);

        assert!(
            rendered.contains("privacy_conflicted: true\nstale: unknown\n"),
            "an underived condition is unknown, not false: {rendered}"
        );
    }

    #[test]
    fn case_show_names_what_an_unavailable_portfolio_costs() {
        let rendered = show(watching_case(), false);

        assert!(
            rendered.ends_with(PORTFOLIO_UNAVAILABLE_FOOTER),
            "an unavailable portfolio explains the unknown conditions it caused: {rendered}"
        );
    }

    #[test]
    fn case_list_nests_every_readiness_line_under_its_case() {
        let outcome = ListOutcome {
            cases: vec![review_ready_case(), watching_case()],
            portfolio_available: true,
        };

        assert_eq!(
            outcome.to_string(),
            concat!(
                "cases\n",
                "- case_id: 33333333-3333-4333-8333-333333333333\n",
                "  revision: 2\n",
                "  occurrence_count: 3\n",
                "  state: review-ready\n",
                "  readiness_basis: occurrence-count\n",
                "  readiness: authorizes semantic review; does not authorize extraction\n",
                "  privacy_conflicted: false\n",
                "  stale: false\n",
                "- case_id: 33333333-3333-4333-8333-333333333333\n",
                "  revision: 1\n",
                "  occurrence_count: 2\n",
                "  state: watching\n",
                "  privacy_conflicted: false\n",
                "  stale: false\n",
            ),
            "a listing indents the readiness vocabulary a receipt prints flush"
        );
    }

    #[test]
    fn an_empty_listing_prints_its_heading_and_the_footer_it_owes() {
        let outcome = ListOutcome {
            cases: Vec::new(),
            portfolio_available: false,
        };

        assert_eq!(
            outcome.to_string(),
            format!("cases\n{PORTFOLIO_UNAVAILABLE_FOOTER}"),
            "an empty listing still reports why its conditions are unavailable"
        );
    }

    fn later_event(
        effect: LaterEventEffect,
        headings: LaterEventHeadings,
        state: Option<CaseState>,
        privacy: ReportedPrivacy,
        notice: Option<&'static str>,
    ) -> LaterEventOutcome {
        LaterEventOutcome {
            effect,
            headings,
            case_id: uuid(CASE_ID),
            event_path: PathBuf::from(
                "reuse-evidence/cases/33333333-3333-4333-8333-333333333333/0002-occurrence-appended.toml",
            ),
            revision: 2,
            state,
            privacy,
            notice,
            event: "schema_version = 1\nsequence = 2\n".to_owned(),
        }
    }

    const APPEND_HEADINGS: LaterEventHeadings = LaterEventHeadings {
        preview: "case append preview",
        created: "appended occurrence",
        existing: "occurrence already recorded",
    };

    #[test]
    fn a_preview_receipt_ends_with_the_exact_event_bytes() {
        let outcome = later_event(
            LaterEventEffect::Preview,
            APPEND_HEADINGS,
            Some(CaseState::ReviewReadyByOccurrenceCount),
            ReportedPrivacy::Derived(Visibility::Private),
            None,
        );

        assert_eq!(
            outcome.to_string(),
            concat!(
                "case append preview\n",
                "case_id: 33333333-3333-4333-8333-333333333333\n",
                "file: reuse-evidence/cases/33333333-3333-4333-8333-333333333333/0002-occurrence-appended.toml\n",
                "revision: 2\n",
                "state: review-ready\n",
                "readiness_basis: occurrence-count\n",
                "readiness: authorizes semantic review; does not authorize extraction\n",
                "privacy: private\n",
                "event:\n",
                "schema_version = 1\n",
                "sequence = 2\n",
            ),
            "a preview shows the operator the exact bytes it would record"
        );
    }

    #[test]
    fn a_recorded_receipt_prints_its_notice_and_no_event_bytes() {
        let outcome = later_event(
            LaterEventEffect::Created,
            VerificationDisposition::Closed.headings(),
            Some(CaseState::Closed),
            ReportedPrivacy::Derived(Visibility::Private),
            Some(VerificationDisposition::Closed.notice()),
        );

        assert_eq!(
            outcome.to_string(),
            concat!(
                "recorded verification: closed\n",
                "case_id: 33333333-3333-4333-8333-333333333333\n",
                "file: reuse-evidence/cases/33333333-3333-4333-8333-333333333333/0002-occurrence-appended.toml\n",
                "revision: 2\n",
                "state: closed\n",
                "privacy: private\n",
                "disposition: closed\n",
            ),
            "a recorded event reports what it did without re-printing the bytes"
        );
    }

    #[test]
    fn an_unconfigured_portfolio_and_an_unresolved_participant_refuse_the_same_privacy_differently()
    {
        let unconfigured = later_event(
            LaterEventEffect::Created,
            APPEND_HEADINGS,
            None,
            ReportedPrivacy::PortfolioUnconfigured,
            None,
        );
        let unresolved = later_event(
            LaterEventEffect::Created,
            APPEND_HEADINGS,
            None,
            ReportedPrivacy::ParticipantsUnresolved,
            None,
        );

        assert!(
            unconfigured
                .to_string()
                .ends_with(&format!("privacy: unknown\n{PORTFOLIO_UNAVAILABLE_FOOTER}")),
            "an unconfigured portfolio names the roots the operator must supply"
        );
        assert!(
            unresolved.to_string().ends_with(&format!(
                "privacy: unknown\n{PARTICIPANTS_UNRESOLVED_FOOTER}"
            )),
            "an unresolved participant names the enrollment the operator must restore"
        );
    }

    #[test]
    fn an_opening_receipt_reports_the_opening_sequence_and_no_state() {
        let outcome = OpenOutcome {
            effect: OpenEffect::Preview,
            case_id: uuid(CASE_ID),
            event_path: PathBuf::from(
                "reuse-evidence/cases/33333333-3333-4333-8333-333333333333/0001-case-opened.toml",
            ),
            privacy: Visibility::Private,
            event: "schema_version = 1\n".to_owned(),
        };

        assert_eq!(
            outcome.to_string(),
            concat!(
                "case open preview\n",
                "case_id: 33333333-3333-4333-8333-333333333333\n",
                "file: reuse-evidence/cases/33333333-3333-4333-8333-333333333333/0001-case-opened.toml\n",
                "revision: 1\n",
                "privacy: private\n",
                "event:\n",
                "schema_version = 1\n",
            ),
            "opening records no readiness and always sits at the opening sequence"
        );
    }

    fn brief(content: DecisionContent) -> String {
        let case = decided_case(content.clone());
        BriefOutcome {
            case,
            privacy: ReportedPrivacy::Derived(Visibility::Private),
            authorization: content
                .authorization()
                .expect("the fixture decision resolves its authorization"),
            decision: content,
        }
        .to_string()
    }

    #[test]
    fn an_implementation_authorizing_brief_projects_every_field_the_decision_recorded() {
        assert_eq!(
            brief(change_decision()),
            concat!(
                "implementation brief\n",
                "case_id: 33333333-3333-4333-8333-333333333333\n",
                "privacy: private\n",
                "implementation: authorized\n",
                "accepted_responsibility_identity:\n",
                "  responsibility: one responsibility\n",
                "  verdict: same_responsibility\n",
                "evidence_bearing_consumers:\n",
                "- repository_id: 11111111-1111-4111-8111-111111111111\n",
                "  consumer: billing totals\n",
                "  independence: arose from a separate consumer need\n",
                "  expectation: migrates to the deepened module\n",
                "  evidence:\n",
                "  - kind: commit\n",
                "    reference: abc123\n",
                "    path: src/lib.rs\n",
                "- repository_id: 22222222-2222-4222-8222-222222222222\n",
                "  consumer: invoice totals\n",
                "  independence: arose from a separate consumer need\n",
                "  evidence:\n",
                "  - kind: commit\n",
                "    reference: abc123\n",
                "    path: src/lib.rs\n",
                "- repository_id: 55555555-5555-4555-8555-555555555555\n",
                "  consumer: statement totals\n",
                "  independence: arose from a separate consumer need\n",
                "  evidence:\n",
                "  - kind: commit\n",
                "    reference: abc123\n",
                "    path: src/lib.rs\n",
                "invariant_contract: totals round half to even\n",
                "non_responsibilities:\n",
                "- currency formatting\n",
                "chosen_home_and_scope:\n",
                "  action: extract_or_deepen_locally\n",
                "  scope: one local module\n",
                "alternatives_rejected:\n",
                "- alternative: publish a public package\n",
                "  reason: no consumer outside this repository\n",
                "existing_packages_considered:\n",
                "- package: rust-decimal\n",
                "  fit: partial\n",
                "  reason: does not own the rounding policy\n",
                "required_consumer_level_tests:\n",
                "- billing totals round trip\n",
                "compatibility_and_release_consequences: no recorded evidence changes\n",
                "migration_order:\n",
                "- order: 1\n",
                "  expectation: billing totals first\n",
                "rollback_or_resplitting_strategy: inline the module back\n",
                "verification_conditions:\n",
                "- both consumers call one module\n",
            ),
            "the brief is projected from the decision, and only a matched consumer gains an expectation"
        );
    }

    #[test]
    fn a_no_implementation_brief_omits_the_fields_no_implementation_needs() {
        assert_eq!(
            brief(no_change_decision()),
            concat!(
                "implementation brief\n",
                "case_id: 33333333-3333-4333-8333-333333333333\n",
                "privacy: private\n",
                "implementation: not authorized\n",
                "decision: authorizes no implementation\n",
                "accepted_responsibility_identity:\n",
                "  responsibility: one responsibility\n",
                "  verdict: different_responsibilities\n",
                "chosen_home_and_scope:\n",
                "  action: retain_intentional_duplication\n",
                "  scope: no shared surface\n",
                "non_responsibilities:\n",
                "- rounding policy\n",
                "alternatives_rejected:\n",
                "- alternative: extract locally\n",
                "  reason: the two totals change for different reasons\n",
                "compatibility_and_release_consequences: nothing changes\n",
                "verification_conditions:\n",
                "- neither consumer gains a dependency\n",
            ),
            "a decision authorizing no implementation prints no consumers, packages, or migration order"
        );
    }
}
