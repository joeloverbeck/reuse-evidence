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
