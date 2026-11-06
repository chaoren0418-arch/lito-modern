//! Legacy S_EVENT/33 and C_ADDR wire contract, verified against JAR E1498871.
//! This codec is not an integrity proof producer. The Rust executable does not
//! contain the legacy image at 0x004ED840. Never substitute the server's expected
//! CRC (or bytes copied from disk) for a measurement of the running legacy image.
use super::native_opcodes::{C_ADDR, S_EVENT};
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
}

/// Snapshot diagnosis only: receiving a challenge never implies a proof was sent
/// or accepted. Reset with the owning world/session. Unknown ranges are equally
/// unsupported; no arbitrary memory access or allocation follows server values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Status {
    #[default]
    NotRequested,
    UnsupportedLegacyImage,
}

/// Serialize an externally measured proof. This function neither measures memory
/// nor authorizes transmission. A future producer must bind the measurement to
/// the active connection, room, challenge and deadline before calling send.
/// protected_state's meaning for a Rust client is not negotiated by the legacy
/// server; choosing false does not make a fabricated checksum an honest proof.
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
        // Arbitrary codec fixture, not a proof accepted by the deployed server.
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
    fn world_reports_unsupported_without_claiming_integrity_success() {
        let mut world = active_world();
        assert_eq!(world.integrity_status, Status::NotRequested);
        assert!(world.accept(&CHALLENGE).unwrap());
        assert_eq!(world.integrity_challenge, Some([0xfedcba98, 5167168, 5]));
        assert_eq!(world.integrity_status, Status::UnsupportedLegacyImage);
        let json = serde_json::to_value(&world).unwrap();
        assert_eq!(json["integrity_status"], "UnsupportedLegacyImage");
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
    fn duplicate_and_unknown_range_remain_unsupported_without_memory_access() {
        let mut world = active_world();
        for body in [
            CHALLENGE,
            CHALLENGE,
            [0x7b, 33, 1, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255],
        ] {
            assert!(world.accept(&body).unwrap());
            assert_eq!(world.integrity_status, Status::UnsupportedLegacyImage);
            assert_eq!(
                world.integrity_challenge,
                Some(Challenge::parse(&body).unwrap().fields())
            );
        }
    }

    #[test]
    fn world_replacement_and_retirement_clear_integrity_diagnosis() {
        let mut world = active_world();
        assert!(world.accept(&CHALLENGE).unwrap());
        assert_eq!(world.integrity_status, Status::UnsupportedLegacyImage);
        world.accept(&[0x28, 5, 0, 0]).unwrap();
        assert_eq!(world.integrity_challenge, None);
        assert_eq!(world.integrity_status, Status::NotRequested);
        assert!(world.accept(&[0xd2]).unwrap());
        assert!(world.accept(&CHALLENGE).unwrap());
        assert_eq!(world.integrity_status, Status::UnsupportedLegacyImage);
        world.clear_preserving_generation();
        assert_eq!(world.integrity_challenge, None);
        assert_eq!(world.integrity_status, Status::NotRequested);
    }
}
