use super::*;
#[test]
fn targeted_effect_age_is_bounded() {
    assert_eq!(age(1, 1), Some(Duration::ZERO));
    assert_eq!(age(1, 500_000_001), Some(Duration::from_millis(500)));
    assert_eq!(age(1, 500_000_002), None);
    assert_eq!(age(1, 0), None);
    assert_eq!(age(i64::MIN, i64::MAX), None);
}
#[test]
fn no_target_history_never_invents_effects() {
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
}
pub(crate) fn verify(before: d::UnvalidatedData, used: d::UnvalidatedData) {
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
    let applied = initial
        .advance(used.clone(), now, &assets, &cancel)
        .unwrap();
    assert_eq!((applied.effects.len(), applied.casts.len()), (1, 1));
    let e = &applied.effects[0];
    let r = applied.casts.values().next().unwrap();
    assert_eq!(e.position, target(&r.receipt).unwrap());
    assert!(target_live(used.data(), &e.target));
    use crate::modern_lito::battle_window::{can_haste, target_current, Target};
    for a in &used.data().actors.0 {
        let target = Target::Player {
            entity: a.entity,
            life: super::super::battle_motion::life(a).unwrap(),
        };
        assert!(can_haste(&used, target));
        assert!(!can_haste(
            &used,
            Target::Player {
                entity: a.entity,
                life: super::super::battle_motion::life(a).unwrap() + 1
            }
        ));
        if a.entity == used.data().personal.entity {
            assert!(!target_current(&used, target));
        }
    }
    if let Some(npcs) = &used.data().npcs {
        for n in &npcs.actors.0 {
            assert_eq!(
                can_haste(
                    &used,
                    Target::Npc {
                        serial: n.serial,
                        life: n.life
                    }
                ),
                n.hp > 0
            );
            assert!(!can_haste(
                &used,
                Target::Npc {
                    serial: n.serial,
                    life: n.life + 1
                }
            ));
        }
    }
    assert!(!can_haste(&used, Target::Honor { entity: 3 }));
    assert!(!can_haste(&used, Target::TimeTower));
    assert!(!can_haste(&used, Target::Siege { slot: 0 }));

    let mut dead = used.data().clone();
    match &e.target {
        StatusTarget::Player(p) => {
            dead.actors
                .0
                .iter_mut()
                .find(|a| a.entity == p.participant.entity)
                .unwrap()
                .vital
                .as_mut()
                .unwrap()
                .values
                .hp = 0;
            assert!(!target_live(&dead, &e.target));
            let mut foreign = p.clone();
            foreign.participant.identity.source += 1;
            assert!(!target_live(used.data(), &StatusTarget::Player(foreign)));
            let mut reborn = p.clone();
            reborn.life += 1;
            assert!(!target_live(used.data(), &StatusTarget::Player(reborn)));
        }
        StatusTarget::Npc(n) => {
            dead.npcs
                .as_mut()
                .unwrap()
                .actors
                .0
                .iter_mut()
                .find(|a| a.serial == n.id.serial)
                .unwrap()
                .hp = 0;
            assert!(!target_live(&dead, &e.target));
            let mut foreign = n.clone();
            foreign.id.world.generation += 1;
            assert!(!target_live(used.data(), &StatusTarget::Npc(foreign)));
        }
    }
    let again = applied
        .advance(
            used.clone(),
            now + Duration::from_millis(16),
            &assets,
            &cancel,
        )
        .unwrap();
    assert_eq!(again.effects.len(), 1);
    let attached = Timeline::new(used.clone(), now)
        .advance(used.clone(), now, &assets, &cancel)
        .unwrap();
    assert!(attached.effects.is_empty() && attached.casts.is_empty());
    assert!(initial
        .advance(used.clone(), now, &assets, &AtomicBool::new(true))
        .is_err());
    assert!(initial.effects.is_empty());
    let scene = super::super::battle_scene::tests::scene_snapshot(used.clone());
    let forms = scene
        .actors()
        .iter()
        .map(|a| a.appearance)
        .collect::<Vec<_>>();
    let body = TeleportAssets::load_with_catalog(&catalog, &forms, &cancel).unwrap();
    assert_eq!(applied.overrides(&scene, &body, now).unwrap().len(), 1);
    let mut canvas = Canvas::new(800, 600).unwrap();
    let blank = canvas.rgba().to_vec();
    applied
        .draw(
            &mut canvas,
            ViewProjection {
                anchor_cell: e.position,
                anchor_pixel: super::super::world_map::PixelPoint { x: 400, y: 300 },
            },
            &assets,
            now,
            &cancel,
        )
        .unwrap();
    assert_ne!(canvas.rgba(), blank, "shared755 must produce target pixels");
    let expired = applied
        .advance(used, now + Duration::from_secs(2), &assets, &cancel)
        .unwrap();
    assert!(
        expired.effects.is_empty(),
        "1200-second status is not a permanent decal"
    );
}
