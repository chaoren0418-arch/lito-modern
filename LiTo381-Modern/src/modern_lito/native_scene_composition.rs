//! Default-off CPU scene study. Native half-cell order is source-qualified;
//! cross-object history, relocated body, roof runtime and lifetime are explicit.
use super::{native_actor_dependency_draft::{ActorRendererJob,VerifiedActorBody},
    native_canonical_draft::RendererJob,native_actor_inspection::BodyAction,
    native_action_inspection::NativeActionSnapshot,native_resources::{NativeStaticAssets,NativeAssetPlan},
    battle_payload::UnvalidatedData,compositor::Canvas,world_depth::NativeDrawCell,
    world_map::{WorldCell,ViewProjection},world_raster::{Raster,PreparedTile,Blend,DrawVisibility},world_roofs::Shade};
use anyhow::{ensure,Context,Result};
use std::{collections::{BTreeMap,BTreeSet},sync::{Arc,atomic::{AtomicBool,Ordering}}};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoofPolicy { StaticSourceOnly, ActorDrivenRuntime }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum LifetimePolicy { ExplicitSingleFrameStudy, NaturalRuntime }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BodyPlacement { ActualObservation, ConditionalCell(WorldCell) }
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub enum SceneObject { Body, Effect(usize) }
#[derive(Clone,Debug,Default)]
pub struct Inputs {
    pub roof:Option<RoofPolicy>,pub lifetime:Option<LifetimePolicy>,
    pub body:Option<BodyPlacement>,pub insertion:Option<Vec<SceneObject>>,pub max_frames:Option<usize>,
}
impl Inputs {
    fn qualify(self,effects:&[usize])->Result<(BodyPlacement,Vec<SceneObject>,usize)> {
        ensure!(self.roof==Some(RoofPolicy::StaticSourceOnly),"Original roof runtime/alias state unqualified");
        ensure!(self.lifetime==Some(LifetimePolicy::ExplicitSingleFrameStudy),"Original natural lifetime unqualified");
        let body=self.body.context("Scene body placement unknown")?;
        let order=self.insertion.context("Original cross-object insertion history unknown; explicit study required")?;
        let expected:BTreeSet<_>=std::iter::once(SceneObject::Body).chain(effects.iter().copied().map(SceneObject::Effect)).collect();
        ensure!(order.len()==expected.len()&&order.iter().copied().collect::<BTreeSet<_>>()==expected,"Scene insertion duplicate/missing/foreign object");
        let limit=self.max_frames.context("Scene frame budget missing")?;ensure!((1..=16).contains(&limit),"Scene frame budget overbound");
        Ok((body,order,limit))
    }
}
struct Binding {cancel:Arc<AtomicBool>,local:Arc<AtomicBool>,data:UnvalidatedData,plan:NativeAssetPlan}
impl Binding {
    fn check(&self,j:&RendererJob)->Result<()> {
        ensure!(!self.cancel.load(Ordering::Acquire)&&!self.local.load(Ordering::Acquire),"Native scene retired/cancelled");
        ensure!(Arc::ptr_eq(&self.cancel,&j.cancellation_token())&&self.data==*j.admitted_observation().data()&&self.plan==*j.source_plan(),"Native scene owner/fullDATA/source mismatch");Ok(())
    }
}
pub struct SceneFrame {binding:Arc<Binding>,canvas:Canvas,body_visibility:DrawVisibility,pub conditional_body:bool,pub visible_terrain:usize}
impl SceneFrame {
    pub fn canvas(&self,j:&RendererJob)->Result<&Canvas>{self.binding.check(j)?;Ok(&self.canvas)}
    pub fn body_visible_pixels(&self,j:&RendererJob)->Result<usize>{self.binding.check(j)?;Ok((0..self.canvas.height()).flat_map(|y|(0..self.canvas.width()).map(move|x|(x,y))).filter(|(x,y)|self.body_visibility.contains(i64::from(*x),i64::from(*y))).count())}
}
struct State<'a> {
    binding:Arc<Binding>,assets:&'a NativeStaticAssets,body:&'a VerifiedActorBody,
    effects:&'a [(usize,NativeActionSnapshot)],tiles:BTreeMap<(u16,u8),PreparedTile>,
    terrain:Vec<(NativeDrawCell,Option<u8>,(u16,u8))>,placement:BodyPlacement,
    order:Vec<SceneObject>,frames:usize,limit:usize,
}
#[derive(Default)]
pub struct SceneModel<'a> {state:Option<State<'a>>}
impl<'a> SceneModel<'a> {
    pub fn local_experiment(j:&ActorRendererJob,assets:&'a NativeStaticAssets,body:&'a VerifiedActorBody,effects:&'a [(usize,NativeActionSnapshot)],inputs:Inputs)->Result<Self> {
        let n=j.native_job();ensure!(assets.verified_plan()==n.source_plan(),"Scene terrain source mismatch");
        let ids:Vec<_>=effects.iter().map(|e|e.0).collect();ensure!(effects.len()<=16&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len(),"Scene effect count/duplicate");
        let (placement,order,limit)=inputs.qualify(&ids)?;
        body.scene_frame(n,BodyAction::Idle,0)?;
        if let BodyPlacement::ConditionalCell(cell)=placement{ensure!(assets.geometry().contains(cell),"Conditional body outside pinned sector");}
        for (index,effect) in effects {
            let source=assets.effect_records().get(*index).context("Scene effect record absent")?;
            ensure!(effect.scene_plan()==n.source_plan()&&u32::from(source.effect.sprite_id)==effect.scene_sprite_id()&&effect.scene_catalog_pins()==j.identity().catalog().action_pins(),"Scene effect record/catalog identity mismatch");
            // Map Layer8 X/Y words are absolute whole-cell coordinates in
            // both fixed 483A80 branches, not sector-local byte coordinates.
            ensure!(assets.geometry().contains(WorldCell{x:i32::from(source.effect.x),y:i32::from(source.effect.y)}),"Scene effect outside pinned sector");
        }
        let binding=Arc::new(Binding{cancel:n.cancellation_token(),local:Arc::new(AtomicBool::new(false)),data:n.admitted_observation().data().clone(),plan:n.source_plan().clone()});binding.check(n)?;
        let terrain=assets.scene_terrain()?;let mut tiles=BTreeMap::new();
        for (id,tile) in assets.scene_tiles(){ensure!(!binding.cancel.load(Ordering::Acquire),"Scene prepare cancelled");tiles.insert(*id,PreparedTile::new(tile.clone())?);}
        binding.check(n)?;
        Ok(Self{state:Some(State{binding,assets,body,effects,tiles,terrain,placement,order,frames:0,limit})})
    }
    pub fn render(&mut self,j:&ActorRendererJob,view:ViewProjection,width:u32,height:u32,body_frame:usize,effect_frame:usize)->Result<SceneFrame> {
        let s=self.state.as_mut().context("Native scene composition disabled")?;let n=j.native_job();s.binding.check(n)?;
        ensure!(width>0&&height>0&&width<=1024&&height<=768,"Native scene canvas overbound");
        if s.frames>=s.limit{s.binding.local.store(true,Ordering::Release);anyhow::bail!("Scene frame budget exhausted; retired");}
        let (body,actual)=s.body.scene_frame(n,BodyAction::Idle,body_frame)?;
        let anchor=match s.placement{BodyPlacement::ActualObservation=>actual,BodyPlacement::ConditionalCell(cell)=>NativeDrawCell::from_world(cell).context("Conditional native coordinate overflow")?};
        // Validate all selected source frames before any paint.
        for (_,effect) in s.effects{effect.scene_frame(effect_frame)?;}
        let mut canvas=Canvas::new(width,height)?;let mut raster=Raster::new(view,&canvas);let mut visible=0;
        for (cell,depth,id) in &s.terrain{visible+=usize::from(raster.tile_roof(*cell,*depth,&s.tiles[id],Shade::Native)?);}
        for object in &s.order{match object{
            SceneObject::Body=>raster.sprite(anchor,body,1,Blend::Over)?,
            SceneObject::Effect(index)=>{let effect=&s.effects.iter().find(|(i,_)|i==index).context("Scene effect insertion absent")?.1;let r=s.assets.effect_records()[*index];let anchor=NativeDrawCell::from_world(WorldCell{x:i32::from(r.effect.x),y:i32::from(r.effect.y)}).context("Scene effect coordinate overflow")?;raster.sprite(anchor,effect.scene_frame(effect_frame)?,0,Blend::Over)?;}
        }}
        let mut masks=raster.paint(&mut canvas,&s.binding.cancel,1)?;s.binding.check(n)?;s.frames+=1;
        Ok(SceneFrame{binding:s.binding.clone(),canvas,body_visibility:masks.remove(0),conditional_body:matches!(s.placement,BodyPlacement::ConditionalCell(_)),visible_terrain:visible})
    }
    pub fn retire(&mut self){if let Some(s)=&self.state{s.binding.local.store(true,Ordering::Release);}}
    pub fn loaded_admission_available(&self)->bool{false}
}
impl Drop for SceneModel<'_>{fn drop(&mut self){self.retire();}}
#[cfg(test)]
#[path="native_scene_composition_tests.rs"]
mod tests;
