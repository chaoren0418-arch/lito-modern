//! G04.4 / G04.7 lifecycle replays (T044-3). Deployed packet envelopes and the
//! server producer order are replayed offline; no socket, game process or DB.
//! Producer references (read-only, lito-launcher-server):
//! - Quest/Time Attack death: ClientSession `S_ACTION ... action=DIE` (no removal);
//!   taming `S_REMOVE_OBJECT ... reason=tamed`; Return-to-Nature re-puts the
//!   released monster with `Protocol.questMonster`/`timeAttackActor`.
//! - Siege War (room mode 3) activates `room.siegeGate`, never `room.door`;
//!   it opens with `S_ACTION` 28 + `S_CHANGE_ATTR` after two outer towers fall,
//!   and `openBattleDoor` ignores every C_OPEN that is not the arena door.
//! - Classic death restart removes the local actor before the next S_MAP_ID.
use super::*;
use crate::modern_lito::{
    native_combat::{self, Feedback},
    native_doors,
    native_navigation::{HeldInput, InputStamp},
    native_obstacles::EncounterMode,
    native_skill_selection::Target,
    native_world::World,
    native_world_scene::movement_tests::{frame, world},
};

fn fixture(name: &str) -> Vec<u8> {
    let f: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-world-current.json"
    ))
    .unwrap();
    let hex = f["packets"][name].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
/// Common S_PUT_OBJECT body: `door` is gfx 2083 action 29, `bot` gfx 61 WALK.
fn put(id: u32, x: u16, y: u16, door: bool) -> Vec<u8> {
    let mut b = fixture(if door { "door" } else { "bot" });
    b[1..3].copy_from_slice(&x.to_le_bytes());
    b[3..5].copy_from_slice(&y.to_le_bytes());
    b[5..9].copy_from_slice(&id.to_le_bytes());
    b
}
fn action(id: u32, action: u8) -> Vec<u8> {
    let mut b = vec![0x20];
    b.extend(id.to_le_bytes());
    b.push(action);
    b
}
fn remove(id: u32) -> Vec<u8> {
    let mut b = vec![0x15];
    b.extend(id.to_le_bytes());
    b
}
fn attribute(x: u16, y: u16, closed: bool) -> Vec<u8> {
    let mut b = vec![0x22];
    b.extend(x.to_le_bytes());
    b.extend(y.to_le_bytes());
    b.extend([0, if closed { 65 } else { 0 }]);
    b
}
fn north(w: &World) -> (u16, u16) {
    let a = w.own_actor().unwrap();
    (a.x, a.y - 1)
}
fn blocked(w: &World) -> bool {
    w.checked_move_request(0, &frame(w, true).navigation).is_err()
}
fn active() -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = LoginPhase::BattlePending;
    c.state.world = world();
    for actor in c.state.world.actors.values_mut() {
        actor.emblem = 0;
    }
    c
}

#[test]
fn lifecycle_tests_encounter_death_release_removal_and_map_change_track_each_life() {
    for (mode, id) in [(2u8, 0x6c00_0042u32), (4, 0x6d00_0042)] {
        let mut w = world();
        w.encounter_mode = EncounterMode::from_room_mode(mode);
        let (x, y) = north(&w);
        assert!(!blocked(&w));
        // Wave spawn / summon / final boss: S_PUT_OBJECT with WALK=0.
        w.accept(&put(id, x, y, false)).unwrap();
        let first = w.actors[&id].incarnation;
        let selected = Target::capture(&w, id).unwrap();
        assert!(blocked(&w), "mode={mode}: live encounter actor occupies its cell");
        assert!(w.attack_request(id).is_ok());
        // Death is S_ACTION 8 without removal; the body stops occupying.
        w.accept(&action(id, 8)).unwrap();
        assert!(w.actors[&id].dead);
        assert!(!blocked(&w), "mode={mode}: corpse does not block");
        assert!(w.attack_request(id).is_err());
        assert!(selected.resolve(&w).is_none());
        // Taming removes; Return-to-Nature re-puts the same object ID alive.
        w.accept(&remove(id)).unwrap();
        assert!(!w.actors.contains_key(&id) && !blocked(&w));
        w.accept(&put(id, x, y, false)).unwrap();
        let released = &w.actors[&id];
        assert!(released.incarnation > first && !released.dead);
        assert!(blocked(&w), "mode={mode}: re-entered life blocks again");
        assert!(selected.resolve(&w).is_none(), "old-life selection stays retired");
        assert!(Target::capture(&w, id).unwrap().resolve(&w).is_some());
        // Leaving the world (S_MAP_ID) discards actors and room qualification.
        let generation = w.generation;
        w.accept(&fixture("world")).unwrap();
        assert!(w.actors.is_empty() && w.encounter_mode.is_none());
        assert!(w.generation > generation && w.door_attributes.is_empty());
        assert_eq!(w.predicted_moves, 0);
    }
}

