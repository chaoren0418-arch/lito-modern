
/// Canonical server S0B status objects. The caller's world/source owner fences
/// this metadata; no clock, PCM or sprite owner crosses a source retirement.
mod status4183 {
 use crate::modern_lito::{native_world::{World,Actor},sprite_actions::{SpriteActionCatalog,ActionKey,Animation,Directionality,FrameEvent},tower_assets::{SharedEffectBank,HonorAssetError},shared_action_catalog::SharedActionCatalog,native_combat_animation::EnteredEvent,native_audio_delivery::{PreparedCue,FiniteEventKey},native_world_effects::{Target,Mix},world_map::{WorldCell,ViewProjection}};
 use anyhow::{Result,ensure,Context};use std::{collections::{BTreeMap,BTreeSet},path::Path,sync::atomic::AtomicBool};
 use serde_json::{Value,json};
 pub(super) fn is_object(w:&World,a:&Actor)->bool {a.graphics==4183&&w.status.is_none_or(|s|s.object_id!=a.id)&&a.name.is_empty()&&a.title.is_empty()}
 fn canonical(a:&Actor)->bool {a.incarnation!=0&&a.walk_action==0&&a.heading==0&&a.speed==0&&a.brave==0&&a.light==0&&!a.dead&&a.motion_from.is_none()&&a.status==0&&a.actions.is_empty()&&a.action_sequence==0&&!a.position_predicted}
 pub(super) fn live_sequences(w:&World)->BTreeSet<u64>{w.actors.values().filter(|a|is_object(w,a)&&canonical(a)).map(|a|a.incarnation).collect()}
 pub(super) fn audio_current(w:&World,key:FiniteEventKey)->bool {key.graphics!=4183||w.actors.values().any(|a|is_object(w,a)&&canonical(a)&&a.incarnation==key.sequence)}
 pub(super) fn prune_audio(w:&World,cues:&mut Vec<PreparedCue>)->usize {let before=cues.len();cues.retain(|cue|{let keep=cue.finite_event().is_none_or(|key|audio_current(w,key));if !keep{cue.discard();}keep});before-cues.len()}
 pub(super) fn qualify(c:&SpriteActionCatalog)->std::result::Result<Vec<(u32,u8,Animation)>,HonorAssetError>{
  let row=c.sprites.get(&4183).ok_or(HonorAssetError::Unsupported("status4183 absent"))?;let resolved=c.resolve_action(4183,ActionKey::new(3))?;let a=resolved.animation;let attrs:[(u32,&[u8]);2]=[(102,b"0"),(104,b"8")];
  if row.id!=4183||row.graphics_id!=4183||resolved.definition_sprite_id!=4183||row.header_issue.is_some()||row.image_count!=Some(1)||row.actions.len()!=1||row.attributes.len()!=2
   ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
   ||a.frames_per_second!=24||a.directionality!=Directionality::Undirected||a.frames.len()!=6
   ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{5}else{n as u32-1}||f.duration_ticks!=4||f.events!=if n==1{vec![FrameEvent::Effect(1453)]}else{vec![]}) {return Err(HonorAssetError::Unsupported("status4183 original action3/Screen/overlap changed"));}
  Ok(vec![(4183,3,a.clone())])
 }
 #[derive(Clone,Debug)]struct Clock {frame:usize,due:u64,ideal:Option<u64>,last:u64,loops:u64,callbacks:u64}
 impl Clock {
  fn new(now:u64)->Result<Self>{Ok(Self{frame:1,due:now.checked_add(584).context("status warmup overflow")?,ideal:None,last:now,loops:0,callbacks:0})}
  /// 4E5B50 increments before entry. 4E34C0 retains the ideal deadline,
  /// clamps overdue remaining delay to1 and adds2; frame0 returns to the
  /// idle queue for its166ms dwell. One delivered callback per model poll.
  fn advance(&mut self,now:u64)->Result<bool>{
   ensure!(now>=self.last,"status callback clock regressed");self.last=now;if now<self.due{return Ok(false);}
   self.callbacks=self.callbacks.checked_add(1).context("status callback overflow")?;
   if self.ideal.is_none(){self.frame=1;self.loops=self.loops.checked_add(1).context("status loop overflow")?;self.due=now.checked_add(168).context("status deadline overflow")?;self.ideal=Some(self.due);return Ok(true);}
   self.frame=(self.frame+1)%6;
   if self.frame==0{self.ideal=None;self.due=now.checked_add(166).context("status zero-frame deadline overflow")?;return Ok(false);}
   let ideal=self.ideal.unwrap();let remaining=166u64.saturating_sub(now.saturating_sub(ideal)).max(1);self.due=now.checked_add(remaining+2).context("status callback deadline overflow")?;self.ideal=Some(ideal.checked_add(168).context("status ideal deadline overflow")?);Ok(false)
  }
 }
 #[derive(Clone)]struct Live {incarnation:u64,position:[u16;2],clock:Clock}
 #[derive(Clone,Default)]pub(super) struct History {generation:Option<u64>,seen:BTreeSet<(u32,u64)>,live:BTreeMap<u32,Live>,omitted:u64,audio_omitted:u64}
 impl History {
  pub(super) fn active_sequences(&self)->BTreeSet<u64>{self.live.values().map(|s|s.incarnation).collect()}
  pub(super) fn for_source(&self,same:bool)->Self{let mut h=self.clone();if !same{h.live.clear();}h}
  pub(super) fn merge_admission(&mut self,other:&Self){if self.generation==other.generation{self.seen.extend(other.seen.iter().copied());}}
 }
 struct Occurrence {incarnation:u64,loop_index:u64}
 #[derive(Default)]pub(super) struct Stage {history:History,bank:Option<SharedEffectBank>,error:Option<String>,events:Vec<EnteredEvent>}
 impl Stage {
  pub(super) fn history(&self)->History{self.history.clone()}
  pub(super) fn restore_history(&mut self,h:History){self.history=h;self.events.clear();}
  pub(super) fn prepare(&mut self,root:&Path,w:&World,cancel:&AtomicBool,shared:&mut Option<SharedActionCatalog>){
   if self.bank.is_some()||self.error.is_some()||!w.actors.values().any(|a|is_object(w,a)){return;}
   let loaded=(||->Result<_>{let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;Ok(SharedEffectBank::load_directional(&c,cancel,qualify)?)})();
   match loaded{Ok(bank)=>self.bank=Some(bank),Err(error)=>self.error=Some(format!("{error:#}"))}
  }
  fn step(&mut self,w:&World,now:u64)->Result<Vec<Occurrence>>{
   if self.history.generation!=Some(w.generation){self.history=History{generation:Some(w.generation),..Default::default()};}
   let current:BTreeSet<_>=w.actors.values().filter(|a|is_object(w,a)).map(|a|(a.id,a.incarnation)).collect();self.history.seen.retain(|key|current.contains(key));self.history.live.retain(|id,s|w.actors.get(id).is_some_and(|a|is_object(w,a)&&canonical(a)&&a.incarnation==s.incarnation));
   if self.error.is_some(){self.history.omitted=self.history.omitted.saturating_add(self.history.live.len()as u64);self.history.live.clear();for a in w.actors.values().filter(|a|is_object(w,a)){if self.history.seen.insert((a.id,a.incarnation)){self.history.omitted+=1;}}return Ok(vec![]);}
   let mut occurrences=vec![];
   for a in w.actors.values().filter(|a|is_object(w,a)){
    if self.history.seen.insert((a.id,a.incarnation)){
     if !canonical(a){self.history.omitted+=1;continue;}
     ensure!(self.history.live.len()<2048,"status object metadata budget");self.history.live.insert(a.id,Live{incarnation:a.incarnation,position:[a.x,a.y],clock:Clock::new(now)?});
    }
    if let Some(live)=self.history.live.get_mut(&a.id){live.position=[a.x,a.y];if live.clock.advance(now)?{if occurrences.len()<32{occurrences.push(Occurrence{incarnation:live.incarnation,loop_index:live.clock.loops});}else{self.history.audio_omitted+=1;}}}
   }
   Ok(occurrences)
  }
  pub(super) fn update(&mut self,w:&World,now_ns:u64)->Result<()>{
   self.events.clear();let occurrences=self.step(w,now_ns/1_000_000)?;
   for occurrence in occurrences{
    let prepared=(||->Result<_>{let bank=self.bank.as_ref().context("status4183 bank absent")?;let animation=bank.animation(4183,3)?;let classified=crate::modern_lito::combat_animation::classify_native_cues(&animation.frames[1].events)?;ensure!(classified.non_audio.is_empty(),"status non-audio event changed");let group=classified.audio.context("status overlap absent")?;ensure!(group.candidates()==[1453],"status audio group changed");Ok(PreparedCue::from_callback_event(bank.sound_bank(),group,0,FiniteEventKey{sequence:occurrence.incarnation,graphics:4183,action:3,source_frame:1},occurrence.loop_index)?)})();
    match prepared{Ok(payload)=>self.events.push(EnteredEvent{actor:0,sequence:occurrence.incarnation,action:3,frame:1,event:"StatusObjectOverlap(1453)".into(),sound:Some(1453),prepared_audio:Some(payload)}),Err(error)=>{self.error=Some(format!("{error:#}"));self.history.omitted+=1;self.history.live.clear();break;}}
   }
   Ok(())
  }
  pub(super) fn draw_to<'s>(&'s self,target:&mut Target<'_,'s>,view:ViewProjection)->(Value,Vec<EnteredEvent>){
   let mut rows=vec![];
   for(id,live)in &self.history.live{
    let position=WorldCell{x:i32::from(live.position[0]),y:i32::from(live.position[1])};let drawn=(||->Result<bool>{let pixel=view.cell_to_pixel(position).context("status projection")?;if pixel.x.abs()>=1000||pixel.y.abs()>=800{return Ok(false);}let bank=self.bank.as_ref().context("status bank unavailable")?;let sample=bank.frame(4183,3,live.clock.frame,false)?;target.submit(sample.frame,position,view,Mix::Screen)?;Ok(true)})();
    rows.push(json!({"id":id,"incarnation":live.incarnation,"position":live.position,"source_frame":if live.clock.loops==0{None}else{Some(live.clock.frame)},"sprite_subframe":if live.clock.frame==0{5}else{live.clock.frame-1},"initial_static_inferred":live.clock.loops==0,"next_callback_ms":live.clock.due,"ideal_deadline_ms":live.clock.ideal,"loops":live.clock.loops,"callbacks":live.clock.callbacks,"drawn":drawn.as_ref().is_ok_and(|v|*v),"error":drawn.err().map(|e|e.to_string())}));
   }
   (json!({"objects":rows,"active":self.history.live.len(),"omitted":self.history.omitted,"audio_omitted":self.history.audio_omitted,"load_error":self.error,"local_expiry":false,"original_clock_execution_qualified":false,"global_interleaving_qualified":false}),self.events.clone())
  }
 }

