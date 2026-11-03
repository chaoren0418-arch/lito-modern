//! Effect-only RGB555 floor-screen regression on the actual shared renderer.
//! The independent complement oracle is a modern presentation policy, not a
//! capture of the native rounding thunk or RGB565 framebuffer. No test opens
//! a window, database, listener or audio device; only the ignored test reads art.
use super::*;
use crate::modern_lito::{
    ammo_protocol, honor_protocol as honor, item_protocol as item, spell_protocol as spell,
    sprite::SpriteFrame, teleport_effect::screen_test_support, timed_protocol as timed,
    weight_protocol as weight, world_map::PixelPoint,
};

#[derive(Clone, Copy, Debug)]
enum Layer {
    Teleport,
    LightArrow,
    Healing,
    Timed,
    HonorHeal,
}

fn initial() -> combat_protocol::Reply {
    let mut reply = super::honor_runtime_tests::actual_fixture();
    let frame = reply.frame.as_mut().unwrap();
    frame.damages = frame
        .world
        .actors
        .iter()
        .map(|a| spell::DamageCastState::initial(a.entity_id, 3))
        .collect();
    frame.ammo = frame
        .world
        .actors
        .iter()
        .map(|a| ammo_protocol::State::initial(a.entity_id, Vec::new()))
        .collect();
    reply.validate().unwrap();
    reply
}

fn input(reply: &combat_protocol::Reply, at: Instant) -> Arc<Input> {
    let frame = reply.frame.as_ref().unwrap();
    let mut input = Input::with_combat(
        Arc::new(frame.world.clone()),
        1,
        Some(Arc::new(frame.clone())),
    )
    .unwrap();
    input.received = at;
    Arc::new(input)
}

/// Full-HP use is a real consumed item/event, with zero applied HP gain. This
/// deliberately prevents health-bar changes from supplying our pixel evidence.
fn potion(
    prior: &combat_protocol::Reply,
    item_id: i64,
    effect: i32,
    delay: i32,
) -> combat_protocol::Reply {
    let mut next = prior.clone();
    let f = next.frame.as_mut().unwrap();
    assert_eq!(f.vitals[0].hp, f.vitals[0].max_hp);
    f.world.revision += 1;
    let s = &mut f.items[0];
    s.request_sequence += 1;
    s.use_count += 1;
    s.use_revision = f.world.revision;
    s.item_id = item_id;
    s.restored = 0;
    s.hp_before = f.vitals[0].hp;
    s.hp_after = s.hp_before;
    s.effect = effect;
    s.delay_millis = delay;
    s.position = f.world.actors[0].position;
    s.stocks
        .iter_mut()
        .find(|s| s.item_id == item_id)
        .unwrap()
        .remaining -= 1;
    // The fixture explicitly qualifies each unit as 1000 raw weight.
    let w = &mut f.weights[0].weight;
    assert!(w.units.iter().all(|u| u.unit_weight == 1000));
    w.supplies_weight -= 1;
    w.total -= 1;
    w.stage = weight::stage(w.total, w.maximum()).unwrap();
    w.hit_penalty = weight::penalty(w.stage);
    let receipt = item::Reply {
        echo: item::Request {
            context: next.echo.context,
            channel_id: next.echo.channel_id,
            action: item::Action::Use,
            match_id: f.world.match_id,
            generation: f.world.generation,
            sequence: s.request_sequence,
            item_id,
        },
        status: item::Status::Ok,
        reason: item::Reason::None,
        frame: Some(f.clone()),
    };
    receipt.validate().unwrap();
    assert!(receipt.confirms(&receipt.echo, Some(prior)));
    next.validate().unwrap();
    assert!(combat_protocol::follows(Some(prior), &next));
    next
}

