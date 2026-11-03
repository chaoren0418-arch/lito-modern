use super::*;
#[path = "npc_disintegrate_protocol_golden.rs"]
mod golden;
#[path = "npc_disintegrate_frame_golden.rs"]
mod old_golden;
fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn data(name: &str) -> Vec<u8> {
    bytes(golden::VECTORS.iter().find(|(n, _)| *n == name).unwrap().1)
}
fn reply(name: &str) -> Reply {
    match app::decode_response_version(&data(name), 29)
        .unwrap()
        .response
    {
        app::Response::NpcDisintegrate(r) => r,
        other => panic!("{other:?}"),
    }
}
fn old(name: &str) -> npc::Reply {
    let raw = bytes(
        old_golden::VECTORS
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1,
    );
    match app::decode_response_version(&raw, 29).unwrap().response {
        app::Response::Npc(r) => r,
        other => panic!("{other:?}"),
    }
}
fn ack() -> protocol::Ack {
    protocol::Ack {
        request_id: 1,
        version: 29,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    }
}

#[test]
fn version30_preserves_all_version29_npc77_payloads() {
    for (name, _) in golden::VECTORS {
        let packet = data(name);
        if name.ends_with("request") {
            let request = wire::decode_request(&packet[16..], 30).unwrap();
            assert_eq!(
                encode_request(
                    u32::from_be_bytes(packet[8..12].try_into().unwrap()),
                    &request,
                    30
                )
                .unwrap(),
                packet
            );
        } else {
            assert_eq!(
                app::decode_response_version(&packet, 30).unwrap(),
                app::decode_response_version(&packet, 29).unwrap(),
                "{name}"
            );
        }
    }
}

