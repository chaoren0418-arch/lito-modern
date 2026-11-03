//! Actual v42 controllers and R240 inventory model over TLS/shared514/owned SQL.
//! Synthetic qualified Knight+Mage/Elf sources and fixed dice, not Main or HWND.
use super::*;
use crate::modern_lito::{
    battle_inventory as inventory, item_protocol as item, timed_protocol as timed,
};

fn applied(v: &LoginSnapshot) {
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
}
fn pause(cs: &mut [Client; 2], d: &Deadline, ms: u64) {
    let until = Instant::now() + Duration::from_millis(ms);
    wait(cs, d, "inventory server cadence", |_| {
        Instant::now() >= until
    });
}
// Test-only bounded numeric state; do not expose the redacted identity/receipt
// structs or change any of the stock/picture admission predicates below.
fn pixel_observation(o: &crate::modern_lito::battle_follow::Observation) -> String {
    let d = o.data();
    let own = d.actors.0.iter().find(|a| a.entity == d.personal.entity);
    format!(
        "phase={:?} revision={} life_pose_hp={:?} gear_revision_clock={:?} supplies_sequence_uses={:?} ammo_revision_sequence_shots={:?}",
        d.world.phase,
        d.world.revision,
        own.map(|a| (battle_window::life(a), a.position.x, a.position.y, a.last_move_sequence, a.vital.as_ref().map(|v| v.values.hp))),
        d.personal.equipment.as_ref().map(|e| (e.revision, e.evaluated_at_epoch_millis, e.valid_until_epoch_millis)),
        d.personal.supplies.as_ref().map(|s| (s.request_sequence, s.history.uses)),
        d.personal.ammo.as_ref().map(|a| (a.revision, a.request_sequence, a.shot_count)),
    )
}
fn pixels(cs: &mut [Client; 2], i: usize, d: &Deadline) -> usize {
    let started = Instant::now();
    let next_report = std::cell::Cell::new(started + Duration::from_secs(2));
    let reports = std::cell::Cell::new(0);
    wait(cs, d, "inventory confirmed stock pixels", |cs| {
        let v = cs[i].view();
        let ready = cs[i].rendered()
            && cs[i].surface.as_ref().is_some_and(|s| {
                let p = &s.observation.data().personal;
                v.canonical.observation().is_some_and(|o| {
                    p.supplies == o.data().personal.supplies && p.ammo == o.data().personal.ammo
                })
            });
        if !ready && Instant::now() >= next_report.get() {
            next_report.set(Instant::now() + Duration::from_secs(10));
            reports.set(reports.get() + 1);
            let surface = cs[i].surface.as_ref();
            let current = v.canonical.observation();
            let equal = surface.zip(current).map(|(s, o)| {
                let painted = &s.observation.data().personal;
                let confirmed = &o.data().personal;
                (
                    painted.equipment == confirmed.equipment,
                    painted.supplies == confirmed.supplies,
                    painted.ammo == confirmed.ammo,
                )
            });
            eprintln!(
                "INVENTORY_PIXEL_WAIT elapsed_ms={} runtime={:?} source={:?} phase={:?} lifecycle={:?} pending={} rendered={} controls={} painted={:?} confirmed={:?} desired={:?} gear_supplies_ammo_equal={equal:?} navigation_gates={:?} outcome={:?} status={:?}",
                started.elapsed().as_millis(), cs[i].runtime.diagnostic_progress(), v.selected_source, v.phase,
                v.canonical.lifecycle().map(|l| (l.revision, l.phase, l.abort, l.settlement)),
                v.world_pending, cs[i].rendered(),
                surface.is_some_and(|s| battle_player::controls_ready(s, &v)),
                surface.map(|s| pixel_observation(&s.observation)),
                current.map(pixel_observation),
                cs[i].runtime.observation().map(pixel_observation),
                surface.zip(current).map(|(s, o)| (
                    battle_player::admitted(s, &v),
                    s.navigation.as_ref().is_some_and(|n| n.matches(&s.observation)),
                    s.navigation.as_ref().is_some_and(|n| n.blockers_current(o)),
                )),
                v.canonical.outcome(), v.status,
            );
        }
        ready
    });
    let surface = cs[i].surface.as_ref().unwrap();
    let model = inventory::model(surface, &cs[i].view(), str::to_owned).unwrap();
    assert_inventory_details(surface, &model);
    if reports.get() != 0 {
        eprintln!(
            "INVENTORY_PIXEL_RECOVERED elapsed_ms={} reports={} runtime={:?}",
            started.elapsed().as_millis(),
            reports.get(),
            cs[i].runtime.diagnostic_progress()
        );
    }
    reports.get()
}
fn ammo(c: &Client) -> data::Ammo {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .personal
        .ammo
        .clone()
        .unwrap()
}
fn select_source(cs: &mut [Client; 2], i: usize, d: &Deadline, source: u32) -> SelectionContext {
    send(cs, i, d, || LoginCommand::SelectSource(source));
    wait(cs, d, "inventory source roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
    let v = cs[i].view();
    assert_eq!(v.characters.len(), 1);
    assert_eq!(v.characters[0].object_id, 3333);
    let context = SelectionContext {
        key: CharacterKey {
            source_id: source,
            object_id: 3333,
        },
        epoch: v.roster_epoch,
    };
    send(cs, i, d, || LoginCommand::SelectCharacter(context));
    wait(cs, d, "inventory qualified source", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::ChoosingChannel
            && !v.record_pending
            && !v.spell_pending
            && matches!(v.equipment, equipment::State::Ready(_))
            && v.record.totals().is_some()
    });
    assert_eq!(gear_snapshot(&cs[i]).source.context, context);
    context
}
fn purchase(cs: &mut [Client; 2], i: usize, d: &Deadline, context: SelectionContext) {
    send(cs, i, d, || {
        LoginCommand::Shop(shop::Request::read(context, CHANNEL, None))
    });
    wait(cs, d, "inventory shop catalog", |cs| {
        !cs[i].view().shop_pending
    });
    assert_eq!(cs[i].view().shop.snapshot().unwrap().offers.len(), 9);
    for id in [
        40010, 40011, 40012, 40013, 40014, 40016, 40068, 40744, 40746,
    ] {
        let before = cs[i].view().shop.snapshot().unwrap().clone();
        let quantity = if i == 0 {
            match id {
                40744 => 2,
                40746 => 8,
                _ => 1,
            }
        } else {
            1
        };
        let q = shop::Request::read(context, CHANNEL, None).mutation(
            &before,
            shop::Action::Buy,
            id,
            quantity,
        );
        send(cs, i, d, || LoginCommand::Shop(q));
        assert_eq!(
            cs[i].view().shop.snapshot(),
            Some(&before),
            "no predicted purchase"
        );
        wait(cs, d, "inventory purchase receipt", |cs| {
            !cs[i].view().shop_pending
        });
        assert_eq!(cs[i].view().shop.reply().unwrap().status, shop::Status::Ok);
        assert_eq!(
            cs[i]
                .view()
                .shop
                .snapshot()
                .unwrap()
                .stack(id)
                .unwrap()
                .quantity,
            quantity
        );
    }
}
fn use_stock(cs: &mut [Client; 2], i: usize, d: &Deadline, id: i64) {
    use_stock_with_confirmation(cs, i, d, id, pixels);
}
fn use_stock_with_confirmation(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    id: i64,
    confirm: impl FnOnce(&mut [Client; 2], usize, &Deadline) -> usize,
) -> usize {
    pause(cs, d, 1100);
    pixels(cs, i, d);
    let old = cs[i].surface.clone().unwrap();
    let v = cs[i].view();
    let before = supplies(&cs[i]);
    let old_ammo = ammo(&cs[i]);
    let source = gear_snapshot(&cs[i]);
    let proof = inventory::model(&old, &v, str::to_owned)
        .unwrap()
        .local(inventory::Key::Supply(id))
        .unwrap();
    let result = input(cs, i, d, Kind::Local(Local::UseItem(id)));
    applied(&result);
    let after = supplies(&cs[i]);
    assert_eq!(after.request_sequence, before.request_sequence + 1);
    assert_eq!(
        after
            .stocks
            .0
            .iter()
            .find(|s| s.item == id)
            .unwrap()
            .remaining,
        0
    );
    for s in before.stocks.0.iter().filter(|s| s.item != id) {
        assert_eq!(after.stocks.0.iter().find(|a| a.item == s.item), Some(s));
    }
    assert_eq!(
        ammo(&cs[i]),
        old_ammo,
        "USE cannot alter arrow selection/shot counters"
    );
    assert_eq!(
        gear_snapshot(&cs[i]),
        source,
        "USE does not consume saved source objects"
    );
    assert!(inventory::local_command(proof, &old, &result).is_none());
    if item::healing(id).is_some() {
        let receipt = after.history.last.as_ref().unwrap();
        assert_eq!(receipt.item, id);
        assert_eq!(after.history.uses, before.history.uses + 1);
        assert_eq!(
            receipt.hp_after - receipt.before.vitals.hp,
            receipt.restored
        );
        if i == 1 && id == 40010 {
            assert!(receipt.restored > 0);
        }
    } else {
        let b = own_actor(&cs[i]).buffs.unwrap();
        let kind = timed::Kind::from_item(id).unwrap();
        match kind {
            timed::Kind::Haste => assert!(b.haste_until_nanos > b.observed_nanos),
            timed::Kind::Brave => {
                assert!(b.brave_until_nanos > b.observed_nanos);
                assert_eq!(b.brave_kind, 1);
            }
            timed::Kind::ElfBrave => {
                assert!(b.brave_until_nanos > b.observed_nanos);
                assert_eq!(b.brave_kind, 3);
            }
            timed::Kind::Wisdom => assert!(b.wisdom_until_nanos > b.observed_nanos),
        }
    }
    let reports = confirm(cs, i, d);
    let m = inventory::model(
        cs[i].surface.as_ref().unwrap(),
        &cs[i].view(),
        str::to_owned,
    )
    .unwrap();
    assert!(
        m.local(inventory::Key::Supply(id)).is_none(),
        "exhausted stock disabled"
    );
    eprintln!(
        "INVENTORY_USED source={} id={id} remaining=0",
        v.selected_source.unwrap()
    );
    reports
}
fn select_ammo(cs: &mut [Client; 2], d: &Deadline, id: i64) {
    pixels(cs, 0, d);
    let old = cs[0].surface.clone().unwrap();
    let before = ammo(&cs[0]);
    let items = supplies(&cs[0]);
    let intent = inventory::model(&old, &cs[0].view(), str::to_owned)
        .unwrap()
        .local(inventory::Key::Ammo(id))
        .unwrap();
    let v = input(cs, 0, d, Kind::Local(Local::SelectAmmo(id)));
    applied(&v);
    let after = ammo(&cs[0]);
    assert_eq!(after.selected_item, id);
    assert_eq!(after.request_sequence, before.request_sequence + 1);
    assert_eq!(after.shot_count, before.shot_count);
    assert_eq!(after.stocks, before.stocks);
    assert_eq!(supplies(&cs[0]), items);
    assert!(inventory::local_command(intent, &old, &v).is_none());
    pixels(cs, 0, d);
    assert!(
        inventory::model(
            cs[0].surface.as_ref().unwrap(),
            &cs[0].view(),
            str::to_owned
        )
        .unwrap()
        .local(inventory::Key::Ammo(id))
        .is_none(),
        "already-selected row does not queue duplicate selection"
    );
}
fn shoot(cs: &mut [Client; 2], d: &Deadline, reason: Reason) {
    pause(cs, d, 1100);
    let target = own_actor(&cs[1]);
    let before = ammo(&cs[0]);
    let v = input(
        cs,
        0,
        d,
        Kind::Attack(Target::Player {
            entity: target.entity,
            life: battle_window::life(&target),
        }),
    );
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            if reason == Reason::None {
                Disposition::Applied
            } else {
                Disposition::Denied
            },
            reason
        ))
    );
    let after = ammo(&cs[0]);
    if reason == Reason::None {
        assert_eq!(after.shot_count, before.shot_count + 1);
        assert_eq!(after.request_sequence, before.request_sequence);
        let old = before
            .stocks
            .0
            .iter()
            .find(|s| s.item == before.selected_item)
            .unwrap();
        assert_eq!(
            after
                .stocks
                .0
                .iter()
                .find(|s| s.item == before.selected_item)
                .unwrap()
                .remaining,
            old.remaining - 1
        );
    } else {
        assert_eq!(after, before);
    }
}
#[test]
#[ignore = "v42 inventory controllers: seven supply catalog/class gates, arrows/real miss-hit/SQL/return; synthetic sources, shared514, no HWND"]
fn java_v42_inventory_knight_mage_use_select_shoot_saved_return() {
    run(false);
}
#[test]
#[ignore = "v42 inventory controllers: ElfBrave success/Wisdom denial, arrows/SQL/source swap; synthetic sources, shared514, no HWND"]
fn java_v42_inventory_knight_elf_use_select_shoot_saved_return() {
    run(true);
}

