mod native77_probe {
    use crate::modern_lito::{
        compositor::Canvas,
        native_combat_animation::EnteredEvent,
        native_disintegrate::{Effects, Launch},
        native_world::World,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    use std::{
        path::Path,
        sync::{atomic::AtomicBool, Arc},
    };
    fn put(id: u32) -> Vec<u8> {
        let mut b = vec![0x0b];
        b.extend(32768u16.to_le_bytes());
        b.extend(32768u16.to_le_bytes());
        b.extend(id.to_le_bytes());
        b.extend(0u16.to_le_bytes());
        b.extend([0, 2, 0, 0]);
        b.extend(0u32.to_le_bytes());
        b.extend(500u16.to_le_bytes());
        b.extend(b"actor\0\0");
        b.push(0);
        b.extend(0u32.to_le_bytes());
        b.extend([0, 0, 0, 100, 0, 50, 0, 0, 0, 0]);
        b
    }
    fn wire(critical: bool) -> Vec<u8> {
        let s = if critical {
            "231264000000c8000000010207000000e42d0000800080028001800000"
        } else {
            "231264000000c8000000010207000000e22d0000800080028001800000"
        };
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
    fn world(critical: bool) -> World {
        let mut w = World::default();
        assert!(w.accept(&[0x28, 0x33, 0x02, 0]).unwrap());
        assert!(w.accept(&[0xd2]).unwrap());
        w.hp = Some([100, 100]);
        w.mp = Some([200, 200]);
        w.learned_skills.insert(77);
        assert!(w.accept(&put(100)).unwrap());
        assert!(w.accept(&put(200)).unwrap());
        assert!(w.accept(&wire(critical)).unwrap());
        w
    }
    fn event(w: &World) -> EnteredEvent {
        EnteredEvent {
            actor: 100,
            sequence: w.projectiles.last().unwrap().action_sequence,
            action: 18,
            frame: 4,
            event: "Hit".into(),
            sound: None,
            prepared_audio: None,
        }
    }
    fn launch(e: &Effects, o: &Arc<()>, w: &World) -> Launch {
        Launch::capture(e, o, w, &event(w)).unwrap()
    }
    fn identity_only(o: &Arc<()>, w: &World) -> Effects {
        let mut minimal = World::default();
        minimal.generation = w.generation;
        let mut e = Effects::default();
        e.prepare(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            o.clone(),
            &minimal,
            &AtomicBool::new(false),
        )
        .unwrap();
        e
    }
    fn prepared(o: &Arc<()>, w: &World) -> Effects {
        let mut e = Effects::default();
        e.prepare(
            Path::new(r"D:/lineage開發/0-客戶端"),
            o.clone(),
            w,
            &AtomicBool::new(false),
        )
        .unwrap();
        e
    }
    fn step(
        e: &mut Effects,
        o: &Arc<()>,
        w: &World,
        t: u64,
    ) -> Vec<crate::modern_lito::native_projectiles::Impact> {
        e.update(o, w, std::iter::empty(), t).unwrap()
    }
    #[test]
    fn capture_rejects_noncast_nonhit_zero_world_dead_and_old_life() {
        let mut w = world(false);
        let o = Arc::new(());
        let e = identity_only(&o, &w);
        let mut v = event(&w);
        assert!(Launch::capture(&e, &o, &w, &v).is_some());
        v.action = 2;
        assert!(Launch::capture(&e, &o, &w, &v).is_none());
        v.action = 18;
        v.event = "Sound".into();
        assert!(Launch::capture(&e, &o, &w, &v).is_none());
        v.event = "Hit".into();
        w.generation = 0;
        assert!(Launch::capture(&e, &o, &w, &v).is_none());
        w.generation = 1;
        w.actors.get_mut(&100).unwrap().dead = true;
        assert!(Launch::capture(&e, &o, &w, &v).is_none());
        w.actors.get_mut(&100).unwrap().dead = false;
        w.accept(&put(100)).unwrap();
        assert!(Launch::capture(&e, &o, &w, &v).is_none());
    }
    #[test]
    fn capture_selects_current_life_even_when_stale_tuple_precedes_it() {
        let mut w = world(false);
        let o = Arc::new(());
        let e = identity_only(&o, &w);
        let mut old = w.projectiles[0].clone();
        old.attacker_incarnation += 100;
        w.projectiles.insert(0, old);
        assert!(Launch::capture(&e, &o, &w, &event(&w)).is_some());
    }
    #[test]
    fn cancellation_before_io_retires_and_graphics_are_explicit() {
        let mut e = Effects::default();
        let o = Arc::new(());
        let w = World::default();
        assert!(e
            .prepare(
                Path::new(r"Z:\absent-native77"),
                o.clone(),
                &w,
                &AtomicBool::new(true)
            )
            .is_err());
        assert!(e.update(&o, &w, std::iter::empty(), 0).is_err());
        assert!(e.draws(&o).is_err());
        for g in [11746, 11748] {
            assert!(crate::modern_lito::native_disintegrate::supports_graphics(
                g
            ));
        }
        for g in [0, 1815, 11747, 11749, 11736] {
            assert!(!crate::modern_lito::native_disintegrate::supports_graphics(
                g
            ));
        }
    }
    #[test]
    #[ignore = "read-only actual TW assets; synthetic native world and timer, no live cast"]
    fn actual_marker_timing_once_normal_and_critical() {
        for critical in [false, true] {
            let w = world(critical);
            let original = serde_json::to_value(&w).unwrap();
            let o = Arc::new(());
            let mut e = prepared(&o, &w);
            let l = launch(&e, &o, &w);
            assert!(step(&mut e, &o, &w, 0).is_empty());
            assert!(e.draws(&o).unwrap().is_empty());
            assert!(e
                .update(&o, &w, [&l].into_iter(), 1_000_000)
                .unwrap()
                .is_empty());
            assert_eq!(e.events().len(), 1);
            assert_eq!(e.events()[0].sound, 717);
            assert_eq!(e.events()[0].source_frame, 1);
            let d = e.draws(&o).unwrap();
            assert_eq!(d.len(), 1);
            assert_eq!(d[0].graphics, if critical { 11748 } else { 11746 });
            assert_eq!(d[0].source_frame, 1);
            assert_eq!(d[0].position(), WorldCell { x: 32770, y: 32769 });
            assert!(e
                .update(&o, &w, [&l].into_iter(), 1_000_000)
                .unwrap()
                .is_empty());
            assert!(e.events().is_empty());
            assert_eq!(e.snapshot()["started"], 1);
            assert!(step(&mut e, &o, &w, 83_000_000).is_empty());
            assert_eq!(e.draws(&o).unwrap()[0].source_frame, 1);
            assert!(step(&mut e, &o, &w, 84_000_000).is_empty());
            assert_eq!(e.draws(&o).unwrap()[0].source_frame, 2);
            assert!(step(&mut e, &o, &w, 166_000_000).is_empty());
            let impacts = step(&mut e, &o, &w, 167_000_000);
            assert_eq!(impacts.len(), 1);
            assert_eq!(impacts[0].target, 200);
            assert_eq!(impacts[0].projectile, 7);
            assert!(step(&mut e, &o, &w, 167_000_000).is_empty());
            assert_eq!(e.snapshot()["hit_entries"], 1);
            let mut frames = vec![3];
            for n in 1..=16 {
                step(&mut e, &o, &w, 167_000_000 + n * 1_000_000_000);
                if let Some(d) = e.draws(&o).unwrap().first() {
                    frames.push(d.source_frame);
                }
            }
            assert_eq!(frames, (3..18).chain([0]).collect::<Vec<_>>());
            assert!(e.draws(&o).unwrap().is_empty());
            assert_eq!(e.snapshot()["active"], 0);
            assert_eq!(serde_json::to_value(&w).unwrap(), original);
        }
    }
    #[test]
    #[ignore = "read-only actual TW assets; synthetic actor lifecycle, no live cast"]
    fn actual_frozen_position_target_life_and_caster_retirement() {
        for mode in 0..4 {
            let mut w = world(false);
            let o = Arc::new(());
            let mut e = prepared(&o, &w);
            let l = launch(&e, &o, &w);
            e.update(&o, &w, [&l].into_iter(), 0).unwrap();
            w.actors.get_mut(&100).unwrap().dead = true;
            match mode {
                0 => {
                    let t = w.actors.get_mut(&200).unwrap();
                    t.x += 20;
                    t.y += 30;
                }
                1 => {
                    w.actors.remove(&200);
                }
                2 => {
                    w.accept(&put(200)).unwrap();
                }
                _ => {
                    w.actors.get_mut(&200).unwrap().dead = true;
                }
            }
            let authority = serde_json::to_value(&w).unwrap();
            step(&mut e, &o, &w, 83_000_000);
            let hits = step(&mut e, &o, &w, 166_000_000);
            assert_eq!(hits.len(), usize::from(mode == 0));
            assert_eq!(
                e.draws(&o).unwrap()[0].position(),
                WorldCell { x: 32770, y: 32769 }
            );
            assert_eq!(serde_json::to_value(&w).unwrap(), authority);
        }
    }
    #[test]
    #[ignore = "read-only actual TW assets; synthetic owner/world ABA, no live cast"]
    fn actual_stale_generation_owner_and_life_stamps_never_rebind() {
        let mut w = world(false);
        let a = Arc::new(());
        let b = Arc::new(());
        let mut e = prepared(&a, &w);
        let old = launch(&e, &a, &w);
        w.generation += 1;
        e.update(&a, &w, [&old].into_iter(), 0).unwrap();
        assert_eq!(e.snapshot()["started"], 0);
        let fresh = launch(&e, &a, &w);
        e.update(&a, &w, [&fresh].into_iter(), 0).unwrap();
        assert_eq!(e.snapshot()["active"], 1);
        e.prepare(
            Path::new(r"D:/lineage開發/0-客戶端"),
            b.clone(),
            &w,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(e.draws(&a).is_err());
        assert!(e.update(&a, &w, [&fresh].into_iter(), 0).is_err());
        let b_stamp = launch(&e, &b, &w);
        e.update(&b, &w, [&fresh, &b_stamp].into_iter(), 0).unwrap();
        assert_eq!(e.snapshot()["started"], 0);
        e.prepare(
            Path::new(r"D:/lineage開發/0-客戶端"),
            a.clone(),
            &w,
            &AtomicBool::new(false),
        )
        .unwrap();
        e.update(&a, &w, [&fresh].into_iter(), 0).unwrap();
        assert_eq!(e.snapshot()["started"], 0);
        w.accept(&put(100)).unwrap();
        w.accept(&wire(false)).unwrap();
        let new = launch(&e, &a, &w);
        e.update(&a, &w, [&fresh, &new].into_iter(), 0).unwrap();
        assert_eq!(e.snapshot()["started"], 1);
        let before = e.snapshot();
        assert!(e.update(&a, &w, std::iter::empty(), 0).is_ok());
        step(&mut e, &a, &w, 10);
        let advanced = e.snapshot();
        assert!(e.update(&a, &w, std::iter::empty(), 9).is_err());
        assert_eq!(e.snapshot(), advanced);
        assert_eq!(before["active"], 1);
        assert!(e
            .prepare(
                Path::new(r"Z:\missing-after-cancel"),
                a.clone(),
                &w,
                &AtomicBool::new(true)
            )
            .is_err());
        assert_eq!(e.snapshot()["active"], 0);
        assert_eq!(e.snapshot()["assets_ready"], false);
        assert!(e.draws(&a).is_err());
    }
    #[test]
    #[ignore = "read-only actual TW assets; synthetic queue bounds and dispatch, no live cast"]
    fn actual_budget_and_late_timer_do_not_catch_up() {
        let mut w = world(false);
        let o = Arc::new(());
        let mut e = prepared(&o, &w);
        let mut stamps = vec![launch(&e, &o, &w)];
        for _ in 1..65 {
            w.accept(&wire(false)).unwrap();
            stamps.push(launch(&e, &o, &w));
        }
        e.update(&o, &w, stamps.iter(), 0).unwrap();
        assert_eq!(e.snapshot()["active"], 64);
        assert_eq!(e.snapshot()["omitted"], 1);
        assert_eq!(e.events().len(), 64);
        assert!(step(&mut e, &o, &w, 60_000_000_000).is_empty());
        assert!(e.draws(&o).unwrap().iter().all(|d| d.source_frame == 2));
        assert!(e.events().is_empty());
        assert_eq!(step(&mut e, &o, &w, 60_083_000_000).len(), 64);
        assert!(step(&mut e, &o, &w, 60_083_000_000).is_empty());
        e.update(&o, &w, stamps.iter(), 60_083_000_000).unwrap();
        assert_eq!(e.snapshot()["started"], 64);
        let saved = e.snapshot();
        w.projectiles.resize(4097, w.projectiles[0].clone());
        assert!(e
            .update(&o, &w, std::iter::empty(), 61_000_000_000)
            .is_err());
        assert_eq!(e.snapshot(), saved);
    }
    #[test]
    #[ignore = "read-only actual TW pixels; canvas wiring only, not native scene or GUI"]
    fn actual_canvas_screen_then_multiply_and_projection_preflight() {
        for critical in [false, true] {
            let w = world(critical);
            let o = Arc::new(());
            let mut e = prepared(&o, &w);
            let l = launch(&e, &o, &w);
            e.update(&o, &w, [&l].into_iter(), 0).unwrap();
            let draws = e.draws(&o).unwrap();
            let d = &draws[0];
            let (main, layer) = d.frames();
            let view = ViewProjection {
                anchor_cell: d.position(),
                anchor_pixel: PixelPoint { x: 300, y: 300 },
            };
            let mut result = Canvas::new(600, 600).unwrap();
            result.clear([82, 115, 148]);
            d.draw_canvas(&mut result, view).unwrap();
            let mut expected = Canvas::new(600, 600).unwrap();
            expected.clear([82, 115, 148]);
            let base = expected.rgba().to_vec();
            expected
                .blit_screen_rgb555(
                    &main.rgba,
                    main.width,
                    main.height,
                    300 + main.x_offset,
                    300 + main.y_offset,
                )
                .unwrap();
            let screen = expected.rgba().to_vec();
            expected
                .blit_multiply_rgb555(
                    &layer.rgba,
                    layer.width,
                    layer.height,
                    300 + layer.x_offset,
                    300 + layer.y_offset,
                )
                .unwrap();
            assert_eq!(result.rgba(), expected.rgba());
            assert_ne!(base, screen);
            assert_ne!(screen, expected.rgba());
            let before = result.rgba().to_vec();
            let bad = ViewProjection {
                anchor_cell: WorldCell {
                    x: i32::MIN,
                    y: i32::MIN,
                },
                anchor_pixel: PixelPoint { x: 0, y: 0 },
            };
            assert!(d.draw_canvas(&mut result, bad).is_err());
            assert_eq!(result.rgba(), before);
        }
    }
    #[test]
    #[ignore = "read-only actual TW assets; preparation lifecycle only, no live cast"]
    fn actual_prepare_generation_and_failed_source_retire_before_draw() {
        let mut w = world(false);
        let o = Arc::new(());
        let mut e = prepared(&o, &w);
        let l = launch(&e, &o, &w);
        e.update(&o, &w, [&l].into_iter(), 0).unwrap();
        assert_eq!(e.draws(&o).unwrap().len(), 1);
        w.generation += 1;
        e.prepare(
            Path::new(r"D:/lineage開發/0-客戶端"),
            o.clone(),
            &w,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(e.draws(&o).unwrap().is_empty());
        let l = launch(&e, &o, &w);
        e.update(&o, &w, [&l].into_iter(), 0).unwrap();
        assert_eq!(e.draws(&o).unwrap().len(), 1);
        let missing = Path::new(
            r"D:/lineage開發/3-lito開發/lito重建版/LiTo381-Modern/target/current/absent-native77-root",
        );
        assert!(!missing.exists());
        assert!(e
            .prepare(missing, Arc::new(()), &w, &AtomicBool::new(false))
            .is_err());
        assert_eq!(e.snapshot()["active"], 0);
        assert_eq!(e.snapshot()["assets_ready"], false);
        assert!(e.draws(&o).is_err());
    }
    #[test]
    #[ignore = "read-only actual TW assets; publication retirement and same-owner retry, no live cast"]
    fn actual_cancel_and_failed_source_old_publication_cannot_relaunch() {
        for fail_source in [false, true] {
            let w = world(false);
            let o = Arc::new(());
            let mut e = prepared(&o, &w);
            let old = launch(&e, &o, &w);
            e.update(&o, &w, [&old].into_iter(), 0).unwrap();
            assert_eq!(e.snapshot()["started"], 1);
            if fail_source {
                let missing = Path::new(
                    r"D:/lineage開發/3-lito開發/lito重建版/LiTo381-Modern/target/current/absent-native77-root",
                );
                assert!(!missing.exists());
                assert!(e
                    .prepare(missing, o.clone(), &w, &AtomicBool::new(false))
                    .is_err());
            } else {
                assert!(e
                    .prepare(
                        Path::new(r"D:/lineage開發/0-客戶端"),
                        o.clone(),
                        &w,
                        &AtomicBool::new(true)
                    )
                    .is_err());
            }
            assert!(Launch::capture(&e, &o, &w, &event(&w)).is_none());
            e.prepare(
                Path::new(r"D:/lineage開發/0-客戶端"),
                o.clone(),
                &w,
                &AtomicBool::new(false),
            )
            .unwrap();
            e.update(&o, &w, [&old].into_iter(), 0).unwrap();
            assert_eq!(e.snapshot()["started"], 0);
            assert!(e.draws(&o).unwrap().is_empty());
            let new = launch(&e, &o, &w);
            e.update(&o, &w, [&old, &new].into_iter(), 0).unwrap();
            assert_eq!(e.snapshot()["started"], 1);
            assert_eq!(e.events().len(), 1);
        }
    }
    #[test]
    fn canonical_hurt_waits_for_effect_while_legacy_fallback_is_preserved() {
        let mut w = world(false);
        assert!(w.actors[&200].actions.is_empty());
        w.accept(&wire(true)).unwrap();
        assert!(w.actors[&200].actions.is_empty());
        let mut legacy = wire(false);
        legacy[16..18].copy_from_slice(&1815u16.to_le_bytes());
        w.accept(&legacy).unwrap();
        assert_eq!(w.actors[&200].actions.last().unwrap().action, 2);
    }
    #[test]
    #[ignore = "read-only actual TW bank; simulated player verifies delivery, no audio device or live cast"]
    fn actual_prepared_payload_publication_and_history_once() {
        use crate::modern_lito::{
            combat_animation::NativeAudioGroup,
            combat_audio::{PlayOutcome, PoolError, SoundBank},
            native_audio_delivery::{
                DeliveryOutcome, FiniteDeliveryHistory, Player, PreparedCue, Scope,
            },
        };
        struct Sink(usize);
        impl Player for Sink {
            fn play(
                &mut self,
                bank: &SoundBank,
                group: &NativeAudioGroup,
                selection: u32,
            ) -> Result<PlayOutcome, PoolError> {
                self.0 += 1;
                assert_eq!(group.selected(selection), Some(717));
                assert!(bank.get(717).is_some());
                Ok(PlayOutcome::Started {
                    sound_id: 717,
                    slot: 0,
                })
            }
        }
        let w = world(false);
        let o = Arc::new(());
        let mut e = prepared(&o, &w);
        let l = launch(&e, &o, &w);
        e.update(&o, &w, [&l].into_iter(), 1).unwrap();
        let cues = e.entered_events();
        let payload = cues[0].prepared_audio.clone().unwrap();
        assert_eq!(payload.finite_event().unwrap().source_frame, 1);
        assert_eq!(payload.finite_event().unwrap().graphics, 11746);
        assert_eq!(payload.prepared_sound().unwrap().pcm_bytes().len(), 188158);
        let publication = Arc::new(());
        let scope = Scope::new(1, w.generation, publication.clone());
        payload.bind(scope.clone()).unwrap();
        let mut sink = Sink(0);
        assert!(matches!(
            payload.submit(&scope, &mut sink).unwrap(),
            DeliveryOutcome::Played(_)
        ));
        assert_eq!(
            payload.clone().submit(&scope, &mut sink).unwrap(),
            DeliveryOutcome::AlreadyConsumed
        );
        assert_eq!(sink.0, 1);
        let mut fresh = prepared(&o, &w);
        let l = launch(&fresh, &o, &w);
        fresh.update(&o, &w, [&l].into_iter(), 0).unwrap();
        let second = fresh.entered_events()[0].prepared_audio.clone().unwrap();
        let mut history = FiniteDeliveryHistory::default();
        history.select_world((1, w.generation, 563));
        let mut first = vec![payload];
        assert_eq!(history.filter_frame(&mut first).unwrap().kept, 1);
        let mut repeated = vec![second.clone()];
        assert_eq!(history.filter_frame(&mut repeated).unwrap().duplicates, 1);
        assert!(repeated.is_empty());
        second.bind(scope.clone()).unwrap();
        assert_eq!(
            second.submit(&scope, &mut sink).unwrap(),
            DeliveryOutcome::AlreadyConsumed
        );
        let mut again = prepared(&o, &w);
        let l = launch(&again, &o, &w);
        again.update(&o, &w, [&l].into_iter(), 0).unwrap();
        let stale: PreparedCue = again.entered_events()[0].prepared_audio.clone().unwrap();
        stale.bind(scope.clone()).unwrap();
        let new_scope = Scope::new(1, w.generation, Arc::new(()));
        assert_eq!(
            stale.submit(&new_scope, &mut sink).unwrap(),
            DeliveryOutcome::Stale
        );
        assert_eq!(sink.0, 1);
        // Global native milliseconds, including a fractional launch instant.
        step(&mut e, &o, &w, 83_000_000);
        assert_eq!(e.draws(&o).unwrap()[0].source_frame, 2);
    }
    #[test]
    #[ignore = "read-only actual TW assets; model admission handoff only, no live cast"]
    fn actual_model_rebuild_restores_only_admission_not_pixels_or_callbacks() {
        let w = world(false);
        let a = Arc::new(());
        let mut old = prepared(&a, &w);
        let l = launch(&old, &a, &w);
        old.update(&a, &w, [&l].into_iter(), 0).unwrap();
        let h = old.history();
        let b = Arc::new(());
        let mut rebuilt = Effects::default();
        rebuilt.restore_history(h);
        rebuilt
            .prepare(
                Path::new(r"D:/lineage開發/0-客戶端"),
                b.clone(),
                &w,
                &AtomicBool::new(false),
            )
            .unwrap();
        assert!(rebuilt.draws(&b).unwrap().is_empty());
        let replay = launch(&rebuilt, &b, &w);
        rebuilt.update(&b, &w, [&replay].into_iter(), 0).unwrap();
        assert_eq!(rebuilt.snapshot()["started"], 0);
        assert!(rebuilt.events().is_empty());
        let mut next = w.clone();
        next.accept(&wire(false)).unwrap();
        let l = launch(&rebuilt, &b, &next);
        rebuilt.update(&b, &next, [&l].into_iter(), 0).unwrap();
        assert_eq!(rebuilt.snapshot()["started"], 1);
    }
    #[test]
    #[ignore = "read-only actual TW fragments; native scene raster path, no GUI"]
    fn actual_fragment_submission_uses_screen_multiply_and_remains_click_through() {
        use crate::modern_lito::{
            native_world_effects::{Mix, Target},
            world_depth::NativeDrawCell,
            world_raster::{Blend, Raster},
        };
        for critical in [false, true] {
            let w = world(critical);
            let o = Arc::new(());
            let mut e = prepared(&o, &w);
            let l = launch(&e, &o, &w);
            e.update(&o, &w, [&l].into_iter(), 0).unwrap();
            let draws = e.draws(&o).unwrap();
            let d = &draws[0];
            let view = ViewProjection {
                anchor_cell: d.position(),
                anchor_pixel: PixelPoint { x: 300, y: 300 },
            };
            let mut queue = Vec::new();
            d.submit_to(&mut Target::Fragments(&mut queue), view)
                .unwrap();
            assert_eq!(queue.len(), 2);
            assert!(matches!(queue[0].mix, Mix::Screen));
            assert!(matches!(queue[1].mix, Mix::Multiply));
            let mut actual = Canvas::new(600, 600).unwrap();
            actual.clear([82, 115, 148]);
            let base = actual.rgba().to_vec();
            let mut raster = Raster::new(view, &actual);
            crate::modern_lito::native_world_layers::enqueue_effects(
                &mut raster,
                &queue,
                &AtomicBool::new(false),
            )
            .unwrap();
            let visibility = raster
                .paint(&mut actual, &AtomicBool::new(false), 1)
                .unwrap();
            assert!((0..600).all(|y| (0..600).all(|x| !visibility[0].contains(x, y))));
            let mut expected = Canvas::new(600, 600).unwrap();
            expected.clear([82, 115, 148]);
            let mut direct = Raster::new(view, &expected);
            let (main, layer) = d.frames();
            let origin = NativeDrawCell::from_world(d.position()).unwrap();
            direct.sprite(origin, main, 0, Blend::Screen).unwrap();
            direct.sprite(origin, layer, 0, Blend::Multiply).unwrap();
            direct
                .paint(&mut expected, &AtomicBool::new(false), 0)
                .unwrap();
            assert_eq!(actual.rgba(), expected.rgba());
            assert_ne!(actual.rgba(), base);
            let mut canvas = Canvas::new(600, 600).unwrap();
            canvas.clear([82, 115, 148]);
            d.submit_to(&mut Target::Canvas(&mut canvas), view).unwrap();
            let mut flat = Canvas::new(600, 600).unwrap();
            flat.clear([82, 115, 148]);
            d.draw_canvas(&mut flat, view).unwrap();
            assert_eq!(canvas.rgba(), flat.rgba());
        }
    }
}
