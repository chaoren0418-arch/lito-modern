// Two real v40 controllers, owned TLS service and shared map554/SPR renderers.
// Death -> explicit receipt -> new-life movement -> abort/return, without HWND.
use super::*;
use crate::modern_lito::{
    battle_action::Guard,
    battle_control::{Command as PlayerCommand, Operation, Outcome},
    battle_operation::{Disposition, Reason as OperationReason},
    battle_player, battle_revival,
    battle_runtime::{BattleRuntime, Status},
    battle_scene::Surface,
    battle_window::Stamp,
    battle_world,
    login_window::{LoginCommand, LoginController, LoginPhase, LoginSnapshot},
    profile::ModernProfile,
    session::SessionController,
    world_map::{WorldCell, DIRECTIONS},
};
use base64::Engine;
use std::sync::Arc;

struct Client {
    controller: SessionController,
    runtime: BattleRuntime,
    surface: Option<Arc<Surface>>,
    auto_load: bool,
    loads: usize,
    motion: bool,
}
impl Client {
    fn view(&self) -> LoginSnapshot {
        self.controller.snapshot()
    }
    fn pump(&mut self) {
        self.controller.poll();
        let view = self.view();
        assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
        if let Some(o)=view.canonical.observation() {
            assert_eq!(o.data().npc_motion.is_some(),self.motion);
        }
        assert!(view.world.frame().is_none());
        assert!(view.combat.is_none() && view.npc.is_none());
        assert!(
            view.battle.observation().is_none(),
            "no legacy presentation fallback"
        );
        self.surface = match battle_player::update(&mut self.runtime, &view, false).unwrap() {
            Status::Ready(s) if battle_player::admitted(&s, &view) => Some(s),
            Status::Failed(reason) => panic!("canonical revival scene: {reason}"),
            _ => None,
        };
        if self.auto_load {
            if let Some(q) = self
                .surface
                .as_ref()
                .and_then(|s| battle_player::canonical_loaded_command(&self.runtime, s, &view))
            {
                if self.controller.command(LoginCommand::Canonical(q)).is_ok() {
                    self.auto_load = false;
                    self.loads += 1;
                }
            }
        }
    }
    fn controls_ready(&self) -> bool {
        self.surface
            .as_ref()
            .is_some_and(|s| battle_player::controls_ready(s, &self.view()))
    }
}
#[track_caller]
fn wait(
    clients: &mut [Client; 2],
    deadline: Instant,
    label: &str,
    condition: impl Fn(&[Client; 2]) -> bool,
) {
    loop {
        for c in clients.iter_mut() {
            c.pump();
        }
        if condition(clients) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "v40 {label} deadline: {} / {}",
            clients[0].view().status,
            clients[1].view().status
        );
        thread::sleep(Duration::from_millis(10));
    }
}
fn submit(
    clients: &mut [Client; 2],
    i: usize,
    deadline: Instant,
    label: &str,
    build: impl Fn() -> LoginCommand,
) {
    loop {
        for c in clients.iter_mut() {
            c.pump();
        }
        // Only retry local admission before acceptance, never a sent command.
        if clients[i].controller.command(build()).is_ok() {
            return;
        }
        assert!(Instant::now() < deadline, "v40 local admission: {label}");
        thread::sleep(Duration::from_millis(10));
    }
}
fn room(clients: &mut [Client; 2], i: usize, deadline: Instant, q: rooms::Request) {
    clients[i]
        .controller
        .command(LoginCommand::Rooms(q))
        .unwrap();
    wait(clients, deadline, "room receipt", |cs| {
        !cs[i].view().room_pending
    });
}
fn room_change(
    context: SelectionContext,
    summary: &rooms::Summary,
    action: rooms::Action,
    value: u8,
) -> rooms::Request {
    rooms::Request::simple(rooms::Common {
        context,
        channel_id: 0,
        action,
        value,
        room_id: summary.room_id,
        expected_revision: summary.revision,
    })
    .unwrap()
}

