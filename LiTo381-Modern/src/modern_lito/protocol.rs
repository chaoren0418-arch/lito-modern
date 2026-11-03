//! Modern LiTo's independent, bounded handshake. This is not native `C_VERSION`,
//! does not use the native XOR cipher, and never fabricates a native integrity CRC.
//!
//! Wire integers are big endian. The 16-byte header is `LTM1`, u32 total frame
//! length, nonzero u32 request ID, u16 kind and zero u16 reserved. HELLO (kind 1,
//! 32 bytes) carries u16 min/max version, u32 supported/required capabilities,
//! u16 numeric schema and zero u16 reserved. ACK (kind 2, 24 bytes) carries u16
//! version/schema and u32 capabilities. These bytes match ModernSessionProtocol.java.
//!
//! Only compatibility is negotiated. No credentials, authentication, gameplay,
//! socket or TLS implementation exists here. Remote passwords MUST NOT be added
//! before authenticated TLS (or equivalent) protects the connection. The caller
//! owns timeouts and must close its transport when this state machine fails.

use std::fmt;
use std::io::{self, Read, Write};
#[cfg(test)]
#[path = "session_extensions_tests.rs"]
mod extension_tests;
#[path = "session_extensions.rs"]
pub mod extensions;

pub const VERSION: u16 = 1;
/// Version 2 adds authority item-lifetime views to equipment results only.
/// Version 3 adds terminal match results; version 4 adds learned skills and
/// authoritative cast events. Version 5 adds central persisted record queries;
/// version 6 adds targeted magic and its atomic combat appendix. Existing v1–5
/// payloads retain their versioned shape. Version 7 adds tagged participants
/// and optional GM room commands (256); the required base remains 7.
/// Version 8 adds match-local shop/items; version 9 optionally adds confirmed
/// tournament weight. Version 10 adds optional timed supplies and authority
/// effect clocks, preserving every earlier version's bytes.
pub const MAX_VERSION: u16 = 47;
/// Upper bound for versioned WORLD/COMBAT/ROOM decoding, not runtime admission.
pub(crate) const IMPLEMENTED_WORLD_VERSION: u16 = MAX_VERSION;
/// v37 additionally life-binds queued mutations; resource/LOAD proofs remain
/// separate authorities. Bare v36 and earlier operations keep their wire shape.
/// Older negotiated sessions keep their exact versioned packet layouts.
pub(crate) const RUNTIME_WORLD_VERSION: u16 = 37;
/// Modern namespace, unrelated to the native JVM-wide NumericBreakthrough modes.
/// Source-persisted values use explicit signed i64 fields, not native byte gauges.
pub const NUMERIC_SCHEMA_I64_V1: u16 = 1;
pub const CAP_EXPLICIT_I64_STATS: u32 = 1;
pub const CAP_SOURCE_SCOPED_IDENTITIES: u32 = 2;
pub const CAP_TOURNAMENT_NAVIGATION: u32 = 4;
pub const CAP_CHARACTER_DATA: u32 = 8;
pub const CAP_EQUIPMENT: u32 = 16;
pub const CAP_ROOMS: u32 = 32;
pub const CAP_WORLD: u32 = 64;
pub const CAP_COMBAT: u32 = 128;
pub const CAP_GM_BOTS: u32 = 256;
pub const CAP_SHOP: u32 = 512;
pub const CAP_WEIGHT: u32 = 1024;
pub const CAP_TIMED: u32 = 2048;
pub const CAP_ARROW: u32 = 4096;
pub const CAP_REGEN: u32 = 8192;
pub const CAP_HONOR: u32 = 16384;
pub const CAP_FIRE_STORM: u32 = 32768;
pub const CAP_MATCH_DESCRIPTOR: u32 = 65536;
pub const CAP_PET_RACE: u32 = 131072;
pub const CAP_NPC: u32 = 262144;
pub const CAP_DISINTEGRATE: u32 = 524288;
// Opt-in server/transport capability; the full game controller offers it after preflight integration.
pub const CAP_NPC_DISINTEGRATE: u32 = 1048576;
pub const CAP_NPC_FIRE_STORM: u32 = 2097152;
pub const CAP_CHAT: u32 = 4194304;
pub const CAP_DIRECT_CHAT: u32 = 8388608;
pub const CAP_FILTERED_CHAT: u32 = 16777216;
pub const CAP_MATCHMAKING: u32 = 33554432;
/// Opt-in v34: 17..64 full S32 sectors, still subject to shared byte/asset budgets.
pub const CAP_LARGE_MAPS: u32 = 67108864;
/// Opt-in canonical read channel; not permission to mutate equipment or render.
pub const CAP_BATTLE_DATA: u32 = 134217728;
/// Explicit life/target admission; never inferred from canonical read support.
pub const CAP_ACTION_LIFE: u32 = 268435456;
/// Explicit self-cast and schema3 support; not offered by the player UI yet.
pub const CAP_GREATER_HASTE: u32 = 536870912;
/// Complete targeted Haste state in schema5; not a cast-command permission.
pub const CAP_HASTE: u32 = 1073741824;
/// Explicit v46 complete speed/material DATA, not Slow cast permission.
pub const CAP_SLOW_DATA: u32 = 0x8000_0000;
pub const REQUIRED_CAPABILITIES: u32 =
    CAP_EXPLICIT_I64_STATS | CAP_SOURCE_SCOPED_IDENTITIES | CAP_TOURNAMENT_NAVIGATION;
pub const KNOWN_CAPABILITIES: u32 = REQUIRED_CAPABILITIES
    | CAP_CHARACTER_DATA
    | CAP_EQUIPMENT
    | CAP_ROOMS
    | CAP_WORLD
    | CAP_COMBAT
    | CAP_GM_BOTS
    | CAP_SHOP
    | CAP_WEIGHT
    | CAP_TIMED
    | CAP_ARROW
    | CAP_REGEN
    | CAP_HONOR
    | CAP_FIRE_STORM
    | CAP_MATCH_DESCRIPTOR
    | CAP_PET_RACE
    | CAP_NPC
    | CAP_DISINTEGRATE
    | CAP_NPC_DISINTEGRATE
    | CAP_NPC_FIRE_STORM
    | CAP_CHAT
    | CAP_DIRECT_CHAT
    | CAP_FILTERED_CHAT
    | CAP_MATCHMAKING
    | CAP_LARGE_MAPS
    | CAP_BATTLE_DATA
    | CAP_ACTION_LIFE
    | CAP_GREATER_HASTE
    | CAP_HASTE
    | CAP_SLOW_DATA;
/// Codec support is not encounter/renderer admission. Enable only after that
/// complete runtime path is qualified, not merely because v17 bytes decode.
pub const RUNTIME_CAPABILITIES: u32 = KNOWN_CAPABILITIES
    & !(CAP_NPC
        | CAP_DISINTEGRATE
        | CAP_NPC_DISINTEGRATE
        | CAP_NPC_FIRE_STORM
        | CAP_CHAT
        | CAP_DIRECT_CHAT
        | CAP_FILTERED_CHAT
        | CAP_MATCHMAKING
        | CAP_LARGE_MAPS
        | CAP_BATTLE_DATA
        | CAP_ACTION_LIFE
        | CAP_GREATER_HASTE
        | CAP_HASTE
        | CAP_SLOW_DATA);
pub const HEADER_BYTES: usize = 16;
pub const MAX_FRAME_BYTES: usize = 40;
const MAGIC: &[u8; 4] = b"LTM1";
const HELLO_KIND: u16 = 1;
const ACK_KIND: u16 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    MalformedFrame,
    UnsupportedVersion,
    UnsupportedCapability,
    UnsupportedSchema,
    UnexpectedMessage,
    RequestIdMismatch,
    Closed,
    Io,
}

