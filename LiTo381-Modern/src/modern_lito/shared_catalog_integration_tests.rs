//! Real shared asset integration for the per-construction catalog lifetime.
//! Counters establish acquisitions, not timing guarantees. Standalone equality
//! is an API-equivalence check; existing independent C# goldens remain required.
use super::*;
use crate::modern_lito::{
    ammo_protocol,
    healing_effect::HealingAssets,
    honor_healing_effect::HonorHealingAssets,
    shared_action_catalog::{test_counts, SharedActionCatalog},
    spell_protocol::DamageCastState,
    timed_effect::TimedAssets,
    tower_assets::TowerAssets,
};

fn all_families() -> combat_protocol::Reply {
    let mut reply = super::honor_runtime_tests::actual_fixture();
    let frame = reply.frame.as_mut().unwrap();
    frame.damages = frame
        .world
        .actors
        .iter()
        .map(|actor| DamageCastState::initial(actor.entity_id, 3))
        .collect();
    frame.ammo = frame
        .world
        .actors
        .iter()
        .map(|actor| ammo_protocol::State::initial(actor.entity_id, Vec::new()))
        .collect();
    assert!(!frame.world.casts.is_empty());
    assert!(!frame.items.is_empty());
    assert!(!frame.timed.is_empty());
    assert!(!frame.weights.is_empty());
    reply.validate().unwrap();
    reply
}

fn counts(expected: usize) {
    assert_eq!(
        test_counts::snapshot(),
        test_counts::Counts {
            load_attempts: expected,
            reads: expected,
            parses: expected,
        }
    );
}

fn timeline(frame: &combat_protocol::Frame, selected: u64, at: Instant) -> Timeline {
    let mut input = Input::with_combat(
        Arc::new(frame.world.clone()),
        selected,
        Some(Arc::new(frame.clone())),
    )
    .unwrap();
    input.received = at;
    Timeline::new(Arc::new(input))
}

macro_rules! same_bank {
    ($actual:expr, $fresh:expr) => {{
        let actual = $actual;
        let fresh = $fresh;
        assert_eq!(actual.catalog_sha256(), fresh.catalog_sha256());
        assert_eq!(actual.encoded_bytes(), fresh.encoded_bytes());
        assert_eq!(actual.rgba_bytes(), fresh.rgba_bytes());
        assert_eq!(
            actual.sprite_sources().collect::<Vec<_>>(),
            fresh.sprite_sources().collect::<Vec<_>>()
        );
    }};
}

#[test]
#[ignore = "requires LITO_SHARED_RESOURCE_ROOT; all eight actual asset banks and full532, no TLS/SQL/GUI/audio device"]
fn actual_eight_banks_share_one_catalog_and_each_new_world_acquires_its_own() {
    let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
    let root = Path::new(&root);
    let cancel = AtomicBool::new(false);
    let reply = all_families();
    let frame = reply.frame.as_ref().unwrap();
    test_counts::reset();
    let loading = Instant::now();
    let mut renderer = load_shared_renderer(root, &frame.world, Some(frame), &cancel).unwrap();
    eprintln!(
        "Single-thread complete eight-bank shared532 load: {:.3}s",
        loading.elapsed().as_secs_f64()
    );
    counts(1);
    assert!(
        renderer.teleport.is_some()
            && renderer.light_arrow.is_some()
            && renderer.healing.is_some()
            && renderer.timed.is_some()
            && renderer.ranged.is_some()
            && renderer.honor.is_some()
    );
    let start = Instant::now();
    let mut timeline = timeline(frame, 1, start);
    let scene = timeline.snapshot(start).unwrap();
    for millis in [0, 50, 200] {
        let at = start + Duration::from_millis(millis);
        let image = renderer
            .render_at(&timeline.snapshot(at).unwrap(), Some(frame), &cancel, at)
            .unwrap();
        assert_eq!(image.bgra.len(), PIXEL_BYTES);
        assert_eq!(image.objectives.len(), 2);
        assert_eq!(image.actors.len(), 2);
        counts(1); // Per-frame rendering must not reacquire the catalog.
    }
    let appearances: Vec<_> = frame
        .world
        .room
        .members
        .iter()
        .map(|member| BaseAppearance {
            class: u8::try_from(member.class).unwrap(),
            sex: member.sex,
        })
        .collect();
    let armed: Vec<_> = appearances
        .iter()
        .zip(&frame.world.actors)
        .map(|(appearance, actor)| CombatAppearance {
            appearance: *appearance,
            weapon_type: frame.vital(actor.entity_id).unwrap().weapon_type,
        })
        .collect();
    {
        let fresh = WorldSceneAssets::load_combat_armed(
            root,
            renderer.map.clone(),
            &armed,
            SceneLimits::default(),
            &cancel,
        )
        .unwrap();
        assert_eq!(fresh.encoded_bytes(), renderer.assets.encoded_bytes());
        assert_eq!(fresh.texture_bytes(), renderer.assets.texture_bytes());
        let mut reused = Canvas::new(WIDTH, HEIGHT).unwrap();
        let mut separate = Canvas::new(WIDTH, HEIGHT).unwrap();
        let a = renderer
            .assets
            .render_into(&scene, &mut reused, &cancel)
            .unwrap();
        let b = fresh.render_into(&scene, &mut separate, &cancel).unwrap();
        assert_eq!(reused.rgba(), separate.rgba());
        assert_eq!(a.actors, b.actors);
        assert_eq!(a.projection, b.projection);
    }
    same_bank!(
        &renderer.teleport.as_ref().unwrap().assets,
        TeleportAssets::load(root, &appearances, &cancel).unwrap()
    );
    same_bank!(
        &renderer.light_arrow.as_ref().unwrap().assets,
        LightArrowAssets::load(root, &appearances, &cancel).unwrap()
    );
    same_bank!(
        &renderer.healing.as_ref().unwrap().assets,
        HealingAssets::load(root, &cancel).unwrap()
    );
    same_bank!(
        &renderer.timed.as_ref().unwrap().assets,
        TimedAssets::load(root, &cancel).unwrap()
    );
    same_bank!(
        &renderer.ranged.as_ref().unwrap().assets,
        RangedAssets::load(root, &cancel).unwrap()
    );
    same_bank!(
        &renderer.honor.as_ref().unwrap().assets,
        TowerAssets::load(root, &cancel).unwrap()
    );
    same_bank!(
        &renderer.honor.as_ref().unwrap().healing,
        HonorHealingAssets::load(root, &cancel).unwrap()
    );
    counts(9); // One common context, plus the eight explicit standalone APIs.
    drop(renderer);
    let mut next = frame.clone();
    next.world.match_id += 1;
    next.world.generation += 1;
    next.world.room.summary.revision += 1;
    next.validate().unwrap();
    let mut second = load_shared_renderer(root, &next.world, Some(&next), &cancel).unwrap();
    counts(10); // No static/global reuse across worlds, even with the same files.
    let at = Instant::now();
    let mut next_timeline = self::timeline(&next, 2, at);
    let next_scene = next_timeline.snapshot(at).unwrap();
    let image = second
        .render_at(&next_scene, Some(&next), &cancel, at)
        .unwrap();
    assert_eq!(image.bgra.len(), PIXEL_BYTES);
    assert_eq!(next_scene.world_id(), next.world.match_id);
    assert_eq!(next_scene.selected_entity_id(), 2);
    let info = second
        .assets
        .render_into(&next_scene, &mut second.canvas, &cancel)
        .unwrap();
    assert_eq!(info.projection.anchor_cell, next.world.actors[1].position);
    counts(10);
}

