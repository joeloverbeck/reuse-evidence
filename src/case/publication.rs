use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use uuid::Uuid;

use crate::{CreateFileOutcome, TerminalFailure, create_file_atomically_if_absent};

use super::naming::{EventFileName, EventType, MAX_CASE_SEQUENCE, OPENING_SEQUENCE};

pub(super) trait RevisionedCase {
    fn revision(&self) -> i64;
}

pub(super) struct Publication {
    expected_revision: i64,
    sequence: i64,
}

#[derive(Clone, Copy)]
pub(super) struct PublicationTarget<'a> {
    pub(super) repository_root: &'a Path,
    pub(super) relative_case_directory: &'a Path,
    pub(super) relative_event_path: &'a Path,
}

#[derive(Clone, Copy)]
pub(super) struct PreparedEvent<'a> {
    pub(super) event_id: Option<Uuid>,
    pub(super) bytes: &'a str,
}

pub(super) enum PublicationOutcome<C, V> {
    Created { case: C, validation: V },
    Existing { case: C, event: ExistingEvent },
}

pub(super) struct ExistingEvent {
    pub(super) sequence: i64,
    pub(super) bytes: String,
}

#[derive(Debug)]
pub(super) enum ExistingEventFailure {
    Unreadable {
        path: PathBuf,
        error: io::Error,
    },
    Invalid {
        path: PathBuf,
        error: toml::de::Error,
    },
    IdentityConflict {
        recorded_sequence: i64,
        recorded_event_id: Uuid,
        prepared_event_id: Option<Uuid>,
        current_revision: i64,
    },
    ContentDrift {
        recorded_event_id: Uuid,
    },
}

#[derive(Debug)]
pub(super) enum PublicationFailure {
    Protocol(TerminalFailure),
    ExistingEvent(ExistingEventFailure),
    RevisionConflict {
        expected_revision: i64,
        current_revision: i64,
    },
}

#[derive(Deserialize)]
struct EventEnvelope {
    sequence: i64,
    event_id: Uuid,
}

struct LockedPublication<C> {
    _opening_event: File,
    case: C,
}

impl<C> LockedPublication<C> {
    fn acquire(
        repository_root: &Path,
        relative_case_directory: &Path,
        mut read_case: impl FnMut() -> Result<C, TerminalFailure>,
    ) -> Result<Self, TerminalFailure> {
        let opening_event_path = repository_root.join(relative_case_directory).join(
            EventFileName::new(OPENING_SEQUENCE, EventType::CaseOpened)
                .expect("the opening event has an accepted file identity")
                .to_string(),
        );
        let opening_event = File::open(&opening_event_path).map_err(|error| {
            TerminalFailure::refusal(
                format!(
                    "case opening event `{}` cannot be opened for write serialization: {error}",
                    opening_event_path.display()
                ),
                "restore the immutable opening event and retry the later case event",
            )
        })?;
        opening_event.lock().map_err(|error| {
            TerminalFailure::refusal(
                format!(
                    "case opening event `{}` cannot serialize a later case event: {error}",
                    opening_event_path.display()
                ),
                "make the immutable opening event lockable and retry the later case event",
            )
        })?;
        let case = read_case()?;
        Ok(Self {
            _opening_event: opening_event,
            case,
        })
    }

    // The receiver is the proof that the opening-event lock is still held.
    #[allow(clippy::unused_self)]
    fn create_event(
        &self,
        absolute_event_path: &Path,
        event: &[u8],
    ) -> Result<CreateFileOutcome, TerminalFailure> {
        create_file_atomically_if_absent(absolute_event_path, event)
    }
}

