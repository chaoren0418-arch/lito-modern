//! Explicit SharedClassicBasicHealV1 presentation profile, not raw-gfx fallback.
//!
//! The source gameplay template retained from380_db.sql uses11722, absent in
//! this shared list.spr. The separately retained2010 skill1 row specifies744
//! (same skill1/action19/MP4). The inspected shared #744 is explicitly heal:
//! action0,21frames at24FPS, source0=0.20:3, source1=0.0:3[109 through
//! source20=0.19:3, attr104=8. This profile binds the semantic basic-heal event
//! to that qualified shared744. It never rewrites the server template, claims
//! 11722 exists, silently substitutes another effect, or computes healing.
//!
//! Native one-shot body1..20 then terminal0 means60ticks=2.5seconds. The
//! effect disappears at completion; the native sound marker is delivered once
//! through the owned audio queue, never played here. Caster19 is supplied by
//! the already-qualified actor bank, not duplicated or replaced by this module.

pub use super::teleport_effect::{SpriteProvenance, VisualSample};
pub use super::tower_assets::{HonorAssetError, HonorPlayback, HonorSample};
use super::{
    combat_animation::NativeAudioGroup,
    combat_audio::SoundBank,
    compositor::Canvas,
    shared_action_catalog::SharedActionCatalog,
    sprite::SpriteFrame,
    sprite_actions::{ActionKey, Animation, Directionality, FrameEvent, SpriteActionCatalog},
    teleport_effect::draw_screen_frame,
    tower_assets::{active, OneShot, Result, SharedEffectBank},
    world_map::PixelPoint,
};
use std::{
    path::Path,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

pub const SEMANTIC_SKILL_ID: u32 = 1;
pub const SOURCE_TEMPLATE_GRAPHIC: u32 = 11722;
pub const PRESENTATION_GRAPHIC: u32 = 744;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HonorHealingProfile {
    SharedClassicBasicHealV1,
}

fn qualify(catalog: &SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>> {
    let sprite = catalog
        .sprites
        .get(&PRESENTATION_GRAPHIC)
        .ok_or(HonorAssetError::Unsupported(
            "missing SharedClassicBasicHealV1 graphic744",
        ))?;
    let resolved = catalog.resolve_action(PRESENTATION_GRAPHIC, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != PRESENTATION_GRAPHIC
        || sprite.image_count != Some(1)
        || sprite.actions.len() != 1
        || resolved.definition_sprite_id != PRESENTATION_GRAPHIC
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != 21
        || sprite.attributes.len() != 1
        || sprite.attributes[0].key != ActionKey::new(104)
        || sprite.attributes[0].body_bytes != b"8"
    {
        return Err(HonorAssetError::Unsupported(
            "unreviewed SharedClassicBasicHealV1 controls",
        ));
    }
    for (index, frame) in animation.frames.iter().enumerate() {
        let expected_subframe = if index == 0 { 20 } else { index as u32 - 1 };
        let expected_events = if index == 1 {
            vec![FrameEvent::Sound(109)]
        } else {
            Vec::new()
        };
        if frame.image_id != 0
            || frame.subframe != expected_subframe
            || frame.duration_ticks != 3
            || frame.events != expected_events
        {
            return Err(HonorAssetError::Unsupported(
                "unreviewed basic-heal frame/cue",
            ));
        }
    }
    Ok(vec![(PRESENTATION_GRAPHIC, 0, animation.clone())])
}

pub struct HonorHealingAssets {
    bank: SharedEffectBank,
    sequence: OneShot,
}
impl HonorHealingAssets {
    /// All-or-nothing worker-only load; no IO in sampling/drawing. The selected
    /// profile is explicit through profile(), not a retry on missing11722.
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        Self::load_with_catalog(&catalog, cancel)
    }
    pub(crate) fn load_with_catalog(
        catalog: &SharedActionCatalog,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let bank = SharedEffectBank::load_with_catalog(catalog, cancel, qualify)?;
        let sequence = OneShot::new(Arc::clone(bank.animation(PRESENTATION_GRAPHIC, 0)?))?;
        Ok(Self { bank, sequence })
    }
    pub fn profile(&self) -> HonorHealingProfile {
        HonorHealingProfile::SharedClassicBasicHealV1
    }
    pub fn duration(&self) -> Duration {
        self.sequence.duration()
    }
    pub fn start_effect(&self) -> HonorPlayback {
        self.sequence.start()
    }
    pub fn effect_frame(&self, elapsed: Duration) -> Result<VisualSample<'_>> {
        let (index, finished) = self.sequence.sample(elapsed)?;
        self.bank.frame(PRESENTATION_GRAPHIC, 0, index, finished)
    }
    /// Anchor is the confirmed target-event position, never later actor motion.
    pub fn draw_effect(
        &self,
        canvas: &mut Canvas,
        anchor: PixelPoint,
        elapsed: Duration,
    ) -> Result<bool> {
        let sample = self.effect_frame(elapsed)?;
        if sample.finished {
            return Ok(false);
        }
        draw_screen_frame(canvas, sample.frame, anchor).map_err(HonorAssetError::Render)?;
        Ok(true)
    }
    pub fn sound_bank(&self) -> Arc<SoundBank> {
        self.bank.sound_bank()
    }
    pub fn sound_groups(&self) -> &[NativeAudioGroup] {
        self.bank.groups()
    }
    pub fn sound_group(&self) -> Result<&NativeAudioGroup> {
        self.bank
            .groups()
            .iter()
            .find(|g| g.candidates() == [109])
            .ok_or(HonorAssetError::Unsupported(
                "missing qualified basic-heal sound",
            ))
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.bank.catalog_sha()
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.bank.sources()
    }
    pub fn sprite_frames(&self, name: &str) -> Result<&[SpriteFrame]> {
        self.bank.frames(name)
    }
    pub fn encoded_bytes(&self) -> usize {
        self.bank.encoded()
    }
    pub fn rgba_bytes(&self) -> usize {
        self.bank.rgba()
    }
}

#[cfg(test)]
mod tests {
    use super::super::sprite_actions::ActionDefinition;
    use super::*;
    fn catalog() -> SpriteActionCatalog {
        let mut text = String::from("1 0 0\n#744 1 heal\n0.fire(0 21,0.20:3");
        for index in 0..20 {
            text.push_str(&format!(
                " 0.{index}:3{}",
                if index == 0 { "[109" } else { "" }
            ));
        }
        text.push_str(")104.attr(8)\n");
        SpriteActionCatalog::parse_for_inspection(text.as_bytes(), Default::default()).unwrap()
    }
    #[test]
    fn explicit_profile_preserves_source_template_and_native_sound() {
        assert_eq!(SOURCE_TEMPLATE_GRAPHIC, 11722);
        assert_eq!(PRESENTATION_GRAPHIC, 744);
        let animations = qualify(&catalog()).unwrap();
        let sequence = OneShot::new(Arc::new(animations[0].2.clone())).unwrap();
        assert_eq!(sequence.duration(), Duration::from_millis(2500));
        assert_eq!(sequence.sample(Duration::ZERO).unwrap(), (1, false));
        assert_eq!(
            sequence.sample(Duration::from_millis(125)).unwrap(),
            (2, false)
        );
        assert_eq!(
            sequence
                .sample(Duration::from_nanos(2_499_999_999))
                .unwrap(),
            (20, false)
        );
        assert_eq!(
            sequence.sample(Duration::from_millis(2500)).unwrap(),
            (0, true)
        );
        let mut playback = sequence.start();
        let sample = playback.advance(Duration::ZERO).unwrap();
        assert_eq!(sample.crossed_cues.len(), 1);
        assert_eq!(sample.crossed_cues[0].event, FrameEvent::Sound(109));
        assert_eq!(sample.crossed_cues[0].tick, 0);
        assert!(playback
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        assert!(playback
            .advance(Duration::MAX)
            .unwrap()
            .crossed_cues
            .is_empty());
        assert!(playback.advance(Duration::ZERO).is_err());
    }
    #[test]
    fn missing744_or_altered_events_do_not_fallback_to11722_or_potions() {
        let mut cat = catalog();
        cat.sprites.remove(&744);
        assert!(qualify(&cat).is_err());
        for event in [
            FrameEvent::Sound(119),
            FrameEvent::Effect(109),
            FrameEvent::Hit,
            FrameEvent::Overlay(744),
            FrameEvent::UninterpretedNumeric {
                marker: b'^',
                value: 109,
            },
        ] {
            let mut cat = catalog();
            let ActionDefinition::Animation(a) = cat
                .sprites
                .get_mut(&744)
                .unwrap()
                .actions
                .get_mut(&ActionKey::new(0))
                .unwrap()
            else {
                panic!("fixture")
            };
            a.frames[1].events = vec![event];
            assert!(qualify(&cat).is_err());
        }
    }
    #[test]
    fn cancelled_load_precedes_every_archive_read() {
        assert!(matches!(
            HonorHealingAssets::load(Path::new("absent"), &AtomicBool::new(true)),
            Err(HonorAssetError::Cancelled)
        ));
    }
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; actual744 screen layer, explicit profile and fixed elapsed; no audio device"]
    fn actual_basic_heal_screen_profile_uses_qualified744_and_expires_without_a_decal() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        let assets = HonorHealingAssets::load(Path::new(&root), &AtomicBool::new(false)).unwrap();
        assert_eq!(
            assets.profile(),
            HonorHealingProfile::SharedClassicBasicHealV1
        );
        let anchor = PixelPoint { x: 400, y: 350 };
        let mut changed = (false, false);
        for ms in [0, 125, 1000, 2499] {
            let elapsed = Duration::from_millis(ms);
            let sample = assets.effect_frame(elapsed).unwrap();
            assert_eq!(sample.graphics_id, 744);
            let result = super::super::teleport_effect::screen_test_support::compare(
                sample.frame,
                anchor,
                |canvas| assets.draw_effect(canvas, anchor, elapsed).unwrap(),
            );
            changed.0 |= result.0;
            changed.1 |= result.1;
        }
        assert_eq!(changed, (true, true));
        super::super::teleport_effect::screen_test_support::retired(|canvas| {
            assets
                .draw_effect(canvas, anchor, assets.duration())
                .unwrap()
        });
        let mut playback = assets.start_effect();
        assert_eq!(
            playback.advance(Duration::ZERO).unwrap().crossed_cues[0].event,
            FrameEvent::Sound(109)
        );
        assert!(playback
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        assert!(playback
            .advance(assets.duration())
            .unwrap()
            .crossed_cues
            .is_empty());
        assert!(playback
            .advance(Duration::MAX)
            .unwrap()
            .crossed_cues
            .is_empty());
    }
}
