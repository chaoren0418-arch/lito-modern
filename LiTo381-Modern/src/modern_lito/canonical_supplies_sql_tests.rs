//! Two v41 player controllers against production Main and owned SQL. The same
//! rendered-intent adapter as the HWND drives items/NPCs/objectives; no legacy
//! World/Combat frame, fake LOAD, production database or native UI claim.
use super::*;
#[path = "canonical_armor_sql_tests.rs"]
mod armor_sql;
#[path = "canonical_bots_sql_tests.rs"]
mod bots;
#[path = "canonical_haste_sql_tests.rs"]
mod haste_sql;
#[path = "canonical_player_healing_sql_tests.rs"]
mod healing_sql;
#[path = "canonical_honor_sql_tests.rs"]
mod honor;
#[path = "canonical_inventory_sql_tests.rs"]
mod inventory_sql;
#[path = "canonical_mixed_modes_sql_tests.rs"]
mod mixed_modes;
#[path = "canonical_race_sql_tests.rs"]
mod race;
#[path = "canonical_map_rotation_tests.rs"]
mod rotation;
#[path = "canonical_siege_sql_tests.rs"]
mod siege;
#[path = "canonical_slow_sql_tests.rs"]
mod slow_sql;
#[path = "canonical_target_haste_sql_tests.rs"]
mod target_haste_sql;
#[path = "canonical_target_haste_time_attack_sql_tests.rs"]
mod target_haste_time_attack;
#[path = "canonical_teleport_sql_tests.rs"]
mod teleport;
#[path = "canonical_time_attack_sql_tests.rs"]
mod time_attack;
use crate::modern_lito::{
    battle_action::Rejection,
    battle_input,
    battle_operation::{Disposition, Reason},
    battle_payload::data,
    battle_window::{self, Intent, Kind, Local, Target},
    character_data::CharacterDataState,
    matchmaking_protocol as matching, shop_protocol as shop,
};

fn source(c: &Client, context: SelectionContext) {
    let v = c.view();
    let CharacterDataState::Ready(s) = &v.character_data else {
        panic!("SQL source missing")
    };
    assert_eq!(s.context, context);
    assert_eq!(s.items.len(), 2);
    assert_eq!(s.items[0].object_id, 1001);
    assert!(s.items[0].equipped);
    assert_eq!(
        s.items[1].quantity,
        if context.key.source_id == 7 { 2 } else { 5 }
    );
    assert_eq!(
        s.stats.food225,
        if context.key.source_id == 7 { 225 } else { 180 }
    );
    assert_eq!(s.stats.max_hp, 1000);
    assert_eq!(gear_snapshot(c).source.context, context);
}

fn selected(cs: &mut [Client; 2], i: usize, d: &Deadline, source_id: u32) -> SelectionContext {
    send(cs, i, d, || LoginCommand::SelectSource(source_id));
    wait(cs, d, "canonical SQL roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
    let v = cs[i].view();
    assert_eq!(v.selected_source, Some(source_id));
    assert_eq!(v.characters.len(), 1);
    assert_eq!(v.characters[0].object_id, 3333);
    let context = SelectionContext {
        key: CharacterKey {
            source_id,
            object_id: 3333,
        },
        epoch: v.roster_epoch,
    };
    send(cs, i, d, || LoginCommand::SelectCharacter(context));
    wait(cs, d, "canonical SQL character data", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::ChoosingChannel
            && !v.record_pending
            && !v.spell_pending
            && matches!(v.equipment, equipment::State::Ready(_))
    });
    source(&cs[i], context);
    context
}

fn enter(cs: &mut [Client; 2], i: usize, d: &Deadline, context: SelectionContext) {
    send(cs, i, d, || LoginCommand::EnterChannel {
        context,
        channel_id: CHANNEL,
    });
    wait(cs, d, "canonical SQL lobby", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::InLobby && !v.room_pending && !v.record_pending && !v.spell_pending
    });
    let v = cs[i].view();
    assert!(
        v.canonical.equipment_supported(),
        "Main must negotiate v41, not legacy fallback"
    );
    assert!(login_window::can_open_shop(&v));
}

