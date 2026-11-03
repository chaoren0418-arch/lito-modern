use super::super::{
    application_protocol as app, npc_protocol as npc, protocol, world_protocol as world,
};
use super::tests::bytes;

#[path = "npc_disintegrate_frame_golden.rs"]
mod golden;

fn data(name: &str) -> Vec<u8> {
    bytes(golden::VECTORS.iter().find(|(n, _)| *n == name).unwrap().1)
}
fn reply(name: &str) -> npc::Reply {
    let decoded =
        app::decode_response_version(&data(name), 29).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    match decoded.response {
        app::Response::Npc(r) => r,
        other => panic!("{name}: unexpected {other:?}"),
    }
}
fn frame(name: &str) -> npc::Frame {
    reply(name).frame.unwrap()
}

#[test]
fn java_complete_frames_decode_with_exact_mana_identity_effects_and_names() {
    assert_eq!(golden::VECTORS.len(), 19);
    for (name, _) in golden::VECTORS {
        let r = reply(name);
        r.validate().unwrap();
        assert!(npc::follows(None, &r), "{name}");
        let f = r.frame.unwrap();
        assert!(
            f.names.as_ref().is_some_and(|names| !names.is_empty()),
            "{name}: real NPC name table is mandatory"
        );
        assert_ne!(
            f.combat.world.descriptor.unwrap().features & world::MatchDescriptor::NPC_DISINTEGRATE,
            0
        );
    }
    for prefix in ["normal", "critical"] {
        let f = frame(&format!("{prefix}-hit"));
        let s = f.combat.npc_disintegrate(1).unwrap();
        assert_eq!((s.count, s.mp_spent, s.lawful), (1, 70, 501));
        assert_eq!(f.combat.vital(1).unwrap().mp, 930);
        assert_eq!(f.combat.world.cast(1).unwrap().mp_spent, 0);
        let h = s.hit.as_ref().unwrap();
        assert_eq!(h.critical, prefix == "critical");
        assert_eq!(h.effect(), if h.critical { 11748 } else { 11746 });
        assert_eq!(f.actor(h.target).unwrap().hp, h.remaining_hp);
        assert_eq!(
            h.target.serial, s.entity_id,
            "equal numbers must retain different PC/NPC identity types"
        );
        let second = frame(&format!("{prefix}-second"));
        let s = second.combat.npc_disintegrate(1).unwrap();
        assert_eq!(
            (s.count, s.last_sequence, s.mp_spent, s.lawful),
            (2, 4, 140, 502)
        );
    }
}

#[test]
fn actual_java_loading_hit_denial_mixed_arrow_recast_regen_are_monotonic() {
    for prefix in ["normal", "critical"] {
        let mut prior = None;
        for suffix in [
            "loading", "before", "hit", "denied", "arrow", "second", "regen",
        ] {
            let r = reply(&format!("{prefix}-{suffix}"));
            assert!(npc::follows(prior.as_ref(), &r), "{prefix}-{suffix}");
            if let Some(p) = &prior {
                assert!(!npc::follows(Some(&r), p), "rewind {prefix}-{suffix}");
            }
            prior = Some(r);
        }
        assert!(
            npc::follows(
                Some(&reply(&format!("{prefix}-before"))),
                &reply(&format!("{prefix}-regen"))
            ),
            "skipped full snapshots"
        );
    }
}

#[test]
fn sparse_quest_keeps_entity_two_and_retired_corpse_receipt_without_cross_team_leak() {
    let own = reply("quest-own");
    let other = reply("quest-other");
    let retired = reply("quest-retired");
    let f = own.frame.as_ref().unwrap();
    assert_eq!(f.combat.npc_disintegrations.len(), 1);
    let s = &f.combat.npc_disintegrations[0];
    assert_eq!((s.entity_id, s.count, s.mp_spent), (2, 1, 70));
    assert_eq!(f.combat.world.audience_lane, Some(1));
    assert_eq!(f.combat.vital(2).unwrap().mp, 930);
    let hidden = &other.frame.as_ref().unwrap().combat.npc_disintegrations[0];
    assert_eq!((hidden.entity_id, hidden.count), (1, 0));
    assert!(!npc::follows(Some(&own), &other));
    assert!(npc::follows(Some(&own), &retired));
    let after = retired.frame.unwrap();
    assert!(after.actor(s.hit.as_ref().unwrap().target).is_none());
    assert_eq!(&after.combat.npc_disintegrations[0], s);
}

#[test]
fn time_attack_saved_envelope_keeps_terminal_damage_and_mana_receipt() {
    let pending = reply("time-attack-finished");
    let saved = reply("time-attack-saved");
    assert!(npc::follows(Some(&pending), &saved));
    assert!(!npc::follows(Some(&saved), &pending));
    let a = pending.frame.unwrap();
    let b = saved.frame.unwrap();
    assert_eq!(a.combat.world.revision + 1, b.combat.world.revision);
    assert_eq!(a.combat.npc_disintegrations, b.combat.npc_disintegrations);
    let s = b.combat.npc_disintegrate(1).unwrap();
    assert!(s.hit.as_ref().unwrap().death);
    assert_eq!(s.revision, a.combat.world.revision);
    let mut changed = b.clone();
    changed.combat.npc_disintegrations[0].revision = b.combat.world.revision;
    assert!(
        changed.validate().is_err(),
        "settlement envelope cannot fabricate a new cast"
    );
}

