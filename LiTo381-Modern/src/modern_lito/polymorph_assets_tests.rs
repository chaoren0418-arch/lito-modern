use super::*;
use crate::modern_lito::sprite_actions::{
    ActionDefinition, Directionality, FrameEvent, OpaqueAttribute,
};
use sha2::{Digest, Sha256};

// Literal source-frame routes independently read with the existing C# PakFile.
// Labels are immaterial; FPS scopes, images, subframes and ordered cues are not.
pub(super) fn fixture() -> SpriteActionCatalog {
    let early = "0.walk(1 4,8.0:4 8.1:4 8.2:4 8.3:4)\n1.attack(1 6,16.0:2 16.1:2 16.2:4<86 16.3:1 16.4:1 16.5:5!)\n";
    let late = "3.Breath(1 6,0.0:5 0.1:4 0.2:4 0.3:4 0.4:4 0.5:4)\n2.Damage(1 3,0.0:4 48.0:6[88 48.1:2)\n8.Death(1 7,56.6:6 56.0:3[89 56.1:3 56.2:4 56.3:4 56.4:3 56.5:3)\n";
    let attrs = "101.shadow(241)102.type(10)105.clothes(1 242)\n";
    SpriteActionCatalog::parse_for_inspection(format!(
        "#240 64 bank\n{early}{late}{attrs}#241 64 shadow\n102.type(0)\n#242 64 clothes\n102.type(0)104.attr(8)\n#5641 64=240 form\n110.fps(48)\n{early}110.fps(24)\n{late}{attrs}"
    ).as_bytes(), Default::default()).unwrap()
}
fn form() -> PolymorphForm {
    PolymorphForm::from_wire(5641).unwrap().unwrap()
}
#[test]
fn body_cues_follow_integer_ms_source_entries_and_never_hit_markers() {
    let catalog = fixture();
    for (action, phase, source, sound, terminal) in [
        (BaseAction::Attack, 41, 2, 86, 268),
        (BaseAction::Damage, 0, 1, 88, 333),
        (BaseAction::Death, 0, 1, 89, 832),
    ] {
        let q = QualifiedPolymorphAction::from_catalog(&catalog, form(), action).unwrap();
        let cues = q.audio_phases().unwrap();
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].0, Duration::from_millis(phase));
        assert_eq!(cues[0].1.candidates(), &[sound]);
        for heading in 0..8 {
            assert_eq!(q.sample(cues[0].0, heading).unwrap().0, source);
            if phase > 0 {
                assert_eq!(
                    q.sample(cues[0].0 - Duration::from_nanos(1), heading)
                        .unwrap()
                        .0,
                    source - 1
                );
            }
            assert_eq!(
                q.sample(Duration::from_millis(terminal), heading)
                    .unwrap()
                    .0,
                0
            );
            assert_eq!(q.sample(Duration::MAX, heading).unwrap(), (0, 0));
        }
        assert!(q.source_duration(q.source_frames().len()).is_err());
    }
    for action in [BaseAction::Walk, BaseAction::Idle] {
        assert!(
            QualifiedPolymorphAction::from_catalog(&catalog, form(), action)
                .unwrap()
                .audio_phases()
                .is_err()
        );
    }
}
fn animation(c: &mut SpriteActionCatalog, id: u32, action: BaseAction) -> &mut Animation {
    match c
        .sprites
        .get_mut(&id)
        .unwrap()
        .actions
        .get_mut(&action.action_key())
        .unwrap()
    {
        ActionDefinition::Animation(a) => a,
        _ => panic!("fixture action"),
    }
}
fn qualify_all(c: &SpriteActionCatalog) -> Result<Vec<QualifiedPolymorphAction>> {
    ACTIONS
        .into_iter()
        .map(|a| QualifiedPolymorphAction::from_catalog(c, form(), a))
        .collect()
}

#[test]
fn wide_wire_ids_never_wrap_into_a_supported_form_or_sprite_zero() {
    assert_eq!(PolymorphForm::from_wire(0).unwrap(), None);
    assert_eq!(form().logical_sprite_id(), 5641);
    assert_eq!(form().graphics_id(), 240);
    for id in [
        -1,
        i64::MIN,
        240,
        241,
        5642,
        65536 + 5641,
        4_294_967_296 + 5641,
        4_294_967_301,
        i64::MAX,
    ] {
        assert!(PolymorphForm::from_wire(id).is_err(), "{id}");
    }
}

