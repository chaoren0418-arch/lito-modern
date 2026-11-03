//! Explicit Native BASE prince body study, with its original route and local
//! supplemental pins. Not authoritative visual equipment or full LOADED proof.
use super::{battle_follow::Observation, compositor::Canvas, native_canonical_draft::RendererJob,
    native_resources::{self as r,BytePin,NativeAssetPlan,NativeBaseArchive,NativeBaseInspectionArchive},
    sprite::{self,DecodedSprite,SpriteBlockReference,SpriteLimits},
    sprite_actions::{ActionKey,ActionLimits,Animation,Directionality,FrameEvent,SpriteActionCatalog},
    world_depth::{NativeDrawCell,NativeDrawKey},world_map::WorldCell};
use anyhow::{ensure,Context,Result};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,path::Path,sync::{Arc,atomic::{AtomicBool,Ordering}}};
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NativeBaseBodyPins {
    pub text_index:BytePin,pub text_pak:BytePin,pub decoded_catalog:BytePin,
    pub sprite_index:BytePin,pub sprite_pak:BytePin,
    /// Exactly the four 3225-{0,1,8,9} original hash-selected MISS indexes.
    pub shard_misses:Vec<(u8,BytePin,u64)>,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NativeBodyRanges {pub text_offset:u64,pub text_raw:BytePin,pub members:Vec<(u32,u64,BytePin)>}
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub enum BodyAction { Walk,Idle }
impl BodyAction {fn id(self)->u32 {match self{Self::Walk=>0,Self::Idle=>3}}}
#[derive(Clone,Copy)]
pub enum BodyStudy { ExplicitUnarmedPrince }
#[derive(Clone)]
pub struct NativeActorSnapshot {
    plan:NativeAssetPlan,pins:NativeBaseBodyPins,observation:Observation,cancel:Arc<AtomicBool>,
    heading:u8,cell:WorldCell,animations:BTreeMap<BodyAction,Animation>,sprites:BTreeMap<u32,DecodedSprite>,
    members:Vec<(String,[u8;32],usize)>,
}
pub struct NativeActorSlice {
    plan:NativeAssetPlan,observation:Observation,cancel:Arc<AtomicBool>,pins:NativeBaseBodyPins,
    action:BodyAction,elapsed_ms:u64,frame:usize,subframe:u32,image:u32,heading:u8,
    canvas:Canvas,depths:Vec<(NativeDrawKey,SpriteBlockReference)>,events:Vec<FrameEvent>,
}
fn active(cancel:&AtomicBool)->Result<()> {ensure!(!cancel.load(Ordering::Acquire),"Native body study cancelled");Ok(())}
/// Native 43DE10 initializes FPS24, opcode110 replaces it, and each frame
/// stores integer (ticks*1000)/FPS independently (then XOR1812). This function
/// models the unmodified catalog path, not speed/brave/global multipliers.
pub fn catalog_frame_ms(ticks:u32,fps:u32)->Result<u32> {
    ensure!(ticks>0 && ticks<=1000 && fps>0 && fps<=1000,"Native clock scalar outside audited bound");
    let ms=ticks.checked_mul(1000).context("Native duration overflow")?/fps;
    ensure!(ms>0 && ms<=32767,"Native signed-word duration outside positive slice");Ok(ms)
}
pub fn relative_frame(durations:&[u32],elapsed_ms:u64)->Result<usize> {
    ensure!(!durations.is_empty() && durations.len()<=128 && durations.iter().all(|d|*d>0 && *d<=32767),"Native bounded duration list");
    let mut end=0u64;
    for (i,d) in durations.iter().enumerate(){end+=u64::from(*d);if elapsed_ms<end{return Ok(i)}}
    anyhow::bail!("Native action start/loop beyond explicit single interval unqualified")
}
impl NativeActorSnapshot {
    /// The explicit study avoids interpreting absent SCENE_ONLY vital/equipment
    /// as an authoritative unarmed weapon. It binds the actual admitted owner,
    /// class/sex, heading and cell; only prince0/male and headings0/1 are ready.
    pub fn load(job:&RendererJob,root:&Path,pins:NativeBaseBodyPins,_study:BodyStudy)->Result<Self> {
        Self::load_impl(job,root,pins,None)
    }
    pub fn load_authoritative(job:&RendererJob,root:&Path,pins:NativeBaseBodyPins,ranges:&NativeBodyRanges)->Result<Self>{
        ensure!(ranges.members.iter().map(|m|m.0).eq([0,1,8,9]),"Actor identity member order differs");
        Self::load_impl(job,root,pins,Some(ranges))
    }
    fn load_impl(job:&RendererJob,root:&Path,pins:NativeBaseBodyPins,ranges:Option<&NativeBodyRanges>)->Result<Self> {
        let cancel=job.cancellation_token();active(&cancel)?;
        let observation=job.admitted_observation().clone();let d=observation.data();let p=&d.personal.player;
        let member=d.room.members.0.iter().find(|m|m.identity.source==p.source && m.identity.id==p.object).context("Native body owner member absent")?;
        ensure!(member.class==0 && member.sex==0,"Native body class/sex not yet qualified");
        let actor=d.actors.0.iter().find(|a|a.entity==d.personal.entity).context("Native body owner actor absent")?;
        ensure!(matches!(actor.heading,0|1),"Native body heading outside pinned slice");
        let heading=actor.heading as u8;let cell=WorldCell{x:actor.position.x,y:actor.position.y};
        let mut text=NativeBaseInspectionArchive::open(root,NativeBaseArchive::Text,pins.text_index,pins.text_pak,&cancel)?;
        let bytes=if let Some(r)=ranges{text.read_verified_member("list.spr",r.text_offset,r.text_raw,Some(pins.decoded_catalog),&cancel)?}else{text.read_member("list.spr",&cancel)?};
        ensure!(bytes.len() as u64==pins.decoded_catalog.bytes && <[u8;32]>::from(Sha256::digest(&bytes))==pins.decoded_catalog.sha256,"Native base catalog drift");
        ensure!(bytes.starts_with(b"11297 0 41210"),"Original base catalog header drift");
        let start=bytes.windows(3).position(|w|w==b"#0\t").context("Native prince physical row missing")?;
        let tail=&bytes[start..];let end=tail.windows(2).position(|w|w==b"\n#").map(|i|i+1).unwrap_or(tail.len());
        ensure!(end<=16*1024,"Native body catalog row exceeds budget");
        let catalog=SpriteActionCatalog::parse(&tail[..end],ActionLimits{max_bytes:16*1024,max_line_bytes:4096,
            max_sprites:1,max_actions:128,max_frames:4096,max_frames_per_action:128,max_events_per_frame:16,max_reference_depth:1})?;
        let row=catalog.sprites.get(&0).context("Native body logical0 absent")?;
        ensure!(row.graphics_id==3225,"Native prince alias contract drift");
        let mut animations=BTreeMap::new();
        for action in [BodyAction::Walk,BodyAction::Idle] {
            let a=catalog.resolve_action(0,ActionKey::new(action.id()))?;
            ensure!(a.definition_sprite_id==0 && a.animation.directionality==Directionality::EightWay
                && a.animation.frames_per_second==24 && a.animation.frames.len()==if action==BodyAction::Walk{4}else{8}
                && a.animation.frames.iter().all(|f|f.duration_ticks==4),"Native prince action contract drift");
            ensure!(!a.animation.has_uninterpreted_events(),"Native body unknown event syntax");
            animations.insert(action,a.animation.clone());
        }
        ensure!(pins.shard_misses.len()==4 && pins.shard_misses.iter().map(|p|p.0).eq([4,5,12,13]),"Native body miss pin roles/order mismatch");
        let mut sprites=BTreeMap::new();let mut members=Vec::new();
        let mut sprite=NativeBaseInspectionArchive::open(root,NativeBaseArchive::Sprite,pins.sprite_index,pins.sprite_pak,&cancel)?;
        let mut pixels=0usize;
        for image in [0,1,8,9] {
            let name=format!("3225-{image}.spr");let shard=(name.bytes().map(u32::from).sum::<u32>()%16) as u8;
            let (_,index,pak_bytes)=pins.shard_misses.iter().find(|p|p.0==shard).context("Native hash shard missing")?;
            r::verify_native_body_shard_miss(root,shard,*index,*pak_bytes,&name,&cancel)?;
            let bytes=if let Some(r)=ranges{let m=r.members.iter().find(|m|m.0==image).context("Actor member authority missing")?;sprite.read_verified_member(&name,m.1,m.2,None,&cancel)?}else{sprite.read_member(&name,&cancel)?};
            ensure!(bytes.len()<=1024*1024,"Native actor selected SPR overbound");
            members.push((name,Sha256::digest(&bytes).into(),bytes.len()));
            let decoded=sprite::decode_world_spr_with_limits(&bytes,SpriteLimits{max_input_bytes:1024*1024,max_frames:32,
                max_blocks:4096,max_block_references:32768,max_frame_dimension:1024,max_total_pixels:4*1024*1024})?;
            ensure!(decoded.frames.iter().all(|f|f.fragments.is_some()),"Native body depth fragments absent");
            pixels+=decoded.frames.iter().map(|f|f.width as usize*f.height as usize).sum::<usize>();
            ensure!(pixels<=4*1024*1024,"Native actor aggregate pixel budget");
            sprites.insert(image,decoded);
        }
        for a in animations.values(){for h in [0,1]{for i in 0..a.frames.len(){let f=a.frame_asset(3225,i,h)?;
            ensure!((f.subframe as usize)<sprites[&f.image_id].frames.len(),"Native body source subframe absent");}}}
        active(&cancel)?;
        Ok(Self{plan:job.source_plan().clone(),pins,observation,cancel,heading,cell,animations,sprites,members})
    }
    pub fn members(&self)->&[(String,[u8;32],usize)]{&self.members}
    pub(crate) fn scene_frame(&self,job:&RendererJob,action:BodyAction,frame:usize)->Result<(&sprite::SpriteFrame,NativeDrawCell)> {
        active(&self.cancel)?;
        ensure!(self.plan==*job.source_plan()&&self.observation.data()==job.admitted_observation().data()
            &&Arc::ptr_eq(&self.cancel,&job.cancellation_token()),"Native scene body owner mismatch");
        let a=&self.animations[&action];ensure!(frame<a.frames.len(),"Native scene body frame outside single interval");
        let source=a.frame_asset(3225,frame,self.heading)?;
        Ok((&self.sprites[&source.image_id].frames[source.subframe as usize],NativeDrawCell::from_world(self.cell).context("Native scene body coordinate overflow")?))
    }
    pub fn duration_ms(&self,action:BodyAction)->Result<Vec<u32>> {self.animations[&action].frames.iter().map(|f|catalog_frame_ms(f.duration_ticks,self.animations[&action].frames_per_second)).collect()}
    pub fn render(&self,action:BodyAction,elapsed_ms:u64)->Result<NativeActorSlice> {
        self.render_observation(&self.observation,action,elapsed_ms)
    }
    pub fn render_observation(&self,observation:&Observation,action:BodyAction,elapsed_ms:u64)->Result<NativeActorSlice> {
        self.observation.check_successor(observation)?;
        let d=observation.data();let p=&d.personal.player;
        let member=d.room.members.0.iter().find(|m|m.identity.source==p.source && m.identity.id==p.object).context("Native current body member absent")?;
        ensure!(member.class==0 && member.sex==0,"Native body successor class/sex unsupported");
        let actor=d.actors.0.iter().find(|a|a.entity==d.personal.entity).context("Native current body actor absent")?;
        ensure!(matches!(actor.heading,0|1),"Native current heading unsupported");
        let heading=actor.heading as u8;let cell=WorldCell{x:actor.position.x,y:actor.position.y};
        active(&self.cancel)?;let a=&self.animations[&action];let frame=relative_frame(&self.duration_ms(action)?,elapsed_ms)?;
        let source=a.frame_asset(3225,frame,heading)?;
        let decoded=&self.sprites[&source.image_id].frames[source.subframe as usize];
        let fragments=decoded.fragments.as_ref().context("Native actor fragments absent")?;
        let anchor=NativeDrawCell::from_world(cell).context("Native actor half-cell overflow")?;
        let mut depths=Vec::new();
        for (i,reference) in fragments.references.iter().enumerate(){active(&self.cancel)?;let cell=anchor.fragment(*reference).context("Native body depth overflow")?;depths.push((cell.overlay_key(reference.depth,i as u64),*reference));}
        depths.sort_by_key(|d|d.0);let mut canvas=Canvas::new(512,256)?;canvas.clear([24,24,32]);let mut rgba=[0;2304];
        for (_,reference) in &depths{active(&self.cancel)?;fragments.atlas[reference.block_id as usize].write_rgba(&mut rgba);
            let (x,y)=anchor.fragment_pixel_offset(*reference).context("Native body pixel overflow")?;
            canvas.blit_rgba(&rgba,24,24,256+i32::try_from(x)?,180+i32::try_from(y)?)?;}
        active(&self.cancel)?;
        Ok(NativeActorSlice{plan:self.plan.clone(),observation:observation.clone(),cancel:self.cancel.clone(),pins:self.pins.clone(),
            action,elapsed_ms,frame,subframe:source.subframe,image:source.image_id,heading,canvas,depths,events:a.frames[frame].events.clone()})
    }
}
impl NativeActorSlice {
    pub fn matches_pins(&self,pins:&NativeBaseBodyPins)->bool{&self.pins==pins}
    pub fn canvas(&self)->&Canvas{&self.canvas}
    pub fn sample(&self)->(BodyAction,u64,usize,u32,u32,u8){(self.action,self.elapsed_ms,self.frame,self.image,self.subframe,self.heading)}
    pub fn depths(&self)->&[(NativeDrawKey,SpriteBlockReference)]{&self.depths}
    pub fn events(&self)->&[FrameEvent]{&self.events}
    pub fn matches_job(&self,job:&RendererJob)->bool {self.plan==*job.source_plan() && self.observation.data()==job.admitted_observation().data()
        && Arc::ptr_eq(&self.cancel,&job.cancellation_token()) && !self.cancel.load(Ordering::Acquire)}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn native_integer_frame_time_is_not_rational_accumulation(){assert_eq!(catalog_frame_ms(4,24).unwrap(),166);assert_eq!(catalog_frame_ms(2,24).unwrap(),83);assert_eq!(catalog_frame_ms(4,36).unwrap(),111);assert!(catalog_frame_ms(0,24).is_err());assert!(catalog_frame_ms(1,0).is_err());assert!(catalog_frame_ms(1,1001).is_err());assert!(catalog_frame_ms(1000,1).is_err());}
    #[test]fn native_single_interval_boundaries_and_unknown_loop(){assert_eq!(relative_frame(&[166;4],165).unwrap(),0);assert_eq!(relative_frame(&[166;4],166).unwrap(),1);assert_eq!(relative_frame(&[166;4],663).unwrap(),3);assert!(relative_frame(&[166;4],664).is_err());assert!(relative_frame(&[166;4],u64::MAX).is_err());assert!(relative_frame(&[],0).is_err());assert!(relative_frame(&[0],0).is_err());}
}
