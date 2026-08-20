//! The type-independent envelope every case event records.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::CASE_SCHEMA_VERSION;
use super::instant::{MalformedInstant, RecordedInstant};
use super::naming::EventType;
use crate::TerminalFailure;

/// The type-independent part of every case event.
///
/// Every event type records these five fields in this order and nothing else
/// distinguishes their envelopes, so the shape, its order, and its validity
/// have one owner. Each event type flattens this into its own body, which is
/// why the recorded bytes are unchanged by the envelope having a type.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct Envelope {
    pub(super) schema_version: i64,
    pub(super) sequence: i64,
    pub(super) event_id: Uuid,
    pub(super) event_type: EventType,
    pub(super) recorded_at: String,
}

/// The wording one event type uses when its envelope is refused.
///
/// The envelope rule is shared; the sentences are not. Each event type names
/// itself and the preview command that renders it, so collapsing the rule does
/// not collapse the terminal text a consumer already depends on.
pub(super) struct EnvelopeRefusal<'a> {
    /// The complete condition for an envelope that is not this event type.
    pub(super) unsupported: &'a str,
    /// The event noun in the identity refusal, such as `opening`.
    pub(super) noun: &'a str,
    /// The event name in the timestamp refusal, such as `early-review authorization`.
    pub(super) instant_name: &'a str,
    /// The preview command every one of these refusals resolves to.
    pub(super) preview_command: &'a str,
}

impl Envelope {
    /// Whether a proposal claims to be a prepared recorded event.
    ///
    /// Human proposal shapes declare none of the envelope fields. Seeing any
    /// one of them therefore selects the prepared shape, including when the
    /// claimed envelope is incomplete and needs a field-specific refusal.
    pub(super) fn is_claimed_by(table: &toml::Table) -> bool {
        [
            "schema_version",
            "sequence",
            "event_id",
            "event_type",
            "recorded_at",
        ]
        .iter()
        .any(|field| table.contains_key(*field))
    }

    /// Render the envelope for an event this run is about to record.
    ///
    /// The schema version and the fresh event identity are the envelope's to
    /// choose; the sequence, the type, and the instant are the caller's.
    pub(super) fn new(sequence: i64, event_type: EventType, recorded_at: RecordedInstant) -> Self {
        Self {
            schema_version: CASE_SCHEMA_VERSION,
            sequence,
            event_id: Uuid::new_v4(),
            event_type,
            recorded_at: recorded_at.to_string(),
        }
    }

    /// Refuse an envelope that is not a supported event of this type.
    ///
    /// The checks run in the order a consumer already sees them: the supported
    /// shape, then the event identity, then the recorded instant.
    pub(super) fn validate(
        &self,
        event_type: EventType,
        refusal: &EnvelopeRefusal<'_>,
    ) -> Result<(), TerminalFailure> {
        if self.schema_version != CASE_SCHEMA_VERSION
            || !event_type.position().permits_sequence(self.sequence)
            || self.event_type != event_type
        {
            return Err(TerminalFailure::refusal(
                refusal.unsupported,
                format!(
                    "use the exact event rendered by `{}`",
                    refusal.preview_command
                ),
            ));
        }
        if self.event_id.get_version_num() != 4 {
            return Err(TerminalFailure::refusal(
                format!(
                    "prepared {} event identity `{}` is not an opaque UUID version 4",
                    refusal.noun, self.event_id
                ),
                format!(
                    "use the exact event rendered by `{}`",
                    refusal.preview_command
                ),
            ));
        }
        self.recorded_instant(refusal).map(|_| ())
    }

    /// Read the recorded instant, refusing a value no reader can recover.
    ///
    /// `recorded_at` stays a string in the recorded shape because its refusal
    /// names the event and its preview command, which a deserialization error
    /// cannot. This is the one place that string becomes an instant.
    fn recorded_instant(
        &self,
        refusal: &EnvelopeRefusal<'_>,
    ) -> Result<RecordedInstant, TerminalFailure> {
        RecordedInstant::parse(&self.recorded_at).map_err(|failure| {
            let condition = match failure {
                MalformedInstant::NotRfc3339 => format!(
                    "prepared {} event timestamp `{}` is not UTC RFC 3339",
                    refusal.instant_name, self.recorded_at
                ),
                MalformedInstant::NotAValidInstant => format!(
                    "prepared {} event timestamp `{}` is not a valid UTC instant",
                    refusal.instant_name, self.recorded_at
                ),
            };
            TerminalFailure::refusal(
                condition,
                format!(
                    "use the exact event rendered by `{}`",
                    refusal.preview_command
                ),
            )
        })
    }
}