fn shop_action(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    action: shop::Action,
    quantity: i64,
) -> shop::Snapshot {
    let before = cs[i].view();
    let nav = before.navigation.as_ref().unwrap();
    let read = shop::Request::read(nav.context, CHANNEL, before.rooms.joined());
    let q = if action == shop::Action::Read {
        read
    } else {
        read.mutation(before.shop.snapshot().unwrap(), action, 40010, quantity)
    };
    send(cs, i, d, || LoginCommand::Shop(q));
    assert!(cs[i].view().shop_pending);
    assert_eq!(
        cs[i].view().shop,
        before.shop,
        "a queued purchase cannot predict inventory or balance"
    );
    wait(cs, d, "canonical SQL shop receipt", |cs| {
        !cs[i].view().shop_pending
    });
    let v = cs[i].view();
    let reply = v.shop.reply().expect("real SQL shop reply");
    assert_eq!(reply.echo, q);
    assert_eq!(
        (reply.status, reply.reason),
        (shop::Status::Ok, shop::Reason::None)
    );
    assert_eq!(
        reply.room.as_ref().unwrap().joined.as_ref(),
        v.rooms.joined()
    );
    source(&cs[i], nav.context);
    v.shop.snapshot().unwrap().clone()
}

fn room_action(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    context: SelectionContext,
    action: rooms::Action,
    value: u8,
) {
    send(cs, i, d, || {
        LoginCommand::Rooms(rooms::Request::read(context, CHANNEL).unwrap())
    });
    wait(cs, d, "canonical SQL room READ", |cs| {
        !cs[i].view().room_pending
    });
    let joined = cs[i].view().rooms.joined().unwrap().clone();
    send(cs, i, d, || {
        LoginCommand::Rooms(room_command(context, &joined, action, value))
    });
    wait(cs, d, "canonical SQL room mutation", |cs| {
        !cs[i].view().room_pending
    });
    let rooms::State::Ready(reply) = cs[i].view().rooms else {
        panic!("room reply")
    };
    assert_eq!(reply.status, rooms::Status::Ok);
}

fn own_actor(c: &Client) -> data::Actor {
    let view = c.view();
    let o = view.canonical.observation().unwrap();
    o.data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap()
        .clone()
}

fn matched(cs: &mut [Client; 2], d: &Deadline, contexts: [SelectionContext; 2]) {
    let criteria = matching::Criteria {
        mode: 2,
        map_id: 554,
        players: 2,
    };
    let start = |i: usize| matching::Request {
        context: contexts[i],
        channel_id: CHANNEL,
        action: matching::Action::Start,
        ticket_id: 0,
        criteria: Some(criteria),
    };
    for c in cs.iter() {
        assert!(c.view().matchmaking.supported());
    }
    send(cs, 0, d, || LoginCommand::Matchmaking(start(0)));
    wait(cs, d, "canonical SQL first queue ticket", |cs| {
        !cs[0].view().room_pending
    });
    let old = cs[0].view().matchmaking.waiting().unwrap().ticket_id;
    assert!(cs[0].view().rooms.joined().is_none());
    let cancel = matching::Request {
        action: matching::Action::Cancel,
        ticket_id: old,
        criteria: None,
        ..start(0)
    };
    send(cs, 0, d, || LoginCommand::Matchmaking(cancel));
    wait(cs, d, "canonical SQL queue cancellation", |cs| {
        !cs[0].view().room_pending
    });
    assert_eq!(
        cs[0].view().matchmaking.snapshot().unwrap().phase,
        matching::Phase::Idle
    );
    for i in 0..2 {
        send(cs, i, d, || LoginCommand::Matchmaking(start(i)));
        assert!(cs[i].view().room_pending);
        assert!(
            cs[i].view().rooms.joined().is_none(),
            "queued intent is not membership"
        );
        wait(cs, d, "canonical SQL matching receipt", |cs| {
            !cs[i].view().room_pending
        });
        if i == 0 {
            let v = cs[0].view();
            let ticket = v.matchmaking.waiting().unwrap();
            assert!(ticket.ticket_id > old);
            assert_eq!(ticket.criteria, Some(criteria));
            assert!(v.rooms.joined().is_none());
            assert!(cs[0]
                .controller
                .command(LoginCommand::Matchmaking(cancel))
                .is_err());
        }
    }
    wait(cs, d, "canonical SQL both humans matched", |cs| {
        cs.iter().all(|c| {
            c.view()
                .matchmaking
                .snapshot()
                .is_some_and(|s| s.phase == matching::Phase::Matched)
        })
    });
    let first = cs[0].view();
    let room = first.rooms.joined().unwrap();
    assert_eq!(
        (
            room.summary.settings.mode,
            room.summary.settings.map_id,
            room.summary.settings.capacity
        ),
        (2, 554, 2)
    );
    assert_eq!(room.owner, contexts[0].key);
    assert!(room
        .members
        .iter()
        .all(|m| m.kind == rooms::MemberKind::Human && !m.ready));
    assert!(!room.summary.satisfied);
    assert_eq!(
        room.members.iter().map(|m| m.team).collect::<BTreeSet<_>>(),
        BTreeSet::from([0, 1])
    );
    assert_eq!(
        room.members
            .iter()
            .map(|m| m.source_name.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["太陽神", "月亮"])
    );
    assert_eq!(
        cs[1].view().rooms.joined().unwrap().summary.room_id,
        room.summary.room_id
    );
    assert_eq!(contexts[0].key.object_id, contexts[1].key.object_id);
    assert_ne!(contexts[0].key.source_id, contexts[1].key.source_id);
}

