use super::*;
use crate::modern_lito::{
    native_world_effects::{Mix, Target},
    world_map::PixelPoint,
};

fn world() -> World {
    let mut world = World::default();
    let f: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-world-current.json"
    ))
    .unwrap();
    for key in ["world", "start", "status", "actor", "bot"] {
        let hex = f["packets"][key].as_str().unwrap();
        let b: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        world.accept(&b).unwrap();
    }
    world
}

#[test]
#[ignore = "read-only original 2942/2943/2944/2954 SPR and 1505/1509/1502 WAV; memory only"]
fn original_dark_elf_layers_audio_history_and_natural_finite_retirement() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let mut shared = None;
    for (gfx, sound, count) in [(2942u16, 1505, 1), (2943, 1509, 1), (2944, 1502, 1)] {
        let mut w = world();
        let own = w.own_actor().unwrap().clone();
        let view = ViewProjection {
            anchor_cell: WorldCell {
                x: i32::from(own.x),
                y: i32::from(own.y),
            },
            anchor_pixel: PixelPoint { x: 400, y: 300 },
        };
        let mut packet = vec![0x37];
        packet.extend(own.id.to_le_bytes());
        packet.extend(gfx.to_le_bytes());
        w.accept(&packet).unwrap();
        let mut invis = vec![0x34];
        invis.extend(own.id.to_le_bytes());
        invis.push(1);
        w.accept(&invis).unwrap();
        let authority = serde_json::to_value(&w).unwrap();
        let mut e = Effects::default();
        e.prepare_with_catalog(root, &w, &cancel, &mut shared);
        assert!(e.dark_elf_error.is_none(), "{:?}", e.dark_elf_error);
        let bank = e.dark_elf_bank.as_ref().expect("original bank");
        assert!(bank.sources().any(|s| s.name == format!("{gfx}-0.spr")));
        assert!(bank.sound_bank().get(sound).is_some());
        if gfx == 2943 {
            let blank = bank.frame(2943, 0, 1, false).unwrap().frame;
            assert!(crate::modern_lito::native_dark_elf_effects::is_blank(
                "2943-0.spr",
                0,
                blank
            ));
            assert!(!crate::modern_lito::native_dark_elf_effects::is_blank(
                "2943-0.spr",
                4,
                blank
            ));
            assert!(!crate::modern_lito::native_dark_elf_effects::is_blank(
                "2942-0.spr",
                0,
                blank
            ));
            let mut changed = blank.clone();
            changed.unknown1 = 0;
            assert!(!crate::modern_lito::native_dark_elf_effects::is_blank(
                "2943-0.spr",
                0,
                &changed
            ));
        }

        if gfx == 2942 {
            let source = bank.sources().find(|s| s.name == "2942-0.spr").unwrap();
            assert_eq!(
                source
                    .sha256
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<String>(),
                "AF84AF8C0CE0BD7459545C6E94E1676C9DF2826C785802362848FDB0DD8ABAFC"
            );
        }
        let mut fragments = Vec::new();
        let (receipt, audio) = e.draw_to(&w, &mut Target::Fragments(&mut fragments), view, 0);
        assert_eq!(receipt["omitted"], 0);
        assert_eq!(fragments.len(), count);
        assert!(if gfx == 2943 {
            matches!(fragments[0].mix, Mix::Multiply)
        } else {
            matches!(fragments[0].mix, Mix::Screen)
        });
        if count == 2 {
            assert!(matches!(fragments[1].mix, Mix::Multiply));
        }
        assert_eq!(audio.len(), 1);
        assert_eq!(audio[0].sound, Some(sound));
        let prepared = audio[0].prepared_audio.as_ref().unwrap();
        assert_eq!(prepared.finite_event().unwrap().graphics, gfx);
        assert_eq!(prepared.finite_event().unwrap().source_frame, 1);
        drop(fragments);
        assert_eq!(serde_json::to_value(&w).unwrap(), authority);

        // Scene rebuild: restore callback progress, then reload original resources.
        let history = e.history();
        let mut restored = Effects::default();
        restored.restore_history(history);
        restored.prepare_with_catalog(root, &w, &cancel, &mut shared);
        let mut fragments = Vec::new();
        let (receipt, audio) =
            restored.draw_to(&w, &mut Target::Fragments(&mut fragments), view, 0);
        assert_eq!(receipt["started"], 1);
        assert!(audio.is_empty());
        assert_eq!(fragments.len(), count);
        drop(fragments);
        // Remove the source actor. The finite cue is frozen at receipt coordinates.
        let mut remove = vec![0x15];
        remove.extend(own.id.to_le_bytes());
        w.accept(&remove).unwrap();
        let after_remove = serde_json::to_value(&w).unwrap();
        let mut callbacks = 0;
        let mut final_ms = 0;
        while let Some(active) = restored.active.first() {
            final_ms = active.playback.next_callback_millis().unwrap();
            let mut fragments = Vec::new();
            let (receipt, audio) = restored.draw_to(
                &w,
                &mut Target::Fragments(&mut fragments),
                view,
                final_ms * 1_000_000,
            );
            assert!(audio.is_empty());
            assert_eq!(receipt["omitted"], 0);
            assert!(fragments.iter().all(|f| f.position == view.anchor_cell));
            if gfx == 2943 && receipt["active"] == 1 {
                let source = receipt["effects"][0]["source_frame"].as_u64().unwrap();
                if source == 0 || source >= 5 {
                    assert_eq!(fragments.len(), 2);
                    assert!(matches!(fragments[0].mix, Mix::Screen));
                    assert!(matches!(fragments[1].mix, Mix::Multiply));
                } else {
                    assert_eq!(fragments.len(), 1);
                    assert!(matches!(fragments[0].mix, Mix::Multiply));
                }
            }
            callbacks += 1;
            assert!(callbacks <= 13);
        }
        if gfx == 2942 {
            assert_eq!(final_ms, 1496);
        }
        assert_eq!(serde_json::to_value(&w).unwrap(), after_remove);
        let history = restored.history();
        restored.restore_history(history);
        let mut fragments = Vec::new();
        let (receipt, audio) = restored.draw_to(
            &w,
            &mut Target::Fragments(&mut fragments),
            view,
            961_000_000_000,
        );
        assert_eq!(receipt["started"], 1);
        assert_eq!(receipt["active"], 0);
        assert!(audio.is_empty() && fragments.is_empty());
    }
}

