//! Shared skill29 effect752: inspected action0 has 16 frames at24FPS,
//! subframes7,0..7,0..6, each3ticks; terminal0 is not drawn. No audio cues.
//! Only shared Lineage archives are read, on the cancellable asset worker.
pub use super::teleport_effect::{SpriteProvenance, VisualSample};
pub use super::tower_assets::{HonorAssetError as SlowAssetError, HonorPlayback};
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

pub const SKILL_ID: u32 = 29;
pub const GRAPHIC: u32 = 752;

fn qualify(catalog: &SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>> {
    let sprite = catalog
        .sprites
        .get(&GRAPHIC)
        .ok_or(SlowAssetError::Unsupported("missing shared Slow752"))?;
    let resolved = catalog.resolve_action(GRAPHIC, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != GRAPHIC
        || sprite.image_count != Some(1)
        || sprite.actions.len() != 1
        || resolved.definition_sprite_id != GRAPHIC
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != 16
        || sprite.attributes.len() != 1
        || sprite.attributes[0].key != ActionKey::new(104)
        || sprite.attributes[0].body_bytes != b"8"
    {
        return Err(SlowAssetError::Unsupported("unreviewed Slow controls"));
    }
    for (index, frame) in animation.frames.iter().enumerate() {
        if frame.image_id != 0
            || frame.subframe != (index as u32 + 7) % 8
            || frame.duration_ticks != 3
            || !frame.events.is_empty()
        {
            return Err(SlowAssetError::Unsupported("unreviewed Slow frame/cue"));
        }
    }
    Ok(vec![(GRAPHIC, 0, animation.clone())])
}

pub struct SlowAssets {
    bank: SharedEffectBank,
    sequence: OneShot,
}
impl SlowAssets {
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
            return Err(SlowAssetError::Unsupported("unexpected Slow audio"));
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
        draw_screen_frame(canvas, sample.frame, anchor).map_err(SlowAssetError::Render)?;
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
    use super::*;
    fn catalog() -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(b"1 0 0\n#752 1 slow\n0.fire(0 16,0.7:3 0.0:3 0.1:3 0.2:3 0.3:3 0.4:3 0.5:3 0.6:3 0.7:3 0.0:3 0.1:3 0.2:3 0.3:3 0.4:3 0.5:3 0.6:3)104.attr(8)\n",Default::default()).unwrap()
    }
    #[test]
    fn exact_shared_sequence_and_terminal_never_loop_forever() {
        let def = qualify(&catalog()).unwrap();
        let sequence = OneShot::new(Arc::new(def[0].2.clone())).unwrap();
        assert_eq!(sequence.duration(), Duration::from_millis(1875));
        for step in 0..15 {
            assert_eq!(
                sequence.sample(Duration::from_millis(step * 125)).unwrap(),
                (step as usize + 1, false)
            );
            assert_eq!(
                sequence
                    .sample(Duration::from_millis((step + 1) * 125) - Duration::from_nanos(1))
                    .unwrap(),
                (step as usize + 1, false)
            );
        }
        assert_eq!(sequence.sample(sequence.duration()).unwrap(), (0, true));
        assert_eq!(sequence.sample(Duration::MAX).unwrap(), (0, true));
    }
    #[test]
    fn unknown_shape_cues_and_cancel_fail_closed() {
        use super::super::sprite_actions::{ActionDefinition, FrameEvent};
        for case in 0..7 {
            let mut cat = catalog();
            let s = cat.sprites.get_mut(&GRAPHIC).unwrap();
            let ActionDefinition::Animation(a) = s.actions.get_mut(&ActionKey::new(0)).unwrap()
            else {
                panic!()
            };
            match case {
                0 => a.frames[2].duration_ticks = 4,
                1 => a.frames[2].subframe = 9,
                2 => a.frames[2].image_id = 1,
                3 => a.frames[2].events.push(FrameEvent::Sound(121)),
                4 => a.frames_per_second = 30,
                5 => s.image_count = Some(2),
                _ => s.attributes[0].body_bytes = b"9".to_vec(),
            }
            assert!(qualify(&cat).is_err());
        }
        assert!(matches!(
            SlowAssets::load(Path::new("absent"), &AtomicBool::new(true)),
            Err(SlowAssetError::Cancelled)
        ));
    }
    #[test]
    #[ignore = "explicit shared root, independent C# raw RGBA goldens, no HWND"]
    fn real_shared_slow29_effect_frames_and_terminal() {
        use super::super::teleport_effect::screen_test_support;
        use sha2::{Digest, Sha256};
        let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
        assert!(root.is_absolute());
        let assets = SlowAssets::load(&root, &AtomicBool::new(false)).unwrap();
        assert_eq!(assets.encoded_bytes(), 36873);
        assert_eq!(assets.sprite_sources().len(), 1);
        let src = assets.sprite_sources().next().unwrap();
        assert_eq!(src.name, "752-0.spr");
        assert_eq!(
            src.archive,
            super::super::resources::SharedArchiveKind::Sprite
        );
        assert_eq!(
            src.sha256
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<String>(),
            "422C997741240B21D980023CDEA270D38B648FA42C4B47D04425301FB0E21F22"
        );
        let goldens = [
            "5CEDD9680A2EBC2A43AB040FAF6F095F5248FB6C9066550687BB75D7EEB0F845",
            "875802B42B16E5D9B75A17C23DC2BCFE28A004AFA749825E9DA363F09132E8EB",
            "02D530ADB80B5A05F468FBBB212AFA0BB970CEF1092714741674F8BC282FA2E5",
            "3AAEA2EF748AB8304ED7B3B502206D6275C2F4AADD1793D774DBBB5D69B4F90E",
            "E730680474BB2BA201897E831A04D9DAA1F5CBF0D06178878A541CA4890D53CA",
            "9EF26866D3BAA5FA0C19DE83F279AC9F3DAD46BCCDCC80CFDF2FFD7BD86645A2",
            "019BF94B1566C23897C6B93F5299E76C1D06B6569ACDD82A7A846CDD535C6CC3",
            "0CFBEE5CF26D1EED9BD1869DFB7458C0285BFCC94BCF8C3A4DF3636EEA7C6596",
        ];
        let frames = assets.sprite_frames("752-0.spr").unwrap();
        assert_eq!(frames.len(), 8);
        for (f, h) in frames.iter().zip(goldens) {
            assert_eq!(
                (f.x_offset, f.y_offset, f.width, f.height),
                (-24, -48, 96, 84)
            );
            assert_eq!(format!("{:X}", Sha256::digest(&f.rgba)), h);
        }
        let mut cursor = assets.start_effect();
        let mut changed = (false, false);
        for n in 0..15u64 {
            let elapsed = Duration::from_millis(n * 125);
            let frame = assets.effect_frame(elapsed).unwrap();
            assert_eq!(frame.subframe, n as usize % 8);
            assert!(!frame.finished);
            let result =
                screen_test_support::compare(frame.frame, PixelPoint { x: 400, y: 350 }, |c| {
                    assets
                        .draw_effect(c, PixelPoint { x: 400, y: 350 }, elapsed)
                        .unwrap()
                });
            changed.0 |= result.0;
            changed.1 |= result.1;
            assert!(cursor.advance(elapsed).unwrap().crossed_cues.is_empty());
        }
        assert_eq!(changed, (true, true));
        assert!(assets.effect_frame(assets.duration()).unwrap().finished);
        screen_test_support::retired(|c| {
            assets
                .draw_effect(c, PixelPoint { x: 400, y: 350 }, assets.duration())
                .unwrap()
        });
    }
}