#[test]
fn reviewed_missing_weapon_families_reset_to_basic_without_dropping_equipment() {
    let catalog = fixture();
    for weapon in [0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17] {
        let stance = form().qualify_weapon(&catalog, weapon).unwrap();
        assert_eq!(stance.form(), form());
        assert_eq!(stance.weapon_type(), weapon);
        assert_eq!(
            stance.requested_family(),
            crate::modern_lito::combat_animation::weapon_stance_base(weapon).unwrap()
        );
        assert_eq!(stance.effective_family(), 0);
    }
    for weapon in [8, 9, 18, 255] {
        assert!(form().qualify_weapon(&catalog, weapon).is_err());
    }
    for id in 4..=7 {
        let mut changed = fixture();
        let fake = changed.sprites[&5641].actions[&ActionKey::new(0)].clone();
        changed
            .sprites
            .get_mut(&5641)
            .unwrap()
            .actions
            .insert(ActionKey::new(id), fake);
        assert!(
            form().qualify_weapon(&changed, 1).is_err(),
            "new action{id} must not silently fall back"
        );
    }
}

#[test]
fn logical_fps_and_walk_provenance_are_not_inherited_from_the_pixel_bank() {
    let catalog = fixture();
    let expected = [
        (4, 48, 332),
        (6, 48, 309),
        (3, 24, 499),
        (6, 24, 1038),
        (7, 24, 1082),
    ];
    for (action, (count, fps, ms)) in ACTIONS.into_iter().zip(expected) {
        let q = QualifiedPolymorphAction::from_catalog(&catalog, form(), action).unwrap();
        assert_eq!(q.source_frames().len(), count);
        assert_eq!(q.frames_per_second(), fps);
        assert_eq!(q.duration(), Duration::from_millis(ms));
        assert_eq!(q.form(), form());
        assert_eq!(q.action(), action);
        if action == BaseAction::Walk {
            let w = q.walk().unwrap();
            assert_eq!(
                (
                    w.provenance().requested_sprite_id,
                    w.provenance().definition_sprite_id,
                    w.provenance().graphics_id
                ),
                (5641, 5641, 240)
            );
            assert_eq!(w.step_duration(), q.duration());
        } else {
            assert!(q.walk().is_none());
        }
    }
}

#[test]
fn every_source_and_heading_keeps_its_exact_bank_image_and_subframe() {
    let catalog = fixture();
    let mut bindings = 0;
    for q in qualify_all(&catalog).unwrap() {
        let bank = QualifiedNpcAction::from_catalog(
            &catalog,
            form().bank().unwrap(),
            bank_action(q.action()),
        )
        .unwrap();
        for source in 0..q.source_frames().len() {
            assert_eq!(
                q.sound_group(source).unwrap(),
                bank.sound_group(source).unwrap()
            );
            for heading in 0..8 {
                assert_eq!(
                    q.body_asset(source, heading).unwrap(),
                    bank.body_asset(source, heading).unwrap()
                );
                bindings += 1;
            }
        }
        assert!(q.body_asset(q.source_frames().len(), 0).is_err());
        assert!(q.body_asset(0, 8).is_err());
        assert!(q.sound_group(usize::MAX).is_err());
    }
    assert_eq!(bindings, 208);
}

#[test]
fn source_boundaries_hold_terminal_zero_and_only_idle_repeats() {
    for q in qualify_all(&fixture()).unwrap() {
        for heading in 0..8 {
            let mut time = Duration::ZERO;
            for source in (1..q.durations.len()).chain(std::iter::once(0)) {
                assert_eq!(q.sample(time, heading).unwrap(), (source, 0));
                let end = time + q.durations[source];
                assert_eq!(
                    q.sample(end - Duration::from_nanos(1), heading).unwrap(),
                    (source, 0)
                );
                time = end;
            }
            let after = if q.action() == BaseAction::Idle {
                (1, 1)
            } else {
                (0, 0)
            };
            assert_eq!(q.sample(time, heading).unwrap(), after);
            assert!(q.sample(Duration::MAX, heading).unwrap().0 < q.source_frames().len());
        }
        assert!(q.sample(Duration::ZERO, 8).is_err());
    }
}