pub(super) fn existing_event<C: RevisionedCase>(
    case: &C,
    absolute_event_path: &Path,
    prepared_event_id: Option<Uuid>,
    proposed_event: &str,
) -> Result<ExistingEvent, ExistingEventFailure> {
    let recorded_bytes = fs::read_to_string(absolute_event_path).map_err(|error| {
        ExistingEventFailure::Unreadable {
            path: absolute_event_path.to_path_buf(),
            error,
        }
    })?;
    let recorded = toml::from_str::<EventEnvelope>(&recorded_bytes).map_err(|error| {
        ExistingEventFailure::Invalid {
            path: absolute_event_path.to_path_buf(),
            error,
        }
    })?;
    if prepared_event_id != Some(recorded.event_id) {
        return Err(ExistingEventFailure::IdentityConflict {
            recorded_sequence: recorded.sequence,
            recorded_event_id: recorded.event_id,
            prepared_event_id,
            current_revision: case.revision(),
        });
    }
    if recorded_bytes != proposed_event {
        return Err(ExistingEventFailure::ContentDrift {
            recorded_event_id: recorded.event_id,
        });
    }
    Ok(ExistingEvent {
        sequence: recorded.sequence,
        bytes: recorded_bytes,
    })
}

impl Publication {
    pub(super) fn new(expected_revision: i64) -> Result<Self, TerminalFailure> {
        if expected_revision < OPENING_SEQUENCE {
            return Err(TerminalFailure::refusal(
                format!("expected revision `{expected_revision}` is not a recorded case revision"),
                "use the positive revision reported by `case show`",
            ));
        }
        if expected_revision >= MAX_CASE_SEQUENCE {
            return Err(TerminalFailure::refusal(
                format!(
                    "expected revision {expected_revision} cannot be appended because the accepted `NNNN` event layout ends at revision {MAX_CASE_SEQUENCE}"
                ),
                "preserve the case unchanged and obtain an accepted event-layout amendment before appending another occurrence",
            ));
        }
        Ok(Self {
            expected_revision,
            sequence: expected_revision + 1,
        })
    }

    pub(super) const fn sequence(&self) -> i64 {
        self.sequence
    }

