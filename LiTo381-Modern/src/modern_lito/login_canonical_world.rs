//! Canonical HWND owner. Never asks for a legacy Frame or converts a canonical
//! revision to legacy command authority. Only this branch owns v39+ pixels.
use super::*;
use crate::modern_lito::{
    battle_control::Command,
    battle_input,
    battle_payload::data as d,
    battle_player,
    battle_runtime::Status,
    battle_window::{Intent, Kind, Local, Stamp, Target},
};

fn retire(state: &WindowState, closing: bool) {
    let retired = state.world_window.borrow_mut().take();
    drop(retired);
    state.world_details_cache.borrow_mut().take();
    state.canonical_route.borrow_mut().cancel();
    state.canonical_route.borrow_mut().poll();
    state.world_navigation.borrow_mut().cancel();
    state.world_navigation.borrow_mut().poll();
    state.world_audio.borrow_mut().stop();
    state.battle_audio.borrow_mut().stop();
    if closing {
        state.world_runtime.borrow_mut().shutdown();
        state.battle_runtime.borrow_mut().shutdown();
    } else {
        state.world_runtime.borrow_mut().clear();
        state.battle_runtime.borrow_mut().clear();
    }
    state.world_runtime.borrow_mut().poll();
    state.battle_runtime.borrow_mut().poll();
    state.world_load_ack.set(None);
}
fn error(state: &WindowState, message: &str) {
    if let Ok(mut ui) = state.ui.try_borrow_mut() {
        ui.local_error = Some(message.into());
    }
}
fn send_command(state: &WindowState, command: Command) -> bool {
    let Ok(mut ui) = state.ui.try_borrow_mut() else {
        return false;
    };
    match ui.controller.command(LoginCommand::Canonical(command)) {
        Ok(()) => {
            ui.local_error = None;
            true
        }
        Err(_) => {
            ui.local_error = Some("操作狀態已更新或尚有指令在途；未重送遊戲操作。".into());
            false
        }
    }
}
fn policy(o: &crate::modern_lito::battle_follow::Observation) -> world::MatchDescriptor {
    let q = &o.data().world.descriptor;
    world::MatchDescriptor {
        ruleset: match q.ruleset {
            d::Ruleset::SceneOnly => world::MatchRuleset::SceneOnly,
            d::Ruleset::PvpSkirmish => world::MatchRuleset::PvpSkirmish,
            d::Ruleset::DeathmatchElimination => world::MatchRuleset::DeathmatchElimination,
            d::Ruleset::HonorTowers => world::MatchRuleset::HonorTowers,
            d::Ruleset::PetRace => world::MatchRuleset::PetRace,
            d::Ruleset::NpcEncounter => world::MatchRuleset::NpcEncounter,
            d::Ruleset::CooperativeTimeAttack => world::MatchRuleset::CooperativeTimeAttack,
            d::Ruleset::SiegeObjectives => world::MatchRuleset::SiegeObjectives,
        },
        features: q.features as u32,
        preparation_timeout_millis: q.preparation_timeout_millis as u32,
        load_timeout_millis: q.load_timeout_millis as u32,
        countdown_nanos: q.countdown_nanos as u64,
        active_timeout_millis: q.active_timeout_millis as u32,
    }
}
fn leaving(state: &WindowState) -> bool {
    state.world_scope.leave_queued.get() || state.world_scope.leave_inflight.get()
}
pub(crate) fn result_status(
    base: String,
    observation: Option<&crate::modern_lito::battle_follow::Observation>,
) -> String {
    let Some(result) = observation.and_then(crate::modern_lito::battle_window::result_notice) else {
        return base;
    };
    format!("{result}\r\n{base}")
}

