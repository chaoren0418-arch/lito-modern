//! CP950 own/hover labels plus the native all-name pass. Viewport edges are
//! adapted to the modern 800x600 canvas. The all-name switch has no input
//! binding here; emblem and suffix images are drawn by their own modules.
use super::{
    compositor::Canvas,
    native_inventory_names::Catalog,
    native_world::{Actor, World},
    native_world_text::Cache,
};
use anyhow::{ensure, Result};
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicBool, Ordering},
};

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
/// Native 48A1E6: base class bodies never receive the `773.img` suffix in the
/// all-name pass.
pub const ALL_NAME_PLAIN_GRAPHICS: [u16; 8] = [0, 1, 61, 48, 138, 37, 734, 1186];
/// Native 48A71A: the hover pass additionally exempts 2786 and 2796.
pub const HOVER_PLAIN_GRAPHICS: [u16; 10] = [0, 1, 61, 48, 138, 37, 734, 1186, 2786, 2796];

/// Presentation inputs that are not owned by the packet World.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Policy {
    /// Native `byte_64BC14`: a process-wide switch, off at start. The original
    /// flips it from the `c` console word (452D87). It only takes effect while
    /// `name_privilege` is nonzero.
    pub all_names: bool,
    /// Actors whose native actor+0x1F relation flag currently resolves true.
    pub flag_1f: BTreeSet<u32>,
}
/// Native `byte_64BC13` as written by the own-actor constructor (4E7AB5): own
/// S_PUT_OBJECT status bit 0x80 stores 2, otherwise 0. The value 1 comes from a
/// protected client-side variable that no server packet sets; it stays outside
/// this reconstruction.
pub fn name_privilege(world: &World) -> u8 {
    world
        .own_actor()
        .map_or(0, |a| if a.status & 0x80 != 0 { 2 } else { 0 })
}
/// Native 489F30 gate: privilege and switch must both be nonzero.
pub fn all_names_active(world: &World, policy: &Policy) -> bool {
    policy.all_names && name_privilege(world) != 0
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Line {
    pub id: u32,
    pub kind: &'static str,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub color: u16,
    /// Full name measured by native 432330 when the `773.img` suffix follows
    /// this line at `x + extent + 7`.
    pub badge: Option<String>,
}
/// Native hover and all-name passes skip a leading +/- prefix through (but not
/// including) the first space, while retaining its original width for layout.
fn visible_name(name: &str) -> String {
    if name.starts_with('+') || name.starts_with('-') {
        name.find(' ').map_or("", |at| &name[at..]).to_owned()
    } else {
        name.to_owned()
    }
}
/// Native 46EA30 keeps the owner text before `(`; 48A00C then appends the last
/// two CP950 bytes of the title. A title shorter than two bytes makes the
/// original read before its buffer, so no suffix is produced here; a tail that
/// is not valid CP950 on its own is dropped for the same reason.
fn owner_text(master: &str, title: &str) -> Option<String> {
    let base = master.split('(').next().unwrap_or("");
    let suffix = if title.is_empty() {
        None
    } else {
        super::native_login::encode_text(title, 256)
            .ok()
            .and_then(|bytes| {
                let tail = bytes.len().checked_sub(2)?;
                super::native_login::decode_text(&bytes[tail..]).ok()
            })
    };
    let text = format!("{base}{}", suffix.unwrap_or_default());
    (!text.is_empty()).then_some(text)
}
/// Native 489F60..48A281 for one roster actor other than the own actor.
fn all_name_lines(
    a: &Actor,
    catalog: &Catalog,
    b: [i32; 4],
    policy: &Policy,
    lines: &mut Vec<Line>,
) {
    let Ok(name) = catalog.resolve(&a.name) else {
        return;
    };
    let Ok(nw) = width(&name) else { return };
    let title = catalog.resolve(&a.title).ok().filter(|s| !s.is_empty());
    let tw = title.as_deref().and_then(|s| width(s).ok());
    // 4E7200 sizes the rectangle from the title, although this pass prints the
    // owner line instead of the title.
    let height = if tw.is_some() { 27 } else { 14 };
    let top = if b[1] - height <= 0 {
        b[3] + 1
    } else {
        b[1] - height
    };
    let rect_width = nw.max(tw.unwrap_or(0)) + 2 + if a.emblem != 0 { 18 } else { 0 };
    let left = ((b[0] + b[2] - rect_width) / 2).clamp(0, (800 - rect_width).max(0));
    let mut y = top - 3;
    let color = lawful_color(a.lawful);
    if !a.master.is_empty() {
        let owner = owner_text(&a.master, title.as_deref().unwrap_or(""))
            .and_then(|text| width(&text).ok().map(|w| (text, w)));
        if let Some((text, ow)) = owner {
            lines.push(Line {
                id: a.id,
                kind: "owner",
                text,
                x: left + (rect_width - ow) / 2,
                y,
                color,
                badge: None,
            });
        }
        y += 13;
    }
    let flag = policy.flag_1f.contains(&a.id);
    // 48A186: the name is printed only for 1F actors and for owned actors.
    if !flag && a.master.is_empty() {
        return;
    }
    let text = visible_name(&name);
    if text.is_empty() {
        return;
    }
    lines.push(Line {
        id: a.id,
        kind: "all",
        text,
        x: left + (rect_width - nw) / 2 + if a.emblem != 0 { 10 } else { 0 },
        y,
        color,
        badge: (!flag && !ALL_NAME_PLAIN_GRAPHICS.contains(&a.graphics)).then_some(name),
    });
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
    layout_with(world, catalog, hits, own_x, hover, &Policy::default())
}
/// `layout` plus the native all-name pass (489F30) and the `773.img` suffix
/// conditions. While the all-name pass is active the original skips the hover
/// block entirely; the own label is unaffected.
pub fn layout_with(
    world: &World,
    catalog: &Catalog,
    hits: &[(u32, [i32; 4])],
    own_x: i32,
    hover: Option<Hover>,
    policy: &Policy,
) -> Vec<Line> {
    let mut lines = Vec::new();
    let Some(own) = world.own_actor() else {
        return lines;
    };
    let special = matches!(world.map_id, Some(500 | 560));
    let all = all_names_active(world, policy);
    let hovered = if all {
        None
    } else {
        hover.and_then(|h| h.resolve(world))
    };
    for (id, b) in hits {
        let is_own = *id == own.id;
        if !world.actor_visible(*id) {
            continue;
        }
        let Some(a) = world.actors.get(id) else {
            continue;
        };
        if all && !is_own {
            all_name_lines(a, catalog, *b, policy, &mut lines);
            continue;
        }
        if !(is_own && !special || hovered == Some(*id)) {
            continue;
        }
        // 48A309: the hover block handles the own actor only on maps 500/560.
        let as_hover = hovered == Some(*id) && (!is_own || special);
        let Ok(name) = catalog.resolve(&a.name) else {
            continue;
        };
        let Ok(nw) = width(&name) else { continue };
        let title = if as_hover {
            catalog
                .resolve(&a.title)
                .ok()
                .filter(|s| !s.is_empty() && width(s).is_ok())
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
            lines.push(Line {
                id: *id,
                kind: "title",
                text,
                // 48A36C: the own actor keeps its own-label horizontal anchor.
                x: if is_own {
                    own_x - 24 - tw / 2
                } else {
                    left + (rect_width - tw) / 2
                },
                y,
                color: lawful_color(a.lawful),
                badge: None,
            });
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
        let text = if is_own {
            name.clone()
        } else {
            visible_name(&name)
        };
        // 48A6F6: a titled non-1F actor outside the plain class bodies gets the
        // suffix image after its hover name.
        let badge = (as_hover
            && !policy.flag_1f.contains(id)
            && !a.title.is_empty()
            && !HOVER_PLAIN_GRAPHICS.contains(&a.graphics))
        .then_some(name);
        if !text.is_empty() {
            lines.push(Line {
                id: *id,
                kind: if is_own { "own" } else { "hover" },
                text,
                x,
                y,
                color,
                badge,
            });
        }
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
        serde_json::json!({"lines":rows,"font_height":12,"charset":136,"ownership":"click_through","viewport_adaptation":"800x600 origin replaces native 48px margin","cache_entries":cache.len(),"cache_bytes":cache.bytes(),"all_names":lines.iter().any(|l| matches!(l.kind, "all" | "owner")),"full_native_equivalence":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> World {
        let f: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-world-current.json"
        ))
        .unwrap();
        let mut w = World::default();
        for k in ["world", "start", "status", "actor", "bot", "hp", "mp"] {
            let h = f["packets"][k].as_str().unwrap();
            let b: Vec<u8> = (0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect();
            w.accept(&b).unwrap();
        }
        let own = w.own_actor().unwrap().id;
        let a = w.actors.get_mut(&own).unwrap();
        a.name = "Own".into();
        a.emblem = 0;
        a.status &= !0x80;
        let a = w.actors.get_mut(&2).unwrap();
        a.title = String::new();
        a.master = String::new();
        a.emblem = 0;
        a.graphics = 3000;
        a.lawful = 0;
        w
    }
    fn catalog() -> Catalog {
        Catalog::from_tables(b"1\r\nzero", b"1\r\nzero").unwrap()
    }
    fn kinds(lines: &[Line]) -> Vec<&'static str> {
        lines.iter().map(|l| l.kind).collect()
    }
    fn hits(w: &World) -> [(u32, [i32; 4]); 2] {
        [
            (2, [300, 200, 340, 260]),
            (w.own_actor().unwrap().id, [390, 200, 430, 260]),
        ]
    }
    #[test]
    fn all_name_gate_needs_own_status_bit_and_switch() {
        let mut w = world();
        let own = w.own_actor().unwrap().id;
        let on = Policy {
            all_names: true,
            ..Default::default()
        };
        assert_eq!(name_privilege(&w), 0);
        assert!(!all_names_active(&w, &on));
        w.actors.get_mut(&own).unwrap().status |= 0x80;
        assert_eq!(name_privilege(&w), 2);
        assert!(all_names_active(&w, &on));
        assert!(!all_names_active(&w, &Policy::default()));
    }
    #[test]
    fn owner_line_keeps_text_before_parenthesis_and_title_tail() {
        assert_eq!(owner_text("Lord(Pledge)", "").as_deref(), Some("Lord"));
        assert_eq!(owner_text("Lord", "x").as_deref(), Some("Lord"));
        assert_eq!(owner_text("Lord", "of").as_deref(), Some("Lordof"));
        assert_eq!(owner_text("Lord", "Sir of").as_deref(), Some("Lordof"));
        assert_eq!(owner_text("主人", "忠犬的").as_deref(), Some("主人的"));
        assert_eq!(owner_text("", ""), None);
    }
    #[test]
    fn all_name_pass_prints_only_flagged_and_owned_actors() {
        let mut w = world();
        let own = w.own_actor().unwrap().id;
        w.actors.get_mut(&own).unwrap().status |= 0x80;
        w.actors.get_mut(&2).unwrap().name = "+7 Wolf".into();
        let c = catalog();
        let h = hits(&w);
        let hover = Hover::capture(&w, 2);
        let mut policy = Policy {
            all_names: true,
            ..Default::default()
        };
        // Neither 1F nor owned: the pass prints nothing, and the hover block is skipped.
        assert_eq!(kinds(&layout_with(&w, &c, &h, 424, hover, &policy)), ["own"]);
        policy.flag_1f.insert(2);
        let rows = layout_with(&w, &c, &h, 424, hover, &policy);
        assert_eq!(kinds(&rows), ["own", "all"]);
        assert_eq!(
            (&*rows[1].text, rows[1].x, rows[1].y, rows[1].color),
            (" Wolf", 299, 183, 0x7fff)
        );
        assert_eq!(rows[1].badge, None);
        policy.flag_1f.clear();
        {
            let a = w.actors.get_mut(&2).unwrap();
            a.master = "Lord(Pledge)".into();
            a.emblem = 9;
        }
        let rows = layout_with(&w, &c, &h, 424, hover, &policy);
        assert_eq!(kinds(&rows), ["own", "owner", "all"]);
        assert_eq!((&*rows[1].text, rows[1].x, rows[1].y), ("Lord", 308, 183));
        assert_eq!((&*rows[2].text, rows[2].x, rows[2].y), (" Wolf", 309, 196));
        assert_eq!(rows[2].badge.as_deref(), Some("+7 Wolf"));
        w.actors.get_mut(&2).unwrap().graphics = 61;
        let rows = layout_with(&w, &c, &h, 424, hover, &policy);
        assert_eq!(rows[2].badge, None);
        // Without the privilege the switch alone changes nothing.
        w.actors.get_mut(&own).unwrap().status &= !0x80;
        assert_eq!(
            kinds(&layout_with(&w, &c, &h, 424, hover, &policy)),
            ["own", "hover"]
        );
    }
    #[test]
    fn hover_suffix_needs_title_without_flag_or_class_body() {
        let mut w = world();
        {
            let a = w.actors.get_mut(&2).unwrap();
            a.name = "Guard".into();
            a.title = "Captain".into();
        }
        let c = catalog();
        let h = hits(&w);
        let hover = Hover::capture(&w, 2);
        let mut policy = Policy::default();
        let rows = layout_with(&w, &c, &h, 424, hover, &policy);
        assert_eq!(kinds(&rows), ["own", "title", "hover"]);
        assert_eq!(rows[2].badge.as_deref(), Some("Guard"));
        assert_eq!(rows[0].badge, None);
        policy.flag_1f.insert(2);
        assert_eq!(layout_with(&w, &c, &h, 424, hover, &policy)[2].badge, None);
        policy.flag_1f.clear();
        w.actors.get_mut(&2).unwrap().graphics = 2796;
        assert_eq!(layout_with(&w, &c, &h, 424, hover, &policy)[2].badge, None);
        {
            let a = w.actors.get_mut(&2).unwrap();
            a.graphics = 3000;
            a.title = String::new();
        }
        let rows = layout_with(&w, &c, &h, 424, hover, &policy);
        assert_eq!(kinds(&rows), ["own", "hover"]);
        assert_eq!(rows[1].badge, None);
        assert_eq!(kinds(&layout(&w, &c, &h, 424, None)), ["own"]);
    }
}
