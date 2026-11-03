//! Opt-in v8 supplies -> actual world -> healing -> durable SQL result.
//! Synthetic source characters/catalogs; fresh isolated settlement MySQL only.
//! Both TLS owners use independent real shared-resource renderers, not a GUI.
//! The parent harness assigns the Java tree to its owned Job before sending GO.

use super::super::{item_protocol as item, shop_protocol as shop};
use super::*;

const RED: i64 = 40010;
const ORANGE: i64 = 40011;
const WHITE: i64 = 40012;

fn owner_shop(
    owner: &mut SessionController,
    deadline: &Deadline,
    context: SelectionContext,
    action: shop::Action,
    item_id: i64,
    quantity: i64,
) -> LoginSnapshot {
    let before = owner.snapshot();
    let prior = before.shop.snapshot().cloned();
    let read = shop::Request::read(context, CHANNEL, before.rooms.joined());
    let request = if action == shop::Action::Read {
        read
    } else {
        read.mutation(
            prior.as_ref().expect("confirmed wallet before mutation"),
            action,
            item_id,
            quantity,
        )
    };
    submit(owner, deadline, "one admitted shop request", || {
        LoginCommand::Shop(request)
    });
    let pending = owner.snapshot();
    assert!(pending.shop_pending);
    assert_eq!(
        pending.shop.snapshot(),
        prior.as_ref(),
        "no optimistic charge or stock change before ACK"
    );
    // submit retries only command() errors before admission. No accepted BUY or
    // SELL is replayed, even if a deadline or connection failure loses its ACK.
    let confirmed = wait_owner(owner, deadline, "authoritative shop receipt", |v| {
        !v.shop_pending && v.shop.reply().is_some_and(|r| r.echo == request)
    });
    let reply = confirmed.shop.reply().unwrap();
    assert_eq!(
        (reply.status, reply.reason),
        (shop::Status::Ok, shop::Reason::None)
    );
    assert_eq!(reply.echo.context, context);
    let room = reply
        .room
        .as_ref()
        .expect("shop ACK includes real room READ");
    assert_eq!(room.status, rooms::Status::Ok);
    assert_eq!(room.joined.as_ref(), confirmed.rooms.joined());
    if action != shop::Action::Read {
        let after = confirmed.shop.snapshot().unwrap();
        assert_eq!(after.round_id, request.round_id);
        assert_eq!(
            after.revision,
            request.expected_revision.checked_add(1).unwrap()
        );
        assert_eq!(after.token, request.token);
    }
    confirmed
}

