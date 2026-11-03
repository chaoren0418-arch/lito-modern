//! Both peers use v39 end-to-end. No WorldRuntime, legacy frame or legacy
//! world/combat READ is available to qualify either client's LOAD request.
use super::*;
use crate::modern_lito::{battle_action, battle_operation, battle_window::Stamp};

fn rejects_prepare_while_attached(c: &mut ModernTlsConnection, q: &battle_world::Request) {
    let retained = c.battle_snapshot().unwrap().clone();
    assert!(matches!(
        c.battle_world(q.clone()),
        Err(crate::modern_lito::transport::TransportError::InvalidState(
            _
        ))
    ));
    assert!(Arc::ptr_eq(&retained, c.battle_snapshot().unwrap()));
    c.ping().unwrap();
}

#[test]
#[ignore = "two v39 authenticated TLS owners, actual shared preflight, no legacy DTO, SQL or HWND"]
fn java_two_v39_players_load_move_and_leave_without_legacy_resource_input() {
    let ReadyPair {
        deadline,
        shared,
        mut peer,
        mut a,
        mut b,
        sun,
        moon,
        ready,
    } = ready_pair(39, 39);
    let prepare = battle_world::Request {
        action: battle_world::Action::Prepare,
        room: ready.summary.room_id as i64,
        room_revision: ready.summary.revision as i64,
        ..battle_world::Request::read(sun, CHANNEL, None)
    };
    let prepared = a.battle_world(prepare.clone()).unwrap();
    assert_eq!(prepared.state, battle_world::State::BeforeEngine);
    assert!(prepared.data.is_none());
    rejects_prepare_while_attached(&mut a, &prepare);
    let key = prepared.attachment.unwrap().world;
    let request = battle_read::Request {
        context: sun,
        channel_id: CHANNEL,
        world: key.clone(),
    };
    let guest_request = battle_read::Request {
        context: moon,
        ..request.clone()
    };
    let mut owner_follow =
        crate::modern_lito::battle_follow::Follower::new(request.clone(), a.ack().unwrap())
            .unwrap();
    let mut guest_follow =
        crate::modern_lito::battle_follow::Follower::new(guest_request.clone(), b.ack().unwrap())
            .unwrap();
    loop {
        if read(&mut a, &request, &mut owner_follow).data.is_some() {
            break;
        }
        deadline.pause("v39 canonical preparation");
    }
    assert_eq!(
        read(&mut b, &guest_request, &mut guest_follow).state,
        battle_read::State::Data
    );
    let first = owner_follow.observation().unwrap().clone();
    let second = guest_follow.observation().unwrap().clone();
    let mut owner = BattleRuntime::new(shared.clone()).unwrap();
    let mut guest = BattleRuntime::new(shared.clone()).unwrap();
    owner.attach(first.clone()).unwrap();
    guest.attach(second.clone()).unwrap();
    owner.poll();
    guest.poll();
    let mut keepalive = Instant::now();
    let mut surfaces = [None, None];
    while surfaces.iter().any(Option::is_none) {
        for (i, runtime) in [&mut owner, &mut guest].into_iter().enumerate() {
            match runtime.poll() {
                BattleStatus::Ready(surface) => {
                    assert!(runtime.prepared(&surface));
                    assert!(runtime
                        .loaded_request(&surface, if i == 0 { &first } else { &second })
                        .is_some());
                    surfaces[i] = Some(surface);
                }
                BattleStatus::Failed(reason) => {
                    panic!("direct canonical preflight failed: {reason}")
                }
                _ => {}
            }
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            a.ping().unwrap();
            b.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("both v39 shared renderers");
    }
    let [Some(owner_surface), Some(guest_surface)] = surfaces else {
        unreachable!()
    };
    assert!(owner.loaded_request(&guest_surface, &first).is_none());
    assert!(guest.loaded_request(&owner_surface, &second).is_none());
    // A missing shared-resource root must fail, never grant a LOAD permission
    // or fall back to native LiTo packages. It creates/edits no resource file.
    let mut missing =
        BattleRuntime::new(shared.join("__missing_canonical_preflight_fixture__")).unwrap();
    missing.attach(first.clone()).unwrap();
    loop {
        if matches!(missing.poll(), BattleStatus::Failed(_)) {
            break;
        }
        deadline.pause("missing resources fail closed");
    }
    assert!(missing.loaded_request(&owner_surface, &first).is_none());
    missing.shutdown();
    join_worker(&mut missing, &deadline);
    read(&mut a, &request, &mut owner_follow);
    read(&mut b, &guest_request, &mut guest_follow);
    assert!(owner_follow
        .observation()
        .unwrap()
        .data()
        .actors
        .0
        .iter()
        .all(|p| !p.loaded));
    let load_a = owner
        .loaded_request(&owner_surface, owner_follow.observation().unwrap())
        .unwrap();
    let load_b = guest
        .loaded_request(&guest_surface, guest_follow.observation().unwrap())
        .unwrap();
    for (connection, q) in [(&mut a, load_a), (&mut b, load_b)] {
        let loaded = connection.battle_world(q).unwrap();
        assert_eq!(loaded.status, world::Status::Ok);
        assert_eq!(loaded.state, battle_world::State::Data);
    }
    loop {
        let phase = read(&mut a, &request, &mut owner_follow)
            .data
            .unwrap()
            .world
            .phase;
        read(&mut b, &guest_request, &mut guest_follow);
        if phase == data::Phase::Active {
            break;
        }
        assert_eq!(phase, data::Phase::Countdown);
        deadline.pause("v39 actual countdown");
    }
    let active = owner_follow.observation().unwrap().clone();
    assert!(owner.loaded_request(&owner_surface, &active).is_none());
    owner.update(active.clone()).unwrap();
    let surface = worker_surface(&mut owner, &active, &deadline);
    let actor = active
        .data()
        .actors
        .0
        .iter()
        .find(|p| p.entity == active.data().personal.entity)
        .unwrap();
    let from = WorldCell {
        x: actor.position.x,
        y: actor.position.y,
    };
    let navigation = surface.navigation.as_ref().unwrap();
    let (heading, target) = DIRECTIONS
        .iter()
        .enumerate()
        .map(|(heading, &(x, y))| {
            (
                heading as u8,
                WorldCell {
                    x: from.x + x,
                    y: from.y + y,
                },
            )
        })
        .find(|(_, to)| {
            navigation.map().can_step(from, *to)
                && active
                    .data()
                    .actors
                    .0
                    .iter()
                    .all(|p| p.position.x != to.x || p.position.y != to.y)
        })
        .unwrap();
    let command = world::Request {
        action: world::Action::Move,
        heading,
        sequence: 1,
        from_x: from.x,
        from_y: from.y,
        match_id: key.match_id as u64,
        generation: key.generation as u64,
        ..world::Request::read(sun, CHANNEL)
    };
    let moved = a
        .battle_operation(
            battle_action::Guard {
                stamp: Stamp::of(&active),
                target: None,
            },
            |id, _| world::encode_request(id, &command),
        )
        .unwrap();
    assert_eq!(moved.disposition, battle_operation::Disposition::Applied);
    rejects_prepare_while_attached(&mut a, &prepare);
    let moved_snapshot = a.battle_snapshot().unwrap().clone();
    assert_eq!(moved_snapshot.observation().unwrap().data(), &moved.data);
    assert_eq!(
        moved_snapshot.attachment().unwrap().lifecycle,
        moved.lifecycle
    );
    read(&mut a, &request, &mut owner_follow);
    let confirmed = owner_follow.observation().unwrap().clone();
    owner.update(confirmed.clone()).unwrap();
    let image = worker_surface(&mut owner, &confirmed, &deadline);
    assert_eq!(
        image
            .geometry
            .actors
            .iter()
            .find(|p| p.entity_id == actor.entity as u64)
            .unwrap()
            .cell,
        target
    );
    let left = b
        .battle_world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(moon, CHANNEL, Some(&key))
        })
        .unwrap();
    assert_eq!(left.state, battle_world::State::Detached);
    assert_world_snapshot(&b, &left);
    read(&mut a, &request, &mut owner_follow);
    let aborted = owner_follow.observation().unwrap().clone();
    assert_eq!(aborted.data().world.phase, data::Phase::Aborted);
    rejects_prepare_while_attached(&mut a, &prepare);
    owner.update(aborted).unwrap();
    assert!(owner.loaded_request(&owner_surface, &first).is_none());
    assert_eq!(
        a.battle_world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(sun, CHANNEL, Some(&key))
        })
        .unwrap()
        .state,
        battle_world::State::Detached
    );
    for connection in [&mut a, &mut b] {
        assert!(connection.world_snapshot().is_none() && connection.combat_snapshot().is_none());
        assert!(connection.battle_snapshot().unwrap().attachment().is_none());
    }
    assert_eq!(
        moved_snapshot.observation().unwrap().data(),
        &moved.data,
        "retained state remains immutable after leave"
    );
    owner.shutdown();
    guest.shutdown();
    join_worker(&mut owner, &deadline);
    join_worker(&mut guest, &deadline);
    assert!(owner.is_quiescent() && guest.is_quiescent());
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
