use lito_modern::modern_lito::{native_doors as doors,native_world::World};
use std::time::{Duration,Instant};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn packet(name:&str)->Vec<u8>{let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();bytes(f["packets"][name].as_str().unwrap())}
fn world()->World {
    let mut w=World::default();for name in ["world","start","status","actor","door","hp","mp"]{w.accept(&packet(name)).unwrap();}
    let own=w.actors.get_mut(&0xfedcba98).unwrap();own.x=32735;own.y=32800;
    w.accept(&bytes("22e07f20800041")).unwrap();w
}
fn action(id:u32,a:u8)->Vec<u8>{let mut b=vec![0x20];b.extend(id.to_le_bytes());b.push(a);b}
#[test]
fn deployed_attribute_and_unsigned_open_vectors_match_native_codec(){
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-doors-current.json")).unwrap();
    assert_eq!(f["attribute_accepted"],4);assert_eq!(f["attribute_rejected"],65532);
    for row in f["attributes"].as_array().unwrap(){
        let a=doors::decode(&bytes(row["body"].as_str().unwrap())).unwrap();
        assert_eq!(a.position,[row["x"].as_u64().unwrap() as u16,row["y"].as_u64().unwrap() as u16]);
        assert_eq!(u64::from(a.direction),row["direction"].as_u64().unwrap());assert_eq!(u64::from(a.passable),row["passable"].as_u64().unwrap());
    }
    for row in f["open_requests"].as_array().unwrap(){
        let mut w=world();w.door_attributes.clear();
        let id=row["id"].as_u64().unwrap() as u32;let x=row["x"].as_u64().unwrap() as u16;let y=row["y"].as_u64().unwrap() as u16;
        let mut door=w.actors.remove(&0x62000000).unwrap();door.id=id;door.x=x;door.y=y;w.actors.insert(id,door);
        let own=w.actors.get_mut(&0xfedcba98).unwrap();own.x=x.saturating_sub(1);own.y=y;
        let mut attr=vec![0x22];attr.extend(x.to_le_bytes());attr.extend(y.to_le_bytes());attr.extend([0,65]);w.accept(&attr).unwrap();
        let choice=doors::Choice::capture(2,&w,id).unwrap();assert_eq!(choice.request(2,&w).unwrap(),bytes(row["body"].as_str().unwrap()));
    }
}
#[test]
fn malformed_attribute_never_partially_changes_world(){
    let w=world();let valid=bytes("22e07f20800041");let mut invalid=Vec::new();
    for n in 0..valid.len(){invalid.push(valid[..n].to_vec());}
    let mut tail=valid.clone();tail.push(0);invalid.push(tail);
    for (at,value) in [(5,2),(5,255),(6,1),(6,64),(6,66),(6,255)]{let mut b=valid.clone();b[at]=value;invalid.push(b);}
    for b in invalid{let mut actual=w.clone();let before=serde_json::to_value(&actual).unwrap();assert!(actual.accept(&b).is_err());assert_eq!(before,serde_json::to_value(actual).unwrap());}
}
#[test]
fn authoritative_toggle_requires_both_packets_and_holds_the_new_pose(){
    let mut w=world();let before=serde_json::to_value(&w).unwrap();let choice=doors::Choice::capture(2,&w,0x62000000).unwrap();
    choice.request(2,&w).unwrap();assert_eq!(before,serde_json::to_value(&w).unwrap());assert!(w.move_request(2).is_err());
    let now=Instant::now();let pending=doors::Pending{choice:choice.clone(),since:now};
    w.accept(&action(choice.id,28)).unwrap();assert_eq!(pending.observe(2,&w,now),doors::Feedback::Pending);assert!(w.move_request(2).is_err());
    w.accept(&bytes("22e07f20800000")).unwrap();assert_eq!(pending.observe(2,&w,now),doors::Feedback::Changed);
    assert_eq!(w.actors[&choice.id].walk_action,28);assert!(w.move_request(2).is_ok());assert!(choice.request(2,&w).is_err());
    let close=doors::Choice::capture(2,&w,choice.id).unwrap();let wait=doors::Pending{choice:close.clone(),since:now};
    assert_eq!(wait.observe(2,&w,now+Duration::from_secs(2)),doors::Feedback::NoResponse);
    assert_eq!(wait.observe(3,&w,now),doors::Feedback::Interrupted);assert!(close.request(3,&w).is_err());
    w.accept(&bytes("22e07f20800041")).unwrap();assert_eq!(wait.observe(2,&w,now),doors::Feedback::Pending);
    w.accept(&action(choice.id,29)).unwrap();assert_eq!(wait.observe(2,&w,now),doors::Feedback::Changed);
    assert_eq!(w.actors[&choice.id].walk_action,29);assert!(w.move_request(2).is_err());
}
#[test]
fn door_identity_removal_replacement_range_and_world_are_retired(){
    let w=world();let choice=doors::Choice::capture(2,&w,0x62000000).unwrap();
    for delta in [[3,0],[0,2]]{let mut far=w.clone();let own=far.actors.get_mut(&0xfedcba98).unwrap();own.x=32736+delta[0];own.y=32800+delta[1];assert!(choice.request(2,&far).is_err());}
    let mut replaced=w.clone();replaced.accept(&packet("door")).unwrap();assert!(replaced.door_attributes.is_empty());
    assert!(choice.request(2,&replaced).is_err());assert!(doors::Choice::capture(2,&replaced,choice.id).is_err());
    replaced.accept(&bytes("22e07f20800041")).unwrap();assert!(choice.request(2,&replaced).is_err());assert!(doors::Choice::capture(2,&replaced,choice.id).is_ok());
    let mut removed=w.clone();let mut b=vec![0x15];b.extend(choice.id.to_le_bytes());removed.accept(&b).unwrap();assert!(removed.door_attributes.is_empty());assert!(!doors::blocks(&removed,32736,32800));
    let mut next=w.clone();next.accept(&packet("world")).unwrap();assert!(next.door_attributes.is_empty());assert!(choice.request(2,&next).is_err());
    let mut unowned=w.clone();unowned.actors.remove(&choice.id);unowned.accept(&bytes("22e07f20800041")).unwrap();assert!(!doors::blocks(&unowned,32736,32800));
}
