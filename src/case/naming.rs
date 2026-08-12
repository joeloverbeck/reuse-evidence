use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(super) const OPENING_SEQUENCE: i64 = 1;
pub(super) const MAX_CASE_SEQUENCE: i64 = 9_999;

/// The steward-relative root beneath which every case's event stream sits.
///
/// ADR 0009 fixes the layout as `reuse-evidence/cases/<case-id>/`, beside the
/// root marker. Like the event file-name grammar, it cannot vary by event type
/// or by command without changing recorded evidence, so it is stated once here
/// rather than at each site that resolves a case.
pub(crate) fn cases_root() -> &'static Path {
    Path::new("reuse-evidence/cases")
}

/// The steward-relative directory holding one case's event stream.
pub(crate) fn case_directory(case_id: Uuid) -> PathBuf {
    cases_root().join(case_id.to_string())
}

/// Where in a case's event stream one event type may appear.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EventPosition {
    Opening,
    Later,
}

impl EventPosition {
    /// Whether an event at this position may occupy `sequence`.
    ///
    /// Callers bound the sequence against the four-digit layout first; this
    /// states only which end of that range the position claims.
    pub(super) const fn permits_sequence(self, sequence: i64) -> bool {
        match self {
            Self::Opening => sequence == OPENING_SEQUENCE,
            Self::Later => sequence > OPENING_SEQUENCE,
        }
    }
}

macro_rules! define_event_types {
    ($(
        $variant:ident => {
            body: $body:literal,
            slug: $slug:literal,
            position: $position:ident
        }
    ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
        pub(super) enum EventType {
            $(
                #[serde(rename = $body)]
                $variant
            ),+
        }

        impl EventType {
            pub(super) const fn body_name(self) -> &'static str {
                match self {
                    $( Self::$variant => $body ),+
                }
            }

            const fn slug(self) -> &'static str {
                match self {
                    $( Self::$variant => $slug ),+
                }
            }

            fn from_slug(slug: &str) -> Option<Self> {
                match slug {
                    $( $slug => Some(Self::$variant), )+
                    _ => None,
                }
            }

            /// Where in a case's event stream this type may appear.
            ///
            /// Declared beside the type's spellings so a later event type
            /// cannot be added without stating it. Two rules read it: which
            /// sequence the type may occupy, and which staged temporary names
            /// belong to which event class.
            pub(super) const fn position(self) -> EventPosition {
                match self {
                    $( Self::$variant => EventPosition::$position ),+
                }
            }
        }
    };
}

define_event_types! {
    CaseOpened => {
        body: "case_opened",
        slug: "case-opened",
        position: Opening
    },
    OccurrenceAppended => {
        body: "occurrence_appended",
        slug: "occurrence-appended",
        position: Later
    },
    EarlyReviewAuthorized => {
        body: "early_review_authorized",
        slug: "early-review-authorized",
        position: Later
    },
    ReuseDecisionAccepted => {
        body: "reuse_decision_accepted",
        slug: "reuse-decision-accepted",
        position: Later
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EventFileName {
    sequence: i64,
    event_type: EventType,
}

impl EventFileName {
    pub(super) const fn new(sequence: i64, event_type: EventType) -> Option<Self> {
        if sequence < OPENING_SEQUENCE || sequence > MAX_CASE_SEQUENCE {
            return None;
        }
        if !event_type.position().permits_sequence(sequence) {
            return None;
        }
        Some(Self {
            sequence,
            event_type,
        })
    }

    pub(super) fn parse(file_name: &str) -> Option<Self> {
        let (sequence, slug) = file_name_components(file_name)?;
        Self::new(sequence, EventType::from_slug(slug)?)
    }

    pub(super) const fn sequence(self) -> i64 {
        self.sequence
    }

    pub(super) const fn event_type(self) -> EventType {
        self.event_type
    }
}

pub(super) fn sequence_from_file_name(file_name: &str) -> Option<i64> {
    file_name_components(file_name).map(|(sequence, _)| sequence)
}

fn file_name_components(file_name: &str) -> Option<(i64, &str)> {
    let (sequence, slug) = file_name.strip_suffix(".toml")?.split_once('-')?;
    if sequence.len() != 4 || !sequence.bytes().all(|byte| byte.is_ascii_digit()) || slug.is_empty()
    {
        return None;
    }
    sequence
        .parse()
        .ok()
        .filter(|sequence| (OPENING_SEQUENCE..=MAX_CASE_SEQUENCE).contains(sequence))
        .map(|sequence| (sequence, slug))
}

impl fmt::Display for EventFileName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{}.toml",
            self.sequence,
            self.event_type.slug()
        )
    }
}

