use super::*;
use crate::modern_lito::{
    application_protocol as app, battle_follow::Follower, battle_read, npc_assets::NpcAction,
    protocol,
};
use sha2::{Digest, Sha256};

impl SharedScene {
    /// Existing effect regressions keep their original exact cue counts. Only
    /// the returned test snapshot is filtered; production clocks, mixer and
    /// scene are untouched. Body audio has separate unfiltered Java/art tests.
    fn render_effects(&mut self, now: Instant, cancel: &AtomicBool) -> Result<Surface> {
        let mut surface = self.render(now, cancel)?;
        if let Some(audio) = &mut surface.audio {
            audio.events = audio
                .events
                .iter()
                .filter(|e| !self.audio.is_body_sequence(e.sequence))
                .cloned()
                .collect::<Vec<_>>()
                .into();
        }
        Ok(surface)
    }
}

#[test]
fn objective_clock_rejects_even_sub_counter_future_instants() {
    let now = Instant::now();
    let future = now + Duration::from_nanos(1);
    assert!(future > now);
    let mut clock = ObjectiveClock {
        target: ObjectiveTarget::Honor { entity: 3, team: 0 },
        graphic: 75,
        cell: WorldCell { x: 100, y: 100 },
        action: 35,
        started: Some(future),
    };
    assert!(clock.elapsed(now).is_err());
    assert_eq!(clock.elapsed(future).unwrap(), Duration::ZERO);
    clock.started = None;
    assert_eq!(clock.elapsed(now).unwrap(), Duration::MAX);
}

pub(crate) fn observation(data: d::UnvalidatedData) -> Observation {
    let request = battle_read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: data.personal.player.source as u32,
                object_id: data.personal.player.object as u64,
            },
            epoch: data.personal.player.epoch as u64,
        },
        channel_id: data.personal.player.channel as u8,
        world: data.world.identity.clone(),
    };
    let mut follow = Follower::new(
        request.clone(),
        protocol::Ack {
            request_id: 1,
            version: if data.speed.is_some() {
                47
            } else if data.haste.is_some() {
                44
            } else if data.npc_motion.is_some() {
                43
            } else if data.greater_haste() {
                42
            } else if data.presentation.is_some() {
                36
            } else {
                35
            },
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
    )
    .unwrap();
    follow
        .accept(battle_read::UnvalidatedReply {
            echo: request,
            state: battle_read::State::Data,
            data: Some(data),
        })
        .unwrap();
    follow.observation().unwrap().clone()
}
pub(crate) fn scene_snapshot(o: Observation) -> SceneSnapshot {
    Input::new(o).unwrap().scene
}
#[test]
fn ordinary_status_icon_requires_exact_authoritative_owner_buff() {
    let mut buffs = d::Buffs {
        entity: 1,
        revision: 1,
        observed_nanos: 100,
        haste_until_nanos: 110,
        brave_until_nanos: 0,
        wisdom_until_nanos: 0,
        brave_kind: 0,
        haste_origin: d::HasteOrigin::Haste,
        slow_until_nanos: 0,
        slow_revision: 0,
    };
    assert_eq!(ordinary_status_effect(&buffs, false), Some(0));
    buffs.slow_until_nanos = 120;
    assert_eq!(ordinary_status_effect(&buffs, true), Some(1));
    assert_eq!(ordinary_status_effect(&buffs, false), Some(0));
    buffs.haste_origin = d::HasteOrigin::GreaterHaste;
    assert_eq!(ordinary_status_effect(&buffs, false), None);
    buffs.slow_until_nanos = 100;
    assert_eq!(ordinary_status_effect(&buffs, true), None);
    buffs.haste_origin = d::HasteOrigin::Haste;
    buffs.haste_until_nanos = 100;
    assert_eq!(ordinary_status_effect(&buffs, true), None);
}
#[test]
fn scene_hud_reads_exact_owner_zero_mp_vitals_not_other_actor_or_saved_stats() {
    let mut data = crate::modern_lito::battle_payload::data_tests::fixture();
    let vital = |hp, mp, maximum_mp| d::Vital {
        values: d::Vitals { hp, maximum_hp: 100, mp, maximum_mp },
        ac: 0, weapon_type: 0, attack_sequence: 0, attack_revision: 0,
        hurt_revision: 0, death_revision: 0, target: 0, attack_interval_nanos: 0,
    };
    data.actors.0[0].vital = Some(vital(7, 0, 0));
    data.actors.0[1].vital = Some(vital(90, 80, 100));
    let own = owner_hud_vitals(&data).unwrap();
    assert_eq!((own.hp, own.maximum_hp, own.mp, own.maximum_mp), (7, 100, 0, 0));
    assert!(crate::modern_lito::equipment_transition::Vitals {
        hp: own.hp, maximum_hp: own.maximum_hp, mp: own.mp, maximum_mp: own.maximum_mp,
    }.valid());
    data.personal.entity = 2;
    assert_eq!(owner_hud_vitals(&data).unwrap().mp, 80);
    data.actors.0[1].vital = None;
    assert!(owner_hud_vitals(&data).is_err());
}
#[test]
#[ignore = "explicit LITO_SHARED_RESOURCE_ROOT and LITO_EFFECT_CATALOG_PATH; read-only real shared 5397/480 icon qualification"]
fn ordinary_status_icons_qualify_real_shared_lineage_assets() {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
    );
    let xml = std::path::PathBuf::from(
        std::env::var_os("LITO_EFFECT_CATALOG_PATH").expect("explicit effect XML"),
    );
    let cancel = AtomicBool::new(false);
    let effects = EffectCatalog::load(&xml, &cancel).unwrap();
    let icons = vip_status::Assets::load(&root, effects, &[0, 1], &cancel).unwrap();
    for (effect, graphic) in [(0, 5397), (1, 480)] {
        let icon = icons.picture(EffectId::from_wire(effect).unwrap()).unwrap();
        assert_eq!(icon.graphic, graphic);
        assert_eq!((icon.width, icon.height), (32, 32));
        assert!(icon.rgba().iter().any(|byte| *byte != 0));
    }
    assert!(vip_status::Assets::load(
        &root,
        icons.catalog().clone(),
        &[0, 1],
        &AtomicBool::new(true)
    )
    .is_err());
}
#[test]
fn canonical_scene_retains_exact_engine_owner_and_separate_source_identities() {
    let data = crate::modern_lito::battle_payload::data_tests::fixture();
    let input = Input::new(observation(data.clone())).unwrap();
    assert_eq!(input.scene.revision(), data.world.revision as u64);
    assert_eq!(input.observation.personal().revision(), data.world.revision);
    assert_eq!(input.observation.personal().player(), &data.personal.player);
    assert_eq!(input.scene.world_id(), data.world.identity.match_id as u64);
    assert_eq!(
        input.scene.selected_entity_id(),
        data.personal.entity as u64
    );
    let actors = input.scene.actors();
    assert_eq!(actors[0].object_id, actors[1].object_id);
    assert_ne!(actors[0].source_id, actors[1].source_id);
    assert_eq!(input.bindings[0].sha256, data.manifest.sectors.0[0].sha256);
}
#[test]
fn geometry_uses_server_digest_standability_and_cancellation_not_local_rehash_trust() {
    let mut data = crate::modern_lito::battle_payload::data_tests::fixture();
    let bytes = vec![0; 32768 + 2 + 16384 + 12 + 2 + 2];
    let key = SharedMapSectorKey {
        map_id: 504,
        sector_x: 0x8000,
        sector_y: 0x8000,
    };
    data.manifest.sectors.0[0].byte_length = bytes.len() as i32;
    data.manifest.sectors.0[0].sha256 = Sha256::digest(&bytes).into();
    let map = WorldMap::from_sector_bytes(
        vec![super::super::world_map::SectorInput { key, bytes }],
        &AtomicBool::new(false),
    )
    .unwrap();
    data.manifest.digest = map.canonical_digest();
    let input = Input::new(observation(data.clone())).unwrap();
    input.check_map(&map, &AtomicBool::new(false)).unwrap();
    assert!(input.check_map(&map, &AtomicBool::new(true)).is_err());
    data.manifest.digest[0] ^= 1;
    assert!(Input::new(observation(data))
        .unwrap()
        .check_map(&map, &AtomicBool::new(false))
        .is_err());
}
#[test]
fn aborted_observation_has_no_scene_and_cancelled_load_does_not_open_a_root() {
    let mut data = crate::modern_lito::battle_payload::data_tests::fixture();
    let obs = observation(data.clone());
    assert!(SharedScene::load(
        Path::new("missing-relative-root"),
        obs,
        &AtomicBool::new(true)
    )
    .is_err());
    data.world.phase = d::Phase::Aborted;
    data.world.abort = d::Abort::ServerAbort;
    assert!(Input::new(observation(data)).is_err());
}

#[test]
fn independently_accepted_observation_still_requires_same_consumer_timeline() {
    let mut data = crate::modern_lito::battle_payload::data_tests::fixture();
    let old = observation(data.clone());
    data.world.loading_elapsed_nanos = 100;
    let next = observation(data.clone());
    old.check_successor(&next).unwrap();
    next.check_successor(&next).unwrap();
    assert!(next.check_successor(&old).is_err());
    data.personal.player.epoch = 17;
    assert!(next.check_successor(&observation(data)).is_err());
}