#[test]
fn changed_header_layers_aliases_and_unreviewed_attributes_are_rejected() {
    assert!(qualify_all(&fixture()).is_ok());
    for mutation in 0..13 {
        let mut c = fixture();
        let logical = c.sprites.get_mut(&5641).unwrap();
        match mutation {
            0 => logical.id = 240,
            1 => logical.graphics_id = 5641,
            2 => logical.image_count = Some(63),
            3 => logical.attributes.clear(),
            4 => {
                logical
                    .attributes
                    .iter_mut()
                    .find(|a| a.key.id == 101)
                    .unwrap()
                    .body_bytes = b"242".to_vec()
            }
            5 => logical.attributes.push(logical.attributes[0].clone()),
            6 => {
                logical.actions.insert(
                    ActionKey::new(0),
                    ActionDefinition::Reference {
                        sprite_id: 240,
                        label_bytes: vec![],
                    },
                );
            }
            7 => {
                logical.actions.remove(&ActionKey::new(8));
            }
            8 => {
                c.sprites.remove(&241);
            }
            9 => c.sprites.get_mut(&242).unwrap().attributes[1].body_bytes = b"9".to_vec(),
            10 => c.sprites.get_mut(&240).unwrap().graphics_id = 5641,
            11 => {
                logical.attributes.push(OpaqueAttribute {
                    key: ActionKey::new(108),
                    label_bytes: vec![],
                    body_bytes: b"2".to_vec(),
                });
            }
            _ => {
                logical.actions.insert(
                    ActionKey::new(110),
                    logical.actions[&ActionKey::new(0)].clone(),
                );
            }
        }
        assert!(qualify_all(&c).is_err(), "mutation={mutation}");
    }
}

#[test]
fn changed_frames_fps_controls_and_overflow_cannot_borrow_a_bank_certificate() {
    assert!(qualify_all(&fixture()).is_ok());
    for mutation in 0..12 {
        let mut c = fixture();
        let a = animation(&mut c, 5641, BaseAction::Attack);
        match mutation {
            0 => a.frames_per_second = 24,
            1 => a.frames_per_second = 0,
            2 => a.directionality = Directionality::Undirected,
            3 => a.frames.clear(),
            4 => a.frames[1].image_id = u32::MAX,
            5 => a.frames[1].subframe = 255,
            6 => a.frames[1].duration_ticks = 0,
            7 => a.frames[1].duration_ticks = u32::MAX,
            8 => a.frames[1].events.push(FrameEvent::Skip),
            9 => a.frames[1].events.push(FrameEvent::Hit),
            10 => a.frames[2].events.clear(),
            _ => a.label_bytes = vec![b'x'; 256 * 1024 + 1],
        }
        assert!(qualify_all(&c).is_err(), "mutation={mutation}");
    }
}

#[test]
fn cancellation_fails_before_opening_the_shared_root() {
    let error = PolymorphAssets::load(
        Path::new("X:/r374-nonexistent"),
        form(),
        &AtomicBool::new(true),
    )
    .err()
    .unwrap();
    assert!(format!("{error:#}").contains("cancelled"));
}

