//! Two actual controllers exercise the same inventory model and dispatch as HWND.
//! Shared resources render both UNEQUIP and EQUIP; no native click claim.
use super::*;
#[path = "battle_haste_controller_tests.rs"]
mod haste;
#[path = "battle_return_controller_tests.rs"]
mod returned;
#[path = "canonical_supplies_sql_tests.rs"]
mod supplies_sql;
#[path = "battle_target_haste_controller_tests.rs"]
mod target_haste;
use crate::modern_lito::{
    battle_action::Guard,
    battle_control::{Command, Operation, Outcome},
    battle_equipment as gear, battle_inventory, battle_player,
    battle_runtime::{BattleRuntime, Status as BattleStatus},
    battle_scene::Surface,
    battle_window::Stamp,
    battle_world, equipment, login_window,
};

struct Client {
    controller: SessionController,
    runtime: BattleRuntime,
    surface: Option<Arc<Surface>>,
    load: bool,
    audio: SoundAudit,
}

/// The player consumes audio on every pump, including while a command is
/// pending. Tests must also retain evidence of those short-lived events, not
/// begin listening only after a synchronous helper returns its receipt.
#[derive(Default)]
struct SoundAudit {
    scope: Option<Stamp>,
    heard: std::collections::BTreeMap<u32, u64>,
    teleport_diagnostics: Vec<String>,
    input_diagnostics: Vec<String>,
}
impl SoundAudit {
    fn same_scope(a: Stamp, b: Stamp) -> bool {
        a.context == b.context
            && a.channel == b.channel
            && a.world == b.world
            && a.entity == b.entity
            && a.life == b.life
    }
    fn enter(&mut self, stamp: Stamp) {
        if self.scope.is_none_or(|s| !Self::same_scope(s, stamp)) {
            self.heard.clear();
        }
        self.scope = Some(stamp);
    }
    fn record(&mut self, id: u32, sequence: u64) {
        self.heard
            .entry(id)
            .and_modify(|n| *n = (*n).max(sequence))
            .or_insert(sequence);
    }
    fn observe_publication(&mut self, runtime: &BattleRuntime, stamp: Stamp) {
        self.enter(stamp);
        // HWND ticks attachment-owned audio immediately after runtime update,
        // even while the newest pixels are not yet admitted. A rendered-only
        // audit loses short cues during async publication/revision changes.
        let view = runtime.audio_view();
        let Some((attachment, prepared)) = view.publication else { return; };
        if !view.playable || !view.attachment.is_some_and(|live| Arc::ptr_eq(live, attachment))
            || prepared.preferences != view.preferences
            || prepared.gain != Some(view.preferences.values.audible_gain(true)) {
            return;
        }
        let now = Instant::now();
        if let Some(frame) = &prepared.frame {
            for event in frame.events.iter() {
                if event.created > now || now.duration_since(event.created) > Duration::from_millis(500) {
                    continue;
                }
                for &id in event.group.candidates() {
                    if event.bank.as_ref().unwrap_or(&frame.bank).get(id).is_some() {
                        self.record(id, event.sequence);
                    }
                }
            }
        }
    }
    fn diagnose_teleport(&mut self, surface: &Surface, admitted: bool) {
        if self.teleport_diagnostics.len() >= 12 { return; }
        let data = surface.observation.data();
        let Some(receipt) = data.actors.0.iter()
            .find(|a| a.entity == data.personal.entity)
            .and_then(|a| a.spells.histories.0.iter().find(|s| s.lane == super::super::battle_payload::data::Lane::Teleport))
            .and_then(|s| s.receipt.as_ref()) else { return; };
        let now = Instant::now();
        let cues: Vec<_> = surface.audio.as_ref().into_iter().flat_map(|f| f.events.iter())
            .filter(|e| e.group.candidates().contains(&119))
            .map(|e| (e.sequence, now.saturating_duration_since(e.created).as_millis()))
            .collect();
        self.teleport_diagnostics.push(format!("TELEPORT_AUDIO_AUDIT revision={} admitted={} server_receipt_age_us={} cue119_sequence_age_ms={:?}",
            data.world.revision, admitted, (data.world.loading_elapsed_nanos - receipt.loading_elapsed_nanos) / 1000, cues));
    }
    fn dump_diagnostics(&mut self) {
        for line in self.input_diagnostics.drain(..) { eprintln!("{line}"); }
        for line in self.teleport_diagnostics.drain(..) { eprintln!("{line}"); }
    }
    fn sequence(&self, stamp: Stamp, id: u32) -> u64 {
        if self.scope.is_some_and(|s| Self::same_scope(s, stamp)) {
            self.heard.get(&id).copied().unwrap_or(0)
        } else {
            0
        }
    }
}
impl Drop for SoundAudit {
    fn drop(&mut self) { self.dump_diagnostics(); }
}

