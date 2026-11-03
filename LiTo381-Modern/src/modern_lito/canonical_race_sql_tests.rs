//! Two v41 controllers, qualified shared5143 pets and actual ordered MOVEs.
//! Ranked SQL is separate from combat totals. No legacy Frame or fake LOAD.
//! Departures use the actual controller/TLS owner, not a fabricated DNF frame.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Flow {
    Finish,
    Withdraw,
    Disconnect,
}
impl Flow {
    fn mode(self) -> &'static str {
        match self {
            Self::Finish => "race-finish",
            Self::Withdraw => "race-withdraw",
            Self::Disconnect => "race-disconnect",
        }
    }
    fn winner(self) -> usize {
        usize::from(self == Self::Withdraw)
    }
    fn attached(self, i: usize) -> bool {
        self == Self::Finish || i == self.winner()
    }
    fn loser_reason(self) -> data::Dnf {
        match self {
            Self::Finish => data::Dnf::WindowClosed,
            Self::Withdraw => data::Dnf::Left,
            Self::Disconnect => data::Dnf::Disconnected,
        }
    }
}

fn race(c: &Client) -> data::Race {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .race
        .clone()
        .unwrap()
}
fn runner(c: &Client) -> data::Runner {
    runner_for(c, own_actor(c).entity)
}
fn runner_for(c: &Client, entity: i64) -> data::Runner {
    race(c)
        .runners
        .0
        .into_iter()
        .find(|r| r.entity == entity)
        .unwrap()
}
fn contains(g: &data::Gate, c: WorldCell) -> bool {
    (g.min_x..=g.max_x).contains(&c.x) && (g.min_y..=g.max_y).contains(&c.y)
}
fn cell(c: &data::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
fn hud(c: &Client) -> String {
    battle_window::hud(&c.surface.as_ref().unwrap().observation).join("\n")
}
fn absent() -> record::Totals {
    record::Totals {
        stored: false,
        wins: 0,
        losses: 0,
        revision: 0,
    }
}
fn selected(cs: &mut [Client; 2], i: usize, d: &Deadline) -> SelectionContext {
    let key = if i == 0 { SUN } else { MOON };
    send(cs, i, d, || LoginCommand::SelectSource(key.source_id));
    wait(cs, d, "canonical race roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
    let v = cs[i].view();
    assert_eq!(v.characters.len(), 1);
    assert_eq!(
        (v.characters[0].source_id, v.characters[0].object_id),
        (key.source_id, key.object_id)
    );
    let context = SelectionContext {
        key,
        epoch: v.roster_epoch,
    };
    send(cs, i, d, || LoginCommand::SelectCharacter(context));
    wait(cs, d, "canonical race selected record", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::ChoosingChannel
            && v.record.context() == Some(context)
            && v.record.totals() == Some(absent())
    });
    context
}

// Test driver only: bounded BFS to enter the next server checkpoint rectangle.
// Every step still passes the actual rendered input adapter and awaits ACK.
fn next_path(c: &Client) -> VecDeque<(u8, WorldCell)> {
    let surface = c.surface.as_ref().unwrap();
    let nav = surface.navigation.as_ref().unwrap();
    let r = runner(c);
    let course = race(c);
    let gate = &course.gates.0[r.next_checkpoint as usize];
    let from = cell(&own_actor(c).position);
    let leave = contains(gate, from);
    let mut queue = VecDeque::from([from]);
    let mut previous = BTreeMap::from([(from, (from, 0u8))]);
    let goal = loop {
        let at = queue
            .pop_front()
            .expect("qualified race checkpoint reachable");
        if at != from && contains(gate, at) != leave {
            break at;
        }
        for (h, (dx, dy)) in DIRECTIONS.iter().copied().enumerate() {
            let to = WorldCell {
                x: at.x + dx,
                y: at.y + dy,
            };
            if !previous.contains_key(&to) && nav.can_step(at, to) {
                assert!(previous.len() < 16_384, "bounded race search");
                previous.insert(to, (at, h as u8));
                queue.push_back(to);
            }
        }
    };
    let mut to = goal;
    let mut path = VecDeque::new();
    loop {
        let (at, h) = previous[&to];
        path.push_front((h, to));
        if at == from {
            return path;
        }
        to = at;
    }
}

fn leave_world(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    context: SelectionContext,
    identity: &data::WorldId,
) {
    let q = Command::world(battle_world::Request {
        action: battle_world::Action::Leave,
        ..battle_world::Request::read(context, CHANNEL, Some(identity))
    });
    send(cs, i, d, || LoginCommand::Canonical(q.clone()));
    wait(cs, d, "canonical race detached", |cs| {
        !cs[i].view().world_pending
            && !cs[i].view().canonical.attached()
            && cs[i].surface.is_none()
            && cs[i].runtime.is_quiescent()
    });
}

pub(super) fn finish_current(
    cs: &mut [Client; 2],
    d: &Deadline,
    contexts: [SelectionContext; 2],
) -> [u8; 16] {
    let entities = [own_actor(&cs[0]).entity, own_actor(&cs[1]).entity];
    finish_current_inner(cs, d, Flow::Finish, contexts, entities)
}

fn finish_current_inner(
    cs: &mut [Client; 2],
    d: &Deadline,
    flow: Flow,
    contexts: [SelectionContext; 2],
    entities: [i64; 2],
) -> [u8; 16] {
    let winner = flow.winner();
    let loser = 1 - winner;
    let mut moves = 0;
    let mut path = VecDeque::new();
    let mut costs = [Duration::ZERO; 3];
    while runner(&cs[winner]).placement == 0 {
        let at = Instant::now();
        wait(cs, d, "canonical race step pixels", |cs| {
            cs[winner].rendered()
        });
        costs[0] += at.elapsed();
        let before = runner(&cs[winner]);
        let actor = own_actor(&cs[winner]);
        let course = race(&cs[winner]);
        let gate = &course.gates.0[before.next_checkpoint as usize];
        let at = Instant::now();
        if path.is_empty() {
            path = next_path(&cs[winner]);
        }
        let (heading, to) = path.pop_front().unwrap();
        costs[1] += at.elapsed();
        let at = Instant::now();
        let v = input(cs, winner, d, Kind::Local(Local::Step(heading)));
        costs[2] += at.elapsed();
        assert_eq!(
            v.canonical.outcome(),
            Some(Outcome::Operation(
                Rejection::None,
                Disposition::Applied,
                Reason::None
            )),
            "race move {moves}, timings pixels/search/receipt={costs:?}"
        );
        let after = runner(&cs[winner]);
        let moved = own_actor(&cs[winner]);
        assert_eq!(
            (cell(&moved.position), moved.last_move_sequence),
            (to, actor.last_move_sequence + 1)
        );
        let crossing = !contains(gate, cell(&actor.position)) && contains(gate, to);
        let checkpoint = before.next_checkpoint + i64::from(crossing);
        let lap = checkpoint == course.gates.0.len() as i64;
        assert_eq!(
            (after.completed_laps, after.next_checkpoint),
            (
                before.completed_laps + i64::from(lap),
                if lap { 0 } else { checkpoint }
            )
        );
        assert_eq!(
            after.progress_move_sequence,
            if crossing {
                moved.last_move_sequence
            } else {
                before.progress_move_sequence
            }
        );
        moves += 1;
        assert!(moves < 4096, "bounded full course");
        if lap {
            eprintln!(
                "CANONICAL_RACE {} lap={} moves={moves} pixels/search/receipt={costs:?}",
                flow.mode(),
                after.completed_laps
            );
        }
    }
    assert_eq!(runner(&cs[winner]).completed_laps, 4);
    assert!(moves >= 4 * race(&cs[winner]).gates.0.len());
    let finished = runner(&cs[winner]);
    assert_eq!(
        race(&cs[winner]).finish_window_deadline_elapsed_nanos - finished.finish_elapsed_nanos,
        10_000_000_000
    );
    let finish_window = race(&cs[winner]).finish_window_deadline_elapsed_nanos;
    wait(cs, d, "canonical race first finish painted", |cs| {
        cs.iter()
            .enumerate()
            .filter(|(i, _)| flow.attached(*i))
            .all(|(_, c)| {
                c.surface.as_ref().is_some_and(|s| {
                    s.observation
                        .data()
                        .race
                        .as_ref()
                        .is_some_and(|r| r.phase == data::RacePhase::FinishWindow)
                        && s.geometry.actors.len() == usize::from(flow == Flow::Finish)
                })
            })
    });
    assert!(
        hud(&cs[winner]).contains("你的名次：第 1 名") && hud(&cs[winner]).contains("完賽 1/2")
    );
    assert!(!battle_input::ready(
        Kind::Local(Local::Step(0)),
        &cs[winner].view()
    ));
    if flow == Flow::Finish {
        assert!(hud(&cs[loser]).contains("已完成 0/4 圈") && !hud(&cs[loser]).contains("你的名次"));
        assert!(battle_input::ready(
            Kind::Local(Local::Step(0)),
            &cs[loser].view()
        ));
    }
    wait(
        cs,
        d,
        "canonical race real finish window and ranked SQL",
        |cs| {
            cs.iter()
                .enumerate()
                .filter(|(i, _)| flow.attached(*i))
                .all(|(_, c)| {
                    c.view().canonical.lifecycle().is_some_and(|l| {
                        l.phase == world::Phase::Finished
                            && l.race_settlement == Some(world::SettlementState::Saved)
                            && l.settlement.is_none()
                    }) && c.surface.as_ref().is_some_and(|s| {
                        s.geometry.actors.is_empty()
                            && s.observation
                                .data()
                                .race
                                .as_ref()
                                .is_some_and(|r| r.phase == data::RacePhase::Complete)
                    })
                })
        },
    );
    assert_eq!(
        runner_for(&cs[winner], entities[loser]).dnf,
        flow.loser_reason()
    );
    assert_eq!(runner_for(&cs[winner], entities[loser]).placement, 0);
    let result = race(&cs[winner]).result_id;
    for (i, c) in cs.iter().enumerate().filter(|(i, _)| flow.attached(*i)) {
        let r = race(c);
        assert_eq!(r.finish_window_deadline_elapsed_nanos, finish_window);
        assert_eq!(r.first_finisher_entity, entities[winner]);
        assert_eq!(r.result_id, result);
        assert_eq!(runner_for(c, entities[loser]).dnf, flow.loser_reason());
        assert!(r.observed_elapsed_nanos >= r.finish_window_deadline_elapsed_nanos);
        assert!(r.result_id.is_some());
        assert!(c
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .result
            .is_none());
        assert!(hud(c).contains("不計入戰鬥勝敗") && !hud(c).contains("已保存"));
        assert!(!battle_input::ready(
            Kind::Local(Local::Step(0)),
            &cs[i].view()
        ));
    }
    if flow == Flow::Finish {
        assert!(hud(&cs[loser]).contains("你未完賽：完賽窗口已關閉"));
    }
    eprintln!(
        "CANONICAL_RACE {} saved moves={moves} winner_source={} loser_dnf={:?} result={result:?}",
        flow.mode(),
        contexts[winner].key.source_id,
        flow.loser_reason()
    );
    result.expect("completed race carries its ranked result identity")
}
fn run(flow: Flow, ordinary: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(175));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        if ordinary {
            "local.lito381.server.ModernOrdinaryPetRaceTestPeer"
        } else {
            "local.lito381.server.ModernCanonicalPetRaceTestPeer"
        },
        flow.mode(),
    );
    assert!(peer.process_job.is_some());
    let mut cs = std::array::from_fn(|_| Client {
        controller: mode_controller(peer.endpoint, &shared, &private_root(&fixtures), ordinary),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "canonical race TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "canonical race login", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = selected(&mut cs, i, &d);
        send(&mut cs, i, &d, || LoginCommand::EnterChannel {
            context: contexts[i],
            channel_id: CHANNEL,
        });
        wait(&mut cs, &d, "canonical race lobby", |cs| {
            cs[i].view().phase == LoginPhase::InLobby && !cs[i].view().room_pending
        });
        assert!(cs[i].view().canonical.supported());
        assert!(!login_window::can_open_shop(&cs[i].view()));
    }
    send(&mut cs, 0, &d, || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(contexts[0], CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 0,
                        map_id: 565,
                        attributes: 0,
                        name: "Canonical ranked race".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "canonical race creator joined", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    let room = cs[0].view().rooms.joined().unwrap().summary.clone();
    send(&mut cs, 1, &d, || {
        LoginCommand::Rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: room.room_id,
                    expected_revision: room.revision,
                    ..rooms::Common::read(contexts[1], CHANNEL)
                },
                String::new(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "canonical race guest joined", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
    }
    cs[0].load = true;
    wait(&mut cs, &d, "canonical race first pet pixels", |cs| {
        !cs[0].load
            && cs.iter().all(|c| {
                c.surface
                    .as_ref()
                    .is_some_and(|s| s.geometry.actors.len() == 2)
            })
    });
    for c in &cs {
        let v = c.view();
        let raw = v.canonical.observation().unwrap().data();
        assert_eq!(
            (raw.room.mode, raw.room.map, raw.manifest.map),
            (0, 565, 5143)
        );
        assert_eq!(raw.world.phase, data::Phase::Loading);
        assert_eq!(raw.world.descriptor.ruleset, data::Ruleset::PetRace);
        assert_eq!(race(c).gates.0.len(), 29);
        assert!(raw.actors.0.iter().all(|a| a.vital.is_none()));
        assert!(hud(c).contains("等待起跑 · 完賽 0/2"));
    }
    cs[1].load = true;
    wait(&mut cs, &d, "canonical race active", |cs| {
        cs.iter()
            .all(|c| c.rendered() && race(c).phase == data::RacePhase::Running)
    });
    let world_id = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    let winner = flow.winner();
    let loser = 1 - winner;
    let entities = [own_actor(&cs[0]).entity, own_actor(&cs[1]).entity];
    if flow != Flow::Finish {
        let surface = cs[loser].surface.as_ref().unwrap().clone();
        let intent = Intent {
            stamp: Stamp::of(&surface.observation),
            kind: Kind::Local(Local::Step(next_path(&cs[loser])[0].0)),
        };
        let stale = battle_input::command(&surface, intent, &cs[loser].view()).unwrap();
        if flow == Flow::Withdraw {
            leave_world(&mut cs, loser, &d, contexts[loser], &world_id);
            room_action(&mut cs, loser, &d, contexts[loser], rooms::Action::Leave, 0);
            assert!(cs[loser].view().rooms.joined().is_none());
            assert_eq!(cs[loser].view().phase, LoginPhase::InLobby);
        } else {
            // Real controller cancellation closes its TLS connection without a
            // WorldLeave. This covers a disconnected participant, not OS-level
            // packet loss, process termination or a reconnect policy.
            send(&mut cs, loser, &d, || LoginCommand::Disconnect);
            wait(&mut cs, &d, "canonical race TLS owner retired", |cs| {
                cs[loser].controller.is_quiescent()
                    && cs[loser].runtime.is_quiescent()
                    && cs[loser].view().phase == LoginPhase::Disconnected
                    && cs[loser].surface.is_none()
            });
        }
        assert!(!battle_player::controls_ready(&surface, &cs[loser].view()));
        assert!(battle_input::command(&surface, intent, &cs[loser].view()).is_err());
        assert!(cs[loser]
            .controller
            .command(LoginCommand::Canonical(stale))
            .is_err());
        wait(
            &mut cs,
            &d,
            "canonical race surviving runner sees departure",
            |cs| {
                cs[winner].rendered()
                    && runner_for(&cs[winner], entities[loser]).dnf == flow.loser_reason()
                    && cs[winner]
                        .surface
                        .as_ref()
                        .is_some_and(|s| s.geometry.actors.len() == 1)
            },
        );
        assert_eq!(race(&cs[winner]).phase, data::RacePhase::Running);
        assert_eq!(runner_for(&cs[winner], entities[loser]).placement, 0);
        assert_eq!(runner(&cs[winner]).dnf, data::Dnf::None);
        assert_eq!(runner(&cs[winner]).completed_laps, 0);
        assert!(battle_input::ready(
            Kind::Local(Local::Step(0)),
            &cs[winner].view()
        ));
        assert!(!cs[loser].view().canonical.attached());
    }
    finish_current_inner(&mut cs, &d, flow, contexts, entities);
    for i in 0..2 {
        if flow == Flow::Disconnect && i == loser {
            cs[i].runtime.shutdown();
            cs[i].controller.command(LoginCommand::Shutdown).unwrap();
            continue;
        }
        if flow.attached(i) {
            leave_world(&mut cs, i, &d, contexts[i], &world_id);
            send(&mut cs, i, &d, || {
                LoginCommand::Rooms(rooms::Request::read(contexts[i], CHANNEL).unwrap())
            });
            wait(&mut cs, &d, "canonical race returned room", |cs| {
                !cs[i].view().room_pending
            });
            let returned = cs[i].view().rooms.joined().unwrap().clone();
            assert_eq!(returned.summary.room_id, room.room_id);
            assert_eq!(returned.summary.ready, 0);
            assert!(!returned.summary.satisfied);
            assert_eq!(
                returned.members.len(),
                if flow == Flow::Finish { 2 - i } else { 1 }
            );
            room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        }
        send(&mut cs, i, &d, || LoginCommand::ReadRecord);
        wait(
            &mut cs,
            &d,
            "canonical race combat record still absent",
            |cs| !cs[i].view().record_pending && cs[i].view().record.totals() == Some(absent()),
        );
        send(&mut cs, i, &d, || LoginCommand::LeaveLobby(contexts[i]));
        wait(&mut cs, &d, "canonical race channels", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingChannel
        });
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "canonical race return roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        selected(&mut cs, i, &d);
        assert_eq!(cs[i].view().record.totals(), Some(absent()));
        cs[i].runtime.shutdown();
        cs[i].controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "canonical race normal shutdown", |cs| {
        cs.iter()
            .all(|c| c.runtime.is_quiescent() && c.controller.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}

#[test]
#[ignore = "pure v41/shared5143: two pet renderers, four ordered laps, finish window, ranked SQL and return; no HWND"]
fn java_v41_sql_pet_race_four_laps_ranked_saved_hud_and_return() {
    run(Flow::Finish, false);
}

#[test]
#[ignore = "pure v41/shared5143: creator WorldLeave and RoomsLeave, surviving controller four laps, LEFT ranked SQL and return"]
fn java_v41_sql_pet_race_creator_withdraw_survivor_saved_and_return() {
    run(Flow::Withdraw, false);
}

#[test]
#[ignore = "pure v41/shared5143: true controller TLS closure, surviving controller four laps, DISCONNECTED ranked SQL and return"]
fn java_v41_sql_pet_race_controller_disconnect_survivor_saved_and_return() {
    run(Flow::Disconnect, false);
}

#[test]
#[ignore = "ordinary schema2 v47/shared5143: no combat/equipment/Slow; four laps, ranked SQL and return; synthetic source, no HWND"]
fn java_ordinary_v47_pet_race_saved_and_return() {
    run(Flow::Finish, true);
}
