//! Pure state and owned-worker tests. Synthetic frames below stand for inputs
//! already confirmed by the separate TLS integration tests; no network/IO/UI.
use super::*;

#[test]
fn siege_mouse_routes_keep_dead_tower_bases_blocked_and_retire_on_gate_or_life_change() {
    use crate::modern_lito::siege_protocol::tests::sample;
    let frame = sample("active").frame.unwrap();
    let s = &frame.world.manifest.sectors[0];
    let map = Arc::new(
        WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: SharedMapSectorKey {
                    map_id: frame.world.manifest.map_id,
                    sector_x: s.x,
                    sector_y: s.y,
                },
                bytes: vec![0; 49170],
            }],
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    let d = frame.world.siege.as_ref().unwrap();
    assert_eq!(combat_objectives(&frame).len(), 67);
    let opened = sample("right").frame.unwrap();
    assert_eq!(combat_objectives(&opened).len(), 3);
    for t in &d.objectives {
        assert!(combat_objectives(&opened).contains(&t.cell));
    }
    let mut nav = Navigation::default();
    let from = frame.world.actors[0].position;
    let goal = WorldCell {
        x: from.x + 1,
        y: from.y + 1,
    };
    assert!(nav
        .request(map.clone(), Arc::new(frame.world.clone()), 1, goal)
        .is_err());
    nav.request_combat(map.clone(), Arc::new(frame.clone()), 1, goal)
        .unwrap();
    poll_until(&mut nav, |n| n.status() == Status::Walking);
    assert!(nav.next_step_combat(&frame, 1, Instant::now()).is_some());
    assert!(nav.next_step_combat(&opened, 1, Instant::now()).is_none());
    assert!(nav.route.is_none());
    let before = sample("lifeactiveplain").frame.unwrap();
    let alive = sample("respawnplain").frame.unwrap();
    let from = before.world.actors[1].position;
    let goal = WorldCell {
        x: from.x + 1,
        y: from.y + 1,
    };
    nav.request_combat(map, Arc::new(before), 2, goal).unwrap();
    poll_until(&mut nav, |n| n.status() == Status::Walking);
    assert!(nav.next_step_combat(&alive, 2, Instant::now()).is_none());
    assert!(nav.route.is_none());
}

#[test]
fn time_attack_dynamic_occupancy_uses_live_tower_and_excludes_departed_players() {
    use crate::modern_lito::time_attack_protocol::tests::sample;
    let live = sample("waiting").frame.unwrap();
    let dead = sample("tower").frame.unwrap();
    let tower = live.combat.world.time_attack.as_ref().unwrap().tower_cell;
    assert_eq!(encounter_occupied(&live), BTreeSet::from([tower]));
    assert!(!encounter_occupied(&dead).contains(&tower));
    let departed = sample("departed").frame.unwrap();
    assert!(occupied(&departed.combat.world, 2).is_empty());
    assert!(!departed.combat.world.actor_occupies(1));
    assert!(departed.combat.world.actor_occupies(2));
}

#[test]
fn quest_dynamic_occupancy_blocks_only_live_objectives_and_never_guesses_enemy_cells() {
    let before = crate::modern_lito::quest_protocol::tests::sample("before");
    let won = crate::modern_lito::quest_protocol::tests::sample("win");
    let loss = crate::modern_lito::quest_protocol::tests::sample("loss");
    let f = before.frame.as_ref().unwrap();
    let target = f.combat.world.quest.as_ref().unwrap().targets[0].cell;
    assert!(encounter_occupied(f).contains(&target));
    assert!(!encounter_occupied(won.frame.as_ref().unwrap()).contains(&target));
    assert!(!encounter_occupied(loss.frame.as_ref().unwrap()).contains(&target));
    assert_eq!(encounter_occupied(f).len(), 1); // The visible NPC is already a corpse.
}
use crate::modern_lito::{
    resources::SharedMapSectorKey,
    world_map::SectorInput,
    world_protocol::{AbortReason, Sector},
};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};

const WAIT: Duration = Duration::from_secs(5);

