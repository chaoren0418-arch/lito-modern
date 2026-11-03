use lito_modern::modern_lito::{native_skills as skills,native_skill_selection::{Model,Target},native_world::World};
fn world()->World {
 let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
 let mut w=World::default();
 for key in ["world","start","status","actor","bot"] {
  let hex=f["packets"][key].as_str().unwrap();
  let bytes:Vec<u8>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();w.accept(&bytes).unwrap();
 }
 let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=1;own.x=32770;own.y=32818;w.actors.insert(1,own);w.status.as_mut().unwrap().object_id=1;
 let target=w.actors.get_mut(&2).unwrap();target.x=32778;target.y=32818;
 w.hp=Some([50,50]);w.mp=Some([100,100]);w.learned_skills.insert(29);w
}
#[test]
fn slow29_wire_uses_current_target_coordinates_and_eight_cell_boundary() {
 let w=world();let before=serde_json::to_value(&w).unwrap();
 assert_eq!(skills::request(&w,29,Some(2)).unwrap(),[0x14,3,4,2,0,0,0,0x0a,0x80,0x32,0x80]);
 assert_eq!(skills::reuse_millis(29),500);assert_eq!(serde_json::to_value(&w).unwrap(),before);
 for (dx,dy) in [(-8,-8),(-8,0),(-8,8),(0,-8),(0,8),(8,-8),(8,0),(8,8)] {
  let mut at=w.clone();let a=at.actors.get_mut(&2).unwrap();a.x=(32770+dx)as u16;a.y=(32818+dy)as u16;
  assert!(skills::request(&at,29,Some(2)).is_ok());
 }
 for (dx,dy) in [(-9,0),(9,0),(0,-9),(0,9)] {
  let mut at=w.clone();let a=at.actors.get_mut(&2).unwrap();a.x=(32770+dx)as u16;a.y=(32818+dy)as u16;
  assert!(skills::request(&at,29,Some(2)).is_err());
 }
 let mut high=w.clone();let mut target=high.actors.remove(&2).unwrap();target.id=0xfedcba98;high.actors.insert(target.id,target);
 assert_eq!(&skills::request(&high,29,Some(0xfedcba98)).unwrap()[3..7],&[0x98,0xba,0xdc,0xfe]);
}
#[test]
fn slow29_invalid_target_and_caster_state_cannot_mutate_world() {
 let w=world();for target in [None,Some(0),Some(1),Some(999)]{assert!(skills::request(&w,29,target).is_err());}
 for kind in 0..9 {
  let mut bad=w.clone();match kind {
   0=>bad.actors.get_mut(&2).unwrap().dead=true,
   1=>bad.actors.get_mut(&2).unwrap().draw_state.invisible=true,
   2=>bad.actors.get_mut(&2).unwrap().draw_state.suppressed=true,
   3=>{bad.learned_skills.remove(&29);},4=>bad.mp=Some([0,100]),5=>bad.hp=Some([0,50]),
   6=>bad.actors.get_mut(&1).unwrap().dead=true,7=>bad.started=false,_=>{bad.movement_blocks.insert(2);},
  }
  let before=serde_json::to_value(&bad).unwrap();assert!(skills::request(&bad,29,Some(2)).is_err());assert_eq!(serde_json::to_value(&bad).unwrap(),before);
 }
}
#[test]
fn slow29_selection_is_revoked_by_new_target_life_connection_and_book() {
 let mut w=world();let mut menu=Model::default();menu.sync(7,&w,true);menu.select(Some(29)).unwrap();
 assert_eq!(menu.rows().iter().find(|r|r.id==29).unwrap().label,"#29 緩速術（選取目標，八格內；消耗 1 顆魔法寶石）");
 let target=Target::capture(&w,2).unwrap();assert_eq!(menu.choice(7,&w,Some(target)).unwrap().target,Some(2));
 w.actors.get_mut(&2).unwrap().incarnation+=1;assert!(menu.choice(7,&w,Some(target)).is_err());
 let target=Target::capture(&w,2).unwrap();assert!(menu.choice(8,&w,Some(target)).is_err());
 w.learned_skills.remove(&29);menu.sync(7,&w,true);assert_eq!(menu.selected(),None);
}
#[test]
fn slow29_cost_reagent_probability_and_feedback_remain_server_owned() {
 use std::time::{Duration,Instant};let mut w=world();w.mp=Some([1,100]);let now=Instant::now();
 let pending=skills::Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
 let before=serde_json::to_value(&w).unwrap();assert!(skills::request(&w,29,Some(2)).is_ok());assert_eq!(serde_json::to_value(&w).unwrap(),before);
 assert_eq!(pending.observe(&w,now+Duration::from_secs(2)),skills::Feedback::NoResponse);
 for code in [278,280,299] {let mut refused=w.clone();refused.last_skill_rejection=Some((pending.messages+1,code));assert_eq!(pending.observe(&refused,now),skills::Feedback::Rejected(code));}
 w.accept(&[0x20,1,0,0,0,19]).unwrap();assert_eq!(pending.observe(&w,now),skills::Feedback::CastAction);assert_eq!(w.mp,Some([1,100]));
 let mut retired=w.clone();retired.generation+=1;assert_eq!(pending.observe(&retired,now),skills::Feedback::Interrupted);
}
#[test]
fn slow29_only_server_speed_packets_apply_slow_or_haste_neutralization() {
 use lito_modern::modern_lito::native_speed;
 let mut w=world();let before=w.actors[&2].speed;
 skills::request(&w,29,Some(2)).unwrap();assert_eq!(w.actors[&2].speed,before);
 w.accept(&[0x29,2,0,0,0,2,0,0]).unwrap();assert_eq!(w.actors[&2].speed,2);assert_eq!(native_speed::duration(100,2,0,0).unwrap(),200);
 w.accept(&[0x29,2,0,0,0,0,0,0]).unwrap();assert_eq!(w.actors[&2].speed,0);
 w.accept(&[0x29,1,0,0,0,2,64,0]).unwrap();assert_eq!(w.actors[&1].speed_seconds,64);
 w.accept(&[0x29,1,0,0,0,0,0,0]).unwrap();assert_eq!(w.actors[&1].speed_seconds,0);
 w.clear_preserving_generation();assert!(w.actors.is_empty());
}
#[test]
fn slow752_uses_both_original_frame_cycles_then_retires_without_invented_audio() {
 use lito_modern::modern_lito::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{ActionKey,ActionLimits,SpriteActionCatalog},
  native_skill_effects::Effects,native_world_effects::Target as DrawTarget,world_map::{WorldCell,PixelPoint,ViewProjection}};
 use std::{path::Path,sync::atomic::AtomicBool};let root=Path::new(r"D:\lineage開發\0-客戶端");let cancel=AtomicBool::new(false);
 let bytes=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
 let catalog=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits::default()).unwrap();let a=catalog.resolve_action(752,ActionKey::new(0)).unwrap().animation;
 assert_eq!(a.frames.iter().map(|f|(f.subframe,f.duration_ticks)).collect::<Vec<_>>(),[7,0,1,2,3,4,5,6,7,0,1,2,3,4,5,6].into_iter().map(|n|(n,3)).collect::<Vec<_>>());
 assert!(a.frames.iter().all(|f|f.events.is_empty()));let mut w=world();w.accept(&[0x37,2,0,0,0,0xf0,2]).unwrap();
 let view=ViewProjection{anchor_cell:WorldCell{x:32778,y:32818},anchor_pixel:PixelPoint{x:400,y:300}};let mut effects=Effects::default();effects.prepare(root,&w,&cancel);
 for step in 0..=16 {let mut fragments=Vec::new();let(receipt,events)=effects.draw_to(&w,&mut DrawTarget::Fragments(&mut fragments),view,step*125_000_000);
  assert!(receipt["load_error"].is_null(),"{receipt}");assert!(events.is_empty());
  if step==16{assert_eq!(receipt["active"],0);assert!(fragments.is_empty());}else{assert_eq!(receipt["active"],1);assert!(!fragments.is_empty());}}
 let mut live=Effects::default();live.prepare(root,&w,&cancel);let mut fragments=Vec::new();live.draw_to(&w,&mut DrawTarget::Fragments(&mut fragments),view,0);
 w.clear_preserving_generation();let mut fragments=Vec::new();let(receipt,events)=live.draw_to(&w,&mut DrawTarget::Fragments(&mut fragments),view,1);
 assert_eq!(receipt["active"],0);assert!(fragments.is_empty()&&events.is_empty());
}