#[cfg(test)]mod status4183_development_tests {
 use super::*;
 const ID:u32=0x71234567;const SECOND:u32=0x72345678;
 const RAW:&[u8]=include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-freezing80-shock87-inspection\20261024-172131-694095\4183.definition.txt");
 fn catalog(raw:&[u8])->crate::modern_lito::sprite_actions::SpriteActionCatalog{crate::modern_lito::sprite_actions::SpriteActionCatalog::parse_for_inspection(&[b"11297 0 0\n".as_slice(),raw].concat(),crate::modern_lito::sprite_actions::ActionLimits::default()).unwrap()}
 fn world()->World{let mut w=crate::modern_lito::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}w}
 fn put(w:&World,id:u32)->Vec<u8>{let a=w.own_actor().unwrap();let mut b=vec![11];b.extend(a.x.to_le_bytes());b.extend(a.y.to_le_bytes());b.extend(id.to_le_bytes());b.extend(4183_u16.to_le_bytes());b.extend([0;4]);b.extend(0_u32.to_le_bytes());b.extend(0_u16.to_le_bytes());b.extend([0,0,0]);b.extend(0_u32.to_le_bytes());b.extend([0,0,0,255,0,0,0]);b.extend(0_u16.to_le_bytes());b.push(0);b}
 fn remove(w:&mut World,id:u32){let mut b=vec![8];b.extend(id.to_le_bytes());w.accept(&b).unwrap();}
 fn key(w:&World)->crate::modern_lito::native_audio_delivery::FiniteEventKey{crate::modern_lito::native_audio_delivery::FiniteEventKey{sequence:w.actors[&ID].incarnation,graphics:4183,action:3,source_frame:1}}
 #[test]fn original_action3_is_undirected_Screen_and_overlap1453_at_source1(){let a=qualify(&catalog(RAW)).unwrap().remove(0).2;assert_eq!(a.frames.len(),6);assert_eq!(a.frames[0].subframe,5);assert_eq!(a.frames[1].events,vec![crate::modern_lito::sprite_actions::FrameEvent::Effect(1453)]);let group=crate::modern_lito::combat_animation::classify_native_cues(&a.frames[1].events).unwrap().audio.unwrap();assert_eq!(group.mode,crate::modern_lito::combat_animation::NativeAudioMode::AllowOverlappingVoices);}
 #[test]fn original_definition_rejects_replacement_sound_duration_blend_and_extra_action(){let s=String::from_utf8(RAW.to_vec()).unwrap();for(a,b)in[("<1453","[1453"),("104.attr(8)","104.attr(128)"),("0.4:4","0.4:5"),("3.animation","0.animation")]{assert!(s.contains(a));assert!(qualify(&catalog(s.replace(a,b).as_bytes())).is_err());}}
 #[test]fn idle_warmup584_then_source1_to5_zero_and1006ms_period(){let mut c=Clock::new(0).unwrap();assert!(!c.advance(583).unwrap());assert_eq!(c.loops,0);assert!(c.advance(584).unwrap());assert_eq!((c.frame,c.due,c.loops),(1,752,1));for(at,frame,due)in[(752,2,920),(920,3,1088),(1088,4,1256),(1256,5,1424),(1424,0,1590)]{assert!(!c.advance(at).unwrap());assert_eq!((c.frame,c.due),(frame,due));}assert!(c.advance(1590).unwrap());assert_eq!((c.frame,c.loops,c.due),(1,2,1758));}
 #[test]fn late_callback_clamps_remaining_to_three_ms_without_same_poll_catchup(){let mut c=Clock::new(0).unwrap();c.advance(584).unwrap();assert!(!c.advance(5000).unwrap());assert_eq!((c.frame,c.due,c.ideal),(2,5003,Some(920)));assert!(!c.advance(5000).unwrap());assert_eq!(c.frame,2);for(at,frame)in[(5003,3),(5006,4),(5009,5),(5012,0)]{assert!(!c.advance(at).unwrap());assert_eq!(c.frame,frame);}assert_eq!(c.due,5178);assert!(c.advance(5178).unwrap());}
 #[test]fn clock_reversal_and_nonwrapping_deadline_overflow_are_explicit_errors(){assert!(Clock::new(u64::MAX).is_err());let mut c=Clock::new(0).unwrap();c.advance(584).unwrap();let before=(c.frame,c.callbacks,c.due);assert!(c.advance(583).is_err());assert_eq!((c.frame,c.callbacks,c.due),before);}
 #[test]fn exact_server_object_creates_only_presentation_without_hp_MP_or_stun(){let mut w=world();let before=(w.hp,w.mp,w.movement_blocks.clone());let b=put(&w,ID);assert_eq!(b.len(),38);w.accept(&b).unwrap();let mut s=Stage::default();assert!(s.step(&w,0).unwrap().is_empty());assert!(s.step(&w,583).unwrap().is_empty());let events=s.step(&w,584).unwrap();assert_eq!(events.len(),1);assert_eq!(events[0].incarnation,w.actors[&ID].incarnation);assert_eq!((w.hp,w.mp,w.movement_blocks.clone()),before);}
 #[test]fn S25_control_is_independent_and_S08_removes_without_curing_other_controls(){let mut w=world();w.accept(&put(&w,ID)).unwrap();let mut s=Stage::default();s.step(&w,0).unwrap();w.accept(&[37,24]).unwrap();assert!(w.movement_blocks.contains(&24));remove(&mut w,ID);s.step(&w,1000).unwrap();assert!(s.history().active_sequences().is_empty());assert!(w.movement_blocks.contains(&24));w.accept(&[37,25]).unwrap();assert!(!w.movement_blocks.contains(&24));}
 #[test]fn same_ID_S0B_restarts_new_incarnation_and_old_audio_lease_is_invalid(){let mut w=world();w.accept(&put(&w,ID)).unwrap();let old=key(&w);let mut s=Stage::default();s.step(&w,0).unwrap();s.step(&w,584).unwrap();w.accept(&put(&w,ID)).unwrap();assert_ne!(old.sequence,key(&w).sequence);assert!(!audio_current(&w,old));assert!(s.step(&w,600).unwrap().is_empty());assert_eq!(s.history.live[&ID].clock.due,1184);assert_eq!(s.step(&w,1184).unwrap().len(),1);}
 #[test]fn identical_object_history_preserves_cursor_and_does_not_repeat_current_callback(){let mut w=world();w.accept(&put(&w,ID)).unwrap();let mut s=Stage::default();s.step(&w,0).unwrap();s.step(&w,584).unwrap();let mut restored=Stage::default();restored.restore_history(s.history());assert!(restored.step(&w,584).unwrap().is_empty());assert_eq!(restored.history.live[&ID].clock.due,752);assert!(restored.step(&w,752).unwrap().is_empty());assert_eq!(restored.history.live[&ID].clock.frame,2);}
 #[test]fn source_A_B_A_keeps_admission_and_never_resurrects_retired_object_clock(){let mut w=world();w.accept(&put(&w,ID)).unwrap();let mut a=Stage::default();a.step(&w,0).unwrap();a.step(&w,584).unwrap();let h=a.history().for_source(false).for_source(true);let mut b=Stage::default();b.restore_history(h);assert!(b.step(&w,800).unwrap().is_empty());assert!(b.history.live.is_empty());w.accept(&put(&w,ID)).unwrap();b.step(&w,900).unwrap();assert_eq!(b.history.live[&ID].clock.due,1484);}
 #[test]fn late_worker_merges_admission_without_copying_a_retired_clock_or_pending_sound(){let mut w=world();w.accept(&put(&w,ID)).unwrap();let mut worker=Stage::default();worker.step(&w,0).unwrap();worker.step(&w,584).unwrap();let mut retired=worker.history().for_source(false);retired.merge_admission(&worker.history());assert!(retired.live.is_empty());let mut resumed=Stage::default();resumed.restore_history(retired);assert!(resumed.step(&w,1600).unwrap().is_empty());assert!(resumed.history.live.is_empty());}
 #[test]fn original_status_seconds_never_delete_a_live_object_without_S08(){let mut w=world();w.accept(&put(&w,ID)).unwrap();let mut s=Stage::default();s.step(&w,0).unwrap();for now in [584,752,920,1088,1256,1424,1590,20_000,20_003,20_006]{s.step(&w,now).unwrap();}assert_eq!(s.history.live.len(),1);assert!(w.actors.contains_key(&ID));remove(&mut w,ID);assert!(s.step(&w,20_010).unwrap().is_empty());assert!(s.history.live.is_empty());}
 #[test]fn unsupported_stance_or_speed_cannot_fall_back_to_generic_actor_graphics(){for n in 0..3{let mut w=world();w.accept(&put(&w,ID)).unwrap();let a=w.actors.get_mut(&ID).unwrap();match n{0=>a.walk_action=4,1=>a.speed=1,_=>a.dead=true};let mut s=Stage::default();s.step(&w,0).unwrap();assert!(s.history.live.is_empty());assert_eq!(s.history.omitted,1);s.step(&w,1000).unwrap();assert_eq!(s.history.omitted,1);}}
 #[test]fn failed_original_reprepare_retires_once_and_keeps_admission(){let mut w=world();w.accept(&put(&w,ID)).unwrap();let mut s=Stage::default();s.step(&w,0).unwrap();let mut restored=Stage::default();restored.restore_history(s.history());restored.error=Some("cancelled resource preparation".into());restored.step(&w,584).unwrap();assert!(restored.history.live.is_empty());assert_eq!(restored.history.omitted,1);restored.step(&w,1000).unwrap();assert_eq!(restored.history.omitted,1);}
 #[test]fn outer_scene_history_keeps_status_audio_keys_and_source_retirement(){let mut w=world();w.accept(&put(&w,ID)).unwrap();let mut s=Stage::default();s.step(&w,0).unwrap();let mut h=crate::modern_lito::native_world_scene::finite_scene_history_development_tests::history(100);h.status=s.history();assert!(h.active_spell_sequences().contains(&w.actors[&ID].incarnation));let mut retired=h.for_source(false);retired.merge_disintegrate_admission(&h);assert!(retired.status.live.is_empty());assert!(retired.for_source(true).status.live.is_empty());}
 #[test]
 #[ignore="LITO_SHARED_RESOURCE_ROOT; original4183/1453 Screen fragments, independent loop latches and S08; no playback"]
 fn original_status_bank_draws_loop_and_prepares_overlap_per_live_incarnation(){let root:std::path::PathBuf=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root").into();let cancel=AtomicBool::new(false);let mut w=world();w.accept(&put(&w,ID)).unwrap();w.accept(&put(&w,SECOND)).unwrap();let before=(w.hp,w.mp,w.movement_blocks.clone());let a=w.own_actor().unwrap();let view=crate::modern_lito::world_map::ViewProjection{anchor_cell:WorldCell{x:i32::from(a.x),y:i32::from(a.y)},anchor_pixel:crate::modern_lito::world_map::PixelPoint{x:400,y:300}};let mut s=Stage::default();s.prepare(&root,&w,&cancel,&mut None);assert!(s.error.is_none());s.update(&w,0).unwrap();let mut fragments=vec![];let(_,audio)=s.draw_to(&mut crate::modern_lito::native_world_effects::Target::Fragments(&mut fragments),view);assert_eq!(fragments.len(),2);assert!(fragments.iter().all(|f|matches!(f.mix,crate::modern_lito::native_world_effects::Mix::Screen)));assert!(audio.is_empty());drop(fragments);s.update(&w,584_000_000).unwrap();let mut fragments=vec![];let(_,mut audio)=s.draw_to(&mut crate::modern_lito::native_world_effects::Target::Fragments(&mut fragments),view);assert_eq!(audio.len(),2);let first=audio[0].prepared_audio.as_ref().unwrap().clone();let same=first.clone();assert!(first.discard());assert!(!same.discard());assert!(audio[1].prepared_audio.as_ref().unwrap().discard());drop(fragments);let mut cues=audio.drain(..).map(|e|e.prepared_audio.unwrap()).collect();remove(&mut w,ID);remove(&mut w,SECOND);assert_eq!(crate::modern_lito::native_world_scene::prune_status4183_audio(&w,&mut cues),2);s.update(&w,752_000_000).unwrap();assert!(s.history.live.is_empty());assert_eq!((w.hp,w.mp,w.movement_blocks.clone()),before);}
}

}

pub(crate) fn prune_status4183_audio(world:&World,cues:&mut Vec<super::native_audio_delivery::PreparedCue>)->usize {status4183::prune_audio(world,cues)}
pub(crate) fn status4183_live_sequences(world:&World)->std::collections::BTreeSet<u64>{status4183::live_sequences(world)}
