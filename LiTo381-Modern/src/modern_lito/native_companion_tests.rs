//! Pure receive/request fixtures: no server, DB, resources or game process.
use super::*;
use crate::modern_lito::{
    native_inventory::{Item, Owner},
    native_item_actions,
};
const PET: u32 = 0x9000_0038;
const ENEMY: u32 = 0x9000_0040;
fn active() -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = LoginPhase::BattlePending;
    c.state.game_connected = true;
    c.state.world = crate::modern_lito::native_world_scene::movement_tests::world();
    let own = c.state.world.own_actor().unwrap().clone();
    let mut pet = own.clone();
    pet.id = PET;
    pet.name = "Pet".into();
    pet.incarnation += 1;
    pet.relations.master = own.relations.name.clone();
    pet.master = own.name.clone();
    pet.x += 1;
    c.state.world.actors.insert(PET, pet);
    let mut enemy = own;
    enemy.id = ENEMY;
    enemy.name = "Enemy".into();
    enemy.x += 2;
    c.state.world.actors.insert(ENEMY, enemy);
    c.state.inventory.owner = Some(Owner {
        connection: 0,
        selection: 1,
        character: "Owner".into(),
    });
    c.state.inventory.items.insert(
        77,
        Item {
            id: 77,
            use_type: 0,
            charge: 0,
            icon: 1,
            blessing: 0,
            count: 4,
            identified: 1,
            name: "Collar".into(),
            detail: vec![],
        },
    );
    c
}
fn choice(c: &LoginController) -> Choice {
    Choice::capture(&c.state, 0, PET).unwrap()
}
fn dialog(id: u32) -> Vec<u8> {
    let mut b = vec![0x2a];
    b.extend(id.to_le_bytes());
    b.extend(b"anicom\0\0");
    b.extend(10u16.to_le_bytes());
    for s in [
        "$469", "80", "100", "12", "20", "7", "Pet", "$611", "44", "-7",
    ] {
        b.extend(s.bytes());
        b.push(0);
    }
    b
}
fn open(c: &mut LoginController, now: Instant) {
    c.state.companions.opened(choice(c), now);
    c.accept_game(&dialog(PET)).unwrap();
}
fn prompt(c: &mut LoginController, action: Action, now: Instant) {
    open(c, now);
    c.state.companions.sent(action, now);
    let b = if action == Action::Attack {
        let mut b = vec![0x57];
        b.extend(PET.to_le_bytes());
        b.push(7);
        b
    } else {
        vec![0x24, 0x45, 1]
    };
    c.accept_game(&b).unwrap();
}
fn authority(c: &LoginController) -> serde_json::Value {
    serde_json::json!({"world":c.state.world,"inventory":c.state.inventory})
}

