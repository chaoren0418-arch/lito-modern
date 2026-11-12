//! Qualified current S39 lightning170: cdhhcdhccs/h/(dc).
//! Original common finite effect at frozen wire caster origin; Hit releases
//! each accepted target's presentation only. HP/MP remain server authority.
use super::{native_world::World,native_combat_animation::EnteredEvent,native_effect_timing::{Clip,Playback},
    native_projectiles::Impact,sprite_actions::{SpriteActionCatalog,ActionKey,Animation,Directionality,FrameEvent},
    tower_assets::{SharedEffectBank,HonorAssetError},shared_action_catalog::SharedActionCatalog,
    world_map::{WorldCell,ViewProjection},native_world_effects::{Target,Mix}};
use anyhow::{ensure,Context,Result};
use std::{collections::{BTreeMap,BTreeSet},path::Path,sync::{Arc,atomic::AtomicBool}};
use serde_json::{json,Value};
#[path = "native_batch1_area.rs"]
mod batch1;
#[path="native_tornado.rs"]
mod tornado;
#[path = "native_earth_jail.rs"]
pub(crate) mod earth_jail;
pub fn capture_target_origin(attack:&Attack,actors:&BTreeMap<u32,super::native_world::Actor>)->Option<[i32;2]> {
    batch1::origin(attack,actors)
}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
pub struct TargetHit {pub id:u32,pub hit:bool}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Attack {pub action:u8,pub caster:u32,pub position:[u16;2],pub heading:u8,pub sequence:u32,pub graphics:u16,pub targets:Vec<TargetHit>}
/// Parse all bytes before admitting any world mutation. Unknown canonical
/// gfx/type profiles are quarantined; no generic replacement animation.
pub fn decode(b:&[u8])->Result<Option<Attack>> {
    ensure!(b.first()==Some(&0x39)&&b.len()>=22,"truncated attack-many header");
    let h=|n|u16::from_le_bytes([b[n],b[n+1]]);let d=|n|u32::from_le_bytes(b[n..n+4].try_into().unwrap());
    ensure!(d(2)!=0&&b[10]<8&&d(11)!=0&&h(15)!=0,"invalid attack-many identity/heading");
    ensure!(b[17]==0||b[17]==8,"unknown attack-many type");ensure!(b[18]==0,"unknown attack-many extra byte");
    let end=b[19..].iter().position(|v|*v==0).map(|n|19+n).ok_or_else(||anyhow::anyhow!("unterminated attack-many text"))?;
    ensure!(end-19<=128&&b.len()>=end+3,"attack-many text/count budget");
    let count=h(end+1) as usize;ensure!(count<=128&&b.len()==end+3+count*5,"attack-many target count/tail");
    let mut targets=Vec::with_capacity(count);
    for n in 0..count {let at=end+3+n*5;let id=d(at);ensure!(id!=0&&matches!(b[at+4],0|0x20),"invalid attack-many target/marker");targets.push(TargetHit{id,hit:b[at+4]!=0});}
    let profile=(matches!(h(15),170|171|1804|1805)&&b[17]==8&&b[1]==18)||(h(15)==758&&b[17]==0&&b[1]==18)||(h(15)==3934&&b[17]==0&&b[1]==19);
    if !profile||end!=19{return Ok(None);}
    Ok(Some(Attack{action:b[1],caster:d(2),position:[h(6),h(8)],heading:b[10],sequence:d(11),graphics:h(15),targets}))
}
#[derive(Clone,Debug,serde::Serialize)]
pub struct Cue {pub target_origin:Option<[i32;2]>,pub attack:Attack,pub caster_incarnation:u64,pub action_sequence:u64,pub targets:Vec<(u32,bool,Option<u64>)>}
pub fn qualify(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&170).ok_or(HonorAssetError::Unsupported("lightning170 absent"))?;
    let a=c.resolve_action(170,ActionKey::new(0))?.animation;
    let subs=[3,0,1,2,3,4,5,2,4,5];let ticks=[2,1,2,2,2,2,2,2,2,2];
    if row.graphics_id!=170||row.image_count!=Some(8)||row.header_issue.is_some()||row.actions.len()!=1
        ||row.attributes.len()!=1||row.attributes[0].key!=ActionKey::new(104)||row.attributes[0].body_bytes.trim_ascii()!=b"8"
        ||a.directionality!=Directionality::EightWay||a.frames_per_second!=24||a.frames.len()!=10
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=subs[n]||f.duration_ticks!=ticks[n]
            ||f.events!=match n{1=>vec![FrameEvent::Sound(101)],2=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("lightning170 original definition changed"));
    }
    Ok(vec![(170,0,a.clone())])
}