#[derive(Debug)]
pub struct ProtocolError {
    pub kind: ErrorKind,
    detail: String,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for ProtocolError {}

fn error(kind: ErrorKind, detail: impl Into<String>) -> ProtocolError {
    ProtocolError {
        kind,
        detail: detail.into(),
    }
}

fn malformed(detail: &str) -> ProtocolError {
    error(ErrorKind::MalformedFrame, detail)
}

fn io_error(cause: io::Error) -> ProtocolError {
    error(
        if cause.kind() == io::ErrorKind::UnexpectedEof {
            ErrorKind::MalformedFrame
        } else {
            ErrorKind::Io
        },
        cause.to_string(),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hello {
    pub request_id: u32,
    pub min_version: u16,
    pub max_version: u16,
    pub supported_capabilities: u32,
    pub required_capabilities: u32,
    pub numeric_schema: u16,
}

impl Hello {
    pub fn current(request_id: u32) -> Self {
        Self {
            request_id,
            min_version: VERSION,
            max_version: MAX_VERSION,
            supported_capabilities: RUNTIME_CAPABILITIES,
            required_capabilities: REQUIRED_CAPABILITIES,
            numeric_schema: NUMERIC_SCHEMA_I64_V1,
        }
    }
}
#[cfg(test)]
mod speed_data_tests {
    use super::*;
    fn offer() -> Hello {
        Hello {
            request_id: 0xfedcba98,
            min_version: 1,
            max_version: 46,
            supported_capabilities: KNOWN_CAPABILITIES,
            required_capabilities: REQUIRED_CAPABILITIES,
            numeric_schema: 1,
        }
    }
    #[test]
    fn speed_high_bit_is_unsigned_explicit_and_version_scoped_in_both_directions() {
        assert_eq!(CAP_SLOW_DATA, 1 << 31);
        assert_eq!(RUNTIME_CAPABILITIES & CAP_SLOW_DATA, 0);
        assert_eq!(Hello::current(1).supported_capabilities & CAP_SLOW_DATA, 0);
        let h = offer();
        let hb = encode(Message::Hello(h)).unwrap();
        assert_eq!(decode(&hb).unwrap(), Message::Hello(h));
        assert_eq!(&hb[20..24], &[255; 4]);
        let a = Ack {
            request_id: h.request_id,
            version: 46,
            numeric_schema: 1,
            capabilities: KNOWN_CAPABILITIES,
        };
        let ab = encode(Message::Ack(a)).unwrap();
        assert_eq!(decode(&ab).unwrap(), Message::Ack(a));
        for version in 1u16..46 {
            assert!(encode(Message::Hello(Hello {
                max_version: version,
                ..h
            }))
            .is_err());
            assert!(encode(Message::Ack(Ack { version, ..a })).is_err());
            let mut bad = hb.clone();
            bad[18..20].copy_from_slice(&version.to_be_bytes());
            assert!(decode(&bad).is_err());
            let mut bad = ab.clone();
            bad[16..18].copy_from_slice(&version.to_be_bytes());
            assert!(decode(&bad).is_err());
        }
        for bit in [
            CAP_TIMED,
            CAP_BATTLE_DATA,
            CAP_ACTION_LIFE,
            CAP_MATCH_DESCRIPTOR,
        ] {
            assert!(validate_capabilities(KNOWN_CAPABILITIES & !bit, 7).is_err());
            assert!(validate_capabilities(KNOWN_CAPABILITIES, KNOWN_CAPABILITIES & !bit).is_err());
        }
    }
    #[test]
    fn speed_ack_cannot_add_an_unoffered_capability_or_revive_a_closed_session() {
        let h = Hello {
            supported_capabilities: KNOWN_CAPABILITIES & !CAP_SLOW_DATA,
            ..offer()
        };
        let mut s = ModernClientSession::new(h).unwrap();
        s.begin().unwrap();
        let a = Ack {
            request_id: h.request_id,
            version: 46,
            numeric_schema: 1,
            capabilities: KNOWN_CAPABILITIES,
        };
        assert!(s.accept_ack(&encode(Message::Ack(a)).unwrap()).is_err());
        assert_eq!(s.state(), ClientState::Closed);
        assert!(s
            .accept_ack(
                &encode(Message::Ack(Ack {
                    capabilities: h.supported_capabilities,
                    ..a
                }))
                .unwrap()
            )
            .is_err());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ack {
    pub request_id: u32,
    pub version: u16,
    pub numeric_schema: u16,
    pub capabilities: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Message {
    Hello(Hello),
    Ack(Ack),
    ExtendedHello(Hello, extensions::Offer),
    ExtendedAck(Ack, u32),
}

impl Message {
    fn request_id(self) -> u32 {
        match self {
            Self::Hello(hello) | Self::ExtendedHello(hello, _) => hello.request_id,
            Self::Ack(ack) | Self::ExtendedAck(ack, _) => ack.request_id,
        }
    }
}

fn validate_capabilities(supported: u32, required: u32) -> Result<(), ProtocolError> {
    let haste = CAP_TIMED | CAP_BATTLE_DATA | CAP_ACTION_LIFE | CAP_MATCH_DESCRIPTOR;
    if [supported, required]
        .iter()
        .any(|c| c & (CAP_GREATER_HASTE | CAP_HASTE | CAP_SLOW_DATA) != 0 && c & haste != haste)
    {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "greater haste dependencies missing",
        ));
    }
    let action = CAP_BATTLE_DATA | CAP_WORLD | CAP_MATCH_DESCRIPTOR;
    if [supported, required]
        .into_iter()
        .any(|c| c & CAP_ACTION_LIFE != 0 && c & action != action)
    {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "life-bound actions require canonical world and descriptor",
        ));
    }
    let battle = CAP_WORLD | CAP_MATCH_DESCRIPTOR;
    if [supported, required]
        .into_iter()
        .any(|c| c & CAP_BATTLE_DATA != 0 && c & battle != battle)
    {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "battle data requires world and descriptor",
        ));
    }
    let large = CAP_WORLD | CAP_MATCH_DESCRIPTOR;
    if [supported, required]
        .into_iter()
        .any(|c| c & CAP_LARGE_MAPS != 0 && c & large != large)
    {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "large maps require world and descriptor",
        ));
    }
    let matching = CAP_EXPLICIT_I64_STATS
        | CAP_SOURCE_SCOPED_IDENTITIES
        | CAP_EQUIPMENT
        | CAP_WORLD
        | CAP_MATCH_DESCRIPTOR;
    if [supported, required]
        .into_iter()
        .any(|c| c & CAP_MATCHMAKING != 0 && c & matching != matching)
    {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "matchmaking requires qualified world, equipment and record identity",
        ));
    }
    if [supported, required]
        .into_iter()
        .any(|c| c & CAP_FILTERED_CHAT != 0 && c & CAP_DIRECT_CHAT == 0)
    {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "filtered chat requires qualified direct preferences",
        ));
    }
    if [supported, required]
        .into_iter()
        .any(|c| c & CAP_DIRECT_CHAT != 0 && c & CAP_CHAT == 0)
    {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "direct chat requires chat membership",
        ));
    }
    if [supported, required].into_iter().any(|c| {
        c & CAP_CHAT != 0
            && c & (REQUIRED_CAPABILITIES | CAP_ROOMS) != (REQUIRED_CAPABILITIES | CAP_ROOMS)
    }) {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "chat requires qualified room navigation",
        ));
    }
    if supported & !KNOWN_CAPABILITIES != 0
        || required & !supported != 0
        || [supported, required].into_iter().any(|caps| {
            caps & (CAP_NPC_DISINTEGRATE | CAP_NPC_FIRE_STORM) != 0
                && caps & (CAP_NPC | CAP_REGEN | CAP_MATCH_DESCRIPTOR)
                    != (CAP_NPC | CAP_REGEN | CAP_MATCH_DESCRIPTOR)
                || caps & CAP_DISINTEGRATE != 0
                    && caps & (CAP_REGEN | CAP_MATCH_DESCRIPTOR)
                        != (CAP_REGEN | CAP_MATCH_DESCRIPTOR)
        })
        || (supported & CAP_NPC != 0
            && supported & (CAP_COMBAT | CAP_MATCH_DESCRIPTOR)
                != (CAP_COMBAT | CAP_MATCH_DESCRIPTOR))
        || (required & CAP_NPC != 0
            && required & (CAP_COMBAT | CAP_MATCH_DESCRIPTOR)
                != (CAP_COMBAT | CAP_MATCH_DESCRIPTOR))
        || (supported & CAP_PET_RACE != 0
            && supported & (CAP_WORLD | CAP_MATCH_DESCRIPTOR) != (CAP_WORLD | CAP_MATCH_DESCRIPTOR))
        || (required & CAP_PET_RACE != 0
            && required & (CAP_WORLD | CAP_MATCH_DESCRIPTOR) != (CAP_WORLD | CAP_MATCH_DESCRIPTOR))
        || (supported & CAP_MATCH_DESCRIPTOR != 0 && supported & CAP_WORLD == 0)
        || (required & CAP_MATCH_DESCRIPTOR != 0 && required & CAP_WORLD == 0)
        || (supported & CAP_FIRE_STORM != 0
            && supported & (CAP_REGEN | CAP_COMBAT | CAP_WEIGHT)
                != (CAP_REGEN | CAP_COMBAT | CAP_WEIGHT))
        || (required & CAP_FIRE_STORM != 0
            && required & (CAP_REGEN | CAP_COMBAT | CAP_WEIGHT)
                != (CAP_REGEN | CAP_COMBAT | CAP_WEIGHT))
        || (supported & CAP_HONOR != 0
            && supported & (CAP_REGEN | CAP_COMBAT | CAP_WEIGHT)
                != (CAP_REGEN | CAP_COMBAT | CAP_WEIGHT))
        || (required & CAP_HONOR != 0
            && required & (CAP_REGEN | CAP_COMBAT | CAP_WEIGHT)
                != (CAP_REGEN | CAP_COMBAT | CAP_WEIGHT))
        || (supported & CAP_REGEN != 0
            && supported & (CAP_COMBAT | CAP_WEIGHT) != (CAP_COMBAT | CAP_WEIGHT))
        || (required & CAP_REGEN != 0
            && required & (CAP_COMBAT | CAP_WEIGHT) != (CAP_COMBAT | CAP_WEIGHT))
        || (supported & CAP_ARROW != 0
            && supported & (CAP_TIMED | CAP_WEIGHT | CAP_SHOP | CAP_COMBAT)
                != (CAP_TIMED | CAP_WEIGHT | CAP_SHOP | CAP_COMBAT))
        || (required & CAP_ARROW != 0
            && required & (CAP_TIMED | CAP_WEIGHT | CAP_SHOP | CAP_COMBAT)
                != (CAP_TIMED | CAP_WEIGHT | CAP_SHOP | CAP_COMBAT))
        || (supported & CAP_TIMED != 0
            && supported & (CAP_WEIGHT | CAP_SHOP | CAP_COMBAT)
                != (CAP_WEIGHT | CAP_SHOP | CAP_COMBAT))
        || (required & CAP_TIMED != 0
            && required & (CAP_WEIGHT | CAP_SHOP | CAP_COMBAT)
                != (CAP_WEIGHT | CAP_SHOP | CAP_COMBAT))
        || (supported & CAP_WEIGHT != 0 && supported & CAP_SHOP == 0)
        || (required & CAP_WEIGHT != 0 && required & CAP_SHOP == 0)
        || (supported & CAP_SHOP != 0 && supported & 228 != 228)
        || (required & CAP_SHOP != 0 && required & 228 != 228)
        || (supported & CAP_GM_BOTS != 0
            && supported & (CAP_ROOMS | CAP_TOURNAMENT_NAVIGATION)
                != (CAP_ROOMS | CAP_TOURNAMENT_NAVIGATION))
        || (required & CAP_GM_BOTS != 0
            && required & (CAP_ROOMS | CAP_TOURNAMENT_NAVIGATION)
                != (CAP_ROOMS | CAP_TOURNAMENT_NAVIGATION))
        || (supported & CAP_COMBAT != 0 && supported & 127 != 127)
        || (required & CAP_COMBAT != 0 && required & 127 != 127)
        || (supported & CAP_WORLD != 0
            && supported & (CAP_ROOMS | CAP_TOURNAMENT_NAVIGATION)
                != (CAP_ROOMS | CAP_TOURNAMENT_NAVIGATION))
        || (required & CAP_WORLD != 0
            && required & (CAP_ROOMS | CAP_TOURNAMENT_NAVIGATION)
                != (CAP_ROOMS | CAP_TOURNAMENT_NAVIGATION))
        || (supported & CAP_ROOMS != 0 && supported & CAP_TOURNAMENT_NAVIGATION == 0)
        || (required & CAP_ROOMS != 0 && required & CAP_TOURNAMENT_NAVIGATION == 0)
        || (supported & CAP_EQUIPMENT != 0
            && supported & (CAP_CHARACTER_DATA | CAP_TOURNAMENT_NAVIGATION)
                != (CAP_CHARACTER_DATA | CAP_TOURNAMENT_NAVIGATION))
        || (required & CAP_EQUIPMENT != 0
            && required & (CAP_CHARACTER_DATA | CAP_TOURNAMENT_NAVIGATION)
                != (CAP_CHARACTER_DATA | CAP_TOURNAMENT_NAVIGATION))
    {
        return Err(error(
            ErrorKind::UnsupportedCapability,
            "unknown capability or required capability not offered",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod large_map_tests {
    use super::*;
    #[test]
    fn large_map_capability_is_independent_optional_and_version_scoped() {
        let mut hello = Hello::current(1);
        assert_eq!(hello.supported_capabilities & CAP_LARGE_MAPS, 0);
        hello.supported_capabilities |= CAP_LARGE_MAPS;
        assert!(encode(Message::Hello(hello)).is_ok());
        hello.max_version = 33;
        assert!(encode(Message::Hello(hello)).is_err());
        assert!(validate_capabilities(
            REQUIRED_CAPABILITIES | CAP_LARGE_MAPS,
            REQUIRED_CAPABILITIES
        )
        .is_err());
        assert!(encode(Message::Ack(Ack {
            request_id: 1,
            version: 33,
            numeric_schema: 1,
            capabilities: RUNTIME_CAPABILITIES | CAP_LARGE_MAPS
        }))
        .is_err());
    }
}

fn validate(message: Message) -> Result<(), ProtocolError> {
    if message.request_id() == 0 {
        return Err(malformed("zero request id"));
    }
    match message {
        Message::ExtendedHello(h, e) => {
            validate(Message::Hello(h))?;
            extensions::validate_offer(h, e)
        }
        Message::ExtendedAck(a, e) => {
            validate(Message::Ack(a))?;
            extensions::validate_selected(a, e)
        }
        Message::Hello(hello) => {
            if hello.max_version < 46 && hello.supported_capabilities & CAP_SLOW_DATA != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "speed/material data requires v46",
                ));
            }
            if hello.max_version < 42 && hello.supported_capabilities & CAP_GREATER_HASTE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "greater haste requires v42",
                ));
            }
            if hello.max_version < 44 && hello.supported_capabilities & CAP_HASTE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "haste state requires version forty-four",
                ));
            }
            if hello.max_version < 37 && hello.supported_capabilities & CAP_ACTION_LIFE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "life-bound actions require v37",
                ));
            }
            if hello.max_version < 35 && hello.supported_capabilities & CAP_BATTLE_DATA != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "battle data requires version thirty-five",
                ));
            }
            if hello.max_version < 34 && hello.supported_capabilities & CAP_LARGE_MAPS != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "large maps require version thirty-four",
                ));
            }
            if hello.max_version < 34 && hello.supported_capabilities & CAP_MATCHMAKING != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "matchmaking requires version thirty-four",
                ));
            }
            if hello.max_version < 33 && hello.supported_capabilities & CAP_FILTERED_CHAT != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "filtered chat requires version thirty-three",
                ));
            }
            if hello.max_version < 32 && hello.supported_capabilities & CAP_CHAT != 0 {
                return Err(error(ErrorKind::UnsupportedCapability, "chat requires v32"));
            }
            if hello.max_version < 17 && hello.supported_capabilities & CAP_NPC != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "NPC capability requires v17",
                ));
            }
            if hello.max_version < 31 && hello.supported_capabilities & CAP_NPC_FIRE_STORM != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "NPC70 requires v31",
                ));
            }
            if hello.max_version < 29 && hello.supported_capabilities & CAP_NPC_DISINTEGRATE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "NPC77 requires v29",
                ));
            }
            if hello.max_version < 27 && hello.supported_capabilities & CAP_DISINTEGRATE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "Disintegrate requires version twenty-seven",
                ));
            }
            if hello.max_version < 16 && hello.supported_capabilities & CAP_PET_RACE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "pet race capability requires v16",
                ));
            }
            if hello.max_version < 15 && hello.supported_capabilities & CAP_MATCH_DESCRIPTOR != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "match descriptor capability requires v15",
                ));
            }
            if hello.max_version < 14 && hello.supported_capabilities & CAP_FIRE_STORM != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "Fire Storm capability requires v14",
                ));
            }
            if hello.max_version < 13 && hello.supported_capabilities & CAP_HONOR != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "Honor capability requires v13",
                ));
            }
            if hello.max_version < 12 && hello.supported_capabilities & CAP_REGEN != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "natural recovery capability requires v12",
                ));
            }
            if hello.max_version < 11 && hello.supported_capabilities & CAP_ARROW != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "arrow capability requires v11",
                ));
            }
            if hello.max_version < 10 && hello.supported_capabilities & CAP_TIMED != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "timed capability requires v10",
                ));
            }
            if hello.min_version == 0
                || hello.max_version == 0
                || hello.min_version > hello.max_version
            {
                return Err(malformed("invalid version interval"));
            }
            if hello.numeric_schema == 0 {
                return Err(malformed("invalid numeric schema"));
            }
            if hello.max_version < 9 && hello.supported_capabilities & CAP_WEIGHT != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "weight capability requires v9",
                ));
            }
            if hello.max_version < 8 && hello.supported_capabilities & CAP_SHOP != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "shop capability requires v8",
                ));
            }
            if hello.max_version < 7 && hello.supported_capabilities & CAP_GM_BOTS != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "GM capability requires v7",
                ));
            }
            validate_capabilities(hello.supported_capabilities, hello.required_capabilities)
        }
        Message::Ack(ack) => {
            if ack.version < 46 && ack.capabilities & CAP_SLOW_DATA != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "speed/material data requires v46",
                ));
            }
            if ack.version < 42 && ack.capabilities & CAP_GREATER_HASTE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "greater haste requires version forty-two",
                ));
            }
            if ack.version < 44 && ack.capabilities & CAP_HASTE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "haste state requires version forty-four",
                ));
            }
            if ack.version < 37 && ack.capabilities & CAP_ACTION_LIFE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "life-bound actions require v37",
                ));
            }
            if ack.version < 35 && ack.capabilities & CAP_BATTLE_DATA != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "battle data requires version thirty-five",
                ));
            }
            if ack.version < 34 && ack.capabilities & CAP_LARGE_MAPS != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "large maps require version thirty-four",
                ));
            }
            if ack.version < 34 && ack.capabilities & CAP_MATCHMAKING != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "matchmaking requires version thirty-four",
                ));
            }
            if ack.version < 33 && ack.capabilities & CAP_FILTERED_CHAT != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "filtered chat requires version thirty-three",
                ));
            }
            if ack.version < 32 && ack.capabilities & CAP_CHAT != 0 {
                return Err(error(ErrorKind::UnsupportedCapability, "chat requires v32"));
            }
            if ack.version < 17 && ack.capabilities & CAP_NPC != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "NPC capability requires v17",
                ));
            }
            if ack.version < 31 && ack.capabilities & CAP_NPC_FIRE_STORM != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "NPC70 requires v31",
                ));
            }
            if ack.version < 29 && ack.capabilities & CAP_NPC_DISINTEGRATE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "NPC77 requires v29",
                ));
            }
            if ack.version < 27 && ack.capabilities & CAP_DISINTEGRATE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "Disintegrate requires version twenty-seven",
                ));
            }
            if ack.version < 16 && ack.capabilities & CAP_PET_RACE != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "pet race capability requires v16",
                ));
            }
            if ack.version < 15 && ack.capabilities & CAP_MATCH_DESCRIPTOR != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "match descriptor capability requires v15",
                ));
            }
            if ack.version < 14 && ack.capabilities & CAP_FIRE_STORM != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "Fire Storm capability requires v14",
                ));
            }
            if ack.version < 13 && ack.capabilities & CAP_HONOR != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "Honor capability requires v13",
                ));
            }
            if ack.version < 12 && ack.capabilities & CAP_REGEN != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "natural recovery capability requires v12",
                ));
            }
            if ack.version < 11 && ack.capabilities & CAP_ARROW != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "arrow capability requires v11",
                ));
            }
            if ack.version < 10 && ack.capabilities & CAP_TIMED != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "timed capability requires v10",
                ));
            }
            if ack.version == 0 || ack.numeric_schema == 0 {
                return Err(malformed("invalid selected version/schema"));
            }
            if ack.version < 9 && ack.capabilities & CAP_WEIGHT != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "weight capability requires v9",
                ));
            }
            if ack.version < 8 && ack.capabilities & CAP_SHOP != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "shop capability requires v8",
                ));
            }
            if ack.version < 7 && ack.capabilities & CAP_GM_BOTS != 0 {
                return Err(error(
                    ErrorKind::UnsupportedCapability,
                    "GM capability requires v7",
                ));
            }
            validate_capabilities(ack.capabilities, 0)
        }
    }
}

