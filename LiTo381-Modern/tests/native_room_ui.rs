use lito_modern::modern_lito::{native_login::{LoginController,LoginPhase,Snapshot},native_lobby::Channel,
    native_room::{self,Packet},native_room_ui::{Draft,MemberChoice}};
fn bytes(v:&serde_json::Value)->Vec<u8>{let s=v.as_str().unwrap();(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn room()->Snapshot {
    let v:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-room-current.json")).unwrap();
    let Packet::Joined(mut r)=native_room::parse(&bytes(&v["packets"]["joined"])).unwrap()else{panic!()};
    let Packet::Member{mut member,..}=native_room::parse(&bytes(&v["packets"]["member_0"])).unwrap()else{panic!()};
    member.name=r.owner.clone();member.master=true;r.members.push(member);
    let mut s=LoginController::default().snapshot().clone();s.phase=LoginPhase::RoomReady;
    s.selected_character=Some(r.owner.clone());s.channel=Some(Channel{id:0,users:2,capacity:100});s.joined_room=Some(r);s
}
#[test]
fn draft_cannot_outlive_connection_character_channel_owner_or_received_configuration(){
    let original=room();let draft=Draft::capture(7,&original,true).unwrap();
    assert!(draft.validate(8,&original).is_err());
    for change in 0..8 {
        let mut s=original.clone();
        match change {
            0=>s.selected_character=Some("other".into()),1=>s.channel.as_mut().unwrap().id+=1,
            2=>s.joined_room.as_mut().unwrap().id+=1,3=>s.joined_room.as_mut().unwrap().owner="other".into(),
            4=>s.joined_room.as_mut().unwrap().capacity+=1,5=>s.joined_room.as_mut().unwrap().options[0]^=1,
            6=>s.phase=LoginPhase::RoomEditing,_=>s.joined_room=None,
        }
        assert!(draft.validate(7,&s).is_err(),"change {change}");
    }
    let mut s=original.clone();s.joined_room.as_mut().unwrap().members[0].state=1;
    assert!(draft.validate(7,&s).is_ok()); // Readiness is not an edit of the displayed configuration.
    s.joined_room=None;s.phase=LoginPhase::LobbyReady;let draft=Draft::capture(7,&s,false).unwrap();
    assert!(draft.settings(7,&s,"新房間","",false,8).is_ok());
    s.joined_room=original.joined_room;assert!(draft.validate(7,&s).is_err());
}
#[test]
fn edits_preserve_opaque_received_options_and_match_formal_bytes_without_retaining_password(){
    let raw=include_bytes!("fixtures/native-room-admin-current.json");
    let v:serde_json::Value=serde_json::from_slice(raw.strip_prefix(&[239,187,191]).unwrap_or(raw)).unwrap();
    let row=v["settings"].as_array().unwrap().iter().find(|r|r["opcode"]==0xdd && r["attributes"].as_u64().unwrap()==60 && r["options"].as_array().is_some_and(|v|v[0]==7) && r["capacity"].as_u64().unwrap()<=100).unwrap();
    let mut s=room();let r=s.joined_room.as_mut().unwrap();
    r.name=row["name"].as_str().unwrap().into();r.attributes=row["attributes"].as_u64().unwrap() as u8;r.password_required=true;
    r.capacity=row["capacity"].as_u64().unwrap() as u16;
    r.options=std::array::from_fn(|i|row["options"][i].as_u64().unwrap() as u8);
    r.selections=row["selections"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap() as u8).collect();
    let draft=Draft::capture(9,&s,true).unwrap();let password=row["password"].as_str().unwrap();
    let settings=draft.settings(9,&s,draft.name(),password,true,draft.capacity()).unwrap();
    assert_eq!(settings.request(0xdd).unwrap().as_slice(),bytes(&row["body"]));
    assert!(draft.settings(9,&s,draft.name(),"",true,8).is_err());
    assert!(draft.settings(9,&s,draft.name(),password,false,8).is_err());
    let cleared=draft.settings(9,&s,draft.name(),"",false,8).unwrap();
    assert_eq!(cleared.attributes,s.joined_room.as_ref().unwrap().attributes&!4);
    assert_eq!(cleared.options.unwrap().0,s.joined_room.as_ref().unwrap().options);
    for cap in [0,1,101,u16::MAX] {assert!(draft.settings(9,&s,"房","",false,cap).is_err());}
    assert!(!format!("{draft:?}").contains(password));
}
#[test]
fn kick_choice_keeps_named_member_across_reordering_and_rejects_changed_identity(){
    let mut s=room();let r=s.joined_room.as_mut().unwrap();let mut peer=r.members[0].clone();peer.name="客人".into();peer.master=false;
    r.members.push(peer);let index=r.members.len()-1;let choice=MemberChoice::capture(3,&s,index).unwrap();
    assert!(MemberChoice::capture(3,&s,0).is_err());
    s.joined_room.as_mut().unwrap().members.swap(0,index);
    assert_eq!(choice.resolve(3,&s).unwrap(),"客人");
    assert!(choice.resolve(4,&s).is_err());
    let original=s.clone();
    for change in 0..5 {let mut s=original.clone();match change {
        0=>s.joined_room.as_mut().unwrap().id+=1,1=>s.joined_room.as_mut().unwrap().owner="另一房主".into(),
        2=>s.joined_room.as_mut().unwrap().members[0].rank+=1,3=>{s.joined_room.as_mut().unwrap().members.remove(0);},
        _=>s.phase=LoginPhase::KickPending,
    };assert!(choice.resolve(3,&s).is_err(),"change {change}");}
}


#[test]
fn lobby_room_choice_follows_id_through_live_counts_and_order_without_crossing_scope(){
    use lito_modern::modern_lito::{native_lobby::Room,native_room_ui::RoomChoice};
    let mut s=room();s.phase=LoginPhase::LobbyReady;s.joined_room=None;
    s.rooms=vec![Room{id:1,name:"原房".into(),attributes:0,users:1,capacity:8,reserved:0,option:0},
        Room{id:2,name:"攻城入口測試".into(),attributes:0,users:28,capacity:30,reserved:0,option:0}];
    let choice=RoomChoice::capture(7,&s,1).unwrap();
    s.rooms[1].users=29;s.rooms[1].name="仍是同房間".into();s.rooms.swap(0,1);
    assert_eq!(choice.index(7,&s).unwrap(),0);assert_eq!(choice.join_index(7,&s).unwrap(),0);
    s.phase=LoginPhase::LoadingRooms;assert_eq!(choice.index(7,&s).unwrap(),0);assert!(choice.join_index(7,&s).is_err());
    assert_eq!(RoomChoice::capture(7,&s,0).unwrap().index(7,&s).unwrap(),0);
    s.phase=LoginPhase::LobbyReady;let original=s.clone();
    for change in 0..8 {let mut x=original.clone();match change {
        0=>x.channel.as_mut().unwrap().id+=1,1=>x.selected_character=Some("別人".into()),
        2=>{x.rooms.remove(0);},3=>x.phase=LoginPhase::RoomJoining,4=>x.channel=None,
        5=>x.joined_room=room().joined_room,6=>x.phase=LoginPhase::Closing,_=>x.selected_character=None,
    };assert!(choice.index(7,&x).is_err(),"change{change}");}
    assert!(choice.index(8,&original).is_err());assert!(RoomChoice::capture(7,&original,usize::MAX).is_err());
    assert!(RoomChoice::capture(7,&room(),0).is_err());
}
