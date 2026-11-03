//! Finite explicit static-source terrain and player fragment interleaving.
use super::{native_appearance_inspection as a,native_appearance_layer_study::Submission,native_resources::{NativeStaticAssets,NativeAssetPlan},world_map::{WorldCell,PixelPoint,ViewProjection},world_depth::NativeDrawCell,world_raster::{Raster,PreparedTile,Blend},world_roofs::Shade,compositor::Canvas};
use anyhow::{Result,ensure,Context};use std::{collections::BTreeMap,sync::atomic::{AtomicBool,Ordering}};
pub(crate) struct Inputs{pub placement:Option<WorldCell>,pub static_source_roofs:Option<bool>,pub terrain_first:Option<bool>}
pub(crate) struct Terrain<'a>{binding:a::Binding,assets:&'a NativeStaticAssets,tiles:BTreeMap<(u16,u8),PreparedTile>,draws:Vec<(NativeDrawCell,Option<u8>,(u16,u8))>,left:usize,retired:AtomicBool}
pub(crate) struct Frame{pub canvas:Canvas,pub body_visible_pixels:usize,pub visible_terrain_references:usize}
impl<'a> Terrain<'a>{
 pub fn new(enabled:bool,b:&a::Binding,expected:&a::Binding,now:i64,assets:&'a NativeStaticAssets,plan:&NativeAssetPlan,budget:usize)->Result<Self>{ensure!(enabled&&b==expected&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff)&&budget>0&&budget<=8&&assets.verified_plan()==plan&&plan.map_id==564,"Terrain study disabled/owner/plan/cutoff/budget mismatch");let draws=assets.scene_terrain()?;let tiles=assets.scene_tiles().iter().map(|(key,tile)|Ok((*key,PreparedTile::new(tile.clone())?))).collect::<Result<BTreeMap<_,_>>>()?;Ok(Self{binding:b.clone(),assets,tiles,draws,left:budget,retired:AtomicBool::new(false)})}
 pub fn draw_counts(&self)->(usize,usize){(self.draws.iter().filter(|(_,d,_)|d.is_none()).count(),self.draws.iter().filter(|(_,d,_)|d.is_some()).count())}
 pub fn render(&mut self,b:&a::Binding,now:i64,player:&[Submission<'_>],input:Inputs)->Result<Frame>{
  ensure!(!self.retired.load(Ordering::Acquire)&&self.left>0&&self.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff),"Terrain study retired/owner/cutoff/budget mismatch");let cell=input.placement.context("Player conditional terrain placement unknown")?;let first=input.terrain_first.context("Terrain-player insertion history unknown")?;ensure!(input.static_source_roofs==Some(true)&&self.assets.geometry().contains(cell),"Actor-driven roof state or foreign terrain cell unqualified");ensure!(player.iter().map(|s|s.kind).collect::<Vec<_>>()==["body","catalog105","shadow"]&&player.iter().map(|s|s.flags).collect::<Vec<_>>()==[0,0,5],"Complete neutral conditional player branch required");
  let view=ViewProjection{anchor_cell:cell,anchor_pixel:PixelPoint{x:256,y:250}};let origin=NativeDrawCell::from_world(cell).context("Player half-cell overflow")?;let mut canvas=Canvas::new(512,384)?;let mut raster=Raster::new(view,&canvas);let mut visible=0;
  if !first{for s in player{raster.sprite(origin,s.frame,if s.kind=="body"{1}else{0},if s.kind=="shadow"{Blend::Shadow}else{Blend::Over})?;}}
  for(anchor,depth,key)in &self.draws{visible+=usize::from(raster.tile_roof(*anchor,*depth,self.tiles.get(key).context("Selected terrain tile absent")?,Shade::Native)?);}
  if first{for s in player{raster.sprite(origin,s.frame,if s.kind=="body"{1}else{0},if s.kind=="shadow"{Blend::Shadow}else{Blend::Over})?;}}
  let mask=raster.paint(&mut canvas,&self.retired,1)?.remove(0);let mut pixels=0;for y in 0..384{for x in 0..512{pixels+=usize::from(mask.contains(x,y));}}self.left-=1;Ok(Frame{canvas,body_visible_pixels:pixels,visible_terrain_references:visible})
 }
 pub fn retire(&self){self.retired.store(true,Ordering::Release);}
}
impl Drop for Terrain<'_>{fn drop(&mut self){self.retire();}}
