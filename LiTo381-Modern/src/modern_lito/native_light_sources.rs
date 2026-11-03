//! Original actor-source selection and source.lmp composition policy.
//! The caller supplies native registration order, resolved metadata/relations
//! and shared render centers. This module never creates authority or actors.
use super::{native_light_color::Remap, native_light_mask::{Mask, Source}};
use anyhow::{ensure, Context, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Master { None, Own, Other }
#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub position: [i32; 2],
    pub raw_light: u8,
    pub action: u8,
    pub walk_action: u8,
    pub direction: u8,
    pub source_frame: u8,
    pub frame_count: u32,
    pub base_class: u8,
    pub subtype: u8,
    pub flag_1f: bool,
    pub flag_50: bool,
    pub mode_23: u8,
    pub master: Master,
}
/// Original4E89F0 classification, including signed action comparisons. These
/// flags are native presentation lifecycle state, not generic Actor.dead.
pub fn classify(a: &Actor) -> u8 {
    let action = a.action as i8;
    if a.base_class == 11 { return if action >= 37 || action == 28 {1} else {11}; }
    if a.flag_1f { return if a.flag_50 || action == 8 {3} else {5}; }
    match a.base_class {
        10 => {
            if a.flag_50 {3} else if action == 8 {2}
            else if a.master == Master::Own {6} else if a.master == Master::Other {5}
            else if a.subtype == 2 && a.mode_23 == 2 {1} else {10}
        }
        0 | 4 | 7 | 8 | 9 | 13 | 14 | 15 => a.base_class,
        5 | 6 | 12 => if a.flag_50 || action == 8 {3} else {a.base_class},
        _ => 1,
    }
}
pub fn blind_two_source(a: &Actor) -> bool { matches!(a.base_class, 5 | 6 | 10 | 12) }

