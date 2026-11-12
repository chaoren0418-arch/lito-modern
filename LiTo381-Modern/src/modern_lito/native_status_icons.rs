//! T046-2: buff／AC／cooldown／MP presentation and authoritative status leases.
//!
//! Every duration comes from a server packet; the client never starts, extends
//! or invents a buff. Sources (read-only, lito-launcher-server):
//! - ICONS1 `S_EVENT(0x7B) c(20) d(204) 204×c`: `tools/combat-continuation/
//!   integration-lifecycle/NativeSourceIcons1.java` (`RULES`, `WEAPONS`,
//!   `duration()`; 36 distinct skills).
//! - ICON_I2H `0x7B c(40) h(seconds)`: `NativeSourceProc68Icon.java` (skill 68).
//! - Doll `0x7B c(56) h(seconds)`: `Protocol.magicDollIcon`, launcher
//!   `native/shared_packetbox.inc` (zero clears, >32767 ignored).
//! - `S_MAGE_SHIELD(0x6D) h(seconds) c(type)`: `Protocol.mageShield` (LiTo
//!   receiver sub_4AFF30); launcher `native/dragon_status.inc`: type 181 is
//!   Dragon Skin with shared icon **3080** (effectlist2 effect107, not spell
//!   icon 3079); nine illusion leases 201…219 with their icons; seconds>32767
//!   is refused; zero removes; a repeat refreshes without a second entry.
//! - `S_SPEED(0x29) d c h`: `Protocol.speed` / `NativeSourceDollSlowWire`:
//!   the local actor keeps the full `writeH` duration as its client timer
//!   (doll 29 slow included); zero with a non-normal mode is permanent.
//!
//! Routing note: LiTo.bin's packet-box receiver is `sub_4B07C0` on 0x7B
//! (0..40 switch); 0xFA stays the LiTo clan label (`shared_packetbox.inc`).
use anyhow::{bail, ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

pub const S_EVENT: u8 = 0x7b;
pub const S_MAGE_SHIELD: u8 = 0x6d;
pub const ICONS1: u8 = 20;
pub const ICON_I2H: u8 = 40;
pub const DOLL: u8 = 56;
pub const ICONS1_LEN: usize = 204;
pub const IMMUNE_TO_HARM: u16 = 68;
pub const DRAGON_SKIN: u8 = 181;
/// Shared status icon (effectlist2.xml effect107 DRAGON_SKIN_S); 3079 is the spell icon.
pub const DRAGON_SKIN_ICON: u16 = 3080;
pub const ILLUSION_SKILLS: [u8; 9] = [201, 204, 206, 209, 211, 214, 216, 217, 219];
pub const ILLUSION_ICONS: [u16; 9] = [3123, 3117, 3097, 3115, 3127, 3113, 3119, 3125, 3111];
/// Native S_MAGE_SHIELD/packet-box refusal bound (`seconds>32767`).
pub const MAX_LEASE_SECONDS: u16 = 32767;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    pub skill: u16,
    pub slot: usize,
    pub mode: u8,
}
const fn r(skill: u16, slot: usize, mode: u8) -> Rule {
    Rule { skill, slot, mode }
}
/// `NativeSourceIcons1.RULES`, same order.
pub const ICONS1_RULES: [Rule; 31] = [
    r(32, 0, 0), r(31, 1, 0), r(14, 3, 0), r(71, 4, 0), r(78, 5, 0), r(104, 7, 0),
    r(47, 8, 2), r(56, 9, 2), r(111, 17, 0), r(55, 18, 0), r(158, 19, 0), r(167, 20, 0),
    r(153, 21, 0), r(134, 22, 0), r(176, 23, 3), r(133, 24, 4), r(171, 26, 0), r(155, 27, 0),
    r(174, 30, 0), r(175, 31, 0), r(173, 32, 0), r(206, 52, 1), r(216, 53, 1), r(217, 54, 1),
    r(191, 55, 0), r(193, 56, 0), r(188, 57, 0), r(211, 58, 0), r(183, 59, 0), r(181, 60, 1),
    r(1017, 61, 0),
];
/// Shared weapon slot 10 (duration, mode 0) and slot 11 (`skill-1`).
pub const ICONS1_WEAPONS: [u16; 6] = [148, 155, 163, 149, 156, 166];
const WEAPON_SLOT: usize = 10;
const WEAPON_SKILL_SLOT: usize = 11;
const MARKER_SLOT: usize = 25;
const EPOCH: std::ops::Range<usize> = 72..76;