#[test]
#[ignore = "explicit shared Lineage5641 body/shadow/attachment and WAV; headless component, not full VIP scene"]
fn real_shared_polymorph_preserves_logical_clock_and_renders_all_routes() {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
    );
    assert!(root.is_absolute());
    let cancel = AtomicBool::new(false);
    super::super::shared_action_catalog::test_counts::reset();
    let context = SharedActionCatalog::load(&root, &cancel).unwrap();
    let assets = PolymorphAssets::load_with_catalog(&context, form(), &cancel).unwrap();
    assert_eq!(context.sha256(), assets.catalog_sha256());
    let sha = assets
        .catalog_sha256()
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<String>();
    assert_eq!(
        sha,
        "523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127"
    );
    assert_eq!(
        super::super::shared_action_catalog::test_counts::snapshot().reads,
        1
    );
    assert_eq!(
        assets.sound_bank().ids().collect::<Vec<_>>(),
        vec![86, 88, 89]
    );
    // No Text/SPR/WAV reads may occur after the construction context is dropped.
    drop(context);
    // One common translation for EVERY source/heading, not a per-frame pivot
    // correction. The death pose extends beyond the original128px preview.
    let (mut left, mut top, mut right, mut bottom) = (0i32, 0i32, 0i32, 0i32);
    for action in ACTIONS {
        for source in 0..assets.action(action).unwrap().source_frames().len() {
            for heading in 0..8 {
                let f = assets.frame(action, heading, source).unwrap();
                for layer in [&f.body, &f.shadow, &f.attachment] {
                    let frame = layer.frame;
                    left = left.min(frame.x_offset);
                    top = top.min(frame.y_offset);
                    right = right.max(frame.x_offset + i32::try_from(frame.width).unwrap());
                    bottom = bottom.max(frame.y_offset + i32::try_from(frame.height).unwrap());
                }
            }
        }
    }
    let (width, height) = (
        u32::try_from(right - left + 16).unwrap(),
        u32::try_from(bottom - top + 16).unwrap(),
    );
    assert!(width <= 512 && height <= 512);
    let anchor = PixelPoint {
        x: 8 - left,
        y: 8 - top,
    };
    let mut canvas = Canvas::new(width * 8, height * 5).unwrap();
    canvas.clear([24, 32, 40]);
    let mut rendered = 0;
    let mut all_pixels = Sha256::new();
    for (row, action) in ACTIONS.into_iter().enumerate() {
        let q = assets.action(action).unwrap();
        let expected = QualifiedPolymorphAction::from_catalog(&fixture(), form(), action).unwrap();
        assert_eq!(q.source_frames(), expected.source_frames());
        assert_eq!(q.duration(), expected.duration());
        for heading in 0..8 {
            for source in 0..q.source_frames().len() {
                let mut tile = Canvas::new(width, height).unwrap();
                tile.clear([24, 32, 40]);
                let f = assets.frame(action, heading, source).unwrap();
                assert_eq!(
                    (f.form, f.action, f.heading, f.source),
                    (form(), action, heading, source)
                );
                assert!(f.body.provenance.name.starts_with("240-"));
                assert!(f.shadow.provenance.name.starts_with("241-"));
                assert!(f.attachment.provenance.name.starts_with("242-"));
                assets
                    .draw_frame(&mut tile, action, heading, source, anchor)
                    .unwrap();
                assert!(tile.rgba().chunks_exact(4).any(|p| p[..3] != [24, 32, 40]),
                    "invisible reference action={action:?} heading={heading} source={source} body={:?} offset=({}, {}) extent={}x{} opaque={}",
                    f.body.asset, f.body.frame.x_offset, f.body.frame.y_offset,
                    f.body.frame.width, f.body.frame.height,
                    f.body.frame.rgba.chunks_exact(4).filter(|p| p[3] != 0).count());
                all_pixels.update(tile.rgba());
                rendered += 1;
            }
            assets
                .draw_frame(
                    &mut canvas,
                    action,
                    heading,
                    1,
                    PixelPoint {
                        x: anchor.x + i32::from(heading) * width as i32,
                        y: anchor.y + row as i32 * height as i32,
                    },
                )
                .unwrap();
        }
    }
    assert_eq!(rendered, 208);
    let before = canvas.rgba().to_vec();
    for (heading, source, anchor) in [
        (8, 0, PixelPoint { x: 0, y: 0 }),
        (0, usize::MAX, PixelPoint { x: 0, y: 0 }),
        (
            0,
            0,
            PixelPoint {
                x: i32::MIN,
                y: i32::MIN,
            },
        ),
    ] {
        assert!(assets
            .draw_frame(&mut canvas, BaseAction::Attack, heading, source, anchor)
            .is_err());
        assert_eq!(canvas.rgba(), before);
    }
    if let Some(path) = std::env::var_os("LITO_POLYMORPH_TEST_PNG") {
        let file = std::fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(file, canvas.width(), canvas.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(canvas.rgba())
            .unwrap();
    }
    let sources = assets.sprite_sources().len();
    for s in assets.sprite_sources() {
        assert!(
            s.name.starts_with("240-") || s.name.starts_with("241-") || s.name.starts_with("242-")
        );
    }
    println!("POLYMORPH5641 logical5641 bank240;5actions/8headings/208frames; SPR={sources} encoded={} RGBA={} WAV=86,88,89; common_preview={width}x{height} anchor={anchor:?}; rendered_sha={:X}; component only, no scene/weapon/SQL/GUI admission", assets.encoded_bytes(), assets.rgba_bytes(), all_pixels.finalize());
}
