//! LiTo 3.81C hurt guard, LT-relative graphics, 4E63C0 queue threshold,
//! LTlist action0 multiplier and the native unsigned32 deadline arithmetic.
//!
//! Static source: analysis PE `LiTo.unpacked-oep-current.exe`
//! (SHA256 4271D92F…AB6E9), read with Capstone only; nothing was executed.
//! Addresses below are that image's VAs. Issue #162 report lists the evidence.
//!
//! * `[0x647E90]` (LT base) has exactly one writer, 43DE5A, which stores the
//!   record count returned by 4D3FF0("list.spr"). 4D3FF0 assembles the
//!   *embedded* list from ten static chunks (pointers 0x62B7F0..0x62B814,
//!   size `[0x5091B0]` = 1,189,423), checks CRC16 (table 0x636380) against
//!   `[0x5091B4]` = 0x48CE, then counts 0xF9-separated records until 0xB3.
//!   Re-evaluating those instructions over the image data gives CRC 0x48CE
//!   (match) and 5894 records, equal to the declared `[0x62B818]`. The base is
//!   therefore derived from static data, not guessed from 1/5894/11297.
//! * `[0x647EA4]` (action0 multiplier) has exactly one writer, 4D42AE: the
//!   second 43DCC0 integer of LTlist.spr's first line (4D4170). Its only
//!   reader, 43E898, scales walk-family frame milliseconds by `m/100`.
use anyhow::{ensure, Result};

/// `[0x647E90]`: record count of the CRC-verified embedded list.spr.
pub const LT_BASE: u16 = 5894;
/// 4DF7C0 table at 0x636940 (nine u16 offsets from `LT_BASE`).
pub const LT_RELATIVE: [u16; 9] = [8, 52, 53, 54, 55, 110, 111, 273, 277];
/// 4DF7C0 literal graphics after the table (obelisks).
pub const LT_LITERAL: [u16; 5] = [0xa6b, 0xa6f, 0xa69, 0xa6d, 0xa72];
/// `base+0x1A`: 4E465E, 44ED0C, 44F296, 4237FA, 42F42B, 40E60A.
pub const LT_SPECIAL_FORM: u16 = 26;
/// 40E640: the special form's landed hit spawns `base+0x1B` instead of hurt.
pub const LT_SPECIAL_FORM_EFFECT: u16 = 27;
/// 40E5D5: an LT-relative target's landed hit spawns `base+0x0E`.
pub const LT_RELATIVE_HIT_EFFECT: u16 = 14;
/// 4E461A, 44ECE7, 4237D5, 42F406, 40E6AC: no hurt pose for these bodies.
pub const NO_HURT_BODIES: [u16; 4] = [0x4dd, 0x636, 0x7d1, 0x15f4];
/// 40E70C: their landed hit spawns 0x4E0 instead.
pub const NO_HURT_BODY_EFFECT: u16 = 0x4e0;
/// The native action-command opcode stored by 4E63C0 (`[eax] = 0x20`).
pub const ACTION_COMMAND: u32 = 0x20;
/// The formal LTlist.spr (decoded SHA256 F55779AE…42603, the same member
/// `native_resources::verify_native_catalog_member` pins) starts with the
/// single integer `285`; 43DCC0 returns 0 for the absent second integer.
pub const FORMAL_LTLIST_ACTION0_MULTIPLIER: i32 = 0;

fn lt(offset: u16) -> u16 {
    LT_BASE + offset
}

/// 4DF7C0: LT-relative family or one of the five literal obelisks.
pub fn lt_relative(graphics: u16) -> bool {
    LT_RELATIVE.iter().any(|&o| lt(o) == graphics) || LT_LITERAL.contains(&graphics)
}

/// The shared guard before every native hurt insertion/start: the four
/// literal bodies, 4DF7C0, and the `base+26` special form never take action 2.
/// 4E45F0 repeats it at action start, so a queued hurt cannot bypass it.
pub fn hurt_exempt(graphics: u16) -> bool {
    NO_HURT_BODIES.contains(&graphics)
        || lt_relative(graphics)
        || graphics == lt(LT_SPECIAL_FORM)
}

/// What 40E5A2..40E698 shows when a hit lands on `graphics`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandedHit {
    /// 40E694: ordinary body, `4E63C0(2)`.
    Hurt,
    /// A separate world effect replaces the hurt pose. Spawning it is not
    /// wired in this module; callers must not show a body hurt instead.
    Effect { graphics: u16 },
}
pub fn landed_hit(graphics: u16) -> LandedHit {
    if NO_HURT_BODIES.contains(&graphics) {
        LandedHit::Effect { graphics: NO_HURT_BODY_EFFECT }
    } else if lt_relative(graphics) {
        LandedHit::Effect { graphics: lt(LT_RELATIVE_HIT_EFFECT) }
    } else if graphics == lt(LT_SPECIAL_FORM) {
        LandedHit::Effect { graphics: lt(LT_SPECIAL_FORM_EFFECT) }
    } else {
        LandedHit::Hurt
    }
}

