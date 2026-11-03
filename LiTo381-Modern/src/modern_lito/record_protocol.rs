//! v5 central persisted match totals. Not a prediction from the current match.
//! Failure is distinct from a successful lookup with no stored row.
use super::application_protocol::{self as app, ApplicationError, SelectionContext};
use std::sync::Arc;

pub const REQUEST_KIND: u16 = 28;
pub const RESULT_KIND: u16 = 29;
pub const RESULT_BODY_BYTES: usize = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok,
    Denied,
    Busy,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Totals {
    pub stored: bool,
    pub wins: i64,
    pub losses: i64,
    pub revision: i64,
}
impl Totals {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if self.wins < 0
            || self.losses < 0
            || self.revision < 0
            || self.wins.checked_add(self.losses) != Some(self.revision)
            || (!self.stored && (self.wins != 0 || self.losses != 0 || self.revision != 0))
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub context: SelectionContext,
    pub status: Status,
    pub totals: Option<Totals>,
}
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context) || (self.status == Status::Ok) != self.totals.is_some()
        {
            return Err(invalid());
        }
        if let Some(totals) = self.totals {
            totals.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    NotRequested,
    Unsupported,
    Ready {
        confirmed: Arc<Reply>,
        refresh_failure: Option<Status>,
    },
    Unavailable {
        context: SelectionContext,
        status: Status,
    },
}
impl State {
    pub fn context(&self) -> Option<SelectionContext> {
        match self {
            Self::Ready { confirmed, .. } => Some(confirmed.context),
            Self::Unavailable { context, .. } => Some(*context),
            _ => None,
        }
    }
    pub fn totals(&self) -> Option<Totals> {
        match self {
            Self::Ready { confirmed, .. } => confirmed.totals,
            _ => None,
        }
    }
    /// Call only after validating the reply against the current selected context.
    /// A failed refresh may retain exactly that context, never another character.
    pub fn receive(&mut self, reply: Reply) -> Result<(), ApplicationError> {
        reply.validate()?;
        if reply.status == Status::Ok {
            if let Self::Ready { confirmed, .. } = self {
                if confirmed.context == reply.context {
                    // The central writer only appends settled matches. Validate
                    // before swapping, including after a failed refresh retained
                    // the prior snapshot. A new selection has a fresh baseline.
                    confirmed.validate()?;
                    let before = confirmed.totals.ok_or_else(invalid)?;
                    let after = reply.totals.ok_or_else(invalid)?;
                    if before.stored && !after.stored
                        || after.wins < before.wins
                        || after.losses < before.losses
                        || after.revision < before.revision
                        || after.revision == before.revision && after != before
                    {
                        return Err(invalid());
                    }
                }
            }
            *self = Self::Ready {
                confirmed: Arc::new(reply),
                refresh_failure: None,
            };
        } else if let Self::Ready {
            confirmed,
            refresh_failure,
        } = self
        {
            if confirmed.context == reply.context {
                *refresh_failure = Some(reply.status);
            } else {
                *self = Self::Unavailable {
                    context: reply.context,
                    status: reply.status,
                };
            }
        } else {
            *self = Self::Unavailable {
                context: reply.context,
                status: reply.status,
            };
        }
        Ok(())
    }
}

fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid central record frame",
    )
}
pub fn encode_request(id: u32, context: SelectionContext) -> Result<Vec<u8>, ApplicationError> {
    if !app::valid_context(context) {
        return Err(invalid());
    }
    let mut bytes = app::header(id, REQUEST_KIND, 36)?;
    bytes.extend_from_slice(&context.key.source_id.to_be_bytes());
    bytes.extend_from_slice(&context.key.object_id.to_be_bytes());
    bytes.extend_from_slice(&context.epoch.to_be_bytes());
    Ok(bytes)
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(5..=super::protocol::MAX_VERSION).contains(&version) || bytes.len() != RESULT_BODY_BYTES {
        return Err(invalid());
    }
    // Length is exact before all fixed slices; no allocation or permissive coercion.
    let context = SelectionContext {
        key: app::CharacterKey {
            source_id: u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            object_id: u64::from_be_bytes(bytes[4..12].try_into().unwrap()),
        },
        epoch: u64::from_be_bytes(bytes[12..20].try_into().unwrap()),
    };
    let status = match bytes[20] {
        0 => Status::Ok,
        1 => Status::Denied,
        2 => Status::Busy,
        3 => Status::Unavailable,
        _ => return Err(invalid()),
    };
    if bytes[21] > 1 || bytes[22..24] != [0, 0] {
        return Err(invalid());
    }
    let totals = Totals {
        stored: bytes[21] == 1,
        wins: i64::from_be_bytes(bytes[24..32].try_into().unwrap()),
        losses: i64::from_be_bytes(bytes[32..40].try_into().unwrap()),
        revision: i64::from_be_bytes(bytes[40..48].try_into().unwrap()),
    };
    totals.validate()?;
    if status != Status::Ok && (totals.stored || totals.revision != 0) {
        return Err(invalid());
    }
    let reply = Reply {
        context,
        status,
        totals: (status == Status::Ok).then_some(totals),
    };
    reply.validate()?;
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> SelectionContext {
        SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        }
    }
    fn hex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .chunks_exact(2)
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect()
    }
    // Literal independent big-endian fixture: 4_000_000_000 wins, 3 losses.
    const REQUEST: &str =
        "4c544d3100000024fedcba98001c0000000000070000000000000d05ffffffffffffffff";
    const RESULT: &str = "4c544d3100000040fedcba98001d0000000000070000000000000d05ffffffffffffffff0001000000000000ee6b2800000000000000000300000000ee6b2803";
    #[test]
    fn literal_i64_and_full_epoch_golden_only_negotiated_v5() {
        assert_eq!(encode_request(0xfedcba98, context()).unwrap(), hex(REQUEST));
        let bytes = hex(RESULT);
        let reply = decode_body(&bytes[16..], 5).unwrap();
        assert_eq!(reply.context, context());
        assert_eq!(
            reply.totals.unwrap(),
            Totals {
                stored: true,
                wins: 4_000_000_000,
                losses: 3,
                revision: 4_000_000_003
            }
        );
        let frame = app::decode_response_version(&bytes, 5).unwrap();
        assert_eq!(frame.request_id, 0xfedcba98);
        assert_eq!(frame.response, app::Response::Record(reply));
        assert_eq!(app::decode_response_version(&bytes, 6).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 7).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 8).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 9).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 10).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 11).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 12).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 13).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 14).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 15).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 16).unwrap(), frame);
        assert_eq!(app::decode_response_version(&bytes, 17).unwrap(), frame);
        for version in [0, 1, 2, 3, 4, super::super::protocol::MAX_VERSION + 1] {
            assert!(app::decode_response_version(&bytes, version).is_err());
        }
        assert!(encode_request(0, context()).is_err());
        assert!(encode_request(
            1,
            SelectionContext {
                epoch: 0,
                ..context()
            }
        )
        .is_err());
    }
    #[test]
    fn absence_is_success_but_denied_busy_and_unavailable_never_become_zero_totals() {
        let mut bytes = hex(RESULT)[16..].to_vec();
        bytes[21..].fill(0);
        let absent = decode_body(&bytes, 5).unwrap();
        assert_eq!(
            absent.totals,
            Some(Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0
            })
        );
        for status in 1..=3 {
            bytes[20] = status;
            assert!(decode_body(&bytes, 5).unwrap().totals.is_none());
            bytes[21] = 1;
            assert!(decode_body(&bytes, 5).is_err());
            bytes[21] = 0;
        }
    }
    #[test]
    fn strict_numeric_flags_lengths_context_and_no_coercion() {
        let original = hex(RESULT);
        let mut largest = original[16..].to_vec();
        largest[24..32].copy_from_slice(&i64::MAX.to_be_bytes());
        largest[32..40].fill(0);
        largest[40..48].copy_from_slice(&i64::MAX.to_be_bytes());
        assert_eq!(
            decode_body(&largest, 5).unwrap().totals.unwrap().wins,
            i64::MAX
        );
        for length in 0..original.len() {
            assert!(app::decode_response_version(&original[..length], 5).is_err());
        }
        let mut tail = original.clone();
        tail.push(0);
        assert!(app::decode_response_version(&tail, 5).is_err());
        for index in [20, 21, 22, 23] {
            let mut body = original[16..].to_vec();
            body[index] = 255;
            assert!(decode_body(&body, 5).is_err());
        }
        for index in [24, 32, 40] {
            let mut body = original[16..].to_vec();
            body[index..index + 8].copy_from_slice(&(-1i64).to_be_bytes());
            assert!(decode_body(&body, 5).is_err());
        }
        for (wins, losses, revision) in [(i64::MAX, 1, i64::MAX), (1, 2, 4)] {
            let mut body = original[16..].to_vec();
            for (offset, value) in [(24, wins), (32, losses), (40, revision)] {
                body[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
            }
            assert!(decode_body(&body, 5).is_err());
        }
        for range in [0..4, 4..12, 12..20] {
            let mut body = original[16..].to_vec();
            body[range].fill(0);
            assert!(decode_body(&body, 5).is_err());
        }
        let mut body = original[16..].to_vec();
        body[21] = 0;
        assert!(decode_body(&body, 5).is_err());
    }
    #[test]
    fn record_header_length_is_rejected_before_reading_an_oversize_body() {
        struct HeaderOnly {
            bytes: std::io::Cursor<Vec<u8>>,
        }
        impl std::io::Read for HeaderOnly {
            fn read(&mut self, target: &mut [u8]) -> std::io::Result<usize> {
                assert!(
                    self.bytes.position() < 16,
                    "decoder tried to allocate/read an invalid record body"
                );
                std::io::Read::read(&mut self.bytes, target)
            }
        }
        for length in [16u32, 63, 65, 65536, u32::MAX] {
            let mut bytes = hex(RESULT)[..16].to_vec();
            bytes[4..8].copy_from_slice(&length.to_be_bytes());
            let mut reader = HeaderOnly {
                bytes: std::io::Cursor::new(bytes),
            };
            assert!(app::read_response_version(&mut reader, 5).is_err());
        }
    }
    #[test]
    fn failed_refresh_retains_only_exact_source_object_epoch_and_recovery_replaces() {
        let good = decode_body(&hex(RESULT)[16..], 5).unwrap();
        for changed in [
            SelectionContext {
                key: app::CharacterKey {
                    source_id: 42,
                    ..context().key
                },
                ..context()
            },
            SelectionContext {
                key: app::CharacterKey {
                    object_id: 4444,
                    ..context().key
                },
                ..context()
            },
            SelectionContext {
                epoch: 1,
                ..context()
            },
        ] {
            let mut state = State::default();
            state.receive(good.clone()).unwrap();
            state
                .receive(Reply {
                    context: context(),
                    status: Status::Busy,
                    totals: None,
                })
                .unwrap();
            assert_eq!(state.totals(), good.totals);
            assert!(matches!(
                state,
                State::Ready {
                    refresh_failure: Some(Status::Busy),
                    ..
                }
            ));
            state
                .receive(Reply {
                    context: changed,
                    status: Status::Unavailable,
                    totals: None,
                })
                .unwrap();
            assert_eq!(state.context(), Some(changed));
            assert!(state.totals().is_none());
            state.receive(good.clone()).unwrap();
            assert!(matches!(
                state,
                State::Ready {
                    refresh_failure: None,
                    ..
                }
            ));
        }
    }

    #[test]
    fn same_context_totals_never_regress_or_rewrite_an_existing_revision() {
        let baseline = Reply {
            context: context(),
            status: Status::Ok,
            totals: Some(Totals {
                stored: true,
                wins: 5,
                losses: 2,
                revision: 7,
            }),
        };
        let mut state = State::default();
        state.receive(baseline.clone()).unwrap();
        // This is not permission to forget the last authoritative revision.
        state
            .receive(Reply {
                context: context(),
                status: Status::Busy,
                totals: None,
            })
            .unwrap();
        let retained = state.clone();
        for totals in [
            Totals {
                stored: true,
                wins: 4,
                losses: 3,
                revision: 7,
            },
            Totals {
                stored: true,
                wins: 6,
                losses: 1,
                revision: 7,
            },
            Totals {
                stored: true,
                wins: 4,
                losses: 2,
                revision: 6,
            },
            Totals {
                stored: true,
                wins: 4,
                losses: 9,
                revision: 13,
            },
            Totals {
                stored: true,
                wins: 9,
                losses: 1,
                revision: 10,
            },
            Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0,
            },
        ] {
            totals.validate().unwrap(); // Each is a legal standalone wire value.
            assert!(state
                .receive(Reply {
                    totals: Some(totals),
                    ..baseline.clone()
                })
                .is_err());
            assert_eq!(
                state, retained,
                "rejection must not mutate confirmed totals or last failure"
            );
        }
        state.receive(baseline.clone()).unwrap();
        assert!(matches!(
            state,
            State::Ready {
                refresh_failure: None,
                ..
            }
        ));
        let advanced = Totals {
            stored: true,
            wins: 8,
            losses: 4,
            revision: 12,
        };
        state
            .receive(Reply {
                totals: Some(advanced),
                ..baseline.clone()
            })
            .unwrap();
        assert_eq!(state.totals(), Some(advanced));

        let absent = Totals {
            stored: false,
            wins: 0,
            losses: 0,
            revision: 0,
        };
        for next_context in [
            SelectionContext {
                key: app::CharacterKey {
                    source_id: 42,
                    ..context().key
                },
                ..context()
            },
            SelectionContext {
                key: app::CharacterKey {
                    object_id: 4444,
                    ..context().key
                },
                ..context()
            },
            SelectionContext {
                epoch: 1,
                ..context()
            },
        ] {
            let mut fresh = state.clone();
            fresh
                .receive(Reply {
                    context: next_context,
                    status: Status::Ok,
                    totals: Some(absent),
                })
                .unwrap();
            assert_eq!(fresh.context(), Some(next_context));
            assert_eq!(fresh.totals(), Some(absent));
        }
    }

    #[test]
    fn stored_identity_cannot_change_at_the_same_zero_revision() {
        let mut state = State::default();
        let mut reply = Reply {
            context: context(),
            status: Status::Ok,
            totals: Some(Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0,
            }),
        };
        state.receive(reply.clone()).unwrap();
        let before = state.clone();
        reply.totals.as_mut().unwrap().stored = true;
        assert!(state.receive(reply.clone()).is_err());
        assert_eq!(state, before);
        let mut stored = State::default();
        stored.receive(reply.clone()).unwrap();
        let before = stored.clone();
        reply.totals.as_mut().unwrap().stored = false;
        assert!(stored.receive(reply).is_err());
        assert_eq!(stored, before);
    }
}
