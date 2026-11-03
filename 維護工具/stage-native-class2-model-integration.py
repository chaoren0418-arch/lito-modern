"""Prepare four reviewable core overrides without mutating frozen product inputs."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];SOURCE=ROOT/'LiTo381-Modern/src/modern_lito'
OUT=ROOT/'維護工具/candidates/native-class2-presentation';OUT.mkdir(exist_ok=True)
pins=json.loads((ROOT/'驗證紀錄/current/native77-lawful/validation-20261017-195458-430865/compiled-inputs.json').read_text(encoding='utf-8-sig'))
def source(name):
    p=SOURCE/name;assert hashlib.sha256(p.read_bytes()).hexdigest().upper()==pins[p.relative_to(ROOT).as_posix()]
    return p.read_text(encoding='utf-8-sig')
def replace(s,old,new):assert old in s,old;return s.replace(old,new)
def save(name,s):(OUT/name).write_text(s,encoding='utf-8',newline='\n')

s=source('native_environment.rs')
s=replace(s,'    fn owner(&self, epoch: u64) -> Result<()> {',"""    /// Accepted World generation boundary, including a destination on the same
    /// numeric map. Old visual leases cannot suppress the destination's current
    /// server calendar/weather or restore an earlier scene's saved brightness.
    pub fn ambient_scene_changed(&mut self,epoch:u64,map:u16)->Result<bool> {
        self.owner(epoch)?;ensure!(self.map==Some(map),"ambient destination map not accepted");
        let mut next=self.clone();next.ambient_override_counter=0;
        next.daylight=next.calendar.map(|calendar|daylight(calendar.day_of_year,calendar.minute_of_day(),next.weather)).transpose()?;
        next.ambient_level=if map_policy_flag(map){Some(1)}else{next.daylight.map(|value|value.level)};
        if self.ambient_override_counter==0 && self.daylight==next.daylight && self.ambient_level==next.ambient_level {return Ok(false)}
        next.bump()?;*self=next;Ok(true)
    }
    fn owner(&self, epoch: u64) -> Result<()> {""")
save('native_environment.rs',s)

s=source('native_world_scene.rs')
s=replace(s,'pub struct Frame {','''/// Already dispatched model callback; publication never replays the animation.
#[derive(Clone,Debug)]
pub(crate) struct HurtCallback {
    target:u32,incarnation:u64,sequence:u64,projectile:u32,accepted:bool,
    generation:u64,owner:std::sync::Arc<()>,dispatch_ns:u64,pub(crate) deferred:bool,
}
impl HurtCallback {
    pub(crate) fn capture(world:&World,actor:&super::native_world::Actor,owner:std::sync::Arc<()>,sequence:u64,projectile:u32,accepted:bool,dispatch_ns:u64)->Self {
        Self{target:actor.id,incarnation:actor.incarnation,sequence,projectile,accepted,generation:world.generation,owner,dispatch_ns,deferred:false}
    }
    pub(crate) fn current(&self,world:&World,owner:&std::sync::Arc<()>)->bool {
        self.generation==world.generation && std::sync::Arc::ptr_eq(&self.owner,owner)
            && world.actors.get(&self.target).is_some_and(|actor|actor.incarnation==self.incarnation && !actor.dead)
    }
    pub(crate) fn identity(&self)->(u32,u64,u64,u32) {(self.target,self.incarnation,self.sequence,self.projectile)}
    pub(crate) fn order(&self)->(u64,u64,u32) {(self.dispatch_ns,self.sequence,self.target)}
    pub(crate) fn receipt(&self)->serde_json::Value {
        serde_json::json!({"target":self.target,"incarnation":self.incarnation,"sequence":self.sequence,"projectile":self.projectile,"accepted":self.accepted,"dispatch_ns":self.dispatch_ns,"deferred_due_to_environment":self.deferred})
    }
}
pub struct Frame {''')
s=replace(s,'    pub native_audio: Vec<super::native_audio_delivery::PreparedCue>,','    pub native_audio: Vec<super::native_audio_delivery::PreparedCue>,\n    pub native_ambient: Vec<super::native_ambient_bridge::Prepared>,\n    pub(crate) ambient_owner:std::sync::Arc<()>,')
s=replace(s,'    pub(crate) ambient_owner:std::sync::Arc<()>,','    pub(crate) ambient_owner:std::sync::Arc<()>,\n    pub(crate) lightning_hurt_callbacks:Vec<HurtCallback>,')
s=replace(s,'disintegrate:super::native_disintegrate::History,restore_other_effects:bool,','disintegrate:super::native_disintegrate::History,call_lightning:super::native_call_lightning::History,restore_other_effects:bool,')
s=replace(s,'self.disintegrate.merge(&other.disintegrate);','self.disintegrate.merge(&other.disintegrate);self.call_lightning.merge(&other.call_lightning);')
s=replace(s,'    disintegrate_effects:super::native_disintegrate::Effects,','    disintegrate_effects:super::native_disintegrate::Effects,\n    call_lightning_effects:super::native_call_lightning::Effects,')
s=replace(s,'disintegrate:self.disintegrate_effects.history(),restore_other_effects:true','disintegrate:self.disintegrate_effects.history(),call_lightning:self.call_lightning_effects.history(),restore_other_effects:true')
s=replace(s,'        self.disintegrate_effects.restore_history(history.disintegrate);','        self.disintegrate_effects.restore_history(history.disintegrate);\n        self.call_lightning_effects.restore_history(history.call_lightning);')
s=replace(s,'            disintegrate_effects:Default::default(),','            disintegrate_effects:Default::default(),\n            call_lightning_effects:Default::default(),')
s=replace(s,'        self.disintegrate_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;','        self.disintegrate_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;\n        self.call_lightning_effects.prepare(&self.root,self.effect_source_owner.clone(),world,cancel)?;')
anchor='        let disintegrate_events=self.disintegrate_effects.entered_events();'
s=replace(s,anchor,anchor+'''
        let lightning_launches:Vec<_>=poses.values().filter_map(|p|p.as_ref().ok()).flat_map(|p|&p.entered_events)
            .filter_map(|event|super::native_call_lightning::Launch::capture(&self.call_lightning_effects,&self.effect_source_owner,world,event)).collect();
        let lightning_impacts=self.call_lightning_effects.update(&self.effect_source_owner,world,lightning_launches.iter(),elapsed_ns)?;
        let lightning_keys:std::collections::BTreeSet<_>=lightning_impacts.iter().map(|i|(i.target,i.sequence,i.projectile)).collect();
        impacts.extend(lightning_impacts);
        let lightning_events=self.call_lightning_effects.entered_events();
        let native_ambient=self.call_lightning_effects.ambient_events().to_vec();
        let mut lightning_impact_receipts=Vec::new();
        let mut lightning_hurt_callbacks=Vec::new();''')
s=replace(s,'            if disintegrate_keys.contains(&(impact.target,impact.sequence,impact.projectile)){disintegrate_impact_receipts.push(row);}',
    '            if lightning_keys.contains(&(impact.target,impact.sequence,impact.projectile)){lightning_hurt_callbacks.push(HurtCallback::capture(world,actor,self.effect_source_owner.clone(),impact.sequence,impact.projectile,accepted,elapsed_ns));lightning_impact_receipts.push(row);}\n            else if disintegrate_keys.contains(&(impact.target,impact.sequence,impact.projectile)){disintegrate_impact_receipts.push(row);}')
anchor='        disintegrate_receipt["effect_submissions"]=serde_json::json!(disintegrate_draws.len()*2);'
s=replace(s,anchor,anchor+'''
        let mut lightning_receipt=self.call_lightning_effects.snapshot();
        lightning_receipt["hurt_callbacks"]=serde_json::json!(lightning_impact_receipts);
        let lightning_draws=self.call_lightning_effects.draws(&self.effect_source_owner)?;
        for draw in &lightning_draws {
            draw.submit_to(&mut effect_target,super::world_map::ViewProjection{anchor_cell:anchor,anchor_pixel:super::world_map::PixelPoint{x:24-camera.x,y:768-camera.y}})?;
        }
        lightning_receipt["effect_submissions"]=serde_json::json!(lightning_draws.len());''')
s=replace(s,'                .chain(disintegrate_events.iter())','                .chain(disintegrate_events.iter())\n                .chain(lightning_events.iter())')
s=replace(s,'        receipt["disintegrate_effects"]=disintegrate_receipt;','        receipt["disintegrate_effects"]=disintegrate_receipt;\n        receipt["call_lightning_effects"]=lightning_receipt;')
s=replace(s,'            native_audio,\n            navigation:','            native_audio,\n            native_ambient,\n            ambient_owner:self.effect_source_owner.clone(),\n            lightning_hurt_callbacks,\n            navigation:')
s=replace(s,'    pub fn light_remap(&self)', '''    pub(crate) fn ambient_source_floor(&self,world:&World)->u64 {
        world.projectiles.iter().map(|cue|cue.action_sequence).chain(self.call_lightning_effects.oldest_active_sequence()).min().unwrap_or_else(||world.visual_revision.saturating_add(1)).max(1)
    }
    pub fn light_remap(&self)''')
save('native_world_scene.rs',s)

s=source('native_world_view.rs')
s=replace(s,'    finite_audio_history:super::native_audio_delivery::FiniteDeliveryHistory,','    finite_audio_history:super::native_audio_delivery::FiniteDeliveryHistory,\n    ambient_bridge:super::native_ambient_bridge::Bridge,\n    pending_ambient:Vec<super::native_ambient_bridge::Prepared>,\n    ambient_source_floor:u64,\n    ambient_source_owner:Option<Arc<()>>,')
s=replace(s,'    ambient_source_owner:Option<Arc<()>>,','    ambient_source_owner:Option<Arc<()>>,\n    deferred_lightning_hurt:Vec<super::native_world_scene::HurtCallback>,')
s=replace(s,'        self.deferred_sounds.clear();','        self.deferred_sounds.clear();\n        self.deferred_lightning_hurt.clear();\n        self.pending_ambient.clear();\n        self.ambient_source_floor=0;\n        self.ambient_source_owner=None;')
s=replace(s,'                                    self.deferred_sounds.append(&mut done.frame.sounds);','''                                    if self.deferred_lightning_hurt.len().saturating_add(done.frame.lightning_hurt_callbacks.len())<=4096 {
                                        self.deferred_lightning_hurt.append(&mut done.frame.lightning_hurt_callbacks);
                                    } else {self.error=Some("lightning callback backlog exceeded".into());}
                                    self.deferred_sounds.append(&mut done.frame.sounds);''')
anchor='                            self.model = Some(done.model);'
s=replace(s,anchor,anchor+'''
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
                            }''')
anchor='    pub fn quiescent(&self) -> bool {'
s=replace(s,anchor,'''    /// UI-owned variant: callbacks may change only visual environment state.
    /// Retired results never reach the queue; stale-environment results retain
    /// their live callbacks before their pixels are rejected.
    pub fn tick_environment(&mut self,root:&Path,connection:u64,world:&World,environment:&mut super::native_environment::State,active:bool)->bool {
        let mut changed=self.tick(root,connection,world,environment,active);
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
                if self.deferred_lightning_hurt.len().saturating_add(frame.lightning_hurt_callbacks.len())<=4096 {
                    self.deferred_lightning_hurt.append(&mut frame.lightning_hurt_callbacks);
                } else {self.error=Some("ambient hurt callback backlog exceeded".into());}
                if self.deferred_sounds.len().saturating_add(self.deferred_native_audio.len()).saturating_add(frame.sounds.len()).saturating_add(frame.native_audio.len())<=4096 {
                    self.deferred_sounds.append(&mut frame.sounds);self.deferred_native_audio.append(&mut frame.native_audio);
                } else {for cue in &frame.native_audio {cue.discard();}self.error=Some("ambient audio backlog exceeded".into());}
            }
            self.rendered_revision=None;self.rendered_environment=None;changed=true;
        }
        if let Some(frame)=&mut self.frame {
            let owner=frame.ambient_owner.clone();
            frame.lightning_hurt_callbacks.retain(|callback|callback.current(world,&owner));
            let mut seen:std::collections::BTreeSet<_>=frame.lightning_hurt_callbacks.iter().map(|callback|callback.identity()).collect();
            for mut callback in std::mem::take(&mut self.deferred_lightning_hurt) {
                if callback.current(world,&owner) && seen.insert(callback.identity()) {
                    callback.deferred=true;frame.lightning_hurt_callbacks.push(callback);
                }
            }
            frame.lightning_hurt_callbacks.sort_by_key(|callback|callback.order());
            frame.receipt["call_lightning_effects"]["hurt_callbacks"]=serde_json::json!(frame.lightning_hurt_callbacks.iter().map(|callback|callback.receipt()).collect::<Vec<_>>());
            frame.receipt["call_lightning_effects"]["deferred_hurt_callbacks"]=serde_json::json!(frame.lightning_hurt_callbacks.iter().filter(|callback|callback.deferred).count());
        }
        changed
    }
'''+anchor)
save('native_world_view.rs',s)

s=source('native_login.rs')
anchor='    pub fn snapshot(&self) -> &Snapshot {'
s=replace(s,anchor,'''    /// Disjoint presentation inputs; gameplay authority remains read-only.
    pub(crate) fn presentation_inputs(&mut self)->(&super::native_world::World,&mut super::native_environment::State) {
        (&self.state.world,&mut self.state.environment)
    }
'''+anchor)
s=replace(s,'        if self.state.world.accept(body)? {','        let previous_world_generation=self.state.world.generation;\n        if self.state.world.accept(body)? {')
s=replace(s,'                    (body.first() == Some(&0x28)).then_some(self.state.world.underwater))?;',"""                    (body.first() == Some(&0x28)).then_some(self.state.world.underwater))?;
                if self.state.world.generation!=previous_world_generation {
                    self.state.environment.ambient_scene_changed(self.transport.snapshot().epoch,self.state.world.map_id.context("ambient destination map")?)?;
                }""")
save('native_login.rs',s)
s=source('native_login_window.rs')
old='''        if self.world_presentation.tick(
            &self.resource_root,
            self.controller.transport().snapshot().epoch,
            &self.controller.snapshot().world,
            &self.controller.snapshot().environment,
            active && !self.closing,
        ) {'''
new='''        let epoch=self.controller.transport().snapshot().epoch;
        let (world,environment)=self.controller.presentation_inputs();
        if self.world_presentation.tick_environment(&self.resource_root,epoch,world,environment,active && !self.closing) {'''
s=replace(s,old,new);save('native_login_window.rs',s)
print(json.dumps(dict(status='staged_uncompiled_core_overrides',files=['native_environment.rs','native_world_scene.rs','native_world_view.rs','native_login.rs','native_login_window.rs'],product_modified=False),ensure_ascii=False))
