//! Render received native coordinates with the explicit LTmap/LTtile namespace.
use super::{
    compositor::Canvas,
    native_multisector_scene::Scene,
    native_resources as nr,
    native_world::World,
    resources as res,
    scene::CameraRect,
    world_map::{SectorInput, WorldCell, WorldMap},
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
#[cfg(test)]
#[path = "native_movement_tests.rs"]
pub(crate) mod movement_tests;
/// Lightweight hit/camera data for the frame actually presented by the host.
/// Raster owner planes are immutable Arc-backed data; no canvas/DIB is copied.
#[derive(Clone)]
pub struct InputSurface {
    static2941: super::native_static2941::DrawGuard,
    identity: FrameIdentity,
    stamp: super::native_navigation::InputStamp,
    revision: u64,
    visibility: Vec<(u32, super::world_raster::DrawVisibility)>,
    ground_ids: Vec<u32>,
    navigation: std::sync::Arc<super::native_navigation::Navigation>,
    projection: super::world_map::ViewProjection,
    minimap_visible: bool,
}
#[derive(Clone, Copy)]
struct FrameIdentity {
    map: u32,
    generation: u64,
    caster_incarnation: u64,
    movement_key: (u32,u64,u16,u16,u8,u8,u64),
    observer_key: Option<(u32,[u16;2])>,
}
impl FrameIdentity {
    fn matches_world(self, world: &World) -> bool {
        if let Some((id,position))=self.observer_key {
            return world.observer_mode && world.view_ready() && world.map_id.map(u32::from)==Some(self.map)
                && world.generation==self.generation && world.status.is_some_and(|s|s.object_id==id)
                && world.observer_camera==Some(position);
        }
        !world.observer_mode && world.map_id.map(u32::from)==Some(self.map)
            && world.generation==self.generation && world.own_actor().is_some_and(|a|
                a.incarnation==self.caster_incarnation && (a.id,a.motion_revision,a.x,a.y,a.speed,a.brave,a.action_sequence)==self.movement_key)
    }
}
impl InputSurface {
    pub fn input_ready(&self, world: &World) -> bool {
        self.static2941.publishable(world) && self.identity.matches_world(world) && self.revision==world.visual_revision
            && self.stamp==super::native_navigation::InputStamp::capture(0,world)
    }
    pub fn overlay_contains(&self,x:i32,y:i32)->bool {
        self.minimap_visible && super::native_minimap::contains(x,y)
    }
    pub fn pick_current(&self,world:&World,x:i32,y:i32,own:u32)->Option<u32> {
        if !self.input_ready(world) || self.overlay_contains(x,y){return None;}
        self.visibility.iter().find(|(id,mask)|*id!=own && mask.contains(i64::from(x),i64::from(y))).map(|(id,_)|*id)
    }
    pub fn ground_cell(&self,world:&World,x:i32,y:i32)->Option<WorldCell> {
        if !self.input_ready(world) || self.overlay_contains(x,y) || !(0..800).contains(&x) || !(0..600).contains(&y){return None;}
        let cell=self.projection.pixel_to_cell(super::world_map::PixelPoint{x,y})?;
        ((0..=65535).contains(&cell.x) && (0..=65535).contains(&cell.y)
            && self.navigation.contains(cell.x,cell.y)).then_some(cell)
    }
    pub fn is_ground_item(&self,id:u32)->bool {self.ground_ids.contains(&id)}
}
/// Already dispatched model callback; publication never replays the animation.
#[derive(Clone,Debug)]
pub(crate) struct HurtCallback {
    target:u32,incarnation:u64,sequence:u64,projectile:u32,graphics:u16,accepted:bool,
    generation:u64,owner:std::sync::Arc<()>,dispatch_ns:u64,pub(crate) deferred:bool,
}
impl HurtCallback {
    pub(crate) fn capture(world:&World,actor:&super::native_world::Actor,owner:std::sync::Arc<()>,sequence:u64,projectile:u32,accepted:bool,dispatch_ns:u64,graphics:u16)->Self {
        Self{target:actor.id,incarnation:actor.incarnation,sequence,projectile,graphics,accepted,generation:world.generation,owner,dispatch_ns,deferred:false}
    }
    pub(crate) fn current(&self,world:&World,owner:&std::sync::Arc<()>)->bool {
        self.generation==world.generation && std::sync::Arc::ptr_eq(&self.owner,owner)
            && world.actors.get(&self.target).is_some_and(|actor|actor.incarnation==self.incarnation && !actor.dead)
    }
    pub(crate) fn identity(&self)->(u32,u64,u64,u32) {(self.target,self.incarnation,self.sequence,self.projectile)}
    pub(crate) fn graphics(&self)->u16 {self.graphics}
    pub(crate) fn order(&self)->(u64,u64,u32) {(self.dispatch_ns,self.sequence,self.target)}
    pub(crate) fn receipt(&self)->serde_json::Value {
        serde_json::json!({"target":self.target,"incarnation":self.incarnation,"sequence":self.sequence,"projectile":self.projectile,"graphics":self.graphics,"accepted":self.accepted,"dispatch_ns":self.dispatch_ns,"deferred_due_to_environment":self.deferred})
    }
}
pub struct Frame {
    status4183_leases: Vec<std::sync::Arc<AtomicBool>>,
    pub(crate) player_walk_plan: Option<std::sync::Arc<super::native_player_callbacks::WalkPlan>>,
    pub(crate) static2941: super::native_static2941::DrawGuard,
    pub ground_items: Vec<super::native_world::Actor>,
    pub hits: Vec<(u32, [i32; 4])>,
    pub visibility: Vec<(u32, super::world_raster::DrawVisibility)>,
    pub sounds: Vec<super::combat_audio::PreparedSound>,
    pub legacy_audio: Vec<super::native_audio_delivery::legacy::PreparedLegacyCue>,
    pub native_audio: Vec<super::native_audio_delivery::PreparedCue>,
    pub native_ambient: Vec<super::native_ambient_bridge::Prepared>,
    pub(crate) ambient_owner:std::sync::Arc<()>,
    pub(crate) finite_hurt_callbacks:Vec<HurtCallback>,
    pub navigation: std::sync::Arc<super::native_navigation::Navigation>,
    movement_key: (u32, u64, u16, u16, u8, u8, u64),
    movement_arrived: bool,
    movement_stamp: super::native_navigation::InputStamp,
    death_arrived: bool,
    movement_generation: u64,
    observer_key: Option<(u32,[u16;2])>,
    caster_incarnation: u64,
    input_revision: u64,
    environment_key: Option<super::native_render_environment::Key>,
    projection: super::world_map::ViewProjection,
    pub canvas: Canvas,
    pub bgra: Vec<u8>,
    pub receipt: serde_json::Value,
}
impl Frame {
    pub(crate) fn status4183_current(&self)->bool {
        self.status4183_leases.iter().all(|lease|lease.load(Ordering::Acquire))
    }
    pub(crate) fn composition_current(&self, world:&World)->bool {
        self.static2941.active(world) && self.identity().matches_world(world)
            && self.input_revision==world.visual_revision
    }
    pub(crate) fn static_current(&self, world: &World, connection: u64) -> bool {
        self.static2941.current(world, connection)
            && (world.player_walk.is_none() || self.input_revision == world.visual_revision)
    }
    pub(crate) fn static_active(&self, world: &World) -> bool { self.static2941.active(world) }
    pub fn environment_key(&self) -> Option<super::native_render_environment::Key> { self.environment_key }
    pub fn death_ready(&self, world: &World) -> bool {
        self.death_arrived && world.death_confirmation_ready() && self.matches_world(world)
    }
    pub fn overlay_contains(&self, x: i32, y: i32) -> bool {
        self.receipt["minimap"]["visible"] == true && super::native_minimap::contains(x,y)
    }
    pub fn pick(&self, x: i32, y: i32, own: u32) -> Option<u32> {
        if self.overlay_contains(x,y) {return None;}
        self.visibility
            .iter()
            .find(|(id, mask)| *id != own && mask.contains(i64::from(x), i64::from(y)))
            .map(|(id, _)| *id)
    }
    pub fn movement_ready(&self, world: &World) -> bool {
        self.movement_arrived && self.matches_world(world)
            && (world.player_walk.is_none() || self.input_revision == world.visual_revision)
            && self.movement_stamp == super::native_navigation::InputStamp::capture(0, world)
            && world.ready() && world.hp.is_some_and(|hp| hp[0] > 0)
            && world.battle_result.is_none() && world.movement_blocks.is_empty()
            && world.movement_lock == super::native_world::MovementLock::Open
            && world.own_actor().is_some_and(|a| !a.dead)
    }
    /// Input can only resolve objects in this rendered authority revision.
    pub fn input_ready(&self, world: &World) -> bool {
        self.matches_world(world) && self.input_revision == world.visual_revision
    }
    pub fn pick_current(&self, world: &World, x: i32, y: i32, own: u32) -> Option<u32> {
        self.input_ready(world)
            .then(|| self.pick(x, y, own))
            .flatten()
    }
    /// Uses exactly the render camera; this is selection only, not a spell send.
    pub fn ground_cell(
        &self,
        world: &World,
        x: i32,
        y: i32,
    ) -> Option<super::world_map::WorldCell> {
        if self.overlay_contains(x,y) || !self.matches_world(world) || !(0..800).contains(&x) || !(0..600).contains(&y) {
            return None;
        }
        let cell = self
            .projection
            .pixel_to_cell(super::world_map::PixelPoint { x, y })?;
        ((0..=65535).contains(&cell.x)
            && (0..=65535).contains(&cell.y)
            && self.navigation.contains(cell.x, cell.y))
        .then_some(cell)
    }
    fn identity(&self)->FrameIdentity {
        FrameIdentity{map:self.navigation.map_id,generation:self.movement_generation,
            caster_incarnation:self.caster_incarnation,movement_key:self.movement_key,observer_key:self.observer_key}
    }
    fn matches_world(&self, world: &World) -> bool {self.static2941.publishable(world) && self.identity().matches_world(world)}
    pub fn input_surface(&self,world:&World)->Option<InputSurface> {
        self.input_ready(world).then(||InputSurface{static2941:self.static2941.clone(),identity:self.identity(),
            stamp:super::native_navigation::InputStamp::capture(0,world),revision:self.input_revision,
            visibility:self.visibility.clone(),ground_ids:self.ground_items.iter().map(|a|a.id).collect(),
            navigation:std::sync::Arc::clone(&self.navigation),projection:self.projection,
            minimap_visible:self.receipt["minimap"]["visible"]==true})
    }

}
pub struct Job {
    receiver: std::sync::mpsc::Receiver<Result<Completion>>,
    worker: Option<std::thread::JoinHandle<()>>,
    cancel: std::sync::Arc<AtomicBool>,
}
impl Job {
    pub fn start(root: std::path::PathBuf, world: World) -> Result<Self> {
        Self::with_model(root, world, None)
    }
    pub fn with_model(
        root: std::path::PathBuf,
        world: World,
        model: Option<Model>,
    ) -> Result<Self> {
        Self::with_model_at(root, world, model, 0)
    }
    pub fn with_model_at(
        root: std::path::PathBuf,
        world: World,
        model: Option<Model>,
        elapsed_ns: u64,
    ) -> Result<Self> {
        Self::with_model_at_labels(root, world, model, elapsed_ns, None)
    }
    pub fn with_model_at_labels(
        root: std::path::PathBuf,
        world: World,
        model: Option<Model>,
        elapsed_ns: u64,
        hover: Option<super::native_world_labels::Hover>,
    ) -> Result<Self> {
        Self::with_model_at_labels_environment(root, world, model, elapsed_ns, hover, None)
    }
    pub fn with_model_at_labels_environment(
        root: std::path::PathBuf, world: World, model: Option<Model>, elapsed_ns: u64,
        hover: Option<super::native_world_labels::Hover>,
        environment: Option<super::native_render_environment::Bound>,
    ) -> Result<Self> {
        let remap=environment.as_ref().and_then(|bound|model.as_ref()
            .and_then(|model|model.lighting.remap_for(bound.key().epoch)));
        Self::with_model_at_labels_environment_remap(root,world,model,elapsed_ns,hover,environment,remap)
    }
    pub fn with_model_at_labels_environment_remap(
        root: std::path::PathBuf, world: World, model: Option<Model>, elapsed_ns: u64,
        hover: Option<super::native_world_labels::Hover>,
        environment: Option<super::native_render_environment::Bound>,
        remap: Option<super::native_light_color::Remap>,
    ) -> Result<Self> {
        let water_history=environment.as_ref().and_then(|bound|model.as_ref()
            .and_then(|model|model.lighting.water_history_for(bound.key().epoch)));
        Self::with_model_at_labels_environment_history(root,world,model,elapsed_ns,hover,environment,remap,water_history)
    }
    pub fn with_model_at_labels_environment_history(
        root:std::path::PathBuf,world:World,model:Option<Model>,elapsed_ns:u64,
        hover:Option<super::native_world_labels::Hover>,environment:Option<super::native_render_environment::Bound>,
        remap:Option<super::native_light_color::Remap>,water_history:Option<super::native_water_clock::History>,
    )->Result<Self> {
        Self::with_model_at_labels_environment_finite_history(root,world,model,elapsed_ns,hover,environment,remap,water_history,None,None,None)
    }
    pub(crate) fn with_model_at_labels_environment_finite_history(
        root:std::path::PathBuf,world:World,model:Option<Model>,elapsed_ns:u64,
        hover:Option<super::native_world_labels::Hover>,environment:Option<super::native_render_environment::Bound>,
        remap:Option<super::native_light_color::Remap>,water_history:Option<super::native_water_clock::History>,
        finite_history:Option<FiniteHistory>,
        wire_audio:Option<super::native_wire_audio::Cursor>,
        ordinary_admission:Option<super::native_world_animation::Admission>,
    )->Result<Self> {
        let (send, receiver) = std::sync::mpsc::sync_channel(1);
        let cancel = std::sync::Arc::new(AtomicBool::new(false));
        let token = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("native-world-load".into())
            .spawn(move || {
                let result = (|| -> Result<Completion> {
                    let reused=model.as_ref().is_some_and(|m|m.root==root && m.world_generation==world.generation && world.map_id.map(u32::from)==Some(m.map_id));
                    let mut model = match model {
                        Some(model) if model.root==root && model.world_generation==world.generation && world.map_id.map(u32::from)==Some(model.map_id) => model,
                        Some(_) => Model::load(&root, &world, &token)?,
                        None => Model::load(&root, &world, &token)?,
                    };
                    if !reused {
                        if let Some(history)=finite_history {model.restore_finite_history(history);}
                        if let Some(admission)=ordinary_admission {model.animator.retire_admitted(admission,&world,elapsed_ns);}
                    }
                    if let (Some(remap),Some(bound)) = (remap,environment.as_ref()) {
                        model.lighting.restore_remap(bound.key().epoch,remap)?;
                    }
                    if let (Some(history),Some(bound))=(water_history,environment.as_ref()) {
                        model.lighting.restore_water_history(bound.key().epoch,history)?;
                    }
                    if let Some(cursor)=wire_audio {model.wire_audio=cursor;}
                    let static2941_ready = model.static2941.prepare(&root, &world, &token, &mut model.effect_catalog)?;
                    let frame = model.draw_at_labels_environment(&world, &token, elapsed_ns, hover, environment.as_ref())?;
                    let static2941_error = model.static2941.error().map(str::to_owned);
                    Ok(Completion { model, frame, static2941_ready, static2941_error })
                })();
                let _ = send.send(result);
            })?;
        Ok(Self {
            receiver,
            worker: Some(worker),
            cancel,
        })
    }
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
    }
    pub fn poll(&mut self) -> Option<Result<Completion>> {
        if !self.worker.as_ref().is_some_and(|w| w.is_finished()) {
            return None;
        }
        let result = self
            .receiver
            .try_recv()
            .map_err(|_| anyhow::anyhow!("world loader ended without result"))
            .and_then(|r| r);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        Some(result)
    }
    /// Callback draw is a synchronous composition boundary. The worker owns
    /// only resource/CPU work; it cannot touch the controller queue or clock.
    pub(crate) fn finish(&mut self) -> Result<Completion> {
        if let Some(worker)=self.worker.take() {
            worker.join().map_err(|_|anyhow::anyhow!("world callback renderer panicked"))?;
        }
        self.receiver.try_recv().context("world callback renderer ended without result")?
    }
    pub fn finished(&self) -> bool {
        self.worker.is_none()
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn check(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "world rendering cancelled");
    Ok(())
}
pub(crate) fn read_pin(path: &Path, limit: u64, cancel: &AtomicBool) -> Result<nr::BytePin> {
    res::reject_reparse_chain(path)?;
    let mut f = res::open_readonly(path, false)?;
    res::verify_open_path(&f, path, false)?;
    let bytes = f.metadata()?.len();
    ensure!(bytes > 0 && bytes <= limit, "world resource byte budget");
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut total = 0;
    loop {
        check(cancel)?;
        let n = f.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        ensure!(total <= bytes, "resource grew during read");
        hash.update(&buffer[..n]);
    }
    ensure!(total == bytes, "resource changed during read");
    Ok(nr::BytePin {
        bytes,
        sha256: hash.finalize().into(),
    })
}
#[derive(Clone)]
pub(crate) struct FiniteHistory {
    static2941: super::native_static2941::History,
    animation:super::native_world_animation::History,
    skills:[super::native_skill_effects::History;4],many:super::native_attack_many::History,
    disintegrate:super::native_disintegrate::History,call_lightning:super::native_call_lightning::History,cone_cold:super::native_cone_cold::History,restore_other_effects:bool,
}
impl FiniteHistory {
    pub(crate) fn for_source(&self,same_source:bool)->Self {
        let mut history=self.clone();history.restore_other_effects &= same_source;history.static2941=history.static2941.for_source(same_source);history
    }
    pub(crate) fn merge_disintegrate_admission(&mut self,other:&Self){self.static2941.merge(&other.static2941);self.disintegrate.merge(&other.disintegrate);self.call_lightning.merge(&other.call_lightning);self.cone_cold.merge(&other.cone_cold);self.animation=other.animation.clone();}
}
pub struct Model {
    status4183: super::native_status4183::Stage,
    static2941: super::native_static2941::Stage,
    world_generation:u64,
    lighting: super::native_world_lighting::Runtime,
    names: super::native_inventory_names::Catalog,
    text_cache: super::native_world_text::Cache,
    clan_markers: super::native_clan_markers::Runtime,
    root: std::path::PathBuf,
    sound_cache: std::collections::BTreeMap<u32, super::combat_audio::PreparedSound>,
    wire_audio: super::native_wire_audio::Cursor,
    composed_frames: u64,
    terrain_reloads: u64,
    load_profile: serde_json::Value,
    navigation: std::sync::Arc<super::native_navigation::Navigation>,
    scene: Scene,
    observed_archives: Option<nr::SharedNativeArchives>,
    native_geometry: Option<NativeTerrainSources>,
    roofs: super::native_world_roofs::Runtime,
    effect_catalog: Option<super::shared_action_catalog::SharedActionCatalog>,
    sprite_bank: super::native_actor_layers::Bank,
    layer_plans: std::collections::BTreeMap<
        u16,
        std::result::Result<super::native_actor_layers::Plan, String>,
    >,
    map_id: u32,
    sector_count: usize,
    classic_sector: Option<[u16;2]>,
    native_sector: Option<[u16;2]>,
    native_block: Option<[u16;2]>,
    resident_sectors: usize,
    geometry_sha: [u8; 32],

    animator: super::native_world_animation::Animator,
    projectiles: super::native_projectiles::Projectiles,
    skill_effects: super::native_skill_effects::Effects,
    item_effects: super::native_skill_effects::Effects,
    reagent_effects: super::native_skill_effects::Effects,
    travel_effects: super::native_skill_effects::Effects,
    ground_effects: super::native_ground_effects::Effects,
    ground_sound_until: Option<(u64, u64)>,
    spell_projectiles: super::native_spell_projectiles::Projectiles,
    class1_effects: super::native_class1::Effects,
    disintegrate_effects:super::native_disintegrate::Effects,
    call_lightning_effects:super::native_call_lightning::Effects,
    cone_cold_effects:super::native_cone_cold::Effects,
    effect_source_owner:std::sync::Arc<()>,
    many_effects:super::native_attack_many::Effects,
}
pub struct Completion {
    pub(crate) static2941_ready: Vec<super::native_static2941::Prepared>,
    pub(crate) static2941_error: Option<String>,
    pub model: Model,
    pub frame: Frame,
}
pub fn render(root: &Path, world: &World, cancel: &AtomicBool) -> Result<Frame> {
    Model::load(root, world, cancel)?.draw(world, cancel)
}
struct Terrain {
    navigation:std::sync::Arc<super::native_navigation::Navigation>,scene:Scene,
    map_id:u32,sector_count:usize,classic_sector:Option<[u16;2]>,native_sector:Option<[u16;2]>,
    native_block:Option<[u16;2]>,resident_sectors:usize,geometry_sha:[u8;32],load_profile:serde_json::Value,
}
struct NativeTerrainSources {
    geometry:nr::NativeRetainedGeometry,
    navigation:std::sync::Arc<super::native_navigation::Navigation>,
}
impl Model {
    pub(crate) fn finite_history(&self)->FiniteHistory {
        FiniteHistory{static2941:self.static2941.history(),animation:self.animator.history(),skills:[self.skill_effects.history(),self.item_effects.history(),self.reagent_effects.history(),self.travel_effects.history()],many:self.many_effects.history(),disintegrate:self.disintegrate_effects.history(),call_lightning:self.call_lightning_effects.history(),cone_cold:self.cone_cold_effects.history(),restore_other_effects:true}
    }
    fn restore_finite_history(&mut self,history:FiniteHistory) {
        self.static2941.restore(&history.static2941);
        self.animator.restore(history.animation);
        if history.restore_other_effects {
            for (effect,h) in [&mut self.skill_effects,&mut self.item_effects,&mut self.reagent_effects,&mut self.travel_effects].into_iter().zip(history.skills) {effect.restore_history(h);}
            self.many_effects.restore_history(history.many);
        }
        self.disintegrate_effects.restore_history(history.disintegrate);
        self.call_lightning_effects.restore_history(history.call_lightning);
        self.cone_cold_effects.restore_history(history.cone_cold);
    }
    pub(crate) fn ambient_source_floor(&self,world:&World)->u64 {
        world.projectiles.iter().map(|cue|cue.action_sequence).chain(self.call_lightning_effects.oldest_active_sequence()).chain(self.cone_cold_effects.oldest_active_sequence()).min().unwrap_or_else(||world.visual_revision.saturating_add(1)).max(1)
    }
    pub fn light_remap(&self) -> super::native_light_color::Remap {self.lighting.remap()}
    pub fn water_history_for(&self,owner:u64)->Option<super::native_water_clock::History> {self.lighting.water_history_for(owner)}
    /// Immutable qualified terrain sources, also used by independent composition audits.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    pub fn navigation(&self) -> std::sync::Arc<super::native_navigation::Navigation> {
        self.navigation.clone()
    }
    fn classic_sector(world:&World)->Option<[u16;2]>{
        if world.map_id!=Some(4){return None;}
        let a=world.own_actor()?;Some([u16::try_from(0x7fff+(i32::from(a.x)-32704).div_euclid(64)).ok()?,u16::try_from(0x7fff+(i32::from(a.y)-32704).div_euclid(64)).ok()?])
    }
    fn native_view(world:&World)->Option<[u16;2]>{
        if world.map_id==Some(4){return None;}
        if world.observer_mode {world.observer_camera} else {world.own_actor().map(|a|[a.x,a.y])}
    }
    fn native_block(world:&World)->Option<[u16;2]>{Self::native_view(world).map(|p|p.map(|v|v&!7))}
    fn native_sector(world:&World)->Option<[u16;2]>{
        if world.map_id==Some(4){return None;}
        let [x,y]=if world.observer_mode {world.observer_camera?} else {let a=world.own_actor()?;[a.x,a.y]};
        Some([u16::try_from(0x7fff+(i32::from(x)-32704).div_euclid(64)).ok()?,u16::try_from(0x7fff+(i32::from(y)-32704).div_euclid(64)).ok()?])
    }
    fn covers_world(&self,world:&World)->bool{world.map_id.map(u32::from)==Some(self.map_id)&&self.classic_sector==Self::classic_sector(world)&&self.native_block==Self::native_block(world)}
    fn load_terrain(root: &Path, world: &World, cancel: &AtomicBool,
        observed_archives:&mut Option<nr::SharedNativeArchives>,
        native_geometry:&mut Option<NativeTerrainSources>) -> Result<Terrain> {
        ensure!(
            root.is_absolute() && world.view_ready(),
            "world resources require ready authority and absolute root"
        );
        check(cancel)?;
        if world.map_id!=Some(4) {
            return Self::load_native_terrain(root,world,cancel,observed_archives,native_geometry);
        }
        let load_start = std::time::Instant::now();
        let mut checkpoint = load_start;
        let mut stages = serde_json::Map::new();
        let mut archive_sources_reused=false;
        let mut shared_archive_sources_reused=false;
        let mut mark = |name: &str| {
            let now = std::time::Instant::now();
            stages.insert(
                name.into(),
                serde_json::json!(now.duration_since(checkpoint).as_micros()),
            );
            checkpoint = now;
        };
        let map_id = u32::from(world.map_id.context("world map")?);
        // C02 skill131's retained destination is original classic map4.
        // This explicit contract is selected before I/O, never as missing-file fallback.
        let classic_sector=Self::classic_sector(world);
        let native_sector=Self::native_sector(world);
        let native_block=Self::native_block(world);
        let sources = nr::NativeSources {
            map_root: root.join(if map_id==4 {"map"}else{"LiTo/LTmap"}),
            archive_root: root.join("LiTo"),
        };
        let map_dir = sources.map_root.join(map_id.to_string());
        res::reject_reparse_chain(&map_dir)?;
        let mut files = Vec::new();
        if let Some(center)=classic_sector {
            // Read only exact sector keys. Unrelated tempseg/backup files in the
            // original shared directory are not candidates for the window.
            for dx in -1i32..=1{for dy in -1i32..=1{
                let x=u16::try_from(i32::from(center[0])+dx)?;let y=u16::try_from(i32::from(center[1])+dy)?;
                let name=format!("{x:04x}{y:04x}.s32");let path=map_dir.join(&name);
                match std::fs::metadata(&path){Ok(m)=>{ensure!(m.is_file(),"classic sector is not a file");files.push((name,path));},Err(e) if e.kind()==std::io::ErrorKind::NotFound&&[x,y]!=center=>{},Err(e)=>return Err(e.into())}
            }}
        }else{
        for entry in std::fs::read_dir(&map_dir)? {
            check(cancel)?;
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("map filename encoding"))?;
            if !name.ends_with(".s32") {
                continue;
            }
            ensure!(
                name.len() == 12 && name[..8].bytes().all(|b| b.is_ascii_hexdigit()),
                "map sector name"
            );
            if let Some(center)=classic_sector {
                let x=u16::from_str_radix(&name[..4],16)?;let y=u16::from_str_radix(&name[4..8],16)?;
                if x.abs_diff(center[0])>1||y.abs_diff(center[1])>1{continue;}
            }
            ensure!(files.len() < 64, "map sector count budget");
            files.push((name, entry.path()));
        }
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        ensure!(!files.is_empty(), "no LTmap S32 sectors");
        let mut sectors = Vec::new();
        let mut inputs = Vec::new();
        let mut total = 0u64;
        for (name, path) in files {
            let pin = read_pin(&path, 8 << 20, cancel)?;
            total += pin.bytes;
            ensure!(total <= 32 << 20, "map bytes budget");
            let x = u16::from_str_radix(&name[..4], 16)?;
            let y = u16::from_str_radix(&name[4..8], 16)?;
            let f = res::open_readonly(&path, false)?;
            res::verify_open_path(&f, &path, false)?;
            let mut bytes = Vec::new();
            f.take(pin.bytes + 1).read_to_end(&mut bytes)?;
            ensure!(
                bytes.len() as u64 == pin.bytes
                    && <[u8; 32]>::from(Sha256::digest(&bytes)) == pin.sha256,
                "map changed during read"
            );
            sectors.push(nr::SectorPin {
                key: nr::NativeSectorKey { map_id, x, y },
                pin,
            });
            inputs.push(SectorInput {
                key: res::SharedMapSectorKey {
                    map_id,
                    sector_x: x,
                    sector_y: y,
                },
                bytes,
            });
        }
        let geometry = if classic_sector.is_some(){WorldMap::from_sector_bytes(inputs,cancel)?}else{WorldMap::from_native_sector_bytes(inputs, cancel)?};
        let navigation = std::sync::Arc::new(super::native_navigation::Navigation::from_geometry(
            &geometry,
        )?);
        mark("geometry_us");
        let sector_count=sectors.len();let geometry_sha=geometry.canonical_digest();
        let assets = if classic_sector.is_some(){
            nr::classic_world4_assets(root,geometry,sectors,cancel)?
        }else{
        archive_sources_reused=observed_archives.is_some();
        if observed_archives.is_none() {
            *observed_archives=Some(nr::SharedNativeArchives::open(
                &sources.archive_root,nr::Limits::default(),cancel)?);
        }
        let shared=observed_archives.as_ref().context("native archive owner absent")?;
        shared_archive_sources_reused=shared.reused_source_owner();
        let mut lease=shared.lease(cancel)?;
        let observed_archives=&mut *lease;
        let archives = observed_archives.pins().to_vec();
        let plan = nr::NativeAssetPlan {
            namespace: nr::ResourceNamespace::NativeLitoV1,
            map_id,
            canonical_sha256: geometry.canonical_digest(),
            sectors,
            archives,
        };
        mark("archive_pins_us");
        // This byte-pinned decoder is independent of the historical Modern leases.
        nr::NativeResourceAdapter::pinned_world().load_static_retained_window(
            &sources,
            &plan,
            nr::Limits::default(),
            observed_archives,
            Self::native_view(world).context("native terrain view center missing")?,
            cancel,
        )?
        };
        let resident_sectors=assets.resident_sector_count();
        mark("static_assets_us");
        let scene = Scene::new_cancel(assets, cancel)?;
        mark("scene_us");
        stages.insert("archive_sources_reused".into(),serde_json::json!(archive_sources_reused));
        stages.insert("shared_archive_sources_reused".into(),serde_json::json!(shared_archive_sources_reused));
        stages.insert("total_us".into(),serde_json::json!(load_start.elapsed().as_micros()));
        Ok(Terrain{navigation,scene,map_id,sector_count,classic_sector,native_sector,native_block,resident_sectors,geometry_sha,load_profile:serde_json::Value::Object(stages)})
    }
    fn load_native_terrain(root:&Path,world:&World,cancel:&AtomicBool,
        observed_archives:&mut Option<nr::SharedNativeArchives>,
        native_geometry:&mut Option<NativeTerrainSources>)->Result<Terrain> {
        let start=std::time::Instant::now();let mut checkpoint=start;
        let mut stages=serde_json::Map::new();
        let mut mark=|name:&str|{let now=std::time::Instant::now();stages.insert(name.into(),serde_json::json!(now.duration_since(checkpoint).as_micros()));checkpoint=now;};
        let map_id=u32::from(world.map_id.context("world map")?);
        let sources=nr::NativeSources{map_root:root.join("LiTo/LTmap"),archive_root:root.join("LiTo")};
        let geometry_sources_reused=native_geometry.is_some();
        if native_geometry.is_none() {
            let geometry=nr::NativeRetainedGeometry::open(&sources.map_root,map_id,nr::Limits::default(),cancel)?;
            let navigation=std::sync::Arc::new(super::native_navigation::Navigation::from_geometry(geometry.geometry())?);
            *native_geometry=Some(NativeTerrainSources{geometry,navigation});
        }
        let retained=native_geometry.as_ref().context("native geometry owner absent")?;
        retained.geometry.validate_sources(&sources.map_root,map_id,cancel)?;
        let navigation=std::sync::Arc::clone(&retained.navigation);
        let geometry_sha=retained.geometry.geometry().canonical_digest();
        let sector_count=retained.geometry.sector_pins().len();
        mark("geometry_us");
        let archive_sources_reused=observed_archives.is_some();
        if observed_archives.is_none() {
            *observed_archives=Some(nr::SharedNativeArchives::open(&sources.archive_root,nr::Limits::default(),cancel)?);
        }
        let shared=observed_archives.as_ref().context("native archive owner absent")?;
        let shared_archive_sources_reused=shared.reused_source_owner();
        let mut lease=shared.lease(cancel)?;
        let archives=&mut *lease;
        let plan=nr::NativeAssetPlan{namespace:nr::ResourceNamespace::NativeLitoV1,map_id,
            canonical_sha256:geometry_sha,sectors:retained.geometry.sector_pins().to_vec(),archives:archives.pins().to_vec()};
        mark("archive_pins_us");
        let assets=nr::NativeResourceAdapter::pinned_world().load_static_retained_geometry_window(
            &sources,&plan,nr::Limits::default(),archives,&retained.geometry,
            Self::native_view(world).context("native terrain view center missing")?,cancel)?;
        let resident_sectors=assets.resident_sector_count();mark("static_assets_us");
        let scene=Scene::new_cancel(assets,cancel)?;mark("scene_us");
        stages.insert("geometry_sources_reused".into(),serde_json::json!(geometry_sources_reused));
        stages.insert("archive_sources_reused".into(),serde_json::json!(archive_sources_reused));
        stages.insert("shared_archive_sources_reused".into(),serde_json::json!(shared_archive_sources_reused));
        stages.insert("total_us".into(),serde_json::json!(start.elapsed().as_micros()));
        Ok(Terrain{navigation,scene,map_id,sector_count,classic_sector:None,native_sector:Self::native_sector(world),
            native_block:Self::native_block(world),resident_sectors,geometry_sha,load_profile:serde_json::Value::Object(stages)})
    }
    pub(crate) fn source_owner_for(&self,root:&Path)->Option<nr::SharedNativeArchives>{
        (self.root==root).then(||self.observed_archives.clone()).flatten()
    }
    pub fn load(root:&Path,world:&World,cancel:&AtomicBool)->Result<Self>{
        let mut observed_archives=None;
        let mut native_geometry=None;
        let Terrain{navigation,scene,map_id,sector_count,classic_sector,native_sector,native_block,resident_sectors,geometry_sha,load_profile}=Self::load_terrain(root,world,cancel,&mut observed_archives,&mut native_geometry)?;
        let load_start=std::time::Instant::now();let mut checkpoint=load_start;
        let terrain_us=load_profile["total_us"].as_u64().unwrap_or(0);
        let mut stages=load_profile.as_object().cloned().unwrap_or_default();
        let mut mark=|name:&str|{let now=std::time::Instant::now();stages.insert(name.into(),serde_json::json!(now.duration_since(checkpoint).as_micros()));checkpoint=now;};
        let effect_catalog=Some(super::shared_action_catalog::SharedActionCatalog::load_for_model(root,cancel)?);
        mark("catalog_us");
        let sprite_bank = super::native_actor_layers::Bank::open(root)?;
        mark("sprite_archive_us");
        let names = super::native_inventory_names::Catalog::load(root, cancel)?;
        mark("world_names_us");
        stages.insert(
            "total_us".into(),
            serde_json::json!(u128::from(terrain_us)+load_start.elapsed().as_micros()),
        );
        Ok(Self {
            world_generation:world.generation,
            lighting: Default::default(),
            root: root.to_path_buf(),
            names,
            text_cache: Default::default(),
            clan_markers: Default::default(),
            sound_cache: Default::default(),
            wire_audio: Default::default(),
            composed_frames: 0,
            terrain_reloads: 0,
            load_profile: serde_json::Value::Object(stages),
            navigation,
            scene,
            observed_archives,
            native_geometry,
            roofs: Default::default(),
            effect_catalog,
            sprite_bank,
            layer_plans: Default::default(),
            map_id,
            sector_count,
            classic_sector,
            native_sector,
            native_block,
            resident_sectors,
            geometry_sha,

            animator: Default::default(),
            projectiles: Default::default(),
            skill_effects: Default::default(),
            item_effects: super::native_skill_effects::Effects::items(),
            reagent_effects: super::native_skill_effects::Effects::reagent(),
            travel_effects: super::native_skill_effects::Effects::travel(),
            ground_effects: Default::default(),
            ground_sound_until: None,
            spell_projectiles: Default::default(),
            class1_effects: Default::default(),
            disintegrate_effects:Default::default(),
            call_lightning_effects:Default::default(),
            status4183:Default::default(),
            cone_cold_effects:Default::default(),
            static2941: Default::default(),
            effect_source_owner:std::sync::Arc::new(()),
            many_effects:Default::default(),
        })
    }
    pub fn draw(&mut self, world: &World, cancel: &AtomicBool) -> Result<Frame> {
        self.draw_at(world, cancel, 0)
    }
    pub fn draw_at(
        &mut self,
        world: &World,
        cancel: &AtomicBool,
        elapsed_ns: u64,
    ) -> Result<Frame> {
        self.draw_at_labels(world, cancel, elapsed_ns, None)
    }
    pub fn draw_at_labels(
        &mut self,
        world: &World,
        cancel: &AtomicBool,
        elapsed_ns: u64,
        hover: Option<super::native_world_labels::Hover>,
    ) -> Result<Frame> {
        self.draw_at_labels_environment(world, cancel, elapsed_ns, hover, None)
    }
    pub fn draw_at_labels_environment(
        &mut self, world: &World, cancel: &AtomicBool, elapsed_ns: u64,
        hover: Option<super::native_world_labels::Hover>,
        environment: Option<&super::native_render_environment::Bound>,
    ) -> Result<Frame> {
        if let Some(bound) = environment { bound.validate_world(world)?; }
        check(cancel)?;
        // A new world retires every transient model runtime even on the same map.
        // Construct first so cancellation/failure leaves the old model intact.
        if self.world_generation!=world.generation {
            let replacement=Self::load(&self.root,world,cancel)?;
            *self=replacement;
        }
        // Direct controller callers and the scene worker share the same reload
        // contract. Never draw or publish input against an old residence window.
        if !self.covers_world(world) {
            if world.map_id.map(u32::from)==Some(self.map_id){
                let next=Self::load_terrain(&self.root,world,cancel,&mut self.observed_archives,&mut self.native_geometry)?;
                if self.native_block.is_some(){ensure!(next.geometry_sha==self.geometry_sha,"native map sources changed during active world");}
                if next.geometry_sha!=self.geometry_sha{self.roofs=Default::default();}
                self.navigation=next.navigation;self.scene=next.scene;self.map_id=next.map_id;
                self.sector_count=next.sector_count;self.classic_sector=next.classic_sector;
                self.native_sector=next.native_sector;self.native_block=next.native_block;
                self.resident_sectors=next.resident_sectors;self.geometry_sha=next.geometry_sha;
                self.load_profile=next.load_profile;
                self.terrain_reloads=self.terrain_reloads.checked_add(1).context("terrain reload counter")?;
            }else{
                let mut next=Self::load(&self.root,world,cancel)?;
                next.lighting=std::mem::take(&mut self.lighting);
                next.wire_audio=self.wire_audio.clone();
                *self=next;
            }
        }
        let _ = self.static2941.prepare(&self.root, world, cancel, &mut self.effect_catalog)?;
        let render_start = std::time::Instant::now();
        self.status4183.prepare(&self.root,world,cancel,&mut self.effect_catalog);
        self.projectiles.prepare(&self.root, cancel, world);
        self.skill_effects.prepare_with_catalog(&self.root,world,cancel,&mut self.effect_catalog);
        self.item_effects.prepare(&self.root, world, cancel);
        self.reagent_effects.prepare(&self.root, world, cancel);
        self.travel_effects.prepare(&self.root, world, cancel);
        self.ground_effects.prepare_with_catalog(&self.root,world,cancel,&mut self.effect_catalog);
        self.spell_projectiles.prepare_with_catalog(&self.root,world,cancel,&mut self.effect_catalog);
        self.class1_effects.prepare_with_catalog(&self.root,world,cancel,&mut self.effect_catalog);
        self.disintegrate_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;
        self.call_lightning_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;
        self.cone_cold_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;
        self.many_effects.prepare_with_catalog(&self.root,world,cancel,&mut self.effect_catalog);
        let scene = &self.scene;
        ensure!(
            world.map_id.map(u32::from) == Some(self.map_id),
            "renderer map mismatch"
        );
        let anchor = scene.anchor();
        let observer=world.observer_mode;
        let (local_id,local_incarnation,view_position)=if observer {
            ensure!(world.view_ready(),"observer view not ready");
            (world.status.unwrap().object_id,0,world.observer_camera.unwrap())
        } else {let own=world.own_actor().context("local actor")?;(own.id,own.incarnation,[own.x,own.y])};
        let roofs = self.roofs.sample(
            scene.roofs(),
            [world.generation, u64::from(local_id), if observer {world.generation} else {local_incarnation}],
            WorldCell {
                x: i32::from(view_position[0]),
                y: i32::from(view_position[1]),
            },
            elapsed_ns,
        )?;
        let project = |x: f64, y: f64| {
            let dx = x - f64::from(anchor.x);
            let dy = y - f64::from(anchor.y);
            (
                (24. + 24. * (dx + dy)).round() as i32,
                (768. + 12. * (-dx + dy)).round() as i32,
            )
        };
        self.animator
            .begin(elapsed_ns, world.actors.keys().copied())?;
        let mut poses = std::collections::BTreeMap::new();
        for actor in world.actors.values() {
            if super::native_status4183::is_object(world,actor) {continue;}
            if let Some(p)=world.player_walk.as_ref().filter(|p|p.current(world)
                && world.own_actor().is_some_and(|own|own.id==actor.id)) {
                self.animator.source_motion(self.effect_catalog.as_ref().expect("Model catalog ownership").catalog(),
                    actor,elapsed_ns,p.frame==0)?;
            }
            poses.insert(
                actor.id,
                world.player_walk.as_ref().and_then(|p|p.pose(world,actor)).unwrap_or_else(||
                    self.animator.pose(self.effect_catalog.as_ref().expect("Model catalog ownership").catalog(), actor, elapsed_ns)),
            );
        }
        let mut impacts = self.projectiles.update(
            world,
            poses
                .values()
                .filter_map(|p| p.as_ref().ok())
                .flat_map(|p| &p.entered_events),
            elapsed_ns,
        );
        let spell_impacts = self.spell_projectiles.update(
            world,
            poses
                .values()
                .filter_map(|p| p.as_ref().ok())
                .flat_map(|p| &p.entered_events),
            elapsed_ns,
        );
        let spell_keys: std::collections::BTreeSet<_> = spell_impacts
            .iter()
            .map(|i| (i.target, i.sequence, i.projectile))
            .collect();
        impacts.extend(spell_impacts);
        let class1_impacts=self.class1_effects.update(world,
            poses.values().filter_map(|p|p.as_ref().ok()).flat_map(|p|&p.entered_events),elapsed_ns);
        let class1_keys:std::collections::BTreeSet<_>=class1_impacts.iter().map(|i|(i.target,i.sequence,i.projectile)).collect();
        impacts.extend(class1_impacts);
        let disintegrate_launches:Vec<_>=poses.values().filter_map(|p|p.as_ref().ok())
            .flat_map(|p|&p.entered_events).filter_map(|event|super::native_disintegrate::Launch::capture(
                &self.disintegrate_effects,&self.effect_source_owner,world,event)).collect();
        let disintegrate_impacts=self.disintegrate_effects.update(&self.effect_source_owner,world,disintegrate_launches.iter(),elapsed_ns)?;
        let disintegrate_keys:std::collections::BTreeSet<_>=disintegrate_impacts.iter().map(|i|(i.target,i.sequence,i.projectile)).collect();
        impacts.extend(disintegrate_impacts);
        let disintegrate_events=self.disintegrate_effects.entered_events();
        let lightning_launches:Vec<_>=poses.values().filter_map(|p|p.as_ref().ok()).flat_map(|p|&p.entered_events)
            .filter_map(|event|super::native_call_lightning::Launch::capture(&self.call_lightning_effects,&self.effect_source_owner,world,event)).collect();
        let lightning_impacts=self.call_lightning_effects.update(&self.effect_source_owner,world,lightning_launches.iter(),elapsed_ns)?;
        let lightning_keys:std::collections::BTreeSet<_>=lightning_impacts.iter().map(|i|(i.target,i.sequence,i.projectile)).collect();
        impacts.extend(lightning_impacts);
        let lightning_events=self.call_lightning_effects.entered_events();
        let cold_launches:Vec<_>=poses.values().filter_map(|p|p.as_ref().ok()).flat_map(|p|&p.entered_events)
            .filter_map(|event|super::native_cone_cold::Launch::capture(&self.cone_cold_effects,&self.effect_source_owner,world,event)).collect();
        let cold_impacts=self.cone_cold_effects.update(&self.effect_source_owner,world,cold_launches.iter(),elapsed_ns)?;
        let cold_keys:std::collections::BTreeSet<_>=cold_impacts.iter().map(|i|(i.target,i.sequence,i.projectile)).collect();
        impacts.extend(cold_impacts);
        let cold_events=self.cone_cold_effects.entered_events();
        let mut cold_impact_receipts=Vec::new();
        let native_ambient=self.call_lightning_effects.ambient_events().to_vec();
        let mut lightning_impact_receipts=Vec::new();
        let mut finite_hurt_callbacks=Vec::new();
        let many_impacts=self.many_effects.update(world,poses.values().filter_map(|p|p.as_ref().ok()).flat_map(|p|&p.entered_events),elapsed_ns);
        let many_keys:std::collections::BTreeSet<_>=many_impacts.iter().map(|i|(i.target,i.sequence,i.projectile)).collect();
        impacts.extend(many_impacts);
        let mut many_impact_receipts=Vec::new();
        let mut class1_impact_receipts=Vec::new();
        let mut disintegrate_impact_receipts=Vec::new();
        let mut impact_receipts = Vec::new();
        let mut spell_impact_receipts = Vec::new();
        for impact in impacts {
            let Some(actor) = world.actors.get(&impact.target) else {
                continue;
            };
            let accepted = self.animator.impact(actor, impact.sequence);
            let row = serde_json::json!({"target":impact.target,"sequence":impact.sequence,"projectile":impact.projectile,"accepted":accepted});
            if lightning_keys.contains(&(impact.target,impact.sequence,impact.projectile)){finite_hurt_callbacks.push(HurtCallback::capture(world,actor,self.effect_source_owner.clone(),impact.sequence,impact.projectile,accepted,elapsed_ns,11736));lightning_impact_receipts.push(row);}
            else if cold_keys.contains(&(impact.target,impact.sequence,impact.projectile)){finite_hurt_callbacks.push(HurtCallback::capture(world,actor,self.effect_source_owner.clone(),impact.sequence,impact.projectile,accepted,elapsed_ns,11738));cold_impact_receipts.push(row);}
            else if disintegrate_keys.contains(&(impact.target,impact.sequence,impact.projectile)){disintegrate_impact_receipts.push(row);}
            else if many_keys.contains(&(impact.target,impact.sequence,impact.projectile)){many_impact_receipts.push(row);}
            else if class1_keys.contains(&(impact.target, impact.sequence, impact.projectile)) {
                class1_impact_receipts.push(row);
            } else if spell_keys.contains(&(impact.target, impact.sequence, impact.projectile)) {
                spell_impact_receipts.push(row);
            } else {
                impact_receipts.push(row);
            }
            if accepted {
                // Re-sample only the affected pose at the same timestamp. Keep
                // its earlier frame events, including any outgoing Hit cue.
                let mut next = self.animator.pose(self.effect_catalog.as_ref().expect("Model catalog ownership").catalog(), actor, elapsed_ns);
                if let (Some(Ok(previous)), Ok(next)) = (poses.get_mut(&actor.id), &mut next) {
                    let mut events = std::mem::take(&mut previous.entered_events);
                    events.append(&mut next.entered_events);
                    next.entered_events = events;
                }
                poses.insert(actor.id, next);
            }
        }
        let own_pose=if observer {None} else {Some(poses.get(&local_id).context("local pose absent")?.as_ref().map_err(|e|anyhow::anyhow!("local actor animation: {e:#}"))?)};
        let position=own_pose.map_or([f64::from(view_position[0]),f64::from(view_position[1])],|p|p.position);
        let (px,py)=project(position[0],position[1]);
        let camera = CameraRect {
            x: px - 400,
            y: py - 300,
            width: 800,
            height: 600,
        };
        if let Some(bound)=environment {
            let motion=if observer {None}else{world.own_actor().and_then(|actor|actor.motion_from.map(|source| {
                let start=project(f64::from(source[0]),f64::from(source[1]));
                let end=project(f64::from(actor.x),f64::from(actor.y));
                super::native_water_clock::Motion{identity:[u64::from(actor.id),actor.incarnation,actor.motion_revision],
                    source,destination:[actor.x,actor.y],offset:[px-start.0,py-start.1],extent:[end.0-start.0,end.1-start.1]}
            }))};
            self.lighting.sample_water_camera(bound.key().epoch,world.generation,motion)?;
        }
        let projection = super::world_map::ViewProjection {
            anchor_cell: anchor,
            anchor_pixel: super::world_map::PixelPoint {
                x: 24 - camera.x,
                y: 768 - camera.y,
            },
        };
        let mut actors: Vec<_> = world
            .actors
            .values()
            .filter(|a| {
                let (x, y) = project(f64::from(a.x), f64::from(a.y));
                (x - camera.x).abs() < 1000 && (y - camera.y).abs() < 800
            })
            .collect();
        ensure!(actors.len() <= 128, "visible actor budget");
        actors.sort_by_key(|a| {
            let p = poses
                .get(&a.id)
                .and_then(|r| r.as_ref().ok())
                .map(|p| p.position)
                .unwrap_or([f64::from(a.x), f64::from(a.y)]);
            let (x, y) = project(p[0], p[1]);
            (y, x, a.id)
        });
        let mut actor_receipts = Vec::new();
        let mut hits = Vec::new();
        let mut audible_actors = Vec::new();
        let mut actor_states = Vec::new();
        let mut ground_items = Vec::new();
        let mut pixels = 0usize;
        let mut prepared_bodies = Vec::new();
        let needs_green=actors.iter().any(|actor|actor.draw_state.green
            && actor.draw_state.visible(!observer&&actor.id==local_id));
        let green_counter=match environment {
            Some(bound) if needs_green=>Some(bound.actor_green_counter(world)?),
            Some(bound)=>bound.actor_green_counter(world).ok(),
            // Explicit resource-only unbound draws retain a fixed inspection
            // phase. The normal world presenter always supplies an owned binding.
            None=>None,
        };
        for actor in actors {
            check(cancel)?;
            if super::native_status4183::is_object(world,actor) {continue;}
            let style = actor.draw_state.style(!observer && actor.id == local_id, green_counter.unwrap_or(0));
            if style == super::native_actor_states::Style::Hidden {
                audible_actors.push(actor.id);
                actor_receipts.push(serde_json::json!({"id":actor.id,"logical_graphics":actor.graphics,"rendered":false,"hidden":true,"draw_state":actor.draw_state,"draw_style":style}));
                continue;
            }
            // Frame selection and display interpolation share one monotonic clock.
            let draw_result = (|| -> Result<serde_json::Value> {
                let pose = poses
                    .get(&actor.id)
                    .context("actor pose absent")?
                    .as_ref()
                    .map_err(|e| anyhow::anyhow!("{e:#}"))?;
                let (graphics, image, subframe) = (pose.graphics, pose.image, pose.subframe);
                let name = format!("{graphics}-{image}.spr");
                self.sprite_bank.ensure(&name, cancel)?;
                let decoded = &self.sprite_bank.get(&name)?.sprite;
                ensure!(
                    !pose.static_prop || decoded.frames.len() == 1,
                    "static prop is not a single-frame sprite"
                );
                let sprite = decoded
                    .frames
                    .get(subframe as usize)
                    .context("actor SPR subframe")?;
                pixels += sprite.rgba.len();
                ensure!(pixels <= 64 << 20, "world actor pixel budget");
                let (x, y) = project(pose.position[0], pose.position[1]);
                ensure!(sprite.fragments.is_some(), "body source fragments missing");
                let bounds = [
                    x - camera.x - 24 + sprite.x_offset,
                    y - camera.y - 12 + sprite.y_offset,
                    x - camera.x - 24 + sprite.x_offset + sprite.width as i32,
                    y - camera.y - 12 + sprite.y_offset + sprite.height as i32,
                ];
                let body_source = self.sprite_bank.get(&name)?.source.clone();
                if !self.layer_plans.contains_key(&actor.graphics) {
                    self.layer_plans.insert(
                        actor.graphics,
                        super::native_actor_layers::Plan::from_catalog(
                            self.effect_catalog.as_ref().expect("Model catalog ownership").catalog(),
                            u32::from(actor.graphics),
                        )
                        .map_err(|e| format!("{e:#}")),
                    );
                }
                let layer_result = (|| -> Result<super::native_actor_layers::Prepared> {
                    let plan = self.layer_plans[&actor.graphics]
                        .as_ref()
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    let bindings = plan.bind(image)?;
                    self.sprite_bank
                        .prepare(&bindings, subframe as usize, cancel)
                })();
                check(cancel)?;
                let (prepared, layer_error) = match layer_result {
                    Ok(b) => (b, None),
                    Err(e) => (Default::default(), Some(format!("{e:#}"))),
                };
                let bindings = prepared.bindings;
                let missing_attachments = prepared.missing_attachments;
                let missing_shadow = prepared.missing_shadow;
                let mut layer_receipts = Vec::new();
                for (member, mix) in &bindings.attachments {
                    layer_receipts.push(serde_json::json!({"kind":"attachment","mix":mix,"source":self.sprite_bank.get(member)?.source,"subframe":subframe,"body_linked":true}));
                }
                if let Some(member) = &bindings.shadow {
                    layer_receipts.push(serde_json::json!({"kind":"shadow","mix":"Shadow","source":self.sprite_bank.get(member)?.source,"subframe":subframe,"body_linked":true}));
                }
                let presentation = super::native_actor_states::Presentation {
                    state: style,
                    body_half: bindings.body_half,
                };
                actor_states.push(presentation);
                audible_actors.push(actor.id);
                prepared_bodies.push((
                    actor.id,
                    name.clone(),
                    subframe as usize,
                    super::world_map::PixelPoint {
                        x: x - camera.x,
                        y: y - camera.y,
                    },
                    bindings,
                ));
                if pose.phase == "ground_item" {
                    ground_items.push(actor.clone());
                }
                hits.push((actor.id, bounds));
                Ok(
                    serde_json::json!({"rendered":true,"id":actor.id,"draw_state":actor.draw_state,"draw_style":style,"body_presentation":presentation,"body_draw_style":presentation.body_style(),"logical_graphics":actor.graphics,"resolved_graphics":graphics,"heading":actor.heading,"world":[actor.x,actor.y],"screen":[x-camera.x,y-camera.y],"archive":body_source.archive,"speed":actor.speed,"brave":actor.brave,"stance":actor.walk_action,"action":pose.action,"phase":pose.phase,"display_position":pose.position,"source_frame":pose.source_frame,"source_frame_events":pose.events,"action_complete":pose.action_complete,"entered_events":pose.entered_events,"sprite_member":name,"sprite_subframe":subframe,"body_source":body_source,"layers":layer_receipts,"layer_error":layer_error,"missing_attachments":missing_attachments,"missing_shadow":missing_shadow,"native_dynamic_draw_flags_complete":false}),
                )
            })();
            match draw_result {
                Ok(receipt) => actor_receipts.push(receipt),
                Err(error) => {
                    if !observer && actor.id == local_id {
                        return Err(error.context("local actor rendering"));
                    }
                    actor_receipts.push(serde_json::json!({"id":actor.id,"logical_graphics":actor.graphics,"action":actor.walk_action,"rendered":false,"error":format!("{error:#}")}));
                }
            }
        }
        let bodies: Vec<_> = prepared_bodies
            .iter()
            .map(|(id, name, sub, center, bindings)| -> Result<_> {
                Ok(super::native_world_layers::LayeredBody {
                    body: super::native_world_layers::Body {
                        id: *id,
                        center: *center,
                        frame: self.sprite_bank.frame(name, *sub)?,
                    },
                    attachments: bindings
                        .attachments
                        .iter()
                        .map(|(member, mix)| {
                            Ok(super::native_world_layers::Attachment {
                                frame: self.sprite_bank.frame(member, *sub)?,
                                mix: *mix,
                            })
                        })
                        .collect::<Result<_>>()?,
                    shadow: bindings
                        .shadow
                        .as_ref()
                        .map(|member| self.sprite_bank.frame(member, *sub))
                        .transpose()?,
                })
            })
            .collect::<Result<_>>()?;
        let mut effect_queue = Vec::new();
        let mut effect_target = super::native_world_effects::Target::Fragments(&mut effect_queue);
        let mut projectile_receipt = self.projectiles.draw_to(
            &self.root,
            cancel,
            &mut effect_target,
            super::world_map::ViewProjection {
                anchor_cell: super::world_map::WorldCell {
                    x: anchor.x,
                    y: anchor.y,
                },
                anchor_pixel: super::world_map::PixelPoint {
                    x: 24 - camera.x,
                    y: 768 - camera.y,
                },
            },
            elapsed_ns,
        );
        projectile_receipt["hurt_callbacks"] = serde_json::json!(impact_receipts);
        let (mut spell_projectile_receipt, spell_events) = self.spell_projectiles.draw_to(
            &mut effect_target,
            super::world_map::ViewProjection {
                anchor_cell: anchor,
                anchor_pixel: super::world_map::PixelPoint {
                    x: 24 - camera.x,
                    y: 768 - camera.y,
                },
            },
            elapsed_ns,
        );
        spell_projectile_receipt["hurt_callbacks"] = serde_json::json!(spell_impact_receipts);
        let (mut class1_receipt,class1_events)=self.class1_effects.draw_to(&mut effect_target,
            super::world_map::ViewProjection{anchor_cell:anchor,anchor_pixel:super::world_map::PixelPoint{x:24-camera.x,y:768-camera.y}},elapsed_ns);
        class1_receipt["hurt_callbacks"]=serde_json::json!(class1_impact_receipts);
        let mut disintegrate_receipt=self.disintegrate_effects.snapshot();
        disintegrate_receipt["hurt_callbacks"]=serde_json::json!(disintegrate_impact_receipts);
        let disintegrate_draws=self.disintegrate_effects.draws(&self.effect_source_owner)?;
        for draw in &disintegrate_draws {
            draw.submit_to(&mut effect_target,super::world_map::ViewProjection{anchor_cell:anchor,
                anchor_pixel:super::world_map::PixelPoint{x:24-camera.x,y:768-camera.y}})?;
        }
        disintegrate_receipt["effect_submissions"]=serde_json::json!(disintegrate_draws.len()*2);
        let mut cold_receipt=self.cone_cold_effects.snapshot();
        cold_receipt["hurt_callbacks"]=serde_json::json!(cold_impact_receipts);
        let cold_draws=self.cone_cold_effects.draws(&self.effect_source_owner)?;
        for draw in &cold_draws {
            draw.submit_to(&mut effect_target,super::world_map::ViewProjection{anchor_cell:anchor,anchor_pixel:super::world_map::PixelPoint{x:24-camera.x,y:768-camera.y}})?;
        }
        cold_receipt["effect_submissions"]=serde_json::json!(cold_draws.len()*2);
        let mut lightning_receipt=self.call_lightning_effects.snapshot();
        lightning_receipt["hurt_callbacks"]=serde_json::json!(lightning_impact_receipts);
        let lightning_draws=self.call_lightning_effects.draws(&self.effect_source_owner)?;
        for draw in &lightning_draws {
            draw.submit_to(&mut effect_target,super::world_map::ViewProjection{anchor_cell:anchor,anchor_pixel:super::world_map::PixelPoint{x:24-camera.x,y:768-camera.y}})?;
        }
        lightning_receipt["effect_submissions"]=serde_json::json!(lightning_draws.len());
        let (mut many_receipt,many_events)=self.many_effects.draw_to(&mut effect_target,
            super::world_map::ViewProjection{anchor_cell:anchor,anchor_pixel:super::world_map::PixelPoint{x:24-camera.x,y:768-camera.y}},elapsed_ns);
        many_receipt["hurt_callbacks"]=serde_json::json!(many_impact_receipts);
        let (skill_effect_receipt, effect_events) = self.skill_effects.draw_to(
            world,
            &mut effect_target,
            super::world_map::ViewProjection {
                anchor_cell: anchor,
                anchor_pixel: super::world_map::PixelPoint {
                    x: 24 - camera.x,
                    y: 768 - camera.y,
                },
            },
            elapsed_ns,
        );
        let (item_effect_receipt, item_events) = self.item_effects.draw_to(
            world,
            &mut effect_target,
            super::world_map::ViewProjection {
                anchor_cell: anchor,
                anchor_pixel: super::world_map::PixelPoint {
                    x: 24 - camera.x,
                    y: 768 - camera.y,
                },
            },
            elapsed_ns,
        );
        let (reagent_effect_receipt, reagent_events) = self.reagent_effects.draw_to(
            world,
            &mut effect_target,
            super::world_map::ViewProjection {
                anchor_cell: anchor,
                anchor_pixel: super::world_map::PixelPoint {
                    x: 24 - camera.x,
                    y: 768 - camera.y,
                },
            },
            elapsed_ns,
        );
        let (travel_effect_receipt, travel_events) = self.travel_effects.draw_to(
            world, &mut effect_target,
            super::world_map::ViewProjection { anchor_cell: anchor,
                anchor_pixel: super::world_map::PixelPoint { x: 24-camera.x, y: 768-camera.y } },
            elapsed_ns,
        );
        let (ground_effect_receipt, ground_events) = self.ground_effects.draw_to(
            world,
            &mut effect_target,
            super::world_map::ViewProjection {
                anchor_cell: anchor,
                anchor_pixel: super::world_map::PixelPoint {
                    x: 24 - camera.x,
                    y: 768 - camera.y,
                },
            },
            elapsed_ns,
        );
        let static2941 = self.static2941.draw_to(world, &mut effect_target, projection, cancel)?;
        let (status4183_receipt,status4183_audio)=self.status4183.draw_to(world,&mut effect_target,projection)?;
        let effect_submissions = effect_queue.len();
        let terrain_start = std::time::Instant::now();
        let layers = super::native_world_layers::compose_with_presentations(
            scene,
            projection,
            800,
            600,
            &bodies,
            &roofs,
            &effect_queue,
            &actor_states,
            cancel,
        )?;
        let terrain_us = terrain_start.elapsed().as_micros();
        let background_gate_receipt=layers.background_gate.as_ref().map(|g|serde_json::json!({"dimensions":g.dimensions(),"covered":g.covered(),
            "sha256":format!("{:X}",Sha256::digest(g.values())),"provenance":"encoded_floor_and_plain_first_pass_object_writes",
            "source_provenance_complete":g.provenance_complete(),"unqualified_object_fragments":g.unqualified_fragments(),
            "background_and_viewport_complete":false,"unsupported_producer_flags_complete":false}));
        let mut canvas = layers.canvas;
        let lighting_receipt = if let Some(bound) = environment {
            let mut source_cells=std::collections::BTreeMap::new();
            let mut source_actions=std::collections::BTreeMap::new();
            let centers = world.actors.values().map(|actor| {
                let pose=poses.get(&actor.id).and_then(|p|p.as_ref().ok());
                let position=pose.map_or([f64::from(actor.x),f64::from(actor.y)],|p|p.position);
                let cell=pose.filter(|p|p.phase=="walk").and_then(|_|actor.motion_from).unwrap_or([actor.x,actor.y]);
                source_cells.insert(actor.id,cell);
                if let Some(pose)=pose {source_actions.insert(actor.id,pose.action);}
                let (x,y)=project(position[0],position[1]);
                (actor.id,[x-camera.x,y-camera.y+6])
            }).collect();
            let mut receipt=self.lighting.draw_with_actions(&self.root,self.effect_catalog.as_ref().expect("Model catalog ownership").catalog(),world,bound,&centers,&source_cells,&source_actions,&mut canvas,cancel)?;
            receipt["water_extra"]=self.lighting.apply_water(&self.root,bound,&mut canvas,cancel)?;
            receipt["water_extra_effect_complete"]=serde_json::json!(bound.key().map!=303);
            receipt["background_gate"]=serde_json::json!(background_gate_receipt);
            receipt
        } else {serde_json::json!({"active":false,"reason":"resource_only_unbound_draw"})};
        let own_x = prepared_bodies
            .iter()
            .find(|(id, _, _, _, _)| *id == local_id)
            .map_or(400, |(_, _, _, center, _)| center.x);
        let label_lines =
            super::native_world_labels::layout(world, &self.names, &hits, own_x, hover);
        let mut label_receipt = super::native_world_labels::draw(
            &mut canvas,
            &mut self.text_cache,
            &label_lines,
            cancel,
        )?;
        label_receipt["sources"] = serde_json::json!(self.names.sources);
        let emblem_receipt = super::native_emblems::draw(&world.emblems, world, &label_lines, &mut canvas, cancel)?;
        let marker_centers = prepared_bodies.iter().map(|(id, _, _, center, _)| (*id, *center)).collect();
        let marker_receipt = self.clan_markers.draw(&self.root, world, &marker_centers, &mut canvas, elapsed_ns, cancel)?;
        let minimap_receipt = super::native_minimap::draw(world, &self.navigation, &mut canvas, &mut self.text_cache, cancel)?;
        self.composed_frames = self.composed_frames.saturating_add(1);
        let hash_start = std::time::Instant::now();
        let mut sounds = Vec::new();
        let mut legacy_audio = Vec::new();
        let mut native_audio = status4183_audio;
        let mut audio_receipts = Vec::new();
        let wire_audio = self.wire_audio.take(world);
        let audible = || {
            poses
                .values()
                .filter_map(|p| p.as_ref().ok())
                .flat_map(|p| &p.entered_events)
                .chain(effect_events.iter())
                .chain(item_events.iter())
                .chain(reagent_events.iter())
                .chain(travel_events.iter())
                .chain(spell_events.iter())
                .chain(class1_events.iter())
                .chain(disintegrate_events.iter())
                .chain(lightning_events.iter())
                .chain(cold_events.iter())
                .chain(many_events.iter())
                .chain(ground_events.iter())
                .chain(wire_audio.events.iter())
                .filter(|e| {
                    (e.sound.is_some() || e.prepared_audio.is_some()) && (e.actor == 0 || audible_actors.contains(&e.actor))
                })
        };
        let audio_budget = 32usize.saturating_sub(native_audio.len());
        let audio_dropped = audible().count().saturating_sub(audio_budget);
        for event in audible().take(audio_budget) {
            if let Some(cue)=&event.prepared_audio {
                let mut row=serde_json::json!({"event":event,"prepared":true,"native_group":true,
                    "mode":format!("{:?}",cue.group().mode),"candidates":cue.group().candidates()});
                if let Some(sound)=cue.prepared_sound() {
                    row["bytes"]=serde_json::json!(sound.file_bytes());
                    row["sha256"]=serde_json::json!(sound.sha256().iter().map(|b|format!("{b:02x}")).collect::<String>());
                } else {row["silent"]=serde_json::json!(true);}
                audio_receipts.push(row);native_audio.push(cue.clone());continue;
            }
            let id = event.sound.unwrap();
            let loaded = (|| -> Result<super::combat_audio::PreparedSound> {
                ensure!(sounds.len() < 32, "frame sound budget");
                if !self.sound_cache.contains_key(&id) {
                    ensure!(self.sound_cache.len() < 128, "sound cache entries");
                    let bytes = res::read_shared_sound_with_limit(
                        &self.root,
                        res::SharedSoundKey { sound_id: id },
                        1 << 20,
                    )?;
                    ensure!(
                        self.sound_cache
                            .values()
                            .map(|s| s.file_bytes())
                            .sum::<usize>()
                            + bytes.len()
                            <= 16 << 20,
                        "sound cache bytes"
                    );
                    self.sound_cache
                        .insert(id, super::combat_audio::PreparedSound::from_wav(id, bytes)?);
                }
                Ok(self.sound_cache[&id].clone())
            })();
            match loaded {
                Ok(sound) => {
                    let mut row=serde_json::json!({"event":event,"prepared":true,"delivery_kind":"legacy_scoped","bytes":sound.file_bytes(),"sha256":sound.sha256().iter().map(|b|format!("{b:02x}")).collect::<String>()});
                    let mut suppressed=false;
                    if event.event=="GroundSound(111)" {
                        suppressed=self.ground_sound_until.is_some_and(|(generation,until)|generation==world.generation && elapsed_ns<until);
                        row["same_id_suppressed"]=serde_json::json!(suppressed);
                        if !suppressed {
                            let ns=(sound.pcm_bytes().len() as u128*1_000_000_000).div_ceil(u128::from(sound.format().byte_rate)).min(u128::from(u64::MAX)) as u64;
                            self.ground_sound_until=Some((world.generation,elapsed_ns.saturating_add(ns)));
                        }
                    }
                    audio_receipts.push(row);if !suppressed {legacy_audio.push(super::native_audio_delivery::legacy::PreparedLegacyCue::from_event(sound.clone(),event)?);sounds.push(sound);}
                }
                Err(error) => audio_receipts.push(serde_json::json!({"event":event,"prepared":false,"error":format!("{error:#}")})),
            }
        }
        let rgba_sha256 = format!("{:x}", Sha256::digest(canvas.rgba()));
        let hash_us = hash_start.elapsed().as_micros();
        let mut receipt = serde_json::json!({"map_id":self.map_id,"world_generation":world.generation,"world_revision":world.revision,"visual_revision":world.visual_revision,"cache_entries":self.sprite_bank.len(),"cache_bytes":self.sprite_bank.bytes(),"local_object_id":local_id,"terrain_namespace":if self.classic_sector.is_some(){"Classic map/4 + Tile"}else{"LiTo/LTmap + LTtile"},"map_sectors":self.sector_count,"terrain_resident_sectors":self.resident_sectors,"native_terrain_center":self.native_sector,"terrain_reload_count":self.terrain_reloads,"native_terrain_block":self.native_block,"native_terrain_block_span":self.native_block.map(|_|8),"native_terrain_window_pixels":self.native_block.map(|_|[1328,864]),"geometry_sha256":self.geometry_sha.iter().map(|b|format!("{b:02x}")).collect::<String>(),"actors":actor_receipts,"rgba_sha256":rgba_sha256,"width":800,"height":600,"static_actor_frame":false,"elapsed_ns":elapsed_ns,"continuous_animation":true,"common_speed_modifiers":true,"source_speed_scaling_complete":false,"local_navigation":"LTmap Layer3 native directional edges","actor_terrain_occlusion_complete":false,"effects_complete":false});
        let mut bgra = Vec::new();
        let bgra_start = std::time::Instant::now();
        canvas.copy_bgra(&mut bgra)?;
        let bgra_us = bgra_start.elapsed().as_micros();
        receipt["minimap"] = minimap_receipt;
        receipt["world_labels"] = label_receipt;
        receipt["status4183"] = status4183_receipt;
        receipt["emblems"] = emblem_receipt;
        receipt["clan_markers"] = marker_receipt;
        receipt["roofs"] = roofs.receipt().clone();
        receipt["roof_definition"] = scene.roofs().receipt();
        receipt["fragment_composition"] = serde_json::json!({"terrain_and_bodies":true,"body_linked_layers":true,"dynamic_draw_flags_complete":false,"actor_states":true,"poison_tick":elapsed_ns/50_000_000,"see_invisible":false,"pixel_ownership":true,"roof_runtime":true,"effect_interleaving":true,"effect_submissions":effect_submissions,"effect_ownership":"click_through","effect_insertion_policy":"profile_then_active_snapshot_order","insertion_policy":"canonical terrain source then projected body roster; not native runtime history","interpolation":"nearest whole-cell depth anchor plus residual pixel translation"});
        receipt["load_profile_us"] = self.load_profile.clone();
        receipt["static_resource_budget"] = serde_json::json!({
            "draw_references":self.scene.assets().draw_references(),"draw_limit":self.scene.assets().draw_reference_limit(),
            "decoded_slots":self.scene.assets().decoded_slots(),"decoded_bytes":self.scene.assets().decoded_bytes(),
            "decoded_limit_bytes":64<<20,"complete_sector_count":self.sector_count});
        receipt["view_mode"]=serde_json::json!(if observer {"observer"} else {"participant"});
        receipt["view_position"]=serde_json::json!(view_position);
        receipt["audio"] = serde_json::json!(audio_receipts);
        receipt["projectiles"] = projectile_receipt;
        receipt["skill_effects"] = skill_effect_receipt;
        receipt["item_effects"] = item_effect_receipt;
        receipt["reagent_effects"] = reagent_effect_receipt;
        receipt["travel_effects"] = travel_effect_receipt;
        receipt["ground_effects"] = ground_effect_receipt;
        receipt["spell_projectiles"] = spell_projectile_receipt;
        receipt["class1_effects"] = class1_receipt;
        receipt["disintegrate_effects"]=disintegrate_receipt;
        receipt["call_lightning_effects"]=lightning_receipt;
        receipt["cone_cold_effects"]=cold_receipt;
        receipt["many_effects"]=many_receipt;
        receipt["audio_events_dropped"] = serde_json::json!(audio_dropped);
        receipt["wire_audio"] = serde_json::json!({"delivered":wire_audio.events.len(),"frame_budget_dropped":wire_audio.dropped,"unsupported_kind":wire_audio.unsupported,"world_queue_dropped":world.wire_sounds_dropped});
        receipt["actor_light_ranges"] = serde_json::json!(world.actors.iter().map(|(id,a)|(*id,a.light)).collect::<std::collections::BTreeMap<_,_>>());
        receipt["actor_green_clock"]=serde_json::json!({"counter":green_counter,
            "delivered_callback_phase":green_counter.is_some(),"resource_only_fixed_phase":environment.is_none(),
            "scene_elapsed_advances_phase":false});
        receipt["world_lighting"] = lighting_receipt;
        receipt["render_profile_us"] = serde_json::json!({"total":render_start.elapsed().as_micros(),"terrain":terrain_us,"rgba_sha256":hash_us,"bgra":bgra_us,"composition_frames":self.composed_frames,"terrain_fragments":layers.terrain_fragments,"prepared_terrain_bytes":scene.prepared_bytes(),"hit_surface_bytes":800*600*2});
        receipt["render_environment"] = match environment {
            Some(bound) => serde_json::json!({"bound":true,"key":bound.key(),
                "captured_authority_revision":bound.state().revision,"captured_state":bound.state()}),
            None => serde_json::json!({"bound":false,"scope":"resource-only unbound draw"}),
        };
        receipt["static2941"] = serde_json::json!({"sequences":static2941.sequences(),"flags":0,"direction6_alias":-1,"shadow":-1,"paint_advances_clock":false});
        Ok(Frame {
            status4183_leases:world.status4183.leases(),
            player_walk_plan: world.own_actor().and_then(|actor|
                super::native_player_callbacks::WalkPlan::load(
                    self.effect_catalog.as_ref().expect("Model catalog ownership").catalog(),actor).ok()).map(std::sync::Arc::new),
            static2941,
            ground_items,
            hits,
            visibility: layers.visibility,
            sounds,
            legacy_audio,
            native_audio,
            native_ambient,
            ambient_owner:self.effect_source_owner.clone(),
            finite_hurt_callbacks,
            navigation: self.navigation.clone(),
            movement_key: if observer {(local_id,0,view_position[0],view_position[1],0,0,0)} else {let own=world.own_actor().unwrap();(own.id,own.motion_revision,own.x,own.y,own.speed,own.brave,own.action_sequence)},
            movement_arrived: own_pose.is_some_and(|p|p.phase=="idle"),
            movement_stamp: super::native_navigation::InputStamp::capture(0, world),
            death_arrived: own_pose.is_some_and(|p|p.phase=="death"&&p.action_complete),
            movement_generation: world.generation,
            observer_key: observer.then_some((local_id,view_position)),
            caster_incarnation: local_incarnation,
            input_revision: world.visual_revision,
            environment_key: environment.map(|bound|bound.key()),
            projection: super::world_map::ViewProjection {
                anchor_cell: anchor,
                anchor_pixel: super::world_map::PixelPoint {
                    x: 24 - camera.x,
                    y: 768 - camera.y,
                },
            },
            canvas,
            bgra,
            receipt,
        })
    }
}

