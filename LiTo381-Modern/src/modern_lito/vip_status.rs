//! Owner-only permanent VIP icons. XML selects pictures, never authority,
//! lifetime, mutually exclusive equipment, or the similarly named GFX domain.
use super::{
    battle_follow::Observation,
    battle_payload::data::Phase,
    compositor::Canvas,
    effect_catalog::{EffectCatalog, EffectId},
    inventory_icons::{IconSource, SharedInventoryIcons},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

fn active(c: &AtomicBool) -> Result<()> {
    ensure!(!c.load(Ordering::Acquire), "VIP status cancelled");
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Picture {
    pub effect: EffectId,
    pub graphic: u32,
    pub source: IconSource,
    pub width: u32,
    pub height: u32,
    rgba: Vec<u8>,
}
impl Picture {
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
}
pub struct Assets {
    catalog: EffectCatalog,
    pictures: BTreeMap<EffectId, Picture>,
}
impl Assets {
    /// Pin precisely the requested effects. Unknown/missing/wrong-sized art
    /// fails the entire candidate, not one transparent slot in a successful set.
    pub fn load(
        root: &Path,
        catalog: EffectCatalog,
        ids: &[i64],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(root.is_absolute(), "VIP status needs absolute shared root");
        ensure!(ids.len() <= 512, "VIP status capacity exceeded");
        let wanted = ids
            .iter()
            .map(|v| EffectId::from_wire(*v))
            .collect::<Result<BTreeSet<_>>>()?;
        for id in &wanted {
            catalog.get(*id)?;
        }
        let mut pictures = BTreeMap::new();
        if !wanted.is_empty() {
            let mut archive = SharedInventoryIcons::open_cancellable(root, cancel)?;
            for id in wanted {
                active(cancel)?;
                let mapping = catalog.get(id)?;
                let icon = archive.load(i64::from(mapping.icon), cancel)?;
                let image = icon.image();
                ensure!(
                    image.raw_offset() == [0, 0]
                        && (1..=32).contains(&image.header_width())
                        && (1..=32).contains(&image.width())
                        && (1..=32).contains(&image.height()),
                    "unqualified VIP status icon layout for effect {} picture {}",
                    id.value(),
                    mapping.icon
                );
                pictures.insert(
                    id,
                    Picture {
                        effect: id,
                        graphic: mapping.icon,
                        source: icon.source().clone(),
                        width: image.width() as u32,
                        height: image.height() as u32,
                        rgba: image.reference_rgba()?,
                    },
                );
            }
        }
        active(cancel)?;
        Ok(Self { catalog, pictures })
    }
    pub fn catalog(&self) -> &EffectCatalog {
        &self.catalog
    }
    pub fn picture(&self, id: EffectId) -> Option<&Picture> {
        self.pictures.get(&id)
    }
    pub fn supports(&self, observation: &Observation) -> Result<bool> {
        Ok(required(observation)?
            .iter()
            .all(|id| self.pictures.contains_key(id)))
    }
    /// CPU-only, all-or-nothing composition in a caller-provided HUD rectangle.
    /// All icons must fit; no truncation, world-camera offsets, or timers.
    pub fn draw(
        &self,
        observation: &Observation,
        canvas: &mut Canvas,
        origin: (u32, u32),
        columns: u32,
        cancel: &AtomicBool,
    ) -> Result<Vec<(EffectId, u32, u32)>> {
        active(cancel)?;
        let ids = required(observation)?;
        ensure!(
            self.supports(observation)?,
            "VIP status resource reload required"
        );
        ensure!(columns > 0 && columns <= 512, "invalid VIP status columns");
        let mut placements = Vec::new();
        for (index, id) in ids.into_iter().enumerate() {
            let x = origin
                .0
                .checked_add((index as u32 % columns) * 32)
                .context("VIP status x overflow")?;
            let y = origin
                .1
                .checked_add((index as u32 / columns) * 32)
                .context("VIP status y overflow")?;
            ensure!(
                x.checked_add(32).is_some_and(|end| end <= canvas.width())
                    && y.checked_add(32).is_some_and(|end| end <= canvas.height()),
                "VIP status strip does not fit"
            );
            placements.push((id, x, y));
        }
        // Qualification and layout precede any visible mutation. A cancelled
        // copy cannot leave earlier icons published on the caller's canvas.
        let mut candidate = Canvas::new(canvas.width(), canvas.height())?;
        candidate.copy_from(canvas)?;
        for (id, x, y) in &placements {
            active(cancel)?;
            let picture = &self.pictures[id];
            candidate.blit_rgba(
                &picture.rgba,
                picture.width,
                picture.height,
                *x as i32,
                *y as i32,
            )?;
        }
        active(cancel)?;
        *canvas = candidate;
        Ok(placements)
    }
}
/// NONE_TIME_ICON semantics: the confirmed owner's applied icon set controls
/// presence. XML duration/death/group flags cannot expire or grant equipment.
pub fn required(o: &Observation) -> Result<Vec<EffectId>> {
    if o.data().world.phase == Phase::Aborted {
        return Ok(Vec::new());
    }
    let Some(vip) = &o.data().vip else {
        return Ok(Vec::new());
    };
    vip.owner
        .icons
        .0
        .iter()
        .map(|v| EffectId::from_wire(*v))
        .collect()
}