fn prop(hp: i64) -> Prop {
    Prop {
        target: ObjectiveTarget::Honor { entity: 3, team: 0 },
        draw_id: 3,
        cell: WorldCell { x: 10, y: 10 },
        graphic: 2665,
        hp,
        maximum: 100,
    }
}
#[test]
fn objective_clocks_animate_only_observed_transitions_without_replaying_history() {
    let now = Instant::now();
    let first = objective_clocks(&[prop(100)], &[], now);
    assert_eq!(first[0].elapsed(now).unwrap(), Duration::MAX);
    let damaged = objective_clocks(&[prop(60)], &first, now);
    assert_eq!(damaged[0].action, 32);
    assert_eq!(damaged[0].elapsed(now).unwrap(), Duration::ZERO);
    let later = now + Duration::from_millis(200);
    let same_state = objective_clocks(&[prop(55)], &damaged, later);
    assert_eq!(
        same_state[0].elapsed(later).unwrap(),
        Duration::from_millis(200)
    );
    let death = objective_clocks(&[prop(0)], &same_state, later);
    assert_eq!(death[0].action, 35);
    assert_eq!(death[0].elapsed(later).unwrap(), Duration::ZERO);
    assert!(death[0].elapsed(now).is_err());
    let duplicate = objective_clocks(&[prop(0)], &death, later + Duration::from_secs(1));
    assert_eq!(
        duplicate[0]
            .elapsed(later + Duration::from_secs(1))
            .unwrap(),
        Duration::from_secs(1)
    );
    assert_eq!(
        objective_clocks(&[prop(0)], &[], later)[0]
            .elapsed(later)
            .unwrap(),
        Duration::MAX
    );
    assert!(objective_clocks(&[], &death, later).is_empty());
}
#[test]
fn objective_clocks_cannot_cross_typed_identity_graphics_or_position() {
    let now = Instant::now();
    let alive = objective_clocks(&[prop(100)], &[], now);
    let dying = objective_clocks(&[prop(0)], &alive, now);
    for target in [
        ObjectiveTarget::Honor { entity: 3, team: 1 },
        ObjectiveTarget::Honor { entity: 4, team: 0 },
    ] {
        let mut replacement = prop(0);
        replacement.target = target;
        assert_eq!(
            objective_clocks(&[replacement], &dying, now)[0]
                .elapsed(now)
                .unwrap(),
            Duration::MAX
        );
    }
    let mut changed = prop(0);
    changed.graphic = 2669;
    assert_eq!(
        objective_clocks(&[changed], &dying, now)[0]
            .elapsed(now)
            .unwrap(),
        Duration::MAX
    );
    let mut changed = prop(0);
    changed.cell.x += 1;
    assert_eq!(
        objective_clocks(&[changed], &dying, now)[0]
            .elapsed(now)
            .unwrap(),
        Duration::MAX
    );
}

#[test]
fn departed_owner_is_retained_but_no_longer_occupies_a_scene_cell() {
    let input = Input::new(observation(
        crate::modern_lito::battle_payload::data_tests::fixture(),
    ))
    .unwrap();
    let s = &input.scene;
    let mut actors = s.actors().to_vec();
    actors[1].cell = actors[0].cell;
    assert!(SceneSnapshot::new(
        s.world_id(),
        s.revision(),
        s.map_digest(),
        s.selected_entity_id(),
        actors.clone()
    )
    .is_err());
    let hidden = SceneSnapshot::new_with_hidden(
        s.world_id(),
        s.revision(),
        s.map_digest(),
        s.selected_entity_id(),
        actors.clone(),
        &[actors[0].entity_id],
    )
    .unwrap();
    assert_eq!(hidden.actors().len(), 2);
    assert!(hidden.hidden(hidden.selected_entity_id()));
    assert!(hidden
        .with_actors(actors.clone())
        .unwrap()
        .hidden(hidden.selected_entity_id()));
    for ids in [vec![99999], vec![actors[0].entity_id, actors[0].entity_id]] {
        assert!(SceneSnapshot::new_with_hidden(
            s.world_id(),
            s.revision(),
            s.map_digest(),
            s.selected_entity_id(),
            actors.clone(),
            &ids
        )
        .is_err());
    }
}