#[test]
fn lifecycle_tests_corpse_reput_is_a_new_object_until_its_death_action() {
    // Classic bootstrap sends a corpse as S_PUT_OBJECT followed by S_ACTION 8.
    let mut w = world();
    let (x, y) = north(&w);
    let id = 0x0123_4567;
    w.accept(&put(id, x, y, false)).unwrap();
    w.accept(&action(id, 8)).unwrap();
    let dead = w.actors[&id].incarnation;
    w.accept(&put(id, x, y, false)).unwrap();
    assert!(!w.actors[&id].dead && w.actors[&id].incarnation > dead);
    w.accept(&action(id, 8)).unwrap();
    assert!(w.actors[&id].dead);
    assert!(w.attack_request(id).is_err());
    assert!(!blocked(&w), "ordinary objects never occupy, alive or dead");
}

#[test]
fn lifecycle_tests_siege_inner_gate_opens_by_authority_and_survives_reentry() {
    let mut w = world();
    w.encounter_mode = EncounterMode::from_room_mode(3);
    assert_eq!(w.encounter_mode, Some(EncounterMode::Siege));
    let (x, y) = north(&w);
    let gate = 0x6200_0007;
    // battle-bootstrap: closed gate (29) then its attribute.
    w.accept(&put(gate, x, y, true)).unwrap();
    w.accept(&attribute(x, y, true)).unwrap();
    assert!(native_doors::record(&w, gate).is_some());
    assert!(blocked(&w));
    let error = native_doors::Choice::capture(0, &w, gate).unwrap_err();
    assert!(error.to_string().contains("守護塔"), "{error:#}");
    // The same gfx in an arena room is the interactive door.
    let mut arena = w.clone();
    arena.encounter_mode = None;
    let choice = native_doors::Choice::capture(0, &arena, gate).unwrap();
    assert!(choice.request(0, &arena).is_ok());
    arena.encounter_mode = Some(EncounterMode::Siege);
    assert!(choice.request(0, &arena).is_err(), "stale arena choice cannot send in Siege");
    // Two outer towers destroyed: S_ACTION 28 + open attribute.
    w.accept(&action(gate, 28)).unwrap();
    w.accept(&attribute(x, y, false)).unwrap();
    assert!(!blocked(&w));
    // A late joiner's bootstrap re-puts the open gate (28) with the same ID.
    let mut reput = put(gate, x, y, true);
    reput[11] = 28;
    w.accept(&reput).unwrap();
    assert!(native_doors::record(&w, gate).is_none(), "old-life attribute retired");
    assert!(w.door_attributes.is_empty());
    assert!(!blocked(&w));
    w.accept(&attribute(x, y, false)).unwrap();
    assert!(native_doors::record(&w, gate).is_some());
    assert!(!blocked(&w));
    // Leaving view removes the gate and every attribute it owned.
    w.accept(&remove(gate)).unwrap();
    assert!(w.door_attributes.is_empty() && !blocked(&w));
}

#[test]
fn lifecycle_tests_controller_qualifies_siege_from_room_mode() {
    use super::super::native_room;
    let f: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-room-current.json"
    ))
    .unwrap();
    let hex = f["packets"]["joined"].as_str().unwrap();
    let bytes: Vec<_> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    let native_room::Packet::Joined(mut room) = native_room::parse(&bytes).unwrap() else {
        panic!()
    };
    room.mode = 3;
    let mut c = active();
    c.state.joined_room = Some(room);
    c.accept_game(&[0x0d, 100, 0, 200, 0]).unwrap();
    assert_eq!(c.state.world.encounter_mode, Some(EncounterMode::Siege));
    assert!(!native_doors::operable(&c.state.world));
    c.state.joined_room.as_mut().unwrap().mode = 1;
    c.accept_game(&[0x0d, 100, 0, 200, 0]).unwrap();
    assert!(native_doors::operable(&c.state.world));
}

