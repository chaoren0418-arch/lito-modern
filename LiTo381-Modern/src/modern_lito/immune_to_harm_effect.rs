//! Shared skill68 effect228, not the separately configured armor-proc GFX11101.
//! The configured-duration native proc sends the skill GFX but no caster action;
//! do not interrupt an attack with an invented action19 or spend manual-cast MP.
//! Resource qualification alone grants no buff, command, protocol or equipment.
use super::{
    combat_animation::{NativeAudioGroup, NativeAudioMode},
    combat_audio::SoundBank,
    compositor::Canvas,
    shared_action_catalog::SharedActionCatalog,
    sprite::SpriteFrame,
    sprite_actions::{ActionKey, Animation, Directionality, FrameEvent, SpriteActionCatalog},
    teleport_effect::{draw_screen_frame, SpriteProvenance, VisualSample},
    tower_assets::{active, HonorAssetError, HonorPlayback, OneShot, Result, SharedEffectBank},
    world_map::PixelPoint,
};
use std::{
    path::Path,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

pub const GRAPHIC: u32 = 228;
pub const SOUND: u32 = 132;
const DURATIONS: [u32; 7] = [4, 2, 2, 4, 4, 2, 2];

fn qualify(catalog: &SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>> {
    let sprite = catalog
        .sprites
        .get(&GRAPHIC)
        .ok_or(HonorAssetError::Unsupported("missing shared protection228"))?;
    let resolved = catalog.resolve_action(GRAPHIC, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != GRAPHIC
        || sprite.image_count != Some(1)
        || sprite.actions.len() != 1
        || resolved.definition_sprite_id != GRAPHIC
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != 7
        || sprite.attributes.len() != 1
        || sprite.attributes[0].key != ActionKey::new(104)
        || sprite.attributes[0].body_bytes != b"8"
    {
        return Err(HonorAssetError::Unsupported(
            "unreviewed protection controls",
        ));
    }
    for (index, frame) in animation.frames.iter().enumerate() {
        let events = if index == 1 {
            vec![FrameEvent::Sound(SOUND)]
        } else {
            Vec::new()
        };
        if frame.image_id != 0
            || frame.subframe != index as u32
            || frame.duration_ticks != DURATIONS[index]
            || frame.events != events
        {
            return Err(HonorAssetError::Unsupported(
                "unreviewed protection frame or cue",
            ));
        }
    }
    Ok(vec![(GRAPHIC, 0, animation.clone())])
}

pub struct ImmuneToHarmAssets {
    bank: SharedEffectBank,
    sequence: OneShot,
}
impl ImmuneToHarmAssets {
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
        if bank.groups().len() != 1
            || bank.groups()[0].mode != NativeAudioMode::SuppressWhileSameIdPlaying
            || bank.sound_bank().len() != 1
            || bank.sound_bank().get(SOUND).is_none()
        {
            return Err(HonorAssetError::Unsupported("unreviewed protection audio"));
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
    /// Anchor is the confirmed event position; later actor motion cannot drag
    /// an old one-shot. No archive or file I/O happens on this render path.
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
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.bank.sources()
    }
    pub fn sprite_frames(&self, name: &str) -> Result<&[SpriteFrame]> {
        self.bank.frames(name)
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.bank.catalog_sha()
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
        SpriteActionCatalog::parse_for_inspection(b"1 0 0\n#228 1 protection\n0.breath(0 7,0.0:4 0.1:2[132 0.2:2 0.3:4 0.4:4 0.5:2 0.6:2)104.attr(8)\n",Default::default()).unwrap()
    }
    fn animation(catalog: &mut SpriteActionCatalog) -> &mut Animation {
        let ActionDefinition::Animation(value) = catalog
            .sprites
            .get_mut(&GRAPHIC)
            .unwrap()
            .actions
            .get_mut(&ActionKey::new(0))
            .unwrap()
        else {
            panic!("fixture")
        };
        value
    }
    #[test]
    fn exact_source_order_tick_boundaries_terminal_and_once_only_sound() {
        let defs = qualify(&catalog()).unwrap();
        let sequence = OneShot::new(Arc::new(defs[0].2.clone())).unwrap();
        assert_eq!(sequence.duration(), Duration::from_nanos(666_666_667));
        let mut cursor = sequence.start();
        let first = cursor.advance(Duration::ZERO).unwrap();
        assert_eq!(first.source_frame, 1);
        assert_eq!(first.crossed_cues.len(), 1);
        assert!(cursor
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        for (index, tick) in [2u64, 4, 8, 12, 14, 16].into_iter().enumerate() {
            let nanos = (tick * 1_000_000_000).div_ceil(24);
            assert_eq!(
                sequence.sample(Duration::from_nanos(nanos - 1)).unwrap(),
                (index + 1, false)
            );
            assert_eq!(
                sequence.sample(Duration::from_nanos(nanos)).unwrap(),
                if index == 5 {
                    (0, true)
                } else {
                    (index + 2, false)
                }
            );
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
        // Slow rendering still crosses the real cue once, not once per frame.
        let mut slow = sequence.start();
        assert_eq!(
            slow.advance(Duration::from_secs(2))
                .unwrap()
                .crossed_cues
                .len(),
            1
        );
        assert!(slow
            .advance(Duration::from_secs(3))
            .unwrap()
            .crossed_cues
            .is_empty());
    }
    #[test]
    fn unknown_controls_frames_aliases_or_added_cues_cannot_be_silently_accepted() {
        for edit in 0..8 {
            let mut c = catalog();
            match edit {
                0 => c.sprites.get_mut(&GRAPHIC).unwrap().graphics_id = 11101,
                1 => c.sprites.get_mut(&GRAPHIC).unwrap().image_count = Some(2),
                2 => c.sprites.get_mut(&GRAPHIC).unwrap().attributes[0].body_bytes = b"9".to_vec(),
                3 => animation(&mut c).directionality = Directionality::EightWay,
                4 => animation(&mut c).frames_per_second = 30,
                5 => {
                    animation(&mut c).frames.pop();
                }
                6 => animation(&mut c).frames[2].subframe = 10,
                _ => animation(&mut c).frames[2].duration_ticks = 3,
            }
            assert!(qualify(&c).is_err(), "edit {edit}");
        }
        for index in 0..7 {
            for event in [
                FrameEvent::Sound(132),
                FrameEvent::Hit,
                FrameEvent::Overlay(228),
                FrameEvent::Effect(11101),
                FrameEvent::Skip,
            ] {
                let mut c = catalog();
                animation(&mut c).frames[index].events.push(event);
                assert!(qualify(&c).is_err());
            }
        }
        let mut c = catalog();
        animation(&mut c).frames[1].events.clear();
        assert!(qualify(&c).is_err());
    }
    #[test]
    fn cancellation_precedes_any_shared_file_access() {
        assert!(matches!(
            ImmuneToHarmAssets::load(Path::new("not-a-resource-root"), &AtomicBool::new(true)),
            Err(HonorAssetError::Cancelled)
        ));
    }
    #[test]
    #[ignore = "explicit shared root; actual228 SPR and132 WAV; no window/audio device"]
    fn actual_shared_protection228_frames_audio_and_fixed_anchor_render() {
        use super::super::{resources::SharedArchiveKind, teleport_effect::screen_test_support};
        use sha2::{Digest, Sha256};
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
        );
        assert!(root.is_absolute());
        let assets = ImmuneToHarmAssets::load(&root, &AtomicBool::new(false)).unwrap();
        assert_eq!(assets.sprite_sources().len(), 1);
        assert_eq!(assets.encoded_bytes(), 99_139);
        assert_eq!(assets.sprite_frames("228-0.spr").unwrap().len(), 7);
        assert_eq!(assets.sound_bank().len(), 1);
        assert_eq!(
            assets
                .sound_bank()
                .get(SOUND)
                .unwrap()
                .sha256()
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<String>(),
            "E5E06BD3E3735737C85268BAF20EF15F3FFD2D2743606B59D89C941F01441F02"
        );
        assert_eq!(assets.rgba_bytes(), 663_206);
        assert_eq!(
            assets
                .catalog_sha256()
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<String>(),
            "523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127"
        );
        let source = assets.sprite_sources().next().unwrap();
        assert_eq!(source.archive, SharedArchiveKind::Sprite);
        assert_eq!(source.name, "228-0.spr");
        assert_eq!(source.bytes, 99_139);
        assert_eq!(
            source
                .sha256
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<String>(),
            "EAF8BEB2CD7BF59D815D75D1B4FDF05924FA940936025045F1FA50CF55CAFA48"
        );
        assert_eq!(
            assets.sound_groups()[0].mode,
            NativeAudioMode::SuppressWhileSameIdPlaying
        );
        // Independent Lin.Helper.Core/SprReader.LoadRaw (R328), straight RGBA
        // including terminal subframe0. Do not regenerate these using this decoder.
        let goldens = [
            (
                (0, -360, 48, 288),
                "6C137F09C9E2159E92BD20F89739F7B8107805B081A9E9DF252436D02884508B",
            ),
            (
                (0, -360, 48, 384),
                "D2651F39899968A8B3DB8330D5225BE7EBB700E6CFA2E709227AB77B65A2ADFB",
            ),
            (
                (0, -336, 48, 360),
                "F1AD07C9D38EED9EE990793D3E11AD894522FE2808C054AD5F54B656284D45B6",
            ),
            (
                (0, -360, 48, 396),
                "D4BBCEBAF6F3396453575938E8E96DAC35E8B3ECCFF0238627559A16903BEB96",
            ),
            (
                (0, -360, 48, 408),
                "92FE8AFD8DFC0ACC864B9D407B5799213FAC0223B0F09CA54FD220EEAF31BF69",
            ),
            (
                (0, -360, 48, 408),
                "7103C3EE3C1280953A5B316D69595A9115B0DD81A43FACB93DDC95851C2DD513",
            ),
            (
                (0, -288, 48, 312),
                "76E0E2BF6478E5E61877B9FAEE6EEAF6B8A3E6F87F94442AD704C7ED9D58FB46",
            ),
        ];
        for (index, (frame, (bounds, rgba_sha))) in assets
            .sprite_frames("228-0.spr")
            .unwrap()
            .iter()
            .zip(goldens)
            .enumerate()
        {
            assert_eq!(
                (frame.x_offset, frame.y_offset, frame.width, frame.height),
                bounds,
                "C# bounds {index}"
            );
            assert_eq!(
                format!("{:X}", Sha256::digest(&frame.rgba)),
                rgba_sha,
                "C# RGBA {index}"
            );
        }
        let mut changed = (false, false);
        let mut playback = assets.start_effect();
        for (index, tick) in [0u64, 2, 4, 8, 12, 14].into_iter().enumerate() {
            let elapsed = Duration::from_nanos((tick * 1_000_000_000).div_ceil(24));
            let sample = assets.effect_frame(elapsed).unwrap();
            assert_eq!(sample.graphics_id, GRAPHIC);
            assert_eq!(sample.sprite_name, "228-0.spr");
            assert_eq!(sample.source_frame, index + 1);
            assert_eq!(sample.subframe, index + 1);
            assert!(!sample.finished);
            for anchor in [
                PixelPoint { x: 400, y: 400 },
                PixelPoint { x: -20, y: 200 },
                PixelPoint { x: 780, y: 570 },
            ] {
                let result = screen_test_support::compare(sample.frame, anchor, |canvas| {
                    assets.draw_effect(canvas, anchor, elapsed).unwrap()
                });
                changed.0 |= result.0;
                changed.1 |= result.1;
            }
            assert_eq!(
                playback.advance(elapsed).unwrap().crossed_cues.len(),
                usize::from(index == 0)
            );
        }
        assert_eq!(changed, (true, true));
        assert!(assets.effect_frame(assets.duration()).unwrap().finished);
        screen_test_support::retired(|canvas| {
            assets
                .draw_effect(canvas, PixelPoint { x: 400, y: 400 }, assets.duration())
                .unwrap()
        });
        assert!(playback
            .advance(Duration::MAX)
            .unwrap()
            .crossed_cues
            .is_empty());
        println!(
            "PROTECTION228_OK sources=1 sound=132 durationNanos={} rgbaBytes={}",
            assets.duration().as_nanos(),
            assets.rgba_bytes()
        );
    }
}