fn npc_route_fixture() -> (Arc<WorldMap>, crate::modern_lito::npc_protocol::Reply) {
    let map = Arc::new(
        WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: SharedMapSectorKey {
                    map_id: 554,
                    sector_x: 0x8000,
                    sector_y: 0x8000,
                },
                bytes: vec![0; 49170],
            }],
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    let mut reply = crate::modern_lito::npc_protocol::tests::read_reply();
    let frame = reply.frame.as_mut().unwrap();
    frame.resources = Some(vec![1649, 2158]);
    frame.combat.world.manifest.digest = map.canonical_digest();
    frame.combat.world.manifest.sectors = map
        .bindings()
        .into_iter()
        .map(|b| Sector {
            x: b.key.sector_x,
            y: b.key.sector_y,
            byte_length: b.byte_length,
            sha256: b.sha256,
        })
        .collect();
    reply.validate().unwrap();
    (map, reply)
}
#[test]
fn npc_navigation_requires_full_roster_and_does_not_plan_through_live_targets() {
    let (map, reply) = npc_route_fixture();
    let frame = reply.frame.as_ref().unwrap();
    let destination = frame.actors[0].cell;
    let mut nav = Navigation::default();
    assert!(nav
        .request_combat(map.clone(), Arc::new(frame.combat.clone()), 1, destination)
        .is_err());
    assert!(nav
        .request(
            map.clone(),
            Arc::new(frame.combat.world.clone()),
            1,
            destination
        )
        .is_err());
    nav.request_npc(map, Arc::new(reply), 1, destination)
        .unwrap();
    poll_until(&mut nav, |n| matches!(n.status(), Status::Stopped(_)));
    assert!(nav.is_quiescent());
}
#[test]
fn npc_navigation_refreshes_dynamic_occupancy_before_each_unsubmitted_step() {
    let (map, reply) = npc_route_fixture();
    let destination = goal(&reply.frame.as_ref().unwrap().combat.world, 0, 4);
    let mut nav = Navigation::default();
    nav.request_npc(map, Arc::new(reply.clone()), 1, destination)
        .unwrap();
    poll_until(&mut nav, |n| n.status() == Status::Walking);
    let next = nav.next_step_npc(&reply, 1, Instant::now()).unwrap();
    let mut occupied = reply.clone();
    let frame = occupied.frame.as_mut().unwrap();
    frame.combat.world.revision += 1;
    frame.revision += 1;
    frame.elapsed_millis += 1;
    frame.actors[0].cell = next.to;
    frame.actors[0].changed_revision = frame.revision;
    occupied.validate().unwrap();
    assert!(nav.next_step_npc(&occupied, 1, Instant::now()).is_none());
    assert!(matches!(nav.status(), Status::Stopped(_)));
    assert!(nav.is_quiescent());
}

#[test]
fn quest_death_or_coalesced_respawn_retires_old_route_and_new_life_can_route() {
    let (map, ordinary) = npc_route_fixture();
    let fixture = |name| {
        let mut reply = crate::modern_lito::quest_life::tests::sample(name);
        reply.frame.as_mut().unwrap().combat.world.manifest = ordinary
            .frame
            .as_ref()
            .unwrap()
            .combat
            .world
            .manifest
            .clone();
        reply.validate().unwrap();
        reply
    };
    let before = fixture("before");
    let revived = fixture("revived");
    let mut nav = Navigation::default();
    for phase in ["dead", "revived"] {
        nav.request_npc(
            map.clone(),
            Arc::new(before.clone()),
            1,
            goal(&before.frame.as_ref().unwrap().combat.world, -3, 0),
        )
        .unwrap();
        poll_until(&mut nav, |n| n.status() == Status::Walking);
        assert!(nav.next_step_npc(&before, 1, Instant::now()).is_some());
        assert!(nav
            .next_step_npc(&fixture(phase), 1, Instant::now())
            .is_none());
        assert_eq!(
            nav.status(),
            Status::Idle,
            "death/life fence cancels without waiting for a bad move ACK"
        );
        assert!(nav.route.is_none() && nav.is_quiescent());
    }
    let combat = &revived.frame.as_ref().unwrap().combat;
    nav.request_npc(
        map,
        Arc::new(revived.clone()),
        1,
        goal(&combat.world, -3, 0),
    )
    .unwrap();
    poll_until(&mut nav, |n| n.status() == Status::Walking);
    assert_eq!(
        nav.route.as_ref().unwrap().input.life_revision,
        crate::modern_lito::quest_life::respawn_revision(combat, 1)
    );
    assert!(nav.next_step_npc(&revived, 1, Instant::now()).is_some());
}

