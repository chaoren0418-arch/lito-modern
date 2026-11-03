//! Owned, preloaded shared-resource scene presentation. No native BIN, SQL,
//! network connection or disk reads are needed after `load` completes.
//!
//! The local actor anchor is an inspection placement, not a spawn point or a
//! collision/occlusion implementation. Server-authoritative movement follows.

use super::{
    actor_assets::{
        preload_action, ActionFrameAssets, ActorActionRequest, ActorAssetLimits,
        SharedSpriteArchives, SpriteArchiveSelection,
    },
    audio::parse_pcm_wav,
    compositor::Canvas,
    map::parse_s32,
    presentation::{ActorPhase, ActorTimeline},
    resources::{
        read_shared_map_sector, read_shared_sound_with_limit, SharedArchive, SharedArchiveKind,
        SharedMapSectorKey, SharedSoundKey,
    },
    scene::{render_tiles, static_draw_list, CameraRect},
    sprite_actions::{ActionKey, ActionLimits, FrameEvent, SpriteActionCatalog},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

pub struct SceneModel {
    background: Canvas,
    canvas: Canvas,
    bgra: Vec<u8>,
    idle: ActionFrameAssets,
    action: ActionFrameAssets,
    timeline: ActorTimeline,
    sounds: BTreeMap<u32, Vec<u8>>,
    current: Option<(ActorPhase, usize, u128)>,
    anchor: (i32, i32),
    anchor_dirty: bool,
    pub graphics_id: u32,
    pub heading: u8,
    pub hit_cues: usize,
    pub sound_requests: usize,
    pub expired_sounds: usize,
}

pub struct SceneUpdate {
    pub changed: bool,
    pub sounds: Vec<u32>,
}

impl SceneModel {
    pub fn load(
        root: &Path,
        heading: u8,
        cancel: &AtomicBool,
        progress: impl Fn(&str),
    ) -> Result<Self> {
        ensure!(heading < 8, "heading must be 0..7");
        let check = || -> Result<()> {
            ensure!(!cancel.load(Ordering::Acquire), "scene loading cancelled");
            Ok(())
        };
        check()?;
        progress("讀取共享 Text/list.spr");
        let text = SharedArchive::open(root, SharedArchiveKind::Text)
            .context("open shared Text archive")?.read_entry("list.spr")
            .context("read shared Text/list.spr")?;
        check()?;
        let catalog = SpriteActionCatalog::parse_for_inspection(&text, ActionLimits::default())?;
        drop(text);
        check()?;
        progress("核對共享角色圖檔與方向");
        // This inspected actor is an explicit coherent Sprite/gfx3225 art set.
        // Some shard entries share its names but contain DIFFERENT versions;
        // never merge those by filesystem order. This is not a claim about
        // native archive priority or a resolver for arbitrary game actors.
        let mut archives = SharedSpriteArchives::open(
            root,
            SpriteArchiveSelection::Explicit(SharedArchiveKind::Sprite),
        ).context("open explicit Sprite archive")?;
        let limits = ActorAssetLimits {
            max_total_pixels: 4 * 1024 * 1024,
            ..ActorAssetLimits::default()
        };
        let idle = preload_action(
            &catalog,
            &mut archives,
            ActorActionRequest {
                sprite_id: 0,
                key: ActionKey::new(3),
                heading,
            },
            limits,
        )?;
        check()?;
        let action = preload_action(
            &catalog,
            &mut archives,
            ActorActionRequest {
                sprite_id: 0,
                key: ActionKey::new(1),
                heading,
            },
            limits,
        )?;
        drop(archives);
        drop(catalog);
        check()?;
        let mut sound_ids = BTreeSet::new();
        for clock in [&idle.clock, &action.clock] {
            for frame in &clock.animation().frames {
                for cue in &frame.events {
                    if let FrameEvent::Sound(id) = cue {
                        sound_ids.insert(*id);
                    }
                }
            }
        }
        ensure!(
            sound_ids.len() <= 16,
            "scene sound bank entry budget exceeded"
        );
        let timeline = ActorTimeline::new(idle.clock, action.clock)?;
        progress("讀取共享音效");
        let mut sounds = BTreeMap::new();
        let mut sound_bytes = 0usize;
        for sound_id in sound_ids {
            check()?;
            let bytes = read_shared_sound_with_limit(
                root,
                SharedSoundKey { sound_id },
                (4 * 1024 * 1024 - sound_bytes) as u64,
            ).with_context(|| format!("read Sound/{sound_id}.wav"))?;
            sound_bytes = sound_bytes
                .checked_add(bytes.len())
                .context("sound budget overflow")?;
            ensure!(
                sound_bytes <= 4 * 1024 * 1024,
                "scene sound bank exceeds 4 MiB"
            );
            parse_pcm_wav(&bytes).with_context(|| format!("decode shared Sound/{sound_id}.wav"))?;
            sounds.insert(sound_id, bytes);
        }
        progress("組合共享地面與靜態物件");
        let sector = parse_s32(&read_shared_map_sector(
            root,
            SharedMapSectorKey {
                map_id: 4,
                sector_x: 0x7ffc,
                sector_y: 0x7ff7,
            },
        ).context("read map/4/7ffc7ff7.s32")?)?;
        check()?;
        let draws = static_draw_list(&sector)?;
        let scene = render_tiles(
            &mut SharedArchive::open(root, SharedArchiveKind::Tile)?,
            &draws,
            CameraRect {
                x: 1120,
                y: 420,
                width: 800,
                height: 600,
            },
        )?;
        check()?;
        let mut model = Self {
            background: scene.canvas,
            canvas: Canvas::new(800, 600)?,
            bgra: vec![],
            idle: idle.assets,
            action: action.assets,
            timeline,
            sounds,
            current: None,
            anchor: (400, 330),
            anchor_dirty: false,
            graphics_id: idle.provenance.graphics_id,
            heading,
            hit_cues: 0,
            sound_requests: 0,
            expired_sounds: 0,
        };
        model.advance(Duration::ZERO)?;
        Ok(model)
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }
    pub fn bgra(&self) -> &[u8] {
        &self.bgra
    }
    pub fn sound(&self, id: u32) -> Result<&[u8]> {
        self.sounds
            .get(&id)
            .map(Vec::as_slice)
            .with_context(|| format!("sound {id} was not preloaded"))
    }
    pub fn trigger_action(&mut self, now: Duration) -> Result<()> {
        self.timeline.trigger_action(now)
    }
    /// Local resource placement only. This is not a world coordinate or a
    /// movement command; collision, pathfinding and server authority follow.
    pub fn place_actor(&mut self, x: i32, y: i32) -> Result<()> {
        ensure!((0..800).contains(&x) && (0..600).contains(&y), "placement outside canvas");
        self.anchor_dirty |= self.anchor != (x, y);
        self.anchor = (x, y);
        Ok(())
    }
    pub fn actor_anchor(&self) -> (i32, i32) { self.anchor }
    pub fn advance(&mut self, now: Duration) -> Result<SceneUpdate> {
        let frame = self.timeline.step(now)?;
        let current = (frame.phase, frame.source_frame, frame.idle_cycle);
        let pixels_changed = self.anchor_dirty || self.current.map(|(phase, index, _)| (phase, index))
            != Some((frame.phase, frame.source_frame));
        if pixels_changed {
            self.canvas.copy_from(&self.background)?;
            let assets = match frame.phase {
                ActorPhase::Idle => &self.idle,
                ActorPhase::Action => &self.action,
            };
            let sprite = assets.frame(frame.source_frame)?;
            self.canvas.blit_rgba(
                &sprite.rgba,
                sprite.width,
                sprite.height,
                self.anchor.0
                    .checked_add(sprite.x_offset)
                    .context("actor anchor overflow")?,
                self.anchor.1
                    .checked_add(sprite.y_offset)
                    .context("actor anchor overflow")?,
            )?;
            self.canvas.copy_bgra(&mut self.bgra)?;
            self.anchor_dirty = false;
        }
        let changed = pixels_changed || self.current != Some(current)
            || frame.hit_cues > 0
            || !frame.sounds.is_empty()
            || frame.expired_sounds > 0;
        self.current = Some(current);
        self.hit_cues = self
            .hit_cues
            .checked_add(frame.hit_cues)
            .context("hit cue count overflow")?;
        self.sound_requests = self
            .sound_requests
            .checked_add(frame.sounds.len())
            .context("sound count overflow")?;
        self.expired_sounds = self
            .expired_sounds
            .checked_add(frame.expired_sounds)
            .context("expired cue count overflow")?;
        Ok(SceneUpdate {
            changed,
            sounds: frame.sounds,
        })
    }
    pub fn status(&self) -> String {
        let (phase, frame, cycle) = self.current.unwrap_or((ActorPhase::Idle, 0, 0));
        format!(
            "共享場景驗證（非連線遊戲） | Sprite/gfx={} heading={} | {:?} frame={} cycle={} | sound={} hit-cue={} expired={}",
            self.graphics_id,
            self.heading,
            phase,
            frame,
            cycle,
            self.sound_requests,
            self.hit_cues,
            self.expired_sounds
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelled_load_does_not_open_a_resource_root() {
        assert!(SceneModel::load(
            Path::new("this-root-must-not-be-opened"),
            0,
            &AtomicBool::new(true),
            |_| {}
        )
        .is_err());
    }
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; deterministic scene/action integration, no audio device"]
    fn real_shared_scene_action_and_sound_are_preloaded_and_advance_without_io() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root");
        let mut model = SceneModel::load(Path::new(&root), 0, &AtomicBool::new(false), |stage| {
            eprintln!("{stage}")
        })
        .unwrap();
        assert_eq!(model.graphics_id, 3225);
        assert_eq!(
            parse_pcm_wav(model.sound(1539).unwrap())
                .unwrap()
                .frame_count(),
            9456
        );
        model.advance(Duration::from_secs(10)).unwrap();
        assert!(model.status().contains("Idle"));
        model.trigger_action(Duration::from_secs(10)).unwrap();
        model.advance(Duration::from_secs(10)).unwrap();
        let sound = model.advance(Duration::from_millis(10_375)).unwrap();
        assert_eq!(sound.sounds, [1539]);
        assert!(model
            .advance(Duration::from_millis(10_375))
            .unwrap()
            .sounds
            .is_empty());
        model.advance(Duration::from_millis(10_600)).unwrap();
        assert_eq!(model.hit_cues, 1);
        let directory =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("target/modern-lito-render-tests");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("shared-map4-actor-reference.png");
        let mut encoder = png::Encoder::new(std::fs::File::create(&path).unwrap(), 800, 600);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(model.canvas().rgba())
            .unwrap();
        model.advance(Duration::from_secs(60)).unwrap();
        assert!(model.status().contains("Idle"));
        assert_eq!(model.sound_requests, 1);
        eprintln!("{}; {}", model.status(), path.display());
    }
}
