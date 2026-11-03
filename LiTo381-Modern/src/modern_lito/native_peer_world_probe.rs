//! Two real players observe every completed movement/door transition.
use super::{native_doors, native_login::LoginController, native_world::destination,native_world_scene::Frame};
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{path::Path,time::{Duration,Instant}};
fn heading(from:[u16;2],to:[u16;2])->Result<u8>{
    (0..8).find(|&h|destination(from[0],from[1],h).ok()==Some((to[0],to[1]))).context("non-adjacent fixture step")
}
fn actor_frame(frame:&Frame,id:u32)->Result<&Value>{
    frame.receipt["actors"].as_array().context("actor frames")?.iter().find(|r|r["id"]==id).context("subject frame missing")
}
#[derive(Default)]
pub struct Probe {
    stage:usize, sent:Option<Instant>, observed_since:Option<Instant>, observed:bool,
    adjacent:Option<[u16;2]>, rows:Vec<Value>, moves:usize, toggles:usize, blocked:usize,
}
impl Probe {
    pub fn receipt(&self)->Value{json!({"stages":self.rows,"moves_sent":self.moves,"toggles_sent":self.toggles,"blocked_requests":self.blocked})}
    fn step(&mut self,c:&mut LoginController,frame:&Frame,h:u8)->Result<()> {
        ensure!(self.moves<128,"world probe movement bound");c.move_checked(h,&frame.navigation)?;self.moves+=1;Ok(())
    }
    fn approach(&mut self,c:&mut LoginController,frame:&Frame,target:[u16;2])->Result<bool>{
        let w=&c.snapshot().world;let own=w.own_actor().context("approach own actor")?;
        if own.x.abs_diff(target[0]).max(own.y.abs_diff(target[1]))==1 {self.adjacent=Some([own.x,own.y]);return Ok(true);}
        let route=frame.navigation.route_to_adjacent_avoiding([own.x,own.y],target,|x,y|native_doors::blocks(w,x,y))?;
        self.step(c,frame,*route.first().context("approach route")?)?;Ok(false)
    }
    pub fn tick(&mut self,c:&mut LoginController,frame:&Frame,root:&Path,role:&str,profile:&str)->Result<bool>{
        let total=if profile=="doors"{14}else{35};
        ensure!(c.snapshot().world.local_corrections==0,"unexpected authoritative correction in world exercise");
        ensure!(c.snapshot().world.hp.is_some_and(|v|v[0]>0),"world exercise unexpected death");
        if self.stage==total {
            std::fs::write(root.join(format!("world-{role}-finished")),b"observed")?;
            return Ok(root.join(format!("world-{}-finished",if role=="a"{"b"}else{"a"})).exists());
        }
        let active=if profile=="doors" {if self.stage<7 {"a"}else{"b"}}
            else if self.stage<16||self.stage>=32 {"a"}else{"b"};
        let active_id=if active=="a"{1}else{3};
        let intent_path=root.join(format!("world-intent-{}.json",self.stage));
        if role==active&&self.sent.is_none()&&frame.movement_ready(&c.snapshot().world)&&frame.input_ready(&c.snapshot().world){
            let before=json!(c.snapshot().world);
            let own=c.snapshot().world.own_actor().context("world own")?;
            let from=[own.x,own.y];
            let mut kind="move";let mut subject=active_id;let mut expected_action=None;let mut expected_passable=None;
            if profile=="doors"{
                let door=c.snapshot().world.actors.values().find(|a|native_doors::is_door(a)).context("actual arena door")?;
                let id=door.id;let position=[door.x,door.y];
                match self.stage%7 {
                    0=>{kind="approach";if !self.approach(c,frame,position)?{return Ok(false);}}
                    1|6=>{
                        kind="blocked";let h=heading(from,position)?;
                        ensure!(frame.navigation.can_step(i32::from(from[0]),i32::from(from[1]),h),"fixture door is also a static wall");
                        ensure!(c.move_checked(h,&frame.navigation).is_err(),"closed door movement admitted");
                        ensure!(before==serde_json::to_value(&c.snapshot().world)?,"blocked move mutated authority");self.blocked+=1;
                    }
                    2|5=>{
                        kind="toggle";subject=id;
                        let choice=native_doors::Choice::capture(c.transport().snapshot().epoch,&c.snapshot().world,id)?;
                        expected_passable=Some(if choice.attribute.passable==65{0}else{65});
                        expected_action=Some(if expected_passable==Some(0){28}else{29});
                        c.toggle_door(choice.clone(),frame)?;
                        ensure!(c.toggle_door(choice,frame).is_err(),"duplicate door request admitted");
                        ensure!(before==serde_json::to_value(&c.snapshot().world)?,"door request predicted authority");self.toggles+=1;
                    }
                    3=>self.step(c,frame,heading(from,position)?)?,
                    4=>self.step(c,frame,heading(from,self.adjacent.context("door approach position")?)?)?,
                    _=>unreachable!(),
                }
            }else{
                if self.stage<32 {
                    let local=self.stage%16;let base=(local/2) as u8;
                    self.step(c,frame,if local%2==0{base}else{(base+4)%8})?;
                }else{
                    let target=&c.snapshot().world.actors[&3];let target_position=[target.x,target.y];
                    match self.stage {
                        32=>{kind="approach";if !self.approach(c,frame,target_position)?{return Ok(false);}}
                        33=>{kind="occupied_player_cell";self.step(c,frame,heading(from,target_position)?)?;}
                        34=>self.step(c,frame,heading(from,self.adjacent.context("overlap origin")?)?)?,
                        _=>unreachable!(),
                    }
                }
            }
            let a=&c.snapshot().world.actors[&subject];
            let intent=json!({"stage":self.stage,"kind":kind,"active":active,"subject":subject,
                "position":[a.x,a.y],"heading":a.heading,"expected_action":expected_action,"expected_passable":expected_passable,
                "before":before,"generation":c.snapshot().world.generation});
            let temporary=root.join(format!("world-intent-{}-{role}.tmp",self.stage));
            std::fs::write(&temporary,serde_json::to_vec(&intent)?)?;std::fs::rename(temporary,&intent_path)?;
            self.sent=Some(Instant::now());
        }
        if intent_path.exists()&&!self.observed {
            let intent:Value=serde_json::from_slice(&std::fs::read(&intent_path)?)?;
            let since=self.observed_since.get_or_insert_with(Instant::now);
            ensure!(since.elapsed()<Duration::from_secs(12),"world stage {} not converged",self.stage);
            let id=intent["subject"].as_u64().context("intent subject")? as u32;
            let w=&c.snapshot().world;let a=w.actors.get(&id).context("received world subject")?;
            let pose=actor_frame(frame,id)?;
            let position=json!([a.x,a.y]);
            let toggle=intent["kind"]=="toggle";
            let converged=position==intent["position"]&&pose["world"]==position&&pose["rendered"]==true && if toggle {
                a.walk_action==intent["expected_action"].as_u64().unwrap() as u8
                    &&native_doors::record(w,id).is_some_and(|r|Some(u64::from(r.attribute.passable))==intent["expected_passable"].as_u64())
                    &&pose["action"]==intent["expected_action"]&&pose["phase"]=="object_action"&&pose["action_complete"]==true
                    &&since.elapsed()>=Duration::from_millis(1600)
            }else{pose["phase"]=="idle"&&pose["heading"]==intent["heading"]&&a.heading==intent["heading"].as_u64().unwrap() as u8};
            let minimap_ready = profile != "movement" || w.own_actor().is_some_and(|own|own.id==id)
                || frame.receipt["minimap"]["markers"].as_array().is_some_and(|rows| rows.iter().any(|m|m["id"]==id && m["world"]==position));
            if converged && minimap_ready {
                if role==active&&toggle{ensure!(c.door_feedback()==native_doors::Feedback::Changed,"door controller acknowledgement missing");}
                let render=if toggle||self.stage==15||self.stage==31||self.stage==33 {
                    super::native_peer_probe::capture(root,role,self.stage+1,"world-stage",frame)?
                }else{json!({"subject":pose})};
                self.rows.push(json!({"intent":intent,"world":w,"subject_frame":pose,"minimap_frame":frame.receipt["minimap"],"render":render}));
                std::fs::write(root.join(format!("world-{role}-observed-{}",self.stage)),b"authoritative frame")?;self.observed=true;
            }
        }
        if self.observed&&root.join(format!("world-{}-observed-{}",if role=="a"{"b"}else{"a"},self.stage)).exists(){
            self.stage+=1;self.sent=None;self.observed_since=None;self.observed=false;
        }
        Ok(false)
    }
}