#[test]
fn race_withdrawal_releases_only_its_cell_and_stops_its_existing_route() {
    use crate::modern_lito::race_protocol::{tests::running, DnfReason};
    let (map, ordinary) = fixture();
    let mut active = running();
    active.manifest = ordinary.manifest.clone();
    let digest = active.race.as_ref().unwrap().course_hash(&active);
    active.race.as_mut().unwrap().course_sha256 = digest;
    active.actors[1].position = WorldCell {
        x: active.actors[0].position.x + 1,
        y: active.actors[0].position.y,
    };
    active.validate(active.actors[0].key).unwrap();
    assert!(occupied(&active, 1).contains(&active.actors[1].position));
    let mut departed = active.clone();
    departed.revision += 1;
    let runner = &mut departed.race.as_mut().unwrap().runners[1];
    runner.revision = departed.revision;
    runner.dnf = DnfReason::Left;
    departed.validate(departed.actors[0].key).unwrap();
    assert!(occupied(&departed, 1).is_empty());
    assert_eq!(
        occupied(&departed, 2),
        BTreeSet::from([departed.actors[0].position])
    );
    let path = world_path::plan(
        &map,
        departed.actors[0].position,
        departed.actors[1].position,
        &occupied(&departed, 1),
        PathLimits::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(path.headings().len(), 1);
    let mut navigation = Navigation::default();
    assert!(navigation
        .request(
            map.clone(),
            Arc::new(departed.clone()),
            2,
            goal(&departed, 3, 0)
        )
        .is_err());
    assert!(navigation.is_quiescent());
    let mut route = ready(map, Arc::new(active.clone()), goal(&active, 0, 3));
    let mut self_left = active.clone();
    self_left.revision += 1;
    let runner = &mut self_left.race.as_mut().unwrap().runners[0];
    runner.revision = self_left.revision;
    runner.dnf = DnfReason::Disconnected;
    self_left.validate(self_left.actors[0].key).unwrap();
    assert!(route.next_step(&self_left, 1, Instant::now()).is_none());
    stopped(&route);
    assert_eq!(
        occupied(&ordinary, 1).len(),
        1,
        "non-race occupancy stays strict"
    );
}

fn fixture() -> (Arc<WorldMap>, Arc<Frame>) {
    let map = Arc::new(
        WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: SharedMapSectorKey {
                    map_id: 514,
                    sector_x: 0x7fff,
                    sector_y: 0x8000,
                },
                bytes: vec![0; 49170],
            }],
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    let mut frame = crate::modern_lito::world_runtime::tests::frame(1);
    frame.phase = Phase::Active;
    frame.manifest.digest = map.canonical_digest();
    frame.manifest.sectors = map
        .bindings()
        .into_iter()
        .map(|binding| Sector {
            x: binding.key.sector_x,
            y: binding.key.sector_y,
            byte_length: binding.byte_length,
            sha256: binding.sha256,
        })
        .collect();
    for actor in &mut frame.actors {
        actor.loaded = true;
    }
    frame.actors[1].position = WorldCell { x: 32744, y: 32808 };
    frame.validate(frame.actors[0].key).unwrap();
    (map, Arc::new(frame))
}
fn goal(frame: &Frame, dx: i32, dy: i32) -> WorldCell {
    WorldCell {
        x: frame.actors[0].position.x + dx,
        y: frame.actors[0].position.y + dy,
    }
}
fn route(map: Arc<WorldMap>, frame: Arc<Frame>, goal: WorldCell) -> Route {
    let plan = world_path::plan(
        &map,
        frame.actors[0].position,
        goal,
        &occupied(&frame, 1),
        PathLimits::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    Route {
        progress: Progress::new(&frame, &frame.actors[0], &plan),
        input: Input {
            objectives: BTreeSet::new(),
            life_revision: 0,
            serial: 1,
            map,
            frame,
            entity: 1,
            goal,
        },
        plan,
        offered: None,
    }
}
fn ready(map: Arc<WorldMap>, frame: Arc<Frame>, goal: WorldCell) -> Navigation {
    Navigation {
        serial: 1,
        route: Some(route(map, frame, goal)),
        status: Some(Status::Walking),
        pending: None,
        job: None,
    }
}
fn ack(frame: &Frame, step: StepIntent) -> Frame {
    let mut frame = frame.clone();
    frame.revision += 1;
    frame.actors[0].position = step.to;
    frame.actors[0].heading = step.heading;
    frame.actors[0].sequence = step.sequence;
    frame.validate(frame.actors[0].key).unwrap();
    frame
}
fn stopped(navigation: &Navigation) {
    assert!(matches!(navigation.status(), Status::Stopped(_)));
    assert!(navigation.route.is_none());
}

#[test]
fn objective_cells_block_a_prepared_step_and_respawn_discards_the_old_route() {
    let (map, frame) = fixture();
    let target = goal(&frame, 4, 0);
    let mut navigation = ready(map.clone(), frame.clone(), target);
    let first = navigation.next_step(&frame, 1, Instant::now()).unwrap();
    // Same authoritative obstacle set is consulted again immediately before
    // admission, not only once in the path worker.
    let mut blocked = ready(map.clone(), frame.clone(), target);
    blocked
        .route
        .as_mut()
        .unwrap()
        .input
        .objectives
        .insert(first.to);
    assert!(blocked.next_step(&frame, 1, Instant::now()).is_none());
    stopped(&blocked);
    let mut combat = crate::modern_lito::honor_protocol::tests::initial()
        .frame
        .unwrap();
    combat.world = (*frame).clone();
    let cells = combat
        .honor
        .as_ref()
        .unwrap()
        .objectives
        .iter()
        .map(|o| o.position)
        .collect();
    let mut old = ready(map, frame.clone(), target);
    old.route.as_mut().unwrap().input.objectives = cells;
    combat.honor.as_mut().unwrap().lives[0].respawn_revision = frame.revision + 1;
    assert!(old.next_step_combat(&combat, 1, Instant::now()).is_none());
    assert!(old.route.is_none());
    assert!(old.pending.is_none());
    assert!(old.is_quiescent());
}

#[test]
fn confirmed_speed_change_uses_same_ack_receipt_and_expiry_cannot_shorten_base_guard() {
    let (map, frame) = fixture();
    let target = goal(&frame, 4, 0);
    let mut r = route(map, frame.clone(), target);
    let t = Instant::now();
    let step = r.advance(&frame, 1, t).unwrap().1.unwrap();
    r.progress.awaiting = Some(step);
    let after = ack(&frame, step);
    r.advance(&after, 1, t).unwrap();
    let base = r.progress.interval;
    let fast = base / 2;
    assert!(r
        .advance_with_interval(&after, 1, t + fast - Duration::from_nanos(1), fast)
        .unwrap()
        .1
        .is_none());
    assert!(r
        .advance_with_interval(&after, 1, t + fast, fast)
        .unwrap()
        .1
        .is_some());
    assert!(r
        .advance_with_interval(&after, 1, t + fast, base)
        .unwrap()
        .1
        .is_none());
    assert!(r
        .advance_with_interval(&after, 1, t + base, base)
        .unwrap()
        .1
        .is_some());
    assert_eq!(r.progress.expected_sequence, step.sequence);
    assert_eq!(r.progress.index, 1);
}

#[test]
fn confirmed_teleport_stops_old_route_even_without_move_sequence_change() {
    let (map, frame) = fixture();
    let mut initial = (*frame).clone();
    initial.casts = initial
        .actors
        .iter()
        .map(|a| crate::modern_lito::spell_protocol::CastState::initial(a.entity_id, 1000))
        .collect();
    let initial = Arc::new(initial);
    let destination = goal(&initial, 4, 0);
    let mut nav = ready(map, initial.clone(), destination);
    let mut jumped = (*initial).clone();
    jumped.revision += 1;
    let from = jumped.actors[0].position;
    jumped.actors[0].position.y += 2;
    jumped.casts[0] = crate::modern_lito::spell_protocol::CastState {
        entity_id: 1,
        request_sequence: 1,
        cast_count: 1,
        cast_revision: jumped.revision,
        cast_interval_millis: 1000,
        mp_spent: 10,
        skill_id: 5,
        from,
        to: jumped.actors[0].position,
    };
    jumped.validate(jumped.actors[0].key).unwrap();
    assert_eq!(jumped.actors[0].sequence, initial.actors[0].sequence);
    assert!(nav.next_step(&jumped, 1, Instant::now()).is_none());
    stopped(&nav);
    assert!(matches!(nav.status(),Status::Stopped(message) if message.contains("瞬間移動")));
}
fn until(mut done: impl FnMut() -> bool) {
    let end = Instant::now() + WAIT;
    while !done() {
        assert!(Instant::now() < end, "owned navigation work did not retire");
        thread::yield_now();
    }
}
fn poll_until(navigation: &mut Navigation, done: impl Fn(&Navigation) -> bool) {
    until(|| {
        navigation.poll();
        done(navigation)
    });
}

struct Release {
    sender: Option<SyncSender<()>>,
    exited: Arc<AtomicBool>,
}
impl Release {
    fn release(&mut self) {
        self.sender
            .take()
            .expect("release once")
            .send(())
            .expect("held worker still owns receiver");
    }
}
impl Drop for Release {
    fn drop(&mut self) {
        // Assertion failure still releases the owned worker; it also has its
        // own bounded fail timeout, so a test failure cannot deadlock Drop.
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(());
        }
    }
}
struct ExitFlag(Arc<AtomicBool>);
impl Drop for ExitFlag {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
fn held_job(result: Route) -> (Job, Release) {
    let serial = result.input.serial;
    let cancel = Arc::new(AtomicBool::new(false));
    let exited = Arc::new(AtomicBool::new(false));
    let exit = exited.clone();
    let (started, waiting) = mpsc::sync_channel(1);
    let (release, resume) = mpsc::sync_channel(1);
    let handle = thread::spawn(move || {
        let _exit = ExitFlag(exit);
        started.send(()).unwrap();
        resume
            .recv_timeout(WAIT)
            .expect("test release guard must retire held worker");
        // Deliberately return an old successful result even after cancellation:
        // poll must discard it by cancellation/serial, not trust its success.
        Ok(result)
    });
    let job = Job {
        serial,
        cancel,
        handle: Some(handle),
    };
    let release = Release {
        sender: Some(release),
        exited,
    };
    waiting
        .recv_timeout(WAIT)
        .expect("owned worker reached hold barrier");
    (job, release)
}
struct OwnedJoin(Option<JoinHandle<()>>);
impl OwnedJoin {
    fn join(&mut self) {
        self.0.take().unwrap().join().unwrap();
    }
}
impl Drop for OwnedJoin {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            let _ = handle.join();
        }
    }
}

