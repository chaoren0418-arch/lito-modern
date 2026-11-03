//! Natural two-player honor deaths and server-timed new lives; no injected HP.
use super::{native_combat::HeldAttack,native_obstacles,native_login::LoginController,
    native_skill_selection::Target,native_world_scene::Frame};
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::path::Path;
#[derive(Default)]
pub struct Probe {
    held:HeldAttack, pressed:bool, attacks:usize, moves:usize,
    target:Option<Target>, life:u64, deaths:Vec<Value>, revivals:Vec<Value>,
    death_frame:Option<Frame>, moved:bool,
    replacements:Vec<Value>,
    last_revived_at:Option<std::time::Instant>,
    movement:super::native_navigation::HeldInput,
    fresh_input_sent:bool,
}
impl Probe {
    pub fn receipt(&self)->Value{json!({"deaths":self.deaths,"revivals":self.revivals,"visibility_replacements":self.replacements,"attacks_sent":self.attacks,"moves_sent":self.moves,"fresh_input_sent":self.fresh_input_sent})}
    pub fn tick(&mut self,c:&mut LoginController,frame:Frame,root:&Path,role:&str)->Result<Option<Value>> {
        let other=if role=="a"{"b"}else{"a"};
        let w=&c.snapshot().world;
        ensure!(w.local_corrections==0,"honor movement corrected by server: {:?}",w.own_actor());
        ensure!(w.battle_result.is_none(),"honor battle ended before lifecycle exercise");
        ensure!(c.snapshot().joined_room.as_ref().is_some_and(|r|r.mode==5),"not actual honor mode");
        let target=w.actors.get(&3).context("honor target missing")?;
        // Opponents may begin outside the rendered viewport in larger arenas.
        let pose=frame.receipt["actors"].as_array().context("honor frames")?.iter().find(|a|a["id"]==3).unwrap_or(&Value::Null);
        if self.target.is_none(){
            self.target=Target::capture(w,3);self.life=target.incarnation;
            if role=="b" {
                self.movement.synchronize(Some(super::native_navigation::InputStamp::capture(c.transport().snapshot().epoch,w)));
                // Keep an unsent old-life press across real death/respawn packets
                // to exercise a delayed GUI input sample without moving B.
                self.movement.press(0x68);
            }
        }
        if w.resurrection_packets==self.revivals.len() as u64 && target.incarnation!=self.life {
            // Re-entering another player's visibility triggers an authoritative
            // S_PUT_OBJECT. It renews the object token but is not a resurrection.
            ensure!(!target.dead&&self.deaths.len()==self.revivals.len(),"unexpected replacement during death");
            ensure!(self.target.context("replacement target")?.resolve(w).is_none(),"old selection survived visibility replacement");
            ensure!(self.replacements.len()<16,"visibility replacement budget");
            self.replacements.push(json!({"old_incarnation":self.life,"actor":target,"resurrection_packets":w.resurrection_packets}));
            self.target=Target::capture(w,3);self.life=target.incarnation;
        }
        if target.dead&&self.deaths.len()==self.revivals.len()&&pose["phase"]=="death"&&pose["action_complete"]==true {
            let image=super::native_peer_probe::capture(root,role,self.deaths.len()+1,"honor-death",&frame)?;
            self.deaths.push(json!({"world":w,"render":image}));self.held.clear();self.pressed=false;
            if role=="b" {
                ensure!(frame.death_ready(w),"own death frame not ready");c.confirm_death(&frame)?;self.death_frame=Some(frame);
            }
            return Ok(None);
        }
        if !target.dead && w.resurrection_packets>self.revivals.len() as u64 && self.revivals.len()<2 {
            ensure!(target.incarnation!=self.life,"resurrection did not renew actor life");
            ensure!(self.deaths.len()==self.revivals.len()+1,"new life without observed death");
            ensure!(w.last_resurrection.is_some_and(|r|r.target==3&&r.source==3&&r.action==0),"honor did not use actual self resurrection");
            ensure!(self.target.context("retired target")?.resolve(w).is_none(),"old target survived resurrection");
            if pose["world"]!=json!([target.x,target.y])||pose["phase"]!="idle"||target.hp_ratio==0{return Ok(None);}
            if role=="b"&&(!frame.movement_ready(w)||!frame.input_ready(w)||!w.hp.is_some_and(|hp|hp[0]>0&&hp[0]==hp[1])){return Ok(None);}
            let retired=if role=="b" {
                let old=self.death_frame.as_ref().context("retired death frame missing")?;
                ensure!(!old.death_ready(w)&&!old.input_ready(w)&&!old.movement_ready(w),"old life frame authorized input");
                ensure!(!w.death_confirmation_pending,"old death confirmation survived");true
            }else{false};
            let retired_input=if role=="b" {
                ensure!(self.movement.direction().is_some(),"old-life held input fixture missing");
                ensure!(self.movement.synchronize(Some(super::native_navigation::InputStamp::capture(c.transport().snapshot().epoch,w))),"new life did not retire input stamp");
                ensure!(self.movement.direction().is_none(),"old-life movement survived resurrection");true
            }else{false};
            let image=super::native_peer_probe::capture(root,role,self.revivals.len()+1,"honor-revived",&frame)?;
            self.revivals.push(json!({"world":w,"render":image,"retired_death_frame_rejected":retired,"retired_held_input_rejected":retired_input}));
            if role=="b"&&self.revivals.len()<2 {self.movement.press(0x68);}
            self.last_revived_at=Some(std::time::Instant::now());
            std::fs::write(root.join(format!("{role}-revival-{}",self.revivals.len())),b"observed new life")?;
            self.target=Target::capture(w,3);self.life=target.incarnation;self.death_frame=None;
        }
        if self.revivals.len()>0&&!root.join(format!("{other}-revival-{}",self.revivals.len())).exists(){return Ok(None);}
        if self.revivals.len()==2 {
            // Stay connected beyond the former eight-second death response
            // deadline before proving restored input on the second life.
            if self.last_revived_at.is_none_or(|t|t.elapsed()<std::time::Duration::from_secs(9)){return Ok(None);}
            let marker=root.join("honor-moved.json");
            if role=="b"&&!self.moved&&frame.movement_ready(w)&&frame.input_ready(w){
                let own=w.own_actor().context("revived own actor")?;
                let heading=(0..8).find(|&h|frame.navigation.can_step(i32::from(own.x),i32::from(own.y),h)
                    &&super::native_world::destination(own.x,own.y,h).is_ok_and(|(x,y)|!native_obstacles::blocks(w,x,y))).context("revived move direction")?;
                let key=[0x68,0x69,0x66,0x63,0x62,0x61,0x64,0x67][heading as usize];
                self.movement.press(key);
                ensure!(self.movement.direction()==Some(heading),"fresh movement press unavailable");
                c.move_checked(heading,&frame.navigation)?;
                self.movement.sent();self.movement.release(key);self.fresh_input_sent=true;
                self.moves+=1;self.moved=true;
                let own=c.snapshot().world.own_actor().unwrap();let temp=root.join("honor-moved.tmp");
                std::fs::write(&temp,serde_json::to_vec(&json!([own.x,own.y]))?)?;std::fs::rename(temp,&marker)?;return Ok(None);
            }
            if marker.exists(){
                let position:Value=serde_json::from_slice(&std::fs::read(marker)?)?;
                if json!([target.x,target.y])==position&&pose["world"]==position&&pose["phase"]=="idle" {
                    std::fs::write(root.join(format!("{role}-honor-finished")),b"observed revived move")?;
                    if root.join(format!("{other}-honor-finished")).exists(){
                        return Ok(Some(super::native_peer_probe::capture(root,role,2,"honor-end",&frame)?));
                    }
                }
            }
            return Ok(None);
        }
        if role=="a"&&!target.dead&&frame.movement_ready(w) {
            let own=w.own_actor().context("honor attacker")?;
            if own.x.abs_diff(target.x).max(own.y.abs_diff(target.y))>1 {
                let route=frame.navigation.route_to_adjacent_avoiding([own.x,own.y],[target.x,target.y],|x,y|native_obstacles::blocks(w,x,y))?;
                ensure!(self.moves<128,"honor movement budget");c.move_checked(*route.first().context("honor route")?,&frame.navigation)?;self.moves+=1;
            }else{
                if !self.pressed{self.held.press(3,c.transport().snapshot().epoch,w);self.pressed=true;}
                if c.continue_attack(&mut self.held,&frame)?{self.attacks+=1;ensure!(self.attacks<256,"honor attack budget");}
            }
        }
        Ok(None)
    }
}