fn teleport(prior: &combat_protocol::Reply, to: WorldCell) -> combat_protocol::Reply {
    let mut next = spell::tests::success(prior);
    let frame = next.frame.as_mut().unwrap();
    frame.world.actors[0].position = to;
    frame.world.casts[0].to = to;
    next.validate().unwrap();
    assert!(spell::confirms_transition(
        prior.frame.as_ref().unwrap(),
        &next
    ));
    let next = next.combat_reply().unwrap();
    assert!(combat_protocol::follows(Some(prior), &next));
    next
}

fn light_arrow(prior: &combat_protocol::Reply) -> combat_protocol::Reply {
    let mut next = spell::tests::damage_success(prior);
    // The source helper predates the exact regen ledger; preserve its applied
    // ten-HP damage as the same authority event, not a renderer Hit callback.
    next.frame.as_mut().unwrap().regen[1].damage_loss += 10;
    next.validate().unwrap();
    assert!(spell::confirms_transition(
        prior.frame.as_ref().unwrap(),
        &next
    ));
    let next = next.combat_reply().unwrap();
    assert!(combat_protocol::follows(Some(prior), &next));
    next
}

fn audio_ids(pixels: &Pixels) -> Vec<u64> {
    pixels
        .audio
        .as_ref()
        .unwrap()
        .events
        .iter()
        .map(|e| e.sequence)
        .collect()
}

/// Only suppress this effect's draw lifetime. Body overrides, all other
/// families, resources, authority frame, camera and audio log stay installed.
fn control(
    renderer: &mut SharedRenderer,
    layer: Layer,
    scene: &SceneSnapshot,
    frame: &combat_protocol::Frame,
    cancel: &AtomicBool,
    at: Instant,
) -> Pixels {
    macro_rules! without_vector {
        ($family:ident, $field:ident) => {{
            let held = std::mem::take(&mut renderer.$family.as_mut().unwrap().$field);
            let result = renderer.render_at(scene, Some(frame), cancel, at);
            assert!(
                renderer.$family.as_ref().unwrap().$field.is_empty(),
                "same revision cannot respawn an isolated effect"
            );
            renderer.$family.as_mut().unwrap().$field = held;
            result.unwrap()
        }};
    }
    match layer {
        Layer::Healing => without_vector!(healing, active),
        Layer::Timed => without_vector!(timed, active),
        Layer::LightArrow => without_vector!(light_arrow, flights),
        Layer::HonorHeal => without_vector!(honor, effects),
        Layer::Teleport => {
            // TP shares one CastVisual between its 250ms effect and its longer
            // body. Taking the whole family/active map would fake a pixel
            // difference through the body. Keep the independent PacedClock,
            // event and sequence; retire only the effect clock in this control.
            let tp = renderer.teleport.as_mut().unwrap();
            let duration = tp.assets.effect_duration();
            let mut held = Vec::new();
            for (&id, cast) in &mut tp.active {
                let mut silent = tp.assets.start_effect();
                silent.advance(duration).unwrap(); // discard control-only cues
                held.push((
                    id,
                    cast.started,
                    std::mem::replace(&mut cast.effect, silent),
                ));
                cast.started = at.checked_sub(duration).unwrap();
            }
            let result = renderer.render_at(scene, Some(frame), cancel, at);
            let tp = renderer.teleport.as_mut().unwrap();
            for (id, started, playback) in held {
                let cast = tp.active.get_mut(&id).unwrap();
                cast.started = started;
                cast.effect = playback;
            }
            result.unwrap()
        }
    }
}

