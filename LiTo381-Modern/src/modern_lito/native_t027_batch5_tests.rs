//! Issue #149 (T027 batch 5: 67, 68, 69, 70, 71, 73, 74, 75). Offline packets and
//! models only; no game, service, GUI or sound device. The skill 69 caster
//! handshake and the own-object `monlist` offer are controller cases in
//! `native_t027_batch5_controller_tests.rs`.
use super::*;
use crate::modern_lito::{
    native_attack_many as many, native_combat_animation::EnteredEvent, native_forced_combat,
    native_inventory::Item,
    native_login::{LoginController, LoginPhase, Snapshot},
    native_portal::Destination,
    native_skill_selection::{cycle_target, Model, Target},
    native_target_policy,
    native_teleport_spell::Bookmarks,
    native_world_effects,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::time::Instant;

const IDS: [u16; 8] = [67, 68, 69, 70, 71, 73, 74, 75];
const ITEM: u32 = 0xfedc_ba98;
const EPOCH: u64 = 7;
const SPOT: Destination = Destination { map: 564, x: 32772, y: 32819 };

fn packet(file: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(file).unwrap();
    let s = v["packets"][key].as_str().unwrap();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
/// Same offline fixture as the earlier batches: own id 1 at 32770/32818, bot 2
/// one cell east, map 564; plus one inventory object for the item-target skill.
fn state() -> Snapshot {
    let mut s = LoginController::default().snapshot().clone();
    for key in ["world", "start", "status", "actor", "bot"] {
        s.world.accept(&packet(include_bytes!("../../tests/fixtures/native-world-current.json"), key)).unwrap();
    }
    let mut own = s.world.actors.remove(&ITEM).unwrap();
    own.id = 1;
    own.x = 32770;
    own.y = 32818;
    s.world.actors.insert(1, own);
    s.world.status.as_mut().unwrap().object_id = 1;
    let bot = s.world.actors.get_mut(&2).unwrap();
    bot.x = 32771;
    bot.y = 32818;
    s.phase = LoginPhase::WorldReady;
    s.game_connected = true;
    s.selected_character = Some("Batch5".into());
    s.selected_source = Some("fixture".into());
    s.world.hp = Some([50, 50]);
    s.world.mp = Some([100, 100]);
    // Formal S1E learned-book bits: row8 cols2..=6 (67–71), row9 cols0..=2 (73–75).
    s.world.accept(&[0x1e, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0x7c, 0x07]).unwrap();
    s.inventory.begin(EPOCH, "Batch5").unwrap();
    s.inventory.items.insert(ITEM, Item { id: ITEM, use_type: 1, charge: 0, icon: 1, blessing: 0, count: 1,
        identified: 1, name: "weapon".into(), detail: vec![0] });
    s
}
fn corpse(mut s: Snapshot) -> Snapshot {
    s.world.actors.get_mut(&2).unwrap().dead = true;
    s
}
fn ready(id: u16) -> Snapshot {
    if id == 75 { corpse(state()) } else { state() }
}
/// Each skill through its own formal request path.
fn cast(s: &Snapshot, id: u16) -> Result<Vec<u8>> {
    match id {
        68 | 70 => request(&s.world, id, None),
        69 => mass_teleport_request(&s.world, SPOT),
        73 => item_request(s, EPOCH, id, &ItemTarget::capture(s, EPOCH, ITEM)?),
        _ => request(&s.world, id, Some(2)),
    }
}
fn effect(id: u32, gfx: u16) -> Vec<u8> {
    let mut b = vec![0x37];
    b.extend(id.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    b
}
/// S39 cdhhcdhccs/h/(dc) from caster 1 at its own cell.
fn s39(gfx: u16, kind: u8, heading: u8, targets: &[(u32, u8)]) -> Vec<u8> {
    let mut b = vec![0x39, 18];
    b.extend(1u32.to_le_bytes());
    b.extend(32770u16.to_le_bytes());
    b.extend(32818u16.to_le_bytes());
    b.push(heading);
    b.extend(9u32.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    b.extend([kind, 0, 0]);
    b.extend((targets.len() as u16).to_le_bytes());
    for &(id, marker) in targets {
        b.extend(id.to_le_bytes());
        b.push(marker);
    }
    b
}
fn pending(w: &World, now: Instant) -> Pending {
    Pending { since: now, casts: w.own_cast_packets, messages: w.message_count,
        generation: w.generation, incarnation: w.own_actor().unwrap().incarnation }
}
fn message(w: &mut World, code: u16) {
    let mut b = vec![0x10];
    b.extend(code.to_le_bytes());
    b.push(0);
    w.accept(&b).unwrap();
}
fn view() -> ViewProjection {
    ViewProjection { anchor_cell: WorldCell { x: 32770, y: 32818 }, anchor_pixel: PixelPoint { x: 400, y: 300 } }
}

#[test]
fn eight_wire_shapes_are_exact_and_costs_are_not_predicted() {
    let mut s = state();
    assert_eq!(s.world.learned_skills.iter().copied().collect::<Vec<_>>(), IDS);
    // HP/MP of one: MP80／30／60／48／200／100, gems and probabilities stay server-owned.
    s.world.hp = Some([1, 50]);
    s.world.mp = Some([1, 100]);
    let expected: [(u16, Vec<u8>); 8] = [
        (67, vec![0x14, 8, 2, 2, 0, 0, 0]),
        (68, vec![0x14, 8, 3, 1, 0, 0, 0]),
        (69, vec![0x14, 8, 4, 0x34, 2, 4, 0x80, 0x33, 0x80]),
        (70, vec![0x14, 8, 5]),
        (71, vec![0x14, 8, 6, 2, 0, 0, 0, 0x03, 0x80, 0x32, 0x80]),
        (73, vec![0x14, 9, 0, 0x98, 0xba, 0xdc, 0xfe]),
        (74, vec![0x14, 9, 1, 2, 0, 0, 0, 0x03, 0x80, 0x32, 0x80]),
        (75, vec![0x14, 9, 2, 2, 0, 0, 0]),
    ];
    for (id, expected) in expected {
        let s = if id == 75 { corpse(s.clone()) } else { s.clone() };
        let before = serde_json::to_value(&s).unwrap();
        let body = cast(&s, id).unwrap();
        assert_eq!(body, expected, "{id}");
        assert_eq!(body.len(), match id { 70 => 3, 69 => 9, 71 | 74 => 11, _ => 7 }, "{id}");
        assert_eq!(serde_json::to_value(&s).unwrap(), before, "{id} mutated state");
        // Ctrl/LFC1 and the player-target policy only ever see actor bodies: a
        // map/x/y tail, a selector and an inventory object ID are not actors.
        let object = match id { 67 | 71 | 74 | 75 => Some(2), 68 => Some(1), _ => None };
        assert_eq!(native_forced_combat::object_target(&body), object, "{id}");
        let mut battle = s.clone();
        battle.phase = LoginPhase::BattlePending;
        native_target_policy::validate(&battle, &body, native_forced_combat::Input::default()).unwrap();
    }
    // 67 without a selection is an explicit self cast; a stale ID never falls back.
    assert_eq!(request(&s.world, 67, None).unwrap()[3..], 1u32.to_le_bytes());
    assert!(request(&s.world, 67, Some(77)).is_err());
    assert!(request(&s.world, 70, Some(2)).is_err() && selector_only(70));
    for id in [71, 74, 75] {
        assert!(request(&s.world, id, None).is_err(), "{id} needs an explicit target");
        assert!(request(&s.world, id, Some(1)).is_err(), "{id} never targets the caster");
    }
    // 69 and 73 have no actor, ground or default form.
    for id in [69, 73] {
        assert!(request(&s.world, id, None).is_err(), "{id}");
        assert!(request(&s.world, id, Some(2)).is_err(), "{id}");
        assert!(ground_request(&s.world, id, [32771, 32818]).is_err(), "{id}");
        assert!(!selector_only(id));
    }
    assert!(destination_skill(69) && destination_skill(5) && !destination_skill(73));
    assert!(item_targeted(73) && !weapon_bound(73));
    assert_eq!(IDS.map(reuse_millis), [10, 1500, 10, 2000, 500, 0, 3000, 10]);
    assert_eq!(IDS.map(client_clock_millis), [0, 0, 0, 0, 500, 0, 0, 0]);
}

#[test]
fn caster_gates_reject_without_partial_mutation() {
    for id in IDS {
        for reason in 0..7 {
            let mut s = ready(id);
            match reason {
                0 => { s.world.learned_skills.remove(&id); }
                1 => s.world.hp = Some([0, 50]),
                2 => s.world.mp = None,
                3 => s.world.mp = Some([0, 100]),
                4 => s.world.actors.get_mut(&1).unwrap().dead = true,
                5 => { s.world.movement_blocks.insert(1); }
                _ => { s.world.actors.remove(&1); }
            }
            let before = serde_json::to_value(&s).unwrap();
            assert!(cast(&s, id).is_err(), "{id}/{reason}");
            assert_eq!(serde_json::to_value(&s).unwrap(), before, "{id}/{reason}");
        }
    }
    // Observers never request a mass teleport.
    let mut s = state();
    s.world.observer_mode = true;
    assert!(mass_teleport_request(&s.world, SPOT).is_err());
}

#[test]
fn target_kind_range_and_clan_follow_the_formal_record() {
    // Living-actor skills refuse corpses and absent actors; 75 refuses the living.
    for id in [67, 71, 74] {
        assert!(cast(&corpse(state()), id).is_err(), "{id} on corpse");
        let mut s = state();
        s.world.actors.remove(&2);
        assert!(cast(&s, id).is_err(), "{id} absent");
    }
    assert!(cast(&state(), 75).is_err(), "75 on a living actor");
    let mut s = corpse(state());
    s.world.actors.remove(&2);
    assert!(cast(&s, 75).is_err());
    // Chebyshev ranges including the diagonal edge.
    for (id, range) in [(71, 3), (74, 10)] {
        let mut s = state();
        let a = s.world.actors.get_mut(&2).unwrap();
        a.x = 32770 + range;
        a.y = 32818 + range;
        assert!(cast(&s, id).is_ok(), "{id} at {range}");
        s.world.actors.get_mut(&2).unwrap().x += 1;
        assert!(cast(&s, id).is_err(), "{id} beyond {range}");
    }
    // 67 and 75 use the native screen diamond, not an unlimited range.
    for id in [67, 75] {
        let mut s = ready(id);
        s.world.actors.get_mut(&2).unwrap().x = 32770 + 18;
        assert!(cast(&s, id).is_ok(), "{id} at 18");
        s.world.actors.get_mut(&2).unwrap().x = 32770 + 20;
        assert!(cast(&s, id).is_err(), "{id} at 20");
    }
    // 68: self, or a displayed same non-empty clan. The server still owns the
    // real membership; Ctrl cannot bypass it (the body is the ordinary one).
    let mut s = state();
    for (own, other, ok) in [("", "", false), ("Pledge", "", false), ("", "Pledge", false),
        ("Pledge", "Other", false), ("Pledge", "Pledge", true)] {
        s.world.actors.get_mut(&1).unwrap().clan = own.into();
        s.world.actors.get_mut(&2).unwrap().clan = other.into();
        assert_eq!(request(&s.world, 68, Some(2)).is_ok(), ok, "{own:?}/{other:?}");
        assert!(request(&s.world, 68, None).is_ok(), "self needs no clan");
    }
    assert_eq!(request(&s.world, 68, Some(2)).unwrap(), [0x14, 8, 3, 2, 0, 0, 0]);
    assert!(request(&corpse(s.clone()).world, 68, Some(2)).is_err());
}

#[test]
fn menu_labels_corpse_intent_and_destination_domain_cannot_be_mixed() {
    let s = state();
    let w = &s.world;
    let mut menu = Model::default();
    menu.sync(EPOCH, w, true);
    assert_eq!(menu.rows().iter().map(|r| r.id).collect::<Vec<_>>(), IDS);
    assert!(menu.rows().iter().all(|r| !r.label.contains("尚未支援")), "{:?}", menu.rows());
    // A living selection never becomes a 75 corpse authorization after death…
    let living = Target::capture(w, 2).unwrap();
    menu.select(Some(75)).unwrap();
    assert!(menu.choice_at(EPOCH, w, Some(living), None).is_err());
    let dead = corpse(s.clone()).world;
    assert!(menu.choice_at(EPOCH, &dead, Some(living), None).is_err());
    // …a captured corpse authorizes only that corpse, and never a revived actor.
    let body = Target::capture(&dead, 2).unwrap();
    assert_eq!(menu.choice_at(EPOCH, &dead, Some(body), None).unwrap().target, Some(2));
    let mut revived = dead.clone();
    revived.actors.get_mut(&2).unwrap().dead = false;
    assert!(menu.choice_at(EPOCH, &revived, Some(body), None).is_err());
    for id in [67, 71, 74] {
        menu.select(Some(id)).unwrap();
        assert!(menu.choice_at(EPOCH, &dead, Some(body), None).is_err(), "{id}: corpse as a living target");
        assert_eq!(menu.choice_at(EPOCH, w, Some(living), None).unwrap().target, Some(2), "{id}");
    }
    // T key: corpse skills cycle corpses only, the others living actors only.
    assert!(corpse_skill(75) && corpse_skill(61) && corpse_skill(41) && !corpse_skill(71));
    assert!(cycle_target(w, [2u32], 1, None, true).is_none());
    let cycled = cycle_target(&dead, [2u32], 1, None, true).unwrap();
    assert_eq!((cycled.resolve_for_skill(&dead, 75), cycled.resolve_for_skill(&dead, 71)), (Some(2), None));
    assert!(cycle_target(&dead, [2u32], 1, None, false).is_none());
    // Selector-only 70 drops a hovered actor instead of sending it.
    menu.select(Some(70)).unwrap();
    assert_eq!(menu.choice_at(EPOCH, w, Some(living), None).unwrap().target, None);
    // 69 (destination) and 73 (inventory object) never pass through the actor path.
    for id in [69, 73] {
        menu.select(Some(id)).unwrap();
        assert!(menu.choice_at(EPOCH, w, Some(living), None).is_err(), "{id}");
        assert!(menu.choice_at(EPOCH, w, None, None).is_err(), "{id}");
    }
    menu.select(Some(71)).unwrap();
    let mut moved = w.clone();
    moved.actors.get_mut(&2).unwrap().incarnation += 1;
    assert!(menu.choice_at(EPOCH, &moved, Some(living), None).is_err(), "stale incarnation");
    assert!(menu.choice_at(EPOCH + 1, w, Some(living), None).is_err(), "other connection");
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
fn mass_teleport_destinations_are_explicit_skill_bound_and_retire_with_their_owner() {
    let mut s = state();
    s.world.learned_skills.insert(5);
    let mut book = Bookmarks::default();
    // Random travel: the current map with zero coordinates, nine bytes.
    let random = book.choose_for(&s, EPOCH, 69, 0).unwrap();
    assert_eq!(random.skill(), 69);
    assert_eq!(random.request(&s, EPOCH, &book).unwrap(), [0x14, 8, 4, 0x34, 2, 0, 0, 0, 0]);
    // A fixed cell only comes from a received bookmark: map before x/y.
    let d = Destination { map: 4, x: 0x8123, y: 0x8456 };
    assert!(book.choose_for(&s, EPOCH, 69, 1).is_err(), "no bookmark received yet");
    book.receive(&s, EPOCH, &bookmark("saved", d)).unwrap();
    let saved = book.choose_for(&s, EPOCH, 69, 1).unwrap();
    assert_eq!(saved.destination(), d);
    assert_eq!(saved.request(&s, EPOCH, &book).unwrap(), [0x14, 8, 4, 4, 0, 0x23, 0x81, 0x56, 0x84]);
    // The same row chosen for skill 5 stays a skill-5 request; neither is the other.
    let five = book.choose(&s, EPOCH, 1).unwrap();
    assert_eq!((five.skill(), five.request(&s, EPOCH, &book).unwrap()[..3].to_vec()), (5, vec![0x14, 0, 4]));
    assert!(book.choose_for(&s, EPOCH, 67, 0).is_err() && book.choose_for(&s, EPOCH, 73, 0).is_err());
    // Learning 5 never authorizes 69.
    let mut unlearned = s.clone();
    unlearned.world.learned_skills.remove(&69);
    assert!(book.choose_for(&unlearned, EPOCH, 69, 0).is_err());
    assert!(saved.request(&unlearned, EPOCH, &book).is_err());
    assert!(five.request(&unlearned, EPOCH, &book).is_ok());
    // Owner, scene and bookmark-revision changes retire the choice (no ABA revival).
    for choice in [&random, &saved] {
        assert!(choice.request(&s, EPOCH + 1, &book).is_err());
        for change in 0..6 {
            let mut next = s.clone();
            match change {
                0 => next.world.generation += 1,
                1 => next.selected_source = Some("other".into()),
                2 => next.selected_character = Some("other".into()),
                3 => next.observer = true,
                4 => next.world.map_id = Some(0),
                _ => next.world.actors.get_mut(&1).unwrap().incarnation += 1,
            }
            assert!(choice.request(&next, EPOCH, &book).is_err(), "change {change}");
        }
    }
    book.receive(&s, EPOCH, &bookmark("saved", Destination { x: 11, ..d })).unwrap();
    assert!(saved.request(&s, EPOCH, &book).is_err());
    random.request(&s, EPOCH, &book).unwrap();
    book.receive(&s, EPOCH, &bookmark("saved", d)).unwrap();
    assert!(saved.request(&s, EPOCH, &book).is_err(), "identical bytes cannot revive the old revision");
}

#[test]
fn object_effects_are_atomic_and_never_grant_form_status_item_or_life() {
    // 67 230, 68 228, 69 2236 (caster), 71 2232, 73 763 (caster), 75 cast 3944 + light 230.
    for (id, gfx) in [(2u32, 230u16), (2, 228), (1, 2236), (2, 2232), (1, 763), (2, 3944), (1, 230)] {
        let original = if gfx == 3944 { corpse(state()) } else { state() };
        let b = effect(id, gfx);
        for n in 0..b.len() {
            let mut s = original.clone();
            let before = serde_json::to_value(&s).unwrap();
            assert!(s.world.accept(&b[..n]).is_err(), "{gfx} prefix {n}");
            assert_eq!(serde_json::to_value(&s).unwrap(), before);
        }
        let mut s = original.clone();
        s.world.accept(&b).unwrap();
        let (w, o) = (&s.world, &original.world);
        assert_eq!(w.effects.len(), o.effects.len() + 1);
        let cue = w.effects.last().unwrap();
        assert_eq!((cue.graphics, cue.target.map(|t| t.0)), (gfx, Some(id)));
        assert_eq!(cue.position, [w.actors[&id].x, w.actors[&id].y], "frozen receipt position");
        assert_eq!((w.hp, w.mp), (o.hp, o.mp));
        assert_eq!(w.movement_blocks, o.movement_blocks);
        assert_eq!(serde_json::to_value(&w.actors).unwrap(), serde_json::to_value(&o.actors).unwrap(),
            "{gfx}: no appearance, death, position or status change");
        assert_eq!((w.map_id, w.portal), (o.map_id, o.portal), "2236 is not a teleport");
        assert_eq!(w.own_cast_packets, o.own_cast_packets, "S37 is not a cast ACK");
        assert_eq!(serde_json::to_value(&s.inventory).unwrap(), serde_json::to_value(&original.inventory).unwrap(),
            "763 is not an enchant");
    }
    // These presentations already have reviewed exact finite definitions.
    for gfx in [228u16, 763, 2232, 2236] {
        assert!(super::super::native_additional_finite_effects::GRAPHICS.contains(&gfx), "{gfx}");
    }
    assert!(super::super::native_additional_finite_effects::BATCH4_GRAPHICS.contains(&230));
}

#[test]
fn centered_and_area_s39_keep_order_misses_and_the_formal_profile_only() {
    let targets = [(2u32, 0x20u8), (3, 0)];
    for (gfx, kind) in [(1819u16, 0u8), (762, 8)] {
        let original = state().world;
        let b = s39(gfx, kind, 2, &targets);
        for n in 0..b.len() {
            let mut w = original.clone();
            let before = serde_json::to_value(&w).unwrap();
            assert!(w.accept(&b[..n]).is_err(), "{gfx} prefix {n}");
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
        }
        let mut w = original.clone();
        let now = Instant::now();
        let p = pending(&w, now);
        w.accept(&b).unwrap();
        assert_eq!(w.many_attacks.len(), 1, "{gfx}");
        let cue = &w.many_attacks[0];
        assert_eq!((cue.attack.graphics, cue.attack.action, cue.attack.position), (gfx, 18, [32770, 32818]));
        assert_eq!(cue.targets.iter().map(|t| (t.0, t.1)).collect::<Vec<_>>(), vec![(2, true), (3, false)]);
        assert_eq!(cue.targets[1].2, None, "unknown target keeps no incarnation");
        // 70 is centered on the frozen wire caster origin; 74 freezes its first target.
        assert_eq!(cue.target_origin, (gfx == 762).then_some([32771, 32818]));
        assert_eq!(w.own_cast_packets, original.own_cast_packets + 1);
        assert_eq!(p.observe(&w, now), Feedback::CastAction);
        assert_eq!((w.hp, w.mp, w.actors[&2].hp_ratio), (original.hp, original.mp, original.actors[&2].hp_ratio));
        // Wrong action/type for the same gfx stays quarantined, without mutation.
        for (action, kind) in [(19u8, kind), (18, if kind == 0 { 8 } else { 0 })] {
            let mut bad = b.clone();
            bad[1] = action;
            bad[17] = kind;
            assert!(many::decode(&bad).unwrap().is_none(), "{gfx}/{action}/{kind}");
            let mut w = original.clone();
            let before = serde_json::to_value(&w.many_attacks).unwrap();
            assert!(!w.accept(&bad).unwrap());
            assert_eq!(serde_json::to_value(&w.many_attacks).unwrap(), before);
            assert_eq!(w.own_cast_packets, original.own_cast_packets);
        }
    }
    // 762: a missed first target still owns the origin; an absent first target
    // falls back one heading step and never borrows the second target.
    let mut w = state().world;
    w.accept(&s39(762, 8, 4, &[(2, 0), (99, 0x20)])).unwrap();
    assert_eq!(w.many_attacks[0].target_origin, Some([32771, 32818]));
    let mut w = state().world;
    w.accept(&s39(762, 8, 4, &[(99, 0x20), (2, 0x20)])).unwrap();
    assert_eq!(w.many_attacks[0].target_origin, Some([32770, 32819]));
    // Later movement of the first target does not move an accepted origin.
    let mut w = state().world;
    w.accept(&s39(762, 8, 4, &[(2, 0x20)])).unwrap();
    w.actors.get_mut(&2).unwrap().x += 3;
    assert_eq!(w.many_attacks[0].target_origin, Some([32771, 32818]));
    // An empty 1819 list is still the caster's accepted cast.
    let mut w = state().world;
    w.accept(&s39(1819, 0, 4, &[])).unwrap();
    assert_eq!((w.many_attacks.len(), w.many_attacks[0].targets.len()), (1, 0));
}

#[test]
fn unqualified_presentations_are_omitted_not_substituted() {
    // 75's eight-bank 3944 cast and the 1819 four-node／762 class1 area effects
    // have no qualified presentation yet: nothing is drawn, played or hit locally.
    let mut w = corpse(state()).world;
    w.effects.clear();
    w.accept(&effect(2, 3944)).unwrap();
    let mut fx = super::super::native_skill_effects::Effects::default();
    let mut fragments = Vec::new();
    let (r, events) = fx.draw_to(&w, &mut native_world_effects::Target::Fragments(&mut fragments), view(), 0);
    assert!(fragments.is_empty() && events.is_empty(), "{r}");
    assert_eq!((r["started"].as_u64(), r["omitted"].as_u64()), (Some(0), Some(1)), "{r}");
    for gfx in [1819u16, 762] {
        let mut w = state().world;
        w.accept(&s39(gfx, if gfx == 762 { 8 } else { 0 }, 2, &[(2, 0x20)])).unwrap();
        let hit = EnteredEvent { prepared_audio: None, actor: 1, sequence: w.actors[&1].action_sequence,
            action: 18, frame: 1, event: "Hit".into(), sound: None };
        let mut fx = many::Effects::default();
        assert!(fx.update(&w, std::iter::once(&hit), 0).is_empty(), "{gfx}: no local Impact or HP");
        let mut fragments = Vec::new();
        let (r, sounds) = fx.draw_to(&mut native_world_effects::Target::Fragments(&mut fragments), view(), 0);
        assert!(fragments.is_empty() && sounds.is_empty(), "{gfx}: {r}");
        assert_eq!(r["omitted"].as_u64(), Some(1), "{gfx}: {r}");
    }
}

#[test]
fn rejections_keep_their_codes_and_silence_times_out() {
    // 276 map refusal (69), 278 MP, 279 HP, 280 failed cast, 299 missing gems.
    for code in [276, 278, 279, 280, 299] {
        let mut w = state().world;
        let now = Instant::now();
        let p = pending(&w, now);
        message(&mut w, code);
        assert_eq!(p.observe(&w, now), Feedback::Rejected(code));
    }
    // 73: only the item path reads the sequenced 79; 685／161 are not replies.
    let mut w = state().world;
    let now = Instant::now();
    let p = pending(&w, now);
    for code in [685, 161] {
        message(&mut w, code);
        assert_eq!(p.observe_item(&w, now), Feedback::Pending, "{code}");
    }
    message(&mut w, 79);
    assert_eq!(p.observe(&w, now), Feedback::Pending);
    assert_eq!(p.observe_item(&w, now), Feedback::Rejected(79));
    let w = state().world;
    let now = Instant::now();
    let p = pending(&w, now);
    assert_eq!(p.observe(&w, now), Feedback::Pending);
    assert_eq!(p.observe(&w, now + std::time::Duration::from_secs(2)), Feedback::NoResponse);
}