#[test]
fn offered_is_not_submitted_and_unacknowledged_time_never_advances() {
    let (map, frame) = fixture();
    let mut nav = ready(map, frame.clone(), goal(&frame, 3, 0));
    let now = Instant::now();
    let step = nav.next_step(&frame, 1, now).unwrap();
    assert_eq!(nav.status(), Status::Walking);
    assert_eq!(nav.route.as_ref().unwrap().progress.index, 0);
    assert!(nav.route.as_ref().unwrap().progress.awaiting.is_none());
    assert_eq!(
        nav.next_step(&frame, 1, now + Duration::from_secs(60)),
        Some(step)
    );
    let mut forged = step;
    forged.sequence += 1;
    assert!(nav.submitted(forged).is_err());
    assert!(nav.route.as_ref().unwrap().progress.awaiting.is_none());
    nav.submitted(step).unwrap();
    assert!(nav.submitted(step).is_err());
    assert_eq!(nav.status(), Status::WaitingForAck);
    assert!(nav
        .next_step(&frame, 1, now + Duration::from_secs(120))
        .is_none());
    assert_eq!(nav.status(), Status::WaitingForAck);
    assert_eq!(nav.route.as_ref().unwrap().progress.index, 0);
    assert_eq!(
        nav.route.as_ref().unwrap().progress.expected_sequence,
        frame.actors[0].sequence
    );
}

