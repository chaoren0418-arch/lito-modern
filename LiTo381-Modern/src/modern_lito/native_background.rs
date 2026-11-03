//! Map303 background IMG sources and native-coordinate composition.
//! This component does not infer terrain sources, modern camera conversion,
//! animation callback history or first-pass object gate provenance.
use super::{
    native_background_geometry::{self as geometry, Input},
    native_background_pixels::{self as pixels, Format, OwnedImage, Target},
    native_resources::NativeSurfaceArchive,
    native_skill_graphics::{layers::DenseSource, Format as CacheFormat},
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
};
use anyhow::{ensure, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::{AtomicBool, Ordering}};

pub struct Bank {
    base: OwnedImage,
    overlay: OwnedImage,
    pub sources: Value,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Receipt {
    pub calls: usize,
    pub written_pixels: usize,
}
fn active(cancel:&AtomicBool)->Result<()> {
    ensure!(!cancel.load(Ordering::Acquire),"background cancelled");Ok(())
}
fn contains(base:&SharedArchive,name:&str)->Result<bool> {
    match base.entry(name) {Ok(_)=>Ok(true),Err(ResourceError::MissingEntry(_))=>Ok(false),Err(e)=>Err(e.into())}
}
impl Bank {
    /// Fixed currently qualified originals. A native index miss is required
    /// before BASE Sprite; missing/corrupt archives and decode failures are errors.
    /// All four localized names are checked so no configured locale is guessed.
    pub fn load_map303(root:&Path,cancel:&AtomicBool)->Result<Self> {
        active(cancel)?;
        let (mut surface,index_pin,pak_pin)=NativeSurfaceArchive::observe(&root.join("LiTo"),cancel)?;
        let names:Vec<_>=[800,803].into_iter().flat_map(|id|["e","c","h","j",""].map(move |s|format!("{id}{s}.img"))).collect();
        let selected:Vec<_>=names.iter().map(String::as_str).collect();
        let mut base=SharedArchive::open_for_names(root,SharedArchiveKind::Sprite,&selected,cancel)?;
        for name in names.iter().filter(|n|n.len()>7) {
            active(cancel)?;
            ensure!(!surface.contains(name)? && !contains(&base,name)?,"localized background override requires qualification: {name}");
        }
        let mut sources=Vec::new();
        let mut load=|id:u16,side:usize,key:u16,sha:&str|->Result<OwnedImage> {
            active(cancel)?;
            let name=format!("{id}.img");let length=8+2*side*side;
            let native=surface.contains(&name)?;
            let raw=if native {surface.read(&name,length,cancel)?} else {
                ensure!(base.entry(&name)?.size as usize==length,"background BASE member length");
                base.read_entry(&name)?
            };
            ensure!(raw.len()==length && format!("{:X}",Sha256::digest(&raw))==sha,"background member differs from qualified IMG: {name}");
            let cache=DenseSource::decode_raw555(&raw)?.cache(CacheFormat::Rgb555);
            ensure!(cache.dimensions()==[side,side] && cache.flag()==0 && cache.key()==key,"background IMG header differs");
            // The background kernel reads every source word, including key and
            // encoded black. Do not convert through a color-key RGBA image.
            let image=OwnedImage::new(side,side,cache.words().to_vec(),Format::Rgb555)?;
            sources.push(json!({"member":name,"archive":if native {"LiTo/LTsurf"}else{"BASE/Sprite"},
                "native_index_miss":!native,"bytes":raw.len(),"sha256":sha,"dimensions":[side,side],"flag":0,"color_key":key}));
            active(cancel)?;Ok(image)
        };
        let image=load(800,384,30654,"D3344959D3DB9F7D9FF24E689DBFD5E67767A4147D99B24E2CCE2EB5B99078A1")?;
        let overlay=load(803,768,29596,"B35213FD927F41222D8B319AD9960D5C60E5583C998C0B10577AB99B4E2B2F15")?;
        active(cancel)?;
        let pin=|p:super::native_resources::BytePin|json!({"bytes":p.bytes,"sha256":p.sha256.iter().map(|b|format!("{b:02X}")).collect::<String>()});
        Ok(Self {base:image,overlay,sources:json!({"surface_index":pin(index_pin),"surface_pak":pin(pak_pin),
            "members":sources,"localized_overrides_absent":true,"whole_BASE_pak_SHA_claimed":false})})
    }

    /// Original 744x552 plane only. Caller supplies the complete first-pass gate
    /// and original half-cell/camera fields; no world/map source fallback occurs.
    pub fn paint(&self,input:Input,target:Target<'_>,cancel:&AtomicBool)->Result<Receipt> {
        active(cancel)?;
        ensure!(input.map_id==303,"background bank only qualifies map303");
        ensure!(target.pitch==744 && target.height==552 && target.gate_pitch==744,"background requires native coordinate plane");
        let calls=geometry::plan(input)?;
        let base=self.base.view();let overlay=self.overlay.view();
        for call in &calls {
            ensure!(call.base_id==800 && call.overlay_id==803,"background call belongs to another bank");
            pixels::validate_paint(base,overlay,call.paint,&target)?;
        }
        let mut written_pixels=0;
        for call in &calls {
            active(cancel)?;
            written_pixels+=pixels::paint(base,overlay,call.paint,Target {words:&mut *target.words,
                pitch:target.pitch,height:target.height,gate:target.gate,gate_pitch:target.gate_pitch})?.written_pixels;
        }
        Ok(Receipt {calls:calls.len(),written_pixels})
    }
}