/// The Java trace uses ModernWorldMapTest.openSectorBytes(), not an installed
/// map. Match those exact geometry bytes/digest; all painted SPR/Tile/Sound
/// assets still come from the real shared root. Never write a fake map to disk.
pub(crate) fn trace_scene(root: &Path, data: &d::UnvalidatedData) -> SharedScene {
    trace_observed_scene(root, observation(data.clone()))
}
pub(crate) fn trace_observed_scene(root: &Path, o: Observation) -> SharedScene {
    trace_observed_scene_with_effects(root, o, None)
}
pub(crate) fn trace_observed_scene_with_effects(
    root: &Path,
    o: Observation,
    effect_catalog: Option<&Path>,
) -> SharedScene {
    let input = Input::new(o).unwrap();
    assert_eq!(input.bindings.len(), 1);
    let binding = &input.bindings[0];
    let bytes = vec![0; 49170];
    assert_eq!(binding.byte_length as usize, bytes.len());
    assert_eq!(binding.sha256, <[u8; 32]>::from(Sha256::digest(&bytes)));
    let map = Arc::new(
        WorldMap::from_sector_bytes(
            vec![super::super::world_map::SectorInput {
                key: binding.key,
                bytes,
            }],
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    SharedScene::load_map(root, input, map, effect_catalog, &AtomicBool::new(false)).unwrap()
}
pub(crate) fn verify_npc_pacing_scene(trace: &[d::UnvalidatedData]) {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
    );
    let cancel = AtomicBool::new(false);
    let mut painter = trace_scene(&root, &trace[4]);
    let now = Instant::now() + Duration::from_secs(60);
    let moved = observation(trace[6].clone());
    painter.observe_at(moved.clone(), now, &cancel).unwrap();
    let actor = &moved.data().npcs.as_ref().unwrap().actors.0[0];
    let motion = &moved
        .data()
        .npc_motion
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .actors
        .0[0];
    let walk = motion.last_walk.as_ref().unwrap();
    let form = NpcAppearance::new(actor.graphic as u32).unwrap();
    let q = painter
        .npcs
        .as_ref()
        .unwrap()
        .action(form, NpcAction::Walk)
        .unwrap()
        .clone();
    let mut prefix = 0u128;
    let mut hashes = std::collections::BTreeSet::new();
    for source in (1..q.source_frames().len()).chain(std::iter::once(0)) {
        let nanos =
            (prefix * walk.duration_millis as u128 * 1_000_000).div_ceil(q.duration().as_nanos());
        let at = now + Duration::from_nanos(nanos as u64);
        let surface = painter.render(at, &cancel).unwrap();
        let placement = &surface.geometry.npcs[0];
        assert_eq!(placement.source_frame, source);
        assert_eq!(placement.cell, cell(&walk.to));
        let anchor = surface
            .geometry
            .projection
            .cell_to_pixel(cell(if source == 0 { &walk.to } else { &walk.from }))
            .unwrap();
        assert_eq!(placement.anchor, anchor);
        let frame = painter
            .npcs
            .as_ref()
            .unwrap()
            .frame(form, NpcAction::Walk, actor.heading as u8, source)
            .unwrap();
        assert_eq!(
            placement.top_left.x,
            anchor.x + frame.body.frame.x_offset - 24
        );
        assert_eq!(
            placement.top_left.y,
            anchor.y + frame.body.frame.y_offset - 12
        );
        assert_eq!(placement.sprite_name, frame.body.provenance.name);
        hashes.insert(Sha256::digest(&surface.bgra).to_vec());
        painter.observe_at(moved.clone(), at, &cancel).unwrap();
        assert_eq!(
            surface.bgra,
            painter.render(at, &cancel).unwrap().bgra,
            "duplicate must not restart paced step"
        );
        prefix += u128::from(q.source_frames()[source].duration_ticks * 1000 / 24) * 1_000_000;
    }
    assert!(
        hashes.len() > 1,
        "confirmed walk must change composed pixels"
    );
    let done = now + Duration::from_millis(walk.duration_millis as u64);
    let surface = painter.render(done, &cancel).unwrap();
    assert_eq!(
        painter
            .motion
            .npc_sample(actor, painter.npcs.as_ref().unwrap(), done)
            .unwrap()
            .action,
        NpcAction::Breath
    );
    assert_eq!(
        surface.geometry.npcs[0].anchor,
        surface
            .geometry
            .projection
            .cell_to_pixel(cell(&walk.to))
            .unwrap()
    );
    // An individually valid but rewritten pace cannot replace this scene.
    let mut bad = trace[8].clone();
    bad.npc_motion
        .as_mut()
        .unwrap()
        .state
        .as_mut()
        .unwrap()
        .actors
        .0[0]
        .attack_millis += 1;
    assert!(painter.observe_at(observation(bad), done, &cancel).is_err());
    assert_eq!(surface.bgra, painter.render(done, &cancel).unwrap().bgra);
    assert_eq!(painter.input.observation.data(), moved.data());
    painter
        .observe_at(observation(trace[8].clone()), done, &cancel)
        .unwrap();
    assert!(painter
        .motion
        .npc_sample(
            &trace[8].npcs.as_ref().unwrap().actors.0[0],
            painter.npcs.as_ref().unwrap(),
            done
        )
        .unwrap()
        .walk_from
        .is_some());
    for (old, new) in [(4, 8), (6, 6)] {
        let mut p = trace_scene(&root, &trace[old]);
        let at = Instant::now() + Duration::from_secs(60);
        p.observe_at(observation(trace[new].clone()), at, &cancel)
            .unwrap();
        let a = &trace[new].npcs.as_ref().unwrap().actors.0[0];
        let pose = p
            .motion
            .npc_sample(a, p.npcs.as_ref().unwrap(), at)
            .unwrap();
        let expected = if old == new {
            // Initial attachment still cannot replay a historical walk.
            assert_eq!(pose.action, NpcAction::Breath);
            assert!(pose.walk_from.is_none());
            cell(&a.cell)
        } else {
            // Coalescing may omit intermediate steps, not the fresh, fully
            // confirmed last step. Never draw from the stale original cell.
            assert_eq!(pose.action, NpcAction::Walk);
            let w = trace[new]
                .npc_motion
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .actors
                .0[0]
                .last_walk
                .as_ref()
                .unwrap();
            assert_eq!(w.sequence, 2);
            assert_ne!(w.from, trace[old].npcs.as_ref().unwrap().actors.0[0].cell);
            assert_eq!(pose.walk_from, Some(cell(&w.from)));
            cell(&w.from)
        };
        let s = p.render(at, &cancel).unwrap();
        assert_eq!(
            s.geometry.npcs[0].anchor,
            s.geometry.projection.cell_to_pixel(expected).unwrap()
        );
    }
    eprintln!("v43 NPC shared scene: complete paced source cycle, origin/terminal anchors, immutable destination, duplicate pixels, rejected pace rewrite and single-step continuation; coalesced updates retain only the last confirmed step, initial snapshots do not replay");
}
pub(crate) fn verify_haste_scene(root: &Path, trace: &[d::UnvalidatedData]) {
    use crate::modern_lito::battle_haste::tests as haste;
    let cancel = AtomicBool::new(false);
    let mut painter = trace_scene(root, &trace[0]);
    let now = Instant::now() + Duration::from_secs(60);
    painter
        .observe_at(observation(trace[1].clone()), now, &cancel)
        .unwrap();
    let drawn = painter.render_effects(now, &cancel).unwrap();
    assert_eq!(haste::counts(&painter.haste), (1, 1));
    let original = painter.haste.clone();
    painter.haste = haste::without_effects(&original);
    let control = painter.render_effects(now, &cancel).unwrap();
    painter.haste = original;
    assert_ne!(
        drawn.bgra, control.bgra,
        "3104 must alter the composed world, not only an isolated canvas"
    );
    assert_eq!(drawn.observation.data(), control.observation.data());
    assert_eq!(
        drawn.bgra,
        painter.render_effects(now, &cancel).unwrap().bgra
    );
    assert!(
        drawn.audio.as_ref().is_none_or(|a| a.events.is_empty()),
        "3104 has no sound marker"
    );
    let expired = painter
        .render_effects(now + Duration::from_secs(2), &cancel)
        .unwrap();
    assert_eq!(haste::counts(&painter.haste).0, 0);
    assert!(
        expired.observation.data().actors.0[0]
            .buffs
            .as_ref()
            .unwrap()
            .haste_until_nanos
            > 0
    );
}

pub(crate) fn verify_item_scenes(
    groups: &std::collections::BTreeMap<String, Vec<d::UnvalidatedData>>,
) {
    use crate::modern_lito::{battle_items, battle_spells};
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
    );
    let cancel = AtomicBool::new(false);
    for owner in ["a", "b"] {
        let trace = &groups[&format!("effects-{owner}")];
        assert_eq!(trace.len(), 15);
        let mut painter = trace_scene(&root, &trace[0]);
        battle_items::tests::verify_coalesced(
            observation(trace[0].clone()),
            observation(trace.last().unwrap().clone()),
            &painter.item_assets,
        );
        let start = Instant::now() + Duration::from_secs(60);
        let mut effects = std::collections::BTreeSet::new();
        for index in (1..trace.len()).step_by(2) {
            let used = observation(trace[index].clone());
            let moved = observation(trace[index + 1].clone());
            battle_items::tests::verify(
                observation(trace[index - 1].clone()),
                used.clone(),
                moved.clone(),
                &painter.item_assets,
            );
            let at = start + Duration::from_secs(index as u64 * 2);
            painter.observe_at(used.clone(), at, &cancel).unwrap();
            let first = painter.render_effects(at, &cancel).unwrap();
            assert_eq!(battle_items::tests::count(&painter.items), 1);
            let event = trace[index]
                .presentation
                .as_ref()
                .unwrap()
                .0
                .iter()
                .filter_map(|p| p.item.as_ref())
                .max_by_key(|e| e.revision)
                .unwrap();
            effects.insert(event.effect);
            let sounds = first
                .audio
                .as_ref()
                .unwrap()
                .events
                .iter()
                .flat_map(|e| e.group.candidates().iter().copied())
                .collect::<Vec<_>>();
            assert_eq!(
                sounds,
                match event.effect {
                    750 => vec![142],
                    751 => vec![121],
                    _ => vec![],
                }
            );
            let sample = at + Duration::from_millis(100);
            painter.observe_at(moved.clone(), sample, &cancel).unwrap();
            let visible = painter.render_effects(sample, &cancel).unwrap();
            let retained = painter.items.clone();
            painter.items = battle_items::tests::without_effects(&retained);
            let control = painter.render_effects(sample, &cancel).unwrap();
            painter.items = retained;
            assert_ne!(
                visible.bgra, control.bgra,
                "actual shared overlay changes world pixels: {} owner {owner}",
                event.effect
            );
            assert_eq!(visible.observation.data(), control.observation.data());
            assert_eq!(
                visible.bgra,
                painter.render_effects(sample, &cancel).unwrap().bgra
            );
            if event.effect == 750 {
                let mut replacement = trace_scene(&root, &trace[index + 1]);
                assert!(replacement
                    .carry_from(&mut painter, sample, &AtomicBool::new(true))
                    .is_err());
                replacement
                    .carry_from(&mut painter, sample, &cancel)
                    .unwrap();
                let carried = replacement.render_effects(sample, &cancel).unwrap();
                assert_eq!(carried.bgra, visible.bgra);
                assert_eq!(
                    carried.audio.unwrap().events[0].sequence,
                    visible.audio.unwrap().events[0].sequence
                );
                assert!(painter.render_effects(sample, &cancel).is_err());
                painter = replacement;
            }
            let end = painter
                .render_effects(at + Duration::from_secs(2), &cancel)
                .unwrap();
            assert!(end.audio.unwrap().events.is_empty());
            assert_eq!(battle_items::tests::count(&painter.items), 0);
        }
        assert_eq!(
            effects.into_iter().collect::<Vec<_>>(),
            vec![189, 191, 194, 197, 750, 751]
        );
        let mut attached = trace_scene(&root, trace.last().unwrap());
        assert_eq!(battle_items::tests::count(&attached.items), 0);
        assert!(attached
            .render_effects(Instant::now(), &cancel)
            .unwrap()
            .audio
            .unwrap()
            .events
            .is_empty());

        let mixed = &groups[&format!("mixed-{owner}")];
        let mut painter = trace_scene(&root, &mixed[0]);
        let at = Instant::now() + Duration::from_secs(60);
        let mut heard = std::collections::BTreeMap::new();
        for index in 1..mixed.len() {
            let now = at + Duration::from_millis((index as u64 - 1) * 100);
            painter
                .observe_at(observation(mixed[index].clone()), now, &cancel)
                .unwrap();
            for e in painter
                .render_effects(now, &cancel)
                .unwrap()
                .audio
                .unwrap()
                .events
                .iter()
            {
                heard.insert(e.sequence, e.group.candidates().to_vec());
            }
        }
        let launch = battle_spells::tests::launch_deadline(&painter.spells);
        for millis in 0..31 {
            let now =
                (at + Duration::from_millis(200)).max(launch) + Duration::from_millis(millis * 20);
            for e in painter
                .render_effects(now, &cancel)
                .unwrap()
                .audio
                .unwrap()
                .events
                .iter()
            {
                heard.insert(e.sequence, e.group.candidates().to_vec());
            }
        }
        assert_eq!(
            heard.len(),
            3,
            "spell and item source sequence 1 must not collide"
        );
        let mut sounds = heard.values().flatten().copied().collect::<Vec<_>>();
        sounds.sort();
        assert_eq!(sounds, vec![97, 121, 142]);
    }
    eprintln!("schema2 item presentation:38 real Java frames; both owner views, all six shared SPRs/seven item uses; fixed origin after MOVE, suppression/age/capacity/cancellation/handoff; mixed skill97+item121/142 distinct bounded cue sequences; synthetic map, no HWND/hardware playback claim");
}

