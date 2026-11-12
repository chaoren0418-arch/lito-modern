//! T021 original finite overlays. Their clocks never own buffs, poison or costs.
use super::{
    sprite_actions::{
        ActionKey, Animation, Directionality, FrameEvent, SpriteActionCatalog, SpriteActions,
    },
    tower_assets::{HonorAssetError, Result},
};

pub const GRAPHICS: [u16; 3] = [2942, 2943, 2944];

// 2943's first four bitmap slots are native no-block records; 2954 still paints.
const SPRITES: &[(&str, usize, usize, &str)] = &[
    (
        "2942-0.spr",
        10474,
        9,
        "AF84AF8C0CE0BD7459545C6E94E1676C9DF2826C785802362848FDB0DD8ABAFC",
    ),
    (
        "2943-0.spr",
        21345,
        13,
        "FD49E50E4EB17E19DE23048925AB15CA9FC3505A152C42B72D4E9762AD3FE716",
    ),
    (
        "2944-0.spr",
        8981,
        9,
        "482C7D8E9685376D72E8060CB00415FF26512FDB0D9DD79A7BB7DE0FB8FF64F9",
    ),
    (
        "2954-0.spr",
        8244,
        13,
        "03FB56D0D1A61EEC7CB903692B4C6C04C5FE49664A40254A3D58235DCE3CA982",
    ),
];

pub fn is_blank(name: &str, index: usize, f: &super::sprite::SpriteFrame) -> bool {
    name == "2943-0.spr"
        && index < 4
        && (
            f.x_offset,
            f.y_offset,
            f.width,
            f.height,
            f.frame_type,
            f.unknown1,
            f.unknown2,
        ) == (192, 120, 0, 0, 0, u16::MAX, 0)
        && f.rgba.is_empty()
        && f.fragments
            .as_ref()
            .is_some_and(|p| p.references.is_empty())
}

pub fn source_matches(name: &str, bytes: &[u8], decoded: &super::sprite::DecodedSprite) -> bool {
    use sha2::{Digest, Sha256};
    let Some((_, size, frames, sha)) = SPRITES.iter().find(|s| s.0 == name) else {
        return false;
    };
    bytes.len() == *size
        && format!("{:X}", Sha256::digest(bytes)) == *sha
        && decoded.frames.len() == *frames
        && decoded.frames.iter().enumerate().all(|(i, f)| {
            if name == "2943-0.spr" && i < 4 {
                is_blank(name, i, f)
            } else {
                f.width > 0
                    && f.height > 0
                    && f.rgba.len() == f.width as usize * f.height as usize * 4
                    && f.fragments
                        .as_ref()
                        .is_some_and(|p| !p.references.is_empty())
            }
        })
}

fn attributes(row: &SpriteActions, expected: &[(u32, &[u8])]) -> bool {
    row.attributes.len() == expected.len()
        && expected.iter().all(|(key, body)| {
            row.attributes
                .iter()
                .filter(|a| a.key == ActionKey::new(*key) && a.body_bytes.trim_ascii() == *body)
                .count()
                == 1
        })
}

pub fn qualify(catalog: &SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>> {
    let mut result = Vec::new();
    for (id, ticks, sound, effect) in [
        (
            2942,
            &[4, 7, 6, 2, 2, 3, 4, 4, 4][..],
            1505,
            b"1 -1".as_slice(),
        ),
        (
            2943,
            &[4, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3][..],
            1509,
            b"1 -1".as_slice(),
        ),
        (
            2944,
            &[4, 3, 3, 3, 3, 4, 4, 4, 4][..],
            1502,
            b"0 -1".as_slice(),
        ),
    ] {
        let row = catalog
            .sprites
            .get(&id)
            .ok_or(HonorAssetError::Unsupported("dark elf definition absent"))?;
        let resolved = catalog.resolve_action(id, ActionKey::new(0))?;
        let a = resolved.animation;
        let mut attrs = vec![
            (102, b"0".as_slice()),
            (104, b"8".as_slice()),
            (109, effect),
        ];
        if id == 2943 {
            attrs.push((105, b"1 2954"));
        }
        if row.id != id
            || row.graphics_id != id
            || row.image_count != Some(1)
            || row.header_issue.is_some()
            || row.actions.len() != 1
            || !attributes(row, &attrs)
            || resolved.definition_sprite_id != id
            || a.directionality != Directionality::Undirected
            || a.frames_per_second != 24
            || a.frames.len() != ticks.len()
            || a.frames.iter().enumerate().any(|(i, f)| {
                f.image_id != 0
                    || f.subframe
                        != if i == 0 {
                            ticks.len() as u32 - 1
                        } else {
                            i as u32 - 1
                        }
                    || f.duration_ticks != ticks[i]
                    || f.events
                        != if i == 1 {
                            vec![FrameEvent::Sound(sound)]
                        } else {
                            vec![]
                        }
            })
        {
            return Err(HonorAssetError::Unsupported(
                "dark elf original controls/timing changed",
            ));
        }
        result.push((id, 0, a.clone()));
    }
    let layer = catalog
        .sprites
        .get(&2954)
        .ok_or(HonorAssetError::Unsupported("shadow armor clothing absent"))?;
    if layer.id != 2954
        || layer.graphics_id != 2954
        || layer.image_count != Some(1)
        || layer.header_issue.is_some()
        || !layer.actions.is_empty()
        || !attributes(layer, &[(102, b"0"), (104, b"128")])
    {
        return Err(HonorAssetError::Unsupported(
            "shadow armor clothing changed",
        ));
    }
    // 105.clothes inherits the parent's bitmap slots, with no independent clock/audio.
    let mut slots = result.iter().find(|r| r.0 == 2943).unwrap().2.clone();
    for frame in &mut slots.frames {
        frame.events.clear();
    }
    result.push((2954, 0, slots));
    Ok(result)
}

#[cfg(test)]
#[path = "native_dark_elf_effect_tests.rs"]
mod tests;
