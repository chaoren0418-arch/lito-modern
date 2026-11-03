use lito_modern::modern_lito::{native_game::{self,Packet},native_login::LoginController};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
#[test]
fn classic_entry_preserves_received_identity_and_formal_check_domain(){
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-classic-entry-current.json")).unwrap();
    for r in f["requests"].as_array().unwrap(){assert_eq!(native_game::classic_entry_request(r["name"].as_str().unwrap()).unwrap().as_slice(),bytes(r["body"].as_str().unwrap()));}
    for r in f["replies"].as_array().unwrap(){let b=bytes(r["body"].as_str().unwrap());let Packet::ClassicEntry(c)=native_game::parse(&b).unwrap()else{panic!()};assert_eq!(c.accepted,r["accepted"].as_bool().unwrap());assert_eq!(c.value,u32::try_from(r["value"].as_u64().unwrap()).unwrap());
        for n in 0..b.len(){assert!(native_game::parse(&b[..n]).is_err());}let mut tail=b;tail.push(0);assert!(native_game::parse(&tail).is_err());
    }
    for v in 0..=255 {if !matches!(v,0|3){assert!(native_game::parse(&[7,v,0,0,0,0]).is_err());}}
    for name in ["","a\0b","😀","NNNNNNNNNNNNNNNNNNNNNNNNNNNN"]{assert!(native_game::classic_entry_request(name).is_err());}
}
#[test]
fn classic_entry_cannot_start_without_current_character_directory(){
    let mut c=LoginController::default();assert!(c.request_classic_entry(0).is_err());assert!(c.snapshot().classic_entry_result.is_none());assert!(c.snapshot().world.map_id.is_none());assert!(c.is_quiescent());
}
