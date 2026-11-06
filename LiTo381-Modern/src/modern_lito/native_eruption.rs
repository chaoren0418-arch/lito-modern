//! E149 45, S23 wire type 0 / graphic 129: class7 propagation then class6.
//! Rendering callbacks never infer damage, HP, status, or a successful cast.
use crate::modern_lito::{native_combat::ProjectileCue,native_combat_animation::EnteredEvent,
    native_world::World,native_world_effects::{Target,Mix},
    sprite_actions::{SpriteActionCatalog,ActionKey,Animation,Directionality,FrameEvent},
    tower_assets::{SharedEffectBank,HonorAssetError},shared_action_catalog::SharedActionCatalog,
    world_map::{WorldCell,ViewProjection},native_audio_delivery::{PreparedCue,FiniteEventKey},
    combat_animation::classify_native_cues};
use std::{collections::BTreeSet,path::Path,sync::atomic::AtomicBool};
use anyhow::{ensure,Context,Result};
use serde_json::json;
const CRUST: &[u32]=&[240,2,2,2,3,4,5,6,7,4];
const EARTH: &[u32]=&[240,2,3,4,8,7,6,5];

fn qualify(c:&SpriteActionCatalog)->crate::modern_lito::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let mut out=Vec::new();
    for (gfx,shadow,count,ticks,controls) in [
        (129,130,4,CRUST,vec![(101,b"130".as_slice()),(102,b"0"),(109,b"7 123")]),
        (123,124,1,EARTH,vec![(101,b"124".as_slice()),(102,b"0"),(109,b"6 -1")])] {
        let r=c.sprites.get(&gfx).ok_or(HonorAssetError::Unsupported("eruption absent"))?;
        let a=c.resolve_action(gfx,ActionKey::new(0))?.animation;
        let layer=c.sprites.get(&shadow).ok_or(HonorAssetError::Unsupported("eruption shadow absent"))?;
        if r.graphics_id!=gfx || r.image_count!=Some(count) || r.header_issue.is_some() || r.actions.len()!=1
            || r.attributes.len()!=controls.len() || controls.iter().any(|(k,v)|r.attributes.iter().filter(|r|r.key==ActionKey::new(*k)&&r.body_bytes.trim_ascii()==*v).count()!=1)
            || a.frames_per_second!=24 || a.directionality!=if gfx==129{Directionality::EightWay}else{Directionality::Undirected}
            || a.frames.len()!=ticks.len() || a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0
                || f.subframe!=if n==0{ticks.len() as u32-1}else{n as u32-1} || f.duration_ticks!=ticks[n]
                || f.events!=match (gfx,n){(129,0)=>vec![FrameEvent::Sound(129)],(129,1)=>vec![FrameEvent::Sound(128)],(123,2)=>vec![FrameEvent::Effect(3080)],(123,4)=>vec![FrameEvent::Hit],_=>vec![]})
            || layer.graphics_id!=shadow || layer.image_count!=Some(count) || layer.header_issue.is_some()
            || !layer.actions.is_empty() || layer.attributes.len()!=1 || layer.attributes[0].key!=ActionKey::new(102)
            || layer.attributes[0].body_bytes.trim_ascii()!=b"0" {
            return Err(HonorAssetError::Unsupported("eruption definition changed"));
        }
        // Class7 folds eight directions into four banks; shadows inherit slots.
        for bank in 0..count {
            let mut slots=a.clone();slots.directionality=Directionality::Undirected;
            for f in &mut slots.frames {f.image_id=bank;}
            out.push((gfx,bank as u8,slots.clone()));
            for f in &mut slots.frames {f.events.clear();}
            out.push((shadow,bank as u8,slots));
        }
    }
    Ok(out)
}
fn cell([x,y]:[u16;2])->WorldCell {WorldCell{x:i32::from(x),y:i32::from(y)}}
fn path(from:[u16;2],to:[u16;2],heading:u8)->Result<(Vec<WorldCell>,u8)> {
    ensure!(heading<8,"eruption heading");
    let start=[2*i32::from(from[0])-32768,i32::from(from[1])];
    let end=[2*i32::from(to[0])-32768,i32::from(to[1])];
    let dx=end[0]-start[0];let dy=end[1]-start[1];let span=(dx.abs()/2).max(dy.abs());
    ensure!(span<=64,"eruption path budget");
    let direction=if dx==0&&dy==0{heading}else {
        let c=20*dy;
        if c>24*dx {if c> -24*dx {4}else if c> -4*dx {5}else if c<=4*dx {7}else{6}}
        else if c>4*dx {3}else if c> -4*dx {2}else if c> -24*dx {1}else{0}
    }%4;
    let step=if span==0{[0,0]}else{[(dx/2*65536/span)*2,dy*65536/span]};
    let mut fixed=start.map(|v|(v as u32).wrapping_shl(16));
    let mut remaining=dx.abs().max(dy.abs());let mut positions=vec![cell(from)];
    loop {
        for i in 0..2 {fixed[i]=fixed[i].wrapping_add(step[i] as u32);}
        let next=[((fixed[0].wrapping_add(0x8000)>>16)&!1) as i32,(fixed[1].wrapping_add(0x8000)>>16) as i32];
        if next==end || remaining<1 {break;}
        ensure!(positions.len()<64,"eruption complete path budget");
        positions.push(WorldCell{x:(next[0]+32768)/2,y:next[1]});remaining-=1;
    }
    Ok((positions,direction))
}
#[derive(Clone)]
struct Node {gfx:u32,position:WorldCell,frame:usize,serial:u64,due:Option<(u64,u64)>,visible:bool,final_draw:bool}
#[derive(Clone)]
struct Cast {cue:ProjectileCue,positions:Vec<WorldCell>,heading:u8,endpoint:[u16;2],nodes:Vec<Node>,order:u64}
impl Cast {
    fn target(&self,w:&World)->[u16;2] {
        w.actors.get(&self.cue.target).filter(|a|Some(a.incarnation)==self.cue.target_incarnation)
            .map_or(self.endpoint,|a|[a.x,a.y])
    }
    fn enter(&mut self,index:usize,now:u64,w:&World,events:&mut Vec<(usize,u32,usize,u64)>)->Result<()> {
        let n=&mut self.nodes[index];
        if n.gfx==129 && n.frame==0 && n.serial>0 {n.visible=false;n.due=None;return Ok(());}
        let terminal=n.gfx==123 && n.frame==0 && n.serial>0;
        let ticks=if n.gfx==129{CRUST}else{EARTH};
        n.frame=(n.frame+1)%ticks.len();n.serial+=1;n.visible=true;n.final_draw=terminal;
        events.push((index,n.gfx,n.frame,n.serial));
        self.order+=1;
        n.due=if terminal{None}else{Some((now+u64::from(ticks[n.frame])*1000/24,self.order))};
        if n.gfx==129 && n.frame==2 {
            ensure!(index+1==self.nodes.len(),"duplicate eruption propagation");
            let (gfx,position)=if index+1<self.positions.len(){(129,self.positions[index+1])}else{(123,cell(self.target(w)))};
            self.nodes.push(Node{gfx,position,frame:0,serial:0,due:None,visible:false,final_draw:false});
            self.enter(index+1,now,w,events)?;
        }
        Ok(())
    }
}
#[derive(Default,Clone)]
pub(super) struct History {generation:Option<u64>,seen:BTreeSet<(u32,u64)>,casts:Vec<Cast>,started:u64,omitted:u64}
impl History {
    pub(super) fn retire(&mut self){self.casts.clear();}
    pub(super) fn merge(&mut self,other:&Self){if self.generation==other.generation{self.seen.extend(other.seen.iter().copied());}}
    pub(super) fn oldest(&self)->Option<u64>{self.casts.iter().map(|c|c.cue.action_sequence).min()}
}
#[derive(Default)]
pub(super) struct Effects {history:History,bank:Option<SharedEffectBank>,error:Option<String>,events:Vec<EnteredEvent>}
impl Effects {
    pub(super) fn active_count(&self)->usize{self.history.casts.len()}
    pub(super) fn history(&self)->History{self.history.clone()}
    pub(super) fn restore(&mut self,h:History){self.history=h;}
    pub(super) fn prepare(&mut self,root:&Path,w:&World,cancel:&AtomicBool,shared:&mut Option<SharedActionCatalog>) {
        if self.bank.is_some()||self.error.is_some()||!(w.learned_skills.contains(&45)||w.projectiles.iter().any(|c|c.projectile.graphics==129)||!self.history.casts.is_empty()){return;}
        let loaded=(||->Result<_>{let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;Ok(SharedEffectBank::load_directional(&c,cancel,qualify)?)})();
        match loaded {
            Ok(bank)=>self.bank=Some(bank),Err(e)=>self.error=Some(format!("{e:#}"))
        }
    }
    pub(super) fn update(&mut self,w:&World,events:&[&EnteredEvent],now:u64,other_active:usize) {
        self.events.clear();
        if self.history.generation!=Some(w.generation){self.history=History{generation:Some(w.generation),..Default::default()};}
        self.history.seen.retain(|k|w.projectiles.iter().any(|c|(c.attacker,c.action_sequence)==*k)||self.history.casts.iter().any(|c|(c.cue.attacker,c.cue.action_sequence)==*k));
        for event in events.iter().filter(|e|e.event=="Hit") {
            let Some(cue)=w.projectiles.iter().find(|c|c.projectile.flight_type==0&&c.projectile.graphics==129&&(c.attacker,c.action_sequence)==(event.actor,event.sequence)) else{continue;};
            if !self.history.seen.insert((cue.attacker,cue.action_sequence)){continue;}
            if self.bank.is_none()||self.history.casts.len()+other_active>=64||w.actors.get(&cue.attacker).is_none_or(|a|a.dead||a.incarnation!=cue.attacker_incarnation){self.history.omitted+=1;continue;}
            let endpoint=w.actors.get(&cue.target).filter(|a|Some(a.incarnation)==cue.target_incarnation).map_or(cue.projectile.to,|a|[a.x,a.y]);
            match path(cue.projectile.from,endpoint,cue.heading) {
                Ok((positions,heading))=>{
                    let root=Node{gfx:129,position:positions[0],frame:0,serial:0,due:Some((now/1_000_000,0)),visible:false,final_draw:false};
                    self.history.casts.push(Cast{cue:cue.clone(),positions,heading,endpoint,nodes:vec![root],order:0});self.history.started+=1;
                },Err(e)=>{self.error=Some(e.to_string());self.history.omitted+=1;}
            }
        }
        for cast in &mut self.history.casts {
            let mut entered=Vec::new();let mut draft=cast.clone();let mut result=Ok(());
            for _ in 0..65 {
                let next=draft.nodes.iter().enumerate().filter_map(|(i,n)|n.due.filter(|(due,_)|*due<=now/1_000_000).map(|d|(d,i))).min_by_key(|(d,_)|*d);
                let Some((_,index))=next else{break;};
                result=draft.enter(index,now/1_000_000,w,&mut entered);if result.is_err(){break;}
            }
            if let Err(e)=result {self.error=Some(e.to_string());self.history.omitted+=1;continue;}
            *cast=draft;
            for (node,gfx,source,serial) in entered {
                let prepared=(||->Result<Option<EnteredEvent>>{
                    let bank=self.bank.as_ref().context("eruption bank missing")?;
                    let a=bank.animation(gfx,if gfx==129{cast.heading}else{0})?;
                    let Some(group)=classify_native_cues(&a.frames[source].events)?.audio else{return Ok(None);};
                    let sound=group.selected(0);
                    let payload=PreparedCue::from_callback_event(bank.sound_bank(),group,0,FiniteEventKey{sequence:cast.cue.action_sequence,graphics:gfx as u16,action:0,source_frame:source},((node as u64+1)<<32)|serial)?;
                    Ok(Some(EnteredEvent{prepared_audio:Some(payload),actor:0,sequence:cast.cue.action_sequence,action:0,frame:source,event:"EruptionSound".into(),sound}))
                })();
                match prepared {Ok(Some(e))=>self.events.push(e),Ok(None)=>{},Err(e)=>{self.error=Some(e.to_string());self.history.omitted+=1;}}
            }
        }
    }
    pub(super) fn draw<'s>(&'s mut self,target:&mut Target<'_,'s>,view:ViewProjection)->(serde_json::Value,Vec<EnteredEvent>) {
        let mut rows=Vec::new();
        if let Some(bank)=&self.bank {for cast in &mut self.history.casts {
            let rendered=(||->Result<_>{
                let mut frames=Vec::new();
                for (index,n) in cast.nodes.iter().enumerate().filter(|(_,n)|n.visible) {
                    let action=if n.gfx==129{cast.heading}else{0};
                    let main=bank.frame(n.gfx,action,n.frame,false)?;
                    let shadow=bank.frame(n.gfx+1,action,n.frame,false)?;
                    let anchor=view.cell_to_pixel(n.position).context("eruption projection")?;
                    for frame in [main.frame,shadow.frame] {
                        anchor.x.checked_add(frame.x_offset).context("eruption x")?;anchor.y.checked_add(frame.y_offset).context("eruption y")?;
                        if matches!(target,Target::Fragments(_)){ensure!(frame.fragments.is_some(),"eruption fragments missing");}
                    }
                    frames.push((index,n.position,main.frame,shadow.frame));
                }
                if let Target::Fragments(queue)=target{ensure!(queue.len()+frames.len()*2<=512,"eruption group budget");}
                let mut samples=Vec::new();
                for (index,position,main,shadow) in frames {
                    target.submit_batch(&[(main,Mix::Over),(shadow,Mix::Shadow)],position,view)?;
                    let n=&mut cast.nodes[index];samples.push(json!({"node":index,"graphics":n.gfx,"source_frame":n.frame,"position":[position.x,position.y],"final_draw":n.final_draw}));
                    if n.final_draw {n.visible=false;}
                }
                Ok(json!({"sequence":cast.cue.action_sequence,"nodes":samples}))
            })();
            match rendered{Ok(row)=>rows.push(row),Err(e)=>{self.history.omitted+=1;rows.push(json!({"error":e.to_string()}));}}
        }}
        self.history.casts.retain(|c|c.nodes.iter().any(|n|n.visible||n.due.is_some()));
        (json!({"started":self.history.started,"active":self.history.casts.len(),"omitted":self.history.omitted,"load_error":self.error,"casts":rows,"hurt_qualified":false}),self.events.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cast()->Cast {
        let (positions,heading)=path([32768,32768],[32771,32768],2).unwrap();
        Cast{cue:ProjectileCue{heading:2,attacker_incarnation:1,target_incarnation:None,action_sequence:1,attacker:1,target:0,damage_marker:0,
            projectile:crate::modern_lito::native_combat::Projectile{id:1,flight_type:0,graphics:129,from:[32768,32768],to:[32771,32768]}},
            nodes:vec![Node{gfx:129,position:positions[0],frame:0,serial:0,due:None,visible:false,final_draw:false}],positions,heading,endpoint:[32771,32768],order:0}
    }
    #[test]
    fn propagation_excludes_endpoint_and_rejects_over_budget_without_truncating() {
        let c=cast();assert_eq!(c.heading,2);assert_eq!(c.positions.iter().map(|p|[p.x,p.y]).collect::<Vec<_>>(),vec![[32768,32768],[32769,32768],[32770,32768]]);
        assert_eq!(path([32768,32768],[32768,32768],7).unwrap().1,3);
        assert!(path([32768,32768],[32833,32768],2).is_err());assert!(path([1,1],[2,1],8).is_err());
    }
    #[test]
    fn nested_callback_schedules_parent_first_and_child_anchor_is_frozen() {
        let mut c=cast();let w=World::default();let mut events=Vec::new();
        c.enter(0,0,&w,&mut events).unwrap();assert_eq!(c.nodes[0].due,Some((83,1)));
        c.enter(0,500,&w,&mut events).unwrap();
        assert_eq!(events,vec![(0,129,1,1),(0,129,2,2),(1,129,1,1)]);
        assert_eq!(c.nodes[0].due,Some((583,2)));assert_eq!(c.nodes[1].due,Some((583,3)));
        c.enter(1,583,&w,&mut events).unwrap();c.enter(2,666,&w,&mut events).unwrap();
        assert_eq!(c.nodes[3].gfx,123);assert_eq!(c.nodes[3].position,cell([32771,32768]));
    }
    #[test]
    fn terminal_earth_frame_waits_for_draw_and_crust_retires_without_reentry() {
        let mut c=cast();let w=World::default();let mut events=Vec::new();
        c.nodes[0].frame=0;c.nodes[0].serial=10;c.nodes[0].visible=true;
        c.enter(0,12000,&w,&mut events).unwrap();assert!(!c.nodes[0].visible);assert!(events.is_empty());
        c.nodes[0].gfx=123;c.nodes[0].serial=8;c.enter(0,12000,&w,&mut events).unwrap();
        assert!(c.nodes[0].final_draw&&c.nodes[0].visible);assert_eq!(c.nodes[0].frame,1);assert_eq!(c.nodes[0].due,None);
    }
    #[test]
    fn source_retirement_preserves_admission_and_late_merge_cannot_restore_graph() {
        let mut h=History{generation:Some(9),casts:vec![cast()],..Default::default()};h.seen.insert((1,1));
        let late=h.clone();h.retire();h.merge(&late);assert!(h.casts.is_empty());assert!(h.seen.contains(&(1,1)));
        let mut other=late;other.generation=Some(10);other.seen.insert((2,2));h.merge(&other);assert!(!h.seen.contains(&(2,2)));
    }
}
