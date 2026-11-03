//! Original background call geometry in its 744x552 coordinate domain.
//! This only plans pixel calls. Resource selection and modern viewport/camera
//! conversion must be qualified by the scene adapter before this is activated.
use anyhow::{ensure, Context, Result};

use super::native_background_pixels::{Blend, Paint, Rect, Strength};
use super::world_depth::NativeDrawCell;

#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub map_id: u16,
    pub level: u8,
    pub class: i32,
    /// Original actor half-cell fields; convert packet whole cells explicitly.
    pub cell: NativeDrawCell,
    /// Original camera fields, not the modern viewport's pixel anchor.
    pub camera: [i32; 2],
    pub offsets: [i32; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Call {
    pub base_id: u16,
    pub overlay_id: u16,
    pub paint: Paint,
}

/// Reject arithmetic outside the original signed domain rather than allowing
/// malformed inputs to wrap into a different background phase.
fn coordinate(cell: i32, camera: i32, scale: i32) -> Result<i32> {
    let cell = cell.checked_mul(scale).context("background cell scale overflow")? / 6;
    let camera = 48i32.checked_sub(camera)
        .and_then(|n| n.checked_mul(5))
        .context("background camera overflow")? / 6;
    cell.checked_add(camera).context("background coordinate overflow")
}

pub fn plan(input: Input) -> Result<Vec<Call>> {
    ensure!(input.level <= 31, "background level outside native light domain");
    let half = input.cell.half_x / 2;
    let horizontal = input.cell.y.checked_add(half).context("background cell X overflow")?;
    let vertical = input.cell.y.checked_sub(half).context("background cell Y overflow")?;
    let px = coordinate(horizontal, input.camera[0], 24)?;
    let py = coordinate(vertical, input.camera[1], 12)?;
    let rx = px.rem_euclid(384);
    let ry = py.rem_euclid(384);
    let quotient = px / 384;
    let strength = if input.level >= 15 || (1..=3).contains(&input.class) {
        Strength::Full
    } else {
        Strength::Attenuated(input.level * 9 / 16)
    };
    // This reproduces both original branches; it does not enable either map
    // family in a scene. Only the map303 caller is currently established.
    let (base_id, overlay_id, blend) = if input.map_id == 303 {
        (800, 803, Blend::Screen)
    } else {
        (922, 923, Blend::Multiply)
    };
    let mut calls = Vec::with_capacity(9);
    for column in 0..3 {
        // Native signed remainder equals +1. Negative odd quotients yield -1
        // and must not be replaced with a bit/parity or Euclidean test.
        let shift = if (quotient + column) % 2 == 1 { (ry + 192) % 384 } else { ry };
        for row in 0..3 {
            let mut x = column * 384 - rx;
            let mut y = row * 384 - shift;
            let (mut left, mut top, mut width, mut height) = (0, 0, 384, 384);
            if x < 0 { left = -x; width += x; x = 0; }
            if x + 384 > 744 { width += 360 - x; }
            if y < 0 { top = -y; height += y; y = 0; }
            // Preserve the original conditional: test 552, cap at 522.
            if y + 384 > 552 { height += 138 - y; }
            if width <= 0 || height <= 0 { continue; }
            calls.push(Call {
                base_id, overlay_id,
                paint: Paint {
                    source: Rect { left: left as usize, top: top as usize,
                        right: (left + width) as usize, bottom: (top + height) as usize },
                    destination: [x as usize, y as usize], offsets: input.offsets,
                    blend, strength,
                },
            });
        }
    }
    Ok(calls)
}
