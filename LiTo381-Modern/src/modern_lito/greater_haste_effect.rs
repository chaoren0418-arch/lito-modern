//! Qualified shared Greater Haste (skill54/action19/effect3104) presentation.
//!
//! The inspected shared list.spr defines #3104 as one undirected action0:
//! six source frames at24FPS, subframes5,0,1,2,3,4 with five ticks each,
//! attr104=8 and NO events. The established one-shot clock presents1..5 then
//! terminal0 (25ticks). The effect expires; it is not a 2400-second decal.
//! Caster action19 and the server's timed status are separate responsibilities.
//! No invented potion sound, private resource, cast permission or speed change.
//!
//! Worker-only, cancellable, all-or-nothing SharedEffectBank loading; its
//! existing catalog/index/encoded/RGBA bounds and native shard routing apply.
//! No file handles or I/O survive into rendering. This resource bank alone
//! does not enable skill54 in a catalog, protocol, scene or player command.

pub use super::teleport_effect::{SpriteProvenance, VisualSample};
pub use super::tower_assets::{HonorAssetError as GreaterHasteAssetError, HonorPlayback};
use super::{
    compositor::Canvas,
    shared_action_catalog::SharedActionCatalog,
    sprite::SpriteFrame,
    sprite_actions::{ActionKey, Animation, Directionality, SpriteActionCatalog},
    teleport_effect::draw_screen_frame,
    tower_assets::{active, OneShot, Result, SharedEffectBank},
    world_map::PixelPoint,
};
use std::{
    path::Path,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

pub const SKILL_ID: u32 = 54;
pub const GRAPHIC: u32 = 3104;

fn qualify(catalog: &SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>> {
    let sprite = catalog
        .sprites
        .get(&GRAPHIC)
        .ok_or(GreaterHasteAssetError::Unsupported(
            "missing shared Greater Haste3104",
        ))?;
    let resolved = catalog.resolve_action(GRAPHIC, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != GRAPHIC
        || sprite.image_count != Some(1)
        || sprite.actions.len() != 1
        || resolved.definition_sprite_id != GRAPHIC
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != 6
        || sprite.attributes.len() != 1
        || sprite.attributes[0].key != ActionKey::new(104)
        || sprite.attributes[0].body_bytes != b"8"
    {
        return Err(GreaterHasteAssetError::Unsupported(
            "unreviewed Greater Haste controls",
        ));
    }
    for (index, frame) in animation.frames.iter().enumerate() {
        if frame.image_id != 0
            || frame.subframe != if index == 0 { 5 } else { index as u32 - 1 }
            || frame.duration_ticks != 5
            || !frame.events.is_empty()
        {
            return Err(GreaterHasteAssetError::Unsupported(
                "unreviewed Greater Haste frame/cue",
            ));
        }
    }
    Ok(vec![(GRAPHIC, 0, animation.clone())])
}

pub struct GreaterHasteAssets {
    bank: SharedEffectBank,
    sequence: OneShot,
}
impl GreaterHasteAssets {
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
        // This definition has no marker. Future altered definitions must not
        // acquire sounds just because the shared bank supports other effects.
        if !bank.groups().is_empty() || !bank.sound_bank().is_empty() {
            return Err(GreaterHasteAssetError::Unsupported(
                "unexpected Greater Haste audio",
            ));
        }
        let sequence = OneShot::new(Arc::clone(bank.animation(GRAPHIC, 0)?))?;
        Ok(Self { bank, sequence })
    }
    pub fn duration(&self) -> Duration {
        self.sequence.duration()
    }
    pub fn start_effect(&self) -> HonorPlayback {
        self.sequence.start()
    }
    pub fn effect_frame(&self, elapsed: Duration) -> Result<VisualSample<'_>> {
        let (index, finished) = self.sequence.sample(elapsed)?;
        self.bank.frame(GRAPHIC, 0, index, finished)
    }
    /// Fixed confirmed cast anchor; later movement must not drag the old effect.
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
        draw_screen_frame(canvas, sample.frame, anchor).map_err(GreaterHasteAssetError::Render)?;
        Ok(true)
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
    use super::super::{
        sprite_actions::{ActionDefinition, FrameEvent},
        teleport_effect::screen_test_support,
    };
    use super::*;
    fn catalog() -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(
            b"1 0 0\n#3104 1 greater haste\n0.fire(0 6,0.5:5 0.0:5 0.1:5 0.2:5 0.3:5 0.4:5)104.attr(8)\n",
            Default::default()).unwrap()
    }
    fn animation(catalog: &mut SpriteActionCatalog) -> &mut Animation {
        let ActionDefinition::Animation(a) = catalog
            .sprites
            .get_mut(&GRAPHIC)
            .unwrap()
            .actions
            .get_mut(&ActionKey::new(0))
            .unwrap()
        else {
            panic!("fixture")
        };
        a
    }
    #[test]
    fn source_order_fractional_boundaries_terminal_and_silent_cursor_are_exact() {
        let definitions = qualify(&catalog()).unwrap();
        let sequence = OneShot::new(Arc::new(definitions[0].2.clone())).unwrap();
        assert_eq!(sequence.duration(), Duration::from_nanos(1_041_666_667));
        assert_eq!(sequence.sample(Duration::ZERO).unwrap(), (1, false));
        let mut cursor = sequence.start();
        assert!(cursor
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        for boundary in 1u64..=5 {
            let nanos = (boundary * 5 * 1_000_000_000).div_ceil(24);
            assert_eq!(
                sequence.sample(Duration::from_nanos(nanos - 1)).unwrap(),
                (boundary as usize, false)
            );
            let expected = if boundary == 5 {
                (0, true)
            } else {
                (boundary as usize + 1, false)
            };
            assert_eq!(
                sequence.sample(Duration::from_nanos(nanos)).unwrap(),
                expected
            );
            let sample = cursor.advance(Duration::from_nanos(nanos)).unwrap();
            assert_eq!((sample.source_frame, sample.finished), expected);
            assert!(sample.crossed_cues.is_empty());
            assert!(cursor
                .advance(Duration::from_nanos(nanos))
                .unwrap()
                .crossed_cues
                .is_empty());
        }
        assert!(cursor.advance(Duration::ZERO).is_err());
        assert!(cursor
            .advance(Duration::MAX)
            .unwrap()
            .crossed_cues
            .is_empty());
        assert_eq!(sequence.sample(Duration::MAX).unwrap(), (0, true));
    }
    #[test]
    fn unknown_controls_frames_aliases_and_all_cues_are_rejected_without_fallback() {
        let mut cat = catalog();
        cat.sprites.remove(&GRAPHIC);
        assert!(qualify(&cat).is_err());
        for edit in 0..8 {
            let mut cat = catalog();
            match edit {
                0 => cat.sprites.get_mut(&GRAPHIC).unwrap().graphics_id = 191,
                1 => {
                    cat.sprites.get_mut(&GRAPHIC).unwrap().attributes[0].body_bytes = b"9".to_vec()
                }
                2 => cat.sprites.get_mut(&GRAPHIC).unwrap().image_count = Some(2),
                3 => animation(&mut cat).frames_per_second = 30,
                4 => {
                    animation(&mut cat).frames.pop();
                }
                5 => animation(&mut cat).frames[2].duration_ticks = 0,
                6 => animation(&mut cat).frames[2].image_id = 1,
                _ => animation(&mut cat).frames[2].subframe = 999,
            }
            assert!(qualify(&cat).is_err(), "edit {edit}");
        }
        for event in [
            FrameEvent::Sound(121),
            FrameEvent::Effect(191),
            FrameEvent::Hit,
            FrameEvent::Overlay(3104),
            FrameEvent::UninterpretedNumeric {
                marker: b'^',
                value: 119,
            },
        ] {
            for index in 0..6 {
                let mut cat = catalog();
                animation(&mut cat).frames[index].events.push(event.clone());
                assert!(qualify(&cat).is_err());
            }
        }
    }
    #[test]
    fn cancellation_precedes_archive_access() {
        assert!(matches!(
            GreaterHasteAssets::load(Path::new("absent"), &AtomicBool::new(true)),
            Err(GreaterHasteAssetError::Cancelled)
        ));
    }
    #[test]
    #[ignore = "explicit shared root; actual skill54 SPR, fixed screen projection, no HWND or audio device"]
    fn real_shared_greater_haste_effect_frames_and_terminal() {
        use sha2::{Digest, Sha256};

        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
        );
        assert!(root.is_absolute());
        let cancel = AtomicBool::new(false);
        let assets = GreaterHasteAssets::load(&root, &cancel).unwrap();
        assert_eq!(assets.sprite_sources().len(), 1);
        assert_eq!(assets.encoded_bytes(), 18_170);
        assert_eq!(assets.rgba_bytes(), 227_312);
        let source = assets.sprite_sources().next().unwrap();
        assert_eq!(
            source.archive,
            super::super::resources::SharedArchiveKind::Sprite
        );
        assert_eq!(source.name, "3104-0.spr");
        assert_eq!(source.bytes, 18_170);
        assert_eq!(
            source
                .sha256
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
            "3f90a09a0e47ec284b0069f31f39f4d1690bad9bda5287c2bcab3f119bae1819"
        );
        // Independent Lin.Helper.Core/SprReader PNG decode (R228), including
        // the terminal subframe5. Straight RGBA bytes, not PNG compression.
        // Bounds were also checked from signed source block coordinates.
        let goldens = [
            (
                (-24, -84, 120, 72),
                "501373F00DCFB47962F4D73F23F87B259AD0EED53F783883142F63F72F24D994",
            ),
            (
                (-24, -96, 96, 84),
                "1103806D2610832CDD5FB881580FCA92F66CEE7987EFB064F1BC4326719E6170",
            ),
            (
                (-24, -84, 96, 72),
                "6565808703B33086A9E8C4023500CEF08BCFDA8E4FD8556F880E5F896035A065",
            ),
            (
                (-24, -84, 120, 72),
                "ED656F9DA25F686C9B9EAD82325992EA51C3692BC40B62A86805DE582A7AF5D8",
            ),
            (
                (-24, -84, 120, 72),
                "21E334B9F5BB4829098CF7EBDD2824D3FA498F08228EAD57233237C13AC7DFDE",
            ),
            (
                (-24, -84, 120, 72),
                "B84B92CC037CE079EA1BC9EBC4891BA97A6F6B7EAFB2F93197C07B1A0DE17975",
            ),
        ];
        let frames = assets.sprite_frames("3104-0.spr").unwrap();
        assert_eq!(frames.len(), goldens.len());
        for (index, (frame, (bounds, rgba_sha))) in frames.iter().zip(goldens).enumerate() {
            assert_eq!(
                (frame.x_offset, frame.y_offset, frame.width, frame.height),
                bounds,
                "C# frame bounds {index}"
            );
            assert_eq!(
                format!("{:X}", Sha256::digest(&frame.rgba)),
                rgba_sha,
                "C# RGBA frame {index}"
            );
        }
        eprintln!(
            "GREATER_HASTE sources={:?} encoded={} rgba={}",
            assets.sprite_sources().collect::<Vec<_>>(),
            assets.encoded_bytes(),
            assets.rgba_bytes()
        );
        let mut changed = (false, false);
        let mut cursor = assets.start_effect();
        for source in 1u64..=5 {
            let elapsed = Duration::from_nanos(((source - 1) * 5 * 1_000_000_000).div_ceil(24));
            let sample = assets.effect_frame(elapsed).unwrap();
            assert_eq!(sample.graphics_id, GRAPHIC);
            assert_eq!(sample.sprite_name, "3104-0.spr");
            assert_eq!(sample.source_frame, source as usize);
            assert_eq!(sample.subframe, source as usize - 1);
            assert!(!sample.finished);
            let anchor = PixelPoint { x: 400, y: 350 };
            let result = screen_test_support::compare(sample.frame, anchor, |canvas| {
                assets.draw_effect(canvas, anchor, elapsed).unwrap()
            });
            changed.0 |= result.0;
            changed.1 |= result.1;
            assert!(cursor.advance(elapsed).unwrap().crossed_cues.is_empty());
        }
        assert_eq!(changed, (true, true));
        assert!(assets.effect_frame(assets.duration()).unwrap().finished);
        screen_test_support::retired(|canvas| {
            assets
                .draw_effect(canvas, PixelPoint { x: 400, y: 350 }, assets.duration())
                .unwrap()
        });
        assert!(cursor
            .advance(Duration::MAX)
            .unwrap()
            .crossed_cues
            .is_empty());
        let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
        assert!(matches!(
            GreaterHasteAssets::load_with_catalog(&catalog, &AtomicBool::new(true)),
            Err(GreaterHasteAssetError::Cancelled)
        ));
    }
}
