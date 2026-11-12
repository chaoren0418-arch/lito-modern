//! Offline packet/model tests for Issue #145. No game process or service.
use lito_modern::modern_lito::{
    native_attack_many,
    native_environment::State as Environment,
    native_inventory::Item,
    native_login::{LoginController, LoginPhase, Snapshot},
    native_portal::{self, Destination},
    native_skill_selection::{Ground, Model, Target},
    native_skills as skills,
    native_teleport_spell::Bookmarks,
    native_world_effects,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::time::{Duration, Instant};
const ITEM: u32 = 0xfedcba98;
const IDS: [u16; 8] = [5, 12, 20, 21, 22, 23, 25, 27];
fn fixture(data: &[u8], key: &str) -> Vec<u8> {
    let value: serde_json::Value = serde_json::from_slice(data).unwrap();
    let text = value["packets"][key].as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn state() -> Snapshot {
    let mut s = LoginController::default().snapshot().clone();
    for key in ["world", "start", "status", "actor", "bot"] {
        s.world
            .accept(&fixture(
                include_bytes!("fixtures/native-world-current.json"),
                key,
            ))
            .unwrap();
    }
    let mut own = s.world.actors.remove(&ITEM).unwrap();
    own.id = 1;
    own.x = 32770;
    own.y = 32818;
    s.world.actors.insert(1, own);
    s.world.status.as_mut().unwrap().object_id = 1;
    let actor = s.world.actors.get_mut(&2).unwrap();
    actor.x = 32771;
    actor.y = 32818;
    s.phase = LoginPhase::WorldReady;
    s.game_connected = true;
    s.selected_character = Some("Batch1".into());
    s.selected_source = Some("fixture".into());
    s.world.hp = Some([50, 50]);
    s.world.mp = Some([100, 100]);
    // Formal S1E learned-book bits, all eight IDs and no fabricated learned state.
    s.world.accept(&[0x1e, 4, 0x10, 0x08, 0x78, 0x05]).unwrap();
    s.inventory.begin(7, "Batch1").unwrap();
    s.inventory.items.insert(
        ITEM,
        Item {
            id: ITEM,
            use_type: 1,
            charge: 0,
            icon: 1,
            blessing: 0,
            count: 1,
            identified: 1,
            name: "weapon".into(),
            detail: vec![0],
        },
    );
    s
}
fn request(s: &Snapshot, id: u16) -> anyhow::Result<Vec<u8>> {
    match id {
        5 => skills::teleport_request(
            &s.world,
            Destination {
                map: 564,
                x: 32772,
                y: 32819,
            },
        ),
        12 | 21 => skills::item_request(s, 7, id, &skills::ItemTarget::capture(s, 7, ITEM)?),
        _ => skills::request(&s.world, id, Some(2)),
    }
}
fn effect(id: u32, gfx: u16) -> Vec<u8> {
    let mut b = vec![0x37];
    b.extend(id.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    b
}
fn many(gfx: u16) -> Vec<u8> {
    let mut b = vec![0x39, 18];
    b.extend(1u32.to_le_bytes());
    b.extend(32770u16.to_le_bytes());
    b.extend(32818u16.to_le_bytes());
    b.push(2);
    b.extend(9u32.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    b.extend([8, 0, 0]);
    b.extend(2u16.to_le_bytes());
    b.extend(2u32.to_le_bytes());
    b.push(0x20);
    b.extend(3u32.to_le_bytes());
    b.push(0);
    b
}

#[test]
fn all_eight_complete_wire_shapes_are_pure_and_use_learned_ids() {
    let s = state();
    assert_eq!(
        s.world.learned_skills.iter().copied().collect::<Vec<_>>(),
        IDS
    );
    let before = serde_json::to_value(&s).unwrap();
    for (id, expected) in [
        (5, vec![0x14, 0, 4, 0x34, 2, 4, 0x80, 0x33, 0x80]),
        (12, vec![0x14, 1, 3, 0x98, 0xba, 0xdc, 0xfe]),
        (20, vec![0x14, 2, 3, 2, 0, 0, 0, 3, 0x80, 0x32, 0x80]),
        (21, vec![0x14, 2, 4, 0x98, 0xba, 0xdc, 0xfe]),
        (22, vec![0x14, 2, 5, 2, 0, 0, 0, 3, 0x80, 0x32, 0x80]),
        (23, vec![0x14, 2, 6, 2, 0, 0, 0]),
        (25, vec![0x14, 3, 0, 2, 0, 0, 0, 3, 0x80, 0x32, 0x80]),
        (27, vec![0x14, 3, 2, 2, 0, 0, 0, 3, 0x80, 0x32, 0x80]),
    ] {
        assert_eq!(request(&s, id).unwrap(), expected, "skill {id}");
    }
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
    assert_eq!(
        [20, 22, 25, 27].map(skills::reuse_millis),
        [500, 300, 500, 3000]
    );
    // No local cost assumption: discounts, materials and eligibility stay server-owned.
    let mut low = s.clone();
    low.world.mp = Some([1, 100]);
    for id in IDS {
        assert!(request(&low, id).is_ok());
    }
}
#[test]
fn every_skill_rejects_unlearned_dead_blocked_and_zero_mana_without_mutation() {
    for id in IDS {
        for change in 0..7 {
            let mut s = state();
            match change {
                0 => {
                    s.world.learned_skills.remove(&id);
                }
                1 => s.world.mp = Some([0, 100]),
                2 => s.world.hp = Some([0, 50]),
                3 => s.world.actors.get_mut(&1).unwrap().dead = true,
                4 => {
                    s.world.movement_blocks.insert(2);
                }
                5 => s.world.started = false,
                _ => s.world.battle_result = Some(0),
            }
            let before = serde_json::to_value(&s).unwrap();
            assert!(request(&s, id).is_err(), "{id}/{change}");
            assert_eq!(serde_json::to_value(&s).unwrap(), before);
        }
    }
}
#[test]
fn actor_ranges_screen_edges_and_stale_selection_are_enforced() {
    for (id, range) in [(20, 1), (22, 6), (25, 6), (27, 4)] {
        for (distance, ok) in [(range, true), (range + 1, false)] {
            let mut s = state();
            s.world.actors.get_mut(&2).unwrap().x = 32770 + distance;
            assert_eq!(request(&s, id).is_ok(), ok, "{id}/{distance}");
        }
    }
    for id in [20, 22, 23, 25, 27] {
        let mut s = state();
        for target in [None, Some(0), Some(1), Some(999)] {
            assert!(skills::request(&s.world, id, target).is_err());
        }
        let target = Target::capture(&s.world, 2).unwrap();
        assert!(lito_modern::modern_lito::native_skill_selection::choice(
            &s.world,
            id,
            Some(target)
        )
        .is_ok());
        s.world.actors.get_mut(&2).unwrap().incarnation += 1;
        assert!(lito_modern::modern_lito::native_skill_selection::choice(
            &s.world,
            id,
            Some(target)
        )
        .is_err());
        s.world.actors.get_mut(&2).unwrap().dead = true;
        assert!(request(&s, id).is_err());
    }
    for (delta, ok) in [(18, true), (19, false), (-19, true), (-20, false)] {
        let mut s = state();
        s.world.actors.get_mut(&2).unwrap().x = (32770i32 + delta) as u16;
        assert_eq!(request(&s, 23).is_ok(), ok);
    }
}
#[test]
fn menu_exposes_eight_skills_and_keeps_item_actor_and_destination_domains_separate() {
    let s = state();
    let mut menu = Model::default();
    menu.sync(7, &s.world, true);
    for id in IDS {
        menu.select(Some(id)).unwrap();
        assert_eq!(menu.selected_skill(7, &s.world).unwrap(), id);
        assert!(!menu
            .rows()
            .iter()
            .find(|row| row.id == id)
            .unwrap()
            .label
            .contains("尚未"));
        assert!(menu.selected_skill(8, &s.world).is_err());
    }
    for id in [12, 21, 5] {
        assert!(skills::request(&s.world, id, Some(2)).is_err());
    }
    for id in [12, 21, 5] {
        assert!(skills::ground_request(&s.world, id, [1, 2]).is_err());
    }
    menu.select(Some(5)).unwrap();
    let ground = Ground::capture(7, &s.world, WorldCell { x: 32772, y: 32819 }).unwrap();
    assert!(menu
        .choice_at(7, &s.world, Target::capture(&s.world, 2), Some(ground))
        .is_err());
    let book = Bookmarks::default();
    let choice = book.choose(&s, 7, 0).unwrap();
    assert_eq!(
        choice.request(&s, 7, &book).unwrap(),
        [0x14, 0, 4, 0x34, 2, 0, 0, 0, 0]
    );
    let mut next = s.world.clone();
    next.generation += 1;
    assert!(menu.choice_at(7, &next, None, Some(ground)).is_err());
}
#[test]
fn item_target_rejects_inventory_revision_aba_epoch_and_scene_changes() {
    for skill in [12, 21] {
        let s = state();
        let target = skills::ItemTarget::capture(&s, 7, ITEM).unwrap();
        assert!(skills::ItemTarget::capture(&s, 7, 2).is_err());
        assert!(skills::item_request(&s, 8, skill, &target).is_err());
        for change in 0..10 {
            let mut changed = s.clone();
            match change {
                0 => changed.inventory.revision += 1,
                1 => {
                    let item = changed.inventory.items.remove(&ITEM).unwrap();
                    changed.inventory.items.insert(ITEM, item);
                    changed.inventory.revision += 2;
                }
                2 => changed.inventory.owner.as_mut().unwrap().selection += 1,
                3 => changed.world.generation += 1,
                4 => changed.world.control_interruptions += 1,
                5 => changed.world.map_id = Some(4),
                6 => changed.world.actors.get_mut(&1).unwrap().incarnation += 1,
                7 => changed.observer = true,
                8 => changed.game_connected = false,
                _ => changed.phase = LoginPhase::LobbyReady,
            }
            assert!(
                skills::item_request(&changed, 7, skill, &target).is_err(),
                "{skill}/{change}"
            );
        }
    }
}
fn bookmark(name: &str, d: Destination) -> Vec<u8> {
    let mut b = vec![0x30];
    b.extend(name.as_bytes());
    b.push(0);
    b.extend(d.x.to_le_bytes());
    b.extend(d.y.to_le_bytes());
    b.extend(d.map.to_le_bytes());
    b
}
#[test]
fn teleport_bookmarks_are_atomic_bounded_and_encode_map_before_coordinates() {
    let s = state();
    let mut book = Bookmarks::default();
    let d = Destination {
        map: 4,
        x: 0x8123,
        y: 0x8456,
    };
    let b = bookmark("saved", d);
    for end in 1..b.len() {
        assert!(book.receive(&s, 7, &b[..end]).is_err());
        assert_eq!(book.labels(&s, 7).len(), 1);
    }
    assert!(!book.receive(&s, 7, &[0x37]).unwrap());
    book.receive(&s, 7, &b).unwrap();
    let choice = book.choose(&s, 7, 1).unwrap();
    assert_eq!(choice.destination(), d);
    assert_eq!(
        choice.request(&s, 7, &book).unwrap(),
        [0x14, 0, 4, 4, 0, 0x23, 0x81, 0x56, 0x84]
    );
    book.receive(&s, 7, &b).unwrap();
    choice.request(&s, 7, &book).unwrap();
    let mut malformed = b.clone();
    malformed.push(0);
    assert!(book.receive(&s, 7, &malformed).is_err());
    choice.request(&s, 7, &book).unwrap();
    for i in 1..50 {
        book.receive(&s, 7, &bookmark(&format!("row{i}"), d))
            .unwrap();
    }
    assert_eq!(book.labels(&s, 7).len(), 51);
    let last = book.choose(&s, 7, 50).unwrap();
    assert!(book.receive(&s, 7, &bookmark("overflow", d)).is_err());
    last.request(&s, 7, &book).unwrap();
    assert!(book.choose(&s, 7, 51).is_err());
    book.receive(
        &s,
        7,
        &bookmark("saved", Destination { map: 4, x: 0, y: 0 }),
    )
    .unwrap();
    assert!(book.choose(&s, 7, 1).is_err());
}
#[test]
fn teleport_choices_retire_on_bookmark_identity_epoch_or_scene_changes() {
    let s = state();
    let mut book = Bookmarks::default();
    let d = Destination {
        map: 4,
        x: 10,
        y: 20,
    };
    let b = bookmark("saved", d);
    book.receive(&s, 7, &b).unwrap();
    let saved = book.choose(&s, 7, 1).unwrap();
    let random = book.choose(&s, 7, 0).unwrap();
    for choice in [&saved, &random] {
        assert!(choice.request(&s, 8, &book).is_err());
        for change in 0..9 {
            let mut next = s.clone();
            match change {
                0 => next.world.generation += 1,
                1 => next.selected_source = Some("other".into()),
                2 => next.selected_character = Some("other".into()),
                3 => {
                    next.inventory.begin(7, "Batch1").unwrap();
                }
                4 => next.game_connected = false,
                5 => next.phase = LoginPhase::LobbyReady,
                6 => next.observer = true,
                7 => next.world.map_id = Some(0),
                _ => next.world.actors.get_mut(&1).unwrap().incarnation += 1,
            }
            assert!(choice.request(&next, 7, &book).is_err(), "change {change}");
        }
    }
    book.receive(&s, 7, &bookmark("saved", Destination { x: 11, ..d }))
        .unwrap();
    assert!(saved.request(&s, 7, &book).is_err());
    random.request(&s, 7, &book).unwrap();
    // Restoring identical bytes cannot revive the old bookmark revision.
    book.receive(&s, 7, &b).unwrap();
    assert!(saved.request(&s, 7, &book).is_err());
    let mut next = s.clone();
    next.inventory.begin(8, "Batch1").unwrap();
    assert_eq!(book.labels(&next, 8).len(), 1);
    assert!(book.choose(&next, 8, 1).is_err());
}
#[test]
fn item_replies_and_weapon_break_material_rejection_only_change_received_authority() {
    for skill in [12, 21, 27] {
        let mut s = state();
        let now = Instant::now();
        let before = s.inventory.clone();
        request(&s, skill).unwrap();
        assert_eq!(s.inventory, before);
        let pending = skills::Pending {
            since: now,
            casts: s.world.own_cast_packets,
            messages: s.world.message_count,
            generation: s.world.generation,
            incarnation: s.world.own_actor().unwrap().incarnation,
        };
        s.world.accept(&[0x10, 79, 0, 0]).unwrap();
        assert_eq!(
            pending.observe_item(&s.world, now),
            skills::Feedback::Rejected(79)
        );
        assert_eq!(pending.observe(&s.world, now), skills::Feedback::Pending);
        s.world.accept(&[0x10, 0x2b, 1, 0]).unwrap();
        assert_eq!(
            pending.observe(&s.world, now),
            skills::Feedback::Rejected(299)
        );
        assert_eq!(s.inventory, before);
        let target = skills::ItemTarget::capture(&s, 7, ITEM).unwrap();
        s.inventory
            .accept(
                7,
                &fixture(
                    include_bytes!("fixtures/native-inventory-current.json"),
                    "extended",
                ),
            )
            .unwrap();
        assert_eq!(s.inventory.items[&ITEM].name, "更新名稱");
        assert!(target.validate(&s, 7).is_err());
    }
}
#[test]
fn blind_and_object_effect_replies_are_authoritative_and_truncation_atomic() {
    let mut s = state();
    let mut environment = Environment::default();
    environment.begin(7).unwrap();
    skills::request(&s.world, 20, Some(2)).unwrap();
    assert_eq!(environment.blind, 0);
    environment.accept(7, &[0x0a, 1]).unwrap();
    assert_eq!(environment.blind, 1);
    environment.accept(7, &[0x0a, 0]).unwrap();
    assert_eq!(environment.blind, 0);
    for (id, gfx) in [
        (5, 169),
        (12, 747),
        (20, 746),
        (21, 748),
        (23, 2166),
        (23, 2167),
        (23, 2168),
        (23, 2169),
        (27, 172),
    ] {
        request(&s, id).unwrap();
        let body = effect(2, gfx);
        let before = serde_json::to_value(&s.world).unwrap();
        for end in 0..body.len() {
            assert!(s.world.accept(&body[..end]).is_err());
            assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
        }
        s.world.accept(&body).unwrap();
        let cue = s.world.effects.last().unwrap();
        assert_eq!(cue.graphics, gfx);
        assert_eq!(cue.target, Some((2, s.world.actors[&2].incarnation)));
        assert_eq!(s.world.mp, Some([100, 100]));
    }
}
#[test]
fn area_replies_preserve_order_misses_origin_and_reject_malformed_packets_atomically() {
    for (skill, gfx) in [(22, 1804), (25, 171)] {
        let mut s = state();
        request(&s, skill).unwrap();
        let body = many(gfx);
        let before = serde_json::to_value(&s.world).unwrap();
        for end in 0..body.len() {
            assert!(s.world.accept(&body[..end]).is_err());
            assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
        }
        let mut extra = body.clone();
        extra.push(0);
        assert!(s.world.accept(&extra).is_err());
        let mut bad = body.clone();
        *bad.last_mut().unwrap() = 1;
        assert!(s.world.accept(&bad).is_err());
        assert!(s.world.accept(&body).unwrap());
        let cue = s.world.many_attacks.last().unwrap();
        assert_eq!(cue.target_origin, Some([32771, 32818]));
        assert_eq!(
            cue.targets[0],
            (2, true, Some(s.world.actors[&2].incarnation))
        );
        assert_eq!(cue.targets[1], (3, false, None));
        assert_eq!(s.world.hp, Some([50, 50]));
        assert_eq!(s.world.mp, Some([100, 100]));
        s.world.actors.get_mut(&2).unwrap().x += 10;
        assert_eq!(s.world.many_attacks[0].target_origin, Some([32771, 32818]));
        let attack = native_attack_many::decode(&body).unwrap().unwrap();
        s.world.actors.remove(&2);
        assert_eq!(
            native_attack_many::capture_target_origin(&attack, &s.world.actors),
            Some([32771, 32818])
        );
        let mut wrong = body.clone();
        wrong[17] = 0;
        assert!(native_attack_many::decode(&wrong).unwrap().is_none());
    }
}
#[test]
fn teleport_keeps_authoritative_position_until_portal_and_retires_old_scene() {
    let mut s = state();
    let original = s.world.clone();
    request(&s, 5).unwrap();
    assert_eq!(
        serde_json::to_value(&s.world).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    let d = Destination {
        map: 564,
        x: 32775,
        y: 32822,
    };
    native_portal::admit_teleport_spell(&s, d, 564).unwrap();
    let mut wrong = d;
    wrong.map = 4;
    assert!(native_portal::admit_teleport_spell(&s, wrong, 564).is_err());
    native_portal::admit_teleport_spell(&s, wrong, 4).unwrap();
    let mut battle = s.clone();
    battle.phase = LoginPhase::BattlePending;
    assert!(native_portal::admit_teleport_spell(&battle, wrong, 4).is_err());
    let b = [0x5a, 0x34, 2, 7, 0x80, 0x36, 0x80];
    for end in 0..b.len() {
        assert!(s.world.accept(&b[..end]).is_err());
    }
    s.world.accept(&b).unwrap();
    assert_eq!(s.world.portal.unwrap().destination, d);
    assert_eq!(s.world.generation, original.generation + 1);
    assert!(s.world.actors.is_empty());
    assert!(!s.world.ready());
    assert_eq!(native_portal::ACK, [8]);
    assert!(s.world.accept(&b).is_err());
    assert!(request(&s, 5).is_err());
    assert!(s.world.learned_skills.contains(&5));
}
#[test]
fn no_portal_rejection_timeout_and_retired_owner_never_invent_teleport_success() {
    let s = state();
    let now = Instant::now();
    let pending = skills::Pending {
        since: now,
        casts: s.world.own_cast_packets,
        messages: s.world.message_count,
        generation: s.world.generation,
        incarnation: s.world.own_actor().unwrap().incarnation,
    };
    assert_eq!(
        pending.observe(&s.world, now + Duration::from_secs(2)),
        skills::Feedback::NoResponse
    );
    assert!(s.world.portal.is_none());
    for code in [276u16, 278, 280, 299] {
        let mut rejected = s.world.clone();
        let mut b = vec![0x10];
        b.extend(code.to_le_bytes());
        b.push(0);
        rejected.accept(&b).unwrap();
        assert_eq!(
            pending.observe(&rejected, now),
            skills::Feedback::Rejected(code)
        );
        assert!(rejected.portal.is_none());
        assert_eq!(rejected.own_actor().unwrap().x, 32770);
    }
    let mut retired = s.world.clone();
    retired.generation += 1;
    assert_eq!(
        pending.observe(&retired, now),
        skills::Feedback::Interrupted
    );
}

#[test]
#[ignore = "requires: assets; read-only original archive, no window/audio device"]
fn original_batch1_effect_assets_render_and_retire_without_authority_writes() {
    use lito_modern::modern_lito::{
        native_combat_animation::EnteredEvent, native_skill_effects::Effects,
    };
    use std::{path::Path, sync::atomic::AtomicBool};
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let view = ViewProjection {
        anchor_cell: WorldCell { x: 32770, y: 32818 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    };
    for gfx in [747, 748, 746, 172, 2166, 2167, 2168, 2169] {
        let mut s = state();
        s.world.accept(&effect(2, gfx)).unwrap();
        let before = serde_json::to_value(&s.world).unwrap();
        let mut effects = Effects::default();
        effects.prepare(root, &s.world, &cancel);
        let mut drawn = false;
        let mut events = Vec::new();
        let mut last = serde_json::Value::Null;
        for tick in 0..150 {
            let mut fragments = Vec::new();
            let (report, audio) = effects.draw_to(
                &s.world,
                &mut native_world_effects::Target::Fragments(&mut fragments),
                view,
                tick * 100_000_000,
            );
            assert!(report["load_error"].is_null(), "{gfx}: {report}");
            drawn |= !fragments.is_empty();
            events.extend(audio);
            last = report;
        }
        assert!(drawn, "{gfx}");
        assert_eq!(last["active"], 0);
        assert!(!events.is_empty(), "{gfx}");
        assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
    }
    for gfx in [1804, 171] {
        let mut s = state();
        s.world.accept(&many(gfx)).unwrap();
        let cue = s.world.many_attacks[0].clone();
        let before = serde_json::to_value(&s.world).unwrap();
        let mut effects = native_attack_many::Effects::default();
        effects.prepare(root, &s.world, &cancel);
        let hit = EnteredEvent {
            prepared_audio: None,
            actor: 1,
            sequence: cue.action_sequence,
            action: 18,
            frame: 1,
            event: "Hit".into(),
            sound: None,
        };
        let mut drawn = false;
        let mut sounds = Vec::new();
        let mut impacts = Vec::new();
        let mut last = serde_json::Value::Null;
        for tick in 0..150 {
            impacts.extend(effects.update(
                &s.world,
                if tick == 0 { Some(&hit) } else { None }.into_iter(),
                tick * 100_000_000,
            ));
            let mut fragments = Vec::new();
            let (report, audio) = effects.draw_to(
                &mut native_world_effects::Target::Fragments(&mut fragments),
                view,
                tick * 100_000_000,
            );
            assert!(report["load_error"].is_null(), "{gfx}: {report}");
            drawn |= !fragments.is_empty();
            sounds.extend(audio);
            last = report;
        }
        assert!(drawn, "{gfx}");
        assert_eq!(last["active"], 0);
        assert_eq!(impacts.len(), 1);
        assert_eq!(
            sounds
                .iter()
                .filter_map(|event| event.sound)
                .collect::<Vec<_>>(),
            if gfx == 1804 { vec![714] } else { vec![99, 98] }
        );
        assert!(sounds.iter().all(|e| e.prepared_audio.is_some()));
        assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
    }
}