pub(crate) fn verify_bow_scenes(
    groups: &std::collections::BTreeMap<String, Vec<d::UnvalidatedData>>,
) {
    use crate::modern_lito::battle_bows::tests as bow;
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
    );
    let cancel = AtomicBool::new(false);
    for owner in ["a", "b"] {
        for name in ["bow-hit", "bow-miss", "bow-life", "bow-honor"] {
            let trace = &groups[&format!("{name}-{owner}")];
            let mut painter = trace_scene(&root, &trace[0]);
            bow::verify_edges(
                observation(trace[0].clone()),
                observation(trace[1].clone()),
                &painter.bow_assets,
            );
            let start = Instant::now() + Duration::from_secs(60);
            painter
                .observe_at(observation(trace[1].clone()), start, &cancel)
                .unwrap();
            assert_eq!(bow::counts(&painter.bows), (1, 0));
            let launch = bow::deadline(&painter.bows, 1);
            let before = launch - Duration::from_nanos(1);
            // Target updates can occur while a local bow action is pending.
            // These deterministic clocks test target-life binding, not RTT/FPS.
            painter
                .observe_at(observation(trace[2].clone()), before, &cancel)
                .unwrap();
            assert_eq!(bow::counts(&painter.bows), (1, 0));
            painter.render_effects(before, &cancel).unwrap();
            assert_eq!(bow::counts(&painter.bows), (1, 0));
            let visible = painter.render_effects(launch, &cancel).unwrap();
            let instant = name == "bow-honor";
            assert_eq!(bow::counts(&painter.bows), (0, usize::from(!instant)));
            let endpoint = if instant {
                cell(
                    &trace[1].presentation.as_ref().unwrap().0[0]
                        .shot
                        .as_ref()
                        .unwrap()
                        .to,
                )
            } else {
                bow::verify_targets(&trace[2], &painter.bows, 1)
            };
            let shot = trace[1].presentation.as_ref().unwrap().0[0]
                .shot
                .as_ref()
                .unwrap();
            if name == "bow-miss" || name == "bow-life" {
                assert_eq!(endpoint, cell(&shot.to));
            }
            if name == "bow-hit" {
                assert_ne!(endpoint, cell(&shot.to));
            }
            let retained = painter.bows.clone();
            painter.bows = bow::without_flights(&retained);
            let control = painter.render_effects(launch, &cancel).unwrap();
            painter.bows = retained;
            if instant {
                assert_eq!(
                    visible.bgra, control.bgra,
                    "adjacent Honor shot has no artificial impact or dwell"
                );
            } else {
                assert_ne!(
                    visible.bgra, control.bgra,
                    "shared SPR66 must change actual world pixels {name}/{owner}"
                );
            }
            assert_eq!(visible.observation.data(), control.observation.data());
            assert_eq!(
                visible.bgra,
                painter.render_effects(launch, &cancel).unwrap().bgra
            );
            assert!(
                visible.audio.as_ref().is_none_or(|a| a.events.is_empty()),
                "SPR66 has no guessed audio"
            );
            if name == "bow-hit" {
                let mut replacement = trace_scene(&root, &trace[2]);
                assert!(replacement
                    .carry_from(&mut painter, launch, &AtomicBool::new(true))
                    .is_err());
                replacement
                    .carry_from(&mut painter, launch, &cancel)
                    .unwrap();
                assert_eq!(
                    replacement.render_effects(launch, &cancel).unwrap().bgra,
                    visible.bgra
                );
                assert_eq!(bow::frozen_endpoint(&replacement.bows), endpoint);
                assert!(painter.render_effects(launch, &cancel).is_err());
                painter = replacement;
            }
            if trace.len() > 3 {
                painter
                    .observe_at(
                        observation(trace[3].clone()),
                        launch + Duration::from_millis(30),
                        &cancel,
                    )
                    .unwrap();
                painter
                    .render_effects(launch + Duration::from_millis(30), &cancel)
                    .unwrap();
                assert_eq!(
                    bow::frozen_endpoint(&painter.bows),
                    endpoint,
                    "flight cannot home after launch"
                );
            }
            let end = painter
                .render_effects(launch + Duration::from_secs(30), &cancel)
                .unwrap();
            assert_eq!(bow::counts(&painter.bows), (0, 0));
            assert_eq!(end.observation.data(), trace.last().unwrap());
        }
        let trace = &groups[&format!("bow-hurt-{owner}")];
        let mut painter = trace_scene(&root, &trace[0]);
        let start = Instant::now() + Duration::from_secs(60);
        painter
            .observe_at(observation(trace[1].clone()), start, &cancel)
            .unwrap();
        let launch = bow::deadline(&painter.bows, 1);
        painter
            .observe_at(
                observation(trace[2].clone()),
                start + launch.duration_since(start) / 2,
                &cancel,
            )
            .unwrap();
        assert!(painter.bows.skipped);
        assert_eq!(
            bow::counts(&painter.bows),
            (1, 0),
            "only counter-shooter's action can remain"
        );
        painter
            .render_effects(launch + Duration::from_secs(1), &cancel)
            .unwrap();
        assert_eq!(bow::counts(&painter.bows), (0, 0));

        let trace = &groups[&format!("bow-paced-{owner}")];
        let mut painter = trace_scene(&root, &trace[0]);
        let start = Instant::now() + Duration::from_secs(60);
        painter
            .observe_at(observation(trace[1].clone()), start, &cancel)
            .unwrap();
        let original = bow::deadline(&painter.bows, 1);
        let step = original.duration_since(start) / 10;
        let body_before_haste = painter.body_audio.clone();
        painter
            .observe_at(observation(trace[2].clone()), start + step, &cancel)
            .unwrap();
        let haste = bow::deadline(&painter.bows, 1);
        super::super::battle_body_audio::tests::verify_pace_update(
            &body_before_haste,
            &painter.body_audio,
            &painter.motion,
            start + step,
        );
        let body_before_brave = painter.body_audio.clone();
        painter
            .observe_at(observation(trace[3].clone()), start + step * 2, &cancel)
            .unwrap();
        let brave = bow::deadline(&painter.bows, 1);
        super::super::battle_body_audio::tests::verify_pace_update(
            &body_before_brave,
            &painter.body_audio,
            &painter.motion,
            start + step * 2,
        );
        assert!(brave < haste && haste < original);
        painter
            .render_effects(brave - Duration::from_nanos(1), &cancel)
            .unwrap();
        assert_eq!(bow::counts(&painter.bows), (1, 0));
        painter.render_effects(brave, &cancel).unwrap();
        assert_eq!(
            bow::counts(&painter.bows),
            (0, 0),
            "adjacent tower flight finishes on first native step"
        );
        assert_eq!(painter.observation().data(), trace.last().unwrap());
    }
    eprintln!("canonical ordinary bow:44 Java frames/12 streams; both sources, real bow21 Hit/shared SPR66, hit/miss/life/Honor, frozen native30ms flight, no invented impact/audio; body-clock haste/brave, hurt/late/capacity/baseline/duplicate/cancel/reload/retirement; synthetic geometry, not HWND or battle TLS");
}

/// Production Main observations and the installed shared map, not synthetic
/// geometry. Pixel/audio/body evidence still does not operate a native HWND.
pub(crate) fn verify_player_healing_shared(
    root: &Path,
    before: Observation,
    after: Observation,
    caster: i64,
) {
    use crate::modern_lito::battle_healing::tests as heal;
    let cancel = AtomicBool::new(false);
    let saved = after.clone();
    let mut painter = SharedScene::load(root, before.clone(), &cancel).unwrap();
    let at = Instant::now() + Duration::from_secs(60);
    painter.observe_at(after.clone(), at, &cancel).unwrap();
    assert_eq!(heal::count(&painter.healing), 1);
    let body = painter
        .healing
        .overrides(
            &painter.input.scene,
            painter.teleport_assets.as_ref().unwrap(),
            at,
        )
        .unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].entity_id, caster as u64);
    let actor = after
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == caster)
        .unwrap();
    let member = after.data().scene_member(actor).unwrap();
    let expected = painter
        .teleport_assets
        .as_ref()
        .unwrap()
        .caster_frame(
            BaseAppearance {
                class: member.class as u8,
                sex: member.sex as u8,
            },
            actor.heading as u8,
            Duration::ZERO,
        )
        .unwrap();
    assert_eq!(
        (
            body[0].graphics_id,
            body[0].source_frame,
            body[0].sprite_name
        ),
        (
            expected.graphics_id,
            expected.source_frame,
            expected.sprite_name
        )
    );
    let painted = painter.render_effects(at, &cancel).unwrap();
    let retained = painter.healing.clone();
    painter.healing = heal::without_effects(&retained);
    let control = painter.render_effects(at, &cancel).unwrap();
    painter.healing = retained;
    assert_ne!(
        painted.bgra, control.bgra,
        "confirmed shared744 changes actual map pixels"
    );
    assert_eq!(painted.observation.data(), saved.data());
    assert_eq!(control.observation.data(), saved.data());
    let cues: Vec<_> = painted
        .audio
        .as_ref()
        .unwrap()
        .events
        .iter()
        .filter(|e| e.group.candidates() == [109])
        .collect();
    assert_eq!(cues.len(), 1);
    let cue = cues[0].sequence;
    let twice = painter.render_effects(at, &cancel).unwrap();
    assert_eq!(painted.bgra, twice.bgra);
    assert_eq!(
        twice
            .audio
            .as_ref()
            .unwrap()
            .events
            .iter()
            .find(|e| e.group.candidates() == [109])
            .unwrap()
            .sequence,
        cue
    );
    heal::verify_player_edges(before, after, &painter.healing_assets, caster);
    eprintln!("HEALING_SHARED_SCENE: production Main observations/shared map, explicit744 visible pixels/action19/109 once, unchanged authoritative HP/MP, repeat/age/cancel/life/gear checks");
}

