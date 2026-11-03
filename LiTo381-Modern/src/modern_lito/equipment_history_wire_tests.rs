use super::*;
use crate::modern_lito::equipment_history::{
    self as history,
    tests::{impacts, next, populated},
};
const CELL: WorldCell = WorldCell { x: 32770, y: 32771 };
// This fixture and its golden bytes intentionally describe schema1's 8 lanes.
const LEGACY_LANES: [Lane; 8] = [
    Lane::Teleport,
    Lane::LightArrow,
    Lane::Healing,
    Lane::FireStorm,
    Lane::Disintegrate,
    Lane::NpcLightArrow,
    Lane::NpcFireStorm,
    Lane::NpcDisintegrate,
];
#[test]
fn accepted_single_byte_mutations_are_canonical_not_silently_normalized() {
    let bytes = encode(&populated()).unwrap();
    let mut accepted = 0;
    let mut rejected = 0;
    for offset in 0..bytes.len() {
        let mut mutation = bytes.clone();
        mutation[offset] ^= 1;
        match decode(&mutation) {
            Ok(state) => {
                accepted += 1;
                assert_eq!(mutation, encode(&state).unwrap(), "offset {offset}");
            }
            Err(_) => rejected += 1,
        }
    }
    assert!(accepted > 0);
    assert!(rejected > 2000);
}
#[test]
fn retained_structure_has_one_encoding_and_cannot_be_omitted() {
    let bytes = encode(&populated()).unwrap();
    assert_eq!(bytes.len(), 2572);
    assert_eq!(bytes[1848], 6);
    assert_eq!(bytes[1925], 1);
    let mut dropped = bytes.clone();
    dropped[1925] = 0;
    assert!(decode(&dropped).is_err());
    let mut fabricated = bytes.clone();
    fabricated[333] = 1;
    assert!(decode(&fabricated).is_err());
    let mut duplicate = bytes[..2132].to_vec();
    duplicate.extend(&bytes[1928..2132]);
    duplicate.extend(&bytes[2132..]);
    duplicate[1925] = 2;
    let len = duplicate.len() as u32;
    duplicate[8..12].copy_from_slice(&len.to_be_bytes());
    assert!(decode(&duplicate).is_err());
}
fn round_trip(state: &State) -> State {
    let bytes = encode(state).unwrap();
    let decoded = decode_bound(&bytes, state.identity, state.world_revision).unwrap();
    assert_eq!(&decoded, state);
    assert_eq!(encode(&decoded).unwrap(), bytes);
    decoded
}
#[test]
fn mixed_price_casts_keep_original_spend_and_only_commit_one_lane() {
    let first = round_trip(&populated());
    let mut state = round_trip(&first.reprofile(12, &next(&first, 100, 50)).unwrap());
    assert!(history::follows_reprofile(&first, &state));
    for (i, lane) in LEGACY_LANES.into_iter().enumerate() {
        let old = state.clone();
        state = round_trip(
            &old.cast(
                lane,
                old.world_revision + 1,
                9 + i as i64,
                CELL,
                impacts(lane),
            )
            .unwrap(),
        );
        assert!(history::follows_cast(&old, &state, lane));
        assert!(!history::follows_reprofile(&old, &state));
        let l = state.spells[&lane].current;
        assert_eq!(l.casts, 2);
        assert_eq!(l.spent, 10 + i as i64);
        assert_ne!(l.spent, l.casts * i64::from(l.cost));
        assert_eq!(
            state.spells[&lane].last.unwrap().before.equipment_revision,
            1
        );
        for other in LEGACY_LANES {
            if other != lane {
                assert_eq!(old.spells[&other], state.spells[&other]);
                assert_eq!(old.targets.get(&other), state.targets.get(&other));
            }
        }
        assert_eq!(old.items, state.items);
    }
    assert_eq!(
        (state.resources.flows.spent, state.resources.vitals.mp),
        (108, 22)
    );
}
#[test]
fn raw_replay_mixed_events_missing_targets_and_insufficient_mp_are_rejected_atomically() {
    let before = populated();
    let bytes = encode(&before).unwrap();
    for lane in LEGACY_LANES {
        assert!(before.cast(lane, 12, 8, CELL, impacts(lane)).is_none());
        assert!(before.cast(lane, 11, 9, CELL, impacts(lane)).is_none());
    }
    assert!(before.cast(Lane::LightArrow, 12, 9, CELL, vec![]).is_none());
    let poor = before.reprofile(12, &next(&before, 100, 0)).unwrap();
    assert!(poor
        .cast(Lane::FireStorm, 13, 9, CELL, impacts(Lane::FireStorm))
        .is_none());
    let free = poor.cast(Lane::Teleport, 13, 9, CELL, vec![]).unwrap();
    assert_eq!(free.resources.vitals.mp, 0);
    assert_eq!(free.spells[&Lane::Teleport].current.casts, 2);
    let after = before
        .cast(Lane::LightArrow, 12, 9, CELL, impacts(Lane::LightArrow))
        .unwrap();
    assert!(history::follows_cast(&before, &after, Lane::LightArrow));
    assert!(!history::follows_cast(&before, &before, Lane::LightArrow));
    assert!(!history::follows_cast(&before, &after, Lane::FireStorm));
    let mut mixed = after.clone();
    mixed.resources = mixed.resources.damage(13, 1).unwrap();
    mixed.world_revision = 13;
    assert!(mixed.valid());
    assert!(!history::follows_cast(&before, &mixed, Lane::LightArrow));
    assert_eq!(bytes, encode(&before).unwrap());
    let mut raw = before.clone();
    raw.resources.life = i64::MAX;
    assert!(!raw.valid());
    assert!(raw.cast(Lane::Teleport, 12, 9, CELL, vec![]).is_none());
    assert!(encode(&raw).is_err());
}
#[test]
fn every_prefix_and_appended_bytes_are_rejected_even_with_forged_length() {
    let bytes = encode(&populated()).unwrap();
    for n in 0..bytes.len() {
        let mut truncated = bytes[..n].to_vec();
        if n >= 12 {
            truncated[8..12].copy_from_slice(&(n as u32).to_be_bytes());
        }
        assert!(decode(&truncated).is_err(), "prefix {n}");
    }
    let mut extra = bytes.clone();
    extra.push(0);
    let n = extra.len() as u32;
    extra[8..12].copy_from_slice(&n.to_be_bytes());
    assert!(decode(&extra).is_err());
    assert!(decode(&vec![0; MAX_BYTES + 1]).is_err());
}
#[test]
fn unknown_tags_reserved_bytes_duplicate_lanes_and_forged_totals_fail_closed() {
    let bytes = encode(&populated()).unwrap();
    for offset in [
        0, 5, 6, 7, 8, 57, 61, 69, 70, 71, 254, 255, 257, 258, 259, 280, 281, 282, 283, 334, 335,
        412, 413, 414, 415, 426, 427, 602, 603, 604, 605, 606, 607, 617, 618, 619, 632, 647,
    ] {
        let mut bad = bytes.clone();
        bad[offset] ^= 0x40;
        assert!(decode(&bad).is_err(), "offset {offset}");
    }
    for (offset, value) in [
        (252, 9),
        (253, 2),
        (256, 8),
        (332, 2),
        (333, 3),
        (428, 0),
        (600, 5),
        (601, 2),
    ] {
        let mut bad = bytes.clone();
        bad[offset] = value;
        assert!(decode(&bad).is_err(), "tag {offset}");
    }
    let mut bad = bytes.clone();
    bad[596..598].copy_from_slice(&u16::MAX.to_be_bytes());
    assert!(decode(&bad).is_err());
    let mut bad = bytes.clone();
    bad[220..228].copy_from_slice(&79i64.to_be_bytes());
    assert!(decode(&bad).is_err());
}
#[test]
fn world_source_epoch_actor_channel_and_revision_are_bound_not_inferred_from_objid() {
    let s = populated();
    let bytes = encode(&s).unwrap();
    let mut identities = vec![s.identity; 6];
    identities[0].world.match_id += 1;
    identities[1].source.epoch = u64::MAX;
    identities[2].channel_id = 3;
    identities[3].actor.entity_id = 2;
    identities[4].source.key.source_id = 42;
    identities[4].actor.identity.source_id = 42;
    identities[5].actor.identity.kind = MemberKind::ServerBot;
    for id in identities {
        assert!(decode_bound(&bytes, id, s.world_revision).is_err());
    }
    assert!(decode_bound(&bytes, s.identity, s.world_revision + 1).is_err());
}
#[test]
fn opaque_epoch_and_i64_maximum_resources_do_not_truncate_or_wrap() {
    let mut state = populated();
    state.spells.clear();
    state.targets.clear();
    state.items = None;
    state.world_revision = 1;
    state.resources = vital_balance::State::initial(
        1,
        0,
        Vitals {
            hp: i64::MAX,
            maximum_hp: i64::MAX,
            mp: i64::MAX,
            maximum_mp: i64::MAX,
        },
    )
    .unwrap();
    state.identity.channel_id = 3;
    for epoch in [1, 1u64 << 63, u64::MAX] {
        state.identity.source.epoch = epoch;
        assert_eq!(encode(&state).unwrap().len(), HEADER_BYTES);
        round_trip(&state);
    }
    state.identity.source.epoch = 0;
    assert!(encode(&state).is_err());
}
fn hit(target: Target) -> Effect {
    Effect {
        kind: Kind::Damage,
        before: Health {
            target,
            life: if matches!(target, Target::Participant(_) | Target::Npc(_)) {
                2
            } else {
                1
            },
            equipment_revision: 0,
            cell: CELL,
            hp: 50,
            maximum_hp: 100,
        },
        raw: 100,
        applied: 50,
        hp_after: 0,
    }
}
pub(super) fn maximum() -> State {
    let mut state = populated();
    let mut fire = (2..=100)
        .map(|id| {
            hit(Target::Participant(Participant {
                entity_id: id,
                identity: ParticipantId {
                    kind: if id == 2 {
                        MemberKind::Human
                    } else {
                        MemberKind::ServerBot
                    },
                    source_id: 42,
                    id: if id == 2 { 3333 } else { id },
                },
            }))
        })
        .collect::<Vec<_>>();
    fire.push(hit(Target::Honor {
        entity_id: 101,
        team: 0,
    }));
    fire.push(hit(Target::Honor {
        entity_id: 102,
        team: 1,
    }));
    state = state.cast(Lane::FireStorm, 12, 9, CELL, fire).unwrap();
    let scope = npc::Scope {
        room_id: 10,
        match_id: 20,
        generation: 30,
    };
    let mut targets = (1..=47)
        .map(|serial| hit(Target::Npc(npc::Id { scope, serial })))
        .collect::<Vec<_>>();
    targets.push(hit(Target::TimeTower(time_attack_protocol::TowerId {
        scope,
    })));
    state = state
        .cast(Lane::NpcFireStorm, 13, 10, CELL, targets.clone())
        .unwrap();
    targets[47] = hit(Target::Npc(npc::Id { scope, serial: 48 }));
    state
        .cast(Lane::NpcFireStorm, 14, 11, CELL, targets)
        .unwrap()
}
#[test]
fn maximum_aoe_keeps_prior_structure_without_unbounded_allocation_or_duplicate_encoding() {
    let s = round_trip(&maximum());
    let bytes = encode(&s).unwrap();
    assert!((22000..MAX_BYTES).contains(&bytes.len()));
    assert_eq!(s.targets[&Lane::FireStorm].last.targets.len(), 101);
    let h = &s.targets[&Lane::NpcFireStorm];
    assert_eq!(h.last.targets.len(), 48);
    assert_eq!(h.structure.as_ref().unwrap().targets.len(), 48);
    assert!(matches!(
        h.structure.as_ref().unwrap().targets[47].before.target,
        Target::TimeTower(_)
    ));
    let repriced = round_trip(&s.reprofile(15, &next(&s, 100, 50)).unwrap());
    assert_eq!(s.targets, repriced.targets);
}