#[test]
fn exact_ack_uses_receipt_cooldown_duplicate_sample_does_not_delay_and_final_ack_arrives() {
    let (map, frame) = fixture();
    let mut nav = ready(map, frame.clone(), goal(&frame, 2, 0));
    let start = Instant::now();
    let first = nav.next_step(&frame, 1, start).unwrap();
    nav.submitted(first).unwrap();
    let first_ack = ack(&frame, first);
    let received = start + Duration::from_secs(10);
    let interval = Duration::from_nanos(frame.actors[0].step_interval_nanos);
    assert!(nav.next_step(&first_ack, 1, received).is_none());
    assert_eq!(nav.status(), Status::Walking);
    assert!(nav
        .next_step(&first_ack, 1, received + interval - Duration::from_nanos(1))
        .is_none());
    let second = nav.next_step(&first_ack, 1, received + interval).unwrap();
    assert_eq!(second.from, first.to);
    assert_eq!(second.sequence, first.sequence + 1);
    assert_eq!(nav.route.as_ref().unwrap().progress.index, 1);
    nav.submitted(second).unwrap();
    let final_ack = ack(&first_ack, second);
    assert!(nav.next_step(&final_ack, 1, received + interval).is_none());
    assert_eq!(nav.status(), Status::Arrived);
    assert!(nav
        .next_step(&final_ack, 1, received + Duration::from_secs(100))
        .is_none());
    assert_eq!(nav.status(), Status::Arrived);
    assert_eq!(nav.route.as_ref().unwrap().progress.index, 2);
}