#[derive(Clone)]struct Active {cue:Cue,start:u64,playback:Playback,frame:usize,finished:bool,flight:Option<batch1::Flight>}
/// Accepted finite callback metadata; no sprite or PCM owner is retained.
#[derive(Clone)]pub(crate) struct History {generation:Option<u64>,seen:BTreeSet<(u32,u64)>,active:Vec<Active>,started:u64,hits:u64,omitted:u64}
#[derive(Default)]
pub struct Effects {generation:Option<u64>,bank:Option<SharedEffectBank>,tornado_bank:Option<SharedEffectBank>,selector_bank:Option<SharedEffectBank>,area_bank:Option<SharedEffectBank>,earth_jail_bank:Option<earth_jail::Bank>,clips:BTreeMap<u16,Clip>,error:Option<String>,seen:BTreeSet<(u32,u64)>,active:Vec<Active>,events:Vec<EnteredEvent>,non_audio:Vec<Value>,started:u64,hits:u64,omitted:u64}
impl Effects {
    pub(crate) fn history(&self)->History {History{generation:self.generation,seen:self.seen.clone(),active:self.active.clone(),started:self.started,hits:self.hits,omitted:self.omitted}}
    pub(crate) fn restore_history(&mut self,h:History){self.generation=h.generation;self.seen=h.seen;self.active=h.active;self.started=h.started;self.hits=h.hits;self.omitted=h.omitted;}
    pub fn prepare(&mut self,root:&Path,w:&World,cancel:&AtomicBool) {
        self.prepare_with_catalog(root,w,cancel,&mut None);
    }
    pub(crate) fn prepare_with_catalog(&mut self,root:&Path,w:&World,cancel:&AtomicBool,
        shared:&mut Option<super::shared_action_catalog::SharedActionCatalog>) {
        if self.error.is_some(){return;}
        if self.tornado_bank.is_none() && (w.learned_skills.contains(&53)||w.many_attacks.iter().any(|c|c.attack.graphics==758)||self.active.iter().any(|a|a.cue.attack.graphics==758)) {
            let loaded=(||->Result<_>{
                let catalog=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                let bank=SharedEffectBank::load_directional(&catalog,cancel,tornado::qualify)?;
                let clip=Clip::new(Arc::clone(bank.animation(758,0)?))?;Ok((bank,clip))
            })();
            match loaded {Ok((bank,clip))=>{self.tornado_bank=Some(bank);self.clips.insert(758,clip);},Err(e)=>{self.error=Some(format!("{e:#}"));return;}}
        }
        if self.earth_jail_bank.is_none() && (w.learned_skills.contains(&30)
            || w.many_attacks.iter().any(|c|c.attack.graphics==1805)
            || self.active.iter().any(|a|a.cue.attack.graphics==1805)) {
            let loaded=(||->Result<_>{
                let catalog=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                earth_jail::Bank::load(root,&catalog,cancel)
            })();
            match loaded {
                Ok((bank,clip))=>{self.earth_jail_bank=Some(bank);self.clips.insert(1805,clip);},
                Err(e)=>{self.error=Some(format!("{e:#}"));return;}
            }
        }
        if self.area_bank.is_none() && (w.learned_skills.iter().any(|id|matches!(id,22|25))
            || w.many_attacks.iter().any(|cue|matches!(cue.attack.graphics,171|1804))
            || self.active.iter().any(|a|matches!(a.cue.attack.graphics,171|1804))) {
            let loaded=(||->Result<_>{
                let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                let bank=SharedEffectBank::load_directional(&c,cancel,batch1::qualify)?;
                let cloud=Clip::new(Arc::clone(bank.animation(1804,0)?))?;
                let impact=Clip::new(Arc::clone(bank.animation(218,0)?))?;
                Ok((bank,cloud,impact))
            })();
            match loaded {
                Ok((bank,cloud,impact))=>{self.area_bank=Some(bank);self.clips.insert(1804,cloud);self.clips.insert(171,impact);},
                Err(e)=>{self.error=Some(format!("{e:#}"));}
            }
        }
        if self.bank.is_none()&&(w.learned_skills.contains(&17)||w.many_attacks.iter().any(|c|c.attack.graphics==170)||self.active.iter().any(|a|a.cue.attack.graphics==170)) {
            let loaded=(||->Result<_>{let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;let bank=SharedEffectBank::load_directional(&c,cancel,qualify)?;
                let clip=Clip::new(Arc::clone(bank.animation(170,0)?))?;Ok((bank,clip))})();
            match loaded{Ok((bank,clip))=>{self.bank=Some(bank);self.clips.insert(170,clip);},Err(e)=>self.error=Some(format!("{e:#}"))}
        }
        if self.selector_bank.is_none()&&(w.many_attacks.iter().any(|c|c.attack.graphics==3934)||self.active.iter().any(|a|a.cue.attack.graphics==3934)) {
            let loaded=(||->Result<_>{let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;let bank=SharedEffectBank::load_reviewed_selectors(&c,cancel)?;
                let clip=Clip::new(Arc::clone(bank.animation(3934,0)?))?;Ok((bank,clip))})();
            match loaded{Ok((bank,clip))=>{self.selector_bank=Some(bank);self.clips.insert(3934,clip);},Err(e)=>self.error=Some(format!("{e:#}"))}
        }
    }
    pub fn update<'a>(&mut self,w:&World,events:impl Iterator<Item=&'a EnteredEvent>,now:u64)->Vec<Impact> {
        self.events.clear();self.non_audio.clear();
        if self.generation!=Some(w.generation){self.generation=Some(w.generation);self.seen.clear();self.active.clear();self.started=0;self.hits=0;self.omitted=0;}
        let live:BTreeSet<_>=w.many_attacks.iter().map(|c|(c.attack.caster,c.action_sequence)).collect();self.seen.retain(|k|live.contains(k)||self.active.iter().any(|a|(a.cue.attack.caster,a.cue.action_sequence)==*k));
        for e in events.filter(|e|e.event=="Hit") {
            let key=(e.actor,e.sequence);let Some(c)=w.many_attacks.iter().find(|c|(c.attack.caster,c.action_sequence)==key)else{continue;};
            if !self.seen.insert(key){continue;}
            if self.active.len()>=64||w.actors.get(&c.attack.caster).is_none_or(|a|a.dead||a.incarnation!=c.caster_incarnation){self.omitted+=1;continue;}
            let Some(clip)=self.clips.get(&c.attack.graphics) else{self.omitted+=1;continue;};
            let flight=if c.attack.graphics==171 {
                match batch1::Flight::launch(c,w) {Ok(f)=>Some(f),Err(_)=>{self.omitted+=1;continue;}}
            } else {None};
            self.active.push(Active{cue:c.clone(),start:now,playback:clip.start(),frame:1,finished:false,flight});self.started+=1;
        }
        let mut impacts=Vec::new();
        for a in &mut self.active {
            if a.finished {continue;}
            if a.cue.attack.graphics!=758 && w.actors.get(&a.cue.attack.caster).is_none_or(|actor|actor.incarnation!=a.cue.caster_incarnation) {a.finished=true;continue;}
            if a.flight.is_some() {
                match self.area_bank.as_ref().context("area bank absent").and_then(|bank|batch1::advance(a,w,bank,now,&mut self.events)) {
                    Ok(values)=>{self.hits+=values.len() as u64;impacts.extend(values);},
                    Err(e)=>{a.finished=true;self.omitted+=1;self.error=Some(format!("{e:#}"));}
                }
                continue;
            }
            match a.playback.advance(super::native_effect_timing::native_elapsed_ns(now,a.start)) {
            Ok(sample)=>{a.frame=sample.source_frame;a.finished=sample.finished;
                for cue in sample.crossed_cues {match cue.event {
                    FrameEvent::Sound(125) if a.cue.attack.graphics==758=>{
                        match self.tornado_bank.as_ref().context("tornado bank missing").and_then(|bank|batch1::audio(bank,&a.cue,758,cue.source_frame,125,1)) {
                            Ok(event)=>self.events.push(event),Err(e)=>{self.omitted+=1;self.error=Some(format!("{e:#}"));a.finished=true;}
                        }
                    },
                    FrameEvent::Sound(718) if a.cue.attack.graphics==1805=>{
                        match self.earth_jail_bank.as_ref().context("earth jail bank absent")
                            .and_then(|bank|batch1::audio(&bank.bank,&a.cue,1805,cue.source_frame,718,1)) {
                            Ok(event)=>self.events.push(event),Err(e)=>{self.error=Some(format!("{e:#}"));a.finished=true;}
                        }
                    },
                    FrameEvent::Sound(714) if a.cue.attack.graphics==1804=>{
                        match self.area_bank.as_ref().context("cloud bank absent").and_then(|bank|batch1::audio(bank,&a.cue,1804,cue.source_frame,714,1)) {
                            Ok(event)=>self.events.push(event),Err(e)=>{self.error=Some(format!("{e:#}"));a.finished=true;}
                        }
                    },
                    FrameEvent::Sound(sound) if a.cue.attack.graphics==170=>self.events.push(EnteredEvent{prepared_audio:None,actor:a.cue.attack.caster,sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:format!("ManySound({sound})"),sound:Some(sound)}),
                    FrameEvent::Effect(_) if a.cue.attack.graphics==3934=>{
                        let prepared=(||->Result<_>{let bank=self.selector_bank.as_ref().context("counter3934 bank absent")?;let animation=bank.animation(3934,0)?;
                            let classified=super::combat_animation::classify_native_cues(&animation.frames[cue.source_frame].events)?;
                            ensure!(classified.non_audio.is_empty(),"counter3934 unsupported non-audio cue");let group=classified.audio.context("counter3934 audio group absent")?;ensure!(group.candidates().len()==1,"finite weighted RNG provider unavailable");
                            let sound=group.selected(0);let payload=super::native_audio_delivery::PreparedCue::from_finite_event(bank.sound_bank(),group,0,super::native_audio_delivery::FiniteEventKey{sequence:a.cue.action_sequence,graphics:3934,action:0,source_frame:cue.source_frame})?;Ok((sound,payload))})();
                        match prepared{Ok((sound,payload))=>self.events.push(EnteredEvent{prepared_audio:Some(payload),actor:a.cue.attack.caster,sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:format!("ManyAudioOverlap({})",sound.unwrap_or(0)),sound}),Err(e)=>{self.error=Some(format!("{e:#}"));self.omitted+=1;a.finished=true;}}
                    },
                    FrameEvent::Hit=>{self.hits+=1;self.non_audio.push(json!({"sequence":a.cue.action_sequence,"source_frame":cue.source_frame,"event":"Hit","authority_callback":false}));
                        for &(id,hit,incarnation) in &a.cue.targets {if hit&&w.actors.get(&id).is_some_and(|actor|Some(actor.incarnation)==incarnation&&!actor.dead){impacts.push(Impact{target:id,sequence:a.cue.action_sequence,projectile:a.cue.attack.sequence});}}},
                    _=>{self.omitted+=1;a.finished=true;}
                }}},Err(_)=>{self.omitted+=1;a.finished=true;}
        }}
        impacts
    }
    pub fn draw_to<'s>(&'s mut self,target:&mut Target<'_,'s>,projection:ViewProjection,now:u64)->(Value,Vec<EnteredEvent>) {
        let mut rows=Vec::new();
        for a in &self.active {
            if a.cue.attack.graphics==758 {
                match self.tornado_bank.as_ref().context("tornado bank missing").and_then(|bank|tornado::draw(bank,a,target,projection)) {
                    Ok(row)=>rows.push(row),Err(e)=>{self.omitted+=1;rows.push(json!({"error":e.to_string()}));}
                }
                continue;
            }
            if a.cue.attack.graphics==1805 {
                let rendered=self.earth_jail_bank.as_ref().context("earth jail bank absent")
                    .and_then(|bank|bank.draw(a,target,projection));
                match rendered {Ok(row)=>rows.push(row),Err(e)=>{self.omitted+=1;rows.push(json!({"error":e.to_string()}));}}
                continue;
            }
            if matches!(a.cue.attack.graphics,171|1804) {
                let (gfx,frame,heading,positions)=if let Some(f)=&a.flight {
                    if f.arrival.is_some() {(218,a.frame,0,f.recipients.clone())}
                    else {(171,f.frame,f.heading(),vec![f.position])}
                } else {
                    let origin=a.cue.target_origin.unwrap_or([i32::from(a.cue.attack.position[0]),i32::from(a.cue.attack.position[1])]);
                    (1804,a.frame,0,vec![WorldCell{x:origin[0],y:origin[1]}])
                };
                let rendered=(||->Result<_>{
                    let bank=self.area_bank.as_ref().context("area bank absent")?;
                    let visual=bank.frame_heading(gfx,0,frame,a.finished,heading)?;
                    let drawn=!a.finished&&!visual.frame.rgba.is_empty();
                    if drawn {
                        ensure!(positions.len()<=128,"area recipient budget");
                        for &position in &positions {projection.cell_to_pixel(position).context("area projection")?;}
                        if let Target::Fragments(queue)=target {ensure!(queue.len()+positions.len()<=512,"area fragments budget");}
                        for &position in &positions {target.submit(visual.frame,position,projection,Mix::Screen)?;}
                    }
                    Ok(json!({"sequence":a.cue.action_sequence,"graphics":gfx,"source_frame":frame,
                        "positions":positions.iter().map(|p|[p.x,p.y]).collect::<Vec<_>>(),"drawn":drawn,"finished":a.finished}))
                })();
                match rendered {Ok(row)=>rows.push(row),Err(e)=>{self.omitted+=1;rows.push(json!({"error":e.to_string()}));}}
                continue;
            }
            let gfx=a.cue.attack.graphics;let bank=if gfx==3934{self.selector_bank.as_ref()}else{self.bank.as_ref()};
            let sample=(||->Result<_>{let bank=bank.context("many finite bank absent")?;Ok(if gfx==3934{bank.frame(3934,0,a.frame,a.finished)?}else{bank.frame_heading(170,0,a.frame,a.finished,a.cue.attack.heading)?})})();
            match sample {
                Ok(v)=>{let position=WorldCell{x:i32::from(a.cue.attack.position[0]),y:i32::from(a.cue.attack.position[1])};let drawn=!a.finished&&!v.frame.rgba.is_empty();
                    if drawn {if let Err(e)=target.submit(v.frame,position,projection,Mix::Screen){self.omitted+=1;rows.push(json!({"error":e.to_string()}));continue;}}
                    rows.push(json!({"sequence":a.cue.action_sequence,"wire_sequence":a.cue.attack.sequence,"graphics":gfx,"position":[position.x,position.y],"heading":a.cue.attack.heading,"targets":a.cue.targets,
                        "sprite":v.sprite_name,"source_frame":a.frame,"subframe":v.subframe,"drawn":drawn,"finished":a.finished,"elapsed_ns":now.saturating_sub(a.start),"native_clock_elapsed_ms":a.playback.sampled_native_millis(),"next_callback_ms":a.playback.next_callback_millis(),"callback_count":a.playback.callback_count()}));},
                Err(e)=>{self.omitted+=1;rows.push(json!({"error":e.to_string()}));}
            }
        }
        self.active.retain(|a|!a.finished);
        (json!({"started":self.started,"hits":self.hits,"active":self.active.len(),"omitted":self.omitted,"load_error":self.error,"effects":rows,"entered_events":self.events,"non_audio_cues":self.non_audio,
            "sources":self.bank.iter().chain(self.selector_bank.iter()).chain(self.area_bank.iter()).chain(self.earth_jail_bank.iter().map(|b|&b.bank)).flat_map(|b|b.sources()).map(|s|json!({"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|v|format!("{v:02x}")).collect::<String>()})).collect::<Vec<_>>(),"complete":false}),self.events.clone())
    }
}
