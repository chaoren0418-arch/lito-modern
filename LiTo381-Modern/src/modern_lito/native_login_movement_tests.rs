//! Controller checks plus an owned loopback peer; no game server or credentials.
use super::*;
use crate::modern_lito::native_world_scene::movement_tests::{frame, world};

fn active() -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = LoginPhase::BattlePending;
    c.state.world = world();
    c
}

fn held(c: &LoginController) -> super::super::native_navigation::HeldInput {
    use super::super::native_navigation::{HeldInput, InputStamp};
    let mut held = HeldInput::default();
    held.synchronize_action(&c.state.world);
    held.synchronize(Some(InputStamp::capture(c.transport.snapshot().epoch, &c.state.world)));
    held.press(0x68);
    held
}

#[test]
fn collision_tests_held_step_rechecks_late_boundary_then_never_auto_resumes() {
    let mut c = active(); let mut held = held(&c);
    let arrived = frame(&c.state.world,true);
    let moving = frame(&c.state.world,false);
    let a = c.state.world.own_actor().unwrap();
    let mut attr = vec![0x22]; attr.extend(a.x.to_le_bytes()); attr.extend(a.y.to_le_bytes());
    attr.extend([0,65]);
    c.state.world.accept(&attr).unwrap();
    assert!(!c.continue_movement(&mut held,Some(&moving)).unwrap());
    assert_eq!(held.direction(),Some(0));
    let before = serde_json::to_value(&c.state).unwrap();
    let error = c.continue_movement(&mut held,Some(&arrived)).unwrap_err();
    assert!(error.to_string().contains("map edge blocks movement"));
    assert_eq!(serde_json::to_value(&c.state).unwrap(),before);
    assert_eq!(held.direction(),None);
    *attr.last_mut().unwrap() = 0;
    c.state.world.accept(&attr).unwrap();
    assert!(!c.continue_movement(&mut held,Some(&arrived)).unwrap());
    assert_eq!(c.state.world.predicted_moves,0);
}

#[test]
fn collision_tests_new_monster_blocks_even_when_arrived_frame_predates_it() {
    let mut c = active(); let f = frame(&c.state.world,true); let mut held = held(&c);
    c.state.world.encounter_mode = Some(super::super::native_obstacles::EncounterMode::TimeAttack);
    let mut a = c.state.world.own_actor().unwrap().clone();
    a.id = 0x6d000001; a.y -= 1; a.graphics = 61; a.walk_action = 0;
    c.state.world.actors.insert(a.id,a);
    let error = c.continue_movement(&mut held,Some(&f)).unwrap_err();
    assert!(error.to_string().contains("動態物件"));
    assert_eq!(held.direction(),None);
    assert_eq!(c.state.world.predicted_moves,0);
}

#[test]
fn collision_tests_zero_hp_and_inactive_world_cancel_without_a_frame() {
    let mut c = active(); let mut held = held(&c);
    c.state.world.accept(&[0x0d,0,0,200,0]).unwrap();
    assert!(!c.continue_movement(&mut held,None).unwrap());
    c.state.world.accept(&[0x0d,100,0,200,0]).unwrap();
    let f = frame(&c.state.world,true);
    assert!(!c.continue_movement(&mut held,Some(&f)).unwrap());
    held.press(0x68); c.state.phase = LoginPhase::ResultReturning;
    assert!(!c.continue_movement(&mut held,None).unwrap());
    assert_eq!(held.direction(),None);
}

#[test]
fn collision_tests_authoritative_hurt_stops_held_walk_without_rolling_back_prediction() {
    let mut c = active(); let mut held = held(&c);
    c.state.world.predict_step(0).unwrap();
    let own = c.state.world.own_actor().unwrap().clone();
    let mut hurt = vec![0x20]; hurt.extend(own.id.to_le_bytes()); hurt.push(2);
    c.state.world.accept(&hurt).unwrap();
    assert!(!c.continue_movement(&mut held,None).unwrap());
    assert_eq!(held.direction(),None);
    let current = c.state.world.own_actor().unwrap();
    assert_eq!((current.x,current.y),(own.x,own.y));
    assert!(current.position_predicted);
    assert!(current.motion_from.is_none(),"hurt interrupts interpolation");
    let f = frame(&c.state.world,true);
    assert!(!c.continue_movement(&mut held,Some(&f)).unwrap());
}

