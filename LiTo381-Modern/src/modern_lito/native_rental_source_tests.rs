//! T055-2 rental-source：C_CA 配裝、S_FB 進村交接、租裝村入口種類。
//! 合成加密 loopback 與直接 dispatcher 呼叫，不代表正式伺服器驗收。
use super::teleport_wire_tests::{bytes, pump, seeded_rental, FaultPeer};
use super::*;
use crate::modern_lito::native_rental_source::TownTransition;
use crate::modern_lito::{native_game, native_services};

const ROOM: u32 = 60001;

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

/// 正式 fixture 的 S_CB 列，改名 TestKnight 並換成指定職業。
fn character_row(class: u8) -> Vec<u8> {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native-game-current.json"
    ))
    .unwrap();
    let mut row = hex(fixture["packets"]["row"].as_str().unwrap());
    let end = row[1..].iter().position(|b| *b == 0).unwrap() + 1;
    row.splice(1..end, b"TestKnight".iter().copied());
    let name_end = row[1..].iter().position(|b| *b == 0).unwrap() + 1;
    let clan_end = row[name_end + 1..].iter().position(|b| *b == 0).unwrap() + name_end + 1;
    row[clan_end + 1] = class;
    row
}

fn services(peer: &FaultPeer, ids: &[u32]) -> Vec<u8> {
    let SocketAddr::V4(addr) = peer.peer else { panic!("IPv4 peer") };
    let mut body = vec![0xfb];
    body.extend((ids.len() as u32).to_le_bytes());
    body.extend(1u32.to_le_bytes());
    for id in ids {
        body.extend(b"Rental Village\0");
        body.extend(addr.ip().octets());
        body.extend(addr.port().to_le_bytes());
        body.extend(id.to_le_bytes());
        body.extend(0u32.to_le_bytes());
    }
    body.extend(b"Rental Battle\0");
    body.extend(addr.ip().octets());
    body.extend(addr.port().to_le_bytes());
    body.extend(0u32.to_le_bytes());
    body.extend(100u32.to_le_bytes());
    body
}