pub(super) fn is_staged_temporary(file_name: &std::ffi::OsStr, position: EventPosition) -> bool {
    let Some(staged_name) = file_name
        .to_str()
        .and_then(|name| name.strip_prefix('.'))
        .and_then(|name| name.strip_suffix(".tmp"))
    else {
        return false;
    };
    let Some((event_file_name, identity)) = staged_name.rsplit_once('.') else {
        return false;
    };
    Uuid::parse_str(identity).is_ok()
        && EventFileName::parse(event_file_name)
            .is_some_and(|identity| identity.event_type().position() == position)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::{EventFileName, EventPosition, EventType, is_staged_temporary};

    #[test]
    fn constructs_the_accepted_name_for_each_event_type() {
        let cases = [
            (1, EventType::CaseOpened, "0001-case-opened.toml"),
            (
                2,
                EventType::OccurrenceAppended,
                "0002-occurrence-appended.toml",
            ),
            (
                2,
                EventType::EarlyReviewAuthorized,
                "0002-early-review-authorized.toml",
            ),
        ];

        for (sequence, event_type, expected) in cases {
            let name = EventFileName::new(sequence, event_type)
                .expect("the accepted sequence should construct an event file name");
            assert_eq!(name.to_string(), expected);
        }
    }

    #[test]
    fn parses_each_accepted_name_back_to_its_envelope_identity() {
        let cases = [
            ("0001-case-opened.toml", 1, EventType::CaseOpened),
            (
                "0002-occurrence-appended.toml",
                2,
                EventType::OccurrenceAppended,
            ),
            (
                "0003-early-review-authorized.toml",
                3,
                EventType::EarlyReviewAuthorized,
            ),
        ];

        for (name, expected_sequence, expected_event_type) in cases {
            let identity = EventFileName::parse(name)
                .expect("an accepted event file name should parse back to its identity");
            assert_eq!(identity.sequence(), expected_sequence);
            assert_eq!(identity.event_type(), expected_event_type);
        }
    }

    #[test]
    fn accepted_reuse_decision_round_trips_through_the_naming_owner() {
        #[derive(serde::Serialize)]
        struct Envelope {
            event_type: EventType,
        }

        let name = EventFileName::new(4, EventType::ReuseDecisionAccepted)
            .expect("an accepted reuse decision is a later case event");

        assert_eq!(name.to_string(), "0004-reuse-decision-accepted.toml");
        assert_eq!(
            EventFileName::parse("0004-reuse-decision-accepted.toml"),
            Some(name)
        );
        assert_eq!(
            EventFileName::new(1, EventType::ReuseDecisionAccepted),
            None,
            "an accepted reuse decision cannot occupy the opening sequence"
        );
        assert_eq!(
            toml::to_string(&Envelope {
                event_type: EventType::ReuseDecisionAccepted,
            })
            .expect("the accepted reuse decision type should serialize"),
            "event_type = \"reuse_decision_accepted\"\n"
        );
        let staged = format!(".{name}.{}.tmp", "00000000-0000-4000-8000-000000000001");
        assert!(is_staged_temporary(
            OsStr::new(&staged),
            EventPosition::Later
        ));
        assert!(!is_staged_temporary(
            OsStr::new(&staged),
            EventPosition::Opening
        ));
    }

    #[test]
    fn rejects_sequences_and_slugs_outside_the_accepted_grammar() {
        for (sequence, event_type) in [
            (0, EventType::CaseOpened),
            (2, EventType::CaseOpened),
            (1, EventType::OccurrenceAppended),
            (1, EventType::EarlyReviewAuthorized),
            (10_000, EventType::OccurrenceAppended),
        ] {
            assert!(
                EventFileName::new(sequence, event_type).is_none(),
                "sequence {sequence} should be invalid for {event_type:?}"
            );
        }

        for malformed in [
            "0000-case-opened.toml",
            "0002-case-opened.toml",
            "0001-occurrence-appended.toml",
            "0001-early-review-authorized.toml",
            "001-occurrence-appended.toml",
            "10000-occurrence-appended.toml",
            "0002-.toml",
            "0002-unknown.toml",
            "0002-occurrence-appended",
        ] {
            assert!(
                EventFileName::parse(malformed).is_none(),
                "`{malformed}` should not parse as an event file name"
            );
        }
    }

    #[test]
    fn recognizes_only_accepted_staged_case_event_names() {
        let identity = "00000000-0000-4000-8000-000000000088";
        for (position, accepted) in [
            (
                EventPosition::Opening,
                format!(".0001-case-opened.toml.{identity}.tmp"),
            ),
            (
                EventPosition::Later,
                format!(".0002-occurrence-appended.toml.{identity}.tmp"),
            ),
            (
                EventPosition::Later,
                format!(".0003-early-review-authorized.toml.{identity}.tmp"),
            ),
        ] {
            assert!(is_staged_temporary(OsStr::new(&accepted), position));
        }

        for (position, rejected) in [
            (
                EventPosition::Opening,
                format!(".0002-occurrence-appended.toml.{identity}.tmp"),
            ),
            (
                EventPosition::Later,
                format!(".0001-case-opened.toml.{identity}.tmp"),
            ),
            (
                EventPosition::Opening,
                format!(".0002-case-opened.toml.{identity}.tmp"),
            ),
            (
                EventPosition::Later,
                format!(".0001-occurrence-appended.toml.{identity}.tmp"),
            ),
            (
                EventPosition::Later,
                format!(".0002-unknown.toml.{identity}.tmp"),
            ),
            (
                EventPosition::Later,
                ".0002-occurrence-appended.toml.not-a-uuid.tmp".to_owned(),
            ),
            (
                EventPosition::Later,
                format!("0002-occurrence-appended.toml.{identity}.tmp"),
            ),
            (
                EventPosition::Later,
                format!(".0002-occurrence-appended.toml.{identity}"),
            ),
        ] {
            assert!(!is_staged_temporary(OsStr::new(&rejected), position));
        }
    }
}
