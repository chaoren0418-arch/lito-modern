use super::*;
fn intent() -> Intent {
    Intent {
        request_id: u32::MAX,
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: u64::MAX,
            },
            channel: 3,
            world: [10, 20, 30],
            entity: 1,
            life: 4,
            revision: 100,
        },
        sequence: 25,
        equipment_revision: 9,
        x: 32768,
        y: 32769,
        target: Target {
            entity: 2,
            identity: participant::ParticipantId {
                kind: participant::MemberKind::Human,
                source_id: 42,
                id: 3333,
            },
            life: 5,
            equipment_revision: 8,
            x: 32767,
            y: 32768,
        },
    }
}
#[test]
fn healing_candidate_intent_keeps_exact_scope_and_full_participant_identity() {
    for kind in [
        participant::MemberKind::Human,
        participant::MemberKind::ServerBot,
    ] {
        let mut q = intent();
        q.target.identity.kind = kind;
        let b = encode_candidate_intent(&q).unwrap();
        assert_eq!(b.len(), 160);
        assert_eq!(
            &b[..16],
            &[0x4c, 0x54, 0x4d, 0x31, 0, 0, 0, 160, 255, 255, 255, 255, 0, 78, 0, 0]
        );
        assert_eq!(&b[28..36], &[255; 8]);
        assert_eq!(&b[120..124], &(kind as u32).to_be_bytes());
        assert_eq!(&b[124..128], &42u32.to_be_bytes());
        assert_eq!(&b[128..136], &3333i64.to_be_bytes());
        assert_eq!(decode_candidate_intent(&b).unwrap(), q);
        assert_eq!(encode_candidate_intent(&q).unwrap(), b);
        assert_eq!(format!("{q:?}"), "PlayerHealingIntent[redacted]");
    }
    assert_eq!(crate::modern_lito::protocol::MAX_VERSION, 47);
}
#[test]
fn self_target_cannot_invent_a_second_life_identity_gear_or_cell() {
    let mut q = intent();
    q.target = Target {
        entity: 1,
        identity: participant::ParticipantId {
            kind: participant::MemberKind::Human,
            source_id: 7,
            id: 3333,
        },
        life: 4,
        equipment_revision: 9,
        x: 32768,
        y: 32769,
    };
    assert!(q.validate().is_ok());
    for change in [
        (|q: &mut Intent| q.target.identity.kind = participant::MemberKind::ServerBot)
            as fn(&mut Intent),
        |q| q.target.life += 1,
        |q| q.target.equipment_revision += 1,
        |q| q.target.x += 1,
        |q| q.target.identity.source_id = 42,
    ] {
        let mut bad = q;
        change(&mut bad);
        assert!(bad.validate().is_err());
    }
}
#[test]
fn candidate_intent_rejects_truncation_appended_bytes_unknown_kind_and_invalid_stamps() {
    let b = encode_candidate_intent(&intent()).unwrap();
    for n in 0..b.len() {
        assert!(decode_candidate_intent(&b[..n]).is_err());
    }
    let mut extra = b.clone();
    extra.push(0);
    extra[4..8].copy_from_slice(&161u32.to_be_bytes());
    assert!(decode_candidate_intent(&extra).is_err());
    for offset in [0, 4, 13, 15, 120] {
        let mut bad = b.clone();
        bad[offset] ^= 0x7f;
        assert!(decode_candidate_intent(&bad).is_err(), "{offset}");
    }
    for change in [
        (|q: &mut Intent| q.request_id = 0) as fn(&mut Intent),
        |q| q.stamp.context.epoch = 0,
        |q| q.stamp.world[2] = 0,
        |q| q.stamp.life = 0,
        |q| q.stamp.revision = 0,
        |q| q.sequence = 0,
        |q| q.equipment_revision = -1,
        |q| q.target.entity = 101,
    ] {
        let mut q = intent();
        change(&mut q);
        assert!(encode_candidate_intent(&q).is_err());
    }
}
#[test]
fn reply_header_limits_are_checked_before_allocating_or_consuming_body() {
    let peer = super::super::protocol::extensions::Selected::new(
        super::super::protocol::Ack {
            request_id: 1,
            version: 47,
            numeric_schema: 1,
            capabilities: super::super::protocol::KNOWN_CAPABILITIES,
        },
        3,
    )
    .unwrap();
    for size in [0, 16, 191, MAX_RESULT_BYTES + 1, u32::MAX as usize] {
        let mut b = encode_candidate_intent(&intent()).unwrap();
        b[4..8].copy_from_slice(&(size as u32).to_be_bytes());
        b[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
        let mut stream = std::io::Cursor::new(b);
        assert!(read_candidate_reply(&mut stream, &intent()).is_err());
        assert_eq!(stream.position(), 16);
        stream.set_position(0);
        assert!(read_reply_selected(&mut stream, &intent(), peer).is_err());
        assert_eq!(stream.position(), 16);
    }
    for offset in [0, 8, 13, 15] {
        let mut b = encode_candidate_intent(&intent()).unwrap();
        b[4..8].copy_from_slice(&(RESULT_PREFIX_BYTES as u32).to_be_bytes());
        b[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
        b[offset] ^= 0x7f;
        let mut stream = std::io::Cursor::new(b);
        assert!(read_candidate_reply(&mut stream, &intent()).is_err());
        assert_eq!(stream.position(), 16);
        stream.set_position(0);
        assert!(read_reply_selected(&mut stream, &intent(), peer).is_err());
        assert_eq!(stream.position(), 16);
    }
}