#[test]
fn denied_consumed_sequence_without_movement_stops_instead_of_skipping_a_cell() {
    let (map, frame) = fixture();
    let mut nav = ready(map, frame.clone(), goal(&frame, 3, 0));
    let now = Instant::now();
    let step = nav.next_step(&frame, 1, now).unwrap();
    nav.submitted(step).unwrap();
    let mut denied = frame.as_ref().clone();
    denied.revision += 1;
    denied.actors[0].sequence = step.sequence;
    assert!(nav.next_step(&denied, 1, now).is_none());
    stopped(&nav);
    assert_eq!(denied.actors[0].position, frame.actors[0].position);
}

#[test]
fn wrong_ack_sequence_position_heading_or_changed_interval_each_stop() {
    let (map, frame) = fixture();
    for change in 0..4 {
        let mut nav = ready(map.clone(), frame.clone(), goal(&frame, 3, 0));
        let now = Instant::now();
        let step = nav.next_step(&frame, 1, now).unwrap();
        nav.submitted(step).unwrap();
        let mut changed = ack(&frame, step);
        match change {
            0 => changed.actors[0].sequence += 1,
            1 => changed.actors[0].position.y += 1,
            2 => changed.actors[0].heading = (step.heading + 1) % 8,
            3 => changed.actors[0].step_interval_nanos += 1,
            _ => unreachable!(),
        }
        assert!(nav.next_step(&changed, 1, now).is_none(), "case {change}");
        stopped(&nav);
    }
}

#[test]
fn changed_identity_map_phase_room_or_regressed_revision_rejects_existing_route() {
    let (map, frame) = fixture();
    for change in 0..9 {
        let mut nav = ready(map.clone(), frame.clone(), goal(&frame, 3, 0));
        let mut changed = frame.as_ref().clone();
        let mut entity = 1;
        match change {
            0 => changed.match_id += 1,
            1 => changed.generation += 1,
            2 => changed.actors[0].key.source_id += 1,
            3 => entity = 2,
            4 => changed.manifest.digest[0] ^= 1,
            5 => {
                changed.phase = Phase::Aborted;
                changed.abort_reason = AbortReason::ServerAbort;
            }
            6 => changed.room.summary.revision += 1,
            7 => changed.revision -= 1,
            8 => {
                changed.actors.remove(0);
            }
            _ => unreachable!(),
        };
        assert!(
            nav.next_step(&changed, entity, Instant::now()).is_none(),
            "case {change}"
        );
        stopped(&nav);
    }
}

