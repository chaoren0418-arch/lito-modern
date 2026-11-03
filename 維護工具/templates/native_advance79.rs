use lito_modern::modern_lito::{native_skills as skills,native_skill_selection::{Model,Target},native_world::World};
fn world()->World {
 let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=World::default();
 for key in ["world","start","status","actor","bot"] {let hex=f["packets"][key].as_str().unwrap();let bytes:Vec<u8>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();w.accept(&bytes).unwrap();}
 let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=1;own.x=32770;own.y=32818;w.actors.insert(1,own);w.status.as_mut().unwrap().object_id=1;
 let target=w.actors.get_mut(&2).unwrap();target.x=32771;target.y=32818;w.hp=Some([50,100]);w.mp=Some([100,100]);w.learned_skills.insert(79);w
}
#[test]
fn advance79_self_target_wire_and_asymmetric_native_screen() {
 let w=world();assert_eq!(skills::request(&w,79,None).unwrap(),[0x14,9,6,1,0,0,0]);assert_eq!(skills::request(&w,79,Some(2)).unwrap(),[0x14,9,6,2,0,0,0]);assert_eq!(skills::reuse_millis(79),10);
 for (dx,dy,allowed) in [(18,0,true),(-18,0,true),(-19,0,true),(19,0,false),(0,-19,true),(0,19,false),(-18,1,true),(18,-1,false),(18,2,false)] {
  let mut at=w.clone();let a=at.actors.get_mut(&2).unwrap();a.x=(32770+dx)as u16;a.y=(32818+dy)as u16;assert_eq!(skills::request(&at,79,Some(2)).is_ok(),allowed,"{dx},{dy}");
 }
 let mut high=w.clone();let mut a=high.actors.remove(&2).unwrap();a.id=0xfedcba98;high.actors.insert(a.id,a);assert_eq!(&skills::request(&high,79,Some(0xfedcba98)).unwrap()[3..],&[0x98,0xba,0xdc,0xfe]);
}
#[test]
fn advance79_invalid_actor_and_caster_state_fail_without_mutation() {
 let w=world();for id in [0,999]{assert!(skills::request(&w,79,Some(id)).is_err());}
 for kind in 0..9 {let mut bad=w.clone();match kind {
  0=>bad.actors.get_mut(&2).unwrap().dead=true,1=>bad.actors.get_mut(&2).unwrap().draw_state.invisible=true,2=>bad.actors.get_mut(&2).unwrap().draw_state.suppressed=true,
  3=>{bad.learned_skills.remove(&79);},4=>bad.mp=Some([0,100]),5=>bad.hp=Some([0,100]),6=>bad.actors.get_mut(&1).unwrap().dead=true,7=>bad.started=false,_=>{bad.movement_blocks.insert(2);},
 }let before=serde_json::to_value(&bad).unwrap();assert!(skills::request(&bad,79,Some(2)).is_err());assert_eq!(serde_json::to_value(&bad).unwrap(),before);}
 let mut invisible=w.clone();invisible.actors.get_mut(&1).unwrap().draw_state.invisible=true;assert!(skills::request(&invisible,79,None).is_ok());
}
#[test]
fn advance79_learned_menu_rejects_stale_target_and_connection() {
 let mut w=world();let mut menu=Model::default();menu.sync(7,&w,true);menu.select(Some(79)).unwrap();assert_eq!(menu.rows().iter().find(|r|r.id==79).unwrap().label,"#79 靈魂昇華（未選取目標時對自己）");
 assert_eq!(menu.choice(7,&w,None).unwrap().target,None);let target=Target::capture(&w,2).unwrap();assert_eq!(menu.choice(7,&w,Some(target)).unwrap().target,Some(2));
 w.actors.get_mut(&2).unwrap().incarnation+=1;assert!(menu.choice(7,&w,Some(target)).is_err());let target=Target::capture(&w,2).unwrap();assert!(menu.choice(8,&w,Some(target)).is_err());
 w.learned_skills.remove(&79);menu.sync(7,&w,true);assert_eq!(menu.selected(),None);
}
#[test]
fn advance79_only_server_packets_change_current_and_maximum_vitals() {
 use std::time::{Instant,Duration};let mut w=world();w.mp=Some([1,100]);let now=Instant::now();let pending=skills::Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
 let before=serde_json::to_value(&w).unwrap();skills::request(&w,79,None).unwrap();assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(pending.observe(&w,now+Duration::from_secs(2)),skills::Feedback::NoResponse);
 for code in [278,280]{let mut refused=w.clone();refused.last_skill_rejection=Some((pending.messages+1,code));assert_eq!(pending.observe(&refused,now),skills::Feedback::Rejected(code));}
 w.accept(&[0x20,1,0,0,0,19]).unwrap();assert_eq!(pending.observe(&w,now),skills::Feedback::CastAction);assert_eq!(w.hp,Some([50,100]));assert_eq!(w.mp,Some([1,100]));
 w.accept(&[0x0d,50,0,120,0]).unwrap();w.accept(&[0x4d,1,0,120,0]).unwrap();assert_eq!(w.hp,Some([50,120]));assert_eq!(w.mp,Some([1,120]));
 w.accept(&[0x0d,50,0,100,0]).unwrap();w.accept(&[0x4d,1,0,100,0]).unwrap();assert_eq!(w.hp,Some([50,100]));assert_eq!(w.mp,Some([1,100]));
 w.generation+=1;assert_eq!(pending.observe(&w,now),skills::Feedback::Interrupted);
}
fn draw(effects:&mut lito_modern::modern_lito::native_skill_effects::Effects,w:&World,now:u64)->(serde_json::Value,Vec<lito_modern::modern_lito::native_combat_animation::EnteredEvent>) {
 use lito_modern::modern_lito::{native_world_effects::Target as DrawTarget,world_map::{WorldCell,PixelPoint,ViewProjection}};
 let mut fragments=Vec::new();effects.draw_to(w,&mut DrawTarget::Fragments(&mut fragments),ViewProjection{anchor_cell:WorldCell{x:32771,y:32818},anchor_pixel:PixelPoint{x:400,y:300}},now)
}
#[test]
fn advance3935_exact_frames_emit_one_overlap1356_scoped_cue() {
 use lito_modern::modern_lito::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{ActionKey,ActionLimits,SpriteActionCatalog,FrameEvent},native_selector_effects,native_skill_effects::Effects,combat_animation::{NativeAudioMode,NativeAudioGroup},combat_audio::{SoundBank,PlayOutcome,PoolError},native_audio_delivery::{Player,Scope,DeliveryOutcome}};
 use std::{path::Path,sync::{Arc,atomic::AtomicBool}};let root=Path::new(r"D:\lineage開發\0-客戶端");let bytes=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let catalog=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits::default()).unwrap();
 native_selector_effects::qualify(&catalog).unwrap();let a=catalog.resolve_action(3935,ActionKey::new(0)).unwrap().animation;
 assert_eq!(a.frames.len(),19);assert_eq!(a.frames[0].subframe,18);assert_eq!(a.frames[1].events,[FrameEvent::Effect(1356)]);
 assert_eq!(a.frames.iter().map(|f|f.duration_ticks).collect::<Vec<_>>(),[4,2,2,2,2,2,2,2,2,2,2,2,2,2,3,4,4,4,5]);
 let mut w=world();w.accept(&[0x37,2,0,0,0,0x5f,0x0f]).unwrap();let mut e=Effects::default();e.prepare(root,&w,&AtomicBool::new(false));let mut now=0;let mut emitted=Vec::new();
 for index in (1..19).chain(std::iter::once(0)) {let(receipt,events)=draw(&mut e,&w,now);assert!(receipt["load_error"].is_null(),"{receipt}");emitted.extend(events);now+=u64::from(a.frames[index].duration_ticks)*1000/24*1_000_000;}
 let(receipt,events)=draw(&mut e,&w,now);assert_eq!(receipt["active"],0);assert!(events.is_empty());assert_eq!(emitted.len(),1);let event=&emitted[0];assert_eq!((event.actor,event.action,event.frame,event.sound),(2,0,1,Some(1356)));assert_eq!(event.event,"EffectAudioOverlap(1356)");
 let cue=event.prepared_audio.as_ref().unwrap();assert_eq!(cue.group().mode,NativeAudioMode::AllowOverlappingVoices);assert_eq!(cue.group().candidates(),[1356]);assert!(cue.prepared_sound().is_some());
 struct Recorder(usize);impl Player for Recorder {fn play(&mut self,_:&SoundBank,g:&NativeAudioGroup,_:u32)->Result<PlayOutcome,PoolError>{assert_eq!(g.mode,NativeAudioMode::AllowOverlappingVoices);self.0+=1;Ok(PlayOutcome::Started{sound_id:1356,slot:0})}}
 let scope=Scope::new(7,w.generation,Arc::new(()));let mut player=Recorder(0);assert_eq!(cue.submit(&scope,&mut player).unwrap(),DeliveryOutcome::Unbound);cue.bind(scope.clone()).unwrap();assert!(matches!(cue.submit(&scope,&mut player).unwrap(),DeliveryOutcome::Played(_)));assert_eq!(cue.clone().submit(&scope,&mut player).unwrap(),DeliveryOutcome::AlreadyConsumed);assert_eq!(player.0,1);
}
#[test]
fn advance3935_late_poll_does_not_catch_up_or_replay_after_retirement() {
 use lito_modern::modern_lito::{native_skill_effects::Effects,native_audio_delivery::{Scope,Player,DeliveryOutcome},combat_animation::NativeAudioGroup,combat_audio::{SoundBank,PlayOutcome,PoolError}};use std::{path::Path,sync::{Arc,atomic::AtomicBool}};
 let mut w=world();w.accept(&[0x37,2,0,0,0,0x5f,0x0f]).unwrap();let mut e=Effects::default();e.prepare(Path::new(r"D:\lineage開發\0-客戶端"),&w,&AtomicBool::new(false));let(r,first)=draw(&mut e,&w,0);assert!(r["load_error"].is_null(),"{r}");assert_eq!(first.len(),1);
 let cue=first[0].prepared_audio.as_ref().unwrap();cue.bind(Scope::new(7,w.generation,Arc::new(()))).unwrap();struct Never;impl Player for Never{fn play(&mut self,_:&SoundBank,_:&NativeAudioGroup,_:u32)->Result<PlayOutcome,PoolError>{panic!("stale event reached player")}}
 assert_eq!(cue.submit(&Scope::new(8,w.generation,Arc::new(())),&mut Never).unwrap(),DeliveryOutcome::Stale);
 let(r,events)=draw(&mut e,&w,10_000_000_000);assert_eq!(r["active"],1);assert!(events.is_empty());let(_,events)=draw(&mut e,&w,10_000_000_001);assert!(events.is_empty());
 w.clear_preserving_generation();let(r,events)=draw(&mut e,&w,10_000_000_002);assert_eq!(r["active"],0);assert!(events.is_empty());
}
