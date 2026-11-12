//! G06.3d teleport terminal branches without S_PORTAL, and server-authoritative
//! destinations. Offline controller replays only: no transport, server or game.
//!
//! Server contract (lito-launcher-server, read-only):
//! - `Protocol.teleportSpellRequest`: the native client locks its action/cursor
//!   before sending skill 5, so every terminal branch releases it with
//!   S_PARALYSE mode 7 (`Protocol.paralysis(TELEPORT_UNLOCK)` = body `25 07`,
//!   4 bytes framed).
//! - `native-thunder-teleport-policy-progress.zh-TW.md:5`: with Thunder Grab
//!   active, skills 5/131 keep the MP/reuse charge, send the Source error, the
//!   unlock and MP, and build **no** portal.
//! - `source-mass-teleport-exit.zh-TW.md:15`: a failed skill-69 body sends the
//!   4-byte unlock exactly once.
use super::*;
use crate::modern_lito::{native_navigation, native_portal::{self, Destination}, native_skills};

const UNLOCK: [u8; 2] = [0x25, 0x07];

fn controller() -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = LoginPhase::WorldReady;
    c.state.game_connected = true;
    c.state.world = super::super::native_world_scene::movement_tests::world();
    for actor in c.state.world.actors.values_mut() {
        actor.emblem = 0;
    }
    c.state.world.hp = Some([100, 100]);
    c.state.world.mp = Some([50, 50]);
    c.state.world.learned_skills.extend([5, 131]);
    c
}

fn pending_skill(c: &mut LoginController, now: Instant) {
    c.skill_pending = Some(native_skills::Pending {
        since: now,
        casts: c.state.world.own_cast_packets,
        messages: c.state.world.message_count,
        generation: c.state.world.generation,
        incarnation: c.state.world.own_actor().unwrap().incarnation,
    });
    c.state.skill_feedback = native_skills::Feedback::Pending;
}

fn portal(d: Destination) -> Vec<u8> {
    let mut body = vec![0x5a];
    for value in [d.map, d.x, d.y] {
        body.extend(value.to_le_bytes());
    }
    body
}

fn scene(c: &LoginController) -> (u64, Option<u16>, Option<(u16, u16)>) {
    let w = &c.state.world;
    (w.generation, w.map_id, w.own_actor().map(|a| (a.x, a.y)))
}

#[test]
fn skill5_unlock_without_portal_releases_the_wait_and_builds_no_portal() {
    let mut c = controller();
    let now = Instant::now();
    let map = c.state.world.map_id.unwrap();
    let stamp = native_navigation::InputStamp::capture(0, &c.state.world);
    c.teleport_spell = Some((stamp, now, map));
    pending_skill(&mut c, now);
    let before = scene(&c);
    // Thunder Grab policy: Source error text, then unlock (MP update omitted).
    let mut text = vec![0x0f, 9];
    text.extend(b"thunder grab\0");
    c.accept_game(&text).unwrap();
    assert!(c.teleport_spell.is_some(), "an uncoded text is not the terminal reply");
    c.accept_game(&UNLOCK).unwrap();
    assert!(c.teleport_spell.is_none() && c.world_tree_request.is_none());
    assert!(c.state.world.portal.is_none());
    assert_eq!(scene(&c), before, "no local move, map change or scene reset");
    assert_eq!(c.teleport_unlocks_without_portal, 1);
    c.observe_skill(now);
    assert_eq!(c.state.skill_feedback, native_skills::Feedback::TeleportReleased);
    assert_eq!(c.state.skill_feedback.label(), "傳送未執行，已解除鎖定");
    assert!(c.skill_pending.is_none() && !c.teleport_released);
    // A late S_PORTAL is not this request's reply: WORLD rejects it unadmitted.
    let own = c.state.world.own_actor().unwrap();
    let late = portal(Destination { map, x: own.x + 1, y: own.y });
    assert!(c.accept_game(&late).is_err());
    assert!(c.state.world.portal.is_none());
    assert_eq!(scene(&c), before);
}