/// The 36 distinct ICONS1 states (31 rules plus five weapon-only skills).
pub fn icons1_skills() -> BTreeSet<u16> {
    ICONS1_RULES.iter().map(|r| r.skill).chain(ICONS1_WEAPONS).collect()
}

/// Lower bound of the encoded duration. Mode 2 bytes below 16 are either
/// `max(1,seconds/2)` (<32 s) or `seconds/4` (>=32 s); `raw*2` is a floor
/// for both, except raw 1 which also encodes a single second.
pub fn seconds_floor(raw: u8, mode: u8) -> Result<u32> {
    let raw = u32::from(raw);
    Ok(match mode {
        0 | 4 => raw * 4,
        1 | 3 => raw * 16,
        2 if raw == 1 => 1,
        2 if raw < 16 => raw * 2,
        2 => raw * 4,
        _ => bail!("unknown ICONS1 duration mode {mode}"),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Icons1Entry {
    pub skill: u16,
    pub raw: u8,
    pub seconds: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Icons1 {
    pub entries: Vec<Icons1Entry>,
    pub epoch_seconds: u32,
}

/// Strict ICONS1 decode: every byte outside the Source layout must be zero.
pub fn decode_icons1(b: &[u8]) -> Result<Icons1> {
    ensure!(b.len() == 6 + ICONS1_LEN && b[0] == S_EVENT && b[1] == ICONS1, "ICONS1 exact 210-byte envelope");
    ensure!(u32::from_le_bytes(b[2..6].try_into()?) == ICONS1_LEN as u32, "ICONS1 data length");
    let data = &b[6..];
    let mut owned = [false; ICONS1_LEN];
    let mut entries = Vec::new();
    for rule in ICONS1_RULES {
        owned[rule.slot] = true;
        let raw = data[rule.slot];
        if raw != 0 {
            entries.push(Icons1Entry { skill: rule.skill, raw, seconds: seconds_floor(raw, rule.mode)? });
        }
    }
    owned[MARKER_SLOT] = true;
    ensure!(matches!(data[MARKER_SLOT], 0 | 3), "ICONS1 slot25 marker");
    ensure!(data[24] == 0 || data[MARKER_SLOT] == 3, "ICONS1 slot24 without marker");
    owned[WEAPON_SLOT] = true;
    owned[WEAPON_SKILL_SLOT] = true;
    let weapon = u16::from(data[WEAPON_SKILL_SLOT]) + 1;
    if data[WEAPON_SLOT] != 0 || data[WEAPON_SKILL_SLOT] != 0 {
        ensure!(ICONS1_WEAPONS.contains(&weapon), "ICONS1 unknown weapon skill {weapon}");
    }
    if data[WEAPON_SLOT] != 0 {
        let raw = data[WEAPON_SLOT];
        entries.retain(|e| e.skill != weapon);
        entries.push(Icons1Entry { skill: weapon, raw, seconds: seconds_floor(raw, 0)? });
    }
    for i in EPOCH {
        owned[i] = true;
    }
    if let Some(i) = (0..ICONS1_LEN).find(|i| !owned[*i] && data[*i] != 0) {
        bail!("ICONS1 unassigned slot {i} is not zero");
    }
    entries.sort_by_key(|e| e.skill);
    Ok(Icons1 { entries, epoch_seconds: u32::from_le_bytes(data[EPOCH].try_into()?) })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum Key {
    Icons1(u16),
    ImmuneToHarm,
    Doll,
    Shield(u8),
    Speed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Lease {
    pub received_ms: u64,
    /// `None` is a server-permanent state (only S_SPEED zero with a mode).
    pub seconds: Option<u32>,
    /// Speed mode (1 haste, 2 slow); zero for other keys.
    pub mode: u8,
}
impl Lease {
    pub fn expires_ms(self) -> Option<u64> {
        self.seconds.map(|s| self.received_ms.saturating_add(u64::from(s) * 1000))
    }
    pub fn remaining_seconds(self, now_ms: u64) -> Option<u32> {
        self.expires_ms().map(|e| (e.saturating_sub(now_ms).div_ceil(1000)) as u32)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct View {
    pub key: Key,
    pub label: String,
    pub icon: Option<u16>,
    pub remaining_seconds: Option<u32>,
}

pub fn shield_icon(kind: u8) -> Option<u16> {
    if kind == DRAGON_SKIN {
        return Some(DRAGON_SKIN_ICON);
    }
    ILLUSION_SKILLS.iter().position(|s| *s == kind).map(|i| ILLUSION_ICONS[i])
}

pub fn label(key: Key, mode: u8) -> String {
    let skill = |id: u16| super::native_skill_selection::catalog::name(id).map(str::to_owned).unwrap_or_else(|| format!("技能 {id}"));
    match key {
        Key::Icons1(1017) => "果實勇敢".into(),
        Key::Icons1(id) => skill(id),
        Key::ImmuneToHarm => skill(IMMUNE_TO_HARM),
        Key::Doll => "魔法娃娃".into(),
        Key::Shield(5) => "護盾狀態（SHIELD）".into(),
        Key::Shield(6) => "大地護膚狀態（EARTH_SKIN）".into(),
        Key::Shield(kind) if shield_icon(kind).is_some() => skill(u16::from(kind)),
        Key::Shield(kind) => format!("狀態（類型 {kind}）"),
        Key::Speed if mode == 2 => "緩速".into(),
        Key::Speed => "加速".into(),
    }
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Leases {
    leases: BTreeMap<Key, Lease>,
    pub icons1_epoch: Option<u32>,
    /// Increments only when the visible lease set or a duration changes.
    pub revision: u64,
    /// Native-refused durations (>32767) that changed nothing.
    pub refused: u64,
}
impl Leases {
    pub fn get(&self, key: Key) -> Option<Lease> {
        self.leases.get(&key).copied()
    }
    pub fn len(&self) -> usize {
        self.leases.len()
    }
    pub fn is_empty(&self) -> bool {
        self.leases.is_empty()
    }
    fn set(&mut self, key: Key, lease: Option<Lease>) -> bool {
        let changed = match lease {
            Some(lease) => self.leases.insert(key, lease) != Some(lease),
            None => self.leases.remove(&key).is_some(),
        };
        if changed {
            self.revision += 1;
        }
        changed
    }
    fn timed(&mut self, key: Key, seconds: u16, now_ms: u64) -> bool {
        if seconds > MAX_LEASE_SECONDS {
            self.refused += 1;
            return false;
        }
        let lease = (seconds != 0).then_some(Lease { received_ms: now_ms, seconds: Some(seconds.into()), mode: 0 });
        self.set(key, lease)
    }
    /// 0x7B subtypes 20／40／56. Other subtypes are not status packets.
    pub fn accept_event(&mut self, b: &[u8], now_ms: u64) -> Result<bool> {
        ensure!(b.first() == Some(&S_EVENT), "not S_EVENT");
        match *b.get(1).context("S_EVENT subtype")? {
            ICONS1 => {
                let icons = decode_icons1(b)?;
                let mut changed = false;
                let stale: Vec<_> = self.leases.keys().copied().filter(|k| matches!(k, Key::Icons1(_))).collect();
                for key in stale {
                    if !icons.entries.iter().any(|e| Key::Icons1(e.skill) == key) {
                        changed |= self.set(key, None);
                    }
                }
                for e in &icons.entries {
                    let lease = Lease { received_ms: now_ms, seconds: Some(e.seconds), mode: 0 };
                    changed |= self.set(Key::Icons1(e.skill), Some(lease));
                }
                self.icons1_epoch = Some(icons.epoch_seconds);
                Ok(changed)
            }
            kind @ (ICON_I2H | DOLL) => {
                ensure!(b.len() == 4, "S_EVENT {kind} exact h(seconds)");
                let seconds = u16::from_le_bytes([b[2], b[3]]);
                Ok(self.timed(if kind == ICON_I2H { Key::ImmuneToHarm } else { Key::Doll }, seconds, now_ms))
            }
            other => bail!("S_EVENT subtype {other} is not a status icon"),
        }
    }
    /// `S_MAGE_SHIELD h(seconds) c(type)`.
    pub fn accept_shield(&mut self, b: &[u8], now_ms: u64) -> Result<bool> {
        ensure!(b.len() == 4 && b[0] == S_MAGE_SHIELD, "S_MAGE_SHIELD exact h c");
        Ok(self.timed(Key::Shield(b[3]), u16::from_le_bytes([b[1], b[2]]), now_ms))
    }
    /// Own-actor `S_SPEED c(mode) h(seconds)`; the full word is the timer.
    pub fn speed(&mut self, mode: u8, seconds: u16, now_ms: u64) -> bool {
        let lease = (mode != 0).then_some(Lease {
            received_ms: now_ms,
            seconds: (seconds != 0).then_some(seconds.into()),
            mode,
        });
        self.set(Key::Speed, lease)
    }
    /// Removes leases whose server duration elapsed; returns the retired keys.
    pub fn poll(&mut self, now_ms: u64) -> Vec<Key> {
        let expired: Vec<_> = self
            .leases
            .iter()
            .filter(|(_, l)| l.expires_ms().is_some_and(|e| now_ms >= e))
            .map(|(k, _)| *k)
            .collect();
        for key in &expired {
            self.set(*key, None);
        }
        expired
    }
    pub fn views(&self, now_ms: u64) -> Vec<View> {
        self.leases
            .iter()
            .filter(|(_, l)| l.expires_ms().is_none_or(|e| now_ms < e))
            .map(|(key, l)| View {
                key: *key,
                label: label(*key, l.mode),
                icon: match key {
                    Key::Shield(kind) => shield_icon(*kind),
                    _ => None,
                },
                remaining_seconds: l.remaining_seconds(now_ms),
            })
            .collect()
    }
}

/// One-line HUD text: AC, MP, skill reuse and up to six server leases.
pub fn summary(world: &super::native_world::World, now_ms: u64, cooldown_ms: Option<u64>) -> String {
    summary_with_mp(world, world.mp, now_ms, cooldown_ms)
}

/// Same as [`summary`], but shows the caller's presented MP (the HUD value
/// gated behind the own action that caused it) instead of the raw `world.mp`.
pub fn summary_with_mp(world: &super::native_world::World, mp: Option<[u16; 2]>, now_ms: u64, cooldown_ms: Option<u64>) -> String {
    let ac = world.status.map(|s| s.armor_class.to_string()).unwrap_or_else(|| "—".into());
    let mp = mp.map(|[c, m]| format!("{c}/{m}")).unwrap_or_else(|| "等待資料".into());
    let cooldown = match cooldown_ms {
        Some(ms) if ms > 0 => format!("冷卻 {}.{} 秒", ms / 1000, (ms % 1000) / 100),
        _ => "技能就緒".into(),
    };
    let views = world.status_icons.views(now_ms);
    let mut parts: Vec<String> = views
        .iter()
        .take(6)
        .map(|v| {
            let icon = v.icon.map(|i| format!("〔圖示 {i}〕")).unwrap_or_default();
            match v.remaining_seconds {
                Some(s) => format!("{}{icon} {s} 秒", v.label),
                None => format!("{}{icon}（持續）", v.label),
            }
        })
        .collect();
    if views.len() > 6 {
        parts.push(format!("等 {} 項", views.len()));
    }
    let buffs = if parts.is_empty() { "無".into() } else { parts.join("、") };
    format!("AC {ac}　MP {mp}　{cooldown}　狀態：{buffs}")
}

#[cfg(test)]
#[path = "native_status_icons_tests.rs"]
mod tests;
