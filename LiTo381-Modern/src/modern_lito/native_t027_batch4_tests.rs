//! Issue #148 (T027 batch 4: 55, 56, 59, 61, 62, 64, 65, 66). Offline packets,
//! models and original text definitions only; no game, service, GUI or sound device.
use super::*;
use crate::modern_lito::{
    native_attack_many as many, native_combat_animation::EnteredEvent, native_forced_combat,
    native_skill_selection::{Model, Target},
    native_world_effects,
    sprite_actions::{ActionLimits, Directionality, FrameEvent, SpriteActionCatalog},
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::time::Instant;

const IDS: [u16; 8] = [55, 56, 59, 61, 62, 64, 65, 66];

fn packet(file: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(file).unwrap();
    let s = v["packets"][key].as_str().unwrap();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
/// Same offline fixture as the earlier batches: own id 1 at 32770/32818, bot 2 one cell east.
fn world() -> World {
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&packet(include_bytes!("../../tests/fixtures/native-world-current.json"), key)).unwrap();
    }
    let mut own = w.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    own.x = 32770;
    own.y = 32818;
    w.actors.insert(1, own);
    w.status.as_mut().unwrap().object_id = 1;
    let bot = w.actors.get_mut(&2).unwrap();
    bot.x = 32771;
    bot.y = 32818;
    w.hp = Some([50, 50]);
    w.mp = Some([100, 100]);
    // Formal S1E learned-book bits: row6 cols6/7, row7 cols2/4/5/7, row8 cols0/1.
    w.accept(&[0x1e, 9, 0, 0, 0, 0, 0, 0, 0xc0, 0xb4, 0x03]).unwrap();
    w
}
fn corpse(mut w: World) -> World {
    w.actors.get_mut(&2).unwrap().dead = true;
    w
}
fn cast(w: &World, id: u16) -> Result<Vec<u8>> {
    match id {
        59 | 62 => request(w, id, None),
        _ => request(w, id, Some(2)),
    }
}
fn ready(id: u16) -> World {
    if id == 61 { corpse(world()) } else { world() }
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
    let mut w = world();
    assert_eq!(w.learned_skills.iter().copied().collect::<Vec<_>>(), IDS);
    // HP/MP of one: MP40／60／50, HP10, gems and probabilities stay server-owned.
    w.hp = Some([1, 50]);
    w.mp = Some([1, 100]);
    let xy = [0x03, 0x80, 0x32, 0x80];
    for (id, prefix) in [(55, [0x14, 6, 6]), (56, [0x14, 6, 7]), (59, [0x14, 7, 2]), (61, [0x14, 7, 4]),
        (62, [0x14, 7, 5]), (64, [0x14, 7, 7]), (65, [0x14, 8, 0]), (66, [0x14, 8, 1])] {
        let w = if id == 61 { corpse(w.clone()) } else { w.clone() };
        let before = serde_json::to_value(&w).unwrap();
        let mut expected = prefix.to_vec();
        if !matches!(id, 59 | 62) { expected.extend(2u32.to_le_bytes()); }
        if matches!(id, 56 | 64 | 65 | 66) { expected.extend(xy); }
        let body = cast(&w, id).unwrap();
        assert_eq!(body, expected, "{id}");
        assert_eq!(body.len(), match id { 59 | 62 => 3, 55 | 61 => 7, _ => 11 }, "{id}");
        assert_eq!(serde_json::to_value(&w).unwrap(), before, "{id} mutated world");
        // Ctrl/LFC1 only ever extends object-target bodies; 59/62 have no actor.
        assert_eq!(native_forced_combat::object_target(&body), (!matches!(id, 59 | 62)).then_some(2), "{id}");
    }
    // 55 without a selection is an explicit self cast; a stale ID never falls back.
    assert_eq!(request(&w, 55, None).unwrap()[3..], 1u32.to_le_bytes());
    assert!(request(&w, 55, Some(77)).is_err());
    for id in [59, 62] { assert!(request(&w, id, Some(2)).is_err(), "{id}"); assert!(selector_only(id)); }
    for id in [56, 61, 64, 65, 66] {
        assert!(request(&w, id, None).is_err(), "{id} needs an explicit target");
        assert!(request(&w, id, Some(1)).is_err(), "{id} never targets the caster");
        assert!(!selector_only(id));
    }
    assert_eq!(IDS.map(reuse_millis), [10, 500, 1500, 10, 1500, 500, 2000, 1500]);
    assert!(IDS.iter().all(|&id| client_clock_millis(id) == 0));
}

