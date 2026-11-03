use lito_modern::modern_lito::{native_world::World,native_skills,native_skill_effects::{self,Effects},
    sprite_actions::{SpriteActionCatalog,ActionLimits,ActionKey},resources::{SharedArchive,SharedArchiveKind},
    native_actor_layers::Bank,native_world_effects::Target,native_world_layers,sprite,
    world_map::{WorldCell,PixelPoint,ViewProjection}};
use std::{path::Path,sync::atomic::AtomicBool};
const ROOT:&str=r"D:\lineage開發\0-客戶端";
fn world()->World {
    let d:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=World::default();
    for k in ["world","start","status","actor","bot"] {let s=d["packets"][k].as_str().unwrap();let b:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
    let mut a=w.actors.remove(&0xfedcba98).unwrap();a.id=1;a.x=32729;a.y=32797;w.actors.insert(1,a);w.status.as_mut().unwrap().object_id=1;
    let a=w.actors.get_mut(&2).unwrap();a.x=32730;a.y=32797;w.hp=Some([70,100]);w.mp=Some([24,24]);w.learned_skills.insert(18);w
}

fn effect()->Vec<u8>{let mut b=vec![0x37];b.extend(2u32.to_le_bytes());b.extend(754u16.to_le_bytes());b}
#[test]
fn low_mana_turn_undead_times_out_without_predicted_mana_death_or_rejection() {
 use std::time::{Instant,Duration};
 let mut w=world();w.mp=Some([2,2]);
 let before=serde_json::to_value(&w).unwrap();
 let packet=native_skills::request(&w,18,Some(2)).unwrap();
 assert_eq!(&packet[..3],&[0x14,2,1]);
 let now=Instant::now();let pending=native_skills::Pending{since:now,casts:w.own_cast_packets,messages:w.last_skill_rejection.map_or(0,|v|v.0),generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
 assert_eq!(pending.observe(&w,now+Duration::from_millis(1999)),native_skills::Feedback::Pending);
 assert_eq!(pending.observe(&w,now+Duration::from_secs(2)),native_skills::Feedback::NoResponse);
 assert_eq!(serde_json::to_value(&w).unwrap(),before);
 assert!(lito_modern::modern_lito::native_peer_actions::valid("turn-undead-refused"));
}
#[test]
fn selector_clock_range_and_dead_target_guards_never_predict_death_or_mana() {
 let mut w=world();let before=serde_json::to_value(&w).unwrap();assert_eq!(native_skills::request(&w,18,Some(2)).unwrap(),vec![0x14,2,1,2,0,0,0,0xda,0x7f,0x1d,0x80]);assert_eq!(serde_json::to_value(&w).unwrap(),before);
 assert_eq!(native_skills::client_clock_millis(18),500);assert_eq!(native_skills::reuse_millis(18),500);
 w.actors.get_mut(&2).unwrap().dead=true;assert!(native_skills::request(&w,18,Some(2)).is_err());w.actors.get_mut(&2).unwrap().dead=false;
 w.actors.get_mut(&2).unwrap().x=32740;assert!(native_skills::request(&w,18,Some(2)).is_err());assert!(native_skills::request(&w,18,Some(1)).is_err());
}
#[test]
fn original_754_zero_frames_sound_and_skipped_subframe_are_exact() {
 let raw=SharedArchive::open(Path::new(ROOT),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let mut c=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
 let animations=native_skill_effects::qualify_turn_undead(&c).unwrap();let a=&animations[0].2;assert_eq!(a.frames.len(),21);assert_eq!(a.total_ticks(),33);assert_eq!(a.frames[0].duration_ticks,0);assert_eq!(a.frames[1].duration_ticks,0);assert!(a.frames.iter().all(|f|f.subframe!=3));
 c.sprites.get_mut(&754).unwrap().actions.clear();assert!(native_skill_effects::qualify_turn_undead(&c).is_err());
}
#[path="support/native_fragment_reference.rs"]mod reference;
#[test]
fn original_turn_undead_frames_pixels_sound_once_and_zero_terminal_retirement_after_death() {
 let cancel=AtomicBool::new(false);let mut w=world();let model=reference::full_native_fixture(Path::new(ROOT),u32::from(w.map_id.unwrap()));
 let mut death=vec![0x20];death.extend(2u32.to_le_bytes());death.push(8);w.accept(&death).unwrap();assert!(w.actors[&2].dead);w.accept(&effect()).unwrap();let position=w.effects[0].position;
 let mut bank=Bank::open(Path::new(ROOT)).unwrap();bank.ensure("754-0.spr",&cancel).unwrap();let source=bank.get("754-0.spr").unwrap().source.clone();let kind=if source.archive=="Sprite"{SharedArchiveKind::Sprite}else{SharedArchiveKind::SpriteShard(source.archive.strip_prefix("Sprite").unwrap().parse().unwrap())};
 let original=SharedArchive::open(Path::new(ROOT),kind).unwrap().read_entry(&source.member).unwrap();let raw=sprite::decode_classic_spr_layered(&original,Default::default()).unwrap();
 let mut fx=Effects::default();fx.prepare(Path::new(ROOT),&w,&cancel);
 let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(position[0]),y:i32::from(position[1])},anchor_pixel:PixelPoint{x:160,y:120}};
 let raw_catalog=SharedArchive::open(Path::new(ROOT),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let c=SpriteActionCatalog::parse_for_inspection(&raw_catalog,ActionLimits::default()).unwrap();let animation=c.resolve_action(754,ActionKey::new(0)).unwrap().animation;
 let mut ticks=0u64;let mut sounds=0;let mut rows=Vec::new();
 for source_index in 2..21 {
  let ns=(ticks*1_000_000_000).div_ceil(24);let mut queue=Vec::new();let (r,e)=fx.draw_to(&w,&mut Target::Fragments(&mut queue),view,ns);
  assert!(r["load_error"].is_null());assert_eq!(r["omitted"],0);assert_eq!(queue.len(),1);assert_eq!(r["effects"][0]["source_frame"],source_index);assert_eq!(r["effects"][0]["subframe"],animation.frames[source_index].subframe);
  let actual=native_world_layers::compose_with_effects(model.scene(),view,320,240,&[],&Default::default(),&queue,&cancel).unwrap();let expected=reference::reference_layers(model.scene(),view,&[(0,view.anchor_pixel,&raw,animation.frames[source_index].subframe as usize,2)],320,240,true);assert_eq!(actual.canvas.rgba(),expected.0);assert!(actual.visibility.is_empty());
  for event in e {assert_eq!(event.sound,Some(102));assert_eq!(event.frame,1);sounds+=1;}
  rows.push(r["effects"][0].clone());ticks+=u64::from(animation.frames[source_index].duration_ticks);
  if source_index==2 {w.actors.remove(&2);}
 }
 assert_eq!(sounds,1);assert_eq!(ticks,33);let mut queue=Vec::new();let (r,e)=fx.draw_to(&w,&mut Target::Fragments(&mut queue),view,1_375_000_000);assert_eq!(r["active"],0);assert_eq!(r["effects"][0]["subframe"],20);assert_eq!(r["effects"][0]["drawn"],false);assert!(queue.is_empty()&&e.is_empty());
 assert_eq!(w.hp,Some([70,100]));assert_eq!(w.mp,Some([24,24]));
 let sound=lito_modern::modern_lito::resources::read_shared_sound(Path::new(ROOT),lito_modern::modern_lito::resources::SharedSoundKey{sound_id:102}).unwrap();use sha2::{Digest,Sha256};
 std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/turn-undead-original-sources.json"),serde_json::to_vec_pretty(&serde_json::json!({"sprite":source,"frames":rows,"sound":{"bytes":sound.len(),"sha256":format!("{:X}",Sha256::digest(&sound))}})).unwrap()).unwrap();
}

#[test]
fn original_boss_summon_circle_definition_and_terminal_are_finite() {
 let raw=SharedArchive::open(Path::new(ROOT),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let c=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();let a=native_skill_effects::qualify_summon_circle(&c).unwrap();assert_eq!(a[0].2.total_ticks(),30);
 let mut playback=lito_modern::modern_lito::native_effect_timing::Clip::new(std::sync::Arc::new(a[0].2.clone())).unwrap().start();let first=playback.advance(0).unwrap();assert_eq!(first.crossed_cues[0].event,lito_modern::modern_lito::sprite_actions::FrameEvent::Sound(131));
 let mut now_ms=0u64;
 for source in 1..a[0].2.frames.len() {
  now_ms+=u64::from(a[0].2.frames[source].duration_ticks)*1000/24;
  let next=playback.advance(now_ms*1_000_000).unwrap();assert_eq!(next.source_frame,if source+1==a[0].2.frames.len(){0}else{source+1});assert!(!next.finished);assert!(next.crossed_cues.is_empty());
 }
 assert!(playback.advance((now_ms+u64::from(a[0].2.frames[0].duration_ticks)*1000/24)*1_000_000).unwrap().finished);
}

#[test]
fn actual_quest554_source_spawn_can_route_to_original_ranged_target() {
 let cancel=AtomicBool::new(false);let mut counts=Vec::new();
 for start in [[32801,32885],[32755,32785]] {
  let mut w=world();w.map_id=Some(554);w.actors.retain(|id,_|*id==1);let a=w.actors.get_mut(&1).unwrap();a.x=start[0];a.y=start[1];
  let m=lito_modern::modern_lito::native_world_scene::Model::load(Path::new(ROOT),&w,&cancel).unwrap();let nav=m.navigation();let target=[32751,32836];
  for goal in [[32745,32830],[32774,32865],[32780,32802],[32800,32856]] {println!("source spawn {start:?} to wave {goal:?}: {:?}",nav.route_to_range_avoiding(start,goal,6,|_,_|false).as_ref().map(Vec::len));}
  let adjacent=nav.route_to_adjacent(start,target);let ranged=nav.route_to_range_avoiding(start,target,6,|_,_|false);
  println!("Quest554 start={start:?} mask={} adjacent={:?} ranged={:?}",nav.mask(i32::from(start[0]),i32::from(start[1])),adjacent.as_ref().map(Vec::len),ranged.as_ref().map(Vec::len));
  if let Ok(path)=ranged {let mut cell=start;for h in &path {assert!(nav.can_step(i32::from(cell[0]),i32::from(cell[1]),*h));let (x,y)=lito_modern::modern_lito::native_world::destination(cell[0],cell[1],*h).unwrap();cell=[x,y];}assert!(cell[0].abs_diff(target[0]).max(cell[1].abs_diff(target[1]))<=6);counts.push((start,path.len()));}
 }
 assert!(!counts.is_empty(),"No native-floor ranged route from either formal Quest554 player spawn");
}

#[test]
fn actual_quest_wall_rejection_replans_to_a_reachable_casting_line() {
 let cancel=AtomicBool::new(false);let mut w=world();w.map_id=Some(554);w.actors.retain(|id,_|*id==1);let a=w.actors.get_mut(&1).unwrap();a.x=32746;a.y=32819;
 let m=lito_modern::modern_lito::native_world_scene::Model::load(Path::new(ROOT),&w,&cancel).unwrap();let nav=m.navigation();let from=[32746,32819];let to=[32752,32825];assert!(!nav.native_line_of_sight(from,to));
 let path=nav.route_to_attack_range_avoiding(from,to,6,|_,_|false).unwrap();assert!(!path.is_empty());let mut cell=from;
 for h in &path {assert!(nav.can_step(i32::from(cell[0]),i32::from(cell[1]),*h));let(x,y)=lito_modern::modern_lito::native_world::destination(cell[0],cell[1],*h).unwrap();cell=[x,y];}
 assert!(nav.native_line_of_sight(cell,to));assert!(cell[0].abs_diff(to[0]).max(cell[1].abs_diff(to[1]))<=6);
 println!("actual Quest casting-line route {from:?} -> {to:?}: {} steps ending {cell:?}",path.len());
}
