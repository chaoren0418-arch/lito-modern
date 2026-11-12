//! Offline lifecycle replays plus one owned synthetic loopback transport test.
//! No production server, game process or client-side revive timer.
use super::*;
use crate::modern_lito::{native_combat, native_navigation, native_skill_selection, native_world};

fn world() -> native_world::World {
    super::super::native_world_scene::movement_tests::world()
}

fn resurrection(target: u32, source: u32, action: u8, graphics: u16) -> Vec<u8> {
    let mut body = vec![0x11];
    body.extend(target.to_le_bytes());
    body.push(action);
    body.extend(source.to_le_bytes());
    body.extend(graphics.to_le_bytes());
    body
}

fn death(w: &mut native_world::World) {
    let mut body = vec![0x20];
    body.extend(w.own_actor().unwrap().id.to_le_bytes());
    body.push(8);
    w.accept(&[0x0d, 0, 0, 200, 0]).unwrap();
    w.accept(&body).unwrap();
    w.accept(&[0xe3]).unwrap();
}

fn moved(id: u32, source: [u16; 2]) -> Vec<u8> {
    let mut body = vec![0x12];
    body.extend(id.to_le_bytes());
    for value in source {
        body.extend(value.to_le_bytes());
    }
    body.push(0);
    body
}

fn ratio(id: u32) -> Vec<u8> {
    let mut body = vec![0x68];
    body.extend(id.to_le_bytes());
    body.push(100);
    body
}

fn controller() -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = LoginPhase::BattlePending;
    c.state.world = world();
    // No unrelated emblem-download transport in the offline controller fixture.
    for actor in c.state.world.actors.values_mut() {
        actor.emblem = 0;
    }
    c
}

fn peer(w: &mut native_world::World) -> u32 {
    let mut a = w.own_actor().unwrap().clone();
    a.id = 27;
    a.x += 1;
    w.actors.insert(a.id, a);
    27
}

