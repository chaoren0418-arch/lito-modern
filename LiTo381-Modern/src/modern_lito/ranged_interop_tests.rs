//! v11 / opt-in v36 ordinary bow + two arrow types over owned TLS/Server/SQL.
//! Source facts/RNG are explicit fixtures; shared resources are real. No GUI.
use super::super::{
    ammo_protocol as ammo, equipment, item_protocol as item, shop_protocol as shop,
    transport::ModernTlsConnection,
};
use super::*;
#[path = "canonical_ammo_interop.rs"]
mod canonical;

fn connect(
    peer: &JavaPeer,
    ca: &[u8],
    key: CharacterKey,
    canonical: bool,
    life_actions: bool,
) -> (ModernTlsConnection, SelectionContext) {
    let mut hello = Hello::current(100);
    if canonical {
        hello.max_version = if life_actions { 37 } else { 36 };
        hello.supported_capabilities |= crate::modern_lito::protocol::CAP_BATTLE_DATA;
        if life_actions {
            hello.supported_capabilities |= crate::modern_lito::protocol::CAP_ACTION_LIFE;
        }
    }
    let mut c = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(ca),
        hello,
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(2),
            io_poll_interval: Duration::from_millis(25),
        },
        Cancellation::new(),
    )
    .unwrap();
    if canonical {
        assert_eq!(c.ack().unwrap().version, if life_actions { 37 } else { 36 });
        assert!(c.supports_battle_data());
    } else {
        assert_eq!(
            (c.ack().unwrap().version, c.ack().unwrap().capabilities),
            (11, 8191)
        );
        assert!(!c.supports_battle_data());
    }
    assert_eq!(c.supports_battle_actions(), life_actions);
    c.authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    let roster = c.characters(key.source_id, 1).unwrap();
    assert_eq!(roster.characters[0].key, key);
    let context = SelectionContext { key, epoch: 1 };
    c.select_character(context).unwrap();
    assert_eq!(c.record().unwrap().totals.unwrap().revision, 0);
    c.skills().unwrap();
    let e = c
        .equipment(equipment::Request::read(context))
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(
        e.weight.as_ref().unwrap().source_gear_weight,
        if key == SUN { 2576 } else { 2575 }
    );
    c.enter_channel(CHANNEL).unwrap();
    (c, context)
}
fn shop_read(c: &mut ModernTlsConnection, context: SelectionContext) -> Arc<shop::Snapshot> {
    let room = c
        .rooms(rooms::Request::read(context, CHANNEL).unwrap())
        .unwrap()
        .joined;
    c.shop(shop::Request::read(context, CHANNEL, room.as_ref()))
        .unwrap()
        .snapshot
        .unwrap()
}
fn read(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    world: &world::Frame,
) -> combat::Frame {
    c.combat(combat::Request::read(context, CHANNEL, world))
        .unwrap()
        .frame
        .unwrap()
}
fn track(old: &combat::Frame, new: combat::Frame, context: SelectionContext) -> combat::Frame {
    let reply = |f: &combat::Frame| combat::Reply {
        echo: combat::Request::read(context, CHANNEL, &f.world),
        status: combat::Status::Ok,
        reason: combat::Reason::None,
        frame: Some(f.clone()),
    };
    assert!(
        combat::follows(Some(&reply(old)), &reply(&new)),
        "genuine complete ammo transition"
    );
    new
}
fn pause(deadline: &Deadline, duration: Duration) {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        deadline.pause("bow fixture server cadence");
    }
}
/// A genuine, authenticated canonical read supplies the admission proof.
/// This raw-transport fixture deliberately permits Loading/Finished proofs to
/// exercise server rejection; it does not pretend to be a painted UI click.
pub(super) fn action_guard(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    w: &world::Frame,
    target: Option<u64>,
) -> crate::modern_lito::battle_action::Guard {
    use crate::modern_lito::{
        battle_action::Guard,
        battle_follow::Follower,
        battle_payload::data::WorldId,
        battle_read,
        battle_window::{Stamp, Target},
    };
    assert!(c.supports_battle_actions());
    let request = battle_read::Request {
        context,
        channel_id: CHANNEL,
        world: WorldId {
            room: w.room.summary.room_id as i64,
            match_id: w.match_id as i64,
            generation: w.generation as i64,
        },
    };
    let mut follower = Follower::new(request.clone(), c.ack().unwrap()).unwrap();
    follower.accept(c.read_battle(request).unwrap()).unwrap();
    let observation = follower
        .observation()
        .expect("actual canonical action proof");
    let target = target.map(|id| {
        if let Some(a) = observation
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity as u64 == id)
        {
            Target::Player {
                entity: a.entity,
                life: crate::modern_lito::battle_window::life(a),
            }
        } else {
            assert!(observation
                .data()
                .honor
                .objectives
                .0
                .iter()
                .any(|a| a.entity as u64 == id));
            Target::Honor { entity: id as i64 }
        }
    });
    Guard {
        stamp: Stamp::of(observation),
        target,
    }
}
pub(super) fn arm_action(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    w: &world::Frame,
    target: Option<u64>,
) {
    if c.supports_battle_actions() {
        let guard = action_guard(c, context, w, target);
        c.arm_battle_action(guard).unwrap();
    }
}
fn select(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    old: &combat::Frame,
    id: i64,
    reason: ammo::Reason,
) -> combat::Frame {
    let current = track(old, read(c, context, &old.world), context);
    let entity = actor(&current.world, context.key).entity_id;
    let previous = current.ammo(entity).unwrap();
    arm_action(c, context, &current.world, None);
    let r = c
        .select_ammo(ammo::Request {
            action: ammo::Action::Select,
            sequence: previous.request_sequence + 1,
            item_id: id,
            ..ammo::Request::read(context, CHANNEL, &current.world)
        })
        .unwrap();
    assert_eq!(r.reason, reason);
    assert_eq!(
        r.status,
        if reason == ammo::Reason::None {
            ammo::Status::Ok
        } else {
            ammo::Status::Denied
        }
    );
    let next = track(&current, r.frame.unwrap(), context);
    let after = next.ammo(entity).unwrap();
    assert_eq!(after.request_sequence, previous.request_sequence + 1);
    assert_eq!(after.stocks, previous.stocks);
    assert_eq!(after.shot_count, previous.shot_count);
    assert_eq!(next.weights, current.weights);
    assert_eq!(next.items, current.items);
    assert_eq!(next.vitals, current.vitals);
    next
}
fn shoot(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    old: &combat::Frame,
    target: u64,
    reason: combat::Reason,
) -> combat::Frame {
    let current = track(old, read(c, context, &old.world), context);
    let entity = actor(&current.world, context.key).entity_id;
    let request = combat::Request {
        action: combat::Action::Attack,
        sequence: current.vital(entity).unwrap().attack_sequence + 1,
        target_entity_id: target,
        ..combat::Request::read(context, CHANNEL, &current.world)
    };
    arm_action(c, context, &current.world, Some(target));
    let reply = c.combat(request).unwrap();
    assert_eq!(reply.reason, reason);
    assert_eq!(
        reply.status,
        if reason == combat::Reason::None {
            combat::Status::Ok
        } else {
            combat::Status::Denied
        }
    );
    let next = track(&current, reply.frame.unwrap(), context);
    let before = current.ammo(entity).unwrap();
    let after = next.ammo(entity).unwrap();
    assert_eq!(next.items, current.items, "arrow is not a potion USE");
    if reason == combat::Reason::None {
        assert_eq!(after.shot_count, before.shot_count + 1);
        assert_eq!(after.shot_revision, next.world.revision);
        assert_eq!(after.from, actor(&current.world, context.key).position);
        assert_eq!(
            after.to,
            current
                .world
                .actors
                .iter()
                .find(|a| a.entity_id == target)
                .unwrap()
                .position
        );
        assert_eq!(after.attack_sequence, request.sequence);
        assert_eq!(after.projectile_sprite, 66);
    } else {
        assert_eq!(after, before);
        assert_eq!(next.weights, current.weights);
    }
    next
}
#[test]
#[ignore = "v11 Java ammo peer + isolated MySQL + real shared bow/arrow; no GUI or production writes"]
fn java_bow_select_miss_exhaust_switch_damage_weight_render_and_sql() {
    run_ammo(false, false);
}