#[test]
#[ignore = "v40 two actual controllers, shared554/SPR, death/revival/new-life MOVE/abort/return; no HWND or production SQL"]
fn java_v40_controllers_shared_death_revival_move_abort_return_and_shutdown() {
    flow(false, false, false, false);
}
#[test]
#[ignore = "v43 controllers/shared554, motion retained across explicit revival, new-life move, abort/return; no HWND"]
fn java_v43_controllers_npc_motion_death_revival_move_abort_return() {
    flow(true, false, false, false);
}
#[test]
#[ignore = "v43 actual TLS controllers/shared554, fast NPC coalescing/render/audio, abort/return; no HWND or production SQL"]
fn java_v43_controllers_shared_npc_cadence_render_audio_and_return() {
    flow(true, true, false, false);
}
#[test]
#[ignore = "v43 TLS/shared554, distant NPC search resumption at the legal service quota, render/audio/return; no HWND"]
fn java_v43_controllers_shared_distant_npc_pursuit_render_audio_and_return() {
    flow(true, true, true, false);
}
#[test]
#[ignore = "v43 TLS/shared554, actual target MOVE commands, bounded NPC pursuit/render/audio/return; no HWND"]
fn java_v43_controllers_shared_moving_targets_npc_pursuit_and_return() {
    flow(true, true, false, true);
}
#[test]
#[ignore = "v43 TLS/shared554, one-node service quota with moving targets, actual pursuit/render/audio/return; no HWND"]
fn java_v43_controllers_shared_low_quota_moving_targets_npc_pursuit_and_return() {
    flow_with_budget(true, true, false, true, true);
}
fn flow(motion:bool, cadence:bool, pursuit:bool, dynamic:bool) {
    flow_with_budget(motion, cadence, pursuit, dynamic, false);
}
fn flow_with_budget(motion:bool, cadence:bool, pursuit:bool, dynamic:bool, low_quota:bool) {
    let shared =
        PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"));
    assert!(shared.is_absolute());
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernNpcServiceTestPeer",
        if low_quota {"quest-dynamic-low-quota-shared-v43"} else if dynamic {"quest-dynamic-shared-v43"} else if pursuit {"quest-pursuit-shared-v43"} else if cadence {"quest-cadence-shared-v43"} else if motion {"quest-revival-shared-v43"} else {"quest-revival-shared-v40"},
        "60000",
    );
    let deadline = Instant::now() + Duration::from_secs(55);
    let profile=ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
        "schema":1,"profile_id":"canonical-revival-controller","display_name":"Isolated revival controller",
        "shared_resource_root":shared,"endpoint":peer.endpoint.to_string(),"tls_server_name":"localhost",
        "trust":{"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
        "connect_timeout_ms":500,"exchange_timeout_ms":3000
    })).unwrap()).unwrap();
    let mut clients = std::array::from_fn(|_| Client {
        controller: SessionController::new_canonical(profile.clone()),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        auto_load: false,
        loads: 0,
        motion,
    });
    for i in 0..2 {
        submit(&mut clients, i, deadline, "connect", || {
            LoginCommand::Connect
        });
        wait(&mut clients, deadline, "TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        submit(&mut clients, i, deadline, "login", || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut clients, deadline, "sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        submit(&mut clients, i, deadline, "source", || {
            LoginCommand::SelectSource(if i == 0 { 7 } else { 42 })
        });
        wait(&mut clients, deadline, "roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        let roster = clients[i].view();
        let context = SelectionContext {
            key: application::CharacterKey {
                source_id: roster.selected_source.unwrap(),
                object_id: 3333,
            },
            epoch: roster.roster_epoch,
        };
        assert!(roster.characters.iter().any(|c| c.object_id == 3333));
        submit(&mut clients, i, deadline, "selection", || {
            LoginCommand::SelectCharacter(context)
        });
        wait(&mut clients, deadline, "catalogs", |cs| {
            let v = cs[i].view();
            v.phase == LoginPhase::ChoosingChannel && !v.record_pending && !v.spell_pending
        });
        submit(&mut clients, i, deadline, "channel", || {
            LoginCommand::EnterChannel {
                context,
                channel_id: 0,
            }
        });
        wait(&mut clients, deadline, "lobby", |cs| {
            let v = cs[i].view();
            v.phase == LoginPhase::InLobby && v.canonical.supported() && !v.room_pending
        });
    }
    let contexts = clients
        .each_ref()
        .map(|c| c.view().navigation.unwrap().context);
    room(
        &mut clients,
        0,
        deadline,
        rooms::Request::draft(
            rooms::Common {
                action: rooms::Action::Create,
                ..rooms::Common::read(contexts[0], 0)
            },
            rooms::Draft::new(
                rooms::PublicDraft {
                    capacity: 4,
                    mode: 2,
                    map_id: 554,
                    attributes: 16,
                    name: "Canonical revival".into(),
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    let summary = clients[0].view().rooms.joined().unwrap().summary.clone();
    room(
        &mut clients,
        1,
        deadline,
        rooms::Request::join(
            rooms::Common {
                action: rooms::Action::Join,
                room_id: summary.room_id,
                expected_revision: summary.revision,
                ..rooms::Common::read(contexts[1], 0)
            },
            String::new(),
        )
        .unwrap(),
    );
    for i in 0..2 {
        room(
            &mut clients,
            i,
            deadline,
            rooms::Request::read(contexts[i], 0).unwrap(),
        );
        let summary = clients[i].view().rooms.joined().unwrap().summary.clone();
        room(
            &mut clients,
            i,
            deadline,
            room_change(contexts[i], &summary, rooms::Action::Ready, 1),
        );
    }
    wait(&mut clients, deadline, "qualified loading", |cs| {
        cs.iter().all(|c| {
            c.surface
                .as_ref()
                .is_some_and(|s| battle_player::load_ready(&c.runtime, s, &c.view()))
        })
    });
    for c in &mut clients {
        c.auto_load = true;
    }
    wait(&mut clients, deadline, "active", |cs| {
        cs.iter().all(|c| {
            c.controls_ready()
                && c.view()
                    .canonical
                    .lifecycle()
                    .is_some_and(|l| l.phase == world::Phase::Active)
        })
    });
    assert!(clients.iter().all(|c| c.loads == 1));
    let keys = clients.each_ref().map(|c| {
        c.view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .world
            .identity
            .clone()
    });
    if cadence {
        verify_live_npc_cadence(&mut clients, deadline, dynamic, low_quota);
    } else {
    wait(&mut clients, deadline, "both dead", |cs| {
        cs.iter().all(|c| {
            c.surface
                .as_ref()
                .is_some_and(|s| battle_player::revival_command(s, &c.view()).is_some())
        })
    });
    for i in 0..2 {
        wait(&mut clients, deadline, "painted death available", |cs| {
            cs[i]
                .surface
                .as_ref()
                .is_some_and(|s| battle_player::revival_command(s, &cs[i].view()).is_some())
        });
        let dead = clients[i].surface.as_ref().unwrap().clone();
        let before = clients[i].view();
        let q = battle_player::revival_command(&dead, &before)
            .expect("explicit painted death confirmation");
        let me = dead
            .observation
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == dead.observation.data().personal.entity)
            .unwrap();
        let old_move = me.last_move_sequence;
        let old_attack = me.vital.as_ref().unwrap().attack_sequence;
        submit(&mut clients, i, deadline, "confirm death", || {
            LoginCommand::Canonical(q.clone())
        });
        let pending = clients[i].view();
        assert!(pending.world_pending);
        assert!(
            battle_revival::Request::from_observation(pending.canonical.observation().unwrap())
                .is_some(),
            "no optimistic revival"
        );
        assert!(
            clients[i]
                .controller
                .command(LoginCommand::Canonical(q.clone()))
                .is_err(),
            "duplicate pending confirmation"
        );
        wait(
            &mut clients,
            deadline,
            "new-life receipt and picture",
            |cs| {
                let v = cs[i].view();
                !v.world_pending
                    && v.canonical.outcome()
                        == Some(Outcome::Revival(battle_revival::Reason::Revived))
                    && cs[i].controls_ready()
                    && v.canonical
                        .observation()
                        .is_some_and(|o| Stamp::of(o).life == 2)
            },
        );
        let alive = clients[i].view();
        let o = alive.canonical.observation().unwrap();
        assert!(battle_revival::Request::from_observation(o).is_none());
        assert!(!battle_player::admitted(&dead, &alive));
        assert!(battle_player::revival_command(&dead, &alive).is_none());
        assert!(
            clients[i]
                .controller
                .command(LoginCommand::Canonical(q))
                .is_err(),
            "old life cannot confirm twice"
        );
        let data = o.data();
        let me = data
            .actors
            .0
            .iter()
            .find(|a| a.entity == data.personal.entity)
            .unwrap();
        let life = data
            .quest
            .as_ref()
            .unwrap()
            .lives
            .0
            .iter()
            .find(|a| a.entity == me.entity)
            .unwrap();
        assert_eq!((life.deaths, life.respawns, life.hp_gain), (1, 1, 1));
        assert_eq!(me.position, life.spawn);
        assert_eq!(me.vital.as_ref().unwrap().values.hp, 1);
        assert_eq!(me.vital.as_ref().unwrap().attack_sequence, old_attack);
        assert_eq!(me.last_move_sequence, old_move + 1);
        let from = WorldCell {
            x: me.position.x,
            y: me.position.y,
        };
        let picture = clients[i].surface.as_ref().unwrap().clone();
        let (heading, to) =
            DIRECTIONS
                .iter()
                .enumerate()
                .map(|(h, &(x, y))| {
                    (
                        h as u8,
                        WorldCell {
                            x: from.x + x,
                            y: from.y + y,
                        },
                    )
                })
                .find(|(_, to)| {
                    picture
                        .navigation
                        .as_ref()
                        .unwrap()
                        .map()
                        .can_step(from, *to)
                        && data
                            .actors
                            .0
                            .iter()
                            .all(|a| a.position.x != to.x || a.position.y != to.y)
                        && data
                            .npcs
                            .as_ref()
                            .unwrap()
                            .actors
                            .0
                            .iter()
                            .all(|a| a.hp == 0 || a.cell.x != to.x || a.cell.y != to.y)
                        && data.quest.as_ref().unwrap().objectives.0.iter().all(|a| {
                            a.hp == 0 || a.target.cell.x != to.x || a.target.cell.y != to.y
                        })
                })
                .expect("qualified free neighbor after revival");
        let movement = world::Request {
            action: world::Action::Move,
            match_id: keys[i].match_id as u64,
            generation: keys[i].generation as u64,
            sequence: me.last_move_sequence as u64 + 1,
            from_x: from.x,
            from_y: from.y,
            heading,
            ..world::Request::read(contexts[i], 0)
        };
        let move_command = crate::modern_lito::battle_input::command(
            &picture,
            crate::modern_lito::battle_window::Intent {
                stamp: Stamp::of(&picture.observation),
                kind: crate::modern_lito::battle_window::Kind::Local(
                    crate::modern_lito::battle_window::Local::Step(heading),
                ),
            },
            &alive,
        )
        .unwrap();
        assert_eq!(
            move_command,
            PlayerCommand::operation(
                Guard {
                    stamp: Stamp::of(&picture.observation),
                    target: None
                },
                Operation::Move(movement)
            )
        );
        submit(&mut clients, i, deadline, "new-life movement", || {
            LoginCommand::Canonical(move_command.clone())
        });
        wait(
            &mut clients,
            deadline,
            "movement receipt and picture",
            |cs| !cs[i].view().world_pending && cs[i].controls_ready(),
        );
        let moved = clients[i].view();
        assert_eq!(
            moved.canonical.outcome(),
            Some(Outcome::Operation(
                crate::modern_lito::battle_action::Rejection::None,
                Disposition::Applied,
                OperationReason::None
            ))
        );
        let o = moved.canonical.observation().unwrap();
        let me = o
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == o.data().personal.entity)
            .unwrap();
        assert_eq!((me.position.x, me.position.y), (to.x, to.y));
        assert_eq!(Stamp::of(o).life, 2);
        assert!(!battle_player::controls_ready(&picture, &moved));
    }
    }
    for i in 0..2 {
        let q = PlayerCommand::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], 0, Some(&keys[i]))
        });
        submit(&mut clients, i, deadline, "leave world", || {
            LoginCommand::Canonical(q.clone())
        });
        assert!(
            clients[i].view().canonical.observation().is_none(),
            "pending leave hides presentation, not an ACK"
        );
        wait(&mut clients, deadline, "leave receipt", |cs| {
            !cs[i].view().world_pending && !cs[i].view().canonical.attached()
        });
        assert!(clients[i].runtime.observation().is_none());
    }
    for i in 0..2 {
        room(
            &mut clients,
            i,
            deadline,
            rooms::Request::read(contexts[i], 0).unwrap(),
        );
        let summary = clients[i].view().rooms.joined().unwrap().summary.clone();
        assert_eq!(summary.ready, 0);
        room(
            &mut clients,
            i,
            deadline,
            room_change(contexts[i], &summary, rooms::Action::Leave, 0),
        );
        assert!(clients[i].view().rooms.joined().is_none());
        submit(&mut clients, i, deadline, "return characters", || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut clients, deadline, "roster return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    for c in &mut clients {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    loop {
        for c in &mut clients {
            c.controller.poll();
            c.runtime.poll();
        }
        if clients
            .iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
        {
            break;
        }
        assert!(Instant::now() < deadline, "v40 owned workers retire");
        thread::sleep(Duration::from_millis(10));
    }
    assert!(clients
        .iter()
        .all(|c| c.view().phase == LoginPhase::Disconnected));
    peer.expect_successful_exit();
}

