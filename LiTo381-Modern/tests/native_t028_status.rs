//! T028-2 (#164): status/self skills 2 (light), 13 (detection), 60 (invisibility).
//! Offline only: fixture packets, no server, no GUI, no shared resources.
//! Server facts are read from E149 source (BattleSpellCatalog LIGHT 3600 s /
//! range 0x0E; broadcastLightState on expiry) and are *not* simulated here.
use lito_modern::modern_lito::{native_skill_selection::{self, Target}, native_skills as skills,
    native_world::World};

fn packet(data: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(data).unwrap();
    let hex = v["packets"][key].as_str().unwrap();
    (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect()
}
fn world() -> World {
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&packet(include_bytes!("fixtures/native-world-current.json"), key)).unwrap();
    }
    let mut own = w.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    w.actors.insert(1, own);
    w.status.as_mut().unwrap().object_id = 1;
    w.accept(&packet(include_bytes!("fixtures/native-skills-current.json"), "mana")).unwrap();
    for skill in [1, 2, 13, 60] { w.learned_skills.insert(skill); }
    w
}
fn change_light(id: u32, range: u8) -> Vec<u8> {
    let mut p = vec![0x1b];
    p.extend(id.to_le_bytes());
    p.push(range);
    p
}
fn draw_state(id: u32, invisible: u8) -> Vec<u8> {
    let mut p = vec![0x34];
    p.extend(id.to_le_bytes());
    p.push(invisible);
    p
}
fn json(w: &World) -> serde_json::Value { serde_json::to_value(w).unwrap() }

/// Skill 2: the request carries no duration; the client never times the light
/// out. Only S_CHANGE_LIGHT (0x1B) moves the range, for own and remote casters,
/// including the server's natural-expiry rebroadcast (0, or an item range).
#[test]
fn light_has_no_client_duration_and_only_server_change_light_moves_range() {
    let mut w = world();
    let before = json(&w);
    assert_eq!(skills::request(&w, 2, None).unwrap(), [0x14, 0, 1]);
    assert!(skills::request(&w, 2, Some(2)).is_err());
    assert_eq!(json(&w), before, "queueing light must not predict MP or light");
    // E149 LightSpell lightRange 0x0E for the caster; remote caster as viewer.
    for (id, range) in [(1u32, 0x0eu8), (2, 0x0e)] {
        w.accept(&change_light(id, range)).unwrap();
        assert_eq!(w.actors[&id].light, range);
    }
    let lit = json(&w);
    // There is no client-side 3600 s lease: re-reading state cannot change it.
    assert_eq!(json(&w), lit);
    assert_eq!(w.actors[&1].light, 0x0e);
    // Server expiry: broadcastLightState sends max(item range, 0).
    for (id, range) in [(1u32, 0u8), (2, 8)] {
        w.accept(&change_light(id, range)).unwrap();
        assert_eq!(w.actors[&id].light, range);
    }
    // Re-cast after expiry is still the same pure request.
    assert_eq!(skills::request(&w, 2, None).unwrap(), [0x14, 0, 1]);
    // An orphan update for an absent actor never creates a light source.
    let orphans = w.orphan_updates;
    w.accept(&change_light(999, 0x0e)).unwrap();
    assert_eq!(w.orphan_updates, orphans + 1);
    assert!(!w.actors.contains_key(&999));
}

/// Skill 2 night brightness inputs: the original 4D02F0 curve is dark at
/// midnight and full at noon, and the actor raw light (signed byte) selects
/// the source.lmp stencil range used by the dark-world mask pass.
#[test]
fn light_range_feeds_night_source_selection_with_original_signed_rules() {
    use lito_modern::modern_lito::{native_environment::{daylight, map_mask_argument},
        native_light_sources::{source_range, Actor, Master, Policy}};
    let night = daylight(0, 0, [0; 3]).unwrap();
    let noon = daylight(0, 12 * 60, [0; 3]).unwrap();
    assert_eq!((night.level, noon.level), (6, 15));
    assert_eq!(daylight(0, 12 * 60, [1, 0, 0]).unwrap().level, 14);
    // Ordinary map: native full-bright bypass; flagged dark map: masked pass.
    assert_eq!(map_mask_argument(4, 0), 15);
    assert_eq!(map_mask_argument(307, 1), 2);
    let actor = |raw_light: u8| Actor { position: [0, 0], raw_light, action: 0, walk_action: 0,
        direction: 0, source_frame: 0, frame_count: 0, base_class: 1, subtype: 0, flag_1f: false,
        flag_50: false, mode_23: 0, master: Master::None };
    let dark = Policy { ambient: u32::from(night.level), underwater: false, blind: 0,
        own_index: Some(0), own_position: [0, 0] };
    assert_eq!(source_range(dark, &actor(0x0e), 0), Some(14));
    assert_eq!(source_range(dark, &actor(0x0e), 1), Some(14), "remote caster light also counts");
    assert_eq!(source_range(dark, &actor(0), 0), None, "expired light has no stencil");
    assert_eq!(source_range(dark, &actor(8), 0), Some(8), "item range after expiry");
    assert_eq!(source_range(dark, &actor(40), 0), Some(15), "range clamps at 15");
    assert_eq!(source_range(dark, &actor(0x80), 0), None, "signed byte: negative is off");
    // Blindness overrides the spell range for the own source only.
    let blind = Policy { blind: 1, ..dark };
    assert_eq!(source_range(blind, &actor(0x0e), 0), Some(5));
    assert_eq!(source_range(blind, &actor(0x0e), 1), None);
}