/// Capture the actual qualified sample and world anchor selected by this
/// renderer. No synthetic SPR, guessed graphics ID or altered timing enters
/// the oracle. Copies are bounded to a single decoded frame per comparison.
fn sample(
    renderer: &SharedRenderer,
    layer: Layer,
    at: Instant,
    projection: ViewProjection,
) -> Option<(SpriteFrame, PixelPoint, u32)> {
    match layer {
        Layer::Teleport => {
            let f = renderer.teleport.as_ref().unwrap();
            let v = f.active.get(&1)?;
            let s = f
                .assets
                .effect_frame(at.saturating_duration_since(v.started))
                .unwrap();
            (!s.finished).then(|| {
                (
                    s.frame.clone(),
                    projection.cell_to_pixel(v.event.from).unwrap(),
                    s.graphics_id,
                )
            })
        }
        Layer::Healing => {
            let f = renderer.healing.as_ref().unwrap();
            let v = f.active.first()?;
            let s = f
                .assets
                .effect_frame(v.effect, at.saturating_duration_since(v.started))
                .unwrap();
            (!s.finished).then(|| {
                (
                    s.frame.clone(),
                    projection.cell_to_pixel(v.position).unwrap(),
                    s.graphics_id,
                )
            })
        }
        Layer::Timed => {
            let f = renderer.timed.as_ref().unwrap();
            let v = f.active.first()?;
            let s = f
                .assets
                .effect_frame(v.effect, at.saturating_duration_since(v.started))
                .unwrap();
            (!s.finished).then(|| {
                (
                    s.frame.clone(),
                    projection.cell_to_pixel(v.position).unwrap(),
                    s.graphics_id,
                )
            })
        }
        Layer::HonorHeal => {
            let f = renderer.honor.as_ref().unwrap();
            let v = f.effects.first()?;
            let s = f
                .healing
                .effect_frame(at.saturating_duration_since(v.started))
                .unwrap();
            (!s.finished).then(|| {
                (
                    s.frame.clone(),
                    projection.cell_to_pixel(v.position).unwrap(),
                    s.graphics_id,
                )
            })
        }
        Layer::LightArrow => {
            let f = renderer.light_arrow.as_ref().unwrap();
            let v = f.flights.first()?;
            let elapsed = at.saturating_duration_since(v.started);
            let s = f.assets.effect_frame(&v.path, elapsed).unwrap();
            (!s.finished).then(|| {
                (
                    s.frame.clone(),
                    projection.cell_to_pixel(v.path.position(elapsed)).unwrap(),
                    s.graphics_id,
                )
            })
        }
    }
}

fn bars(canvas: &mut Canvas, pixels: &Pixels, frame: &combat_protocol::Frame) {
    draw_health_bars(canvas, &pixels.actors, &frame.vitals).unwrap();
    for o in &frame.honor.as_ref().unwrap().objectives {
        let placement = pixels
            .objectives
            .iter()
            .find(|p| p.entity_id == o.entity_id)
            .unwrap();
        draw_objective_health_bar(canvas, placement, o).unwrap();
    }
}

fn bgra(canvas: &Canvas) -> Vec<u8> {
    let mut bytes = Vec::new();
    canvas.copy_bgra(&mut bytes).unwrap();
    bytes
}

