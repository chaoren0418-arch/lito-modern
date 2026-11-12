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
        include_bytes!("fixtures/native-wind-cutter-current.json"),
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
    let mut w = world();
    let mut menu=lito_modern::modern_lito::native_skill_selection::Model::default();
    menu.sync(99,&w,true);
    assert!(menu.rows().iter().any(|r|r.id==7&&r.label.contains("風刃")));
    menu.select(Some(7)).unwrap();
    let selected=menu.choice(99,&w,lito_modern::modern_lito::native_skill_selection::Target::capture(&w,2)).unwrap();
    assert_eq!((selected.skill,selected.target),(7,Some(2)));
    assert!(menu.choice(100,&w,lito_modern::modern_lito::native_skill_selection::Target::capture(&w,2)).is_err());
    w.learned_skills.remove(&7);
    assert!(skills::request(&w,7,Some(2)).is_err());
    w.learned_skills.insert(7);
    w.actors.get_mut(&2).unwrap().dead=true;
    assert!(skills::request(&w,7,Some(2)).is_err());
    w.actors.get_mut(&2).unwrap().dead=false;
    assert!(skills::request(&w,7,Some(999)).is_err());
    assert_eq!(skills::request(&w, 7, Some(2)).unwrap(), p("request"));
    assert!(skills::request(&w, 7, None).is_err());
    assert!(skills::request(&w, 7, Some(1)).is_err());
    w.actors.get_mut(&2).unwrap().x = 32740;
    assert!(skills::request(&w, 7, Some(2)).is_err());
    let a = combat::decode_attack(&p("hit")).unwrap().unwrap();
    let shot = a.projectile.unwrap();
    assert_eq!((a.action, shot.flight_type, shot.graphics), (18, 0, 1799));
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
    let mut sound719 = 0;
    let mut sound711=0;
    let mut secondary_hit=0;
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
        for c in s["non_audio_cues"].as_array().unwrap() {
            assert_eq!(c["event"], "Hit");
            assert_eq!(c["source_frame"], 1);
            assert_eq!(c["authority_callback"], false);
            secondary_hit += 1;
        }
        for e in s["entered_events"].as_array().unwrap() {
            if e["sound"]==711 {sound711+=1;}
            if e["sound"] == 719 {
                sound719 += 1;
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
                .join(format!("target/current/verification/native-wind-cutter-{ms}.png"));
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
    // Six movement callbacks: the cyclic flight (sub_4E5B50) enters its source1
    // sound slot at the launch and once more before arrival. Whether the second
    // entry is audible is the same-ID suppression policy, not this count.
    assert_eq!(sound719, 2);
    assert_eq!(sound711,1);
    assert_eq!(secondary_hit,1);
    assert_eq!(gfx, std::collections::BTreeSet::from([1799, 1800]));
    assert_eq!(frames.last().unwrap()["spell_projectiles"]["active"], 0);
    assert_eq!(serde_json::to_value(&w).unwrap(), authority);
    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-wind-cutter.json"),
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
        std::path::Path::new(r"D:\lineage開發\0-客戶端"),1799,&std::sync::atomic::AtomicBool::new(false)).unwrap();
    assert_eq!(assets.sprite_sources().len(),2);
    let mut flight_samples=std::collections::BTreeSet::new();
    let mut impact_samples=std::collections::BTreeSet::new();
    let from=WorldCell{x:32729,y:32797};
    let mut headings=std::collections::BTreeSet::new();
    for (dx,dy) in [(0,-4),(4,-4),(4,0),(4,4),(0,4),(-4,4),(-4,0),(-4,-4)] {
        let path=lito_modern::modern_lito::light_arrow_effect::LightArrowPath::new(from,WorldCell{x:from.x+dx,y:from.y+dy}).unwrap();
        headings.insert(path.heading());
        let frame=assets.effect_frame(&path,std::time::Duration::ZERO).unwrap();
        assert_eq!(frame.graphics_id,1799);
        flight_samples.insert((frame.sprite_name.to_owned(),frame.subframe));
        let impact=assets.effect_frame(&path,path.impact_at()).unwrap();
        impact_samples.insert((impact.sprite_name.to_owned(),impact.subframe));
        assert_eq!(assets.effect_frame(&path,path.impact_at()).unwrap().graphics_id,1800);
        let mut playback=assets.start_effect(&path);
        let sample=playback.advance(assets.effect_duration(&path)).unwrap();
        assert!(sample.finished);
        assert_eq!(sample.crossed_cues.iter().filter(|c|c.event==lito_modern::modern_lito::sprite_actions::FrameEvent::Hit).count(),1);
        for sound in [719,711] {assert_eq!(sample.crossed_cues.iter().filter(|c|c.event==lito_modern::modern_lito::sprite_actions::FrameEvent::Sound(sound)).count(),1);}
        assert!(playback.advance(assets.effect_duration(&path)).unwrap().crossed_cues.is_empty());
    }
    assert_eq!(headings.len(),8);
    assert_eq!(flight_samples.len(),1,"undirected flight must reuse original image in every heading");
    assert_eq!(impact_samples.len(),1,"undirected impact must reuse original image in every heading");
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
#[test]
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
        // Native movement and finite callbacks advance one step per dispatch
        // and reschedule from it (424540/42F300 + 4D65F0): one late poll can
        // never catch up. Drive the clock at 1ms and collect every cue from
        // the draw that follows its update.
        let mut events = Vec::new();
        let mut non_audio = Vec::new();
        let mut r = serde_json::Value::Null;
        for ms in 0..=3000u64 {
            assert!(engine
                .update(&w, std::iter::empty(), ms * 1_000_000)
                .is_empty());
            let (receipt, entered) = engine.draw(&mut canvas, projection, ms * 1_000_000);
            assert!(receipt["clock_errors"].as_array().unwrap().is_empty(), "{receipt}");
            events.extend(entered);
            non_audio.extend(receipt["non_audio_cues"].as_array().unwrap().iter().cloned());
            r = receipt;
        }
        assert_eq!(r["arrivals"], 1);
        assert_eq!(r["active"], 0);
        // "miss" has no target actor: it flies the six wire cells, so the
        // cyclic flight re-enters its source1 sound slot once more
        // (4-frame flights at movement 5, the 5-frame wind flight at 6).
        let launch_sounds = if key == "miss" { 2 } else { 1 };
        assert_eq!(events.iter().filter(|e| e.sound == Some(719)).count(), launch_sounds);
        assert_eq!(events.iter().filter(|e| e.sound == Some(711)).count(), 1);
        assert_eq!(non_audio.len(),1);
        assert_eq!(non_audio[0]["authority_callback"],false);
        assert!(engine
            .draw(&mut canvas, projection, 3_000_000_000)
            .1
            .is_empty());
        assert_eq!(serde_json::to_value(&w).unwrap(), authority);
    }
}