#[test]
#[ignore = "v36 authenticated canonical combat + two workers + isolated MySQL; no GUI/production writes"]
fn java_canonical_bow_commands_public_events_private_views_workers_and_sql() {
    run_ammo(true, false);
}

#[test]
#[ignore = "v37 life-bound item/ammo/ranged TLS, canonical shared workers and isolated SQL; no HWND"]
fn java_v37_guarded_bow_supplies_ammo_damage_terminal_recovery_and_saved_sql() {
    run_ammo(true, true);
}

fn run_ammo(canonical: bool, life_actions: bool) {
    assert!(!life_actions || canonical);
    let deadline = Deadline(Instant::now() + Duration::from_secs(if canonical { 180 } else { 60 }));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernCombatTestPeer",
        if life_actions {
            "life-ammo"
        } else if canonical {
            "canonical-ammo"
        } else {
            "ammo"
        },
    );
    assert!(peer.process_job.is_some());
    let (mut sun, sc) = connect(&peer, &ca, SUN, canonical, life_actions);
    let (mut moon, mc) = connect(&peer, &ca, MOON, canonical, life_actions);
    let mut sw = shop_read(&mut sun, sc);
    let mut mw = shop_read(&mut moon, mc);
    for (c, context, wallet) in [(&mut sun, sc, &mut sw), (&mut moon, mc, &mut mw)] {
        assert_eq!(wallet.offers.len(), 9);
        for id in [
            40010, 40011, 40012, 40013, 40014, 40016, 40068, 40744, 40746,
        ] {
            if ammo::supported(id) {
                assert_eq!(wallet.offer(id).unwrap().support, shop::Support::Ammo);
            }
            let qty = if context.key == SUN {
                if id == 40744 {
                    2
                } else if id == 40746 {
                    8
                } else {
                    1
                }
            } else {
                1
            };
            let request = shop::Request::read(context, CHANNEL, None).mutation(
                wallet,
                shop::Action::Buy,
                id,
                qty,
            );
            let bought = c.shop(request).unwrap();
            assert_eq!(bought.status, shop::Status::Ok);
            *wallet = bought.snapshot.unwrap();
        }
        assert_eq!(wallet.weight.as_ref().unwrap().units.len(), 9);
    }
    for _ in 0..3 {
        assert_eq!(shop_read(&mut sun, sc), sw);
    }
    let created = sun
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sc, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Bow ammo TLS".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let room = created.joined.unwrap();
    let room_id = room.summary.room_id;
    let directory = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap();
    let summary = directory
        .rooms
        .iter()
        .find(|r| r.room_id == room_id)
        .unwrap();
    moon.rooms(
        rooms::Request::join(
            rooms::Common {
                context: mc,
                channel_id: CHANNEL,
                action: rooms::Action::Join,
                value: 0,
                room_id,
                expected_revision: summary.revision,
            },
            String::new(),
        )
        .unwrap(),
    )
    .unwrap();
    let room = sun
        .rooms(rooms::Request::read(sc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    sun.rooms(room_command(sc, &room, rooms::Action::Ready, 1))
        .unwrap();
    let room = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let ready = moon
        .rooms(room_command(mc, &room, rooms::Action::Ready, 1))
        .unwrap()
        .joined
        .unwrap();
    assert!(ready.summary.satisfied);
    let mut wf = sun
        .world(world::Request {
            context: sc,
            channel_id: CHANNEL,
            action: world::Action::Prepare,
            heading: 0,
            room_id,
            expected_room_revision: ready.summary.revision,
            match_id: 0,
            generation: 0,
            sequence: 0,
            from_x: 0,
            from_y: 0,
            manifest_digest: [0; 32],
        })
        .unwrap()
        .frame
        .unwrap();
    while wf.phase == world::Phase::Preparing {
        deadline.pause("bow qualified preparation");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert_eq!(wf.phase, world::Phase::Loading);
    moon.world(world_request(mc, &wf, world::Action::Read))
        .unwrap();
    let initial = Arc::new(read(&mut sun, sc, &wf));
    let sid = actor(&wf, SUN).entity_id;
    let mid = actor(&wf, MOON).entity_id;
    arm_action(&mut sun, sc, &wf, None);
    let early = sun.select_ammo(ammo::Request {
        action: ammo::Action::Select,
        sequence: 1,
        item_id: 40746,
        ..ammo::Request::read(sc, CHANNEL, &wf)
    });
    if life_actions {
        assert!(matches!(
            early,
            Err(
                crate::modern_lito::transport::TransportError::GuardRejected(
                    crate::modern_lito::battle_action::Rejection::NotActive
                )
            )
        ));
        let crate::modern_lito::application_protocol::Response::Combat(recovery) =
            sun.take_action_recovery().unwrap()
        else {
            panic!("loading bow rejection must return actual combat READ")
        };
        assert_eq!(recovery.echo.action, combat::Action::Read);
        assert_eq!(recovery.frame.as_ref().unwrap().ammo, initial.ammo);
        assert_eq!(
            recovery.frame.as_ref().unwrap().world.revision,
            initial.world.revision
        );
    } else {
        let early = early.unwrap();
        assert_eq!(early.reason, ammo::Reason::NotActive);
        assert_eq!(early.status, ammo::Status::Denied);
        assert_eq!(early.frame.as_ref().unwrap().ammo, initial.ammo);
        assert_eq!(
            early.frame.as_ref().unwrap().world.revision,
            initial.world.revision
        );
    }
    sun.ping().unwrap();
    assert_eq!(initial.vital(sid).unwrap().weapon_type, 4);
    assert_eq!(initial.ammo(sid).unwrap().selected_item_id, 40744);
    assert_eq!(initial.ammo(sid).unwrap().stocks.len(), 2);
    assert_eq!(initial.item(sid).unwrap().stocks.len(), 7);
    assert_eq!(initial.weight(sid), sw.weight.as_ref());
    assert_eq!(initial.weight(mid), mw.weight.as_ref());
    let mut sr = WorldRuntime::new(shared.clone()).unwrap();
    let mut mr = WorldRuntime::new(shared.clone()).unwrap();
    let mut keepalive = Instant::now();
    combat_images_with_progress(&mut sr, &mut mr, &initial, &deadline, || {
        if keepalive.elapsed() >= Duration::from_secs(1) {
            sun.ping().unwrap();
            moon.ping().unwrap();
            keepalive = Instant::now();
        }
    });
    // Canonical pixels never grant LOAD permission: the full legacy preflight
    // above remains mandatory. This opt-in tests live reads/presentation only.
    let mut observer = canonical.then(|| {
        canonical::Observer::new(
            &shared,
            &wf,
            sc,
            mc,
            sun.ack().unwrap(),
            moon.ack().unwrap(),
        )
    });
    if let Some(o) = &mut observer {
        o.check(&mut sun, &mut moon, &initial, &deadline);
    }
    sun.world(world::Request {
        manifest_digest: wf.manifest.digest,
        ..world_request(sc, &wf, world::Action::Loaded)
    })
    .unwrap();
    wf = moon
        .world(world::Request {
            manifest_digest: wf.manifest.digest,
            ..world_request(mc, &wf, world::Action::Loaded)
        })
        .unwrap()
        .frame
        .unwrap();
    while wf.phase != world::Phase::Active {
        deadline.pause("bow countdown");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    let mut current = track(&initial, read(&mut sun, sc, &wf), sc);
    if let Some(o) = &mut observer {
        o.check(&mut sun, &mut moon, &current, &deadline);
    }
    let use_req = item::Request {
        action: item::Action::Use,
        sequence: 1,
        item_id: 40013,
        ..item::Request::read(sc, CHANNEL, &current.world)
    };
    arm_action(&mut sun, sc, &current.world, None);
    let used = sun.use_item(use_req).unwrap();
    assert_eq!(used.status, item::Status::Ok);
    current = track(&current, used.frame.unwrap(), sc);
    if let Some(o) = &mut observer {
        o.check(&mut sun, &mut moon, &current, &deadline);
    }
    let hp = current.vital(mid).unwrap().hp;
    current = shoot(&mut sun, sc, &current, mid, combat::Reason::None);
    assert!(!current.ammo(sid).unwrap().hit);
    assert_eq!(current.vital(mid).unwrap().hp, hp);
    assert_eq!(
        current.ammo(sid).unwrap().stock(40744).unwrap().remaining,
        1
    );
    if let Some(o) = &mut observer {
        o.check(&mut sun, &mut moon, &current, &deadline);
    }
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    pause(&deadline, Duration::from_millis(800));
    current = shoot(&mut sun, sc, &current, mid, combat::Reason::None);
    assert!(current.ammo(sid).unwrap().hit);
    assert!(current.vital(mid).unwrap().hp < hp);
    assert_eq!(
        current.ammo(sid).unwrap().stock(40744).unwrap().remaining,
        0
    );
    assert_eq!(
        current.ammo(sid).unwrap().selected_item_id,
        40744,
        "never silently autoswitch"
    );
    assert_eq!(
        current.weight(sid).unwrap().total + 1,
        initial.weight(sid).unwrap().total - 3,
        "last arrow removes its stack; haste removes weight3"
    );
    if let Some(o) = &mut observer {
        o.check(&mut sun, &mut moon, &current, &deadline);
    }
    pause(&deadline, Duration::from_millis(800));
    current = shoot(&mut sun, sc, &current, mid, combat::Reason::NoAmmo);
    if let Some(o) = &mut observer {
        o.check(&mut sun, &mut moon, &current, &deadline);
    }
    current = select(&mut sun, sc, &current, 40010, ammo::Reason::Unsupported);
    current = select(&mut sun, sc, &current, 40744, ammo::Reason::Empty);
    current = select(&mut sun, sc, &current, 40746, ammo::Reason::None);
    assert_eq!(current.ammo(sid).unwrap().selected_item_id, 40746);
    // Same selected ID is an explicit acknowledged selection, not a shot/use.
    current = select(&mut sun, sc, &current, 40746, ammo::Reason::None);
    if let Some(o) = &mut observer {
        o.check(&mut sun, &mut moon, &current, &deadline);
    }
    while current.world.phase == world::Phase::Active {
        assert!(current.ammo(sid).unwrap().shot_count < 10);
        pause(&deadline, Duration::from_millis(800));
        current = shoot(&mut sun, sc, &current, mid, combat::Reason::None);
        let other = read(&mut moon, mc, &current.world);
        assert_eq!(other.ammo, current.ammo);
        assert_eq!(other.vitals, current.vitals);
        if let Some(o) = &mut observer {
            o.check(&mut sun, &mut moon, &current, &deadline);
        }
        combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    }
    assert_eq!(current.world.phase, world::Phase::Finished);
    assert!(current.ammo(sid).unwrap().shot_count > 2);
    assert_eq!(current.item(sid).unwrap().use_count, 1);
    assert_eq!(current.ammo(mid).unwrap().shot_count, 0);
    current = track(&current, read(&mut sun, sc, &current.world), sc);
    let terminal_read = sun
        .select_ammo(ammo::Request::read(sc, CHANNEL, &current.world))
        .unwrap();
    assert_eq!(terminal_read.status, ammo::Status::Ok);
    assert_eq!(terminal_read.reason, ammo::Reason::None);
    assert_eq!(terminal_read.frame.as_ref().unwrap().ammo, current.ammo);
    current = track(&current, terminal_read.frame.unwrap(), sc);
    arm_action(&mut sun, sc, &current.world, None);
    let late = sun.select_ammo(ammo::Request {
        action: ammo::Action::Select,
        sequence: current.ammo(sid).unwrap().request_sequence + 1,
        item_id: 40746,
        ..ammo::Request::read(sc, CHANNEL, &current.world)
    });
    if life_actions {
        assert!(matches!(
            late,
            Err(
                crate::modern_lito::transport::TransportError::GuardRejected(
                    crate::modern_lito::battle_action::Rejection::NotActive
                )
            )
        ));
        let crate::modern_lito::application_protocol::Response::Combat(recovery) =
            sun.take_action_recovery().unwrap()
        else {
            panic!("terminal bow rejection must return real combat READ")
        };
        assert_eq!(recovery.echo.action, combat::Action::Read);
        assert_eq!(recovery.frame.as_ref().unwrap().ammo, current.ammo);
        current = track(&current, recovery.frame.unwrap(), sc);
    } else {
        let late = late.unwrap();
        assert_eq!(late.reason, ammo::Reason::NotActive);
        assert_eq!(late.frame.as_ref().unwrap().ammo, current.ammo);
        current = track(&current, late.frame.unwrap(), sc);
    }
    sun.ping().unwrap();
    while current.world.completion.as_ref().unwrap().state != world::SettlementState::Saved {
        deadline.pause("bow durable receipt");
        current = track(&current, read(&mut sun, sc, &current.world), sc);
    }
    let won = sun.record().unwrap().totals.unwrap();
    let lost = moon.record().unwrap().totals.unwrap();
    assert_eq!((won.wins, won.losses, won.revision), (1, 0, 1));
    assert_eq!((lost.wins, lost.losses, lost.revision), (0, 1, 1));
    assert_eq!(sun.record().unwrap().totals, Some(won));
    if let Some(o) = &mut observer {
        o.check(&mut sun, &mut moon, &current, &deadline);
        o.verify_result(&current);
        o.shutdown(&deadline);
    }
    sun.world(world_request(sc, &current.world, world::Action::Leave))
        .unwrap();
    moon.world(world_request(mc, &current.world, world::Action::Read))
        .unwrap();
    moon.world(world_request(mc, &current.world, world::Action::Leave))
        .unwrap();
    let reset = shop_read(&mut sun, sc);
    assert_eq!((reset.round_id, reset.balance), (2, 20000));
    assert!(reset.stacks.is_empty());
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        deadline.pause("bow resource retirement");
    }
    sun.close().unwrap();
    moon.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