/// Wait only for local admission/current pixels. Once queued, this immutable
/// mutation is never resent; later NPC clicks are new user intents/sequences.
// The actual page model supplies local spell intents. Target spells first
// select the page entry, then retain the caller's separately painted target.
// This is controller/adapter evidence, not native combobox or mouse evidence.
fn skill_page_intent(surface: &Surface, view: &LoginSnapshot, kind: Kind) -> Option<Intent> {
    use crate::modern_lito::battle_skillbook::{self as book, Selection};
    use crate::modern_lito::battle_window::Skill;
    let id = match kind {
        Kind::Heal(_) => 1,
        Kind::LightArrow(_) => 4,
        Kind::Local(Local::Teleport) => 5,
        Kind::Local(Local::GreaterHaste) => 54,
        Kind::Haste(_) => 43,
        Kind::Slow(_) => 29,
        Kind::Local(Local::FireStorm) => 70,
        Kind::Disintegrate(_) => 77,
        _ => {
            return Some(Intent {
                stamp: Stamp::of(&surface.observation),
                kind,
            })
        }
    };
    let model = book::model(surface, view, str::to_owned)?;
    let choice = model.choice(id)?;
    let selection = book::select(choice, surface, view).expect("enabled skill-page choice");
    let mut detached = view.clone();
    detached.navigation = None;
    assert!(book::select(choice, surface, &detached).is_none());
    assert!(book::model(surface, &detached, str::to_owned).is_none());
    let mut pending = view.clone();
    pending.world_pending = true;
    assert!(book::select(choice, surface, &pending).is_none());
    assert!(book::model(surface, &pending, str::to_owned)
        .unwrap()
        .rows
        .iter()
        .all(|r| r.choice.is_none()));
    for field in 0..10 {
        let mut bad = choice;
        match field {
            0 => bad.stamp.context.key.source_id += 1,
            1 => bad.stamp.context.key.object_id += 1,
            2 => bad.stamp.context.epoch ^= 1,
            3 => bad.stamp.channel ^= 1,
            4..=6 => bad.stamp.world[field - 4] += 1,
            7 => bad.stamp.entity += 1,
            8 => bad.stamp.life += 1,
            _ => bad.stamp.revision += 1,
        }
        assert!(
            book::select(bad, surface, view).is_none(),
            "skill page stale field {field}"
        );
    }
    let intent = match selection {
        Selection::Cast(intent) => {
            assert_eq!(intent.kind, kind);
            intent
        }
        Selection::Target(skill) => {
            assert!(matches!(
                (skill, kind),
                (Skill::Heal, Kind::Heal(_))
                    | (Skill::LightArrow, Kind::LightArrow(_))
                    | (Skill::Disintegrate, Kind::Disintegrate(_))
                    | (Skill::Haste, Kind::Haste(_))
                    | (Skill::Slow, Kind::Slow(_))
            ));
            Intent {
                stamp: choice.stamp,
                kind,
            }
        }
    };
    eprintln!(
        "SKILL_PAGE id={id} selection={selection:?} source={}",
        choice.stamp.context.key.source_id
    );
    Some(intent)
}