/// Encodes one whole frame; rejects invalid caller-built messages as well as peer input.
pub fn encode(message: Message) -> Result<Vec<u8>, ProtocolError> {
    validate(message)?;
    if let Message::ExtendedHello(h, e) = message {
        let mut b = encode(Message::Hello(h))?;
        b[4..8].copy_from_slice(&40u32.to_be_bytes());
        b[12..14].copy_from_slice(&extensions::HELLO_KIND.to_be_bytes());
        b.extend(e.supported.to_be_bytes());
        b.extend(e.required.to_be_bytes());
        return Ok(b);
    }
    if let Message::ExtendedAck(a, e) = message {
        let mut b = encode(Message::Ack(a))?;
        b[4..8].copy_from_slice(&28u32.to_be_bytes());
        b[12..14].copy_from_slice(&extensions::ACK_KIND.to_be_bytes());
        b.extend(e.to_be_bytes());
        return Ok(b);
    }
    let (size, kind) = match message {
        Message::Hello(_) => (32_u32, HELLO_KIND),
        Message::Ack(_) => (24_u32, ACK_KIND),
        _ => unreachable!("extension encoding returned above"),
    };
    let mut frame = Vec::with_capacity(size as usize);
    frame.extend_from_slice(MAGIC);
    frame.extend_from_slice(&size.to_be_bytes());
    frame.extend_from_slice(&message.request_id().to_be_bytes());
    frame.extend_from_slice(&kind.to_be_bytes());
    frame.extend_from_slice(&0_u16.to_be_bytes());
    match message {
        Message::Hello(hello) => {
            frame.extend_from_slice(&hello.min_version.to_be_bytes());
            frame.extend_from_slice(&hello.max_version.to_be_bytes());
            frame.extend_from_slice(&hello.supported_capabilities.to_be_bytes());
            frame.extend_from_slice(&hello.required_capabilities.to_be_bytes());
            frame.extend_from_slice(&hello.numeric_schema.to_be_bytes());
            frame.extend_from_slice(&0_u16.to_be_bytes());
        }
        Message::Ack(ack) => {
            frame.extend_from_slice(&ack.version.to_be_bytes());
            frame.extend_from_slice(&ack.numeric_schema.to_be_bytes());
            frame.extend_from_slice(&ack.capabilities.to_be_bytes());
        }
        _ => unreachable!("extension encoding returned above"),
    }
    Ok(frame)
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

// Requires exactly the fixed header or a larger validated slice, never peer length indexing.
fn validate_header(header: &[u8]) -> Result<usize, ProtocolError> {
    if header.len() < HEADER_BYTES || &header[..4] != MAGIC {
        return Err(malformed("not a modern LTM1 frame"));
    }
    let size = u32_at(header, 4);
    if size < HEADER_BYTES as u32 || size > MAX_FRAME_BYTES as u32 {
        return Err(malformed("oversize/undersize frame"));
    }
    if u32_at(header, 8) == 0 {
        return Err(malformed("zero request id"));
    }
    if u16_at(header, 14) != 0 {
        return Err(malformed("nonzero header reserved field"));
    }
    let expected = match u16_at(header, 12) {
        extensions::HELLO_KIND => 40,
        extensions::ACK_KIND => 28,
        HELLO_KIND => 32,
        ACK_KIND => 24,
        _ => {
            return Err(error(
                ErrorKind::UnexpectedMessage,
                "unknown handshake message kind",
            ));
        }
    };
    if size != expected {
        return Err(malformed("wrong message length"));
    }
    Ok(expected as usize)
}

pub fn decode(frame: &[u8]) -> Result<Message, ProtocolError> {
    if frame.len() < HEADER_BYTES || frame.len() > MAX_FRAME_BYTES {
        return Err(malformed("frame outside handshake bounds"));
    }
    let expected = validate_header(frame)?;
    if frame.len() != expected {
        return Err(malformed("truncated frame or trailing bytes"));
    }
    let kind = u16_at(frame, 12);
    if kind == extensions::HELLO_KIND || kind == extensions::ACK_KIND {
        let hello = kind == extensions::HELLO_KIND;
        let size: usize = if hello { 32 } else { 24 };
        let mut base = frame[..size].to_vec();
        base[4..8].copy_from_slice(&(size as u32).to_be_bytes());
        base[12..14].copy_from_slice(&(if hello { HELLO_KIND } else { ACK_KIND }).to_be_bytes());
        let extended = match decode(&base)? {
            Message::Hello(h) => Message::ExtendedHello(
                h,
                extensions::Offer {
                    supported: u32_at(frame, 32),
                    required: u32_at(frame, 36),
                },
            ),
            Message::Ack(a) => Message::ExtendedAck(a, u32_at(frame, 24)),
            _ => unreachable!("normalized legacy handshake kind"),
        };
        validate(extended)?;
        return Ok(extended);
    }
    let request_id = u32_at(frame, 8);
    let message = if u16_at(frame, 12) == HELLO_KIND {
        if u16_at(frame, 30) != 0 {
            return Err(malformed("nonzero hello reserved field"));
        }
        Message::Hello(Hello {
            request_id,
            min_version: u16_at(frame, 16),
            max_version: u16_at(frame, 18),
            supported_capabilities: u32_at(frame, 20),
            required_capabilities: u32_at(frame, 24),
            numeric_schema: u16_at(frame, 28),
        })
    } else {
        Message::Ack(Ack {
            request_id,
            version: u16_at(frame, 16),
            numeric_schema: u16_at(frame, 18),
            capabilities: u32_at(frame, 20),
        })
    };
    validate(message)?;
    Ok(message)
}

/// Reads exactly one frame, retaining subsequent frames in a coalesced stream.
/// Length and kind are checked before any peer-sized allocation or payload read.
/// The caller supplies transport timeouts and handles failure by closing transport.
pub fn read_frame(reader: &mut impl Read) -> Result<Vec<u8>, ProtocolError> {
    let mut header = [0_u8; HEADER_BYTES];
    reader.read_exact(&mut header[..4]).map_err(io_error)?;
    if &header[..4] != MAGIC {
        return Err(malformed("not a modern LTM1 frame"));
    }
    reader.read_exact(&mut header[4..]).map_err(io_error)?;
    let size = validate_header(&header)?;
    let mut frame = vec![0_u8; size];
    frame[..HEADER_BYTES].copy_from_slice(&header);
    reader
        .read_exact(&mut frame[HEADER_BYTES..])
        .map_err(io_error)?;
    decode(&frame)?;
    Ok(frame)
}

pub fn write_frame(writer: &mut impl Write, message: Message) -> Result<(), ProtocolError> {
    writer.write_all(&encode(message)?).map_err(io_error)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientState {
    Fresh,
    AwaitingAck,
    /// Compatibility only. Not authentication, asset validation or gameplay readiness.
    Negotiated(Ack),
    Closed,
}

/// An owned state machine per connection; no process-wide native numeric mode.
pub struct ModernClientSession {
    offered: Hello,
    offered_extensions: Option<extensions::Offer>,
    selected_extensions: u32,
    state: ClientState,
}

impl ModernClientSession {
    pub fn new(hello: Hello) -> Result<Self, ProtocolError> {
        validate(Message::Hello(hello))?;
        if hello.min_version < VERSION || hello.max_version > MAX_VERSION {
            return Err(error(
                ErrorKind::UnsupportedVersion,
                "requested protocol version is outside this client's implementation",
            ));
        }
        if hello.numeric_schema != NUMERIC_SCHEMA_I64_V1 {
            return Err(error(
                ErrorKind::UnsupportedSchema,
                "this client implements modern I64 schema 1 only",
            ));
        }
        if hello.required_capabilities & CAP_EXPLICIT_I64_STATS == 0 {
            return Err(error(
                ErrorKind::UnsupportedCapability,
                "modern schema requires explicit stat widths",
            ));
        }
        Ok(Self {
            offered: hello,
            offered_extensions: None,
            selected_extensions: 0,
            state: ClientState::Fresh,
        })
    }

    pub fn new_extended(hello: Hello, offer: extensions::Offer) -> Result<Self, ProtocolError> {
        let mut session = Self::new(hello)?;
        validate(Message::ExtendedHello(hello, offer))?;
        session.offered_extensions = Some(offer);
        Ok(session)
    }

    pub fn extensions(&self) -> Option<u32> {
        matches!(self.state, ClientState::Negotiated(_)).then_some(self.selected_extensions)
    }

    pub fn state(&self) -> ClientState {
        self.state
    }

    pub fn close(&mut self) {
        self.selected_extensions = 0;
        self.state = ClientState::Closed;
    }

    pub fn begin(&mut self) -> Result<Vec<u8>, ProtocolError> {
        if self.state != ClientState::Fresh {
            let kind = if self.state == ClientState::Closed {
                ErrorKind::Closed
            } else {
                ErrorKind::UnexpectedMessage
            };
            self.close();
            return Err(error(kind, "hello can be sent exactly once per connection"));
        }
        let bytes = encode(
            self.offered_extensions
                .map_or(Message::Hello(self.offered), |e| {
                    Message::ExtendedHello(self.offered, e)
                }),
        )?;
        self.state = ClientState::AwaitingAck;
        Ok(bytes)
    }

    pub fn accept_ack(&mut self, bytes: &[u8]) -> Result<Ack, ProtocolError> {
        let result = self.validate_ack(bytes);
        match result {
            Ok((ack, extensions)) => {
                self.selected_extensions = extensions;
                self.state = ClientState::Negotiated(ack);
                Ok(ack)
            }
            Err(failure) => {
                self.close();
                Err(failure)
            }
        }
    }

    /// Transport helper: a partial/failed HELLO write invalidates this connection.
    pub fn begin_into(&mut self, writer: &mut impl Write) -> Result<(), ProtocolError> {
        let bytes = self.begin()?;
        if let Err(failure) = writer.write_all(&bytes) {
            self.close();
            return Err(io_error(failure));
        }
        Ok(())
    }

    /// Transport helper: malformed/truncated input closes the state as well as
    /// semantic ACK errors. Actual socket closure and deadlines remain caller-owned.
    pub fn accept_ack_from(&mut self, reader: &mut impl Read) -> Result<Ack, ProtocolError> {
        if self.state != ClientState::AwaitingAck {
            // Fail before performing any potentially blocking I/O.
            return self.accept_ack(&[]);
        }
        match read_frame(reader) {
            Ok(bytes) => self.accept_ack(&bytes),
            Err(failure) => {
                self.close();
                Err(failure)
            }
        }
    }

    fn validate_ack(&self, bytes: &[u8]) -> Result<(Ack, u32), ProtocolError> {
        if self.state != ClientState::AwaitingAck {
            return Err(error(
                if self.state == ClientState::Closed {
                    ErrorKind::Closed
                } else {
                    ErrorKind::UnexpectedMessage
                },
                "ack requires a pending hello",
            ));
        }
        let (ack, selected) = match (self.offered_extensions, decode(bytes)?) {
            (None, Message::Ack(a)) => (a, 0),
            (Some(e), Message::ExtendedAck(a, x))
                if x & !e.supported == 0 && x & e.required == e.required =>
            {
                (a, x)
            }
            _ => {
                return Err(error(
                    ErrorKind::UnexpectedMessage,
                    "wrong handshake kind or unoffered extensions",
                ))
            }
        };
        if ack.request_id != self.offered.request_id {
            return Err(error(
                ErrorKind::RequestIdMismatch,
                "ack request id does not match pending hello",
            ));
        }
        if !(self.offered.min_version..=self.offered.max_version).contains(&ack.version) {
            return Err(error(
                ErrorKind::UnsupportedVersion,
                "server selected an unoffered version",
            ));
        }
        if ack.numeric_schema != self.offered.numeric_schema {
            return Err(error(
                ErrorKind::UnsupportedSchema,
                "server selected an unoffered schema",
            ));
        }
        if ack.capabilities & !self.offered.supported_capabilities != 0
            || ack.capabilities & self.offered.required_capabilities
                != self.offered.required_capabilities
        {
            return Err(error(
                ErrorKind::UnsupportedCapability,
                "server omitted required or added unoffered capabilities",
            ));
        }
        Ok((ack, selected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Identical to ModernSessionBoundaryTest.java: actual encoded/decoded cross-language vectors.
    const GOLDEN_HELLO: &str = "4c544d3100000020fedcba9800010000000100010000000f0000000700010000";
    const GOLDEN_ACK: &str = "4c544d3100000018fedcba9800020000000100010000000f";

    fn bytes(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }

    fn hello() -> Hello {
        // Preserve the original cap15 wire golden; current cap31 is covered separately.
        Hello {
            supported_capabilities: REQUIRED_CAPABILITIES | CAP_CHARACTER_DATA,
            max_version: VERSION,
            ..Hello::current(0xfedc_ba98)
        }
    }
    fn ack() -> Ack {
        Ack {
            request_id: 0xfedc_ba98,
            version: VERSION,
            numeric_schema: NUMERIC_SCHEMA_I64_V1,
            capabilities: REQUIRED_CAPABILITIES | CAP_CHARACTER_DATA,
        }
    }

    #[test]
    fn java_golden_vectors_match_both_directions() {
        assert_eq!(
            encode(Message::Hello(hello())).unwrap(),
            bytes(GOLDEN_HELLO)
        );
        assert_eq!(encode(Message::Ack(ack())).unwrap(), bytes(GOLDEN_ACK));
        assert_eq!(
            decode(&bytes(GOLDEN_HELLO)).unwrap(),
            Message::Hello(hello())
        );
        assert_eq!(decode(&bytes(GOLDEN_ACK)).unwrap(), Message::Ack(ack()));
        let mut written = Vec::new();
        write_frame(&mut written, Message::Hello(hello())).unwrap();
        assert_eq!(written, bytes(GOLDEN_HELLO));
    }

    #[test]
    fn negotiated_is_terminal_compatibility_only_and_duplicate_closes() {
        let mut session = ModernClientSession::new(hello()).unwrap();
        assert_eq!(session.begin().unwrap(), bytes(GOLDEN_HELLO));
        assert_eq!(session.state(), ClientState::AwaitingAck);
        assert_eq!(session.accept_ack(&bytes(GOLDEN_ACK)).unwrap(), ack());
        assert_eq!(session.state(), ClientState::Negotiated(ack()));
        assert_eq!(
            session.accept_ack(&bytes(GOLDEN_ACK)).unwrap_err().kind,
            ErrorKind::UnexpectedMessage
        );
        assert_eq!(session.state(), ClientState::Closed);
        assert_eq!(session.begin().unwrap_err().kind, ErrorKind::Closed);
    }

    #[test]
    fn zero_request_unknown_kind_bits_reserved_and_wrong_lengths_fail() {
        for (offset, value) in [(0, 0), (7, 33), (12, 1), (15, 1), (20, 0x80), (31, 1)] {
            let mut frame = bytes(GOLDEN_HELLO);
            frame[offset] = value;
            assert!(decode(&frame).is_err(), "offset={offset}");
        }
        let mut frame = bytes(GOLDEN_HELLO);
        frame[8..12].fill(0);
        assert_eq!(decode(&frame).unwrap_err().kind, ErrorKind::MalformedFrame);
        assert!(encode(Message::Hello(Hello::current(0))).is_err());
        for length in 0..32 {
            assert!(decode(&bytes(GOLDEN_HELLO)[..length]).is_err());
        }
        let mut extra = bytes(GOLDEN_ACK);
        extra.push(0);
        assert!(decode(&extra).is_err());
        assert!(decode(&vec![0; MAX_FRAME_BYTES + 1]).is_err());
    }

    #[test]
    fn stream_handles_fragmentation_coalescing_and_bounds_before_body_read() {
        struct OneByte(Cursor<Vec<u8>>);
        impl Read for OneByte {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                let length = output.len().min(1);
                self.0.read(&mut output[..length])
            }
        }
        let combined = [bytes(GOLDEN_HELLO), bytes(GOLDEN_ACK)].concat();
        let mut fragmented = OneByte(Cursor::new(combined));
        assert_eq!(read_frame(&mut fragmented).unwrap(), bytes(GOLDEN_HELLO));
        assert_eq!(fragmented.0.position(), 32);
        assert_eq!(read_frame(&mut fragmented).unwrap(), bytes(GOLDEN_ACK));
        let mut oversized = bytes(GOLDEN_HELLO);
        oversized[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
        let mut cursor = Cursor::new(oversized);
        assert_eq!(
            read_frame(&mut cursor).unwrap_err().kind,
            ErrorKind::MalformedFrame
        );
        assert_eq!(cursor.position(), HEADER_BYTES as u64);
        let mut native = Cursor::new(vec![0, 0, 0, 0, 0, 0]);
        assert!(read_frame(&mut native).is_err());
        assert_eq!(native.position(), 4);
        for length in 0..32 {
            assert!(read_frame(&mut Cursor::new(bytes(GOLDEN_HELLO)[..length].to_vec())).is_err());
        }
    }

    #[test]
    fn reordered_stale_mismatched_and_downgraded_acks_fail_closed() {
        let mut fresh = ModernClientSession::new(hello()).unwrap();
        assert!(fresh.accept_ack(&bytes(GOLDEN_ACK)).is_err());
        assert_eq!(fresh.state(), ClientState::Closed);
        for (bad, kind) in [
            (
                Ack {
                    request_id: 7,
                    ..ack()
                },
                ErrorKind::RequestIdMismatch,
            ),
            (
                Ack {
                    version: 2,
                    ..ack()
                },
                ErrorKind::UnsupportedVersion,
            ),
            (
                Ack {
                    numeric_schema: 4,
                    ..ack()
                },
                ErrorKind::UnsupportedSchema,
            ),
            (
                Ack {
                    capabilities: CAP_EXPLICIT_I64_STATS,
                    ..ack()
                },
                ErrorKind::UnsupportedCapability,
            ),
        ] {
            let mut session = ModernClientSession::new(hello()).unwrap();
            session.begin().unwrap();
            assert_eq!(
                session
                    .accept_ack(&encode(Message::Ack(bad)).unwrap())
                    .unwrap_err()
                    .kind,
                kind
            );
            assert_eq!(session.state(), ClientState::Closed);
        }
        let mut duplicate = ModernClientSession::new(hello()).unwrap();
        duplicate.begin().unwrap();
        assert!(duplicate.begin().is_err());
        assert_eq!(duplicate.state(), ClientState::Closed);
        let mut reflected = ModernClientSession::new(hello()).unwrap();
        reflected.begin().unwrap();
        assert!(reflected.accept_ack(&bytes(GOLDEN_HELLO)).is_err());
        assert_eq!(reflected.state(), ClientState::Closed);
    }

    #[test]
    fn independent_sessions_can_select_different_optional_capabilities() {
        let mut first_hello = hello();
        first_hello.required_capabilities = CAP_EXPLICIT_I64_STATS;
        first_hello.supported_capabilities = CAP_EXPLICIT_I64_STATS;
        let mut first = ModernClientSession::new(first_hello).unwrap();
        let mut second = ModernClientSession::new(Hello::current(7)).unwrap();
        first.begin().unwrap();
        second.begin().unwrap();
        let first_ack = Ack {
            capabilities: CAP_EXPLICIT_I64_STATS,
            ..ack()
        };
        first
            .accept_ack(&encode(Message::Ack(first_ack)).unwrap())
            .unwrap();
        second
            .accept_ack(
                &encode(Message::Ack(Ack {
                    request_id: 7,
                    ..ack()
                }))
                .unwrap(),
            )
            .unwrap();
        assert_eq!(first.state(), ClientState::Negotiated(first_ack));
        first.close();
        assert!(matches!(second.state(), ClientState::Negotiated(_)));
    }

    #[test]
    fn current_client_requires_navigation_before_authentication() {
        let offer = Hello::current(17);
        assert_ne!(offer.required_capabilities & CAP_TOURNAMENT_NAVIGATION, 0);
        let mut client = ModernClientSession::new(offer).unwrap();
        client.begin().unwrap();
        let auth_only = Ack {
            request_id: 17,
            version: VERSION,
            numeric_schema: NUMERIC_SCHEMA_I64_V1,
            capabilities: CAP_EXPLICIT_I64_STATS | CAP_SOURCE_SCOPED_IDENTITIES,
        };
        assert_eq!(
            client
                .accept_ack(&encode(Message::Ack(auth_only)).unwrap())
                .unwrap_err()
                .kind,
            ErrorKind::UnsupportedCapability
        );
        assert_eq!(client.state(), ClientState::Closed);
    }

    #[test]
    fn equipment_capability_requires_navigation_and_data_in_each_declared_mask() {
        for mask in [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27] {
            assert!(validate_capabilities(mask, 0).is_err());
            assert!(validate_capabilities(31, mask).is_err());
        }
        assert!(validate_capabilities(31, 7).is_ok());
        assert!(validate_capabilities(31, 28).is_ok());
        for mask in [7, 15, 31] {
            assert!(validate_capabilities(mask, 7).is_ok());
        }
    }
    #[test]
    fn room_capability_requires_navigation_but_does_not_require_equipment() {
        for supported in [32, 33, 34, 35, 40, 41, 42, 43] {
            assert!(validate_capabilities(supported, 0).is_err());
        }
        assert!(validate_capabilities(63, 32).is_err());
        for supported in [39, 47, 63] {
            assert!(validate_capabilities(supported, 7).is_ok());
        }
        assert!(validate_capabilities(63, 36).is_ok());
    }
    #[test]
    fn gm_capability_requires_v7_rooms_navigation_but_is_not_an_account_grant() {
        for mask in [256, 263, 288] {
            assert!(validate_capabilities(mask, 0).is_err());
        }
        for mask in [295, 303, 319, 367, 383, 511] {
            assert!(validate_capabilities(mask, 7).is_ok());
            let ack = Ack {
                request_id: 1,
                version: 7,
                numeric_schema: 1,
                capabilities: mask,
            };
            assert!(encode(Message::Ack(ack)).is_ok());
            for version in 1..=6 {
                assert!(encode(Message::Ack(Ack { version, ..ack })).is_err());
            }
        }
        let offer = Hello::current(1);
        assert!(encode(Message::Hello(Hello {
            max_version: 6,
            ..offer
        }))
        .is_err());
        // v7 tagged room rosters work even when GM command capability is absent.
        assert!(encode(Message::Ack(Ack {
            request_id: 1,
            version: 7,
            numeric_schema: 1,
            capabilities: 47
        }))
        .is_ok());
    }
    #[test]
    fn shop_requires_v8_and_full_world_combat_dependencies_but_remains_optional() {
        for mask in [512, 519, 548, 612, 639, 735] {
            assert!(validate_capabilities(mask, 7).is_err(), "mask {mask}");
        }
        for mask in [767u32, 1023]
            .into_iter()
            .filter(|m| m & CAP_COMBAT != 0 && m & CAP_WORLD != 0)
        {
            assert!(validate_capabilities(mask, 7).is_ok());
            let ack = Ack {
                request_id: 1,
                version: 8,
                numeric_schema: 1,
                capabilities: mask,
            };
            assert!(encode(Message::Ack(ack)).is_ok());
            for version in 1..8 {
                assert!(encode(Message::Ack(Ack { version, ..ack })).is_err());
            }
        }
        assert!(encode(Message::Hello(Hello {
            max_version: 7,
            ..Hello::current(1)
        }))
        .is_err());
        assert!(encode(Message::Hello(Hello {
            max_version: 7,
            supported_capabilities: 511,
            ..Hello::current(1)
        }))
        .is_ok());
    }
    #[test]
    fn world_capability_requires_rooms_and_navigation_and_is_optional() {
        for mask in [64, 68, 96, 128, 254, 256] {
            assert!(validate_capabilities(mask, 0).is_err());
        }
        for mask in [111, 127, 255] {
            assert!(validate_capabilities(mask, 7).is_ok());
            assert!(validate_capabilities(mask, 64).is_err());
            assert!(validate_capabilities(mask, 100).is_ok());
        }
        let hello = encode(Message::Hello(Hello::current(0xfedcba98))).unwrap();
        assert_eq!(&hello[20..28], &[0, 3, 255, 255, 0, 0, 0, 7]);
        let mut unknown = hello;
        // All 32 bits are assigned at v46; a high bit without its dependencies is still invalid.
        unknown[20..24].copy_from_slice(&CAP_SLOW_DATA.to_be_bytes());
        assert!(decode(&unknown).is_err());
    }
    #[test]
    fn character_data_is_offered_but_an_explicit_navigation_only_peer_remains_supported() {
        let offer = Hello::current(17);
        assert_eq!(offer.supported_capabilities, 262143);
        assert_eq!(offer.required_capabilities, 7);
        let mut client = ModernClientSession::new(offer).unwrap();
        client.begin().unwrap();
        let navigation_only = Ack {
            request_id: 17,
            version: VERSION,
            numeric_schema: NUMERIC_SCHEMA_I64_V1,
            capabilities: 7,
        };
        assert_eq!(
            client
                .accept_ack(&encode(Message::Ack(navigation_only)).unwrap())
                .unwrap(),
            navigation_only
        );
    }

    #[test]
    fn client_does_not_offer_unimplemented_schemas_or_versions() {
        for invalid in [
            Hello {
                numeric_schema: 4,
                ..hello()
            },
            Hello {
                max_version: MAX_VERSION + 1,
                ..hello()
            },
            Hello {
                required_capabilities: 0,
                ..hello()
            },
            Hello {
                supported_capabilities: CAP_EXPLICIT_I64_STATS,
                ..hello()
            },
        ] {
            assert!(ModernClientSession::new(invalid).is_err());
        }
        let inverted = Hello {
            min_version: 2,
            max_version: 1,
            ..hello()
        };
        assert!(encode(Message::Hello(inverted)).is_err());
    }

    #[test]
    fn current_offer_negotiates_all_supported_versions_without_unknown_bits_or_shape_guessing() {
        let offered = Hello::current(19);
        assert_eq!(
            (
                offered.min_version,
                offered.max_version,
                offered.supported_capabilities
            ),
            (1, MAX_VERSION, RUNTIME_CAPABILITIES)
        );
        for version in 1..=MAX_VERSION {
            let mut client = ModernClientSession::new(offered).unwrap();
            client.begin().unwrap();
            let ack = Ack {
                request_id: 19,
                version,
                numeric_schema: 1,
                capabilities: 31,
            };
            assert_eq!(
                client
                    .accept_ack(&encode(Message::Ack(ack)).unwrap())
                    .unwrap(),
                ack
            );
        }
        let mut legacy = ModernClientSession::new(Hello {
            max_version: 1,
            supported_capabilities: 255,
            ..offered
        })
        .unwrap();
        legacy.begin().unwrap();
        assert!(legacy
            .accept_ack(
                &encode(Message::Ack(Ack {
                    request_id: 19,
                    version: 2,
                    numeric_schema: 1,
                    capabilities: 31
                }))
                .unwrap()
            )
            .is_err());
        assert_eq!(legacy.state(), ClientState::Closed);
        assert!(ModernClientSession::new(Hello {
            supported_capabilities: KNOWN_CAPABILITIES & !CAP_TIMED,
            ..offered
        })
        .is_err());
        let mut current = ModernClientSession::new(offered).unwrap();
        current.begin().unwrap();
        assert!(current
            .accept_ack(
                &encode(Message::Ack(Ack {
                    request_id: 19,
                    version: MAX_VERSION + 1,
                    numeric_schema: 1,
                    capabilities: 31
                }))
                .unwrap()
            )
            .is_err());
    }

    #[test]
    fn weight_requires_v9_and_complete_shop_dependencies_without_changing_old_masks() {
        for version in 1..=8 {
            let old_mask = if version >= 8 {
                1023
            } else if version >= 7 {
                511
            } else {
                255
            };
            assert!(encode(Message::Ack(Ack {
                request_id: 1,
                version,
                numeric_schema: 1,
                capabilities: old_mask
            }))
            .is_ok());
            assert!(encode(Message::Ack(Ack {
                request_id: 1,
                version,
                numeric_schema: 1,
                capabilities: 2047
            }))
            .is_err());
        }
        assert!(encode(Message::Ack(Ack {
            request_id: 1,
            version: 9,
            numeric_schema: 1,
            capabilities: 2047
        }))
        .is_ok());
        for missing in [
            CAP_SHOP,
            CAP_COMBAT,
            CAP_WORLD,
            CAP_ROOMS,
            CAP_TOURNAMENT_NAVIGATION,
        ] {
            assert!(encode(Message::Ack(Ack {
                request_id: 1,
                version: 9,
                numeric_schema: 1,
                capabilities: 2047 & !missing
            }))
            .is_err());
        }
        assert!(encode(Message::Hello(Hello {
            max_version: 8,
            ..Hello::current(1)
        }))
        .is_err());
    }

    #[test]
    fn timed_requires_v10_weight_shop_combat_and_does_not_change_legacy_masks() {
        for version in 1..=9 {
            assert!(encode(Message::Ack(Ack {
                request_id: 1,
                version,
                numeric_schema: 1,
                capabilities: 4095
            }))
            .is_err());
        }
        assert!(encode(Message::Ack(Ack {
            request_id: 1,
            version: 10,
            numeric_schema: 1,
            capabilities: 4095
        }))
        .is_ok());
        for missing in [CAP_WEIGHT, CAP_SHOP, CAP_COMBAT] {
            assert!(validate_capabilities(4095 & !missing, 7).is_err());
        }
        for (version, capabilities) in [(1, 15), (3, 127), (7, 511), (8, 1023), (9, 2047)] {
            assert!(encode(Message::Ack(Ack {
                request_id: 1,
                version,
                numeric_schema: 1,
                capabilities
            }))
            .is_ok());
        }
    }

    #[test]
    fn pet_race_requires_v16_world_and_explicit_descriptor_but_not_combat() {
        let base = REQUIRED_CAPABILITIES | CAP_ROOMS | CAP_WORLD | CAP_MATCH_DESCRIPTOR;
        let capabilities = base | CAP_PET_RACE;
        for version in 1..=15 {
            assert!(encode(Message::Ack(Ack {
                request_id: 1,
                version,
                numeric_schema: 1,
                capabilities
            }))
            .is_err());
            assert!(encode(Message::Hello(Hello {
                max_version: version,
                supported_capabilities: capabilities,
                ..Hello::current(1)
            }))
            .is_err());
        }
        assert!(encode(Message::Ack(Ack {
            request_id: 1,
            version: 16,
            numeric_schema: 1,
            capabilities
        }))
        .is_ok());
        for dependency in [
            CAP_WORLD,
            CAP_MATCH_DESCRIPTOR,
            CAP_ROOMS,
            CAP_TOURNAMENT_NAVIGATION,
        ] {
            assert!(validate_capabilities(capabilities & !dependency, 0).is_err());
            assert!(validate_capabilities(capabilities, capabilities & !dependency).is_err());
        }
        assert!(validate_capabilities(capabilities, base | CAP_PET_RACE).is_ok());
        assert_eq!(capabilities & CAP_COMBAT, 0);
        assert!(encode(Message::Ack(Ack {
            request_id: 1,
            version: 15,
            numeric_schema: 1,
            capabilities: 131071
        }))
        .is_ok());
    }

    #[test]
    fn transport_helpers_fail_closed_on_partial_io() {
        struct FailedWriter;
        impl Write for FailedWriter {
            fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "test writer"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut failed_write = ModernClientSession::new(hello()).unwrap();
        assert_eq!(
            failed_write.begin_into(&mut FailedWriter).unwrap_err().kind,
            ErrorKind::Io
        );
        assert_eq!(failed_write.state(), ClientState::Closed);
        let mut truncated = ModernClientSession::new(hello()).unwrap();
        truncated.begin().unwrap();
        assert!(truncated
            .accept_ack_from(&mut Cursor::new(bytes(GOLDEN_ACK)[..20].to_vec()))
            .is_err());
        assert_eq!(truncated.state(), ClientState::Closed);
        let mut fresh = ModernClientSession::new(hello()).unwrap();
        let mut untouched = Cursor::new(bytes(GOLDEN_ACK));
        assert!(fresh.accept_ack_from(&mut untouched).is_err());
        assert_eq!(untouched.position(), 0);
    }

    /// First run `mvn -DskipTests test-compile` in LiTo381-Server, then provide
    /// LITO_MODERN_TEST_JAVA_CLASSPATH containing target/test-classes + target/classes
    /// (joined with the OS classpath separator). Optional LITO_MODERN_TEST_JAVA
    /// overrides the java executable. No network/DB/published artifacts are used.
    #[test]
    #[ignore = "requires the Java test-only stdio peer compiled in LiTo381-Server"]
    fn actual_java_server_negotiates_rust_generated_hello_over_stdio() {
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

        let classpath = std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH").expect(
            "set LITO_MODERN_TEST_JAVA_CLASSPATH to server target/test-classes and target/classes",
        );
        let executable = std::env::var_os("LITO_MODERN_TEST_JAVA").unwrap_or_else(|| "java".into());
        let seed = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32)
            ^ std::process::id();

        // Real varying request IDs, three valid intersections, and malformed peer rejection.
        for (index, stats_only, tamper) in [
            (0_u32, false, false),
            (1, true, false),
            (2, false, false),
            (3, false, true),
        ] {
            let request_id = seed.wrapping_add(index.wrapping_mul(0x9e37_79b9)) | 0x8000_0001;
            let mut offer = Hello::current(request_id);
            if stats_only {
                offer.required_capabilities = CAP_EXPLICIT_I64_STATS;
            }
            let mut client = ModernClientSession::new(offer).unwrap();
            let mut command = Command::new(&executable);
            command
                .arg("-cp")
                .arg(&classpath)
                .arg("local.lito381.server.ModernSessionStdioTestPeer");
            if stats_only {
                command.arg("--stats-only");
            }
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("start local Java test peer");
            {
                let mut input = child.stdin.take().unwrap();
                if tamper {
                    let mut frame = client.begin().unwrap();
                    frame[20] = 0x80; // Unknown capability: Java must reject, not echo ACK constants.
                    input.write_all(&frame).unwrap();
                } else {
                    client.begin_into(&mut input).unwrap();
                }
            } // EOF prevents the one-frame peer from waiting for additional input.
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("Java stdio handshake exceeded 10 seconds");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let output = child.wait_with_output().unwrap();
            if tamper {
                assert!(!output.status.success(), "Java accepted unknown capability");
                assert!(output.stdout.is_empty());
                assert!(client
                    .accept_ack_from(&mut Cursor::new(output.stdout))
                    .is_err());
                assert_eq!(client.state(), ClientState::Closed);
                eprintln!("Java rejected Rust-generated malformed hello: id={request_id:#x}");
            } else {
                assert!(
                    output.status.success(),
                    "Java peer failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(
                    output.stdout.len(),
                    24,
                    "Java must return exactly one actual ACK"
                );
                let accepted = client
                    .accept_ack_from(&mut Cursor::new(output.stdout))
                    .unwrap();
                assert_eq!(accepted.request_id, request_id);
                assert_eq!(
                    accepted.capabilities,
                    if stats_only {
                        CAP_EXPLICIT_I64_STATS
                    } else {
                        REQUIRED_CAPABILITIES | CAP_CHARACTER_DATA
                    }
                );
                assert_eq!(accepted.version, MAX_VERSION);
                assert_eq!(accepted.numeric_schema, NUMERIC_SCHEMA_I64_V1);
                assert_eq!(client.state(), ClientState::Negotiated(accepted));
                eprintln!(
                    "Rust HELLO -> Java boundary ACK -> Rust NEGOTIATED: id={request_id:#x}, capabilities={:#x}",
                    accepted.capabilities
                );
            }
        }
    }
}
