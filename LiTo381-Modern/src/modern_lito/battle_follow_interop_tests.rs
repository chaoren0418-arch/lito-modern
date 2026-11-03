//! Genuine sequences, including arbitrary skipped observations. Semantic
//! single-frame acceptance is required before counting temporal rejection.
use super::{
    super::{data::*, decode, encode},
    Peer,
};
use crate::modern_lito::{
    application_protocol as app, battle_follow::Follower, battle_read as read, protocol,
};
fn request(d: &UnvalidatedData) -> read::Request {
    read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: d.personal.player.source as u32,
                object_id: d.personal.player.object as u64,
            },
            epoch: d.personal.player.epoch as u64,
        },
        channel_id: d.personal.player.channel as u8,
        world: d.world.identity.clone(),
    }
}
fn fresh(d: &UnvalidatedData) -> Follower {
    Follower::new(
        request(d),
        protocol::Ack {
            request_id: 1,
            version: 35,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
    )
    .unwrap()
}
fn reply(d: &UnvalidatedData) -> read::UnvalidatedReply {
    read::UnvalidatedReply {
        echo: request(d),
        state: read::State::Data,
        data: Some(d.clone()),
    }
}
fn traces() -> std::collections::BTreeMap<String, Vec<UnvalidatedData>> {
    let mut peer = Peer::start_mode(true);
    let mut groups = std::collections::BTreeMap::<String, Vec<UnvalidatedData>>::new();
    let mut count = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:101");
            break;
        }
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "FRAME");
        let label = fields[1];
        let (group, index) = label.rsplit_once('-').unwrap();
        let bytes = fields[3]
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let d = decode(&bytes).unwrap();
        let encoded = encode(&d).unwrap();
        assert_eq!(encoded, bytes);
        d.check_scene_values(&d.personal.player, &d.world.identity)
            .unwrap_or_else(|_| panic!("{label}: single scene rejected"));
        let series = groups.entry(group.to_owned()).or_default();
        assert_eq!(index.parse::<usize>().unwrap(), series.len());
        series.push(d);
        peer.ack(&encoded);
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 101);
    assert_eq!(groups.len(), 22);
    groups
}
#[test]
#[ignore = "explicit Java durable gear authority and canonical follower; not TLS SET or UI admission"]
fn java_durable_equipment_changes_follow_both_owners_without_repricing_past_events() {
    let mut peer = Peer::start_named(Some("durable-equipment"));
    let mut series: [Vec<UnvalidatedData>; 2] = std::array::from_fn(|_| Vec::new());
    for n in 0..12 {
        let line = peer.line();
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "FRAME");
        assert_eq!(
            fields[1],
            format!("gear-{}-{}", n / 2, if n % 2 == 0 { "a" } else { "b" })
        );
        let bytes = fields[3]
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let d = decode(&bytes).unwrap();
        let rebuilt = encode(&d).unwrap();
        assert_eq!(rebuilt, bytes);
        d.check_scene_values(&d.personal.player, &d.world.identity)
            .unwrap();
        series[n % 2].push(d);
        peer.ack(&rebuilt);
    }
    assert_eq!(peer.line(), "CLOSED:12");
    peer.finish();
    let owner = &series[0];
    assert_eq!(
        owner
            .iter()
            .map(|d| d.personal.equipment.as_ref().unwrap().revision)
            .collect::<Vec<_>>(),
        vec![0, 0, 1, 1, 2, 2]
    );
    assert!(series[1]
        .iter()
        .all(|d| d.personal.equipment.as_ref().unwrap().revision == 0));
    for (a, b) in owner.iter().zip(&series[1]) {
        assert_ne!(a.personal.player.source, b.personal.player.source);
        assert_eq!(
            a.actors, b.actors,
            "both owners observe the same atomic public equipment/skill state"
        );
    }
    let before = &owner[1].personal.history.as_ref().unwrap().0;
    let changed = &owner[2].personal.history.as_ref().unwrap().0;
    assert!(crate::modern_lito::equipment_history::follows_reprofile(
        before, changed
    ));
    assert_eq!(before.resources.flows.spent, changed.resources.flows.spent);
    assert_eq!(
        owner[1].personal.supplies.as_ref().unwrap().stocks,
        owner[2].personal.supplies.as_ref().unwrap().stocks
    );
    assert_eq!(
        owner[3].personal.supplies.as_ref().unwrap().stocks,
        owner[4].personal.supplies.as_ref().unwrap().stocks
    );
    let final_items = &owner[5].personal.supplies.as_ref().unwrap().stocks.0;
    assert_eq!(
        final_items
            .iter()
            .find(|s| s.item == 40010)
            .unwrap()
            .remaining,
        0
    );
    assert_eq!(
        owner[3].actors.0[0].vital.as_ref().unwrap().values.hp,
        owner[4].actors.0[0].vital.as_ref().unwrap().values.hp
    );
    let mut pairs = 0;
    for sequence in &series {
        for (i, old) in sequence.iter().enumerate() {
            for new in sequence.iter().skip(i) {
                let mut follower = Follower::new(
                    request(old),
                    protocol::Ack {
                        request_id: 1,
                        version: 37,
                        numeric_schema: 1,
                        capabilities: protocol::KNOWN_CAPABILITIES,
                    },
                )
                .unwrap();
                follower.accept(reply(old)).unwrap();
                let retained = follower.observation().unwrap().clone();
                follower
                    .accept(reply(new))
                    .expect("genuine durable gear trace, including coalesced changes");
                assert_eq!(retained.data(), old);
                assert_eq!(follower.observation().unwrap().data(), new);
                assert_eq!(
                    follower.observation().unwrap().personal().revision(),
                    new.world.revision
                );
                if new.world.revision > old.world.revision {
                    assert!(
                        follower.accept(reply(old)).is_err(),
                        "old equipment must not roll back a confirmed new revision"
                    );
                    assert_eq!(follower.observation().unwrap().data(), new);
                }
                pairs += 1;
            }
        }
    }
    assert_eq!(pairs, 42);
}
#[test]
#[ignore = "explicit Java WorldService with durable gear and terminal persistence; not network SET or UI"]
fn java_service_equipment_history_reaches_both_owners_and_terminal_without_legacy_frames() {
    let mut peer = Peer::start_named(Some("service-equipment"));
    let mut series: [Vec<UnvalidatedData>; 2] = std::array::from_fn(|_| Vec::new());
    for n in 0..8 {
        let line = peer.line();
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "FRAME");
        assert_eq!(
            fields[1],
            format!(
                "service-gear-{}-{}",
                n / 2,
                if n % 2 == 0 { "a" } else { "b" }
            )
        );
        let bytes = fields[3]
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let d = decode(&bytes).unwrap();
        let rebuilt = encode(&d).unwrap();
        assert_eq!(rebuilt, bytes);
        d.check_scene_values(&d.personal.player, &d.world.identity)
            .unwrap();
        series[n % 2].push(d);
        peer.ack(&rebuilt);
    }
    assert_eq!(peer.line(), "CLOSED:8");
    peer.finish(); // Java asserts actual saved receipt and both LEAVEs before CLOSED.
    assert_eq!(
        series[0]
            .iter()
            .map(|d| d.personal.equipment.as_ref().unwrap().revision)
            .collect::<Vec<_>>(),
        vec![0, 1, 1, 1]
    );
    assert!(series[1]
        .iter()
        .all(|d| d.personal.equipment.as_ref().unwrap().revision == 0));
    let mut pairs = 0;
    for sequence in &series {
        assert_eq!(sequence[0].world.phase, Phase::Active);
        assert_eq!(sequence[1].world.phase, Phase::Active);
        assert_eq!(sequence[2].world.phase, Phase::Finished);
        assert_eq!(sequence[3].world.phase, Phase::Finished);
        for (i, old) in sequence.iter().enumerate() {
            for new in sequence.iter().skip(i) {
                let mut follower = Follower::new(
                    request(old),
                    protocol::Ack {
                        request_id: 1,
                        version: 37,
                        numeric_schema: 1,
                        capabilities: protocol::KNOWN_CAPABILITIES,
                    },
                )
                .unwrap();
                follower.accept(reply(old)).unwrap();
                let retained = follower.observation().unwrap().clone();
                follower
                    .accept(reply(new))
                    .expect("actual service equipment/terminal publication");
                assert_eq!(retained.data(), old);
                assert_eq!(follower.observation().unwrap().data(), new);
                if new.world.revision > old.world.revision {
                    assert!(follower.accept(reply(old)).is_err());
                    assert_eq!(follower.observation().unwrap().data(), new);
                }
                pairs += 1;
            }
        }
    }
    assert_eq!(pairs, 20);
}
#[test]
#[ignore = "explicit compiled Java Engine traces; shared geometry qualification for six modes, no SQL/UI"]
fn java_canonical_modes_qualify_exact_navigation_without_load_permission() {
    let mut groups = traces();
    let mut inputs = groups.clone();
    // The 101-frame temporal set has modes 0/2/3/4/5, not Deathmatch.
    // Include genuine mode1 publications from the independent full-payload
    // peer; never rename a fixture's mode just to satisfy coverage.
    let mut peer = Peer::start();
    let mut total = 0;
    let mut added = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:40");
            break;
        }
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "FRAME");
        let bytes = fields[3]
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let data = decode(&bytes).unwrap();
        let encoded = encode(&data).unwrap();
        assert_eq!(encoded, bytes);
        inputs.insert(format!("snapshot-{}", fields[1]), vec![data.clone()]);
        if data.room.mode == 1 && data.world.phase != Phase::Aborted {
            groups.insert(format!("snapshot-{}", fields[1]), vec![data]);
            added += 1;
        }
        peer.ack(&encoded);
        total += 1;
    }
    peer.finish();
    assert_eq!(total, 40);
    assert!(added > 0);
    crate::modern_lito::battle_space::tests::verify_traces(&groups);
    let mut peer = Peer::start_named(Some("player-input"));
    for _ in 0..4 {
        let line = peer.line();
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "FRAME");
        let bytes = fields[3]
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let data = decode(&bytes).unwrap();
        let encoded = encode(&data).unwrap();
        assert_eq!(encoded, bytes);
        assert!(inputs.insert(fields[1].into(), vec![data]).is_none());
        peer.ack(&encoded);
    }
    assert_eq!(peer.line(), "CLOSED:4");
    peer.finish();
    crate::modern_lito::battle_input::tests::verify_traces(&inputs);
}
#[test]
#[ignore = "explicit compiled Java canonical trace peer; no SQL, GUI or command permission"]
fn java_canonical_sequences_allow_catchup_and_reject_rewrites_atomically() {
    let groups = traces();
    let count: usize = groups.values().map(Vec::len).sum();
    // Fresh engine revisions do not grant permission to reuse a request that
    // an earlier canonical observation already reported as denied.
    let requests = &groups["requests"];
    for (before, after) in [(1, 2), (3, 4)] {
        let old = &requests[before];
        let new = &requests[after];
        assert_ne!(old.personal.entity, old.actors.0[0].entity);
        let sequence = old.actors.0[0].spells.sequence;
        let mut forged = new.clone();
        let spells = &mut forged.actors.0[0].spells;
        let cast = spells
            .histories
            .0
            .iter_mut()
            .find(|s| s.lane == Lane::Healing)
            .unwrap();
        cast.ledger.last_cast_sequence = sequence;
        cast.last.as_mut().unwrap().sequence = sequence;
        cast.receipt.as_mut().unwrap().impact.usage.sequence = sequence;
        // Move the enclosing observation forward as a malicious replay would.
        // Otherwise the existing same-engine-revision scene check already
        // rejects the changed request, before the new temporal gate runs.
        forged.world.revision += 1;
        if let Some(history) = &mut forged.personal.history {
            history.0.world_revision = forged.world.revision;
        }
        forged
            .check_scene_values(&new.personal.player, &new.world.identity)
            .unwrap();
        let mut follower = fresh(old);
        follower.accept(reply(old)).unwrap();
        assert!(
            follower.accept(reply(&forged)).is_err(),
            "denied request reused, observation={before}->{after}"
        );
        assert_eq!(follower.observation().unwrap().data(), old);
        follower.accept(reply(new)).unwrap();
    }
    let mut pairs = 0;
    let mut motion_counts = [0usize; 8];
    let mut rejects = 0;
    let mut same_clock = 0;
    let mut regen_clock_only = 0;
    let mut forward_rejections = 0;
    for (label, series) in &groups {
        // Every ordered pair is a valid polling gap, not only adjacent commands.
        for (i, old) in series.iter().enumerate() {
            for (j, new) in series.iter().enumerate().skip(i) {
                let mut f = fresh(old);
                f.accept(reply(old)).unwrap();
                let retained = f.observation().unwrap().clone();
                f.accept(reply(new))
                    .unwrap_or_else(|_| panic!("{label} {i}->{j}: legitimate trace rejected"));
                assert_eq!(retained.data(), old);
                assert_eq!(f.observation().unwrap().data(), new);
                let decisions = crate::modern_lito::battle_motion::tests::verify_pair(
                    retained.clone(),
                    f.observation().unwrap().clone(),
                );
                for (sum, value) in motion_counts.iter_mut().zip(decisions) {
                    *sum += value;
                }
                let baseline = crate::modern_lito::battle_scene::baseline_counts(
                    f.observation().unwrap().clone(),
                );
                if new.world.phase == Phase::Aborted {
                    assert!(baseline.is_err(), "aborted scene must retire");
                } else {
                    let (actors, props, npc_forms, pet_forms) =
                        baseline.expect("direct canonical scene plan");
                    assert_eq!(actors, new.actors.0.len());
                    assert_eq!(
                        props,
                        new.honor.objectives.0.len()
                            + new.quest.as_ref().map_or(0, |q| q.objectives.0.len())
                            + usize::from(new.time_attack.is_some())
                            + new.siege.as_ref().map_or(0, |s| s.objectives.0.len())
                    );
                    assert_eq!(
                        npc_forms,
                        new.npcs.as_ref().map_or(0, |n| n.graphics.0.len())
                    );
                    assert_eq!(pet_forms, usize::from(new.race.is_some()));
                }
                assert_eq!(
                    f.observation().unwrap().personal().revision(),
                    new.world.revision
                );
                pairs += 1;
                if old.world.revision == new.world.revision
                    && old.world.loading_elapsed_nanos < new.world.loading_elapsed_nanos
                {
                    same_clock += 1;
                }
                if old.world.revision == new.world.revision
                    && old.personal.regen_clock != new.personal.regen_clock
                {
                    regen_clock_only += 1;
                }
                let mut corrupt = new.clone();
                corrupt.actors.0[0].heading ^= 1;
                corrupt
                    .check_scene_values(&new.personal.player, &new.world.identity)
                    .unwrap();
                assert!(
                    f.accept(reply(&corrupt)).is_err(),
                    "{label} {j}: same-revision pose rewrite"
                );
                assert_eq!(f.observation().unwrap().data(), new);
                rejects += 1;
                if old != new {
                    assert!(
                        f.accept(reply(old)).is_err(),
                        "{label} {j}->{i}: rollback accepted"
                    );
                    assert_eq!(f.observation().unwrap().data(), new);
                    rejects += 1;
                }
                if new.world.revision > old.world.revision {
                    let mut renamed = new.clone();
                    renamed.room.name = Text("changed frozen room".into());
                    renamed
                        .check_scene_values(&new.personal.player, &new.world.identity)
                        .unwrap();
                    let mut start = fresh(old);
                    start.accept(reply(old)).unwrap();
                    assert!(
                        start.accept(reply(&renamed)).is_err(),
                        "{label}: frozen room changed in a newer revision"
                    );
                    assert_eq!(start.observation().unwrap().data(), old);
                    forward_rejections += 1;
                    if let Some(skill) = new.personal.skills.0.first() {
                        let mut renamed = new.clone();
                        renamed.personal.skills.0[0].name = Text(format!("{}!", skill.name.0));
                        renamed
                            .check_scene_values(&new.personal.player, &new.world.identity)
                            .unwrap();
                        assert!(
                            start.accept(reply(&renamed)).is_err(),
                            "{label}: frozen source skill changed"
                        );
                        forward_rejections += 1;
                    }
                    if let Some(index) = new.actors.0.iter().position(|v| {
                        v.entity != new.personal.entity
                            && v.vital.is_some()
                            && old
                                .actors
                                .0
                                .iter()
                                .find(|o| o.entity == v.entity)
                                .is_some_and(|o| {
                                    o.resources.as_ref().unwrap().equipment_revision
                                        == v.resources.as_ref().unwrap().equipment_revision
                                })
                    }) {
                        let mut altered = new.clone();
                        altered.actors.0[index].vital.as_mut().unwrap().ac += 1;
                        altered
                            .check_scene_values(&new.personal.player, &new.world.identity)
                            .unwrap();
                        assert!(
                            start.accept(reply(&altered)).is_err(),
                            "{label}: foreign actor changed AC without equipment era"
                        );
                        forward_rejections += 1;
                    }
                    if old.world.phase == Phase::Active && new.world.phase == Phase::Active {
                        // A replay disguised by a fresh outer engine number must
                        // not roll actor/mode ledgers back. Match clocks are copied
                        // forward, so this is not merely the old-clock rejection.
                        let mut stale = old.clone();
                        stale.world.revision = new.world.revision + 1;
                        stale.world.loading_elapsed_nanos = new.world.loading_elapsed_nanos;
                        stale.world.active_elapsed_nanos = new.world.active_elapsed_nanos;
                        if let Some(h) = &mut stale.personal.history {
                            h.0.world_revision = stale.world.revision;
                        }
                        stale
                            .check_scene_values(&new.personal.player, &new.world.identity)
                            .unwrap();
                        assert!(
                            f.accept(reply(&stale)).is_err(),
                            "{label} {i}->{j}: old state under fresh revision"
                        );
                        assert_eq!(f.observation().unwrap().data(), new);
                        forward_rejections += 1;
                    }
                }
            }
        }
    }
    assert!(same_clock > 0);
    assert!(regen_clock_only > 0);
    assert!(forward_rejections > 100);
    assert!(
        motion_counts.iter().all(|count| *count > 0),
        "every body motion needs a positive engine trace: {motion_counts:?}"
    );
    eprintln!("canonical motion decisions PC[attack,hurt,death,walk] NPC[attack,hurt,death,walk]={motion_counts:?}; decisions across ordered pairs, not unique events");
    eprintln!("canonical trace frames={count}, groups={}, ordered pairs={pairs}, atomic rejections={rejects}, forward rejections={forward_rejections}, denied-request replays=2, same-revision clock advances={same_clock}, regen-clock-only={regen_clock_only}",groups.len());
}

#[test]
#[ignore = "explicit Java engine trace and shared Lineage art; synthetic test terrain, no GUI or SQL"]
fn java_canonical_race_and_target_changes_animate_real_shared_resources() {
    let groups = traces();
    crate::modern_lito::battle_scene::tests::verify_shared_motion(&groups);
}

#[test]
#[ignore = "explicit Java engine trace and shared Lineage art/audio; synthetic terrain, no GUI/device/SQL"]
fn java_canonical_body_cues_share_pixel_clocks_and_remain_atomic() {
    let groups = traces();
    crate::modern_lito::battle_scene::body_audio_tests::verify(&groups);
}