#[test]
fn nine_actions_use_exact_cdss_and_never_predict_authority() {
    let mut c = active();
    let now = Instant::now();
    open(&mut c, now);
    let before = authority(&c);
    for a in Action::ALL {
        let b = c
            .state
            .companions
            .action(&c.state, 0, &choice(&c), a)
            .unwrap();
        let mut expected = vec![0x27, 0x38, 0, 0, 0x90];
        expected.extend(a.wire().bytes());
        expected.extend([0, 0]);
        assert_eq!(b, expected);
        c.state.companions.sent(a, now);
        assert_eq!(authority(&c), before);
        open(&mut c, now);
    }
}
#[test]
fn received_menu_routes_outside_rental_and_keeps_numeric_fields() {
    let mut c = active();
    open(&mut c, Instant::now());
    let m = c.state.companions.menu.as_ref().unwrap();
    assert_eq!(
        (
            m.hp,
            m.max_hp,
            m.mp,
            m.max_mp,
            m.level,
            m.experience_percent,
            m.lawful
        ),
        (80, 100, 12, 20, 7, 44, -7)
    );
    assert!(c.state.merchant.dialog.is_none());
    assert!(m.text().contains("80/100"));
}
#[test]
fn malformed_menu_is_atomic_and_wrong_owner_cannot_bind() {
    let mut c = active();
    let now = Instant::now();
    c.state.companions.opened(choice(&c), now);
    let good = dialog(PET);
    for n in 0..good.len() {
        assert!(c.receive_companion(&good[..n]).is_err() || n == 0);
        assert!(c.state.companions.menu.is_none());
    }
    let mut extra = good.clone();
    extra.push(0);
    assert!(c.accept_game(&extra).is_err());
    c.accept_game(&dialog(ENEMY)).unwrap();
    assert!(c.state.companions.menu.is_none());
    c.accept_game(&good).unwrap();
    assert!(c.state.companions.menu.is_some());
}
#[test]
fn target_reply_requires_server_prompt_echoes_mode_and_is_one_shot() {
    let mut c = active();
    let now = Instant::now();
    open(&mut c, now);
    assert!(c
        .state
        .companions
        .target_request(&c.state, 0, ENEMY, now)
        .is_err());
    prompt(&mut c, Action::Attack, now);
    assert_eq!(
        c.state
            .companions
            .target_request(&c.state, 0, ENEMY, now)
            .unwrap(),
        [0x4f, 0x38, 0, 0, 0x90, 7, 0x40, 0, 0, 0x90]
    );
    c.state.companions.replied();
    assert!(c
        .state
        .companions
        .target_request(&c.state, 0, ENEMY, now)
        .is_err());
}
#[test]
fn target_prompt_rejects_wrong_companion_malformed_and_expired() {
    let mut c = active();
    let now = Instant::now();
    open(&mut c, now);
    c.state.companions.sent(Action::Attack, now);
    c.accept_game(&[0x57, 0x40, 0, 0, 0x90, 0]).unwrap();
    assert!(!c.state.companions.can_target(now));
    assert!(c.accept_game(&[0x57, 0x38, 0, 0, 0x90]).is_err());
    prompt(&mut c, Action::Attack, now);
    assert!(c
        .state
        .companions
        .target_request(&c.state, 0, ENEMY, now + Duration::from_secs(16))
        .is_err());
    assert!(c
        .state
        .companions
        .target_request(&c.state, 0, PET, now)
        .is_err());
    c.state.world.actors.get_mut(&ENEMY).unwrap().dead = true;
    assert!(c
        .state
        .companions
        .target_request(&c.state, 0, ENEMY, now)
        .is_err());
}
#[test]
fn rename_prompt_reply_uses_chcs_and_native_name_limits() {
    let mut c = active();
    let now = Instant::now();
    prompt(&mut c, Action::Rename, now);
    assert_eq!(
        c.state
            .companions
            .rename_request(&c.state, 0, "Dog", now)
            .unwrap(),
        b"\x23\x45\x01\x01Dog\0"
    );
    for name in [
        "",
        "A",
        "Bad!",
        "Bad\0Name",
        "名字😀",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZAB",
    ] {
        assert!(
            c.state
                .companions
                .rename_request(&c.state, 0, name, now)
                .is_err(),
            "{name:?}"
        );
    }
    assert!(c
        .state
        .companions
        .rename_request(&c.state, 0, "Dog", now + Duration::from_secs(31))
        .is_err());
    c.state.companions.replied();
    assert!(c
        .state
        .companions
        .rename_request(&c.state, 0, "Dog", now)
        .is_err());
}
#[test]
fn unsolicited_and_duplicate_prompts_do_not_create_or_extend_capability() {
    let mut c = active();
    let now = Instant::now();
    c.accept_game(&[0x24, 0x45, 1]).unwrap();
    assert!(!c.state.companions.can_rename(now));
    prompt(&mut c, Action::Rename, now);
    assert!(c.accept_game(&[0x24, 0x45, 1, 0]).is_err());
    c.state.companions.replied();
    c.accept_game(&[0x24, 0x45, 1]).unwrap();
    assert!(!c.state.companions.can_rename(now));
    assert!(
        !c.receive_companion(&[0x24, 0x41, 1]).unwrap(),
        "unrelated ask belongs to another handler"
    );
}
#[test]
fn stale_selection_fails_on_scene_life_owner_visibility_and_controls() {
    let c = active();
    let selected = choice(&c);
    for kind in 0..9 {
        let mut c = active();
        match kind {
            0 => c.state.world.generation += 1,
            1 => c.state.world.actors.get_mut(&PET).unwrap().incarnation += 1,
            2 => c.state.world.actors.get_mut(&PET).unwrap().target_revision += 1,
            3 => c.state.world.actors.get_mut(&PET).unwrap().relations.master = b"Other".to_vec(),
            4 => c.state.world.actors.get_mut(&PET).unwrap().dead = true,
            5 => c.state.world.control_interruptions += 1,
            6 => c.state.observer = true,
            7 => c.state.world.hp = Some([0, 100]),
            _ => {
                c.state.world.actors.remove(&PET);
            }
        }
        assert!(selected.validate(&c.state, 0).is_err(), "case {kind}");
    }
    assert!(selected.validate(&c.state, 1).is_err());
    assert!(Choice::capture(&c.state, 0, ENEMY).is_err());
}
#[test]
fn timeout_only_retires_ui_and_never_expires_or_dismisses_actor() {
    let mut c = active();
    let now = Instant::now();
    open(&mut c, now);
    let before = authority(&c);
    c.state.companions.sent(Action::Dismiss, now);
    let mut session = std::mem::take(&mut c.state.companions);
    session.tick(&c.state, 0, now + Duration::from_secs(7200));
    c.state.companions = session;
    assert_eq!(authority(&c), before);
    let mut b = vec![0x15];
    b.extend(PET.to_le_bytes());
    c.accept_game(&b).unwrap();
    assert!(!c.state.world.actors.contains_key(&PET));
    assert!(c.state.companions.menu.is_none());
}
#[test]
fn menu_wait_timeout_allows_retry_without_claiming_success() {
    let mut c = active();
    let now = Instant::now();
    c.state.companions.opened(choice(&c), now);
    let mut session = std::mem::take(&mut c.state.companions);
    session.tick(&c.state, 0, now + Duration::from_secs(9));
    assert!(session.can_request());
    assert!(session.menu.is_none());
    assert!(session.feedback.contains("未收到"));
}
#[test]
fn give_uses_current_coordinates_and_checks_inventory_quantity_and_distance() {
    let mut c = active();
    let pet = choice(&c);
    let item = native_item_actions::Choice::capture(&c.state, 77).unwrap();
    let before = authority(&c);
    let b = pets::give(&c.state, 0, &pet, &item, 2).unwrap();
    let a = &c.state.world.actors[&PET];
    let mut expected = vec![0x11];
    expected.extend(PET.to_le_bytes());
    expected.extend(a.x.to_le_bytes());
    expected.extend(a.y.to_le_bytes());
    expected.extend(77u32.to_le_bytes());
    expected.extend(2u32.to_le_bytes());
    assert_eq!(b, expected);
    assert_eq!(authority(&c), before);
    for q in [0, 5, u32::MAX] {
        assert!(pets::give(&c.state, 0, &pet, &item, q).is_err());
    }
    c.state.world.actors.get_mut(&PET).unwrap().x += 5;
    assert!(pets::give(&c.state, 0, &pet, &item, 1).is_err());
    c.state.world.actors.get_mut(&PET).unwrap().x -= 5;
    c.state.inventory.items.get_mut(&77).unwrap().count = 3;
    assert!(pets::give(&c.state, 0, &pet, &item, 1).is_err());
}
#[test]
fn collar_use_preserves_inventory_and_capacity_is_not_invented() {
    let c = active();
    let item = native_item_actions::Choice::capture(&c.state, 77).unwrap();
    let before = authority(&c);
    assert_eq!(
        native_item_actions::request(&c.state, &item, native_item_actions::Action::Use).unwrap(),
        [0x1c, 77, 0, 0, 0, 0]
    );
    assert_eq!(authority(&c), before);
    let text = pets::summary(&c.state);
    assert!(text.contains("CHA："));
    assert!(text.contains("尚未收到剩餘容量"));
    assert!(text.contains("尚未收到到期時間"));
}
#[test]
fn failed_transport_does_not_open_menu_or_consume_prompt() {
    let mut c = active();
    let selected = choice(&c);
    assert!(c.companion_talk(selected).is_err());
    assert!(c.state.companions.menu.is_none());
    assert!(c.state.companions.can_request());
    let now = Instant::now();
    prompt(&mut c, Action::Rename, now);
    let before = authority(&c);
    assert!(c.companion_rename("Dog").is_err());
    assert!(c.state.companions.can_rename(now));
    assert_eq!(authority(&c), before);
}
#[test]
fn interruption_and_cancel_clear_pending_companion_state() {
    let mut c = active();
    prompt(&mut c, Action::Attack, Instant::now());
    c.interrupt_world_actions();
    assert!(c.state.companions.menu.is_none());
    prompt(&mut c, Action::Rename, Instant::now());
    c.cancel();
    assert!(!c.state.companions.can_rename(Instant::now()));
}