pub(crate) fn verify_healing_scenes(
    groups: &std::collections::BTreeMap<String, Vec<d::UnvalidatedData>>,
) {
    use crate::modern_lito::{battle_healing::tests as heal, battle_spells};
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
    );
    let cancel = AtomicBool::new(false);
    assert_eq!(groups.len(), 8);
    for owner in ["a", "b"] {
        for name in ["heal-full", "heal-hurt"] {
            let trace = &groups[&format!("{name}-{owner}")];
            let moved = if name == "heal-full" { 3 } else { 2 };
            let mut painter = trace_scene(&root, &trace[0]);
            heal::verify_edges(
                observation(trace[0].clone()),
                observation(trace[1].clone()),
                observation(trace[moved].clone()),
                &painter.healing_assets,
            );
            let at = Instant::now() + Duration::from_secs(60);
            painter
                .observe_at(observation(trace[1].clone()), at, &cancel)
                .unwrap();
            let overrides = painter
                .healing
                .overrides(
                    &painter.input.scene,
                    painter.teleport_assets.as_ref().unwrap(),
                    at,
                )
                .unwrap();
            assert_eq!(overrides.len(), 1);
            let expected = painter
                .teleport_assets
                .as_ref()
                .unwrap()
                .caster_frame(
                    BaseAppearance { class: 1, sex: 0 },
                    trace[1].actors.0[0].heading as u8,
                    Duration::ZERO,
                )
                .unwrap();
            assert_eq!(overrides[0].graphics_id, expected.graphics_id);
            assert_eq!(overrides[0].source_frame, expected.source_frame);
            assert_eq!(overrides[0].sprite_name, expected.sprite_name);
            let painted = painter.render_effects(at, &cancel).unwrap();
            assert_eq!(
                heal::count(&painter.healing),
                1,
                "full HP still has a successful effect"
            );
            let retained = painter.healing.clone();
            painter.healing = heal::without_effects(&retained);
            let control = painter.render_effects(at, &cancel).unwrap();
            painter.healing = retained;
            assert_ne!(
                painted.bgra, control.bgra,
                "qualified744 must alter actual world pixels"
            );
            assert_eq!(painted.observation.data(), control.observation.data());
            assert_eq!(
                painted.bgra,
                painter.render_effects(at, &cancel).unwrap().bgra
            );
            let sound = painted.audio.as_ref().unwrap();
            assert_eq!(sound.events.len(), 1);
            assert_eq!(sound.events[0].group.candidates(), [109]);
            if name == "heal-full" {
                painter
                    .observe_at(observation(trace[2].clone()), at, &cancel)
                    .unwrap();
                assert_eq!(
                    heal::count(&painter.healing),
                    1,
                    "denied request has no new effect"
                );
            }
            let after = at + Duration::from_millis(if name == "heal-full" { 1200 } else { 700 });
            painter
                .observe_at(observation(trace[moved].clone()), after, &cancel)
                .unwrap();
            assert!(painter
                .healing
                .overrides(
                    &painter.input.scene,
                    painter.teleport_assets.as_ref().unwrap(),
                    after
                )
                .unwrap()
                .is_empty());
            let still = painter.render_effects(after, &cancel).unwrap();
            assert_eq!(
                heal::count(&painter.healing),
                1,
                "hurt/MOVE cannot erase a committed tower effect"
            );
            let retained = painter.healing.clone();
            painter.healing = heal::without_effects(&retained);
            let control = painter.render_effects(after, &cancel).unwrap();
            painter.healing = retained;
            assert_ne!(still.bgra, control.bgra);
            let mut replacement = trace_scene(&root, &trace[moved]);
            assert!(replacement
                .carry_from(&mut painter, after, &AtomicBool::new(true))
                .is_err());
            assert_eq!(
                painter.render_effects(after, &cancel).unwrap().bgra,
                still.bgra
            );
            replacement
                .carry_from(&mut painter, after, &cancel)
                .unwrap();
            assert_eq!(
                replacement.render_effects(after, &cancel).unwrap().bgra,
                still.bgra
            );
            assert!(painter.render_effects(after, &cancel).is_err());
            painter = replacement;
            let final_at = at + Duration::from_millis(2500);
            let expired = painter.render_effects(final_at, &cancel).unwrap();
            assert_eq!(heal::count(&painter.healing), 0);
            assert!(expired.audio.as_ref().unwrap().events.is_empty());
            assert_eq!(expired.observation.data(), &trace[moved]);
            if name == "heal-full" {
                heal::verify_coalesced(
                    observation(trace[0].clone()),
                    observation(trace[5].clone()),
                    observation(trace[6].clone()),
                    &painter.healing_assets,
                );
                let mut historical = trace_scene(&root, &trace[1]);
                let displayed = historical.render_effects(Instant::now(), &cancel).unwrap();
                assert_eq!(heal::count(&historical.healing), 0);
                assert!(displayed.audio.as_ref().unwrap().events.is_empty());
            }
        }
        let trace = &groups[&format!("heal-paced-{owner}")];
        let mut painter = trace_scene(&root, &trace[0]);
        let at = Instant::now() + Duration::from_secs(60);
        painter
            .observe_at(observation(trace[1].clone()), at, &cancel)
            .unwrap();
        let later = at + Duration::from_millis(300);
        let normal = heal::body_elapsed(&painter.healing, 1, later);
        painter
            .observe_at(
                observation(trace[2].clone()),
                at + Duration::from_millis(100),
                &cancel,
            )
            .unwrap();
        let fast = heal::body_elapsed(&painter.healing, 1, later);
        painter
            .observe_at(
                observation(trace[3].clone()),
                at + Duration::from_millis(200),
                &cancel,
            )
            .unwrap();
        let brave = heal::body_elapsed(&painter.healing, 1, later);
        assert!(
            normal < fast && fast < brave,
            "same action19 clock is retimed, never restarted"
        );
        painter.render_effects(later, &cancel).unwrap();
        assert_eq!(heal::count(&painter.healing), 1);

        let trace = &groups[&format!("heal-mixed-{owner}")];
        let mut painter = trace_scene(&root, &trace[0]);
        let at = Instant::now() + Duration::from_secs(60);
        let mut heard = std::collections::BTreeMap::new();
        for (index, millis) in [(1, 0), (2, 0), (3, 100)] {
            let now = at + Duration::from_millis(millis);
            painter
                .observe_at(observation(trace[index].clone()), now, &cancel)
                .unwrap();
            for e in painter
                .render_effects(now, &cancel)
                .unwrap()
                .audio
                .unwrap()
                .events
                .iter()
            {
                heard.insert(e.sequence, e.group.candidates().to_vec());
            }
        }
        let launch = battle_spells::tests::launch_deadline(&painter.spells)
            .max(at + Duration::from_millis(100));
        for ms in [0, 50, 100] {
            for e in painter
                .render_effects(launch + Duration::from_millis(ms), &cancel)
                .unwrap()
                .audio
                .unwrap()
                .events
                .iter()
            {
                heard.insert(e.sequence, e.group.candidates().to_vec());
            }
        }
        let mut sounds = heard.values().flatten().copied().collect::<Vec<_>>();
        sounds.sort();
        assert_eq!(
            sounds,
            vec![97, 109, 121],
            "independent source sequence1 cues never collide"
        );
        assert_eq!(heard.len(), 3);
    }
    eprintln!("canonical healing:36 real Java frames/8 owner streams; full/damaged Honor tower, immediate shared744/action19/109, denial/MOVE/hurt/coalescing/age/capacity/handoff, paced body, mixed97+109+121; synthetic geometry, no TLS/HWND/audio-device claim");
}

