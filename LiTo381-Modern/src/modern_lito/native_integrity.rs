//! Legacy S_EVENT/33 and C_ADDR wire contract, verified against JAR E1498871.
//!
//! Compatibility profile, not a memory measurement: the Rust executable does not
//! contain the legacy image, so it never dereferences the challenged address.
//! The only range answered is the deployed legacy anchor (0x004ED840, 5 bytes),
//! which the server source documents as the relocation-free `call $+5`
//! instruction (`E8 00 00 00 00`) of the fixed-base call/pop helper. Rust
//! computes the seeded CRC over that declared instruction encoding with the
//! deployed algorithm; the server constant is only used by tests as an oracle.
//! Any other address/length stays unsupported and is not answered. Reading
//! LiTo.bin from disk is not an alternative: the section is Themida-packed.
use super::native_opcodes::{C_ADDR, S_EVENT};

/// Deployed challenge range (`NativeClientIntegrity.RANGE_START/LENGTH`).
pub const LEGACY_ANCHOR_ADDRESS: u32 = 0x004E_D840;
/// Relocation-free `call $+5` encoding the legacy anchor helper starts with.
pub const LEGACY_ANCHOR_BYTES: [u8; 5] = [0xE8, 0, 0, 0, 0];
/// Deployed seed (`NativeClientIntegrity.CRC_SEED`).
pub const CRC_SEED: u32 = 0xD883_C01B;
/// The Rust client has no legacy protector layer; it reports 0 honestly.
pub const PROTECTED_STATE: bool = false;

/// Reflected CRC-32 (0xEDB88320) seeded like the deployed server: the seed is
/// complemented as the initial register and the result is complemented again.
pub fn seeded_crc(bytes: &[u8], seed: u32) -> u32 {
    let mut crc = !seed;
    for &byte in bytes {
        let mut value = (crc ^ u32::from(byte)) & 0xff;
        for _ in 0..8 {
            value = if value & 1 != 0 { 0xEDB8_8320 ^ (value >> 1) } else { value >> 1 };
        }
        crc = value ^ (crc >> 8);
    }
    !crc
}
use anyhow::{ensure, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub id: u32,
    pub address: u32,
    pub length: u32,
}

impl Challenge {
    pub fn parse(body: &[u8]) -> Result<Self> {
        ensure!(
            body.len() == 14 && body[..2] == [S_EVENT, 33],
            "invalid integrity challenge envelope"
        );
        let value = Self {
            id: u32::from_le_bytes(body[2..6].try_into()?),
            address: u32::from_le_bytes(body[6..10].try_into()?),
            length: u32::from_le_bytes(body[10..14].try_into()?),
        };
        ensure!(
            value.id != 0 && value.address != 0 && value.length != 0,
            "invalid integrity challenge"
        );
        // These are wire values, never pointers to dereference in this process.
        Ok(value)
    }

    pub fn fields(self) -> [u32; 3] {
        [self.id, self.address, self.length]
    }

    /// Compatibility-profile reply for the deployed anchor range only. No
    /// memory access or allocation follows the server-supplied values.
    pub fn compatibility_proof(self) -> Option<[u8; 13]> {
        (self.address == LEGACY_ANCHOR_ADDRESS && self.length as usize == LEGACY_ANCHOR_BYTES.len())
            .then(|| address_proof(self.id, seeded_crc(&LEGACY_ANCHOR_BYTES, CRC_SEED), PROTECTED_STATE))
            .and_then(Result::ok)
    }
}

/// Client-side state only: `ReplyQueued` means C_ADDR entered the owning
/// connection's send queue. The deployed server sends no acceptance packet, so
/// server-side verification is never claimed here. Reset with the world.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Status {
    #[default]
    NotRequested,
    /// Supported anchor challenge parsed; reply not yet queued.
    ReplyPending,
    ReplyQueued,
    /// Address/length outside the compatibility profile; never answered.
    UnsupportedRange,
}

