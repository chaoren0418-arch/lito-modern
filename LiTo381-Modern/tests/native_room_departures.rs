use lito_modern::modern_lito::native_room::{self,Departure,Packet,RetiredDepartures,Room};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn room()->Room {
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-room-current.json")).unwrap();
    let Packet::Joined(mut r)=native_room::parse(&bytes(f["packets"]["joined"].as_str().unwrap())).unwrap() else {panic!()};
    let Packet::Member{member,..}=native_room::parse(&bytes(f["packets"]["member_0"].as_str().unwrap())).unwrap() else {panic!()};
    r.id=1;r.members=vec![member.clone(),member];
    r.members[0].name="PeerFixture(測試乙)".into();r.members[1].name="PeerFixture2(測試乙)".into();r
}
// Actual S_D0 bodies from the 20261013-062002-218 simultaneous exit failure.
const A:&str="d001000000506565724669787475726528b4fab8d5a4412900";
const B:&str="d00100000050656572466978747572653228b4fab8d5a4412900";
fn apply(active:&mut Option<Room>,retired:&mut RetiredDepartures,body:&str,own:&str)->anyhow::Result<Departure>{
    let Packet::Departed{room,name}=native_room::parse(&bytes(body))? else {panic!()};
    native_room::depart(active,retired,room,&name,own)
}
#[test]
fn recorded_simultaneous_departures_accept_both_network_orderings() {
    for (own,first,second) in [("PeerFixture(測試乙)",A,B),("PeerFixture2(測試乙)",B,A)] {
        let mut active=Some(room());let mut retired=RetiredDepartures::default();
        assert_eq!(apply(&mut active,&mut retired,first,own).unwrap(),Departure::SelfLeft);
        assert!(active.is_none());
        assert_eq!(apply(&mut active,&mut retired,second,own).unwrap(),Departure::RetiredPeer);
        assert!(active.is_none());
        assert!(apply(&mut active,&mut retired,second,own).is_err(),"a retired departure is consumed once");
        let mut active=Some(room());let mut retired=RetiredDepartures::default();
        assert_eq!(apply(&mut active,&mut retired,second,own).unwrap(),Departure::PeerLeft);
        assert_eq!(apply(&mut active,&mut retired,first,own).unwrap(),Departure::SelfLeft);
        assert!(apply(&mut active,&mut retired,second,own).is_err());
    }
}
#[test]
fn retired_notifications_cannot_authorize_unknown_or_rejoined_room_changes() {
    let own="PeerFixture(測試乙)";let peer="PeerFixture2(測試乙)";
    let mut active=Some(room());let mut retired=RetiredDepartures::default();
    apply(&mut active,&mut retired,A,own).unwrap();
    assert!(native_room::depart(&mut active,&mut retired,2,peer,own).is_err());
    assert!(native_room::depart(&mut active,&mut retired,1,"unknown",own).is_err());
    assert!(native_room::depart(&mut active,&mut retired,1,own,own).is_err());
    let mut next=room();next.id=2;active=Some(next);
    assert!(apply(&mut active,&mut retired,B,own).is_err());
    assert_eq!(active.as_ref().unwrap().members.len(),2);
    // Controller clears this tombstone on a new room acceptance or connection reset.
    retired.clear();active=None;
    assert!(apply(&mut active,&mut retired,B,own).is_err());
}
