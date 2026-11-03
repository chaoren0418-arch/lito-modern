use lito_modern::modern_lito::{
    native_item_actions::{self as actions, Action, Choice, Feedback, Pending},
    native_login::{LoginController, LoginPhase, Snapshot},
};
use std::time::{Duration, Instant};
const ID: u32 = 0xfedcba98;
fn packet(data: &[u8], key: &str) -> Vec<u8> {
    let f: serde_json::Value = serde_json::from_slice(data).unwrap();
    let s = f["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn inv_packet(key: &str) -> Vec<u8> {
    packet(
        include_bytes!("fixtures/native-inventory-current.json"),
        key,
    )
}
fn state() -> Snapshot {
    let mut s = LoginController::default().snapshot().clone();
    s.phase = LoginPhase::LobbyReady;
    s.game_connected = true;
    s.inventory.begin(2, "測試角色").unwrap();
    s.inventory.accept(2, &inv_packet("batch")).unwrap();
    s.inventory.items.get_mut(&ID).unwrap().use_type = 0;
    s
}
fn battle() -> Snapshot {
    let mut s = state();
    s.phase = LoginPhase::BattlePending;
    for k in ["world", "start", "status", "actor", "bot"] {
        s.world
            .accept(&packet(
                include_bytes!("fixtures/native-world-current.json"),
                k,
            ))
            .unwrap();
    }
    s.world.hp = Some([50, 50]);
    s
}
#[test]
fn request_envelopes_match_actual_jar_unsigned_vectors() {
    let f = include_bytes!("fixtures/native-item-actions-current.json");
    assert_eq!(actions::use_request(ID).unwrap(), packet(f, "use"));
    assert_eq!(
        actions::use_request(u32::MAX).unwrap(),
        packet(f, "use_max")
    );
    assert_eq!(
        actions::drop_request([32768, 65535], ID, u32::MAX).unwrap(),
        packet(f, "drop")
    );
    assert!(actions::use_request(0).is_err());
    assert!(actions::drop_request([0, 0], 0, 1).is_err());
    assert!(actions::drop_request([0, 0], 1, 0).is_err());
}
#[test]
fn requests_do_not_predict_inventory_count_name_or_equipment() {
    let s = state();
    let before = serde_json::to_value(&s.inventory).unwrap();
    let choice = Choice::capture(&s, ID).unwrap();
    assert!(actions::request(&s, &choice, Action::Use).is_ok());
    assert_eq!(serde_json::to_value(&s.inventory).unwrap(), before);
    let mut s = s;
    for ty in [1, 2, 23, 25] {
        s.inventory.items.get_mut(&ID).unwrap().use_type = ty;
        let choice = Choice::capture(&s, ID).unwrap();
        assert_eq!(
            actions::request(&s, &choice, Action::Use).unwrap(),
            actions::use_request(ID).unwrap()
        );
    }
}
#[test]
fn specialized_or_unknown_use_types_never_fall_through_to_default_packet() {
    let mut s = state();
    for ty in 0..=255 {
        s.inventory.items.get_mut(&ID).unwrap().use_type = ty;
        let c = Choice::capture(&s, ID).unwrap();
        assert_eq!(
            actions::request(&s, &c, Action::Use).is_ok(),
            [0, 1, 2, 11, 18, 19, 20, 21, 22, 23, 24, 25].contains(&ty)
        );
    }
}
#[test]
fn stale_character_connection_reselection_and_changed_instance_are_rejected() {
    let original = state();
    let c = Choice::capture(&original, ID).unwrap();
    for change in 0..7 {
        let mut s = original.clone();
        match change {
            0 => s.inventory.owner.as_mut().unwrap().connection += 1,
            1 => s.inventory.owner.as_mut().unwrap().selection += 1,
            2 => s.inventory.owner.as_mut().unwrap().character = "另一角色".into(),
            3 => {
                s.inventory.items.remove(&ID);
            }
            4 => s.inventory.items.get_mut(&ID).unwrap().count -= 1,
            5 => s.inventory.items.get_mut(&ID).unwrap().use_type = 10,
            _ => s.game_connected = false,
        };
        assert!(actions::request(&s, &c, Action::Use).is_err());
    }
}
#[test]
fn transition_dead_and_blocked_worlds_cannot_issue_item_requests() {
    let original = battle();
    let choice = Choice::capture(&original, ID).unwrap();
    assert!(actions::request(&original, &choice, Action::Use).is_ok());
    for change in 0..5 {
        let mut s = original.clone();
        match change {
            0 => s.phase = LoginPhase::BattleResult,
            1 => s.world.hp = Some([0, 50]),
            2 => s.world.battle_result = Some(0),
            3 => {
                let id = s.world.own_actor().unwrap().id;
                s.world.actors.get_mut(&id).unwrap().dead = true;
            }
            _ => {
                s.world.movement_blocks.insert(1);
            }
        };
        assert!(actions::request(&s, &choice, Action::Use).is_err());
    }
}
#[test]
fn drop_uses_current_world_position_and_authoritative_quantity_bounds() {
    let mut s = battle();
    let c = Choice::capture(&s, ID).unwrap();
    let a = s.world.own_actor().unwrap();
    let expected = actions::drop_request([a.x, a.y], ID, 4).unwrap();
    assert_eq!(actions::request(&s, &c, Action::Drop(4)).unwrap(), expected);
    assert!(actions::request(&s, &c, Action::Drop(0)).is_err());
    s.inventory.items.get_mut(&ID).unwrap().count = 3;
    let c = Choice::capture(&s, ID).unwrap();
    assert!(actions::request(&s, &c, Action::Drop(4)).is_err());
    s.phase = LoginPhase::LobbyReady;
    assert!(actions::request(&s, &c, Action::Drop(1)).is_err());
}
#[test]
fn pending_tracks_only_selected_item_changes_and_does_not_claim_success() {
    let mut s = state();
    let now = Instant::now();
    let p = Pending::new(&s, Choice::capture(&s, ID).unwrap(), now);
    assert_eq!(p.observe(&s, now), Feedback::Pending);
    s.inventory
        .accept(2, &[0x1a, 7, 0, 0, 0, 1, 0, 0, 0])
        .unwrap();
    assert_eq!(p.observe(&s, now), Feedback::Pending);
    s.inventory.accept(2, &inv_packet("count")).unwrap();
    assert_eq!(p.observe(&s, now), Feedback::InventoryChanged);
    let p = Pending::new(&s, Choice::capture(&s, ID).unwrap(), now);
    s.inventory.accept(2, &inv_packet("remove")).unwrap();
    assert_eq!(p.observe(&s, now), Feedback::InventoryChanged);
}
#[test]
fn pending_expires_without_replay_and_interrupts_on_owner_or_world_change() {
    let s = state();
    let now = Instant::now();
    let p = Pending::new(&s, Choice::capture(&s, ID).unwrap(), now);
    assert_eq!(
        p.observe(&s, now + Duration::from_secs(2)),
        Feedback::NoResponse
    );
    let mut changed = s.clone();
    changed.inventory.suspend();
    assert_eq!(p.observe(&changed, now), Feedback::Interrupted);
    let s = battle();
    let p = Pending::new(&s, Choice::capture(&s, ID).unwrap(), now);
    for kind in 0..3 {
        let mut changed = s.clone();
        match kind {
            0 => changed.world.generation += 1,
            1 => changed.phase = LoginPhase::RoomReady,
            _ => {
                let id = changed.world.own_actor().unwrap().id;
                changed.world.actors.get_mut(&id).unwrap().incarnation += 1;
            }
        };
        assert_eq!(p.observe(&changed, now), Feedback::Interrupted);
    }
}
#[test]
fn formal_ground_items_preserve_quantity_identity_and_strict_envelope() {
    use lito_modern::modern_lito::native_world::actor;
    for (key, id, gfx, name) in [
        ("ground_gem", 0xfedcba98, 773, "測試 (2)"),
        ("ground_potion", 0xfffffffe, 45, "藥水 (2)"),
    ] {
        let b = packet(
            include_bytes!("fixtures/native-item-actions-current.json"),
            key,
        );
        let a = actor(&b).unwrap();
        assert_eq!(
            (a.id, a.graphics, a.experience, a.x, a.y),
            (id, gfx, 2, 32768, 65535)
        );
        assert_eq!(a.name, name);
        assert_eq!((a.walk_action, a.hp_ratio, a.tail_word), (0, 255, 65535));
        for end in 0..b.len() {
            assert!(actor(&b[..end]).is_err());
        }
        let mut trailing = b;
        trailing.push(0);
        assert!(actor(&trailing).is_err());
    }
}

#[test]
fn target_location_envelope_matches_actual_sql_loaded_formal_parser() {
    let f: serde_json::Value = serde_json::from_slice(
        include_bytes!("fixtures/native-target-item-current.json")).unwrap();
    let hex = f["packet"].as_str().unwrap();
    let expected: Vec<u8> = (0..hex.len()).step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
    assert_eq!(actions::target_location_request(9_100_001, 0, [32778,32820]).unwrap(), expected);
    assert_eq!(actions::target_location_request(u32::MAX,u32::MAX,[0,u16::MAX]).unwrap(),
        vec![0x1c,255,255,255,255,255,255,255,255,0,0,255,255]);
    assert!(actions::target_location_request(0,0,[0,0]).is_err());
}
fn targeted_battle() -> Snapshot {
    let mut s = battle();
    s.inventory.items.get_mut(&ID).unwrap().use_type = 5;
    s
}
#[test]
fn type_five_requires_explicit_world_target_without_predicting_authority() {
    let s = targeted_battle();
    let before = serde_json::to_value(&s).unwrap();
    let cursor = actions::TargetChoice::capture(&s,ID).unwrap();
    assert!(actions::request(&s,cursor.choice(),Action::Use).is_err());
    assert_eq!(actions::target_request(&s,&cursor,0,[32778,32820]).unwrap(),
        actions::target_location_request(ID,0,[32778,32820]).unwrap());
    assert_eq!(serde_json::to_value(&s).unwrap(),before);
    let mut lobby = s.clone();
    lobby.phase = LoginPhase::LobbyReady;
    assert!(actions::TargetChoice::capture(&lobby,ID).is_err());
    let mut ordinary = s.clone();
    ordinary.inventory.items.get_mut(&ID).unwrap().use_type = 0;
    assert!(actions::TargetChoice::capture(&ordinary,ID).is_err());
}
#[test]
fn target_cursor_rejects_retired_world_even_with_unchanged_inventory() {
    let original = targeted_battle();
    let cursor = actions::TargetChoice::capture(&original,ID).unwrap();
    for change in 0..15 {
        let mut s = original.clone();
        let own = s.world.own_actor().unwrap().id;
        match change {
            0 => s.world.generation += 1,
            1 => s.world.local_corrections += 1,
            2 => s.world.movement_locks += 1,
            3 => s.world.control_interruptions += 1,
            4 => s.world.actors.get_mut(&own).unwrap().incarnation += 1,
            5 => s.world.actors.get_mut(&own).unwrap().dead = true,
            6 => s.world.hp = Some([0,50]),
            7 => s.world.battle_result = Some(0),
            8 => { s.world.movement_blocks.insert(1); },
            9 => s.inventory.owner.as_mut().unwrap().connection += 1,
            10 => s.inventory.owner.as_mut().unwrap().selection += 1,
            11 => s.inventory.items.get_mut(&ID).unwrap().count -= 1,
            12 => s.game_connected = false,
            13 => s.observer = true,
            _ => s.world.observer_mode = true,
        }
        assert!(actions::target_request(&s,&cursor,0,[32778,32820]).is_err(),"change {change}");
    }
}
#[test]
fn actor_target_requires_current_live_coordinates_and_presence() {
    let mut s = targeted_battle();
    let cursor = actions::TargetChoice::capture(&s,ID).unwrap();
    let a = s.world.actors.values().find(|a| Some(a.id)!=s.world.own_actor().map(|a|a.id)).unwrap();
    let (target,position) = (a.id,[a.x,a.y]);
    assert!(actions::target_request(&s,&cursor,target,position).is_ok());
    s.world.actors.get_mut(&target).unwrap().x += 1;
    assert!(actions::target_request(&s,&cursor,target,position).is_err());
    s.world.actors.get_mut(&target).unwrap().x = position[0];
    s.world.actors.get_mut(&target).unwrap().dead = true;
    assert!(actions::target_request(&s,&cursor,target,position).is_err());
    s.world.actors.remove(&target);
    assert!(actions::target_request(&s,&cursor,target,position).is_err());
}

#[test]
fn target_pending_requires_exact_new_rejection_or_authoritative_portal() {
    let mut s=targeted_battle();let now=Instant::now();
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-target-rejection-current.json")).unwrap();
    let packet:Vec<u8>=f["packet"].as_str().unwrap().split_whitespace().map(|b|u8::from_str_radix(b,16).unwrap()).collect();
    s.world.accept(&packet).unwrap();
    let p=Pending::new_target(&s,Choice::capture(&s,ID).unwrap(),now);
    assert_eq!(p.observe(&s,now),Feedback::Pending); // an old text is not this request's reply
    s.world.accept(&[0x0f,9,b'O',b'K',0]).unwrap();
    assert_eq!(p.observe(&s,now),Feedback::Pending); // unrelated system text
    s.world.accept(&packet).unwrap();
    assert_eq!(p.observe(&s,now),Feedback::TargetRejected);
    let p=Pending::new_target(&s,Choice::capture(&s,ID).unwrap(),now);
    assert_eq!(p.observe(&s,now+Duration::from_secs(2)),Feedback::TargetNoResponse);
    let mut reset=s.clone();reset.world.generation+=1;
    assert_eq!(p.observe(&reset,now),Feedback::Interrupted);
    let mut portal=vec![0x5a];portal.extend(s.world.map_id.unwrap().to_le_bytes());
    portal.extend([32778u16,32820u16].into_iter().flat_map(u16::to_le_bytes));
    s.world.accept(&portal).unwrap();assert_eq!(p.observe(&s,now),Feedback::SceneUpdated);
    s.inventory.owner.as_mut().unwrap().connection+=1;
    assert_eq!(p.observe(&s,now),Feedback::Interrupted);
}

#[test]
fn ordinary_portal_admission_requires_live_room_and_same_map() {
    use lito_modern::modern_lito::{native_portal::{admit,Destination},native_room};
    let mut s=battle();
    // Same exact parsed formal room fixture used by room integration tests.
    let room_fixture:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-room-current.json")).unwrap();
    let hex=room_fixture["packets"]["joined"].as_str().unwrap();
    let bytes:Vec<u8>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
    s.joined_room=Some(match native_room::parse(&bytes).unwrap(){native_room::Packet::Joined(room)=>room,_=>panic!("joined fixture")});
    s.joined_room.as_mut().unwrap().mode=0;
    let d=Destination{map:s.world.map_id.unwrap(),x:32778,y:32820};
    assert!(admit(&s,d).is_ok());
    assert!(admit(&s,Destination{map:d.map+1,..d}).is_err());
    for change in 0..6 {
        let mut bad=s.clone();
        match change {0=>bad.phase=LoginPhase::RoomReady,1=>bad.game_connected=false,
            2=>bad.observer=true,3=>bad.joined_room=None,4=>bad.world.hp=Some([0,50]),
            _=>bad.world.battle_result=Some(0)}
        assert!(admit(&bad,d).is_err());
    }
    s.joined_room.as_mut().unwrap().mode=6;
    assert!(admit(&s,Destination{map:d.map+1,..d}).is_ok());
    // Adding ordinary travel must not narrow the old rental gate. A server
    // portal is still handled by World authority; no movement permission or HP
    // is granted by admission itself.
    s.world.hp=Some([0,50]);
    let own=s.world.own_actor().unwrap().id;s.world.actors.get_mut(&own).unwrap().dead=true;
    assert!(admit(&s,Destination{map:d.map+1,..d}).is_ok());
}