#[cfg(test)]
pub(crate) mod resurrection_tests {
    use super::*;

    pub(crate) fn completed_frame(world: &World) -> Frame {
        let mut frame = movement_tests::frame(world, true);
        frame.death_arrived = true;
        frame
    }

    fn dead_world() -> World {
        let mut world = movement_tests::world();
        let mut body = vec![0x20];
        body.extend(world.own_actor().unwrap().id.to_le_bytes());
        body.push(8);
        world.accept(&[0x0d, 0, 0, 200, 0]).unwrap();
        world.accept(&body).unwrap();
        world.accept(&[0xe3]).unwrap();
        world
    }

    #[test]
    fn completed_death_frame_rechecks_authority_before_acknowledging() {
        let world = dead_world();
        let mut frame = movement_tests::frame(&world, true);
        assert!(!frame.death_ready(&world), "movement completion is not death completion");
        frame.death_arrived = true;
        assert!(frame.death_ready(&world));
        for case in 0..4 {
            let mut changed = world.clone();
            match case {
                0 => { changed.accept(&[0x0d, 1, 0, 200, 0]).unwrap(); }
                1 => { changed.set_observer_mode(true); }
                2 => { changed.accept(&[0xd3, 0, 0, 0, 0]).unwrap(); }
                _ => { changed.acknowledge_death(|_| Ok(())).unwrap(); }
            }
            assert!(!frame.death_ready(&changed), "case {case}");
        }
    }

    #[test]
    fn old_death_frame_cannot_acknowledge_the_next_life_even_at_the_same_cell() {
        let mut world = dead_world();
        let mut frame = movement_tests::frame(&world, true);
        frame.death_arrived = true;
        let id = world.own_actor().unwrap().id;
        let mut body = vec![0x11];
        body.extend(id.to_le_bytes()); body.push(0);
        body.extend(id.to_le_bytes()); body.extend(0u16.to_le_bytes());
        world.accept(&body).unwrap();
        assert!(!frame.death_ready(&world));
        let mut death = vec![0x20]; death.extend(id.to_le_bytes()); death.push(8);
        world.accept(&death).unwrap(); world.accept(&[0xe3]).unwrap();
        assert!(world.death_confirmation_ready());
        assert!(!frame.death_ready(&world));
        let mut fresh = movement_tests::frame(&world, true);
        fresh.death_arrived = true;
        assert!(fresh.death_ready(&world));
    }
}