/// Serialize a C_ADDR body. Callers must bind it to the active connection,
/// world generation and challenge before sending.
pub fn address_proof(challenge_id: u32, checksum: u32, protected_state: bool) -> Result<[u8; 13]> {
    ensure!(challenge_id != 0, "invalid native C_ADDR proof");
    let mut body = [0; 13];
    body[0] = C_ADDR;
    body[1..5].copy_from_slice(&challenge_id.to_le_bytes());
    body[5..9].copy_from_slice(&checksum.to_le_bytes());
    body[9..13].copy_from_slice(&u32::from(protected_state).to_le_bytes());
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::native_world::World;

    const CHALLENGE: [u8; 14] = [
        0x7b, 33, 0x98, 0xba, 0xdc, 0xfe, 0x40, 0xd8, 0x4e, 0, 5, 0, 0, 0,
    ];

    fn active_world() -> World {
        let mut world = World::default();
        assert!(world.accept(&[0x28, 5, 0, 0]).unwrap());
        assert!(world.accept(&[0xd2]).unwrap());
        world
    }

    #[test]
    fn deployed_challenge_fields_are_unsigned_little_endian() {
        assert_eq!(
            Challenge::parse(&CHALLENGE).unwrap().fields(),
            [0xfedcba98, 5167168, 5]
        );
    }

    #[test]
    fn challenge_rejects_every_truncation_tail_opcode_subtype_and_zero_field() {
        for n in 0..CHALLENGE.len() {
            assert!(Challenge::parse(&CHALLENGE[..n]).is_err(), "prefix {n}");
        }
        let mut tail = CHALLENGE.to_vec();
        tail.push(0);
        assert!(Challenge::parse(&tail).is_err());
        for offset in [0, 1] {
            let mut body = CHALLENGE;
            body[offset] ^= 1;
            assert!(Challenge::parse(&body).is_err());
        }
        for start in [2, 6, 10] {
            let mut body = CHALLENGE;
            body[start..start + 4].fill(0);
            assert!(Challenge::parse(&body).is_err());
        }
    }

    #[test]
    fn c_addr_encodes_both_protection_states_and_checksum_boundaries() {
        // Arbitrary codec fixture (non-matching checksum).
        assert_eq!(
            address_proof(0xfedcba98, 0x01234567, true).unwrap(),
            [0x2a, 0x98, 0xba, 0xdc, 0xfe, 0x67, 0x45, 0x23, 1, 1, 0, 0, 0]
        );
        for checksum in [0, u32::MAX] {
            let body = address_proof(u32::MAX, checksum, false).unwrap();
            assert_eq!(&body[1..5], &[255; 4]);
            assert_eq!(&body[5..9], &checksum.to_le_bytes());
            assert_eq!(&body[9..], &[0; 4]);
        }
        assert!(address_proof(0, 0, false).is_err());
    }

    #[test]
    fn seeded_crc_matches_deployed_server_oracle() {
        // Oracles from JAR E1498871 NativeClientIntegrity and its server test.
        assert_eq!(seeded_crc(&LEGACY_ANCHOR_BYTES, CRC_SEED), 0x1569_D958);
        // Standard CRC-32 check value: seed 0 is the plain reflected CRC.
        assert_eq!(seeded_crc(b"123456789", 0), 0xCBF4_3926);
        assert_ne!(seeded_crc(&[0xE8, 0, 0, 0, 1], CRC_SEED), 0x1569_D958);
    }

    #[test]
    fn compatibility_proof_answers_only_the_deployed_anchor_range() {
        let challenge = Challenge::parse(&CHALLENGE).unwrap();
        assert_eq!(
            challenge.compatibility_proof().unwrap(),
            [0x2a, 0x98, 0xba, 0xdc, 0xfe, 0x58, 0xd9, 0x69, 0x15, 0, 0, 0, 0]
        );
        for (address, length) in [(LEGACY_ANCHOR_ADDRESS, 4), (LEGACY_ANCHOR_ADDRESS, 6),
            (LEGACY_ANCHOR_ADDRESS + 1, 5), (u32::MAX, u32::MAX)] {
            assert_eq!(Challenge { id: 1, address, length }.compatibility_proof(), None);
        }
    }

    #[test]
    fn world_marks_supported_challenge_pending_until_queued() {
        let mut world = active_world();
        assert_eq!(world.integrity_status, Status::NotRequested);
        assert_eq!(world.integrity_reply(), None);
        assert!(world.accept(&CHALLENGE).unwrap());
        assert_eq!(world.integrity_challenge, Some([0xfedcba98, 5167168, 5]));
        assert_eq!(world.integrity_status, Status::ReplyPending);
        assert_eq!(world.integrity_reply(), Challenge::parse(&CHALLENGE).unwrap().compatibility_proof());
        world.integrity_reply_queued();
        assert_eq!(world.integrity_status, Status::ReplyQueued);
        assert_eq!(world.integrity_reply(), None);
        let json = serde_json::to_value(&world).unwrap();
        assert_eq!(json["integrity_status"], "ReplyQueued");
        assert!(!world.ready());
    }

    #[test]
    fn malformed_challenge_does_not_replace_prior_receipt() {
        let mut world = active_world();
        assert!(world.accept(&CHALLENGE).unwrap());
        let before = serde_json::to_value(&world).unwrap();
        for n in 2..CHALLENGE.len() {
            assert!(world.accept(&CHALLENGE[..n]).is_err());
            assert_eq!(serde_json::to_value(&world).unwrap(), before);
        }
        for start in [2, 6, 10] {
            let mut body = CHALLENGE;
            body[start..start + 4].fill(0);
            assert!(world.accept(&body).is_err());
            assert_eq!(serde_json::to_value(&world).unwrap(), before);
        }
        let mut tail = CHALLENGE.to_vec();
        tail.push(0);
        assert!(world.accept(&tail).is_err());
        assert_eq!(serde_json::to_value(&world).unwrap(), before);
    }

    #[test]
    fn duplicate_is_answered_once_and_unknown_range_is_never_answered() {
        let mut world = active_world();
        assert!(world.accept(&CHALLENGE).unwrap());
        world.integrity_reply_queued();
        assert!(world.accept(&CHALLENGE).unwrap());
        assert_eq!(world.integrity_status, Status::ReplyQueued);
        assert_eq!(world.integrity_reply(), None);
        let mut fresh = CHALLENGE;
        fresh[2] ^= 1;
        assert!(world.accept(&fresh).unwrap());
        assert_eq!(world.integrity_status, Status::ReplyPending);
        assert_eq!(world.integrity_reply().unwrap()[1..5], fresh[2..6]);
        let unknown = [0x7b, 33, 1, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255];
        assert!(world.accept(&unknown).unwrap());
        assert_eq!(world.integrity_status, Status::UnsupportedRange);
        assert_eq!(world.integrity_challenge, Some(Challenge::parse(&unknown).unwrap().fields()));
        assert_eq!(world.integrity_reply(), None);
        world.integrity_reply_queued();
        assert_eq!(world.integrity_status, Status::UnsupportedRange);
    }

    #[test]
    fn world_replacement_and_retirement_clear_integrity_diagnosis() {
        let mut world = active_world();
        assert!(world.accept(&CHALLENGE).unwrap());
        assert_eq!(world.integrity_status, Status::ReplyPending);
        world.accept(&[0x28, 5, 0, 0]).unwrap();
        assert_eq!(world.integrity_challenge, None);
        assert_eq!(world.integrity_status, Status::NotRequested);
        assert_eq!(world.integrity_reply(), None);
        assert!(world.accept(&[0xd2]).unwrap());
        assert!(world.accept(&CHALLENGE).unwrap());
        world.integrity_reply_queued();
        assert_eq!(world.integrity_status, Status::ReplyQueued);
        world.clear_preserving_generation();
        assert_eq!(world.integrity_challenge, None);
        assert_eq!(world.integrity_status, Status::NotRequested);
    }
}