    pub(super) fn publish<C, B, V>(
        self,
        target: PublicationTarget<'_>,
        event: PreparedEvent<'_>,
        mut read_case: impl FnMut() -> Result<C, TerminalFailure>,
        before_revision: impl FnOnce(&C) -> Result<B, TerminalFailure>,
        after_revision: impl FnOnce(&C, B) -> Result<V, TerminalFailure>,
    ) -> Result<PublicationOutcome<C, V>, PublicationFailure>
    where
        C: RevisionedCase,
    {
        let mut locked = LockedPublication::acquire(
            target.repository_root,
            target.relative_case_directory,
            &mut read_case,
        )
        .map_err(PublicationFailure::Protocol)?;
        let absolute_event_path = target.repository_root.join(target.relative_event_path);
        if absolute_event_path.exists() {
            let existing = existing_event(
                &locked.case,
                &absolute_event_path,
                event.event_id,
                event.bytes,
            )
            .map_err(PublicationFailure::ExistingEvent)?;
            return Ok(PublicationOutcome::Existing {
                case: locked.case,
                event: existing,
            });
        }
        let before_revision =
            before_revision(&locked.case).map_err(PublicationFailure::Protocol)?;
        if locked.case.revision() != self.expected_revision {
            return Err(PublicationFailure::RevisionConflict {
                expected_revision: self.expected_revision,
                current_revision: locked.case.revision(),
            });
        }
        let validation =
            after_revision(&locked.case, before_revision).map_err(PublicationFailure::Protocol)?;
        match locked
            .create_event(&absolute_event_path, event.bytes.as_bytes())
            .map_err(PublicationFailure::Protocol)?
        {
            CreateFileOutcome::Created => Ok(PublicationOutcome::Created {
                case: locked.case,
                validation,
            }),
            CreateFileOutcome::Occupied => {
                locked.case = read_case().map_err(PublicationFailure::Protocol)?;
                let existing = existing_event(
                    &locked.case,
                    &absolute_event_path,
                    event.event_id,
                    event.bytes,
                )
                .map_err(PublicationFailure::ExistingEvent)?;
                Ok(PublicationOutcome::Existing {
                    case: locked.case,
                    event: existing,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use uuid::Uuid;

    use super::*;

    const CASE_ID: Uuid = Uuid::from_u128(0x00000000_0000_4000_8000_000000000011);
    const EVENT_ID: Uuid = Uuid::from_u128(0x00000000_0000_4000_8000_000000000099);
    const DIFFERENT_EVENT_ID: Uuid = Uuid::from_u128(0x00000000_0000_4000_8000_000000000098);

    struct Fixture {
        root: PathBuf,
        relative_case_directory: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after the Unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "reuse-evidence-publication-{name}-{}-{nonce}",
                std::process::id()
            ));
            let relative_case_directory =
                PathBuf::from("reuse-evidence/cases").join(CASE_ID.to_string());
            fs::create_dir_all(root.join(&relative_case_directory))
                .expect("case directory should be creatable");
            fs::write(
                root.join(&relative_case_directory).join(
                    EventFileName::new(OPENING_SEQUENCE, EventType::CaseOpened)
                        .expect("the opening event has an accepted file identity")
                        .to_string(),
                ),
                "immutable opening event\n",
            )
            .expect("opening event should be writable");
            Self {
                root,
                relative_case_directory,
            }
        }

        fn event_path(&self, event_type: EventType) -> PathBuf {
            self.relative_case_directory.join(
                EventFileName::new(2, event_type)
                    .expect("the later event has an accepted file identity")
                    .to_string(),
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct TestCase {
        revision: i64,
    }

    impl RevisionedCase for TestCase {
        fn revision(&self) -> i64 {
            self.revision
        }
    }

    fn event_bytes(event_id: Uuid) -> String {
        format!(
            "schema_version = 1\nsequence = 2\nevent_id = \"{event_id}\"\nevent_type = \"occurrence_appended\"\nrecorded_at = \"2026-08-10T12:00:00Z\"\npayload = \"typed event\"\n"
        )
    }

    #[test]
    fn publish_creates_typed_sequence_file() {
        let fixture = Fixture::new("create");
        let event_path = fixture.event_path(EventType::OccurrenceAppended);
        let event = event_bytes(EVENT_ID);
        let publication = Publication::new(1).expect("expected revision should be valid");

        let outcome = publication
            .publish(
                PublicationTarget {
                    repository_root: &fixture.root,
                    relative_case_directory: &fixture.relative_case_directory,
                    relative_event_path: &event_path,
                },
                PreparedEvent {
                    event_id: Some(EVENT_ID),
                    bytes: &event,
                },
                || Ok(TestCase { revision: 1 }),
                |_| Ok(()),
                |_, ()| Ok("validated"),
            )
            .expect("publication should create the event");

        assert!(matches!(
            outcome,
            PublicationOutcome::Created {
                validation: "validated",
                ..
            }
        ));
        assert_eq!(
            fs::read_to_string(fixture.root.join(event_path))
                .expect("published event should be readable"),
            event
        );
    }

    #[test]
    fn occupied_path_returns_existing() {
        let fixture = Fixture::new("occupied");
        let event_path = fixture.event_path(EventType::OccurrenceAppended);
        fs::write(fixture.root.join(&event_path), "recorded event\n")
            .expect("occupied event should be writable");
        let locked =
            LockedPublication::acquire(&fixture.root, &fixture.relative_case_directory, || {
                Ok(TestCase { revision: 1 })
            })
            .expect("publication should lock and re-read the case");

        let outcome = locked
            .create_event(&fixture.root.join(event_path), b"proposed event\n")
            .expect("occupied event should be readable");

        assert!(matches!(outcome, CreateFileOutcome::Occupied));
    }

    #[test]
    fn matching_identity_and_bytes_is_idempotent() {
        let fixture = Fixture::new("idempotent");
        let event_path = fixture.event_path(EventType::OccurrenceAppended);
        let event = event_bytes(EVENT_ID);
        fs::write(fixture.root.join(&event_path), &event)
            .expect("recorded event should be writable");
        let publication = Publication::new(1).expect("expected revision should be valid");

        let outcome = publication
            .publish(
                PublicationTarget {
                    repository_root: &fixture.root,
                    relative_case_directory: &fixture.relative_case_directory,
                    relative_event_path: &event_path,
                },
                PreparedEvent {
                    event_id: Some(EVENT_ID),
                    bytes: &event,
                },
                || Ok(TestCase { revision: 2 }),
                |_| -> Result<(), TerminalFailure> {
                    panic!("an exact retry must not repeat event eligibility")
                },
                |_, ()| Ok("unreachable"),
            )
            .expect("matching identity and bytes should be idempotent");

        assert!(matches!(
            outcome,
            PublicationOutcome::Existing {
                event: ExistingEvent {
                    sequence: 2,
                    bytes,
                },
                ..
            } if bytes == event
        ));
    }

    #[test]
    fn different_identity_is_a_revision_conflict() {
        let fixture = Fixture::new("identity-conflict");
        let event_path = fixture.event_path(EventType::OccurrenceAppended);
        fs::write(fixture.root.join(&event_path), event_bytes(EVENT_ID))
            .expect("recorded event should be writable");
        let publication = Publication::new(1).expect("expected revision should be valid");

        let result = publication.publish(
            PublicationTarget {
                repository_root: &fixture.root,
                relative_case_directory: &fixture.relative_case_directory,
                relative_event_path: &event_path,
            },
            PreparedEvent {
                event_id: Some(DIFFERENT_EVENT_ID),
                bytes: &event_bytes(DIFFERENT_EVENT_ID),
            },
            || Ok(TestCase { revision: 2 }),
            |_| Ok(()),
            |_, ()| Ok(()),
        );
        assert!(
            matches!(
                result,
                Err(PublicationFailure::ExistingEvent(
                    ExistingEventFailure::IdentityConflict {
                        recorded_sequence: 2,
                        recorded_event_id: EVENT_ID,
                        prepared_event_id: Some(DIFFERENT_EVENT_ID),
                        current_revision: 2,
                    }
                ))
            ),
            "a different identity must return invariant conflict data"
        );
    }

    #[test]
    fn matching_identity_with_drifted_bytes_refuses() {
        let fixture = Fixture::new("content-drift");
        let event_path = fixture.event_path(EventType::OccurrenceAppended);
        let recorded = event_bytes(EVENT_ID);
        fs::write(fixture.root.join(&event_path), &recorded)
            .expect("recorded event should be writable");
        let proposed = recorded.replace("typed event", "drifted event");
        let publication = Publication::new(1).expect("expected revision should be valid");

        let result = publication.publish(
            PublicationTarget {
                repository_root: &fixture.root,
                relative_case_directory: &fixture.relative_case_directory,
                relative_event_path: &event_path,
            },
            PreparedEvent {
                event_id: Some(EVENT_ID),
                bytes: &proposed,
            },
            || Ok(TestCase { revision: 2 }),
            |_| Ok(()),
            |_, ()| Ok(()),
        );
        assert!(
            matches!(
                result,
                Err(PublicationFailure::ExistingEvent(
                    ExistingEventFailure::ContentDrift {
                        recorded_event_id: EVENT_ID,
                    }
                ))
            ),
            "matching identity with different bytes must return invariant drift data"
        );
    }

    #[test]
    fn stale_expected_revision_refuses_without_writing() {
        let fixture = Fixture::new("stale-revision");
        let event_path = fixture.event_path(EventType::OccurrenceAppended);
        let publication = Publication::new(1).expect("expected revision should be valid");

        let result = publication.publish(
            PublicationTarget {
                repository_root: &fixture.root,
                relative_case_directory: &fixture.relative_case_directory,
                relative_event_path: &event_path,
            },
            PreparedEvent {
                event_id: Some(EVENT_ID),
                bytes: &event_bytes(EVENT_ID),
            },
            || Ok(TestCase { revision: 2 }),
            |_| Ok(()),
            |_, ()| -> Result<(), TerminalFailure> {
                panic!("stale publication must refuse before event eligibility")
            },
        );
        assert!(
            matches!(
                result,
                Err(PublicationFailure::RevisionConflict {
                    expected_revision: 1,
                    current_revision: 2,
                })
            ),
            "stale expected revision must return invariant conflict data"
        );
        assert!(
            !fixture.root.join(event_path).exists(),
            "stale publication must not create an event"
        );
    }
}