pub(crate) fn verify_shared_motion(
    groups: &std::collections::BTreeMap<String, Vec<d::UnvalidatedData>>,
) {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
    );
    let cancel = AtomicBool::new(false);
    verify_teleport_and_reload(&root, &groups["teleport"]);
    for name in [
        "spell70",
        "spell70-empty",
        "spell77-normal",
        "spell77-critical",
        "spell77-npc",
        "spell70-npc",
    ] {
        verify_spells(&root, name, &groups[name]);
    }
    for name in [
        "arrow-player",
        "arrow-life",
        "arrow-honor",
        "arrow-npc",
        "arrow-hurt",
        "npc",
    ] {
        verify_arrow_scene(&root, name, &groups[name]);
    }
    let departed = &groups["time-leave"];
    let mut leaving = trace_scene(&root, &departed[0]);
    let start = Instant::now();
    assert_eq!(
        leaving
            .render_effects(start, &cancel)
            .unwrap()
            .geometry
            .actors
            .len(),
        2
    );
    leaving
        .observe_at(
            observation(departed[1].clone()),
            start + Duration::from_secs(1),
            &cancel,
        )
        .unwrap();
    let only_guest = leaving
        .render_effects(start + Duration::from_secs(1), &cancel)
        .unwrap();
    assert_eq!(
        only_guest
            .geometry
            .actors
            .iter()
            .map(|a| a.entity_id)
            .collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(
        only_guest.observation.personal().player(),
        &departed[1].personal.player
    );
    assert_eq!(only_guest.observation.data().personal.entity, 1);
    leaving
        .observe_at(
            observation(departed[2].clone()),
            start + Duration::from_secs(2),
            &cancel,
        )
        .unwrap();
    assert!(leaving
        .render_effects(start + Duration::from_secs(2), &cancel)
        .unwrap()
        .geometry
        .actors
        .is_empty());
    let race = &groups["race"];
    verify_pet_audio(&root, race);
    let mut scene = trace_scene(&root, &race[2]);
    let t = Instant::now();
    let initial = scene.render_effects(t, &cancel).unwrap();
    assert_eq!(initial.geometry.actors.len(), 2);
    let idle = scene
        .render_effects(t + Duration::from_millis(584), &cancel)
        .unwrap();
    assert!(idle.geometry.actors.iter().all(|a| a.source_frame == 1));
    let at = t + Duration::from_secs(1);
    scene
        .observe_at(observation(race[3].clone()), at, &cancel)
        .unwrap();
    let mut samples = std::collections::BTreeSet::new();
    let mut last = None;
    for millis in [0, 25, 50, 75] {
        let surface = scene
            .render_effects(at + Duration::from_millis(millis), &cancel)
            .unwrap();
        let actor = surface
            .geometry
            .actors
            .iter()
            .find(|a| a.entity_id == 1)
            .unwrap();
        samples.insert((actor.source_frame, actor.subframe));
        last = Some(surface);
    }
    assert!(
        samples.len() > 1,
        "confirmed race movement must animate shared body/shadow"
    );
    let retained = last.unwrap();
    let again_at = at + Duration::from_millis(75);
    scene
        .observe_at(observation(race[3].clone()), again_at, &cancel)
        .unwrap();
    let again = scene.render_effects(again_at, &cancel).unwrap();
    assert_eq!(retained.bgra, again.bgra);
    assert_eq!(retained.geometry.actors, again.geometry.actors);
    let settled = scene
        .render_effects(at + Duration::from_millis(100), &cancel)
        .unwrap();
    assert_eq!(
        settled.geometry.projection.anchor_cell,
        cell(&race[3].actors.0[0].position)
    );
    assert_eq!(
        settled
            .geometry
            .actors
            .iter()
            .find(|a| a.entity_id == 1)
            .unwrap()
            .source_frame,
        0
    );
    scene
        .observe_at(
            observation(race[4].clone()),
            t + Duration::from_secs(2),
            &cancel,
        )
        .unwrap();
    scene
        .render_effects(t + Duration::from_secs(2), &cancel)
        .unwrap();
    scene
        .observe_at(
            observation(race[5].clone()),
            t + Duration::from_secs(3),
            &cancel,
        )
        .unwrap();
    let finished = scene
        .render_effects(t + Duration::from_secs(3), &cancel)
        .unwrap();
    assert_eq!(
        finished
            .geometry
            .actors
            .iter()
            .map(|a| a.entity_id)
            .collect::<Vec<_>>(),
        vec![2]
    );
    scene
        .observe_at(
            observation(race.last().unwrap().clone()),
            t + Duration::from_secs(4),
            &cancel,
        )
        .unwrap();
    assert!(scene
        .render_effects(t + Duration::from_secs(4), &cancel)
        .unwrap()
        .geometry
        .actors
        .is_empty());
    assert_eq!(initial.observation.data(), &race[2]);

    let motion = &groups["motion"];
    let mut scene = trace_scene(&root, &motion[0]);
    let start = Instant::now();
    for (i, data) in motion.iter().enumerate().skip(1) {
        let at = start + Duration::from_secs(i as u64);
        scene
            .observe_at(observation(data.clone()), at, &cancel)
            .unwrap();
        let expected = match i {
            1 | 2 => NpcAction::Walk,
            3 => NpcAction::Attack,
            _ => NpcAction::Damage,
        };
        let npc = &data.npcs.as_ref().unwrap().actors.0[0];
        assert_eq!(scene.motion.npc_pose(npc.serial, at).unwrap().0, expected);
        if i == 3 {
            assert_eq!(
                scene
                    .motion
                    .pose(data.actors.0[0].entity, at)
                    .unwrap()
                    .action,
                BaseAction::Damage
            );
        }
        if i == 4 {
            assert_eq!(
                scene
                    .motion
                    .pose(data.actors.0[0].entity, at)
                    .unwrap()
                    .action,
                BaseAction::Attack,
                "NPC-target melee must use the participant incarnation, not the respawn counter"
            );
        }
        let surface = scene.render_effects(at, &cancel).unwrap();
        assert_eq!(surface.geometry.npcs.len(), 1);
        assert_eq!(surface.geometry.npcs[0].graphics_id, npc.graphic as u32);
        assert_eq!(surface.geometry.npcs[0].cell, cell(&npc.cell));
        let later = scene
            .render_effects(at + Duration::from_millis(750), &cancel)
            .unwrap();
        assert_eq!(surface.observation.data(), data);
        assert_eq!(later.observation.data(), data);
    }

    let siege = &groups["siege"];
    let mut scene = trace_scene(&root, &siege[0]);
    let start = Instant::now();
    let mut collapses = 0;
    for (i, data) in siege.iter().enumerate().skip(1) {
        let at = start + Duration::from_secs(i as u64 * 5);
        let previous = scene.input.props.clone();
        scene
            .observe_at(observation(data.clone()), at, &cancel)
            .unwrap();
        let newly_dead = scene
            .input
            .props
            .iter()
            .filter(|p| {
                p.hp == 0
                    && previous
                        .iter()
                        .any(|old| old.target == p.target && old.hp > 0)
            })
            .map(|p| p.target.clone())
            .collect::<Vec<_>>();
        let first = scene.render_effects(at, &cancel).unwrap();
        if !newly_dead.is_empty() {
            objective_audio::verify(&scene.objectives, scene.towers.as_ref().unwrap(), at);
            let cues = first.audio.as_ref().unwrap();
            assert_eq!(cues.events.len(), newly_dead.len());
            assert!(cues
                .events
                .iter()
                .all(|e| e.group.candidates() == [360]
                    && e.bank.as_ref().unwrap().get(360).is_some()));
            let duplicate = scene.render_effects(at, &cancel).unwrap();
            assert_eq!(
                duplicate
                    .audio
                    .as_ref()
                    .unwrap()
                    .events
                    .iter()
                    .map(|e| e.sequence)
                    .collect::<Vec<_>>(),
                cues.events.iter().map(|e| e.sequence).collect::<Vec<_>>()
            );
        }
        let middle = scene
            .render_effects(at + Duration::from_millis(500), &cancel)
            .unwrap();
        let final_pose = scene
            .render_effects(at + Duration::from_secs(4), &cancel)
            .unwrap();
        assert!(final_pose.audio.as_ref().unwrap().events.is_empty());
        for target in newly_dead {
            let sample = |s: &Surface| {
                s.objectives
                    .iter()
                    .find(|(key, _)| key == &target)
                    .map(|(_, p)| (p.source_frame, p.subframe))
                    .unwrap()
            };
            assert_ne!(sample(&first), sample(&middle));
            assert_eq!(sample(&final_pose).0, 0);
            collapses += 1;
        }
    }
    assert!(collapses > 0, "actual engine tower death must animate");
    let mut historical = trace_scene(&root, siege.last().unwrap());
    assert!(historical
        .render_effects(Instant::now(), &cancel)
        .unwrap()
        .audio
        .as_ref()
        .unwrap()
        .events
        .is_empty());
    assert!(historical
        .render_effects(Instant::now(), &cancel)
        .unwrap()
        .objectives
        .iter()
        .all(|(_, p)| p.source_frame == 0));
    eprintln!("canonical shared-art trace: race samples={samples:?}, duplicate stable, 584ms idle, finish removal; NPC walk/attack/hurt and player hurt rendered; tower collapses={collapses}; exact synthetic Java geometry, not installed-map/TLS/GUI proof");
}

