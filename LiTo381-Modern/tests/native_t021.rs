//! T021 offline authority and request tests. No service, DB or game process.
use lito_modern::modern_lito::{
    native_inventory::Item,
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{Model, Target},
    native_skills as skills,
};
use std::time::{Duration, Instant};
const ITEM: u32 = 0xfedcba98;
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
    s.selected_character = Some("DarkElf".into());
    s.selected_source = Some("fixture".into());
    s.world.hp = Some([50, 50]);
    s.world.mp = Some([100, 100]);
    // Formal S1E learned-book bits, all eight IDs and no fabricated learned state.
    let mut book = vec![0x1e, 13];
    book.extend([0; 12]);
    book.push(0x0f);
    s.world.accept(&book).unwrap();
    s.inventory.begin(7, "DarkElf").unwrap();
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
    if id == 100 {
        skills::item_request(s, 7, id, &skills::ItemTarget::capture(s, 7, ITEM)?)
    } else {
        skills::request(&s.world, id, None)
    }
}
fn effect(s: &mut Snapshot, id: u32, gfx: u16) {
    let mut b = vec![0x37];
    b.extend(id.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    s.world.accept(&b).unwrap();
}
fn pending(s: &Snapshot, now: Instant) -> skills::Pending {
    skills::Pending {
        since: now,
        casts: s.world.own_cast_packets,
        messages: s.world.message_count,
        generation: s.world.generation,
        incarnation: s.world.own_actor().unwrap().incarnation,
    }
}
#[test]
fn wire_shapes_and_menu_keep_selectors_separate_from_inventory_objects() {
    let mut s = state();
    assert_eq!(
        s.world.learned_skills.iter().copied().collect::<Vec<_>>(),
        [97, 98, 99, 100]
    );
    let stale = Target::capture(&s.world, 2).unwrap();
    s.world.actors.remove(&2);
    let before = serde_json::to_value(&s).unwrap();
    let mut menu = Model::default();
    menu.sync(7, &s.world, true);
    for id in 97..=99 {
        menu.select(Some(id)).unwrap();
        let choice = menu.choice(7, &s.world, Some(stale)).unwrap();
        assert_eq!(choice.target, None);
        assert_eq!(request(&s, id).unwrap(), [0x14, 12, (id - 97) as u8]);
        assert!(skills::request(&s.world, id, Some(1)).is_err());
        assert!(menu
            .rows()
            .iter()
            .find(|r| r.id == id)
            .unwrap()
            .label
            .contains("自己"));
    }
    assert_eq!(
        request(&s, 100).unwrap(),
        [0x14, 12, 3, 0x98, 0xba, 0xdc, 0xfe]
    );
    assert!(skills::request(&s.world, 100, Some(ITEM)).is_err());
    assert!(skills::item_request(
        &s,
        7,
        98,
        &skills::ItemTarget::capture(&s, 7, ITEM).unwrap()
    )
    .is_err());
    assert_eq!([97, 98, 99, 100].map(skills::reuse_millis), [9990, 0, 0, 0]);
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
}
#[test]
fn mp_hp_and_missing_material_boundaries_remain_server_owned() {
    for id in 97..=100 {
        for mp in [0, 1, 4, 5, 9, 10, 11] {
            for hp in [0, 1, 2, 4, 5, 6] {
                let mut s = state();
                s.world.mp = Some([mp, 100]);
                s.world.hp = Some([hp, 50]);
                let before = serde_json::to_value(&s).unwrap();
                assert_eq!(request(&s, id).is_ok(), mp > 0 && hp > 0, "{id}/{mp}/{hp}");
                assert_eq!(serde_json::to_value(&s).unwrap(), before);
            }
        }
        for mode in 0..7 {
            let mut s = state();
            match mode {
                0 => {
                    s.world.learned_skills.remove(&id);
                }
                1 => s.world.hp = None,
                2 => s.world.mp = None,
                3 => s.world.actors.get_mut(&1).unwrap().dead = true,
                4 => {
                    s.world.movement_blocks.insert(2);
                }
                5 => s.world.started = false,
                _ => s.world.battle_result = Some(0),
            }
            let before = serde_json::to_value(&s).unwrap();
            assert!(request(&s, id).is_err());
            assert_eq!(serde_json::to_value(&s).unwrap(), before);
        }
    }
    let mut s = state();
    s.inventory.items.clear();
    assert!(request(&s, 98).is_ok());
    assert!(request(&s, 100).is_err()); // Material 40321 is not an inventory object ID.
}
#[test]
fn stone_selection_is_invalidated_by_inventory_owner_map_and_actor_changes() {
    let s = state();
    let target = skills::ItemTarget::capture(&s, 7, ITEM).unwrap();
    for mode in 0..10 {
        let mut n = s.clone();
        match mode {
            0 => n.inventory.revision += 1,
            1 => {
                n.inventory.items.remove(&ITEM);
            }
            2 => n.inventory.items.get_mut(&ITEM).unwrap().count += 1,
            3 => {
                n.inventory.begin(8, "Other").unwrap();
            }
            4 => n.world.generation += 1,
            5 => n.world.actors.get_mut(&1).unwrap().incarnation += 1,
            6 => n.world.map_id = Some(999),
            7 => n.game_connected = false,
            8 => n.observer = true,
            _ => n.world.observer_mode = true,
        }
        assert!(skills::item_request(&n, 7, 100, &target).is_err(), "{mode}");
    }
    assert!(skills::item_request(&s, 8, 100, &target).is_err());
}
#[test]
fn s34_and_remove_object_reappearance_are_authoritative_and_aba_safe() {
    let mut s = state();
    let now = Instant::now();
    let p = pending(&s, now);
    let costs = (s.world.hp, s.world.mp, s.inventory.clone());
    for gfx in 2941..=2944 {
        effect(&mut s, 1, gfx);
    }
    assert_eq!(p.observe(&s.world, now), skills::Feedback::Pending);
    assert!(!s.world.own_actor().unwrap().draw_state.invisible);
    s.world.accept(&[0x34, 1, 0, 0, 0, 1]).unwrap();
    assert!(s.world.own_actor().unwrap().draw_state.invisible);
    // L13: even after 32/320 seconds the client has no authority to expire invisibility.
    for seconds in [32, 320, 961] {
        assert_eq!(
            p.observe(&s.world, now + Duration::from_secs(seconds)),
            skills::Feedback::NoResponse
        );
        assert!(s.world.own_actor().unwrap().draw_state.invisible);
    }
    s.world.accept(&[0x34, 1, 0, 0, 0, 0]).unwrap();
    assert!(!s.world.own_actor().unwrap().draw_state.invisible);
    let target = Target::capture(&s.world, 2).unwrap();
    s.world.accept(&[0x34, 2, 0, 0, 0, 1]).unwrap();
    assert!(target.resolve(&s.world).is_none());
    s.world.accept(&[0x34, 2, 0, 0, 0, 0]).unwrap();
    assert!(target.resolve(&s.world).is_none());
    let fresh = Target::capture(&s.world, 2).unwrap();
    // Historical S08 label: formal E149 Protocol.removeObject is wire 0x15.
    s.world.accept(&[0x15, 2, 0, 0, 0]).unwrap();
    assert!(fresh.resolve(&s.world).is_none());
    s.world
        .accept(&fixture(
            include_bytes!("fixtures/native-world-current.json"),
            "bot",
        ))
        .unwrap();
    assert!(s.world.actor_visible(2));
    assert!(fresh.resolve(&s.world).is_none());
    assert_eq!((s.world.hp, s.world.mp, s.inventory), costs);
}
#[test]
fn rejection_and_timeout_never_fabricate_costs_buff_or_refinement_success() {
    for id in 97..=100 {
        for code in [278u16, 279, 299, 280, 285, 316] {
            let mut s = state();
            let now = Instant::now();
            let p = pending(&s, now);
            let before = (s.world.hp, s.world.mp, s.inventory.clone());
            request(&s, id).unwrap();
            effect(&mut s, 1, 2941 + id - 97);
            let mut b = vec![0x10];
            b.extend(code.to_le_bytes());
            b.push(0);
            s.world.accept(&b).unwrap();
            assert_eq!(p.observe(&s.world, now), skills::Feedback::Rejected(code));
            // A later unrelated message must not erase a rejection.
            s.world.accept(&[0x10, 1, 0, 0]).unwrap();
            assert_eq!(
                p.observe(&s.world, now + Duration::from_secs(3)),
                skills::Feedback::Rejected(code)
            );
            assert_eq!((s.world.hp, s.world.mp, s.inventory), before);
        }
    }
    assert_eq!(skills::Feedback::Rejected(279).label(), "HP 不足");
}
#[test]
fn costs_and_stone_inventory_change_only_on_receipts_and_reject_truncation() {
    let mut s = state();
    let target = skills::ItemTarget::capture(&s, 7, ITEM).unwrap();
    request(&s, 100).unwrap();
    effect(&mut s, 1, 2944);
    assert_eq!(s.inventory.items[&ITEM].count, 1);
    s.world.accept(&[0x0d, 1, 0, 50, 0]).unwrap();
    s.world.accept(&[0x4d, 5, 0, 100, 0]).unwrap();
    assert_eq!((s.world.hp, s.world.mp), (Some([1, 50]), Some([5, 100])));
    let mut count = vec![0x1a];
    count.extend(ITEM.to_le_bytes());
    count.extend(2u32.to_le_bytes());
    for n in 1..count.len() {
        let before = s.inventory.clone();
        assert!(s.inventory.accept(7, &count[..n]).is_err());
        assert_eq!(s.inventory, before);
    }
    s.inventory.accept(7, &count).unwrap();
    assert_eq!(s.inventory.items[&ITEM].count, 2);
    assert!(skills::item_request(&s, 7, 100, &target).is_err());
    let mut remove = vec![0x17];
    remove.extend(ITEM.to_le_bytes());
    s.inventory.accept(7, &remove).unwrap();
    assert!(!s.inventory.items.contains_key(&ITEM));
    for b in [
        vec![0x34, 1, 0, 0, 0],
        vec![0x34, 1, 0, 0, 0, 2],
        vec![0x37, 1, 0, 0, 0, 0],
        vec![0x15, 2, 0, 0],
    ] {
        let before = serde_json::to_value(&s.world).unwrap();
        assert!(s.world.accept(&b).is_err());
        assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
    }
}

#[test]
fn shadow_armor_ac_and_poison_color_require_explicit_server_updates() {
    let mut s = state();
    let ac = s.world.status.unwrap().armor_class;
    request(&s, 99).unwrap();
    effect(&mut s, 1, 2943);
    assert_eq!(s.world.status.unwrap().armor_class, ac);
    s.world.accept(&[0x52, 0xfd, 1, 2, 3, 4]).unwrap();
    assert_eq!(s.world.status.unwrap().armor_class, -3);
    request(&s, 98).unwrap();
    effect(&mut s, 1, 2942);
    assert!(!s.world.own_actor().unwrap().draw_state.green);
    s.world.accept(&[0x32, 1, 0, 0, 0, 1, 0]).unwrap();
    assert!(s.world.own_actor().unwrap().draw_state.green);
    s.world.accept(&[0x32, 1, 0, 0, 0, 0, 0]).unwrap();
    assert!(!s.world.own_actor().unwrap().draw_state.green);
    s.world.accept(&[0x52, ac as u8, 0, 0, 0, 0]).unwrap();
    assert_eq!(s.world.status.unwrap().armor_class, ac);
}