#[test]
fn t021_history_does_not_restore_effects_into_a_different_world_generation() {
    let mut w = world();
    let a = crate::modern_lito::native_dark_elf_effects::qualify(
        &SpriteActionCatalog::parse_for_inspection(
            include_bytes!("../../tests/fixtures/native-dark-elf.spr.txt"),
            crate::modern_lito::sprite_actions::ActionLimits::default(),
        )
        .unwrap(),
    )
    .unwrap()
    .remove(0)
    .2;
    let e = Effects {
        generation: Some(w.generation),
        seen: [9].into(),
        active: vec![Active {
            cue: Effect {
                sequence: 9,
                graphics: 2942,
                target: None,
                position: [0, 0],
            },
            start: 0,
            playback: Clip::new(Arc::new(a)).unwrap().start(),
        }],
        ..Default::default()
    };
    let mut restored = Effects::default();
    restored.restore_history(e.history());
    w.generation += 1;
    let mut fragments = Vec::new();
    let view = ViewProjection {
        anchor_cell: WorldCell { x: 0, y: 0 },
        anchor_pixel: PixelPoint { x: 0, y: 0 },
    };
    let (receipt, audio) = restored.draw_to(&w, &mut Target::Fragments(&mut fragments), view, 0);
    assert_eq!(receipt["active"], 0);
    assert!(audio.is_empty() && fragments.is_empty());
}
