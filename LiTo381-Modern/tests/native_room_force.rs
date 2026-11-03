use lito_modern::modern_lito::{native_room::{self,Packet},native_login::LoginController};
#[test]
fn deployed_force_contract_is_bounded_and_preserves_authoritative_name() {
    let oracle:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-force-current.json")).unwrap();
    for force in 0..=255 {
        assert_eq!(native_room::force_request(force).is_ok(),force<=1);
    }
    for force in 0..=1 {
        let s=oracle["packets"][force.to_string()].as_str().unwrap();
        let bytes:Vec<u8>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
        assert_eq!(native_room::parse(&bytes).unwrap(),Packet::Force{name:"測試成員".into(),force});
        for n in 0..bytes.len(){assert!(native_room::parse(&bytes[..n]).is_err());}
        let mut extra=bytes.clone();extra.push(0);assert!(native_room::parse(&extra).is_err());
        let mut invalid=bytes;*invalid.last_mut().unwrap()=2;assert!(native_room::parse(&invalid).is_err());
        assert_eq!(native_room::force_request(force).unwrap().as_slice(),[0xd3,force]);
    }
    assert!(native_room::parse(&[0xd4,0,0]).is_err());
}
#[test]
fn disconnected_controller_cannot_queue_a_force_change() {
    let mut c=LoginController::default();
    for force in 0..=255 {assert!(c.set_force(force).is_err());}
    assert!(c.is_quiescent());
}
fn captured_room_packets()->Vec<Vec<u8>> {
    let fixture:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-room-force-order-current.json")).unwrap();
    ["joined","force","member"].iter().map(|key| {
        let s=fixture["packets"][key].as_str().unwrap();
        (0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()
    }).collect()
}

#[test]
fn captured_force_before_roster_does_not_create_a_member_or_override_full_snapshot() {
    use native_room::PendingRosterForces;
    let p=captured_room_packets();
    let Packet::Joined(mut room)=native_room::parse(&p[0]).unwrap() else {panic!()};
    let Packet::Force{name,force}=native_room::parse(&p[1]).unwrap() else {panic!()};
    let Packet::Member{room:id,mut member}=native_room::parse(&p[2]).unwrap() else {panic!()};
    let now=std::time::Instant::now();let mut pending=PendingRosterForces::default();
    assert!(!pending.apply(&mut room,&name,force,true,now).unwrap());
    assert!(room.members.is_empty());
    pending.member_received(id,&member.name).unwrap();native_room::upsert(&mut room,member.clone()).unwrap();
    assert_eq!(room.members[0].force,1);
    assert!(!pending.expired(now+std::time::Duration::from_secs(9)));
    // A subsequent full snapshot is authoritative, even if an earlier force differed.
    pending.clear();room.members.clear();member.force=0;
    pending.apply(&mut room,&name,1,true,now).unwrap();pending.member_received(id,&name).unwrap();
    native_room::upsert(&mut room,member).unwrap();assert_eq!(room.members[0].force,0);
    assert!(pending.apply(&mut room,&name,1,false,now).unwrap());assert_eq!(room.members[0].force,1);
}
#[test]
fn incomplete_roster_force_is_bounded_and_cannot_cross_room_or_connection() {
    use native_room::PendingRosterForces;
    let Packet::Joined(mut room)=native_room::parse(&captured_room_packets()[0]).unwrap() else {panic!()};
    room.capacity=2;let now=std::time::Instant::now();let mut p=PendingRosterForces::default();
    assert!(p.apply(&mut room,"unknown",1,false,now).is_err());
    assert!(!p.apply(&mut room,"one",1,true,now).unwrap());
    assert!(!p.apply(&mut room,"two",0,true,now).unwrap());
    assert!(p.apply(&mut room,"three",0,true,now).is_err());
    p.apply(&mut room,"one",0,true,now+std::time::Duration::from_secs(7)).unwrap();
    assert!(p.expired(now+std::time::Duration::from_secs(8)),"duplicates cannot extend the deadline");
    assert!(p.member_received(room.id+1,"one").is_err());
    room.id+=1;assert!(p.apply(&mut room,"one",1,true,now).is_err());
    p.clear();assert!(!p.expired(now+std::time::Duration::from_secs(9)));
    assert!(!p.apply(&mut room,"one",1,true,now).unwrap());
}
