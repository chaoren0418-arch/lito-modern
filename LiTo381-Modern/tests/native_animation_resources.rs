use lito_modern::modern_lito::{
    resources::{SharedArchive, SharedArchiveKind},
    sprite_actions::{ActionDefinition, ActionLimits, SpriteActionCatalog},
};
#[test]
fn inspect_actual_world_animation_definitions() {
    let bytes = SharedArchive::open(
        std::path::Path::new(r"D:\lineage開發\0-客戶端"),
        SharedArchiveKind::Text,
    )
    .unwrap()
    .read_entry("list.spr")
    .unwrap();
    let catalog =
        SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
    let mut sprites = SharedArchive::open(
        std::path::Path::new(r"D:\lineage開發\0-客戶端"),
        SharedArchiveKind::Sprite,
    )
    .unwrap();
    for id in [0, 61, 936, 143, 834, 460, 148, 237, 727, 98] {
        let clip =
            lito_modern::modern_lito::native_world_animation::resolve(&catalog, id, 3).unwrap();
        assert!(clip.animation.frames.len() > 1);
        for frame in &clip.animation.frames {
            let member = format!("{}-{}.spr", clip.graphics, frame.image_id);
            let raw = sprites.read_entry(&member).unwrap();
            let decoded = lito_modern::modern_lito::sprite::decode_classic_spr_with_limits(
                &raw,
                Default::default(),
            )
            .unwrap();
            assert!((frame.subframe as usize) < decoded.frames.len(), "{member}");
        }
    }
    let board = &catalog.sprites[&1544];
    assert_eq!(board.image_count, Some(1));
    assert!(board.actions.is_empty());
    let raw = sprites.read_entry("1544-0.spr").unwrap();
    assert_eq!(
        lito_modern::modern_lito::sprite::decode_classic_spr_with_limits(&raw, Default::default())
            .unwrap()
            .frames
            .len(),
        1
    );
    let mut bank=lito_modern::modern_lito::native_actor_layers::Bank::open(std::path::Path::new(r"D:\lineage開發\0-客戶端")).unwrap();
    bank.ensure("869-0.spr",&std::sync::atomic::AtomicBool::new(false)).unwrap();
    let loaded=bank.get("869-0.spr").unwrap();
    let frame=&loaded.sprite.frames[0];
    let out=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/869-original.png");
    let mut enc=png::Encoder::new(std::fs::File::create(&out).unwrap(),frame.width,frame.height);enc.set_color(png::ColorType::Rgba);enc.set_depth(png::BitDepth::Eight);enc.write_header().unwrap().write_image_data(&frame.rgba).unwrap();
    println!("869 source={:?} frames={} offset={},{} size={}x{}",loaded.source,loaded.sprite.frames.len(),frame.x_offset,frame.y_offset,frame.width,frame.height);
    let mut report = Vec::new();
    for id in [
        0, 61, 936, 143, 834, 460, 148, 1544, 237, 727, 98, 2083, 3225, 2665, 2667, 2669, 2671, 869,
    ] {
        let row = &catalog.sprites[&id];
        let actions:Vec<_>=row.actions.iter().filter(|(k,_)|k.id<100).map(|(k,a)|{
   match a {
    ActionDefinition::Animation(a)=>serde_json::json!({"action":k.id,"label":String::from_utf8_lossy(&a.label_bytes),"fps":a.frames_per_second,"frames":a.frames.iter().map(|f|serde_json::json!({"image":f.image_id,"sub":f.subframe,"ticks":f.duration_ticks,"events":format!("{:?}",f.events)})).collect::<Vec<_>>()}),
    _=>serde_json::json!({"action":k.id,"definition":format!("{a:?}")})
   }
  }).collect();
        report.push(serde_json::json!({"id":id,"graphics":row.graphics_id,"header":String::from_utf8_lossy(&row.header_bytes),"attributes":format!("{:?}",row.attributes),"actions":actions}));
    }
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/verification/native-animation-definitions.json");
    std::fs::write(&output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("{}", output.display());
}
