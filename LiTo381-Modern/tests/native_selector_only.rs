//! Regression of captured C02 selector-only packets and actual S39/3934 acknowledgement.
use lito_modern::modern_lito::{native_skills as skills,native_skill_selection as selection,native_attack_many as many,native_world::World};
fn fixture()->serde_json::Value{serde_json::from_slice(include_bytes!("fixtures/native-selector-caster3-s39-current.json")).unwrap()}
fn bytes(hex:&str)->Vec<u8>{(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect()}
fn world()->World{let f=fixture();let mut w=World::default();for row in f["packets"].as_array().unwrap(){assert!(w.accept(&bytes(row["hex"].as_str().unwrap())).unwrap());}assert_eq!(w.own_actor().unwrap().id,3);w}
fn s39()->Vec<u8>{bytes(fixture()["s39"]["hex"].as_str().unwrap())}
#[test]
fn nine_server_selectors_ignore_hover_and_never_spend_local_authority(){
 let w=world();let before=serde_json::to_value(&w).unwrap();
 for (id,hex) in [(14,"140105"),(31,"140306"),(32,"140307"),(49,"140600"),(52,"140603"),(54,"140605"),(72,"140807"),(78,"140905"),(206,"141905")]{
  let hover=selection::Target::capture(&w,1);assert!(hover.is_some());let choice=selection::choice(&w,id,hover).unwrap();assert!(choice.target.is_none()&&choice.ground.is_none());
  assert_eq!(skills::request(&w,id,choice.target).unwrap(),bytes(hex));assert!(skills::request(&w,id,Some(1)).is_err());assert!(skills::request(&w,id,Some(3)).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
 }
}
#[test]
fn selector_learning_death_and_mana_guards_do_not_forge_requests(){
 for id in [14,31,32,49,52,54,72,78,206]{
  let mut w=world();w.learned_skills.remove(&id);assert!(skills::request(&w,id,None).is_err());
  let mut w=world();w.mp=Some([0,1000]);assert!(skills::request(&w,id,None).is_err());
  let mut w=world();w.actors.get_mut(&3).unwrap().dead=true;assert!(skills::request(&w,id,None).is_err());
 }
}
#[test]
fn counter72_actual_s39_acks_cast_without_s37_or_local_resource_damage(){
 let mut w=world();let hp=w.hp;let mp=w.mp;let target_hp=w.actors[&1].hp_ratio;let count=w.own_cast_packets;let effects=w.effects.len();let now=std::time::Instant::now();
 let pending=skills::Pending{since:now,casts:count,messages:w.message_count,generation:w.generation,incarnation:w.actors[&3].incarnation};
 assert_eq!(pending.observe(&w,now),skills::Feedback::Pending);assert!(w.accept(&s39()).unwrap());assert_eq!(w.own_cast_packets,count+1);assert_eq!(pending.observe(&w,now),skills::Feedback::CastAction);
 assert_eq!((w.hp,w.mp,w.actors[&1].hp_ratio),(hp,mp,target_hp));assert_eq!(w.effects.len(),effects);let cue=w.many_attacks.last().unwrap();assert_eq!((cue.attack.caster,cue.attack.action,cue.attack.graphics),(3,19,3934));assert_eq!(cue.attack.position,[32729,32797]);assert_eq!(cue.attack.targets.len(),1);assert_eq!((cue.attack.targets[0].id,cue.attack.targets[0].hit),(1,true));
}
#[test]
fn counter72_malformed_capture_is_atomic_and_unknown_profiles_stay_quarantined(){
 let w=world();let before=serde_json::to_value(&w).unwrap();let packet=s39();
 for end in 0..packet.len(){let mut w=w.clone();assert!(w.accept(&packet[..end]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
 let mut bad=Vec::new();
 for (at,value) in [(10,8),(17,1),(18,1),(26,1)]{let mut b=packet.clone();b[at]=value;bad.push(b);}
 for (at,n) in [(2,4),(11,4),(15,2),(22,4)]{let mut b=packet.clone();b[at..at+n].fill(0);bad.push(b);}
 let mut b=packet.clone();b[20]=2;bad.push(b);let mut b=packet.clone();b.push(0);bad.push(b);
 for b in bad{let mut w=w.clone();assert!(w.accept(&b).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
 for (action,gfx,kind) in [(18,3934u16,0),(19,3934,8),(19,170,0),(18,171,8)]{let mut b=packet.clone();b[1]=action;b[15..17].copy_from_slice(&gfx.to_le_bytes());b[17]=kind;assert!(many::decode(&b).unwrap().is_none());}
 let mut lightning=packet;lightning[1]=18;lightning[15..17].copy_from_slice(&170u16.to_le_bytes());lightning[17]=8;let old=many::decode(&lightning).unwrap().unwrap();assert_eq!((old.action,old.graphics),(18,170));
}
