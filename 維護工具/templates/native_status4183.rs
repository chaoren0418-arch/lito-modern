
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
}

pub(crate) fn prune_status4183_audio(world:&World,cues:&mut Vec<super::native_audio_delivery::PreparedCue>)->usize {status4183::prune_audio(world,cues)}
pub(crate) fn status4183_live_sequences(world:&World)->std::collections::BTreeSet<u64>{status4183::live_sequences(world)}
