//! #165 (T028-3) skill 63 Life Stream current-version re-qualification.
//!
//! E149 source (read-only `javap -c` of formal-server-E1498871.jar,
//! `BattleSpellCatalog.<clinit>` GROUND_SPELLS): `GroundSpell(63, 7, 6, 19,
//! 2231, -1, 20, 10, 320, 0, 3, 1, 40318L, 1, "life-stream")`, i.e. row7 /
//! column6, action19, gfx2231, unlimited range, MP20, 320 s lease, pulse 0,
//! radius 3, one segment, reagent 40318x1. These equal the captured BBC59308
//! fixture, so the captured wire stays the current oracle. The 320 s lease is
//! server state: the client never owns a 320 s timer, animation or pulse.
use lito_modern::modern_lito::{
    compositor::Canvas,
    native_skill_effects::Effects,
    native_skills as skills,
    native_world::World,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::{path::PathBuf, sync::atomic::AtomicBool};

/// (row, column, action, gfx, mp, lease_seconds, pulse_ms) from E149.
const E149_LIFE_STREAM: (u8, u8, u8, u16, u16, u64, u64) = (7, 6, 19, 2231, 20, 320, 0);

fn root() -> PathBuf {
    std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\lineage開發\0-客戶端"))
}
fn fixture() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-reagent-skills-current.json")).unwrap()
}
fn hex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
fn p(key: &str) -> Vec<u8> {
    hex(fixture()["packets"][key].as_str().unwrap())
}
fn world() -> World {
    let mut w = World::default();
    let base: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&hex(base["packets"][key].as_str().unwrap())).unwrap();
    }
    let mut a = w.actors.remove(&0xfedcba98).unwrap();
    a.id = 1;
    a.x = 32729;
    a.y = 32797;
    w.actors.insert(1, a);
    w.status.as_mut().unwrap().object_id = 1;
    let t = w.actors.get_mut(&2).unwrap();
    t.x = 32735;
    t.y = 32800;
    w.hp = Some([50, 50]);
    w.mp = Some([80, 80]);
    w.accept(&p("book")).unwrap();
    w
}
fn projection() -> ViewProjection {
    ViewProjection { anchor_cell: WorldCell { x: 32735, y: 32800 }, anchor_pixel: PixelPoint { x: 400, y: 300 } }
}

#[test]
fn e149_life_stream_row_matches_captured_request_cast_effect_and_mana() {
    let (row, column, action, gfx, mp, lease, pulse) = E149_LIFE_STREAM;
    let w = world();
    let request = skills::ground_request(&w, 63, [32735, 32800]).unwrap();
    assert_eq!(request, p("request"));
    // C_USE_SPELL row/column, then unsigned x/y; skill id = row*8+column+1.
    assert_eq!((request[1], request[2]), (row, column));
    assert_eq!(u32::from(row) * 8 + u32::from(column) + 1, 63);
    assert_eq!(p("cast").last().copied(), Some(action));
    let effect = p("effect");
    assert_eq!(u16::from_le_bytes([effect[5], effect[6]]), gfx);
    let mana = p("mana");
    assert_eq!(80 - u16::from_le_bytes([mana[1], mana[2]]), mp);
    let lease_row = &fixture()["lease"];
    assert_eq!(lease_row["expires_ns"].as_u64(), Some(lease * 1_000_000_000));
    assert_eq!(lease_row["damage_pulses"].as_u64(), Some(pulse));
}

#[test]
fn overlapping_life_stream_effects_are_independent_server_events_without_client_lease() {
    let mut w = world();
    let before = serde_json::to_value(&w).unwrap();
    w.accept(&p("effect")).unwrap();
    w.accept(&p("effect")).unwrap();
    assert_eq!(w.effects.len(), 2);
    assert_ne!(w.effects[0].sequence, w.effects[1].sequence);
    for e in &w.effects {
        assert_eq!((e.graphics, e.target, e.position), (2231, None, [32735, 32800]));
    }
    // No HP/MP, item, cast or actor change is derived from the lease or its
    // radius; only the server's mana packet moves MP.
    let mut after = serde_json::to_value(&w).unwrap();
    for key in ["effects", "visual_revision"] {
        after[key] = before[key].clone();
    }
    assert_eq!(after, before);
    assert_eq!((w.hp, w.mp, w.own_cast_packets), (Some([50, 50]), Some([80, 80]), 0));
    // A new cast while a lease would still be active is not blocked locally.
    assert!(skills::ground_request(&w, 63, [32735, 32800]).is_ok());
}

#[test]
fn overlapping_life_streams_play_once_each_and_nothing_remains_at_320_seconds() {
    let (.., lease, _) = E149_LIFE_STREAM;
    let mut w = world();
    w.accept(&p("effect")).unwrap();
    let cancel = AtomicBool::new(false);
    let mut fx = Effects::reagent();
    fx.prepare(&root(), &w, &cancel);
    let mut c = Canvas::new(800, 600).unwrap();
    let mut sounds = 0;
    let mut sample = |w: &World, fx: &mut Effects, now: u64, sounds: &mut usize| {
        let (r, audio) = fx.draw(w, &mut c, projection(), now);
        assert!(r["load_error"].is_null(), "{r}");
        assert_eq!(r["omitted"], 0);
        for a in audio {
            assert_eq!((a.sound, a.action, a.frame), (Some(841), 3, 1));
            *sounds += 1;
        }
        r
    };
    let r = sample(&w, &mut fx, 0, &mut sounds);
    assert_eq!((r["started"].as_u64(), r["active"].as_u64()), (Some(1), Some(1)));
    // Second server effect 2 s later on the same cell: an independent run.
    w.accept(&p("effect")).unwrap();
    let r = sample(&w, &mut fx, 2_000_000_000, &mut sounds);
    assert_eq!((r["started"].as_u64(), r["active"].as_u64()), (Some(2), Some(2)));
    // 76 ticks @24 FPS = 3.1667 s after each start.
    let r = sample(&w, &mut fx, 3_100_000_000, &mut sounds);
    assert_eq!(r["active"].as_u64(), Some(2));
    let r = sample(&w, &mut fx, 3_300_000_000, &mut sounds);
    assert_eq!(r["active"].as_u64(), Some(1));
    let r = sample(&w, &mut fx, 5_100_000_000, &mut sounds);
    assert_eq!(r["active"].as_u64(), Some(1));
    let r = sample(&w, &mut fx, 5_300_000_000, &mut sounds);
    assert_eq!(r["active"].as_u64(), Some(0));
    // Lease boundary on the client clock: neither 319.999 s nor 320 s
    // restarts, removes, pulses or plays anything.
    for now in [lease * 1_000_000_000 - 1_000_000, lease * 1_000_000_000, (lease + 2) * 1_000_000_000] {
        let r = sample(&w, &mut fx, now, &mut sounds);
        assert_eq!((r["started"].as_u64(), r["active"].as_u64()), (Some(2), Some(0)));
    }
    assert_eq!(sounds, 2);
    assert_eq!(w.effects.len(), 2);
}
