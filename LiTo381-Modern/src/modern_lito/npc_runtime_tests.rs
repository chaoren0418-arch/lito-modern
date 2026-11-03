//! NPC integration tests. A Java-captured snapshot is used as input; changing
//! its geometry below is an explicit compositor fixture, not a live server map.
use super::*;
use crate::modern_lito::{npc_assets::NpcAppearance, npc_presentation::Plan, npc_protocol as npc};

fn fixture() -> npc::Reply {
    npc::tests::read_reply()
}
fn plan(r: &npc::Reply) -> Arc<Plan> {
    let f = r.frame.as_ref().unwrap();
    let forms = f
        .actors
        .iter()
        .map(|n| NpcAppearance::new(n.graphic).unwrap())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    Arc::new(Plan::new(f.scope, forms).unwrap())
}
fn input(r: npc::Reply, p: Arc<Plan>, now: Instant) -> Arc<Input> {
    let f = r.frame.as_ref().unwrap();
    let mut i = Input::with_npc(
        Arc::new(f.combat.world.clone()),
        1,
        Some(Arc::new(f.combat.clone())),
        Some(NpcInput {
            reply: Arc::new(r),
            plan: p,
        }),
    )
    .unwrap();
    i.received = now;
    Arc::new(i)
}
fn advanced(r: &npc::Reply) -> npc::Reply {
    let mut r = r.clone();
    let f = r.frame.as_mut().unwrap();
    f.combat.world.revision += 1;
    f.revision += 1;
    f.elapsed_millis += 1;
    r
}
fn attack(r: &npc::Reply) -> npc::Reply {
    let mut r = advanced(r);
    let f = r.frame.as_mut().unwrap();
    f.combat.vitals[0].attack_sequence += 1;
    f.actors[0].hp -= 14;
    f.actors[0].changed_revision = f.revision;
    let e = f
        .exchanges
        .iter_mut()
        .find(|e| e.direction == npc::Direction::PlayerToNpc)
        .unwrap();
    e.revision = f.combat.world.revision;
    e.request_sequence += 1;
    e.outcome.remaining_hp -= 14;
    r.validate().unwrap();
    r
}
fn abort(r: &npc::Reply) -> npc::Reply {
    let mut r = advanced(r);
    let f = r.frame.as_mut().unwrap();
    f.combat.world.phase = Phase::Aborted;
    f.combat.world.abort_reason = world_protocol::AbortReason::ServerAbort;
    f.closed = true;
    f.actors.clear();
    f.exchanges
        .retain(|e| e.direction == npc::Direction::PlayerToNpc);
    r.validate().unwrap();
    r
}
#[test]
fn authoritative_npc_abort_clears_pending_frame_but_foreign_abort_cannot() {
    let r = fixture();
    let p = plan(&r);
    let mut runtime = WorldRuntime::with_loader(
        PathBuf::from("never-opened"),
        Arc::new(|_, _, _, _, _| panic!("aborted preflight must never start")),
    );
    runtime
        .update_npc(Arc::new(r.clone()), 1, p.clone())
        .unwrap();
    let mut foreign = abort(&r);
    foreign.echo.context.epoch += 1;
    assert!(runtime.update_npc(Arc::new(foreign), 1, p.clone()).is_err());
    assert!(runtime.desired.is_some());
    runtime.update_npc(Arc::new(abort(&r)), 1, p).unwrap();
    assert!(runtime.desired.is_none());
    assert!(runtime.is_quiescent());
    assert!(matches!(runtime.poll(), RuntimeStatus::Idle));
}