#[test]
fn caster_gates_reject_without_partial_mutation() {
    for id in IDS {
        for reason in 0..7 {
            let mut w = ready(id);
            match reason {
                0 => { w.learned_skills.remove(&id); }
                1 => w.hp = Some([0, 50]),
                2 => w.mp = None,
                3 => w.mp = Some([0, 100]),
                4 => w.actors.get_mut(&1).unwrap().dead = true,
                5 => { w.movement_blocks.insert(1); }
                _ => { w.actors.remove(&1); }
            }
            let before = serde_json::to_value(&w).unwrap();
            assert!(cast(&w, id).is_err(), "{id}/{reason}");
            assert_eq!(serde_json::to_value(&w).unwrap(), before, "{id}/{reason}");
        }
    }
}

#[test]
fn target_kind_and_range_follow_the_formal_record() {
    // Living-actor skills refuse corpses and absent actors; 61 refuses the living.
    for id in [55, 56, 64, 65, 66] {
        assert!(cast(&corpse(world()), id).is_err(), "{id} on corpse");
        let mut w = world();
        w.actors.remove(&2);
        assert!(cast(&w, id).is_err(), "{id} absent");
    }
    assert!(cast(&world(), 61).is_err(), "61 on a living actor");
    let mut w = corpse(world());
    w.actors.remove(&2);
    assert!(cast(&w, 61).is_err());
    // Chebyshev ranges including the diagonal edge.
    for (id, range) in [(56, 3), (64, 3), (65, 10), (66, 10)] {
        let mut w = world();
        let a = w.actors.get_mut(&2).unwrap();
        a.x = 32770 + range;
        a.y = 32818 + range;
        assert!(cast(&w, id).is_ok(), "{id} at {range}");
        w.actors.get_mut(&2).unwrap().x += 1;
        assert!(cast(&w, id).is_err(), "{id} beyond {range}");
    }
    // 55 and 61 use the native screen diamond, not an unlimited range.
    for id in [55, 61] {
        let mut w = ready(id);
        w.actors.get_mut(&2).unwrap().x = 32770 + 18;
        assert!(cast(&w, id).is_ok(), "{id} at 18");
        w.actors.get_mut(&2).unwrap().x = 32770 + 20;
        assert!(cast(&w, id).is_err(), "{id} at 20");
    }
}

#[test]
fn menu_labels_and_corpse_intent_cannot_be_upgraded() {
    let w = world();
    let mut menu = Model::default();
    menu.sync(7, &w, true);
    assert_eq!(menu.rows().iter().map(|r| r.id).collect::<Vec<_>>(), IDS);
    assert!(menu.rows().iter().all(|r| !r.label.contains("尚未支援")), "{:?}", menu.rows());
    // A living selection never becomes a 61 corpse authorization after death…
    let living = Target::capture(&w, 2).unwrap();
    menu.select(Some(61)).unwrap();
    assert!(menu.choice_at(7, &w, Some(living), None).is_err());
    let dead = corpse(w.clone());
    assert!(menu.choice_at(7, &dead, Some(living), None).is_err());
    // …a captured corpse authorizes only that corpse, and never a revived actor.
    let body = Target::capture(&dead, 2).unwrap();
    assert_eq!(menu.choice_at(7, &dead, Some(body), None).unwrap().target, Some(2));
    let mut revived = dead.clone();
    revived.actors.get_mut(&2).unwrap().dead = false;
    assert!(menu.choice_at(7, &revived, Some(body), None).is_err());
    menu.select(Some(64)).unwrap();
    assert!(menu.choice_at(7, &dead, Some(body), None).is_err(), "corpse cannot become a living target");
    // Selector-only 59/62 drop a hovered actor instead of sending it.
    for id in [59, 62] {
        menu.select(Some(id)).unwrap();
        let choice = menu.choice_at(7, &w, Some(living), None).unwrap();
        assert_eq!(choice.target, None, "{id}");
    }
    menu.select(Some(66)).unwrap();
    let mut moved = w.clone();
    moved.actors.get_mut(&2).unwrap().incarnation += 1;
    assert!(menu.choice_at(7, &moved, Some(living), None).is_err(), "stale incarnation");
    assert!(menu.choice_at(8, &w, Some(living), None).is_err(), "other connection");
}

