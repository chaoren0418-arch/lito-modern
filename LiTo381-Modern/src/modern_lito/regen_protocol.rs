//! v12 authoritative natural recovery. These are applied amounts, not local
//! timers or raw RNG rolls. Negative recovery is not a combat hurt event.
use super::{
    application_protocol::{self as app, ApplicationError},
    combat_protocol as combat, item_protocol as item, world_protocol as world,
};

pub const STATE_BYTES: usize = 104;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub entity_id: u64,
    pub hp_gain: i64,
    pub hp_loss: i64,
    pub hp_revision: i64,
    pub hp_before: i64,
    pub hp_after: i64,
    pub mp_gain: i64,
    pub mp_loss: i64,
    pub mp_revision: i64,
    pub mp_before: i64,
    pub mp_after: i64,
    pub item_gain: i64,
    pub damage_loss: i64,
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid authoritative natural recovery",
    )
}
impl State {
    pub fn initial(entity_id: u64) -> Self {
        Self {
            entity_id,
            hp_gain: 0,
            hp_loss: 0,
            hp_revision: 0,
            hp_before: 0,
            hp_after: 0,
            mp_gain: 0,
            mp_loss: 0,
            mp_revision: 0,
            mp_before: 0,
            mp_after: 0,
            item_gain: 0,
            damage_loss: 0,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if self.entity_id == 0
            || self.entity_id > i64::MAX as u64
            || self.item_gain < 0
            || self.damage_loss < 0
            || !axis_valid(
                self.hp_gain,
                self.hp_loss,
                self.hp_revision,
                self.hp_before,
                self.hp_after,
                1,
            )
            || !axis_valid(
                self.mp_gain,
                self.mp_loss,
                self.mp_revision,
                self.mp_before,
                self.mp_after,
                0,
            )
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn notice(&self) -> &'static str {
        "自然 HP／MP 恢復由伺服器確認；可能為負值，不在本機倒數或推算。"
    }
}
fn axis_valid(gain: i64, loss: i64, revision: i64, before: i64, after: i64, minimum: i64) -> bool {
    if gain < 0 || loss < 0 || revision < 0 {
        return false;
    }
    if revision == 0 {
        return gain == 0 && loss == 0 && before == 0 && after == 0;
    }
    before >= minimum
        && after >= minimum
        && before != after
        && if after > before {
            gain >= after - before
        } else {
            loss >= before - after
        }
}
pub(crate) fn spell_spent(f: &combat::Frame, entity: u64) -> i128 {
    i128::from(f.world.cast(entity).map_or(0, |s| s.mp_spent))
        + i128::from(f.damage(entity).map_or(0, |s| s.mp_spent))
        + i128::from(super::honor_protocol::heal_spent(f, entity))
        + i128::from(super::fire_storm_protocol::spent(f, entity))
        + i128::from(f.npc_spell(entity).map_or(0, |s| s.mp_spent))
        + i128::from(super::disintegrate_protocol::spent(f, entity))
        + i128::from(f.npc_disintegrate(entity).map_or(0, |s| s.mp_spent))
        + i128::from(f.npc_fire(entity).map_or(0, |s| s.mp_spent))
}
pub fn validate_frame(f: &combat::Frame) -> Result<(), ApplicationError> {
    if f.regen.is_empty() {
        return Ok(());
    }
    if f.regen.len() != f.world.actors.len() || f.regen.len() != f.vitals.len() {
        return Err(invalid());
    }
    for ((s, actor), v) in f.regen.iter().zip(&f.world.actors).zip(&f.vitals) {
        s.validate()?;
        let initial = State::initial(s.entity_id);
        let use_state = f.item(s.entity_id);
        let use_rev = use_state.map_or(0, |i| i.use_revision);
        let tp_rev = f.world.cast(s.entity_id).map_or(0, |c| c.cast_revision);
        let magic_rev = f.damage(s.entity_id).map_or(0, |c| c.cast_revision as i64);
        let heal_rev = super::honor_protocol::heal_revision(f, s.entity_id);
        let fire_rev = super::fire_storm_protocol::revision(f, s.entity_id);
        let npc_rev = f.npc_spell(s.entity_id).map_or(0, |s| s.revision);
        let dis_rev = super::disintegrate_protocol::revision(f, s.entity_id);
        let npc_dis_rev = f.npc_disintegrate(s.entity_id).map_or(0, |s| s.revision);
        let npc_fire_rev = f.npc_fire(s.entity_id).map_or(0, |s| s.revision);
        let respawn_rev = super::quest_life::respawn_revision(f, s.entity_id);
        if s.entity_id != actor.entity_id
            || s.hp_revision > f.world.revision
            || s.mp_revision > f.world.revision
            || s.hp_before > v.max_hp
            || s.hp_after > v.max_hp
            || s.mp_before > v.max_mp
            || s.mp_after > v.max_mp
            || [s.hp_revision, s.mp_revision].into_iter().any(|r| {
                r > 0
                    && [
                        use_rev,
                        v.hurt_revision,
                        tp_rev,
                        magic_rev,
                        heal_rev,
                        fire_rev,
                        npc_rev,
                        dis_rev,
                        npc_dis_rev,
                        npc_fire_rev,
                        respawn_rev,
                    ]
                    .contains(&r)
            })
            || v.death_revision > 0
                && [s.hp_revision, s.mp_revision]
                    .into_iter()
                    .any(|r| r >= v.death_revision && (v.hp == 0 || r <= respawn_rev))
            || (s.damage_loss == 0) != (v.hurt_revision == 0)
            || !item_total_valid(use_state, s.item_gain)
            || i128::from(v.max_hp) + i128::from(s.hp_gain) - i128::from(s.hp_loss)
                + i128::from(s.item_gain)
                + i128::from(super::quest_life::respawn_gain(f, s.entity_id))
                - i128::from(s.damage_loss)
                != i128::from(v.hp)
            || i128::from(v.max_mp) + i128::from(s.mp_gain)
                - i128::from(s.mp_loss)
                - spell_spent(f, s.entity_id)
                != i128::from(v.mp)
            || matches!(
                f.world.phase,
                world::Phase::Loading | world::Phase::Countdown
            ) && *s != initial
            || actor.identity().character().is_none() && {
                let mut bot = s.clone();
                bot.damage_loss = 0;
                bot != initial
            }
            || f.world.completion.as_ref().is_some_and(|c| {
                c.event
                    .engine_revision
                    .checked_add(1)
                    .is_none_or(|r| s.hp_revision > r || s.mp_revision > r)
            })
        {
            return Err(invalid());
        }
        // The latest HP-changing positive/negative recovery or USE is an exact
        // anchor. Only a later combat hurt can lower it; no other healing exists.
        let (mut anchor_rev, mut anchor_hp) = if s.hp_revision > use_rev {
            (s.hp_revision, s.hp_after)
        } else {
            (use_rev, use_state.map_or(0, |i| i.hp_after))
        };
        if respawn_rev > anchor_rev {
            anchor_rev = respawn_rev;
            anchor_hp = v.max_hp;
        }
        if anchor_rev > 0
            && (v.hp > anchor_hp || v.hurt_revision <= anchor_rev && v.hp != anchor_hp)
        {
            return Err(invalid());
        }
        if s.mp_revision > 0
            && (v.mp > s.mp_after
                || tp_rev
                    .max(magic_rev)
                    .max(heal_rev)
                    .max(fire_rev)
                    .max(npc_rev)
                    .max(dis_rev)
                    .max(npc_dis_rev)
                    .max(npc_fire_rev)
                    <= s.mp_revision
                    && v.mp != s.mp_after)
        {
            return Err(invalid());
        }
    }
    Ok(())
}
fn item_total_valid(state: Option<&item::ItemState>, gain: i64) -> bool {
    let Some(state) = state else {
        return gain == 0;
    };
    if state.use_count == 0 {
        return gain == 0;
    }
    if state.use_count == 1 {
        return gain == state.restored;
    }
    let maximum = state
        .stocks
        .iter()
        .try_fold(0i128, |sum, s| {
            let used = s.initial.checked_sub(s.remaining)?;
            if used < 0 {
                return None;
            }
            let bound = item::healing(s.item_id).map_or(0, |k| {
                i128::from(k.maximum) + i128::from(state.fixed_recovery)
            });
            sum.checked_add(i128::from(used).checked_mul(bound)?)
        })
        .and_then(|sum| {
            if let Some(k) = item::healing(state.item_id) {
                sum.checked_sub(i128::from(k.maximum) + i128::from(state.fixed_recovery))?
                    .checked_add(i128::from(state.restored))
            } else {
                Some(sum)
            }
        });
    gain >= state.restored && maximum.is_some_and(|max| i128::from(gain) <= max)
}
fn axis_follows(
    a: (i64, i64, i64, i64, i64),
    b: (i64, i64, i64, i64, i64),
    old_world: i64,
) -> bool {
    let (ag, al, ar, _, _) = a;
    let (bg, bl, br, before, after) = b;
    if bg < ag || bl < al || br < ar {
        return false;
    }
    if br == ar {
        return a == b;
    }
    br > old_world
        && (bg > ag || bl > al)
        && if after > before {
            bg - ag >= after - before
        } else {
            bl - al >= before - after
        }
}
/// Keep the existing potion proof: skipped rolls have bounded maxima and the
/// newest consumed bottle reports its exact applied restoration.
fn item_gain_follows(a: &combat::Frame, b: &combat::Frame, entity: u64, gain: i64) -> bool {
    let Some((old, new)) = a.item(entity).zip(b.item(entity)) else {
        return gain == 0;
    };
    if new.use_count == old.use_count {
        return gain == 0;
    }
    if new.use_count < old.use_count || old.stocks.len() != new.stocks.len() {
        return false;
    }
    if new.use_count - old.use_count == 1 {
        return gain == new.restored;
    }
    let maximum = old
        .stocks
        .iter()
        .zip(&new.stocks)
        .try_fold(0i128, |sum, (a, b)| {
            let used = a.remaining.checked_sub(b.remaining)?;
            if used < 0 {
                return None;
            }
            let bound = item::healing(a.item_id).map_or(0, |k| {
                i128::from(k.maximum) + i128::from(new.fixed_recovery)
            });
            sum.checked_add(i128::from(used).checked_mul(bound)?)
        })
        .and_then(|sum| {
            if let Some(k) = item::healing(new.item_id) {
                sum.checked_sub(i128::from(k.maximum) + i128::from(new.fixed_recovery))?
                    .checked_add(i128::from(new.restored))
            } else {
                Some(sum)
            }
        });
    gain >= new.restored && maximum.is_some_and(|max| i128::from(gain) <= max)
}
pub fn follows(a: &combat::Frame, b: &combat::Frame) -> bool {
    if a.regen.len() != b.regen.len() {
        return false;
    }
    a.regen.iter().zip(&b.regen).all(|(old, new)| {
        let Some((av, bv)) = a.vital(old.entity_id).zip(b.vital(new.entity_id)) else {
            return false;
        };
        if old.entity_id != new.entity_id
            || new.item_gain < old.item_gain
            || new.damage_loss < old.damage_loss
            || (av.hp == 0 && !super::quest_life::respawned(a, b, old.entity_id)
                || matches!(
                    a.world.phase,
                    world::Phase::Finished | world::Phase::Aborted
                ))
                && old != new
            || !axis_follows(
                (
                    old.hp_gain,
                    old.hp_loss,
                    old.hp_revision,
                    old.hp_before,
                    old.hp_after,
                ),
                (
                    new.hp_gain,
                    new.hp_loss,
                    new.hp_revision,
                    new.hp_before,
                    new.hp_after,
                ),
                a.world.revision,
            )
            || !axis_follows(
                (
                    old.mp_gain,
                    old.mp_loss,
                    old.mp_revision,
                    old.mp_before,
                    old.mp_after,
                ),
                (
                    new.mp_gain,
                    new.mp_loss,
                    new.mp_revision,
                    new.mp_before,
                    new.mp_after,
                ),
                a.world.revision,
            )
            || !item_gain_follows(a, b, old.entity_id, new.item_gain - old.item_gain)
            || (new.damage_loss > old.damage_loss) != (bv.hurt_revision > av.hurt_revision)
            || new.damage_loss > old.damage_loss && bv.hurt_revision <= a.world.revision
        {
            return false;
        }
        let hp = i128::from(av.hp) + i128::from(new.hp_gain - old.hp_gain)
            - i128::from(new.hp_loss - old.hp_loss)
            + i128::from(new.item_gain - old.item_gain)
            + i128::from(super::quest_life::respawn_gain(b, new.entity_id))
            - i128::from(super::quest_life::respawn_gain(a, old.entity_id))
            - i128::from(new.damage_loss - old.damage_loss);
        let mp = i128::from(av.mp) + i128::from(new.mp_gain - old.mp_gain)
            - i128::from(new.mp_loss - old.mp_loss)
            - (spell_spent(b, new.entity_id) - spell_spent(a, old.entity_id));
        hp == i128::from(bv.hp) && mp == i128::from(bv.mp)
    })
}
pub(crate) fn hp_follows(a: &combat::Frame, b: &combat::Frame, index: usize) -> bool {
    let Some((x, y)) = a.regen.get(index).zip(b.regen.get(index)) else {
        return false;
    };
    i128::from(a.vitals[index].hp) + i128::from(y.hp_gain)
        - i128::from(x.hp_gain)
        - i128::from(y.hp_loss)
        + i128::from(x.hp_loss)
        + i128::from(y.item_gain)
        - i128::from(x.item_gain)
        - i128::from(y.damage_loss)
        + i128::from(x.damage_loss)
        + i128::from(super::quest_life::respawn_gain(b, y.entity_id))
        - i128::from(super::quest_life::respawn_gain(a, x.entity_id))
        == i128::from(b.vitals[index].hp)
}
pub(crate) fn mp_follows(a: &combat::Frame, b: &combat::Frame, index: usize) -> bool {
    let Some((x, y)) = a.regen.get(index).zip(b.regen.get(index)) else {
        return false;
    };
    i128::from(a.vitals[index].mp) + i128::from(y.mp_gain)
        - i128::from(x.mp_gain)
        - i128::from(y.mp_loss)
        + i128::from(x.mp_loss)
        - (spell_spent(b, y.entity_id) - spell_spent(a, x.entity_id))
        == i128::from(b.vitals[index].mp)
}
pub(crate) fn negotiated(response: &app::Response, enabled: bool) -> bool {
    let frame = match response {
        app::Response::Combat(r) => r.frame.as_ref(),
        app::Response::Siege(r) => r.frame.as_ref(),
        app::Response::Cast(r) => r.frame.as_ref(),
        app::Response::Item(r) => r.frame.as_ref(),
        app::Response::Ammo(r) => r.frame.as_ref(),
        _ => return true,
    };
    frame.is_none_or(|f| {
        f.world.feature_presence_matches(
            super::world_protocol::MatchDescriptor::REGEN,
            !f.regen.is_empty(),
            enabled,
        )
    })
}
pub(crate) fn decode_states(
    bytes: &[u8],
    actors: usize,
    sparse: bool,
) -> Result<(Vec<State>, usize), ApplicationError> {
    if bytes.len() < 8 || bytes[2..8].iter().any(|n| *n != 0) {
        return Err(invalid());
    }
    let count = u16::from_be_bytes(bytes[..2].try_into().unwrap()) as usize;
    if count > 100 || count != 0 && count != actors || bytes.len() - 8 < count * STATE_BYTES {
        return Err(invalid());
    }
    let mut rows = Vec::with_capacity(count);
    let mut previous = 0;
    for index in 0..count {
        let b = &bytes[8 + index * STATE_BYTES..8 + (index + 1) * STATE_BYTES];
        let n = |i| i64::from_be_bytes(b[i..i + 8].try_into().unwrap());
        let row = State {
            entity_id: n(0) as u64,
            hp_gain: n(8),
            hp_loss: n(16),
            hp_revision: n(24),
            hp_before: n(32),
            hp_after: n(40),
            mp_gain: n(48),
            mp_loss: n(56),
            mp_revision: n(64),
            mp_before: n(72),
            mp_after: n(80),
            item_gain: n(88),
            damage_loss: n(96),
        };
        row.validate()?;
        if row.entity_id <= previous
            || row.entity_id > 100
            || !sparse && row.entity_id != index as u64 + 1
        {
            return Err(invalid());
        }
        previous = row.entity_id;
        rows.push(row);
    }
    Ok((rows, 8 + count * STATE_BYTES))
}

#[cfg(test)]
#[path = "regen_protocol_tests.rs"]
pub(crate) mod tests;
