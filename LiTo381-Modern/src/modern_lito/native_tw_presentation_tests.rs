// Included by each TW presentation module so the same contract covers all loaders.
fn world() -> World {
    let mut world = World::default();
    world.accept(&[0x28, 0x33, 0x02, 0]).unwrap();
    world.accept(&[0xd2]).unwrap();
    let mut status = vec![0; 37];
    status[0] = 0x0c;
    status[1..5].copy_from_slice(&100u32.to_le_bytes());
    for offset in [16, 18, 20, 22] {
        status[offset..offset + 2].copy_from_slice(&200u16.to_le_bytes());
    }
    world.accept(&status).unwrap();
    for id in [100u32, 200] {
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
        world.accept(&b).unwrap();
    }
    world.learned_skills.insert(SKILL);
    world
}

fn projectile(world: &mut World) {
    let wire = "231264000000c8000000010207000000e22d0000800080028001800000";
    let mut bytes: Vec<u8> = (0..wire.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&wire[i..i + 2], 16).unwrap())
        .collect();
    bytes[16..18].copy_from_slice(&GRAPHICS.to_le_bytes());
    world.accept(&bytes).unwrap();
}

fn fail(_: &Path, _: &AtomicBool) -> Result<Assets> {
    Err(
        anyhow::anyhow!("unqualified current TW {FEATURE} namespace")
            .context("TW presentation load"),
    )
}

#[test]
fn unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation() {
    for learned_trigger in [true, false] {
        let mut w = world();
        if !learned_trigger {
            w.learned_skills.clear();
            projectile(&mut w);
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let owner = Arc::new(());
        let mut effects = Effects::default();
        let cancel = AtomicBool::new(false);
        let mut calls = 0;
        for now in 0..3 {
            let before = serde_json::to_value(&w).unwrap();
            effects
                .prepare_with_loader(root, owner.clone(), &w, &cancel, |r, c| {
                    calls += 1;
                    fail(r, c)
                })
                .unwrap();
            let launches: Vec<_> = w
                .projectiles
                .iter()
                .map(|cue| Launch {
                    owner: owner.clone(),
                    publication: effects.publication.clone(),
                    generation: w.generation,
                    key: key(cue),
                })
                .collect();
            assert!(effects
                .update(&owner, &w, launches.iter(), now)
                .unwrap()
                .is_empty());
            assert!(effects.draws(&owner).unwrap().is_empty());
            assert!(effects.events().is_empty());
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
            let snapshot = effects.snapshot();
            assert_eq!(snapshot["presentation"]["state"], "unavailable");
            assert_eq!(
                snapshot["presentation"]["reason"],
                format!("TW presentation load: unqualified current TW {FEATURE} namespace")
            );
            assert_eq!(snapshot["assets_ready"], false);
            assert_eq!(snapshot["active"], 0);
            assert_eq!(calls, 1);
        }
        // The presentation failure neither removes permission nor blocks server state.
        w.learned_skills.insert(SKILL);
        let request = crate::modern_lito::native_skills::request(&w, SKILL, Some(200)).unwrap();
        assert_eq!(
            &request[..3],
            &[0x14, ((SKILL - 1) / 8) as u8, ((SKILL - 1) % 8) as u8]
        );
        w.accept(&[0x0d, 99, 0, 200, 0]).unwrap();
        w.accept(&[0x4d, 80, 0, 200, 0]).unwrap();
        assert_eq!(w.hp, Some([99, 200]));
        assert_eq!(w.mp, Some([80, 200]));
        assert!(effects
            .update(&owner, &w, std::iter::empty(), 4)
            .unwrap()
            .is_empty());
    }
}

#[test]
fn unavailable_history_survives_owner_and_scene_replacement_but_new_generation_retries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut w = world();
    let cancel = AtomicBool::new(false);
    let mut effects = Effects::default();
    effects
        .prepare_with_loader(root, Arc::new(()), &w, &cancel, fail)
        .unwrap();
    effects
        .prepare_with_loader(root, Arc::new(()), &w, &cancel, |_, _| {
            panic!("owner retry")
        })
        .unwrap();
    let mut history = History {
        generation: Some(w.generation),
        ..Default::default()
    };
    history.merge(&effects.history());
    effects.retire();
    assert_eq!(effects.snapshot()["presentation"]["state"], "pending");
    let mut replacement = Effects::default();
    replacement.restore_history(history);
    let owner = Arc::new(());
    replacement
        .prepare_with_loader(root, owner.clone(), &w, &cancel, |_, _| {
            panic!("scene retry")
        })
        .unwrap();
    assert_eq!(
        replacement.snapshot()["presentation"]["state"],
        "unavailable"
    );
    w.generation += 1;
    let mut calls = 0;
    replacement
        .prepare_with_loader(root, owner.clone(), &w, &cancel, |r, c| {
            calls += 1;
            fail(r, c)
        })
        .unwrap();
    assert_eq!(calls, 1);
    // Also cover callers observing a new generation through update before prepare.
    w.generation += 1;
    replacement
        .update(&owner, &w, std::iter::empty(), 0)
        .unwrap();
    assert_eq!(replacement.snapshot()["presentation"]["state"], "pending");
    replacement
        .prepare_with_loader(root, owner, &w, &cancel, |r, c| {
            calls += 1;
            fail(r, c)
        })
        .unwrap();
    assert_eq!(calls, 2);
}

#[test]
fn cancellation_during_loader_remains_an_error_and_retires_owner() {
    let mut effects = Effects::default();
    let owner = Arc::new(());
    let cancel = AtomicBool::new(false);
    let w = world();
    let error = effects
        .prepare_with_loader(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            owner.clone(),
            &w,
            &cancel,
            |r, c| {
                c.store(true, Ordering::Release);
                fail(r, c)
            },
        )
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"));
    assert!(effects.update(&owner, &w, std::iter::empty(), 0).is_err());
    assert_eq!(effects.snapshot()["presentation"]["state"], "pending");
}

#[test]
fn no_demand_stays_pending_and_unavailable_does_not_relax_source_guard() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let owner = Arc::new(());
    let cancel = AtomicBool::new(false);
    let mut w = world();
    w.learned_skills.clear();
    let mut effects = Effects::default();
    effects
        .prepare_with_loader(root, owner.clone(), &w, &cancel, |_, _| panic!("no demand"))
        .unwrap();
    assert_eq!(effects.snapshot()["presentation"]["state"], "pending");
    w.learned_skills.insert(SKILL);
    effects
        .prepare_with_loader(root, owner.clone(), &w, &cancel, fail)
        .unwrap();
    assert!(effects
        .update(&Arc::new(()), &w, std::iter::empty(), 0)
        .is_err());
    assert!(effects
        .prepare_with_loader(&root.join("src"), owner, &w, &cancel, |_, _| panic!(
            "reused owner"
        ))
        .unwrap_err()
        .to_string()
        .contains("another root"));
    // An explicitly different resource source must not inherit another root's failure.
    let owner = Arc::new(());
    effects
        .prepare_with_loader(root, owner.clone(), &w, &cancel, fail)
        .unwrap();
    let history = effects.history();
    let mut replacement = Effects::default();
    replacement.restore_history(history);
    let mut calls = 0;
    replacement
        .prepare_with_loader(&root.join("src"), Arc::new(()), &w, &cancel, |r, c| {
            calls += 1;
            fail(r, c)
        })
        .unwrap();
    assert_eq!(calls, 1);
}