fn assert_bag(snapshot: &shop::Snapshot, balance: i64, expected: &[(i64, i64, i64)]) {
    assert_eq!(snapshot.allowance, 20_000);
    assert_eq!(snapshot.balance, balance);
    assert_eq!(
        snapshot
            .stacks
            .iter()
            .map(|s| (s.item_id, s.quantity, s.unit_paid_price))
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
#[ignore = "requires v8 Java items peer, explicit owned MySQL fixture and real shared assets; no production DB, GUI or audio device"]
fn java_items_shop_buy_sell_unready_real_healing_result_reset_and_isolated_sql_records() {
    let deadline = Deadline(Instant::now() + Duration::from_millis(60_000));
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernCombatTestPeer",
        "items",
    );
    assert!(
        peer.process_job.is_some(),
        "SQL may start only inside the gated owned Job"
    );
    let mut owner = SessionController::new(profile(peer.endpoint, &shared, &ca));
    submit(&mut owner, &deadline, "items connect", || {
        LoginCommand::Connect
    });
    wait_owner(&mut owner, &deadline, "items verified TLS", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, &deadline, "items AUTH", || {
        LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        }
    });
    wait_owner(&mut owner, &deadline, "items sources", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, &deadline, "items Sun source", || {
        LoginCommand::SelectSource(7)
    });
    let roster = wait_owner(&mut owner, &deadline, "items Sun roster", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    assert_eq!(roster.characters.len(), 1);
    assert_eq!(
        CharacterKey {
            source_id: roster.characters[0].source_id,
            object_id: roster.characters[0].object_id,
        },
        SUN
    );
    let context = SelectionContext {
        key: SUN,
        epoch: roster.roster_epoch,
    };
    submit(&mut owner, &deadline, "items select", || {
        LoginCommand::SelectCharacter(context)
    });
    wait_owner(&mut owner, &deadline, "items channels", |v| {
        v.phase == LoginPhase::ChoosingChannel
    });
    let absent = record::Totals {
        stored: false,
        wins: 0,
        losses: 0,
        revision: 0,
    };
    let initial = wait_owner(&mut owner, &deadline, "SQL absent Sun record", |v| {
        !v.record_pending && v.record.context() == Some(context) && v.record.totals().is_some()
    });
    assert_eq!(initial.record.totals(), Some(absent));
    let learned = wait_owner(&mut owner, &deadline, "qualified Sun skills", |v| {
        !v.spell_pending && v.skills.teleport().is_some() && v.skills.light_arrow().is_some()
    });
    assert_eq!(learned.skills.teleport().unwrap().mp_cost, 10);
    assert_eq!(learned.skills.light_arrow().unwrap().mp_cost, 3);
    submit(&mut owner, &deadline, "items lobby", || {
        LoginCommand::EnterChannel {
            context,
            channel_id: CHANNEL,
        }
    });
    wait_owner(&mut owner, &deadline, "items lobby ACK", |v| {
        v.phase == LoginPhase::InLobby
    });

    let mut guest = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(&ca),
        Hello::current(100),
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(2),
            io_poll_interval: Duration::from_millis(25),
        },
        Cancellation::new(),
    )
    .unwrap();
    assert_eq!(
        (
            guest.ack().unwrap().version,
            guest.ack().unwrap().capabilities
        ),
        (8, 1023)
    );
    assert!(guest.supports_shop() && guest.supports_record() && guest.supports_combat());
    guest
        .authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    let moon_roster = guest.characters(42, 1).unwrap();
    assert_eq!(moon_roster.characters.len(), 1);
    assert_eq!(moon_roster.characters[0].key, MOON);
    assert_eq!(SUN.object_id, MOON.object_id);
    assert_ne!(SUN.source_id, MOON.source_id);
    let moon_context = SelectionContext {
        key: MOON,
        epoch: 1,
    };
    guest.select_character(moon_context).unwrap();
    let moon_initial = guest.record().unwrap();
    assert_eq!(
        (
            moon_initial.context,
            moon_initial.status,
            moon_initial.totals
        ),
        (moon_context, record::Status::Ok, Some(absent))
    );
    let moon_skills = guest.skills().unwrap();
    assert_eq!(moon_skills.status, spell::Status::Ok);
    assert!(
        moon_skills.skills.is_empty(),
        "same numeric object is not the same source owner"
    );
    guest.enter_channel(CHANNEL).unwrap();

    let initial_shop = owner_shop(&mut owner, &deadline, context, shop::Action::Read, 0, 0);
    let first_wallet = initial_shop.shop.snapshot().unwrap().clone();
    assert_eq!(first_wallet.round_id, 1);
    assert_bag(&first_wallet, 20_000, &[]);
    assert_eq!(
        first_wallet
            .offers
            .iter()
            .map(|o| (o.item_id, o.buy_price))
            .collect::<Vec<_>>(),
        vec![(RED, 40), (ORANGE, 200), (WHITE, 600)]
    );
    assert!(first_wallet
        .offers
        .iter()
        .all(|o| o.support == shop::Support::Healing && o.healing.is_some()));
    assert_eq!(
        first_wallet
            .offer(RED)
            .unwrap()
            .healing
            .as_ref()
            .unwrap()
            .effect,
        189
    );
    let buy = owner_shop(&mut owner, &deadline, context, shop::Action::Buy, RED, 3);
    assert_bag(buy.shop.snapshot().unwrap(), 19_880, &[(RED, 3, 40)]);
    let sell = owner_shop(&mut owner, &deadline, context, shop::Action::Sell, RED, 1);
    assert_bag(sell.shop.snapshot().unwrap(), 19_920, &[(RED, 2, 40)]);
    let orange = owner_shop(&mut owner, &deadline, context, shop::Action::Buy, ORANGE, 1);
    assert_bag(
        orange.shop.snapshot().unwrap(),
        19_720,
        &[(RED, 2, 40), (ORANGE, 1, 200)],
    );
    let bought = owner_shop(&mut owner, &deadline, context, shop::Action::Buy, WHITE, 1);
    let purchased = bought.shop.snapshot().unwrap().clone();
    assert_bag(
        &purchased,
        19_120,
        &[(RED, 2, 40), (ORANGE, 1, 200), (WHITE, 1, 600)],
    );
    let moon_shop = guest
        .shop(shop::Request::read(moon_context, CHANNEL, None))
        .unwrap();
    assert_eq!(moon_shop.status, shop::Status::Ok);
    let moon_wallet = moon_shop.snapshot.unwrap();
    assert_bag(&moon_wallet, 20_000, &[]);
    assert_eq!(moon_wallet.offer(RED).unwrap().buy_price, 80);
    assert_ne!(
        moon_wallet.token, purchased.token,
        "source-owned catalog prices stay separate"
    );

    submit(&mut owner, &deadline, "items CREATE", || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(context, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Owned supplies TLS interop".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    let created = wait_owner(&mut owner, &deadline, "items creator JOIN", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    });
    let room_id = created.rooms.joined().unwrap().summary.room_id;
    assert_eq!(created.rooms.joined().unwrap().owner, SUN);
    let directory = guest
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap();
    let summary = directory
        .rooms
        .iter()
        .find(|r| r.room_id == room_id)
        .unwrap();
    let joined = guest
        .rooms(
            rooms::Request::join(
                rooms::Common {
                    context: moon_context,
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
    assert_eq!(joined.status, rooms::Status::Ok);
    assert_eq!(joined.joined.unwrap().members.len(), 2);
    let seen = wait_owner(&mut owner, &deadline, "items guest refresh", |v| {
        v.rooms.joined().is_some_and(|r| r.members.len() == 2)
    });
    let ready_room = seen.rooms.joined().unwrap().clone();
    submit(&mut owner, &deadline, "Sun READY before shop", || {
        LoginCommand::Rooms(room_command(context, &ready_room, rooms::Action::Ready, 1))
    });
    let ready = wait_owner(&mut owner, &deadline, "Sun READY receipt", |v| {
        !v.room_pending
            && v.rooms.joined().is_some_and(|r| {
                r.members
                    .iter()
                    .any(|m| m.matches_character(SUN) && m.ready)
            })
    });
    let ready_revision = ready.rooms.joined().unwrap().summary.revision;
    let opened = owner_shop(&mut owner, &deadline, context, shop::Action::Read, 0, 0);
    let unready = opened.rooms.joined().unwrap();
    assert_eq!(unready.summary.room_id, room_id);
    assert_eq!(unready.summary.revision, ready_revision + 1);
    assert!(unready.members.iter().all(|m| !m.ready));
    assert!(!unready.summary.satisfied);
    assert_bag(
        opened.shop.snapshot().unwrap(),
        19_120,
        &[(RED, 2, 40), (ORANGE, 1, 200), (WHITE, 1, 600)],
    );
    // One extra actual READ proves reopening does not reapply an old purchase.
    let reopened = owner_shop(&mut owner, &deadline, context, shop::Action::Read, 0, 0);
    assert_eq!(reopened.shop.snapshot(), opened.shop.snapshot());
    assert_eq!(reopened.rooms.joined(), opened.rooms.joined());
    let ready_room = reopened.rooms.joined().unwrap().clone();
    submit(&mut owner, &deadline, "Sun ready for actual world", || {
        LoginCommand::Rooms(room_command(context, &ready_room, rooms::Action::Ready, 1))
    });
    wait_owner(&mut owner, &deadline, "Sun actual ready ACK", |v| {
        !v.room_pending
            && v.rooms.joined().is_some_and(|r| {
                r.members
                    .iter()
                    .any(|m| m.matches_character(SUN) && m.ready)
            })
    });
    let room = guest
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    assert_ne!(room.members[0].team, room.members[1].team);
    let all_ready = guest
        .rooms(room_command(moon_context, &room, rooms::Action::Ready, 1))
        .unwrap();
    assert_eq!(all_ready.status, rooms::Status::Ok);
    assert!(all_ready.joined.unwrap().summary.satisfied);

    let loading = wait_owner(
        &mut owner,
        &deadline,
        "items true PREPARE and fresh qualification",
        |v| {
            v.combat
                .as_ref()
                .is_some_and(|f| f.world.phase == world::Phase::Loading)
        },
    )
    .combat
    .unwrap();
    assert!(loading.vitals.iter().all(|v| v.hp == 100
        && v.max_hp == 100
        && v.mp == 100
        && v.max_mp == 100
        && v.weapon_type == 0
        && v.attack_sequence == 0
        && v.attack_revision == 0
        && v.death_revision == 0));
    let sun_entity = actor(&loading.world, SUN).entity_id;
    let moon_entity = actor(&loading.world, MOON).entity_id;
    assert_ne!(sun_entity, moon_entity);
    assert_eq!(
        actor(&loading.world, SUN).position,
        WorldCell { x: 32724, y: 32784 }
    );
    assert_eq!(loading.items.len(), 2);
    assert_eq!(
        loading
            .item(sun_entity)
            .unwrap()
            .stocks
            .iter()
            .map(|s| (s.item_id, s.initial, s.remaining))
            .collect::<Vec<_>>(),
        vec![(RED, 2, 2), (ORANGE, 1, 1), (WHITE, 1, 1)]
    );
    assert!(loading.item(moon_entity).unwrap().stocks.is_empty());
    assert!(loading
        .items
        .iter()
        .all(|s| s.request_sequence == 0 && s.use_count == 0 && s.use_revision == 0));
    guest
        .world(world_request(
            moon_context,
            &loading.world,
            world::Action::Read,
        ))
        .unwrap();
    assert_eq!(
        guest
            .combat(combat::Request::read(moon_context, CHANNEL, &loading.world))
            .unwrap()
            .frame
            .unwrap(),
        *loading
    );
    let mut sun_runtime = WorldRuntime::new(shared.clone()).unwrap();
    let mut moon_runtime = WorldRuntime::new(shared).unwrap();
    let mut keepalive = Instant::now();
    combat_images_with_progress(
        &mut sun_runtime,
        &mut moon_runtime,
        &loading,
        &deadline,
        || {
            owner.poll();
            let view = owner.snapshot();
            assert_ne!(view.phase, LoginPhase::Failed);
            assert!(view
                .world
                .frame()
                .is_some_and(|f| f.phase == world::Phase::Loading));
            if keepalive.elapsed() >= Duration::from_secs(1) {
                guest.ping().unwrap();
                keepalive = Instant::now();
            }
        },
    );
    let loaded_request = world::Request {
        manifest_digest: loading.world.manifest.digest,
        ..world_request(context, &loading.world, world::Action::Loaded)
    };
    submit(
        &mut owner,
        &deadline,
        "items first real render LOAD",
        || LoginCommand::World(loaded_request),
    );
    wait_owner(&mut owner, &deadline, "combat owner LOAD ACK", |v| {
        !v.world_pending && v.world.frame().is_some_and(|f| actor(f, SUN).loaded)
    });
    let countdown_started = Instant::now();
    let countdown = guest
        .world(world::Request {
            manifest_digest: loading.world.manifest.digest,
            ..world_request(moon_context, &loading.world, world::Action::Loaded)
        })
        .unwrap();
    assert_eq!(countdown.frame.unwrap().phase, world::Phase::Countdown);
    let mut current = wait_owner(
        &mut owner,
        &deadline,
        "items real three second countdown",
        |v| {
            v.combat
                .as_ref()
                .is_some_and(|f| f.world.phase == world::Phase::Active)
        },
    )
    .combat
    .unwrap();
    assert!(countdown_started.elapsed() >= Duration::from_secs(3));
    let active_started = Instant::now();
    let original_image = combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);

    // First RNG use must be the server-chosen adjacent teleport, before any
    // attack, potion recovery roll or damage spell. No client destination.
    let before_tp = current.clone();
    let from = actor(&before_tp.world, SUN).position;
    submit(&mut owner, &deadline, "items first RNG teleport", || {
        LoginCommand::CastTeleport
    });
    assert!(owner.snapshot().spell_pending);
    assert_eq!(owner.snapshot().combat.as_ref(), Some(&before_tp));
    current = wait_owner(&mut owner, &deadline, "items atomic teleport", |v| {
        !v.spell_pending
            && v.combat
                .as_ref()
                .is_some_and(|f| f.world.cast(sun_entity).is_some_and(|c| c.cast_count == 1))
    })
    .combat
    .unwrap();
    let teleport = *current.world.cast(sun_entity).unwrap();
    assert_eq!(teleport.request_sequence, 1);
    assert_eq!(teleport.from, from);
    assert_eq!(teleport.to, actor(&current.world, SUN).position);
    assert_ne!(teleport.to, from);
    assert_eq!(current.vital(sun_entity).unwrap().mp, 90);
    assert_eq!(current.vital(sun_entity).unwrap().hp, 100);
    assert_eq!(current.items, before_tp.items);
    let teleported_image = combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);
    assert_eq!(teleported_image.projection.anchor_cell, teleport.to);
    assert_ne!(teleported_image.bgra.as_ref(), original_image.bgra.as_ref());
    let teleport_until =
        Instant::now() + Duration::from_millis(u64::from(teleport.cast_interval_millis).max(800));
    let mut teleport_sound = false;
    while Instant::now() < teleport_until {
        if let Some(image) = rendered(&mut sun_runtime, &current.world, SUN) {
            teleport_sound |= image
                .audio
                .as_ref()
                .is_some_and(|a| a.events.iter().any(|e| e.group.candidates().contains(&119)));
        }
        owner.poll();
        deadline.pause("real shared teleport presentation");
    }
    assert!(teleport_sound);
    let heading = DIRECTIONS
        .iter()
        .position(|&(dx, dy)| teleport.to.x + dx == from.x && teleport.to.y + dy == from.y)
        .expect("server RNG selects a legal adjacent teleport cell") as u8;
    let movement = world::Request {
        heading,
        sequence: actor(&current.world, SUN).sequence + 1,
        from_x: teleport.to.x,
        from_y: teleport.to.y,
        ..world_request(context, &current.world, world::Action::Move)
    };
    submit(&mut owner, &deadline, "items MOVE back", || {
        LoginCommand::World(movement)
    });
    current = wait_owner(&mut owner, &deadline, "items confirmed MOVE back", |v| {
        !v.world_pending
            && v.combat
                .as_ref()
                .is_some_and(|f| actor(&f.world, SUN).sequence == movement.sequence)
    })
    .combat
    .unwrap();
    assert_eq!(actor(&current.world, SUN).position, from);
    assert_eq!(current.world.cast(sun_entity).unwrap(), &teleport);
    let after_move =
        Instant::now() + Duration::from_nanos(actor(&current.world, SUN).step_interval_nanos);
    while Instant::now() < after_move {
        deadline.pause("original MOVE cross-action pace");
    }

    let mut moon_before = guest
        .combat(combat::Request::read(moon_context, CHANNEL, &current.world))
        .unwrap()
        .frame
        .unwrap();
    let mut next_attack = Instant::now();
    let mut attempts = 0;
    let wounded = loop {
        assert!(
            attempts < 20,
            "bounded real hit rolls must eventually produce healable HP"
        );
        while Instant::now() < next_attack {
            deadline.pause("Moon original one-second melee interval");
        }
        let sequence = moon_before.vital(moon_entity).unwrap().attack_sequence + 1;
        let attack = guest
            .combat(combat::Request {
                action: combat::Action::Attack,
                sequence,
                target_entity_id: sun_entity,
                ..combat::Request::read(moon_context, CHANNEL, &moon_before.world)
            })
            .unwrap();
        assert_eq!(
            (attack.status, attack.reason),
            (combat::Status::Ok, combat::Reason::None)
        );
        let after = attack.frame.unwrap();
        assert_eq!(after.vital(moon_entity).unwrap().attack_sequence, sequence);
        assert!(
            after.vital(moon_entity).unwrap().attack_revision
                > moon_before.vital(moon_entity).unwrap().attack_revision
        );
        assert_eq!(
            after.vital(moon_entity).unwrap().last_target_entity_id,
            sun_entity
        );
        assert_eq!(after.items, moon_before.items);
        assert!(
            after.vital(sun_entity).unwrap().hp > 0,
            "one hit must not be a lethal fixture"
        );
        attempts += 1;
        if after.vital(sun_entity).unwrap().hp < 100 {
            break after;
        }
        // A miss is a new successful authority event, not permission to replay
        // the same request. Keep HP unchanged and use the next scoped sequence.
        assert_eq!(after.vital(sun_entity), moon_before.vital(sun_entity));
        next_attack = Instant::now()
            + Duration::from_nanos(after.vital(moon_entity).unwrap().attack_interval_nanos);
        moon_before = after;
        owner.poll();
    };
    assert_eq!(wounded.items, current.items);
    current = wait_owner(
        &mut owner,
        &deadline,
        "items Sun observes Moon damage",
        |v| {
            v.combat
                .as_ref()
                .is_some_and(|f| f.world.revision >= wounded.world.revision)
        },
    )
    .combat
    .unwrap();
    assert_eq!(*current, wounded);
    let wounded_image = combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);
    let before_use = current.clone();
    use crate::modern_lito::{login_window::test_world_details_text, world_window::DetailsPage};
    let before_details =
        test_world_details_text(&owner.snapshot(), DetailsPage::Character).unwrap();
    let before_inventory =
        test_world_details_text(&owner.snapshot(), DetailsPage::Inventory).unwrap();
    let before_vital = before_use.vital(sun_entity).unwrap();
    assert!(before_details.contains(&format!(
        "HP：{} / {}",
        before_vital.hp, before_vital.max_hp
    )));
    assert!(before_inventory.contains("補給 #40010：剩餘 2／開場 2"));
    assert!(test_world_details_text(&owner.snapshot(), DetailsPage::Skills).is_some());
    submit(&mut owner, &deadline, "single red potion admission", || {
        LoginCommand::UseItem { item_id: RED }
    });
    assert!(owner.snapshot().item_pending);
    assert_eq!(
        owner.snapshot().combat.as_ref(),
        Some(&before_use),
        "HP and stocks stay confirmed while USE is pending"
    );
    assert_eq!(
        test_world_details_text(&owner.snapshot(), DetailsPage::Character).unwrap(),
        before_details,
        "inspection must not predict HP before the server receipt"
    );
    assert_eq!(
        test_world_details_text(&owner.snapshot(), DetailsPage::Inventory).unwrap(),
        before_inventory,
        "inspection must not predict inventory deductions before the server receipt"
    );
    current = wait_owner(
        &mut owner,
        &deadline,
        "atomic red recovery and decrement",
        |v| {
            !v.item_pending
                && v.combat
                    .as_ref()
                    .is_some_and(|f| f.item(sun_entity).is_some_and(|i| i.use_count == 1))
        },
    )
    .combat
    .unwrap();
    let used = current.item(sun_entity).unwrap().clone();
    assert_eq!(
        (
            used.request_sequence,
            used.use_count,
            used.item_id,
            used.effect,
            used.delay_millis
        ),
        (1, 1, RED, 189, 500)
    );
    assert_eq!(used.use_revision, current.world.revision);
    assert_eq!(used.hp_before, before_use.vital(sun_entity).unwrap().hp);
    assert_eq!(used.hp_after, current.vital(sun_entity).unwrap().hp);
    assert!(used.hp_after > used.hp_before && used.hp_after <= 100);
    assert_eq!(used.restored, used.hp_after - used.hp_before);
    assert_eq!(used.position, actor(&current.world, SUN).position);
    assert_eq!(used.stock(RED).unwrap().remaining, 1);
    assert_eq!(used.stock(ORANGE).unwrap().remaining, 1);
    assert_eq!(used.stock(WHITE).unwrap().remaining, 1);
    let after_details = test_world_details_text(&owner.snapshot(), DetailsPage::Character).unwrap();
    let after_inventory =
        test_world_details_text(&owner.snapshot(), DetailsPage::Inventory).unwrap();
    assert!(after_details.contains(&format!("HP：{} / 100", used.hp_after)));
    assert!(after_inventory.contains("補給 #40010：剩餘 1／開場 2"));
    assert_ne!(after_details, before_details);
    assert_ne!(after_inventory, before_inventory);
    assert_eq!(
        current.vital(sun_entity).unwrap().mp,
        before_use.vital(sun_entity).unwrap().mp
    );
    assert_eq!(current.damages, before_use.damages);
    assert_eq!(current.world.casts, before_use.world.casts);
    assert_eq!(current.vital(moon_entity), before_use.vital(moon_entity));
    assert_eq!(
        guest
            .combat(combat::Request::read(moon_context, CHANNEL, &current.world))
            .unwrap()
            .frame
            .unwrap(),
        *current
    );
    let healing_image = combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);
    assert_eq!(
        healing_image
            .combat
            .as_ref()
            .unwrap()
            .item(sun_entity)
            .unwrap(),
        &used
    );
    assert!(
        healing_image.presentation_notice.is_none(),
        "qualified 189 overlay may not be silently skipped"
    );
    assert_ne!(healing_image.bgra.as_ref(), wounded_image.bgra.as_ref());
    // Runtime loaded and qualified the real shared 189/194/197 SPR/actions as
    // part of both first renders. Consume 189's native 625ms one-shot without
    // sending another USE. These healing definitions have no sound cue.
    let healing_until = Instant::now() + Duration::from_millis(700);
    while Instant::now() < healing_until {
        if let Some(image) = rendered(&mut sun_runtime, &current.world, SUN) {
            assert_eq!(
                image.combat.as_ref().unwrap().item(sun_entity).unwrap(),
                &used
            );
            assert!(image.presentation_notice.is_none());
            // Do not infer a potion WAV from this shared sound history, which
            // can still contain the attack. Pure healing-assets tests qualify
            // the cue-free 189 definition; this test checks the actual render.
        }
        owner.poll();
        deadline.pause("qualified healing overlay lifecycle");
    }
    let stable = guest
        .use_item(item::Request::read(moon_context, CHANNEL, &current.world))
        .unwrap();
    assert_eq!(stable.status, item::Status::Ok);
    assert_eq!(stable.frame.unwrap().item(sun_entity).unwrap(), &used);

    let fixed_positions = current
        .world
        .actors
        .iter()
        .map(|a| a.position)
        .collect::<Vec<_>>();
    let mut casts = 0;
    let mut next_cast = Instant::now();
    let mut light_arrow_sound = false;
    while current.vital(moon_entity).unwrap().hp > 0 {
        assert!(
            casts < 24,
            "bounded real server casts must reach terminal HP"
        );
        assert!(
            active_started.elapsed() < Duration::from_secs(45),
            "do not extend the peer's ACTIVE lifetime"
        );
        while Instant::now() < next_cast {
            deadline.pause("original shared spell cooldown");
        }
        let before = current.clone();
        let sequence = before.world.cast(sun_entity).unwrap().request_sequence + 1;
        submit(&mut owner, &deadline, "items light-arrow admission", || {
            LoginCommand::CastLightArrow {
                target_entity_id: moon_entity,
            }
        });
        assert!(owner.snapshot().spell_pending);
        assert_eq!(owner.snapshot().combat.as_ref(), Some(&before));
        current = wait_owner(
            &mut owner,
            &deadline,
            "items authoritative light-arrow ACK",
            |v| {
                !v.spell_pending
                    && v.combat.as_ref().is_some_and(|f| {
                        f.world.cast(sun_entity).unwrap().request_sequence == sequence
                    })
            },
        )
        .combat
        .unwrap();
        let damage = current.damage(sun_entity).unwrap();
        let previous_damage = before.damage(sun_entity).unwrap();
        assert_eq!(damage.cast_count, previous_damage.cast_count + 1);
        assert_eq!(
            (damage.skill_id, damage.target_entity_id, damage.mp_cost),
            (4, moon_entity, 3)
        );
        assert_eq!(damage.from, actor(&before.world, SUN).position);
        assert_eq!(damage.to, actor(&before.world, MOON).position);
        let damage_revision = current
            .world
            .completion
            .as_ref()
            .map_or(current.world.revision, |c| c.event.engine_revision + 1);
        assert_eq!(damage.cast_revision, damage_revision as u64);
        assert!(current.vital(moon_entity).unwrap().hp < before.vital(moon_entity).unwrap().hp);
        assert_eq!(
            current.vital(sun_entity).unwrap().mp,
            before.vital(sun_entity).unwrap().mp - 3
        );
        assert_eq!(
            100 - current.vital(sun_entity).unwrap().mp,
            10 + damage.mp_spent as i64
        );
        assert_eq!(current.vital(sun_entity).unwrap().hp, used.hp_after);
        assert_eq!(current.vital(sun_entity).unwrap().attack_sequence, 0);
        assert_eq!(current.vital(sun_entity).unwrap().attack_revision, 0);
        assert_eq!(current.world.cast(sun_entity).unwrap().cast_count, 1);
        assert_eq!(
            current.world.cast(sun_entity).unwrap().cast_revision,
            teleport.cast_revision
        );
        assert_eq!(current.item(sun_entity).unwrap(), &used);
        assert_eq!(
            current
                .world
                .actors
                .iter()
                .map(|a| a.position)
                .collect::<Vec<_>>(),
            fixed_positions
        );
        let other = guest
            .combat(combat::Request::read(moon_context, CHANNEL, &current.world))
            .unwrap()
            .frame
            .unwrap();
        assert_eq!(other.vitals, current.vitals);
        assert_eq!(other.damages, current.damages);
        assert_eq!(other.items, current.items);
        if current.vital(moon_entity).unwrap().hp == 0 {
            assert_eq!(other.world.phase, world::Phase::Finished);
            assert_eq!(
                other.world.completion.as_ref().unwrap().event,
                current.world.completion.as_ref().unwrap().event
            );
        } else {
            assert_eq!(other, *current);
        }
        next_cast = Instant::now()
            + Duration::from_millis(u64::from(
                current.world.cast(sun_entity).unwrap().cast_interval_millis,
            ));
        combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);
        let animation_until = Instant::now() + Duration::from_millis(1_200);
        while Instant::now() < animation_until {
            if let Some(image) = rendered(&mut sun_runtime, &current.world, SUN) {
                light_arrow_sound |= image
                    .audio
                    .as_ref()
                    .is_some_and(|a| a.events.iter().any(|e| e.group.candidates().contains(&97)));
            }
            owner.poll();
            deadline.pause("real shared 167/97 light-arrow presentation");
        }
        casts += 1;
    }
    assert!(casts >= 2);
    assert!(light_arrow_sound);
    assert_eq!(current.world.phase, world::Phase::Finished);
    assert_eq!(current.vital(moon_entity).unwrap().hp, 0);
    assert!(current.vital(moon_entity).unwrap().death_revision > 0);
    let saved = wait_owner(
        &mut owner,
        &deadline,
        "items durable SQL SAVED receipt",
        |v| {
            v.world.frame().is_some_and(|f| {
                f.phase == world::Phase::Finished
                    && f.completion
                        .as_ref()
                        .is_some_and(|c| c.state == world::SettlementState::Saved)
            })
        },
    );
    let terminal = saved.combat.unwrap();
    let completion = terminal.world.completion.as_ref().unwrap();
    assert_eq!(completion.event.participants.len(), 2);
    assert_eq!(
        (
            completion.event.participants[0].key,
            completion.event.participants[0].verdict
        ),
        (SUN, world::Verdict::Win)
    );
    assert_eq!(
        (
            completion.event.participants[1].key,
            completion.event.participants[1].verdict
        ),
        (MOON, world::Verdict::Loss)
    );
    assert_eq!(terminal.item(sun_entity).unwrap(), &used);
    let leave = world_request(context, &terminal.world, world::Action::Leave);
    submit(&mut owner, &deadline, "items world LEAVE", || {
        LoginCommand::World(leave)
    });
    let left = wait_owner(&mut owner, &deadline, "items LEAVE ACK", |v| {
        !v.world_pending && v.world.frame().is_none()
    });
    assert!(left.combat.is_none() && !left.item_pending && !left.combat_pending);
    let moon_terminal = guest
        .world(world_request(
            moon_context,
            &terminal.world,
            world::Action::Read,
        ))
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(
        moon_terminal.completion.as_ref().unwrap().state,
        world::SettlementState::Saved
    );
    assert_eq!(
        moon_terminal.completion.as_ref().unwrap().event,
        completion.event
    );
    guest
        .world(world_request(
            moon_context,
            &moon_terminal,
            world::Action::Leave,
        ))
        .unwrap();
    let returned = wait_owner(&mut owner, &deadline, "items same room after result", |v| {
        v.world.frame().is_none()
            && v.rooms
                .joined()
                .is_some_and(|r| r.summary.ready == 0 && !r.summary.satisfied)
    });
    assert_eq!(returned.rooms.joined().unwrap().summary.room_id, room_id);
    assert_eq!(returned.rooms.joined().unwrap().members.len(), 2);
    let reset = owner_shop(&mut owner, &deadline, context, shop::Action::Read, 0, 0);
    let wallet = reset.shop.snapshot().unwrap();
    assert_eq!(wallet.round_id, 2);
    assert_eq!(wallet.revision, purchased.revision + 1);
    assert_bag(wallet, 20_000, &[]);
    let reset_again = owner_shop(&mut owner, &deadline, context, shop::Action::Read, 0, 0);
    assert_eq!(
        reset_again.shop.snapshot(),
        Some(wallet),
        "duplicate READ/terminal never grants a second allowance"
    );

    let won = record::Totals {
        stored: true,
        wins: 1,
        losses: 0,
        revision: 1,
    };
    wait_owner(
        &mut owner,
        &deadline,
        "items Sun actual SQL record refresh",
        |v| {
            !v.record_pending
                && v.record.context() == Some(context)
                && v.record.totals() == Some(won)
        },
    );
    let lost = guest.record().unwrap();
    assert_eq!(
        (lost.context, lost.status),
        (moon_context, record::Status::Ok)
    );
    assert_eq!(
        lost.totals,
        Some(record::Totals {
            stored: true,
            wins: 0,
            losses: 1,
            revision: 1
        })
    );
    submit(
        &mut owner,
        &deadline,
        "items explicit SQL replay verification",
        || LoginCommand::ReadRecord,
    );
    let refreshed = wait_owner(&mut owner, &deadline, "items manual SQL record ACK", |v| {
        !v.record_pending
    });
    assert_eq!(
        refreshed.record.totals(),
        Some(won),
        "same UUID replay cannot award the same numeric object twice"
    );
    assert!(matches!(
        refreshed.record,
        record::State::Ready {
            refresh_failure: None,
            ..
        }
    ));

    sun_runtime.shutdown();
    moon_runtime.shutdown();
    while !sun_runtime.is_quiescent() || !moon_runtime.is_quiescent() {
        sun_runtime.poll();
        moon_runtime.poll();
        deadline.pause("items owned render worker retirement");
    }
    guest.close().unwrap();
    submit(&mut owner, &deadline, "items normal TLS shutdown", || {
        LoginCommand::Shutdown
    });
    while !owner.is_quiescent() {
        owner.poll();
        deadline.pause("items owned session retirement");
    }
    assert_eq!(owner.snapshot().phase, LoginPhase::Disconnected);
    for page in [
        DetailsPage::Character,
        DetailsPage::Inventory,
        DetailsPage::Skills,
    ] {
        assert!(
            test_world_details_text(&owner.snapshot(), page).is_none(),
            "disconnected inspection must retire its character data"
        );
    }
    // Peer additionally verifies >=10 fresh shop reads, >=5 actual SQL reads,
    // same-UUID SQL replay, no leaked room/world/backend workers and SQL exit.
    peer.expect_successful_exit(&deadline);
    deadline.check("complete items TLS/SQL flow");
}
