//! Actual Quest progression and natural Turn Undead inputs; no NPC/clock authority writes.
use super::{native_login::LoginController,native_world_scene::Frame};
use anyhow::{Result,ensure,Context};use serde_json::{json,Value};
use std::{path::{Path,PathBuf},time::{Instant,Duration}};
struct Pending {skill:u16,target:u32,incarnation:u64,before:Value,sent:Instant}
#[derive(Default)]
pub struct Probe {last_progress:Option<Instant>,pending:Option<Pending>,actions:Vec<Value>,frames:Vec<Value>,images:Vec<Value>,pixels:Vec<(PathBuf,Vec<u8>)>,moves:usize,
 success_target:Option<u32>,finished:Option<Instant>,endpoint:Value,sound_captured:bool,last_captured:bool,saw_failure:bool}
impl Probe {
 pub fn receipt(&self)->Value {json!({"actions":self.actions,"frames":self.frames,"images":self.images,"moves_sent":self.moves,"success_target":self.success_target,"saw_failure":self.saw_failure,"endpoint":self.endpoint})}
 fn capture(&mut self,root:&Path,role:&str,label:&str,frame:&Frame)->Result<()> {
  ensure!(self.pixels.len()<8,"TU pixel capture budget");let path=root.join(format!("{role}-turn-undead-{label}.png"));self.pixels.push((path.clone(),frame.canvas.rgba().to_vec()));self.images.push(json!({"png":path,"frame":frame.receipt}));Ok(())
 }
 fn save(&mut self)->Result<()> {
  for (path,bytes) in self.pixels.drain(..){let mut encoder=png::Encoder::new(std::fs::File::create(path)?,800,600);encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.write_header()?.write_image_data(&bytes)?;}Ok(())
 }
 pub fn save_error(&mut self,root:&Path,role:&str)->Result<()> {self.save()?;std::fs::write(root.join(format!("{role}-tu-error.json")),serde_json::to_vec_pretty(&self.receipt())?)?;Ok(())}
 pub fn tick(&mut self,c:&mut LoginController,frame:&Frame,root:&Path,role:&str)->Result<bool> {
  if role=="a"&&self.last_progress.is_none_or(|t|t.elapsed()>Duration::from_secs(1)) {
   let w=&c.snapshot().world;let actors:Vec<_>=w.actors.values().map(|a|json!({"id":a.id,"gfx":a.graphics,"position":[a.x,a.y],"dead":a.dead,"ratio":a.hp_ratio})).collect();
   std::fs::write(root.join("a-tu-progress.json"),serde_json::to_vec(&json!({"hp":w.hp,"mp":w.mp,"own":w.own_actor().map(|a|[a.x,a.y]),"map":w.map_id,"moves":self.moves,"actions":self.actions.len(),"actors":actors}))?)?;self.last_progress=Some(Instant::now());
  }
  let fx=&frame.receipt["skill_effects"];
  if self.pending.as_ref().is_some_and(|p|p.skill==18)||fx["effects"].as_array().is_some_and(|rows|rows.iter().any(|r|r["graphics"]==754)) {
   ensure!(self.frames.len()<2048,"TU frame budget");self.frames.push(json!({"effect":fx,"audio":frame.receipt["audio"],"hp":c.snapshot().world.hp,"mp":c.snapshot().world.mp,"elapsed_ns":frame.receipt["elapsed_ns"]}));
   if fx["effects"].as_array().is_some_and(|rows|rows.iter().any(|r|r["graphics"]==754&&r["drawn"]==true)) && !self.sound_captured {self.capture(root,role,"effect-first",frame)?;self.sound_captured=true;}
   if fx["effects"].as_array().is_some_and(|rows|rows.iter().any(|r|r["graphics"]==754&&r["drawn"]==true&&r["subframe"]==19))&&!self.last_captured {self.capture(root,role,"last-positive-frame",frame)?;self.last_captured=true;}
  }
  if root.join("turn-undead-finished").exists() {
   let done=self.finished.get_or_insert_with(Instant::now);
   if done.elapsed()>Duration::from_millis(500)&&self.endpoint.is_null(){self.save()?;self.endpoint=json!({"world":c.snapshot().world,"skill_feedback":c.snapshot().skill_feedback});std::fs::write(root.join(format!("{role}-tu-endpoint")),b"observed")?;}
   return Ok(!self.endpoint.is_null()&&["a","b"].iter().all(|r|root.join(format!("{r}-tu-endpoint")).exists()));
  }
  if role!="a"{return Ok(false);}
  let w=&c.snapshot().world;ensure!(w.own_actor().is_some_and(|a|!a.dead),"Quest caster died");
  if let Some(pending)=&self.pending {
   if pending.sent.elapsed()<Duration::from_millis(1800){return Ok(false);}
   if w.own_cast_packets==pending.before["casts"].as_u64().unwrap() {
    if pending.sent.elapsed()<Duration::from_secs(3){return Ok(false);}
    ensure!(c.snapshot().skill_feedback==super::native_skills::Feedback::NoResponse,"Quest pending action unresolved");
    self.actions.push(json!({"skill":pending.skill,"target":pending.target,"before":pending.before,"after":{"hp":w.hp,"mp":w.mp,"casts":w.own_cast_packets},"success":false,"failed":false,"no_response":true,"no_prediction":true}));self.pending=None;return Ok(false);
   }
   let successful=pending.skill==18&&w.effects.iter().any(|e|e.graphics==754&&e.target.is_some_and(|t|t.0==pending.target&&t.1==pending.incarnation));
   let failed=pending.skill==18&&c.snapshot().skill_feedback==super::native_skills::Feedback::Rejected(280);
   if pending.skill==18 {ensure!(successful||failed,"TU has neither effect nor failure response");}
   self.saw_failure|=failed;
   self.actions.push(json!({"skill":pending.skill,"target":pending.target,"before":pending.before,"after":{"hp":w.hp,"mp":w.mp,"casts":w.own_cast_packets,"target":w.actors.get(&pending.target)},"success":successful,"failed":failed,"no_prediction":true}));
   if successful {
    ensure!(w.actors.get(&pending.target).is_none_or(|a|a.dead),"TU success did not receive authority death");self.success_target=Some(pending.target);
    ensure!(self.sound_captured&&self.last_captured,"TU first/last positive frame missing");std::fs::write(root.join("turn-undead-finished"),b"formal NPC death and original effect observed")?;
   }
   self.pending=None;return Ok(false);
  }
  if !frame.movement_ready(w){return Ok(false);}
  ensure!(self.actions.len()<512,"Quest spell input budget");
  let own=w.own_actor().context("Quest own actor")?;
  let spar=w.actors.values().filter(|a|a.graphics==145&&!a.dead).min_by_key(|a|own.x.abs_diff(a.x).max(own.y.abs_diff(a.y)));
  let boss=w.actors.values().filter(|a|a.graphics==240&&!a.dead).min_by_key(|a|own.x.abs_diff(a.x).max(own.y.abs_diff(a.y)));
  let target=if let Some(spar)=spar{Some((18,spar))}else if let Some(boss)=boss {Some((if self.actions.iter().any(|a|a["skill"]==18&&a["before"]["target"]["graphics"]==240){0}else{18},boss))}
   else {w.actors.values().filter(|a|!a.dead&&matches!(a.graphics,1649|2158|1082|1047|1020|1578|1517|2535)).min_by_key(|a|own.x.abs_diff(a.x).max(own.y.abs_diff(a.y))).map(|a|(15,a))};
  let Some((skill,target))=target else{return Ok(false);};
  let range=if skill==0{2}else if skill==18{8}else{6};
  let from=[own.x,own.y];let to=[target.x,target.y];
  let needs_sight=skill!=18;
  if own.x.abs_diff(target.x).max(own.y.abs_diff(target.y))>range || (needs_sight&&!frame.navigation.native_line_of_sight(from,to)) {
   let path=if needs_sight {frame.navigation.route_to_attack_range_avoiding(from,to,range,|x,y|super::native_obstacles::blocks(w,x,y))?}else{frame.navigation.route_to_range_avoiding(from,to,range,|x,y|super::native_obstacles::blocks(w,x,y))?};
   ensure!(self.moves<512,"Quest route step budget");c.move_checked(*path.first().context("Quest approach route")?,&frame.navigation)?;self.moves+=1;return Ok(false);
  }
  if skill==0 {return Ok(false);} // Stand near the actual boss and wait for its natural summon.
  ensure!(w.mp.is_some_and(|mp|mp[0]>=20),"Quest caster MP depleted");
  let (id,inc)=(target.id,target.incarnation);let before=json!({"hp":w.hp,"mp":w.mp,"casts":w.own_cast_packets,"target":target});let world_before=serde_json::to_value(w)?;
  c.cast(skill,Some(id),frame)?;ensure!(serde_json::to_value(&c.snapshot().world)?==world_before,"Quest cast predicted authority");
  self.pending=Some(Pending{skill,target:id,incarnation:inc,before,sent:Instant::now()});Ok(false)
 }
}
