//! World mask pass in shared body/camera coordinates, before HUD overlays.
use super::{compositor::Canvas, native_light_color::{Channels, Format, Remap},
    native_light_mask::{Mask, Source}, native_light_sources as sources,
    native_render_environment::Bound, native_world::World,
    sprite_actions::{ActionKey, SpriteActionCatalog}};
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeMap, path::Path, sync::atomic::{AtomicBool, Ordering}};

/// Original catalog loader initializes class=1, subtype=0 and stores the low
/// byte of attributes102/108. Read the logical row, without inheriting actions.
pub fn metadata(catalog: &SpriteActionCatalog, logical: u16) -> Result<[u8; 2]> {
    let row = catalog.sprites.get(&u32::from(logical)).context("light catalog row absent")?;
    ensure!(row.header_issue.is_none(), "unverified light catalog header");
    let mut output = [1, 0];
    for (index, id) in [102, 108].into_iter().enumerate() {
        let mut attrs = row.attributes.iter().filter(|a|a.key.id==id);
        if let Some(attr) = attrs.next() {
            ensure!(attr.key==ActionKey::new(id) && attrs.next().is_none(), "ambiguous light attribute {id}");
            let text = std::str::from_utf8(&attr.body_bytes).context("non-ASCII light attribute")?;
            let mut words = text.split_ascii_whitespace();
            let value: i32 = words.next().context("empty light attribute")?.parse()?;
            ensure!(words.next().is_none(), "light attribute arity");
            output[index] = value as u8;
        }
    }
    Ok(output)
}

