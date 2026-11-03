//! Qualified class1 one-shot presentation, not server skill77 permission.
//!
//! 43E6B2 maps109 to647E5C; 4A677C copies the first target's coordinates
//! into44EF00's own actor. It does NOT keep a target pointer or follow later
//! movement. 4E0EF0 retains this effect until caster18's ! launches vtable+4.
//! Once launched,44F350 ->44EF80 ->4E5B50 enters sources1..17,0; terminal0
//! is still drawn for its3 ticks, then44F330 unlinks before destruction.
//! 4E5C00/4E6EA0 select clothes105's same image and4E7000 uses the parent's
//! subframe/position with the child's104 flags. Insert main first, layer second;
//! the shared per-cell/depth queue reverses equal-depth insertion (483815).
//! Main uses the existing explicit floor-screen policy; layer128 is exact
//! RGB555 floor multiplication. RGB565/native screen rounding remain unproven.
//!
//! This API starts AFTER the qualified caster launch marker. Runtime admission,
//! event deduplication, caster playback, damage and UI integration are separate.
use super::{
    combat_animation::CrossedCue,
    disintegrate_assets::DisintegrateAssets,
    sprite::SpriteFrame,
    sprite_actions::{ActionKey, Animation},
    world_depth::NativeDrawCell,
    world_map::WorldCell,
    world_raster::{Blend, Raster},
};
use anyhow::{ensure, Context, Result};
use std::{sync::Arc, time::Duration};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Variant {
    Normal,
    Critical,
}
impl Variant {
    fn sprite_index(self) -> usize {
        match self {
            Self::Normal => 0,
            Self::Critical => 2,
        }
    }
    pub fn graphic(self) -> u32 {
        match self {
            Self::Normal => 11746,
            Self::Critical => 11748,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Binding {
    pub world_id: u64,
    pub map_digest: [u8; 32],
    pub event_id: u64,
    pub start_revision: u64,
    /// Authoritative target position captured when the cast was admitted.
    pub target: WorldCell,
}
impl Binding {
    fn validate(self) -> Result<Self> {
        ensure!(
            [self.world_id, self.event_id, self.start_revision]
                .into_iter()
                .all(|x| (1..=i64::MAX as u64).contains(&x))
                && self.map_digest != [0; 32],
            "invalid disintegrate event binding"
        );
        NativeDrawCell::from_world(self.target).context("disintegrate anchor overflow")?;
        Ok(self)
    }
}
#[derive(Clone)]
struct Sequence {
    animation: Animation,
    starts: [u64; 18],
    ends: [u64; 18],
}
impl Sequence {
    fn new(animation: &Animation) -> Result<Self> {
        ensure!(
            animation.frames.len() == 18 && animation.duration_ratio() == (60, 24),
            "unqualified disintegrate timeline"
        );
        let mut starts = [0; 18];
        let mut ends = [0; 18];
        let mut total = 0;
        for i in 0..18 {
            starts[i] = total;
            total += u64::from(animation.frames[(i + 1) % 18].duration_ticks);
            ends[i] = total;
        }
        ensure!(
            total == 60 && starts[17] == 57,
            "disintegrate terminal dwell"
        );
        Ok(Self {
            animation: animation.clone(),
            starts,
            ends,
        })
    }
    fn tick(elapsed: Duration) -> u64 {
        ((elapsed.as_nanos() * 24 / 1_000_000_000).min(60)) as u64
    }
    fn source(&self, tick: u64) -> usize {
        if tick >= 57 {
            0
        } else {
            self.ends.partition_point(|&end| end <= tick) + 1
        }
    }
}
/// Owned event lifetime: no pointer/reference into a target actor or scene.
#[derive(Clone)]
pub struct DisintegrateEffect {
    assets: Arc<DisintegrateAssets>,
    binding: Binding,
    variant: Variant,
    sequence: Sequence,
    next_cue_position: usize,
    last_elapsed: Option<Duration>,
}
pub struct EffectSample {
    pub source_frame: usize,
    pub subframe: usize,
    pub finished: bool,
    pub crossed_cues: Vec<CrossedCue>,
}
/// Only produced from already-qualified owned resources. No caller-supplied
/// partial-alpha pixels, blend flags, arbitrary SPRs or mutable scene pointer.
pub struct DisintegrateDraw<'a> {
    pub(crate) binding: Binding,
    main: &'a SpriteFrame,
    layer: &'a SpriteFrame,
}
impl DisintegrateDraw<'_> {
    pub fn binding(&self) -> Binding {
        self.binding
    }
    pub fn frames(&self) -> (&SpriteFrame, &SpriteFrame) {
        (self.main, self.layer)
    }
    pub(crate) fn enqueue<'a>(&'a self, raster: &mut Raster<'a>) -> Result<()> {
        let origin =
            NativeDrawCell::from_world(self.binding.target).context("effect anchor overflow")?;
        raster.sprite(origin, self.main, 0, Blend::Screen)?;
        raster.sprite(origin, self.layer, 0, Blend::Multiply)?;
        Ok(())
    }
}
impl DisintegrateEffect {
    pub fn start(
        assets: Arc<DisintegrateAssets>,
        binding: Binding,
        variant: Variant,
    ) -> Result<Self> {
        let binding = binding.validate()?;
        let action = assets
            .catalog()
            .catalog()
            .resolve_action(variant.graphic(), ActionKey::new(0))?;
        let sequence = Sequence::new(action.animation)?;
        Ok(Self {
            assets,
            binding,
            variant,
            sequence,
            next_cue_position: 0,
            last_elapsed: None,
        })
    }
    pub fn binding(&self) -> Binding {
        self.binding
    }
    pub fn variant(&self) -> Variant {
        self.variant
    }
    pub fn duration(&self) -> Duration {
        Duration::from_millis(2500)
    }
    pub fn terminal_start(&self) -> Duration {
        Duration::from_millis(2375)
    }
    pub fn assets(&self) -> &Arc<DisintegrateAssets> {
        &self.assets
    }
    /// A late tick crosses each source cue once in source order. Caller still
    /// applies its audio-age/world-generation policy; Hit never changes HP.
    pub fn advance(&mut self, elapsed: Duration) -> Result<EffectSample> {
        ensure!(
            self.last_elapsed.is_none_or(|last| elapsed >= last),
            "disintegrate elapsed regressed"
        );
        let tick = Sequence::tick(elapsed);
        let end = self.sequence.starts.partition_point(|&start| start <= tick);
        let mut crossed_cues = Vec::new();
        crossed_cues.try_reserve_exact(2)?;
        for i in self.next_cue_position..end {
            let source = (i + 1) % 18;
            for (event_index, event) in self.sequence.animation.frames[source]
                .events
                .iter()
                .enumerate()
            {
                ensure!(crossed_cues.len() < 2, "disintegrate cue bound");
                crossed_cues.push(CrossedCue {
                    source_frame: source,
                    event_index,
                    tick: self.sequence.starts[i],
                    event: event.clone(),
                });
            }
        }
        let source_frame = self.sequence.source(tick);
        let subframe = self.sequence.animation.frames[source_frame].subframe as usize;
        self.last_elapsed = Some(elapsed);
        self.next_cue_position = end;
        Ok(EffectSample {
            source_frame,
            subframe,
            finished: tick == 60,
            crossed_cues,
        })
    }
    /// Frame lookup does not consume audio/hit cues. The fixed target cell is
    /// projected by the current scene, so camera motion works without following
    /// a moved/removed target. A completed effect supplies no drawable frame.
    pub fn draw(&self, elapsed: Duration) -> Result<Option<DisintegrateDraw<'_>>> {
        ensure!(
            self.last_elapsed.is_none_or(|last| elapsed >= last),
            "disintegrate draw elapsed regressed"
        );
        let tick = Sequence::tick(elapsed);
        if tick == 60 {
            return Ok(None);
        }
        let source = self.sequence.source(tick);
        let subframe = self.sequence.animation.frames[source].subframe as usize;
        let index = self.variant.sprite_index();
        let main = &self.assets.sprites()[index].decoded().frames[subframe];
        let layer = &self.assets.sprites()[index + 1].decoded().frames[subframe];
        Ok(Some(DisintegrateDraw {
            binding: self.binding,
            main,
            layer,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::sprite_actions::{ActionLimits, SpriteActionCatalog};
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; all36 independent composed frames/PCM717/timeline and ownership; no hardware playback"]
    fn shared_one_shots_match_csharp_composites_and_release_owned_assets() {
        shared_one_shots_match_csharp_composites_and_release_owned_assets_with_profile(crate::modern_lito::disintegrate_assets::DisintegrateAssets::load);
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; explicit current launcher TW profile; original CSharp SPR/pixel/clock assertions retained, no legacy root fallback"]
    fn shared_one_shots_match_csharp_composites_and_release_owned_assets_current_launcher_tw_profile() {
        shared_one_shots_match_csharp_composites_and_release_owned_assets_with_profile(crate::modern_lito::disintegrate_assets::DisintegrateAssets::load_current_tw);
    }

    fn shared_one_shots_match_csharp_composites_and_release_owned_assets_with_profile(profile_loader: fn(&std::path::Path, &std::sync::atomic::AtomicBool) -> anyhow::Result<crate::modern_lito::disintegrate_assets::DisintegrateAssets>) {
        use crate::modern_lito::{
            compositor::Canvas,
            sprite_actions::FrameEvent,
            world_map::{PixelPoint, ViewProjection},
        };
        use sha2::{Digest, Sha256};
        use std::sync::atomic::AtomicBool;
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
        );
        let assets = Arc::new(profile_loader(&root, &AtomicBool::new(false)).unwrap());
        let bank = assets.sound_bank();
        let sound = bank.get(717).unwrap();
        let fmt = sound.format();
        assert_eq!(
            (bank.len(), sound.file_bytes(), sound.pcm_bytes().len()),
            (1, 188278, 188158)
        );
        assert_eq!(
            (
                fmt.channels,
                fmt.sample_rate,
                fmt.byte_rate,
                fmt.block_align,
                fmt.bits_per_sample
            ),
            (1, 22050, 44100, 2, 16)
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(sound.pcm_bytes())),
            "0ce5415ef5e686488d174965d527390b45bac8084d1f6f897ccedce2ea8227d5"
        );
        assert_eq!(
            sound
                .sha256()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
            "ce44eb59e186f2becb660df9a9f8add27e63a5a24ce0efd7bef0b62dc13f4a7f"
        );
        assert_eq!(assets.sound_group().candidates(), &[717]);
        assert_eq!(
            assets.sound_group().mode,
            crate::modern_lito::combat_animation::NativeAudioMode::SuppressWhileSameIdPlaying
        );
        let target = WorldCell { x: 32724, y: 32784 };
        let binding = Binding {
            world_id: 1,
            map_digest: [1; 32],
            event_id: 1,
            start_revision: 1,
            target,
        };
        let view = ViewProjection {
            anchor_cell: target,
            anchor_pixel: PixelPoint { x: 400, y: 300 },
        };
        for (variant, expected) in [
            (
                Variant::Normal,
                "027528e98b208cc65f8f0d9e59aadfc60c4d7e6162555091023564debf635bbc",
            ),
            (
                Variant::Critical,
                "f4c3d2ff5515ad5d898c005a187a11e6532507561f05f456d7e9f7bcda9ea604",
            ),
        ] {
            let mut effect = DisintegrateEffect::start(assets.clone(), binding, variant).unwrap();
            let mut digest = Sha256::new();
            let mut cues = Vec::new();
            for i in 0..18 {
                let elapsed =
                    Duration::from_nanos((effect.sequence.starts[i] * 1_000_000_000).div_ceil(24));
                let sample = effect.advance(elapsed).unwrap();
                assert_eq!(
                    (sample.source_frame, sample.subframe, sample.finished),
                    ((i + 1) % 18, i, false)
                );
                cues.extend(sample.crossed_cues);
                assert!(effect.advance(elapsed).unwrap().crossed_cues.is_empty());
                let draw = effect.draw(elapsed).unwrap().unwrap();
                assert_eq!(draw.binding().target, target);
                let mut canvas = Canvas::new(800, 600).unwrap();
                let mut raster = Raster::new(view, &canvas);
                draw.enqueue(&mut raster).unwrap();
                raster
                    .paint(&mut canvas, &AtomicBool::new(false), 0)
                    .unwrap();
                digest.update(canvas.rgba());
            }
            assert_eq!(format!("{:x}", digest.finalize()), expected);
            assert_eq!(cues.len(), 2);
            assert_eq!(
                (cues[0].source_frame, cues[0].tick, &cues[0].event),
                (1, 0, &FrameEvent::Sound(717))
            );
            assert_eq!(
                (cues[1].source_frame, cues[1].tick, &cues[1].event),
                (3, 4, &FrameEvent::Hit)
            );
            assert!(effect.draw(Duration::from_millis(2499)).unwrap().is_some());
            assert!(
                effect
                    .advance(Duration::from_millis(2500))
                    .unwrap()
                    .finished
            );
            assert!(effect.draw(Duration::from_millis(2500)).unwrap().is_none());
            assert!(effect
                .advance(Duration::MAX)
                .unwrap()
                .crossed_cues
                .is_empty());
            assert!(effect.draw(Duration::MAX).unwrap().is_none());
            assert!(effect.advance(Duration::ZERO).is_err());
            assert!(effect.draw(Duration::ZERO).is_err());
        }
        let mut late = DisintegrateEffect::start(assets.clone(), binding, Variant::Normal).unwrap();
        let sample = late.advance(Duration::MAX).unwrap();
        assert!(sample.finished);
        assert_eq!(sample.crossed_cues.len(), 2);
        assert!(late.advance(Duration::MAX).unwrap().crossed_cues.is_empty());
        // Dropping the original owner does not invalidate a started event. The
        // event contains no target pointer; target removal is tested in Scene.
        let weak = Arc::downgrade(&assets);
        drop(assets);
        assert!(weak.upgrade().is_some());
        drop(late);
        assert!(weak.upgrade().is_none());
        // The separate prepared sound may safely outlive graphics for a voice.
        assert_eq!(bank.get(717).unwrap().pcm_bytes().len(), 188158);
    }
    #[test]
    fn qualified_source_order_and_tick_edges_keep_terminal_dwell() {
        let text=b"#1 1\n0.effect(0 18,0.17:3 0.0:2[717 0.1:2 0.2:6! 0.3:5 0.4:5 0.5:3 0.6:3 0.7:3 0.8:3 0.9:2 0.10:1 0.11:3 0.12:1 0.13:4 0.14:5 0.15:5 0.16:4)\n";
        let catalog =
            SpriteActionCatalog::parse_for_inspection(text, ActionLimits::default()).unwrap();
        let seq = Sequence::new(
            catalog
                .resolve_action(1, ActionKey::new(0))
                .unwrap()
                .animation,
        )
        .unwrap();
        assert_eq!(
            seq.starts,
            [0, 2, 4, 10, 15, 20, 23, 26, 29, 32, 34, 35, 38, 39, 43, 48, 53, 57]
        );
        for i in 0..18 {
            let at = Duration::from_nanos((seq.starts[i] * 1_000_000_000).div_ceil(24));
            assert_eq!(seq.source(Sequence::tick(at)), (i + 1) % 18);
            if i > 0 {
                assert_eq!(seq.source(Sequence::tick(at - Duration::from_nanos(1))), i);
            }
        }
        assert_eq!(Sequence::tick(Duration::from_millis(2499)), 59);
        assert_eq!(Sequence::tick(Duration::from_millis(2500)), 60);
        assert_eq!(Sequence::tick(Duration::MAX), 60);
        let valid = Binding {
            world_id: 1,
            map_digest: [1; 32],
            event_id: 1,
            start_revision: 1,
            target: WorldCell { x: 32768, y: 32768 },
        };
        valid.validate().unwrap();
        for b in [
            Binding {
                world_id: 0,
                ..valid
            },
            Binding {
                event_id: u64::MAX,
                ..valid
            },
            Binding {
                start_revision: 0,
                ..valid
            },
            Binding {
                map_digest: [0; 32],
                ..valid
            },
            Binding {
                target: WorldCell { x: i32::MAX, y: 0 },
                ..valid
            },
        ] {
            assert!(b.validate().is_err());
        }
    }
}
