//! Full pinned map projection with one immutable sector anchor. Camera is Modern
//! inspection. Native cell/scan/depth keys are reused; canonical sector then
//! serialized insertion is an explicit snapshot policy, not runtime load history.
use super::{
    compositor::Canvas,
    native_resources::NativeStaticAssets,
    scene::CameraRect,
    world_depth::{NativeDrawCell, NativeDrawKey},
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use anyhow::{ensure, Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
struct Draw {
    cell: NativeDrawCell,
    key: NativeDrawKey,
    tile: (u16, u8),
    depth: Option<u8>,
    source: u64,
    group: Option<super::world_roofs::Group>,
}
pub struct Scene {
    assets: NativeStaticAssets,
    anchor: WorldCell,
    bounds: Bounds,
    draws: Vec<Draw>,
    multi: bool,
    prepared: std::collections::BTreeMap<(u16, u8), super::world_raster::PreparedTile>,
    prepared_bytes: usize,
    roofs: super::native_world_roofs::Definition,
}
impl Scene {
    pub fn new(assets: NativeStaticAssets) -> Result<Self> {
        Self::new_cancel(assets, &AtomicBool::new(false))
    }
    pub fn new_cancel(assets: NativeStaticAssets, cancel: &AtomicBool) -> Result<Self> {
        ensure!(!cancel.load(Ordering::Acquire), "roof scene load cancelled");
        let roofs = super::native_world_roofs::Definition::from_map(assets.geometry())?;
        let mut raw_roofs = assets.scene_raw_sources(cancel)?;
        let bindings = assets.geometry().bindings();
        ensure!(!bindings.is_empty(), "whole map empty");
        let (ox, oy) = bindings[0].key.world_origin();
        let anchor = WorldCell { x: ox, y: oy };
        let mut bounds = Bounds {
            left: i32::MAX,
            top: i32::MAX,
            right: i32::MIN,
            bottom: i32::MIN,
        };
        for b in &bindings {
            let (x, y) = b.key.world_origin();
            let (dx, dy) = (i64::from(x) - i64::from(ox), i64::from(y) - i64::from(oy));
            let px = i32::try_from(24 * (dx + dy))?;
            let py = i32::try_from(12 * (-dx + dy))?;
            bounds.left = bounds.left.min(px);
            bounds.top = bounds.top.min(py);
            bounds.right = bounds
                .right
                .max(px.checked_add(3072).context("map X extent")?);
            bounds.bottom = bounds
                .bottom
                .max(py.checked_add(1536).context("map Y extent")?);
        }
        let mut draws = Vec::new();
        let mut insertion = 0u64;
        let terrain = assets.scene_terrain()?;
        let groups = assets.scene_groups();
        ensure!(terrain.len() == groups.len(), "roof source group alignment");
        for ((cell, depth, tile), group) in terrain.into_iter().zip(groups) {
            let key = match depth {
                None => cell.floor_key(),
                Some(d) => {
                    insertion = insertion
                        .checked_add(1)
                        .context("source insertion overflow")?;
                    cell.overlay_key(d, insertion)
                }
            };
            draws.push(Draw {
                cell,
                key,
                tile,
                depth,
                source: insertion,
                group,
            });
        }
        draws.sort_by_key(|d| d.key);
        let multi = bindings.len() > 1;
        let mut prepared = std::collections::BTreeMap::new();
        let mut prepared_bytes = 0usize;
        for (&key, tile) in assets.scene_tiles() {
            ensure!(!cancel.load(Ordering::Acquire), "roof scene load cancelled");
            let mut tile = super::world_raster::PreparedTile::new(tile.clone())?
                .with_object_header(assets.scene_source_header(key)?)?;
            if let Some(raw) = raw_roofs.remove(&key) {
                tile = tile.with_roof_raw(raw)?;
            }
            prepared_bytes = prepared_bytes
                .checked_add(tile.retained_bytes())
                .context("prepared terrain bytes")?;
            ensure!(prepared_bytes <= 64 << 20, "prepared terrain budget");
            prepared.insert(key, tile);
        }
        Ok(Self {
            assets,
            anchor,
            bounds,
            draws,
            multi,
            prepared,
            prepared_bytes,
            roofs,
        })
    }
    pub fn roofs(&self) -> &super::native_world_roofs::Definition {
        &self.roofs
    }
    pub fn assets(&self) -> &NativeStaticAssets {
        &self.assets
    }
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }
    pub fn anchor(&self) -> WorldCell {
        self.anchor
    }
    pub fn initial_camera(&self) -> CameraRect {
        CameraRect {
            x: ((i64::from(self.bounds.left) + i64::from(self.bounds.right) - 800) / 2) as i32,
            y: ((i64::from(self.bounds.top) + i64::from(self.bounds.bottom) - 600) / 2) as i32,
            width: 800,
            height: 600,
        }
    }
    pub fn pan_camera(&self, c: CameraRect, dx: i32, dy: i32) -> Result<CameraRect> {
        ensure!(
            c.width > 0 && c.height > 0 && c.width <= 4096 && c.height <= 4096,
            "camera dimensions"
        );
        let x = (i64::from(c.x) + i64::from(dx)).clamp(
            i64::from(self.bounds.left),
            (i64::from(self.bounds.right) - i64::from(c.width)).max(i64::from(self.bounds.left)),
        );
        let y = (i64::from(c.y) + i64::from(dy)).clamp(
            i64::from(self.bounds.top),
            (i64::from(self.bounds.bottom) - i64::from(c.height)).max(i64::from(self.bounds.top)),
        );
        Ok(CameraRect {
            x: i32::try_from(x)?,
            y: i32::try_from(y)?,
            ..c
        })
    }
    pub fn render(&self, camera: CameraRect, cancel: &AtomicBool) -> Result<Canvas> {
        if !self.multi {
            return self.assets.render_static(camera, cancel);
        }
        ensure!(
            camera.width > 0 && camera.height > 0 && camera.width <= 4096 && camera.height <= 4096,
            "camera budget"
        );
        let view = ViewProjection {
            anchor_cell: self.anchor,
            anchor_pixel: PixelPoint {
                x: 24i32.checked_sub(camera.x).context("camera X")?,
                y: 768i32.checked_sub(camera.y).context("camera Y")?,
            },
        };
        let mut canvas = Canvas::new(camera.width, camera.height)?;
        canvas.clear([18, 25, 35]);
        for d in &self.draws {
            ensure!(
                !cancel.load(Ordering::Acquire),
                "multi-sector render cancelled"
            );
            let p = d
                .cell
                .screen_origin(view)
                .context("map projected coordinate")?;
            if i64::from(p.x) + 24 <= 0
                || i64::from(p.y) + 24 <= 0
                || i64::from(p.x) >= i64::from(camera.width)
                || i64::from(p.y) >= i64::from(camera.height)
            {
                continue;
            }
            let t = &self.assets.scene_tiles()[&d.tile];
            canvas.blit_rgba(&t.rgba, t.width, t.height, p.x, p.y)?;
        }
        ensure!(
            !cancel.load(Ordering::Acquire),
            "multi-sector render cancelled"
        );
        Ok(canvas)
    }
    pub(crate) fn enqueue<'a>(
        &'a self,
        raster: &mut super::world_raster::Raster<'a>,
        view: ViewProjection,
        height: u32,
        roofs: &super::native_world_roofs::Frame,
        cancel: &AtomicBool,
    ) -> Result<usize> {
        let rows = NativeDrawCell::visible_rows(view, height).context("visible terrain rows")?;
        let start = self
            .draws
            .partition_point(|d| d.cell.scan_row() < *rows.start());
        let end = self
            .draws
            .partition_point(|d| d.cell.scan_row() <= *rows.end());
        let mut visible = 0;
        for (i, d) in self.draws[start..end].iter().enumerate() {
            if i % 256 == 0 {
                ensure!(!cancel.load(Ordering::Acquire), "layered terrain cancelled");
            }
            // The key is for indexing only. Preserve source insertion when queueing ties.
            let depth = d.depth;
            visible += usize::from(raster.tile_roof_source(
                d.cell,
                depth,
                &self.prepared[&d.tile],
                roofs.shades.shade(
                    d.group,
                    matches!(self.prepared[&d.tile].kind() as u8, 6 | 7),
                ),
                d.source,
            )?);
        }
        Ok(visible)
    }
    pub fn prepared_bytes(&self) -> usize {
        self.prepared_bytes
    }
}