fn verify_live_npc_cadence(clients: &mut [Client; 2], deadline: Instant, dynamic: bool, low_quota: bool) {
    use crate::modern_lito::battle_payload::data as d;
    use std::collections::{BTreeMap, BTreeSet};
    let mut revisions: [BTreeSet<i64>; 2] = std::array::from_fn(|_| BTreeSet::new());
    let mut sequences: [BTreeMap<i64, i64>; 2] = std::array::from_fn(|_| BTreeMap::new());
    let mut attacks: [BTreeSet<(i64, i64)>; 2] = std::array::from_fn(|_| BTreeSet::new());
    let mut audio: [BTreeSet<u64>; 2] = std::array::from_fn(|_| BTreeSet::new());
    let mut coalesced = [0; 2];
    let mut walking_pictures = [0; 2];
    let mut target_moves = [0; 2];
    let mut concurrent_chase_steps = [0; 2];
    let mut pending_moves = [None; 2];
    let mut next_moves = [Instant::now(); 2];
    // The new quota-one fixture keeps targets moving for eight commands;
    // existing quota-64 and static cadence timings stay unchanged.
    let required_moves = if low_quota { 8 } else { 4 };
    let seconds = if low_quota { 8 } else if dynamic { 6 } else { 4 };
    let end = Instant::now() + Duration::from_secs(seconds);
    while Instant::now() < end {
        assert!(Instant::now() < deadline, "live NPC cadence deadline");
        for (i,c) in clients.iter_mut().enumerate() {
            c.pump();
            if dynamic {
                drive_target_walk(c, &mut target_moves[i], &mut pending_moves[i], &mut next_moves[i], required_moves);
            }
            let v = c.view();
            let o = v.canonical.observation().expect("live canonical observation");
            let data = o.data();
            assert_eq!(data.world.phase, d::Phase::Active);
            revisions[i].insert(data.world.revision);
            let motion = data.npc_motion.as_ref().unwrap().state.as_ref().unwrap();
            for a in &motion.actors.0 {
                let sequence = a.last_walk.as_ref().map_or(0, |w| w.sequence);
                if let Some(previous) = sequences[i].insert(a.serial, sequence) {
                    assert!(sequence >= previous);
                    if sequence - previous > 1 { coalesced[i] += 1; }
                    if dynamic && (target_moves[i] < required_moves || pending_moves[i].is_some()) {
                        concurrent_chase_steps[i] += sequence - previous;
                    }
                }
            }
            for e in &data.npcs.as_ref().unwrap().exchanges.0 {
                if e.direction == d::Direction::NpcToParticipant { attacks[i].insert((e.serial,e.revision)); }
            }
            if let Some(s) = &c.surface {
                assert!(battle_player::admitted(s, &v));
                // Inspect the actual asynchronous worker's admitted picture,
                // not a separately constructed Motion/Scene in this test.
                let drawn = s.observation.data();
                let facts = drawn.npc_motion.as_ref().unwrap().state.as_ref().unwrap();
                for npc in &drawn.npcs.as_ref().unwrap().actors.0 {
                    let p = s.geometry.npcs.iter().find(|p| p.id.serial == npc.serial as u64).unwrap();
                    assert_eq!((p.cell.x,p.cell.y),(npc.cell.x,npc.cell.y));
                    if npc.action == d::NpcAction::Walk && p.source_frame != 0 {
                        let w = facts.actors.0.iter().find(|a|a.serial==npc.serial).unwrap().last_walk.as_ref().unwrap();
                        let origin = s.geometry.projection.cell_to_pixel(WorldCell{x:w.from.x,y:w.from.y}).unwrap();
                        let destination = s.geometry.projection.cell_to_pixel(p.cell).unwrap();
                        if p.anchor == origin && origin != destination { walking_pictures[i] += 1; }
                    }
                }
                if let Some(frame) = &s.audio {
                    for event in frame.events.iter() {
                        let bank = event.bank.as_ref().unwrap_or(&frame.bank);
                        assert!(event.group.candidates().iter().all(|id| *id==0 || bank.get(*id).is_some_and(|s| !s.pcm_bytes().is_empty())));
                        audio[i].insert(event.sequence);
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    let reads = revisions.each_ref().map(BTreeSet::len);
    let strikes = attacks.each_ref().map(BTreeSet::len);
    let sounds = audio.each_ref().map(BTreeSet::len);
    eprintln!("v43 actual TLS/shared554, seconds={seconds}, dynamic={dynamic}, lowQuota={low_quota}, confirmed target moves={target_moves:?}, concurrent chase steps={concurrent_chase_steps:?}: observations={reads:?}, coalesced={coalesced:?}, moving worker pictures={walking_pictures:?}, confirmed NPC attacks={strikes:?}, qualified audio events={sounds:?}; audio counts include PC hurt/NPC cues, not hardware playback");
    for i in 0..2 {
        assert!(reads[i] >= 15, "200ms reads must continue: {reads:?}");
        assert!(coalesced[i] > 0 && walking_pictures[i] > 0, "real merged updates must animate their confirmed last step: {coalesced:?}/{walking_pictures:?}");
        assert!(strikes[i] >= 2 && sounds[i] > 0, "live NPC attacks/audio were not observed: {strikes:?}/{sounds:?}");
        if dynamic {
            assert_eq!(target_moves[i], required_moves, "actual accepted movement, not fixture teleportation");
            assert!(pending_moves[i].is_none());
            assert!(concurrent_chase_steps[i] > 0, "NPC must advance while its target moves, not only afterwards");
            assert!(sequences[i].values().any(|s| *s >= 10), "NPC completed a real approach");
        }
    }
}

// Real, spaced controller moves change the NPC's target geometry while
// it approaches; then the player holds still for attack/audio observations.
// Coordinates are read from admitted shared-map pictures, never injected into
// server state. Both clients stay in their independently owned Quest lanes.
fn drive_target_walk(c: &mut Client, count: &mut usize,
    pending: &mut Option<(WorldCell, i64)>, next: &mut Instant, required_moves: usize) {
    let view = c.view();
    if view.world_pending { return; }
    if let Some((to, sequence)) = pending.take() {
        assert_eq!(view.canonical.outcome(), Some(Outcome::Operation(
            crate::modern_lito::battle_action::Rejection::None,
            Disposition::Applied, OperationReason::None)));
        let data = view.canonical.observation().unwrap().data();
        let me = data.actors.0.iter().find(|a| a.entity == data.personal.entity).unwrap();
        assert_eq!((me.position.x, me.position.y, me.last_move_sequence), (to.x, to.y, sequence));
        *count += 1;
        *next = Instant::now() + Duration::from_millis(220);
    }
    if *count == required_moves || Instant::now() < *next || !c.controls_ready() { return; }
    let picture = c.surface.as_ref().unwrap();
    let data = picture.observation.data();
    let me = data.actors.0.iter().find(|a| a.entity == data.personal.entity).unwrap();
    let from = WorldCell { x: me.position.x, y: me.position.y };
    let npcs = &data.npcs.as_ref().unwrap().actors.0;
    // ACTIVE admission can precede the first server-owned wave publication.
    // Wait for actual actors; never fabricate an NPC or move before it exists.
    if !npcs.iter().any(|a| a.hp > 0) { return; }
    let (heading, to) = DIRECTIONS.iter().enumerate().map(|(h, &(x,y))|
        (h as u8, WorldCell{x:from.x+x, y:from.y+y}))
        .filter(|(_, to)| picture.navigation.as_ref().unwrap().can_step(from, *to)
            && data.actors.0.iter().all(|a| (a.position.x,a.position.y)!=(to.x,to.y))
            && npcs.iter().all(|a| a.hp==0 || (a.cell.x,a.cell.y)!=(to.x,to.y))
            && data.quest.as_ref().unwrap().objectives.0.iter().all(|a| a.hp==0 || (a.target.cell.x,a.target.cell.y)!=(to.x,to.y)))
        .max_by_key(|(_,to)| npcs.iter().filter(|a|a.hp>0).map(|a|
            (to.x-a.cell.x).abs().max((to.y-a.cell.y).abs())).min().unwrap())
        .expect("free shared-map target movement away from the pursuing NPC");
    let Ok(command) = crate::modern_lito::battle_input::command(picture,
        crate::modern_lito::battle_window::Intent{stamp:Stamp::of(&picture.observation),
            kind:crate::modern_lito::battle_window::Kind::Local(
                crate::modern_lito::battle_window::Local::Step(heading))}, &view) else { return; };
    let expected_sequence = me.last_move_sequence + 1;
    if c.controller.command(LoginCommand::Canonical(command)).is_ok() {
        *pending = Some((to,expected_sequence));
        *next = Instant::now() + Duration::from_millis(220);
    }
}
