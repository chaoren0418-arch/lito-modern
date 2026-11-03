use super::*;
use crate::modern_lito::{
    application_protocol as app, battle_follow::Follower, battle_read, protocol,
};

fn observation(data: d::UnvalidatedData) -> Observation {
    let request = battle_read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: data.personal.player.source as u32,
                object_id: data.personal.player.object as u64,
            },
            epoch: data.personal.player.epoch as u64,
        },
        channel_id: data.personal.player.channel as u8,
        world: data.world.identity.clone(),
    };
    let mut f = Follower::new(
        request.clone(),
        protocol::Ack {
            request_id: 1,
            version: 35,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
    )
    .unwrap();
    f.accept(battle_read::UnvalidatedReply {
        echo: request,
        state: battle_read::State::Data,
        data: Some(data),
    })
    .unwrap();
    f.observation().unwrap().clone()
}
fn initial() -> d::UnvalidatedData {
    let mut data = crate::modern_lito::battle_payload::data_tests::fixture();
    data.world.revision = 2;
    data.world.phase = d::Phase::Active;
    data.world.loading_elapsed_nanos = 100_000_000;
    data.world.active_elapsed_nanos = 0;
    for a in &mut data.actors.0 {
        a.loaded = true;
        a.step_interval_nanos = 200_000_000;
    }
    data
}
fn return_proof(action: BaseAction) -> finish::QualifiedReturn {
    use crate::modern_lito::{
        combat_animation::QualifiedCombatAnimation, sprite_actions::SpriteActionCatalog,
        world_scene::BaseAppearance,
    };
    let catalog = SpriteActionCatalog::parse_for_inspection(
        b"#0 64 fixture\n1.attack(1 3,0.0:8 0.1:2 0.2:4)\n2.damage(1 3,0.0:9 0.1:1 0.2:2)\n8.death(1 3,0.0:7 0.1:2 0.2:4)\n",
        Default::default()).unwrap();
    let q = QualifiedCombatAnimation::from_catalog(
        &catalog,
        BaseAppearance { class: 0, sex: 0 },
        action.combat().unwrap(),
    )
    .unwrap();
    finish::QualifiedReturn::base(action, &q).unwrap()
}
#[test]
fn qualified_body_return_keeps_facing_and_only_new_confirmed_actions_cancel_it() {
    let now = Instant::now();
    let actor = initial().actors.0.remove(0);
    let mut body = Body::initial(&actor, now);
    body.start(BaseAction::Attack, now, Some(5));
    body.cue_revision = Some(7);
    body.idle_return = Some(
        finish::IdleReturn::start(return_proof(BaseAction::Attack), &body.clock, now).unwrap(),
    );
    // Six active ticks at24FPS=250ms, plus584ms; source0's8 ticks do not count.
    let end = now + Duration::from_millis(834);
    let mut before = body.clone();
    before.settle(end - Duration::from_nanos(1)).unwrap();
    assert_eq!(before.action, BaseAction::Attack);
    assert!(before.cue_revision.is_some());
    let mut after = body.clone();
    after.settle(end + Duration::from_millis(10)).unwrap();
    assert_eq!(after.action, BaseAction::Idle);
    assert_eq!(after.heading, Some(5));
    assert_eq!(
        after.clock.peek(end + Duration::from_millis(10)).unwrap(),
        Duration::from_millis(10)
    );
    assert!(after.cue_revision.is_none());
    assert!(after.idle_return.is_none());
    for action in [
        BaseAction::Walk,
        BaseAction::Attack,
        BaseAction::Damage,
        BaseAction::Death,
        BaseAction::Idle,
    ] {
        let mut changed = body.clone();
        changed.start(action, now + Duration::from_millis(500), None);
        assert!(changed.idle_return.is_none());
        changed.settle(end).unwrap();
        assert_eq!(
            changed.action, action,
            "obsolete idle tail overwrote {action:?}"
        );
    }
    assert!(Body::initial(&actor, now).idle_return.is_none());
}
#[test]
fn player_return_certificate_is_atomic_and_rejects_death_stale_owner_or_changed_timing() {
    let now = Instant::now();
    let cancel = AtomicBool::new(false);
    let mut timeline = Timeline::new(observation(initial()), now).unwrap();
    let body = timeline.bodies.get_mut(&1).unwrap();
    body.start(BaseAction::Attack, now, Some(2));
    body.cue_revision = Some(3);
    let proof = return_proof(BaseAction::Attack);
    let proofs = BTreeMap::from([(1, proof)]);
    let attached = timeline.with_player_returns(&proofs, now, &cancel).unwrap();
    assert!(timeline.bodies[&1].idle_return.is_none());
    let end = now + Duration::from_millis(834);
    assert_eq!(
        attached
            .pose(1, end - Duration::from_nanos(1))
            .unwrap()
            .action,
        BaseAction::Attack
    );
    assert_eq!(attached.pose(1, end).unwrap().action, BaseAction::Idle);
    assert!(attached.player_audio(1, end).unwrap().is_none());
    assert!(attached
        .player_audio(1, end - Duration::from_nanos(1))
        .unwrap()
        .is_some());
    assert!(timeline
        .with_player_returns(&proofs, now, &AtomicBool::new(true))
        .is_err());
    assert!(timeline
        .with_player_returns(&proofs, now + Duration::from_nanos(1), &cancel)
        .is_err());
    assert!(timeline
        .with_player_returns(&BTreeMap::from([(999, proof)]), now, &cancel)
        .is_err());
    assert!(attached
        .with_player_returns(
            &BTreeMap::from([(1, return_proof(BaseAction::Damage))]),
            now,
            &cancel
        )
        .is_err());
    let mut dead = timeline.clone();
    dead.bodies
        .get_mut(&1)
        .unwrap()
        .start(BaseAction::Death, now, None);
    assert!(dead.with_player_returns(&proofs, now, &cancel).is_err());
    assert_eq!(
        dead.pose(1, end + Duration::from_secs(10)).unwrap().action,
        BaseAction::Death
    );
    assert_eq!(
        attached
            .with_player_returns(&proofs, now, &cancel)
            .unwrap()
            .pose(1, end)
            .unwrap()
            .action,
        BaseAction::Idle
    );
}
#[test]
fn walk_origin_requires_one_confirmed_adjacent_sequence_and_correct_heading() {
    let before = initial().actors.0[0].clone();
    for (heading, (dx, dy)) in crate::modern_lito::world_map::DIRECTIONS
        .into_iter()
        .enumerate()
    {
        let mut after = before.clone();
        after.position.x += dx;
        after.position.y += dy;
        after.heading = heading as i32;
        after.last_move_sequence += 1;
        assert!(single_step(&before, &after));
        after.last_move_sequence += 1;
        assert!(!single_step(&before, &after));
        after.last_move_sequence -= 1;
        after.heading = ((heading + 1) % 8) as i32;
        assert!(!single_step(&before, &after));
        after.heading = heading as i32;
        after.position.x += dx;
        after.position.y += dy;
        assert!(!single_step(&before, &after));
    }
    assert!(!single_step(&before, &before));
    let now = Instant::now();
    let mut body = Body::initial(&before, now);
    body.walk_from = Some(crate::modern_lito::world_map::WorldCell { x: 1, y: 1 });
    for action in [
        BaseAction::Idle,
        BaseAction::Attack,
        BaseAction::Damage,
        BaseAction::Death,
    ] {
        body.start(action, now, None);
        assert!(body.walk_from.is_none());
    }
}
#[test]
fn observed_denied_sequence_then_confirmed_move_keeps_single_step_without_inventing_a_path() {
    let t = Instant::now();
    let cancel = AtomicBool::new(false);
    let mut data = initial();
    let old = Timeline::new(observation(data.clone()), t).unwrap();
    data.world.revision += 1;
    data.actors.0[0].last_move_sequence = 1;
    let denied = old
        .advance(
            observation(data.clone()),
            t + Duration::from_millis(10),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        denied
            .pose(1, t + Duration::from_millis(10))
            .unwrap()
            .action,
        BaseAction::Idle
    );
    data.world.revision += 1;
    data.actors.0[0].last_move_sequence = 2;
    data.actors.0[0].position.x += 1;
    data.actors.0[0].heading = 2;
    let moved = observation(data);
    let skipped = old
        .advance(moved.clone(), t + Duration::from_millis(20), &cancel)
        .unwrap();
    assert_eq!(
        skipped
            .pose(1, t + Duration::from_millis(20))
            .unwrap()
            .action,
        BaseAction::Idle
    );
    assert_eq!(
        skipped
            .pose(1, t + Duration::from_millis(20))
            .unwrap()
            .walk_from,
        None
    );
    let continuous = denied
        .advance(moved, t + Duration::from_millis(20), &cancel)
        .unwrap();
    let pose = continuous.pose(1, t + Duration::from_millis(70)).unwrap();
    assert_eq!(pose.action, BaseAction::Walk);
    assert!(pose.walk_from.is_some());
    assert_eq!(pose.elapsed, Duration::from_millis(50));
}
#[test]
fn confirmed_move_plays_one_step_and_repeated_or_clock_only_data_does_not_restart_it() {
    let t = Instant::now();
    let cancel = AtomicBool::new(false);
    let data = initial();
    let old = Timeline::new(observation(data.clone()), t).unwrap();
    let mut moved = data.clone();
    moved.world.revision += 1;
    moved.actors.0[0].last_move_sequence = 1;
    moved.actors.0[0].position.x += 1;
    moved.actors.0[0].heading = 2;
    let next = observation(moved.clone());
    let motion = old
        .advance(next.clone(), t + Duration::from_millis(10), &cancel)
        .unwrap();
    assert_eq!(
        old.pose(1, t + Duration::from_millis(60)).unwrap().action,
        BaseAction::Idle
    );
    let pose = motion.pose(1, t + Duration::from_millis(60)).unwrap();
    assert_eq!(pose.action, BaseAction::Walk);
    assert_eq!(pose.elapsed, Duration::from_millis(50));
    assert_eq!(
        pose.walk_from,
        Some(crate::modern_lito::world_map::WorldCell {
            x: data.actors.0[0].position.x,
            y: data.actors.0[0].position.y,
        })
    );
    let repeated = motion
        .advance(next, t + Duration::from_millis(80), &cancel)
        .unwrap();
    moved.world.loading_elapsed_nanos += 5;
    moved.world.active_elapsed_nanos += 5;
    let clock = repeated
        .advance(observation(moved), t + Duration::from_millis(90), &cancel)
        .unwrap();
    assert_eq!(
        clock
            .pose(1, t + Duration::from_millis(110))
            .unwrap()
            .elapsed,
        Duration::from_millis(100)
    );
    let done = clock.pose(1, t + Duration::from_millis(230)).unwrap();
    assert_eq!(done.action, BaseAction::Idle);
    assert_eq!(done.elapsed, Duration::from_millis(20));
    assert_eq!(done.walk_from, None);
}
#[test]
fn missed_moves_do_not_queue_catchup_and_rejected_updates_leave_old_clocks_intact() {
    let t = Instant::now();
    let data = initial();
    let obs = observation(data.clone());
    let old = Timeline::new(obs.clone(), t).unwrap();
    let mut changed = data.clone();
    changed.world.revision += 7;
    changed.actors.0[0].last_move_sequence = 5;
    changed.actors.0[0].position.y += 2;
    let moved = observation(changed);
    assert!(old
        .advance(moved.clone(), t, &AtomicBool::new(true))
        .is_err());
    assert!(old
        .advance(
            moved.clone(),
            t - Duration::from_nanos(1),
            &AtomicBool::new(false)
        )
        .is_err());
    let new = old
        .advance(moved, t + Duration::from_secs(1), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        new.pose(1, t + Duration::from_secs(1)).unwrap().action,
        BaseAction::Idle
    );
    assert_eq!(
        new.pose(1, t + Duration::from_secs(1)).unwrap().walk_from,
        None
    );
    assert_eq!(
        new.pose(1, t + Duration::from_millis(1250)).unwrap().action,
        BaseAction::Idle
    );
    assert!(new
        .advance(obs, t + Duration::from_secs(2), &AtomicBool::new(false))
        .is_err());
    assert_eq!(
        old.pose(1, t + Duration::from_secs(3)).unwrap().action,
        BaseAction::Idle
    );
    let mut aborted = data;
    aborted.world.phase = d::Phase::Aborted;
    aborted.world.abort = d::Abort::ServerAbort;
    assert!(Timeline::new(observation(aborted), t).is_err());
}
#[test]
fn canonical_buff_pace_matches_existing_native_staged_rounding_without_legacy_frame() {
    use crate::modern_lito::timed_protocol::{Pace, State};
    let mut actor = initial().actors.0[0].clone();
    for base in [
        1_000_000,
        199_999_999,
        200_000_000,
        777_000_000,
        10_000_000_000,
    ] {
        for haste in [false, true] {
            for brave_kind in [0, 1, 3] {
                let state = State {
                    entity_id: 1,
                    revision: 1,
                    observed_nanos: 10,
                    haste_until_nanos: if haste { 20 } else { 0 },
                    brave_until_nanos: if brave_kind > 0 { 20 } else { 0 },
                    wisdom_until_nanos: 0,
                    brave_kind,
                };
                actor.step_interval_nanos = base as i64;
                actor.buffs = Some(d::Buffs {
                    entity: 1,
                    revision: 1,
                    observed_nanos: state.observed_nanos,
                    haste_until_nanos: state.haste_until_nanos,
                    brave_until_nanos: state.brave_until_nanos,
                    wisdom_until_nanos: 0,
                    brave_kind: brave_kind as i32,
                    slow_until_nanos: 0,
                    slow_revision: 0,
                    haste_origin: if state.haste_until_nanos == 0 {
                        d::HasteOrigin::None
                    } else {
                        d::HasteOrigin::Potion
                    },
                });
                assert_eq!(
                    pace(&actor, BaseAction::Walk),
                    (base, state.effective_nanos(base, Pace::Move))
                );
                assert_eq!(pace(&actor, BaseAction::Idle), (1, 1));
            }
        }
    }
}
#[test]
fn historical_corpse_is_terminal_and_old_physical_target_is_not_a_player_alias() {
    let t = Instant::now();
    let mut data = initial();
    let actor = &mut data.actors.0[0];
    actor.vital = Some(d::Vital {
        values: d::Vitals {
            hp: 0,
            maximum_hp: 100,
            mp: 0,
            maximum_mp: 0,
        },
        ac: 0,
        weapon_type: 0,
        attack_sequence: 1,
        attack_revision: 1,
        hurt_revision: 2,
        death_revision: 2,
        target: 900,
        attack_interval_nanos: 200_000_000,
    });
    // Pure body/target helpers; protocol-admitted combat histories are covered
    // by the actual Java trace below, not claimed by this isolated fixture.
    let corpse = Body::initial(actor, t);
    assert!(corpse.historical_death);
    assert_eq!(corpse.action, BaseAction::Death);
    let target = attack(&data, &data.actors.0[0], &data.room.members.0[0]);
    assert_eq!(target.revision, 1);
    assert!(target.cell.is_none());
    assert_eq!(
        heading(&d::Cell { x: 1, y: 1 }, &d::Cell { x: 1, y: 1 }),
        None
    );
}