fn check_layer(
    renderer: &mut SharedRenderer,
    layer: Layer,
    timeline: &mut Timeline,
    reply: &combat_protocol::Reply,
    started: Instant,
    duration: Duration,
    extra: &[Duration],
    cancel: &AtomicBool,
    observed: &mut std::collections::BTreeSet<u32>,
    sounds: &mut BTreeMap<u32, std::collections::BTreeSet<u64>>,
) {
    let frame = reply.frame.as_ref().unwrap();
    let unchanged = frame.clone();
    // Freeze the SceneSnapshot; both sides still sample any caster override at
    // precisely the same instant, preserving its separate presentation clock.
    let scene = timeline.snapshot(started).unwrap();
    let mut times = vec![
        Duration::ZERO,
        Duration::from_millis(125),
        duration - Duration::from_nanos(1),
        duration,
        duration + Duration::from_nanos(1),
        duration + Duration::from_secs(1),
    ];
    times.extend_from_slice(extra);
    times.sort();
    times.dedup();
    for elapsed in times {
        let at = started + elapsed;
        let actual = renderer.render_at(&scene, Some(frame), cancel, at).unwrap();
        assert!(actual.presentation_notice.is_none());
        let selected = actual
            .actors
            .iter()
            .find(|a| a.entity_id == scene.selected_entity_id())
            .unwrap();
        let selected_scene = scene
            .actors()
            .iter()
            .find(|a| a.entity_id == scene.selected_entity_id())
            .unwrap();
        // WorldScene::render_into_objectives anchors the selected Scene cell at
        // canvas center. A caster override changes frame/offset, never this
        // anchor or cell; TP's effect is separately projected from event.from.
        assert_eq!(selected.cell, selected_scene.cell);
        assert_eq!(
            selected.anchor,
            PixelPoint {
                x: (WIDTH / 2) as i32,
                y: (HEIGHT / 2) as i32
            }
        );
        let projection = ViewProjection {
            anchor_cell: selected.cell,
            anchor_pixel: selected.anchor,
        };
        let sample = sample(renderer, layer, at, projection);
        let control = control(renderer, layer, &scene, frame, cancel, at);
        assert_eq!(
            actual.actors, control.actors,
            "effect isolation cannot change caster/body provenance"
        );
        assert_eq!(
            actual.objectives, control.objectives,
            "tower body and native shadow stay identical"
        );
        assert_eq!(audio_ids(&actual), audio_ids(&control));
        if let Some((sprite, anchor, gfx)) = sample {
            assert!(elapsed < duration);
            // renderer.canvas is the exact control scene. Reapply the same
            // post-effect health bars after the independent per-channel oracle.
            let rgba = screen_test_support::reference(&renderer.canvas, &sprite, anchor);
            let mut expected = Canvas::new(WIDTH, HEIGHT).unwrap();
            expected.blit_rgba(&rgba, WIDTH, HEIGHT, 0, 0).unwrap();
            bars(&mut expected, &control, frame);
            assert_eq!(
                actual.bgra.as_ref(),
                bgra(&expected),
                "{layer:?}/{gfx} independent screen at {elapsed:?}"
            );
            let mut opaque = Canvas::new(WIDTH, HEIGHT).unwrap();
            opaque.copy_from(&renderer.canvas).unwrap();
            crate::modern_lito::teleport_effect::draw_frame(&mut opaque, &sprite, anchor).unwrap();
            bars(&mut opaque, &control, frame);
            if actual.bgra != control.bgra && actual.bgra.as_ref() != bgra(&opaque) {
                observed.insert(gfx);
            }
        } else {
            assert!(elapsed >= duration, "{layer:?} lost its active layer early");
            assert_eq!(
                actual.bgra, control.bgra,
                "retired {layer:?} leaves no decal"
            );
        }
        for event in actual.audio.as_ref().unwrap().events.iter() {
            if event.bank.is_none() {
                continue;
            } // Count effect-owned cues, not body audio.
            for &sound in event.group.candidates() {
                if [119, 97, 109, 121, 142].contains(&sound) {
                    sounds.entry(sound).or_default().insert(event.sequence);
                }
            }
        }
        let repeated = renderer.render_at(&scene, Some(frame), cancel, at).unwrap();
        assert_eq!(
            actual.bgra, repeated.bgra,
            "same frame/time cannot restart or duplicate pixels"
        );
        assert_eq!(
            audio_ids(&actual),
            audio_ids(&repeated),
            "same frame/time cannot re-enqueue sound"
        );
        assert_eq!(
            frame, &unchanged,
            "screen mixing cannot change HP/MP/stock/revisions"
        );
    }
}

#[test]
fn full_hp_consumption_fixture_preserves_exact_ledger_and_does_not_supply_pixel_evidence() {
    let mut before = initial(); // Pure manifest constants; no resource read.
    for (item, effect, delay) in [(40010, 189, 500), (40011, 194, 1000), (40012, 197, 1000)] {
        let next = potion(&before, item, effect, delay);
        let a = before.frame.as_ref().unwrap();
        let b = next.frame.as_ref().unwrap();
        assert_eq!(a.vitals, b.vitals);
        assert_eq!(a.regen, b.regen);
        assert_eq!(b.items[0].restored, 0);
        assert_eq!(b.items[0].use_count, a.items[0].use_count + 1);
        assert_eq!(b.weights[0].weight.total, a.weights[0].weight.total - 1);
        before = next;
    }
}

