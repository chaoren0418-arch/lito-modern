use super::*;

// Use real Java command exports, then advance the global revision and its
// mandatory speed/history envelope mirrors without changing actor/target facts.
// This is a proof-admission check, not evidence of native window interaction.
macro_rules! unchanged_painted_proof {
    ($contract:ident, $request:expr, $before:expr) => {{
        let q = $request;
        let before = &$before;
        let original = $contract::encode_intent(&q, 47).unwrap();
        let mut newer = before.clone();
        newer.world.revision += 1;
        newer.speed.as_mut().unwrap().state.revision = newer.world.revision;
        newer.personal.history.as_mut().unwrap().0.world_revision = newer.world.revision;
        q.validate_before(&newer).unwrap();
        let o = super::super::battle_scene::tests::observation(newer.clone());
        assert!(
            q.current(&o),
            "unrelated world revision must preserve proof"
        );
        for case in 0..17 {
            let mut bad = q;
            match case {
                0 => bad.request_id = 0,
                1 => bad.stamp.context.key.source_id += 1,
                2 => bad.stamp.context.key.object_id += 1,
                3 => bad.stamp.context.epoch += 1,
                4 => bad.stamp.channel = (q.stamp.channel + 1) % 4,
                5 => bad.stamp.world[0] += 1,
                6 => bad.stamp.world[1] += 1,
                7 => bad.stamp.world[2] += 1,
                8 => bad.stamp.entity += 1,
                9 => bad.stamp.life += 1,
                10 => bad.stamp.revision = newer.world.revision + 1,
                11 => bad.sequence -= 1,
                12 => bad.sequence += 1,
                13 => bad.equipment_revision += 1,
                14 => bad.x += 1,
                15 => bad.y += 1,
                16 => bad.buff_revision += 1,
                _ => unreachable!(),
            }
            assert!(
                !bad.current(&o),
                "changed proof field {case} cannot survive"
            );
            // v42's receipt contract allows a skipped sequence; player current()
            // has always required exactly next. Keep that protocol distinction.
            if case != 12 {
                assert!(bad.validate_before(&newer).is_err(), "changed field {case}");
            }
        }
        assert_eq!($contract::encode_intent(&q, 47).unwrap(), original);
        assert_eq!(before.world.revision + 1, newer.world.revision);
        eprintln!(
            "UNCHANGED_PAINTED_PROOF type={} revision={} -> {} rejected_fields=17 immutable=true",
            stringify!($contract),
            q.stamp.revision,
            newer.world.revision
        );
    }};
}
fn changed_targets(
    t: crate::modern_lito::battle_target_haste::Target,
) -> Vec<crate::modern_lito::battle_target_haste::Target> {
    use crate::modern_lito::battle_target_haste::Target;
    let count = if matches!(t, Target::Player { .. }) {
        10
    } else {
        8
    };
    (0..count)
        .map(|case| {
            let mut bad = t;
            match &mut bad {
                Target::Player {
                    entity,
                    kind,
                    source,
                    id,
                    life,
                    equipment_revision,
                    x,
                    y,
                    buff_revision,
                } => match case {
                    0 => *entity += 1,
                    1 => *kind ^= 1,
                    2 => *source += 1,
                    3 => *id += 1,
                    4 => *life += 1,
                    5 => *equipment_revision += 1,
                    6 => *x += 1,
                    7 => *y += 1,
                    8 => *buff_revision += 1,
                    9 => {
                        return Target::Npc {
                            world: [1, 2, 3],
                            serial: 1,
                            life: 1,
                            changed_revision: 1,
                            x: 0,
                            y: 0,
                        }
                    }
                    _ => unreachable!(),
                },
                Target::Npc {
                    world,
                    serial,
                    life,
                    changed_revision,
                    x,
                    y,
                } => match case {
                    0..=2 => world[case] += 1,
                    3 => *serial += 1,
                    4 => *life += 1,
                    5 => *changed_revision += 1,
                    6 => *x += 1,
                    7 => *y += 1,
                    _ => unreachable!(),
                },
            }
            bad
        })
        .collect()
}
macro_rules! unchanged_targeted_proof {
    ($contract:ident, $q:expr, $before:expr) => {{
        unchanged_painted_proof!($contract, $q, $before);
        let mut newer = $before.clone();
        newer.world.revision += 1;
        newer.speed.as_mut().unwrap().state.revision = newer.world.revision;
        newer.personal.history.as_mut().unwrap().0.world_revision = newer.world.revision;
        let o = super::super::battle_scene::tests::observation(newer.clone());
        for target in changed_targets($q.target) {
            let bad = $contract::Intent { target, ..$q };
            assert!(!bad.current(&o), "changed target cannot survive");
            assert!(bad.validate_before(&newer).is_err());
        }
    }};
}

