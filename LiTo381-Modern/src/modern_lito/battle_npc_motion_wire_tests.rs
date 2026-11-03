//! Same Java-produced engine publication, two independent typed codecs.
use super::{
    super::{
        self as payload,
        npc_motion::{self as motion, Candidate},
    },
    Peer,
};

fn check(p: &Candidate) {
    p.check_values(&p.data.personal.player, &p.data.world.identity)
        .unwrap();
}
fn reject(p: &Candidate, edit: impl FnOnce(&mut Candidate)) {
    let mut bad = p.clone();
    edit(&mut bad);
    let parsed = motion::decode(&motion::encode(&bad).unwrap()).unwrap();
    assert!(parsed
        .check_values(&p.data.personal.player, &p.data.world.identity)
        .is_err());
}
fn attached(p: &Candidate) -> payload::UnvalidatedData {
    payload::decode_versioned(&motion::encode(p).unwrap(), 43).unwrap()
}
fn reply(d: payload::UnvalidatedData) -> crate::modern_lito::battle_read::UnvalidatedReply {
    use crate::modern_lito::{application_protocol as app, battle_read as read};
    let p = &d.personal.player;
    read::UnvalidatedReply {
        echo: read::Request {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: p.source as u32,
                    object_id: p.object as u64,
                },
                epoch: p.epoch as u64,
            },
            channel_id: p.channel as u8,
            world: d.world.identity.clone(),
        },
        state: read::State::Data,
        data: Some(d),
    }
}
fn follower(p: &Candidate, version: u16) -> crate::modern_lito::battle_follow::Follower {
    use crate::modern_lito::{battle_follow::Follower, protocol};
    Follower::new(
        reply(p.data.clone()).echo,
        protocol::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
    )
    .unwrap()
}
fn motion_frames(mode: &str, expected: usize) -> Vec<(String, Candidate)> {
    let mut peer = Peer::start_named(Some(mode));
    let mut frames = Vec::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, format!("CLOSED:{expected}"));
            break;
        }
        let fields = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], "MOTION");
        let hex = fields[2];
        assert!(hex.len() % 2 == 0 && hex.len() <= motion::MAX_BYTES * 2);
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let p = motion::decode(&bytes).unwrap();
        check(&p);
        let encoded = motion::encode(&p).unwrap();
        assert_eq!(bytes, encoded);
        peer.ack(&encoded);
        for v in [
            35,
            36,
            37,
            41,
            42,
            crate::modern_lito::protocol::MAX_VERSION + 1,
            u16::MAX,
        ] {
            assert!(payload::decode_versioned(&bytes, v).is_err());
        }
        let attached = payload::decode_versioned(&bytes, 43).unwrap();
        assert_eq!(payload::decode_versioned(&bytes, 44).unwrap(), attached);
        assert_eq!(payload::decode_versioned(&bytes, 45).unwrap(), attached);
        assert_eq!(attached.npc_motion.as_ref().unwrap().state, p.motion);
        assert_eq!(payload::encode(&attached).unwrap(), bytes);
        attached
            .check_scene_values(&attached.personal.player, &attached.world.identity)
            .unwrap();
        let old = payload::encode(&p.data).unwrap();
        assert_eq!(&old[16..], &bytes[16..old.len()]);
        frames.push((fields[1].to_owned(), p));
    }
    peer.finish();
    assert_eq!(frames.len(), expected);
    frames
}
#[test]
#[ignore = "explicit compiled Java publication peer and v43 follower; TLS and UI require separate tests"]
fn java_npc_motion_schema4_scoped_pace_steps_bounds_and_legacy_rejection() {
    let frames = motion_frames("npc-motion", 22);
    crate::modern_lito::battle_scene::tests::verify_npc_pacing_scene(
        &frames.iter().map(|(_, p)| attached(p)).collect::<Vec<_>>(),
    );
    // The real production follower admits v43 atomically and preserves the
    // extension in Observation; an old handshake or a stripped extension fails.
    for (_, p) in &frames {
        let mut f = follower(p, 43);
        let d = attached(p);
        f.accept(reply(d.clone())).unwrap();
        assert_eq!(f.observation().unwrap().data(), &d);
        assert!(f.accept(reply(p.data.clone())).is_err());
        assert_eq!(f.observation().unwrap().data(), &d);
        assert!(follower(p, 42).accept(reply(d)).is_err());
    }
    // All forward skip pairs, duplicate observations and both owner timelines.
    let mut pairs = 0;
    for owner in 0..2 {
        let series = (owner..16)
            .step_by(2)
            .map(|i| &frames[i].1)
            .collect::<Vec<_>>();
        for (i, old) in series.iter().enumerate() {
            for (j, new) in series.iter().enumerate().skip(i) {
                old.check_successor(new)
                    .unwrap_or_else(|e| panic!("owner{owner}/{i}->{j}: {e:?}"));
                pairs += 1;
                let mut accepted = follower(old, 43);
                accepted.accept(reply(attached(old))).unwrap();
                accepted.accept(reply(attached(new))).unwrap();
                if old.data.world.revision < new.data.world.revision {
                    assert!(new.check_successor(old).is_err());
                }
            }
        }
    }
    let moved = &frames[6].1;
    assert_eq!(frames[6].0, "walk100-0");
    let m = moved.motion.as_ref().unwrap();
    let w = m.actors.0[0].last_walk.as_ref().unwrap();
    assert_eq!(
        (w.sequence, w.life, w.elapsed_millis, w.duration_millis),
        (1, 1, 100, 100)
    );
    assert_eq!(m.actors.0[0].attack_millis, 750);
    assert_eq!(frames[16].1.motion.as_ref().unwrap().actors.0.len(), 128);
    let a = &frames[18].1;
    let b = &frames[19].1;
    assert_eq!((a.data.world.lane, b.data.world.lane), (0, 1));
    assert!(a.motion.as_ref().unwrap().actors.0.iter().all(|x| b
        .motion
        .as_ref()
        .unwrap()
        .actors
        .0
        .iter()
        .all(|y| x.serial != y.serial)));
    reject(a, |v| v.motion = b.motion.clone());
    assert!(frames[20].1.motion.is_none());
    assert!(frames[20].1.data.greater_haste());
    for case in 0..25 {
        reject(moved, |v| {
            if case == 0 {
                v.motion = None;
                return;
            }
            let m = v.motion.as_mut().unwrap();
            match case {
                1 => m.world.generation += 1,
                2 => m.revision += 1,
                3 => m.elapsed_millis += 1,
                4 => m.actors.0.clear(),
                5 => m.actors.0.push(m.actors.0[0].clone()),
                6 => m.actors.0[0].serial += 1,
                7 => m.actors.0[0].life += 1,
                8 => m.actors.0[0].walk_millis = 0,
                9 => m.actors.0[0].attack_millis = 60_001,
                10 => m.actors.0[0].last_walk = None,
                _ => {
                    let w = m.actors.0[0].last_walk.as_mut().unwrap();
                    match case {
                        11 => w.sequence = 0,
                        12 => w.sequence = i64::MAX,
                        13 => w.life = 0,
                        14 => w.life += 1,
                        15 => w.roster_revision = 1,
                        16 => w.roster_revision = i64::MAX,
                        17 => w.elapsed_millis = -1,
                        18 => w.elapsed_millis = m.elapsed_millis + 1,
                        19 => w.duration_millis += 1,
                        20 => w.from = w.to.clone(),
                        21 => w.to.x += 1,
                        22 => w.heading = 8,
                        23 => w.heading = (w.heading + 1) % 8,
                        24 => w.from.x = i32::MIN,
                        _ => unreachable!(),
                    }
                }
            }
        });
    }
    let next = &frames[8].1;
    assert_eq!(frames[8].0, "walk200-0");
    for case in 0..4 {
        let mut bad = next.clone();
        let n = &mut bad.data.npcs.as_mut().unwrap().actors.0[0];
        let actor = &mut bad.motion.as_mut().unwrap().actors.0[0];
        match case {
            0 => actor.attack_millis += 1,
            1 => actor.last_walk.as_mut().unwrap().sequence = 1,
            2 => {
                let w = actor.last_walk.as_mut().unwrap();
                w.elapsed_millis = 99;
                n.next_move_millis = 199;
            }
            3 => {
                let w = actor.last_walk.as_mut().unwrap();
                w.from.x = w.to.x + 1;
                w.from.y = w.to.y;
                w.heading = 6;
                n.heading = 6;
            }
            _ => unreachable!(),
        }
        check(&bad);
        assert!(moved.check_successor(&bad).is_err(), "temporal {case}");
        let mut accepted = follower(moved, 43);
        let before = attached(moved);
        accepted.accept(reply(before.clone())).unwrap();
        assert!(accepted.accept(reply(attached(&bad))).is_err());
        assert_eq!(accepted.observation().unwrap().data(), &before);
        accepted.accept(reply(attached(next))).unwrap();
        // Rejected candidate cannot mutate either previously retained value.
        assert_eq!(
            moved,
            motion::decode(&motion::encode(moved).unwrap())
                .as_ref()
                .unwrap()
        );
        moved.check_successor(next).unwrap();
    }
    eprintln!("NPC motion v43:22 Java publications, {pairs} forward/duplicate pairs admitted by the actual follower; 26 scene corruptions + 4 individually valid history rewrites rejected; schema2/3 sections byte-identical; old versions reject schema4 and v43 requires its envelope; TLS and UI require separate tests");
}

#[test]
#[ignore = "real Java engine 100ms ticks / 200ms observations, two NPCs and shared SPR/WAV; synthetic terrain, no TLS/HWND/device"]
fn java_v43_coalesced_npc_walk_and_attack_audio_share_confirmed_clock() {
    let frames = motion_frames("npc-motion-presentation", 26);
    for owner in 0..2 {
        let mut f = follower(&frames[owner].1, 43);
        let mut trace = Vec::new();
        for (index, (name, p)) in frames.iter().skip(owner).step_by(2).enumerate() {
            assert_eq!(name, &format!("cadence{}-{owner}", index * 200));
            let data = attached(p);
            f.accept(reply(data.clone())).unwrap();
            assert_eq!(f.observation().unwrap().data(), &data);
            trace.push(data);
        }
        crate::modern_lito::battle_scene::npc_cadence_tests::verify(&trace);
    }
}
