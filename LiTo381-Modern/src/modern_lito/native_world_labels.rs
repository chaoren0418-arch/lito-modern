//! Ordinary CP950 own/hover labels. Viewport edges are adapted to the modern
//! 800x600 canvas; privileged all-name mode, emblems and special NPC badges are
//! deliberately outside this qualified subset.
use super::{
    compositor::Canvas, native_inventory_names::Catalog, native_world::World,
    native_world_text::Cache,
};
use anyhow::{ensure, Result};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hover {
    id: u32,
    generation: u64,
    incarnation: u64,
    revision: u64,
}
impl Hover {
    pub fn capture(world: &World, id: u32) -> Option<Self> {
        let a = world.actors.get(&id)?;
        world.actor_visible(id).then_some(Self {
            id,
            generation: world.generation,
            incarnation: a.incarnation,
            revision: a.target_revision,
        })
    }
    pub fn resolve(self, world: &World) -> Option<u32> {
        (self.generation == world.generation
            && world.actor_visible(self.id)
            && world.actors.get(&self.id).is_some_and(|a| {
                a.incarnation == self.incarnation && a.target_revision == self.revision
            }))
        .then_some(self.id)
    }
}
pub fn lawful_color(lawful: i16) -> u16 {
    let value = i32::from(lawful);
    if value < 0 {
        let low = (31 - ((-value >> 6) + 1).min(31)) as u16;
        0x7c00 | (low << 5) | low
    } else if value < 500 {
        0x7fff
    } else {
        let low = (31 - (((value - 500) >> 8) + 1).min(31)) as u16;
        (low << 10) | (low << 5) | 31
    }
}
fn width(text: &str) -> Result<i32> {
    let bytes = super::native_login::encode_text(text, 256)?;
    Ok(bytes.len() as i32 * 6)
}
/// Native 4318E0 writes a name as colour runs: `\fX` inside the resolved text
/// (monster, NPC or ground-item names from desc-c) switches the palette entry
/// and is not drawn. The first run keeps `kind`; later runs are `"run"`. The
/// six-pixel-per-byte advance between runs is inferred from the positioning
/// code, 4315B0 itself was not decoded. Placement still uses the byte length
/// of the whole text, escapes included, as the original does.
fn push_runs(
    lines: &mut Vec<Line>,
    id: u32,
    kind: &'static str,
    text: &str,
    mut x: i32,
    y: i32,
    color: u16,
) {
    for (n, run) in super::native_text_format::runs(text, color)
        .into_iter()
        .enumerate()
    {
        let Ok(advance) = width(&run.text) else {
            return;
        };
        lines.push(Line {
            id,
            kind: if n == 0 { kind } else { "run" },
            text: run.text,
            x,
            y,
            color: run.color,
        });
        x += advance;
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Line {
    pub id: u32,
    pub kind: &'static str,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub color: u16,
}
/// Byte-width layout follows native 4E7200 / 489EAF / 48A2A1. The canvas origin
/// replaces the original 48px window margin. Widths intentionally do not use GDI
/// measurement: the original positions text using six pixels per CP950 byte.
pub fn layout(
    world: &World,
    catalog: &Catalog,
    hits: &[(u32, [i32; 4])],
    own_x: i32,
    hover: Option<Hover>,
) -> Vec<Line> {
    let mut lines = Vec::new();
    let Some(own) = world.own_actor() else {
        return lines;
    };
    let special = matches!(world.map_id, Some(500 | 560));
    for (id, b) in hits {
        let is_own = *id == own.id;
        if !world.actor_visible(*id)
            || !(is_own && !special || hover.and_then(|h| h.resolve(world)) == Some(*id))
        {
            continue;
        }
        let a = &world.actors[id];
        // `%i` entries take the runtime name; the entered character's own
        // name is used for it (see `Catalog::expand`).
        let who = Some(own.name.as_str())
            .filter(|n| !n.is_empty() && !n.chars().any(char::is_control));
        let Ok(name) = catalog.expand(&a.name, who) else {
            continue;
        };
        let Ok(nw) = width(&name) else { continue };
        let title = if !is_own {
            catalog
                .expand(&a.title, who)
                .ok()
                .filter(|s| width(s).is_ok())
        } else {
            None
        };
        let tw = title.as_ref().map_or(0, |s| width(s).unwrap());
        let height = if title.is_some() { 27 } else { 14 };
        let top = if b[1] - height <= 0 {
            b[3] + 1
        } else {
            b[1] - height
        };
        let rect_width = nw.max(tw) + 2 + if a.emblem != 0 { 18 } else { 0 };
        let left = ((b[0] + b[2] - rect_width) / 2).clamp(0, (800 - rect_width).max(0));
        let mut y = top - 3;
        if let Some(text) = title {
            push_runs(
                &mut lines,
                *id,
                "title",
                &text,
                left + (rect_width - tw) / 2,
                y,
                lawful_color(a.lawful),
            );
            y += 13;
        }
        let x = if is_own {
            own_x - 24 - nw / 2
        } else {
            left + (rect_width - nw) / 2
        } + if a.emblem != 0 { 10 } else { 0 };
        let color = if special || (!is_own && a.tail_byte > 1) {
            lawful_color(a.lawful)
        } else if a.tail_byte != 0 {
            0x001f
        } else {
            0x7c00
        };
        // Native hover skips a leading +/- prefix through (but not including)
        // the first space, while retaining its original width for positioning.
        let text = if !is_own && (name.starts_with('+') || name.starts_with('-')) {
            name.find(' ').map_or("", |at| &name[at..]).to_owned()
        } else {
            name
        };
        push_runs(
            &mut lines,
            *id,
            if is_own { "own" } else { "hover" },
            &text,
            x,
            y,
            color,
        );
    }
    // The native own label precedes the hover overlay regardless of actor ID.
    lines.sort_by_key(|l| if l.id == own.id { 0 } else { 1 });
    lines
}
pub fn draw(
    canvas: &mut Canvas,
    cache: &mut Cache,
    lines: &[Line],
    cancel: &AtomicBool,
) -> Result<serde_json::Value> {
    let mut rows = Vec::new();
    for line in lines {
        ensure!(!cancel.load(Ordering::Acquire), "world labels cancelled");
        let result = (|| -> Result<_> {
            let mask = cache.get(&line.text, cancel)?;
            mask.draw(canvas, line.x, line.y, line.color, 0)?;
            Ok(
                serde_json::json!({"line":line,"face":mask.face,"advance":mask.advance,"size":[mask.width,mask.height]}),
            )
        })();
        match result {
            Ok(row) => rows.push(row),
            Err(e) => rows.push(serde_json::json!({"line":line,"error":format!("{e:#}")})),
        }
    }
    ensure!(!cancel.load(Ordering::Acquire), "world labels cancelled");
    Ok(
        serde_json::json!({"lines":rows,"font_height":12,"charset":136,"ownership":"click_through","viewport_adaptation":"800x600 origin replaces native 48px margin","cache_entries":cache.len(),"cache_bytes":cache.bytes(),"all_names":false,"full_native_equivalence":false}),
    )
}

#[cfg(test)]
mod run_tests {
    use super::*;
    #[test]
    fn colour_runs_advance_six_pixels_per_cp950_byte() {
        let mut lines = Vec::new();
        push_runs(&mut lines, 9, "hover", "\\fT試煉\\f2ab", 100, 20, 0x7c00);
        assert_eq!(lines.len(), 2);
        assert_eq!(
            (lines[0].kind, &*lines[0].text, lines[0].x, lines[0].y, lines[0].color),
            ("hover", "試煉", 100, 20, 0x3e8c)
        );
        assert_eq!(
            (lines[1].kind, &*lines[1].text, lines[1].x, lines[1].y, lines[1].color),
            ("run", "ab", 124, 20, 0x03e0)
        );
        // A name without format codes is one line, exactly as before.
        let mut lines = Vec::new();
        push_runs(&mut lines, 9, "own", " Sword", 7, 8, 31);
        assert_eq!(lines.len(), 1);
        assert_eq!(
            (lines[0].kind, &*lines[0].text, lines[0].x, lines[0].color),
            ("own", " Sword", 7, 31)
        );
        // Only format codes: nothing to draw.
        let mut lines = Vec::new();
        push_runs(&mut lines, 9, "hover", "\\fT\\f2", 0, 0, 31);
        assert!(lines.is_empty());
    }
}