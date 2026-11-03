use lito_modern::modern_lito::{native_portal as p,native_world::World};
fn bytes(v:&serde_json::Value)->Vec<u8>{let s=v.as_str().unwrap();(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn packets()->serde_json::Value{serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap()}
fn fixture()->serde_json::Value{serde_json::from_slice(include_bytes!("fixtures/native-portal-current.json")).unwrap()}
fn active()->World{let f=packets();let mut w=World::default();for key in ["world","status","actor","bot","hp","mp"]{if key=="status"{w.accept(&[0xd2]).unwrap();}w.accept(&bytes(&f["packets"][key])).unwrap();}assert!(w.ready());w}
#[test]
fn formal_portal_payload_is_exact_and_malformed_packets_do_not_retire_world(){
 let f=fixture();assert_eq!(p::ACK.to_vec(),bytes(&f["ack"]));let b=bytes(&f["boundary"]);assert_eq!(p::parse(&b).unwrap(),p::Destination{map:65535,x:0,y:65535});
 for n in 0..b.len(){let mut w=active();let prior=serde_json::to_value(&w).unwrap();assert!(w.accept(&b[..n]).is_err());assert_eq!(serde_json::to_value(w).unwrap(),prior);}
 let mut tail=b;tail.push(0);assert!(p::parse(&tail).is_err());assert!(World::default().accept(&bytes(&f["same_map"])).is_err());
}
#[test]
fn same_map_portal_retires_old_scene_and_waits_for_matching_received_self(){
 let f=fixture();let v=packets();let mut w=active();w.learned_skills.insert(15);let generation=w.generation;
 w.accept(&bytes(&f["same_map"])).unwrap();assert!(!w.ready());assert!(w.started&&w.actors.is_empty()&&w.hp.is_none());assert_eq!(w.generation,generation+1);assert!(w.learned_skills.contains(&15));
 assert!(!w.finish_portal_if_ready().unwrap());assert!(w.accept(&bytes(&f["same_map"])).is_err());
 w.accept(&bytes(&v["packets"]["status"])).unwrap();w.accept(&bytes(&v["packets"]["bot"])).unwrap();assert!(!w.finish_portal_if_ready().unwrap());
 w.accept(&bytes(&v["packets"]["actor"])).unwrap();assert!(!w.ready());assert!(w.finish_portal_if_ready().unwrap());assert!(w.ready());assert_eq!(w.completed_portals,1);assert!(!w.finish_portal_if_ready().unwrap());
}
#[test]
fn different_map_portal_rejects_wrong_rebuild_and_cancellation_retires_pending(){
 let f=fixture();let v=packets();let mut w=active();let packet=bytes(&f["rooms"][0]["portal"]);w.accept(&packet).unwrap();let destination=w.portal.unwrap().destination;assert_eq!(w.map_id,Some(destination.map));
 w.accept(&bytes(&v["packets"]["status"])).unwrap();w.accept(&bytes(&v["packets"]["actor"])).unwrap();assert!(w.finish_portal_if_ready().is_err());assert!(!w.ready());
 w.clear_preserving_generation();assert!(w.portal.is_none()&&!w.ready()&&w.actors.is_empty());
}
