//! Finite conditional original 4E0200/4E7000 submission and half-cell CPU study.
//! Explicit neutral flags/loader state are assumptions, never runtime observations.
use super::{native_appearance_inspection as a,sprite::{self,DecodedSprite,SpriteFrame},compositor::Canvas,world_map::{ViewProjection,WorldCell,PixelPoint},world_depth::NativeDrawCell,world_raster::{Raster,Blend}};
use anyhow::{Result,ensure,Context};
use sha2::{Digest,Sha256};
use std::sync::atomic::{AtomicBool,Ordering};

pub(crate) struct Inputs{pub draw_branch_reached:Option<bool>,pub actor_flags:Option<u16>,pub layer_flags:Option<u16>,pub base_shadow_route:Option<bool>,pub all_layer_pointers_present:Option<bool>}
pub(crate) struct Asset{pub name:String,pub bank:u32,pub sha256:[u8;32],pub sprite:DecodedSprite}
impl Asset{
 pub fn new(name:String,bank:u32,bytes:&[u8],sha256:[u8;32],frames:usize)->Result<Self>{
  ensure!(bytes.len()<=1024*1024&&<[u8;32]>::from(Sha256::digest(bytes))==sha256,"Pinned layer resource digest/budget mismatch");
  let sprite=sprite::decode_world_spr_with_limits(bytes.strip_prefix(b"SPR_0").unwrap_or(bytes),sprite::SpriteLimits{max_input_bytes:1024*1024,max_frames:255,max_blocks:8192,max_block_references:65536,max_frame_dimension:2048,max_total_pixels:4*1024*1024})?;
  ensure!(sprite.frames.len()==frames,"Pinned layer frame count mismatch");Ok(Self{name,bank,sha256,sprite})
 }
}
pub(crate) struct Submission<'a>{pub kind:&'static str,pub asset:&'a Asset,pub frame:&'a SpriteFrame,pub flags:u16}
pub(crate) struct Sample{pub canvas:Canvas,pub submission_order:Vec<&'static str>,pub fragment_count:Vec<usize>,pub renderer_ready:bool,pub native_framebuffer_parity_qualified:bool}
pub(crate) struct Model{binding:a::Binding,budget:usize,retired:AtomicBool}
impl Model{
 pub fn new(enabled:bool,b:&a::Binding,expected:&a::Binding,now:i64,budget:usize)->Result<Self>{
  ensure!(enabled&&b==expected&&b.character_type==3&&b.sex==0&&b.class==734&&b.effective_equipped&&b.retained_type==2,"Layer study default off or bounded historical Class mismatch");
  ensure!(now>=b.observed&&(b.cutoff==-1||now<b.cutoff)&&budget>0&&budget<=64,"Layer study cutoff/budget");
  Ok(Self{binding:b.clone(),budget,retired:AtomicBool::new(false)})
 }
 fn active(&self,b:&a::Binding,now:i64)->Result<()>{ensure!(!self.retired.load(Ordering::Acquire)&&self.budget>0&&self.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff),"Layer study retired/exhausted/owner cutoff mismatch");Ok(())}
 pub fn submissions<'a>(&self,b:&a::Binding,now:i64,bank:u32,subframe:usize,body:&'a Asset,shadow:&'a Asset,layer:&'a Asset,input:Inputs)->Result<Vec<Submission<'a>>>{
  ensure!((72..80).contains(&bank)||(88..96).contains(&bank),"Same directed attack47 bank required");
  self.directed_submissions(b,now,bank,subframe,body,shadow,layer,input)
 }
 pub fn idle_submissions<'a>(&self,b:&a::Binding,now:i64,bank:u32,subframe:usize,body:&'a Asset,shadow:&'a Asset,layer:&'a Asset,input:Inputs)->Result<Vec<Submission<'a>>>{
  ensure!((80..88).contains(&bank)&&subframe==0,"Only audited first idle49 directed bank/subframe required");
  self.directed_submissions(b,now,bank,subframe,body,shadow,layer,input)
 }
 pub fn idle_cycle_submissions<'a>(&self,b:&a::Binding,now:i64,bank:u32,subframe:usize,body:&'a Asset,shadow:&'a Asset,layer:&'a Asset,input:Inputs)->Result<Vec<Submission<'a>>>{
  ensure!((80..88).contains(&bank)&&subframe<=8,"Audited idle49 directed cycle required");
  self.directed_submissions(b,now,bank,subframe,body,shadow,layer,input)
 }
 fn directed_submissions<'a>(&self,b:&a::Binding,now:i64,bank:u32,subframe:usize,body:&'a Asset,shadow:&'a Asset,layer:&'a Asset,input:Inputs)->Result<Vec<Submission<'a>>>{
  self.active(b,now)?;
  ensure!(input.draw_branch_reached==Some(true)&&input.base_shadow_route==Some(true)&&input.all_layer_pointers_present==Some(true),"Original draw/base shadow/105 loader branch unknown");
  let flags=input.actor_flags.context("Actor flags unknown")?;let layer_flags=input.layer_flags.context("Catalog105 draw flags unknown")?;
  // Only neutral paint and the source-proven bit2 non-submission branch.
  // Other high flags and player/effect/override routes remain unqualified.
  ensure!(matches!(flags,0|2)&&layer_flags==0,"Unqualified native layer flags");
  ensure!(body.bank==bank&&shadow.bank==bank&&layer.bank==bank&&body.name==format!("3476-{bank}.spr")&&shadow.name==format!("3477-{bank}.spr")&&layer.name==format!("6260-{bank}.spr"),"Same directed appearance bank required");
  let frame=|asset:&'a Asset|->Result<&'a SpriteFrame>{let f=asset.sprite.frames.get(subframe).context("Shared actor subframe outside layer member")?;ensure!(f.fragments.is_some(),"Actual source fragments required");Ok(f)};
  let mut list=vec![Submission{kind:"body",asset:body,frame:frame(body)?,flags}];
  if flags&2==0{list.push(Submission{kind:"catalog105",asset:layer,frame:frame(layer)?,flags:layer_flags});}
  // Pointer-qualified shadow draws with (actorflags&0xfff)|5. A skipped105
  // layer does not mean absence of the loaded pointer or a changed subframe.
  list.push(Submission{kind:"shadow",asset:shadow,frame:frame(shadow)?,flags:(flags&0xfff)|5});
  Ok(list)
 }
 pub fn render(&mut self,b:&a::Binding,now:i64,list:&[Submission<'_>])->Result<Sample>{
  self.active(b,now)?;ensure!(list.len()==3&&list.iter().map(|s|s.kind).collect::<Vec<_>>()==["body","catalog105","shadow"]&&list.iter().map(|s|s.flags).collect::<Vec<_>>()==[0,0,5],"CPU paint only explicit neutral source branch");
  let view=ViewProjection{anchor_cell:WorldCell{x:32724,y:32784},anchor_pixel:PixelPoint{x:100,y:140}};let origin=NativeDrawCell::from_world(view.anchor_cell).context("Conditional anchor overflow")?;
  let mut canvas=Canvas::new(200,200)?;let mut raster=Raster::new(view,&canvas);let mut counts=Vec::new();
  for s in list{let count=s.frame.fragments.as_ref().context("Layer fragments absent")?.references.len();ensure!(count<=65536,"Layer references overbound");counts.push(count);raster.sprite(origin,s.frame,if s.kind=="body"{1}else{0},if s.kind=="shadow"{Blend::Shadow}else{Blend::Over})?;}
  raster.paint(&mut canvas,&self.retired,1)?;self.active(b,now)?;self.budget-=1;
  Ok(Sample{canvas,submission_order:list.iter().map(|s|s.kind).collect(),fragment_count:counts,renderer_ready:false,native_framebuffer_parity_qualified:false})
 }
 pub fn retire(&self){self.retired.store(true,Ordering::Release);}
}
impl Drop for Model{fn drop(&mut self){self.retire();}}
