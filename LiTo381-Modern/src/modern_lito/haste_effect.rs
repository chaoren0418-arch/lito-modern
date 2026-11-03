//! Qualified shared Haste (skill43/action19/effect755) presentation.
//!
//! The inspected shared list.spr defines #755 as one undirected action0:
//! six source frames at24FPS, subframes5,0,1,2,3,4 with five ticks each,
//! attr104=8 and NO events. The established one-shot clock presents1..5 then
//! terminal0 (25ticks). The effect expires; it is not a 1200-second decal.
//! Caster action19, target admission and the server's timed status are separate responsibilities.
//! No invented potion sound, private resource, cast permission or speed change.
//!
//! Worker-only, cancellable, all-or-nothing SharedEffectBank loading; its
//! existing catalog/index/encoded/RGBA bounds and native shard routing apply.
//! No file handles or I/O survive into rendering. This resource bank alone
//! does not enable skill43 in a catalog, protocol, scene or player command.

pub use super::teleport_effect::{SpriteProvenance, VisualSample};
pub use super::tower_assets::{HonorAssetError as HasteAssetError, HonorPlayback};
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

pub const SKILL_ID: u32 = 43;
pub const GRAPHIC: u32 = 755;

fn qualify(catalog: &SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>> {
    let sprite = catalog
        .sprites
        .get(&GRAPHIC)
        .ok_or(HasteAssetError::Unsupported("missing shared Haste755"))?;
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
        return Err(HasteAssetError::Unsupported("unreviewed Haste controls"));
    }
    for (index, frame) in animation.frames.iter().enumerate() {
        if frame.image_id != 0
            || frame.subframe != if index == 0 { 5 } else { index as u32 - 1 }
            || frame.duration_ticks != 5
            || !frame.events.is_empty()
        {
            return Err(HasteAssetError::Unsupported("unreviewed Haste frame/cue"));
        }
    }
    Ok(vec![(GRAPHIC, 0, animation.clone())])
}

pub struct HasteAssets {
    bank: SharedEffectBank,
    sequence: OneShot,
}
impl HasteAssets {
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
            return Err(HasteAssetError::Unsupported("unexpected Haste audio"));
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
    /// Fixed confirmed target-event anchor; later movement must not drag the old effect.
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
        draw_screen_frame(canvas, sample.frame, anchor).map_err(HasteAssetError::Render)?;
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
            b"1 0 0\n#755 1 haste\n0.fire(0 6,0.5:5 0.0:5 0.1:5 0.2:5 0.3:5 0.4:5)104.attr(8)\n",
            Default::default(),
        )
        .unwrap()
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
            FrameEvent::Overlay(755),
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
            HasteAssets::load(Path::new("absent"), &AtomicBool::new(true)),
            Err(HasteAssetError::Cancelled)
        ));
    }
    #[test]
    #[ignore = "explicit shared root; actual skill43 SPR, fixed screen projection, no HWND or audio device"]
    fn real_shared_haste43_effect_frames_and_terminal() {
        use sha2::{Digest, Sha256};

        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
        );
        assert!(root.is_absolute());
        let cancel = AtomicBool::new(false);
        let assets = HasteAssets::load(&root, &cancel).unwrap();
        assert_eq!(assets.sprite_sources().len(), 1);
        assert_eq!(assets.encoded_bytes(), 17_762);
        assert!(assets.rgba_bytes() >= 203_904 && assets.rgba_bytes() < 512_000);
        let source = assets.sprite_sources().next().unwrap();
        assert_eq!(
            source.archive,
            super::super::resources::SharedArchiveKind::Sprite
        );
        assert_eq!(source.name, "755-0.spr");
        assert_eq!(source.bytes, 17_762);
        assert_eq!(
            source
                .sha256
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
            "1c53743466461dee1bda98151aae4ff78adba979c7616437031d5f894e193996"
        );
        // Independent Lin.Helper.Core/SprReader.LoadRaw decode (R252), including
        // the terminal subframe5. Straight RGBA bytes, not PNG compression.
        // Bounds were also checked from signed source block coordinates.
        let goldens = [
            (
                (-24, -84, 120, 72),
                "77AACB5AAF7C05B2B8E32FD1CD412F18C41ABA87B45F07EAA03C5E96CA99302D",
            ),
            (
                (-24, -96, 96, 84),
                "B56B24420F06794018D97F3A1B2488933007FB694B05235EB90B46F17783EBEE",
            ),
            (
                (-24, -84, 96, 72),
                "BDF06CD2F2236ADFAAB873C662867FBEFAC5AEA4CD2DBE15F554F72FB95B1BDF",
            ),
            (
                (-24, -84, 120, 72),
                "71F7890BCEC617B4D545C7783D67FEC5A69E5F30D9261D050B5C3BE8CA142E2D",
            ),
            (
                (-24, -84, 120, 72),
                "8BAD0FF370A1165AE0B9CAB9974C4918A9EADB118E48437C7296C79E15BD530F",
            ),
            (
                (-24, -96, 120, 84),
                "083797805F0A4B8EAAF59D0F6CEBECD85271B7537768FF499D4BBA8FE1F16B82",
            ),
        ];
        let frames = assets.sprite_frames("755-0.spr").unwrap();
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
            "HASTE43 sources={:?} encoded={} rgba={}",
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
            assert_eq!(sample.sprite_name, "755-0.spr");
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
            HasteAssets::load_with_catalog(&catalog, &AtomicBool::new(true)),
            Err(HasteAssetError::Cancelled)
        ));
    }
}
