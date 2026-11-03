//! Logical5641 actions18/19, bank240/241/242. Pixels never supply logical FPS.
//! Native 4E490D/4E5B50 enters1..N-1,0; WORD milliseconds are per source.
//! The single Hit is a visual launch marker, not combat authority or audio.
use super::{active, PolymorphForm, QualifiedPolymorphAction};
use crate::modern_lito::{
    npc_assets::{LoadedLayers, NpcLayer},
    shared_action_catalog::SharedActionCatalog,
    sprite_actions::{
        ActionKey, Animation, Directionality, FrameAsset, FrameEvent, SpriteActionCatalog,
    },
    world_scene::BaseAction,
};
use anyhow::{ensure, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
    time::Duration,
};

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Directed,
    Undirected,
}
#[derive(Clone)]
pub(crate) struct QualifiedCast {
    form: PolymorphForm,
    animation: Animation,
    durations: Vec<Duration>,
    terminal: Duration,
    launch: Duration,
}
impl QualifiedCast {
    pub(crate) fn from_catalog(catalog: &SpriteActionCatalog, form: PolymorphForm) -> Result<Self> {
        Self::from_catalog_kind(catalog, form, Kind::Directed)
    }
    pub(crate) fn from_catalog_kind(
        catalog: &SpriteActionCatalog,
        form: PolymorphForm,
        kind: Kind,
    ) -> Result<Self> {
        // Reuse the complete logical header/FPS-scope and native layer proof.
        QualifiedPolymorphAction::from_catalog(catalog, form, BaseAction::Idle)?;
        let (key, image, ticks, hit): (_, _, &[u32], _) = match kind {
            Kind::Directed => (18, 32, &[2, 2, 3, 4, 1, 6, 4], 5),
            Kind::Undirected => (19, 40, &[5, 5, 5, 5, 4], 3),
        };
        let logical = catalog.resolve_action(form.logical_sprite_id(), ActionKey::new(key))?;
        let bank = catalog.resolve_action(form.graphics_id(), ActionKey::new(key))?;
        ensure!(
            logical.definition_sprite_id == form.logical_sprite_id()
                && bank.definition_sprite_id == form.graphics_id()
                && logical.animation.frames_per_second == 48
                && bank.animation.frames_per_second == 24
                && logical.animation.directionality == Directionality::EightWay
                && bank.animation.directionality == Directionality::EightWay
                && logical.animation.frames == bank.animation.frames,
            "unqualified polymorph caster identity/rate/routes"
        );
        let animation = logical.animation.clone();
        ensure!(
            animation.frames.len() == ticks.len(),
            "unreviewed polymorph cast count"
        );
        let mut durations = Vec::new();
        for (i, f) in animation.frames.iter().enumerate() {
            ensure!(
                f.image_id == image
                    && f.subframe as usize == i
                    && f.duration_ticks == ticks[i]
                    && f.events.as_slice()
                        == if i == hit {
                            &[FrameEvent::Hit][..]
                        } else {
                            &[][..]
                        },
                "unreviewed polymorph cast source/control"
            );
            durations.push(Duration::from_millis(u64::from(
                f.duration_ticks * 1000 / 48,
            )));
        }
        let terminal = durations.iter().skip(1).copied().sum();
        let launch = durations[1..hit].iter().copied().sum();
        Ok(Self {
            form,
            animation,
            durations,
            terminal,
            launch,
        })
    }
    pub(crate) fn terminal(&self) -> Duration {
        self.terminal
    }
    pub(crate) fn launch(&self) -> Duration {
        self.launch
    }
    pub(crate) fn sample(&self, elapsed: Duration, heading: u8) -> Result<usize> {
        ensure!(heading < 8, "polymorph cast heading");
        let mut end = Duration::ZERO;
        for source in 1..self.durations.len() {
            end += self.durations[source];
            if elapsed < end {
                return Ok(source);
            }
        }
        Ok(0)
    }
    pub(crate) fn asset(&self, source: usize, heading: u8) -> Result<FrameAsset> {
        ensure!(
            source < self.durations.len() && heading < 8,
            "polymorph cast frame outside action"
        );
        Ok(self
            .animation
            .frame_asset(self.form.graphics_id(), source, heading)?)
    }
}
pub(crate) struct Assets {
    action: QualifiedCast,
    pixels: LoadedLayers,
}
impl Assets {
    pub(crate) fn load(
        context: &SharedActionCatalog,
        form: PolymorphForm,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::load_kind(context, form, Kind::Directed, cancel)
    }
    pub(crate) fn load_kind(
        context: &SharedActionCatalog,
        form: PolymorphForm,
        kind: Kind,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let action = match kind {
            Kind::Directed => QualifiedCast::from_catalog(context.catalog(), form)?,
            Kind::Undirected => QualifiedCast::from_catalog_kind(context.catalog(), form, kind)?,
        };
        let mut plan = BTreeMap::<String, BTreeSet<usize>>::new();
        for source in 0..action.durations.len() {
            for heading in 0..8 {
                let asset = action.asset(source, heading)?;
                for graphics_id in [240, 241, 242] {
                    let a = FrameAsset {
                        graphics_id,
                        ..asset
                    };
                    plan.entry(a.file_name())
                        .or_default()
                        .insert(a.subframe as usize);
                }
            }
        }
        ensure!(plan.len() == 24, "polymorph cast layer plan changed");
        let pixels = LoadedLayers::load(context, plan, cancel)?;
        active(cancel)?;
        Ok(Self { action, pixels })
    }
    pub(crate) fn action(&self) -> &QualifiedCast {
        &self.action
    }
    pub(crate) fn frame(&self, elapsed: Duration, heading: u8) -> Result<Frame<'_>> {
        let source = self.action.sample(elapsed, heading)?;
        let asset = self.action.asset(source, heading)?;
        Ok(Frame {
            form: self.action.form,
            source,
            body: self.pixels.layer(asset)?,
            shadow: self.pixels.layer(FrameAsset {
                graphics_id: 241,
                ..asset
            })?,
            attachment: self.pixels.layer(FrameAsset {
                graphics_id: 242,
                ..asset
            })?,
        })
    }
    #[cfg(test)]
    pub(crate) fn sources(
        &self,
    ) -> impl ExactSizeIterator<Item = &crate::modern_lito::teleport_effect::SpriteProvenance> {
        self.pixels.sources()
    }
    pub(crate) fn encoded_bytes(&self) -> usize {
        self.pixels.encoded
    }
    pub(crate) fn rgba_bytes(&self) -> usize {
        self.pixels.rgba
    }
}
pub(crate) struct Frame<'a> {
    pub(crate) form: PolymorphForm,
    pub(crate) source: usize,
    pub(crate) body: NpcLayer<'a>,
    pub(crate) shadow: NpcLayer<'a>,
    pub(crate) attachment: NpcLayer<'a>,
}

#[cfg(test)]
#[path = "polymorph_cast_tests.rs"]
mod tests;
