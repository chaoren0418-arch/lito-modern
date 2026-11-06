use lito_modern::modern_lito::{
    compositor::Canvas,
    ranged_effect::{RangedAssets, RangedPath},
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::{path::Path, sync::atomic::AtomicBool, time::Duration};

#[test]
fn original_arrow_and_sting_resources_draw_all_headings_and_retire_at_arrival() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let from = WorldCell { x: 32729, y: 32797 };
    let projection = ViewProjection {
        anchor_cell: from,
        anchor_pixel: PixelPoint { x: 200, y: 200 },
    };
    for graphics in [66, 2989] {
        let assets = RangedAssets::load_graphics(root, graphics, &cancel).unwrap();
        assert_eq!(assets.sprite_sources().len(), 8);
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
            assert_eq!(sample.sprite_name, format!("{graphics}-{heading}.spr"));
            assert_eq!((sample.source_frame, sample.subframe), (1, 0));
            assert_eq!(assets.sprite_frames(heading).unwrap().len(), 3);
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
fn cancelled_and_unqualified_special_projectiles_fail_explicitly() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    assert!(RangedAssets::load_graphics(root, 2989, &AtomicBool::new(true)).is_err());
    // Energy arrow has a sound and secondary effect. It must retain its own
    // resource contract rather than silently use ordinary arrow66 pixels.
    assert!(RangedAssets::load_graphics(root, 2349, &AtomicBool::new(false)).is_err());
}