#[test]
fn sound_audit_retains_polled_cues_without_crossing_attachment_or_life() {
    let mut audit = SoundAudit::default();
    let stamp = Stamp {
        context: SelectionContext { key: SUN, epoch: 1 },
        channel: 1,
        world: [1, 2, 3],
        entity: 1,
        life: 1,
        revision: 1,
    };
    audit.enter(stamp);
    audit.record(717, 4);
    audit.record(717, 3);
    audit.enter(Stamp {
        revision: 9,
        ..stamp
    }); // New frame has no live cue.
    assert_eq!(audit.sequence(stamp, 717), 4);
    assert_eq!(audit.sequence(stamp, 710), 0);
    for changed in [
        Stamp {
            world: [1, 2, 4],
            ..stamp
        },
        Stamp {
            channel: 2,
            ..stamp
        },
        Stamp { entity: 2, ..stamp },
        Stamp { life: 2, ..stamp },
        Stamp {
            context: SelectionContext {
                epoch: 2,
                ..stamp.context
            },
            ..stamp
        },
        Stamp {
            context: SelectionContext {
                key: crate::modern_lito::application_protocol::CharacterKey {
                    source_id: 42,
                    ..stamp.context.key
                },
                ..stamp.context
            },
            ..stamp
        },
    ] {
        assert_eq!(audit.sequence(changed, 717), 0);
        audit.enter(changed);
        assert_eq!(audit.sequence(changed, 717), 0);
        audit.enter(stamp);
        assert_eq!(audit.sequence(stamp, 717), 0);
        audit.record(717, 4);
    }
}
impl Client {
    fn view(&self) -> LoginSnapshot {
        self.controller.snapshot()
    }
    fn pump(&mut self) {
        self.controller.poll();
        let view = self.view();
        assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
        login_window::validate_snapshot(&view)
            .expect("controller snapshot must be accepted by the real UI");
        assert!(
            view.world.frame().is_none()
                && view.combat.is_none()
                && view.npc.is_none()
                && view.battle.observation().is_none()
        );
        self.surface = match battle_player::update(&mut self.runtime, &view, false).unwrap() {
            BattleStatus::Ready(s) => {
                let admitted = battle_player::admitted(&s, &view);
                self.audio.diagnose_teleport(&s, admitted);
                admitted.then_some(s)
            },
            BattleStatus::Failed(e) => panic!("equipment controller resources: {e}"),
            _ => None,
        };
        if let Some(observation) = view.canonical.observation() {
            self.audio.observe_publication(&self.runtime, Stamp::of(observation));
        }
        if let Some(s) = &self.surface {
            crate::modern_lito::battle_minimap::tests::verify_surface(s);
        }
        if self.load {
            if let Some(q) = self
                .surface
                .as_ref()
                .and_then(|s| battle_player::canonical_loaded_command(&self.runtime, s, &view))
            {
                if self.controller.command(LoginCommand::Canonical(q)).is_ok() {
                    self.load = false;
                }
            }
        }
    }
    fn rendered(&self) -> bool {
        let view = self.view();
        !view.world_pending
            && self.surface.as_ref().is_some_and(|s| {
                battle_player::controls_ready(s, &view)
                    && view.canonical.observation().is_some_and(|o| {
                        s.observation.data().personal.equipment == o.data().personal.equipment
                    })
            })
    }
}
fn wait(
    cs: &mut [Client; 2],
    d: &Deadline,
    label: &'static str,
    done: impl Fn(&[Client; 2]) -> bool,
) {
    loop {
        for c in cs.iter_mut() {
            c.pump();
        }
        if done(cs) {
            return;
        }
        d.pause(label);
    }
}
fn send(cs: &mut [Client; 2], i: usize, d: &Deadline, build: impl Fn() -> LoginCommand) {
    // Read/poll may transiently hold the worker inbox, but a submitted mutation
    // is never retried. Test callers retain the original immutable command.
    loop {
        for c in cs.iter_mut() {
            c.pump();
        }
        if cs[i].controller.command(build()).is_ok() {
            return;
        }
        d.pause("equipment controller local admission");
    }
}
/// Same read-only adapter used by the canonical HWND, exercised against real
/// server observations. This is not evidence of native scrolling/clicking.
fn assert_inventory_details(surface: &Surface, model: &battle_inventory::Model) {
    use crate::modern_lito::{battle_details, battle_inventory::Key};
    let o = &surface.observation;
    let summary = battle_details::inventory(o, None, |_| {
        panic!("overview must not expand item descriptions")
    })
    .unwrap();
    assert!(summary.contains("背包總覽"));
    assert!(
        !summary.contains("實例 ")
            && !summary.contains("補給 #")
            && !summary.contains("箭矢 #")
            && !summary.contains("材料 #")
    );
    let saved_model = model.clone();
    for row in &model.rows {
        let focus = model.selection(row.key).unwrap();
        let text = battle_details::inventory(o, Some(focus), str::to_owned).unwrap();
        assert!(text.contains("所選物品詳細資料"));
        let counts = ["實例 ", "補給 #", "箭矢 #", "材料 #"].map(|tag| text.matches(tag).count());
        let expected = match row.key {
            Key::Source(id) => {
                let e = o.data().personal.equipment.as_ref().unwrap();
                let item = e
                    .inventory
                    .0
                    .iter()
                    .find(|i| i.object as u64 == id)
                    .unwrap();
                assert!(text.contains(&format!(
                    "實例 {id}／目錄 {}／數量 {}",
                    item.catalog, item.quantity
                )));
                let equipped = o.personal().equipped(item.object).unwrap();
                assert!(text.contains(if equipped {
                    "[已裝備（LiTo 確認）]"
                } else {
                    "[背包（LiTo 確認）]"
                }));
                assert!(text.contains("期限確認"));
                [1, 0, 0, 0]
            }
            Key::Supply(id) => {
                let stock = o
                    .personal()
                    .supplies()
                    .unwrap()
                    .stocks
                    .0
                    .iter()
                    .find(|s| s.item == id)
                    .unwrap();
                assert!(text.contains(&format!(
                    "補給 #{id}：剩餘 {}／開場 {}",
                    stock.remaining, stock.initial
                )));
                [0, 1, 0, 0]
            }
            Key::Ammo(id) => {
                let ammo = o.personal().ammunition().unwrap();
                let stock = ammo.stocks.iter().find(|s| s.facts.item_id == id).unwrap();
                assert!(text.contains(&format!(
                    "箭矢 #{id}{}：剩餘 {}／開場 {}",
                    if ammo.selected_item_id == id {
                        " [已選取]"
                    } else {
                        ""
                    },
                    stock.remaining,
                    stock.initial
                )));
                [0, 0, 1, 0]
            }
            Key::Material(id) => {
                let m = o.data().personal.materials.as_ref().unwrap();
                let stock = m.stocks.0.iter().find(|s| s.item == id).unwrap();
                assert!(text.contains(&format!(
                    "材料 #{id} · {}：剩餘 {}／開場 {}／已消耗 {}／單位重量 {}",
                    battle_inventory::material_name(id, str::to_owned),
                    stock.remaining,
                    stock.initial,
                    stock.initial - stock.remaining,
                    stock.unit_weight
                )));
                assert!(text.contains("不能在此使用或裝備"));
                assert!(row.equip.is_none() && row.unequip.is_none() && row.local.is_none());
                assert!(model.local(row.key).is_none());
                [0, 0, 0, 1]
            }
        };
        assert_eq!(
            counts, expected,
            "only selected namespace/row is expanded: {:?}",
            row.key
        );
        let mut earlier = focus;
        earlier.stamp.revision -= 1;
        assert_eq!(
            battle_details::inventory(o, Some(earlier), str::to_owned),
            Some(text),
            "same-owner focus displays current confirmed data, not old stock values"
        );
        for field in 0..9 {
            let mut bad = focus;
            match field {
                0 => bad.stamp.context.key.source_id += 1,
                1 => bad.stamp.context.key.object_id += 1,
                2 => bad.stamp.context.epoch ^= 1,
                3 => bad.stamp.channel ^= 1,
                4..=6 => bad.stamp.world[field - 4] += 1,
                7 => bad.stamp.entity += 1,
                _ => bad.stamp.life += 1,
            }
            assert!(
                battle_details::inventory(o, Some(bad), str::to_owned).is_none(),
                "foreign focus {field}"
            );
        }
        let mut missing = focus;
        missing.key = match row.key {
            Key::Source(_) => Key::Source(u64::MAX),
            Key::Supply(_) => Key::Supply(i64::MAX),
            Key::Ammo(_) => Key::Ammo(i64::MAX),
            Key::Material(_) => Key::Material(i64::MAX),
        };
        assert!(battle_details::inventory(o, Some(missing), str::to_owned).is_none());
    }
    assert_eq!(
        *model, saved_model,
        "reading details must not change command proofs"
    );
    eprintln!(
        "INVENTORY_DETAILS source={} rows={} revision={}",
        model.stamp.context.key.source_id,
        model.rows.len(),
        model.stamp.revision
    );
}

