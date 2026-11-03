use super::*;
use crate::modern_lito::{protocol as p, world_map::WorldCell, world_protocol as world};

fn race(f: &Frame) -> &RaceData {
    f.race.as_ref().unwrap()
}
fn race_mut(f: &mut Frame) -> &mut RaceData {
    f.race.as_mut().unwrap()
}
fn rehash_course(f: &mut Frame) {
    let hash = race(f).course_hash(f);
    race_mut(f).course_sha256 = hash;
}
fn rehash_result(f: &mut Frame) {
    let hash = race(f).result_hash(f);
    race_mut(f).result_content_sha256 = hash;
}
pub(crate) fn waiting() -> Frame {
    let mut f = crate::modern_lito::world_runtime::tests::frame(9);
    f.room.summary.settings.mode = 0;
    f.room.summary.settings.map_id = 565;
    f.room.summary.settings.attributes = 0;
    f.actors[1].position.y = 32786;
    f.descriptor = Some(MatchDescriptor {
        ruleset: MatchRuleset::PetRace,
        features: MatchDescriptor::PET_RACE,
        preparation_timeout_millis: 30_000,
        load_timeout_millis: 30_000,
        countdown_nanos: 3_000_000_001,
        active_timeout_millis: 60_000,
    });
    f.race = Some(RaceData {
        state: RaceState::Waiting,
        settlement: Settlement::None,
        course_sha256: [0; 32],
        first_finisher_entity_id: 0,
        finish_window_deadline_elapsed_nanos: 0,
        terminal_revision: 0,
        result_id: [0; 16],
        result_content_sha256: [0; 32],
        gates: vec![
            Gate {
                min_x: 32726,
                max_x: 32727,
                min_y: 32784,
                max_y: 32784,
            },
            Gate {
                min_x: 32728,
                max_x: 32729,
                min_y: 32784,
                max_y: 32784,
            },
        ],
        runners: (1..=2)
            .map(|entity_id| Runner {
                entity_id,
                revision: 0,
                progress_move_sequence: 0,
                completed_laps: 0,
                next_checkpoint: 0,
                placement: 0,
                finish_elapsed_nanos: 0,
                dnf: DnfReason::None,
                base_graphics_id: 2145,
            })
            .collect(),
    });
    rehash_course(&mut f);
    f.validate(f.actors[0].key).unwrap();
    f
}
pub(crate) fn running() -> Frame {
    let mut f = waiting();
    f.phase = Phase::Active;
    f.revision = 2;
    for actor in &mut f.actors {
        actor.loaded = true;
    }
    race_mut(&mut f).state = RaceState::Running;
    f.validate(f.actors[0].key).unwrap();
    f
}
fn reply(f: Frame) -> world::Reply {
    world::Reply {
        echo: world::Request::read(
            app::SelectionContext {
                key: f.actors[0].key,
                epoch: 1,
            },
            1,
        ),
        status: world::Status::Ok,
        reason: world::Reason::None,
        frame: Some(f),
    }
}
fn follows_world(a: &Frame, b: &Frame) -> bool {
    world::follows(Some(&reply(a.clone())), &reply(b.clone()))
}
fn ordinary_move(old: &Frame, to: WorldCell) -> Frame {
    let mut f = old.clone();
    f.revision += 1;
    f.actors[0].sequence += 1;
    f.actors[0].position = to;
    f.validate(f.actors[0].key).unwrap();
    f
}
fn first_gate() -> (Frame, Frame) {
    let before = ordinary_move(&running(), WorldCell { x: 32725, y: 32784 });
    let mut after = before.clone();
    after.revision += 1;
    after.actors[0].sequence += 1;
    after.actors[0].position.x = 32726;
    let revision = after.revision;
    let sequence = after.actors[0].sequence;
    let r = &mut race_mut(&mut after).runners[0];
    r.revision = revision;
    r.progress_move_sequence = sequence;
    r.next_checkpoint = 1;
    after.validate(after.actors[0].key).unwrap();
    (before, after)
}
fn finish_window() -> Frame {
    let mut f = running();
    f.revision = 11;
    f.actors[0].sequence = 24;
    f.actors[0].position = WorldCell { x: 32728, y: 32784 };
    let r = race_mut(&mut f);
    r.state = RaceState::FinishWindow;
    r.first_finisher_entity_id = 1;
    r.finish_window_deadline_elapsed_nanos = 11_000_000_000;
    r.runners[0].revision = 11;
    r.runners[0].progress_move_sequence = 24;
    r.runners[0].completed_laps = 4;
    r.runners[0].placement = 1;
    r.runners[0].finish_elapsed_nanos = 1_000_000_000;
    f.validate(f.actors[0].key).unwrap();
    f
}
pub(crate) fn complete() -> Frame {
    let mut f = finish_window();
    f.phase = Phase::Finished;
    f.revision = 20;
    let r = race_mut(&mut f);
    r.state = RaceState::Complete;
    r.settlement = Settlement::Pending;
    r.terminal_revision = 20;
    r.result_id = hex("00112233445566778899aabbccddeeff").try_into().unwrap();
    r.runners[1].revision = 20;
    r.runners[1].dnf = DnfReason::WindowClosed;
    rehash_result(&mut f);
    f.validate(f.actors[0].key).unwrap();
    f
}
fn assert_bad(mut f: Frame, change: impl FnOnce(&mut Frame)) {
    change(&mut f);
    assert!(f.validate(f.actors[0].key).is_err());
}
fn wire(f: &Frame) -> Vec<u8> {
    let mut out = Vec::new();
    encode(&mut out, f.race.as_ref());
    out
}
fn read_exact(bytes: &[u8], template: &Frame) -> Result<RaceData, ApplicationError> {
    let (value, used) = decode(bytes)?;
    if used != bytes.len() {
        return Err(invalid());
    }
    let value = value.ok_or_else(invalid)?;
    let mut f = template.clone();
    f.race = Some(value.clone());
    f.validate(f.actors[0].key)?;
    Ok(value)
}
fn hex(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
/// Independent manually specified BE canonical bytes matching Java layout;
/// no production writer or hash/context helper contributes these bytes.
fn literal_course() -> Vec<u8> {
    hex(concat!(
        "00154e45575f4d415443485f5045545f524143455f5631",
        "000000000000023500000202",
        "0202020202020202020202020202020202020202020202020202020202020202",
        "0000000100007fff0000800000021a38",
        "0101010101010101010101010101010101010101010101010101010101010101",
        "0000000000000861000000000bebc2000000000400000002540be40000000002",
        "00007fd600007fd70000801000008010",
        "00007fd800007fd90000801000008010"
    ))
}

#[test]
fn canonical_course_and_result_bytes_match_independent_java_layout() {
    let f = complete();
    let course_sha: [u8; 32] = Sha256::digest(literal_course()).into();
    assert_eq!(race(&f).course_sha256, course_sha);
    let mut result = hex(concat!(
        "001e4c49544f5f4d4f4445524e5f5045545f524143455f524553554c545f5631",
        "00112233445566778899aabbccddeeff",
        "000000000000000900000000000000030000000000000014",
        "000000000000023500000202",
        "0202020202020202020202020202020202020202020202020202020202020202"
    ));
    result.extend(course_sha);
    result.extend(hex(concat!(
        "0000000000000001000000028fa6ae0000000002",
        "0000000000000001000000070000000000000d05",
        "000000000000000b00000000000000180000000000000004",
        "00000000000000000000000000000001000000003b9aca00",
        "00000000000000000000000000000861",
        "00000000000000020000002a0000000000000d05",
        "000000000000001400000000000000000000000000000000",
        "000000000000000000000000000000000000000000000000",
        "00000000000000030000000000000861"
    )));
    let result_sha: [u8; 32] = Sha256::digest(result).into();
    assert_eq!(race(&f).result_content_sha256, result_sha);
}

#[test]
fn disabled_is_eight_bytes_enabled_is_112_plus_gates16_and_runners72() {
    let mut off = Vec::new();
    encode(&mut off, None);
    assert_eq!(off, vec![0; 8]);
    assert_eq!(decode(&off).unwrap(), (None, 8));
    for i in 1..8 {
        let mut bad = off.clone();
        bad[i] = 1;
        assert!(decode(&bad).is_err());
    }
    let f = waiting();
    let bytes = wire(&f);
    assert_eq!(bytes.len(), 112 + 2 * 16 + 2 * 72);
    assert_eq!(&bytes[..8], hex("0100000000020204"));
    assert_eq!(&bytes[40..112], &[0; 72]);
    assert_eq!(&bytes[144..152], &1i64.to_be_bytes());
    assert_eq!(&bytes[208..216], &2145i64.to_be_bytes());
    assert_eq!(&bytes[216..224], &2i64.to_be_bytes());
    assert_eq!(read_exact(&bytes, &f).unwrap(), *race(&f));
    let terminal = complete();
    assert_eq!(
        read_exact(&wire(&terminal), &terminal).unwrap(),
        *race(&terminal)
    );
}

#[test]
fn every_truncated_prefix_reserved_count_enum_and_tail_fails_closed() {
    let f = waiting();
    let good = wire(&f);
    for end in 0..good.len() {
        assert!(read_exact(&good[..end], &f).is_err(), "prefix {end}");
    }
    for (offset, value) in [
        (0, 2),
        (1, 5),
        (2, 4),
        (3, 1),
        (4, 1),
        (5, 1),
        (5, 65),
        (6, 1),
        (6, 11),
        (7, 3),
        (7, 5),
    ] {
        let mut bad = good.clone();
        bad[offset] = value;
        assert!(
            read_exact(&bad, &f).is_err(),
            "offset {offset} value {value}"
        );
    }
    let mut extra = good;
    extra.push(0);
    assert!(read_exact(&extra, &f).is_err());
}

#[test]
fn signed_i64_boundaries_do_not_wrap_into_valid_ids_revisions_or_progress() {
    let f = waiting();
    let good = wire(&f);
    for offset in [40, 48, 56, 144, 152, 160, 168, 176, 184, 192, 200, 208] {
        for value in [i64::MIN, -1, i64::MAX] {
            let mut bad = good.clone();
            bad[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
            assert!(read_exact(&bad, &f).is_err(), "i64 at {offset} = {value}");
        }
    }
}

#[test]
fn v16_world_appendix_is_strict_and_legacy_world_bytes_stay_legacy() {
    let f = waiting();
    let expected = reply(f.clone());
    let bytes = world::tests::wire_complete_version(&expected, 16);
    assert_eq!(
        app::decode_response_version(&bytes, 16).unwrap().response,
        app::Response::World(expected.clone())
    );
    for version in 1..=15 {
        assert!(
            app::decode_response_version(&bytes, version).is_err(),
            "v{version}"
        );
    }
    let mut tail = bytes;
    tail.push(0);
    let len = tail.len() as u32;
    tail[4..8].copy_from_slice(&len.to_be_bytes());
    assert!(app::decode_response_version(&tail, 16).is_err());
    for version in 1..=16 {
        let legacy = reply(crate::modern_lito::world_runtime::tests::frame(9));
        // The shared complete helper starts at v3. v1/v2 have no four-byte
        // completion-presence suffix; construct those explicit old bodies.
        let mut bytes = world::tests::wire_complete_version(&legacy, version.max(3));
        if version < 3 {
            assert_eq!(&bytes[bytes.len() - 4..], &[0; 4]);
            bytes.truncate(bytes.len() - 4);
            let len = bytes.len() as u32;
            bytes[4..8].copy_from_slice(&len.to_be_bytes());
        }
        assert_eq!(
            app::decode_response_version(&bytes, version)
                .unwrap()
                .response,
            app::Response::World(legacy)
        );
    }
}

#[test]
fn maximum_ten_humans_and_sixty_four_gates_roundtrip_without_relaxing_adjacent_limits() {
    let mut frame = waiting();
    let source_members = frame.room.members.clone();
    let source_actors = frame.actors.clone();
    let initial_runner = race(&frame).runners[0].clone();
    frame.room.summary.settings.capacity = 10;
    frame.room.summary.users = 10;
    frame.room.summary.ready = 10;
    frame.room.members = (0..10)
        .map(|i| {
            let mut member = source_members[usize::from(i >= 5)].clone();
            member.key.object_id += i % 5;
            member
        })
        .collect();
    frame.actors = (0..10)
        .map(|i| {
            let mut actor = source_actors[usize::from(i >= 5)].clone();
            actor.entity_id = i as u64 + 1;
            actor.key = frame.room.members[i].key;
            actor.position = WorldCell {
                x: 32720 + i as i32,
                y: 32784,
            };
            actor
        })
        .collect();
    race_mut(&mut frame).runners = (1..=10)
        .map(|entity_id| Runner {
            entity_id,
            ..initial_runner.clone()
        })
        .collect();
    race_mut(&mut frame).gates = (0..64)
        .map(|i| Gate {
            min_x: 32704 + i,
            max_x: 32704 + i,
            min_y: 32780,
            max_y: 32780,
        })
        .collect();
    rehash_course(&mut frame);
    frame.validate(frame.actors[0].key).unwrap();
    let expected = reply(frame);
    let bytes = world::tests::wire_complete_version(&expected, 16);
    assert_eq!(
        app::decode_response_version(&bytes, 16).unwrap().response,
        app::Response::World(expected)
    );
    let start = bytes.len() - (112 + 64 * 16 + 10 * 72);
    for (offset, value) in [(start + 5, 65), (start + 6, 11)] {
        let mut invalid = bytes.clone();
        invalid[offset] = value;
        assert!(app::decode_response_version(&invalid, 16).is_err());
    }
}

#[test]
fn capability_does_not_authorize_a_race_without_its_exact_match_descriptor() {
    let response = app::Response::World(reply(waiting()));
    let all = p::Ack {
        request_id: 1,
        version: 16,
        numeric_schema: 1,
        capabilities: p::KNOWN_CAPABILITIES,
    };
    assert!(world::descriptor_negotiated(&response, &all));
    for ack in [
        p::Ack { version: 15, ..all },
        p::Ack {
            capabilities: all.capabilities & !p::CAP_PET_RACE,
            ..all
        },
        p::Ack {
            capabilities: all.capabilities & !p::CAP_MATCH_DESCRIPTOR,
            ..all
        },
    ] {
        assert!(!world::descriptor_negotiated(&response, &ack));
    }
    assert_bad(waiting(), |f| f.descriptor = None);
    assert_bad(waiting(), |f| f.race = None);
    assert_bad(waiting(), |f| {
        f.descriptor.as_mut().unwrap().features |= MatchDescriptor::COMBAT
    });
    let mut ordinary = crate::modern_lito::world_runtime::tests::frame(9);
    ordinary.descriptor = Some(MatchDescriptor {
        ruleset: MatchRuleset::SceneOnly,
        features: 0,
        ..waiting().descriptor.unwrap()
    });
    assert!(world::descriptor_negotiated(
        &app::Response::World(reply(ordinary)),
        &all
    ));
}

#[test]
fn waiting_loading_countdown_running_and_abort_have_exact_phase_shapes() {
    for phase in [Phase::Preparing, Phase::Loading, Phase::Countdown] {
        let mut f = waiting();
        f.phase = phase;
        for a in &mut f.actors {
            a.loaded = phase == Phase::Countdown;
        }
        f.validate(f.actors[0].key).unwrap();
    }
    for wrong in [
        RaceState::Running,
        RaceState::FinishWindow,
        RaceState::Complete,
        RaceState::Aborted,
    ] {
        assert_bad(waiting(), |f| race_mut(f).state = wrong);
    }
    assert_bad(waiting(), |f| f.actors[0].sequence = 1);
    let mut aborted = running();
    aborted.phase = Phase::Aborted;
    aborted.abort_reason = world::AbortReason::ActiveTimeout;
    aborted.revision = 3;
    race_mut(&mut aborted).state = RaceState::Aborted;
    race_mut(&mut aborted).terminal_revision = 3;
    aborted.validate(aborted.actors[0].key).unwrap();
    assert!(follows_world(&running(), &aborted));
    assert!(follows_world(&aborted, &aborted));
    assert!(!follows_world(&aborted, &running()));
    assert_bad(aborted.clone(), |f| {
        race_mut(f).settlement = Settlement::Saved
    });
    assert_bad(aborted, |f| race_mut(f).result_id = [1; 16]);
}

#[test]
fn current_progress_revision_requires_exact_move_sequence_and_last_gate_position() {
    let (_, after) = first_gate();
    assert_bad(after.clone(), |f| f.actors[0].position.x = 32725);
    assert_bad(after.clone(), |f| f.actors[0].sequence += 1);
    let mut later = ordinary_move(&after, WorldCell { x: 32727, y: 32784 });
    later.actors[0].position.y = 32785;
    later.validate(later.actors[0].key).unwrap();
    assert!(follows_world(&after, &later));
}

#[test]
fn ranks_finish_times_and_window_cannot_fabricate_a_complete_result() {
    assert_bad(finish_window(), |f| {
        race_mut(f).first_finisher_entity_id = 2
    });
    assert_bad(finish_window(), |f| {
        race_mut(f).finish_window_deadline_elapsed_nanos += 1
    });
    assert_bad(finish_window(), |f| race_mut(f).runners[0].placement = 2);
    assert_bad(finish_window(), |f| {
        race_mut(f).runners[0].completed_laps = 3
    });
    assert_bad(finish_window(), |f| {
        race_mut(f).runners[0].dnf = DnfReason::Left
    });
    assert_bad(finish_window(), |f| {
        race_mut(f).runners[0].finish_elapsed_nanos = 60_000_000_000
    });
    assert_bad(complete(), |f| {
        race_mut(f).runners[1].dnf = DnfReason::None;
        race_mut(f).runners[1].revision = 0;
        rehash_result(f);
    });
    assert_bad(complete(), |f| {
        race_mut(f).runners[1].revision -= 1;
        rehash_result(f);
    });
    let mut second = finish_window();
    second.revision = 12;
    second.actors[1].sequence = 26;
    second.actors[1].position = WorldCell { x: 32728, y: 32784 };
    let r = &mut race_mut(&mut second).runners[1];
    r.revision = 12;
    r.progress_move_sequence = 26;
    r.completed_laps = 4;
    r.placement = 2;
    r.finish_elapsed_nanos = 2_000_000_000;
    second.validate(second.actors[0].key).unwrap();
    assert_bad(second.clone(), |f| race_mut(f).runners[1].placement = 1);
    assert_bad(second.clone(), |f| {
        race_mut(f).runners[1].finish_elapsed_nanos = 999_999_999
    });
    assert_bad(second, |f| {
        race_mut(f).runners[1].finish_elapsed_nanos = 11_000_000_001
    });
}

#[test]
fn complete_receipt_has_exact_terminal_revision_and_a_single_settlement_transition() {
    let pending = complete();
    assert_bad(pending.clone(), |f| f.revision += 1);
    assert_bad(pending.clone(), |f| {
        race_mut(f).settlement = Settlement::Saved
    });
    for settlement in [Settlement::Saved, Settlement::Failed] {
        let mut next = pending.clone();
        next.revision += 1;
        race_mut(&mut next).settlement = settlement;
        next.validate(next.actors[0].key).unwrap();
        assert!(follows_world(&pending, &next));
        assert!(follows_world(&next, &next));
        assert!(!follows_world(&next, &pending));
        assert_bad(next.clone(), |f| f.revision += 1);
        let mut changed = next.clone();
        changed.actors[0].sequence += 1;
        assert!(!follows_world(&pending, &changed));
        let mut changed = next;
        race_mut(&mut changed).result_id[0] ^= 1;
        rehash_result(&mut changed);
        changed.validate(changed.actors[0].key).unwrap();
        assert!(!follows_world(&pending, &changed));
    }
}

#[test]
fn hashes_bind_all_source_course_form_interval_and_result_identity_fields() {
    assert_bad(waiting(), |f| f.manifest.digest[0] ^= 1);
    assert_bad(waiting(), |f| f.manifest.sectors[0].sha256[0] ^= 1);
    assert_bad(waiting(), |f| f.room.summary.settings.map_id = 566);
    assert_bad(waiting(), |f| race_mut(f).gates.swap(0, 1));
    assert_bad(waiting(), |f| race_mut(f).runners[0].base_graphics_id = 945);
    assert_bad(waiting(), |f| race_mut(f).runners[0].base_graphics_id = 938);
    assert_bad(waiting(), |f| {
        for a in &mut f.actors {
            a.step_interval_nanos += 1;
        }
    });
    assert_bad(complete(), |f| {
        f.actors[1].key.source_id = 43;
        f.room.members[1].key.source_id = 43;
    });
    assert_bad(complete(), |f| f.match_id += 1);
    assert_bad(complete(), |f| {
        f.generation += 1;
        f.room.summary.revision += 1;
    });
    let mut changed = running();
    for r in &mut race_mut(&mut changed).runners {
        r.base_graphics_id = 938;
    }
    rehash_course(&mut changed);
    changed.validate(changed.actors[0].key).unwrap();
    assert!(!follows_world(&running(), &changed));
}

#[test]
fn coalesced_moves_preserve_progress_without_requiring_the_latest_position_in_an_old_gate() {
    let a = running();
    let mut b = a.clone();
    b.revision = 9;
    b.actors[0].sequence = 7;
    b.actors[0].position = WorldCell { x: 32727, y: 32785 };
    let r = &mut race_mut(&mut b).runners[0];
    r.revision = 8;
    r.progress_move_sequence = 6;
    r.completed_laps = 1;
    b.validate(b.actors[0].key).unwrap();
    assert!(follows_world(&a, &b));
    assert!(!follows_world(&b, &a));
    let mut impossible = b.clone();
    impossible.actors[0].sequence = 1;
    race_mut(&mut impossible).runners[0].progress_move_sequence = 1;
    assert!(!follows_world(&a, &impossible));
    let mut same_revision = b.clone();
    same_revision.revision = a.revision;
    assert!(!follows_world(&a, &same_revision));
}

#[test]
fn an_individual_dnf_retires_only_that_racer_and_does_not_create_a_result() {
    let before = running();
    for reason in [DnfReason::Left, DnfReason::Disconnected] {
        let mut after = before.clone();
        after.revision += 1;
        let revision = after.revision;
        race_mut(&mut after).runners[1].revision = revision;
        race_mut(&mut after).runners[1].dnf = reason;
        after.validate(after.actors[0].key).unwrap();
        assert!(follows_world(&before, &after));
        assert!(after.actor_occupies(1) && !after.actor_occupies(2));
        assert!(after.completion.is_none());
        assert_eq!(race(&after).settlement, Settlement::None);
        let mut ghost = after.clone();
        ghost.revision += 1;
        ghost.actors[1].sequence += 1;
        ghost.actors[1].position.x += 1;
        ghost.validate(ghost.actors[0].key).unwrap();
        assert!(!follows_world(&after, &ghost));
        let mut resurrected = after.clone();
        resurrected.revision += 1;
        race_mut(&mut resurrected).runners[1] = race(&before).runners[1].clone();
        resurrected.validate(resurrected.actors[0].key).unwrap();
        assert!(!follows_world(&after, &resurrected));
        let mut overlap = after.clone();
        overlap.actors[1].position = overlap.actors[0].position;
        overlap.validate(overlap.actors[0].key).unwrap();
    }
    assert_bad(before.clone(), |f| {
        f.actors[1].position = f.actors[0].position
    });
    let old = reply(before.clone());
    let mut leave = old.clone();
    leave.echo.action = world::Action::Leave;
    leave.echo.match_id = before.match_id;
    leave.echo.generation = before.generation;
    leave.frame = None;
    leave.validate().unwrap();
    assert!(world::follows(Some(&old), &leave));
    leave.echo.action = world::Action::Read;
    assert!(!world::follows(Some(&old), &leave));
}

#[test]
fn inactive_runner_cannot_gain_a_sequence_without_moving() {
    let finished = finish_window();
    let mut inactive = vec![(finished, 0usize)];
    for reason in [DnfReason::Left, DnfReason::Disconnected] {
        let mut frame = running();
        frame.revision += 1;
        let revision = frame.revision;
        let runner = &mut race_mut(&mut frame).runners[1];
        runner.revision = revision;
        runner.dnf = reason;
        inactive.push((frame, 1));
    }
    for (old, index) in inactive {
        old.validate(old.actors[0].key).unwrap();
        let mut new = old.clone();
        new.revision += 1;
        new.actors[index].sequence += 1;
        new.validate(new.actors[0].key).unwrap();
        assert!(
            !follows_world(&old, &new),
            "inactive runner index {index} gained an ACK sequence"
        );
    }
}

#[test]
fn inactive_move_denial_preserves_the_exact_frame_and_remains_a_valid_reply() {
    let loading = waiting();
    let mut countdown = loading.clone();
    countdown.phase = Phase::Countdown;
    countdown
        .actors
        .iter_mut()
        .for_each(|actor| actor.loaded = true);
    let mut examples = vec![(loading, 0usize), (countdown, 0), (finish_window(), 0)];
    for reason in [DnfReason::Left, DnfReason::Disconnected] {
        let mut frame = running();
        frame.revision += 1;
        let revision = frame.revision;
        race_mut(&mut frame).runners[1].revision = revision;
        race_mut(&mut frame).runners[1].dnf = reason;
        examples.push((frame, 1));
    }
    for (frame, index) in examples {
        let mut old = reply(frame.clone());
        old.echo.context.key = frame.actors[index].key;
        old.validate().unwrap();
        let mut denial = old.clone();
        denial.echo.action = world::Action::Move;
        denial.echo.match_id = frame.match_id;
        denial.echo.generation = frame.generation;
        denial.echo.sequence = frame.actors[index].sequence + 1;
        denial.echo.from_x = frame.actors[index].position.x;
        denial.echo.from_y = frame.actors[index].position.y;
        denial.echo.heading = 0;
        denial.status = world::Status::Denied;
        denial.reason = world::Reason::NotActive;
        denial.validate().unwrap();
        assert!(denial.confirms(&denial.echo));
        assert!(world::follows(Some(&old), &denial));
        assert_eq!(denial.frame, old.frame);
    }
}

#[test]
fn exact_single_move_must_enter_the_expected_gate_not_move_within_it() {
    let (before, after) = first_gate();
    assert!(follows_world(&before, &after));
    let mut inside = before.clone();
    inside.actors[0].position.x = 32726;
    inside.validate(inside.actors[0].key).unwrap();
    let mut false_crossing = after.clone();
    false_crossing.actors[0].position.x = 32727;
    false_crossing
        .validate(false_crossing.actors[0].key)
        .unwrap();
    assert!(!follows_world(&inside, &false_crossing));
}

#[test]
fn a_runner_revision_cannot_advance_without_progress_or_a_new_dnf() {
    let (_, before) = first_gate();
    let mut after = before.clone();
    after.revision += 1;
    let revision = after.revision;
    race_mut(&mut after).runners[0].revision = revision;
    after.validate(after.actors[0].key).unwrap();
    assert!(!follows_world(&before, &after));
}

#[test]
fn public_follows_rejects_unvalidated_overflow_and_incomplete_rosters() {
    let valid = running();
    let mut bad = valid.clone();
    race_mut(&mut bad).runners[0].completed_laps = i64::MAX;
    assert!(!super::follows(&bad, &valid));
    assert!(!super::follows(&valid, &bad));
    let mut bad = valid.clone();
    bad.actors.clear();
    assert!(!super::follows(&bad, &valid));
    assert!(!super::follows(&valid, &bad));
    let mut bad = valid.clone();
    race_mut(&mut bad).runners.clear();
    assert!(!super::follows(&bad, &valid));
    assert!(!super::follows(&valid, &bad));
}
