//! Per-request LFC1 intent, appended to plaintext before the existing cipher/frame owner.
use super::native_login::{LoginPhase, Snapshot};

/// Sampled for each UI dispatch (including each held-attack repeat), never latched.
/// Non-UI callers default to ordinary requests.
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub foreground: bool,
    pub ctrl: bool,
}

/// Mirrors native/forced_combat.h object layouts. A seven-byte body alone is
/// insufficient: coordinates, clocks, selectors and item IDs are not actors.
fn object_target(body: &[u8]) -> Option<u32> {
    let offset = match (body.first().copied()?, body.len()) {
        (0x17 | 0x18 | 0x7a, 9) => 1,
        (0x14, 7 | 11) => {
            let skill = u16::from(body[1]) * 8 + u16::from(body[2]) + 1;
            if body[2] > 7
                || skill > 220
                || matches!(skill, 5 | 12 | 21 | 51 | 58 | 60 | 63 | 69 | 100 | 116 | 118)
            {
                return None;
            }
            3
        }
        _ => return None,
    };
    let target = u32::from_le_bytes(body[offset..offset + 4].try_into().unwrap());
    (target != 0).then_some(target)
}

pub(crate) fn append(body: &mut Vec<u8>, input: Input, state: &Snapshot) {
    if !input.foreground || !input.ctrl
        || state.phase != LoginPhase::BattlePending
        // Native room mode 0 is Pet Race; missing/unknown modes fail closed.
        || !state.joined_room.as_ref().is_some_and(|room| matches!(room.mode, 1..=5))
        || state.observer || state.world.observer_mode
        || state.battle_withdrawal_pending || !state.world.ready()
        || state.world.battle_result.is_some()
        || object_target(body).is_none()
    {
        return;
    }
    let opcode = body[0];
    body.extend_from_slice(&[0x4c, 0x46, 0x43, 0x31, opcode, 0x01, 0x67, 0xa5]);
    // native_connection::Session encrypts the extended body exactly once, then
    // native_wire::frame writes its new body.len() + 2 little-endian length.
}

#[cfg(test)]
#[path = "native_forced_combat_tests.rs"]
pub(crate) mod tests;