fn inventory_page_intent(surface: &Surface, view: &LoginSnapshot, kind: Kind) -> Option<Intent> {
    use crate::modern_lito::battle_inventory::{self as inventory, Key};
    let key = match kind {
        Kind::Local(Local::UseItem(id)) => Key::Supply(id),
        Kind::Local(Local::SelectAmmo(id)) => Key::Ammo(id),
        _ => return None,
    };
    let model = inventory::model(surface, view, str::to_owned)?;
    let intent = model.local(key)?;
    inventory::local_command(intent, surface, view)?;
    let mut pending = view.clone();
    pending.world_pending = true;
    assert!(inventory::local_command(intent, surface, &pending).is_none());
    assert!(inventory::model(surface, &pending, str::to_owned)
        .unwrap()
        .rows
        .iter()
        .all(|r| r.local.is_none() && r.equip.is_none() && r.unequip.is_none()));
    let mut detached = view.clone();
    detached.navigation = None;
    assert!(inventory::local_command(intent, surface, &detached).is_none());
    assert!(inventory::model(surface, &detached, str::to_owned).is_none());
    for field in 0..10 {
        let mut bad = intent;
        match field {
            0 => bad.stamp.context.key.source_id += 1,
            1 => bad.stamp.context.key.object_id += 1,
            2 => bad.stamp.context.epoch ^= 1,
            3 => bad.stamp.channel ^= 1,
            4..=6 => bad.stamp.world[field - 4] += 1,
            7 => bad.stamp.entity += 1,
            8 => bad.stamp.life += 1,
            _ => bad.stamp.revision += 1,
        }
        assert!(
            inventory::local_command(bad, surface, view).is_none(),
            "inventory page stale field {field}"
        );
    }
    assert!(model
        .rows
        .iter()
        .filter(|r| matches!(r.key, Key::Source(_)))
        .all(|r| r.local.is_none()));
    eprintln!(
        "INVENTORY_PAGE key={key:?} source={}",
        intent.stamp.context.key.source_id
    );
    Some(intent)
}