#[test]
fn lifecycle_tests_attack_wait_retires_on_target_or_attacker_replacement() {
    let mut w = world();
    let (x, y) = north(&w);
    let id = 0x6c00_0050;
    w.accept(&put(id, x, y, false)).unwrap();
    let start = std::time::Instant::now();
    let pending = native_combat::Pending::capture(&w, id, start);
    assert_eq!(pending.observe(&w, start), Feedback::Pending);
    let mut replaced = w.clone();
    replaced.accept(&put(id, x, y, false)).unwrap();
    assert_eq!(pending.observe(&replaced, start), Feedback::Interrupted);
    let mut cycled = w.clone();
    cycled.accept(&remove(id)).unwrap();
    cycled.accept(&put(id, x, y, false)).unwrap();
    assert_eq!(pending.observe(&cycled, start), Feedback::Interrupted);
    let mut rebuilt = w.clone();
    rebuilt.accept(&fixture("actor")).unwrap();
    assert_eq!(pending.observe(&rebuilt, start), Feedback::Interrupted);
    // A counted reply is still the reply, even if a new life followed it.
    replaced.own_attack_packets += 1;
    assert_eq!(pending.observe(&replaced, start), Feedback::Acknowledged);
    // Unrelated peers joining and leaving do not cancel the wait.
    let mut peers = w.clone();
    peers.accept(&put(27, x + 1, y, false)).unwrap();
    peers.accept(&remove(27)).unwrap();
    assert_eq!(pending.observe(&peers, start), Feedback::Pending);
}

#[test]
fn lifecycle_tests_controller_poll_interrupts_attack_after_same_id_reentry() {
    let mut c = active();
    let (x, y) = north(&c.state.world);
    let (target, peer) = (0x6d00_0060, 0x0000_0099);
    c.accept_game(&put(target, x, y, false)).unwrap();
    c.accept_game(&put(peer, x + 1, y, false)).unwrap();
    c.attack_pending = Some(native_combat::Pending::capture(
        &c.state.world, target, Instant::now()));
    c.state.attack_feedback = Feedback::Pending;
    c.accept_game(&put(peer, x + 1, y, false)).unwrap();
    c.poll();
    assert!(c.attack_pending.is_some(), "another object's life is unrelated");
    c.accept_game(&remove(target)).unwrap();
    c.accept_game(&put(target, x, y, false)).unwrap();
    c.poll();
    assert!(c.attack_pending.is_none());
    assert_eq!(c.state.attack_feedback, Feedback::Interrupted);
    assert_eq!(c.state.phase, LoginPhase::BattlePending);
}

#[test]
fn lifecycle_tests_local_removal_and_map_rebuild_retire_press_and_old_frame() {
    let mut c = active();
    let mut held = HeldInput::default();
    held.synchronize_action(&c.state.world);
    held.synchronize(Some(InputStamp::capture(c.transport.snapshot().epoch, &c.state.world)));
    held.press(0x68);
    let old = frame(&c.state.world, true);
    let own = c.state.world.own_actor().unwrap().id;
    // Classic death restart: own S_REMOVE_OBJECT before the next S_MAP_ID.
    c.state.world.accept(&remove(own)).unwrap();
    assert!(!c.state.world.ready());
    assert!(!c.continue_movement(&mut held, Some(&old)).unwrap());
    assert_eq!(held.direction(), None);
    assert!(c.move_presented(0, &old).is_err());
    let generation = c.state.world.generation;
    for name in ["world", "start", "status", "actor"] {
        c.state.world.accept(&fixture(name)).unwrap();
    }
    assert!(c.state.world.ready() && c.state.world.generation > generation);
    let error = c.move_presented(0, &old).unwrap_err();
    assert!(error.to_string().contains("等待目前步伐"), "{error:#}");
    let fresh = frame(&c.state.world, true);
    assert!(!c.continue_movement(&mut held, Some(&fresh)).unwrap(), "old press stays retired");
    assert_eq!(held.direction(), None);
    held.press(0x68);
    let error = c.continue_movement(&mut held, Some(&fresh)).unwrap_err();
    assert!(error.to_string().contains("native send owner"), "{error:#}");
    assert_eq!(c.state.world.predicted_moves, 0);
}
