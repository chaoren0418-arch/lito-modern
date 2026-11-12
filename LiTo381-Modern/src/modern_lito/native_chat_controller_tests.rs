//! Controller routing of native chat (T060). No transport is started: a send
//! that reaches the connection fails, which the tests rely on.
use super::*;
use crate::modern_lito::native_chat::{Channel, Origin, Preference};

fn say(opcode: u8, kind: u8, id: u32, text: &[u8]) -> Vec<u8> {
    let mut b = vec![opcode, kind];
    b.extend(id.to_le_bytes());
    b.extend(text);
    b.push(0);
    b.extend(33000u16.to_le_bytes());
    b.extend(32800u16.to_le_bytes());
    b
}

fn lobby() -> LoginController {
    let mut controller = LoginController::default();
    controller.game = true;
    controller.state.selected_character = Some("甲".into());
    controller.state.phase = LoginPhase::LobbyReady;
    controller
}

#[test]
fn chat_receipts_are_accepted_before_any_world_exists() {
    let mut controller = lobby();
    assert!(controller.state.world.map_id.is_none());
    controller.accept_game(&say(0x13, 0, 7, b"a: hi")).unwrap();
    controller.accept_game(&say(0x2d, 2, 9, b"<npc> hey")).unwrap();
    controller.accept_game(&[0x14, b'b', 0, b'y', b'o', 0]).unwrap();
    let chat = &controller.snapshot().chat;
    let origins: Vec<_> = chat.entries.iter().map(|e| e.origin).collect();
    assert_eq!(origins, [Origin::Player, Origin::Npc, Origin::WhisperReceived]);
    assert_eq!(chat.entries[1].position, Some([33000, 32800]));
    assert_eq!(controller.state.phase, LoginPhase::LobbyReady);
}

#[test]
fn chat_receipts_in_battle_do_not_touch_world_state() {
    let mut controller = lobby();
    assert!(controller.state.world.accept(&[0x28, 5, 0, 0]).unwrap());
    assert!(controller.state.world.accept(&[0xd2]).unwrap());
    controller.state.phase = LoginPhase::BattlePending;
    let revision = controller.state.world.revision;
    controller.accept_game(&say(0x13, 3, 7, b"a: hi")).unwrap();
    controller.accept_game(&say(0x2d, 0, 0, b"\\fThint")).unwrap();
    assert_eq!(controller.state.world.revision, revision);
    assert!(controller.state.world.unhandled_opcodes.is_empty());
    assert_eq!(controller.snapshot().chat.entries.len(), 2);
}

#[test]
fn malformed_chat_receipt_is_an_error_and_changes_nothing() {
    let mut controller = lobby();
    controller.accept_game(&say(0x13, 0, 7, b"a: hi")).unwrap();
    let before = serde_json::to_value(controller.snapshot()).unwrap();
    let mut trailing = say(0x2d, 0, 7, b"npc: hi");
    trailing.push(0);
    assert!(controller.accept_game(&trailing).is_err());
    assert!(controller.accept_game(&say(0x2d, 4, 7, b"npc: hi")).is_err());
    assert!(controller.accept_game(&[0x14, b'b', 0]).is_err());
    assert_eq!(serde_json::to_value(controller.snapshot()).unwrap(), before);
}

#[test]
fn history_does_not_follow_another_character() {
    let mut controller = lobby();
    controller.accept_game(&say(0x13, 0, 7, b"a: one")).unwrap();
    controller.accept_game(&[0x14, b'b', 0, b's', 0]).unwrap();
    assert_eq!(controller.snapshot().chat.entries.len(), 2);
    controller.state.selected_character = Some("乙".into());
    controller.accept_game(&say(0x13, 0, 7, b"a: two")).unwrap();
    let chat = &controller.snapshot().chat;
    assert_eq!(chat.entries.len(), 1);
    assert_eq!(chat.entries[0].text, "a: two");
    // Returning to the character list clears it without a new packet.
    controller.state.selected_character = None;
    controller.bind_chat();
    assert!(controller.snapshot().chat.entries.is_empty());
}

#[test]
fn sending_is_refused_outside_chat_phases_before_any_validation() {
    let mut controller = lobby();
    for phase in [LoginPhase::CharactersReady, LoginPhase::CharacterSelected,
        LoginPhase::RoomJoining, LoginPhase::BattleResult, LoginPhase::Failed] {
        controller.state.phase = phase;
        for error in [
            controller.chat_say(Channel::Normal, "hi").unwrap_err(),
            controller.chat_tell("b", "hi").unwrap_err(),
            controller.chat_preference(Preference::World, false).unwrap_err(),
        ] {
            assert!(error.to_string().contains("目前不能聊天"), "{phase:?}: {error}");
        }
    }
    controller.state.phase = LoginPhase::LobbyReady;
    controller.state.selected_character = None;
    assert!(controller.chat_say(Channel::Normal, "hi").unwrap_err().to_string().contains("目前不能聊天"));
    controller.state.selected_character = Some("甲".into());
    controller.game = false;
    assert!(controller.chat_say(Channel::Normal, "hi").unwrap_err().to_string().contains("目前不能聊天"));
    assert!(controller.snapshot().chat.entries.is_empty());
}

#[test]
fn invalid_text_is_refused_before_the_connection_and_a_failed_whisper_is_not_recorded() {
    let mut controller = lobby();
    for phase in [LoginPhase::LobbyReady, LoginPhase::RoomReady, LoginPhase::BattlePending, LoginPhase::WorldReady] {
        controller.state.phase = phase;
        let too_long = controller.chat_say(Channel::Normal, &"a".repeat(256)).unwrap_err().to_string();
        assert!(too_long.contains("255"), "{too_long}");
        let unmappable = controller.chat_tell("b", "😀").unwrap_err().to_string();
        assert!(unmappable.contains("MS950"), "{unmappable}");
        assert!(controller.chat_tell("", "hi").unwrap_err().to_string().contains("密語對象"));
        // Valid text reaches the absent connection and fails there.
        assert!(controller.chat_say(Channel::Shout, "hi").is_err());
        assert!(controller.chat_tell("b", "hi").is_err());
        assert!(controller.chat_preference(Preference::Whisper, true).is_err());
    }
    assert!(controller.snapshot().chat.entries.is_empty());
}
