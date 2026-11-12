//! Pure state fixtures. No server TTL, authentication or inventory authority is simulated.
use super::*;

fn staged() -> LoginController {
    let mut c = LoginController::default();
    c.login_peer = Some("127.0.0.1:31001".parse().unwrap());
    authenticate(&mut c);
    c.state.selected_source = Some("測試乙".into());
    c.state.selected_character = Some("RecoveryKnight".into());
    c.state.phase = LoginPhase::HandoffClosing;
    c.state.game_connected = true;
    c.state.world.generation = 17;
    c.state.world.map_id = Some(500);
    c.state.world.learned_skills.insert(43);
    c.state.world.mp = Some([70, 110]);
    let owner = super::super::native_merchant::Owner {
        connection: 1, generation: 17, incarnation: 2, control_interruptions: 0,
        inventory: super::super::native_inventory::Owner {
            connection: 1, selection: 1, character: "RecoveryKnight".into(),
        },
    };
    let mut pending = super::super::native_teleport::Pending::new(
        owner, "RecoveryKnight".into(), "TeleportMe", "LT_TELEPORT_B").unwrap();
    pending.destination = Some("127.0.0.1:31002".parse().unwrap());
    c.teleport = Some(pending);
    c
}

fn authenticate(c: &mut LoginController) {
    c.credentials = Some((Zeroizing::new("RecoveryFixture".into()), Zeroizing::new("SyntheticOnly".into())));
}

fn retired(c: &LoginController) {
    assert!(c.credentials.is_none() && c.teleport.is_none() && c.recovery_character.is_none());
    assert!(c.state.world.map_id.is_none() && c.state.world.mp.is_none());
    assert!(c.state.world.actors.is_empty() && c.state.world.learned_skills.is_empty());
    assert!(c.state.inventory.owner.is_none() && c.state.selected_character.is_none());
    assert!(c.state.joined_room.is_none() && c.state.services.is_none() && !c.state.game_connected);
    assert!(c.state.environment.epoch.is_none() && c.rental_return_reconnect.is_none());
    assert!(c.is_quiescent());
    assert_eq!(c.state.world.generation, 17);
    assert_eq!(c.transport.snapshot().epoch, 0);
}

#[test]
fn second_cancel_preserves_unused_login_correlation_without_authority() {
    let mut c = staged();
    for _ in 0..4 {
        c.cancel(); c.poll(); retired(&c);
        let r = c.teleport_recovery.as_ref().expect("unused recovery erased by cancel");
        assert_eq!(r.login_peer, "127.0.0.1:31001".parse().unwrap());
        assert_eq!((&*r.account, &*r.source, &*r.character), ("RecoveryFixture", "測試乙", "RecoveryKnight"));
    }
    assert_eq!(c.state.phase, LoginPhase::Closed);
}

#[test]
fn cancel_after_failure_preserves_unused_login_correlation() {
    let mut c = staged();
    c.fail("synthetic destination failure".into());
    c.cancel(); c.cancel(); c.poll(); retired(&c);
    assert!(c.teleport_recovery.is_some());
    assert_eq!(c.state.error.as_deref(), Some("synthetic destination failure"));
    assert_eq!(c.state.phase, LoginPhase::Failed);
}

#[test]
fn explicit_source_choice_consumes_matching_recovery_once() {
    let mut c = staged(); c.cancel(); authenticate(&mut c);
    c.recovery_character = c.take_teleport_recovery("測試乙");
    assert_eq!(c.recovery_character.as_deref(), Some("RecoveryKnight"));
    assert!(c.take_teleport_recovery("測試乙").is_none());
    c.cancel(); c.poll(); retired(&c);
    assert!(c.teleport_recovery.is_none());
}

#[test]
fn peer_account_source_and_missing_credentials_consume_mismatched_recovery() {
    for case in 0..6 {
        let mut c = staged(); c.cancel(); authenticate(&mut c);
        let mut source = "測試乙";
        match case {
            0 => c.login_peer = Some("127.0.0.2:31001".parse().unwrap()),
            1 => c.login_peer = Some("127.0.0.1:31003".parse().unwrap()),
            2 => c.credentials.as_mut().unwrap().0 = Zeroizing::new("OtherAccount".into()),
            3 => source = "測試甲",
            4 => c.credentials = None,
            _ => c.login_peer = None,
        }
        assert!(c.take_teleport_recovery(source).is_none(), "binding {case}");
        c.login_peer = Some("127.0.0.1:31001".parse().unwrap()); authenticate(&mut c);
        assert!(c.take_teleport_recovery("測試乙").is_none(), "mismatch revived");
    }
}

#[test]
fn only_a_new_authorized_destination_can_replace_recovery() {
    let mut c = staged(); c.cancel();
    let saved = c.teleport_recovery.take();
    c = staged(); c.teleport_recovery = saved;
    c.teleport.as_mut().unwrap().character = "NextKnight".into();
    c.cancel();
    assert_eq!(c.teleport_recovery.as_ref().unwrap().character, "NextKnight");
    let mut ordinary = staged(); ordinary.teleport.as_mut().unwrap().destination = None;
    ordinary.cancel(); ordinary.cancel(); ordinary.poll(); retired(&ordinary);
    assert!(ordinary.teleport_recovery.is_none());
}

fn unused_wait() -> LoginController {
    let mut c = staged(); c.cancel(); c.teleport_recovery = None;
    c.game = true; c.readiness_acked = true; c.state.game_connected = true;
    c.state.phase = LoginPhase::CharacterPending;
    c.state.selected_source = Some("測試乙".into());
    c.state.sources = vec![Source { name: "測試乙".into(), endpoint: "127.0.0.1:31002".parse().unwrap() }];
    c.state.selected_character = Some("RecoveryKnight".into());
    c.recovery_character = Some("RecoveryKnight".into());
    // A valid synthetic inventory owner; transport intentionally remains
    // disconnected so the enqueue-failure case exercises that boundary.
    c.state.inventory.begin(1, "RecoveryKnight").unwrap();
    c.deadline = Some(Instant::now() - Duration::from_millis(1));
    c
}

#[test]
fn failed_c8_enqueue_retires_unconfirmed_recovery() {
    let mut c = unused_wait(); c.poll();
    assert_eq!(c.state.phase, LoginPhase::Failed);
    assert!(c.state.error.as_deref().unwrap().contains("recovery source confirmation"));
    retired(&c);
    assert!(c.state.characters.is_empty() && c.teleport_recovery.is_none());
}

#[test]
fn partial_or_unbound_continuation_cannot_refresh_source() {
    for case in 0..12 {
        let mut c = unused_wait();
        match case {
            0 => c.state.character_result = Some(0),
            1 => c.state.inventory.batches = 1,
            2 => c.state.selected_source = None,
            3 => c.state.sources.clear(),
            4 => c.state.phase = LoginPhase::CharactersReady,
            5 => c.state.game_connected = false,
            6 => c.game = false,
            7 => c.readiness_acked = false,
            8 => c.state.selected_character = Some("AnotherKnight".into()),
            9 => c.recovery_character = None,
            10 => c.state.notice = Some(super::super::native_game::Notice { message_id: Some(438), text: None }),
            _ => c.teleport = staged().teleport,
        }
        let before = serde_json::to_value(&c.state).unwrap();
        let deadline = c.deadline;
        assert!(c.refresh_recovery_source().is_err(), "binding {case}");
        assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
        assert_eq!(c.deadline, deadline);
        assert!(c.is_quiescent());
    }
}