/// 4E63C0: with more than three pending commands, an action-2 command whose
/// predecessor is the same action-2 command is dropped. Everything else is
/// appended (the native array grows without a count limit). `pending` is the
/// actor queue *excluding* the running action: 4E85F0 pops the front before
/// 4E45F0 starts it. The native array also stores non-0x20 commands (for
/// example 4E6480's 0x12); the modern queue only has action commands.
pub fn queue_suppresses(pending: usize, last_action: Option<u8>, action: u8) -> bool {
    pending > 3 && action == 2 && last_action == Some(2)
}

/// 44ECCE..44ED59 / 44F258..44F2E3 effect-frame Hit, and the same order at
/// 4237BC..42382B / 42F3ED..42F45F.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HurtDispatch {
    Ignore,
    /// 4E6B70 true (current action == stance and moving flag): `4E63C0(2)`.
    Enqueue,
    /// Otherwise `4E45F0(2)` immediately replaces the running action;
    /// pending commands stay queued.
    Interrupt,
}
pub struct HurtTarget {
    /// `[+0x10] == 8`.
    pub dead: bool,
    /// 4E8EC0: running or any pending action is 44/45.
    pub action_44_45: bool,
    pub graphics: u16,
    /// 4E6B70.
    pub walking: bool,
}
pub fn dispatch(target: &HurtTarget) -> HurtDispatch {
    if target.dead || target.action_44_45 || hurt_exempt(target.graphics) {
        HurtDispatch::Ignore
    } else if target.walking {
        HurtDispatch::Enqueue
    } else {
        HurtDispatch::Interrupt
    }
}

/// 43DCC0 over one line: skip non-digits (a '-' sets the sign, ' ' keeps it,
/// any other non-digit clears it), then accumulate decimal digits with x86
/// 32-bit wrap. End of line returns 0. Bytes >= 0x80 are rejected because the
/// sign-extended argument of the unresolved ctype import is unqualified.
fn next_integer(line: &[u8], cursor: &mut usize) -> Result<i32> {
    let mut negative = false;
    while let Some(&c) = line.get(*cursor) {
        ensure!(c < 0x80, "LTlist header byte outside the qualified ctype range");
        if c.is_ascii_digit() {
            break;
        }
        negative = match c {
            b'-' => true,
            b' ' => negative,
            _ => false,
        };
        *cursor += 1;
    }
    let mut value = 0i32;
    while let Some(&c) = line.get(*cursor).filter(|c| c.is_ascii_digit()) {
        value = value.wrapping_mul(10).wrapping_add(i32::from(c - b'0'));
        *cursor += 1;
    }
    Ok(if negative { value.wrapping_neg() } else { value })
}

/// 4D4275..4D42AE: copy the first line (to NUL or CR) into a 0x200 stack
/// buffer, discard the first 43DCC0 integer and keep the second. The native
/// copy has no bound; a line that would overrun that buffer is an error here.
pub fn ltlist_action0_multiplier(decoded: &[u8]) -> Result<i32> {
    let end = decoded
        .iter()
        .position(|&b| b == 0 || b == b'\r')
        .unwrap_or(decoded.len());
    ensure!(end < 0x200, "LTlist header overruns the native line buffer");
    let line = &decoded[..end];
    let mut cursor = 0;
    next_integer(line, &mut cursor)?;
    next_integer(line, &mut cursor)
}

/// Walk family checked at 43E8A9..43E8D9 (note: 83 is not in this list).
pub const ACTION0_FAMILY: [u8; 11] = [0, 4, 11, 20, 24, 40, 46, 50, 54, 58, 62];
/// 43E898..43E8F1: `ms * m / 100` with signed truncation, only when `m != 0`
/// and the action is in [`ACTION0_FAMILY`].
pub fn scale_action0_frame_ms(ms: i32, action: u8, multiplier: i32) -> i32 {
    if multiplier == 0 || !ACTION0_FAMILY.contains(&action) {
        return ms;
    }
    // imul ecx,edx keeps the low dword; the /100 is an exact signed quotient.
    ms.wrapping_mul(multiplier) / 100
}

/// 4D65F0: `deadline = clock + delay` as a wrapping dword (4D6610 `add`).
pub fn deadline(now: u32, delay: u32) -> u32 {
    now.wrapping_add(delay)
}

/// 4D6618..4D662F: scan from the back while the existing deadline is
/// (unsigned) greater; insert after the first `<=`. Equal deadlines are FIFO
/// and a wrapped deadline sorts before every un-wrapped one.
pub fn insertion_index(deadlines: &[u32], due: u32) -> usize {
    deadlines.iter().rposition(|&d| d <= due).map_or(0, |i| i + 1)
}

/// 4D6570 drain order over a fixed `cutoff` sample: pop the front while
/// `cutoff >= front` (unsigned `jb` exit). Returns the popped deadlines.
pub fn drain(deadlines: &mut Vec<u32>, cutoff: u32) -> Vec<u32> {
    let n = deadlines
        .iter()
        .take_while(|&&d| super::native_clock::due(d, cutoff))
        .count();
    deadlines.drain(..n).collect()
}

#[cfg(test)]
#[path = "native_hurt_tests.rs"]
mod tests;