#[test]
#[ignore = "requires LITO_SHARED_RESOURCE_ROOT; loads only one real catalog then verifies all cancelled consumers without bank reads"]
fn actual_common_catalog_does_not_bypass_any_consumer_cancellation_gate() {
    let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
    let cancel = AtomicBool::new(false);
    test_counts::reset();
    let catalog = SharedActionCatalog::load(Path::new(&root), &cancel).unwrap();
    counts(1);
    cancel.store(true, Ordering::Release);
    let appearance = [BaseAppearance { class: 0, sex: 0 }];
    let map = Arc::new(
        WorldMap::from_sector_bytes(
            vec![crate::modern_lito::world_map::SectorInput {
                key: SharedMapSectorKey {
                    map_id: 514,
                    sector_x: 0x7fff,
                    sector_y: 0x8000,
                },
                bytes: vec![0; 49_170],
            }],
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    for result in [
        WorldSceneAssets::load_with_catalog(
            &catalog,
            map.clone(),
            &appearance,
            SceneLimits::default(),
            &cancel,
        ),
        WorldSceneAssets::load_combat_armed_with_catalog(
            &catalog,
            map,
            &[CombatAppearance {
                appearance: appearance[0],
                weapon_type: 0,
            }],
            SceneLimits::default(),
            &cancel,
        ),
    ] {
        let error = result
            .err()
            .expect("cancelled scene consumer must not load");
        assert_eq!(error.to_string(), "world scene loading/rendering cancelled");
    }
    assert!(matches!(
        TeleportAssets::load_with_catalog(&catalog, &appearance, &cancel),
        Err(crate::modern_lito::teleport_effect::TeleportError::Cancelled)
    ));
    assert!(matches!(
        LightArrowAssets::load_with_catalog(&catalog, &appearance, &cancel),
        Err(crate::modern_lito::light_arrow_effect::LightArrowError::Cancelled)
    ));
    assert!(matches!(
        HealingAssets::load_with_catalog(&catalog, &cancel),
        Err(crate::modern_lito::healing_effect::HealingError::Cancelled)
    ));
    assert!(matches!(
        TimedAssets::load_with_catalog(&catalog, &cancel),
        Err(crate::modern_lito::timed_effect::TimedError::Cancelled)
    ));
    assert!(matches!(
        RangedAssets::load_with_catalog(&catalog, &cancel),
        Err(crate::modern_lito::ranged_effect::RangedError::Cancelled)
    ));
    assert!(matches!(
        TowerAssets::load_with_catalog(&catalog, &cancel),
        Err(crate::modern_lito::tower_assets::HonorAssetError::Cancelled)
    ));
    assert!(matches!(
        HonorHealingAssets::load_with_catalog(&catalog, &cancel),
        Err(crate::modern_lito::tower_assets::HonorAssetError::Cancelled)
    ));
    counts(1);
}
