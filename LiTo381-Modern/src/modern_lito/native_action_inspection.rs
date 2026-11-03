//! Native LTtext/LTsprite source-unit CPU slice only. Explicit action selection,
//! explicit original catalog integer milliseconds; no inferred actor binding
//! or scheduler phase/loop/LOADED grant.
use super::{compositor::Canvas, native_resources::{self as r, BytePin, NativeAssetPlan},
    sprite::{self, DecodedSprite, SpriteLimits}, sprite_actions::{ActionKey, ActionLimits, Animation, SpriteActionCatalog},
    world_depth::{NativeDrawCell, NativeDrawKey}};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::{AtomicBool, Ordering}};

/// Supplementary LOCAL provenance. LTtext is not yet part of SourceAttachment.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CatalogPins { pub index: BytePin, pub pak: BytePin, pub decoded_catalog: BytePin }
pub struct NativeActionSnapshot {
    plan: NativeAssetPlan,
    pins: CatalogPins,
    sprite_id: u32,
    action_id: u32,
    animation: Animation,
    sprite: DecodedSprite,
    sprite_sha256: [u8;32],
    firepot_creation_facts: bool,
}
pub struct NativeActionSlice {
    plan: NativeAssetPlan,
    catalog: CatalogPins,
    sprite_id: u32,
    action_id: u32,
    tick: u64,
    subframe: u32,
    canvas: Canvas,
    depths: Vec<(NativeDrawKey, sprite::SpriteBlockReference)>,
}
fn active(cancel: &AtomicBool) -> Result<()> { ensure!(!cancel.load(Ordering::Acquire), "Native action cancelled"); Ok(()) }
impl NativeActionSnapshot {
    /// Only the two independently identified FirePot rows are qualified here.
    /// Action3 is explicit inspection, not an asserted Layer8 default action.
    pub fn load_firepot(plan: &NativeAssetPlan, text_root: &Path, sprite_root: &Path,
        pins: CatalogPins, sprite_id: u32, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        ensure!(matches!(sprite_id,38|123), "Native action route outside audited slice");
        ensure!(plan.namespace == r::ResourceNamespace::NativeLitoV1, "Native namespace required");
        let text = r::read_native_inspection_member(text_root,true,pins.index,pins.pak,"ltlist.spr",cancel)?;
        ensure!(text.len() as u64 == pins.decoded_catalog.bytes && <[u8;32]>::from(Sha256::digest(&text)) == pins.decoded_catalog.sha256,
            "Native decoded catalog provenance drift");
        // The Native header is one integer. Parse a bounded original physical
        // row verbatim; do not synthesize a Shared three-integer file header.
        ensure!(text.starts_with(b"285\r\n") || text.starts_with(b"285\n"), "Native header mismatch");
        let prefix=format!("#{sprite_id}");
        let rows: Vec<_> = text.split(|b|*b==b'\n').filter(|row|row.starts_with(prefix.as_bytes())
            && row.get(prefix.len()).is_some_and(|b|b.is_ascii_whitespace())).collect();
        ensure!(rows.len()==1 && rows[0].len()<=4096, "Native action row absent/ambiguous/overbound");
        let c=SpriteActionCatalog::parse(rows[0],ActionLimits {max_bytes:4096,max_line_bytes:4096,
            max_sprites:1,max_actions:8,max_frames:128,max_frames_per_action:128,max_events_per_frame:8,max_reference_depth:1})?;
        let row=c.sprites.get(&sprite_id).context("Native action row missing")?;
        ensure!(row.graphics_id==sprite_id, "Native alias graphics unsupported");
        let a=c.resolve_action(sprite_id,ActionKey::new(3))?;
        ensure!(a.definition_sprite_id==sprite_id && a.animation.frames.len()==11
            && a.animation.frames.iter().enumerate().all(|(i,f)|f.image_id==0 && f.subframe==i as u32
                && f.duration_ticks==2 && f.events.is_empty()), "Native FirePot frame contract drift");
        let animation=a.animation.clone();
        let scalar=|id,value:&[u8]|row.attributes.iter().filter(|v|v.key==ActionKey::new(id)).count()==1&&row.attributes.iter().any(|v|v.key==ActionKey::new(id)&&v.body_bytes.iter().copied().filter(|v|!v.is_ascii_whitespace()).collect::<Vec<_>>()==value);
        let firepot_creation_facts=!row.actions.contains_key(&ActionKey::new(0))&&scalar(102,b"0")&&scalar(104,b"8")&&!row.attributes.iter().any(|v|matches!(v.key.id,101|105));
        let idx=plan.archives.iter().find(|p|p.role==r::ArchiveRole::SpriteIndex).context("Native sprite IDX pin missing")?;
        let pak=plan.archives.iter().find(|p|p.role==r::ArchiveRole::SpritePak).context("Native sprite PAK pin missing")?;
        let bytes=r::read_native_inspection_member(sprite_root,false,idx.pin,pak.pin,&format!("{sprite_id}-0.spr"),cancel)?;
        let sprite_sha256=Sha256::digest(&bytes).into();
        active(cancel)?;
        let sprite=sprite::decode_world_spr_with_limits(&bytes,SpriteLimits {max_input_bytes:1024*1024,max_frames:32,
            max_blocks:4096,max_block_references:32768,max_frame_dimension:1024,max_total_pixels:4*1024*1024})?;
        ensure!(sprite.frames.len()==11 && sprite.frames.iter().all(|f|f.fragments.is_some()), "Native SPR frame contract mismatch");
        active(cancel)?;
        Ok(Self{plan:plan.clone(),pins,sprite_id,action_id:3,animation,sprite,sprite_sha256,firepot_creation_facts})
    }
    pub fn sprite_sha256(&self)->[u8;32] {self.sprite_sha256}
    pub(crate) fn scene_firepot_creation_facts(&self)->bool{self.firepot_creation_facts}
    pub(crate) fn scene_catalog_pins(&self)->CatalogPins {self.pins}
    pub(crate) fn scene_plan(&self)->&NativeAssetPlan {&self.plan}
    pub(crate) fn scene_sprite_id(&self)->u32 {self.sprite_id}
    pub(crate) fn scene_frame(&self,frame:usize)->Result<&sprite::SpriteFrame> {
        ensure!(frame<self.animation.frames.len(),"Native scene effect frame outside single interval");
        Ok(&self.sprite.frames[self.animation.frames[frame].subframe as usize])
    }
    pub fn durations(&self)->Vec<u32> {self.animation.frames.iter().map(|f|f.duration_ticks).collect()}
    pub fn duration_ms(&self)->Result<Vec<u32>> {
        self.animation.frames.iter().map(|f|super::native_actor_inspection::catalog_frame_ms(f.duration_ticks,self.animation.frames_per_second)).collect()
    }
    /// Explicit event frame selection; elapsed wall time is never used as a
    /// catalog interval. Only an independently qualified scheduler may select.
    pub fn render_frame_index(&self,frame:usize,cancel:&AtomicBool)->Result<NativeActionSlice> {
        ensure!(frame<self.animation.frames.len(),"Native event frame outside source action");
        self.render_tick(self.animation.frames[..frame].iter().map(|f|u64::from(f.duration_ticks)).sum(),cancel)
    }
    /// Native's own 43DE10 establishes FPS24 and per-frame integer milliseconds.
    /// Explicit phase0, unmodified catalog and one interval; no loop inference.
    pub fn render_catalog_millis(&self,elapsed_ms:u64,cancel:&AtomicBool)->Result<NativeActionSlice> {
        ensure!(self.animation.frames_per_second==24,"Native FirePot FPS outside audited route");
        let durations=self.animation.frames.iter().map(|f|super::native_actor_inspection::catalog_frame_ms(f.duration_ticks,24)).collect::<Result<Vec<_>>>()?;
        let frame=super::native_actor_inspection::relative_frame(&durations,elapsed_ms)?;
        let tick=self.animation.frames[..frame].iter().map(|f|u64::from(f.duration_ticks)).sum();
        self.render_tick(tick,cancel)
    }
    /// Exact catalog tick domain, one non-wrapping action interval [0,22).
    /// No implicit FPS24 default from the shared parser crosses this boundary.
    pub fn render_tick(&self, tick:u64, cancel:&AtomicBool)->Result<NativeActionSlice> {
        active(cancel)?;
        ensure!(tick<self.animation.total_ticks(), "Native looping/start phase unqualified");
        let mut end=0;
        let frame=self.animation.frames.iter().find(|f|{end+=u64::from(f.duration_ticks);tick<end}).context("Native tick outside action")?;
        let decoded=&self.sprite.frames[frame.subframe as usize];
        let fragments=decoded.fragments.as_ref().context("Native source fragments missing")?;
        let anchor=NativeDrawCell{half_x:0,y:0};
        let mut depths=Vec::new();
        for (i,reference) in fragments.references.iter().enumerate(){
            active(cancel)?;
            let cell=anchor.fragment(*reference).context("Native depth overflow")?;
            depths.push((cell.overlay_key(reference.depth,i as u64),*reference));
        }
        depths.sort_by_key(|d|d.0);
        let mut canvas=Canvas::new(512,256)?;
        canvas.clear([24,24,32]);
        let mut rgba=[0;2304];
        for (_,reference) in &depths {
            active(cancel)?;
            fragments.atlas[reference.block_id as usize].write_rgba(&mut rgba);
            let (x,y)=anchor.fragment_pixel_offset(*reference).context("Native projection overflow")?;
            canvas.blit_rgba(&rgba,24,24,256+i32::try_from(x)?,160+i32::try_from(y)?)?;
        }
        active(cancel)?;
        Ok(NativeActionSlice{plan:self.plan.clone(),catalog:self.pins,sprite_id:self.sprite_id,
            action_id:self.action_id,tick,subframe:frame.subframe,canvas,depths})
    }
}
impl NativeActionSlice {
    pub fn verified_plan(&self)->&NativeAssetPlan {&self.plan}
    pub fn canvas(&self)->&Canvas {&self.canvas}
    pub fn sample(&self)->(u32,u32,u64,u32) {(self.sprite_id,self.action_id,self.tick,self.subframe)}
    pub fn depth_references(&self)->&[(NativeDrawKey,sprite::SpriteBlockReference)] {&self.depths}
    pub fn catalog_pins(&self)->CatalogPins {self.catalog}
}
