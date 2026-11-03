//! Actual poison/cure inputs only. Barrier files never alter game authority.
use super::{native_login::LoginController, native_skills::Feedback, native_world_scene::Frame};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::{Path, PathBuf}, time::{Duration, Instant}};

#[derive(Default)]
pub struct Probe {
    phase: u8,
    episode: u8,
    green: bool,
    clear_since: Option<Instant>,
    clear_hp: u16,
    clear_min_hp: u16,
    episode_hp: u16,
    sent: Option<Instant>,
    before: Value,
    skill: u16,
    attempts: usize,
    moves: usize,
    since: Option<Instant>,
    phase_since: Option<Instant>,
    done_since: Option<Instant>,
    previous: Value,
    frames: Vec<Value>,
    states: Vec<Value>,
    casts: Vec<Value>,
    images: Vec<Value>,
    pending_images: Vec<(PathBuf, Vec<u8>)>,
    captured: BTreeSet<(u64, bool)>,
    endpoint: Value,
}
impl Probe {
    fn capture(&mut self,root:&Path,role:&str,label:&str,frame:&Frame)->Result<()> {
        ensure!(self.images.len()<32,"poison image memory bound");
        let path=root.join(format!("{role}-round1-{label}.png"));
        self.images.push(json!({"png":path,"frame":frame.receipt}));
        self.pending_images.push((path,frame.canvas.rgba().to_vec()));
        Ok(())
    }
    /// Encode after the timed scenario (or a terminal error), so PNG I/O
    /// cannot skip the short 871 terminal dwell in the render sampling loop.
    pub fn save_images(&mut self)->Result<()> {
        for (path,pixels) in self.pending_images.drain(..) {
            let mut encoder=png::Encoder::new(std::fs::File::create(path)?,800,600);
            encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(&pixels)?;
        }
        Ok(())
    }
    fn state(c:&LoginController)->Value {
        let s=c.snapshot();json!({"world":s.world,"inventory":s.inventory,"skill_feedback":s.skill_feedback})
    }
    pub fn receipt(&self)->Value {
        json!({"casts":self.casts,"frames":self.frames,"states":self.states,"images":self.images,
            "endpoint":self.endpoint,"moves_sent":self.moves,"attempts":self.attempts,
            "episodes_observed":self.episode,"runtime_authority_mutations":0})
    }
    fn send(&mut self,c:&mut LoginController,frame:&Frame,skill:u16)->Result<()> {
        self.before=Self::state(c);self.skill=skill;
        c.cast(skill,Some(1),frame)?;
        ensure!(c.cast(skill,Some(1),frame).is_err(),"duplicate poison/cure pending admitted");
        ensure!(self.before["world"]==serde_json::to_value(&c.snapshot().world)?
            && self.before["inventory"]==serde_json::to_value(&c.snapshot().inventory)?,"predicted poison/cure authority");
        self.sent=Some(Instant::now());if skill==11 {self.attempts+=1;}
        Ok(())
    }
    pub fn tick(&mut self,c:&mut LoginController,frame:&Frame,root:&Path,role:&str,profile:&str)->Result<bool> {
        let start=*self.since.get_or_insert_with(Instant::now);
        ensure!(start.elapsed()<Duration::from_secs(180),"poison scenario timeout phase={} state={}",self.phase,Self::state(c));
        let w=&c.snapshot().world;let target=w.actors.get(&1).context("poison target")?;
        let green=target.draw_state.green;
        let observation=json!({"ms":start.elapsed().as_millis(),"hp":w.hp,"mp":w.mp,"green":green,
            "own_cast_packets":w.own_cast_packets,"feedback":c.snapshot().skill_feedback,"inventory":c.snapshot().inventory});
        let changed=self.previous["hp"]!=observation["hp"] || self.previous["mp"]!=observation["mp"]
            || self.previous["green"]!=observation["green"] || self.previous["own_cast_packets"]!=observation["own_cast_packets"]
            || self.previous["feedback"]!=observation["feedback"] || self.previous["inventory"]!=observation["inventory"];
        if changed {ensure!(self.states.len()<1024,"poison state bound");self.states.push(observation.clone());self.previous=observation;}
        let fx=&frame.receipt["skill_effects"];
        ensure!(fx["load_error"].is_null() && fx["omitted"]==0,"poison resources failed: {fx}");
        if changed || fx["active"].as_u64().is_some_and(|n|n>0) || fx["entered_events"].as_array().is_some_and(|v|!v.is_empty()) {
            ensure!(self.frames.len()<2048,"poison frame bound");
            self.frames.push(json!({"ms":start.elapsed().as_millis(),"hp":w.hp,"mp":w.mp,"green":green,"effect":fx,"audio":frame.receipt["audio"]}));
        }
        if let Some(rows)=fx["effects"].as_array() {
            for row in rows {
                if row["drawn"]!=true || !matches!(row["graphics"].as_u64(),Some(745|871)){continue;}
                let sequence=row["sequence"].as_u64().unwrap();let terminal=row["source_frame"]==0;
                if self.captured.insert((sequence,terminal)) {
                    self.capture(root,role,&format!("poison-fx-{sequence}-{}",if terminal{"terminal"}else{"initial"}),frame)?;
                }
            }
        }
        if green!=self.green {
            self.green=green;
            if green {
                self.episode+=1;self.episode_hp=w.hp.unwrap()[0];self.clear_since=None;
            } else {
                self.clear_since=Some(Instant::now());self.clear_hp=w.hp.unwrap()[0];self.clear_min_hp=self.clear_hp;
            }
            self.capture(root,role,&format!("poison-{}-{}",if green{"green"}else{"clear"},self.episode),frame)?;
        }
        if role=="a" {
            if green && w.hp.unwrap()[0]+5<=self.episode_hp {
                let path=root.join(format!("a-poison-tick-{}",self.episode));
                if !path.exists(){std::fs::write(path,serde_json::to_vec(&Self::state(c))?)?;}
            }
            if let Some(clear)=self.clear_since {
                self.clear_min_hp=self.clear_min_hp.min(w.hp.unwrap()[0]);
                ensure!(self.clear_min_hp>=self.clear_hp,"poison damage continued after authoritative clear");
                if clear.elapsed()>Duration::from_millis(6500) {
                    let path=root.join(format!("a-poison-clean-{}",self.episode));
                    if !path.exists(){std::fs::write(path,serde_json::to_vec(&Self::state(c))?)?;}
                }
            }
        }
        if root.join("poison-finished").exists() {
            let done=*self.done_since.get_or_insert_with(Instant::now);
            if done.elapsed()>Duration::from_millis(500) && self.endpoint.is_null() {
                self.save_images()?;
                self.endpoint=Self::state(c);std::fs::write(root.join(format!("{role}-poison-endpoint")),b"received")?;
            }
            let other=if role=="a"{"b"}else{"a"};
            return Ok(!self.endpoint.is_null() && root.join(format!("{other}-poison-endpoint")).exists());
        }
        if role=="a" {
            if self.episode==0 && frame.movement_ready(w) {
                let own=w.own_actor().unwrap();let other=&w.actors[&3];
                if own.x.abs_diff(other.x).max(own.y.abs_diff(other.y))>1 {
                    let route=frame.navigation.route_to_adjacent_avoiding([own.x,own.y],[other.x,other.y],|x,y|super::native_obstacles::blocks(w,x,y))?;
                    ensure!(self.moves<128,"poison approach bound");c.move_checked(*route.first().context("poison approach empty")?,&frame.navigation)?;self.moves+=1;
                }
            }
            return Ok(false);
        }
        let full=profile=="poison-cure";
        if self.phase==0 && frame.movement_ready(w) {
            let own=w.own_actor().unwrap();
            if own.x.abs_diff(target.x).max(own.y.abs_diff(target.y))>1 {return Ok(false);}
            ensure!(self.attempts<8,"natural poison probability exhausted eight casts");
            self.send(c,frame,if profile=="cure-refused" {9}else{11})?;self.phase=1;
        } else if self.phase==1 {
            if self.sent.unwrap().elapsed()<Duration::from_millis(if full {1500}else{3500}){return Ok(false);}
            let w=&c.snapshot().world;
            if !full {
                ensure!(w.own_cast_packets==0 && w.effects.is_empty() && !green,"refused poison/cure changed authority");
                let feedback=if profile=="poison-missing" {Feedback::Rejected(299)}else{Feedback::NoResponse};
                ensure!(c.snapshot().skill_feedback==feedback,"poison/cure refusal feedback");
                ensure!(self.before["world"]["mp"]==serde_json::to_value(w.mp)? && self.before["inventory"]==serde_json::to_value(&c.snapshot().inventory)?,"refused poison/cure spent resources");
                // NoResponse has a two-second observation deadline.
                self.casts.push(json!({"skill":self.skill,"before":self.before,"after":Self::state(c),"no_prediction":true,"duplicate_pending_rejected":true}));
                std::fs::write(root.join("poison-finished"),b"verified refusal")?;self.phase=7;
            } else {
                ensure!(w.own_cast_packets==self.before["world"]["own_cast_packets"].as_u64().unwrap()+1,"poison cast action absent");
                ensure!(w.mp.unwrap()[0] as u64+10==self.before["world"]["mp"][0].as_u64().unwrap(),"poison MP cost mismatch");
                ensure!(matches!(c.snapshot().skill_feedback,Feedback::CastAction|Feedback::Rejected(280)),"poison feedback mismatch");
                self.casts.push(json!({"skill":11,"before":self.before,"after":Self::state(c),"applied":green,"no_prediction":true,"duplicate_pending_rejected":true}));
                if green {self.phase=if self.episode==1 {2}else{5};} else {self.phase=0;}
            }
        } else if self.phase==2 && root.join("a-poison-tick-1").exists() && frame.movement_ready(w) {
            ensure!(green,"poison expired before cure input");self.send(c,frame,9)?;self.phase=3;
        } else if self.phase==3 {
            if self.sent.unwrap().elapsed()<Duration::from_millis(1500){return Ok(false);}
            let w=&c.snapshot().world;
            ensure!(!green && c.snapshot().skill_feedback==Feedback::CastAction,"cure failed to clear authority");
            ensure!(w.own_cast_packets==self.before["world"]["own_cast_packets"].as_u64().unwrap()+1,"cure action absent");
            ensure!(w.mp.unwrap()[0] as u64+15==self.before["world"]["mp"][0].as_u64().unwrap(),"cure MP cost mismatch");
            ensure!(self.before["inventory"]==serde_json::to_value(&c.snapshot().inventory)?,"cure invented reagent spending");
            self.casts.push(json!({"skill":9,"before":self.before,"after":Self::state(c),"no_prediction":true,"duplicate_pending_rejected":true}));self.phase=4;
        } else if self.phase==4 && root.join("a-poison-clean-1").exists() {
            self.phase=0;
        } else if self.phase==5 && !green {
            ensure!(root.join("a-poison-tick-2").exists(),"natural poison never damaged target");
            self.phase=6;self.phase_since=Some(Instant::now());
        } else if self.phase==6 && root.join("a-poison-clean-2").exists() {
            ensure!(fx["active"]==0,"poison/cure effect did not retire");
            std::fs::write(root.join("poison-finished"),b"cure and natural expiry observed")?;self.phase=7;
        }
        if changed {std::fs::write(root.join(format!("{role}-poison-journal.json")),serde_json::to_vec(&self.receipt())?)?;}
        Ok(false)
    }
}
