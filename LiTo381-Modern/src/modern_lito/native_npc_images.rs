//! Original page IMG assets. These pictures do not authorize destination state.
use super::{native_offline_menu_skin::{decode_img, CopyMode, Img}, native_resources::NativeSurfaceArchive};
use anyhow::{ensure, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path, sync::atomic::{AtomicBool, Ordering}};

pub struct Bank { images: BTreeMap<u16, Img>, pub sources: Value }
impl Bank {
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        ensure!(!cancel.load(Ordering::Acquire), "NPC images cancelled");
        let (mut archive, index, pak) = NativeSurfaceArchive::observe(&root.join("LiTo"), cancel)?;
        let mut images = BTreeMap::new();
        let mut members = Vec::new();
        for (id, expected) in [(7311, "90CBBA28E178BE11BCA139BA7E6FEB34374CF7D4378555E1E3015EAB8ABABB92"),
                               (7312, "0E60E9F1293D8014BFA22FE11A0057671F810C76F639D74CED9942577BCA9D77")] {
            let name = format!("{id}.img");
            let bytes = archive.read(&name, 372, cancel)?;
            let digest = format!("{:X}", Sha256::digest(&bytes));
            ensure!(digest == expected, "original NPC IMG differs: {name}");
            let image = decode_img(&bytes, CopyMode::ColorKey)?;
            ensure!((image.width, image.height) == (13, 14), "NPC IMG dimensions differ");
            members.push(json!({"member":name,"bytes":bytes.len(),"sha256":digest,
                               "rgba_sha256":format!("{:X}",Sha256::digest(&image.rgba))}));
            images.insert(id, image);
        }
        ensure!(!cancel.load(Ordering::Acquire), "NPC images cancelled");
        let sources = json!({"index":{"bytes":index.bytes,"sha256":index.sha256.iter().map(|b|format!("{b:02X}")).collect::<String>()},
                             "pak":{"bytes":pak.bytes,"sha256":pak.sha256.iter().map(|b|format!("{b:02X}")).collect::<String>()},"members":members});
        Ok(Self { images, sources })
    }
    pub fn glyph(&self, html: &str, declared: &[u16], action: &str) -> Option<&Img> {
        if html != "TeleportMe" { return None; }
        let id = match action { "LT_TELEPORT_B" => 7311, "LT_TELEPORT_C" => 7312, _ => return None };
        declared.contains(&id).then(|| self.images.get(&id)).flatten()
    }
}
