//! Pure packet/presentation tests: no sockets, game processes, or external assets.
use super::*;
use crate::modern_lito::{
    native_navigation::{HeldInput, InputStamp, Navigation},
    native_world::{destination, MovementLock},
    resources::SharedMapSectorKey,
};
use std::sync::Arc;

pub(crate) fn world() -> World {
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-world-current.json"
    ))
    .unwrap();
    let mut world = World::default();
    for key in ["world", "start", "status", "actor"] {
        let hex = fixture["packets"][key].as_str().unwrap();
        let bytes: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        world.accept(&bytes).unwrap();
    }
    world
}

fn navigation(map_id: u32, closed: bool) -> Arc<Navigation> {
    // One synthetic LTmap S32 sector: layer1, layer2 count, layer3, portals.
    let mut bytes = vec![0; 32768];
    bytes.extend(0u16.to_le_bytes());
    for _ in 0..4096 {
        bytes.extend([u8::from(closed), 0, u8::from(closed), 0]);
    }
    bytes.extend(0i32.to_le_bytes());
    let map = WorldMap::from_native_sector_bytes(
        vec![SectorInput {
            key: SharedMapSectorKey {
                map_id,
                sector_x: 0x7fff,
                sector_y: 0x8000,
            },
            bytes,
        }],
        &AtomicBool::new(false),
    )
    .unwrap();
    Arc::new(Navigation::from_geometry(&map).unwrap())
}

pub(crate) fn frame(world: &World, arrived: bool) -> Frame {
    let a = world.own_actor().unwrap();
    Frame {
        status4183_leases: Vec::new(),
        player_walk_plan: None,
        static2941: Default::default(),
        ground_items: vec![],
        hits: vec![],
        visibility: vec![],
        sounds: vec![],
        legacy_audio: vec![],
        native_audio: vec![],
        native_ambient: vec![],
        ambient_owner: Arc::new(()),
        finite_hurt_callbacks: vec![],
        navigation: navigation(u32::from(world.map_id.unwrap()), false),
        movement_key: (
            a.id,
            a.motion_revision,
            a.x,
            a.y,
            a.speed,
            a.brave,
            a.action_sequence,
        ),
        movement_arrived: arrived,
        movement_stamp: InputStamp::capture(0, world),
        death_arrived: false,
        movement_generation: world.generation,
        observer_key: None,
        caster_incarnation: a.incarnation,
        input_revision: world.visual_revision,
        environment_key: None,
        projection: super::super::world_map::ViewProjection {
            anchor_cell: WorldCell {
                x: i32::from(a.x),
                y: i32::from(a.y),
            },
            anchor_pixel: super::super::world_map::PixelPoint { x: 0, y: 0 },
        },
        canvas: Canvas::new(1, 1).unwrap(),
        bgra: vec![0; 4],
        receipt: serde_json::json!({}),
    }
}

fn acknowledge(w: &mut World) {
    assert!(w
        .acknowledge_movement_lock(|body| {
            assert_eq!(body, [0x5a]);
            Ok(())
        })
        .unwrap());
}

#[test]
fn blink_requires_queued_ack_then_exact_mode17_and_deduplicates_ack() {
    let mut w = world();
    assert!(!w
        .acknowledge_movement_lock(|_| panic!("unsolicited C_BLINK"))
        .unwrap());
    w.accept(&[0x63]).unwrap();
    assert_eq!(w.movement_lock, MovementLock::AckPending);
    assert!(w.move_request(0).is_err());
    w.accept(&[0x25, 17]).unwrap();
    assert_eq!(
        w.movement_lock,
        MovementLock::AckPending,
        "release before ack cannot open"
    );
    acknowledge(&mut w);
    assert_eq!(w.movement_lock, MovementLock::AwaitRelease);
    assert!(
        w.move_request(0).is_err(),
        "queue admission is not server release"
    );
    w.accept(&[0x63]).unwrap();
    assert!(!w
        .acknowledge_movement_lock(|_| panic!("duplicate C_BLINK"))
        .unwrap());
    assert_eq!(w.movement_lock, MovementLock::AwaitRelease);
    w.accept(&[0x25, 7]).unwrap();
    assert!(
        w.move_request(0).is_err(),
        "teleport release cannot clear bit 8"
    );
    w.accept(&[0x25, 17]).unwrap();
    assert_eq!(w.movement_lock, MovementLock::Open);
    assert!(w.move_request(0).is_ok());
    w.accept(&[0x25, 17]).unwrap();
    assert_eq!(w.movement_lock, MovementLock::Open);
    w.accept(&[0x63]).unwrap();
    assert!(
        w.move_request(0).is_err(),
        "a later lock requires its own handshake"
    );
    assert_eq!(w.predicted_moves, 0);
}