/// Deferred arrow hurt (S_ATTACK flight 0x7f, graphics 66, damage>0) never
/// advances the target's action sequence; only the presented arrival does.
#[test]
fn collision_tests_delayed_arrow_hurt_on_local_body_retires_held_walk_but_bystander_hit_does_not() {
    use super::super::{
        native_navigation::{HeldInput, ImpactStamp},
        native_projectiles::Projectiles,
        native_world_animation::Animator,
        native_world_scene::{dispatch_impact, Frame},
        sprite_actions::{ActionLimits, SpriteActionCatalog},
    };
    let catalog = SpriteActionCatalog::parse(
        b"#61 16 body 0.walk(1 2,0.0:4 0.1:4) 3.breath(1 2,8.0:4 8.1:4) 2.damage(1 2,16.0:4 16.1:4)\n\
          #62 16 archer 3.breath(1 2,8.0:4 8.1:4) 1.attack(1 2,0.0:4! 0.1:4)\n",
        ActionLimits::default(),
    ).unwrap();
    let mut c = active();
    let own_id = c.state.world.own_actor().unwrap().id;
    let own = c.state.world.actors.get_mut(&own_id).unwrap();
    own.graphics = 61; own.walk_action = 0; own.motion_from = None;
    let own = own.clone();
    let (archer, bystander) = (0x6d00_0101u32, 0x6d00_0102u32);
    let mut a = own.clone();
    a.id = archer; a.graphics = 62; a.x = own.x + 3; a.incarnation += 1000;
    c.state.world.actors.insert(archer, a.clone());
    let mut b = own.clone();
    b.id = bystander; b.y = own.y + 3; b.incarnation += 2000;
    c.state.world.actors.insert(bystander, b.clone());
    let arrow = |target: u32, id: u32, to: [u16; 2]| {
        let mut p = vec![0x23, 1];
        p.extend(archer.to_le_bytes()); p.extend(target.to_le_bytes());
        p.extend([5, 6]); p.extend(id.to_le_bytes()); p.extend(66u16.to_le_bytes()); p.push(0x7f);
        for v in [a.x, a.y, to[0], to[1]] { p.extend(v.to_le_bytes()); }
        p.extend([0, 0]); p
    };
    let (mut animator, mut projectiles, mut stamp) =
        (Animator::default(), Projectiles::default(), ImpactStamp::default());
    // Same order as Model::draw: poses -> projectile arrivals -> dispatch -> re-pose.
    let mut present = |w: &super::super::native_world::World, now: u64, accepted: &mut Vec<u32>| -> Frame {
        animator.begin(now, w.actors.keys().copied()).unwrap();
        let mut poses = std::collections::BTreeMap::new();
        for actor in w.actors.values() { poses.insert(actor.id, animator.pose(&catalog, actor, now).unwrap()); }
        let impacts = projectiles.update(w, poses.values().flat_map(|p| &p.entered_events), now);
        for impact in impacts {
            let actor = &w.actors[&impact.target];
            if dispatch_impact(&mut animator, &mut stamp, actor, impact.sequence, Some(own_id)) {
                accepted.push(actor.id);
                poses.insert(actor.id, animator.pose(&catalog, actor, now).unwrap());
            }
        }
        let mut f = frame(w, poses[&own_id].phase == "idle");
        f.local_impacts = stamp;
        f
    };
    let mut accepted = Vec::new();
    let mut held = HeldInput::default();
    let idle = present(&c.state.world, 0, &mut accepted);
    assert!(!c.continue_movement(&mut held, Some(&idle)).unwrap(), "baseline only");
    held.press(0x68);

    // Arrow at a bystander: its hurt is accepted, the local press survives
    // and reaches the C_MOVE send owner.
    c.state.world.accept(&arrow(bystander, 0x7001, [b.x, b.y])).unwrap();
    assert_eq!(c.state.world.actors[&bystander].action_sequence, b.action_sequence, "hurt deferred");
    present(&c.state.world, 1_000_000, &mut accepted);
    let f = present(&c.state.world, 5_000_000_000, &mut accepted);
    assert_eq!(accepted, [bystander]);
    assert_eq!(held.direction(), Some(0));
    let error = c.continue_movement(&mut held, Some(&f)).unwrap_err();
    assert!(error.to_string().contains("native send owner"), "{error:#}");

    // Arrow at the local body while the key is held.
    held.press(0x68);
    let before = c.state.world.own_actor().unwrap().action_sequence;
    c.state.world.accept(&arrow(own_id, 0x7002, [own.x, own.y])).unwrap();
    assert_eq!(c.state.world.own_actor().unwrap().action_sequence, before, "no authoritative hurt");
    present(&c.state.world, 10_000_000_000, &mut accepted);
    let hurt = present(&c.state.world, 15_000_000_000, &mut accepted);
    assert_eq!(accepted, [bystander, own_id]);
    assert!(!hurt.movement_ready(&c.state.world), "hurt pose owns the body");
    assert!(!c.continue_movement(&mut held, Some(&hurt)).unwrap());
    assert_eq!(held.direction(), None, "accepted local arrival retires the press");
    let healed = present(&c.state.world, 40_000_000_000, &mut accepted);
    assert!(healed.movement_ready(&c.state.world), "hurt animation completed");
    assert!(!c.continue_movement(&mut held, Some(&healed)).unwrap());
    assert_eq!(held.direction(), None, "old press never resumes after hurt");
    assert_eq!(c.state.world.predicted_moves, 0);
    // A fresh press after the hurt is admitted to the send owner again.
    held.press(0x68);
    let error = c.continue_movement(&mut held, Some(&healed)).unwrap_err();
    assert!(error.to_string().contains("native send owner"), "{error:#}");
}

