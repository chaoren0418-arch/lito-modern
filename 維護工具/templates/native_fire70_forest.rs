//! Original1819 forest: three descriptor-free children then relocated root, all at radius one.
//! Metadata clocks and bounded, shared assets; no HP or native hurt substitute.
use super::{native_attack_many::Cue,native_world::World,native_effect_timing::{Clip,Playback},
    native_combat_animation::EnteredEvent,sprite_actions::{ActionKey,Animation,Directionality,FrameEvent,SpriteActionCatalog},
    tower_assets::{HonorAssetError,SharedEffectBank},shared_action_catalog::SharedActionCatalog,
    native_world_effects::{Target,Mix},world_map::{WorldCell,ViewProjection}};
use std::{collections::BTreeSet,path::Path,sync::{Arc,atomic::AtomicBool}};
use anyhow::{Result,ensure,Context};
use serde_json::{json,Value};
pub(crate) fn qualify(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&1819).ok_or(HonorAssetError::Unsupported("firestorm1819 absent"))?;
    let resolved=c.resolve_action(1819,ActionKey::new(0))?;let a=resolved.animation;
    let ticks=[6,3,3,3,4,4,4,4,4,4,4,4,5,5];let attrs:[(u32,&[u8]);2]=[(104,b"8"),(109,b"0 -1")];
    if row.id!=1819||row.graphics_id!=1819||resolved.definition_sprite_id!=1819||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=2
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=14
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{13}else{n as u32-1}||f.duration_ticks!=ticks[n]
            ||f.events!=match n{1=>vec![FrameEvent::Sound(710)],7=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("firestorm1819 original class/audio/timing changed"));
    }
    Ok(vec![(1819,0,a.clone())])
}
#[derive(Clone)]struct Node {position:WorldCell,birth:u64,playback:Playback,source:usize,finished:bool}
#[derive(Clone)]struct Clock {nodes:Vec<Node>,last:Option<u64>}
struct Entry {node:usize,source:usize,callback:u64,event:FrameEvent}
impl Clock {
    fn new(origin:WorldCell,clip:&Clip,births:[u64;4])->Result<Self> {
        let positions:Vec<_>=OFFSETS.iter().map(|[dx,dy]|Ok(WorldCell{x:origin.x.checked_add(*dx).context("firestorm X")?,y:origin.y.checked_add(*dy).context("firestorm Y")?})).collect::<Result<_>>()?;
        Ok(Self{nodes:positions.into_iter().zip(births).map(|(position,birth)|Node{position,birth,playback:clip.start(),source:1,finished:false}).collect(),last:None})
    }
    fn advance(&mut self,now:u64)->Result<Vec<Entry>> {
        ensure!(self.last.is_none_or(|old|now>=old),"firestorm clock regressed");ensure!(self.nodes.iter().all(|n|now>=n.birth),"firestorm node not born");let mut entries=vec![];
        for(node,n)in self.nodes.iter_mut().enumerate(){if n.finished{continue;}let s=n.playback.advance((now/1_000_000-n.birth/1_000_000)*1_000_000)?;n.source=s.source_frame;n.finished=s.finished;let callback=n.playback.callback_count().context("firestorm native callback count")?;entries.extend(s.crossed_cues.into_iter().map(|c|Entry{node,source:c.source_frame,callback,event:c.event}));}
        self.last=Some(now);Ok(entries)
    }
    fn finished(&self)->bool {self.nodes.iter().all(|n|n.finished)}
}
#[derive(Clone)]struct Cast {cue:Cue,clock:Clock,finished:bool}
#[derive(Clone,Default)]pub(crate) struct History {generation:Option<u64>,seen:BTreeSet<(u32,u64)>,active:Vec<Cast>,started:u64,omitted:u64}
impl History {
    pub(crate) fn active_sequences(&self)->BTreeSet<u64>{self.active.iter().filter(|a|!a.finished).map(|a|a.cue.action_sequence).collect()}
    pub(crate) fn merge_admission(&mut self,other:&Self){if self.generation==other.generation{self.seen.extend(other.seen.iter().copied());}}
}
#[derive(Default)]pub(crate) struct Stage {history:History,bank:Option<SharedEffectBank>,clip:Option<Clip>,error:Option<String>,events:Vec<EnteredEvent>,diagnostics:Vec<Value>}
impl Stage {
    pub(crate) fn history(&self)->History{self.history.clone()}
    pub(crate) fn restore(&mut self,mut h:History,same_source:bool){if !same_source{h.active.clear();}self.history=h;}
    pub(crate) fn active_casts(&self)->usize{self.history.active.len()}
    pub(crate) fn prepare(&mut self,root:&Path,w:&World,cancel:&AtomicBool,shared:&mut Option<SharedActionCatalog>){
        if self.bank.is_some()||self.error.is_some()||!(w.learned_skills.contains(&70)||w.many_attacks.iter().any(|c|c.attack.graphics==1819)||!self.history.active.is_empty()){return;}
        let loaded=(||->Result<_>{let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;let bank=SharedEffectBank::load_directional(&c,cancel,qualify)?;let clip=Clip::new(Arc::clone(bank.animation(1819,0)?))?;Ok((bank,clip))})();match loaded{Ok((bank,clip))=>{self.bank=Some(bank);self.clip=Some(clip);},Err(e)=>self.error=Some(e.to_string())}
    }
    pub(crate) fn update<'a>(&mut self,w:&World,events:impl Iterator<Item=&'a EnteredEvent>,now:u64,other_active:usize){
        self.events.clear();self.diagnostics.clear();if self.history.generation!=Some(w.generation){self.history=History{generation:Some(w.generation),..Default::default()};}
        let live:BTreeSet<_>=w.many_attacks.iter().map(|c|(c.attack.caster,c.action_sequence)).chain(self.history.active.iter().map(|a|(a.cue.attack.caster,a.cue.action_sequence))).collect();self.history.seen.retain(|k|live.contains(k));
        for event in events.filter(|e|e.event=="Hit") {let key=(event.actor,event.sequence);let Some(cue)=w.many_attacks.iter().find(|c|(c.attack.caster,c.action_sequence)==key&&c.attack.graphics==1819)else{continue;};if !self.history.seen.insert(key){continue;}
            if self.history.active.len().saturating_add(other_active)>=64||w.actors.get(&cue.attack.caster).is_none_or(|a|a.dead||a.incarnation!=cue.caster_incarnation){self.history.omitted+=1;continue;}
            let result=(||->Result<_>{let clip=self.clip.as_ref().context("firestorm clip absent")?;Clock::new(WorldCell{x:i32::from(cue.attack.position[0]),y:i32::from(cue.attack.position[1])},clip,[now;4])})();
            match result{Ok(clock)=>{self.history.active.push(Cast{cue:cue.clone(),clock,finished:false});self.history.started+=1;},Err(e)=>{self.history.omitted+=1;self.diagnostics.push(json!({"omitted":e.to_string()}));}}
        }
        for cast in &mut self.history.active {if cast.finished{continue;}if self.error.is_some()&&self.bank.is_none(){cast.finished=true;self.history.omitted+=1;continue;}
            let result=(||->Result<_>{let entries=cast.clock.advance(now)?;let mut audio=vec![];for entry in entries {match entry.event {
                FrameEvent::Sound(sound)=>{ensure!(sound==710,"firestorm sound changed");let bank=self.bank.as_ref().context("firestorm audio bank absent")?;let a=bank.animation(1819,0)?;let classified=super::combat_animation::classify_native_cues(&a.frames[entry.source].events)?;ensure!(classified.non_audio.is_empty(),"firestorm audio marker changed");let group=classified.audio.context("firestorm sound group absent")?;
                    let occurrence=((entry.node as u64+1)<<32)|entry.callback;let payload=super::native_audio_delivery::PreparedCue::from_callback_event(bank.sound_bank(),group,0,super::native_audio_delivery::FiniteEventKey{sequence:cast.cue.action_sequence,graphics:1819,action:0,source_frame:entry.source},occurrence)?;
                    audio.push(EnteredEvent{actor:0,sequence:cast.cue.action_sequence,action:0,frame:entry.source,event:format!("FirestormSound710Node{}",entry.node),sound:Some(710),prepared_audio:Some(payload)});
                },
                FrameEvent::Hit=>self.diagnostics.push(json!({"sequence":cast.cue.action_sequence,"node":entry.node,"source_frame":entry.source,"root_target_list":entry.node==3,"targets":if entry.node==3{Some(&cast.cue.targets)}else{None},"authority_callback":false,"relative_graphics_hurt_binding_pending":true})),
                _=>anyhow::bail!("unreviewed firestorm event"),
            }}Ok(audio)})();
            match result{Ok(audio)=>{if self.events.len().checked_add(audio.len()).is_some_and(|n|n<=32){self.events.extend(audio);}else{self.history.omitted+=1;self.diagnostics.push(json!({"sequence":cast.cue.action_sequence,"audio_batch_omitted":true,"no_retry":true}));}},Err(e)=>{self.history.omitted+=1;cast.finished=true;self.diagnostics.push(json!({"sequence":cast.cue.action_sequence,"error":e.to_string()}));}}
            cast.finished|=cast.clock.finished();
        }
    }
    pub(crate) fn draw_to<'s>(&'s mut self,target:&mut Target<'_,'s>,view:ViewProjection)->(Value,Vec<EnteredEvent>){
        let mut rows=vec![];for cast in &self.history.active {let result=(||->Result<_>{let bank=self.bank.as_ref().context("firestorm frame bank absent")?;let mut frames=vec![];let mut nodes=vec![];
            for (ordinal,n)in cast.clock.nodes.iter().enumerate(){let sample=bank.frame(1819,0,n.source,n.finished)?;let drawn=!cast.finished&&!n.finished&&!sample.frame.rgba.is_empty();if drawn{frames.push((sample.frame,n.position));}nodes.push(json!({"node":ordinal,"root":ordinal==3,"position":[n.position.x,n.position.y],"source_frame":n.source,"sprite":sample.sprite_name,"drawn":drawn,"finished":n.finished,"next_callback_ms":n.playback.next_callback_millis(),"callbacks":n.playback.callback_count()}));}
            submit_group(target,&frames,view)?;Ok(json!({"sequence":cast.cue.action_sequence,"wire_sequence":cast.cue.attack.sequence,"copies":4,"nodes":nodes,"targets":cast.cue.targets,"finished":cast.finished}))})();rows.push(match result{Ok(row)=>row,Err(e)=>json!({"sequence":cast.cue.action_sequence,"error":e.to_string(),"drawn":false})});}
        self.history.active.retain(|a|!a.finished);(json!({"started":self.history.started,"active":self.history.active.len(),"omitted":self.history.omitted,"load_error":self.error,"groups":rows,"diagnostics":self.diagnostics,"complete":false}),self.events.clone())
    }
}
fn submit_group<'s>(target:&mut Target<'_,'s>,frames:&[(&'s super::sprite::SpriteFrame,WorldCell)],view:ViewProjection)->Result<()> {
    for(frame,position)in frames{let p=view.cell_to_pixel(*position).context("firestorm projection")?;p.x.checked_add(frame.x_offset).context("firestorm anchor X")?;p.y.checked_add(frame.y_offset).context("firestorm anchor Y")?;}
    if let Target::Fragments(queue)=target {ensure!(queue.len().checked_add(frames.len()).is_some_and(|n|n<=512),"firestorm group fragment budget");ensure!(frames.iter().all(|(f,_)|f.fragments.is_some()),"firestorm source fragments absent");}
    for(frame,position)in frames{target.submit(frame,*position,view,Mix::Screen)?;}Ok(())
}

const OFFSETS:[[i32;2];4]=[[1,1],[-1,-1],[-1,1],[1,-1]];