#[test]
fn death_ack_is_transactional_once_per_life_and_never_revives_locally() {
    let mut w = world();
    death(&mut w);
    let before = serde_json::to_value(&w).unwrap();
    assert!(w
        .acknowledge_death(|_| anyhow::bail!("queue full"))
        .is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    for life in 1..=2 {
        let mut sent = 0;
        w.acknowledge_death(|body| {
            assert_eq!(body, [0xe0]);
            sent += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(sent, 1);
        assert_eq!(w.death_confirmations_sent, life);
        assert!(w.own_actor().unwrap().dead);
        assert_eq!(w.hp.unwrap()[0], 0);
        assert!(w.acknowledge_death(|_| panic!("duplicate CE0")).is_err());
        assert!(w.accept(&[0xe3]).is_err());
        let id = w.own_actor().unwrap().id;
        w.accept(&resurrection(id, id, 0, 0)).unwrap();
        assert!(!w.death_confirmation_ready());
        assert!(w.move_request(0).is_err(), "S11 cannot invent HP");
        w.accept(&[0x0d, 200, 0, 200, 0]).unwrap();
        assert!(w.move_request(0).is_ok());
        if life == 1 {
            death(&mut w);
        }
    }
}

#[test]
fn death_ack_rechecks_health_role_terminal_state_and_counter_before_sending() {
    let mut w = world();
    death(&mut w);
    for case in 0..6 {
        let mut changed = w.clone();
        match case {
            0 => changed.accept(&[0x0d, 1, 0, 200, 0]).unwrap(),
            1 => {
                changed.observer_mode = true;
                true
            }
            2 => changed.accept(&[0xd3, 0, 0, 0, 0]).unwrap(),
            3 => {
                changed.actors.clear();
                true
            }
            4 => {
                changed.death_confirmations_sent = u64::MAX;
                true
            }
            _ => {
                changed.last_death_confirmed_action =
                    Some(changed.own_actor().unwrap().action_sequence);
                true
            }
        };
        let before = serde_json::to_value(&changed).unwrap();
        assert!(changed
            .acknowledge_death(|_| panic!("invalid death sent CE0"))
            .is_err());
        assert_eq!(serde_json::to_value(&changed).unwrap(), before);
    }
}

#[test]
fn resurrection_parse_failures_and_exhausted_counters_are_atomic() {
    let mut w = world();
    death(&mut w);
    let id = w.own_actor().unwrap().id;
    let body = resurrection(id, u32::MAX, 255, u16::MAX);
    let mut invalid = (0..body.len())
        .map(|n| body[..n].to_vec())
        .collect::<Vec<_>>();
    let mut trailing = body.clone();
    trailing.push(0);
    invalid.push(trailing);
    invalid.push(resurrection(0, id, 0, 0));
    invalid.push(resurrection(id, 0, 0, 0));
    for body in invalid {
        let mut actual = w.clone();
        let before = serde_json::to_value(&actual).unwrap();
        assert!(actual.accept(&body).is_err());
        assert_eq!(serde_json::to_value(actual).unwrap(), before);
    }
    for counter in 0..2 {
        let mut actual = w.clone();
        if counter == 0 {
            actual.visual_revision = u64::MAX;
        } else {
            actual.resurrection_packets = u64::MAX;
        }
        let before = serde_json::to_value(&actual).unwrap();
        assert!(actual.accept(&body).is_err());
        assert_eq!(serde_json::to_value(actual).unwrap(), before);
    }
    w.accept(&body).unwrap();
    assert_eq!(w.last_resurrection.unwrap().source, u32::MAX);
    assert_eq!(w.own_actor().unwrap().graphics, u16::MAX);
    assert_eq!(w.own_actor().unwrap().walk_action, 255);
}

#[test]
fn spawn_relocation_snaps_once_and_honor_vitals_preserve_next_real_walk() {
    let mut w = world();
    let id = peer(&mut w);
    // E149 respawnAtBattleSpawn: S11, converted source-cell S12, HP/ratio.
    w.accept(&resurrection(id, id, 0, 61)).unwrap();
    w.accept(&moved(id, [33000, 33001])).unwrap();
    assert_eq!([w.actors[&id].x, w.actors[&id].y], [33000, 33000]);
    assert_eq!(w.actors[&id].motion_from, None);
    w.accept(&moved(id, [33000, 33000])).unwrap();
    assert_eq!(w.actors[&id].motion_from, Some([33000, 33000]));
    // Honor and third-party spell revivals carry no relocation; ratio closes
    // the window so the first subsequent walk still animates from its source.
    for source in [id, 99] {
        let position = [w.actors[&id].x, w.actors[&id].y];
        w.accept(&resurrection(id, source, 0, 61)).unwrap();
        assert_eq!([w.actors[&id].x, w.actors[&id].y], position);
        w.accept(&ratio(id)).unwrap();
        w.accept(&moved(id, position)).unwrap();
        assert_eq!(w.actors[&id].motion_from, Some(position));
    }
}

#[test]
fn resurrection_window_retires_on_remove_replacement_map_and_settlement() {
    let w = world();
    let id = w.own_actor().unwrap().id;
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-world-current.json"
    ))
    .unwrap();
    for name in ["actor", "world"] {
        let mut actual = w.clone();
        actual.accept(&resurrection(id, id, 0, 0)).unwrap();
        let hex = fixture["packets"][name].as_str().unwrap();
        let body: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        actual.accept(&body).unwrap();
        assert!(actual.resurrection_relocations.is_empty());
    }
    let mut actual = w.clone();
    actual.accept(&resurrection(id, id, 0, 0)).unwrap();
    let mut remove = vec![0x15];
    remove.extend(id.to_le_bytes());
    actual.accept(&remove).unwrap();
    assert!(actual.resurrection_relocations.is_empty());
    actual.accept(&resurrection(id, id, 0, 0)).unwrap();
    assert!(!actual.actors.contains_key(&id));
    assert!(actual.resurrection_relocations.is_empty());
    let mut actual = w;
    death(&mut actual);
    actual.accept(&[0xd3, 0, 0, 0, 0]).unwrap();
    let before = serde_json::to_value(&actual).unwrap();
    actual.accept(&resurrection(id, id, 0, 0)).unwrap();
    assert_eq!(serde_json::to_value(actual).unwrap(), before);
}

#[test]
fn coalesced_death_and_resurrection_retire_held_inputs_without_local_health() {
    let mut w = world();
    let other = peer(&mut w);
    let id = w.own_actor().unwrap().id;
    let mut attack = native_combat::HeldAttack::default();
    attack.press(other, 7, &w);
    let selection = native_skill_selection::Target::capture(&w, id).unwrap();
    let stamp = native_navigation::InputStamp::capture(7, &w);
    death(&mut w);
    w.accept(&resurrection(id, other, 0, 61)).unwrap();
    w.accept(&[0x0d, 100, 0, 200, 0]).unwrap();
    assert!(selection.resolve(&w).is_none());
    assert_ne!(stamp, native_navigation::InputStamp::capture(7, &w));
    assert_eq!(
        attack.ready(7, &w, native_combat::Feedback::Idle, true, true),
        None
    );
    assert!(!attack.active());
    attack.press(other, 7, &w);
    assert_eq!(
        attack.ready(7, &w, native_combat::Feedback::Idle, true, true),
        Some(other)
    );
}

#[test]
fn controller_revival_retires_old_life_work_but_preserves_withdrawal_deadline() {
    let mut c = controller();
    let id = c.state.world.own_actor().unwrap().id;
    let other = peer(&mut c.state.world);
    let now = Instant::now();
    c.deadline = Some(now + Duration::from_secs(8));
    c.state.battle_withdrawal_pending = true;
    c.attack_pending = Some(native_combat::Pending {
        since: now,
        count: 0,
        target: other,
        generation: c.state.world.generation,
        messages: 0,
    });
    c.skill_pending = Some(super::super::native_skills::Pending {
        since: now,
        casts: 0,
        messages: 0,
        generation: c.state.world.generation,
        incarnation: c.state.world.own_actor().unwrap().incarnation,
    });
    c.skill_item_pending = true;
    c.skill_ready = Some(now + Duration::from_secs(10));
    let stamp = native_navigation::InputStamp::capture(0, &c.state.world);
    c.world_tree_request = Some((stamp, now));
    c.teleport_spell = Some((stamp, now, c.state.world.map_id.unwrap()));
    death(&mut c.state.world);
    c.accept_game(&resurrection(id, id, 0, 61)).unwrap();
    assert!(c.attack_pending.is_none() && c.skill_pending.is_none() && c.skill_ready.is_none());
    assert!(!c.skill_item_pending);
    assert!(c.world_tree_request.is_none() && c.teleport_spell.is_none());
    assert_eq!(
        c.state.attack_feedback,
        native_combat::Feedback::Interrupted
    );
    assert_eq!(
        c.state.item_feedback,
        super::super::native_item_actions::Feedback::Interrupted
    );
    assert_eq!(
        c.state.pickup_feedback,
        super::super::native_ground_pickup::Feedback::Interrupted
    );
    assert_eq!(c.deadline, Some(now + Duration::from_secs(8)));
    assert!(c.state.battle_withdrawal_pending);
    assert_eq!(c.state.world.hp.unwrap()[0], 0);
    assert_eq!(c.state.phase, LoginPhase::BattlePending);
}

#[test]
fn peer_or_orphan_revival_does_not_clear_own_death_or_unrelated_work() {
    let mut c = controller();
    let other = peer(&mut c.state.world);
    death(&mut c.state.world);
    let own = c.state.world.own_actor().unwrap().clone();
    c.skill_ready = Some(Instant::now() + Duration::from_secs(10));
    for target in [99, other] {
        c.attack_pending = Some(native_combat::Pending {
            since: Instant::now(),
            count: 0,
            target: other,
            generation: c.state.world.generation,
            messages: 0,
        });
        c.accept_game(&resurrection(target, target, 0, 61)).unwrap();
        assert_eq!(c.state.world.own_actor().unwrap(), &own);
        assert!(c.state.world.death_confirmation_pending);
        assert!(c.skill_ready.is_some());
        assert_eq!(c.attack_pending.is_none(), target == other);
    }
}

#[test]
fn delayed_authority_keeps_dead_world_and_can_finish_instead_of_reviving() {
    let mut c = controller();
    death(&mut c.state.world);
    c.state.world.acknowledge_death(|_| Ok(())).unwrap();
    let dead = c.state.world.own_actor().unwrap().clone();
    // Polling has no respawn producer. No speculative HP, second CE0, or
    // synthetic 5-second deadline is installed by the death acknowledgement.
    for _ in 0..3 {
        c.poll();
    }
    assert_eq!(c.state.phase, LoginPhase::BattlePending);
    assert_eq!(c.state.world.own_actor().unwrap(), &dead);
    assert_eq!(c.state.world.death_confirmations_sent, 1);
    assert!(c.deadline.is_none());
    c.accept_game(&[0xd3, 0, 0, 0, 0]).unwrap();
    assert_eq!(c.state.phase, LoginPhase::BattleResult);
    c.accept_game(&resurrection(dead.id, dead.id, 0, 0))
        .unwrap();
    assert!(c.state.world.own_actor().unwrap().dead);
    assert_eq!(c.state.world.resurrection_packets, 0);
    c.cancel();
    assert!(c.state.world.actors.is_empty());
}

#[test]
fn actual_ce0_transport_waits_beyond_eight_seconds_for_synthetic_honor_revival() -> Result<()> {
    use super::super::{native_connection, native_wire};
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    fn read_body(socket: &mut TcpStream) -> Result<Vec<u8>> {
        let mut header = [0; 2];
        socket.read_exact(&mut header)?;
        let length = usize::from(u16::from_le_bytes(header));
        ensure!((3..=64).contains(&length), "synthetic wire frame bound");
        let mut body = vec![0; length - 2];
        socket.read_exact(&mut body)?;
        Ok(body)
    }

    let mut c = controller();
    death(&mut c.state.world);
    let id = c.state.world.own_actor().unwrap().id;
    let frame =
        super::super::native_world_scene::resurrection_tests::completed_frame(&c.state.world);
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let address = listener.local_addr()?;
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-version-current.json"
    ))?;
    let hex = fixture["accepted"].as_str().unwrap();
    let accepted: Vec<_> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    let server = thread::spawn(move || -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    ensure!(Instant::now() < deadline, "owned accept timeout");
                    thread::sleep(Duration::from_millis(2));
                }
                Err(e) => return Err(e.into()),
            }
        };
        socket.set_nonblocking(false)?;
        socket.set_read_timeout(Some(Duration::from_secs(15)))?;
        socket.set_write_timeout(Some(Duration::from_secs(5)))?;
        let seed = 0xfffffffe;
        socket.write_all(&[7, 0, 1, 0xfe, 0xff, 0xff, 0xff])?;
        let mut rx = native_wire::CipherState::new(seed);
        let mut version = read_body(&mut socket)?;
        rx.decrypt(&mut version);
        ensure!(version[0] == 0, "expected C_VERSION");
        let mut tx = native_wire::CipherState::new(seed);
        let mut accepted = accepted;
        tx.encrypt(&mut accepted);
        socket.write_all(&native_wire::frame(&accepted)?)?;
        let mut confirmation = read_body(&mut socket)?;
        rx.decrypt(&mut confirmation);
        ensure!(confirmation == [0xe0], "expected exactly CE0");
        // Deliberately exceed the former 8s login deadline. This is a synthetic
        // delayed delivery, not a claim that E149 schedules Honor at 8.5s.
        thread::sleep(Duration::from_millis(8500));
        for mut body in [
            resurrection(id, id, 0, 0),
            vec![0x0d, 100, 0, 200, 0],
            ratio(id),
        ] {
            tx.encrypt(&mut body);
            socket.write_all(&native_wire::frame(&body)?)?;
        }
        ensure!(socket.read(&mut [0])? == 0, "unexpected retry after CE0");
        Ok(())
    });

    let result = (|| -> Result<()> {
        let epoch = c
            .transport
            .start(address, native_connection::Limits::default())?;
        let handshake_end = Instant::now() + Duration::from_secs(6);
        while c.transport.snapshot().phase != native_connection::Phase::VersionAccepted {
            c.transport.poll();
            ensure!(
                Instant::now() < handshake_end,
                "owned handshake timeout: {:?}",
                c.transport.snapshot()
            );
            thread::sleep(Duration::from_millis(2));
        }
        // Fixture authority is explicit; only the production encrypted transport
        // and confirmation/revival controller paths are under test here.
        c.game = true;
        c.state.game_connected = true;
        let before = c.transport.queued_tx_counter();
        c.confirm_death(&frame)?;
        ensure!(
            c.transport.queued_tx_counter() == before.map(|n| n + 1),
            "one CE0 queued"
        );
        ensure!(
            c.confirm_death(&frame).is_err(),
            "same frame confirmed twice"
        );
        let start = Instant::now();
        while c.state.world.hp.is_none_or(|hp| hp[0] == 0) {
            c.poll();
            ensure!(
                c.state.phase == LoginPhase::BattlePending,
                "death wait interrupted: {:?}",
                c.state.error
            );
            ensure!(
                start.elapsed() < Duration::from_secs(12),
                "delayed S11/HP not received"
            );
            if c.state.world.resurrection_packets == 0 {
                ensure!(
                    c.state.world.own_actor().is_some_and(|a| a.dead),
                    "predicted revival"
                );
            }
            thread::sleep(Duration::from_millis(5));
        }
        ensure!(
            start.elapsed() >= Duration::from_secs(8),
            "delay was not exercised"
        );
        ensure!(
            c.transport.snapshot().epoch == epoch && c.deadline.is_none(),
            "new connection or stale deadline"
        );
        ensure!(
            c.state.world.resurrection_packets == 1 && c.state.world.death_confirmations_sent == 1,
            "lifecycle count"
        );
        ensure!(
            !c.state.world.own_actor().unwrap().dead && !frame.death_ready(&c.state.world),
            "old corpse frame survived"
        );
        Ok(())
    })();
    c.cancel();
    let cleanup_end = Instant::now() + Duration::from_secs(3);
    while !c.is_quiescent() && Instant::now() < cleanup_end {
        c.poll();
        thread::sleep(Duration::from_millis(5));
    }
    let server_result = server.join().expect("owned synthetic peer panicked");
    result?;
    server_result?;
    ensure!(c.is_quiescent(), "owned worker not retired");
    Ok(())
}
