use super::{super::*, Peer};
use crate::modern_lito::battle_payload::{data::*, vip as codec};

#[test]
#[ignore = "explicit compiled Java engine peer; schema10 candidate scene/following, NOT live TLS/UI"]
fn java_complete_vip_schema10_preserves_versions_owners_wide_stats_and_all_sections() {
    let mut peer = Peer::start_named(Some("vip10"));
    let mut frames = std::collections::BTreeMap::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:64");
            break;
        }
        let fields: Vec<_> = line.splitn(3, ':').collect();
        assert_eq!(fields[0], "FRAME");
        let label = fields[1];
        let raw: Vec<u8> = fields[2]
            .as_bytes()
            .chunks_exact(2)
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect();
        let c = codec::decode(&raw).unwrap_or_else(|e| panic!("{label}: {e:?}"));
        c.data
            .check_vip_context(&c.owner.player, &c.owner.world)
            .unwrap_or_else(|e| panic!("context {label}: {e:?}"));
        c.data
            .check_combat_with_healing(c.healing.as_ref().map(|h| h.0.as_slice()))
            .unwrap_or_else(|e| panic!("combat {label}: {e:?}"));
        personal::vip_facts(&c, &c.owner.player, &c.owner.world)
            .unwrap_or_else(|e| panic!("personal {label}: {e:?}"));
        c.data
            .check_scene_body()
            .unwrap_or_else(|e| panic!("scene body {label}: {e:?}"));
        c.check_values(&c.owner.player, &c.owner.world)
            .unwrap_or_else(|e| panic!("full scene {label}: {e:?}"));
        assert_eq!(c.owner.player.object, 3333);
        assert_eq!(c.data.room.members.0[0].identity.id, 3333);
        assert_eq!(c.data.room.members.0[1].identity.id, 3333);
        assert_eq!(
            c.owner.player.source,
            if label.ends_with("-7") { 7 } else { 42 }
        );
        let bytes = codec::encode(&c).unwrap_or_else(|e| panic!("encode {label}: {e:?}"));
        assert_eq!(bytes, raw, "{label}");
        assert!(data::encode(&c.data).is_err());
        assert!(decode(&raw).is_err());
        assert!(armor::decode(&raw).is_err());
        assert!(c
            .data
            .check_context(&c.data.personal.player, &c.data.world.identity)
            .is_err());
        // Every byte-prefix truncation, including owner-tail loss, is rejected.
        for n in 0..raw.len() {
            assert!(codec::decode(&raw[..n]).is_err(), "{label}: {n}");
        }
        let p = &c.data.personal.player;
        let w = &c.data.world.identity;
        let mut wrong = p.clone();
        wrong.epoch += 1;
        assert!(c.check_binding(&wrong, w).is_err());
        wrong = p.clone();
        wrong.source = if p.source == 7 { 42 } else { 7 };
        assert!(c.check_binding(&wrong, w).is_err());
        let mutations: Vec<Box<dyn Fn(&mut codec::Candidate)>> = vec![
            Box::new(|x| x.owner.versions.saved = x.owner.versions.runtime + 1),
            Box::new(|x| x.owner.versions.runtime += 1),
            Box::new(|x| x.owner.versions.lifecycle = -1),
            Box::new(|x| x.owner.entity = 3),
            Box::new(|x| x.owner.life += 1),
            Box::new(|x| x.owner.world_revision += 1),
            Box::new(|x| x.owner.abilities.0[0] = x.owner.abilities.0[0].wrapping_add(1)),
            Box::new(|x| x.data.world.descriptor.features &= !codec::FEATURE),
            Box::new(|x| x.data.world.descriptor.features |= 1 << 27),
            Box::new(|x| {
                x.data
                    .personal
                    .history
                    .as_mut()
                    .unwrap()
                    .0
                    .identity
                    .source
                    .epoch += 1
            }),
            Box::new(|x| x.owner.icons.0.push(12345)),
            Box::new(|x| {
                x.armor.0.pop();
            }),
        ];
        for change in mutations {
            let mut bad = c.clone();
            change(&mut bad);
            assert!(bad.check_binding(p, w).is_err(), "mutation {label}");
            assert!(codec::encode(&bad).is_err());
        }
        if !c.owner.items.0.is_empty() {
            let mut bad = c.clone();
            bad.owner.items.0[0].lifetime.state = LifetimeState::Expired;
            assert!(codec::encode(&bad).is_err());
            bad = c.clone();
            bad.owner.items.0.push(bad.owner.items.0[0].clone());
            assert!(codec::encode(&bad).is_err());
            bad = c.clone();
            bad.owner.items.0[0].applied.catalog += 1;
            assert!(codec::encode(&bad).is_err());
        }
        if label.starts_with("wide") {
            let e = c.data.personal.equipment.as_ref().unwrap();
            assert_eq!(e.effective.maximum_hp, 5_000_000_100);
            assert_eq!(e.effective.maximum_mp, 6_000_000_050);
            assert_eq!(c.owner.abilities.0[29], 7_000_000_000);
            assert_eq!(c.owner.abilities.0[30], -5_000_000_000);
        }
        if label == "durable-7" || label == "durable-42" {
            assert_eq!(c.owner.versions.saved, if p.source == 7 { 1 } else { 0 });
            assert_eq!(c.owner.versions.runtime, if p.source == 7 { 2 } else { 1 });
        }
        if label.contains("expired") || label.contains("repaid") {
            assert_eq!(c.owner.versions.saved, 0);
            assert_eq!(c.owner.versions.runtime, 2);
            assert!(c.owner.items.0.is_empty());
        }
        if label.starts_with("elimination-finished") {
            assert_eq!(c.data.world.phase, Phase::Finished);
            assert!(c.data.result.is_some());
        }
        assert_eq!(format!("{c:?}"), "UnvalidatedVipBattle[redacted]");
        peer.ack(&bytes);
        assert!(frames.insert(label.to_owned(), c).is_none());
    }
    assert_eq!(frames.len(), 64);
    peer.finish();
    for prefix in ["armor", "healing"] {
        let paid = &frames[&format!("{prefix}-paid-7")];
        let expired = &frames[&format!("{prefix}-expired-7")];
        for (a, b) in paid
            .healing
            .as_ref()
            .unwrap()
            .0
            .iter()
            .zip(&expired.healing.as_ref().unwrap().0)
        {
            assert_eq!(a.incoming, b.incoming);
            let old = a.outgoing.as_ref().unwrap();
            let now = b.outgoing.as_ref().unwrap();
            assert_eq!(old.receipt, now.receipt); // Paid facts unchanged; current cost/version reprice.
            assert_eq!(old.ledger.equipment_revision, 0);
            assert_eq!(now.ledger.equipment_revision, 2);
        }
        assert_eq!(paid.data.personal.ammo, expired.data.personal.ammo);
        assert_eq!(
            paid.data.personal.materials,
            expired.data.personal.materials
        );
        let source42 = &frames[&format!("{prefix}-paid-42")];
        assert_eq!(paid.data.actors, source42.data.actors);
        assert_eq!(paid.status, source42.status);
        assert_ne!(paid.data.personal.player, source42.data.personal.player);
    }
    assert!(matches!(
        &frames["haste43-cast-7"].status,
        healing::Status::Haste(_)
    ));
    assert!(matches!(
        &frames["slow-cast-7"].status,
        healing::Status::Speed(_)
    ));
    assert_eq!(
        frames["slow-cast-7"]
            .data
            .personal
            .materials
            .as_ref()
            .unwrap()
            .stocks
            .0[0]
            .remaining,
        3
    );
    let mut ordered_pairs = 0;
    let mut emitted_events = 0;
    for source in [7, 42] {
        for stages in [
            vec!["basic-loading", "basic-active", "basic-aborted"],
            vec![
                "healing-loading",
                "healing-active",
                "healing-bow",
                "healing-paid",
                "healing-expired",
                "healing-repaid",
            ],
            vec![
                "armor-loading",
                "armor-active",
                "armor-bow",
                "armor-paid",
                "armor-expired",
                "armor-repaid",
            ],
            vec!["haste43-loading", "haste43-cast", "haste43-expired"],
            vec!["haste54-loading", "haste54-cast", "haste54-expired"],
            vec!["slow-cast", "slow-expired", "slow-neutralized"],
            vec![
                "durable-loading",
                "durable-active",
                "durable-expiry",
                "durable",
            ],
            vec![
                "elimination-loading",
                "elimination-active",
                "elimination-finished",
            ],
        ] {
            let values: Vec<_> = stages
                .iter()
                .map(|s| &frames[&format!("{s}-{source}")])
                .collect();
            for (i, a) in values.iter().enumerate() {
                let mut follower =
                    codec::Follower::new(a.owner.player.clone(), a.owner.world.clone()).unwrap();
                assert!(follower.accept((*a).clone()).unwrap().is_empty());
                for b in &values[i..] {
                    a.check_successor(b).unwrap_or_else(|e| {
                        panic!(
                            "direct catch-up {} -> {} source {source}: {e:?}",
                            a.owner.world_revision, b.owner.world_revision
                        )
                    });
                    ordered_pairs += 1;
                    emitted_events += follower
                        .accept((*b).clone())
                        .unwrap_or_else(|e| {
                            panic!(
                                "follow {} -> {} source {source}: {e:?}",
                                a.owner.world_revision, b.owner.world_revision
                            )
                        })
                        .len();
                    assert!(follower.accept((*b).clone()).unwrap().is_empty());
                    let retained = follower.current().unwrap().candidate().clone();
                    let mut bad = (*b).clone();
                    bad.owner.versions.lifecycle += 1;
                    assert!(bad
                        .check_values(&bad.owner.player, &bad.owner.world)
                        .is_ok());
                    assert!(follower.accept(bad).is_err());
                    assert!(follower.current().unwrap().candidate() == &retained);
                    if b.owner.world_revision > a.owner.world_revision {
                        assert!(follower.accept((*a).clone()).is_err());
                        assert!(follower.current().unwrap().candidate() == &retained);
                    }
                }
            }
        }
    }
    for label in [
        "basic-loading-7",
        "healing-loading-7",
        "armor-loading-7",
        "haste43-loading-7",
    ] {
        for epoch in [i64::MIN, -1, i64::MAX] {
            let mut c = frames[label].clone();
            c.owner.player.epoch = epoch;
            c.data.personal.player.epoch = epoch;
            if let Some(m) = &mut c.data.personal.materials {
                m.player.epoch = epoch;
            }
            c.data
                .personal
                .history
                .as_mut()
                .unwrap()
                .0
                .identity
                .source
                .epoch = epoch as u64;
            c.check_values(&c.owner.player, &c.owner.world)
                .unwrap_or_else(|e| panic!("high-bit epoch {label}: {e:?}"));
            let raw = codec::encode(&c).unwrap();
            assert!(codec::decode(&raw).unwrap() == c);
            let mut follower =
                codec::Follower::new(c.owner.player.clone(), c.owner.world.clone()).unwrap();
            assert!(follower.accept(c.clone()).unwrap().is_empty());
            assert!(follower.accept(c).unwrap().is_empty());
        }
    }
    // These still pass the local envelope binding. Only a complete scene proof
    // detects the wrong clocks, manifest, food or resource arithmetic.
    type Mutation = Box<dyn Fn(&mut codec::Candidate)>;
    let mutations: Vec<Mutation> = vec![
        Box::new(|c| c.data.world.descriptor.load_timeout_millis = 49),
        Box::new(|c| c.data.world.descriptor.countdown_nanos = 30_000_000_001),
        Box::new(|c| c.data.world.descriptor.active_timeout_millis = 1_800_001),
        Box::new(|c| c.data.actors.0[0].position.x = -1),
        Box::new(|c| c.data.actors.0[0].heading = 8),
        Box::new(|c| c.data.personal.equipment.as_mut().unwrap().saved.food225 = 226),
        Box::new(|c| c.data.actors.0[1].resources.as_mut().unwrap().vitals.hp += 1),
        Box::new(|c| c.data.world.active_elapsed_nanos = -1),
    ];
    let original = &frames["basic-active-7"];
    let mut follower =
        codec::Follower::new(original.owner.player.clone(), original.owner.world.clone()).unwrap();
    follower.accept(original.clone()).unwrap();
    for (index, mutation) in mutations.iter().enumerate() {
        let mut bad = original.clone();
        mutation(&mut bad);
        assert!(
            bad.check_binding(&bad.owner.player, &bad.owner.world)
                .is_ok(),
            "binding {index}"
        );
        assert!(
            bad.check_values(&bad.owner.player, &bad.owner.world)
                .is_err(),
            "scene {index}"
        );
        assert!(follower.accept(bad).is_err());
        assert!(follower.current().unwrap().candidate() == original);
    }
    let prior = &frames["basic-loading-7"];
    assert!(original
        .data
        .personal_facts(&original.owner.player, &original.owner.world)
        .is_err());
    let mut ordinary_etc = original.clone();
    ordinary_etc.owner.items.0.clear();
    ordinary_etc.owner.abilities.0.fill(0);
    ordinary_etc.owner.icons.0.clear();
    ordinary_etc
        .check_binding(&ordinary_etc.owner.player, &ordinary_etc.owner.world)
        .unwrap();
    assert!(ordinary_etc
        .check_values(&ordinary_etc.owner.player, &ordinary_etc.owner.world)
        .is_err());
    assert!(follower.accept(ordinary_etc).is_err());
    assert!(follower.current().unwrap().candidate() == original);
    // Changing only the current session must NOT rewrite a previously paid
    // healing receipt's scope. High-bit acceptance is not a history rebind.
    let mut wrong_receipt_scope = frames["healing-paid-7"].clone();
    wrong_receipt_scope.owner.player.epoch = -1;
    wrong_receipt_scope.data.personal.player.epoch = -1;
    wrong_receipt_scope
        .data
        .personal
        .materials
        .as_mut()
        .unwrap()
        .player
        .epoch = -1;
    wrong_receipt_scope
        .data
        .personal
        .history
        .as_mut()
        .unwrap()
        .0
        .identity
        .source
        .epoch = u64::MAX;
    assert!(wrong_receipt_scope
        .check_values(
            &wrong_receipt_scope.owner.player,
            &wrong_receipt_scope.owner.world
        )
        .is_err());
    let mut rewritten = original.clone();
    rewritten.owner.items.0[0].applied.abilities.0[29] += 1;
    rewritten.owner.abilities.0[29] += 1;
    rewritten
        .check_values(&rewritten.owner.player, &rewritten.owner.world)
        .unwrap();
    assert!(prior.check_successor(&rewritten).is_err());
    let mut regressed_clock = frames["durable-expiry-7"].clone();
    regressed_clock
        .data
        .personal
        .equipment
        .as_mut()
        .unwrap()
        .evaluated_at_epoch_millis = 5000;
    regressed_clock
        .check_values(&regressed_clock.owner.player, &regressed_clock.owner.world)
        .unwrap();
    assert!(regressed_clock
        .check_successor(&frames["durable-7"])
        .is_err());
    assert_eq!(ordered_pairs, 164);
    assert!(emitted_events > 0);
    println!("VIP_SCENE: 64 real engine frames; {ordered_pairs} ordered pairs; {emitted_events} emitted armor events; 12 high-bit epochs; 8 malformed scenes; atomic rejection, ETC isolation and pinned templates");
}