#[derive(Default)]
pub struct Runtime {
    source: Option<Source>, mask: Option<Mask>, channels: Option<Channels>, remap: Remap,
    owner: Option<u64>,
    water_channels:Option<super::native_water_light::Channels>,
    shade:Option<super::native_water_light::Shade>,
    water_history:super::native_water_clock::History,
}
impl Runtime {
    pub fn water_history_for(&self,owner:u64)->Option<super::native_water_clock::History> {
        self.water_history.validate_owner(owner).ok().map(|_|self.water_history.clone())
    }
    pub fn restore_water_history(&mut self,owner:u64,history:super::native_water_clock::History)->Result<()> {
        history.validate_owner(owner)?;self.water_history=history;Ok(())
    }
    pub fn sample_water_camera(&mut self,owner:u64,generation:u64,motion:Option<super::native_water_clock::Motion>)->Result<()> {
        self.water_history.bind(owner)?;self.water_history.sample(owner,generation,motion)?;Ok(())
    }
    pub fn apply_water(&mut self,root:&Path,bound:&Bound,canvas:&mut Canvas,cancel:&AtomicBool)->Result<serde_json::Value> {
        let key=bound.key();
        if !key.underwater {return Ok(serde_json::json!({"active":false,"reason":"dry_world"}));}
        if key.map==303 {return Ok(serde_json::json!({"active":false,"reason":"map303_light_producer_pending","complete":false}));}
        ensure!(!cancel.load(Ordering::Acquire),"water draw cancelled");
        self.water_history.validate_owner(key.epoch)?;
        ensure!(bound.state().water_clock.owner()==Some(key.epoch),"water phase owner mismatch");
        let params=super::native_water_light::Params{phase:bound.state().water_clock.core.phase(),scroll:self.water_history.values()};
        if self.shade.is_none() {self.shade=Some(super::native_water_light::Shade::load(root,cancel)?);}
        if self.water_channels.is_none() {self.water_channels=Some(super::native_water_light::Channels::new(Format::Rgb555));}
        let result=canvas.apply_native_water(self.water_channels.as_ref().unwrap(),self.mask.as_ref().context("water source mask absent")?,
            self.remap.row(),self.shade.as_ref().unwrap(),params)?;
        ensure!(!cancel.load(Ordering::Acquire),"water draw cancelled");
        Ok(serde_json::json!({"active":true,"pixels":result,"phase":params.phase,"scroll":params.scroll,
            "counter":bound.state().water_clock.core.counter(),"phase_is_animation_sample":true,"pass_before_world_labels":true}))
    }
    pub fn remap(&self) -> Remap { self.remap.clone() }
    pub fn remap_for(&self, owner:u64) -> Option<Remap> {
        (self.owner==Some(owner) && owner!=0).then(||self.remap())
    }
    pub fn restore_remap(&mut self, owner: u64, remap: Remap) -> Result<()> {
        ensure!(owner!=0,"retired light cache owner");
        self.owner=Some(owner); self.remap = remap; Ok(())
    }
    /// Allows a retained original resource without opening an archive again.
    pub fn from_source(source: Source) -> Self { Self {source:Some(source), ..Default::default()} }
    pub fn draw(&mut self, root: &Path, catalog: &SpriteActionCatalog, world: &World,
        bound: &Bound, centers: &BTreeMap<u32,[i32;2]>, canvas: &mut Canvas,
        cancel: &AtomicBool) -> Result<serde_json::Value> {
        let cells=world.actors.iter().map(|(id,a)|(*id,[a.x,a.y])).collect();
        self.draw_with_sources(root,catalog,world,bound,centers,&cells,canvas,cancel)
    }
    /// Moving sources retain their source cell until the shared body pose is
    /// terminal. Pixel centers still use its interpolated display position.
    pub fn draw_with_sources(&mut self, root:&Path,catalog:&SpriteActionCatalog,world:&World,
        bound:&Bound,centers:&BTreeMap<u32,[i32;2]>,cells:&BTreeMap<u32,[u16;2]>,
        canvas:&mut Canvas,cancel:&AtomicBool)->Result<serde_json::Value> {
        self.draw_with_actions(root,catalog,world,bound,centers,cells,&BTreeMap::new(),canvas,cancel)
    }
    /// Water classification uses the action currently displayed by the body
    /// animator, not the actor's retained action history or generic dead bit.
    pub fn draw_with_actions(&mut self,root:&Path,catalog:&SpriteActionCatalog,world:&World,
        bound:&Bound,centers:&BTreeMap<u32,[i32;2]>,cells:&BTreeMap<u32,[u16;2]>,
        actions:&BTreeMap<u32,u8>,canvas:&mut Canvas,cancel:&AtomicBool)->Result<serde_json::Value> {
        bound.validate_world(world)?;
        ensure!(!cancel.load(Ordering::Acquire), "light draw cancelled");
        ensure!(canvas.width()==800 && canvas.height()==600, "world light viewport dimensions");
        let key = bound.key();
        if self.owner!=Some(key.epoch) {self.owner=Some(key.epoch);self.remap=Default::default();}
        let water_sources=key.underwater && key.blind==0;
        if water_sources {ensure!(world.relations.owner()==Some(key.epoch),"water relation GAME owner mismatch");}
        if !key.underwater && key.blind==0 && key.mask_argument>=15 {
            return Ok(serde_json::json!({"active":false,"reason":"native_full_bright_mask_bypass","cached_level":self.remap.level()}));
        }
        // The original map303 kernel also consults an external byte gate. Do
        // not fabricate that plane or apply an unconditional darkening pass.
        if key.map==303 {
            return Ok(serde_json::json!({"active":false,"reason":"map303_original_byte_gate_pending","complete":false}));
        }
        let own_id = world.status.map(|s|s.object_id);
        let own_position = if world.observer_mode {
            world.observer_camera.context("light observer camera absent")?
        } else {
            let actor = world.own_actor().context("light local actor absent")?;
            *cells.get(&actor.id).context("light local source cell absent")?
        };
        let native = |p:[u16;2]| [2*i32::from(p[0])-0x8000,i32::from(p[1])];
        let mut ordered: Vec<_> = world.actors.values().collect();
        ordered.sort_by_key(|actor|actor.id as i32);
        let ids: Vec<_> = ordered.iter().map(|a|a.id).collect();
        let own_index = if world.observer_mode {None} else {ids.iter().position(|id|Some(*id)==own_id)};
        let mut actors = Vec::with_capacity(ordered.len());
        let mut projected = Vec::with_capacity(ordered.len());
        let mut classifications=BTreeMap::new();
        let own_name=world.own_actor().map(|a|a.relations.name.as_slice());
        let native_own = native(own_position);
        let policy=sources::Policy {ambient:u32::from(key.mask_argument),underwater:key.underwater,
            blind:key.blind,own_index,own_position:native_own};
        let mut selected_range=0u32;
        for (index,actor) in ordered.into_iter().enumerate() {
            let center = *centers.get(&actor.id).context("light shared body projection absent")?;
            let cell=*cells.get(&actor.id).context("light actor source cell absent")?;
            ensure!(cell==[actor.x,actor.y] || actor.motion_from==Some(cell),"light source cell outside actor motion");
            let position = native(cell);
            let inside = (i64::from(position[0])-i64::from(native_own[0])).abs()<=35
                && (i64::from(position[1])-i64::from(native_own[1])).abs()<=15;
            let eligible=inside && selected_range<700;
            let [base_class,subtype] = if eligible && (water_sources || (key.blind==2 && own_index!=Some(index))) {
                metadata(catalog,actor.graphics)?
            } else {[1,0]};
            let mut source=sources::Actor {position,raw_light:actor.light,
                base_class,action:0,walk_action:actor.walk_action,direction:actor.heading,source_frame:0,frame_count:0,
                subtype,flag_1f:false,flag_50:actor.draw_state.native_light_dead,
                mode_23:actor.unknown&15,master:actor.relations.master_kind(own_name)};
            if water_sources && eligible {
                source.action=*actions.get(&actor.id).context("water shared body action absent")?;
                // Original type11 classification returns before consulting1F.
                if base_class!=11 {
                    source.flag_1f=actor.relations.flag_1f.resolve(|graphics|Ok(metadata(catalog,graphics)?[0]))?;
                }
                classifications.insert(actor.id,sources::classify(&source));
            }
            if eligible {
                if let Some(range)=sources::source_range(policy,&source,index) {selected_range+=u32::from(range);}
            }
            actors.push(source);
            projected.push(center);
        }
        if self.source.is_none() { self.source=Some(Source::load(root,cancel)?); }
        if self.mask.is_none() { self.mask=Some(Mask::for_pixels(800,600)?); }
        if self.channels.is_none() {self.channels=Some(Channels::new(Format::Rgb555));}
        let composition = sources::compose(self.source.as_ref().unwrap(), self.mask.as_mut().unwrap(),
            &mut self.remap,policy,&actors,&projected)?;
        ensure!(!cancel.load(Ordering::Acquire), "light draw cancelled");
        // The producer runs in unblinded water, but the ordinary RGB mask
        // kernel is bypassed. Its mask is consumed by the separate water pass.
        let pixels = if composition.active && !water_sources {
            Some(canvas.apply_native_light_mask(self.channels.as_ref().unwrap(),self.mask.as_ref().unwrap(),self.remap.row(),None)?)
        } else {None};
        Ok(serde_json::json!({"active":composition.active && !water_sources,"composition":composition,"pixels":pixels,
            "reason":if water_sources {Some("native_unblinded_water_mask_bypass")} else {None},
            "source_classification_complete":true,"water_source_classes":classifications,
            "actor_ids_signed_order":ids,"shared_interpolated_body_projection":true,"extra_walking_offset":false,
            "source_cell_policy":"caller supplies current or motion-source cells; Model uses shared pose phase",
            "cached_level":self.remap.level(),"remap_row":self.remap.row(),"pass_before_world_labels":true,
            "special_global_filters_complete":false,"water_extra_effect_complete":false}))
    }
}
