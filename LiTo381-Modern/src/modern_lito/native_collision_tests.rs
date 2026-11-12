//! Synthetic geometry and real packet envelopes; no external service or process.
use super::*;
use crate::modern_lito::{
    native_doors, native_obstacles,
    native_world::{destination, World},
};

fn packet(name: &str) -> Vec<u8> {
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
fn world() -> World {
    let mut w = World::default();
    for name in ["world", "start", "status", "actor", "hp"] {
        w.accept(&packet(name)).unwrap();
    }
    let a = w.actors.get_mut(&w.status.unwrap().object_id).unwrap();
    a.x = 32736;
    a.y = 32800;
    w
}
fn navigation() -> Navigation {
    Navigation {
        map_id: 564,
        sectors: BTreeMap::from([((511, 512), vec![0; 4096].into_boxed_slice())]),
        portal_sources: BTreeSet::new(),
    }
}
fn edge(nav: &mut Navigation, x: u16, y: u16, bit: u8) {
    let s = nav
        .sectors
        .get_mut(&(i32::from(x) / 64, i32::from(y) / 64))
        .unwrap();
    s[usize::from(y % 64) * 64 + usize::from(x % 64)] |= 1 << bit;
}
fn attribute(w: &mut World, x: u16, y: u16, direction: u8, closed: bool) {
    let mut b = vec![0x22];
    b.extend(x.to_le_bytes());
    b.extend(y.to_le_bytes());
    b.extend([direction, if closed { 65 } else { 0 }]);
    w.accept(&b).unwrap();
}
fn actor(w: &mut World, id: u32, x: u16, y: u16, door: bool) {
    let mut b = packet(if door { "door" } else { "bot" });
    b[1..3].copy_from_slice(&x.to_le_bytes());
    b[3..5].copy_from_slice(&y.to_le_bytes());
    b[5..9].copy_from_slice(&id.to_le_bytes());
    w.accept(&b).unwrap();
}
fn action(w: &mut World, id: u32, action: u8) {
    let mut b = vec![0x20];
    b.extend(id.to_le_bytes());
    b.push(action);
    w.accept(&b).unwrap();
}

#[test]
fn unowned_boundaries_are_directional_and_latest_packet_wins() {
    let mut w = world();
    let nav = navigation();
    for (bit, heading, reverse, x, y) in [(0, 0, 4, 32736, 32800), (1, 2, 6, 32736, 32800)] {
        attribute(&mut w, x, y, bit, true);
        assert!(w.door_attributes.values().all(|r| r.owner.is_none()));
        assert!(
            !native_doors::blocks(&w, x, y),
            "edge is not a whole occupied cell"
        );
        assert!(w.checked_move_request(heading, &nav).is_err());
        let (tx, ty) = destination(x, y, heading).unwrap();
        assert!(
            !nav.can_step_with_edges(tx.into(), ty.into(), reverse, |x, y, d| {
                native_doors::edge_override(&w, x, y, d)
            })
        );
        attribute(&mut w, x, y, bit, false);
        assert!(w.checked_move_request(heading, &nav).is_ok());
    }
    assert_eq!(
        w.door_attributes.len(),
        2,
        "directions have independent keys"
    );
    assert_eq!(w.predicted_moves, 0);
}

#[test]
fn merged_static_dynamic_diagonals_cover_every_four_edge_combination() {
    let (x, y) = (32736u16, 32800u16);
    for h in [1, 3, 5, 7] {
        let (dx, dy) = DIR[h as usize];
        let key = |x: i32, y: i32, dx, dy| match (dx, dy) {
            (0, -1) => (x as u16, y as u16, 0),
            (1, 0) => (x as u16, y as u16, 1),
            (0, 1) => (x as u16, (y + 1) as u16, 0),
            (-1, 0) => ((x - 1) as u16, y as u16, 1),
            _ => panic!(),
        };
        let (sx, sy) = (i32::from(x), i32::from(y));
        let keys = [
            key(sx, sy, dx, 0),
            key(sx + dx, sy, 0, dy),
            key(sx, sy, 0, dy),
            key(sx, sy + dy, dx, 0),
        ];
        for static_mask in 0u8..16 {
            for dynamic_mask in 0u8..16 {
                let mut nav = navigation();
                let mut w = world();
                for (i, &(ex, ey, bit)) in keys.iter().enumerate() {
                    if static_mask & (1 << i) != 0 {
                        edge(&mut nav, ex, ey, bit);
                    }
                    if dynamic_mask & (1 << i) != 0 {
                        attribute(&mut w, ex, ey, bit, true);
                    }
                }
                let merged = static_mask | dynamic_mask;
                let expected = merged & 3 == 0 || merged & 12 == 0;
                assert_eq!(
                    w.checked_move_request(h, &nav).is_ok(),
                    expected,
                    "heading={h}, static={static_mask}, dynamic={dynamic_mask}"
                );
            }
        }
    }
}

#[test]
fn open_attribute_overrides_static_edge_but_cannot_create_missing_sector() {
    let mut nav = navigation();
    let mut w = world();
    edge(&mut nav, 32736, 32800, 1);
    assert!(w.checked_move_request(2, &nav).is_err());
    attribute(&mut w, 32736, 32800, 1, false);
    assert!(w.checked_move_request(2, &nav).is_ok());
    let id = w.own_actor().unwrap().id;
    w.actors.get_mut(&id).unwrap().x = 32767;
    attribute(&mut w, 32767, 32800, 1, false);
    assert!(w.checked_move_request(2, &nav).is_err());
}

#[test]
fn castle_multi_tile_directional_updates_need_no_sprite_or_invented_rectangle() {
    for direction in [0, 1] {
        let mut w = world();
        let nav = navigation();
        // Castle writers send one edge per entrance tile, potentially offset
        // from the door sprite. No object ID is present in S_CHANGE_ATTR.
        for delta in -2i32..=2 {
            let (x, y) = if direction == 0 {
                ((32736 + delta) as u16, 32800)
            } else {
                (32736, (32800 + delta) as u16)
            };
            attribute(&mut w, x, y, direction, true);
        }
        let h = if direction == 0 { 0 } else { 2 };
        assert!(w.checked_move_request(h, &nav).is_err());
        for delta in -2i32..=2 {
            let (x, y) = if direction == 0 {
                ((32736 + delta) as u16, 32800)
            } else {
                (32736, (32800 + delta) as u16)
            };
            attribute(&mut w, x, y, direction, false);
        }
        assert!(w.checked_move_request(h, &nav).is_ok());
    }
}

#[test]
fn arena_and_siege_gates_block_all_entry_directions_until_attribute_opens() {
    for id in [0x62000000, 0x62000001] {
        let mut w = world();
        let nav = navigation();
        actor(&mut w, id, 32736, 32800, true);
        attribute(&mut w, 32736, 32800, 0, true);
        let own = w.own_actor().unwrap().id;
        for h in 0..8 {
            let (dx, dy) = DIR[h as usize];
            let a = w.actors.get_mut(&own).unwrap();
            a.x = (32736 - dx) as u16;
            a.y = (32800 - dy) as u16;
            let before = serde_json::to_value(&w).unwrap();
            assert!(w.checked_move_request(h, &nav).is_err());
            assert_eq!(before, serde_json::to_value(&w).unwrap());
        }
        action(&mut w, id, 28);
        assert!(native_obstacles::blocks(&w, 32736, 32800));
        attribute(&mut w, 32736, 32800, 0, false);
        assert!(!native_obstacles::blocks(&w, 32736, 32800));
        assert!(w.checked_move_request(7, &nav).is_ok());
    }
}

#[test]
fn closing_arena_gate_allows_departure_and_opening_it_does_not_erase_terrain() {
    let mut w = world();
    let mut nav = navigation();
    let door = 0x62000001;
    actor(&mut w, door, 32736, 32800, true);
    attribute(&mut w, 32736, 32800, 0, true);
    for h in 0..8 {
        assert!(w.checked_move_request(h, &nav).is_ok(), "departure heading={h}");
    }
    action(&mut w, door, 28);
    attribute(&mut w, 32736, 32800, 0, false);
    edge(&mut nav, 32736, 32800, 0);
    let id = w.own_actor().unwrap().id;
    w.actors.get_mut(&id).unwrap().y -= 1;
    assert!(w.checked_move_request(4, &nav).is_err(), "server still checks terrain");
}

#[test]
fn quest_and_time_attack_occupancy_is_live_destination_only_in_all_directions() {
    for id in [0x6c000000, 0x6cffffff, 0x6d000000, 0x6dffffff] {
        let mut w = world();
        let nav = navigation();
        w.encounter_mode =
            native_obstacles::EncounterMode::from_room_mode(if id < 0x6d000000 { 2 } else { 4 });
        actor(&mut w, id, 32736, 32800, false);
        // Zero displayed ratio does not certify death.
        let mut hp = vec![0x68];
        hp.extend(id.to_le_bytes());
        hp.push(0);
        w.accept(&hp).unwrap();
        let own = w.own_actor().unwrap().id;
        for h in 0..8 {
            let (dx, dy) = DIR[h as usize];
            let a = w.actors.get_mut(&own).unwrap();
            a.x = (32736 - dx) as u16;
            a.y = (32800 - dy) as u16;
            assert!(w.checked_move_request(h, &nav).is_err());
        }
        action(&mut w, id, 8);
        assert!(!native_obstacles::blocks(&w, 32736, 32800));
        assert!(w.checked_move_request(7, &nav).is_ok());
    }
}

#[test]
fn monster_move_removal_and_replacement_never_leave_cached_occupancy() {
    let mut w = world();
    let id = 0x6c000123u32;
    w.encounter_mode = Some(native_obstacles::EncounterMode::Quest);
    actor(&mut w, id, 32736, 32800, false);
    let mut b = vec![0x12];
    b.extend(id.to_le_bytes());
    b.extend(32736u16.to_le_bytes());
    b.extend(32800u16.to_le_bytes());
    b.push(2);
    w.accept(&b).unwrap();
    assert!(!native_obstacles::blocks(&w, 32736, 32800));
    assert!(native_obstacles::blocks(&w, 32737, 32800));
    actor(&mut w, id, 32738, 32800, false);
    assert!(!native_obstacles::blocks(&w, 32737, 32800));
    assert!(native_obstacles::blocks(&w, 32738, 32800));
    let mut remove = vec![0x15];
    remove.extend(id.to_le_bytes());
    w.accept(&remove).unwrap();
    assert!(!native_obstacles::blocks(&w, 32738, 32800));
}

#[test]
fn ordinary_players_companions_and_same_graphics_outside_namespace_do_not_block() {
    for id in [2, 0x6bffffff, 0x6e000000, 0xfedcba97] {
        let mut w = world();
        actor(&mut w, id, 32737, 32800, false);
        assert!(!native_obstacles::blocks(&w, 32737, 32800));
        assert!(w.checked_move_request(2, &navigation()).is_ok());
    }
}

#[test]
fn encounter_namespaces_are_scoped_to_authoritative_room_mode() {
    for id in [0x6c000001, 0x6d000001] {
        for mode in 0..=7 {
            let mut w = world();
            actor(&mut w, id, 32737, 32800, false);
            assert!(
                !native_obstacles::blocks(&w, 32737, 32800),
                "no room authority"
            );
            w.encounter_mode = native_obstacles::EncounterMode::from_room_mode(mode);
            assert_eq!(
                native_obstacles::blocks(&w, 32737, 32800),
                (id == 0x6c000001 && mode == 2) || (id == 0x6d000001 && mode == 4)
            );
        }
    }
}

#[test]
fn u16_bounds_and_missing_diagonal_intermediates_are_never_opened_by_attributes() {
    let nav = Navigation {
        map_id: 564,
        sectors: BTreeMap::from([
            ((0, 0), vec![0; 4096].into_boxed_slice()),
            ((1, 1), vec![0; 4096].into_boxed_slice()),
            ((1023, 1023), vec![0; 4096].into_boxed_slice()),
            ((1024, 1023), vec![0; 4096].into_boxed_slice()),
        ]),
        portal_sources: BTreeSet::new(),
    };
    for (x, y, h) in [
        (0, 0, 0),
        (0, 0, 6),
        (65535, 65535, 2),
        (65535, 65535, 4),
        (63, 63, 3),
    ] {
        assert!(!nav.can_step(x, y, h));
        assert!(!nav.can_step_with_edges(x, y, h, |_, _, _| Some(false)));
    }
}

#[test]
fn rejected_attribute_is_atomic_and_world_reset_discards_dynamic_edges() {
    let mut w = world();
    attribute(&mut w, 32736, 32800, 0, true);
    let before = serde_json::to_value(&w).unwrap();
    for b in [
        vec![0x22, 0, 0],
        vec![0x22, 0, 0, 0, 0, 2, 65],
        vec![0x22, 0, 0, 0, 0, 1, 64],
    ] {
        assert!(w.accept(&b).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    w.accept(&packet("world")).unwrap();
    assert!(w.door_attributes.is_empty());
}