#[test]
fn world_ready_supports_companions_and_collar_but_rechecks_death_and_scene() {
    let mut c = active();
    c.state.phase = LoginPhase::WorldReady;
    open(&mut c, Instant::now());
    assert!(c.state.companions.menu.is_some());
    let item = native_item_actions::Choice::capture(&c.state, 77).unwrap();
    assert_eq!(
        native_item_actions::request(&c.state, &item, native_item_actions::Action::Use).unwrap(),
        [0x1c, 77, 0, 0, 0, 0]
    );
    let pending = native_item_actions::Pending::new(&c.state, item.clone(), Instant::now());
    c.state.world.generation += 1;
    assert_eq!(
        pending.observe(&c.state, Instant::now()),
        native_item_actions::Feedback::Interrupted
    );
    c.state.world.hp = Some([0, 100]);
    assert!(
        native_item_actions::request(&c.state, &item, native_item_actions::Action::Use).is_err()
    );
    assert!(Choice::capture(&c.state, 0, PET).is_err());
}

#[test]
fn another_ask_retires_rename_and_capacity_messages_preserve_authority() {
    let mut c = active();
    let now = Instant::now();
    prompt(&mut c, Action::Rename, now);
    assert!(!c.receive_companion(&[0x24, 0x41, 1]).unwrap());
    assert!(!c.state.companions.can_rename(now));
    let inventory = c.state.inventory.clone();
    let actors = c.state.world.actors.clone();
    // S_MESSAGE_CODE = c h s...; the zero byte is argument count.
    for code in [319u16, 353, 942] {
        let mut b = vec![0x10];
        b.extend(code.to_le_bytes());
        b.push(0);
        c.accept_game(&b).unwrap();
        assert_eq!(c.state.world.last_message.as_ref().unwrap().0, code);
        assert_eq!(c.state.inventory, inventory);
        assert_eq!(c.state.world.actors, actors);
    }
}
