//! Bounded original tile-pointer lookup and one actor Layer5 half-roof branch.
use super::{native_appearance_inspection::Binding,native_resources::{NativeStaticAssets,NativeAssetPlan},world_depth::NativeDrawCell,world_map::{WorldCell,PixelPoint,ViewProjection},world_roofs::Shade,world_raster::{Raster,PreparedTile},compositor::Canvas};
use anyhow::{Result,ensure,Context};use std::{collections::{BTreeMap,BTreeSet},cmp::Reverse,sync::atomic::AtomicBool};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct State{pub flags:u32,pub weight:u8,pub target:u16}
pub(crate) struct Inputs{pub local_actor_pointer_matches:Option<bool>,pub old_cell:Option<WorldCell>,pub new_cell:Option<WorldCell>,pub no_dynamic_cell_entries:Option<bool>,pub source_initialized_tiles:Option<bool>,pub scheduler_sample_ms:Option<u32>}
pub(crate) struct Tick{pub timer_removed_before_callback:Option<bool>,pub callback_group_matches:Option<bool>,pub dispatch_ms:Option<u32>,pub scheduler_sample_ms:Option<u32>}
#[derive(Clone,Copy,PartialEq,Eq)]enum Direction{Down,Up}
pub(crate) struct Model<'a>{binding:Binding,assets:&'a NativeStaticAssets,draws:Vec<(NativeDrawCell,Option<u8>,(u16,u8))>,tiles:BTreeMap<(u16,u8),PreparedTile>,states:Vec<State>,groups:Vec<Vec<usize>>,triggers:BTreeMap<WorldCellKey,Vec<(usize,u8)>>,cell:WorldCell,group:usize,due:Option<u32>,direction:Direction,left:usize,retired:bool,alias_keys:usize,alias_groups:usize,last_clock:u32}
type WorldCellKey=(i32,i32);
impl<'a> Model<'a>{
 pub fn new(enabled:bool,b:&Binding,expected:&Binding,now:i64,assets:&'a NativeStaticAssets,plan:&NativeAssetPlan,initial:WorldCell,group:usize,budget:usize,input:&Inputs)->Result<Self>{
  ensure!(enabled&&b==expected&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff)&&assets.verified_plan()==plan&&plan.map_id==564&&plan.sectors.len()==1&&budget>0&&budget<=16,"Roof disabled/owner/source/budget mismatch");
  ensure!(input.no_dynamic_cell_entries==Some(true)&&input.source_initialized_tiles==Some(true),"Original initial resident tile flags/dynamic entries unknown");
  let geometry=assets.geometry();let binding=geometry.bindings()[0];let sector=geometry.sector(binding.key).context("Roof source sector absent")?;ensure!(sector.overlays.is_empty(),"Ground-overlay resident lookup not audited");
  let draws=assets.scene_terrain()?;let raw=assets.scene_roof_raw(&AtomicBool::new(false))?;let tiles=assets.scene_tiles().iter().map(|(k,t)|{let p=PreparedTile::new(t.clone())?;Ok((*k,if let Some(r)=raw.get(k){p.with_roof_raw(r.clone())?}else{p}))}).collect::<Result<BTreeMap<_,_>>>()?;
  let origin=NativeDrawCell::from_world(WorldCell{x:binding.key.world_origin().0,y:binding.key.world_origin().1}).context("Roof origin overflow")?;
  let mut resident:BTreeMap<(i32,i32,u16,u8),Vec<usize>>=BTreeMap::new();let mut states=Vec::new();
  for(i,(cell,depth,key))in draws.iter().enumerate(){let kind=tiles[key].kind() as u8;let flags=if kind&0x10!=0{8}else if kind&4!=0{0x100}else{u32::from(kind&0x20)<<2};states.push(State{flags,weight:0,target:0});if depth.is_some(){resident.entry((cell.half_x,cell.y,key.0,key.1)).or_default().push(i);}}
  // 483770 insertion sorts by depth, newest before old on equality. The
  // subsequent 433210/432F70/4330D0 search compares tile pointer, not depth.
  for entries in resident.values_mut(){entries.sort_by_key(|i|(draws[*i].1,Reverse(*i)));}
  let mut groups=Vec::new();let mut owners:BTreeMap<usize,BTreeSet<usize>>=BTreeMap::new();
  for(g,source)in sector.objects.iter().enumerate(){let mut refs=Vec::new();for t in &source.tiles{ensure!(t.unknown==0&&t.tile_id>=0,"Opaque/unresolved Native tile pointer unqualified");let k=(origin.half_x+i32::from(t.x),origin.y+i32::from(t.y),t.tile_id as u16,t.subtile);if let Some(list)=resident.get(&k){let id=list[0];refs.push(id);owners.entry(id).or_default().insert(g);}}groups.push(refs);}
  let alias_keys=owners.values().filter(|x|x.len()>1).count();let alias_groups=owners.values().filter(|x|x.len()>1).map(|x|x.len()).sum();
  let mut triggers:BTreeMap<WorldCellKey,Vec<(usize,u8)>>=BTreeMap::new();for r in &sector.opacity{if r.x>=128||r.y>=64{continue;}ensure!(usize::from(r.object_index)<groups.len()&&r.kind<=1,"Roof Layer5 index/kind unknown");triggers.entry((origin.half_x+i32::from(r.x),origin.y+i32::from(r.y))).or_default().push((usize::from(r.object_index),r.kind));}
  ensure!(group<groups.len()&&!groups[group].is_empty()&&groups[group].iter().all(|i|matches!(states[*i].flags,0|8|0x80|0x100)),"Selected roof requires audited source base flags");let c=NativeDrawCell::from_world(initial).context("Initial cell overflow")?;ensure!(geometry.contains(initial)&&!triggers.contains_key(&(c.half_x,c.y)),"Initial actor roof set must be explicitly empty");
  let last_clock=input.scheduler_sample_ms.context("Initial roof clock unknown")?;
  Ok(Self{binding:b.clone(),assets,draws,tiles,states,groups,triggers,cell:initial,group,due:None,direction:Direction::Down,left:budget,retired:false,alias_keys,alias_groups,last_clock})
 }
 fn active(&self,b:&Binding,now:i64)->Result<()>{ensure!(!self.retired&&self.left>0&&self.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff),"Roof retired/budget/full owner cutoff mismatch");Ok(())}
 fn records(&self,cell:WorldCell)->Result<Vec<(usize,u8)>>{let c=NativeDrawCell::from_world(cell).context("Actor half-cell overflow")?;Ok(self.triggers.get(&(c.half_x,c.y)).cloned().unwrap_or_default())}
 pub fn trigger(&self)->Result<WorldCell>{self.triggers.iter().find_map(|(&(x,y),v)|{if v==&vec![(self.group,0)]&&(x+0x8000)%2==0{Some(WorldCell{x:(x+0x8000)/2,y})}else{None}}).context("One exact source kind0 trigger required")}
 pub fn alias_counts(&self)->(usize,usize){(self.alias_keys,self.alias_groups)}
 pub fn alias_pair(&self)->Result<(usize,usize,usize)>{for a in 0..self.groups.len(){for b in a+1..self.groups.len(){if let Some(id)=self.groups[a].iter().find(|i|self.groups[b].contains(i)){return Ok((a,b,*id));}}}anyhow::bail!("Cross-group source alias absent")}
 pub fn states(&self)->Vec<State>{self.groups[self.group].iter().map(|i|self.states[*i]).collect()}
 pub fn read_group(&self,b:&Binding,now:i64,group:usize)->Result<Vec<(usize,State)>>{self.active(b,now)?;Ok(self.groups.get(group).context("Group outside source")?.iter().map(|id|(*id,self.states[*id])).collect())}
 pub fn due(&self,b:&Binding,now:i64)->Result<u32>{self.active(b,now)?;self.due.context("No pending roof callback")}
 pub fn clock(&self,b:&Binding,now:i64)->Result<u32>{self.active(b,now)?;Ok(self.last_clock)}
 pub fn move_actor(&mut self,b:&Binding,now:i64,input:&Inputs)->Result<()>{
  self.active(b,now)?;ensure!(input.local_actor_pointer_matches==Some(true)&&input.no_dynamic_cell_entries==Some(true)&&input.source_initialized_tiles==Some(true)&&input.old_cell==Some(self.cell),"Actor old cell/local pointer/resident state unknown");let new=input.new_cell.context("Actor new cell unknown")?;let sampled=input.scheduler_sample_ms.context("Roof write scheduler clock unknown")?;ensure!(sampled>=self.last_clock,"Backward/wrapping actor roof write clock unqualified");let due=sampled.checked_add(50).context("Roof clock wraps")?;ensure!(self.assets.geometry().contains(new),"Foreign actor cell");let old=self.records(self.cell)?;let target=self.records(new)?;
  ensure!((old.is_empty()&&target==vec![(self.group,0)]&&self.due.is_none())||(old==vec![(self.group,0)]&&target.is_empty()&&self.due.is_none()),"Only settled enter/exit single Half source group qualified");
  if target.is_empty(){ensure!(self.groups[self.group].iter().all(|id|self.states[*id].flags&1!=0&&self.states[*id].flags&0xc00==0),"Restore requires settled half state");for id in &self.groups[self.group]{let s=&mut self.states[*id];s.weight=17;s.target=0;s.flags=(s.flags&!0x400)|0x800;}self.direction=Direction::Up;}
  else{for id in &self.groups[self.group]{let s=&mut self.states[*id];if s.flags&0x100!=0{s.weight=16;}else{s.weight=31;}s.target=1;s.flags=(s.flags&!0x800)|0x400;}self.direction=Direction::Down;}
  self.cell=new;self.due=Some(due);self.last_clock=sampled;Ok(())
 }
 pub fn callback(&mut self,b:&Binding,now:i64,input:&Tick)->Result<()>{
  self.active(b,now)?;let due=self.due.context("No roof callback")?;ensure!(input.timer_removed_before_callback==Some(true)&&input.callback_group_matches==Some(true),"Roof callback removal/group unknown");let dispatch=input.dispatch_ms.context("Roof dispatch clock unknown")?;let sampled=input.scheduler_sample_ms.context("Roof reschedule fresh clock unknown")?;ensure!(dispatch>=due&&sampled>=dispatch,"Early/nonmonotonic roof callback");let next=sampled.checked_add(50).context("Roof callback clock wraps")?;
  let mut more=false;for id in &self.groups[self.group]{let s=&mut self.states[*id];match self.direction{Direction::Down=>{if s.flags&0x400==0{continue;}if s.weight>15{let weight=s.weight-4;if weight>=17{s.weight=weight;more=true;continue;}}s.weight=0;s.flags=(s.flags&0xffff_fbfc)|u32::from(s.target);},Direction::Up=>{if s.flags&0x800==0{continue;}s.weight+=4;let limit=if s.flags&0x100!=0||s.target&1!=0{16}else{31};if s.weight<=limit{more=true;}else{s.weight=0;s.target=0;s.flags=if limit==16{(s.flags&0xffff_f7fd)|1}else{s.flags&0xffff_f7fc};}}}}
  self.due=if more{Some(next)}else{None};self.last_clock=sampled;self.left-=1;Ok(())
 }
 pub fn render(&self,b:&Binding,now:i64,anchor:WorldCell)->Result<Canvas>{self.active(b,now)?;let mut canvas=Canvas::new(512,384)?;let view=ViewProjection{anchor_cell:anchor,anchor_pixel:PixelPoint{x:256,y:250}};let mut raster=Raster::new(view,&canvas);for(i,(cell,depth,key))in self.draws.iter().enumerate(){let s=self.states[i];let shade=if s.flags&0xc00!=0{Shade::Weight(s.weight)}else if s.flags&1!=0||s.flags&0x100!=0{Shade::Half}else{Shade::Native};raster.tile_roof(*cell,*depth,&self.tiles[key],shade)?;}raster.paint(&mut canvas,&AtomicBool::new(false),0)?;Ok(canvas)}
 pub fn retire(&mut self){self.retired=true;self.due=None;}
}
impl Drop for Model<'_>{fn drop(&mut self){self.retire();}}
#[path="native_roof_transition_study.rs"]pub(crate) mod multi;
