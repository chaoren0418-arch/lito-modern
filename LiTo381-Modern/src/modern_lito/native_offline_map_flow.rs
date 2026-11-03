//! Explicit offline map menu and CPU preview. No match admission or native clock.
use super::{compositor::Canvas, native_resources as r, resource_cipher, scene::CameraRect};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}}};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapChoice { pub mode: u8, pub map_id: u32, pub name: String, pub source_row: usize }
#[derive(Clone, Debug)]
pub struct Catalog { choices: Vec<MapChoice>, pub decoded_pin: r::BytePin, pub record_index: usize, pub member_offset: usize }
fn pin(bytes: &[u8]) -> r::BytePin { r::BytePin { bytes: bytes.len() as u64, sha256: Sha256::digest(bytes).into() } }
impl Catalog {
    /// The caller explicitly selects the pinned LTtext-k resource; no live language inference.
    pub fn from_lttext(enabled: bool, index: &[u8], pak: &[u8], index_pin: r::BytePin, pak_pin: r::BytePin, decoded_pin: r::BytePin) -> Result<Self> {
        ensure!(enabled, "offline map menu disabled");
        ensure!(index.len() >= 4 && index.len() <= 8<<20 && pak.len() <= 64<<20 && pin(index)==index_pin && pin(pak)==pak_pin, "LTtext pin/budget mismatch");
        let count=u32::from_le_bytes(index[..4].try_into()?) as usize;
        ensure!(count > 0 && count <= 100000 && index.len()==4+count*28, "exact old28 LTtext index required");
        let mut index=index.to_vec();resource_cipher::decode_payload_in_place(&mut index[4..]);let mut selected=None;
        for (i,row) in index[4..].chunks_exact(28).enumerate() {
            let name=&row[4..24];let end=name.iter().position(|b|*b==0).unwrap_or(20);
            let offset=u32::from_le_bytes(row[..4].try_into()?) as usize;let len=u32::from_le_bytes(row[24..].try_into()?) as usize;
            let limit=offset.checked_add(len).context("LTtext range overflow")?;ensure!(len<=8<<20 && limit<=pak.len(), "LTtext range invalid");
            if name[..end].eq_ignore_ascii_case(b"LTmapstr-k.tbl") { ensure!(selected.is_none() && len>0, "map member ambiguous/empty"); selected=Some((i,offset,limit)); }
        }
        let (record_index,member_offset,end)=selected.context("LTmapstr-k.tbl absent")?;
        let mut bytes=pak[member_offset..end].to_vec();resource_cipher::decode_payload_in_place(&mut bytes);
        ensure!(pin(&bytes)==decoded_pin && bytes.len()<=8192 && !bytes.contains(&0), "map table content mismatch");
        let text=std::str::from_utf8(&bytes)?;let mut lines=text.lines();let declared:usize=lines.next().context("map header absent")?.parse()?;
        let rows:Vec<_>=lines.collect();ensure!(declared>0 && declared<=1000 && rows.len()>=declared && rows[declared..].iter().all(|s|s.is_empty()), "map row count mismatch");
        let mut choices=Vec::new();let mut ids=BTreeSet::new();
        for (source_row,line) in rows[..declared].iter().enumerate() {
            if line.starts_with(';') || line.starts_with("no exist#") {continue;}
            let fields:Vec<_>=line.split('#').collect();ensure!(fields.len()>=3, "map fields missing");
            let map_id:u32=fields[0].parse()?;ensure!(map_id>0 && ids.insert(map_id), "map ID zero/duplicate");
            let mode=match fields[1].to_ascii_lowercase().as_str() {"petrace"=>0,"deathmatch"=>1,"questbattle"=>2,"siegewar"=>3,"timeattack"=>4,"honormatch"=>5,"town"=>6,_=>anyhow::bail!("unknown map category")};
            ensure!(!fields[2].is_empty() && fields[2].len()<192, "map name absent/oversize");
            choices.push(MapChoice {mode,map_id,name:fields[2].to_owned(),source_row});
        }
        Ok(Self{choices,decoded_pin,record_index,member_offset})
    }
    pub fn choices(&self) -> &[MapChoice] { &self.choices }
}