/// Skill 13: the request is selector-only; revealing is server-authored via
/// S34 invisible=0. A selection captured before the actor hid stays retired
/// after the reveal, and a new capture is required.
#[test]
fn detection_reveal_restores_hidden_actor_and_retires_pre_hide_selection() {
    let mut w = world();
    let old = Target::capture(&w, 2).unwrap();
    assert_eq!(old.resolve(&w), Some(2));
    w.accept(&draw_state(2, 1)).unwrap();
    assert!(!w.actor_visible(2));
    assert!(Target::capture(&w, 2).is_none());
    assert_eq!(old.resolve(&w), None);
    assert!(skills::request(&w, 1, Some(2)).is_err(), "hidden actor is not a heal target");
    let hidden = json(&w);
    assert_eq!(skills::request(&w, 13, None).unwrap(), [0x14, 1, 4]);
    assert!(skills::request(&w, 13, Some(2)).is_err());
    assert_eq!(native_skill_selection::choice(&w, 13, Some(old)).unwrap().target, None);
    assert_eq!(json(&w), hidden, "detection must not reveal locally");
    // Server reply: effect 749 on the caster, then S34 reveal of the target.
    w.accept(&[0x37, 1, 0, 0, 0, 0xed, 2]).unwrap();
    assert!(!w.actor_visible(2));
    w.accept(&draw_state(2, 0)).unwrap();
    assert!(w.actor_visible(2));
    assert_eq!(old.resolve(&w), None, "pre-hide selection cannot silently revive");
    let fresh = Target::capture(&w, 2).unwrap();
    assert_eq!(fresh.resolve(&w), Some(2));
    assert_eq!(skills::request(&w, 1, Some(2)).unwrap(), [0x14, 0, 0, 2, 0, 0, 0]);
    // Malformed S34 never partially changes visibility.
    let before = json(&w);
    let bad = draw_state(2, 2);
    assert!(w.accept(&bad).is_err());
    for end in 0..bad.len() { assert!(w.accept(&bad[..end]).is_err()); }
    let mut long = draw_state(2, 1);
    long.push(0);
    assert!(w.accept(&long).is_err());
    assert_eq!(json(&w), before);
}

/// Skill 60: own invisibility is server-applied (S34 on self). The caster
/// stays locally visible (half style), its own selection is not retired, and a
/// repeat cast is still the self/tick envelope with no target.
#[test]
fn invisibility_self_state_is_server_applied_and_cast_stays_self_tick() {
    let _controller = lito_modern::modern_lito::native_login::LoginController::default();
    let mut w = world();
    let own_revision = w.actors[&1].target_revision;
    let before = json(&w);
    let start = skills::ClientTick::now().unwrap();
    let sent = skills::request(&w, 60, None).unwrap();
    let end = skills::ClientTick::now().unwrap();
    assert_eq!((&sent[..3], sent.len()), (&[0x14u8, 7, 3][..], 7));
    let tick = u32::from_le_bytes(sent[3..].try_into().unwrap());
    assert!(tick.wrapping_sub(start.as_millis()) <= end.as_millis().wrapping_sub(start.as_millis()));
    assert_eq!(json(&w), before, "queueing invisibility must not hide locally");
    w.accept(&draw_state(1, 1)).unwrap();
    assert!(w.actors[&1].draw_state.invisible);
    assert!(w.actor_visible(1), "own invisible caster remains drawn");
    assert_eq!(w.actors[&1].target_revision, own_revision);
    assert!(skills::request(&w, 60, Some(2)).is_err());
    assert_eq!(&skills::request(&w, 60, None).unwrap()[..3], [0x14, 7, 3]);
    // Remote viewers: the other actor hiding is a hide, not a half style.
    w.accept(&draw_state(2, 1)).unwrap();
    assert!(!w.actor_visible(2));
    w.accept(&draw_state(1, 0)).unwrap();
    assert!(!w.actors[&1].draw_state.invisible);
}