fn verify_pet_audio(root: &Path, trace: &[d::UnvalidatedData]) {
    let cancel = AtomicBool::new(false);
    let mut painter = trace_scene(root, &trace[2]);
    let at = Instant::now() + Duration::from_secs(20);
    // Attachment time, not resource-load duration, is the idle baseline.
    let mut baseline = super::super::world_runtime::PetPresentation::from_scene(
        super::super::pet_animation::PetAssets::load(root, &painter.input.pet_forms, &cancel)
            .unwrap(),
        &painter.input.scene,
    )
    .unwrap();
    baseline
        .update(&painter.input.scene, &painter.map, at, &cancel)
        .unwrap();
    painter.pets = Some(baseline);
    assert!(painter
        .render_effects(at, &cancel)
        .unwrap()
        .audio
        .unwrap()
        .events
        .is_empty());
    painter
        .render_effects(at + Duration::from_millis(550), &cancel)
        .unwrap();
    let cue_at = at + Duration::from_millis(600);
    let visible = painter.render_effects(cue_at, &cancel).unwrap();
    let bank = &painter.pets.as_ref().unwrap().assets;
    // Actual pinned shared pet actions contain NO audio markers. A separate
    // four-form resource probe verifies every frame; do not invent a bark to
    // make this integration positive. Mixer withdrawal has synthetic tests.
    assert!(bank.sound_groups().is_empty());
    assert!(bank.sound_bank().is_empty());
    for form in &painter.input.pet_forms {
        for kind in [
            super::super::pet_animation::PetAction::Walk,
            super::super::pet_animation::PetAction::Breath,
        ] {
            let action = bank.action(*form, kind).unwrap();
            for source in 0..action.source_frames().len() {
                assert!(action.sound_group(source).unwrap().is_none());
            }
        }
    }
    let audio = visible.audio.as_ref().unwrap();
    assert!(
        audio.events.is_empty(),
        "silent shared animation stays silent"
    );
    let sequence = |s: &Surface| {
        s.audio
            .as_ref()
            .unwrap()
            .events
            .iter()
            .map(|e| e.sequence)
            .collect::<Vec<_>>()
    };
    let duplicate = painter.render_effects(cue_at, &cancel).unwrap();
    assert_eq!(sequence(&duplicate), sequence(&visible));
    assert_eq!(duplicate.bgra, visible.bgra);
    // Failed candidate playback must not retire the accepted pet cursors.
    let mut candidate = painter.pets.clone().unwrap();
    assert!(Arc::ptr_eq(
        &candidate.assets,
        &painter.pets.as_ref().unwrap().assets
    ));
    assert!(candidate
        .update(
            &painter.input.scene,
            &painter.map,
            cue_at,
            &AtomicBool::new(true)
        )
        .is_err());
    assert_eq!(
        sequence(&painter.render_effects(cue_at, &cancel).unwrap()),
        sequence(&visible)
    );
    let mut replacement = trace_scene(root, &trace[2]);
    assert!(replacement
        .carry_from(&mut painter, cue_at, &AtomicBool::new(true))
        .is_err());
    replacement
        .carry_from(&mut painter, cue_at, &cancel)
        .unwrap();
    assert!(painter.render_effects(cue_at, &cancel).is_err());
    let resumed = replacement.render_effects(cue_at, &cancel).unwrap();
    assert_eq!(resumed.bgra, visible.bgra);
    assert_eq!(sequence(&resumed), sequence(&visible));
    // True MOVE/finish also preserve the source-qualified silent bank.
    let move_at = cue_at + Duration::from_millis(1);
    replacement
        .observe_at(observation(trace[3].clone()), move_at, &cancel)
        .unwrap();
    let moved = replacement.render_effects(move_at, &cancel).unwrap();
    let expected = replacement
        .pets
        .as_mut()
        .unwrap()
        .update(&replacement.input.scene, &replacement.map, move_at, &cancel)
        .unwrap()
        .2;
    assert_eq!(
        moved.audio.as_ref().unwrap().events.len(),
        expected.events.len()
    );
    assert!(sequence(&moved).is_empty());
    assert_eq!(
        sequence(&visible),
        sequence(&resumed),
        "old published batch immutable"
    );
    let later = move_at + Duration::from_secs(5);
    assert!(
        replacement
            .render_effects(later, &cancel)
            .unwrap()
            .audio
            .unwrap()
            .events
            .is_empty(),
        "no stalled idle backlog"
    );
    replacement
        .observe_at(observation(trace.last().unwrap().clone()), later, &cancel)
        .unwrap();
    assert!(replacement
        .render_effects(later, &cancel)
        .unwrap()
        .audio
        .unwrap()
        .events
        .is_empty());
    eprintln!("canonical pet audio: actual shared walk/breath markers are silent, no invented sound; empty bank reaches Surface, duplicate/cancel/reload pixel-identical, MOVE/finish/pause remain silent; positive withdrawal tested with explicit synthetic mixer events, no device claim");
}

fn verify_arrow_scene(root: &Path, name: &str, trace: &[d::UnvalidatedData]) {
    let cancel = AtomicBool::new(false);
    let mut painter = trace_scene(root, &trace[0]);
    let observations = trace.iter().cloned().map(observation).collect::<Vec<_>>();
    super::super::battle_spells::arrow_tests::verify(
        name,
        &observations,
        &painter.spell_assets,
        &painter.input.scene,
    );
    let at = Instant::now() + Duration::from_secs(60);
    painter
        .observe_at(observations[1].clone(), at, &cancel)
        .unwrap();
    let start = painter.render_effects(at, &cancel).unwrap();
    assert_eq!(start.observation.data(), &trace[1]);
    let launch = super::super::battle_spells::tests::launch_deadline(&painter.spells);
    let before = super::super::battle_spells::arrow_tests::pre_index(name);
    painter
        .observe_at(
            observations[before].clone(),
            launch - Duration::from_nanos(1),
            &cancel,
        )
        .unwrap();
    let firing = painter.render_effects(launch, &cancel).unwrap();
    assert_eq!(firing.audio.as_ref().unwrap().events.len(), 1);
    assert_eq!(
        firing.audio.as_ref().unwrap().events[0].group.candidates(),
        &[97]
    );
    let sample = launch + Duration::from_millis(60);
    painter
        .observe_at(observations.last().unwrap().clone(), sample, &cancel)
        .unwrap();
    let visible = painter.render_effects(sample, &cancel).unwrap();
    let retained = painter.spells.clone();
    painter.spells = super::super::battle_spells::arrow_tests::without_effects(&retained);
    let control = painter.render_effects(sample, &cancel).unwrap();
    painter.spells = retained;
    assert_ne!(
        visible.bgra, control.bgra,
        "projectile/impact actually changes world pixels: {name}"
    );
    assert_eq!(visible.observation.data(), control.observation.data());
    assert_eq!(
        visible.bgra,
        painter.render_effects(sample, &cancel).unwrap().bgra
    );
    if name == "arrow-npc" {
        let mut replacement = trace_scene(root, trace.last().unwrap());
        assert!(replacement
            .carry_from(&mut painter, sample, &AtomicBool::new(true))
            .is_err());
        replacement
            .carry_from(&mut painter, sample, &cancel)
            .unwrap();
        let carried = replacement.render_effects(sample, &cancel).unwrap();
        assert_eq!(
            carried.bgra, visible.bgra,
            "flight target/phase survives bank reload"
        );
        assert_eq!(
            carried.audio.as_ref().unwrap().events[0].sequence,
            visible.audio.as_ref().unwrap().events[0].sequence
        );
        assert!(painter.render_effects(sample, &cancel).is_err());
        painter = replacement;
    }
    let terminal = painter
        .render_effects(at + Duration::from_secs(30), &cancel)
        .unwrap();
    assert!(terminal.audio.unwrap().events.is_empty());
    assert_eq!(terminal.observation.data(), trace.last().unwrap());
    eprintln!("canonical {name}: typed launch-time target, immutable native60ms path/167->219, PCM97 one-shot, hurt/retirement/life-safe, bounded late launch, actual world pixels and finite retirement");
}

fn verify_spells(root: &Path, name: &str, trace: &[d::UnvalidatedData]) {
    let cancel = AtomicBool::new(false);
    let mut painter = trace_scene(root, &trace[0]);
    super::super::battle_spells::tests::verify(
        observation(trace[0].clone()),
        observation(trace[1].clone()),
        observation(trace[2].clone()),
        &painter.spell_assets,
        &painter.input.scene,
    );
    let at = Instant::now() + Duration::from_secs(60);
    painter
        .observe_at(observation(trace[1].clone()), at, &cancel)
        .unwrap();
    let body = painter
        .spells
        .overrides(&painter.input.scene, &painter.spell_assets, at)
        .unwrap();
    assert_eq!(body.len(), 1);
    let gfx = body[0].graphics_id;
    let source = body[0].source_frame;
    let start = painter.render_effects(at, &cancel).unwrap();
    let actor = start
        .geometry
        .actors
        .iter()
        .find(|a| a.entity_id == 1)
        .unwrap();
    assert_eq!((actor.graphics_id, actor.source_frame), (gfx, source));
    // Sample the qualified launch window, without hardware playback or a GUI.
    let mut heard = Vec::new();
    let launch = super::super::battle_spells::tests::launch_deadline(&painter.spells);
    for millis in [0, 50, 100, 200] {
        let frame = painter
            .render_effects(launch + Duration::from_millis(millis), &cancel)
            .unwrap();
        if let Some(audio) = frame.audio {
            for event in audio.events.iter() {
                if !heard.contains(&event.sequence) {
                    heard.push(event.sequence);
                }
            }
        }
        assert_eq!(frame.observation.data(), &trace[1]);
    }
    assert_eq!(
        heard.len(),
        1,
        "one immutable audio event exposed across surfaces: {name}"
    );
    let sample = launch + Duration::from_millis(200);
    let previous = painter.render_effects(sample, &cancel).unwrap();
    painter
        .observe_at(observation(trace[1].clone()), sample, &cancel)
        .unwrap();
    let duplicate = painter.render_effects(sample, &cancel).unwrap();
    assert_eq!(previous.bgra, duplicate.bgra);
    if name == "spell77-critical" {
        let mut next = trace_scene(root, &trace[1]);
        assert!(next
            .carry_from(&mut painter, sample, &AtomicBool::new(true))
            .is_err());
        assert!(painter.terminal.is_none());
        next.carry_from(&mut painter, sample, &cancel).unwrap();
        let carried = next.render_effects(sample, &cancel).unwrap();
        assert_eq!(
            previous.bgra, carried.bgra,
            "spell clock handoff pixel identity"
        );
        assert_eq!(
            previous
                .audio
                .as_ref()
                .unwrap()
                .events
                .iter()
                .map(|e| e.sequence)
                .collect::<Vec<_>>(),
            carried
                .audio
                .as_ref()
                .unwrap()
                .events
                .iter()
                .map(|e| e.sequence)
                .collect::<Vec<_>>()
        );
        assert!(painter.render_effects(sample, &cancel).is_err());
        painter = next;
    }
    painter
        .observe_at(
            observation(trace[2].clone()),
            sample + Duration::from_millis(1),
            &cancel,
        )
        .unwrap();
    painter
        .render_effects(sample + Duration::from_millis(1), &cancel)
        .unwrap();
    let final_frame = painter
        .render_effects(at + Duration::from_secs(20), &cancel)
        .unwrap();
    assert!(final_frame.audio.unwrap().events.is_empty());
    assert_eq!(final_frame.observation.data(), &trace[2]);
    eprintln!("canonical {name}: shared caster18/effect/PCM, one cast regardless victim count, exact fixed-cell binding, no historical/duplicate/stalled replay, target life independent, finite retirement");
}

