//! Java public engine command -> schema5 reply; exact target echo, bounded framing,
//! fragmented reads and full before/after validation. No UI/TLS or source SQL here.
use super::{payload, Peer};
#[test]
#[ignore = "explicit Java engine command contract peer; not registered TLS/UI permission"]
fn java_target_haste_command_pc_bot_npc_echo_denial_and_atomic_history() {
    use crate::modern_lito::battle_target_haste as contract;
    let mut peer = Peer::start_named(Some("haste43-command"));
    let hex = |s: &str| {
        assert_eq!(s.len() % 2, 0);
        assert!(s.len() <= contract::MAX_RESULT_BYTES * 2);
        s.as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>()
    };
    let expected = [
        ("loading", contract::Reason::NotActive),
        ("pc-other", contract::Reason::None),
        ("pc-cooldown", contract::Reason::TooFast),
        ("pc-self", contract::Reason::None),
        ("retired", contract::Reason::NotActive),
        ("last-mp", contract::Reason::None),
        ("no-mp", contract::Reason::NoMp),
        ("unlearned", contract::Reason::Unlearned),
        ("npc", contract::Reason::None),
        ("npc-stale", contract::Reason::StaleTarget),
        ("npc-cooldown", contract::Reason::TooFast),
        ("npc-refresh", contract::Reason::None),
        ("bot", contract::Reason::None),
    ];
    let mut inactive_skip_checks = 0;
    for (label, reason) in expected {
        let line = peer.line();
        let fields = line.splitn(5, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0], "TARGET43");
        assert_eq!(fields[1], label);
        let before = payload::decode_versioned(&hex(fields[2]), 44).unwrap();
        let request = hex(fields[3]);
        let bytes = hex(fields[4]);
        let q = contract::decode_intent(&request, 44).unwrap();
        assert_eq!(q.request_id, u32::MAX);
        assert_eq!(contract::encode_intent(&q, 44).unwrap(), request);
        let reply = contract::decode_reply(&bytes, 44, &q).unwrap();
        // Actual framed reader must tolerate fragmented delivery and leave the
        // following coalesced frame unread; the command echo remains independent.
        struct Fragmented(std::io::Cursor<Vec<u8>>);
        impl std::io::Read for Fragmented {
            fn read(&mut self, dst: &mut [u8]) -> std::io::Result<usize> {
                let n = dst.len().min(3);
                std::io::Read::read(&mut self.0, &mut dst[..n])
            }
        }
        let mut joined = bytes.clone();
        joined.extend_from_slice(b"NEXT");
        let mut stream = Fragmented(std::io::Cursor::new(joined));
        assert_eq!(
            contract::read_reply(&mut stream, &request, 44).unwrap(),
            reply
        );
        assert_eq!(stream.0.position() as usize, bytes.len());
        let mut short = std::io::Cursor::new(&bytes[..bytes.len() - 1]);
        assert!(contract::read_reply(&mut short, &request, 44).is_err());
        assert_eq!(reply.reason, reason);
        assert_eq!(
            reply.validate_after(&q, &before).is_ok(),
            label != "npc-stale",
            "{label}: pending command must account for all own spending"
        );
        assert_eq!(format!("{reply:?}"), "TargetHasteReceipt[redacted]");
        // The stale server-denial frame is valid, but this client would reject
        // constructing/sending that request from its newer accepted observation.
        assert_eq!(q.validate_before(&before).is_ok(), label != "npc-stale");
        if label.starts_with("npc") {
            assert!(matches!(q.target, contract::Target::Npc { .. }));
        } else {
            assert!(matches!(q.target, contract::Target::Player { .. }));
        }
        if label == "bot" {
            assert!(matches!(q.target, contract::Target::Player { kind: 1, .. }));
        }
        if reason == contract::Reason::NotActive {
            let mut skipped = reply.clone();
            let actor = skipped
                .data
                .actors
                .0
                .iter_mut()
                .find(|a| a.entity == q.stamp.entity)
                .unwrap();
            actor.spells.sequence = q.sequence + 1;
            actor.spells.request_revision = skipped.data.world.revision;
            if actor.spells.sequence < actor.spells.request_revision {
                // A locally valid passive snapshot is not a valid reply to a
                // single pending command when it consumes multiple sequences.
                skipped.validate().unwrap();
                inactive_skip_checks += 1;
                assert!(skipped.validate_after(&q, &before).is_err());
            }
        }
        let unchanged = before.clone();
        let mut wrong = q;
        wrong.request_id -= 1;
        assert!(contract::decode_reply(&bytes, 44, &wrong).is_err());
        wrong = q;
        wrong.sequence += 1;
        assert!(contract::decode_reply(&bytes, 44, &wrong).is_err());
        wrong = q;
        match &mut wrong.target {
            contract::Target::Player { source, .. } => *source += 1,
            contract::Target::Npc { serial, .. } => *serial += 1,
        }
        assert!(contract::decode_reply(&bytes, 44, &wrong).is_err());
        wrong = q;
        wrong.stamp.context.epoch ^= 1;
        assert!(contract::decode_reply(&bytes, 44, &wrong).is_err());
        for version in [1, 42, 43, crate::modern_lito::protocol::MAX_VERSION + 1] {
            assert!(contract::decode_reply(&bytes, version, &q).is_err());
        }
        for offset in [12, 14, 121, 123, 185, 187, 211, 212] {
            let mut bad = bytes.clone();
            bad[offset] ^= 127;
            assert!(contract::decode_reply(&bad, 44, &q).is_err());
        }
        let mut bad = bytes.clone();
        bad[184] = 14;
        assert!(contract::decode_reply(&bad, 44, &q).is_err());
        bad = bytes.clone();
        bad[221] = 4;
        assert!(contract::decode_reply(&bad, 44, &q).is_err());
        assert!(contract::decode_reply(&bytes[..bytes.len() - 1], 44, &q).is_err());
        bad = bytes.clone();
        bad.push(0);
        assert!(contract::decode_reply(&bad, 44, &q).is_err());
        let mut changed = reply.clone();
        changed.echo.sequence += 1;
        assert!(changed.validate_after(&q, &before).is_err(), "{label}");
        let mut changed = reply.clone();
        changed.echo.request_id -= 1;
        assert!(changed.validate_after(&q, &before).is_err(), "{label}");
        let mut old = before.clone();
        old.personal.player.source = if old.personal.player.source == 7 {
            42
        } else {
            7
        };
        assert!(reply.validate_after(&q, &old).is_err());
        assert_eq!(
            before, unchanged,
            "failed candidates never consume previous state"
        );
        if reason == contract::Reason::None {
            let mut lie = reply.clone();
            lie.reason = contract::Reason::TooFast;
            assert!(lie.validate().is_err());
        } else if reason != contract::Reason::NotActive {
            let mut lie = reply.clone();
            lie.reason = contract::Reason::None;
            assert!(lie.validate().is_err());
        }
        peer.ack(&bytes);
    }
    assert!(inactive_skip_checks > 0);
    assert_eq!(peer.line(), "CLOSED:13");
    peer.finish();
}