#[test]
fn object_effects_are_atomic_and_never_grant_status_cost_or_life() {
    // 55 3943, 56 2230, 64 2177, 66 760, 61 cast 3944 and success light 230.
    for (id, gfx) in [(2, 3943), (2, 2230), (2, 2177), (2, 760), (2, 3944), (2, 230), (1, 3943)] {
        let original = if gfx == 3944 { corpse(world()) } else { world() };
        let b = effect(id, gfx);
        for n in 0..b.len() {
            let mut w = original.clone();
            let before = serde_json::to_value(&w).unwrap();
            assert!(w.accept(&b[..n]).is_err(), "{gfx} prefix {n}");
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
        }
        let mut w = original.clone();
        w.accept(&b).unwrap();
        assert_eq!(w.effects.len(), original.effects.len() + 1);
        let cue = w.effects.last().unwrap();
        assert_eq!((cue.graphics, cue.target.map(|t| t.0)), (gfx, Some(id)));
        assert_eq!(cue.position, [w.actors[&id].x, w.actors[&id].y], "frozen receipt position");
        assert_eq!((w.hp, w.mp), (original.hp, original.mp));
        assert_eq!(w.movement_blocks, original.movement_blocks);
        assert_eq!(w.actors[&2].dead, original.actors[&2].dead, "3944/230 are not a revival");
        assert_eq!(w.own_cast_packets, original.own_cast_packets, "S37 is not a cast ACK");
    }
}

