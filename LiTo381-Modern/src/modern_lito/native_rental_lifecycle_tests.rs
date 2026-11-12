//! G06.3c synthetic encrypted loopback coverage, not formal-server acceptance.
use super::teleport_wire_tests::{bytes, pump, seeded_rental, FaultPeer};
use super::*;
use crate::modern_lito::{native_merchant as merchant, native_services};

const NPC: u32 = 1812987996;
const PURCHASE: u32 = 1828716573;
const KEY: u32 = 1879048249;

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn character_row() -> Vec<u8> {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native-game-current.json"
    ))
    .unwrap();
    let mut row = hex(fixture["packets"]["row"].as_str().unwrap());
    let end = row[1..].iter().position(|b| *b == 0).unwrap() + 1;
    row.splice(1..end, b"TestKnight".iter().copied());
    // S_CB 數值區第一個位元組是職業；租裝配裝表只提供職業 0..4（T055-2），騎士=1。
    let name_end = row[1..].iter().position(|b| *b == 0).unwrap() + 1;
    let clan_end = row[name_end + 1..].iter().position(|b| *b == 0).unwrap() + name_end + 1;
    row[clan_end + 1] = 1;
    row
}

fn member() -> Vec<u8> {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native-room-current.json"
    ))
    .unwrap();
    let mut row = hex(fixture["packets"]["member_0"].as_str().unwrap());
    row[1..5].copy_from_slice(&60001u32.to_le_bytes());
    let end = row[5..].iter().position(|b| *b == 0).unwrap() + 5;
    row.splice(5..end, b"TestKnight".iter().copied());
    row
}

fn item(id: u32, name: &str) -> Vec<u8> {
    let mut body = vec![0x16];
    body.extend(id.to_le_bytes());
    body.extend([0, 0, 1, 0, 0]); // use type, charge, icon, blessing
    body.extend(1u32.to_le_bytes());
    body.push(1);
    body.extend(name.as_bytes());
    body.extend([0, 0]);
    body
}

fn remove(id: u32) -> Vec<u8> {
    let mut body = vec![0x17];
    body.extend(id.to_le_bytes());
    body
}

fn departure() -> Vec<u8> {
    let mut body = vec![0xd0];
    body.extend(60001u32.to_le_bytes());
    body.extend(b"TestKnight\0");
    body
}

fn rental(peer: &FaultPeer) -> LoginController {
    let mut c = seeded_rental(peer);
    c.readiness_acked = true;
    c.state
        .environment
        .begin(c.transport.snapshot().epoch)
        .unwrap();
    c.state.redirect = Some(peer.peer);
    c.state.services = Some(native_services::Directory {
        rooms: vec![native_services::DirectRoom {
            name: "Rental".into(),
            endpoint: peer.peer,
            id: 60001,
            users: 1,
        }],
        servers: vec![],
    });
    c
}

fn control(peer: &FaultPeer, c: &mut LoginController, body: Vec<u8>, phase: LoginPhase) {
    peer.emit(vec![body]);
    pump(c, |c| c.state.phase == phase);
    assert!(c.state.error.is_none());
}

fn cleanup(c: &mut LoginController, peer: &mut FaultPeer) {
    c.cancel();
    pump(c, |c| c.is_quiescent());
    peer.close();
}