#[test]
fn preflight_cannot_be_inferred_from_empty_loading_or_silently_dropped() {
    let r = fixture();
    let f = r.frame.as_ref().unwrap();
    assert!(Input::with_combat(
        Arc::new(f.combat.world.clone()),
        1,
        Some(Arc::new(f.combat.clone()))
    )
    .is_err());
    assert!(Plan::new(f.scope, vec![]).is_err());
    assert!(Plan::new(f.scope, vec![NpcAppearance::ALL[0]; 2]).is_err());
    let p = plan(&r);
    let mut loading = r.clone();
    let f = loading.frame.as_mut().unwrap();
    f.combat.world.phase = Phase::Loading;
    f.combat.world.revision = 2;
    f.revision = 1;
    f.elapsed_millis = 0;
    f.spawned = 0;
    f.actors.clear();
    f.exchanges.clear();
    for v in &mut f.combat.vitals {
        v.hp = v.max_hp;
        v.attack_sequence = 0;
        v.attack_revision = 0;
        v.hurt_revision = 0;
        v.death_revision = 0;
    }
    let i = input(loading, p.clone(), Instant::now());
    assert!(i.npc.is_some());
    let a = input(r.clone(), p, Instant::now());
    let mut b = advanced(&r);
    b.frame.as_mut().unwrap().actors[0].graphic = 1578;
    assert!(a.npc.as_ref().unwrap().plan.validate(&b).is_err());
}
#[test]
fn plan_context_and_generation_are_pinned_not_just_the_graphic() {
    let r = fixture();
    let p = plan(&r);
    let now = Instant::now();
    let a = input(r.clone(), p.clone(), now);
    let extra = Arc::new(
        Plan::new(
            r.echo.world,
            vec![NpcAppearance::ALL[0], NpcAppearance::new(1578).unwrap()],
        )
        .unwrap(),
    );
    let b = input(advanced(&r), extra, now);
    assert!(!follows(&a, &b));
    let mut wrong = r.clone();
    wrong.echo.context.epoch += 1;
    assert!(!follows(&a, &input(wrong, p.clone(), now)));
    let mut scope = r.echo.world;
    scope.generation += 1;
    assert!(Plan::new(scope, vec![NpcAppearance::ALL[0]])
        .unwrap()
        .validate(&r)
        .is_err());
    let f = r.frame.as_ref().unwrap();
    assert!(Input::with_npc(
        Arc::new(f.combat.world.clone()),
        2,
        Some(Arc::new(f.combat.clone())),
        a.npc.clone()
    )
    .is_err());
}
#[test]
fn npc_receipt_animates_player_without_faking_pvp_target_or_replaying_poll() {
    let r = fixture();
    let p = plan(&r);
    let now = Instant::now();
    let mut t = Timeline::new(input(r.clone(), p.clone(), now));
    assert_eq!(
        t.snapshot(now).unwrap().actors()[0].action,
        BaseAction::Idle
    );
    let attacked = attack(&r);
    let f = attacked.frame.as_ref().unwrap();
    assert_eq!(f.combat.vitals[0].attack_revision, 0);
    assert_eq!(f.combat.vitals[0].last_target_entity_id, 0);
    let next = now + Duration::from_millis(100);
    t.update(input(attacked.clone(), p.clone(), next)).unwrap();
    let s = t.snapshot(next).unwrap();
    assert_eq!(s.actors()[0].action, BaseAction::Attack);
    assert_eq!(s.actors()[0].heading, 2);
    t.update(input(attacked, p, next + Duration::from_millis(100)))
        .unwrap();
    assert!(
        t.snapshot(next + Duration::from_millis(100))
            .unwrap()
            .actors()[0]
            .action_elapsed
            > Duration::ZERO
    );
}
#[test]
fn newer_hurt_and_confirmed_move_supersede_coalesced_npc_attack() {
    let r = fixture();
    let p = plan(&r);
    let now = Instant::now();
    let mut hurt = advanced(&attack(&r));
    let f = hurt.frame.as_mut().unwrap();
    f.combat.vitals[0].hurt_revision = f.combat.world.revision;
    f.combat.vitals[0].hp -= 1;
    let mut t = Timeline::new(input(r.clone(), p.clone(), now));
    t.update(input(hurt, p.clone(), now + Duration::from_millis(10)))
        .unwrap();
    assert_eq!(
        t.snapshot(now + Duration::from_millis(10))
            .unwrap()
            .actors()[0]
            .action,
        BaseAction::Damage
    );
    let mut moved = advanced(&attack(&r));
    let f = moved.frame.as_mut().unwrap();
    f.combat.world.actors[0].position.y += 1;
    f.combat.world.actors[0].sequence += 1;
    let mut t = Timeline::new(input(r, p.clone(), now));
    t.update(input(moved, p, now + Duration::from_millis(10)))
        .unwrap();
    assert_eq!(
        t.snapshot(now + Duration::from_millis(10))
            .unwrap()
            .actors()[0]
            .action,
        BaseAction::Walk
    );
}
#[test]
fn npc_health_ratio_remains_exact_past_f64_integer_range() {
    let mut c = Canvas::new(64, 32).unwrap();
    let anchor = PixelPoint { x: 32, y: 20 };
    let top = PixelPoint { x: 32, y: 20 };
    draw_compact_health_bar(
        &mut c,
        9_007_199_254_740_992,
        9_007_199_254_740_993,
        anchor,
        top,
    )
    .unwrap();
    // Exactly one below full must leave the 48th interior column red.
    let pixel = (14 * 64 + 55) * 4;
    assert_eq!(&c.rgba()[pixel..pixel + 4], &[110, 30, 30, 255]);
    assert!(draw_compact_health_bar(&mut c, -1, i64::MAX, anchor, top).is_err());
    assert!(draw_compact_health_bar(&mut c, 1, 0, anchor, top).is_err());
}
#[test]
fn failed_npc_preflight_never_publishes_ready_or_loaded_ack() {
    let r = fixture();
    let p = plan(&r);
    let loads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = loads.clone();
    let mut runtime = WorldRuntime::with_loader(
        PathBuf::from("never-opened"),
        Arc::new(move |_, _, _, npc, _| {
            assert!(npc.is_some());
            count.fetch_add(1, Ordering::AcqRel);
            anyhow::bail!("test missing NPC SPR or sound")
        }),
    );
    runtime.update_npc(Arc::new(r), 1, p).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match runtime.poll() {
            RuntimeStatus::Ready(_) => panic!("unqualified NPC resources acknowledged"),
            RuntimeStatus::Failed(_) => break,
            _ => {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(1));
            }
        }
    }
    assert_eq!(loads.load(Ordering::Acquire), 1);
    runtime.shutdown();
    while !runtime.is_quiescent() {
        runtime.poll();
        assert!(Instant::now() < deadline);
    }
}