#[test]
fn new_occupancy_rechecks_exact_client_edge_and_conservative_diagonal_policy() {
    let (map, frame) = fixture();
    let mut nav = ready(map.clone(), frame.clone(), goal(&frame, 3, 0));
    let offered = nav.next_step(&frame, 1, Instant::now()).unwrap();
    let mut blocked = frame.as_ref().clone();
    blocked.revision += 1;
    blocked.actors[1].position = offered.to;
    assert!(nav.next_step(&blocked, 1, Instant::now()).is_none());
    stopped(&nav);
    assert!(nav.submitted(offered).is_err());
    // A diagonal may use either open, unoccupied orthogonal intermediate.
    let mut nav = ready(map, frame.clone(), goal(&frame, 1, -1));
    let step = nav.next_step(&frame, 1, Instant::now()).unwrap();
    assert_eq!(step.heading, 1);
    let mut one_corner = frame.as_ref().clone();
    one_corner.actors[1].position = goal(&frame, 1, 0);
    assert_eq!(nav.next_step(&one_corner, 1, Instant::now()), Some(step));
    assert!(!occupied(&one_corner, 1).contains(&frame.actors[0].position));
}

#[test]
fn invalid_request_does_not_cancel_current_valid_route_or_mint_serial() {
    let (map, frame) = fixture();
    let mut nav = ready(map.clone(), frame.clone(), goal(&frame, 3, 0));
    let step = nav.next_step(&frame, 1, Instant::now()).unwrap();
    for change in 0..4 {
        let mut bad = frame.as_ref().clone();
        let entity = if change == 0 { 999 } else { 1 };
        match change {
            1 => bad.phase = Phase::Loading,
            2 => bad.manifest.digest[0] ^= 1,
            3 => bad.actors[0].loaded = false,
            _ => {}
        }
        assert!(nav
            .request(map.clone(), Arc::new(bad), entity, goal(&frame, 2, 1))
            .is_err());
        assert_eq!(nav.serial, 1);
        assert_eq!(nav.next_step(&frame, 1, Instant::now()), Some(step));
    }
    nav.serial = u64::MAX;
    assert!(nav
        .request(map, frame.clone(), 1, goal(&frame, 2, 1))
        .is_err());
    assert_eq!(nav.next_step(&frame, 1, Instant::now()), Some(step));
}

#[test]
fn exhausted_sequence_stops_and_zero_step_goal_arrives_without_submission() {
    let (map, frame) = fixture();
    let mut exhausted = frame.as_ref().clone();
    exhausted.actors[0].sequence = i64::MAX as u64;
    let exhausted = Arc::new(exhausted);
    let mut nav = ready(map.clone(), exhausted.clone(), goal(&exhausted, 1, 0));
    assert!(nav.next_step(&exhausted, 1, Instant::now()).is_none());
    stopped(&nav);
    let mut nav = ready(map, exhausted.clone(), exhausted.actors[0].position);
    assert!(nav.next_step(&exhausted, 1, Instant::now()).is_none());
    assert_eq!(nav.status(), Status::Arrived);
    assert_eq!(nav.route.as_ref().unwrap().progress.index, 0);
}

#[test]
fn cancel_before_poll_never_starts_pending_worker_or_keeps_an_offered_route() {
    let (map, frame) = fixture();
    let mut nav = Navigation::default();
    assert_eq!(nav.status(), Status::Idle);
    nav.request(map.clone(), frame.clone(), 1, goal(&frame, 3, 0))
        .unwrap();
    assert!(nav.pending.is_some() && nav.job.is_none());
    assert!(!nav.is_quiescent());
    nav.cancel();
    nav.poll();
    assert!(nav.pending.is_none() && nav.job.is_none() && nav.route.is_none());
    assert!(nav.is_quiescent());
    assert_eq!(nav.status(), Status::Idle);
    nav = ready(map, frame.clone(), goal(&frame, 3, 0));
    let offered = nav.next_step(&frame, 1, Instant::now()).unwrap();
    nav.cancel();
    assert!(nav.submitted(offered).is_err());
    assert!(nav.next_step(&frame, 1, Instant::now()).is_none());
}

#[test]
fn actual_worker_rejects_occupied_target_then_new_request_recovers() {
    let (map, frame) = fixture();
    let mut nav = Navigation::default();
    nav.request(map.clone(), frame.clone(), 1, frame.actors[1].position)
        .unwrap();
    poll_until(&mut nav, |n| {
        matches!(n.status(), Status::Stopped(_)) && n.is_quiescent()
    });
    stopped(&nav);
    nav.request(map, frame.clone(), 1, goal(&frame, 3, 0))
        .unwrap();
    poll_until(&mut nav, |n| n.route.is_some() && n.is_quiescent());
    assert_eq!(nav.status(), Status::Walking);
    assert!(nav.next_step(&frame, 1, Instant::now()).is_some());
}