/// GAME 已選角、尚未選配裝；S_FB 目錄沿用 LOGIN 端廣告（直連房與 GAME 同端點）。
fn selected(peer: &FaultPeer, class: u8) -> LoginController {
    let mut c = LoginController::default();
    c.start(
        peer.peer,
        Limits {
            io_slice: Duration::from_millis(5),
            write_chunk: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::ReadyForLogin);
    c.game = true;
    c.state.game_connected = true;
    c.state.redirect = Some(peer.peer);
    c.state.services = Some(native_services::parse(&services(peer, &[ROOM])).unwrap());
    let mut directory = native_game::Directory::default();
    directory.start(1).unwrap();
    let native_game::Packet::Row(row) = native_game::parse(&character_row(class)).unwrap() else {
        panic!("character row fixture")
    };
    directory.push(row).unwrap();
    c.state.characters = directory.view().unwrap();
    c.state.selected_character = Some("TestKnight".into());
    c.state.phase = LoginPhase::CharacterSelected;
    c.readiness_acked = true;
    c.state.environment.begin(c.transport.snapshot().epoch).unwrap();
    c
}

fn set_reply(index: u8) -> Vec<u8> {
    let mut body = vec![super::super::native_opcodes::S_LITO_ENTER_CHANNEL_RESULT, 0, index];
    body.extend([0; 10]);
    body
}

fn control(peer: &FaultPeer, c: &mut LoginController, body: Vec<u8>, phase: LoginPhase) {
    peer.emit(vec![body]);
    pump(c, |c| c.state.phase == phase);
    assert!(c.state.error.is_none(), "{:?}", c.state.error);
}

fn cleanup(c: &mut LoginController, peer: &mut FaultPeer) {
    c.cancel();
    pump(c, |c| c.is_quiescent());
    peer.close();
}

fn ops(peer: &FaultPeer) -> Vec<u8> {
    peer.commands().iter().map(|b| b[0]).collect()
}

fn portal(map: u16, x: u16, y: u16) -> Vec<u8> {
    let mut body = vec![0x5a];
    body.extend(map.to_le_bytes());
    body.extend(x.to_le_bytes());
    body.extend(y.to_le_bytes());
    body
}

#[test]
fn item_set_selection_follows_selected_class_and_never_sends_out_of_range() {
    // 職業 1（騎士）三組、3（法師）兩組、6（幻術師）沒有租裝配裝。
    for (class, choices) in [(1u8, 3u8), (3, 2), (6, 0)] {
        let mut peer = FaultPeer::new();
        let mut c = selected(&peer, class);
        assert_eq!(c.selected_character_class(), Some(class));
        assert_eq!(c.rental_set_choices(), choices);
        for index in choices..=4 {
            let queued = c.transport.queued_tx_counter();
            assert!(c.select_rental_set(index).is_err(), "class {class} index {index}");
            assert_eq!(c.transport.queued_tx_counter(), queued);
            assert_eq!(c.state.phase, LoginPhase::CharacterSelected);
            assert!(c.state.rental_set.is_none());
        }
        if choices > 0 {
            let last = choices - 1;
            c.select_rental_set(last).unwrap();
            assert_eq!(c.state.phase, LoginPhase::RentalSetPending);
            control(&peer, &mut c, set_reply(last), LoginPhase::RentalReady);
            assert_eq!(c.state.rental_set, Some(last));
            pump(&mut c, |_| peer.commands().iter().any(|b| b.first() == Some(&0xca)));
            let sent: Vec<_> = peer
                .commands()
                .into_iter()
                .filter(|b| b.first() == Some(&0xca))
                .collect();
            assert_eq!(sent, vec![vec![0xca, last]]);
        } else {
            assert!(peer.commands().iter().all(|b| b.first() != Some(&0xca)));
        }
        // 沒有目錄列（例如重新選角後目錄尚未到齊）時也不能送 C_CA。
        c.state.characters.clear();
        assert!(c.select_rental_set(0).is_err());
        cleanup(&mut c, &mut peer);
    }
}

#[test]
fn directory_handoff_keeps_cached_room_and_join_must_be_town_mode() {
    for case in 0..3 {
        let mut peer = FaultPeer::new();
        let mut c = selected(&peer, 1);
        c.select_rental_set(1).unwrap();
        control(&peer, &mut c, set_reply(1), LoginPhase::RentalReady);
        c.join_rental_room(0).unwrap();
        assert_eq!(c.state.phase, LoginPhase::RentalArmPending);
        // 重送同一房間（人數更新）可接受；交接狀態不變。
        c.accept_game(&services(&peer, &[ROOM])).unwrap();
        assert_eq!(c.state.phase, LoginPhase::RentalArmPending);
        // 不再提供已快取房間的目錄不能取代交接目標。
        let before = c.state.services.clone();
        assert!(c.accept_game(&services(&peer, &[ROOM + 1])).is_err());
        assert_eq!(c.state.services, before);
        assert_eq!(c.requested_room, Some(ROOM));
        control(&peer, &mut c, vec![0xf0, 0], LoginPhase::RentalAwaitPermission);
        assert!(c.accept_game(&services(&peer, &[])).is_err());
        control(&peer, &mut c, vec![0xf5], LoginPhase::RoomJoining);
        assert!(c.accept_game(&services(&peer, &[ROOM + 2])).is_err());
        assert_eq!(c.state.phase, LoginPhase::RoomJoining);
        let mut joined = bytes("joined");
        let mode = joined
            .windows(3)
            .position(|w| w == [6, 0xf4, 0x01])
            .expect("mode6/map500 in joined fixture");
        match case {
            0 => {
                joined[mode] = 2; // 非租裝村模式
                assert!(c.accept_game(&joined).is_err());
                assert!(c.state.joined_room.is_none());
                assert_eq!(c.state.phase, LoginPhase::RoomJoining);
            }
            1 => {
                joined[mode + 1..mode + 3].copy_from_slice(&4u16.to_le_bytes()); // 非租裝村地圖
                assert!(c.accept_game(&joined).is_err());
                assert!(c.state.joined_room.is_none());
                assert_eq!(c.state.phase, LoginPhase::RoomJoining);
            }
            _ => {
                c.accept_game(&joined).unwrap();
                let room = c.state.joined_room.as_ref().unwrap();
                assert_eq!((room.id, room.mode, room.map_id), (ROOM, 6, 500));
                assert_eq!(c.state.phase, LoginPhase::RoomAwaitSelf);
            }
        }
        pump(&mut c, |_| peer.commands().iter().any(|b| b.first() == Some(&0xcf)));
        let all: Vec<_> = peer
            .commands()
            .iter()
            .map(|b| b[0])
            .filter(|op| *op != 0x20)
            .collect();
        assert_eq!(all, [0xca, 0xd9, 0xcf]);
        cleanup(&mut c, &mut peer);
    }
}

#[test]
fn town_portals_ack_only_ground_hotel_enter_and_hotel_return() {
    // (目前地圖, 目的地圖, 預期種類)；None 表示拒絕且不送 C08。
    let cases: [(u16, u16, Option<TownTransition>); 8] = [
        (500, 500, Some(TownTransition::GroundPortal)),
        (500, 502, Some(TownTransition::HotelEnter)),
        (502, 500, Some(TownTransition::HotelReturn)),
        (560, 560, Some(TownTransition::GroundPortal)),
        (500, 560, None),
        (500, 4, None),
        (560, 501, None),
        (503, 560, None),
    ];
    for (current, target, expected) in cases {
        let mut peer = FaultPeer::new();
        let mut c = seeded_rental(&peer);
        c.readiness_acked = true;
        c.state.environment.begin(c.transport.snapshot().epoch).unwrap();
        c.state.world.map_id = Some(current);
        c.state.merchant.clear();
        let own = c.state.world.own_actor().unwrap();
        let body = portal(target, own.x, own.y);
        let queued = c.transport.queued_tx_counter();
        match expected {
            Some(kind) => {
                c.accept_game(&body).unwrap();
                assert_eq!(c.state.rental_town_transition, Some(kind), "{current}->{target}");
                pump(&mut c, |_| ops(&peer).contains(&0x08));
                assert_eq!(ops(&peer).iter().filter(|&&op| op == 0x08).count(), 1);
            }
            None => {
                assert!(c.accept_game(&body).is_err(), "{current}->{target}");
                assert!(c.state.rental_town_transition.is_none());
                assert!(c.state.world.portal.is_none());
                assert_eq!(c.transport.queued_tx_counter(), queued);
            }
        }
        cleanup(&mut c, &mut peer);
        assert!(!ops(&peer).contains(&0x08) || expected.is_some());
    }
}
