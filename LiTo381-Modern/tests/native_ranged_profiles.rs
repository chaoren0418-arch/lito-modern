use lito_modern::modern_lito::{
    compositor::Canvas,
    ranged_effect::{RangedAssets, RangedPath},
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::{path::Path, sync::atomic::AtomicBool, time::Duration};

#[test]
fn original_physical_resources_draw_all_headings_and_retire_at_arrival() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let from = WorldCell { x: 32729, y: 32797 };
    let projection = ViewProjection {
        anchor_cell: from,
        anchor_pixel: PixelPoint { x: 200, y: 200 },
    };
    for graphics in [66, 1156, 2349, 2989] {
        let assets = RangedAssets::load_graphics(root, graphics, &cancel).unwrap();
        assert_eq!(
            assets.sprite_sources().len(),
            if graphics == 1156 { 1 } else { 8 }
        );
        assert_eq!(
            assets.shadow_source().map(|s| s.name.as_str()),
            (graphics == 1156).then_some("1157-0.spr")
        );
        assert_eq!(
            assets.declared_secondary(),
            (graphics == 2349).then_some(219)
        );
        for (heading, (dx, dy)) in [
            (0, (0, -3)),
            (1, (3, -3)),
            (2, (3, 0)),
            (3, (3, 3)),
            (4, (0, 3)),
            (5, (-3, 3)),
            (6, (-3, 0)),
            (7, (-3, -3)),
        ] {
            let path = RangedPath::new(
                from,
                WorldCell {
                    x: from.x + dx,
                    y: from.y + dy,
                },
            )
            .unwrap();
            let sample = assets.effect_frame(&path, Duration::ZERO).unwrap();
            assert_eq!(sample.graphics_id, graphics);
            let direction = if graphics == 1156 { 0 } else { heading };
            assert_eq!(sample.sprite_name, format!("{graphics}-{direction}.spr"));
            let special = matches!(graphics, 1156 | 2349);
            assert_eq!(
                (sample.source_frame, sample.subframe),
                (if special { 0 } else { 1 }, 0)
            );
            assert_eq!(
                assets.sprite_frames(heading).unwrap().len(),
                if special { 1 } else { 3 }
            );
            let mut canvas = Canvas::new(400, 400).unwrap();
            let blank = canvas.rgba().to_vec();
            assert!(assets
                .draw_effect(&mut canvas, &path, projection, Duration::ZERO)
                .unwrap());
            assert_ne!(canvas.rgba(), blank);
            let flying = canvas.rgba().to_vec();
            assert!(!assets
                .draw_effect(&mut canvas, &path, projection, path.finished_at())
                .unwrap());
            assert_eq!(canvas.rgba(), flying);
            assert!(!assets
                .draw_effect(&mut canvas, &path, projection, Duration::MAX)
                .unwrap());
        }
    }
}

#[test]
fn cancelled_and_unknown_projectiles_fail_explicitly() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    assert!(RangedAssets::load_graphics(root, 2989, &AtomicBool::new(true)).is_err());
    assert!(RangedAssets::load_graphics(root, 65535, &AtomicBool::new(false)).is_err());
}

#[test]
fn special_pixels_match_original_screen_blend_and_body_then_shadow_composition() {
    use lito_modern::modern_lito::{
        resources::SharedArchive, sprite::decode_world_spr_with_limits,
    };
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let from = WorldCell { x: 32729, y: 32797 };
    let path = RangedPath::new(from, WorldCell { x: 32735, y: 32800 }).unwrap();
    let view = ViewProjection {
        anchor_cell: from,
        anchor_pixel: PixelPoint { x: 200, y: 200 },
    };
    for graphics in [1156, 2349] {
        let assets = RangedAssets::load_graphics(root, graphics, &AtomicBool::new(false)).unwrap();
        let sample = assets.effect_frame(&path, Duration::ZERO).unwrap();
        let anchor = view.cell_to_pixel(sample.position).unwrap();
        let mut actual = Canvas::new(400, 400).unwrap();
        let mut expected = Canvas::new(400, 400).unwrap();
        actual.clear([48, 96, 144]);
        expected.clear([48, 96, 144]);
        let frame = sample.frame;
        if graphics == 2349 {
            expected
                .blit_screen_rgb555(
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    anchor.x + frame.x_offset,
                    anchor.y + frame.y_offset,
                )
                .unwrap();
        } else {
            expected
                .blit_rgba(
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    anchor.x + frame.x_offset,
                    anchor.y + frame.y_offset,
                )
                .unwrap();
            let body_only = expected.rgba().to_vec();
            let source = assets.shadow_source().unwrap();
            let bytes = SharedArchive::open(root, source.archive)
                .unwrap()
                .read_entry(&source.name)
                .unwrap();
            let shadow = decode_world_spr_with_limits(&bytes, Default::default()).unwrap();
            let frame = &shadow.frames[0];
            expected
                .blit_native_shadow_rgb555(
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    anchor.x + frame.x_offset,
                    anchor.y + frame.y_offset,
                )
                .unwrap();
            assert_ne!(expected.rgba(), body_only, "shadow must contribute pixels");
        }
        assets
            .draw_effect(&mut actual, &path, view, Duration::ZERO)
            .unwrap();
        assert_eq!(actual.rgba(), expected.rgba(), "graphics {graphics}");
    }
}
