//! Explicit offline map menu and CPU preview. No match admission or native clock.
use super::{compositor::Canvas, native_resources as r, resource_cipher, scene::CameraRect};
use super::native_resources::city::{CityAssets,LoadTrace};
enum Scene { Normal(super::native_multisector_scene::Scene), City(CityAssets) }
impl Scene {
 // Modern profile policy: complete plans with at least16 sectors use rows.
 // Identity/source authority remains the same exact catalog loader for all maps.
 fn load(b:&BoundMap,cancel:&AtomicBool)->Result<Self>{if b.plan.sectors.len()>=16{Ok(Self::City(CityAssets::load_whole_bounded(&b.sources,&b.plan,Default::default(),cancel,&LoadTrace::default())?))}else{Ok(Self::Normal(super::native_multisector_scene::Scene::new(r::NativeResourceAdapter::pinned_inspection().load_static(&b.sources,&b.plan,Default::default(),cancel)?)?))}}
 fn verified_plan(&self)->&r::NativeAssetPlan{match self{Self::Normal(s)=>s.assets().verified_plan(),Self::City(s)=>s.verified_plan()}}
 fn initial_camera(&self)->CameraRect{match self{Self::Normal(s)=>s.initial_camera(),Self::City(s)=>s.initial_camera()}}
 fn pan_camera(&self,c:CameraRect,x:i32,y:i32)->Result<CameraRect>{match self{Self::Normal(s)=>s.pan_camera(c,x,y),Self::City(s)=>s.pan_camera(c,x,y)}}
 fn render(&self,c:CameraRect,cancel:&AtomicBool)->Result<Canvas>{match self{Self::Normal(s)=>s.render(c,cancel),Self::City(s)=>s.render(c,cancel)}}
}
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}}};

pub use super::native_offline_map_flow::{MapChoice,Catalog,BoundMap};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page { Menu, Loading, OfflinePreview }
static NEXT_FLOW: AtomicU64=AtomicU64::new(1);
pub struct OfflineMapFlow {
    id:u64, catalog:Catalog, maps:Vec<BoundMap>, selected:Option<usize>, page:Page, epoch:u64,
    assets:Option<Scene>, camera:CameraRect, revision:u64, frame_guard:Option<Arc<AtomicU64>>,
}
/// A prepared load is private to the selected map and this exact menu generation.
pub struct LoadJob { flow:u64, epoch:u64, binding:BoundMap }
pub struct ReadyMap { flow:u64, epoch:u64, assets:Scene, canvas:Canvas }
pub struct PreviewFrame { canvas:Canvas, guard:Arc<AtomicU64>, revision:u64, pub camera:CameraRect }
fn initial_camera()->CameraRect {CameraRect{x:1136,y:468,width:800,height:600}}
impl PreviewFrame {
    pub fn rgba(&self)->Result<&[u8]> {ensure!(self.guard.load(Ordering::Acquire)==self.revision, "preview frame retired");Ok(self.canvas.rgba())}
}
impl LoadJob {
    pub fn load(self,cancel:&AtomicBool)->Result<ReadyMap> {
        let assets=Scene::load(&self.binding,cancel)?;
        let canvas=assets.render(assets.initial_camera(),cancel)?;
        Ok(ReadyMap{flow:self.flow,epoch:self.epoch,assets,canvas})
    }
}
impl OfflineMapFlow {
    pub fn new(enabled:bool,catalog:Catalog,maps:Vec<BoundMap>)->Result<Self> {
        ensure!(enabled && !maps.is_empty() && maps.len()<=64, "offline preview disabled/no binding");let mut ids=BTreeSet::new();
        for m in &maps {
            ensure!(catalog.choices().iter().any(|c|c.mode==m.mode && c.map_id==m.plan.map_id) && ids.insert((m.mode,m.plan.map_id)), "mode/map catalog binding mismatch");
            ensure!(m.plan.namespace==r::ResourceNamespace::NativeLitoV1 && !m.plan.sectors.is_empty() && m.plan.sectors.iter().all(|s|s.key.map_id==m.plan.map_id), "explicit complete Native sector plan required");
        }
        let id=NEXT_FLOW.fetch_add(1,Ordering::Relaxed);ensure!(id>0 && id<u64::MAX, "menu identity exhausted");
        Ok(Self{id,catalog,maps,selected:None,page:Page::Menu,epoch:0,assets:None,camera:initial_camera(),revision:0,frame_guard:None})
    }
    pub fn page(&self)->Page {self.page}
    pub fn choices(&self)->&[MapChoice] {self.catalog.choices()}
    pub fn available(&self,mode:u8,map_id:u32)->bool {self.maps.iter().any(|m|m.mode==mode && m.plan.map_id==map_id)}
    pub fn selected(&self)->Option<&MapChoice> {self.selected.map(|i|{let m=&self.maps[i];self.catalog.choices().iter().find(|c|c.mode==m.mode && c.map_id==m.plan.map_id).unwrap()})}
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
        self.camera=ready.assets.initial_camera();self.assets=Some(ready.assets);self.revision=1;
        let guard=Arc::new(AtomicU64::new(1));self.frame_guard=Some(guard.clone());self.page=Page::OfflinePreview;
        Ok(PreviewFrame{canvas:ready.canvas,guard,revision:1,camera:self.camera})
    }
    /// Modern inspection camera only: does not move an actor or advance any native timer.
    pub fn pan(&mut self,dx:i32,dy:i32,cancel:&AtomicBool)->Result<PreviewFrame> {
        ensure!(self.page==Page::OfflinePreview, "preview is not open");let mut camera=self.camera;
        camera=self.assets.as_ref().context("preview assets absent")?.pan_camera(camera,dx,dy)?;
        let revision=self.revision.checked_add(1).context("preview revision exhausted")?;
        let canvas=self.assets.as_ref().context("preview assets absent")?.render(camera,cancel)?;
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
