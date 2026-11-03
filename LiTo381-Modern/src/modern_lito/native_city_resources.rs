//! Separate pinned whole-city profile: complete geometry/all selected slot validation,
//! implicit floor and compact non-floor row index. No full-map draw Vec.
//! Source insertion is canonical snapshot policy; no native streaming/spawn grant.
use super::*;
use crate::modern_lito::{world_depth::{NativeDrawCell,NativeDrawKey},world_map::{PixelPoint,ViewProjection},native_multisector_scene::Bounds};
use std::{mem::size_of,sync::atomic::AtomicUsize};
#[path="native_city_tile_cache.rs"]mod tile_cache;
pub use tile_cache::{CacheMetrics,CacheProgress,DEFAULT_CACHE_BYTES};
use tile_cache::{TileStorage,EncodedMember};
pub const INDEX_PAYLOAD_LIMIT:usize=8<<20;
pub const FRAME_FRAGMENT_LIMIT:usize=262144;
const MAX_INDEX_ROWS:usize=4096;
#[derive(Default)]pub struct LoadTrace{pub phase:AtomicUsize,pub checkpoints:AtomicUsize,pub validated_slots:AtomicUsize}
fn check(cancel:&AtomicBool,trace:&LoadTrace)->Result<()>{trace.checkpoints.fetch_add(1,Ordering::Relaxed);active(cancel)}
#[derive(Clone,Copy)]struct RowItem{insertion:u32,entry:u16,sector:u8,x:u8,y:u8,slot:u8,depth:u8}
#[derive(Clone,Copy,Debug,serde::Serialize)]pub struct Metrics{pub source_references:usize,pub non_floor_references:usize,pub index_rows:usize,pub index_payload_capacity_bytes:usize,pub row_item_bytes:usize,pub decoded_bytes:usize,pub decoded_slots:usize,pub encoded_source_bytes_validated:usize,pub qualified_slots:usize,pub whole_required_decoded_bytes:usize,pub encoded_resident_bytes:usize,pub paged:bool}
#[derive(Clone,Copy,Debug,serde::Serialize)]pub struct FrameMetrics{pub visible_fragments:usize,pub candidate_fragments:usize,pub visited_index_rows:usize,pub fragment_capacity_bytes:usize,pub fragment_limit:usize}
struct Fragment{key:NativeDrawKey,point:PixelPoint,tile:(u16,u8)}
pub struct CityAssets{plan:NativeAssetPlan,map:WorldMap,bindings:Vec<super::super::world_map::SectorBinding>,rows:BTreeMap<i64,Vec<RowItem>>,tiles:TileStorage,anchor:WorldCell,bounds:Bounds,metrics:Metrics}
/// A cancellable source iterator, preserving floor/overlay/group/tile order and
/// refusing negative object IDs. Geometry owns these records; no temporary Vec.
fn visit(map:&WorldMap,cancel:&AtomicBool,trace:&LoadTrace,mut f:impl FnMut(usize,u8,u8,Option<u8>,u16,u8)->Result<()>)->Result<()> {
 for(si,b)in map.bindings().iter().enumerate(){let s=map.sector(b.key).context("city sector missing")?;ensure!(s.floor.len()==8192&&s.objects.len()<=65536,"city sector shape/group budget");let mut count=0usize;
  for(i,t)in s.floor.iter().enumerate(){check(cancel,trace)?;f(si,(i%128)as u8,(i/128)as u8,None,t.tile_id,t.subtile)?;count+=1;}
  for t in &s.overlays{check(cancel,trace)?;f(si,t.x,t.y,Some(0),t.tile.tile_id,t.tile.subtile)?;count+=1;ensure!(count<=FRAME_FRAGMENT_LIMIT,"city per-sector source budget");}
  for g in &s.objects{for t in &g.tiles{check(cancel,trace)?;let r=t.shared_tile_reference().context("unsupported negative city object Tile ID")?;f(si,t.x,t.y,Some(t.layer),r.tile_id,r.subtile)?;count+=1;ensure!(count<=FRAME_FRAGMENT_LIMIT,"city per-sector source budget");}}
 }Ok(())
}
impl CityAssets{
 /// Backward-compatible R483 city inspection entry; its exact-size guards remain.
 pub fn load(sources:&NativeSources,plan:&NativeAssetPlan,limits:Limits,cancel:&AtomicBool,trace:&LoadTrace)->Result<Self>{
  let n=match plan.map_id{534=>16,535=>25,_=>anyhow::bail!("city profile requires exact534/535 identity")};
  ensure!(plan.sectors.len()==n,"city whole sorted sector plan required");
  Self::load_whole(sources,plan,limits,cancel,trace)
 }
 /// Generic row inspection entry requires the existing exact LTtext/R474
 /// whole-directory authority loader, rather than accepting an ID/count assertion.
 /// Catalog IO is the same bounded existing loader; geometry/pixels are cancellable.
 pub fn load_catalog(inputs:&super::super::native_multisector_resources::Inputs,mode:u8,map_id:u32,limits:Limits,cancel:&AtomicBool,trace:&LoadTrace)->Result<Self>{
  active(cancel)?;let(_,maps,_)=super::super::native_multisector_resources::load(inputs)?;active(cancel)?;
  let b=maps.iter().find(|b|b.mode==mode&&b.plan.map_id==map_id).context("row profile exact qualified mode/map absent")?;
  Self::load_whole(&b.sources,&b.plan,limits,cancel,trace)
 }
 /// Opt-in residence profile, bound to the same complete original catalog.
 /// Complete qualification is retained even when RGBA residence is paged.
 pub fn load_catalog_bounded(inputs:&super::super::native_multisector_resources::Inputs,mode:u8,map_id:u32,limits:Limits,cache_bytes:usize,cancel:&AtomicBool,trace:&LoadTrace)->Result<Self>{
  active(cancel)?;let(_,maps,_)=super::super::native_multisector_resources::load(inputs)?;active(cancel)?;
  let b=maps.iter().find(|b|b.mode==mode&&b.plan.map_id==map_id).context("row profile exact qualified mode/map absent")?;
  Self::load_profile(&b.sources,&b.plan,limits,Some(cache_bytes),cancel,trace)
 }
 pub(crate) fn load_whole_bounded(sources:&NativeSources,plan:&NativeAssetPlan,limits:Limits,cancel:&AtomicBool,trace:&LoadTrace)->Result<Self>{
  Self::load_profile(sources,plan,limits,Some(DEFAULT_CACHE_BYTES.min(limits.decoded_bytes)),cancel,trace)
 }
 /// Crate-only backend; player callers are bound by the exact original complete
 /// sector authority loader before preparing a private generation's LoadJob.
 pub(crate) fn load_whole(sources:&NativeSources,plan:&NativeAssetPlan,limits:Limits,cancel:&AtomicBool,trace:&LoadTrace)->Result<Self>{
  Self::load_profile(sources,plan,limits,None,cancel,trace)
 }
 fn load_profile(sources:&NativeSources,plan:&NativeAssetPlan,limits:Limits,cache_bytes:Option<usize>,cancel:&AtomicBool,trace:&LoadTrace)->Result<Self>{
  check(cancel,trace)?;let limits=limits.validate()?;
  if let Some(n)=cache_bytes{ensure!((24*24*4..=DEFAULT_CACHE_BYTES.min(limits.decoded_bytes)).contains(&n),"city cache residence limit");}
  ensure!(plan.namespace==ResourceNamespace::NativeLitoV1&&plan.canonical_sha256!=[0;32],"city source identity");
  ensure!(!plan.sectors.is_empty()&&plan.sectors.len()<=64&&plan.sectors.windows(2).all(|p|p[0].key<p[1].key),"row profile whole sorted sector plan required");
  ensure!(plan.archives.len()==4&&plan.archives.iter().zip([ArchiveRole::TileIndex,ArchiveRole::TilePak,ArchiveRole::SpriteIndex,ArchiveRole::SpritePak]).all(|(p,r)|p.role==r),"city exact archive roles");
  for p in &plan.archives{pin_valid(p.pin,p.role.limit(limits))?;}
  let mut total=0u64;for p in &plan.sectors{pin_valid(p.pin,limits.sector_bytes)?;ensure!(p.key.map_id==plan.map_id,"city sector identity");total=total.checked_add(p.pin.bytes).context("city source size overflow")?;ensure!(total<=limits.map_bytes as u64,"city input byte budget");}
  trace.phase.store(1,Ordering::Release);let mut inputs=Vec::new();
  for p in &plan.sectors{check(cancel,trace)?;let k=p.key;let mut file=PinnedFile::open(&sources.map_root,&[k.map_id.to_string()],&format!("{:04x}{:04x}.s32",k.x,k.y),p.pin,limits.sector_bytes,cancel)?;inputs.push(SectorInput{key:k.geometry_key(),bytes:file.read_range(0,p.pin.bytes as usize,cancel)?});}
  trace.phase.store(2,Ordering::Release);let map=WorldMap::from_native_sector_bytes(inputs,cancel)?;check(cancel,trace)?;ensure!(map.canonical_digest()==plan.canonical_sha256,"city whole canonical geometry differs");
  let bindings=map.bindings();let(ox,oy)=bindings[0].key.world_origin();let anchor=WorldCell{x:ox,y:oy};let mut bounds=Bounds{left:i32::MAX,top:i32::MAX,right:i32::MIN,bottom:i32::MIN};
  for b in &bindings{let(x,y)=b.key.world_origin();let(dx,dy)=(i64::from(x)-i64::from(ox),i64::from(y)-i64::from(oy));let px=i32::try_from(24*(dx+dy))?;let py=i32::try_from(12*(-dx+dy))?;bounds.left=bounds.left.min(px);bounds.top=bounds.top.min(py);bounds.right=bounds.right.max(px.checked_add(3072).context("city extent X")?);bounds.bottom=bounds.bottom.max(py.checked_add(1536).context("city extent Y")?);}
  trace.phase.store(3,Ordering::Release);let mut counts:BTreeMap<i64,usize>=BTreeMap::new();let mut required:BTreeMap<u16,BTreeSet<usize>>=BTreeMap::new();let mut source_references=0usize;let mut non_floor_references=0usize;
  visit(&map,cancel,trace,|si,x,y,depth,id,slot|{source_references+=1;required.entry(id).or_default().insert(usize::from(slot));if depth.is_some(){non_floor_references+=1;ensure!(non_floor_references.checked_mul(size_of::<RowItem>()).is_some_and(|b|b<=INDEX_PAYLOAD_LIMIT),"city row-index payload budget");let(wx,wy)=bindings[si].key.world_origin();let origin=NativeDrawCell::from_world(WorldCell{x:wx,y:wy}).context("city origin")?;let c=NativeDrawCell{half_x:origin.half_x.checked_add(i32::from(x)).context("city half X")?,y:origin.y.checked_add(i32::from(y)).context("city Y")?};*counts.entry(c.scan_row()).or_default()+=1;ensure!(counts.len()<=MAX_INDEX_ROWS,"city index row budget");}Ok(())})?;
  let mut rows:BTreeMap<i64,Vec<RowItem>>=BTreeMap::new();for(row,count)in counts{check(cancel,trace)?;let mut v=Vec::new();v.try_reserve_exact(count)?;rows.insert(row,v);}
  let index_payload_capacity_bytes=rows.values().map(|v|v.capacity()*size_of::<RowItem>()).sum::<usize>();ensure!(index_payload_capacity_bytes<=INDEX_PAYLOAD_LIMIT,"city actual index capacity budget");
  let mut insertion=0u32;visit(&map,cancel,trace,|si,x,y,depth,id,slot|{if let Some(depth)=depth{insertion=insertion.checked_add(1).context("city insertion overflow")?;let(wx,wy)=bindings[si].key.world_origin();let o=NativeDrawCell::from_world(WorldCell{x:wx,y:wy}).context("city origin")?;let c=NativeDrawCell{half_x:o.half_x+i32::from(x),y:o.y+i32::from(y)};rows.get_mut(&c.scan_row()).context("city counted row missing")?.push(RowItem{insertion,entry:id,sector:u8::try_from(si)?,x,y,slot,depth});}Ok(())})?;
  let qualified_slots=required.values().try_fold(0usize,|n,s|n.checked_add(s.len()).context("city qualified count overflow"))?;
  let whole_required_decoded_bytes=qualified_slots.checked_mul(24*24*4).context("city qualified pixels overflow")?;
  // Residence is the caller's bound; the global decode ceiling is independent.
  let paged=cache_bytes.is_some_and(|capacity|whole_required_decoded_bytes>capacity);
  trace.phase.store(4,Ordering::Release);let mut archive=NativeArchive::open(&sources.archive_root,plan.archives[0],plan.archives[1],limits,cancel)?;let mut tiles=BTreeMap::new();let mut encoded_members=BTreeMap::new();let mut decoded_bytes=0usize;let mut encoded_bytes=0usize;
  for(id,slots)in required{check(cancel,trace)?;let raw=archive.member(&format!("{id}.til"),cancel)?;encoded_bytes=encoded_bytes.checked_add(raw.len()).context("city encoded overflow")?;ensure!(encoded_bytes<=limits.map_bytes,"city encoded source budget");let remaining=limits.decoded_bytes.checked_sub(decoded_bytes).context("city decoded exhausted")?;if !paged{ensure!(slots.len().checked_mul(24*24*4).is_some_and(|b|b<=remaining),"city selected pixel budget");}let input=ValidatedTileInput::new_with_empty_tail(&raw,TileLimits{max_input_bytes:limits.entry_bytes,max_total_pixels:if paged{24*24}else{remaining/4},..Default::default()})?;
   for slot in &slots{check(cancel,trace)?;let t=input.decode_selected(&BTreeSet::from([*slot]),false)?.remove(slot).context("city selected tile absent")?;trace.validated_slots.fetch_add(1,Ordering::Release);if paged{ensure!(t.rgba.len()==24*24*4&&t.rgba.capacity()==24*24*4,"city qualified pixel allocation shape");drop(t);}else{decoded_bytes=decoded_bytes.checked_add(t.rgba.len()).context("city pixels overflow")?;ensure!(decoded_bytes<=limits.decoded_bytes,"city decoded budget");tiles.insert((id,u8::try_from(*slot)?),t);}}
   drop(input);if paged{encoded_members.insert(id,EncodedMember{raw,qualified:slots});}
  }
  let _sprites=NativeArchive::open(&sources.archive_root,plan.archives[2],plan.archives[3],limits,cancel)?;check(cancel,trace)?;
  let metrics=Metrics{source_references,non_floor_references,index_rows:rows.len(),index_payload_capacity_bytes,row_item_bytes:size_of::<RowItem>(),decoded_bytes,decoded_slots:tiles.len(),encoded_source_bytes_validated:encoded_bytes,qualified_slots,whole_required_decoded_bytes,encoded_resident_bytes:if paged{encoded_bytes}else{0},paged};
  let tiles=if paged{TileStorage::paged(encoded_members,cache_bytes.unwrap(),qualified_slots,encoded_bytes)?}else{TileStorage::Eager(tiles)};trace.phase.store(5,Ordering::Release);
  Ok(Self{plan:plan.clone(),map,bindings,rows,tiles,anchor,bounds,metrics})
 }
 pub fn verified_plan(&self)->&NativeAssetPlan{&self.plan}pub fn geometry(&self)->&WorldMap{&self.map}pub fn metrics(&self)->Metrics{let mut m=self.metrics;if let Some(c)=self.cache_metrics(){m.decoded_bytes=c.resident_decoded_bytes;m.decoded_slots=c.resident_slots;}m}pub fn bounds(&self)->Bounds{self.bounds}pub fn anchor(&self)->WorldCell{self.anchor}
 pub fn cache_metrics(&self)->Option<CacheMetrics>{self.tiles.metrics()}
 pub fn cache_progress(&self)->Option<std::sync::Arc<CacheProgress>>{self.tiles.progress()}
 pub fn initial_camera(&self)->CameraRect{CameraRect{x:((i64::from(self.bounds.left)+i64::from(self.bounds.right)-800)/2)as i32,y:((i64::from(self.bounds.top)+i64::from(self.bounds.bottom)-600)/2)as i32,width:800,height:600}}
 pub fn pan_camera(&self,c:CameraRect,dx:i32,dy:i32)->Result<CameraRect>{ensure!(c.width>0&&c.height>0&&c.width<=4096&&c.height<=4096,"city camera dimensions");Ok(CameraRect{x:i32::try_from((i64::from(c.x)+i64::from(dx)).clamp(i64::from(self.bounds.left),(i64::from(self.bounds.right)-i64::from(c.width)).max(i64::from(self.bounds.left))))?,y:i32::try_from((i64::from(c.y)+i64::from(dy)).clamp(i64::from(self.bounds.top),(i64::from(self.bounds.bottom)-i64::from(c.height)).max(i64::from(self.bounds.top))))?,..c})}
 pub fn render(&self,c:CameraRect,cancel:&AtomicBool)->Result<Canvas>{Ok(self.render_measured(c,cancel)?.0)}
 pub fn render_measured(&self,camera:CameraRect,cancel:&AtomicBool)->Result<(Canvas,FrameMetrics)>{
  self.render_with_fragment_limit(camera,FRAME_FRAGMENT_LIMIT,cancel)
 }
 /// Caller may tighten the frame limit; it cannot expand the existing bound.
 pub fn render_with_fragment_limit(&self,camera:CameraRect,limit:usize,cancel:&AtomicBool)->Result<(Canvas,FrameMetrics)>{
  active(cancel)?;ensure!(limit>0&&limit<=FRAME_FRAGMENT_LIMIT,"city frame fragment limit invalid");ensure!(camera.width>0&&camera.height>0&&camera.width<=4096&&camera.height<=4096,"city frame camera budget");let view=ViewProjection{anchor_cell:self.anchor,anchor_pixel:PixelPoint{x:24i32.checked_sub(camera.x).context("city camera X")?,y:768i32.checked_sub(camera.y).context("city camera Y")?}};let band=NativeDrawCell::visible_rows(view,camera.height).context("city visible row band")?;let mut fragments=Vec::new();let mut candidates=0usize;
  let mut append=|cell:NativeDrawCell,key:NativeDrawKey,tile:(u16,u8)|->Result<()>{active(cancel)?;candidates+=1;let point=cell.screen_origin(view).context("city visible projection")?;if i64::from(point.x)+24>0&&i64::from(point.y)+24>0&&i64::from(point.x)<i64::from(camera.width)&&i64::from(point.y)<i64::from(camera.height){ensure!(fragments.len()<limit,"city visible fragment budget");fragments.push(Fragment{key,point,tile});}Ok(())};
  for b in &self.bindings{active(cancel)?;let(wx,wy)=b.key.world_origin();let o=NativeDrawCell::from_world(WorldCell{x:wx,y:wy}).context("city floor origin")?;let lo=*band.start()-o.scan_row();let hi=*band.end()-o.scan_row();let s=self.map.sector(b.key).context("city floor sector")?;
   for y in 0..64i64{active(cancel)?;let start=((2*y-hi).max(0)+1).div_euclid(2)*2;let end=(2*y-lo).min(126);if start>end{continue;}for even in (start..=end).step_by(2){for x in [even,even+1]{let t=s.floor[(y*128+x)as usize];let cell=NativeDrawCell{half_x:o.half_x+i32::try_from(x)?,y:o.y+i32::try_from(y)?};append(cell,cell.floor_key(),(t.tile_id,t.subtile))?;}}}
  }
  let mut visited_index_rows=0usize;for(_,items)in self.rows.range(band){active(cancel)?;visited_index_rows+=1;for r in items{let(wx,wy)=self.bindings[usize::from(r.sector)].key.world_origin();let o=NativeDrawCell::from_world(WorldCell{x:wx,y:wy}).context("city indexed origin")?;let cell=NativeDrawCell{half_x:o.half_x+i32::from(r.x),y:o.y+i32::from(r.y)};append(cell,cell.overlay_key(r.depth,u64::from(r.insertion)),(r.entry,r.slot))?;}}
  active(cancel)?;fragments.sort_unstable_by_key(|f|f.key);active(cancel)?;let metrics=FrameMetrics{visible_fragments:fragments.len(),candidate_fragments:candidates,visited_index_rows,fragment_capacity_bytes:fragments.capacity()*size_of::<Fragment>(),fragment_limit:limit};let mut canvas=Canvas::new(camera.width,camera.height)?;canvas.clear([18,25,35]);self.tiles.draw(&fragments,&mut canvas,cancel)?;active(cancel)?;Ok((canvas,metrics))
 }
}