#[test]
fn centered_and_area_s39_keep_order_misses_and_the_formal_profile_only() {
    let targets = [(2u32, 0x20u8), (3, 0)];
    for (gfx, kind) in [(757u16, 0u8), (1812, 0), (3924, 8)] {
        let original = world();
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
        // 59/62 are centered on the frozen wire caster origin; 65 freezes its first target.
        assert_eq!(cue.target_origin, (gfx == 3924).then_some([32771, 32818]));
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
    // 3924: a missed first target still owns the origin; an absent first target
    // falls back one heading step and never borrows the second target.
    let mut w = world();
    w.accept(&s39(3924, 8, 4, &[(2, 0), (99, 0x20)])).unwrap();
    assert_eq!(w.many_attacks[0].target_origin, Some([32771, 32818]));
    let mut w = world();
    w.accept(&s39(3924, 8, 4, &[(99, 0x20), (2, 0x20)])).unwrap();
    assert_eq!(w.many_attacks[0].target_origin, Some([32770, 32819]));
    // Later movement of the first target does not move an accepted origin.
    let mut w = world();
    w.accept(&s39(3924, 8, 4, &[(2, 0x20)])).unwrap();
    w.actors.get_mut(&2).unwrap().x += 3;
    assert_eq!(w.many_attacks[0].target_origin, Some([32771, 32818]));
}

#[test]
fn unqualified_presentations_are_omitted_not_substituted() {
    // 3943/3944 (eight-bank) and 757/1812/3924 have no qualified presentation yet.
    let mut w = world();
    w.effects.clear();
    for gfx in [3943, 3944] { w.accept(&effect(2, gfx)).unwrap(); }
    let mut fx = super::super::native_skill_effects::Effects::default();
    let mut fragments = Vec::new();
    let (r, events) = fx.draw_to(&w, &mut native_world_effects::Target::Fragments(&mut fragments), view(), 0);
    assert!(fragments.is_empty() && events.is_empty(), "{r}");
    assert_eq!((r["started"].as_u64(), r["omitted"].as_u64()), (Some(0), Some(2)), "{r}");
    for gfx in [757u16, 1812, 3924] {
        let mut w = world();
        w.accept(&s39(gfx, if gfx == 3924 { 8 } else { 0 }, 2, &[(2, 0x20)])).unwrap();
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
    for code in [278, 279, 280, 299] {
        let mut w = world();
        let now = Instant::now();
        let p = pending(&w, now);
        message(&mut w, code);
        assert_eq!(p.observe(&w, now), Feedback::Rejected(code));
    }
    let w = world();
    let now = Instant::now();
    let p = pending(&w, now);
    assert_eq!(p.observe(&w, now), Feedback::Pending);
    assert_eq!(p.observe(&w, now + std::time::Duration::from_secs(2)), Feedback::NoResponse);
}

/// The exact original `list.spr` rows (row15937, SHA256 523C61AD…), text only.
const ROWS: &[u8] = b"#230  1       lfs orange          0.Breath(0 7,0.0:4 0.1:2[124 0.2:2 0.3:4 0.4:4 0.5:2 0.6:2) 104.attr(8)\n#760  1       fog of sleeping     0.fire(0 21,0.20:6 0.0:6[136 0.1:6 0.2:6 0.3:6 0.4:6 0.5:6 0.6:6 0.7:6 0.8:6 0.9:6 0.10:6 0.11:6 0.12:6 0.13:6 0.14:6 0.15:6 0.16:6 0.17:6 0.18:6 0.19:6) 104.attr(8)\n#2230 1   disease m\t\t  0.animation(0 11,0.10:3 0.0:3[839 0.1:3 0.2:3 0.3:3 0.4:3 0.5:3 0.6:3 0.7:3 0.8:3 0.9:3) 102.type(0) 104.attr(128)\n";
/// A parse failure of a changed row is also a rejection (never a fallback).
fn qualify(rows: &[u8]) -> std::result::Result<Vec<(u32, u8, super::super::sprite_actions::Animation)>, String> {
    let c = SpriteActionCatalog::parse_for_inspection(&[b"11297 0 0\n".as_slice(), rows].concat(), ActionLimits::default())
        .map_err(|e| format!("parse: {e}"))?;
    super::super::native_additional_finite_effects::qualify_batch4(&c).map_err(|e| format!("qualify: {e:?}"))
}

#[test]
fn batch4_finite_definitions_are_exact_and_visual_only() {
    let qualified = qualify(ROWS).unwrap_or_else(|e| panic!("original rows: {e}"));
    assert_eq!(qualified.iter().map(|q| q.0).collect::<Vec<_>>(), [2230, 760, 230]);
    for (gfx, frames, sound, end_ms) in [(2230u32, 11usize, 839u32, 1375u64), (760, 21, 136, 5250), (230, 7, 124, 830)] {
        let a = &qualified.iter().find(|q| q.0 == gfx).unwrap().2;
        assert_eq!((a.directionality, a.frames.len()), (Directionality::Undirected, frames), "{gfx}");
        assert_eq!(a.frames[1].events, vec![FrameEvent::Sound(sound)], "{gfx}: sound on source1");
        assert!(a.frames.iter().enumerate().all(|(n, f)| n == 1 || f.events.is_empty()), "{gfx}");
        // Integer native callbacks; the visual end is never a 64s/32s status lifetime.
        let clip = super::super::native_effect_timing::Clip::new(std::sync::Arc::new(a.clone())).unwrap();
        let mut p = clip.start();
        let mut s = p.advance(0).unwrap();
        let mut at = 0;
        while !s.finished {
            at = p.next_callback_millis().unwrap();
            s = p.advance(at * 1_000_000).unwrap();
        }
        assert_eq!(at, end_ms, "{gfx}");
    }
    let text = std::str::from_utf8(ROWS).unwrap();
    for (from, to) in [("[839", "[843"), ("104.attr(128)", "104.attr(8)"), ("[136", "<136"), ("0.10:6", "0.10:5"),
        ("[124", "[122"), ("#760  1", "#760  8")] {
        assert!(text.contains(from), "{from}");
        assert!(qualify(text.replacen(from, to, 1).as_bytes()).is_err(), "{from} -> {to} must not qualify");
    }
    // The earlier reviewed specs never contain the batch-4 graphics (separate bank).
    for gfx in super::super::native_additional_finite_effects::BATCH4_GRAPHICS {
        assert!(!super::super::native_additional_finite_effects::GRAPHICS.contains(gfx));
    }
}
