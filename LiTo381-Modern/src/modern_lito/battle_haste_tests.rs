use super::*;

#[test]
fn receipt_age_is_bounded_and_checked() {
    assert_eq!(age(1, 1), Some(Duration::ZERO));
    assert_eq!(age(1, 500_000_001), Some(Duration::from_millis(500)));
    assert_eq!(age(1, 500_000_002), None);
    assert_eq!(age(1, 0), None);
    assert_eq!(age(i64::MIN, i64::MAX), None);
}
#[test]
fn no_history_never_invents_effects_and_cancellation_is_atomic() {
    let o = super::super::battle_scene::tests::observation(
        super::super::battle_payload::data_tests::fixture(),
    );
    assert!(!enabled(&o));
    let now = Instant::now();
    let t = Timeline::new(o.clone(), now);
    let assets = Assets { effect: None };
    let out = t
        .advance(o.clone(), now, &assets, &AtomicBool::new(false))
        .unwrap();
    assert!(out.effects.is_empty() && out.casts.is_empty());
    assert!(t
        .advance(o.clone(), now, &assets, &AtomicBool::new(true))
        .is_err());
    assert!(t
        .advance(
            o,
            now - Duration::from_nanos(1),
            &assets,
            &AtomicBool::new(false)
        )
        .is_err());
    assert!(t.effects.is_empty());
}

pub(crate) fn counts(t: &Timeline) -> (usize, usize) {
    (t.effects.len(), t.casts.len())
}
pub(crate) fn without_effects(t: &Timeline) -> Timeline {
    let mut out = t.clone();
    out.effects.clear();
    out
}
pub(crate) fn verify_frames(frames: &[d::UnvalidatedData]) {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared assets"),
    );
    assert!(root.is_absolute());
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    let obs = super::super::battle_scene::tests::observation;
    let before = obs(frames[0].clone());
    let used = obs(frames[1].clone());
    let assets = Assets::load(&catalog, &before, &cancel).unwrap();
    let now = Instant::now() + Duration::from_secs(2);
    let initial = Timeline::new(before, now);
    let applied = initial
        .advance(used.clone(), now, &assets, &cancel)
        .unwrap();
    assert_eq!(counts(&applied), (1, 1));
    assert_eq!(
        counts(
            &applied
                .advance(
                    used.clone(),
                    now + Duration::from_millis(16),
                    &assets,
                    &cancel
                )
                .unwrap()
        ),
        (1, 1)
    );
    assert_eq!(
        counts(
            &Timeline::new(used.clone(), now)
                .advance(used.clone(), now, &assets, &cancel)
                .unwrap()
        ),
        (0, 0),
        "historical attachment must not replay"
    );
    assert!(initial
        .advance(used.clone(), now, &assets, &AtomicBool::new(true))
        .is_err());
    assert_eq!(counts(&initial), (0, 0));
    let scene = super::super::battle_scene::tests::scene_snapshot(used.clone());
    let forms = scene
        .actors()
        .iter()
        .map(|a| a.appearance)
        .collect::<Vec<_>>();
    let body = TeleportAssets::load_with_catalog(&catalog, &forms, &cancel).unwrap();
    assert_eq!(
        applied.overrides(&scene, &body, now).unwrap().len(),
        1,
        "action19"
    );
    let mut painted = Canvas::new(800, 600).unwrap();
    let blank = painted.rgba().to_vec();
    let projection = ViewProjection {
        anchor_cell: applied.effects[0].position,
        anchor_pixel: super::super::world_map::PixelPoint { x: 400, y: 300 },
    };
    applied
        .draw(&mut painted, projection, &assets, now, &cancel)
        .unwrap();
    assert_ne!(painted.rgba(), blank, "shared3104 must produce pixels");
    let expired = applied
        .advance(used.clone(), now + Duration::from_secs(2), &assets, &cancel)
        .unwrap();
    assert_eq!(
        expired.effects.len(),
        0,
        "2400-second status is not a permanent decal"
    );
    assert!(
        applied
            .advance(obs(frames[6].clone()), now, &assets, &cancel)
            .is_err(),
        "aborted scene must retire"
    );
    let gear = Timeline::new(obs(frames[7].clone()), now);
    let gear_cast = gear
        .advance(obs(frames[8].clone()), now, &assets, &cancel)
        .unwrap();
    assert_eq!(counts(&gear_cast), (1, 1));
    let changed = gear_cast
        .advance(obs(frames[9].clone()), now, &assets, &cancel)
        .unwrap();
    assert_eq!(
        counts(&changed),
        (0, 0),
        "new equipment must not retain old cast body/effect"
    );
    let recast = changed
        .advance(obs(frames[10].clone()), now, &assets, &cancel)
        .unwrap();
    assert_eq!(counts(&recast), (1, 1));
    let revived = recast
        .advance(obs(frames[11].clone()), now, &assets, &cancel)
        .unwrap();
    assert_eq!(
        counts(&revived),
        (0, 0),
        "new life must clear old cast visuals"
    );
    let q = super::super::battle_greater_haste::Intent::from_observation(&used).unwrap();
    assert!(q.current(&used));
    super::super::battle_scene::tests::verify_haste_scene(&root, frames);
    for frame in [&frames[3], &frames[7], &frames[9], &frames[11]] {
        assert!(!q.current(&obs(frame.clone())));
    }
}
pub(crate) fn verify_neutralization(before: d::UnvalidatedData, used: d::UnvalidatedData) {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared assets"),
    );
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    let obs = super::super::battle_scene::tests::observation;
    let before = obs(before);
    let used = obs(used);
    let assets = Assets::load(&catalog, &before, &cancel).unwrap();
    let now = Instant::now() + Duration::from_secs(2);
    let initial = Timeline::new(before, now);
    let out = initial
        .advance(used.clone(), now, &assets, &cancel)
        .unwrap();
    assert_eq!(
        counts(&out),
        (1, 1),
        "paid neutralization still renders the confirmed cast"
    );
    assert_eq!(
        counts(
            &out.advance(used.clone(), now + Duration::from_secs(2), &assets, &cancel)
                .unwrap()
        ),
        (0, 1)
    );
    assert_eq!(
        counts(
            &Timeline::new(used.clone(), now)
                .advance(used, now, &assets, &cancel)
                .unwrap()
        ),
        (0, 0),
        "no historical replay"
    );
}