fn verify_teleport_and_reload(root: &Path, trace: &[d::UnvalidatedData]) {
    use crate::modern_lito::{
        battle_teleport,
        world_map::{PixelPoint, ViewProjection},
    };
    let cancel = AtomicBool::new(false);
    let mut scene = trace_scene(root, &trace[0]);
    // Leave room for synchronous resource qualification while all samples use
    // one deterministic presentation clock, not a hardware FPS claim.
    let at = Instant::now() + Duration::from_secs(20);
    let before = scene.render_effects(at, &cancel).unwrap();
    scene
        .observe_at(observation(trace[1].clone()), at, &cancel)
        .unwrap();
    let first = scene.render_effects(at, &cancel).unwrap();
    assert_eq!(
        first.observation.data().actors.0[0].position,
        trace[1].actors.0[0].position
    );
    assert_eq!(before.observation.data(), &trace[0]);
    let bank = scene.teleport_assets.as_ref().unwrap();
    battle_teleport::tests::verify_audio(
        observation(trace[0].clone()),
        observation(trace[1].clone()),
        bank,
    );
    assert!(before.audio.as_ref().unwrap().events.is_empty());
    let sound = first.audio.as_ref().unwrap();
    assert_eq!(sound.events.len(), 1);
    assert_eq!(sound.events[0].group.candidates(), [119]);
    assert!(sound.events[0].bank.as_ref().unwrap().get(119).is_some());
    let overrides = scene
        .teleports
        .overrides(&scene.input.scene, bank, at)
        .unwrap();
    assert_eq!(
        overrides.len(),
        1,
        "fresh confirmed teleport must play caster body"
    );
    assert_eq!(overrides[0].entity_id, 1);
    let receipt = trace[1].actors.0[0]
        .spells
        .histories
        .0
        .iter()
        .find(|s| s.lane == d::Lane::Teleport)
        .unwrap()
        .receipt
        .as_ref()
        .unwrap();
    let origin = cell(&receipt.impact.usage.before.cell);
    let projection = ViewProjection {
        anchor_cell: origin,
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    };
    let mut actual = Canvas::new(800, 600).unwrap();
    let mut expected = Canvas::new(800, 600).unwrap();
    actual.clear([20, 30, 40]);
    expected.clear([20, 30, 40]);
    scene
        .teleports
        .draw(
            &mut actual,
            projection,
            bank,
            at + Duration::from_millis(200),
            &cancel,
        )
        .unwrap();
    assert!(bank
        .draw_effect(
            &mut expected,
            projection.anchor_pixel,
            Duration::from_millis(200)
        )
        .unwrap());
    assert_eq!(
        actual.rgba(),
        expected.rgba(),
        "teleport effect must keep exact receipt origin"
    );
    let mut empty = Canvas::new(800, 600).unwrap();
    empty.clear([20, 30, 40]);
    assert_ne!(actual.rgba(), empty.rgba());
    let historical = battle_teleport::Timeline::new(observation(trace[1].clone()), at);
    historical
        .draw(&mut empty, projection, bank, at, &cancel)
        .unwrap();
    assert!(historical
        .overrides(&scene.input.scene, bank, at)
        .unwrap()
        .is_empty());
    let mut finished = Canvas::new(800, 600).unwrap();
    finished.clear([20, 30, 40]);
    scene
        .teleports
        .draw(
            &mut finished,
            projection,
            bank,
            at + Duration::from_secs(10),
            &cancel,
        )
        .unwrap();
    assert_eq!(finished.rgba(), empty.rgba());
    let resumed_at = at + Duration::from_millis(250);
    let expected = scene.render_effects(resumed_at, &cancel).unwrap();
    let mut replacement = trace_scene(root, &trace[1]);
    assert!(replacement
        .carry_from(&mut scene, resumed_at, &AtomicBool::new(true))
        .is_err());
    assert!(scene.terminal.is_none());
    assert!(replacement.carry_from(&mut scene, at, &cancel).is_err());
    assert!(scene.terminal.is_none());
    replacement
        .carry_from(&mut scene, resumed_at, &cancel)
        .unwrap();
    assert!(scene.render_effects(resumed_at, &cancel).is_err());
    let resumed = replacement.render_effects(resumed_at, &cancel).unwrap();
    assert_eq!(
        resumed.bgra, expected.bgra,
        "resource handoff must preserve current caster/effect clocks"
    );
    assert_eq!(resumed.geometry.actors, expected.geometry.actors);
    assert_eq!(
        resumed.audio.as_ref().unwrap().events[0].sequence,
        sound.events[0].sequence
    );
    assert_eq!(
        resumed.audio.as_ref().unwrap().events[0].created,
        sound.events[0].created
    );
    replacement
        .observe_at(observation(trace[1].clone()), resumed_at, &cancel)
        .unwrap();
    assert_eq!(
        replacement
            .render_effects(resumed_at, &cancel)
            .unwrap()
            .bgra,
        resumed.bgra
    );
    replacement
        .observe_at(
            observation(trace[2].clone()),
            at + Duration::from_secs(2),
            &cancel,
        )
        .unwrap();
    let second = replacement
        .render_effects(at + Duration::from_secs(2), &cancel)
        .unwrap();
    assert_eq!(second.observation.data(), &trace[2]);
    assert_eq!(second.audio.as_ref().unwrap().events.len(), 1);
    assert!(second.audio.as_ref().unwrap().events[0].sequence > sound.events[0].sequence);
    assert_eq!(
        replacement
            .teleports
            .overrides(
                &replacement.input.scene,
                replacement.teleport_assets.as_ref().unwrap(),
                at + Duration::from_secs(2)
            )
            .unwrap()
            .len(),
        1
    );
    let moved_at = at + Duration::from_millis(2100);
    replacement
        .observe_at(observation(trace[3].clone()), moved_at, &cancel)
        .unwrap();
    assert!(
        replacement
            .teleports
            .overrides(
                &replacement.input.scene,
                replacement.teleport_assets.as_ref().unwrap(),
                moved_at
            )
            .unwrap()
            .is_empty(),
        "confirmed movement must release the casting pose"
    );
    let walking = replacement.render_effects(moved_at, &cancel).unwrap();
    let from = cell(&trace[2].actors.0[0].position);
    let to = cell(&trace[3].actors.0[0].position);
    assert_ne!(from, to);
    assert_eq!(walking.geometry.projection.anchor_cell, from);
    let actor = walking
        .geometry
        .actors
        .iter()
        .find(|a| a.entity_id == 1)
        .unwrap();
    assert_eq!(actor.cell, to, "visual origin must not roll back authority");
    assert_eq!(actor.source_frame, 1, "native walk begins at source1");
    assert_eq!(
        actor.anchor,
        walking.geometry.projection.cell_to_pixel(from).unwrap()
    );
    assert_eq!(walking.observation.data(), &trace[3]);
    let repeated = replacement.render_effects(moved_at, &cancel).unwrap();
    assert_eq!(walking.bgra, repeated.bgra);
    assert_eq!(walking.geometry.actors, repeated.geometry.actors);
    let settled_at =
        moved_at + Duration::from_nanos(trace[3].actors.0[0].step_interval_nanos as u64);
    let settled = replacement.render_effects(settled_at, &cancel).unwrap();
    assert_eq!(settled.geometry.projection.anchor_cell, to);
    assert_eq!(
        settled.geometry.projection.anchor_pixel,
        PixelPoint { x: 400, y: 300 }
    );
    assert_eq!(settled.observation.data(), &trace[3]);
    let done = at + Duration::from_secs(12);
    assert!(replacement
        .teleports
        .overrides(
            &replacement.input.scene,
            replacement.teleport_assets.as_ref().unwrap(),
            done
        )
        .unwrap()
        .is_empty());
    assert!(replacement
        .render_effects(done, &cancel)
        .unwrap()
        .audio
        .unwrap()
        .events
        .is_empty());
    eprintln!("canonical teleport: receipt-origin pixels/119 cue, caster at confirmed destination, 500ms inclusive age/no restart, no baseline/duplicate/stalled replay, cancelled/backward handoff atomic, successful handoff pixel/cue-identical; no hardware output claim");
}
