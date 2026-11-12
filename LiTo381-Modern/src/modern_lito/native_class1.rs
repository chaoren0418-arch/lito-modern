//! Qualified native class1 #252 and #236. Frozen wire destination, two heading steps
//! on launch, no flight/secondary. The effect Hit releases presentation hurt.
use super::{native_combat::ProjectileCue, native_combat_animation::EnteredEvent,
    native_effect_timing::{Clip,Playback}, native_projectiles::Impact, native_world::World,
    shared_action_catalog::SharedActionCatalog, sprite_actions::{ActionKey,Animation,Directionality,FrameEvent,SpriteActionCatalog},
    tower_assets::{HonorAssetError,SharedEffectBank}, world_map::{WorldCell,ViewProjection}};
use std::{collections::{BTreeMap,BTreeSet},path::Path,sync::{Arc,atomic::AtomicBool}};
use serde_json::{json,Value};
pub fn supports_graphics(gfx:u16)->bool {matches!(gfx,252|236|1801|1811)}
pub fn qualify(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let mut rows=qualify_chill(c)?;rows.extend(qualify_vampiric(c)?);rows.extend(qualify_stalac(c)?);rows.extend(qualify_sunburst(c)?);Ok(rows)
}
fn qualify_stalac(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&1801).ok_or(HonorAssetError::Unsupported("stalac absent"))?;
    let a=c.resolve_action(1801,ActionKey::new(0))?.animation;
    let attributes=[(101,b"1802".as_slice()),(105,b"1 1803"),(109,b"1 -1")];
    let ticks=[5,4,3,3,4,5,5,5,5];
    if row.graphics_id!=1801||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1
        || row.attributes.len()!=3||attributes.iter().any(|(id,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*id)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        || a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=9
        || a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{8}else{n as u32-1}
            ||f.duration_ticks!=ticks[n]||f.events!=match n{1=>vec![FrameEvent::Sound(712)],4=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("stalac original definition changed"));
    }
    for (id,expected) in [(1802,vec![(102,b"0".as_slice())]),(1803,vec![(102,b"0".as_slice()),(104,b"8")])] {
        let row=c.sprites.get(&id).ok_or(HonorAssetError::Unsupported("stalac layer absent"))?;
        if row.graphics_id!=id||row.image_count!=Some(1)||row.header_issue.is_some()||!row.actions.is_empty()
            ||row.attributes.len()!=expected.len()||expected.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1) {
            return Err(HonorAssetError::Unsupported("stalac original layer changed"));
        }
    }
    Ok(vec![(1801,0,a.clone())])
}
fn qualify_chill(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {qualify_one(c,252,110)}
fn qualify_vampiric(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {qualify_one(c,236,103)}
fn qualify_one(c:&SpriteActionCatalog,gfx:u32,sound:u32)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&gfx).ok_or(HonorAssetError::Unsupported("chill touch absent"))?;
    let a=c.resolve_action(gfx,ActionKey::new(0))?.animation;
    let ticks=[3,1,2,3,4,1];
    if row.graphics_id!=gfx || row.header_issue.is_some() || row.image_count!=Some(8)
        || row.actions.len()!=1 || row.attributes.len()!=2
        || [104,109].iter().any(|key|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)).count()!=1)
        || row.attributes.iter().any(|v|!((v.key==ActionKey::new(104)&&v.body_bytes.trim_ascii()==b"8")
            ||(v.key==ActionKey::new(109)&&v.body_bytes.trim_ascii()==b"1 -1")))
        || a.directionality!=Directionality::EightWay || a.frames_per_second!=24 || a.frames.len()!=6
        || a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{5}else{n as u32-1}
            || f.duration_ticks!=ticks[n] || f.events!=match n{1=>vec![FrameEvent::Sound(sound)],5=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("chill touch original definition changed"));
    }
    Ok(vec![(gfx,0,a.clone())])
}
pub fn position(to:[u16;2],heading:u8)->anyhow::Result<WorldCell> {
    anyhow::ensure!(heading<8,"class1 heading");
    let delta=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)][heading as usize];
    Ok(WorldCell{x:i32::from(to[0])+2*delta.0,y:i32::from(to[1])+2*delta.1})
}
#[derive(Clone)]
struct Active {cue:ProjectileCue,position:WorldCell,heading:u8,start:u64,playback:Playback,frame:usize,finished:bool}
struct Layers {bank:super::native_actor_layers::Bank,bindings:super::native_actor_layers::Bindings}
#[derive(Default)]
pub struct Effects {generation:Option<u64>,banks:BTreeMap<u16,SharedEffectBank>,clips:BTreeMap<u16,Clip>,error:Option<String>,
    layers:BTreeMap<u16,Layers>,
    seen:BTreeSet<(u32,u64)>,active:Vec<Active>,events:Vec<EnteredEvent>,non_audio:Vec<Value>,started:u64,hits:u64,omitted:u64}
