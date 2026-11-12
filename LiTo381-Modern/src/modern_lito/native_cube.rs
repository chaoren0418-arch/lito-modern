//! The four server cube GFX use action3 and two body-linked clothing layers.
use super::sprite_actions::{ActionKey, Directionality, FrameEvent, SpriteActionCatalog};
use anyhow::{ensure, Context, Result};

pub(crate) fn is_graphics(graphics: u32) -> bool {
    matches!(graphics, 6706 | 6712 | 6718 | 6724)
}

pub(crate) fn qualify(catalog: &SpriteActionCatalog, graphics: u32) -> Result<()> {
    ensure!(is_graphics(graphics), "unknown cube graphics");
    let row = catalog
        .sprites
        .get(&graphics)
        .context("cube definition absent")?;
    let action = catalog
        .resolve_action(graphics, ActionKey::new(3))?
        .animation;
    let shadow = (graphics + 4).to_string();
    let clothing = format!("2 {} {}", graphics - 1, graphics + 1);
    ensure!(
        row.header_issue.is_none()
            && row.graphics_id == graphics
            && row.image_count == Some(1)
            && row.actions.len() == 2
            && row.actions.contains_key(&ActionKey::new(8))
            && row.attributes.len() == 3
            && [
                (101, shadow.as_bytes()),
                (104, b"128".as_slice()),
                (105, clothing.as_bytes())
            ]
            .iter()
            .all(|(key, value)| row
                .attributes
                .iter()
                .any(|attribute| attribute.key == ActionKey::new(*key)
                    && attribute.body_bytes.trim_ascii() == *value)),
        "cube body/layer controls changed"
    );
    let subframes = [
        11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 4, 5, 6, 7, 8, 9, 10,
    ];
    let ticks = [
        7, 4, 4, 4, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
    ];
    ensure!(
        action.directionality == Directionality::Undirected
            && action.frames_per_second == 24
            && action.frames.len() == subframes.len()
            && action.frames.iter().enumerate().all(|(index, frame)| {
                let events = match index {
                    1 => vec![FrameEvent::Effect(4839)],
                    4 | 14 => vec![FrameEvent::Effect(4840)],
                    6 | 15 => vec![FrameEvent::Hit],
                    _ => Vec::new(),
                };
                frame.image_id == 0
                    && frame.subframe == subframes[index]
                    && frame.duration_ticks == ticks[index]
                    && frame.events == events
            }),
        "cube action3 definition changed"
    );
    Ok(())
}
