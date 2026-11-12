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
    /// Image placed by the document after action `index` (original inline
    /// layout). Requires the TeleportMe page and an image this bank verified;
    /// the command-to-image mapping must agree with the document placement.
    pub fn trailing_glyph(&self, html: &str, trailing: &[(usize, u16)], index: usize, action: &str) -> Option<&Img> {
        if html != "TeleportMe" { return None; }
        let (_, id) = trailing.iter().find(|(i, _)| *i == index)?;
        let expected = match action { "LT_TELEPORT_B" => 7311, "LT_TELEPORT_C" => 7312, _ => return None };
        (*id == expected).then(|| self.images.get(id)).flatten()
    }
}

/// Placement of one owner-drawn row for `<a>label</a>   <img>`: the label at
/// the left inset, then one collapsed HTML space, then the image, vertically
/// centred. A long label is clipped so the image stays inside the row; when the
/// row cannot hold the image at all, only the label is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrailingRow { pub text: [i32; 4], pub image: Option<[i32; 2]> }
pub fn trailing_row(row: [i32; 4], inset: i32, label_width: i32, space: i32, image: (i32, i32)) -> TrailingRow {
    let [left, top, right, bottom] = row;
    let (left, right) = (left.saturating_add(inset), right.saturating_sub(inset));
    let (label_width, space) = (label_width.max(0), space.max(0));
    let room = right.saturating_sub(left).saturating_sub(space).saturating_sub(image.0);
    if image.0 <= 0 || image.1 <= 0 || room <= 0 {
        return TrailingRow { text: [left, top, right, bottom], image: None };
    }
    let text_right = left.saturating_add(label_width.min(room));
    let x = text_right.saturating_add(space);
    let y = top.saturating_add((bottom.saturating_sub(top).saturating_sub(image.1)) / 2);
    TrailingRow { text: [left, top, text_right, bottom], image: Some([x, y]) }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn trailing_image_follows_label_with_one_space_and_is_centred() {
        // 4px inset, 60px label, 4px space, 13x14 IMG in a 200x24 row.
        let row = trailing_row([0, 10, 200, 34], 4, 60, 4, (13, 14));
        assert_eq!(row.text, [4, 10, 64, 34]);
        assert_eq!(row.image, Some([68, 15]));
    }
    #[test]
    fn long_label_is_clipped_so_the_image_stays_in_the_row() {
        let row = trailing_row([0, 0, 100, 20], 4, 500, 4, (13, 14));
        let [x, y] = row.image.unwrap();
        assert_eq!(row.text[2] + 4, x);
        assert!(x + 13 <= 96 && y == 3);
    }
    #[test]
    fn row_too_narrow_or_empty_image_draws_only_the_label() {
        assert_eq!(trailing_row([0, 0, 20, 20], 4, 50, 4, (13, 14)).image, None);
        assert_eq!(trailing_row([0, 0, 200, 20], 4, 50, 4, (0, 14)).image, None);
        assert_eq!(trailing_row([0, 0, 200, 20], 4, 50, 4, (0, 14)).text, [4, 0, 196, 20]);
    }
}