impl Effects {
    pub fn prepare(&mut self,root:&Path,w:&World,cancel:&AtomicBool) {
        self.prepare_with_catalog(root,w,cancel,&mut None);
    }
    pub(crate) fn prepare_with_catalog(&mut self,root:&Path,w:&World,cancel:&AtomicBool,
        shared:&mut Option<super::shared_action_catalog::SharedActionCatalog>) {
        for (skill,gfx) in [(10,252u16),(28,236u16),(16,1801u16),(46,1811u16)] {
            if self.banks.contains_key(&gfx)||self.error.is_some()||!(w.learned_skills.contains(&skill)||w.projectiles.iter().any(|c|c.projectile.graphics==gfx)||self.active.iter().any(|a|a.cue.projectile.graphics==gfx)){continue;}
            let loaded=(||->anyhow::Result<_>{let catalog=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;
                let bank=SharedEffectBank::load_directional(&catalog,cancel,match gfx{252=>qualify_chill,236=>qualify_vampiric,1811=>qualify_sunburst,_=>qualify_stalac})?;
                let animation=bank.animation(u32::from(gfx),0)?;
                let clip=Clip::new(Arc::clone(animation))?;
                let layers=if gfx==1801 {
                    let bindings=super::native_actor_layers::Plan::from_catalog(catalog.catalog(),1801)?.bind(0)?;
                    let mut bank=super::native_actor_layers::Bank::open(root)?;
                    for f in &animation.frames {let p=bank.prepare(&bindings,f.subframe as usize,cancel)?;
                        anyhow::ensure!(p.missing_shadow.is_none()&&p.missing_attachments.is_empty(),"stalac layers incomplete");}
                    Some(Layers{bank,bindings})
                }else{None};
                Ok((bank,clip,layers))})();
            match loaded {Ok((bank,clip,layers))=>{self.banks.insert(gfx,bank);self.clips.insert(gfx,clip);if let Some(layers)=layers{self.layers.insert(gfx,layers);}},Err(e)=>self.error=Some(format!("{e:#}"))}
        }
    }
    pub fn update<'a>(&mut self,w:&World,events:impl Iterator<Item=&'a EnteredEvent>,now:u64)->Vec<Impact> {
        self.events.clear();self.non_audio.clear();
        if self.generation!=Some(w.generation){self.generation=Some(w.generation);self.seen.clear();self.active.clear();self.started=0;self.hits=0;self.omitted=0;}
        let live:BTreeSet<_>=w.projectiles.iter().map(|c|(c.attacker,c.action_sequence)).collect();self.seen.retain(|k|live.contains(k)||self.active.iter().any(|a|(a.cue.attacker,a.cue.action_sequence)==*k));
        for event in events.filter(|e|e.event=="Hit") {
            let key=(event.actor,event.sequence);
            let Some(cue)=w.projectiles.iter().find(|c|(c.attacker,c.action_sequence)==key&&supports_graphics(c.projectile.graphics)) else {continue;};
            if !self.seen.insert(key){continue;}
            if self.active.len()>=64||w.actors.get(&cue.attacker).is_none_or(|a|a.dead||a.incarnation!=cue.attacker_incarnation){self.omitted+=1;continue;}
            let Some(clip)=self.clips.get(&cue.projectile.graphics) else{self.omitted+=1;continue;};
            let heading=cue.heading;
            let Ok(position)=(if cue.projectile.graphics==1811 {Ok(WorldCell{x:i32::from(cue.projectile.to[0]),y:i32::from(cue.projectile.to[1])})}else{position(cue.projectile.to,heading)}) else{self.omitted+=1;continue;};
            self.active.push(Active{cue:cue.clone(),position,heading,start:now,playback:clip.start(),frame:1,finished:false});self.started+=1;
        }
        let mut impacts=Vec::new();
        for a in &mut self.active {
            match a.playback.advance((now / 1_000_000).saturating_sub(a.start / 1_000_000) * 1_000_000) {
                Ok(sample)=>{a.frame=sample.source_frame;a.finished=sample.finished;
                    for cue in sample.crossed_cues {match cue.event {
                        FrameEvent::Sound(sound)=>{
                            let prepared=(||->anyhow::Result<_>{
                                let bank=self.banks.get(&a.cue.projectile.graphics).ok_or_else(||anyhow::anyhow!("class1 bank missing"))?;
                                let group=super::combat_animation::classify_native_cues(&[FrameEvent::Sound(sound)])?.audio.ok_or_else(||anyhow::anyhow!("class1 sound missing"))?;
                                super::native_audio_delivery::PreparedCue::from_finite_event(bank.sound_bank(),group,0,
                                    super::native_audio_delivery::FiniteEventKey{sequence:a.cue.action_sequence,graphics:a.cue.projectile.graphics,action:0,source_frame:cue.source_frame})
                            })();
                            match prepared {Ok(payload)=>self.events.push(EnteredEvent{prepared_audio:Some(payload),actor:0,sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:format!("Class1Sound({sound})"),sound:Some(sound)}),Err(e)=>{self.omitted+=1;self.error=Some(e.to_string());a.finished=true;}}
                        },
                        FrameEvent::Hit=>{self.hits+=1;self.non_audio.push(json!({"sequence":a.cue.action_sequence,"source_frame":cue.source_frame,"event":"Hit","authority_callback":false}));
                            // 1811's relative-gfx hurt guard remains unqualified;
                            // retain its Hit as a diagnostic, never synthesize a generic hurt.
                            if a.cue.projectile.graphics!=1811 && a.cue.damage_marker>0&&w.actors.get(&a.cue.target).is_some_and(|actor|Some(actor.incarnation)==a.cue.target_incarnation&&!actor.dead){
                                impacts.push(Impact{target:a.cue.target,sequence:a.cue.action_sequence,projectile:a.cue.projectile.id});}}
                        _=>{self.omitted+=1;a.finished=true;}
                    }}
                },Err(_)=>{self.omitted+=1;a.finished=true;}
            }
        }
        impacts
    }
    pub fn draw_to<'s>(&'s mut self,target:&mut super::native_world_effects::Target<'_,'s>,projection:ViewProjection,now:u64)->(Value,Vec<EnteredEvent>) {
        let mut rows=Vec::new();
        for a in &self.active { if let Some(bank)=self.banks.get(&a.cue.projectile.graphics) {
            match bank.frame_heading(u32::from(a.cue.projectile.graphics),0,a.frame,a.finished,a.heading) {
                Ok(v)=>{
                    let mut layer_receipts=Vec::new();
                    let layered=(||->anyhow::Result<()> {
                        let mut submissions=vec![(v.frame,if a.cue.projectile.graphics==1801{super::native_world_effects::Mix::Over}else{super::native_world_effects::Mix::Screen})];
                        if let Some(layers)=self.layers.get(&a.cue.projectile.graphics) {
                            for (name,mix) in &layers.bindings.attachments {
                                let frame=layers.bank.frame(name,v.subframe)?;
                                let mode=match mix{super::native_actor_layers::Mix::Over=>super::native_world_effects::Mix::Over,super::native_actor_layers::Mix::Screen=>super::native_world_effects::Mix::Screen};
                                submissions.push((frame,mode));
                                layer_receipts.push(json!({"kind":"clothes","sprite":name,"subframe":v.subframe,"mix":mix,"drawn":!a.finished,"source":layers.bank.get(name)?.source}));
                            }
                            if let Some(name)=&layers.bindings.shadow {
                                let frame=layers.bank.frame(name,v.subframe)?;
                                submissions.push((frame,super::native_world_effects::Mix::Shadow));
                                layer_receipts.push(json!({"kind":"shadow","sprite":name,"subframe":v.subframe,"mix":"Shadow","drawn":!a.finished,"source":layers.bank.get(name)?.source}));
                            }
                        }
                        if !a.finished {target.submit_batch(&submissions,a.position,projection)?;}
                        Ok(())
                    })();
                    if let Err(e)=layered {self.omitted+=1;rows.push(json!({"error":e.to_string()}));continue;}
                    rows.push(json!({"sequence":a.cue.action_sequence,"projectile":a.cue.projectile.id,"graphics":a.cue.projectile.graphics,"position":[a.position.x,a.position.y],"heading":a.heading,
                        "sprite":v.sprite_name,"source_frame":a.frame,"subframe":v.subframe,"layers":layer_receipts,"drawn":!a.finished,"finished":a.finished,"elapsed_ns":now.saturating_sub(a.start),"native_clock_elapsed_ms":a.playback.sampled_native_millis(),"next_callback_ms":a.playback.next_callback_millis(),"callback_count":a.playback.callback_count()}));},
                Err(e)=>{self.omitted+=1;rows.push(json!({"error":e.to_string()}));}
            }
        }}
        self.active.retain(|a|!a.finished);
        (json!({"started":self.started,"hits":self.hits,"active":self.active.len(),"omitted":self.omitted,"load_error":self.error,"effects":rows,
            "entered_events":self.events,"non_audio_cues":self.non_audio,
            "sources":self.banks.values().flat_map(|b|b.sources()).map(|s|json!({"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|v|format!("{v:02x}")).collect::<String>()})).collect::<Vec<_>>(),
            "complete":false}),self.events.clone())
    }
    pub fn draw(&mut self,canvas:&mut super::compositor::Canvas,projection:ViewProjection,now:u64)->(Value,Vec<EnteredEvent>) {
        self.draw_to(&mut super::native_world_effects::Target::Canvas(canvas),projection,now)
    }
}

fn qualify_sunburst(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&1811).ok_or(HonorAssetError::Unsupported("sunburst missing"))?;
    let a=c.resolve_action(1811,ActionKey::new(0))?.animation;
    let ticks=[4,1,1,1,1,1,1,1,1,1,1,4,4,4,4,4,4,4,4];
    if row.graphics_id!=1811 || row.image_count!=Some(1) || row.header_issue.is_some()
        || row.actions.len()!=1 || row.attributes.len()!=2
        || [(104,b"8".as_slice()),(109,b"1 -1")].iter().any(|(k,v)|row.attributes.iter().filter(|r|r.key==ActionKey::new(*k)&&r.body_bytes.trim_ascii()==*v).count()!=1)
        || a.directionality!=Directionality::Undirected || a.frames_per_second!=24 || a.frames.len()!=19
        || a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0 || f.subframe!=if n==0{18}else{n as u32-1}
            || f.duration_ticks!=ticks[n] || f.events!=match n{1=>vec![FrameEvent::Sound(709)],11=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("sunburst definition changed"));
    }
    Ok(vec![(1811,0,a.clone())])
}

#[derive(Clone)]
pub(crate) struct History {generation:Option<u64>,seen:BTreeSet<(u32,u64)>,active:Vec<Active>,started:u64,hits:u64,omitted:u64}
impl History {
    pub(crate) fn retire(&mut self){self.active.clear();}
    pub(crate) fn merge(&mut self,other:&Self){if self.generation==other.generation{self.seen.extend(other.seen.iter().copied());}}
    pub(crate) fn oldest(&self)->Option<u64>{self.active.iter().map(|a|a.cue.action_sequence).min()}
}
impl Effects {
    pub(crate) fn history(&self)->History{History{generation:self.generation,seen:self.seen.clone(),active:self.active.clone(),started:self.started,hits:self.hits,omitted:self.omitted}}
    pub(crate) fn restore(&mut self,h:History){self.generation=h.generation;self.seen=h.seen;self.active=h.active;self.started=h.started;self.hits=h.hits;self.omitted=h.omitted;}
}