fn assert_inventory_long_description(surface: &Surface, model: &battle_inventory::Model) {
    use crate::modern_lito::{battle_details, battle_inventory::Key};
    let row = model
        .rows
        .iter()
        .find(|r| matches!(r.key, Key::Source(_)))
        .unwrap();
    let focus = model.selection(row.key).unwrap();
    let long = format!("{}說明尾端驗證", "逐行物品說明\r\n".repeat(300));
    let calls = std::cell::Cell::new(0);
    let text = battle_details::inventory(&surface.observation, Some(focus), |_| {
        calls.set(calls.get() + 1);
        long.clone()
    })
    .unwrap();
    assert_eq!(calls.get(), 1, "do not expand other source items");
    assert!(text.contains(&long) && text.contains("說明尾端驗證"));
    assert_eq!(text.matches("逐行物品說明").count(), 300);
    assert!(
        battle_details::inventory(&surface.observation, Some(focus), |_| "bad\0label".into())
            .is_none()
    );
    assert!(
        battle_details::inventory(&surface.observation, Some(focus), |_| "界"
            .repeat(crate::modern_lito::world_window::MAX_DETAILS_UTF16))
        .is_none()
    );
}

fn gear_snapshot(c: &Client) -> Arc<equipment::Snapshot> {
    let equipment::State::Ready(g) = c.view().equipment else {
        panic!("no real equipment token");
    };
    g
}