#[test]
#[ignore = "requires explicit shared Lineage root; real map554/PC/NPC art and audio preflight, no GUI or server admission"]
fn actual_shared_npcs_preload_once_and_render_typed_geometry_on_the_world_worker() {
    let root =
        PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"));
    let cancel = AtomicBool::new(false);
    let bindings = crate::modern_lito::quest_resource_probe_tests::bindings();
    let map = WorldMap::load_shared(&root, &bindings, &cancel).unwrap();
    let mut r = fixture();
    let f = r.frame.as_mut().unwrap();
    let anchor = WorldCell { x: 32778, y: 32823 };
    let mut cells = vec![anchor];
    for y in anchor.y - 4..=anchor.y + 4 {
        for x in anchor.x - 4..=anchor.x + 4 {
            let p = WorldCell { x, y };
            if p != anchor && map.standing(p) {
                cells.push(p);
            }
        }
    }
    assert!(cells.len() >= 5);
    for (a, cell) in f.combat.world.actors.iter_mut().zip(&cells) {
        a.position = *cell;
    }
    for (a, cell) in f.actors.iter_mut().zip(&cells[3..]) {
        a.cell = *cell;
    }
    // Exercise direct body-linked attr105/attr8 with the second visual fixture.
    f.actors[1].graphic = 1578;
    f.combat.world.manifest = world_protocol::Manifest {
        version: 1,
        map_id: 554,
        digest: map.canonical_digest(),
        sectors: bindings
            .iter()
            .map(|b| world_protocol::Sector {
                x: b.key.sector_x,
                y: b.key.sector_y,
                byte_length: b.byte_length,
                sha256: b.sha256,
            })
            .collect(),
    };
    r.validate().unwrap();
    let p = plan(&r);
    let now = Instant::now();
    let initial = input(r.clone(), p.clone(), now);
    super::super::shared_action_catalog::test_counts::reset();
    let mut renderer = load_shared_renderer_npcs(
        &root,
        &initial.frame,
        initial.combat.as_deref(),
        initial.npc.as_ref(),
        &cancel,
    )
    .unwrap();
    let counts = super::super::shared_action_catalog::test_counts::snapshot();
    assert_eq!(
        (counts.load_attempts, counts.reads, counts.parses),
        (1, 1, 1)
    );
    let mut timeline = Timeline::new(initial.clone());
    let snapshot = timeline.snapshot(now).unwrap();
    let a = renderer
        .render_at_npcs(
            &snapshot,
            initial.combat.as_deref(),
            initial.npc.as_ref(),
            &cancel,
            now,
        )
        .unwrap();
    assert_eq!(
        (a.actors.len(), a.npcs.len(), a.objectives.len()),
        (3, 2, 0)
    );
    assert_eq!(a.bgra.len(), PIXEL_BYTES);
    assert!(
        a.audio.as_ref().unwrap().events.is_empty(),
        "historical attack replay"
    );
    assert_eq!(a.npcs[0].id.scope, r.echo.world);
    assert!(a.npcs.iter().all(|n| n.life == 1 && n.alive));
    assert!(a.npcs.iter().any(|n| n.graphics_id == 1578));
    // Use actual qualified duration; these NPC idle sources are not all 150 ms.
    let dwell = renderer
        .npc
        .as_ref()
        .unwrap()
        .assets
        .action(
            NpcAppearance::ALL[0],
            crate::modern_lito::npc_assets::NpcAction::Breath,
        )
        .unwrap()
        .source_frames()[1]
        .duration_ticks
        * 1000
        / 24;
    let b = renderer
        .render_at_npcs(
            &snapshot,
            initial.combat.as_deref(),
            initial.npc.as_ref(),
            &cancel,
            now + Duration::from_millis(u64::from(dwell)),
        )
        .unwrap();
    assert!(a
        .npcs
        .iter()
        .zip(&b.npcs)
        .any(|(a, b)| a.source_frame != b.source_frame));
    assert_eq!(
        super::super::shared_action_catalog::test_counts::snapshot(),
        counts
    );
    assert!(renderer
        .render_at(&snapshot, initial.combat.as_deref(), &cancel, now)
        .is_err());
    // The real owned worker loads the same resources before publishing ACK.
    drop(renderer);
    let mut runtime = WorldRuntime::new(root).unwrap();
    runtime
        .update_npc(Arc::new(r.clone()), 1, p.clone())
        .unwrap();
    assert!(matches!(runtime.poll(), RuntimeStatus::Loading { .. }));
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        match runtime.poll() {
            RuntimeStatus::Ready(image) => {
                assert!(image.ready_to_ack);
                assert_eq!(image.npcs.len(), 2);
                assert!(image.npc.is_some());
                break;
            }
            RuntimeStatus::Failed(e) => panic!("NPC resource worker failed: {e}"),
            _ => {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(5));
            }
        }
    }
    runtime.update_npc(Arc::new(abort(&r)), 1, p).unwrap();
    assert!(!matches!(runtime.poll(), RuntimeStatus::Ready(_)));
    runtime.shutdown();
    while !runtime.is_quiescent() {
        runtime.poll();
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    assert!(matches!(runtime.poll(), RuntimeStatus::Idle));
}