fn input(cs: &mut [Client; 2], i: usize, d: &Deadline, kind: Kind) -> LoginSnapshot {
    let started = Instant::now();
    let mut cycles = 0u32;
    let mut no_surface = 0u32;
    let mut no_choice = 0u32;
    let mut no_command = 0u32;
    let mut command_errors = std::collections::BTreeMap::<String, u32>::new();
    let mut controller_errors = std::collections::BTreeMap::<String, u32>::new();
    let queued_at = loop {
        cycles += 1;
        for c in cs.iter_mut() {
            c.pump();
        }
        let view = cs[i].view();
        if let Some(surface) = cs[i].surface.as_ref() {
            let is_inventory =
                matches!(kind, Kind::Local(Local::UseItem(_) | Local::SelectAmmo(_)));
            let Some(intent) = (if is_inventory {
                inventory_page_intent(surface, &view, kind)
            } else {
                skill_page_intent(surface, &view, kind)
            }) else {
                no_choice += 1;
                d.pause("canonical skill-page choice admission");
                continue;
            };
            let q = if is_inventory {
                crate::modern_lito::battle_inventory::local_command(intent, surface, &view)
            } else {
                match battle_input::command(surface, intent, &view) {
                    Ok(q) => Some(q),
                    Err(e) => {
                        let mut reason = e.to_string();
                        if let Kind::Slow(target) = kind {
                            use crate::modern_lito::battle_slow::Intent as Proof;
                            if let (Some(painted), Some(current)) = (
                                Proof::from_observation(&surface.observation, target),
                                view.canonical
                                    .observation()
                                    .and_then(|o| Proof::from_observation(o, target)),
                            ) {
                                reason.push_str(&format!(" [revision={}, sequence={}, gear={}, cell={}, buff={}, target={}]",
                                    painted.stamp.revision != current.stamp.revision,
                                    painted.sequence != current.sequence,
                                    painted.equipment_revision != current.equipment_revision,
                                    (painted.x, painted.y) != (current.x, current.y),
                                    painted.buff_revision != current.buff_revision,
                                    painted.target != current.target));
                            }
                        }
                        *command_errors.entry(reason).or_default() += 1;
                        None
                    }
                }
            };
            if let Some(q) = q {
                let admitted = cs[i].controller.command(LoginCommand::Canonical(q));
                if admitted.is_ok() {
                    let pending = cs[i].view();
                    assert_eq!(
                        pending
                            .canonical
                            .observation()
                            .unwrap()
                            .personal()
                            .supplies(),
                        view.canonical.observation().unwrap().personal().supplies(),
                        "queued input cannot predict item consumption"
                    );
                    let before = view.canonical.observation().unwrap();
                    assert_eq!(
                        pending
                            .canonical
                            .observation()
                            .unwrap()
                            .data()
                            .personal
                            .materials,
                        before.data().personal.materials,
                        "queued input cannot predict material consumption"
                    );
                    let actor = before
                        .data()
                        .actors
                        .0
                        .iter()
                        .find(|a| a.entity == before.data().personal.entity)
                        .unwrap();
                    assert_eq!(
                        own_actor(&cs[i]).spells,
                        actor.spells,
                        "queued input cannot predict casts or MP expenditure"
                    );
                    break Instant::now();
                }
                *controller_errors.entry(admitted.unwrap_err()).or_default() += 1;
            } else {
                no_command += 1;
            }
        } else {
            no_surface += 1;
        }
        d.pause("painted canonical SQL intent admission");
    };
    assert!(cs[i].view().world_pending);
    wait(cs, d, "canonical SQL operation receipt", |cs| {
        !cs[i].view().world_pending
    });
    let result = cs[i].view();
    // Keep the formal player's poll cadence. Synchronous redirected stderr
    // after the receipt can block this test thread past a short audio cue's
    // expiry while its resource worker keeps publishing normally.
    cs[i].audio.input_diagnostics.push(format!("INPUT_ADMISSION kind={kind:?} source={} cycles={cycles} no_surface={no_surface} no_choice={no_choice} no_command={no_command} command_errors={command_errors:?} controller_errors={controller_errors:?} queue_ms={} receipt_ms={} total_ms={} outcome={:?}",
        result.navigation.as_ref().map(|n| n.context.key.source_id).unwrap_or(0),
        queued_at.duration_since(started).as_millis(),queued_at.elapsed().as_millis(),
        started.elapsed().as_millis(),result.canonical.outcome()));
    result
}

fn supplies(c: &Client) -> data::Supplies {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .personal()
        .supplies()
        .unwrap()
        .clone()
}

fn heal(cs: &mut [Client; 2], i: usize, d: &Deadline) {
    wait(cs, d, "real NPC damage before canonical potion", |cs| {
        let a = own_actor(&cs[i]);
        let v = &a.vital.unwrap().values;
        cs[i].rendered() && v.hp > 0 && v.hp < v.maximum_hp
    });
    let before = supplies(&cs[i]);
    assert_eq!(before.history.uses, 0);
    assert_eq!(before.stocks.0[0].initial, if i == 0 { 3 } else { 2 });
    let old_surface = cs[i].surface.clone().unwrap();
    let old_view = cs[i].view();
    let old_choice =
        crate::modern_lito::battle_inventory::model(&old_surface, &old_view, str::to_owned)
            .unwrap()
            .local(crate::modern_lito::battle_inventory::Key::Supply(40010))
            .unwrap();
    let v = input(cs, i, d, Kind::Local(Local::UseItem(40010)));
    assert!(
        crate::modern_lito::battle_inventory::local_command(old_choice, &old_surface, &v).is_none(),
        "an old inventory page must not become another potion use after a receipt"
    );
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    let after = supplies(&cs[i]);
    assert_eq!(after.request_sequence, 1);
    assert_eq!(after.history.uses, 1);
    assert_eq!(
        after.stocks.0[0].remaining,
        before.stocks.0[0].remaining - 1
    );
    let receipt = after.history.last.as_ref().unwrap();
    assert!(receipt.restored > 0 && receipt.restored <= 20);
    assert_eq!(
        receipt.hp_after - receipt.before.vitals.hp,
        receipt.restored
    );
    wait(cs, d, "canonical potion receipt rendered", |cs| {
        cs[i].rendered()
            && cs[i].surface.as_ref().is_some_and(|s| {
                s.observation
                    .personal()
                    .supplies()
                    .is_some_and(|p| p.history.uses == 1)
            })
    });
}