#[test]
fn world_tree_unlock_without_portal_releases_the_request() {
    let mut c = controller();
    let now = Instant::now();
    let stamp = native_navigation::InputStamp::capture(0, &c.state.world);
    c.world_tree_request = Some((stamp, now));
    let before = scene(&c);
    c.accept_game(&UNLOCK).unwrap();
    assert!(c.world_tree_request.is_none() && c.teleport_spell.is_none());
    assert!(!c.teleport_released, "no skill wait to resolve");
    assert_eq!(c.teleport_unlocks_without_portal, 1);
    assert_eq!(c.state.phase, LoginPhase::WorldReady);
    assert!(c.state.world.portal.is_none());
    assert_eq!(scene(&c), before);
    // The exact world-tree destination is no longer admitted after release.
    let tree = portal(Destination { map: 4, x: 33051, y: 32337 });
    assert!(c.accept_game(&tree).is_err());
    assert_eq!(c.state.phase, LoginPhase::WorldReady);
    assert!(c.state.world.portal.is_none());
}

#[test]
fn coded_rejection_after_unlock_in_the_same_batch_keeps_its_reason() {
    let mut c = controller();
    let now = Instant::now();
    let stamp = native_navigation::InputStamp::capture(0, &c.state.world);
    c.teleport_spell = Some((stamp, now, c.state.world.map_id.unwrap()));
    pending_skill(&mut c, now);
    // Server rejections send mode 7 first, then S_MESSAGE_CODE.
    c.accept_game(&UNLOCK).unwrap();
    c.accept_game(&[0x10, 0x14, 1, 0]).unwrap();
    c.observe_skill(now);
    assert_eq!(c.state.skill_feedback, native_skills::Feedback::Rejected(276));
    assert!(c.teleport_spell.is_none() && c.skill_pending.is_none());
}

#[test]
fn unsolicited_unlock_and_server_lock_pair_do_not_release_unrelated_work() {
    let mut c = controller();
    let now = Instant::now();
    pending_skill(&mut c, now);
    let before = scene(&c);
    c.accept_game(&UNLOCK).unwrap();
    assert_eq!(c.teleport_unlocks_without_portal, 0);
    assert!(!c.teleport_released && c.skill_pending.is_some());
    c.observe_skill(now);
    assert_eq!(c.state.skill_feedback, native_skills::Feedback::Pending);
    // Server TELEPORT_LOCK (mode 6) blocks; mode 7 lifts only that block.
    c.skill_pending = None;
    c.accept_game(&[0x25, 0x06]).unwrap();
    assert!(c.state.world.movement_blocks.contains(&6));
    c.accept_game(&UNLOCK).unwrap();
    assert!(c.state.world.movement_blocks.is_empty());
    assert_eq!(c.teleport_unlocks_without_portal, 0);
    assert_eq!(scene(&c), before);
}

#[test]
fn malformed_unlock_fails_without_releasing_the_request() {
    let mut c = controller();
    let now = Instant::now();
    let stamp = native_navigation::InputStamp::capture(0, &c.state.world);
    let map = c.state.world.map_id.unwrap();
    c.teleport_spell = Some((stamp, now, map));
    for bad in [&[0x25, 0x07, 0x00][..], &[0x25][..]] {
        assert!(c.accept_game(bad).is_err());
        assert!(c.teleport_spell.is_some());
        assert_eq!(c.teleport_unlocks_without_portal, 0);
    }
}

#[test]
fn skill5_admits_server_coordinates_not_the_requested_ones() {
    let c = controller();
    let map = c.state.world.map_id.unwrap();
    // Random travel requests x=y=0; a bookmark requests its own cell. Either
    // way the server's S_PORTAL cell is the destination, on the requested map.
    for server in [Destination { map, x: 32_700, y: 32_800 }, Destination { map, x: 1, y: 1 }] {
        native_portal::admit_teleport_spell(&c.state, server, map).unwrap();
    }
    assert!(native_portal::admit_teleport_spell(&c.state, Destination { map: map.wrapping_add(1), x: 1, y: 1 }, map).is_err());
    let mut unlearned = controller();
    unlearned.state.world.learned_skills.remove(&5);
    assert!(native_portal::admit_teleport_spell(&unlearned.state, Destination { map, x: 1, y: 1 }, map).is_err());
}

#[test]
fn accepted_portal_destination_is_the_server_cell_and_rebuild_must_match_it() {
    let mut w = super::super::native_world_scene::movement_tests::world();
    let own = w.own_actor().unwrap().clone();
    let map = w.map_id.unwrap();
    let server = Destination { map, x: own.x + 3, y: own.y + 2 };
    w.accept(&portal(server)).unwrap();
    let pending = w.portal.unwrap();
    assert_eq!((pending.destination, pending.object_id), (server, own.id));
    assert!(!w.ready(), "input waits for the server rebuild");
    assert!(!w.finish_portal_if_ready().unwrap(), "no rebuilt actor yet");
}
