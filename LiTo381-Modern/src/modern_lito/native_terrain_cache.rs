//! Bounded static terrain window. Actors/effects are never retained here.
use super::{compositor::Canvas, scene::CameraRect};
use anyhow::{ensure, Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct TerrainCache {
    region: Option<(CameraRect, Canvas)>,
    pub builds: u64,
    pub hits: u64,
}
impl TerrainCache {
    pub fn clear(&mut self) {
        self.region = None;
    }
    pub fn bytes(&self) -> usize {
        self.region.as_ref().map_or(0, |(_, c)| c.rgba().len())
    }
    pub fn render_with(
        &mut self,
        camera: CameraRect,
        cancel: &AtomicBool,
        render: impl FnOnce(CameraRect) -> Result<Canvas>,
    ) -> Result<Canvas> {
        ensure!(!cancel.load(Ordering::Acquire), "terrain cache cancelled");
        ensure!(
            camera.width > 0 && camera.height > 0 && camera.width <= 1024 && camera.height <= 1024,
            "terrain viewport budget"
        );
        let contains = self.region.as_ref().is_some_and(|(r, _)| {
            i64::from(camera.x) >= i64::from(r.x)
                && i64::from(camera.y) >= i64::from(r.y)
                && i64::from(camera.x) + i64::from(camera.width)
                    <= i64::from(r.x) + i64::from(r.width)
                && i64::from(camera.y) + i64::from(camera.height)
                    <= i64::from(r.y) + i64::from(r.height)
        });
        if !contains {
            let region = CameraRect {
                x: camera.x.checked_sub(128).context("terrain X extent")?,
                y: camera.y.checked_sub(128).context("terrain Y extent")?,
                width: camera.width + 256,
                height: camera.height + 256,
            };
            let canvas = render(region)?;
            ensure!(
                canvas.width() == region.width && canvas.height() == region.height,
                "terrain render dimensions"
            );
            ensure!(!cancel.load(Ordering::Acquire), "terrain cache cancelled");
            self.region = Some((region, canvas));
            self.builds += 1;
        } else {
            self.hits += 1;
        }
        let (r, canvas) = self.region.as_ref().unwrap();
        let result = canvas.crop(
            u32::try_from(i64::from(camera.x) - i64::from(r.x))?,
            u32::try_from(i64::from(camera.y) - i64::from(r.y))?,
            camera.width,
            camera.height,
        )?;
        ensure!(!cancel.load(Ordering::Acquire), "terrain cache cancelled");
        Ok(result)
    }
}
