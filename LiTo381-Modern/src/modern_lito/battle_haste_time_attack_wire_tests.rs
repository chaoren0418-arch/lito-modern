//! Complete cooperative schema5 and production Follower, not just an internal
//! capture. Synthetic Java map; the separate Main/SQL test owns shared resources.
use super::{admission, data, haste, payload, Peer};
use crate::modern_lito::world_protocol::{MatchDescriptor as D, MatchRuleset};

#[test]
fn cooperative_haste_descriptor_requires_complete_dependencies_and_explicit_new_contract() {
    let d = D {
        ruleset: MatchRuleset::CooperativeTimeAttack,
        features: D::TIME_ATTACK
            | D::NPC
            | D::COMBAT
            | D::SPELLS
            | D::ITEMS
            | D::WEIGHT
            | D::TIMED
            | D::HASTE_CANDIDATE,
        preparation_timeout_millis: 2000,
        load_timeout_millis: 2000,
        countdown_nanos: 0,
        active_timeout_millis: 60_000,
    };
    d.validate_haste_candidate(4).unwrap();
    assert!(d.validate(4).is_err());
    for bit in [
        D::NPC,
        D::COMBAT,
        D::SPELLS,
        D::ITEMS,
        D::WEIGHT,
        D::TIMED,
        D::HASTE_CANDIDATE,
    ] {
        assert!(D {
            features: d.features & !bit,
            ..d
        }
        .validate_haste_candidate(4)
        .is_err());
    }
    for bit in [D::GREATER_HASTE, D::AMMO, D::ISOLATED_FORCES, D::FIRE_STORM] {
        assert!(D {
            features: d.features | bit,
            ..d
        }
        .validate_haste_candidate(4)
        .is_err());
    }
    assert!(D {
        active_timeout_millis: 60_001,
        ..d
    }
    .validate_haste_candidate(4)
    .is_err());
    for mode in [0, 1, 2, 3, 5] {
        assert!(d.validate_haste_candidate(mode).is_err());
    }
    D {
        features: d.features & !(D::TIMED | D::HASTE_CANDIDATE),
        ..d
    }
    .validate(4)
    .unwrap();
}

#[test]
#[ignore = "compiled Java: cooperative43 complete schema5, terminal effects, source results and atomic production Follower; no TLS/HWND"]
fn java_haste43_time_attack_schema5_all_terminals_and_owner_transitions() {
    let mut peer = Peer::start_named(Some("haste43-time"));
    let mut groups = std::collections::BTreeMap::<String, Vec<(String, haste::Candidate)>>::new();
    let mut count = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:140");
            break;
        }
        let parts = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(parts[0], "HASTE43");
        let bytes = (0..parts[2].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&parts[2][i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let p = haste::decode(&bytes).unwrap();
        p.check_values(&p.data.personal.player, &p.data.world.identity)
            .unwrap_or_else(|e| panic!("{}: {e:?}", parts[1]));
        admission::values(&p);
        assert_eq!(haste::encode(&p).unwrap(), bytes);
        peer.ack(&bytes);
        assert_eq!(
            (p.data.room.mode, p.data.room.map, p.data.manifest.map),
            (4, 540, 554)
        );
        assert_eq!(
            p.data.world.descriptor.ruleset,
            data::Ruleset::CooperativeTimeAttack
        );
        assert!(p.data.time_attack.is_some() && p.effects.npcs.is_some());
        for version in 35..45 {
            assert!(payload::decode_versioned(&bytes, version).is_err());
        }
        if matches!(
            p.data.world.phase,
            data::Phase::Finished | data::Phase::Aborted
        ) {
            assert!(p.effects.targets.0.iter().all(|e| e.active.is_none()));
            if p.data.world.phase == data::Phase::Finished {
                let r = p.data.result.as_ref().unwrap();
                assert_eq!(r.participants.0.len(), 2);
                assert!(r
                    .participants
                    .0
                    .iter()
                    .all(|r| r.identity.id == 3333 && r.team == 0));
                assert_eq!(
                    r.participants
                        .0
                        .iter()
                        .map(|r| r.identity.source)
                        .collect::<Vec<_>>(),
                    vec![7, 42]
                );
                for r in &r.participants.0 {
                    assert_eq!(
                        r.verdict,
                        if parts[1].starts_with("clear") {
                            data::Verdict::Win
                        } else {
                            data::Verdict::Loss
                        }
                    );
                }
            } else {
                assert!(p.data.result.is_none());
            }
        }
        let (kind, _) = parts[1].split_once('.').unwrap();
        let (_, owner) = parts[1].rsplit_once('-').unwrap();
        groups
            .entry(format!("{kind}-{owner}"))
            .or_default()
            .push((parts[1].into(), p));
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 140);
    assert_eq!(groups.len(), 16);
    let mut transitions = 0;
    for stages in groups.values() {
        if stages[0].0.starts_with("depart.") {
            let a = &stages
                .iter()
                .find(|(label, _)| label.contains(".npc-"))
                .unwrap()
                .1;
            let b = &stages
                .iter()
                .find(|(label, _)| label.contains(".left-"))
                .unwrap()
                .1;
            // A live recipient cannot lose an unexpired effect without a mode departure.
            let mut premature = b.clone();
            let t = premature.data.time_attack.as_mut().unwrap();
            t.departed.0.clear();
            t.leader = 1;
            premature
                .check_values(
                    &premature.data.personal.player,
                    &premature.data.world.identity,
                )
                .unwrap();
            assert!(a.check_successor(&premature).is_err());
            admission::reject(a, &premature);
            // Conversely, carrying the departed player's old effect is not a valid snapshot.
            let mut retained = b.clone();
            retained.data.actors.0[0].buffs = a.data.actors.0[0].buffs.clone();
            let grant = a
                .effects
                .targets
                .0
                .iter()
                .find(|e| {
                    matches!(&e.target,
                haste::Target::Player(p) if p.entity == 1)
                })
                .unwrap()
                .clone();
            retained.effects.targets.0.insert(0, grant);
            assert!(retained
                .check_values(
                    &retained.data.personal.player,
                    &retained.data.world.identity
                )
                .is_err());
            admission::reject(a, &retained);
        }
        for (i, (from, a)) in stages.iter().enumerate() {
            for (to, b) in &stages[i..] {
                a.check_successor(b)
                    .unwrap_or_else(|e| panic!("{from} -> {to}: {e:?}"));
                admission::transition(a, b);
                transitions += 1;
            }
        }
        let first = &stages[0].1;
        let last = &stages.last().unwrap().1;
        let mut wrong = last.clone();
        wrong.data.time_attack.as_mut().unwrap().leader = 61;
        assert!(wrong
            .check_values(&wrong.data.personal.player, &wrong.data.world.identity)
            .is_err());
        admission::reject(first, &wrong);
    }
    assert_eq!(transitions, 684);
}