fn waves(cs: &mut [Client; 2], i: usize, d: &Deadline, fire: bool) {
    let mut seen = BTreeSet::new();
    let mut next = Instant::now();
    loop {
        for c in cs.iter_mut() {
            c.pump();
        }
        let view = cs[i].view();
        let o = view.canonical.observation().unwrap();
        assert_eq!(o.data().world.phase, data::Phase::Active);
        assert_eq!(o.data().world.lane, i as i32);
        assert!(o
            .data()
            .npcs
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .all(|n| n.lane == i as i32));
        if o.data()
            .quest
            .as_ref()
            .unwrap()
            .objectives
            .0
            .iter()
            .any(|q| q.hp > 0)
        {
            break;
        }
        if !view.world_pending && cs[i].rendered() && Instant::now() >= next {
            let alive = o
                .data()
                .npcs
                .as_ref()
                .unwrap()
                .actors
                .0
                .iter()
                .find(|n| n.hp > 0);
            if let Some(n) = alive.filter(|n| {
                cs[i]
                    .surface
                    .as_ref()
                    .unwrap()
                    .geometry
                    .npcs
                    .iter()
                    .any(|p| p.id.serial == n.serial as u64 && p.life == n.life as u64 && p.alive)
            }) {
                let (stage, target) = (
                    n.stage,
                    Target::Npc {
                        serial: n.serial,
                        life: n.life,
                    },
                );
                let result = input(
                    cs,
                    i,
                    d,
                    if fire {
                        Kind::Local(Local::FireStorm)
                    } else {
                        Kind::Attack(target)
                    },
                );
                assert_eq!(
                    result.canonical.outcome(),
                    Some(Outcome::Operation(
                        Rejection::None,
                        Disposition::Applied,
                        Reason::None
                    ))
                );
                if fire {
                    let actor = own_actor(&cs[i]);
                    let history = actor
                        .spells
                        .histories
                        .0
                        .iter()
                        .find(|s| s.lane == data::Lane::NpcFireStorm)
                        .unwrap();
                    let cost = if i == 0 { 46 } else { 48 };
                    assert_eq!(history.ledger.casts, i64::from(stage));
                    assert_eq!(history.ledger.cost, cost);
                    assert_eq!(history.ledger.spent, i64::from(stage * cost));
                    wait(cs, d, "canonical SQL fire receipt rendered", |cs| {
                        cs[i].surface.as_ref().is_some_and(|s| {
                            s.observation.data().actors.0.iter().any(|a| {
                                a.entity == s.observation.data().personal.entity
                                    && a.spells.histories.0.iter().any(|h| {
                                        h.lane == data::Lane::NpcFireStorm
                                            && h.ledger.casts == i64::from(stage)
                                    })
                            })
                        })
                    });
                    battle_window::validate(cs[i].surface.as_ref().unwrap()).unwrap();
                }
                seen.insert(stage);
                next = Instant::now() + Duration::from_millis(if fire { 2100 } else { 750 });
            }
        }
        d.pause("canonical SQL four waves");
    }
    assert_eq!(seen, BTreeSet::from([1, 2, 3, 4]));
}

