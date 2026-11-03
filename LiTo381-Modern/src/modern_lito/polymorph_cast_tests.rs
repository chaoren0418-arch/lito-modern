use super::*;
use crate::modern_lito::sprite_actions::ActionDefinition;
fn form() -> PolymorphForm {
    PolymorphForm::from_wire(5641).unwrap().unwrap()
}
fn fixture() -> SpriteActionCatalog {
    let mut c = super::super::tests::fixture();
    for (id, fps) in [(240, 24), (5641, 48)] {
        let row = format!("#{id} 64\n110.fps({fps})\n18.cast(1 7,32.0:2 32.1:2 32.2:3 32.3:4 32.4:1 32.5:6! 32.6:4)\n19.cast(1 5,40.0:5 40.1:5 40.2:5 40.3:5! 40.4:4)\n");
        let source =
            SpriteActionCatalog::parse_for_inspection(row.as_bytes(), Default::default()).unwrap();
        c.sprites.get_mut(&id).unwrap().actions.insert(
            ActionKey::new(18),
            source.sprites[&id].actions[&ActionKey::new(18)].clone(),
        );
        c.sprites.get_mut(&id).unwrap().actions.insert(
            ActionKey::new(19),
            source.sprites[&id].actions[&ActionKey::new(19)].clone(),
        );
    }
    c
}
#[test]
fn undirected_cast_keeps_action19_routes_word_milliseconds_and_fixed_tail() {
    use crate::modern_lito::{
        battle_motion::finish::{IdleReturn, QualifiedReturn},
        paced_animation::PacedClock,
    };
    let q = QualifiedCast::from_catalog_kind(&fixture(), form(), Kind::Undirected).unwrap();
    assert_eq!(q.launch(), Duration::from_millis(208));
    assert_eq!(q.terminal(), Duration::from_millis(395));
    for heading in 0..8 {
        for (ms, source) in [(0, 1), (104, 2), (208, 3), (312, 4), (395, 0)] {
            assert_eq!(
                q.sample(Duration::from_millis(ms), heading).unwrap(),
                source
            );
            if ms > 0 {
                assert_ne!(
                    q.sample(Duration::from_millis(ms) - Duration::from_nanos(1), heading)
                        .unwrap(),
                    source
                );
            }
            let a = q.asset(source, heading).unwrap();
            assert_eq!(
                (a.graphics_id, a.image_id, a.subframe),
                (240, 40 + u32::from(heading), source as u32)
            );
        }
    }
    let now = std::time::Instant::now();
    let mut clock = PacedClock::new(now);
    let mut tail = IdleReturn::start(QualifiedReturn::polymorph_cast(&q), &clock, now).unwrap();
    assert_eq!(tail.deadline(), now + Duration::from_millis(979));
    let changed = now + Duration::from_millis(100);
    clock.retime(changed, 2, 1).unwrap();
    tail.retime(&clock, changed).unwrap();
    assert_eq!(tail.deadline(), now + Duration::from_micros(831500));
    let ended = now + Duration::from_micros(247500);
    clock.retime(ended, 1, 2).unwrap();
    tail.retime(&clock, ended).unwrap();
    assert_eq!(tail.deadline(), now + Duration::from_micros(831500));
    assert!(q.asset(5, 0).is_err());
    assert!(q.sample(Duration::ZERO, 8).is_err());
}
#[test]
fn undirected_definition_cannot_borrow_directional_assets_or_ignore_controls() {
    for mutation in 0..4 {
        let mut c = fixture();
        let ActionDefinition::Animation(a) = c
            .sprites
            .get_mut(&5641)
            .unwrap()
            .actions
            .get_mut(&ActionKey::new(19))
            .unwrap()
        else {
            panic!()
        };
        match mutation {
            0 => a.frames_per_second = 24,
            1 => a.frames[3].events.clear(),
            2 => a.frames[0].image_id = 32,
            _ => a.frames[4].duration_ticks = 5,
        }
        assert!(QualifiedCast::from_catalog_kind(&c, form(), Kind::Undirected).is_err());
    }
    let mut c = fixture();
    c.sprites
        .get_mut(&5641)
        .unwrap()
        .actions
        .remove(&ActionKey::new(19));
    assert!(QualifiedCast::from_catalog_kind(&c, form(), Kind::Undirected).is_err());
}
#[test]
#[ignore = "explicit shared Lineage action19 SPRs; headless three-layer verification"]
fn real_shared_polymorph_utility_caster_retains_all_five_sources_eight_headings() {
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let cancel = AtomicBool::new(false);
    let c = SharedActionCatalog::load(&root, &cancel).unwrap();
    let a = Assets::load_kind(&c, form(), Kind::Undirected, &cancel).unwrap();
    assert_eq!(a.sources().len(), 24);
    assert_eq!(a.action().terminal(), Duration::from_millis(395));
    let mut empty = Vec::new();
    for heading in 0..8 {
        for (ms, source) in [(0, 1), (104, 2), (208, 3), (312, 4), (395, 0)] {
            let f = a.frame(Duration::from_millis(ms), heading).unwrap();
            assert_eq!(f.source, source);
            for (layer, id) in [(&f.body, 240), (&f.shadow, 241), (&f.attachment, 242)] {
                assert_eq!(
                    (
                        layer.asset.graphics_id,
                        layer.asset.image_id,
                        layer.asset.subframe
                    ),
                    (id, 40 + u32::from(heading), source as u32)
                );
                if id == 240 {
                    assert!(!layer.frame.rgba.is_empty());
                }
                if layer.frame.rgba.is_empty() {
                    empty.push((
                        layer.provenance.name.clone(),
                        source,
                        layer.frame.width,
                        layer.frame.height,
                    ));
                }
            }
        }
    }
    assert!(Assets::load_kind(&c, form(), Kind::Undirected, &AtomicBool::new(true)).is_err());
    assert!(
        empty.is_empty(),
        "reviewed action19 layers are all nonempty"
    );
    eprintln!("POLYMORPH_UTILITY_CAST: 40 shared frame routes/24 SPRs; logical48FPS, marker208/terminal395/idle979ms; all three layers nonempty; no visual Hit authority");
}
#[test]
fn caster_uses_logical_word_ms_and_hit_is_not_an_audio_or_damage_event() {
    let q = QualifiedCast::from_catalog(&fixture(), form()).unwrap();
    assert_eq!(q.launch(), Duration::from_millis(206));
    assert_eq!(q.terminal(), Duration::from_millis(414));
    for heading in 0..8 {
        for (ms, source) in [
            (0, 1),
            (41, 2),
            (103, 3),
            (186, 4),
            (206, 5),
            (331, 6),
            (414, 0),
        ] {
            assert_eq!(
                q.sample(Duration::from_millis(ms), heading).unwrap(),
                source
            );
            if ms > 0 {
                assert_ne!(
                    q.sample(Duration::from_millis(ms) - Duration::from_nanos(1), heading)
                        .unwrap(),
                    source
                );
            }
            let a = q.asset(source, heading).unwrap();
            assert_eq!(
                (a.graphics_id, a.image_id, a.subframe),
                (240, 32 + u32::from(heading), source as u32)
            );
        }
        assert_eq!(q.sample(Duration::MAX, heading).unwrap(), 0);
    }
    assert!(q.sample(Duration::ZERO, 8).is_err());
    assert!(q.asset(7, 0).is_err());
}
#[test]
fn malformed_or_changed_cast_resources_never_fall_back_to_base_or_bank_timing() {
    for which in 0..7 {
        let mut c = fixture();
        let ActionDefinition::Animation(a) = c
            .sprites
            .get_mut(&5641)
            .unwrap()
            .actions
            .get_mut(&ActionKey::new(18))
            .unwrap()
        else {
            panic!()
        };
        match which {
            0 => a.frames_per_second = 24,
            1 => a.frames[5].events.clear(),
            2 => a.frames[5].duration_ticks = 0,
            3 => a.frames[0].image_id = 40,
            4 => a.frames[1].subframe = 7,
            5 => {
                a.frames.remove(1);
            }
            _ => a.directionality = Directionality::Undirected,
        }
        assert!(QualifiedCast::from_catalog(&c, form()).is_err());
    }
    let mut c = fixture();
    c.sprites
        .get_mut(&5641)
        .unwrap()
        .actions
        .remove(&ActionKey::new(18));
    assert!(QualifiedCast::from_catalog(&c, form()).is_err());
}
#[test]
fn accelerated_cast_retimes_the_active_phase_but_not_the_post_terminal_wait() {
    use crate::modern_lito::{
        battle_motion::finish::{IdleReturn, QualifiedReturn},
        paced_animation::PacedClock,
    };
    use std::time::Instant;
    let now = Instant::now();
    let q = QualifiedCast::from_catalog(&fixture(), form()).unwrap();
    let mut clock = PacedClock::new(now);
    let mut tail = IdleReturn::start(QualifiedReturn::polymorph_cast(&q), &clock, now).unwrap();
    assert_eq!(tail.deadline(), now + Duration::from_millis(998));
    let change = now + Duration::from_millis(100);
    clock.retime(change, 2, 1).unwrap();
    tail.retime(&clock, change).unwrap();
    assert_eq!(tail.deadline(), now + Duration::from_millis(841));
    assert_eq!(
        clock.until(change, q.launch()).unwrap(),
        now + Duration::from_millis(153)
    );
    let terminal = now + Duration::from_millis(257);
    assert_eq!(q.sample(clock.peek(terminal).unwrap(), 0).unwrap(), 0);
    clock.retime(terminal, 1, 2).unwrap();
    tail.retime(&clock, terminal).unwrap();
    assert_eq!(tail.deadline(), now + Duration::from_millis(841));
}
#[test]
#[ignore = "explicit shared Lineage action18 SPRs; headless resource verification"]
fn real_shared_polymorph_caster_keeps_three_layers_and_exact_native_milliseconds() {
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let cancel = AtomicBool::new(false);
    let context = SharedActionCatalog::load(&root, &cancel).unwrap();
    let bank = Assets::load(&context, form(), &cancel).unwrap();
    assert_eq!(bank.sources().len(), 24);
    assert!(bank.encoded_bytes() > 0 && bank.rgba_bytes() > 0);
    assert_eq!(bank.action().launch(), Duration::from_millis(206));
    assert_eq!(bank.action().terminal(), Duration::from_millis(414));
    let mut count = 0;
    let mut empty = Vec::new();
    for heading in 0..8 {
        for ms in [0, 41, 103, 186, 206, 331, 414] {
            let f = bank.frame(Duration::from_millis(ms), heading).unwrap();
            assert_eq!(f.form, form());
            for (layer, id) in [(&f.body, 240), (&f.shadow, 241), (&f.attachment, 242)] {
                assert_eq!(
                    (
                        layer.asset.graphics_id,
                        layer.asset.image_id,
                        layer.asset.subframe
                    ),
                    (id, 32 + u32::from(heading), f.source as u32)
                );
                if layer.frame.rgba.is_empty() {
                    empty.push((
                        layer.provenance.name.clone(),
                        f.source,
                        layer.frame.width,
                        layer.frame.height,
                    ));
                }
                if id == 240 {
                    assert!(!layer.frame.rgba.is_empty(), "cast body must be visible");
                }
            }
            count += 1;
        }
    }
    assert!(Assets::load(&context, form(), &AtomicBool::new(true)).is_err());
    // Explicit empty attachment, not a missing SPR or missing subframe.
    assert_eq!(empty, vec![("242-32.spr".to_owned(), 0, 0, 0)]);
    eprintln!("POLYMORPH_CAST: {count} frame routes;24 shared body/shadow/attachment SPRs; logical48FPS; launch206ms/terminal414ms; no audio/hit authority");
}