#[test]
fn active_cancel_retains_same_owned_handle_until_release_then_discards_late_success() {
    let (map, frame) = fixture();
    let mut nav = Navigation::default();
    let (job, mut release) = held_job(route(map, frame.clone(), goal(&frame, 3, 0)));
    let identity = job.handle.as_ref().unwrap().thread().id();
    let cancel = job.cancel.clone();
    nav.serial = job.serial;
    nav.job = Some(job);
    nav.cancel();
    assert!(cancel.load(Ordering::Acquire));
    nav.poll();
    assert_eq!(nav.status(), Status::Stopping);
    assert!(!nav.is_quiescent());
    assert_eq!(
        nav.job
            .as_ref()
            .unwrap()
            .handle
            .as_ref()
            .unwrap()
            .thread()
            .id(),
        identity
    );
    assert!(!release.exited.load(Ordering::Acquire));
    release.release();
    poll_until(&mut nav, Navigation::is_quiescent);
    assert!(release.exited.load(Ordering::Acquire));
    assert!(nav.job.is_none() && nav.route.is_none());
    assert_eq!(nav.status(), Status::Idle);
}

#[test]
fn new_targets_coalesce_behind_one_retained_worker_and_only_latest_can_publish() {
    let (map, frame) = fixture();
    let mut nav = Navigation::default();
    let (job, mut release) = held_job(route(map.clone(), frame.clone(), goal(&frame, 3, 0)));
    let identity = job.handle.as_ref().unwrap().thread().id();
    nav.serial = job.serial;
    nav.job = Some(job);
    nav.request(map.clone(), frame.clone(), 1, goal(&frame, 0, 3))
        .unwrap();
    let final_goal = goal(&frame, -3, 0);
    nav.request(map, frame.clone(), 1, final_goal).unwrap();
    nav.poll();
    assert_eq!(nav.serial, 3);
    assert_eq!(nav.pending.as_ref().unwrap().goal, final_goal);
    assert_eq!(
        nav.job
            .as_ref()
            .unwrap()
            .handle
            .as_ref()
            .unwrap()
            .thread()
            .id(),
        identity
    );
    assert!(nav.route.is_none());
    assert!(!release.exited.load(Ordering::Acquire));
    release.release();
    poll_until(&mut nav, |n| n.is_quiescent() && n.route.is_some());
    assert!(release.exited.load(Ordering::Acquire));
    let latest = nav.route.as_ref().unwrap();
    assert_eq!(latest.input.serial, 3);
    assert_eq!(latest.input.goal, final_goal);
    assert_eq!(latest.plan.cells().last(), Some(&final_goal));
    assert_eq!(latest.plan.steps(), 3);
    // Equal-cost BFS uses documented heading order, not a straight-line bias.
    // For this west target the first shortest route begins SW (5), then W/NW.
    assert_eq!(latest.plan.headings(), &[5, 6, 7]);
    assert_eq!(nav.next_step(&frame, 1, Instant::now()).unwrap().heading, 5);
}

#[test]
fn navigation_drop_cancels_but_waits_for_actual_owned_worker_retirement() {
    let (map, frame) = fixture();
    let mut nav = Navigation::default();
    let (job, mut release) = held_job(route(map, frame.clone(), goal(&frame, 3, 0)));
    let cancel = job.cancel.clone();
    nav.serial = job.serial;
    nav.job = Some(job);
    let (done, finished): (SyncSender<()>, Receiver<()>) = mpsc::sync_channel(1);
    let mut dropper = OwnedJoin(Some(thread::spawn(move || {
        drop(nav);
        done.send(()).unwrap();
    })));
    until(|| cancel.load(Ordering::Acquire));
    assert!(matches!(finished.try_recv(), Err(TryRecvError::Empty)));
    assert!(!release.exited.load(Ordering::Acquire));
    release.release();
    finished
        .recv_timeout(WAIT)
        .expect("Navigation Drop must finish after owned worker");
    dropper.join();
    assert!(release.exited.load(Ordering::Acquire));
}
