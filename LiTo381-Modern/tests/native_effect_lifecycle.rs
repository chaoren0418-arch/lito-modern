use lito_modern::modern_lito::{
    compositor::Canvas,
    native_effect_timing::{sample_ticks, Clip},
    native_skill_effects::Effects,
    native_world::World,
    resources::{SharedArchive, SharedArchiveKind},
    sprite_actions::{
        ActionFrame, ActionKey, ActionLimits, Animation, Directionality, FrameEvent,
        SpriteActionCatalog,
    },
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::{
    collections::BTreeSet,
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};
const ROOT: &str = r"D:\lineage開發\0-客戶端";
fn ns(tick: u64) -> u64 {
    (tick * 1_000_000_000).div_ceil(24)
}
fn animation(ticks: &[u32]) -> Animation {
    Animation {
        label_bytes: vec![],
        directionality: Directionality::Undirected,
        frames_per_second: 24,
        frames: ticks
            .iter()
            .enumerate()
            .map(|(i, t)| ActionFrame {
                image_id: 0,
                subframe: i as u32,
                duration_ticks: *t,
                events: vec![FrameEvent::Sound(100 + i as u32)],
            })
            .collect(),
    }
}
fn catalog() -> SpriteActionCatalog {
    let bytes = SharedArchive::open(Path::new(ROOT), SharedArchiveKind::Text)
        .unwrap()
        .read_entry("list.spr")
        .unwrap();
    SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap()
}
fn packet(key: &str) -> Vec<u8> {
    let d: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let s = d["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn world() -> World {
    let mut w = World::default();
    for k in ["world", "start", "status", "actor", "bot"] {
        w.accept(&packet(k)).unwrap();
    }
    w.learned_skills.extend([1, 3]);
    w
}
fn effect(w: &mut World, id: u32, gfx: u16) {
    let mut b = vec![0x37];
    b.extend(id.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    w.accept(&b).unwrap();
}
fn projection() -> ViewProjection {
    ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    }
}
#[test]
fn explicit_action_dwell_and_cues_survive_skips_without_replay_or_clock_regression() {
    let clip = Clip::new_explicit_action(Arc::new(animation(&[2, 1, 3]))).unwrap();
    let mut p = clip.start();
    let first = p.advance(0).unwrap();
    assert_eq!((first.source_frame, first.finished), (1, false));
    assert_eq!(first.crossed_cues[0].event, FrameEvent::Sound(101));
    assert!(p.advance(0).unwrap().crossed_cues.is_empty());
    assert_eq!(p.advance(ns(1) - 1).unwrap().source_frame, 1);
    let terminal = p.advance(ns(4)).unwrap();
    assert_eq!((terminal.source_frame, terminal.finished), (0, false));
    assert_eq!(
        terminal
            .crossed_cues
            .iter()
            .map(|c| c.source_frame)
            .collect::<Vec<_>>(),
        vec![2, 0]
    );
    assert!(p.advance(ns(4) - 1).is_err());
    assert!(!p.advance(ns(6) - 1).unwrap().finished);
    assert!(p.advance(ns(6)).unwrap().finished);
    assert!(p.advance(u64::MAX).unwrap().crossed_cues.is_empty());
    let all = clip.start().advance(u64::MAX).unwrap();
    assert!(all.finished);
    assert_eq!(
        all.crossed_cues
            .iter()
            .map(|c| c.source_frame)
            .collect::<Vec<_>>(),
        vec![1, 2, 0]
    );
}
#[test]
fn single_frame_zero_duration_and_invalid_resource_shapes_have_explicit_bounds() {
    let mut p = Clip::new(Arc::new(animation(&[2]))).unwrap().start();
    let s = p.advance(0).unwrap();
    assert_eq!(
        (s.source_frame, s.finished, s.crossed_cues.len()),
        (0, false, 1)
    );
    assert!(!p.advance(83_000_000 - 1).unwrap().finished);
    assert!(p.advance(83_000_000).unwrap().finished);
    let s = Clip::new(Arc::new(animation(&[1, 0, 1])))
        .unwrap()
        .start()
        .advance(0)
        .unwrap();
    assert_eq!(s.source_frame, 2);
    assert_eq!(s.crossed_cues.len(), 2);
    for ticks in [vec![], vec![0], vec![1; 257], vec![1441]] {
        assert!(Clip::new(Arc::new(animation(&ticks))).is_err());
    }
    let mut a = animation(&[1, 1]);
    a.frames_per_second = 25;
    assert!(Clip::new(Arc::new(a)).is_err());
    let mut a = animation(&[1, 1]);
    a.frames[0].events = vec![FrameEvent::Sound(1); 1025];
    assert!(Clip::new(Arc::new(a)).is_err());
    assert_eq!(sample_ticks(&[], u64::MAX), (0, true));
}
#[test]
fn original_class0_resources_match_every_boundary_including_terminal_dwell() {
    use sha2::{Digest, Sha256};
    let catalog = catalog();
    let mut rows = Vec::new();
    let mut sprites = SharedArchive::open(Path::new(ROOT), SharedArchiveKind::Sprite).unwrap();
    for gfx in [168, 221, 744] {
        let a = catalog
            .resolve_action(gfx, ActionKey::new(0))
            .unwrap()
            .animation;
        let clip = Clip::new(Arc::new(a.clone())).unwrap();
        let mut playback = clip.start();
        let mut start = 0;
        let mut start_ms = 0;
        let mut seen = BTreeSet::new();
        for source in (1..a.frames.len()).chain(std::iter::once(0)) {
            let f = &a.frames[source];
            let sample = playback.advance(start_ms * 1_000_000).unwrap();
            assert_eq!((sample.source_frame, sample.finished), (source, false));
            for e in sample.crossed_cues {
                assert!(seen.insert((e.source_frame, e.event_index)));
            }
            start += u64::from(f.duration_ticks);
            start_ms += u64::from(f.duration_ticks) * 1000 / 24;
            assert_eq!(
                playback.advance(start_ms * 1_000_000 - 1).unwrap().source_frame,
                source
            );
        }
        assert!(playback.advance(start_ms * 1_000_000).unwrap().finished);
        assert_eq!(
            seen.len(),
            a.frames.iter().map(|f| f.events.len()).sum::<usize>()
        );
        let terminal = a.frame_asset(gfx, 0, 0).unwrap();
        let raw = sprites.read_entry(&terminal.file_name()).unwrap();
        let decoded = lito_modern::modern_lito::sprite::decode_classic_spr_with_limits(
            &raw,
            Default::default(),
        )
        .unwrap();
        let pixels = &decoded.frames[terminal.subframe as usize].rgba;
        rows.push(serde_json::json!({"graphics":gfx,"fps":a.frames_per_second,"total_ticks":start,"native_duration_ms":start_ms,
            "terminal_sprite":{"name":terminal.file_name(),"bytes":raw.len(),"sha256":format!("{:x}",Sha256::digest(&raw)),"nontransparent_pixels":pixels.chunks_exact(4).filter(|p|p[3]!=0).count()},
            "frames":a.frames.iter().enumerate().map(|(i,f)|serde_json::json!({"source":i,"image":f.image_id,"subframe":f.subframe,"ticks":f.duration_ticks,"events":format!("{:?}",f.events)})).collect::<Vec<_>>()}));
    }
    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/current/verification/native-effect-catalog.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
#[test]
fn s37_snapshot_survives_movement_death_removal_replacement_and_queue_eviction() {
    let mut w = world();
    effect(&mut w, 2, 744);
    let origin = w.effects[0].position;
    let cancel = AtomicBool::new(false);
    let mut fx = Effects::default();
    fx.prepare(Path::new(ROOT), &w, &cancel);
    let mut canvas = Canvas::new(800, 600).unwrap();
    let (r, events) = fx.draw(&w, &mut canvas, projection(), 0);
    assert_eq!(r["active"], 1);
    assert!(!events.is_empty());
    for variant in 0..5 {
        match variant {
            0 => {
                let a = w.actors.get_mut(&2).unwrap();
                a.x += 2;
                a.y += 3;
            }
            1 => w.actors.get_mut(&2).unwrap().dead = true,
            2 => w.actors.get_mut(&2).unwrap().incarnation += 1,
            3 => {
                w.actors.remove(&2);
            }
            _ => w.effects.clear(),
        }
        let before = serde_json::to_value(&w).unwrap();
        let (r, events) = fx.draw(&w, &mut canvas, projection(), (variant + 1) * 10_000_000);
        assert_eq!(r["active"], 1);
        assert_eq!(r["effects"][0]["position"], serde_json::json!(origin));
        assert!(events.is_empty());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    w.generation += 1;
    assert_eq!(
        fx.draw(&w, &mut canvas, projection(), 60_000_000).0["active"],
        0
    );
    // S37 was accepted while the object existed; consuming it after removal
    // must still start the already authoritative snapshot once.
    let mut w = world();
    effect(&mut w, 2, 221);
    w.actors.remove(&2);
    let mut fx = Effects::default();
    fx.prepare(Path::new(ROOT), &w, &cancel);
    let (r, _) = fx.draw(&w, &mut canvas, projection(), 0);
    assert_eq!(r["started"], 1);
    assert_eq!(r["omitted"], 0);
    let (r, e) = fx.draw(&w, &mut canvas, projection(), 0);
    assert_eq!(r["started"], 1);
    assert!(e.is_empty());
    for _ in 0..65 {
        let mut b = vec![0x53];
        b.extend(origin[0].to_le_bytes());
        b.extend(origin[1].to_le_bytes());
        b.extend(221u16.to_le_bytes());
        w.accept(&b).unwrap();
    }
    let (r, _) = fx.draw(&w, &mut canvas, projection(), 0);
    assert_eq!(r["active"], 64);
    assert_eq!(r["omitted"], 2);
}
#[test]
fn scene_publishes_original_terminal_frames_then_retires_without_changing_authority() {
    use lito_modern::modern_lito::native_world_scene::Model;
    let mut w = world();
    let own = w.status.as_ref().unwrap().object_id;
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(Path::new(ROOT), &w, &cancel).unwrap();
    effect(&mut w, own, 221);
    effect(&mut w, own, 744);
    let catalog = catalog();
    let mut times = BTreeSet::from([0]);
    let mut terminals = Vec::new();
    for gfx in [221, 744] {
        let a = catalog
            .resolve_action(gfx, ActionKey::new(0))
            .unwrap()
            .animation;
        let mut start_ms = 0u64;
        for f in &a.frames[1..] {
            let due = start_ms + u64::from(f.duration_ticks) * 1000 / 24;
            if due > start_ms { times.insert(due * 1_000_000 - 1); }
            times.insert(due * 1_000_000);
            start_ms = due;
        }
        let end_ms = start_ms + u64::from(a.frames[0].duration_ticks) * 1000 / 24;
        times.extend([end_ms * 1_000_000 - 1, end_ms * 1_000_000, end_ms * 1_000_000 + 1]);
        terminals.push((gfx, start_ms * 1_000_000, end_ms * 1_000_000));
    }
    let before = serde_json::to_value(&w).unwrap();
    let mut frames = Vec::new();
    let mut seen = BTreeSet::new();
    for now in times {
        let f = model.draw_at(&w, &cancel, now).unwrap();
        let mut r = f.receipt;
        for e in r["skill_effects"]["entered_events"].as_array().unwrap() {
            assert!(seen.insert(e.to_string()));
        }
        for &(gfx, start, end) in &terminals {
            if now >= start && now < end {
                let row = r["skill_effects"]["effects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["graphics"] == gfx)
                    .unwrap();
                assert_eq!(row["source_frame"], 0);
                assert_eq!(row["drawn"], true);
                assert_eq!(row["finished"], false);
            }
        }
        if terminals.iter().any(|(_, start, _)| now == *start)
            || terminals.iter().all(|(_, _, end)| now > *end)
        {
            let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                "target/current/verification/native-effect-terminal-{now}.png"
            ));
            let mut enc = png::Encoder::new(std::fs::File::create(&p).unwrap(), 800, 600);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()
                .unwrap()
                .write_image_data(f.canvas.rgba())
                .unwrap();
            r["png"] = serde_json::json!(p);
        }
        frames.push(r);
    }
    assert_eq!(frames.last().unwrap()["skill_effects"]["active"], 0);
    assert_eq!(seen.len(), 2);
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/current/verification/native-effect-terminal.json"),
        serde_json::to_vec_pretty(&frames).unwrap(),
    )
    .unwrap();
}

#[test]
fn native_class0_integer_frames_and_late_callback_deadlines_match_original_timer() {
    // Pinned PE4271D92F:43E888..43E895 truncates ticks*1000/FPS per frame;
    // 4D65FE..4D6610 schedules each next callback from its actual dispatch time.
    let clip = Clip::new(Arc::new(animation(&[2, 1, 3]))).unwrap();
    let mut p = clip.start();
    for (time, source, finished, cues) in [
        (0, 1, false, 1), (40_999_999, 1, false, 0),
        (41_000_000, 2, false, 1), (165_999_999, 2, false, 0),
        (166_000_000, 0, false, 1), (248_999_999, 0, false, 0),
        (249_000_000, 0, true, 0),
    ] {
        let sample = p.advance(time).unwrap();
        assert_eq!((sample.source_frame, sample.finished, sample.crossed_cues.len()),
                   (source, finished, cues), "native timer at {time}ns");
    }
    let mut late = clip.start();
    assert_eq!(late.advance(0).unwrap().source_frame, 1);
    let dispatch = late.advance(120_000_000).unwrap();
    assert_eq!((dispatch.source_frame, dispatch.finished, dispatch.crossed_cues.len()), (2, false, 1));
    assert!(late.advance(120_000_000).unwrap().crossed_cues.is_empty());
    assert_eq!(late.advance(244_999_999).unwrap().source_frame, 2);
    assert_eq!(late.advance(245_000_000).unwrap().source_frame, 0);
    assert!(!late.advance(327_999_999).unwrap().finished);
    assert!(late.advance(328_000_000).unwrap().finished);
}

#[test]
fn native_effect_quantizes_common_clock_and_preserves_late_frames_without_world_changes() {
    let mut w = world();
    let id = w.own_actor().unwrap().id;
    effect(&mut w, id, 221);
    let before = serde_json::to_value(&w).unwrap();
    let mut fx = Effects::default();
    fx.prepare(Path::new(ROOT), &w, &AtomicBool::new(false));
    let mut canvas = Canvas::new(800, 600).unwrap();
    let mut samples = Vec::new();
    for (now, source, active, callbacks, due) in [
        (900_000, 1, 1, 1, Some(83)),
        (82_999_999, 1, 1, 1, Some(83)),
        (83_000_000, 2, 1, 2, Some(166)),
        (1_000_000_000, 3, 1, 3, Some(1083)),
        (1_083_000_000, 4, 1, 4, Some(1166)),
        (1_166_000_000, 0, 1, 5, Some(1249)),
        (1_249_000_000, 0, 0, 6, None),
    ] {
        let (r, events) = fx.draw(&w, &mut canvas, projection(), now);
        assert_eq!(r["active"], active);
        assert_eq!(r["effects"][0]["source_frame"], source);
        assert_eq!(r["effects"][0]["callback_count"], callbacks);
        assert_eq!(r["effects"][0]["next_callback_ms"], serde_json::json!(due));
        assert_eq!(r["effects"][0]["native_clock_elapsed_ms"], now / 1_000_000);
        if now == 900_000 { assert_eq!(events.len(), 1); } else { assert!(events.is_empty()); }
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
        samples.push(r);
    }
    assert_eq!(fx.draw(&w, &mut canvas, projection(), 2_000_000_000).0["active"], 0);
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-class0-clock-samples.json"), serde_json::to_vec_pretty(&samples).unwrap()).unwrap();
}