#[test]
fn normal_return_and_reentry_preserve_base_items_but_retire_services_and_interactions() {
    // Town and two hotel maps, buy/sell lists and numeric lease input.
    for (map, interaction) in [(500, 0), (501, 1), (503, 2)] {
        let mut peer = FaultPeer::new();
        let mut c = rental(&peer);
        c.state.world.map_id = Some(map);
        let generation = c.state.world.generation;
        let base = c.state.inventory.items.clone();
        let old_owner = c.state.inventory.owner.clone().unwrap();
        c.accept_game(&item(PURCHASE, "Rented weapon")).unwrap();
        c.accept_game(&item(KEY, "Hotel key")).unwrap();
        let now = Instant::now();
        match interaction {
            0 => c
                .state
                .merchant
                .accept(
                    merchant::Packet::List(merchant::Listing::Buy {
                        npc: NPC,
                        rows: vec![merchant::BuyRow {
                            id: 7,
                            icon: 1,
                            price: 10,
                            name: "Rental".into(),
                            detail: vec![],
                        }],
                    }),
                    now,
                )
                .unwrap(),
            1 => c
                .state
                .merchant
                .accept(
                    merchant::Packet::List(merchant::Listing::Sell {
                        npc: NPC,
                        rows: vec![merchant::SellRow {
                            id: PURCHASE,
                            price: 5,
                        }],
                    }),
                    now,
                )
                .unwrap(),
            _ => c
                .state
                .merchant
                .accept_input(
                    merchant::NumericInput {
                        npc: NPC,
                        auxiliary: 0,
                        current: 1,
                        minimum: 1,
                        maximum: 8,
                        mode: 0,
                        prompt: "Keys".into(),
                        html: "inn".into(),
                        callback: "room".into(),
                        arguments: vec![],
                    },
                    now,
                )
                .unwrap(),
        }
        let stale_revision = c.state.merchant.revision;
        c.return_from_rental().unwrap();
        assert_eq!(c.state.phase, LoginPhase::RentalRestartPending);
        assert!(c.state.rental_return_outcome.is_none());
        assert!(c.state.world.ready()); // no invented departure before server ack
        assert!(c.state.merchant.owner.is_none());
        assert!(c.return_from_rental().is_err());
        control(&peer, &mut c, vec![0x65], LoginPhase::ReturningCharacters);
        peer.emit(vec![
            vec![0xf0, 0],
            remove(PURCHASE),
            departure(),
            remove(KEY),
        ]);
        pump(&mut c, |c| c.state.joined_room.is_none());
        assert_eq!(c.state.phase, LoginPhase::ReturningCharacters);
        assert!(c.state.inventory.owner.is_none() && c.state.inventory.items.is_empty());
        assert!(c.state.world.map_id.is_none());
        assert_eq!(c.state.world.generation, generation);
        assert!(c.state.rental_return_outcome.is_none());
        control(&peer, &mut c, vec![0xf5], LoginPhase::LoadingCharacters);
        peer.emit(vec![vec![0x03, 1], character_row()]);
        pump(&mut c, |c| c.state.phase == LoginPhase::CharactersReady);
        assert_eq!(
            c.state.rental_return_outcome,
            Some(RentalReturnOutcome::Normal)
        );
        assert_eq!(c.transport.snapshot().epoch, old_owner.connection);
        c.select_character(0).unwrap();
        control(&peer, &mut c, vec![0xc8, 0], LoginPhase::CharacterSelected);
        // No S7C replay: same-character base inventory must survive the selector.
        assert_eq!(c.state.inventory.items, base);
        assert_ne!(
            c.state.inventory.owner.as_ref().unwrap().selection,
            old_owner.selection
        );
        c.select_rental_set(1).unwrap();
        let mut selected = vec![
            super::super::native_opcodes::S_LITO_ENTER_CHANNEL_RESULT,
            0,
            1,
        ];
        selected.extend([0; 10]);
        control(&peer, &mut c, selected, LoginPhase::RentalReady);
        c.join_rental_room(0).unwrap();
        control(
            &peer,
            &mut c,
            vec![0xf0, 0],
            LoginPhase::RentalAwaitPermission,
        );
        control(&peer, &mut c, vec![0xf5], LoginPhase::RoomJoining);
        peer.emit(vec![
            bytes("joined"),
            member(),
            bytes("world"),
            bytes("start"),
            bytes("status"),
            bytes("own_actor"),
            bytes("npc"),
            bytes("hp"),
        ]);
        pump(&mut c, |c| {
            c.state.world.ready()
                && c.state.phase == LoginPhase::BattlePending
                && c.state.world.actors.contains_key(&NPC)
        });
        assert!(c.state.world.generation > generation);
        assert_eq!(c.state.world.map_id, Some(500));
        assert_eq!(c.state.inventory.items, base);
        let queued = c.transport.queued_tx_counter();
        assert!(c.rental_merchant_order(stale_revision, 0, 1).is_err());
        assert!(c.rental_npc_quantity(stale_revision, 1).is_err());
        assert!(c.rental_npc_action(NPC, "room").is_err());
        assert_eq!(c.transport.queued_tx_counter(), queued);
        // Fresh interaction with the same NPC is possible in the new generation.
        let own = c.state.world.actors.get_mut(&1).unwrap();
        own.x = 32777;
        own.y = 32819;
        c.rental_npc_talk(NPC).unwrap();
        peer.emit(vec![bytes("dialog")]);
        pump(&mut c, |c| !c.state.merchant.waiting);
        assert!(c.state.merchant.dialog.is_some());
        assert_ne!(c.state.merchant.revision, stale_revision);
        c.state
            .merchant
            .accept(
                merchant::Packet::List(merchant::Listing::Buy {
                    npc: NPC,
                    rows: vec![merchant::BuyRow {
                        id: 7,
                        icon: 1,
                        price: 10,
                        name: "Fresh rental".into(),
                        detail: vec![],
                    }],
                }),
                Instant::now(),
            )
            .unwrap();
        assert!(c.rental_merchant_order(stale_revision, 0, 1).is_err());
        c.rental_merchant_order(c.state.merchant.revision, 0, 1)
            .unwrap();
        pump(&mut c, |_| {
            peer.commands().iter().any(|b| b.first() == Some(&0x28))
        });
        let commands = peer.commands();
        // Selecting starts the normal GAME Alive timer once; it is not a return command.
        assert!(commands.iter().filter(|b| b[0] == 0x20).count() <= 1);
        let opcodes: Vec<_> = commands
            .iter()
            .map(|b| b[0])
            .filter(|op| *op != 0x20)
            .collect();
        assert_eq!(
            opcodes,
            [0x10, 0xd9, 0xd0, 0xc8, 0xc9, 0xca, 0xd9, 0xcf, 0x29, 0x28]
        );
        assert_eq!(commands[3], source_request("測試乙").unwrap().as_slice());
        assert_eq!(
            commands.last().unwrap(),
            &merchant::transaction(NPC, false, &[(7, 1)]).unwrap()
        );
        cleanup(&mut c, &mut peer);
    }
}