#[test]
fn qualified_walk_cycle_fits_short_steps_and_preserves_exact_native_tick_boundaries() {
    for (millis, tick) in [(0, 0), (50, 4), (100, 8), (150, 12)] {
        let phase = walk_elapsed(Duration::from_millis(millis), 200_000_000, 16, 24).unwrap();
        assert_eq!(phase.as_nanos() * 24 / 1_000_000_000, tick);
    }
    assert!(walk_elapsed(Duration::ZERO, 0, 16, 24).is_err());
    assert!(walk_elapsed(Duration::ZERO, 200_000_000, 0, 24).is_err());
    assert!(walk_elapsed(Duration::ZERO, 200_000_000, 16, 0).is_err());
    assert_eq!(
        walk_elapsed(Duration::MAX, 200_000_000, 16, 24).unwrap(),
        walk_elapsed(Duration::from_millis(200), 200_000_000, 16, 24).unwrap()
    );
}

/// Called for every genuine ordered Java pair, including missed transitions.
/// This checks body decisions against admitted data, not fake wire fixtures.
pub(crate) fn verify_pair(old: Observation, new: Observation) -> [usize; 8] {
    let mut counts = [0; 8];
    if old.data().world.phase == d::Phase::Aborted || new.data().world.phase == d::Phase::Aborted {
        return counts;
    }
    let t = Instant::now();
    let c = AtomicBool::new(false);
    let initial = Timeline::new(old.clone(), t).unwrap();
    for a in &old.data().actors.0 {
        let p = initial.pose(a.entity, t).unwrap();
        assert_eq!(
            p.action,
            if dead(a) {
                BaseAction::Death
            } else {
                BaseAction::Idle
            }
        );
        assert_eq!(
            p.elapsed,
            if dead(a) {
                Duration::MAX
            } else {
                Duration::ZERO
            }
        );
    }
    let at = t + Duration::from_millis(100);
    let next = initial
        .advance(new.clone(), at, &c)
        .expect("real canonical motion pair");
    let repeated = next
        .advance(new.clone(), at + Duration::from_millis(1), &c)
        .unwrap();
    let future = at + Duration::from_millis(50);
    for (a, b) in old.data().actors.0.iter().zip(&new.data().actors.0) {
        match next.pose(b.entity, at).unwrap().action {
            BaseAction::Attack => counts[0] += 1,
            BaseAction::Damage => counts[1] += 1,
            BaseAction::Death => counts[2] += 1,
            BaseAction::Walk => counts[3] += 1,
            BaseAction::Idle => {}
        }
        let p = next.pose(b.entity, future).unwrap();
        let q = repeated.pose(b.entity, future).unwrap();
        assert_eq!(p.action, q.action, "poll must not restart pose");
        assert!(
            p.elapsed.as_nanos().abs_diff(q.elapsed.as_nanos()) <= 1,
            "poll must not restart body time"
        );
        if dead(b) {
            assert_eq!(p.action, BaseAction::Death);
        }
        if !dead(b) && (life(a) != life(b) || weapon(a) != weapon(b)) {
            assert_eq!(
                p.action,
                BaseAction::Idle,
                "do not play previous life/weapon's retained attack"
            );
        }
    }
    if let Some(n) = &new.data().npcs {
        for a in &n.actors.0 {
            let (pose, elapsed) = next.npc_pose(a.serial, at).unwrap();
            match pose {
                NpcAction::Attack => counts[4] += 1,
                NpcAction::Damage => counts[5] += 1,
                NpcAction::Death => counts[6] += 1,
                NpcAction::Walk => counts[7] += 1,
                NpcAction::Breath => {}
            }
            if a.hp == 0 {
                assert_eq!(pose, NpcAction::Death);
            }
            let again = repeated
                .npc_pose(a.serial, at + Duration::from_millis(1))
                .unwrap();
            assert_eq!(again.0, pose);
            assert!(again.1 >= elapsed);
        }
    }
    if let Some(n) = &old.data().npcs {
        for a in &n.actors.0 {
            if new
                .data()
                .npcs
                .as_ref()
                .is_none_or(|n| n.actors.0.iter().all(|v| v.serial != a.serial))
            {
                assert!(
                    next.npc_pose(a.serial, future).is_err(),
                    "retired serial removed"
                );
            }
        }
    }
    counts
}
#[test]
fn slow_paces_walk_attack_and_cast_with_native_staged_brave_rounding() {
    let mut actor = initial().actors.0[0].clone();
    actor.buffs = Some(d::Buffs {
        entity: 1,
        revision: 1,
        observed_nanos: 10,
        haste_until_nanos: 0,
        brave_until_nanos: 0,
        wisdom_until_nanos: 0,
        brave_kind: 0,
        haste_origin: d::HasteOrigin::None,
        slow_until_nanos: 20,
        slow_revision: 1,
    });
    for base in [1, 199, 200, 700, 777, 10_000] {
        for kind in [0, 1, 3] {
            actor.buffs.as_mut().unwrap().brave_kind = kind;
            actor.buffs.as_mut().unwrap().brave_until_nanos = if kind == 0 { 0 } else { 20 };
            for attack in [false, true] {
                let mut expected = base * 2;
                if kind != 0 {
                    expected = (expected as f64 * 0.755) as u64;
                }
                if kind == 3 {
                    expected = (expected as f64 * 0.855) as u64;
                    if attack {
                        expected = (expected as f64 * 0.9) as u64;
                    }
                }
                assert_eq!(
                    paced_nanos(&actor, base * 1_000_000, attack),
                    expected.max(1) * 1_000_000
                );
            }
        }
    }
    let b = actor.buffs.as_mut().unwrap();
    b.brave_until_nanos = 0;
    b.brave_kind = 0;
    b.observed_nanos = 20;
    assert_eq!(
        paced_nanos(&actor, 700_000_000, false),
        700_000_000,
        "expired slow must not stay visually active"
    );
}
pub(crate) fn verify_speed_npc_pacing(data: &d::UnvalidatedData) {
    let Some(roster) = data.npcs.as_ref() else {
        return;
    };
    let n = data.speed.as_ref().unwrap().state.npcs.as_ref().unwrap();
    for a in &roster.actors.0 {
        let m = n
            .actors
            .0
            .iter()
            .find(|m| m.id.serial == a.serial && m.life == a.life)
            .unwrap();
        let (pace, elapsed) = npc_pace(data, a)
            .unwrap()
            .expect("schema6 NPC must retain its motion clock");
        let expected = if m.speed.slow_until_millis != 0 {
            m.attack_millis * 2
        } else if m.speed.haste_until_millis != 0 {
            ((m.attack_millis as f64 * 0.755) as i32).max(1)
        } else {
            m.attack_millis
        };
        assert_eq!(pace.attack_millis, expected);
        assert_eq!(elapsed, n.elapsed_millis);
        assert_eq!(
            pace.last_walk,
            m.last_walk.as_ref(),
            "current Slow cannot rewrite a committed walk"
        );
    }
}