#[test]
fn fixed_java_requests_preserve_unsigned_epoch_wide_objid_and_distinct_target() {
    assert_eq!(golden::VECTORS.len(), 13);
    for name in ["normal-request", "critical-request", "wide-request"] {
        let packet = data(name);
        assert_eq!(packet.len(), 96);
        let r = wire::decode_request(&packet[16..], 29).unwrap();
        assert_eq!(
            encode_request(
                u32::from_be_bytes(packet[8..12].try_into().unwrap()),
                &r,
                29
            )
            .unwrap(),
            packet
        );
        if name == "wide-request" {
            assert_eq!(r.context.key.object_id, 9_007_199_254_740_993);
            assert_eq!(r.context.epoch, u64::MAX);
            assert_eq!(
                (r.target.serial, r.sequence, r.caster_life, r.target_life),
                (23, 17, 19, 29)
            );
        }
        assert!(encode_request(0, &r, 29).is_err());
        for v in 1..29 {
            assert!(encode_request(91, &r, v).is_err());
        }
    }
}
#[test]
fn actual_java_success_denial_recast_and_cross_type_bridge_follow_atomically() {
    for prefix in ["normal", "critical"] {
        let before = old(&format!("{prefix}-before"));
        let hit = reply(&format!("{prefix}-hit"));
        assert!(hit.echo.ready_in(before.frame.as_ref().unwrap()));
        assert!(hit.confirms(&hit.echo));
        assert!(confirms_transition(before.frame.as_ref().unwrap(), &hit));
        assert!(npc::follows(Some(&before), &hit.npc_reply()));
        let f = hit.frame.as_ref().unwrap();
        assert_eq!(f.combat.vital(1).unwrap().mp, 930);
        assert_eq!(
            f.combat
                .npc_disintegrate(1)
                .unwrap()
                .hit
                .as_ref()
                .unwrap()
                .critical,
            prefix == "critical"
        );
        let denied = reply(&format!("{prefix}-denied"));
        assert_eq!(denied.reason, Reason::TooFast);
        assert!(confirms_transition(f, &denied));
        assert!(npc::follows(Some(&hit.npc_reply()), &denied.npc_reply()));
        assert_eq!(
            denied.frame.as_ref().unwrap().combat.npc_disintegrations,
            f.combat.npc_disintegrations
        );
        let second = reply(&format!("{prefix}-second"));
        assert!(confirms_transition(denied.frame.as_ref().unwrap(), &second));
        assert!(npc::follows(Some(&denied.npc_reply()), &second.npc_reply()));
        let s = second
            .frame
            .as_ref()
            .unwrap()
            .combat
            .npc_disintegrate(1)
            .unwrap();
        assert_eq!((s.count, s.mp_spent, s.lawful), (2, 140, 502));
        assert!(
            !confirms_transition(before.frame.as_ref().unwrap(), &second),
            "one request cannot claim two casts"
        );
    }
}
#[test]
fn sparse_quest_terminal_cooperative_and_absent_unavailable_remain_distinct() {
    let q = reply("quest-hit");
    assert_eq!(
        q.frame.as_ref().unwrap().combat.npc_disintegrations[0].entity_id,
        2
    );
    assert!(q.validate().is_ok());
    let win = reply("time-attack-finished");
    assert_eq!(win.status, npc::Status::Ok);
    assert_eq!(
        win.frame.as_ref().unwrap().combat.world.phase,
        world::Phase::Finished
    );
    let inactive = reply("time-attack-inactive");
    assert_eq!(inactive.reason, Reason::NotActive);
    assert!(confirms_transition(win.frame.as_ref().unwrap(), &inactive));
    assert!(npc::follows(Some(&win.npc_reply()), &inactive.npc_reply()));
    let absent = reply("unavailable");
    assert!(absent.frame.is_none());
    assert_eq!(absent.npc_reply().status, npc::Status::Unavailable);
    assert!(absent.validate().is_ok());
    assert!(!confirms_transition(win.frame.as_ref().unwrap(), &absent));
}
#[test]
fn scope_sequence_source_epoch_channel_and_independent_lives_are_confirmed_exactly() {
    let r = reply("normal-hit");
    let edits: &[fn(&mut Request)] = &[
        |q| q.context.key.source_id += 1,
        |q| q.context.key.object_id += 1,
        |q| q.context.epoch += 1,
        |q| q.channel_id += 1,
        |q| q.target.scope.generation += 1,
        |q| q.target.serial += 1,
        |q| q.sequence += 1,
        |q| q.caster_life += 1,
        |q| q.target_life += 1,
    ];
    for edit in edits {
        let mut altered = r.echo;
        edit(&mut altered);
        assert!(!r.confirms(&altered));
    }
    let before = old("normal-before").frame.unwrap();
    let mut q = r.echo;
    q.caster_life += 1;
    assert!(!q.ready_in(&before));
    q = r.echo;
    q.target_life += 1;
    assert!(!q.ready_in(&before));
    q = r.echo;
    q.sequence += 1;
    assert!(!q.ready_in(&before));
    assert!(!r.echo.ready_in(&old("normal-loading").frame.unwrap()));
}
#[test]
fn fake_success_refunds_reason_lies_and_extra_casts_are_rejected() {
    let r = reply("normal-hit");
    for reason in [
        Reason::Dead,
        Reason::NoMp,
        Reason::Lawful,
        Reason::TargetDead,
        Reason::NotActive,
    ] {
        let mut bad = r.clone();
        bad.status = npc::Status::Denied;
        bad.reason = reason;
        assert!(bad.validate().is_err(), "{reason:?}");
    }
    let mut bad = r.clone();
    bad.status = npc::Status::Busy;
    bad.reason = Reason::TooFast;
    assert!(bad.validate().is_err());
    bad = r.clone();
    bad.frame.as_mut().unwrap().combat.vitals[0].mp += 70;
    assert!(bad.validate().is_err());
    bad = r.clone();
    bad.echo.caster_life += 1;
    assert!(bad.validate().is_err());
    let mut denied = reply("normal-denied");
    denied.status = npc::Status::Ok;
    denied.reason = Reason::None;
    assert!(denied.validate().is_err());
    let mut missing = r.clone();
    missing.frame = None;
    assert!(missing.validate().is_err());
}
#[test]
fn all_application_truncations_reserved_fields_wrong_lengths_and_old_versions_reject() {
    for name in [
        "normal-hit",
        "quest-hit",
        "time-attack-finished",
        "unavailable",
    ] {
        let packet = data(name);
        for n in 0..packet.len() {
            assert!(
                app::decode_response_version(&packet[..n], 29).is_err(),
                "{name}:{n}"
            );
        }
        for v in 1..29 {
            assert!(app::decode_response_version(&packet, v).is_err());
        }
        for at in [15, 37, 38, 39, 96, 97, 98, 99] {
            let mut bad = packet.clone();
            bad[at] = 255;
            assert!(
                app::decode_response_version(&bad, 29).is_err(),
                "{name}:{at}"
            );
        }
        let mut trailing = packet.clone();
        trailing.push(0);
        let n = trailing.len() as u32;
        trailing[4..8].copy_from_slice(&n.to_be_bytes());
        assert!(app::decode_response_version(&trailing, 29).is_err());
    }
}
#[test]
fn no_unnegotiated_response_even_absent_can_bypass_descriptor_gate() {
    assert_eq!(
        protocol::RUNTIME_CAPABILITIES & protocol::CAP_NPC_DISINTEGRATE,
        0
    );
    assert_eq!(protocol::RUNTIME_WORLD_VERSION, 37);
    for name in ["normal-hit", "unavailable"] {
        let response = app::Response::NpcDisintegrate(reply(name));
        assert!(world::descriptor_negotiated(&response, &ack()));
        for bit in [
            protocol::CAP_NPC_DISINTEGRATE,
            protocol::CAP_NPC,
            protocol::CAP_REGEN,
            protocol::CAP_MATCH_DESCRIPTOR,
        ] {
            let mut bad = ack();
            bad.capabilities &= !bit;
            assert!(!negotiated(&bad));
            assert!(!world::descriptor_negotiated(&response, &bad));
        }
        let mut bad = ack();
        bad.version = 28;
        assert!(!world::descriptor_negotiated(&response, &bad));
    }
    for bit in [
        protocol::CAP_NPC,
        protocol::CAP_REGEN,
        protocol::CAP_COMBAT,
        protocol::CAP_MATCH_DESCRIPTOR,
    ] {
        let mut h = protocol::Hello::current(1);
        h.max_version = 29;
        h.supported_capabilities = protocol::KNOWN_CAPABILITIES & !bit;
        assert!(protocol::encode(protocol::Message::Hello(h)).is_err());
    }
}