#[test]
fn old_world_has_no_slow_pixels_and_clocks_never_reverse() {
    let o = super::super::battle_scene::tests::observation(
        super::super::battle_payload::data_tests::fixture(),
    );
    assert!(!enabled(&o));
    let now = Instant::now();
    let t = Timeline::new(o.clone(), now);
    let assets = Assets { effect: None };
    let cancel = AtomicBool::new(false);
    let next = t.advance(o.clone(), now, &assets, &cancel).unwrap();
    assert!(next.effects.is_empty() && next.casts.is_empty());
    assert!(t
        .advance(o.clone(), now - Duration::from_nanos(1), &assets, &cancel)
        .is_err());
    assert!(t.advance(o, now, &assets, &AtomicBool::new(true)).is_err());
    assert_eq!(age(1, 500_000_001), Some(Duration::from_millis(500)));
    assert_eq!(age(1, 500_000_002), None);
    assert_eq!(age(i64::MIN, i64::MAX), None);
}
fn verify(before: d::UnvalidatedData, used: d::UnvalidatedData, expect_effect: bool) {
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
    assert_eq!(
        (applied.effects.len(), applied.casts.len()),
        (usize::from(expect_effect), 1)
    );
    let caster = applied.casts.values().next().unwrap();
    let e = Effect {
        target: status(&caster.receipt).unwrap().clone(),
        position: target(&caster.receipt).unwrap(),
        started: now,
        duration: Duration::from_millis(1875),
    };
    let r = applied.casts.values().next().unwrap();
    assert_eq!(e.position, target(&r.receipt).unwrap());
    assert!(target_live(used.data(), &e.target));
    use crate::modern_lito::battle_window::{can_slow, target_current, Target};
    for a in &used.data().actors.0 {
        let target = Target::Player {
            entity: a.entity,
            life: super::super::battle_motion::life(a).unwrap(),
        };
        assert!(can_slow(&used, target));
        assert!(!can_slow(
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
                can_slow(
                    &used,
                    Target::Npc {
                        serial: n.serial,
                        life: n.life
                    }
                ),
                n.hp > 0
            );
            assert!(!can_slow(
                &used,
                Target::Npc {
                    serial: n.serial,
                    life: n.life + 1
                }
            ));
        }
    }
    assert!(!can_slow(&used, Target::Honor { entity: 3 }));
    assert!(!can_slow(&used, Target::TimeTower));
    assert!(!can_slow(&used, Target::Siege { slot: 0 }));

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
    assert_eq!(again.effects.len(), usize::from(expect_effect));
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
    assert_eq!(
        canvas.rgba() != blank,
        expect_effect,
        "only an applied Slow casts shared752 pixels; a paid miss does not"
    );
    let expired = applied
        .advance(used, now + Duration::from_secs(2), &assets, &cancel)
        .unwrap();
    assert!(
        expired.effects.is_empty(),
        "slow status is not a permanent decal"
    );
}

#[test]
#[ignore = "Java actual speed command exports and real shared SPR: Slow hit/miss, Haste schema6 and GreaterHaste neutralization; no HWND"]
fn real_speed_commands_and_slow752_visuals_preserve_existing_haste() {
    use crate::modern_lito::{
        battle_greater_haste as greater, battle_payload as payload, battle_slow as slow,
        battle_target_haste as haste,
    };
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_MODERN_SPEED_COMMAND_FIXTURES")
            .expect("explicit Java command fixtures"),
    );
    assert!(root.is_absolute());
    let read = |name: &str| std::fs::read(root.join(name)).unwrap();
    for (name, effect) in [("slow", true), ("miss", false)] {
        let before = payload::decode_versioned(&read(&format!("{name}.before")), 47).unwrap();
        let q = slow::decode_intent(&read(&format!("{name}.request")), 47).unwrap();
        let r = slow::decode_reply(&read(&format!("{name}.reply")), 47, &q).unwrap();
        r.validate_after(&q, &before).unwrap();
        unchanged_targeted_proof!(slow, q, before);
        assert_eq!(r.reason, slow::Reason::None);
        assert_eq!(
            slow::Intent::from_observation(
                &super::super::battle_scene::tests::observation(before.clone()),
                q.target.selection()
            )
            .unwrap(),
            q
        );
        verify(before, r.data.clone(), effect);
    }
    let before = payload::decode_versioned(&read("haste.before"), 47).unwrap();
    let q = haste::decode_intent(&read("haste.request"), 47).unwrap();
    let r = haste::decode_reply(&read("haste.reply"), 47, &q).unwrap();
    r.validate_after(&q, &before).unwrap();
    unchanged_targeted_proof!(haste, q, before);
    assert_eq!(r.reason, haste::Reason::None);
    let expected = haste::Intent::from_observation(
        &super::super::battle_scene::tests::observation(before.clone()),
        q.target.selection(),
    )
    .unwrap();
    assert_eq!(
        haste::Intent {
            request_id: q.request_id,
            ..expected
        },
        q
    );
    super::super::battle_target_haste_visual::tests::verify(before, r.data.clone());
    let before = payload::decode_versioned(&read("neutralized.before"), 47).unwrap();
    let q = greater::decode_intent(&read("neutralized.request"), 47).unwrap();
    let r = greater::decode_reply(&read("neutralized.reply"), 47, &q).unwrap();
    r.validate_after(&q, &before).unwrap();
    unchanged_painted_proof!(greater, q, before);
    assert_eq!(r.reason, greater::Reason::None);
    assert!(greater::decode_reply(&read("neutralized.reply"), 45, &q).is_err());
    super::super::battle_haste::tests::verify_neutralization(before, r.data.clone());
    let b = r
        .data
        .actors
        .0
        .iter()
        .find(|a| a.entity == q.stamp.entity)
        .unwrap()
        .buffs
        .as_ref()
        .unwrap();
    assert_eq!(
        (b.haste_origin, b.haste_until_nanos, b.slow_until_nanos),
        (d::HasteOrigin::None, 0, 0)
    );
}