#[derive(Clone, Copy, Debug)]
pub struct Policy {
    pub ambient: u32,
    pub underwater: bool,
    pub blind: u8,
    pub own_index: Option<usize>,
    pub own_position: [i32; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Blit {
    pub actor_index: usize,
    pub range: u8,
    pub center: [i32; 2],
    pub origin: [i64; 2],
    pub size: [usize; 2],
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Receipt {
    pub active: bool,
    pub effective_ambient: u32,
    pub accumulated_range: u32,
    pub outside_bounds: usize,
    pub blits: Vec<Blit>,
}
/// Selection after the native bounds check. Shared with the World adapter so
/// metadata/action lookups stop at the same700 range budget as composition.
pub fn source_range(policy:Policy,actor:&Actor,index:usize)->Option<u8> {
    if policy.blind!=0 {
        if policy.own_index!=Some(index) && (policy.blind!=2 || !blind_two_source(actor)) {return None;}
        Some(5)
    } else if policy.underwater && classify(actor)==5 {Some(14)}
    else if policy.underwater && classify(actor)==3 {Some(5)}
    else {
        let raw=actor.raw_light as i8;
        (raw>0).then(||(raw as u8).min(15))
    }
}
/// A skipped full-bright pass retains both the old mask and remap. Its caller
/// must bypass color transformation, matching the outer world-paint gate.
pub fn compose(source: &Source, mask: &mut Mask, remap: &mut Remap, policy: Policy,
    actors: &[Actor], centers: &[[i32; 2]]) -> Result<Receipt> {
    ensure!(actors.len() <= 2048 && actors.len() == centers.len(), "light actor/center budget");
    ensure!(policy.blind <= 2, "light blindness domain");
    if let Some(own) = policy.own_index {
        ensure!(actors.get(own).is_some_and(|a| a.position == policy.own_position), "light owner binding");
    }
    let effective = if policy.underwater {3} else {policy.ambient};
    let mut result = Receipt { effective_ambient: effective, ..Default::default() };
    if policy.blind == 0 && effective >= 15 { return Ok(result); }
    remap.select(if policy.blind != 0 {0} else {effective}, policy.underwater);
    mask.clear();
    result.active = true;
    for (index, actor) in actors.iter().enumerate() {
        let du = i64::from(actor.position[0]) - i64::from(policy.own_position[0]);
        let dv = i64::from(actor.position[1]) - i64::from(policy.own_position[1]);
        if du.abs() > 35 || dv.abs() > 15 { result.outside_bounds += 1; continue; }
        let Some(range)=source_range(policy,actor,index) else {continue;};
        let stencil = source.stencil(range)?;
        let center = centers[index];
        let origin = [((i64::from(center[0]) - stencil.width() as i64) / 2) & !3,
            (i64::from(center[1]) - stencil.height() as i64) / 2];
        mask.compose_centered(stencil, center[0], center[1]);
        result.blits.push(Blit { actor_index: index, range, center, origin,
            size: [stencil.width(), stencil.height()] });
        result.accumulated_range += u32::from(range);
        if result.accumulated_range >= 700 { break; }
    }
    Ok(result)
}

/// The retained native projection is used for original-coordinate comparisons.
/// Modern scene rendering supplies centers from the *same* interpolated pose
/// and camera as its body pass instead of adding these offsets a second time.
pub fn native_center(a: &Actor, origin: [i32; 2], scroll: [i32; 2]) -> Result<[i32; 2]> {
    let u = i64::from(a.position[0]) - i64::from(origin[0]);
    let v = i64::from(a.position[1]) - i64::from(origin[1]);
    let mut x = 12 * (((u + 1) & !1) + 2 * v - 2) - i64::from(scroll[0]) + 24;
    let mut y = 6 * (2 * v - (u & !1)) - i64::from(scroll[1]) + 6;
    if a.action == a.walk_action {
        ensure!(a.direction < 8, "light walking direction");
        let (base, stride) = match a.frame_count {12 => (32, 12), 8 => (176, 8), 6 => (128, 6), _ => (0, 4)};
        let phase = if stride == 4 {usize::from(a.source_frame.min(3))}
                    else {usize::from(a.source_frame)};
        ensure!(phase < stride, "light walking source frame");
        let at = (base + usize::from(a.direction) * stride + phase) * 2;
        x += i64::from(WALK_OFFSETS[at]);
        y += i64::from(WALK_OFFSETS[at + 1]);
    }
    Ok([i32::try_from(x).context("light projected X overflow")?,
        i32::try_from(y).context("light projected Y overflow")?])
}

// The pinned original 62E7E8..62EF68 tables are appended by candidate preparation.

const WALK_OFFSETS: [i32; 480] = [
    0, 0, -6, -3, -12, -6, -18, -9, 0, 0, 0, -6, 0, -12, 0, -18,
    0, 0, 6, -3, 12, -6, 18, -9, 0, 0, 12, 0, 24, 0, 36, 0,
    0, 0, 6, 3, 12, 6, 18, 9, 0, 0, 0, 6, 0, 12, 0, 18,
    0, 0, -6, 3, -12, 6, -18, 9, 0, 0, -12, 0, -24, 0, -36, 0,
    0, 0, -2, -1, -4, -2, -6, -3, -8, -4, -10, -5, -12, -6, -14, -7,
    -16, -8, -18, -9, -20, -10, -22, -11, 0, 0, 0, -2, 0, -4, 0, -6,
    0, -8, 0, -10, 0, -12, 0, -14, 0, -16, 0, -18, 0, -20, 0, -22,
    0, 0, 2, -1, 4, -2, 6, -3, 8, -4, 10, -5, 12, -6, 14, -7,
    16, -8, 18, -9, 20, -10, 22, -11, 0, 0, 4, 0, 8, 0, 12, 0,
    16, 0, 20, 0, 24, 0, 28, 0, 32, 0, 36, 0, 40, 0, 44, 0,
    0, 0, 2, 1, 4, 2, 6, 3, 8, 4, 10, 5, 12, 6, 14, 7,
    16, 8, 18, 9, 20, 10, 22, 11, 0, 0, 0, 2, 0, 4, 0, 6,
    0, 8, 0, 10, 0, 12, 0, 14, 0, 16, 0, 18, 0, 20, 0, 22,
    0, 0, -2, 1, -4, 2, -6, 3, -8, 4, -10, 5, -12, 6, -14, 7,
    -16, 8, -18, 9, -20, 10, -22, 11, 0, 0, -4, 0, -8, 0, -12, 0,
    -16, 0, -20, 0, -24, 0, -28, 0, -32, 0, -36, 0, -40, 0, -44, 0,
    0, 0, -4, -2, -8, -4, -12, -6, -16, -8, -20, -10, 0, 0, 0, -4,
    0, -8, 0, -12, 0, -16, 0, -20, 0, 0, 4, -2, 8, -4, 12, -6,
    16, -8, 20, -10, 0, 0, 8, 0, 16, 0, 24, 0, 32, 0, 40, 0,
    0, 0, 4, 2, 8, 4, 12, 6, 16, 8, 20, 10, 0, 0, 0, 4,
    0, 8, 0, 12, 0, 16, 0, 20, 0, 0, -4, 2, -8, 4, -12, 6,
    -16, 8, -20, 10, 0, 0, -8, 0, -16, 0, -24, 0, -32, 0, -40, 0,
    0, 0, -3, -1, -6, -3, -9, -4, -12, -6, -15, -7, -18, -9, -21, -10,
    0, 0, 0, -3, 0, -6, 0, -9, 0, -12, 0, -15, 0, -18, 0, -21,
    0, 0, 3, -1, 6, -3, 9, -4, 12, -6, 15, -7, 18, -9, 21, -10,
    0, 0, 6, 0, 12, 0, 18, 0, 24, 0, 30, 0, 36, 0, 42, 0,
    0, 0, 3, 1, 6, 3, 9, 4, 12, 6, 15, 7, 18, 9, 21, 10,
    0, 0, 0, 3, 0, 6, 0, 9, 0, 12, 0, 15, 0, 18, 0, 21,
    0, 0, -3, 1, -6, 3, -9, 4, -12, 6, -15, 7, -18, 9, -21, 10,
    0, 0, -6, 0, -12, 0, -18, 0, -24, 0, -30, 0, -36, 0, -42, 0,
];