pub(super) unsafe fn refresh(hwnd: HWND, state: &WindowState, view: &LoginSnapshot, closing: bool) {
    let scope = if closing { None } else { world_ui_scope(view) };
    let changed = state.world_scope.replace(scope);
    if changed || closing || scope.is_none() {
        retire(state, closing);
        if closing || scope.is_none() {
            return;
        }
    }
    // The old resource/Frame owner is not an alternate renderer for this connection.
    state.world_audio.borrow_mut().stop();
    state.world_runtime.borrow_mut().clear();
    state.world_runtime.borrow_mut().poll();
    state.world_navigation.borrow_mut().cancel();
    state.world_navigation.borrow_mut().poll();
    let foreground = GetAncestor(GetForegroundWindow(), GA_ROOTOWNER) == hwnd;
    let surface = match battle_player::update(&mut state.battle_runtime.borrow_mut(), view, false) {
        Ok(Status::Ready(s)) if battle_player::admitted(&s, view) => Some(s),
        Ok(Status::Failed(_)) | Err(_) => {
            state
                .world_scope
                .failure
                .set(Some("共享資源／新版場景驗證失敗，正在返回房間。"));
            state.world_scope.leave_queued.set(true);
            None
        }
        _ => None,
    };
    let audio_notice = state
        .battle_audio
        .borrow_mut()
        .tick(&state.battle_runtime.borrow(), foreground, Instant::now())
        .notice;
    if state.world_window.borrow().is_none() {
        let preserve = state
            .chat_window
            .borrow()
            .as_ref()
            .is_some_and(super::super::chat_window::ChatWindow::has_keyboard_focus)
            || state.direct_chat_window.borrow().as_ref().is_some_and(
                super::super::direct_chat_window::DirectChatWindow::has_keyboard_focus,
            );
        match WorldWindow::open_with_shared_inventory(hwnd, !preserve, &state.shared_root) {
            Ok(w) => *state.world_window.borrow_mut() = Some(w),
            Err(_) => {
                state
                    .world_scope
                    .failure
                    .set(Some("無法建立新版世界視窗，正在返回房間。"));
                state.world_scope.leave_queued.set(true);
            }
        }
    }
    let (action, painted, settings, chat) =
        state
            .world_window
            .borrow()
            .as_ref()
            .map_or((None, None, false, false), |w| {
                (
                    w.take_action(),
                    w.painted_surface(),
                    w.take_settings_request(),
                    w.take_chat_request(),
                )
            });
    if settings && state.settings.borrow_mut().show(hwnd).is_err() {
        error(state, "無法開啟遊戲設定；未送出遊戲操作。");
    }
    if chat {
        open_chat(hwnd, state);
    }
    // A VIP preview can be displayed, but painted gameplay intents still
    // require the older command-qualified observation (currently absent).
    let painted = painted.filter(|s| battle_player::admitted_observation(&s.observation, view));
    let mut submitted = false;
    match scoped_world_action(changed, action) {
        Some(WorldAction::Leave) => {
            state.canonical_route.borrow_mut().cancel();
            state.world_scope.leave_queued.set(true);
            if !view.world_pending {
                state.world_scope.leave_inflight.set(false);
            }
        }
        Some(
            WorldAction::CancelRoute
            | WorldAction::Canonical(Intent {
                kind: Kind::Cancel, ..
            }),
        ) => state.canonical_route.borrow_mut().cancel(),
        Some(WorldAction::Canonical(intent)) if !leaving(state) => {
            if let Some(painted) = &painted {
                if matches!(intent.kind, Kind::Navigate(_)) {
                    if state
                        .canonical_route
                        .borrow_mut()
                        .request(painted, intent, view)
                        .is_err()
                    {
                        error(state, "點選的地形或生命已更新；未啟動路徑。");
                    }
                } else {
                    state.canonical_route.borrow_mut().cancel();
                    match battle_input::command(painted, intent, view) {
                        Ok(q) => submitted = send_command(state, q),
                        Err(e) => error(state, &e.to_string()),
                    }
                }
            } else {
                error(state, "尚未繪製目前角色的場景；未送出操作。");
            }
        }
        Some(WorldAction::CanonicalRevival(proof)) if !leaving(state) => {
            state.canonical_route.borrow_mut().cancel();
            if let Some(q) = painted
                .as_ref()
                .filter(|s| {
                    crate::modern_lito::battle_revival::Request::from_observation(&s.observation)
                        == Some(proof)
                })
                .and_then(|s| battle_player::revival_command(s, view))
            {
                submitted = send_command(state, q);
            } else {
                error(state, "死亡狀態或生命已更新；未送出復活確認。");
            }
        }
        Some(WorldAction::CanonicalSkill(choice)) if !leaving(state) => {
            use crate::modern_lito::battle_skillbook::{self as skillbook, Selection};
            state.canonical_route.borrow_mut().cancel();
            match painted
                .as_ref()
                .and_then(|s| skillbook::select(choice, s, view))
            {
                Some(Selection::Target(skill)) => {
                    if let Some(window) = state.world_window.borrow().as_ref() {
                        if window.selected_skill(skill).is_err() {
                            state.text_fault.set(true);
                        }
                    }
                }
                Some(Selection::Cast(intent)) => {
                    match battle_input::command(painted.as_ref().unwrap(), intent, view) {
                        Ok(q) => {
                            submitted = send_command(state, q.clone());
                            if submitted {
                                if let Some(window) = state.world_window.borrow().as_ref() {
                                    window.submitted_hotbar_command(choice.stamp, q,
                                        view.canonical.receipt_epoch());
                                    window.close_details();
                                }
                            }
                        }
                        Err(e) => error(state, &e.to_string()),
                    }
                }
                None => error(
                    state,
                    "技能、生命或場景已更新；未送出操作，請重新選擇技能。",
                ),
            }
        }
        Some(WorldAction::CanonicalInventory(intent)) if !leaving(state) => {
            state.canonical_route.borrow_mut().cancel();
            if let Some(q) = painted
                .as_ref()
                .and_then(|s| crate::modern_lito::battle_inventory::local_command(intent, s, view))
            {
                submitted = send_command(state, q.clone());
                if submitted {
                    if let Some(window) = state.world_window.borrow().as_ref() {
                        window.submitted_hotbar_command(intent.stamp, q,
                            view.canonical.receipt_epoch());
                    }
                }
            } else {
                error(
                    state,
                    "本場物品、數量或生命已更新；未送出操作，請重新選擇。",
                );
            }
        }
        Some(WorldAction::CanonicalEquipment(proof)) if !leaving(state) => {
            state.canonical_route.borrow_mut().cancel();
            if let Some(q) = painted.as_ref().and_then(|s| {
                crate::modern_lito::battle_inventory::command(proof, &s.observation, view)
            }) {
                submitted = send_command(state, q);
            } else {
                error(
                    state,
                    "裝備、生命或場景已更新；本次未送出，請重新確認物品。",
                );
            }
        }
        Some(_) => {
            state.canonical_route.borrow_mut().cancel();
            error(state, "新版世界不接受沒有畫面生命綁定的舊操作。");
        }
        None => {}
    }
    if leaving(state) {
        state.canonical_route.borrow_mut().cancel();
    }
    if state.world_scope.leave_queued.get()
        && !view.world_pending
        && !state.world_scope.leave_inflight.get()
    {
        if let Some(q) = view.canonical.leave_command() {
            if send_command(state, q) {
                state.world_scope.leave_queued.set(false);
                state.world_scope.leave_inflight.set(true);
                submitted = true;
            }
        }
    }
    if !submitted && !leaving(state) {
        let q = state
            .canonical_route
            .borrow_mut()
            .next(painted.as_deref(), view, Instant::now());
        if let Some(q) = q {
            if send_command(state, q.clone()) {
                submitted = true;
                if state.canonical_route.borrow_mut().submitted(&q).is_err() {
                    state.canonical_route.borrow_mut().cancel();
                    error(state, "移動已送出，但路徑狀態已更換；停止後續步進。");
                }
            } else {
                state.canonical_route.borrow_mut().cancel();
            }
        }
    }
    state.canonical_route.borrow_mut().poll();
    let ready = !submitted
        && !leaving(state)
        && !view.world_pending
        && painted
            .as_ref()
            .is_some_and(|s| battle_player::controls_ready(s, view));
    if let Some(window) = state.world_window.borrow().as_ref() {
        window.set_chat_available(view.chat.supported() && !leaving(state));
        let mut inventory = if !leaving(state)
            && window.details_page()
                == Some(crate::modern_lito::world_window::DetailsPage::Inventory)
        {
            painted.as_ref().and_then(|s| {
                crate::modern_lito::battle_inventory::model(s, view, |t| {
                    state.descriptions.borrow().state.item_label(t)
                })
            })
        } else {
            None
        };
        if !ready {
            if let Some(m) = inventory.as_mut() {
                m.clear_proofs();
            }
        }
        let mut hotbar_inventory = if inventory.is_some() {
            inventory.clone()
        } else if !leaving(state) {
            painted.as_ref().and_then(|s| {
                crate::modern_lito::battle_inventory::model(s, view, |t| {
                    state.descriptions.borrow().state.item_label(t)
                })
            })
        } else { None };
        if !ready {
            if let Some(m) = hotbar_inventory.as_mut() { m.clear_proofs(); }
        }
        if window.update_inventory(inventory).is_err() {
            state.text_fault.set(true);
        }
        if window.update_character_panel(
            painted.as_ref().map(|s| &s.observation), hotbar_inventory.as_ref()).is_err() {
            state.text_fault.set(true);
        }
        let mut hotbar_skills = if !leaving(state) {
            painted.as_ref().and_then(|s| {
                crate::modern_lito::battle_skillbook::model(s, view, |t| {
                    state.descriptions.borrow().state.item_label(t)
                })
            })
        } else { None };
        if !ready {
            if let Some(m) = hotbar_skills.as_mut() { m.clear_proofs(); }
        }
        if window.update_hotbar(hotbar_inventory, hotbar_skills.clone(), &view.canonical).is_err() {
            state.text_fault.set(true);
        }
        let skillbook = hotbar_skills.filter(|_| window.details_page()
            == Some(crate::modern_lito::world_window::DetailsPage::Skills));
        if window.update_skillbook(skillbook).is_err() {
            state.text_fault.set(true);
        }
        if let Some(page) = window.details_page() {
            let descriptions = state.descriptions.borrow();
            let o = painted
                .as_ref()
                .filter(|_| !leaving(state))
                .map(|s| &s.observation);
            let mut key = WorldDetailsCache::new(view, page, descriptions.revision);
            key.inventory = window.inventory_selection();
            key.character_equipped = window.character_equipped_selection();
            key.canonical = o.map(|o| {
                (
                    Stamp::of(o),
                    o.data().world.loading_elapsed_nanos,
                    o.data().world.active_elapsed_nanos,
                )
            });
            if window.details_need_text()
                || state.world_details_cache.borrow().as_ref() != Some(&key)
            {
                let text = o
                    .and_then(|o| {
                        let label = |t: &str| descriptions.state.item_label(t);
                        if page == crate::modern_lito::world_window::DetailsPage::Inventory {
                            crate::modern_lito::battle_details::inventory(o, key.inventory, label)
                        } else if page == crate::modern_lito::world_window::DetailsPage::Character {
                            crate::modern_lito::battle_details::character(o, key.character_equipped, label)
                        } else {
                            crate::modern_lito::battle_details::text(o, page, label)
                        }
                    })
                    .unwrap_or_else(|| "目前場景或角色資料尚未一致，已清除舊檢視。".into());
                if window.update_details(&text).is_err() {
                    state.text_fault.set(true);
                }
                *state.world_details_cache.borrow_mut() = Some(key);
            }
        }
        let local = state
            .ui
            .try_borrow()
            .ok()
            .and_then(|ui| ui.local_error.clone());
        let notice = state
            .world_scope
            .failure
            .get()
            .or(local.as_deref())
            .or(audio_notice)
            .or(state.canonical_route.borrow().status().text());
        let mut status = result_status(view.canonical.display_status(view.world_pending, notice),
            surface.as_ref().map(|s| &s.observation));
        if let Some(s) = &surface {
            if let Some(n) = crate::modern_lito::battle_window::presentation_notice(s) {
                status.push_str("\r\n");
                status.push_str(n);
            }
            for n in crate::modern_lito::battle_window::hud(&s.observation) {
                status.push_str("\r\n");
                status.push_str(&n);
            }
        }
        let status: String = status.chars().take(500).collect();
        let can = |kind| ready && battle_input::ready(kind, view);
        let npc = view
            .canonical
            .observation()
            .is_some_and(|o| o.data().npcs.is_some());
        let t = if npc {
            Target::Npc { serial: 1, life: 1 }
        } else {
            Target::Player { entity: 1, life: 1 }
        };
        let attack = can(Kind::Attack(t));
        if window
            .update_with_roster(
                None,
                None,
                &status,
                can(Kind::Local(Local::Step(0))),
                None,
                attack,
                can(Kind::Local(Local::Teleport)),
                battle_input::skill_available(4, view),
                can(Kind::LightArrow(t)),
                None,
            )
            .is_err()
        {
            state.text_fault.set(true);
        }
        if let Some(surface) = surface.as_ref().filter(|_| !leaving(state)) {
            if window
                .update_match_descriptor(Some(&policy(&surface.observation)))
                .is_err()
                || window.update_canonical(surface.clone()).is_err()
            {
                state.text_fault.set(true);
            }
        } else {
            window.invalidate_input();
        }
        if window
            .update_honor(
                battle_input::skill_available(1, view),
                ready && battle_input::healing_ready(view),
            )
            .is_err()
            || window
                .update_disintegrate(
                    battle_input::skill_available(77, view),
                    can(Kind::Disintegrate(t)),
                )
                .is_err()
        {
            state.text_fault.set(true);
        }
        window.update_fire_storm(
            battle_input::skill_available(70, view),
            can(Kind::Local(Local::FireStorm)),
        );
        if window
            .update_target_haste(battle_input::skill_available(43, view), can(Kind::Haste(t)))
            .is_err()
        {
            state.text_fault.set(true);
        }
        if window
            .update_slow(battle_input::skill_available(29, view), can(Kind::Slow(t)))
            .is_err()
        {
            state.text_fault.set(true);
        }
        window.update_greater_haste(
            battle_input::skill_available(54, view),
            can(Kind::Local(Local::GreaterHaste)),
        );
        let revival = painted.as_ref().and_then(|s| {
            crate::modern_lito::battle_revival::Request::from_observation(&s.observation)
        });
        window.update_canonical_revival(
            revival,
            ready
                && painted
                    .as_ref()
                    .is_some_and(|s| battle_player::revival_command(s, view).is_some()),
        );
        let o = view.canonical.observation();
        let labels = std::array::from_fn(|i| {
            let id = item::UI_ITEMS[i];
            let n = o
                .and_then(|o| o.personal().supplies())
                .and_then(|s| s.stocks.0.iter().find(|s| s.item == id))
                .map_or(0, |s| s.remaining);
            let token = if i < 3 {
                ["$26", "$235", "$238"][i].into()
            } else {
                format!(
                    "${}",
                    crate::modern_lito::timed_protocol::Kind::ALL[i - 3].description_id()
                )
            };
            let shortcut = if i < 4 { format!("（F{}）", i + 1) } else { String::new() };
            format!("{} ×{}{}", state.descriptions.borrow().state.item_label(&token), n, shortcut)
        });
        let flags = o.map_or(0, |o| o.data().world.descriptor.features as u32);
        if window
            .update_supplies(
                labels,
                flags & world::MatchDescriptor::ITEMS != 0,
                flags & world::MatchDescriptor::TIMED != 0,
                std::array::from_fn(|i| can(Kind::Local(Local::UseItem(item::UI_ITEMS[i])))),
            )
            .is_err()
        {
            state.text_fault.set(true);
        }
        let labels = std::array::from_fn(|i| {
            let id = crate::modern_lito::ammo_protocol::UI_ITEMS[i];
            let ammo = o.and_then(|o| o.data().personal.ammo.as_ref());
            format!(
                "{} ×{}{}",
                if i == 0 { "銀箭" } else { "米索莉箭" },
                ammo.and_then(|a| a.stocks.0.iter().find(|s| s.item == id))
                    .map_or(0, |s| s.remaining),
                if ammo.is_some_and(|a| a.selected_item == id) {
                    "／已選用"
                } else {
                    ""
                }
            )
        });
        if window
            .update_ammo(
                labels,
                flags & world::MatchDescriptor::AMMO != 0,
                std::array::from_fn(|i| {
                    can(Kind::Local(Local::SelectAmmo(
                        crate::modern_lito::ammo_protocol::UI_ITEMS[i],
                    )))
                }),
            )
            .is_err()
        {
            state.text_fault.set(true);
        }
    }
    if !submitted && !leaving(state) {
        if let Some(q) = surface.as_ref().and_then(|s| {
            battle_player::canonical_loaded_command(&state.battle_runtime.borrow(), s, view)
        }) {
            send_command(state, q);
        }
    }
}
