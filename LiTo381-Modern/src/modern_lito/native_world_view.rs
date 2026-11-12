//! One render job, cached map resources, and generation-fenced publication.
use super::{
    native_world::World,
    native_world_scene::{Frame, Job, Model},
};
use std::{path::{Path,PathBuf},sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub connection: u64,
    pub generation: u64,
    pub map: u16,
}
#[derive(Default)]
pub struct Fence {
    key: Option<Key>,
}
impl Fence {
    pub fn select(&mut self, key: Option<Key>) -> bool {
        let changed = self.key != key;
        self.key = key;
        changed
    }
    pub fn accepts(&self, key: Key) -> bool {
        self.key == Some(key)
    }
}
struct PendingRender {
    key: Key,
    source_root: PathBuf,
    publication: Arc<()>,
    revision: u64,
    effect_sequences:std::collections::BTreeSet<u64>,
    environment: super::native_render_environment::Key,
    job: Job,
}
#[derive(Default)]
pub struct Presentation {
    static2941_ready: Vec<super::native_static2941::Prepared>,
    pub hover: Option<super::native_world_labels::Hover>,
    fence: Fence,
    job: Option<PendingRender>,
    publication: Arc<()>,
    model: Option<Model>,
    source_owner: Option<super::native_resources::SharedNativeArchives>,
    source_root: Option<std::path::PathBuf>,
    keep_source_owner: bool,
    light_owner: Option<u64>,
    light_remap: super::native_light_color::Remap,
    water_history:super::native_water_clock::History,
    rendered_revision: Option<u64>,
    rendered_environment: Option<super::native_render_environment::Key>,
    deferred_sounds: Vec<super::combat_audio::PreparedSound>,
    deferred_legacy_audio: Vec<super::native_audio_delivery::legacy::PreparedLegacyCue>,
    deferred_native_audio: Vec<super::native_audio_delivery::PreparedCue>,
    finite_audio_history:super::native_audio_delivery::FiniteDeliveryHistory,
    ordinary_wire_cursor:super::native_wire_audio::Cursor,
    ordinary_admission:super::native_world_animation::Admission,
    ambient_bridge:super::native_ambient_bridge::Bridge,
    pending_ambient:Vec<super::native_ambient_bridge::Prepared>,
    ambient_source_floor:u64,
    ambient_source_owner:Option<Arc<()>>,
    deferred_finite_hurt:Vec<super::native_world_scene::HurtCallback>,
    pub retired_environment_results: u64,
    pub retired_environment_errors: u64,
    pub retired_context_results: u64,
    pub retired_context_errors: u64,
    clock: Option<std::time::Instant>,
    clock_owner:Option<Key>,
    finite_visual_history:Option<(Key,PathBuf,super::native_world_scene::FiniteHistory)>,
    last_request: Option<std::time::Instant>,
    pub frame: Option<Frame>,
    pub error: Option<String>,
}
impl Presentation {
    pub fn cancel(&mut self) {
        self.static2941_ready.clear();
        self.publication=Arc::new(());
        self.keep_source_owner=false;
        self.source_owner=None;
        self.source_root=None;
        self.fence.select(None);
        self.hover = None;
        self.frame = None;
        self.model = None;
        self.light_owner = None;
        self.light_remap = Default::default();
        self.water_history.detach_scene();
        self.error = None;
        self.rendered_revision = None;
        self.rendered_environment = None;
        for cue in &self.deferred_legacy_audio {cue.discard();}
        self.deferred_legacy_audio.clear();
        self.deferred_sounds.clear();
        self.deferred_finite_hurt.clear();
        self.pending_ambient.clear();
        self.ambient_source_floor=0;
        self.ambient_source_owner=None;
        for cue in &self.deferred_native_audio {cue.discard();}
        self.deferred_native_audio.clear();
        // The world-owned callback clock/history survives resource cancellation.
        // A different connection/generation/map retires it at the next active tick.
        self.last_request = None;
        if let Some(pending) = &self.job {
            pending.job.cancel();
        }
    }
    pub fn audio_scope(&self,connection:u64,generation:u64)->super::native_audio_delivery::Scope {
        super::native_audio_delivery::Scope::new(connection,generation,Arc::clone(&self.publication))
    }
    /// UI-owned variant: callbacks may change only visual environment state.
    /// Retired results never reach the queue; stale-environment results retain
    /// their live callbacks before their pixels are rejected.
    pub fn tick_environment(&mut self,root:&Path,connection:u64,world:&World,environment:&mut super::native_environment::State,active:bool)->bool {
        let mut changed=self.prune_static_frame(world, connection);
        changed |= self.tick(root,connection,world,environment,active);
        let scene=if active && connection!=0 && world.generation!=0 && world.view_ready() {
            world.map_id.map(|map|super::native_ambient_bridge::Scene::new(connection,world.generation,map,Arc::clone(self.ambient_source_owner.as_ref().unwrap_or(&self.publication))))
        } else {None};
        let result=(||->anyhow::Result<()> {
            if let Some(scene)=scene {
                let scene=scene?;self.ambient_bridge.bind(scene.clone(),environment)?;
                for event in std::mem::take(&mut self.pending_ambient) {self.ambient_bridge.apply(&scene,environment,&event)?;}
                if self.ambient_source_floor!=0 {self.ambient_bridge.prune_before(&scene,self.ambient_source_floor)?;}
            } else {self.ambient_bridge.retire(environment)?;self.pending_ambient.clear();}
            Ok(())
        })();
        if let Err(error)=result {self.error=Some(format!("ambient callback: {error:#}"));changed=true;}
        let expected=super::native_render_environment::Bound::capture(connection,world,environment).ok().map(|bound|bound.key());
        if self.frame.as_ref().is_some_and(|frame|frame.environment_key()!=expected) {
            if let Some(mut frame)=self.frame.take() {
                if self.deferred_finite_hurt.len().saturating_add(frame.finite_hurt_callbacks.len())<=4096 {
                    self.deferred_finite_hurt.append(&mut frame.finite_hurt_callbacks);
                } else {self.error=Some("ambient hurt callback backlog exceeded".into());}
                if self.deferred_sounds.len().saturating_add(self.deferred_native_audio.len()).saturating_add(frame.sounds.len()).saturating_add(frame.native_audio.len())<=4096 {
                    self.deferred_sounds.append(&mut frame.sounds);self.deferred_legacy_audio.append(&mut frame.legacy_audio);self.deferred_native_audio.append(&mut frame.native_audio);
                } else {for cue in &frame.legacy_audio {cue.discard();}for cue in &frame.native_audio {cue.discard();}self.error=Some("ambient audio backlog exceeded".into());}
            }
            self.rendered_revision=None;self.rendered_environment=None;changed=true;
        }
        if let Some(frame)=&mut self.frame {
            let owner=frame.ambient_owner.clone();
            frame.finite_hurt_callbacks.retain(|callback|callback.current(world,&owner));
            let mut seen:std::collections::BTreeSet<_>=frame.finite_hurt_callbacks.iter().map(|callback|callback.identity()).collect();
            for mut callback in std::mem::take(&mut self.deferred_finite_hurt) {
                if callback.current(world,&owner) && seen.insert(callback.identity()) {
                    callback.deferred=true;frame.finite_hurt_callbacks.push(callback);
                }
            }
            frame.finite_hurt_callbacks.sort_by_key(|callback|callback.order());
            for (graphics,key) in [(11736,"call_lightning_effects"),(11738,"cone_cold_effects")] {
                frame.receipt[key]["hurt_callbacks"]=serde_json::json!(frame.finite_hurt_callbacks.iter().filter(|callback|callback.graphics()==graphics).map(|callback|callback.receipt()).collect::<Vec<_>>());
                frame.receipt[key]["deferred_hurt_callbacks"]=serde_json::json!(frame.finite_hurt_callbacks.iter().filter(|callback|callback.graphics()==graphics && callback.deferred).count());
            }
        }
        changed
    }
    /// Run after controller callbacks as well as at tick entry. Preserve already
    /// admitted audio/hurt deliveries while discarding a retired raster.
    pub(crate) fn prune_static_frame(&mut self, world: &World, connection: u64) -> bool {
        if !self.frame.as_ref().is_some_and(|frame| !frame.static_current(world, connection)) { return false; }
        let mut frame = self.frame.take().expect("checked frame");
        self.rendered_revision = None;
        if self.deferred_sounds.len().saturating_add(self.deferred_native_audio.len())
            .saturating_add(frame.sounds.len()).saturating_add(frame.native_audio.len()) <= 4096 {
            self.deferred_sounds.append(&mut frame.sounds);
            self.deferred_legacy_audio.append(&mut frame.legacy_audio);
            self.deferred_native_audio.append(&mut frame.native_audio);
        } else {
            for cue in &frame.legacy_audio { cue.discard(); }
            for cue in &frame.native_audio { cue.discard(); }
            self.error = Some("static retirement audio backlog exceeded".into());
        }
        if self.deferred_finite_hurt.len().saturating_add(frame.finite_hurt_callbacks.len()) <= 4096 {
            self.deferred_finite_hurt.append(&mut frame.finite_hurt_callbacks);
        } else { self.error = Some("static retirement hurt backlog exceeded".into()); }
        true
    }
    pub(crate) fn take_static2941_ready(&mut self) -> Vec<super::native_static2941::Prepared> {
        std::mem::take(&mut self.static2941_ready)
    }
    pub fn quiescent(&self) -> bool {
        self.job.is_none()
    }
    /// Poll never joins an unfinished job. Stale results are dropped even when
    /// cancellation arrived after the decoder had completed successfully.
    pub fn tick(&mut self, root: &Path, connection: u64, world: &World,
        environment: &super::native_environment::State, active: bool) -> bool {
        self.tick_mode(root,connection,world,environment,active,false,true)
    }
    pub(crate) fn callback_draw(&mut self, root: &Path, connection: u64, world: &World,
        environment: &super::native_environment::State) -> anyhow::Result<usize> {
        self.rendered_revision=None;
        self.tick_mode(root,connection,world,environment,true,true,true);
        if let Some(error)=&self.error {anyhow::bail!("callback draw: {error}");}
        let frame=self.frame.as_ref().ok_or_else(||anyhow::anyhow!("callback frame absent"))?;
        anyhow::ensure!(frame.composition_current(world),"callback frame revision changed");
        Ok(frame.player_walk_plan.as_ref().map_or(0,|p|p.frame_count()))
    }
    fn tick_mode(&mut self, root: &Path, connection: u64, world: &World,
        environment: &super::native_environment::State, active: bool,
        synchronous: bool, request: bool) -> bool {
        let root_changed=self.source_root.as_deref()!=Some(root);
        if root_changed {
            self.source_owner=None;self.source_root=Some(root.to_path_buf());
            self.light_owner=None;self.light_remap=Default::default();self.water_history=Default::default();
        }
        if active {self.keep_source_owner=true;}
        let binding = if active && world.view_ready() {
            Some(super::native_render_environment::Bound::capture(connection, world, environment))
        } else {None};
        let (bound, binding_error) = match binding {
            Some(Ok(bound)) => (Some(bound), None),
            Some(Err(error)) => (None, Some(format!("{error:#}"))),
            None => (None, None),
        };
        let expected_environment = bound.as_ref().map(|v|v.key());
        let target = if active && binding_error.is_none() {
            world.map_id.map(|map| Key {
                connection,
                generation: world.generation,
                map,
            })
        } else {
            None
        };
        if let Some(key)=target {
            self.finite_audio_history.select_world((key.connection,key.generation,key.map));
            if self.clock_owner!=Some(key) {
                self.clock_owner=Some(key);self.clock=Some(std::time::Instant::now());
                self.finite_visual_history=None;
                self.ordinary_wire_cursor=Default::default();self.ordinary_admission=Default::default();
            }
        }
        // A pending worker may still be drawing an earlier World ring. Retain
        // both rings until it is polled, including when its context is retired.
        let mut retained:std::collections::BTreeSet<_>=world.effects.iter().map(|e|e.sequence)
            .chain(world.many_attacks.iter().map(|e|e.action_sequence))
            .chain(world.projectiles.iter().map(|e|e.action_sequence)).collect();
        if let Some(pending)=&self.job {
            if self.finite_audio_history.is_world((pending.key.connection,pending.key.generation,pending.key.map)) {
                retained.extend(pending.effect_sequences.iter().copied());
            }
        }
        retained.extend(world.status4183.sequences());
        self.finite_audio_history.retain_sequences(&retained);
        let status_retired=self.frame.as_ref().is_some_and(|frame|!frame.status4183_current());
        if status_retired {
            self.frame=None;self.rendered_revision=None;
        }
        self.deferred_native_audio.retain(|cue| {
            if cue.status_current(){true}else{cue.discard();false}
        });
        let mut changed = self.fence.select(target) || root_changed;
        let light_owner = environment.epoch.filter(|epoch|*epoch==connection && connection!=0);
        let mut water_error=None;
        if self.light_owner != light_owner {
            self.light_owner = light_owner;
            self.light_remap = Default::default();
            if let Some(owner)=light_owner {
                if let Err(error)=self.water_history.bind(owner) {water_error=Some(format!("{error:#}"));}
            }else{self.water_history.retire();}
        }
        if changed {
            // A new token fences late success/error results even if the same
            // GAME/map key or source path is selected again after retirement.
            self.publication=Arc::new(());
            if self.keep_source_owner {
                if let Some(source)=self.model.as_ref().and_then(|m|m.source_owner_for(root)){self.source_owner=Some(source);}
            }
            self.hover = None;
            self.frame = None;
            self.model = None;
            self.error = None;
            self.rendered_revision = None;
            self.rendered_environment = None;
            for cue in &self.deferred_legacy_audio {cue.discard();}
        self.deferred_legacy_audio.clear();
        self.deferred_sounds.clear();
        self.deferred_finite_hurt.clear();
        self.pending_ambient.clear();
        self.ambient_source_floor=0;
        self.ambient_source_owner=None;
        for cue in &self.deferred_native_audio {cue.discard();}
        self.deferred_native_audio.clear();
            if self.clock.is_none(){self.clock=Some(std::time::Instant::now());}
            self.last_request = None;
            if let Some(pending) = &self.job {
                pending.job.cancel();
            }
        }
        changed |= status_retired;
        if let Some(error) = binding_error {
            changed |= self.error.as_deref() != Some(error.as_str());
            self.error = Some(error);
        }
        if let Some(error)=water_error {self.error=Some(error);changed=true;}
        // Do not cancel an in-flight same-world job on every S0A/S33 update.
        // Recover its reusable model and one-shot audio after completion, but
        // never publish its old visual environment into the new one.
        if self.frame.as_ref().is_some_and(|f|f.environment_key()!=expected_environment) {
            self.frame = None;
            changed = true;
        }
        if let Some(PendingRender{key,source_root,publication,revision,environment:captured_environment,job,..}) = &mut self.job {
            if let Some(result) = if synchronous {Some(job.finish())} else {job.poll()} {
                let same_root=source_root.as_path()==root;
                let current_context=same_root&&Arc::ptr_eq(publication,&self.publication);
                if !current_context {
                    if let Ok(done)=&result {for cue in &done.frame.legacy_audio {cue.discard();}for cue in &done.frame.native_audio {cue.discard();}}
                    if result.is_ok(){self.retired_context_results=self.retired_context_results.saturating_add(1);}
                    else{self.retired_context_errors=self.retired_context_errors.saturating_add(1);}
                }
                // A retired worker cannot publish pixels/audio, but a cast it
                // already admitted must not relaunch after cancellation/root ABA.
                if self.clock_owner==Some(*key) {
                    if let Ok(done)=&result {
                        let incoming=done.model.finite_history();
                        if let Some((owner,_,history))=&mut self.finite_visual_history {
                            if *owner==*key {history.merge_disintegrate_admission(&incoming);}
                        }else{
                            self.finite_visual_history=Some((*key,source_root.clone(),incoming.for_source(false)));
                        }
                    }
                }
                // A retired scene may keep only its immutable source lease.
                // Explicit cancel disables this path; old frames stay fenced.
                if self.keep_source_owner {
                    if let Ok(done)=&result {
                        if let Some(source)=done.model.source_owner_for(root){self.source_owner=Some(source);}
                    }
                }
                // One worker at a time: retain native selector history across
                // map/generation changes within this GAME, never across owners.
                if same_root && self.light_owner == Some(key.connection) {
                    if let Ok(done) = &result {
                        self.light_remap=done.model.light_remap();
                        if let Some(history)=done.model.water_history_for(key.connection) {self.water_history=history;}
                    }
                }
                if current_context && self.fence.accepts(*key) && world.view_ready() {
                    match result {
                        Ok(mut done) => {
                            // Resource readiness is independent of a stale light/environment frame.
                            // Only the accepted root/publication/world may hand proofs to the controller.
                            self.static2941_ready = std::mem::take(&mut done.static2941_ready);
                            done.frame.receipt["static2941_resource_error"] = serde_json::json!(done.static2941_error);
                            self.finite_visual_history=Some((*key,source_root.clone(),done.model.finite_history()));
                            self.model = Some(done.model);
                            self.ambient_source_floor=self.model.as_ref().expect("accepted model").ambient_source_floor(world);
                            self.ambient_source_owner=Some(Arc::clone(&done.frame.ambient_owner));
                            let scene=super::native_ambient_bridge::Scene::new(key.connection,key.generation,key.map,Arc::clone(&done.frame.ambient_owner));
                            match scene {
                                Ok(scene) if self.pending_ambient.len().saturating_add(done.frame.native_ambient.len())<=4096=> {
                                    for event in done.frame.native_ambient.drain(..) {
                                        if event.bind(&scene) {self.pending_ambient.push(event);} else {self.error=Some("ambient scene binding mismatch".into());}
                                    }
                                }
                                Ok(_)=>self.error=Some("ambient callback backlog exceeded".into()),
                                Err(error)=>self.error=Some(format!("ambient scene: {error:#}")),
                            }
                            let scope=super::native_audio_delivery::Scope::new(key.connection,key.generation,Arc::clone(publication));
                            let bound=(||->anyhow::Result<()> {
                                anyhow::ensure!(done.frame.sounds.len()==done.frame.legacy_audio.len(),"legacy frame payload alignment");
                                anyhow::ensure!(self.deferred_sounds.len()==self.deferred_legacy_audio.len(),"deferred legacy payload alignment");
                                for cue in &done.frame.legacy_audio {cue.bind(scope.clone())?;}
                                Ok(())
                            })().and_then(|_|done.frame.native_audio.iter().try_for_each(|cue|cue.bind(scope.clone()))).and_then(|_| {
                                let report=self.finite_audio_history.filter_frame(&mut done.frame.native_audio)?;
                                done.frame.receipt["native_finite_audio_history"]=serde_json::json!(report);Ok(())
                            });
                            if let Err(e)=bound {self.error=Some(format!("native audio publication: {e:#}"));}
                            else if Some(*captured_environment) == expected_environment && done.frame.static_active(world)
                                && done.frame.status4183_current() {
                                if done.frame.environment_key() != expected_environment {
                                    self.error = Some("world frame environment binding mismatch".into());
                                } else {
                                    let count = self.deferred_sounds.len()+self.deferred_native_audio.len();
                                    let mut sounds = std::mem::take(&mut self.deferred_sounds);
                                    sounds.append(&mut done.frame.sounds);
                                    done.frame.sounds = sounds;
                                    let mut legacy=std::mem::take(&mut self.deferred_legacy_audio);
                                    legacy.append(&mut done.frame.legacy_audio);done.frame.legacy_audio=legacy;
                                    let mut native=std::mem::take(&mut self.deferred_native_audio);
                                    native.append(&mut done.frame.native_audio);done.frame.native_audio=native;
                                    done.frame.receipt["deferred_environment_audio"] = serde_json::json!(count);
                                    self.frame = Some(done.frame);
                                    self.rendered_revision = Some(*revision);
                                    self.rendered_environment = Some(*captured_environment);
                                }
                            } else {
                                self.retired_environment_results = self.retired_environment_results.saturating_add(1);
                                self.rendered_revision = None;
                                if self.deferred_sounds.len().saturating_add(self.deferred_native_audio.len()).saturating_add(done.frame.sounds.len()).saturating_add(done.frame.native_audio.len()) > 4096 {
                                    self.error = Some("world environment audio backlog exceeded".into());
                                } else {
                                    if self.deferred_finite_hurt.len().saturating_add(done.frame.finite_hurt_callbacks.len())<=4096 {
                                        self.deferred_finite_hurt.append(&mut done.frame.finite_hurt_callbacks);
                                    } else {self.error=Some("lightning callback backlog exceeded".into());}
                                    self.deferred_sounds.append(&mut done.frame.sounds);
                                    self.deferred_legacy_audio.append(&mut done.frame.legacy_audio);
                                    self.deferred_native_audio.append(&mut done.frame.native_audio);
                                }
                            }
                        }
                        Err(e) => {
                            if Some(*captured_environment) == expected_environment {
                                self.error = Some(format!("{e:#}"));
                            } else {
                                self.retired_environment_errors = self.retired_environment_errors.saturating_add(1);
                            }
                        }
                    }
                    changed = true;
                }
            }
            if job.finished() {
                self.job = None;
            }
        }
        if !world.view_ready() {
            changed |= self.frame.take().is_some();
        }
        if let Some(key) = target.filter(|_| world.view_ready() && bound.is_some()) {
            if request && self.job.is_none()
                && self.error.is_none()
                && (self.rendered_revision != Some(world.visual_revision)
                    || self.rendered_environment != expected_environment
                    || self
                        .last_request
                        .is_none_or(|t| t.elapsed() >= std::time::Duration::from_millis(50)))
            {
                let now = std::time::Instant::now();
                let elapsed_ns = self
                    .clock
                    .get_or_insert(now)
                    .elapsed()
                    .as_nanos()
                    .min(u128::from(u64::MAX)) as u64;
                self.last_request = Some(now);
                let captured_environment = expected_environment.expect("validated environment binding");
                let finite_history=self.finite_visual_history.as_ref()
                    .filter(|(owner,_,_)|*owner==key).map(|(_,source,history)|history.for_source(source.as_path()==root));
                match Job::with_model_at_labels_environment_finite_history(
                    root.to_path_buf(),
                    world.clone(),
                    self.model.take(),
                    elapsed_ns,
                    self.hover,
                    bound,
                    Some(self.light_remap.clone()),
                    Some(self.water_history.clone()),
                    finite_history,
                    Some(self.ordinary_wire_cursor.clone()),
                    Some(self.ordinary_admission.clone()),
                ) {
                    Ok(job) => {self.ordinary_wire_cursor.admit(world);self.ordinary_admission.admit(world);self.job=Some(PendingRender{key,source_root:root.to_path_buf(),publication:Arc::clone(&self.publication),revision:world.visual_revision,effect_sequences:world.effects.iter().map(|e|e.sequence).chain(world.many_attacks.iter().map(|e|e.action_sequence)).chain(world.projectiles.iter().map(|e|e.action_sequence)).collect(),environment:captured_environment,job});},
                    Err(e) => {
                        self.error = Some(format!("{e:#}"));
                        changed = true;
                    }
                }
            }
        }
        if synchronous && request && self.job.is_some() {
            changed |= self.tick_mode(root,connection,world,environment,active,true,false);
        }
        changed
    }
}