#[test]
fn collision_tests_observer_round_trip_and_short_lock_retire_the_press() {
    let mut c = active(); let mut held = held(&c);
    c.state.world.set_observer_mode(true);
    c.state.world.set_observer_mode(false);
    let f = frame(&c.state.world,true);
    assert!(!c.continue_movement(&mut held,Some(&f)).unwrap());
    held.press(0x68);
    c.state.world.accept(&[0x25,24]).unwrap();
    c.state.world.accept(&[0x25,25]).unwrap();
    let f = frame(&c.state.world,true);
    assert!(!c.continue_movement(&mut held,Some(&f)).unwrap());
    assert_eq!(held.direction(),None);
}

#[test]
fn collision_tests_release_and_no_frame_wait_without_replaying_after_send_error() {
    let mut c = active(); let mut held = held(&c);
    held.release(0x68);
    assert!(!c.continue_movement(&mut held,None).unwrap());
    assert_eq!(held.direction(),Some(0),"quick tap retained until arrived");
    let f = frame(&c.state.world,true);
    let error = c.continue_movement(&mut held,Some(&f)).unwrap_err();
    assert!(error.to_string().contains("native send owner"));
    assert_eq!(held.direction(),None);
    assert!(!c.continue_movement(&mut held,Some(&f)).unwrap());
}