fn run(elf: bool) {
    run_with_render_delay(elf, false, false);
}

#[test]
#[ignore = "v42 actual TLS/shared514/owned SQL: explicitly hold only receiver painting across miss, hit and NoAmmo; release, catch up, use, save and return; not HWND or reproduction of R289"]
fn java_v42_delayed_inventory_receiver_catches_up_before_use_and_saved_return() {
    run_with_render_delay(false, true, false);
}

#[test]
#[ignore = "v42 actual TLS/shared514/owned SQL: receiver waits 350 ms before every real frame throughout use, shots, save and return; not HWND or proof of R289 cause"]
fn java_v42_continuously_slow_inventory_receiver_uses_saves_and_returns() {
    run_with_render_delay(false, false, true);
}

#[test]
#[ignore = "v42 TLS/shared514/owned SQL: hold real receiver painting after stock USE receipt, exercise bounded wait diagnostics, recover/save/return; not HWND or reproduction of R289"]
fn java_v42_inventory_stock_wait_diagnostics_recovers_saved_return() {
    run_with_diagnostics(false, false, false, false, true);
}

fn run_with_render_delay(elf: bool, gate_receiver: bool, pace_receiver: bool) {
    run_with_entry(elf, gate_receiver, pace_receiver, false);
}
#[test]
#[ignore = "ordinary schema2 v47/shared514: Knight/Mage use/shoot/kill/saved result/source reload; synthetic source, real TLS/SQL, no HWND"]
fn java_ordinary_v47_deathmatch_knight_mage_saved_and_return() {
    run_with_entry(false, false, false, true);
}
#[test]
#[ignore = "ordinary schema2 v47/shared514: Knight/Elf class gates/use/shoot/kill/saved result/source reload; synthetic source, real TLS/SQL, no HWND"]
fn java_ordinary_v47_deathmatch_knight_elf_saved_and_return() {
    run_with_entry(true, false, false, true);
}
fn run_with_entry(elf: bool, gate_receiver: bool, pace_receiver: bool, ordinary: bool) {
    run_with_diagnostics(elf, gate_receiver, pace_receiver, ordinary, false);
}
fn run_with_diagnostics(
    elf: bool,
    gate_receiver: bool,
    pace_receiver: bool,
    ordinary: bool,
    diagnose_stock_wait: bool,
) {
    assert!(!(gate_receiver && pace_receiver));
    assert!(!diagnose_stock_wait || (!gate_receiver && !pace_receiver));
    let d = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        if ordinary {
            "local.lito381.server.ModernOrdinaryInventoryTestPeer"
        } else {
            "local.lito381.server.ModernCombatTestPeer"
        },
        if elf {
            "inventory-ammo-elf"
        } else {
            "inventory-ammo"
        },
    );
    assert!(peer.process_job.is_some());
    let mut render_gate = None;
    let mut render_pace = None;
    let mut cs = std::array::from_fn(|i| {
        let runtime = if (gate_receiver || diagnose_stock_wait) && i == 1 {
            let (runtime, gate) =
                crate::modern_lito::battle_runtime::render_gate_tests::shared(shared.clone())
                    .unwrap();
            render_gate = Some(gate);
            runtime
        } else if pace_receiver && i == 1 {
            let (runtime, pace) =
                crate::modern_lito::battle_runtime::render_gate_tests::paced_shared(shared.clone())
                    .unwrap();
            render_pace = Some(pace);
            runtime
        } else {
            BattleRuntime::new(shared.clone()).unwrap()
        };
        Client {
            controller: mode_controller(peer.endpoint, &shared, &private_root(&fixtures), ordinary),
            runtime,
            surface: None,
            load: false,
            audio: SoundAudit::default(),
        }
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "inventory TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "inventory authentication", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = select_source(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
        purchase(&mut cs, i, &d, contexts[i]);
    }
    let source = cs.each_ref().map(gear_snapshot);
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
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Canonical inventory".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "inventory creator joins", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    let room = cs[0].view().rooms.joined().unwrap().clone();
    send(&mut cs, 1, &d, || {
        LoginCommand::Rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: room.summary.room_id,
                    expected_revision: room.summary.revision,
                    ..rooms::Common::read(contexts[1], CHANNEL)
                },
                String::new(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "inventory second member joins", |cs| {
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
    wait(&mut cs, &d, "inventory first qualified LOAD", |cs| {
        !cs[0].load && cs.iter().all(|c| c.surface.is_some())
    });
    for c in &cs {
        let v = c.view();
        assert_eq!(
            v.canonical.observation().unwrap().data().world.phase,
            data::Phase::Loading
        );
        assert!(
            inventory::model(c.surface.as_ref().unwrap(), &v, str::to_owned)
                .unwrap()
                .rows
                .iter()
                .all(|r| r.local.is_none())
        );
    }
    cs[1].load = true;
    wait(&mut cs, &d, "inventory countdown and active pixels", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    for i in 0..2 {
        pixels(&mut cs, i, &d);
        let v = cs[i].view();
        let o = v.canonical.observation().unwrap();
        assert_eq!(o.data().world.descriptor.active_timeout_millis, 120000);
        assert_eq!(o.data().world.descriptor.countdown_nanos, 3000000000);
        let class = o.data().room.members.0[i].class as u16;
        assert_eq!(
            class,
            if i == 0 {
                1
            } else if elf {
                2
            } else {
                3
            }
        );
        assert_eq!(supplies(&cs[i]).stocks.0.len(), 7);
        assert_eq!(ammo(&cs[i]).stocks.0.len(), 2);
        let m = inventory::model(cs[i].surface.as_ref().unwrap(), &v, str::to_owned).unwrap();
        for k in timed::Kind::ALL {
            assert_eq!(
                m.local(inventory::Key::Supply(k.item_id())).is_some(),
                k.eligible(class)
            );
        }
        assert!(m
            .rows
            .iter()
            .filter(|r| matches!(r.key, inventory::Key::Source(_)))
            .all(|r| r.local.is_none()));
    }
    for id in [40013, 40010, 40011, 40012, 40014] {
        use_stock(&mut cs, 0, &d, id);
    }
    select_ammo(&mut cs, &d, 40746);
    select_ammo(&mut cs, &d, 40744);
    pixels(&mut cs, 0, &d);
    let old_picture = cs[0].surface.clone().unwrap();
    let old_arrow = inventory::model(&old_picture, &cs[0].view(), str::to_owned)
        .unwrap()
        .local(inventory::Key::Ammo(40746))
        .unwrap();
    let target_hp = own_actor(&cs[1]).vital.unwrap().values.hp;
    let held_picture = render_gate.as_ref().filter(|_| gate_receiver).map(|gate| {
        gate.arm();
        wait(
            &mut cs,
            &d,
            "inventory receiver render gate entered",
            |_| gate.holding(),
        );
        // Gate entry may race the preceding publication; take the last output
        // only after entry and a fresh pump, before any of the three shots.
        cs[1].pump();
        cs[1]
            .surface
            .clone()
            .expect("receiver has real pre-shot pixels")
    });
    shoot(&mut cs, &d, Reason::None);
    assert!(
        !ammo(&cs[0]).hit,
        "fixed first ER miss still consumes an arrow"
    );
    assert!(
        inventory::local_command(old_arrow, &old_picture, &cs[0].view()).is_none(),
        "a shot invalidates old arrow-page stock proof"
    );
    shoot(&mut cs, &d, Reason::None);
    assert!(ammo(&cs[0]).hit);
    wait(&mut cs, &d, "inventory actual target damage", |cs| {
        own_actor(&cs[1]).vital.unwrap().values.hp < target_hp
    });
    assert_eq!(
        ammo(&cs[0]).selected_item,
        40744,
        "never auto-switch exhausted arrows"
    );
    shoot(&mut cs, &d, Reason::NoAmmo);
    if let Some(gate) = render_gate.as_ref().filter(|_| gate_receiver) {
        let revision = cs[0]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .world
            .revision;
        wait(
            &mut cs,
            &d,
            "inventory receiver authority advances while painting held",
            |cs| {
                cs[1]
                    .view()
                    .canonical
                    .observation()
                    .is_some_and(|o| o.data().world.revision >= revision)
            },
        );
        let held = held_picture.as_ref().unwrap();
        let latest = cs[1].view();
        let current = latest.canonical.observation().unwrap();
        let hp = own_actor(&cs[1]).vital.unwrap().values.hp;
        assert!(
            gate.holding(),
            "receiver must still be deliberately stalled"
        );
        assert!(
            Arc::ptr_eq(held, cs[1].surface.as_ref().unwrap()),
            "new authority cannot relabel retained pixels"
        );
        assert!(held.observation.data().world.revision < current.data().world.revision);
        assert!(hp < target_hp);
        eprintln!("INVENTORY_RENDER_GATE held_revision={} confirmed_revision={} hp_before={target_hp} hp_after={hp}", held.observation.data().world.revision, current.data().world.revision);
        gate.release();
        wait(
            &mut cs,
            &d,
            "inventory receiver real pixels catch up after release",
            |cs| {
                gate.completed()
                    && cs[1].surface.as_ref().is_some_and(|s| {
                        s.observation.data().world.revision >= revision
                            && s.observation.data().actors.0.iter().any(|a| {
                                a.entity == s.observation.data().personal.entity
                                    && a.vital.as_ref().is_some_and(|v| v.values.hp == hp)
                            })
                    })
            },
        );
        pixels(&mut cs, 1, &d);
        eprintln!("INVENTORY_RENDER_GATE caught_up=true stock_and_controls_confirmed=true");
    }
    for id in [
        40010,
        40011,
        40012,
        40013,
        40014,
        if elf { 40068 } else { 40016 },
    ] {
        if diagnose_stock_wait && id == 40010 {
            pixels(&mut cs, 1, &d);
            let gate = render_gate.as_ref().unwrap();
            gate.arm();
            wait(&mut cs, &d, "inventory stock wait gate holds", |_| {
                gate.holding()
            });
            // Start the independent, bounded release only after the real USE
            // receipt. No made-up stocks, extended deadline or weaker predicate.
            let reports = use_stock_with_confirmation(&mut cs, 1, &d, id, |cs, i, d| {
                std::thread::scope(|scope| {
                    let release = scope.spawn(|| {
                        let until = Instant::now() + Duration::from_secs(3);
                        while Instant::now() < until {
                            std::thread::park_timeout(Duration::from_millis(5));
                        }
                        gate.release();
                    });
                    let reports = pixels(cs, i, d);
                    release.join().unwrap();
                    reports
                })
            });
            assert!(
                reports >= 1,
                "must observe a real stock/pixel mismatch before recovery"
            );
            assert!(gate.completed());
            eprintln!("INVENTORY_STOCK_WAIT_DIAGNOSTICS reports={reports} recovered=true");
        } else {
            use_stock(&mut cs, 1, &d, id);
        }
    }
    pixels(&mut cs, 0, &d);
    assert!(inventory::model(
        cs[0].surface.as_ref().unwrap(),
        &cs[0].view(),
        str::to_owned
    )
    .unwrap()
    .local(inventory::Key::Ammo(40744))
    .is_none());
    select_ammo(&mut cs, &d, 40746);
    while cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .phase
        == data::Phase::Active
    {
        assert!(ammo(&cs[0]).shot_count < 10);
        shoot(&mut cs, &d, Reason::None);
    }
    wait(&mut cs, &d, "inventory saved terminal", |cs| {
        cs.iter().all(|c| {
            c.view()
                .canonical
                .lifecycle()
                .is_some_and(|l| l.settlement == Some(world::SettlementState::Saved))
        })
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
    for i in 0..2 {
        pixels(&mut cs, i, &d);
        let v = cs[i].view();
        let m = inventory::model(cs[i].surface.as_ref().unwrap(), &v, str::to_owned).unwrap();
        assert!(m
            .rows
            .iter()
            .all(|r| r.local.is_none() && r.equip.is_none() && r.unequip.is_none()));
        assert_eq!(gear_snapshot(&cs[i]), source[i]);
        assert!(!login_window::can_read_record(&v));
        assert!(cs[i].controller.command(LoginCommand::ReadRecord).is_err(),
            "world result is authoritative until confirmed retirement; do not loop a forbidden roster read");
        let leave = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&world_id))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(leave.clone()));
        wait(&mut cs, &d, "inventory confirmed return", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        wait(&mut cs, &d, "inventory returned record admission", |cs| {
            login_window::can_read_record(&cs[i].view())
        });
        send(&mut cs, i, &d, || LoginCommand::ReadRecord);
        wait(&mut cs, &d, "inventory SQL totals", |cs| {
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
        send(&mut cs, i, &d, || {
            LoginCommand::Shop(shop::Request::read(contexts[i], CHANNEL, None))
        });
        wait(&mut cs, &d, "inventory next round wallet", |cs| {
            !cs[i].view().shop_pending
        });
        let v = cs[i].view();
        let wallet = v.shop.snapshot().unwrap();
        assert_eq!((wallet.round_id, wallet.balance), (2, 20000));
        assert!(wallet.stacks.is_empty());
        send(&mut cs, i, &d, || LoginCommand::LeaveLobby(contexts[i]));
        wait(&mut cs, &d, "inventory channel return", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingChannel
        });
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "inventory roster return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    for i in 0..2 {
        let source_id = if i == 0 { 42 } else { 7 };
        let context = select_source(&mut cs, i, &d, source_id);
        assert!(context.epoch > contexts[i].epoch);
        assert!(inventory::model(&old_picture, &cs[i].view(), str::to_owned).is_none());
        assert!(inventory::local_command(old_arrow, &old_picture, &cs[i].view()).is_none());
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(source_id == 7),
                losses: i64::from(source_id == 42),
                revision: 1
            })
        );
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "inventory normal shutdown", |cs| {
        cs.iter().all(|c| {
            c.runtime.is_quiescent()
                && c.controller.is_quiescent()
                && c.view().phase == LoginPhase::Disconnected
        })
    });
    peer.expect_successful_exit(&d);
    assert!(render_gate.as_ref().is_none_or(|g| g.completed()));
    if let Some(pace) = render_pace {
        let frames = pace.completed_frames();
        let waited = pace.waited_millis();
        assert!(frames >= 20, "must slow sustained painting, not one frame");
        assert!(waited >= frames as u64 * 350);
        eprintln!("INVENTORY_RENDER_PACE completed_real_frames={frames} waited_millis={waited} minimum_delay_per_frame_millis=350 stock_use_saved_return=true");
    }
}
