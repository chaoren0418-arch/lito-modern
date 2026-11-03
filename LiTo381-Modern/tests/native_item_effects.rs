use lito_modern::modern_lito::{
    compositor::Canvas,
    native_skill_effects::Effects,
    native_skills::Effect,
    native_world::World,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::AtomicBool};
#[test]
fn original_potion_effects_follow_class0_frames_terminal_dwell_and_retire() {
    let mut w = World::default();
    w.generation = 1;
    for (n, graphics) in [189, 194, 197].into_iter().enumerate() {
        w.effects.push(Effect {
            sequence: n as u64 + 1,
            graphics,
            target: None,
            position: [10 + n as u16 * 3, 10],
        });
    }
    let unchanged = serde_json::to_value(&w).unwrap();
    let mut fx = Effects::items();
    fx.prepare(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &w,
        &AtomicBool::new(false),
    );
    let mut skill = Effects::default();
    skill.prepare(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &w,
        &AtomicBool::new(false),
    );
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    std::fs::create_dir_all(&output).unwrap();
    let mut rows = vec![];
    for (tick, source) in [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (5, 5),
        (7, 6),
        (9, 7),
        (11, 8),
        (13, 9),
        (15, 0),
        (16, 0),
        (17, 0),
        (18, 0),
    ] {
        let mut c = Canvas::new(800, 600).unwrap();
        let project = ViewProjection {
            anchor_cell: WorldCell { x: 10, y: 10 },
            anchor_pixel: PixelPoint { x: 250, y: 300 },
        };
        let now = (tick * 1_000_000_000u64).div_ceil(24);
        let (mut r, audio) = fx.draw(&w, &mut c, project, now);
        assert_eq!(r["started"], 3);
        assert_eq!(r["omitted"], 0);
        assert!(r["load_error"].is_null());
        assert!(audio.is_empty());
        assert_eq!(r["active"], if tick < 18 { 3 } else { 0 });
        for e in r["effects"].as_array().unwrap() {
            assert_eq!(e["source_frame"], source);
            assert_eq!(e["drawn"], tick < 18);
        }
        assert_eq!(r["sources"].as_array().unwrap().len(), 3);
        let skill_receipt = skill.draw(&w, &mut c, project, now).0;
        assert_eq!(skill_receipt["started"], 0);
        assert_eq!(skill_receipt["omitted"], 0);
        assert!(skill_receipt["sources"].is_null());
        r["tick"] = serde_json::json!(tick);
        r["rgba_sha256"] = serde_json::json!(format!("{:X}", Sha256::digest(c.rgba())));
        if [0, 15, 17, 18].contains(&tick) {
            let file = output.join(format!("native-item-effect-{tick}.png"));
            let mut e = png::Encoder::new(std::fs::File::create(&file).unwrap(), 800, 600);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .unwrap()
                .write_image_data(c.rgba())
                .unwrap();
            r["png"] = serde_json::json!(file);
        }
        rows.push(r);
        assert_eq!(serde_json::to_value(&w).unwrap(), unchanged);
    }
    assert_eq!(rows[9]["rgba_sha256"], rows[11]["rgba_sha256"]);
    assert_ne!(rows[11]["rgba_sha256"], rows[12]["rgba_sha256"]);
    std::fs::write(
        output.join("native-item-effects.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
#[test]
fn original_type9_ground_items_are_single_frame_static_sprites() {
    use lito_modern::modern_lito::{
        native_world::actor,
        native_world_animation::Animator,
        resources::{SharedArchive, SharedArchiveKind},
        sprite::decode_classic_spr_with_limits,
        sprite_actions::{ActionLimits, SpriteActionCatalog},
    };
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let bytes = SharedArchive::open(root, SharedArchiveKind::Text)
        .unwrap()
        .read_entry("list.spr")
        .unwrap();
    let catalog =
        SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
    let mut sprites = SharedArchive::open(root, SharedArchiveKind::Sprite).unwrap();
    let fixture: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-item-actions-current.json"))
            .unwrap();
    let mut canvas = Canvas::new(240, 160).unwrap();
    let mut report = vec![];
    for (index, key) in ["ground_potion", "ground_gem"].into_iter().enumerate() {
        let hex = fixture["packets"][key].as_str().unwrap();
        let b: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let mut a = actor(&b).unwrap();
        a.incarnation = 1;
        let mut anim = Animator::default();
        let pose = anim.pose(&catalog, &a, 0).unwrap();
        let later = anim.pose(&catalog, &a, 9_000_000_000).unwrap();
        assert_eq!(pose.phase, "ground_item");
        assert!(pose.static_prop && pose.action_complete);
        assert_eq!(
            (pose.graphics, pose.image, pose.subframe),
            (u32::from(a.graphics), 0, 0)
        );
        assert_eq!(
            (later.graphics, later.image, later.subframe),
            (pose.graphics, pose.image, pose.subframe)
        );
        let member = format!("{}-0.spr", a.graphics);
        let raw = sprites.read_entry(&member).unwrap();
        let decoded = decode_classic_spr_with_limits(&raw, Default::default()).unwrap();
        assert_eq!(decoded.frames.len(), 1);
        let f = &decoded.frames[0];
        assert!(f.rgba.chunks_exact(4).any(|p| p[3] != 0));
        let x = 70 + index as i32 * 100 + f.x_offset;
        let y = 100 + f.y_offset;
        canvas.blit_rgba(&f.rgba, f.width, f.height, x, y).unwrap();
        report.push(serde_json::json!({"graphics":a.graphics,"header":String::from_utf8_lossy(&catalog.sprites[&u32::from(a.graphics)].header_bytes),"member":member,"bytes":raw.len(),"sha256":format!("{:X}",Sha256::digest(&raw)),"position":[x,y],"size":[f.width,f.height]}));
        a.walk_action = 1;
        assert!(anim.pose(&catalog, &a, 10_000_000_000).is_err());
    }
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    let path = output.join("native-ground-items.png");
    let mut png = png::Encoder::new(std::fs::File::create(&path).unwrap(), 240, 160);
    png.set_color(png::ColorType::Rgba);
    png.set_depth(png::BitDepth::Eight);
    png.write_header()
        .unwrap()
        .write_image_data(canvas.rgba())
        .unwrap();
    std::fs::write(output.join("native-ground-items.json"),serde_json::to_vec_pretty(&serde_json::json!({"png":path,"sources":report,"rgba_sha256":format!("{:X}",Sha256::digest(canvas.rgba()))})).unwrap()).unwrap();
}