#[test]
#[ignore = "two v41 controllers, production Main/owned SQL/shared554; shop, potion, four waves, saved result and return; no HWND"]
fn java_v41_sql_supplies_rendered_use_saved_results_shop_return_and_source_reload() {
    flow(false);
}
#[test]
#[ignore = "two v41 controllers + Main/owned SQL/shared554: queue, shop, readiness, countdown, NPC70, saved result and return; no HWND"]
fn java_v41_sql_matchmaking_shop_countdown_fire_saved_results_and_return() {
    flow(true);
}
fn flow(matchmaking: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(160));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernQuestSqlTestPeer",
        if matchmaking {
            "quest-matchmaking-sql"
        } else {
            "quest-supplies-sql"
        },
    );
    let profile = profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new_canonical_development(profile.clone()),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    let mut rounds = [0; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "canonical SQL TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "canonical SQL sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = selected(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
        let empty = shop_action(&mut cs, i, &d, shop::Action::Read, 0);
        assert!(empty.stacks.is_empty());
        assert_eq!(empty.balance, 20000);
        let bought = shop_action(
            &mut cs,
            i,
            &d,
            shop::Action::Buy,
            if i == 0 { 4 } else { 2 },
        );
        assert_eq!(bought.balance, if i == 0 { 19840 } else { 19920 });
        rounds[i] = bought.round_id;
    }
    if matchmaking {
        matched(&mut cs, &d, contexts);
    } else {
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
                            mode: 2,
                            attributes: 16,
                            map_id: 554,
                            name: "Canonical SQL supplies".into(),
                        },
                        String::new(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
        });
        wait(&mut cs, &d, "canonical SQL create", |cs| {
            !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
        });
        let r = cs[0].view().rooms.joined().unwrap().clone();
        send(&mut cs, 1, &d, || {
            LoginCommand::Rooms(
                rooms::Request::join(
                    rooms::Common {
                        action: rooms::Action::Join,
                        room_id: r.summary.room_id,
                        expected_revision: r.summary.revision,
                        ..rooms::Common::read(contexts[1], CHANNEL)
                    },
                    String::new(),
                )
                .unwrap(),
            )
        });
        wait(&mut cs, &d, "canonical SQL join", |cs| {
            cs.iter().all(|c| {
                !c.view().room_pending
                    && c.view()
                        .rooms
                        .joined()
                        .is_some_and(|r| r.members.len() == 2)
            })
        });
    }
    room_action(&mut cs, 0, &d, contexts[0], rooms::Action::Ready, 1);
    assert!(
        cs[0]
            .view()
            .rooms
            .joined()
            .unwrap()
            .member(contexts[0].key)
            .unwrap()
            .ready
    );
    let sold = shop_action(&mut cs, 0, &d, shop::Action::Sell, 1);
    assert_eq!(sold.balance, 19860);
    assert_eq!(sold.stack(40010).unwrap().quantity, 3);
    assert!(
        !cs[0]
            .view()
            .rooms
            .joined()
            .unwrap()
            .member(contexts[0].key)
            .unwrap()
            .ready
    );
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
    }
    cs[0].load = true;
    wait(&mut cs, &d, "canonical SQL first LOAD barrier", |cs| {
        cs.iter().all(|c| c.surface.is_some())
            && cs[0]
                .view()
                .canonical
                .observation()
                .is_some_and(|o| o.data().actors.0[0].loaded)
    });
    for c in &cs {
        assert_eq!(
            c.view().canonical.observation().unwrap().data().world.phase,
            data::Phase::Loading
        );
    }
    cs[1].load = true;
    if matchmaking {
        wait(
            &mut cs,
            &d,
            "canonical SQL both loaded then countdown",
            |cs| {
                cs.iter().all(|c| {
                    c.view().canonical.observation().is_some_and(|o| {
                        o.data().world.phase == data::Phase::Countdown
                            && o.data().actors.0.iter().all(|a| a.loaded)
                    })
                })
            },
        );
    }
    wait(&mut cs, &d, "canonical SQL active render", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    for i in 0..2 {
        battle_window::validate(cs[i].surface.as_ref().unwrap()).unwrap();
        let view = cs[i].view();
        assert!(!login_window::can_open_shop(&view));
        assert!(cs[i]
            .controller
            .command(LoginCommand::Shop(shop::Request::read(
                contexts[i],
                CHANNEL,
                view.rooms.joined()
            )))
            .is_err());
        if !matchmaking {
            heal(&mut cs, i, &d);
        }
    }
    for i in 0..2 {
        waves(&mut cs, i, &d, matchmaking);
    }
    let moon = cs[1]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .quest
        .as_ref()
        .unwrap()
        .objectives
        .0[0]
        .clone();
    let foreign = Intent {
        stamp: Stamp::of(cs[0].view().canonical.observation().unwrap()),
        kind: Kind::Attack(Target::Quest {
            slot: moon.id.slot,
            born: moon.born_revision,
        }),
    };
    assert!(battle_input::target_command(foreign, &cs[0].view()).is_err());
    let mut next = Instant::now();
    loop {
        for c in &mut cs {
            c.pump();
        }
        let view = cs[0].view();
        let o = view.canonical.observation().unwrap();
        if o.data().world.phase == data::Phase::Finished {
            break;
        }
        if cs[0].rendered() && !view.world_pending && Instant::now() >= next {
            let q = &o.data().quest.as_ref().unwrap().objectives.0[0];
            let result = input(
                &mut cs,
                0,
                &d,
                if matchmaking {
                    Kind::Local(Local::FireStorm)
                } else {
                    Kind::Attack(Target::Quest {
                        slot: q.id.slot,
                        born: q.born_revision,
                    })
                },
            );
            assert_eq!(
                result.canonical.outcome(),
                Some(Outcome::Operation(
                    Rejection::None,
                    Disposition::Applied,
                    Reason::None
                ))
            );
            next = Instant::now() + Duration::from_millis(if matchmaking { 2100 } else { 750 });
        }
        d.pause("canonical SQL objective finish");
    }
    wait(&mut cs, &d, "canonical SQL saved result", |cs| {
        cs.iter().all(|c| {
            c.view().canonical.lifecycle().is_some_and(|l| {
                l.phase == world::Phase::Finished
                    && l.settlement == Some(world::SettlementState::Saved)
            })
        })
    });
    let result = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .result
        .clone()
        .unwrap();
    assert_eq!(
        Some(&result),
        cs[1]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .result
            .as_ref()
    );
    assert_eq!(
        (result.mode, result.map, result.participants.0.len()),
        (2, 554, 2)
    );
    let mut stale_shop = None;
    for i in 0..2 {
        let w = cs[i]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .world
            .identity
            .clone();
        let leave = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&w))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(leave.clone()));
        wait(&mut cs, &d, "canonical SQL world return", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && !cs[i].view().record_pending
                && cs[i].runtime.is_quiescent()
        });
        if matchmaking {
            assert_eq!(
                cs[i].view().matchmaking.snapshot().unwrap().phase,
                matching::Phase::Matched
            );
        }
        send(&mut cs, i, &d, || LoginCommand::ReadRecord);
        wait(&mut cs, &d, "canonical SQL totals readback", |cs| {
            !cs[i].view().record_pending
        });
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(i == 0),
                losses: i64::from(i == 1),
                revision: 1
            })
        );
        let reset = shop_action(&mut cs, i, &d, shop::Action::Read, 0);
        assert!(reset.round_id > rounds[i]);
        assert_eq!(reset.balance, 20000);
        assert!(reset.stacks.is_empty());
        let bought = shop_action(&mut cs, i, &d, shop::Action::Buy, 1);
        assert_eq!(bought.stack(40010).unwrap().quantity, 1);
    }
    for i in (0..2).rev() {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        if matchmaking {
            assert!(matches!(
                cs[i].view().matchmaking,
                matching::State::NotRequested
            ));
        }
        if i == 0 {
            let current = shop_action(&mut cs, i, &d, shop::Action::Read, 0);
            stale_shop = Some(shop::Request::read(contexts[i], CHANNEL, None).mutation(
                &current,
                shop::Action::Buy,
                40010,
                1,
            ));
        }
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "canonical SQL roster return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    let reselected = selected(&mut cs, 0, &d, 42);
    enter(&mut cs, 0, &d, reselected);
    assert!(cs[0]
        .controller
        .command(LoginCommand::Shop(stale_shop.unwrap()))
        .is_err());
    let fresh = shop_action(&mut cs, 0, &d, shop::Action::Read, 0);
    assert_eq!(fresh.balance, 20000);
    assert!(fresh.stacks.is_empty());
    assert_eq!(cs[0].view().record.totals().unwrap().losses, 1);
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "canonical SQL normal shutdown", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
