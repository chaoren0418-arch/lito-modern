use lito_modern::modern_lito::{native_login::LoginController,native_room::{self,Settings,Packet}};
fn fixture()->serde_json::Value {
    let raw=include_bytes!("fixtures/native-room-admin-current.json");
    serde_json::from_slice(raw.strip_prefix(&[0xef,0xbb,0xbf]).unwrap_or(raw)).unwrap()
}
fn bytes(v:&serde_json::Value)->Vec<u8>{let s=v.as_str().unwrap();(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
#[test]
fn settings_match_formal_vectors_and_wire_capacity_domain(){
    let f=fixture();assert_eq!(f["settings"].as_array().unwrap().len(),144);
    for row in f["settings"].as_array().unwrap(){
        let selections:Vec<u8>=row["selections"].as_array().unwrap().iter().map(|x|x.as_u64().unwrap() as u8).collect();
        let values=row["options"].as_array().map(|a|std::array::from_fn(|i|a[i].as_u64().unwrap() as u8));
        let s=Settings{name:row["name"].as_str().unwrap(),password:row["password"].as_str().unwrap(),attributes:row["attributes"].as_u64().unwrap() as u8,
            capacity:row["capacity"].as_u64().unwrap() as u16,options:values.map(|a|(a,selections.as_slice()))};
        assert_eq!(s.request(row["opcode"].as_u64().unwrap() as u8).unwrap().as_slice(),bytes(&row["body"]));
    }
    let mut s=Settings{name:"測試",password:"",attributes:0,capacity:8,options:None};
    let mut accepted=0;for cap in 0..=u16::MAX {s.capacity=cap;accepted+=usize::from(s.request(0xcc).is_ok());}
    assert_eq!(accepted,f["accepted_wire_capacities"].as_u64().unwrap() as usize);
    s.capacity=8;let mut accepted=0;for attr in 0..=u8::MAX {s.attributes=attr;s.password=if attr&4!=0 {"密碼"}else{""};accepted+=usize::from(s.request(0xdd).is_ok());}
    assert_eq!(accepted,f["accepted_attributes"].as_u64().unwrap() as usize);
}
#[test]
fn settings_reject_unrepresentable_fields_and_invalid_option_combinations(){
    let mut s=Settings{name:"房",password:"",attributes:0,capacity:8,options:None};
    for name in ["","a\0b","a\nb","😀"]{s.name=name;assert!(s.request(0xcc).is_err());}
    let long="N".repeat(90);s.name=&long;assert!(s.request(0xcc).is_err());s.name="房";
    s.password="密碼";assert!(s.request(0xcc).is_err());s.attributes=4;s.password="密碼長";assert!(s.request(0xcc).is_err());
    s.password="";assert!(s.request(0xcc).is_err());s.attributes=0;
    for opts in [[8,0,1,0],[0,3,1,0],[0,0,0,1],[0,0,128,0],[0,0,1,32]] {s.options=Some((opts,&[]));assert!(s.request(0xdd).is_err());}
    for choices in [&[0][..],&[1;9][..]]{s.options=Some(([1,1,1,1],choices));assert!(s.request(0xdd).is_err());}
    s.options=None;assert!(s.request(0xcf).is_err());
}
#[test]
fn observer_and_kick_replies_match_formal_and_reject_partial_envelopes(){
    let f=fixture();let mut replies=vec![];
    for row in f["observer"].as_array().unwrap(){let enabled=row["enabled"].as_bool().unwrap();assert_eq!(native_room::observer_request(enabled).as_slice(),bytes(&row["request"]));let b=bytes(&row["reply"]);assert_eq!(native_room::parse(&b).unwrap(),Packet::Observer(enabled));replies.push(b);}
    for row in f["kick"].as_array().unwrap(){let name=row["name"].as_str().unwrap();let request=native_room::kick_request(name);
        // Formal C_D4 accepts longer strings; this client only targets received 27-byte member identities.
        if name.len()==63 {assert!(request.is_err());continue;}
        assert_eq!(request.unwrap().as_slice(),bytes(&row["request"]));let b=bytes(&row["notice"]);assert_eq!(native_room::parse(&b).unwrap(),Packet::Kicked(name.to_owned()));replies.push(b);
    }
    for row in f["kick_status"].as_array().unwrap(){let b=bytes(&row["body"]);assert_eq!(native_room::parse(&b).unwrap(),Packet::KickStatus(row["state"]=="ACCEPTED"));replies.push(b);}
    for b in replies{for n in 0..b.len(){assert!(native_room::parse(&b[..n]).is_err());}let mut tail=b;tail.push(0);assert!(native_room::parse(&tail).is_err());}
    for value in 2..=u8::MAX {assert!(native_room::parse(&[0xe5,value]).is_err());assert!(native_room::parse(&[0xfc,value,0,0,0]).is_err());}
}
#[test]
fn admin_actions_require_live_phase_and_do_not_mutate_idle_snapshot(){
    let mut c=LoginController::default();let s=Settings{name:"房",password:"",attributes:0,capacity:8,options:None};
    assert!(c.create_room(&s).is_err());assert!(c.edit_room(&s).is_err());assert!(c.kick_member("測試").is_err());assert!(c.set_observer(true).is_err());
    assert!(!c.snapshot().observer);assert!(c.snapshot().room_kick_result.is_none());assert!(c.snapshot().joined_room.is_none());assert!(c.is_quiescent());
}
