//! Offline consumer of BIN's pinned background RGBA and 496F05 selector.
//! No live flag/owner, scene assignment, Native geometry or draw parity is inferred.
use super::{adapter, Canvas, Img};
use anyhow::{ensure, Result};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::Path;

const BYTES: usize = 640 * 480 * 4;
const NAMESPACE: &str = "NATIVE_BACKGROUND_RGBA_REFERENCE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundId {
    Image6314,
    Image7482,
}
impl BackgroundId {
    pub fn number(self) -> u32 {
        match self {
            Self::Image6314 => 6314,
            Self::Image7482 => 7482,
        }
    }
    pub fn rgba_sha256(self) -> &'static str {
        match self {
            Self::Image6314 => "949463E781BA723BAE1EDA005C1A346E2086EA003317CE7C41F140A8B61850E8",
            Self::Image7482 => "A7A1B6AEDA3B9F9259802F428ABEE3FBC809931C6CF41EA8FF07001A7D4E6BBF",
        }
    }
}

/// Explicit caller-supplied source byte; None means uninitialized/unknown.
/// The fixed PE's disk zero does not qualify a current flag value.
pub fn select(flag_64afa0: Option<u8>) -> Option<BackgroundId> {
    flag_64afa0.map(|value| {
        if value == 0 {
            BackgroundId::Image6314
        } else {
            BackgroundId::Image7482
        }
    })
}

pub struct BackgroundAsset {
    id: BackgroundId,
    image: Img,
}
impl BackgroundAsset {
    /// Caller buffers are accepted only when they equal the delivered reference.
    /// No RGB555 decode or native word conversion is performed a second time.
    pub fn from_rgba(id: BackgroundId, rgba: Vec<u8>) -> Result<Self> {
        ensure!(rgba.len() == BYTES, "background RGBA length drift");
        ensure!(
            format!("{:X}", Sha256::digest(&rgba)) == id.rgba_sha256(),
            "background RGBA SHA drift"
        );
        Ok(Self {
            id,
            image: Img {
                width: 640,
                height: 480,
                rgba,
            },
        })
    }
    pub fn id(&self) -> BackgroundId {
        self.id
    }
    pub fn image(&self) -> &Img {
        &self.image
    }

    /// Read-only confined input root; uses the existing bounded pinned reader.
    pub fn load(root: &Path, id: BackgroundId) -> Result<Self> {
        ensure!(
            root.is_absolute(),
            "explicit absolute BIN resource root required"
        );
        let path = root.join(format!("resources/modern-rgba/{}.rgba", id.number()));
        let pin =
            json!({"path":path,"bytes":BYTES,"sha256":id.rgba_sha256(),"namespace":NAMESPACE});
        let rgba = adapter::read_pinned(root, &pin, NAMESPACE, BYTES as u64)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Self::from_rgba(id, rgba)
    }

    /// Caller-chosen CPU preview geometry, independent of unknown Native origin.
    pub fn blit_preview(&self, canvas: &mut Canvas, origin: [i32; 2]) -> Result<()> {
        canvas.blit_rgba(
            &self.image.rgba,
            self.image.width,
            self.image.height,
            origin[0],
            origin[1],
        )
    }
}

/// Unknown input performs no file access and supplies no implicit default.
pub fn load_selected(root: &Path, flag_64afa0: Option<u8>) -> Result<Option<BackgroundAsset>> {
    select(flag_64afa0)
        .map(|id| BackgroundAsset::load(root, id))
        .transpose()
}

#[cfg(test)]
#[path = "native_background_assets_tests.rs"]
mod tests;
