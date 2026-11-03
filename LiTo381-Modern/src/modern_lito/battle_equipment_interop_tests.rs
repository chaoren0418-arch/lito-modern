//! Actual authenticated v41 SET over TLS, real shared LOAD, active state,
//! stale CAS, v40 peer isolation and persistence after returning to the room.
use super::*;
use crate::modern_lito::{battle_equipment as gear, battle_window::Stamp, equipment};

fn observation(c: &ModernTlsConnection) -> crate::modern_lito::battle_follow::Observation {
    c.battle_snapshot().unwrap().observation().unwrap().clone()
}
fn refresh(c: &mut ModernTlsConnection, context: SelectionContext, w: &data::WorldId) {
    let r = c
        .battle_world(battle_world::Request::read(context, CHANNEL, Some(w)))
        .unwrap();
    assert_eq!(r.status, world::Status::Ok);
    assert!(c.world_snapshot().is_none());
}
#[test]
#[ignore = "v41 live equipment TLS + shared514 resources + isolated gear store; no HWND or SQL"]
fn java_v41_live_equipment_tls_shared_load_atomic_cas_and_return_persistence() {
    flow(41);
}
#[test]
#[ignore = "v43 equipment TLS/schema4 with v40 peer, atomic SET/stale CAS/return; no HWND or SQL"]
fn java_v43_live_equipment_tls_schema4_atomic_cas_and_return() {
    flow(43);
}
fn flow(version: u16) {
    let ReadyPair {
        deadline,
        shared,
        mut peer,
        mut a,
        mut b,
        sun,
        moon,
        ready,
    } = ready_pair_mode(version, 40, Some("canonical-equipment"));
    assert!(a.supports_battle_equipment());
    assert!(!b.supports_battle_equipment());
    let original = a
        .equipment(equipment::Request::read(sun))
        .unwrap()
        .snapshot
        .unwrap();
    let other = b
        .equipment(equipment::Request::read(moon))
        .unwrap()
        .snapshot
        .unwrap();
    let item = original
        .flags
        .iter()
        .find(|f| f.equipped)
        .expect("source-qualified equipped weapon")
        .item_id;
    let prepared = a
        .battle_world(battle_world::Request {
            action: battle_world::Action::Prepare,
            room: ready.summary.room_id as i64,
            room_revision: ready.summary.revision,
            ..battle_world::Request::read(sun, CHANNEL, None)
        })
        .unwrap();
    assert_eq!(prepared.status, world::Status::Ok);
    let w = prepared.attachment.unwrap().world;
    loop {
        refresh(&mut a, sun, &w);
        refresh(&mut b, moon, &w);
        if a.battle_snapshot().unwrap().observation().is_some()
            && b.battle_snapshot().unwrap().observation().is_some()
        {
            break;
        }
        deadline.pause("v41 preparation");
    }
    let mut runtimes = [
        BattleRuntime::new(shared.clone()).unwrap(),
        BattleRuntime::new(shared).unwrap(),
    ];
    runtimes[0].attach(observation(&a)).unwrap();
    runtimes[1].attach(observation(&b)).unwrap();
    let mut loads = [None, None];
    let mut keepalive = Instant::now();
    while loads.iter().any(Option::is_none) {
        for (i, c) in [&a, &b].iter().enumerate() {
            match runtimes[i].poll() {
                BattleStatus::Ready(surface) => {
                    loads[i] = runtimes[i].loaded_request(&surface, &observation(c))
                }
                BattleStatus::Failed(reason) => panic!("v41 shared preflight: {reason}"),
                _ => {}
            }
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            a.ping().unwrap();
            b.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("v41 canonical LOAD qualification");
    }
    assert_eq!(
        a.battle_world(loads[0].take().unwrap()).unwrap().status,
        world::Status::Ok
    );
    assert_eq!(
        b.battle_world(loads[1].take().unwrap()).unwrap().status,
        world::Status::Ok
    );
    loop {
        refresh(&mut a, sun, &w);
        refresh(&mut b, moon, &w);
        if observation(&a).data().world.phase == data::Phase::Active {
            break;
        }
        deadline.pause("v41 countdown");
    }
    let old = observation(&a);
    let q = gear::Request::from_observation(&old, &original, item, false).unwrap();
    let retained = a.battle_snapshot().unwrap().clone();
    let mut wrong = q;
    wrong.stamp.context.key.source_id = 42;
    wrong.equipment.context = wrong.stamp.context;
    assert!(a.battle_equipment(wrong).is_err());
    assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
    let guest_q =
        gear::Request::from_observation(&observation(&b), &other, other.flags[0].item_id, true)
            .unwrap();
    assert!(b.battle_equipment(guest_q).is_err());
    b.ping().unwrap();
    let mut stale = q;
    stale.stamp.life += 1;
    let mut controller_state = crate::modern_lito::battle_control::State::default();
    controller_state.align(&a);
    let before = a.battle_snapshot().unwrap().clone();
    controller_state
        .execute(
            &mut a,
            crate::modern_lito::battle_control::Command::equipment(stale),
            1,
        )
        .unwrap();
    assert_eq!(
        controller_state.outcome(),
        Some(crate::modern_lito::battle_control::Outcome::NotSent)
    );
    assert!(controller_state.equipment_reply().is_none());
    assert!(
        Arc::ptr_eq(&before, a.battle_snapshot().unwrap()),
        "late local refusal does not send or change world state"
    );
    a.ping().unwrap();
    assert_eq!(
        a.battle_equipment(stale).unwrap().reason,
        gear::Reason::StaleLife
    );
    let changed = a.battle_equipment(q).unwrap();
    assert_eq!(changed.disposition, gear::Disposition::Applied);
    assert_eq!(changed.durable_revision, 1);
    assert_eq!(observation(&a).data(), &changed.data);
    assert_eq!(old.data().personal.equipment.as_ref().unwrap().revision, 0);
    assert_eq!(
        changed
            .equipment
            .as_ref()
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .equipped(item),
        Some(false)
    );
    let conflict = a.battle_equipment(q).unwrap();
    assert_eq!(conflict.disposition, gear::Disposition::NotCommitted);
    assert_eq!(
        conflict.equipment.as_ref().unwrap().status,
        equipment::Status::Conflict
    );
    let fresh = conflict
        .equipment
        .as_ref()
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    let no_op = gear::Request::from_observation(&observation(&a), fresh, item, false).unwrap();
    let same = a.battle_equipment(no_op).unwrap();
    assert_eq!(same.disposition, gear::Disposition::Unchanged);
    let next = gear::Request::from_observation(
        &observation(&a),
        same.equipment.as_ref().unwrap().snapshot.as_ref().unwrap(),
        item,
        true,
    )
    .unwrap();
    let restored = a.battle_equipment(next).unwrap();
    assert_eq!(restored.disposition, gear::Disposition::Applied);
    assert_eq!(restored.durable_revision, 2);
    refresh(&mut b, moon, &w);
    let guest = observation(&b);
    assert_eq!(
        guest.data().personal.equipment.as_ref().unwrap().revision,
        other.revision
    );
    assert_eq!(
        guest
            .data()
            .actors
            .0
            .iter()
            .find(|x| x.entity == old.data().personal.entity)
            .unwrap()
            .resources
            .as_ref()
            .unwrap()
            .equipment_revision,
        2
    );
    // A real MOVE after re-equipping must still use the new canonical state.
    let active = observation(&a);
    runtimes[0].update(active.clone()).unwrap();
    let surface = worker_surface(&mut runtimes[0], &active, &deadline);
    let actor = active
        .data()
        .actors
        .0
        .iter()
        .find(|x| x.entity == active.data().personal.entity)
        .unwrap();
    let from = WorldCell {
        x: actor.position.x,
        y: actor.position.y,
    };
    let heading = DIRECTIONS
        .iter()
        .enumerate()
        .find_map(|(h, (dx, dy))| {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            (surface
                .navigation
                .as_ref()
                .unwrap()
                .map()
                .can_step(from, to)
                && active
                    .data()
                    .actors
                    .0
                    .iter()
                    .all(|x| x.position.x != to.x || x.position.y != to.y))
            .then_some(h as u8)
        })
        .unwrap();
    let command = world::Request {
        action: world::Action::Move,
        heading,
        sequence: actor.last_move_sequence as u64 + 1,
        from_x: from.x,
        from_y: from.y,
        match_id: w.match_id as u64,
        generation: w.generation as u64,
        ..world::Request::read(sun, CHANNEL)
    };
    let moved = a
        .battle_operation(
            crate::modern_lito::battle_action::Guard {
                stamp: Stamp::of(&active),
                target: None,
            },
            |id, _| world::encode_request(id, &command),
        )
        .unwrap();
    assert_eq!(
        moved.disposition,
        crate::modern_lito::battle_operation::Disposition::Applied
    );
    for r in &mut runtimes {
        r.shutdown();
        join_worker(r, &deadline);
    }
    b.battle_world(battle_world::Request {
        action: battle_world::Action::Leave,
        ..battle_world::Request::read(moon, CHANNEL, Some(&w))
    })
    .unwrap();
    let stopped = a.battle_equipment(next).unwrap();
    assert_eq!(stopped.reason, gear::Reason::NotActive);
    assert_eq!(stopped.disposition, gear::Disposition::NotCommitted);
    assert_eq!(stopped.durable_revision, -1);
    a.battle_world(battle_world::Request {
        action: battle_world::Action::Leave,
        ..battle_world::Request::read(sun, CHANNEL, Some(&w))
    })
    .unwrap();
    assert!(a.battle_equipment(next).is_err());
    let saved = a
        .equipment(equipment::Request::read(sun))
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(saved.revision, 2);
    assert_eq!(saved.equipped(item), Some(true));
    assert_eq!(
        b.equipment(equipment::Request::read(moon))
            .unwrap()
            .snapshot
            .unwrap()
            .revision,
        other.revision
    );
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