#[test]
fn queued_scene_and_npc_packets_cannot_revive_return_or_extend_its_deadline() {
    let mut peer = FaultPeer::new();
    let mut c = rental(&peer);
    c.return_from_rental().unwrap();
    for stage in 0..4 {
        let state = serde_json::to_value(&c.state).unwrap();
        let deadline = c.deadline;
        let queued = c.transport.queued_tx_counter();
        for body in [
            bytes("world"),
            bytes("start"),
            bytes("status"),
            bytes("own_actor"),
            bytes("dialog"),
            vec![0x2b, 0],
            vec![0x5b, 0],
            vec![0x33, 0],
            vec![0xf4, 0],
        ] {
            c.accept_game(&body).unwrap();
            assert_eq!(serde_json::to_value(&c.state).unwrap(), state);
            assert_eq!(c.deadline, deadline);
            assert_eq!(c.transport.queued_tx_counter(), queued);
        }
        match stage {
            0 => c.accept_game(&[0x65]).unwrap(),
            1 => {
                c.accept_game(&[0xf0, 0]).unwrap();
                c.accept_game(&departure()).unwrap();
            }
            2 => {
                c.accept_game(&[0xf5]).unwrap();
                c.accept_game(&[0x03, 1]).unwrap();
                c.accept_game(&character_row()).unwrap();
            }
            _ => {}
        }
    }
    assert_eq!(c.state.phase, LoginPhase::CharactersReady);
    assert_eq!(
        c.state.rental_return_outcome,
        Some(RentalReturnOutcome::Normal)
    );
    let before = serde_json::to_value(&c.state).unwrap();
    for body in [vec![0x65], vec![0xf0, 0], vec![0xf5]] {
        assert!(c.accept_game(&body).is_err());
        assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    }
    cleanup(&mut c, &mut peer);
}

#[test]
fn rental_mode_return_still_requires_own_departure_without_a_loadout_marker() {
    let mut peer = FaultPeer::new();
    let mut c = rental(&peer);
    c.state.rental_set = None;
    c.return_from_rental().unwrap();
    c.accept_game(&[0x65]).unwrap();
    c.accept_game(&[0xf0, 0]).unwrap();
    let before = serde_json::to_value(&c.state).unwrap();
    assert!(c
        .accept_game(&[0xf5])
        .unwrap_err()
        .to_string()
        .contains("rental departure"));
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    c.accept_game(&departure()).unwrap();
    c.accept_game(&[0xf5]).unwrap();
    c.accept_game(&[0x03, 1]).unwrap();
    c.accept_game(&character_row()).unwrap();
    assert_eq!(
        c.state.rental_return_outcome,
        Some(RentalReturnOutcome::Normal)
    );
    cleanup(&mut c, &mut peer);
}

#[test]
fn return_without_departure_times_out_instead_of_claiming_normal_completion() {
    let mut peer = FaultPeer::new();
    let mut c = rental(&peer);
    c.return_from_rental().unwrap();
    c.accept_game(&[0x65]).unwrap();
    c.accept_game(&[0xf0, 0]).unwrap();
    c.deadline = Some(Instant::now() - Duration::from_millis(1));
    c.poll();
    assert_eq!(c.state.phase, LoginPhase::Failed);
    assert!(c.state.rental_return_outcome.is_none());
    assert!(c.state.inventory.owner.is_none() && c.state.merchant.owner.is_none());
    assert!(c.credentials.is_none());
    cleanup(&mut c, &mut peer);
}

#[test]
fn reconnect_directory_is_reported_as_recovery_and_never_as_normal_return() {
    let mut c = LoginController::default();
    c.rental_return_reconnect = Some(RentalReturnReconnect {
        login_peer: "127.0.0.1:1".parse().unwrap(),
        source: "測試乙".into(),
        closing: false,
    });
    c.state.selected_source = Some("測試乙".into());
    c.state.phase = LoginPhase::CharactersReady;
    c.poll_rental_return_reconnect().unwrap();
    assert_eq!(
        c.state.rental_return_outcome,
        Some(RentalReturnOutcome::Reconnected)
    );
    assert!(c.rental_return_reconnect.is_none());
}
