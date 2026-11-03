use lito_modern::modern_lito::{native_merchant as m,native_inventory};
use std::time::{Instant,Duration};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn fixture()->serde_json::Value{serde_json::from_slice(include_bytes!("fixtures/native-hotel-input-current.json")).unwrap()}
#[test]
fn formal_numeric_request_bounds_and_reply_match_pinned_protocol(){
 let f=fixture();let b=bytes(f["input"].as_str().unwrap());let input=m::parse_input(&b).unwrap();
 assert_eq!((input.minimum,input.current,input.maximum,input.mode),(1,1,8,0));assert_eq!(input.html,"inn2");assert_eq!(input.arguments,["旅館服務生-席巴"]);
 assert_eq!(input.reply(8).unwrap(),bytes(f["reply"].as_str().unwrap()));assert!(input.reply(0).is_err());assert!(input.reply(9).is_err());
 for n in 0..b.len(){assert!(m::parse_input(&b[..n]).is_err(),"truncated {n}");}let mut tail=b.clone();tail.push(0);assert!(m::parse_input(&tail).is_err());
 for (offset,v) in [(1,0),(9,9),(13,2),(17,u32::MAX)]{let mut bad=b.clone();bad[offset..offset+4].copy_from_slice(&v.to_le_bytes());assert!(m::parse_input(&bad).is_err());}
 let edge=m::parse_input(&bytes(f["boundary"].as_str().unwrap())).unwrap();assert_eq!(edge.npc,u32::MAX);assert_eq!(edge.maximum,i32::MAX as u32);assert!(edge.reply(1).is_err());
}
#[test]
fn quantity_request_is_owned_expiring_and_cannot_be_reused_after_submission(){
 let f=fixture();let input=m::parse_input(&bytes(f["input"].as_str().unwrap())).unwrap();let now=Instant::now();
 let owner=m::Owner{connection:2,generation:4,incarnation:1,control_interruptions:0,inventory:native_inventory::Owner{connection:2,selection:1,character:"玩家".into()}};
 let mut s=m::Session::default();assert!(s.accept_input(input.clone(),now).is_err());s.begin(owner.clone(),input.npc,now).unwrap();s.accept_input(input.clone(),now).unwrap();let rev=s.revision;
 assert!(s.input_reply(rev,8,now).is_ok());assert!(s.input_reply(rev+1,8,now).is_err());assert!(s.input_reply(rev,8,now+Duration::from_secs(30)).is_err());
 s.submitted();assert!(s.input_reply(rev,8,now).is_err());
 s.begin(owner.clone(),input.npc,now).unwrap();s.accept_input(input.clone(),now).unwrap();let mut changed=owner.clone();changed.generation+=1;s.tick(Some(&changed),now);assert!(s.input.is_none());
 s.begin(owner.clone(),input.npc,now).unwrap();s.accept_input(input,now).unwrap();s.tick(Some(&owner),now+Duration::from_secs(30));assert!(s.input.is_none());
}
