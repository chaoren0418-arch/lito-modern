//! Shared SPR/audio and real compositor; frozen Java snapshots on synthetic S32.
use super::*;
use crate::modern_lito::{
    disintegrate_presentation::npc_tests::reply, npc_presentation::Plan, npc_protocol as npc,
};
fn input(r: npc::Reply, p: Arc<Plan>, at: Instant) -> Arc<Input> {
    let f = r.frame.as_ref().unwrap();
    let id = f
        .combat
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(r.echo.context.key))
        .unwrap()
        .entity_id;
    let mut i = Input::with_npc(
        Arc::new(f.combat.world.clone()),
        id,
        Some(Arc::new(f.combat.clone())),
        Some(NpcInput {
            reply: Arc::new(r),
            plan: p,
        }),
    )
    .unwrap();
    i.received = at;
    Arc::new(i)
}
#[test]
fn npc77_timeline_clears_prior_body_without_changing_npc_authority() {
    let before = reply("normal-before");
    let hit = reply("normal-hit");
    let t = Instant::now();
    let plan = Arc::new(Plan::from_reply(&before).unwrap());
    let mut timeline = Timeline::new(input(before, plan.clone(), t));
    timeline
        .update(input(hit.clone(), plan, t + Duration::from_millis(100)))
        .unwrap();
    assert!(timeline.actors[&1].pose.is_none());
    assert!(timeline.actors[&1].walk_until.is_none());
    assert_eq!(timeline.input.npc.as_ref().unwrap().reply.as_ref(), &hit);
}
fn sound(p: &Pixels) -> Vec<u64> {
    p.audio
        .as_ref()
        .unwrap()
        .events
        .iter()
        .filter(|e| e.group.candidates() == [717])
        .map(|e| e.sequence)
        .collect()
}
#[test]
#[ignore = "requires shared Lineage SPR/caster18/Sound717; actual compositor with frozen Java NPC77 receipts, not HWND or production map"]
fn actual_npc77_runtime_shared_sprites_pixel_difference_audio_and_terminal_dwell() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
    let cancel = AtomicBool::new(false);
    for prefix in ["normal", "critical"] {
        let before = reply(&format!("{prefix}-before"));
        let hit = reply(&format!("{prefix}-hit"));
        let f = &before.frame.as_ref().unwrap().combat;
        let map = crate::modern_lito::siege_space::tests::map(&f.world);
        assert_eq!(map.canonical_digest(), f.world.manifest.digest);
        let plan = Arc::new(Plan::from_reply(&before).unwrap());
        let first = NpcInput {
            reply: Arc::new(before.clone()),
            plan: plan.clone(),
        };
        let mut renderer =
            load_qualified_map_renderer(&root, &f.world, Some(f), Some(&first), &cancel, map)
                .unwrap();
        let t = Instant::now();
        let mut timeline = Timeline::new(input(before.clone(), plan.clone(), t));
        let p = renderer
            .render_at_npcs(
                &timeline.snapshot(t).unwrap(),
                Some(f),
                Some(&first),
                &cancel,
                t,
            )
            .unwrap();
        assert!(sound(&p).is_empty());
        let at = t + Duration::from_millis(50);
        timeline
            .update(input(hit.clone(), plan.clone(), at))
            .unwrap();
        let scene = timeline.snapshot(at).unwrap();
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
        let next = NpcInput {
            reply: Arc::new(hit.clone()),
            plan,
        };
        let nf = hit.frame.as_ref().unwrap();
        let f = &nf.combat;
        renderer
            .render_at_npcs(&scene, Some(f), Some(&next), &cancel, at)
            .unwrap();
        assert!(renderer
            .disintegrate
            .as_ref()
            .unwrap()
            .draws(at)
            .unwrap()
            .is_empty());
        let launch = at + delay;
        let scene = timeline.snapshot(launch).unwrap();
        let p = renderer
            .render_at_npcs(&scene, Some(f), Some(&next), &cancel, launch)
            .unwrap();
        assert_eq!(sound(&p).len(), 1);
        let repeated = renderer
            .render_at_npcs(&scene, Some(f), Some(&next), &cancel, launch)
            .unwrap();
        assert_eq!(p.bgra, repeated.bgra);
        assert_eq!(sound(&p), sound(&repeated));
        let visible_at = launch + Duration::from_millis(125);
        let scene = timeline.snapshot(visible_at).unwrap();
        let visible = renderer
            .render_at_npcs(&scene, Some(f), Some(&next), &cancel, visible_at)
            .unwrap();
        super::disintegrate_runtime_tests::capture(&format!("npc77-{prefix}"), &visible);
        let dis = renderer.disintegrate.as_ref().unwrap();
        let effects = dis.draws(visible_at).unwrap();
        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0].binding().target,
            f.npc_disintegrate(1).unwrap().hit.as_ref().unwrap().to
        );
        let overrides = dis
            .overrides(&renderer.light_arrow.as_ref().unwrap().assets, visible_at)
            .unwrap();
        let npcs = renderer.npc.as_ref().unwrap().draws().unwrap();
        let roof = renderer
            .assets
            .sample_roofs(&mut renderer.roofs, &scene, visible_at)
            .unwrap();
        let mut without = Canvas::new(WIDTH, HEIGHT).unwrap();
        let info = renderer
            .assets
            .render_into_npcs_roofs_effects(
                &scene,
                &mut without,
                &cancel,
                &overrides,
                &[],
                &npcs,
                Some(&roof),
                &[],
            )
            .unwrap();
        draw_health_bars(&mut without, &info.actors, &f.vitals).unwrap();
        for p in &info.npcs {
            let a = nf.actor(p.id).unwrap();
            if a.alive() {
                draw_compact_health_bar(&mut without, a.hp, a.max_hp, p.anchor, p.top_left)
                    .unwrap();
            }
        }
        let mut bytes = Vec::new();
        without.copy_bgra(&mut bytes).unwrap();
        assert_ne!(
            visible.bgra.as_ref(),
            bytes.as_slice(),
            "only fixed-cell NPC77 effect omitted"
        );
        assert_eq!(visible.actors, info.actors);
        assert_eq!(visible.npcs, info.npcs);
        for ns in [2_375_000_000, 2_499_999_999, 2_500_000_000] {
            let at = launch + Duration::from_nanos(ns);
            let p = renderer
                .render_at_npcs(
                    &timeline.snapshot(at).unwrap(),
                    Some(f),
                    Some(&next),
                    &cancel,
                    at,
                )
                .unwrap();
            assert_eq!(
                renderer
                    .disintegrate
                    .as_ref()
                    .unwrap()
                    .draws(at)
                    .unwrap()
                    .len(),
                usize::from(ns < 2_500_000_000)
            );
            assert!(sound(&p).is_empty());
        }
        assert_eq!(hit, reply(&format!("{prefix}-hit")));
    }
}