#[test]
fn collision_tests_controller_qualifies_encounter_from_room_and_retires_it_in_world() {
    use super::super::{native_room, native_obstacles::EncounterMode};
    let f: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-room-current.json")).unwrap();
    let hex = f["packets"]["joined"].as_str().unwrap();
    let bytes: Vec<_> = (0..hex.len()).step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
    let native_room::Packet::Joined(room) = native_room::parse(&bytes).unwrap() else { panic!() };
    let mut c = active(); c.state.joined_room = Some(room);
    for mode in 0..=7 {
        c.state.joined_room.as_mut().unwrap().mode = mode;
        c.accept_game(&[0x0d,100,0,200,0]).unwrap();
        assert_eq!(c.state.world.encounter_mode,EncounterMode::from_room_mode(mode));
    }
    c.state.joined_room.as_mut().unwrap().mode = 2;
    c.accept_game(&[0x0d,100,0,200,0]).unwrap();
    assert_eq!(c.state.world.encounter_mode,Some(EncounterMode::Quest));
    c.state.phase = LoginPhase::WorldReady;
    c.accept_game(&[0x0d,100,0,200,0]).unwrap();
    assert_eq!(c.state.world.encounter_mode,None);
    c.state.phase = LoginPhase::BattlePending; c.state.joined_room = None;
    c.accept_game(&[0x0d,100,0,200,0]).unwrap();
    assert_eq!(c.state.world.encounter_mode,None);
}

#[test]
fn movement_tests_controller_rejects_in_flight_before_transport_or_prediction() {
    let mut c = active();
    let stale = frame(&c.state.world, true);
    c.state.world.predict_step(0).unwrap();
    let moving = frame(&c.state.world, false);
    let before = serde_json::to_value(&c.state).unwrap();
    for f in [&stale, &moving] {
        let error = c.move_presented(0, f).unwrap_err();
        assert!(error.to_string().contains("等待目前步伐"));
        assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    }
}

#[test]
fn movement_tests_valid_preflight_but_failed_queue_cannot_predict() {
    let mut c = active();
    let f = frame(&c.state.world, true);
    let before = serde_json::to_value(&c.state).unwrap();
    let error = c.move_presented(0, &f).unwrap_err();
    assert!(error
        .to_string()
        .contains("native send owner is not current/ready"));
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert_eq!(c.state.world.predicted_moves, 0);
}

#[test]
fn movement_tests_controller_malformed_blink_never_creates_ack_or_lock() {
    let mut c = active();
    let before = serde_json::to_value(&c.state).unwrap();
    assert!(c.accept_game(&[0x63, 0]).is_err());
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert!(c
        .accept_game(&[0x63])
        .unwrap_err()
        .to_string()
        .contains("native send owner"));
    assert_eq!(
        c.state.world.movement_lock,
        super::super::native_world::MovementLock::AckPending
    );
    c.accept_game(&[0x25, 17]).unwrap();
    assert!(
        c.state.world.move_request(0).is_err(),
        "failed C_BLINK cannot qualify release"
    );
}

