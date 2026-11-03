use lito_modern::modern_lito::{
    native_combat as combat, native_skills as skills, native_world::World,
};
fn packet(bytes: &[u8], key: &str) -> Vec<u8> {
    let d: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let s = d["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn p(key: &str) -> Vec<u8> {
    packet(
        include_bytes!("fixtures/native-fire-arrow-current.json"),
        key,
    )
}
fn world() -> World {
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&packet(
            include_bytes!("fixtures/native-world-current.json"),
            key,
        ))
        .unwrap();
    }
    let mut own = w.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    own.x = 32729;
    own.y = 32797;
    w.actors.insert(1, own);
    w.status.as_mut().unwrap().object_id = 1;
    let t = w.actors.get_mut(&2).unwrap();
    t.x = 32735;
    t.y = 32800;
    w.hp = Some([50, 50]);
    w.mp = Some([10, 10]);
    w.accept(&p("book")).unwrap();
    w
}
#[test]
fn formal_attack_spell_layout_and_request_guards() {
    spell_authority_is_atomic_and_does_not_preempt_arrival_hurt();
    class4_zero_distance_zero_damage_and_miss_keep_authority();
    let mut w = world();
    w.learned_skills.remove(&15);
    assert!(skills::request(&w,15,Some(2)).is_err());
    w.learned_skills.insert(15);
    w.actors.get_mut(&2).unwrap().dead=true;
    assert!(skills::request(&w,15,Some(2)).is_err());
    w.actors.get_mut(&2).unwrap().dead=false;
    assert!(skills::request(&w,15,Some(999)).is_err());
    assert_eq!(skills::request(&w, 15, Some(2)).unwrap(), p("request"));
    assert!(skills::request(&w, 15, None).is_err());
    assert!(skills::request(&w, 15, Some(1)).is_err());
    w.actors.get_mut(&2).unwrap().x = 32740;
    assert!(skills::request(&w, 15, Some(2)).is_err());
    let a = combat::decode_attack(&p("hit")).unwrap().unwrap();
    let shot = a.projectile.unwrap();
    assert_eq!((a.action, shot.flight_type, shot.graphics), (18, 0, 1583));
    let a = combat::decode_attack(&p("boundary")).unwrap().unwrap();
    assert_eq!(a.attacker, u32::MAX);
    assert_eq!(a.projectile.unwrap().id, u32::MAX);
    for key in ["hit", "miss", "zero", "adjacent"] {
        assert!(combat::decode_attack(&p(key)).unwrap().is_some());
    }
}
fn spell_authority_is_atomic_and_does_not_preempt_arrival_hurt() {
    let mut w = world();
    let before = serde_json::to_value(&w).unwrap();
    let b = p("hit");
    for n in 0..b.len() {
        assert!(w.accept(&b[..n]).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    w.accept(&b).unwrap();
    assert_eq!(w.own_cast_packets, 1);
    assert_eq!(w.own_attack_packets, 0);
    assert!(w.actors[&2].actions.is_empty());
    assert_eq!(w.mp, Some([10, 10]));
    assert_eq!(w.hp, Some([50, 50]));
    let pending = skills::Pending {
        since: std::time::Instant::now(),
        casts: 0,
        messages: 0,
        generation: w.generation,
        incarnation: w.actors[&1].incarnation,
    };
    assert_eq!(
        pending.observe(&w, std::time::Instant::now()),
        skills::Feedback::CastAction
    );
}
#[test]
fn original_class4_flight_impact_audio_and_hurt_are_once() {
    use lito_modern::modern_lito::native_world_scene::Model;
    use std::{path::Path, sync::atomic::AtomicBool};
    let cancel = AtomicBool::new(false);
    let mut w = world();
    let mut model = Model::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    model.draw_at(&w, &cancel, 0).unwrap();
    w.accept(&p("hit")).unwrap();
    let authority = serde_json::to_value(&w).unwrap();
    let mut frames = Vec::new();
    let mut callbacks = Vec::new();
    let mut sound97 = 0;
    let mut sound99=0;
    let mut gfx = std::collections::BTreeSet::new();
    for ms in (0..=3000).step_by(50) {
        let f = model.draw_at(&w, &cancel, ms * 1_000_000).unwrap();
        let mut r = f.receipt;
        let s = &r["spell_projectiles"];
        assert!(s["load_error"].is_null());
        assert_eq!(s["omitted"], 0);
        for c in s["hurt_callbacks"].as_array().unwrap() {
            assert_eq!(c["accepted"], true);
            callbacks.push(ms);
        }
        for e in s["entered_events"].as_array().unwrap() {
            if e["sound"]==99 {sound99+=1;}
            if e["sound"] == 97 {
                sound97 += 1;
            }
        }
        for f in s["flights"].as_array().unwrap() {
            if f["drawn"] == true {
                gfx.insert(f["graphics"].as_u64().unwrap());
            }
        }
        for a in r["actors"].as_array().unwrap() {
            if a["id"] == 2 && a["phase"] == "damage" {
                assert_eq!(callbacks.len(), 1);
            }
        }
        if [0, 250, 500, 750, 1000, 1500, 2000, 2450, 3000].contains(&ms) {
            let p = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("target/current/verification/native-fire-arrow-{ms}.png"));
            let mut e = png::Encoder::new(std::fs::File::create(&p).unwrap(), 800, 600);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .unwrap()
                .write_image_data(f.canvas.rgba())
                .unwrap();
            r["png"] = serde_json::json!(p);
        }
        frames.push(r);
    }
    assert_eq!(callbacks.len(), 1);
    assert_eq!(sound97, 1);
    assert_eq!(sound99,1);
    assert_eq!(gfx, std::collections::BTreeSet::from([1583, 218]));
    assert_eq!(frames.last().unwrap()["spell_projectiles"]["active"], 0);
    assert_eq!(serde_json::to_value(&w).unwrap(), authority);
    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-fire-arrow.json"),
        serde_json::to_vec_pretty(&frames).unwrap(),
    )
    .unwrap();
}
#[test]
fn class4_launch_freezes_target_and_retires_reused_identity() {
    use lito_modern::modern_lito::{
        compositor::Canvas,
        native_combat_animation::EnteredEvent,
        native_spell_projectiles::Projectiles,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    let assets=lito_modern::modern_lito::light_arrow_effect::LightArrowAssets::load_class4_effect_only(
        std::path::Path::new(r"D:\lineage開發\0-客戶端"),1583,&std::sync::atomic::AtomicBool::new(false)).unwrap();
    assert_eq!(assets.sprite_sources().len(),5);
    let from=WorldCell{x:32729,y:32797};
    let mut headings=std::collections::BTreeSet::new();
    for (dx,dy) in [(0,-4),(4,-4),(4,0),(4,4),(0,4),(-4,4),(-4,0),(-4,-4)] {
        let path=lito_modern::modern_lito::light_arrow_effect::LightArrowPath::new(from,WorldCell{x:from.x+dx,y:from.y+dy}).unwrap();
        headings.insert(path.heading());
        let frame=assets.effect_frame(&path,std::time::Duration::ZERO).unwrap();
        assert_eq!(frame.graphics_id,1583);
        assert_eq!(assets.effect_frame(&path,path.impact_at()).unwrap().graphics_id,218);
        let mut playback=assets.start_effect(&path);
        let sample=playback.advance(assets.effect_duration(&path)).unwrap();
        assert!(sample.finished);
        for sound in [97,99] {assert_eq!(sample.crossed_cues.iter().filter(|c|c.event==lito_modern::modern_lito::sprite_actions::FrameEvent::Sound(sound)).count(),1);}
        assert!(playback.advance(assets.effect_duration(&path)).unwrap().crossed_cues.is_empty());
    }
    assert_eq!(headings.len(),8);
    let mut w = world();
    w.accept(&p("hit")).unwrap();
    let cue = w.projectiles[0].clone();
    let mut engine = Projectiles::default();
    engine.prepare(
        std::path::Path::new(r"D:\lineage開發\0-客戶端"),
        &w,
        &std::sync::atomic::AtomicBool::new(false),
    );
    let e = EnteredEvent {
        actor: 1,
        sequence: cue.action_sequence,
        action: 18,
        frame: 1,
        event: "Hit".into(),
        sound: None,
        prepared_audio: None,
    };
    assert!(engine.update(&w, [&e].into_iter(), 0).is_empty());
    w.actors.get_mut(&2).unwrap().x += 1;
    let mut canvas = Canvas::new(800, 600).unwrap();
    let projection = ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    };
    let (r, _) = engine.draw(&mut canvas, projection, 60_000_000);
    assert_eq!(r["flights"][0]["to"], serde_json::json!([32735, 32800]));
    engine.update(&w, [&e].into_iter(), 60_000_000);
    let (r, _) = engine.draw(&mut canvas, projection, 60_000_000);
    assert_eq!(r["launched"], 1);
    w.actors.get_mut(&2).unwrap().incarnation += 1;
    assert!(engine
        .update(&w, std::iter::empty(), 120_000_000)
        .is_empty());
    let (r, _) = engine.draw(&mut canvas, projection, 120_000_000);
    assert_eq!(r["active"], 0);
    w.generation += 1;
    engine.update(&w, std::iter::empty(), 180_000_000);
    let (r, _) = engine.draw(&mut canvas, projection, 180_000_000);
    assert_eq!(r["launched"], 0);
}
fn class4_zero_distance_zero_damage_and_miss_keep_authority() {
    use lito_modern::modern_lito::{
        compositor::Canvas,
        native_combat_animation::EnteredEvent,
        native_spell_projectiles::Projectiles,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    let mut engine = Projectiles::default();
    let projection = ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    };
    let mut canvas = Canvas::new(800, 600).unwrap();
    for (index, key) in ["hit", "zero", "miss"].into_iter().enumerate() {
        let mut w = world();
        w.generation = index as u64 + 1;
        let t = w.actors.get_mut(&2).unwrap();
        t.x = 32729;
        t.y = 32797;
        w.accept(&p(key)).unwrap();
        let authority = serde_json::to_value(&w).unwrap();
        engine.prepare(
            std::path::Path::new(r"D:\lineage開發\0-客戶端"),
            &w,
            &std::sync::atomic::AtomicBool::new(false),
        );
        let e = EnteredEvent {
            actor: 1,
            sequence: w.projectiles[0].action_sequence,
            action: 18,
            frame: 1,
            event: "Hit".into(),
            sound: None,
        prepared_audio: None,
        };
        let impacts = engine.update(&w, [&e].into_iter(), 0);
        assert_eq!(impacts.len(), usize::from(key == "hit"));
        assert!(engine
            .update(&w, std::iter::empty(), 3_000_000_000)
            .is_empty());
        let (r, events) = engine.draw(&mut canvas, projection, 3_000_000_000);
        assert_eq!(r["arrivals"], 1);
        assert_eq!(r["active"], 0);
        assert_eq!(events.iter().filter(|e| e.sound == Some(97)).count(), 1);
        assert!(engine
            .draw(&mut canvas, projection, 3_000_000_000)
            .1
            .is_empty());
        assert_eq!(serde_json::to_value(&w).unwrap(), authority);
    }
}
