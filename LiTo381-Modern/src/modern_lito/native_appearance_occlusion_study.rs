//! Finite CPU occlusion with source fragment depth and explicit scene insertion.
use super::{native_appearance_inspection as a,native_appearance_layer_study::Submission,sprite::SpriteFrame,compositor::Canvas,world_raster::{Raster,Blend},world_map::{WorldCell,ViewProjection,PixelPoint},world_depth::NativeDrawCell};
use anyhow::{Result,ensure,Context};
use std::sync::atomic::{AtomicBool,Ordering};
pub(crate) struct Inputs{pub prop_delta:Option<(i32,i32)>,pub player_first:Option<bool>,pub prop_flags:Option<u16>}
pub(crate) struct Scene{binding:a::Binding,left:usize,retired:AtomicBool}
pub(crate) struct Frame{pub canvas:Canvas,pub body_visible_pixels:usize,pub prop_reference_count:usize}
impl Scene{
 pub fn new(enabled:bool,b:&a::Binding,expected:&a::Binding,now:i64,budget:usize)->Result<Self>{ensure!(enabled&&b==expected&&budget>0&&budget<=8&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff),"Conditional scene disabled/owner/cutoff/budget mismatch");Ok(Self{binding:b.clone(),left:budget,retired:AtomicBool::new(false)})}
 pub fn render<'s>(&mut self,b:&a::Binding,now:i64,player:&[Submission<'s>],prop:Option<&'s SpriteFrame>,input:Inputs)->Result<Frame>{
  ensure!(!self.retired.load(Ordering::Acquire)&&self.left>0&&self.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff),"Scene retired/exhausted/owner/time mismatch");
  let(delta_x,delta_y)=input.prop_delta.context("Conditional prop position unknown")?;let first=input.player_first.context("Cross-object insertion history unknown")?;ensure!((-2..=2).contains(&delta_x)&&(-2..=2).contains(&delta_y)&&input.prop_flags==Some(0),"Prop coordinate/flags unqualified");
  ensure!(player.iter().map(|s|s.kind).collect::<Vec<_>>()==["body","catalog105","shadow"]&&player.iter().map(|s|s.flags).collect::<Vec<_>>()==[0,0,5],"Only complete neutral conditional player branch");
  let prop_reference_count=if let Some(f)=prop{let refs=&f.fragments.as_ref().context("Prop source fragments unknown")?.references;ensure!(!refs.is_empty()&&refs.len()<=65536,"Prop frame missing/overbound");refs.len()}else{0};
  let anchor=WorldCell{x:32724,y:32784};let view=ViewProjection{anchor_cell:anchor,anchor_pixel:PixelPoint{x:160,y:210}};let player_origin=NativeDrawCell::from_world(anchor).context("Player origin overflow")?;let prop_origin=NativeDrawCell::from_world(WorldCell{x:anchor.x+delta_x,y:anchor.y+delta_y}).context("Prop origin overflow")?;
  let mut canvas=Canvas::new(320,300)?;let mut raster=Raster::new(view,&canvas);
  let submit_player=|raster:&mut Raster<'s>|->Result<()>{for s in player{raster.sprite(player_origin,s.frame,if s.kind=="body"{1}else{0},if s.kind=="shadow"{Blend::Shadow}else{Blend::Over})?;}Ok(())};
  if first{submit_player(&mut raster)?;if let Some(f)=prop{raster.sprite(prop_origin,f,0,Blend::Over)?;}}else{if let Some(f)=prop{raster.sprite(prop_origin,f,0,Blend::Over)?;}submit_player(&mut raster)?;}
  let mask=raster.paint(&mut canvas,&self.retired,1)?.remove(0);let mut visible=0;for y in 0..300{for x in 0..320{visible+=usize::from(mask.contains(x,y));}}
  self.left-=1;Ok(Frame{canvas,body_visible_pixels:visible,prop_reference_count})
 }
 pub fn retire(&self){self.retired.store(true,Ordering::Release);}
}
impl Drop for Scene{fn drop(&mut self){self.retire();}}