#[derive(Clone, Debug)]
pub struct BoundMap { pub mode: u8, pub sources: r::NativeSources, pub plan: r::NativeAssetPlan }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page { Menu, Loading, OfflinePreview }
static NEXT_FLOW: AtomicU64=AtomicU64::new(1);
pub struct OfflineMapFlow {
    id:u64, catalog:Catalog, maps:Vec<BoundMap>, selected:Option<usize>, page:Page, epoch:u64,
    assets:Option<r::NativeStaticAssets>, camera:CameraRect, revision:u64, frame_guard:Option<Arc<AtomicU64>>,
}
/// A prepared load is private to the selected map and this exact menu generation.
pub struct LoadJob { flow:u64, epoch:u64, binding:BoundMap }
pub struct ReadyMap { flow:u64, epoch:u64, assets:r::NativeStaticAssets, canvas:Canvas }
pub struct PreviewFrame { canvas:Canvas, guard:Arc<AtomicU64>, revision:u64, pub camera:CameraRect }
fn initial_camera()->CameraRect {CameraRect{x:1136,y:468,width:800,height:600}}
impl PreviewFrame {
    pub fn rgba(&self)->Result<&[u8]> {ensure!(self.guard.load(Ordering::Acquire)==self.revision, "preview frame retired");Ok(self.canvas.rgba())}
}
impl LoadJob {
    pub fn load(self,cancel:&AtomicBool)->Result<ReadyMap> {
        let assets=r::NativeResourceAdapter::pinned_inspection().load_static(&self.binding.sources,&self.binding.plan,Default::default(),cancel)?;
        let canvas=assets.render_static(initial_camera(),cancel)?;
        Ok(ReadyMap{flow:self.flow,epoch:self.epoch,assets,canvas})
    }
}
impl OfflineMapFlow {
    pub fn new(enabled:bool,catalog:Catalog,maps:Vec<BoundMap>)->Result<Self> {
        ensure!(enabled && !maps.is_empty() && maps.len()<=64, "offline preview disabled/no binding");let mut ids=BTreeSet::new();
        for m in &maps {
            ensure!(catalog.choices.iter().any(|c|c.mode==m.mode && c.map_id==m.plan.map_id) && ids.insert((m.mode,m.plan.map_id)), "mode/map catalog binding mismatch");
            ensure!(m.plan.namespace==r::ResourceNamespace::NativeLitoV1 && m.plan.sectors.len()==1 && m.plan.sectors[0].key.map_id==m.plan.map_id, "explicit single Native sector required");
        }
        let id=NEXT_FLOW.fetch_add(1,Ordering::Relaxed);ensure!(id>0 && id<u64::MAX, "menu identity exhausted");
        Ok(Self{id,catalog,maps,selected:None,page:Page::Menu,epoch:0,assets:None,camera:initial_camera(),revision:0,frame_guard:None})
    }
    pub fn page(&self)->Page {self.page}
    pub fn choices(&self)->&[MapChoice] {self.catalog.choices()}
    pub fn available(&self,mode:u8,map_id:u32)->bool {self.maps.iter().any(|m|m.mode==mode && m.plan.map_id==map_id)}
    pub fn selected(&self)->Option<&MapChoice> {self.selected.map(|i|{let m=&self.maps[i];self.catalog.choices.iter().find(|c|c.mode==m.mode && c.map_id==m.plan.map_id).unwrap()})}
    pub fn select(&mut self,mode:u8,map_id:u32)->Result<()> {
        ensure!(self.page==Page::Menu, "return to menu before selection");
        let i=self.maps.iter().position(|m|m.mode==mode && m.plan.map_id==map_id).context("map has no verified Native preview binding")?;
        self.selected=Some(i);Ok(())
    }
    pub fn begin_entry(&mut self)->Result<LoadJob> {
        ensure!(self.page==Page::Menu, "map load already active");let binding=self.maps[self.selected.context("select a map first")?].clone();
        self.epoch=self.epoch.checked_add(1).context("menu generation exhausted")?;self.page=Page::Loading;
        Ok(LoadJob{flow:self.id,epoch:self.epoch,binding})
    }
    /// Commit only an internally verified load; rejection leaves current state intact.
    pub fn complete_entry(&mut self,ready:ReadyMap)->Result<PreviewFrame> {
        ensure!(self.page==Page::Loading && ready.flow==self.id && ready.epoch==self.epoch, "old/foreign map completion");
        ensure!(ready.assets.verified_plan()==&self.maps[self.selected.context("selection missing")?].plan, "loaded plan mismatch");
        self.assets=Some(ready.assets);self.camera=initial_camera();self.revision=1;
        let guard=Arc::new(AtomicU64::new(1));self.frame_guard=Some(guard.clone());self.page=Page::OfflinePreview;
        Ok(PreviewFrame{canvas:ready.canvas,guard,revision:1,camera:self.camera})
    }
    /// Modern inspection camera only: does not move an actor or advance any native timer.
    pub fn pan(&mut self,dx:i32,dy:i32,cancel:&AtomicBool)->Result<PreviewFrame> {
        ensure!(self.page==Page::OfflinePreview, "preview is not open");let mut camera=self.camera;
        camera.x=(i64::from(camera.x)+i64::from(dx)).clamp(0,3072-i64::from(camera.width)) as i32;
        camera.y=(i64::from(camera.y)+i64::from(dy)).clamp(0,1536-i64::from(camera.height)) as i32;
        let revision=self.revision.checked_add(1).context("preview revision exhausted")?;
        let canvas=self.assets.as_ref().context("preview assets absent")?.render_static(camera,cancel)?;
        let guard=self.frame_guard.as_ref().context("frame guard absent")?.clone();
        self.camera=camera;self.revision=revision;guard.store(revision,Ordering::Release);
        Ok(PreviewFrame{canvas,guard,revision,camera})
    }
    pub fn back(&mut self)->Result<()> {
        ensure!(self.page!=Page::Menu, "already at map menu");let epoch=self.epoch.checked_add(1).context("menu generation exhausted")?;
        if let Some(guard)=self.frame_guard.take(){guard.store(0,Ordering::Release);}
        self.assets=None;self.page=Page::Menu;self.epoch=epoch;self.camera=initial_camera();self.revision=0;Ok(())
    }
}
impl Drop for OfflineMapFlow {fn drop(&mut self){if let Some(guard)=&self.frame_guard{guard.store(0,Ordering::Release);}}}
#[cfg(test)]#[path="native_offline_map_flow_tests.rs"]mod tests;
