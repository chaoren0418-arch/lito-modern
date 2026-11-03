//! Worker-owned local terrain locator. Only qualified shared geometry and the
//! authoritative self position; no radar, path permission, new assets or I/O.
use super::{
    battle_follow::Observation,
    battle_payload::data::{self as d, Phase},
    battle_space::Navigation,
    compositor::Canvas,
    world_map::{PixelPoint, WorldCell, WorldMap, DIRECTIONS},
};
use anyhow::{ensure, Context, Result};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub(crate) const X: i32 = 8;
pub(crate) const Y: i32 = 8;
pub(crate) const WIDTH: u32 = 176;
pub(crate) const HEIGHT: u32 = 134;
const RADIUS: i32 = 20;
const CENTER: PixelPoint = PixelPoint { x: 88, y: 66 };
const BACKGROUND: [u8; 3] = [12, 18, 24];
const BORDER: [u8; 3] = [100, 135, 147];
const UNKNOWN: [u8; 3] = [18, 25, 31];
const CLOSED: [u8; 3] = [38, 46, 51];
const TERRAIN: [u8; 3] = [61, 88, 93];
const SELF: [u8; 3] = [85, 240, 226];
const DEAD: [u8; 3] = [207, 163, 115];

fn actor_alive(actor: &d::Actor) -> bool {
    actor.resources.as_ref().map(|r| r.vitals.hp > 0)
        .or_else(|| actor.vital.as_ref().map(|v| v.values.hp > 0))
        .unwrap_or(true)
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    map: u32,
    digest: [u8; 32],
    position: WorldCell,
    heading: u8,
    alive: bool,
}
pub(crate) struct Panel {
    observation: Observation,
    key: Key,
    rgba: Arc<[u8]>,
}
impl Panel {
    pub(crate) fn prepare(
        nav: &Navigation,
        previous: Option<&Self>,
        cancel: &AtomicBool,
    ) -> Result<Arc<Self>> {
        active(cancel)?;
        let o = nav.observation();
        let d = o.data();
        ensure!(
            d.world.phase != Phase::Aborted,
            "retired world cannot own a minimap"
        );
        let actor = d
            .actors
            .0
            .iter()
            .find(|a| a.entity == d.personal.entity)
            .context("minimap owner missing")?;
        let map = nav.map();
        let key = Key {
            map: map.map_id(),
            digest: map.canonical_digest(),
            position: WorldCell {
                x: actor.position.x,
                y: actor.position.y,
            },
            heading: actor.heading.try_into()?,
            alive: actor_alive(actor),
        };
        ensure!(
            key.heading < 8 && map.contains(key.position),
            "invalid minimap owner geometry"
        );
        let rgba = if let Some(old) = previous.filter(|p| p.key == key) {
            old.rgba.clone()
        } else {
            raster(key, map, cancel)?.into()
        };
        active(cancel)?;
        // Pixels may be reused; authority must always retain this exact snapshot.
        Ok(Arc::new(Self {
            observation: o.clone(),
            key,
            rgba,
        }))
    }
    pub(crate) fn matches(&self, o: &Observation) -> bool {
        std::ptr::eq(self.observation.data(), o.data())
    }
    pub(crate) fn contains(&self, point: PixelPoint) -> bool {
        point.x >= X
            && point.y >= Y
            && i64::from(point.x) < i64::from(X) + i64::from(WIDTH)
            && i64::from(point.y) < i64::from(Y) + i64::from(HEIGHT)
    }
    pub(crate) fn draw(&self, canvas: &mut Canvas) -> Result<()> {
        ensure!(
            canvas.width() == 800 && canvas.height() == 600,
            "minimap surface dimensions changed"
        );
        canvas.blit_rgba(&self.rgba, WIDTH, HEIGHT, X, Y)
    }
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "minimap preparation cancelled"
    );
    Ok(())
}
fn put(rgba: &mut [u8], x: i32, y: i32, rgb: [u8; 3]) {
    if x >= 0 && y >= 0 && x < WIDTH as i32 && y < HEIGHT as i32 {
        let at = (y as usize * WIDTH as usize + x as usize) * 4;
        rgba[at..at + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
}
// The same isometric orientation as ViewProjection: +X is right/up, +Y right/down.
fn project(dx: i32, dy: i32) -> Option<PixelPoint> {
    (i64::from(dx).abs() <= i64::from(RADIUS) && i64::from(dy).abs() <= i64::from(RADIUS)).then(
        || PixelPoint {
            x: CENTER.x + (dx + dy) * 2,
            y: CENTER.y + dy - dx,
        },
    )
}
fn cell_color(map: &WorldMap, cell: WorldCell) -> [u8; 3] {
    if !map.contains(cell) {
        UNKNOWN
    } else if map.standing(cell) {
        TERRAIN
    } else {
        CLOSED
    }
}
fn raster(key: Key, map: &WorldMap, cancel: &AtomicBool) -> Result<Vec<u8>> {
    let mut rgba = vec![0; WIDTH as usize * HEIGHT as usize * 4];
    for p in rgba.chunks_exact_mut(4) {
        p.copy_from_slice(&[BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255]);
    }
    for y in 0..HEIGHT as i32 {
        put(&mut rgba, 0, y, BORDER);
        put(&mut rgba, WIDTH as i32 - 1, y, BORDER);
    }
    for x in 0..WIDTH as i32 {
        put(&mut rgba, x, 0, BORDER);
        put(&mut rgba, x, HEIGHT as i32 - 1, BORDER);
    }
    for dy in -RADIUS..=RADIUS {
        active(cancel)?;
        for dx in -RADIUS..=RADIUS {
            let p = project(dx, dy).unwrap();
            let color = key
                .position
                .x
                .checked_add(dx)
                .zip(key.position.y.checked_add(dy))
                .map_or(UNKNOWN, |(x, y)| cell_color(map, WorldCell { x, y }));
            // Four pixels per isometric cell; adjacent cells tile without gaps.
            for x in -1..=1 {
                put(&mut rgba, p.x + x, p.y, color);
            }
            put(&mut rgba, p.x, p.y + 1, color);
        }
    }
    label(&mut rgba, &format!("MAP {}", key.map), 6, 5, 2);
    label(&mut rgba, "N", 5, 20, 1);
    let coords = format!("{},{}", key.position.x, key.position.y);
    let scale = if coords.len() * 8 <= WIDTH as usize - 12 {
        2
    } else {
        1
    };
    label(&mut rgba, &coords, 6, 117, scale);
    // No entity/objective/occupancy dots: this is not an enemy radar or a path map.
    let color = if key.alive { SELF } else { DEAD };
    for y in -3i32..=3 {
        for x in -3i32..=3 {
            if x.abs() + y.abs() <= 3 {
                put(&mut rgba, CENTER.x + x, CENTER.y + y, BACKGROUND);
            }
        }
    }
    let (dx, dy) = DIRECTIONS[usize::from(key.heading)];
    let end = project(dx * 3, dy * 3).unwrap();
    let steps = (end.x - CENTER.x).abs().max((end.y - CENTER.y).abs());
    for step in 0..=steps {
        put(
            &mut rgba,
            CENTER.x + (end.x - CENTER.x) * step / steps,
            CENTER.y + (end.y - CENTER.y) * step / steps,
            color,
        );
    }
    for y in -1..=1 {
        for x in -1..=1 {
            put(&mut rgba, CENTER.x + x, CENTER.y + y, color);
        }
    }
    Ok(rgba)
}
fn glyph(c: u8) -> [u8; 5] {
    match c {
        b'0' => [7, 5, 5, 5, 7],
        b'1' => [2, 6, 2, 2, 7],
        b'2' => [7, 1, 7, 4, 7],
        b'3' => [7, 1, 7, 1, 7],
        b'4' => [5, 5, 7, 1, 1],
        b'5' => [7, 4, 7, 1, 7],
        b'6' => [7, 4, 7, 5, 7],
        b'7' => [7, 1, 1, 1, 1],
        b'8' => [7, 5, 7, 5, 7],
        b'9' => [7, 5, 7, 1, 7],
        b'M' => [5, 7, 7, 5, 5],
        b'A' => [2, 5, 7, 5, 5],
        b'P' => [7, 5, 7, 4, 4],
        b'N' => [5, 7, 7, 7, 5],
        b',' => [0, 0, 0, 2, 4],
        b'-' => [0, 0, 7, 0, 0],
        _ => [0; 5],
    }
}
fn label(rgba: &mut [u8], text: &str, x: i32, y: i32, scale: i32) {
    for (i, c) in text.bytes().enumerate() {
        for (row, bits) in glyph(c).into_iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            put(
                                rgba,
                                x + (i as i32 * 4 + col) * scale + sx,
                                y + row as i32 * scale + sy,
                                [209, 225, 228],
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "battle_minimap_tests.rs"]
pub(crate) mod tests;