#[test]
fn failed_ack_queue_never_unlocks_and_can_be_retried_without_duplicate_send() {
    let mut w = world();
    w.accept(&[0x63]).unwrap();
    let before = serde_json::to_value(&w).unwrap();
    assert!(w
        .acknowledge_movement_lock(|_| anyhow::bail!("queue full"))
        .is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    w.accept(&[0x25, 17]).unwrap();
    assert_eq!(w.movement_lock, MovementLock::AckPending);
    acknowledge(&mut w);
    w.accept(&[0x25, 17]).unwrap();
    assert!(w.move_request(0).is_ok());
}

#[test]
fn malformed_lock_and_release_packets_are_atomic_in_every_handshake_phase() {
    for phase in [
        MovementLock::Open,
        MovementLock::AckPending,
        MovementLock::AwaitRelease,
    ] {
        let mut w = world();
        w.movement_lock = phase;
        for body in [
            vec![],
            vec![0x63, 0],
            vec![0x25],
            vec![0x25, 17, 0],
            vec![0x25, 16],
            vec![0x25, 255],
        ] {
            let before = serde_json::to_value(&w).unwrap();
            assert!(w.accept(&body).is_err(), "{body:?}");
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
        }
    }
}

#[test]
fn mode17_preserves_all_independent_restrictions_and_coordinates() {
    let mut w = world();
    for mode in [2, 4, 6, 10, 12, 22, 24] {
        w.accept(&[0x25, mode]).unwrap();
    }
    let restrictions = w.movement_blocks.clone();
    let own = w.own_actor().unwrap().clone();
    w.accept(&[0x63]).unwrap();
    acknowledge(&mut w);
    w.accept(&[0x25, 17]).unwrap();
    assert_eq!(w.movement_lock, MovementLock::Open);
    assert_eq!(w.movement_blocks, restrictions);
    assert_eq!(w.own_actor().unwrap(), &own);
    assert!(w.move_request(0).is_err());
    for mode in [3, 5, 7, 11, 13, 23, 25] {
        w.accept(&[0x25, mode]).unwrap();
    }
    assert!(w.move_request(0).is_ok());
}

#[test]
fn in_flight_and_stale_arrival_cannot_authorize_a_second_prediction() {
    let mut w = world();
    let idle = frame(&w, true);
    assert!(idle.movement_ready(&w));
    w.predict_step(0).unwrap();
    assert!(!idle.movement_ready(&w));
    assert!(!frame(&w, false).movement_ready(&w));
    assert!(frame(&w, true).movement_ready(&w));
    assert!(
        w.own_actor().unwrap().position_predicted,
        "arrival is not an ACK"
    );
    assert_eq!(w.predicted_moves, 1);
}

#[test]
fn quake_racing_an_in_flight_step_keeps_uncertainty_and_cancels_held_retry() {
    let mut w = world();
    let mut held = HeldInput::default();
    held.synchronize(Some(InputStamp::capture(1, &w)));
    held.press(0x68);
    w.predict_step(0).unwrap();
    held.sent();
    let before = w.own_actor().unwrap().clone();
    let old_arrival = frame(&w, true);
    // The server can silently reject that in-flight C_MOVE. There is no
    // coordinate/sequence in mode24, so neither acceptance nor rollback is known.
    w.accept(&[0x25, 24]).unwrap();
    assert!(!frame(&w, true).movement_ready(&w));
    assert!(w.move_request(0).is_err());
    w.accept(&[0x25, 25]).unwrap();
    assert_eq!(w.own_actor().unwrap(), &before);
    assert!(w.own_actor().unwrap().position_predicted);
    assert!(
        !old_arrival.movement_ready(&w),
        "batched apply/release invalidates old input"
    );
    held.synchronize(Some(InputStamp::capture(1, &w)));
    assert_eq!(
        held.direction(),
        None,
        "release must not auto-retry the rejected step"
    );
    assert_eq!(w.predicted_moves, 1);
}

#[test]
fn authority_after_silent_refusal_replaces_prediction_and_old_frames() {
    let mut w = world();
    let original = w.own_actor().unwrap().clone();
    w.predict_step(0).unwrap();
    let predicted = frame(&w, true);
    w.accept(&[0x25, 24]).unwrap();
    w.accept(&[0x25, 25]).unwrap();
    // A later explicit move can elicit a stale-source correction. S12 contains
    // a source cell, so heading south below targets the original position.
    let mut correction = vec![0x12];
    correction.extend(original.id.to_le_bytes());
    correction.extend(original.x.to_le_bytes());
    correction.extend((original.y - 1).to_le_bytes());
    correction.push(4);
    w.accept(&correction).unwrap();
    let a = w.own_actor().unwrap();
    assert_eq!((a.x, a.y), (original.x, original.y));
    assert!(!a.position_predicted);
    assert!(a.motion_from.is_none());
    assert!(!predicted.movement_ready(&w));
    w.accept(&[0x63]).unwrap();
    let locked = frame(&w, true);
    assert!(!locked.movement_ready(&w));
    acknowledge(&mut w);
    assert!(!locked.movement_ready(&w));
    w.accept(&[0x25, 17]).unwrap();
    assert!(
        !locked.movement_ready(&w),
        "locked presentation must be refreshed after release"
    );
    assert!(frame(&w, true).movement_ready(&w));
    assert_eq!((w.predicted_moves, w.local_corrections), (1, 1));
}

#[test]
fn local_collision_preflight_is_atomic_for_all_headings_and_wrong_maps() {
    let w = world();
    let open = navigation(564, false);
    let closed = navigation(564, true);
    let wrong_map = navigation(565, false);
    let before = serde_json::to_value(&w).unwrap();
    let a = w.own_actor().unwrap();
    for h in 0..8 {
        let body = w.checked_move_request(h, &open).unwrap();
        assert_eq!(&body[1..3], &a.x.to_le_bytes());
        assert_eq!(&body[3..5], &a.y.to_le_bytes());
        assert_eq!(body[5], h);
        assert!(destination(a.x, a.y, h).is_ok());
        assert!(w.checked_move_request(h, &closed).is_err());
        assert!(w.checked_move_request(h, &wrong_map).is_err());
    }
    assert!(w.checked_move_request(8, &open).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
}

#[test]
fn unloaded_terrain_and_restrictions_never_produce_a_move_packet() {
    let mut w = world();
    let nav = navigation(564, false);
    let id = w.own_actor().unwrap().id;
    w.actors.get_mut(&id).unwrap().x = 32704;
    assert!(w.checked_move_request(6, &nav).is_err());
    assert!(w.checked_move_request(7, &nav).is_err());
    for mode in [2, 4, 6, 10, 12, 22, 24] {
        w.accept(&[0x25, mode]).unwrap();
        assert!(w.checked_move_request(2, &nav).is_err());
        w.accept(&[0x25, mode + 1]).unwrap();
    }
    w.accept(&[0x63]).unwrap();
    assert!(w.checked_move_request(2, &nav).is_err());
    assert_eq!(w.predicted_moves, 0);
}

#[test]
fn scene_reset_retires_handshake_without_treating_unsolicited_release_as_authority() {
    let mut w = world();
    w.predict_step(0).unwrap();
    w.accept(&[0x63]).unwrap();
    acknowledge(&mut w);
    w.clear_preserving_generation();
    assert_eq!(w.movement_lock, MovementLock::Open);
    assert!(!w
        .acknowledge_movement_lock(|_| panic!("retired ack"))
        .unwrap());
    assert!(!w.accept(&[0x25, 17]).unwrap());
    assert!(!w.ready());
    assert!(w.move_request(0).is_err());
}
