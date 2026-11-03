//! Live Java engine publications, exact schema6 codecs, complete candidate
//! scene/sequence checks. Still no negotiated commands, live Observation or UI.
use super::{
    super::{self as payload, data as d, speed},
    Peer,
};
#[path = "battle_speed_admission_tests.rs"]
mod admission;

#[test]
#[ignore = "explicit Java test-only RNG, shared Map514 SPR/status art; deterministic first-cast Slow miss, no SQL/TLS/HWND"]
fn java_first_slow_miss_keeps_target_status_and_pixels_clear() {
    use crate::modern_lito::battle_scene::tests::{
        observation, trace_observed_scene_with_effects,
    };
    use std::{collections::BTreeMap, path::PathBuf, sync::atomic::AtomicBool, time::{Duration, Instant}};
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
    let effects = PathBuf::from(std::env::var_os("LITO_EFFECT_CATALOG_PATH").expect("effect XML"));
    let mut peer = Peer::start_named(Some("speed-first-miss"));
    let mut frames = BTreeMap::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:6");
            break;
        }
        let parts = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(parts[0], "SPEED");
        let bytes = parts[2].as_bytes().chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let candidate = speed::decode(&bytes).unwrap();
        candidate.check_components().unwrap();
        candidate.check_values(&candidate.data.personal.player, &candidate.data.world.identity).unwrap();
        let data = payload::decode_versioned(&bytes, 46).unwrap();
        assert_eq!(speed::encode(&candidate).unwrap(), bytes);
        assert!(frames.insert(parts[1].to_owned(), data).is_none());
        peer.ack(&bytes);
    }
    peer.finish();
    assert_eq!(frames.len(), 6);
    let before_source = &frames["first-before-1"];
    let after_source = &frames["first-miss-1"];
    let before_target = &frames["first-before-2"];
    let after_target = &frames["first-miss-2"];
    let next_source = &frames["next-self-1"];
    let next_target = &frames["next-self-2"];
    assert_eq!(before_source.personal.player.source, 7);
    assert_eq!(before_target.personal.player.source, 42);
    assert_eq!(after_source.personal.materials.as_ref().unwrap().uses, 1);
    assert_eq!(before_source.personal.materials.as_ref().unwrap().uses, 0);
    assert_eq!(after_source.actors.0[0].vital.as_ref().unwrap().values.mp, 80);
    assert_eq!(before_source.actors.0[0].vital.as_ref().unwrap().values.mp, 100);
    let receipt = after_source.actors.0[0].spells.histories.0.iter()
        .find(|h| h.lane == d::Lane::Slow).unwrap().receipt.as_ref().unwrap();
    let d::Detail::Slow(detail) = &receipt.detail else { panic!("server Slow receipt") };
    assert!(detail.roll > detail.probability);
    assert_eq!(detail.roll, 100);
    assert_eq!(receipt.casts, 1, "the fixture did not retry a miss");
    assert_eq!(next_source.personal.materials.as_ref().unwrap().uses, 2);
    assert_eq!(next_source.actors.0[0].vital.as_ref().unwrap().values.mp, 60);
    let next_receipt = next_source.actors.0[0].spells.histories.0.iter()
        .find(|h| h.lane == d::Lane::Slow).unwrap().receipt.as_ref().unwrap();
    let d::Detail::Slow(next_detail) = &next_receipt.detail else { panic!("continued Slow receipt") };
    let crate::modern_lito::battle_payload::haste::StatusTarget::Player(next_player) = &next_detail.target else {
        panic!("continued command must target a player")
    };
    assert_eq!(next_player.participant.entity, next_source.personal.entity,
        "the second planned cast targets self, not a retry on source 42");
    assert_eq!(next_receipt.casts, 2);
    let owner_status = |data: &d::UnvalidatedData| {
        data.actors.0.iter().find(|a| a.entity == data.personal.entity)
            .and_then(|a| a.buffs.as_ref())
            .and_then(|b| crate::modern_lito::battle_scene::ordinary_status_effect(b, true))
    };
    assert_eq!(owner_status(before_target), None);
    assert_eq!(owner_status(after_target), None);
    assert_eq!(owner_status(next_target), None);
    let cancel = AtomicBool::new(false);
    let mut scene = trace_observed_scene_with_effects(
        &root, observation(before_target.clone()), Some(&effects));
    assert_eq!(scene.effect_catalog_path(), Some(effects.as_path()));
    let now = Instant::now();
    let before = scene.render(now, &cancel).unwrap();
    scene.observe_at(observation(after_target.clone()), now + Duration::from_secs(1), &cancel).unwrap();
    let after = scene.render(now + Duration::from_secs(1), &cancel).unwrap();
    scene.observe_at(observation(next_target.clone()), now + Duration::from_secs(3), &cancel).unwrap();
    let continued = scene.render(now + Duration::from_secs(3), &cancel).unwrap();
    let status_pixels = |bgra: &[u8]| -> Vec<u8> {
        (8usize..40).flat_map(|y| {
            let start = (y * 800 + 232) * 4;
            bgra[start..start + 32 * 4].to_vec()
        }).collect()
    };
    assert_eq!(status_pixels(&before.bgra), status_pixels(&after.bgra),
        "a valid first-cast miss cannot paint a target Slow icon");
    assert_eq!(status_pixels(&after.bgra), status_pixels(&continued.bgra),
        "a later self cast cannot invent source-42 status art");
    assert_eq!(after.observation.data(), after_target);
    assert_eq!(continued.observation.data(), next_target);
    eprintln!("R397_FIRST_MISS source=7 target_source=42 roll={} chance={} mp=100->80 materials=0->1 casts=1 status=none icon_unchanged=true next_self_cast=2 mp=60 materials=2 target_icon_unchanged=true",
        detail.roll, detail.probability);
}
#[test]
#[ignore = "explicit compiled Java schema6 candidate peer; not TLS/UI admission"]
fn java_speed29_schema6_lossless_components_and_old_versions_closed() {
    let mut peer = Peer::start_named(Some("speed29"));
    let mut frames = Vec::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:81");
            break;
        }
        let f = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(f.len(), 3);
        assert_eq!(f[0], "SPEED");
        let bytes = f[2]
            .as_bytes()
            .chunks_exact(2)
            .map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let p = speed::decode(&bytes).unwrap_or_else(|e| panic!("{} decode {e:?}", f[1]));
        p.check_components()
            .unwrap_or_else(|e| panic!("{} components {e:?}", f[1]));
        p.check_values(&p.data.personal.player, &p.data.world.identity)
            .unwrap_or_else(|e| panic!("{} full values {e:?}", f[1]));
        assert_eq!(speed::encode(&p).unwrap(), bytes);
        admission::values(&p);
        let admitted = payload::decode_versioned(&bytes, 47).unwrap();
        crate::modern_lito::battle_motion::tests::verify_speed_npc_pacing(&admitted);
        peer.ack(&bytes);
        for v in 1..speed::VERSION {
            assert!(
                payload::decode_versioned(&bytes, v).is_err(),
                "{} version {v}",
                f[1]
            );
        }
        assert!(payload::decode(&bytes).is_err());
        assert!(payload::haste::decode(&bytes).is_err());
        assert!(payload::encode(&p.data).is_err());
        assert!(p
            .data
            .check_scene_values(&p.data.personal.player, &p.data.world.identity)
            .is_err());
        let history = &p.data.personal.history.as_ref().unwrap().0;
        let h = crate::modern_lito::equipment_history_wire::encode_with_schema(history, 4).unwrap();
        assert_eq!(
            crate::modern_lito::equipment_history_wire::decode_with_schema(&h, 4).unwrap(),
            *history
        );
        for schema in 1..=3 {
            assert!(
                crate::modern_lito::equipment_history_wire::encode_with_schema(history, schema)
                    .is_err()
            );
            assert!(
                crate::modern_lito::equipment_history_wire::decode_with_schema(&h, schema).is_err()
            );
        }
        for at in [0, 4, 8, 14, 16, bytes.len() - 1] {
            assert!(speed::decode(&bytes[..at]).is_err());
        }
        let mut b = bytes.clone();
        b.push(0);
        assert!(speed::decode(&b).is_err());
        let mut b = bytes.clone();
        b[4..6].copy_from_slice(&5u16.to_be_bytes());
        assert!(speed::decode(&b).is_err());
        frames.push((f[1].to_owned(), p));
    }
    peer.finish();
    assert_eq!(frames.len(), 81);
    let mut timelines = 0;
    for prefix in ["pc-", "npc-", "gear-", "budget-"] {
        let eligible = |name: &str| {
            name.starts_with(prefix)
                && !name.starts_with("pc-wide")
                && !name.starts_with("npc-prerequisite")
        };
        for (i, (name, a)) in frames.iter().enumerate().filter(|(_, (n, _))| eligible(n)) {
            for (next, b) in frames[i..]
                .iter()
                .filter(|(n, p)| eligible(n) && p.data.personal.player == a.data.personal.player)
            {
                a.check_successor(b)
                    .unwrap_or_else(|e| panic!("sequence {name} -> {next}: {e:?}"));
                admission::transition(a, b);
                timelines += 1;
            }
        }
    }
    assert!(timelines >= 200);
    let frame = |name: &str| &frames.iter().find(|(n, _)| n == name).unwrap().1;
    let gear = frame("gear-reequip-1");
    let ledger = &gear.data.actors.0[0]
        .spells
        .histories
        .0
        .iter()
        .find(|h| h.lane == d::Lane::Slow)
        .unwrap()
        .ledger;
    assert!(ledger.prior_casts > 0);
    assert_eq!(
        gear.data.personal.materials.as_ref().unwrap().uses,
        ledger.casts
    );
    assert_ne!(
        gear.data.personal.materials.as_ref().unwrap().uses,
        ledger.prior_casts + ledger.casts
    );
    let a = frame("pc-slow-1");
    let b = frame("pc-slow-2");
    assert_eq!(a.data.personal.player.object, b.data.personal.player.object);
    assert_ne!(a.data.personal.player.source, b.data.personal.player.source);
    assert_eq!(a.data.personal.materials.as_ref().unwrap().uses, 1);
    assert_eq!(b.data.personal.materials.as_ref().unwrap().uses, 0);
    assert_eq!(a.data.actors, b.data.actors);
    assert_eq!(a.effects, b.effects);
    assert_eq!(
        frame("pc-wide-1").data.actors.0[0]
            .resources
            .as_ref()
            .unwrap()
            .vitals
            .mp,
        4_999_999_980
    );
    assert!(matches!(
        frame("pc-greater-neutral-2").data.actors.0[1]
            .spells
            .histories
            .0
            .iter()
            .find(|s| s.lane == d::Lane::GreaterHaste)
            .unwrap()
            .receipt
            .as_ref()
            .unwrap()
            .detail,
        d::Detail::GreaterHasteNeutralized(_)
    ));
    let n = frame("npc-haste-1");
    let e = &n.effects.targets.0[0];
    assert_eq!(
        e.active.as_ref().unwrap().receipt.impact.lane,
        d::Lane::Haste
    );
    assert_eq!(e.walk.as_ref().unwrap().receipt.impact.lane, d::Lane::Slow);
    assert_eq!(
        n.effects.npcs.as_ref().unwrap().actors.0[0]
            .last_walk
            .as_ref()
            .unwrap()
            .duration_millis,
        200
    );
    assert_eq!(
        frame("pc-expired-1").data.actors.0[1]
            .buffs
            .as_ref()
            .unwrap()
            .slow_until_nanos,
        0
    );
    assert!(
        frame("pc-expired-1").data.actors.0[1]
            .buffs
            .as_ref()
            .unwrap()
            .slow_revision
            > 0
    );
    let q = frame("quest-lanes-2");
    assert_eq!(q.data.world.lane, 1);
    assert_eq!(q.data.actors.0.len(), 1);
    assert_eq!(q.data.actors.0[0].entity, 2);
    let full =
        |p: &speed::Candidate| p.check_values(&p.data.personal.player, &p.data.world.identity);
    let reject_full = |name: &str, base: &speed::Candidate, edit: fn(&mut speed::Candidate)| {
        let mut p = base.clone();
        edit(&mut p);
        assert!(full(&p).is_err(), "accepted full mutation {name}");
        if name == "materials omitted" {
            assert!(speed::encode(&p).is_err());
        } else {
            let encoded =
                speed::encode(&p).unwrap_or_else(|e| panic!("encode mutation {name}: {e:?}"));
            let parsed = speed::decode(&encoded).unwrap();
            assert!(full(&parsed).is_err(), "accepted roundtrip mutation {name}");
        }
        admission::reject(base, &p);
        full(base).unwrap();
    };
    reject_full("material weight mismatch", a, |p| {
        p.data.personal.materials.as_mut().unwrap().stocks.0[0].unit_weight += 1;
    });
    reject_full("wrong total weight", a, |p| {
        p.data.personal.weight.as_mut().unwrap().total_weight += 1;
    });
    reject_full("wrong material weight total", a, |p| {
        p.data.personal.weight.as_mut().unwrap().supplies_weight += 1;
    });
    reject_full("missing active proof", a, |p| {
        p.effects.targets.0.clear();
    });
    reject_full("expired deadline rewritten", a, |p| {
        p.data.actors.0[1].buffs.as_mut().unwrap().slow_until_nanos += 1;
    });
    reject_full("foreign owner", a, |p| {
        p.data.personal.player.source = 99;
    });
    reject_full("materials omitted", a, |p| {
        p.data.personal.materials = None;
    });
    reject_full("future world clock", a, |p| {
        p.data.world.active_elapsed_nanos += 1;
    });
    reject_full("history stock debits", gear, |p| {
        p.data.personal.materials.as_mut().unwrap().uses += 1;
    });
    reject_full("old walk duration sped up", n, |p| {
        p.effects.npcs.as_mut().unwrap().actors.0[0]
            .last_walk
            .as_mut()
            .unwrap()
            .duration_millis = 75;
    });
    reject_full("historical walk status overwritten", n, |p| {
        let m = &mut p.effects.npcs.as_mut().unwrap().actors.0[0];
        m.walk_speed = m.speed.clone();
    });
    reject_full("walk provenance removed", n, |p| {
        p.effects.targets.0[0].walk = None;
    });
    reject_full("active replaced by historical walk", n, |p| {
        p.effects.targets.0[0].active = p.effects.targets.0[0].walk.clone();
    });
    reject_full("NPC future lease", n, |p| {
        p.effects.npcs.as_mut().unwrap().actors.0[0]
            .speed
            .haste_until_millis += 1;
    });
    reject_full(
        "greater neutralized actor swapped",
        frame("pc-greater-neutral-1"),
        |p| {
            let r = p.data.actors.0[1]
                .spells
                .histories
                .0
                .iter_mut()
                .find(|h| h.lane == d::Lane::GreaterHaste)
                .unwrap()
                .receipt
                .as_mut()
                .unwrap();
            let d::Detail::GreaterHasteNeutralized(n) = &mut r.detail else {
                panic!()
            };
            n.before.entity = 1;
            n.after.entity = 1;
        },
    );
    reject_full(
        "miss substituted for successful grant",
        frame("pc-miss-keeps-1"),
        |p| {
            let r = p.data.actors.0[0]
                .spells
                .histories
                .0
                .iter()
                .find(|h| h.lane == d::Lane::Slow)
                .unwrap()
                .receipt
                .clone()
                .unwrap();
            p.effects.targets.0[0].active.as_mut().unwrap().receipt = r;
        },
    );
    reject_full(
        "three casts in one new request",
        frame("budget-miss-1"),
        |p| {
            let r = &mut p.effects.targets.0[0].active.as_mut().unwrap().receipt;
            assert_eq!(r.casts, 3);
            assert_eq!(r.impact.usage.sequence, 3);
            r.casts = 1;
            r.spent = i64::from(r.impact.usage.cost);
        },
    );
    reject_full(
        "wrong cost for one intervening cast",
        frame("budget-miss-1"),
        |p| {
            let r = &mut p.effects.targets.0[0].active.as_mut().unwrap().receipt;
            assert_eq!(r.impact.usage.cost, 17);
            r.spent += 1;
        },
    );
    // Both snapshots can be valid in isolation, but a match cannot gain stocks
    // or change its frozen material price after equipment/clock changes.
    let rewrite_weight = |p: &mut speed::Candidate| {
        use crate::modern_lito::weight_protocol as weight;
        let mut stocks = p
            .data
            .personal
            .supplies
            .as_ref()
            .unwrap()
            .stocks
            .0
            .iter()
            .map(|s| (s.item, s.remaining))
            .collect::<Vec<_>>();
        if let Some(a) = &p.data.personal.ammo {
            stocks.extend(a.stocks.0.iter().map(|s| (s.item, s.remaining)));
        }
        stocks.extend(
            p.data
                .personal
                .materials
                .as_ref()
                .unwrap()
                .stocks
                .0
                .iter()
                .map(|s| (s.item, s.remaining)),
        );
        let w = p.data.personal.weight.as_mut().unwrap();
        w.supplies_weight = stocks
            .iter()
            .map(|(id, n)| {
                weight::stack_weight(
                    *n,
                    w.units
                        .0
                        .iter()
                        .find(|u| u.item == *id)
                        .unwrap()
                        .unit_weight,
                )
                .unwrap()
            })
            .sum();
        w.total_weight = w.source_gear_weight + w.supplies_weight;
        w.stage = i32::from(
            weight::stage(w.total_weight, f64::from_bits(w.maximum_weight_bits as u64)).unwrap(),
        );
    };
    let old = frame("gear-active-1");
    let next = frame("gear-unequip-1");
    let mut extra = next.clone();
    let stock = &mut extra.data.personal.materials.as_mut().unwrap().stocks.0[0];
    stock.initial += 1;
    stock.remaining += 1;
    rewrite_weight(&mut extra);
    full(&extra).expect("isolated extra-stock candidate");
    assert!(old.check_successor(&extra).is_err());
    admission::reject(old, &extra);
    let mut repriced = next.clone();
    repriced.data.personal.materials.as_mut().unwrap().stocks.0[0].unit_weight += 1000;
    repriced
        .data
        .personal
        .weight
        .as_mut()
        .unwrap()
        .units
        .0
        .iter_mut()
        .find(|u| u.item == 40318)
        .unwrap()
        .unit_weight += 1000;
    rewrite_weight(&mut repriced);
    full(&repriced).expect("isolated repriced candidate");
    assert!(old.check_successor(&repriced).is_err());
    admission::reject(old, &repriced);
    let mut rewritten = n.clone();
    rewritten.effects.npcs.as_mut().unwrap().actors.0[0]
        .last_walk
        .as_mut()
        .unwrap()
        .elapsed_millis += 1;
    rewritten.data.npcs.as_mut().unwrap().actors.0[0].next_move_millis += 1;
    full(&rewritten).expect("individually valid rewritten historical walk");
    assert!(frame("npc-neutral-1").check_successor(&rewritten).is_err());
    assert!(a.check_successor(b).is_err(), "foreign owner");
    assert!(
        frame("pc-abort-1")
            .check_successor(frame("pc-active-1"))
            .is_err(),
        "terminal reversed"
    );
    assert!(
        frame("npc-haste-1")
            .check_successor(frame("npc-walk-1"))
            .is_err(),
        "history reversed"
    );
    for (_, p) in &frames {
        let mut player = p.data.personal.player.clone();
        player.epoch ^= 1;
        assert!(p.check_values(&player, &p.data.world.identity).is_err());
        let mut world = p.data.world.identity.clone();
        world.generation += 1;
        assert!(p.check_values(&p.data.personal.player, &world).is_err());
    }
    eprintln!("schema6 verified {} engine publications, {timelines} complete successors, 18 full-scene mutations, 6 sequence mutations and {} foreign contexts", frames.len(),frames.len()*2);
    let reject = |base: &speed::Candidate, edit: fn(&mut speed::Candidate)| {
        let mut p = base.clone();
        edit(&mut p);
        let parsed = speed::decode(&speed::encode(&p).unwrap()).unwrap();
        assert!(parsed.check_components().is_err());
        base.check_components().unwrap();
    };
    reject(a, |p| {
        p.data.personal.materials.as_mut().unwrap().player.source = 42
    });
    reject(a, |p| {
        p.data.personal.materials.as_mut().unwrap().player.epoch += 1
    });
    reject(a, |p| {
        p.data.personal.materials.as_mut().unwrap().world.generation += 1
    });
    reject(a, |p| p.data.personal.materials.as_mut().unwrap().uses += 1);
    reject(a, |p| {
        p.data.personal.materials.as_mut().unwrap().stocks.0[0].remaining += 1
    });
    reject(a, |p| {
        p.data.personal.materials.as_mut().unwrap().last_sequence += 1
    });
    reject(a, |p| {
        p.data.personal.materials.as_mut().unwrap().last_life += 1
    });
    reject(a, |p| {
        p.data.personal.materials.as_mut().unwrap().stocks.0[0].item = 40013
    });
    reject(a, |p| {
        p.data.personal.materials.as_mut().unwrap().stocks.0[0].unit_weight = -1
    });
    reject(a, |p| {
        p.data.actors.0[1].buffs.as_mut().unwrap().slow_revision = i64::MAX
    });
    reject(a, |p| {
        p.effects.targets.0[0]
            .active
            .as_mut()
            .unwrap()
            .caster
            .identity
            .source = 42
    });
    reject(a, |p| {
        p.effects.targets.0[0]
            .active
            .as_mut()
            .unwrap()
            .receipt
            .loading_elapsed_nanos += 1
    });
    reject(a, |p| {
        let r = p.data.actors.0[0]
            .spells
            .histories
            .0
            .iter_mut()
            .find(|s| s.lane == d::Lane::Slow)
            .unwrap()
            .receipt
            .as_mut()
            .unwrap();
        if let d::Detail::Slow(s) = &mut r.detail {
            s.probability = 91;
        } else {
            panic!()
        }
    });
    reject(a, |p| {
        let r = p.data.actors.0[0]
            .spells
            .histories
            .0
            .iter_mut()
            .find(|s| s.lane == d::Lane::Slow)
            .unwrap()
            .receipt
            .as_mut()
            .unwrap();
        if let d::Detail::Slow(s) = &mut r.detail {
            if let speed::StatusTarget::Player(t) = &mut s.target {
                t.after.slow_until_nanos += 1;
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    });
    reject(frame("npc-prerequisite-1"), |p| {
        let r = p.data.actors.0[0]
            .spells
            .histories
            .0
            .iter_mut()
            .find(|s| s.lane == d::Lane::Slow)
            .unwrap()
            .receipt
            .as_mut()
            .unwrap();
        if let d::Detail::Slow(s) = &mut r.detail {
            s.missing_effect.as_mut().unwrap().required_skill = 1014;
        } else {
            panic!()
        }
    });
}
#[test]
#[ignore = "explicit compiled Java v46 negotiated read peer; no TLS/Main/UI claim"]
fn java_speed46_negotiated_read_and_atomic_observation_following() {
    use crate::modern_lito::{
        application_protocol as app, battle_follow::Follower, battle_read as read, protocol,
    };
    use std::io::Write;
    let hex = |s: &str| {
        assert_eq!(s.len() % 2, 0);
        s.as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>()
    };
    let mut peer = Peer::start_named(Some("speed46-read"));
    let hello = protocol::Hello {
        request_id: 0xfedcba98,
        min_version: 46,
        max_version: 46,
        supported_capabilities: protocol::KNOWN_CAPABILITIES,
        required_capabilities: protocol::KNOWN_CAPABILITIES,
        numeric_schema: 1,
    };
    let mut session = protocol::ModernClientSession::new(hello).unwrap();
    let bytes = session.begin().unwrap();
    let input = peer.child.stdin.as_mut().unwrap();
    writeln!(
        input,
        "{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    )
    .unwrap();
    input.flush().unwrap();
    let line = peer.line();
    let ack = session
        .accept_ack(&hex(line.strip_prefix("ACK:").unwrap()))
        .unwrap();
    assert_eq!(ack.version, 46);
    assert_ne!(ack.capabilities & protocol::CAP_SLOW_DATA, 0);
    let mut followers: Vec<(String, d::Player, Follower)> = Vec::new();
    let mut count = 0;
    let mut material_rejects = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:81");
            break;
        }
        let parts = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(parts[0], "READ");
        let bytes = hex(parts[2]);
        let frame = app::decode_response_version(&bytes, ack.version).unwrap();
        assert_eq!(frame.request_id, 77);
        let app::Response::BattleData(reply) = frame.response else {
            panic!("not READ DATA")
        };
        assert_eq!(reply.state, read::State::Data);
        let data = reply.data.as_ref().unwrap();
        let player = data.personal.player.clone();
        assert!(data.speed.is_some());
        assert!(data.haste.is_none() && data.npc_motion.is_none());
        let payload = payload::encode(data).unwrap();
        let mut rebuilt = read::encode_request(77, &reply.echo).unwrap();
        rebuilt[4..8].copy_from_slice(&((72 + payload.len()) as u32).to_be_bytes());
        rebuilt[12..14].copy_from_slice(&read::RESULT_KIND.to_be_bytes());
        rebuilt.extend([1, 0, 0, 0]);
        rebuilt.extend((payload.len() as u32).to_be_bytes());
        rebuilt.extend(payload);
        assert_eq!(rebuilt, bytes);
        peer.ack(&rebuilt);
        for v in 35..46 {
            assert!(app::decode_response_version(&bytes, v).is_err());
        }
        let label = parts[1];
        let group = ["pc-", "npc-", "gear-", "budget-"]
            .into_iter()
            .find(|prefix| {
                label.starts_with(prefix)
                    && !label.starts_with("pc-wide")
                    && !label.starts_with("npc-prerequisite")
            })
            .unwrap_or(label)
            .to_owned();
        let index = followers
            .iter()
            .position(|(g, p, _)| *g == group && *p == player)
            .unwrap_or_else(|| {
                followers.push((
                    group,
                    player,
                    Follower::new(reply.echo.clone(), ack).unwrap(),
                ));
                followers.len() - 1
            });
        let follower = &mut followers[index].2;
        follower.accept(reply.clone()).unwrap();
        let keep = follower.observation().unwrap().clone();
        let mut wrong = reply.clone();
        wrong.echo.context.epoch ^= 1;
        assert!(follower.accept(wrong).is_err());
        if reply
            .data
            .as_ref()
            .unwrap()
            .personal
            .materials
            .as_ref()
            .unwrap()
            .uses
            > 0
        {
            let mut wrong = reply.clone();
            wrong
                .data
                .as_mut()
                .unwrap()
                .personal
                .materials
                .as_mut()
                .unwrap()
                .stocks
                .0[0]
                .remaining += 1;
            assert!(follower.accept(wrong).is_err());
            material_rejects += 1;
        }
        assert_eq!(follower.observation().unwrap().data(), keep.data());
        assert_eq!(follower.observation().unwrap().personal(), keep.personal());
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 81);
    assert!(material_rejects > 25);
    eprintln!("v46 HELLO/ACK -> 81 full READ replies -> atomic production Follower; {material_rejects} material rewrites and 81 foreign epochs rejected; v35-45 reject every new reply; no TLS/Main/UI claim");
}
