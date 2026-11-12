//! Issue #149: original-resource memory rendering for T027 batch 5; no game,
//! service, GUI or sound device. Packet/model cases live in the library
//! (`native_skills::t027_batch5_tests` and the controller cases under
//! `native_login::teleport_wire_tests::t027_batch5`) so CI runs them.
use lito_modern::modern_lito::{
    native_login::LoginController,
    native_skill_effects::Effects,
    native_world_effects,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
const ITEM: u32 = 0xfedcba98;
fn fixture(data: &[u8], key: &str) -> Vec<u8> {
    let value: serde_json::Value = serde_json::from_slice(data).unwrap();
    let text = value["packets"][key].as_str().unwrap();
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap()).collect()
}
fn world() -> lito_modern::modern_lito::native_world::World {
    let mut w = LoginController::default().snapshot().world.clone();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&fixture(include_bytes!("fixtures/native-world-current.json"), key)).unwrap();
    }
    let mut own = w.actors.remove(&ITEM).unwrap();
    own.id = 1;
    own.x = 32770;
    own.y = 32818;
    w.actors.insert(1, own);
    w.status.as_mut().unwrap().object_id = 1;
    let actor = w.actors.get_mut(&2).unwrap();
    actor.x = 32771;
    actor.y = 32818;
    w.hp = Some([50, 50]);
    w.mp = Some([100, 100]);
    w
}
fn view() -> ViewProjection {
    ViewProjection { anchor_cell: WorldCell { x: 32770, y: 32818 }, anchor_pixel: PixelPoint { x: 400, y: 300 } }
}

/// 68 immune-to-harm 228, 73 create-magical-weapon 763, 71 decay-potion 2232 and
/// 69 mass-teleport 2236 (all Screen, already reviewed exact definitions): each
/// draws real pixels at its receipt cell, prepares exactly its source-1 sound
/// once and retires at its visual end. None of them is a status, enchant or
/// teleport acknowledgement.
#[test]
#[ignore = "original resource memory rendering; no GUI or sound device"]
fn original_batch5_object_effects_use_exact_banks_blends_and_sounds() {
    let root = std::path::Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = std::sync::atomic::AtomicBool::new(false);
    for (gfx, sound, end_ms) in [(228u16, 132u32, 830u64), (763, 138, 1909), (2232, 838, 1952), (2236, 836, 455)] {
        let mut w = world();
        let mut b = vec![0x37];
        b.extend(2u32.to_le_bytes());
        b.extend(gfx.to_le_bytes());
        w.accept(&b).unwrap();
        let mut fx = Effects::default();
        fx.prepare(root, &w, &cancel);
        let mut audio = Vec::new();
        let mut drawn = false;
        let mut finished_at = None;
        for step in 0..=300u64 {
            let now = step * 10_000_000;
            let mut fragments = Vec::new();
            let (r, e) = fx.draw_to(&w, &mut native_world_effects::Target::Fragments(&mut fragments), view(), now);
            assert!(r["load_error"].is_null(), "{gfx}: {r}");
            assert_eq!(r["omitted"], 0, "{gfx}: {r}");
            for f in &fragments {
                assert_eq!(f.position, WorldCell { x: 32771, y: 32818 }, "{gfx}");
                assert!(matches!(f.mix, native_world_effects::Mix::Screen), "{gfx}");
            }
            drawn |= !fragments.is_empty();
            audio.extend(e.into_iter().filter_map(|e| e.sound));
            if finished_at.is_none() && r["active"] == 0 { finished_at = Some(step * 10); }
        }
        assert!(drawn, "{gfx}");
        assert_eq!(audio, vec![sound], "{gfx}");
        let at = finished_at.expect("finite effect retires");
        assert!(at >= end_ms && at < end_ms + 20, "{gfx}: retired at {at}ms, visual end {end_ms}ms");
    }
}
