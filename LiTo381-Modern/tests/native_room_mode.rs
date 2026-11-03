use lito_modern::modern_lito::native_room::{self,Packet,Info};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn fixture()->serde_json::Value{serde_json::from_slice(include_bytes!("fixtures/native-room-mode-current.json")).unwrap()}
#[test]
fn formal_room_modes_maps_summary_and_configuration_reject_truncation_and_tails(){
    let f=fixture();assert_eq!(f["rejected_directions"],254);
    for (name,raw) in f["packets"].as_object().unwrap(){
        let b=bytes(raw.as_str().unwrap());assert!(matches!(native_room::parse(&b).unwrap(),Packet::Info{room:u32::MAX,..}),"{name}");
        for n in 0..b.len(){assert!(native_room::parse(&b[..n]).is_err(),"{name}:{n}");}
        let mut tail=b.clone();tail.push(0);assert!(native_room::parse(&tail).is_err());
        let mut zero=b;zero[2..6].fill(0);assert!(native_room::parse(&zero).is_err());
    }
    for v in 0..=255{
        let expected=f["steps"].as_array().unwrap().iter().find(|r|r["value"]==v);
        match expected{Some(row)=>assert_eq!(*native_room::mode_step(v).unwrap(),bytes(row["body"].as_str().unwrap())),None=>assert!(native_room::mode_step(v).is_err())}
    }
    for mode in 7..=255{assert!(native_room::parse(&[0xd5,2,1,0,0,0,mode]).is_err());}
}
#[test]
fn updates_preserve_members_and_reject_other_rooms_without_retaining_passwords(){
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-room-current.json")).unwrap();
    let Packet::Joined(mut room)=native_room::parse(&bytes(f["packets"]["joined"].as_str().unwrap())).unwrap()else{panic!()};
    room.id=u32::MAX;let before=room.clone();assert!(native_room::apply_info(&mut room,1,Info::Mode(5)).is_err());assert_eq!(room,before);
    let f=fixture();for name in ["summary","configuration"]{
        let Packet::Info{room:id,update}=native_room::parse(&bytes(f["packets"][name].as_str().unwrap())).unwrap()else{panic!()};
        native_room::apply_info(&mut room,id,update).unwrap();
    }
    assert_eq!(room.mode,5);assert_eq!(room.map_id,510);assert_eq!(room.owner,"測試角色");assert_eq!(room.name,"模式房間");assert!(room.password_required);assert_eq!(room.capacity,8);assert_eq!(room.members,before.members);
    let encoded=serde_json::to_value(&room).unwrap();assert!(encoded.get("password").is_none());
}