#[derive(Debug, Default)]
struct UiAudit {
    unequipped: bool,
    reequipped: bool,
    moved: bool,
    returned: bool,
    closed: bool,
    close_requested_before_deadline: bool,
    move_origin: Option<(Stamp, i64, WorldCell)>,
}
impl UiAudit {
    fn observe_move(&mut self, stamp: Stamp, sequence: i64, cell: WorldCell) {
        if !self.reequipped {
            return;
        }
        let Some((before, before_sequence, before_cell)) = self.move_origin else {
            self.move_origin = Some((stamp, sequence, cell));
            return;
        };
        self.moved |= stamp.context == before.context
            && stamp.channel == before.channel
            && stamp.world == before.world
            && stamp.entity == before.entity
            && stamp.life == before.life
            && stamp.revision > before.revision
            && sequence > before_sequence
            && cell != before_cell;
    }
}
struct UiController {
    owner: SessionController,
    audit: Arc<std::sync::Mutex<UiAudit>>,
    item: u64,
    operator_deadline: Instant,
}
impl LoginController for UiController {
    fn poll(&mut self) {
        self.owner.poll();
    }
    fn command(&mut self, q: LoginCommand) -> Result<(), String> {
        let closing =
            matches!(&q, LoginCommand::Shutdown) && Instant::now() < self.operator_deadline;
        let result = self.owner.command(q);
        if result.is_ok() && closing {
            let mut audit = self.audit.lock().unwrap();
            audit.close_requested_before_deadline |= audit.returned;
        }
        result
    }
    fn is_quiescent(&self) -> bool {
        self.owner.is_quiescent()
    }
    fn snapshot(&self) -> LoginSnapshot {
        let view = self.owner.snapshot();
        let mut audit = self.audit.lock().unwrap();
        if let Some(o) = view.canonical.observation() {
            if let Some(e) = o.data().personal.equipment.as_ref() {
                audit.unequipped |=
                    e.revision >= 1 && o.personal().equipped(self.item as i64) == Some(false);
                audit.reequipped |= audit.unequipped
                    && e.revision >= 2
                    && o.personal().equipped(self.item as i64) == Some(true);
            }
            if let Some(actor) = o
                .data()
                .actors
                .0
                .iter()
                .find(|a| a.entity == o.data().personal.entity)
            {
                audit.observe_move(
                    Stamp::of(o),
                    actor.last_move_sequence,
                    WorldCell {
                        x: actor.position.x,
                        y: actor.position.y,
                    },
                );
            }
        }
        audit.returned |= audit.reequipped
            && !view.world_pending
            && !view.canonical.attached()
            && view.phase == LoginPhase::InLobby;
        audit.closed |= audit.returned
            && audit.close_requested_before_deadline
            && view.phase == LoginPhase::Disconnected
            && self.owner.is_quiescent();
        view
    }
}
#[test]
fn equipment_ui_move_audit_requires_a_later_same_life_move_after_reequip() {
    let mut audit = UiAudit::default();
    let before = Stamp {
        context: SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: 1,
        },
        channel: 0,
        world: [10, 20, 30],
        entity: 1,
        life: 1,
        revision: 10,
    };
    let from = WorldCell { x: 32768, y: 32768 };
    let to = WorldCell { x: 32769, y: 32768 };
    audit.observe_move(before, 5, from);
    assert!(!audit.moved && audit.move_origin.is_none());
    audit.reequipped = true;
    audit.observe_move(before, 5, from);
    audit.observe_move(before, 5, from);
    assert!(
        !audit.moved,
        "a pre-existing nonzero sequence is not UI movement"
    );
    let mut next = before;
    next.revision += 1;
    audit.observe_move(next, 5, to);
    audit.observe_move(next, 6, from);
    assert!(
        !audit.moved,
        "both position and movement sequence must advance"
    );
    next.life += 1;
    audit.observe_move(next, 6, to);
    assert!(!audit.moved, "a respawn is not this life's movement");
    next.life = before.life;
    audit.observe_move(next, 6, to);
    assert!(audit.moved);
}
#[test]
#[ignore = "two v41 controllers + shared514/SPR, inventory SET/render/MOVE/return, isolated gear store; no HWND"]
fn java_v41_equipment_controller_inventory_atomic_render_move_and_return() {
    flow(false);
}
#[test]
#[ignore = "INTERACTIVE v41 inventory HWND: operator UNEQUIP, EQUIP, MOVE, return and close; isolated service/store"]
fn java_v41_equipment_window_inventory_move_return_and_close() {
    flow(true);
}
fn flow(window: bool) {
    let d = if window {
        Deadline(Instant::now() + Duration::from_secs(220))
    } else {
        Deadline::new()
    };
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernCombatTestPeer",
        "canonical-equipment",
        window.then_some("240000"),
    );
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new_canonical_development(profile(
            peer.endpoint,
            &shared,
            &ca,
        )),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        send(&mut cs, i, &d, || {
            LoginCommand::SelectSource(if i == 0 { 7 } else { 42 })
        });
        wait(&mut cs, &d, "roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        let roster = cs[i].view();
        let context = SelectionContext {
            key: CharacterKey {
                source_id: roster.selected_source.unwrap(),
                object_id: 3333,
            },
            epoch: roster.roster_epoch,
        };
        send(&mut cs, i, &d, || LoginCommand::SelectCharacter(context));
        wait(&mut cs, &d, "catalogs", |cs| {
            let v = cs[i].view();
            v.phase == LoginPhase::ChoosingChannel
                && !v.record_pending
                && !v.spell_pending
                && matches!(v.equipment, equipment::State::Ready(_))
        });
        send(&mut cs, i, &d, || LoginCommand::EnterChannel {
            context,
            channel_id: CHANNEL,
        });
        wait(&mut cs, &d, "lobby", |cs| {
            let v = cs[i].view();
            v.phase == LoginPhase::InLobby && v.canonical.equipment_supported() && !v.room_pending
        });
    }
    let contexts = cs.each_ref().map(|c| c.view().navigation.unwrap().context);
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
                        name: "Equipment controllers".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "create", |cs| {
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
    wait(&mut cs, &d, "join", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    for i in 0..2 {
        send(&mut cs, i, &d, || {
            LoginCommand::Rooms(rooms::Request::read(contexts[i], CHANNEL).unwrap())
        });
        wait(&mut cs, &d, "fresh room before ready", |cs| {
            !cs[i].view().room_pending
        });
        let room = cs[i].view().rooms.joined().unwrap().clone();
        send(&mut cs, i, &d, || {
            LoginCommand::Rooms(room_command(contexts[i], &room, rooms::Action::Ready, 1))
        });
        wait(&mut cs, &d, "ready", |cs| !cs[i].view().room_pending);
        assert!(
            cs[i]
                .view()
                .rooms
                .joined()
                .unwrap()
                .members
                .iter()
                .find(|m| m.key == contexts[i].key)
                .unwrap()
                .ready
        );
    }
    for c in &mut cs {
        c.load = true;
    }
    wait(&mut cs, &d, "active and rendered", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view()
                    .canonical
                    .lifecycle()
                    .is_some_and(|l| l.phase == world::Phase::Active)
        })
    });
    let original = gear_snapshot(&cs[0]);
    let guest_original = gear_snapshot(&cs[1]);
    for c in &cs {
        assert!(matches!(
            c.view().shop,
            crate::modern_lito::shop_protocol::State::Unsupported
        ));
    }
    let item = original.flags.iter().find(|f| f.equipped).unwrap().item_id;
    for c in &mut cs {
        let v = c.view();
        assert!(v.world_owns_lobby());
        assert!(!login_window::can_open_shop(&v));
        assert!(!login_window::can_read_record(&v));
        assert!(!login_window::gm_input_ready(&v));
        for action in [
            rooms::Action::Read,
            rooms::Action::Configure,
            rooms::Action::Leave,
            rooms::Action::Team,
            rooms::Action::Ready,
        ] {
            assert!(login_window::room_common(&v, action, None, 0).is_none());
        }
        assert!(c.controller.command(LoginCommand::ReadRecord).is_err());
    }
    if window {
        for c in &mut cs {
            c.runtime.shutdown();
        }
        while cs.iter().any(|c| !c.runtime.is_quiescent()) {
            for c in &mut cs {
                c.runtime.poll();
            }
            d.pause("retire preflight renderers");
        }
        let [owner, mut guest] = cs;
        let (stop, receive) = mpsc::sync_channel::<()>(1);
        let audit = Arc::new(std::sync::Mutex::new(UiAudit::default()));
        let until = d.0;
        let result = thread::scope(|scope| {
            let worker = scope.spawn(move || {
                let d = Deadline(until);
                while receive.recv_timeout(Duration::from_millis(100))
                    == Err(mpsc::RecvTimeoutError::Timeout)
                {
                    guest.controller.poll();
                }
                guest.controller.command(LoginCommand::Shutdown).unwrap();
                while !guest.controller.is_quiescent() {
                    guest.controller.poll();
                    d.pause("interactive guest shutdown");
                }
            });
            let operator_deadline = Instant::now() + Duration::from_secs(150);
            let result = crate::modern_lito::login_window::run_connected_test(
                &profile(peer.endpoint, &shared, &ca),
                Box::new(UiController {
                    owner: owner.controller,
                    audit: audit.clone(),
                    item,
                    operator_deadline,
                }),
                operator_deadline,
            );
            let _ = stop.send(());
            worker.join().unwrap();
            result
        });
        eprintln!("equipment HWND audit: {:?}", *audit.lock().unwrap());
        result.unwrap();
        let audit = audit.lock().unwrap();
        assert!(
            audit.unequipped && audit.reequipped && audit.moved && audit.returned && audit.closed,
            "real equipment UI flow incomplete"
        );
        peer.expect_successful_exit(&d);
        return;
    }
    let first_picture = cs[0].surface.clone().unwrap();
    let old_view = cs[0].view();
    let model = battle_inventory::model(&first_picture, &old_view, str::to_owned).unwrap();
    assert_inventory_details(&first_picture, &model);
    assert_inventory_long_description(&first_picture, &model);
    let stale = model.request(item, false).unwrap();
    assert!(model.request(item, true).is_none());
    assert!(model.request(u64::MAX, false).is_none());
    let q = battle_inventory::command(stale, &first_picture.observation, &old_view).unwrap();
    send(&mut cs, 0, &d, || LoginCommand::Canonical(q.clone()));
    assert!(cs[0].view().world_pending);
    let pending = cs[0].view();
    assert!(
        Arc::ptr_eq(&original, &gear_snapshot(&cs[0])),
        "no optimistic saved state"
    );
    assert!(battle_inventory::command(stale, &first_picture.observation, &pending).is_none());
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(q.clone()))
        .is_err());
    assert!(cs[0]
        .controller
        .command(LoginCommand::ReturnCharacters(contexts[0]))
        .is_err());
    wait(&mut cs, &d, "unequip receipt and actual resources", |cs| {
        cs[0].rendered() && gear_snapshot(&cs[0]).revision == 1
    });
    let unequipped = cs[0].view();
    assert_eq!(
        unequipped
            .canonical
            .received_after(&q, old_view.canonical.receipt_epoch()),
        Some(Outcome::Equipment(
            gear::Reason::None,
            gear::Disposition::Applied,
            Some(equipment::Status::Ok)
        ))
    );
    assert_eq!(gear_snapshot(&cs[0]).equipped(item), Some(false));
    assert_eq!(original.equipped(item), Some(true));
    assert!(battle_inventory::command(stale, &first_picture.observation, &unequipped).is_none());
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(q))
        .is_err());
    let current = cs[0].surface.clone().unwrap();
    let owner_id = current.observation.data().personal.entity;
    let weapon = |s: &Surface| {
        s.observation
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == owner_id)
            .unwrap()
            .vital
            .as_ref()
            .unwrap()
            .weapon_type
    };
    assert_ne!(
        weapon(&first_picture),
        weapon(&current),
        "real live weapon changes after UNEQUIP"
    );
    assert_eq!(gear_snapshot(&cs[1]), guest_original);
    let model = battle_inventory::model(&current, &unequipped, str::to_owned).unwrap();
    assert_inventory_details(&current, &model);
    let re = model.request(item, true).unwrap();
    assert!(model.request(item, false).is_none());
    let mut wrong = re;
    wrong.stamp.life += 1;
    assert!(battle_inventory::command(wrong, &current.observation, &unequipped).is_none());
    wrong = re;
    wrong.stamp.context.key.source_id = 42;
    wrong.equipment.context = wrong.stamp.context;
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::equipment(wrong)))
        .is_err());
    let q = battle_inventory::command(re, &current.observation, &unequipped).unwrap();
    send(&mut cs, 0, &d, || LoginCommand::Canonical(q.clone()));
    wait(&mut cs, &d, "reequip receipt and actual resources", |cs| {
        cs[0].rendered() && gear_snapshot(&cs[0]).revision == 2
    });
    let restored = cs[0].surface.clone().unwrap();
    assert_eq!(weapon(&first_picture), weapon(&restored));
    assert_eq!(gear_snapshot(&cs[0]).equipped(item), Some(true));
    // A stale/nonexistent inventory ID can still reach the protocol from an
    // external caller. Exercise the real server rejection, not a local mock.
    let saved = gear_snapshot(&cs[0]);
    let prior = cs[0].view().canonical.receipt_epoch();
    let denied_q = Command::equipment(
        gear::Request::from_observation(&restored.observation, &saved, 99999, true).unwrap(),
    );
    send(&mut cs, 0, &d, || LoginCommand::Canonical(denied_q.clone()));
    wait(&mut cs, &d, "exact server equipment denial", |cs| {
        cs[0].rendered()
            && cs[0]
                .view()
                .canonical
                .received_after(&denied_q, prior)
                .is_some()
    });
    let denied = cs[0].view();
    assert_eq!(
        denied.canonical.received_after(&denied_q, prior),
        Some(Outcome::Equipment(
            gear::Reason::None,
            gear::Disposition::NotCommitted,
            Some(equipment::Status::Denied)
        ))
    );
    assert_eq!(
        denied.canonical.equipment_reply().unwrap().reason,
        Some(equipment::Reason::UnknownItem)
    );
    assert!(denied.status.contains("背包中沒有這個物品"));
    assert!(denied
        .canonical
        .command_status()
        .contains("背包中沒有這個物品"));
    let now = gear_snapshot(&cs[0]);
    assert_eq!(now.revision, saved.revision);
    assert_eq!(now.flags, saved.flags);
    assert_eq!(now.view_token, saved.view_token);
    assert_eq!(weapon(cs[0].surface.as_ref().unwrap()), weapon(&restored));
    assert_eq!(gear_snapshot(&cs[1]), guest_original);
    let restored = cs[0].surface.clone().unwrap();
    let o = &restored.observation;
    let a = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == owner_id)
        .unwrap();
    let from = WorldCell {
        x: a.position.x,
        y: a.position.y,
    };
    let (h, to) = DIRECTIONS
        .iter()
        .enumerate()
        .find_map(|(h, (dx, dy))| {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            (restored
                .navigation
                .as_ref()
                .unwrap()
                .map()
                .can_step(from, to)
                && o.data()
                    .actors
                    .0
                    .iter()
                    .all(|a| a.position.x != to.x || a.position.y != to.y))
            .then_some((h, to))
        })
        .unwrap();
    let w = o.data().world.identity.clone();
    let move_q = Command::operation(
        Guard {
            stamp: Stamp::of(o),
            target: None,
        },
        Operation::Move(world::Request {
            action: world::Action::Move,
            heading: h as u8,
            sequence: a.last_move_sequence as u64 + 1,
            from_x: from.x,
            from_y: from.y,
            match_id: w.match_id as u64,
            generation: w.generation as u64,
            ..world::Request::read(contexts[0], CHANNEL)
        }),
    );
    send(&mut cs, 0, &d, || LoginCommand::Canonical(move_q.clone()));
    wait(&mut cs, &d, "movement after equipment", |cs| {
        cs[0].rendered()
            && cs[0]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .actors
                .0
                .iter()
                .any(|a| a.entity == owner_id && a.position.x == to.x && a.position.y == to.y)
    });
    for i in 0..2 {
        let before_leave = gear_snapshot(&cs[i]);
        let q = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&w))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(q.clone()));
        assert!(cs[i].view().canonical.observation().is_none());
        wait(&mut cs, &d, "world leave", |cs| {
            !cs[i].view().world_pending && !cs[i].view().canonical.attached()
        });
        assert!(cs[i]
            .controller
            .command(LoginCommand::Canonical(Command::equipment(re)))
            .is_err());
        let returned = cs[i].view();
        assert!(!returned.world_owns_lobby());
        assert!(returned.status.contains("已重新讀取伺服器確認的裝備"));
        assert!(
            !Arc::ptr_eq(&before_leave, &gear_snapshot(&cs[i])),
            "LEAVE must perform actual READ"
        );
        assert!(login_window::room_common(&returned, rooms::Action::Read, None, 0).is_some());
        assert!(matches!(
            returned.shop,
            crate::modern_lito::shop_protocol::State::Unsupported
        ));
        assert!(!login_window::can_open_shop(&returned));
    }
    assert_eq!(gear_snapshot(&cs[0]).revision, 2);
    assert_eq!(gear_snapshot(&cs[0]).equipped(item), Some(true));
    let guest_read = gear_snapshot(&cs[1]);
    assert!(guest_read.evaluated_at_millis >= guest_original.evaluated_at_millis);
    let mut guest_expected = (*guest_original).clone();
    // A new READ reevaluates wall-clock lifetimes. All saved facts, tokens,
    // flags, revisions and lifetime decisions must still remain unchanged.
    guest_expected.evaluated_at_millis = guest_read.evaluated_at_millis;
    assert!(
        guest_read.as_ref() == &guest_expected,
        "guest source or equipment was changed by the other owner"
    );
    if !window {
        returned::verify(&mut cs, &d, contexts);
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    loop {
        for c in &mut cs {
            c.controller.poll();
            c.runtime.poll();
        }
        if cs
            .iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
        {
            break;
        }
        d.pause("owned controller/resource shutdown");
    }
    peer.expect_successful_exit(&d);
}
