use super::*;

fn hit(target: d::Target) -> d::Effect {
    d::Effect {
        target,
        kind: d::EffectKind::Damage,
        life: 1,
        equipment_revision: 0,
        cell: d::Cell { x: 32769, y: 32769 },
        hp: 100,
        maximum_hp: 100,
        raw: 1,
        applied: 1,
        hp_after: 99,
    }
}
#[test]
fn participant_selector_keeps_source_identity_and_never_retargets_another_life() {
    let data = super::super::battle_payload::data_tests::fixture();
    let mut hit = hit(d::Target::Participant {
        entity: 1,
        identity: data.room.members.0[0].identity.clone(),
    });
    assert_eq!(
        resolved_arrow_target(&data, &hit).unwrap(),
        cell(&data.actors.0[0].position)
    );
    hit.life = 2;
    assert_eq!(resolved_arrow_target(&data, &hit).unwrap(), cell(&hit.cell));
    hit.target = d::Target::Participant {
        entity: 2,
        identity: data.room.members.0[0].identity.clone(),
    };
    assert!(
        resolved_arrow_target(&data, &hit).is_err(),
        "equal SQL object IDs in two sources cannot alias"
    );
}
#[test]
fn retired_npc_does_not_resolve_equal_player_number_or_another_world() {
    let mut data = super::super::battle_payload::data_tests::fixture();
    let mut hit = hit(d::Target::Npc {
        scope: data.world.identity.clone(),
        serial: 1,
    });
    assert!(resolved_arrow_target(&data, &hit).is_err());
    data.npcs = Some(d::Npcs {
        revision: 1,
        elapsed_millis: 0,
        spawned: 1,
        closed: false,
        actors: d::List(vec![]),
        exchanges: d::List(vec![]),
        graphics: d::List(vec![]),
        names: d::List(vec![]),
    });
    assert_eq!(resolved_arrow_target(&data, &hit).unwrap(), cell(&hit.cell));
    assert_ne!(cell(&hit.cell), cell(&data.actors.0[0].position));
    let d::Target::Npc { scope, .. } = &mut hit.target else {
        panic!()
    };
    scope.generation += 1;
    assert!(resolved_arrow_target(&data, &hit).is_err());
    hit.target = d::Target::TimeTower(data.world.identity.clone());
    assert!(resolved_arrow_target(&data, &hit).is_err());
}
pub(crate) fn without_effects(timeline: &Timeline) -> Timeline {
    let mut control = timeline.clone();
    control.effects.clear();
    control
}
pub(crate) fn pre_index(name: &str) -> usize {
    // Honor revives at its current cell in this fixture. Use the following
    // real accepted MOVE to distinguish the replacement life from the receipt.
    if matches!(name, "arrow-hurt" | "arrow-life") {
        3
    } else {
        2
    }
}
fn path(timeline: &Timeline) -> &LightArrowPath {
    let Playback::Arrow { path, .. } = &timeline.effects[0].playback else {
        panic!("not an arrow")
    };
    path
}
pub(crate) fn verify(name: &str, trace: &[Observation], assets: &Assets, scene: &SceneSnapshot) {
    let cancel = AtomicBool::new(false);
    let t = Instant::now();
    let at = t + Duration::from_secs(1);
    let first = Timeline::new(trace[0].clone(), t);
    let pending = first
        .advance(trace[1].clone(), at, assets, &cancel)
        .unwrap();
    assert_eq!(pending.casts.len(), 1);
    assert!(pending.effects.is_empty());
    assert_eq!(pending.overrides(scene, assets, at).unwrap().len(), 1);
    let cast = pending.casts.values().next().unwrap();
    assert!(cast.kind == Kind::Arrow);
    let impact = &cast.receipt.impact;
    let receipt_cell = cell(&impact.targets.0[0].cell);
    let launch = cast.launch_at;
    let pre_index = pre_index(name);
    let pre = &trace[pre_index];
    let primed = pending
        .advance(
            pre.clone(),
            launch - Duration::from_nanos(1),
            assets,
            &cancel,
        )
        .unwrap();
    assert!(primed.effects.is_empty());
    assert_eq!(primed.casts.len(), 1);
    assert_eq!(
        primed
            .overrides(scene, assets, launch - Duration::from_nanos(1))
            .unwrap()
            .len(),
        usize::from(name != "arrow-hurt")
    );
    let expected_target = arrow_target(pre.data(), impact).unwrap();
    if matches!(name, "arrow-player" | "arrow-npc" | "arrow-hurt") {
        assert_ne!(expected_target, receipt_cell);
    }
    if matches!(name, "arrow-life" | "npc") {
        assert_eq!(expected_target, receipt_cell);
    }
    if name == "arrow-life" {
        assert_ne!(expected_target, cell(&pre.data().actors.0[1].position));
    }
    if name == "npc" {
        assert!(pre.data().npcs.as_ref().unwrap().actors.0.is_empty());
    }
    if name == "arrow-hurt" {
        assert!(
            pre.data().actors.0[0].vital.as_ref().unwrap().hurt_revision > impact.usage.revision
        );
    }
    let mut launched = primed
        .advance(pre.clone(), launch, assets, &cancel)
        .unwrap();
    assert_eq!(launched.effects.len(), 1);
    assert_eq!(launched.next_event, 1);
    assert_eq!(path(&launched).from(), cell(&impact.usage.before.cell));
    assert_eq!(path(&launched).target(), expected_target);
    let immutable_path = path(&launched).clone();
    let first_audio = launched.audio(assets, launch).unwrap();
    assert_eq!(first_audio.events.len(), 1);
    assert_eq!(first_audio.events[0].group.candidates(), &[97]);
    assert_eq!(first_audio.events[0].created, launch);
    assert!(first_audio.events[0]
        .bank
        .as_ref()
        .unwrap()
        .get(97)
        .is_some());
    let mut duplicate = launched
        .advance(pre.clone(), launch, assets, &cancel)
        .unwrap();
    assert_eq!(duplicate.next_event, 1);
    assert_eq!(path(&duplicate), &immutable_path);
    assert_eq!(
        duplicate.audio(assets, launch).unwrap().events[0].sequence,
        first_audio.events[0].sequence
    );
    let later = trace.last().unwrap();
    let at_later = launch + Duration::from_millis(60);
    let moved = launched
        .advance(later.clone(), at_later, assets, &cancel)
        .unwrap();
    assert_eq!(
        path(&moved),
        &immutable_path,
        "after launch movement is not homing"
    );
    if matches!(name, "arrow-player" | "arrow-npc") {
        assert_ne!(arrow_target(later.data(), impact).unwrap(), expected_target);
    }
    let bank = assets.light.as_ref().unwrap();
    let projection = ViewProjection {
        anchor_cell: immutable_path.from(),
        anchor_pixel: super::super::world_map::PixelPoint { x: 400, y: 300 },
    };
    for elapsed in [
        Duration::ZERO,
        Duration::from_millis(60),
        immutable_path.impact_at(),
    ] {
        let mut actual = Canvas::new(800, 600).unwrap();
        actual.clear([20, 30, 40]);
        let mut expected = Canvas::new(800, 600).unwrap();
        expected.clear([20, 30, 40]);
        let mut blank = Canvas::new(800, 600).unwrap();
        blank.clear([20, 30, 40]);
        launched
            .draw_projected(&mut actual, projection, assets, launch + elapsed, &cancel)
            .unwrap();
        bank.draw_effect(&mut expected, &immutable_path, projection, elapsed)
            .unwrap();
        assert_eq!(actual.rgba(), expected.rgba());
        assert_ne!(actual.rgba(), blank.rgba());
        let sample = bank.effect_frame(&immutable_path, elapsed).unwrap();
        assert_eq!(
            sample.graphics_id,
            if elapsed < immutable_path.impact_at() {
                167
            } else {
                219
            }
        );
    }
    // Native vector takes its first step immediately. Adjacent arrival still
    // owns one launch sound even when the very first image is impact219.
    assert_ne!(
        immutable_path.position(Duration::ZERO),
        immutable_path.from()
    );
    if name == "arrow-hurt" || name == "npc" {
        assert_eq!(immutable_path.impact_at(), Duration::ZERO);
    }
    let mut stale = primed
        .advance(
            pre.clone(),
            launch + MAX_AGE + Duration::from_nanos(1),
            assets,
            &cancel,
        )
        .unwrap();
    assert!(stale.effects.is_empty() && stale.skipped);
    assert_eq!(stale.next_event, 0);
    assert!(stale
        .audio(assets, launch + MAX_AGE + Duration::from_nanos(1))
        .unwrap()
        .events
        .is_empty());
    let mut delayed = primed
        .advance(pre.clone(), launch + MAX_AGE, assets, &cancel)
        .unwrap();
    assert_eq!(delayed.effects.len(), 1);
    assert_eq!(delayed.effects[0].started, launch + MAX_AGE);
    assert_eq!(
        delayed.audio(assets, launch + MAX_AGE).unwrap().events[0].created,
        launch + MAX_AGE
    );
    let mut saturated = primed.clone();
    saturated.effects = vec![launched.effects[0].clone(); MAX_FLIGHTS];
    let saturated = saturated
        .advance(pre.clone(), launch, assets, &cancel)
        .unwrap();
    assert_eq!(saturated.effects.len(), MAX_FLIGHTS);
    assert!(saturated.skipped);
    assert_eq!(saturated.next_event, 0);
    let baseline = Timeline::new(trace[1].clone(), at)
        .advance(trace[1].clone(), launch, assets, &cancel)
        .unwrap();
    assert!(baseline.casts.is_empty() && baseline.effects.is_empty());
    assert!(primed
        .advance(
            pre.clone(),
            launch - Duration::from_nanos(2),
            assets,
            &cancel
        )
        .is_err());
    assert!(primed
        .advance(pre.clone(), launch, assets, &AtomicBool::new(true))
        .is_err());
    assert_eq!(primed.next_event, 0);
    let end = launch + bank.effect_duration(&immutable_path);
    let mut finished = moved.advance(later.clone(), end, assets, &cancel).unwrap();
    assert!(finished.effects.is_empty());
    assert!(finished.audio(assets, end).unwrap().events.is_empty());
    assert_eq!(finished.observation.data(), later.data());
}