#[test]
fn movement_tests_encrypted_single_owner_correction_ack_release_and_next_step() {
    use crate::modern_lito::{
        native_version,
        native_wire::{self, CipherState},
        native_world::MovementLock,
    };
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
    };

    fn read(socket: &mut TcpStream, cipher: &mut CipherState) -> Vec<u8> {
        let mut header = [0; 2];
        socket.read_exact(&mut header).unwrap();
        let length = usize::from(u16::from_le_bytes(header));
        assert!((3..=native_wire::MAX_FRAME).contains(&length));
        let mut body = vec![0; length - 2];
        socket.read_exact(&mut body).unwrap();
        cipher.decrypt(&mut body);
        body
    }
    fn send(socket: &mut TcpStream, cipher: &mut CipherState, mut body: Vec<u8>) {
        cipher.encrypt(&mut body);
        socket
            .write_all(&native_wire::frame(&body).unwrap())
            .unwrap();
    }
    fn pump(c: &mut LoginController, ready: impl Fn(&LoginController) -> bool) {
        let until = Instant::now() + Duration::from_secs(10);
        while !ready(c) {
            for event in c.transport.poll() {
                if let Event::Packet { body, .. } = event {
                    c.accept_game(&body).unwrap();
                }
            }
            assert!(
                Instant::now() < until,
                "loopback timeout: {:?}",
                c.transport.snapshot()
            );
            thread::sleep(Duration::from_millis(2));
        }
    }

    let mut c = active();
    let request = c.state.world.move_request(0).unwrap();
    let own = c.state.world.own_actor().unwrap();
    let mut correction = vec![0x12];
    correction.extend(own.id.to_le_bytes());
    correction.extend(own.x.to_le_bytes());
    correction.extend((own.y - 1).to_le_bytes());
    correction.push(4);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let (release, release_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(10);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < until, "owned listener accept timeout");
                    thread::sleep(Duration::from_millis(2));
                }
                Err(e) => panic!("owned listener: {e}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let seed = 0x12345678u32;
        let mut key = vec![1];
        key.extend(seed.to_le_bytes());
        socket
            .write_all(&native_wire::frame(&key).unwrap())
            .unwrap();
        let mut rx = CipherState::new(seed);
        let mut tx = CipherState::new(seed);
        assert_eq!(
            read(&mut socket, &mut rx),
            native_version::version_request(0)
        );
        let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-version-current.json"
        ))
        .unwrap();
        let hex = fixture["accepted"].as_str().unwrap();
        let accepted = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        send(&mut socket, &mut tx, accepted);
        assert_eq!(read(&mut socket, &mut rx), request);
        // Deliberate silent bind refusal followed by authoritative recovery;
        // these packets exercise the client, not a simulated server rules claim.
        for body in [
            vec![0x25, 24],
            vec![0x25, 25],
            correction,
            vec![0x63],
            vec![0x63],
        ] {
            send(&mut socket, &mut tx, body);
        }
        assert_eq!(read(&mut socket, &mut rx), [0x5a]);
        release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        send(&mut socket, &mut tx, vec![0x25, 17]);
        send(&mut socket, &mut tx, vec![0x25, 17]);
        // A duplicate ACK or extra in-flight move appears here and fails exact matching.
        assert_eq!(read(&mut socket, &mut rx), request);
    });
    c.transport
        .start(
            peer,
            Limits {
                read_chunk: 2,
                write_chunk: 3,
                ..Limits::default()
            },
        )
        .unwrap();
    pump(&mut c, |c| {
        c.transport.snapshot().phase == Phase::VersionAccepted
    });
    let initial_counter = c.transport.queued_tx_counter().unwrap();
    let old = frame(&c.state.world, true);
    let mut held = held(&c);
    assert!(c.continue_movement(&mut held, Some(&old)).unwrap());
    assert!(!c.continue_movement(&mut held, Some(&old)).unwrap());
    assert!(c.move_presented(0, &old).is_err());
    pump(&mut c, |c| c.state.world.movement_locks == 2);
    assert_eq!(c.state.world.movement_lock, MovementLock::AwaitRelease);
    let locked = frame(&c.state.world, true);
    assert!(!c.continue_movement(&mut held, Some(&locked)).unwrap());
    assert_eq!(held.direction(), None);
    assert!(c.move_presented(0, &locked).is_err());
    assert_eq!(
        c.transport.queued_tx_counter(),
        Some(initial_counter.wrapping_add(7))
    );
    assert_eq!(
        (
            c.state.world.predicted_moves,
            c.state.world.local_corrections
        ),
        (1, 1)
    );
    release.send(()).unwrap();
    pump(&mut c, |c| {
        c.state.world.movement_lock == MovementLock::Open
    });
    assert!(c.move_presented(0, &locked).is_err());
    let fresh = frame(&c.state.world, true);
    assert!(!c.continue_movement(&mut held, Some(&fresh)).unwrap());
    held.press(0x68);
    assert!(c.continue_movement(&mut held, Some(&fresh)).unwrap());
    server.join().unwrap();
    assert_eq!(c.state.world.predicted_moves, 2);
    assert_eq!(
        c.transport.queued_tx_counter(),
        Some(initial_counter.wrapping_add(13))
    );
    c.transport.cancel();
    pump(&mut c, |c| c.transport.is_quiescent());
}