#[test]
#[ignore = "LITO_SHARED_RESOURCE_ROOT; real map532/actors/towers/all ten attr8 gfx and WAV banks, fixed-clock layer isolation; no GUI/audio device/DB"]
fn actual_ten_attr8_graphics_match_screen_oracle_not_opaque_and_retire_once() {
    let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
    let root = Path::new(&root);
    let cancel = AtomicBool::new(false);
    let mut confirmed = initial();
    let f = confirmed.frame.as_ref().unwrap();
    let mut renderer = load_shared_renderer(root, &f.world, Some(f), &cancel).unwrap();
    let start = Instant::now();
    let mut timeline = Timeline::new(input(&confirmed, start));
    renderer
        .render_at(&timeline.snapshot(start).unwrap(), Some(f), &cancel, start)
        .unwrap();
    let mut seen = std::collections::BTreeSet::new();
    let mut sounds = BTreeMap::new();
    for (i, (item, effect, delay)) in [(40010, 189, 500), (40011, 194, 1000), (40012, 197, 1000)]
        .into_iter()
        .enumerate()
    {
        confirmed = potion(&confirmed, item, effect, delay);
        let at = start + Duration::from_secs(1 + i as u64 * 2);
        timeline.update(input(&confirmed, at)).unwrap();
        check_layer(
            &mut renderer,
            Layer::Healing,
            &mut timeline,
            &confirmed,
            at,
            Duration::from_millis(625),
            &[],
            &cancel,
            &mut seen,
            &mut sounds,
        );
    }
    // Pick a distinct, actually standable adjacent cell, without guessing a
    // numeric-map alias or sending a client-predicted movement command.
    let f = confirmed.frame.as_ref().unwrap();
    let from = f.world.actors[0].position;
    let to = crate::modern_lito::world_map::DIRECTIONS
        .iter()
        .map(|&(x, y)| WorldCell {
            x: from.x + x,
            y: from.y + y,
        })
        .find(|&cell| {
            renderer.map.standing(cell)
                && f.world.actors.iter().all(|a| a.position != cell)
                && f.honor
                    .as_ref()
                    .unwrap()
                    .objectives
                    .iter()
                    .all(|o| o.position != cell)
        })
        .expect("qualified adjacent teleport destination");
    confirmed = teleport(&confirmed, to);
    let at = start + Duration::from_secs(7);
    timeline.update(input(&confirmed, at)).unwrap();
    check_layer(
        &mut renderer,
        Layer::Teleport,
        &mut timeline,
        &confirmed,
        at,
        Duration::from_millis(250),
        &[],
        &cancel,
        &mut seen,
        &mut sounds,
    );

    confirmed = light_arrow(&confirmed);
    let cast_at = start + Duration::from_secs(11);
    timeline.update(input(&confirmed, cast_at)).unwrap();
    let scene = timeline.snapshot(cast_at).unwrap();
    let appearance = scene
        .actors()
        .iter()
        .find(|a| a.entity_id == 1)
        .unwrap()
        .appearance;
    let delay = renderer
        .light_arrow
        .as_ref()
        .unwrap()
        .assets
        .cast_launch_delay(appearance)
        .unwrap();
    renderer
        .render_at(&scene, confirmed.frame.as_ref(), &cancel, cast_at)
        .unwrap();
    assert!(renderer.light_arrow.as_ref().unwrap().flights.is_empty());
    let launched = cast_at + delay;
    renderer
        .render_at(
            &timeline.snapshot(launched).unwrap(),
            confirmed.frame.as_ref(),
            &cancel,
            launched,
        )
        .unwrap();
    let light = renderer.light_arrow.as_ref().unwrap();
    assert_eq!(light.flights.len(), 1);
    let path = light.flights[0].path.clone();
    let duration = light.assets.effect_duration(&path);
    check_layer(
        &mut renderer,
        Layer::LightArrow,
        &mut timeline,
        &confirmed,
        launched,
        duration,
        &[
            path.impact_at(),
            path.impact_at() + Duration::from_millis(125),
        ],
        &cancel,
        &mut seen,
        &mut sounds,
    );

    confirmed = honor::tests::heal(&confirmed, 0).combat_reply().unwrap();
    let at = start + Duration::from_secs(16);
    timeline.update(input(&confirmed, at)).unwrap();
    check_layer(
        &mut renderer,
        Layer::HonorHeal,
        &mut timeline,
        &confirmed,
        at,
        Duration::from_millis(2500),
        &[],
        &cancel,
        &mut seen,
        &mut sounds,
    );

    for (i, kind) in [timed::Kind::Haste, timed::Kind::Brave, timed::Kind::Wisdom]
        .into_iter()
        .enumerate()
    {
        let at = start + Duration::from_secs(21 + i as u64 * 4);
        let prior = confirmed;
        confirmed = timed::tests::use_timed(&prior, kind, (21 + i as i64 * 4) * 1_000_000_000)
            .combat_reply();
        assert!(combat_protocol::follows(Some(&prior), &confirmed));
        timeline.update(input(&confirmed, at)).unwrap();
        let duration = renderer
            .timed
            .as_ref()
            .unwrap()
            .assets
            .duration(kind.effect())
            .unwrap();
        check_layer(
            &mut renderer,
            Layer::Timed,
            &mut timeline,
            &confirmed,
            at,
            duration,
            &[],
            &cancel,
            &mut seen,
            &mut sounds,
        );
    }
    assert_eq!(
        seen,
        [167, 169, 189, 191, 194, 197, 219, 744, 750, 751]
            .into_iter()
            .collect(),
        "each real gfx must visibly differ from both no-effect and old opaque output"
    );
    for id in [119, 97, 109, 121, 142] {
        assert_eq!(
            sounds.get(&id).map(|ids| ids.len()),
            Some(1),
            "sound {id} is owned exactly once"
        );
    }
    assert!(renderer.ranged.as_ref().unwrap().tracker.flights.is_empty());
    let at = start + Duration::from_secs(40);
    let scene = timeline.snapshot(at).unwrap();
    let f = confirmed.frame.as_ref().unwrap();
    renderer.render_at(&scene, Some(f), &cancel, at).unwrap();
    assert!(renderer.teleport.as_ref().unwrap().active.is_empty());
    assert!(renderer.light_arrow.as_ref().unwrap().flights.is_empty());
    assert!(renderer.healing.as_ref().unwrap().active.is_empty());
    assert!(renderer.timed.as_ref().unwrap().active.is_empty());
    assert!(renderer.honor.as_ref().unwrap().effects.is_empty());
    cancel.store(true, Ordering::Release);
    assert!(renderer.render_at(&scene, Some(f), &cancel, at).is_err());
    drop(renderer);

    // A fresh owned renderer sees retained revisions as history, never as new
    // casts/uses. No old sound or decal is imported through its new catalog.
    cancel.store(false, Ordering::Release);
    let mut fresh = load_shared_renderer(root, &f.world, Some(f), &cancel).unwrap();
    let mut late = Timeline::new(input(&confirmed, at));
    let pixels = fresh
        .render_at(&late.snapshot(at).unwrap(), Some(f), &cancel, at)
        .unwrap();
    assert!(pixels.audio.as_ref().unwrap().events.is_empty());
    assert!(fresh.teleport.as_ref().unwrap().active.is_empty());
    assert!(fresh.light_arrow.as_ref().unwrap().flights.is_empty());
    assert!(fresh.healing.as_ref().unwrap().active.is_empty());
    assert!(fresh.timed.as_ref().unwrap().active.is_empty());
    assert!(fresh.honor.as_ref().unwrap().effects.is_empty());
}