#[test]
fn dropped_ledger_wrong_mana_life_cell_scope_and_borrowed_spell_revision_fail_closed() {
    let base = frame("normal-hit");
    let edits: &[fn(&mut npc::Frame)] = &[
        |f| f.combat.npc_disintegrations.clear(),
        |f| f.combat.npc_disintegrations[0] = super::State::initial(1, 70, 500),
        |f| f.combat.vitals[0].mp += 1,
        |f| {
            f.combat.npc_disintegrations[0]
                .hit
                .as_mut()
                .unwrap()
                .caster_life += 1
        },
        |f| {
            f.combat.npc_disintegrations[0]
                .hit
                .as_mut()
                .unwrap()
                .target_life += 1
        },
        |f| {
            f.combat.npc_disintegrations[0]
                .hit
                .as_mut()
                .unwrap()
                .target
                .scope
                .generation += 1
        },
        |f| f.combat.npc_disintegrations[0].hit.as_mut().unwrap().to.x += 1,
        |f| f.combat.npc_disintegrations[0].hit.as_mut().unwrap().lane = 1,
        |f| f.combat.world.casts[0].request_sequence = 0,
        |f| f.actors[0].hp += 1,
    ];
    for (i, edit) in edits.iter().enumerate() {
        let mut bad = base.clone();
        edit(&mut bad);
        assert!(bad.validate().is_err(), "corruption {i}");
    }
    let mut mixed = frame("normal-arrow");
    mixed.combat.npc_disintegrations[0].revision = mixed.combat.npc_spells[0].revision;
    assert!(
        mixed.validate().is_err(),
        "NPC77 cannot borrow light-arrow revision"
    );
}

#[test]
fn changing_history_or_configuration_in_newer_snapshot_is_rejected() {
    let old = reply("normal-hit");
    for edit in [
        (|f: &mut npc::Frame| {
            f.combat.npc_disintegrations[0]
                .hit
                .as_mut()
                .unwrap()
                .critical = true
        }) as fn(&mut npc::Frame),
        |f| f.combat.npc_disintegrations[0].initial_lawful += 1,
        |f| f.names.as_mut().unwrap().clear(),
        |f| f.combat.npc_disintegrations[0].last_sequence += 1,
    ] {
        let mut bad = reply("normal-denied");
        edit(bad.frame.as_mut().unwrap());
        assert!(!npc::follows(Some(&old), &bad));
    }
    let mut old_budget = frame("normal-before").combat;
    let mut new_budget = frame("normal-second").combat;
    // Individual ledgers may look monotonic while collectively spending more
    // than the single shared request-sequence budget.
    old_budget.world.casts[0].request_sequence = 2;
    new_budget.world.casts[0].request_sequence = 4;
    assert!(!super::frames_follow(&old_budget, &new_budget));
}

#[test]
fn full_application_truncation_reserved_bits_and_prior_versions_are_rejected() {
    for name in [
        "normal-loading",
        "normal-hit",
        "quest-own",
        "time-attack-saved",
    ] {
        let packet = data(name);
        for n in 0..packet.len() {
            assert!(
                app::decode_response_version(&packet[..n], 29).is_err(),
                "{name}:{n}"
            );
        }
        for version in 1..29 {
            assert!(
                app::decode_response_version(&packet, version).is_err(),
                "{name}:v{version}"
            );
        }
        let mut trailing = packet.clone();
        trailing.push(0);
        assert!(app::decode_response_version(&trailing, 29).is_err());
    }
    let f = frame("normal-hit");
    let section = super::encode_states(
        &f.combat.npc_disintegrations,
        f.scope,
        f.combat.world.actors.len(),
        29,
    )
    .unwrap();
    let packet = data("normal-hit");
    let start = packet
        .windows(section.len())
        .position(|b| b == section)
        .unwrap();
    for offset in [0, 2, 7, 8 + 64, 8 + 66, 8 + 143] {
        let mut bad = packet.clone();
        bad[start + offset] ^= 0x40;
        assert!(
            app::decode_response_version(&bad, 29).is_err(),
            "section offset {offset}"
        );
    }
}

#[test]
fn diagnostic_offer_stays_opt_in_and_runtime_plan_pins_npc77_resources() {
    assert_eq!(protocol::MAX_VERSION, 47);
    assert_eq!(
        protocol::RUNTIME_CAPABILITIES & protocol::CAP_NPC_DISINTEGRATE,
        0
    );
    assert_eq!(protocol::RUNTIME_WORLD_VERSION, 37);
    let r = reply("normal-loading");
    let admitted = super::super::npc_presentation::Plan::from_reply(&r).unwrap();
    assert!(admitted.validate(&r).is_ok());
    let f = r.frame.as_ref().unwrap();
    let plan = super::super::npc_presentation::Plan::new(
        f.scope,
        f.resources
            .as_ref()
            .unwrap()
            .iter()
            .map(|g| super::super::npc_assets::NpcAppearance::new(*g).unwrap())
            .collect(),
    )
    .unwrap();
    assert!(
        plan.validate(&r)
            .unwrap_err()
            .to_string()
            .contains("NPC77 resource capability changed"),
        "manually constructed plans cannot bypass admission"
    );
}
