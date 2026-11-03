//! FA clan flag / original 7550..7556 IMG overlay. Snapshot insertion order,
//! interpolated body anchors, hidden-body suppression and clock epoch resets
//! are explicit modern policies; not a reconstruction of the old render loop.
use super::{
    compositor::Canvas,
    native_offline_menu_skin::{decode_img, CopyMode, Img},
    native_resources::NativeSurfaceArchive,
    native_world::World,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    world_map::PixelPoint,
};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "clan marker cancelled");
    Ok(())
}
pub struct Bank {
    pub images: Vec<Img>,
    pub sources: Value,
    /// Diagnostic wall times, excluded from immutable source identity.
    pub load_profile_us: Value,
}
fn contains(a: &SharedArchive, name: &str) -> Result<bool> {
    match a.entry(name) {
        Ok(_) => Ok(true),
        Err(ResourceError::MissingEntry(_)) => Ok(false),
        Err(e) => Err(e.into()),
    }
}
impl Bank {
    /// Fixed qualified originals. A language or shard override is rejected
    /// rather than silently using the wrong resource namespace.
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let started = Instant::now();
        let lt = root.join("LiTo");
        let surface_start = Instant::now();
        let (mut surface, idx, pak) = NativeSurfaceArchive::observe(&lt, cancel)?;
        let surface_us = surface_start.elapsed().as_micros();
        let names: Vec<_> = (7550..=7556)
            .flat_map(|id| ["c", "h", "j", "e", ""].map(move |s| format!("{id}{s}.img")))
            .collect();
        let base_names: Vec<_> = names.iter().map(String::as_str).collect();
        let base_start = Instant::now();
        let base =
            SharedArchive::open_for_names(root, SharedArchiveKind::Sprite, &base_names, cancel)?;
        let base_us = base_start.elapsed().as_micros();
        let mut shard_us = 0;
        let mut shards: BTreeMap<u8, SharedArchive> = BTreeMap::new();
        let mut images = Vec::new();
        let mut sources = Vec::new();
        for id in 7550..=7556 {
            for suffix in ["c", "h", "j", "e", ""] {
                active(cancel)?;
                let name = format!("{id}{suffix}.img");
                let shard = (name.bytes().map(u32::from).sum::<u32>() % 16) as u8;
                if !shards.contains_key(&shard) {
                    let shard_start = Instant::now();
                    let selected: Vec<_> = names
                        .iter()
                        .filter(|n| (n.bytes().map(u32::from).sum::<u32>() % 16) as u8 == shard)
                        .map(String::as_str)
                        .collect();
                    shards.insert(
                        shard,
                        SharedArchive::open_for_names(
                            root,
                            SharedArchiveKind::SpriteShard(shard),
                            &selected,
                            cancel,
                        )?,
                    );
                    shard_us += shard_start.elapsed().as_micros();
                }
                ensure!(
                    !contains(&shards[&shard], &name)?,
                    "clan marker shard override requires qualification: {name}"
                );
                if !suffix.is_empty() {
                    ensure!(
                        !surface.contains(&name)? && !contains(&base, &name)?,
                        "localized clan marker requires qualification: {name}"
                    );
                    continue;
                }
                let bytes = surface.read(&name, 872, cancel)?;
                let img = decode_img(&bytes, CopyMode::ColorKey)?;
                ensure!(
                    img.width == 24 && img.height == 18,
                    "qualified clan marker dimensions"
                );
                sources.push(json!({"archive":"LiTo/LTsurf","member":name,"bytes":bytes.len(),"sha256":format!("{:X}",Sha256::digest(&bytes)),"rgba_sha256":format!("{:X}",Sha256::digest(&img.rgba))}));
                images.push(img);
            }
        }
        active(cancel)?;
        let pin = |p: super::native_resources::BytePin| json!({"bytes":p.bytes,"sha256":p.sha256.iter().map(|b|format!("{b:02X}")).collect::<String>()});
        Ok(Self {
            images,
            sources: json!({"index":pin(idx),"pak":pin(pak),"members":sources}),
            load_profile_us: json!({"surface_with_pin":surface_us,"base_index":base_us,"shard_indexes":shard_us,"total":started.elapsed().as_micros()}),
        })
    }
    pub fn blit(&self, canvas: &mut Canvas, phase: u8, anchor: PixelPoint) -> Result<()> {
        let image = self
            .images
            .get(usize::from(phase))
            .context("marker phase outside 0..6")?;
        canvas.blit_rgba(&image.rgba, image.width, image.height, anchor.x, anchor.y)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Clock {
    scope: Option<(u64, Option<u16>, u32, u64)>,
    last_ms: u64,
    phase: u8,
    accumulated_ms: u64,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Marker {
    pub id: u32,
    pub phase: u8,
    pub anchor: Option<[i32; 2]>,
}
impl Clock {
    /// Draw-before-advance, one shared accumulator, no remainder/catch-up.
    /// All active remote actors consume delta, even when offscreen/hidden.
    pub fn sample(
        &mut self,
        world: &World,
        centers: &BTreeMap<u32, PixelPoint>,
        elapsed_ns: u64,
    ) -> Vec<Marker> {
        let own = match world.own_actor() {
            Some(a) => a,
            None => {
                *self = Self::default();
                return Vec::new();
            }
        };
        let scope = (world.generation, world.map_id, own.id, own.incarnation);
        let ms = elapsed_ns / 1_000_000;
        let delta = if self.scope != Some(scope) || ms < self.last_ms {
            *self = Self {
                scope: Some(scope),
                last_ms: ms,
                ..Default::default()
            };
            0
        } else {
            (ms - self.last_ms).min(500)
        };
        self.last_ms = ms;
        let mut actors: Vec<_> = world
            .actors
            .values()
            .filter(|a| {
                a.clan_marker_active && a.id != own.id && !a.name.eq_ignore_ascii_case(&own.name)
            })
            .collect();
        actors.sort_by_key(|a| (a.incarnation, a.id));
        let mut rows = Vec::with_capacity(actors.len());
        for actor in actors {
            let anchor = centers
                .get(&actor.id)
                .and_then(|p| Some([p.x.checked_add(13)?, p.y.checked_sub(62)?]));
            rows.push(Marker {
                id: actor.id,
                phase: self.phase,
                anchor,
            });
            self.accumulated_ms += delta;
            if self.accumulated_ms >= 350 {
                self.accumulated_ms = 0;
                self.phase = (self.phase + 1) % 7;
            }
        }
        rows
    }
}
#[derive(Default)]
pub struct Runtime {
    clock: Clock,
    bank: Option<std::result::Result<Bank, String>>,
}
impl Runtime {
    pub fn draw(
        &mut self,
        root: &Path,
        world: &World,
        centers: &BTreeMap<u32, PixelPoint>,
        canvas: &mut Canvas,
        elapsed_ns: u64,
        cancel: &AtomicBool,
    ) -> Result<Value> {
        active(cancel)?;
        let mut clock = self.clock.clone();
        let markers = clock.sample(world, centers, elapsed_ns);
        let mut rows = Vec::new();
        let mut load_profile = None;
        for marker in markers {
            active(cancel)?;
            let eligible = marker.anchor.is_some_and(|[x, y]| {
                x > 0
                    && y > 0
                    && i64::from(x) < i64::from(canvas.width())
                    && i64::from(y) < i64::from(canvas.height())
            });
            let mut row = json!({"marker":marker,"drawn":false});
            if eligible {
                if self.bank.is_none() {
                    let loaded = Bank::load(root, cancel).map_err(|e| format!("{e:#}"));
                    active(cancel)?;
                    load_profile = loaded.as_ref().ok().map(|b| b.load_profile_us.clone());
                    self.bank = Some(loaded);
                }
                active(cancel)?;
                match self.bank.as_ref().unwrap() {
                    Ok(bank) => {
                        let [x, y] = marker.anchor.unwrap();
                        bank.blit(canvas, marker.phase, PixelPoint { x, y })?;
                        row["drawn"] = json!(true);
                    }
                    Err(error) => row["error"] = json!(error),
                }
            }
            rows.push(row);
        }
        active(cancel)?;
        self.clock = clock;
        Ok(
            json!({"rows":rows,"phase":self.clock.phase,"accumulated_ms":self.clock.accumulated_ms,
            "sources":self.bank.as_ref().and_then(|r|r.as_ref().ok()).map(|b|&b.sources),
            "load_profile_us":load_profile,
            "ownership":"click_through","shared_clock":true,"full_native_equivalence":false,
            "modern_policies":["snapshot_insertion_order","interpolated_body_anchor","hidden_body_suppression","800x600_viewport","epoch_reset_and_monotonic_render_delta","ascii_casefold_own_name"]}),
        )
    }
}
