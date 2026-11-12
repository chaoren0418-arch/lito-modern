//! Issue #150 (T027 batch 6: 76, 79, 80, 87, 88, 89, 90, 91). Offline packets,
//! models and the original Text definitions only; no game, service, GUI or
//! sound device.
use super::*;
use crate::modern_lito::{
    native_additional_finite_effects::qualify_batch6,
    native_attack_many as many, native_combat_animation::EnteredEvent, native_forced_combat,
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_effects::qualify_finite,
    native_skill_selection::{Model, Target},
    native_target_policy, native_world_effects,
    sprite_actions::{ActionLimits, Animation, FrameEvent, SpriteActionCatalog},
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::time::Instant;

const IDS: [u16; 8] = [76, 79, 80, 87, 88, 89, 90, 91];
const SELECTORS: [u16; 5] = [80, 88, 89, 90, 91];
const ITEM: u32 = 0xfedc_ba98;
const EPOCH: u64 = 7;

fn packet(file: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(file).unwrap();
    let s = v["packets"][key].as_str().unwrap();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
/// Same offline fixture as the earlier batches: own id 1 at 32770/32818, bot 2
/// one cell east, map 564. The caster holds a two-hand sword (walk family 50).
fn state() -> Snapshot {
    let mut s = LoginController::default().snapshot().clone();
    for key in ["world", "start", "status", "actor", "bot"] {
        s.world.accept(&packet(include_bytes!("../../tests/fixtures/native-world-current.json"), key)).unwrap();
    }
    let mut own = s.world.actors.remove(&ITEM).unwrap();
    own.id = 1;
    own.x = 32770;
    own.y = 32818;
    own.walk_action = TWO_HAND_SWORD_WALK_ACTION;
    s.world.actors.insert(1, own);
    s.world.status.as_mut().unwrap().object_id = 1;
    let bot = s.world.actors.get_mut(&2).unwrap();
    bot.x = 32771;
    bot.y = 32818;
    s.phase = LoginPhase::WorldReady;
    s.game_connected = true;
    s.selected_character = Some("Batch6".into());
    s.selected_source = Some("fixture".into());
    s.world.hp = Some([500, 500]);
    s.world.mp = Some([100, 100]);
    // Formal S1E learned-book bits: row9 cols3／6／7 (76, 79, 80), row10 cols6／7
    // (87, 88), row11 cols0..=2 (89–91).
    s.world.accept(&[0x1e, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xc8, 0xc0, 0x07]).unwrap();
    s
}
fn corpse(mut s: Snapshot) -> Snapshot {
    s.world.actors.get_mut(&2).unwrap().dead = true;
    s
}
/// Each skill through its formal request path: selectors carry no target.
fn cast(s: &Snapshot, id: u16) -> Result<Vec<u8>> {
    request(&s.world, id, if SELECTORS.contains(&id) { None } else { Some(2) })
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
    // HP/MP of one: MP20／20／60／15／5／5／10／10, HP50／120／100／100, the two gems of
    // 76 and every probability stay server-owned.
    s.world.hp = Some([1, 500]);
    s.world.mp = Some([1, 100]);
    let expected: [(u16, Vec<u8>); 8] = [
        (76, vec![0x14, 9, 3, 2, 0, 0, 0, 0x03, 0x80, 0x32, 0x80]),
        (79, vec![0x14, 9, 6, 2, 0, 0, 0]),
        (80, vec![0x14, 9, 7]),
        (87, vec![0x14, 10, 6, 2, 0, 0, 0, 0x03, 0x80, 0x32, 0x80]),
        (88, vec![0x14, 10, 7]),
        (89, vec![0x14, 11, 0]),
        (90, vec![0x14, 11, 1]),
        (91, vec![0x14, 11, 2]),
    ];
    for (id, expected) in expected {
        let before = serde_json::to_value(&s).unwrap();
        let body = cast(&s, id).unwrap();
        assert_eq!(body, expected, "{id}");
        assert_eq!(body.len(), match id { 76 | 87 => 11, 79 => 7, _ => 3 }, "{id}");
        assert_eq!(serde_json::to_value(&s).unwrap(), before, "{id} mutated state");
        // Ctrl/LFC1 and the player-target policy only ever see actor bodies.
        let object = (!SELECTORS.contains(&id)).then_some(2);
        assert_eq!(native_forced_combat::object_target(&body), object, "{id}");
        let mut battle = s.clone();
        battle.phase = LoginPhase::BattlePending;
        native_target_policy::validate(&battle, &body, native_forced_combat::Input::default()).unwrap();
        assert_eq!(selector_only(id), SELECTORS.contains(&id), "{id}");
        assert!(!item_targeted(id) && !destination_skill(id) && !corpse_targeted(id) && !text_skill(id), "{id}");
    }
    // 79 without a selection is an explicit self cast; a stale ID never falls back.
    assert_eq!(request(&s.world, 79, None).unwrap(), [0x14, 9, 6, 1, 0, 0, 0]);
    assert!(request(&s.world, 79, Some(77)).is_err());
    for id in [76, 87] {
        assert!(request(&s.world, id, None).is_err(), "{id} needs an explicit target");
        assert!(request(&s.world, id, Some(1)).is_err(), "{id} never targets the caster");
        assert!(request(&s.world, id, Some(77)).is_err(), "{id} absent target");
    }
    for id in SELECTORS {
        assert!(request(&s.world, id, Some(2)).is_err(), "{id} takes no target");
        assert!(request(&s.world, id, Some(1)).is_err(), "{id} takes no target, not even self");
        assert!(ground_request(&s.world, id, [32771, 32818]).is_err(), "{id}");
    }
    assert_eq!(IDS.map(reuse_millis), [3000, 10, 3000, 8000, 3000, 5000, 3000, 3000]);
    assert_eq!(IDS.map(client_clock_millis), [0; 8]);
}

#[test]
fn caster_gates_reject_without_partial_mutation() {
    for id in IDS {
        for reason in 0..7 {
            let mut s = state();
            match reason {
                0 => { s.world.learned_skills.remove(&id); }
                1 => s.world.hp = Some([0, 500]),
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
        let mut s = state();
        s.world.observer_mode = true;
        assert!(cast(&s, id).is_err(), "{id} observer");
    }
}

#[test]
fn target_kind_range_and_weapon_stance_follow_the_formal_record() {
    // Living-actor skills refuse corpses and absent actors.
    for id in [76, 79, 87] {
        assert!(cast(&corpse(state()), id).is_err(), "{id} on corpse");
        let mut s = state();
        s.world.actors.remove(&2);
        assert!(cast(&s, id).is_err(), "{id} absent");
    }
    // Chebyshev ranges including the diagonal edge: 76 ten tiles, 87 one tile.
    for (id, range) in [(76, 10), (87, 1)] {
        let mut s = state();
        let a = s.world.actors.get_mut(&2).unwrap();
        a.x = 32770 + range;
        a.y = 32818 + range;
        assert!(cast(&s, id).is_ok(), "{id} at {range}");
        s.world.actors.get_mut(&2).unwrap().x += 1;
        assert!(cast(&s, id).is_err(), "{id} beyond {range}");
    }
    // 79 uses the native screen diamond, not an unlimited range.
    let mut s = state();
    s.world.actors.get_mut(&2).unwrap().x = 32770 + 18;
    assert!(cast(&s, 79).is_ok());
    s.world.actors.get_mut(&2).unwrap().x = 32770 + 20;
    assert!(cast(&s, 79).is_err());
    assert!(request(&s.world, 79, None).is_ok(), "self stays available");
    // 87: walk family 50 (two-hand sword) or 54 (edoryu); 91: exactly 50. The
    // other six never read the stance; the real weapon stays server-checked.
    for walk in [0u8, 4, 11, 20, 24, 40, 46, 50, 54, 58] {
        let mut s = state();
        s.world.actors.get_mut(&1).unwrap().walk_action = walk;
        let before = serde_json::to_value(&s).unwrap();
        assert_eq!(cast(&s, 87).is_ok(), matches!(walk, 50 | 54), "87 walk {walk}");
        assert_eq!(cast(&s, 91).is_ok(), walk == 50, "91 walk {walk}");
        for id in [76, 79, 80, 88, 89, 90] {
            assert!(cast(&s, id).is_ok(), "{id} walk {walk}");
        }
        assert_eq!(serde_json::to_value(&s).unwrap(), before);
    }
    assert_eq!((TWO_HAND_SWORD_WALK_ACTION, EDORYU_WALK_ACTION), (50, 54));
    // The target's stance is irrelevant.
    let mut s = state();
    s.world.actors.get_mut(&2).unwrap().walk_action = 0;
    assert!(cast(&s, 87).is_ok());
}

#[test]
fn menu_labels_selectors_and_stale_targets_cannot_be_mixed() {
    let s = state();
    let w = &s.world;
    let mut menu = Model::default();
    menu.sync(EPOCH, w, true);
    assert_eq!(menu.rows().iter().map(|r| r.id).collect::<Vec<_>>(), IDS);
    assert!(menu.rows().iter().all(|r| !r.label.contains("尚未支援")), "{:?}", menu.rows());
    let living = Target::capture(w, 2).unwrap();
    // Selector-only skills drop a hovered actor instead of sending it.
    for id in SELECTORS {
        menu.select(Some(id)).unwrap();
        let choice = menu.choice_at(EPOCH, w, Some(living), None).unwrap();
        assert_eq!((choice.skill, choice.target, choice.ground), (id, None, None), "{id}");
        assert_eq!(menu.choice_at(EPOCH, w, None, None).unwrap().target, None, "{id}");
    }
    // Actor skills keep the explicit living target; a corpse never qualifies.
    let dead = corpse(s.clone()).world;
    let body = Target::capture(&dead, 2).unwrap();
    for id in [76, 79, 87] {
        menu.select(Some(id)).unwrap();
        assert_eq!(menu.choice_at(EPOCH, w, Some(living), None).unwrap().target, Some(2), "{id}");
        assert!(menu.choice_at(EPOCH, &dead, Some(body), None).is_err(), "{id}: corpse as a living target");
        assert!(menu.choice_at(EPOCH, &dead, Some(living), None).is_err(), "{id}: target died");
        let mut moved = w.clone();
        moved.actors.get_mut(&2).unwrap().incarnation += 1;
        assert!(menu.choice_at(EPOCH, &moved, Some(living), None).is_err(), "{id}: stale incarnation");
        assert!(menu.choice_at(EPOCH + 1, w, Some(living), None).is_err(), "{id}: other connection");
    }
    // 79 without a selection is the caster; 76／87 never default to anyone.
    menu.select(Some(79)).unwrap();
    assert_eq!(menu.choice_at(EPOCH, w, None, None).unwrap().target, None);
    for id in [76, 87] {
        menu.select(Some(id)).unwrap();
        assert!(menu.choice_at(EPOCH, w, None, None).is_err(), "{id}");
    }
    // The stance gate also holds through the menu path.
    let mut bare = w.clone();
    bare.actors.get_mut(&1).unwrap().walk_action = 0;
    for id in [87, 91] {
        menu.select(Some(id)).unwrap();
        assert!(menu.choice_at(EPOCH, &bare, Some(living), None).is_err(), "{id}");
    }
}

#[test]
fn hp_cost_boundaries_stay_with_the_server_and_279_differs_from_278() {
    // Formal handlers need HP strictly above the cost (equal is S10 279) and
    // answer 278 for MP. The client sends the same three bytes on both sides of
    // the boundary and never spends HP or MP itself.
    for (id, cost) in [(88u16, 50u16), (89, 120), (90, 100), (91, 100)] {
        for hp in [1, cost - 1, cost, cost + 1] {
            let mut s = state();
            s.world.hp = Some([hp, 500]);
            s.world.mp = Some([1, 100]);
            let before = serde_json::to_value(&s).unwrap();
            assert_eq!(cast(&s, id).unwrap(), [0x14, ((id - 1) / 8) as u8, ((id - 1) % 8) as u8], "{id} at HP {hp}");
            assert_eq!(serde_json::to_value(&s).unwrap(), before, "{id} at HP {hp}");
        }
    }
    for code in [278u16, 279, 280, 299] {
        let mut w = state().world;
        let now = Instant::now();
        let p = pending(&w, now);
        let (hp, mp) = (w.hp, w.mp);
        message(&mut w, code);
        assert_eq!(p.observe(&w, now), Feedback::Rejected(code));
        assert_eq!((w.hp, w.mp), (hp, mp), "{code}: a refusal changes no points");
    }
    assert_eq!(Feedback::Rejected(279).label(), "HP 不足");
    assert_eq!(Feedback::Rejected(278).label(), "MP 不足");
    // The accepted knight casts answer with S20 action 19 for the caster; a
    // refusal received afterwards in the same pending window still wins.
    let mut w = state().world;
    let now = Instant::now();
    let p = pending(&w, now);
    assert_eq!(p.observe(&w, now), Feedback::Pending);
    w.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(p.observe(&w, now), Feedback::CastAction);
    assert_eq!((w.hp, w.mp), (Some([500, 500]), Some([100, 100])), "the action is not a cost receipt");
    // Somebody else's action 19 is not this caster's reply; silence times out.
    let mut w = state().world;
    let p = pending(&w, now);
    w.accept(&[0x20, 2, 0, 0, 0, 19]).unwrap();
    assert_eq!(p.observe(&w, now), Feedback::Pending);
    assert_eq!(p.observe(&w, now + std::time::Duration::from_secs(2)), Feedback::NoResponse);
}

#[test]
fn object_effects_are_atomic_and_never_grant_status_or_spend_points() {
    // 76 3932 (main target), 79 3935 (target or self), 87 4434 (target), 88 4824,
    // 89 4648, 90 5831, 91 5832 (caster) and the counter-barrier trigger 10710.
    for (id, gfx) in [(2u32, 3932u16), (2, 3935), (1, 3935), (2, 4434), (1, 4824), (1, 4648),
        (1, 5831), (1, 5832), (1, 10710)] {
        let original = state();
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
        assert_eq!((w.hp, w.mp), (o.hp, o.mp), "{gfx}: no HP／MP cost or maximum change");
        assert_eq!(w.movement_blocks, o.movement_blocks, "{gfx}: 4434 is not a stun");
        assert_eq!(serde_json::to_value(&w.actors).unwrap(), serde_json::to_value(&o.actors).unwrap(),
            "{gfx}: no speed, damage, death or position change");
        assert_eq!(w.own_cast_packets, o.own_cast_packets, "S37 is not a cast ACK");
        assert_eq!(w.many_attacks.len(), o.many_attacks.len());
        // A later move of the actor does not move the accepted effect.
        let frozen = cue.position;
        s.world.actors.get_mut(&id).unwrap().x += 3;
        assert_eq!(s.world.effects.last().unwrap().position, frozen);
    }
}

#[test]
fn freezing_blizzard_s39_keeps_order_misses_and_the_formal_profile_only() {
    let targets = [(2u32, 0x20u8), (3, 0)];
    let original = state().world;
    let b = s39(3933, 0, 2, &targets);
    for n in 0..b.len() {
        let mut w = original.clone();
        let before = serde_json::to_value(&w).unwrap();
        assert!(w.accept(&b[..n]).is_err(), "prefix {n}");
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    let mut w = original.clone();
    let now = Instant::now();
    let p = pending(&w, now);
    w.accept(&b).unwrap();
    assert_eq!(w.many_attacks.len(), 1);
    let cue = &w.many_attacks[0];
    assert_eq!((cue.attack.graphics, cue.attack.action, cue.attack.position), (3933, 18, [32770, 32818]));
    assert_eq!(cue.targets.iter().map(|t| (t.0, t.1)).collect::<Vec<_>>(), vec![(2, true), (3, false)]);
    assert_eq!(cue.targets[1].2, None, "unknown target keeps no incarnation");
    assert_eq!(cue.target_origin, None, "centred on the frozen wire caster origin");
    assert_eq!(w.own_cast_packets, original.own_cast_packets + 1);
    assert_eq!(p.observe(&w, now), Feedback::CastAction);
    // No local damage, freeze or status: HP, MP, ratios and blocks are untouched.
    assert_eq!((w.hp, w.mp, w.actors[&2].hp_ratio), (original.hp, original.mp, original.actors[&2].hp_ratio));
    assert_eq!(w.movement_blocks, original.movement_blocks);
    // Wrong action/type for the same gfx stays quarantined, without mutation.
    for (action, kind) in [(19u8, 0u8), (18, 8)] {
        let mut bad = b.clone();
        bad[1] = action;
        bad[17] = kind;
        assert!(many::decode(&bad).unwrap().is_none(), "{action}/{kind}");
        let mut w = original.clone();
        let before = serde_json::to_value(&w.many_attacks).unwrap();
        assert!(!w.accept(&bad).unwrap());
        assert_eq!(serde_json::to_value(&w.many_attacks).unwrap(), before);
        assert_eq!(w.own_cast_packets, original.own_cast_packets);
    }
    // An empty list is still the caster's accepted cast.
    let mut w = state().world;
    w.accept(&s39(3933, 0, 4, &[])).unwrap();
    assert_eq!((w.many_attacks.len(), w.many_attacks[0].targets.len()), (1, 0));
    // The 31-frame presentation is not qualified yet: nothing is drawn, played
    // or hit locally, and no generic effect is substituted.
    let mut w = state().world;
    w.accept(&s39(3933, 0, 2, &[(2, 0x20)])).unwrap();
    let hit = EnteredEvent { prepared_audio: None, actor: 1, sequence: w.actors[&1].action_sequence,
        action: 18, frame: 1, event: "Hit".into(), sound: None };
    let mut fx = many::Effects::default();
    assert!(fx.update(&w, std::iter::once(&hit), 0).is_empty(), "no local Impact or HP");
    let mut fragments = Vec::new();
    let (r, sounds) = fx.draw_to(&mut native_world_effects::Target::Fragments(&mut fragments), view(), 0);
    assert!(fragments.is_empty() && sounds.is_empty(), "{r}");
    assert_eq!(r["omitted"].as_u64(), Some(1), "{r}");
}

/// Byte-exact rows of the original Text `list.spr` (decoded size 4,067,417,
/// SHA-256 523C61AD…E8B9F127), CRLF and indentation included.
const ORIGINAL: [&str; 8] = [
    "#3932 1 mass slow\r\n\t0.effect(0 13,0.12:4 0.0:4 0.1:4<1349 0.2:4 0.3:4 0.4:4 0.5:4 0.6:4 0.7:4 0.8:5 0.9:5 0.10:6 0.11:7)\r\n\t102.type(0)\r\n\t104.attr(8)\r\n\t109.effect(1 -1)\r\n",
    "#3935 1 advance holiness\r\n\t0.effect(0 19,0.18:4 0.0:2<1356 0.1:2 0.2:2 0.3:2 0.4:2 0.5:2 0.6:2 0.7:2 0.8:2 0.9:2 0.10:2 0.11:2 0.12:2 0.13:3 0.14:4 0.15:4 0.16:4 0.17:5)\r\n\t102.type(0)\r\n\t104.attr(8)\r\n\t109.effect(0 -1)\r\n",
    "#4434 1 shock sturn\r\n\t0.animation(0 6,0.5:1 0.0:1 0.1:1[1498 0.2:1 0.3:1 0.4:1)\r\n\t104.attr(8)\r\n\t109.effect(1 -1)\r\n",
    "#4824 1 knight reduction armor effect\r\n 0.animation(0 19,0.18:4 0.0:1 0.1:1 0.2:1<3089 0.3:2 0.4:2 0.5:2 0.6:3 0.7:3 0.8:3 0.9:3 0.10:4 0.11:4 0.12:4 0.13:4 0.14:4 0.15:4 0.16:4 0.17:4)\r\n 104.attr(8)\r\n 109.effect(0 -1)\r\n",
    "#4648 1 knight bounce attack effect\r\n  0.animation(0 22,0.21:4 0.0:1 0.1:1 0.2:1 0.3:1<1924 0.4:1 0.5:1 0.6:2 0.7:2 0.8:2 0.9:2 0.10:2 0.11:2 0.12:3 0.13:3 0.14:3 0.15:4 0.16:4 0.17:4 0.18:4 0.19:4 0.20:4)\r\n  104.attr(8)\r\n  109.effect(0 -1)\r\n",
    "#5831 1 solid carriage\r\n  0.animation(0 14,0.13:4 0.0:4<3973 0.1:4 0.2:4 0.3:4 0.4:4 0.5:4 0.6:4 0.7:4 0.8:4 0.9:4! 0.10:4 0.11:4 0.12:4)\r\n  102.type(0)\r\n  104.attr(8)\r\n",
    "#5832 1 counter barrier\r\n  0.animation(0 17,0.16:4 0.0:2 0.1:2<3974 0.2:2 0.3:3 0.4:3 0.5:4 0.6:4 0.7:4 0.8:4 0.9:4 0.10:4 0.11:4 0.12:4 0.13:4 0.14:4! 0.15:4)\r\n  102.type(0)\r\n  104.attr(8)\r\n",
    "#10710 1 counter barrier hit\r\n      0.animation(0 6,0.5:3 0.0:1 0.1:2 0.2:3 0.3:3 0.4:4!)\r\n      102.type(0)\r\n      104.attr(8)\r\n",
];
fn original() -> String {
    format!("{} 0 0\r\n{}", ORIGINAL.len(), ORIGINAL.concat())
}
fn catalog(text: &str) -> SpriteActionCatalog {
    SpriteActionCatalog::parse_for_inspection(text.as_bytes(), ActionLimits::default()).unwrap()
}
/// No-delay model of the native callbacks: every source frame lasts
/// `floor(ticks × 1000 / 24)` ms; playback starts at source 1 and source 0 is
/// the terminal dwell.
fn millis(a: &Animation, frames: std::ops::Range<usize>) -> u64 {
    a.frames[frames].iter().map(|f| u64::from(f.duration_ticks) * 1000 / 24).sum()
}
/// (source frame of the single audio cue, its overlap flag, its sound).
fn audio(a: &Animation) -> Vec<(usize, bool, u32)> {
    a.frames.iter().enumerate().flat_map(|(n, f)| f.events.iter().filter_map(move |e| match e {
        FrameEvent::Effect(id) => Some((n, true, *id)),
        FrameEvent::Sound(id) => Some((n, false, *id)),
        _ => None,
    })).collect()
}
fn hits(a: &Animation) -> Vec<usize> {
    a.frames.iter().enumerate().filter(|(_, f)| f.events.contains(&FrameEvent::Hit)).map(|(n, _)| n).collect()
}

#[test]
fn original_definitions_qualify_exactly_with_their_cue_and_end_times() {
    let c = catalog(&original());
    // gfx, frames, audio (source frame, overlap `<`, sound), cue ms, visual end ms.
    // The five rows with a 109 row are refused by the generic received-finite
    // qualifier and admitted only through their exact pinned definition.
    for (gfx, frames, cue, cue_ms, end_ms) in [
        (3932u32, 13usize, (2usize, true, 1349u32), 166u64, 2451u64),
        (3935, 19, (1, true, 1356), 0, 2076),
        (4434, 6, (2, false, 1498), 41, 246),
        (4824, 19, (3, true, 3089), 82, 2366),
        (4648, 22, (4, true, 1924), 123, 2281),
    ] {
        assert!(qualify_finite(&c, gfx).is_err(), "{gfx}: generic qualifier must refuse the 109 row");
        let mut rows = qualify_batch6(&c, gfx).expect("pinned").unwrap();
        assert_eq!(rows.len(), 1, "{gfx}");
        let (id, action, a) = rows.remove(0);
        assert_eq!((id, action, a.frames.len(), a.frames_per_second), (gfx, 0, frames, 24), "{gfx}");
        assert_eq!(a.frames[0].subframe as usize, frames - 1, "{gfx}: source 0 is the last image");
        assert!(a.frames[1..].iter().enumerate().all(|(n, f)| f.subframe as usize == n && f.image_id == 0), "{gfx}");
        assert_eq!(audio(&a), vec![cue], "{gfx}");
        assert!(hits(&a).is_empty(), "{gfx}");
        assert_eq!((millis(&a, 1..cue.0), millis(&a, 0..frames)), (cue_ms, end_ms), "{gfx}");
    }
    // 90／91 and the counter-barrier trigger carry no 109 row: the generic
    // qualifier already admits them, Hit frames included (diagnostic only).
    for (gfx, frames, cue, cue_ms, hit, hit_ms, end_ms) in [
        (5831u32, 14usize, Some((1usize, true, 3973u32)), 0u64, 10usize, 1494u64, 2324u64),
        (5832, 17, Some((2, true, 3974)), 83, 15, 1993, 2491),
        (10710, 6, None, 0, 5, 374, 665),
    ] {
        assert!(qualify_batch6(&c, gfx).is_none(), "{gfx}");
        let a = qualify_finite(&c, gfx).unwrap().remove(0).2;
        assert_eq!(a.frames.len(), frames, "{gfx}");
        assert_eq!(audio(&a), cue.into_iter().collect::<Vec<_>>(), "{gfx}");
        assert_eq!(hits(&a), vec![hit], "{gfx}");
        if let Some(cue) = cue { assert_eq!(millis(&a, 1..cue.0), cue_ms, "{gfx}"); }
        assert_eq!((millis(&a, 1..hit), millis(&a, 0..frames)), (hit_ms, end_ms), "{gfx}");
    }
    assert!(qualify_batch6(&c, 3933).is_none() && qualify_batch6(&c, 4183).is_none() && qualify_batch6(&c, 230).is_none());
}

#[test]
fn changed_original_definitions_are_refused_not_approximated() {
    let text = original();
    let all = [3932u32, 3935, 4434, 4824, 4648];
    // (replacement, graphics that must now be refused); every other pinned row
    // must still qualify, so one changed row never disables the others.
    let cases: [(&str, &str, &[u32]); 14] = [
        ("109.effect(1 -1)", "109.effect(0 -1)", &[3932, 4434]),
        ("109.effect(0 -1)", "109.effect(1 -1)", &[3935, 4824, 4648]),
        ("104.attr(8)", "104.attr(0)", &all),
        ("#3932 1 mass slow", "#3932 1 mass slow2", &[3932]),
        ("<1349", "<1350", &[3932]),
        ("0.11:7)", "0.11:6)", &[3932]),
        ("<1356", "[1356", &[3935]),
        ("0.17:5)", "0.17:4)", &[3935]),
        ("[1498", "<1498", &[4434]),
        ("0.5:1 0.0:1", "0.5:2 0.0:1", &[4434]),
        ("<3089", "<3090", &[4824]),
        ("0.18:4 0.0:1 0.1:1 0.2:1<3089", "0.18:4 0.0:1 0.1:1<3089 0.2:1", &[4824]),
        ("<1924", "<1925", &[4648]),
        ("0.20:4)", "0.20:5)", &[4648]),
    ];
    for (from, to, refused) in cases {
        assert!(text.contains(from), "{from}");
        let c = catalog(&text.replace(from, to));
        for gfx in all {
            let result = qualify_batch6(&c, gfx).expect("pinned");
            assert_eq!(result.is_err(), refused.contains(&gfx), "{from} -> {to}: {gfx}");
        }
    }
    // A missing row is an error for that graphic, never a fallback to another.
    let c = catalog(&format!("{} 0 0\r\n{}", ORIGINAL.len() - 1, ORIGINAL[1..].concat()));
    assert!(qualify_batch6(&c, 3932).expect("pinned").is_err());
    assert!(qualify_batch6(&c, 3935).expect("pinned").is_ok());
}
