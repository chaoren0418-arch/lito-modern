//! Catch-up observations may contain several casts/equipment eras. Adjacent
//! command-only ledger transitions cannot be reused as polling validators.
use super::*;
fn flow(a: &Flows, b: &Flows) -> bool {
    [
        a.hp_gain,
        a.hp_loss,
        a.restored,
        a.damaged,
        a.hp_granted,
        a.mp_gain,
        a.mp_loss,
        a.spent,
        a.mp_granted,
        a.hp_clipped,
        a.mp_clipped,
    ]
    .iter()
    .zip([
        b.hp_gain,
        b.hp_loss,
        b.restored,
        b.damaged,
        b.hp_granted,
        b.mp_gain,
        b.mp_loss,
        b.spent,
        b.mp_granted,
        b.hp_clipped,
        b.mp_clipped,
    ])
    .all(|(a, b)| b >= *a)
}
fn resources(a: &Resources, b: &Resources, rev: i64, delta: i64) -> bool {
    event(a, b, a.revision, b.revision, rev)
        && a.birth_hp == b.birth_hp
        && a.birth_mp == b.birth_mp
        && advances(a.life, b.life, delta)
        && advances(a.equipment_revision, b.equipment_revision, delta)
        && flow(&a.flows, &b.flows)
        && (a.equipment_revision != b.equipment_revision
            || a.vitals.maximum_hp == b.vitals.maximum_hp
                && a.vitals.maximum_mp == b.vitals.maximum_mp
                && a.flows.hp_clipped == b.flows.hp_clipped
                && a.flows.mp_clipped == b.flows.mp_clipped)
}
fn spell(a: &Spell, b: &Spell, rev: i64, delta: i64) -> bool {
    let x = &a.ledger;
    let y = &b.ledger;
    if a.lane != b.lane
        || !advances(x.casts, y.casts, delta)
        || y.spent < x.spent
        || y.prior_casts < x.prior_casts
        || y.prior_spent < x.prior_spent
        || !advances(x.equipment_revision, y.equipment_revision, delta)
        || y.profile_revision < x.profile_revision
    {
        return false;
    }
    if x.equipment_revision == y.equipment_revision {
        if x.profile_revision != y.profile_revision
            || x.cost != y.cost
            || x.prior_casts != y.prior_casts
            || x.prior_spent != y.prior_spent
        {
            return false;
        }
    } else if y.profile_revision <= rev || y.prior_casts < x.casts || y.prior_spent < x.spent {
        return false;
    }
    if x.casts == y.casts {
        return x.spent == y.spent && a.last == b.last && a.receipt == b.receipt;
    }
    let Some(last) = &b.last else {
        return false;
    };
    last.revision > rev
        && last.sequence > x.last_cast_sequence
        && y.spent - x.spent >= i64::from(last.cost)
        && (y.casts - x.casts != 1 || y.spent - x.spent == i64::from(last.cost))
        && a.last.as_ref().is_none_or(|old| {
            last.before.life >= old.before.life
                && last.before.equipment_revision >= old.before.equipment_revision
        })
}
fn supply(a: &Supplies, b: &Supplies, rev: i64, delta: i64) -> bool {
    let x = &a.history;
    let y = &b.history;
    advances(a.request_sequence, b.request_sequence, delta)
        && event(
            &a.request_sequence,
            &b.request_sequence,
            a.request_revision,
            b.request_revision,
            rev,
        )
        && advances(x.uses, y.uses, delta)
        && y.restored >= x.restored
        && y.profile_revision >= x.profile_revision
        && y.equipment_revision >= x.equipment_revision
        && (x.equipment_revision != y.equipment_revision
            || x.profile_revision == y.profile_revision && x.fixed_recovery == y.fixed_recovery)
        && (x.equipment_revision == y.equipment_revision || y.profile_revision > rev)
        && (x.uses != y.uses || x.restored == y.restored && x.last == y.last)
        && (x.uses == y.uses
            || y.last
                .as_ref()
                .is_some_and(|l| l.revision > rev && l.sequence > a.request_sequence))
        && a.stocks.0.len() == b.stocks.0.len()
        && a.stocks
            .0
            .iter()
            .zip(&b.stocks.0)
            .all(|(x, y)| x.item == y.item && x.initial == y.initial && y.remaining <= x.remaining)
}
fn personal(a: &UnvalidatedData, b: &UnvalidatedData) -> bool {
    let x = &a.personal;
    let y = &b.personal;
    let rev = a.world.revision;
    let delta = b.world.revision - rev;
    if x.skills != y.skills || x.spell_birth_revision != y.spell_birth_revision {
        return false;
    }
    if !optional(&x.equipment, &y.equipment, |e, f| {
        if e.source != f.source
            || e.object != f.object
            || e.name != f.name
            || e.class != f.class
            || e.sex != f.sex
            || e.level != f.level
            || e.saved != f.saved
            || e.inventory != f.inventory
            || f.revision < e.revision
        {
            return false;
        }
        e.revision != f.revision || e == f
    }) || !optional(&x.supplies, &y.supplies, |s, t| supply(s, t, rev, delta))
        || !optional(&x.regen_clock, &y.regen_clock, |c, d| {
            clock(c, d, b.world.active_elapsed_nanos, false)
        })
    {
        return false;
    }
    if !optional(&x.ammo, &y.ammo, |c, d| {
        event(c, d, c.revision, d.revision, rev)
            && advances(c.request_sequence, d.request_sequence, delta)
            && advances(c.shot_count, d.shot_count, delta)
            && d.selection_revision >= c.selection_revision
            && (c.selection_revision != d.selection_revision || c.selected_item == d.selected_item)
            && (c.selection_revision == d.selection_revision || d.selection_revision > rev)
            && (c.shot_count != d.shot_count
                || c.shot_revision == d.shot_revision
                    && c.shot_item == d.shot_item
                    && c.attack_sequence == d.attack_sequence
                    && c.last_target_entity == d.last_target_entity
                    && c.from == d.from
                    && c.to == d.to
                    && c.projectile_sprite == d.projectile_sprite
                    && c.hit == d.hit)
            && (c.shot_count == d.shot_count || d.shot_revision > rev)
            && c.stocks.0.len() == d.stocks.0.len()
            && c.stocks.0.iter().zip(&d.stocks.0).all(|(a, b)| {
                let mut copy = a.clone();
                copy.remaining = b.remaining;
                copy == *b && b.remaining <= a.remaining
            })
    }) {
        return false;
    }
    optional(&x.weight, &y.weight, |a, b| a.units == b.units)
}
pub(super) fn follows(
    a: &UnvalidatedData,
    b: &UnvalidatedData,
) -> Result<(), app::ApplicationError> {
    let rev = a.world.revision;
    let delta = b.world.revision - rev;
    for (x, y) in a.actors.0.iter().zip(&b.actors.0) {
        ensure(
            x.entity == y.entity
                && (!x.loaded || y.loaded)
                && x.step_interval_nanos == y.step_interval_nanos
                && x.cast_interval_nanos == y.cast_interval_nanos
                && advances(x.last_move_sequence, y.last_move_sequence, delta)
                && optional(&x.resources, &y.resources, |x, y| {
                    resources(x, y, rev, delta)
                }),
        )?;
        if let (Some(v), Some(w)) = (&x.vital, &y.vital) {
            ensure(
                advances(v.attack_sequence, w.attack_sequence, delta)
                    && w.attack_revision >= v.attack_revision
                    && w.hurt_revision >= v.hurt_revision
                    && w.death_revision >= v.death_revision
                    && (w.attack_revision == v.attack_revision
                        || w.attack_revision > rev && w.attack_sequence > v.attack_sequence)
                    && (w.hurt_revision == v.hurt_revision || w.hurt_revision > rev)
                    && (w.death_revision == v.death_revision || w.death_revision > rev)
                    && (w.attack_revision != v.attack_revision || w.target == v.target),
            )?;
            let same_gear = x
                .resources
                .as_ref()
                .zip(y.resources.as_ref())
                .is_some_and(|(r, s)| r.equipment_revision == s.equipment_revision);
            let same_life = x
                .resources
                .as_ref()
                .zip(y.resources.as_ref())
                .is_some_and(|(r, s)| r.life == s.life);
            ensure(
                v.values.hp != 0
                    || !same_life
                    || x.position == y.position && x.heading == y.heading,
            )?;
            ensure(
                !same_gear
                    || v.ac == w.ac
                        && v.weapon_type == w.weapon_type
                        && v.attack_interval_nanos == w.attack_interval_nanos,
            )?;
        }
        let p = &x.spells;
        let q = &y.spells;
        ensure(
            advances(p.sequence, q.sequence, delta)
                && (p.sequence == q.sequence) == (p.request_revision == q.request_revision)
                && event(
                    &p.sequence,
                    &q.sequence,
                    p.request_revision,
                    q.request_revision,
                    rev,
                )
                && p.initial_lawful == q.initial_lawful
                && q.lawful >= p.lawful
                && p.histories.0.len() == q.histories.0.len()
                && p.histories.0.iter().zip(&q.histories.0).all(|(s, t)| {
                    spell(s, t, rev, delta)
                        && (s.ledger.casts == t.ledger.casts
                            || t.last.as_ref().is_some_and(|c| c.sequence > p.sequence))
                }),
        )?;
        let cast_delta: i128 = p
            .histories
            .0
            .iter()
            .zip(&q.histories.0)
            .map(|(s, t)| i128::from(t.ledger.casts) - i128::from(s.ledger.casts))
            .sum();
        ensure(cast_delta <= i128::from(q.sequence) - i128::from(p.sequence))?;
        match (&p.structure, &q.structure) {
            (Some(s), Some(t)) => ensure(
                t.casts >= s.casts
                    && (s == t || t.impact.usage.sequence > p.sequence)
                    && event(s, t, s.impact.usage.revision, t.impact.usage.revision, rev),
            )?,
            (Some(_), None) => return Err(invalid()),
            (None, Some(t)) => {
                ensure(t.impact.usage.revision > rev && t.impact.usage.sequence > p.sequence)?
            }
            _ => {}
        }
        ensure(optional(&x.buffs, &y.buffs, |c, d| {
            event(c, d, c.revision, d.revision, rev) && d.observed_nanos >= c.observed_nanos
        }))?;
        ensure(optional(&x.regeneration, &y.regeneration, |c, d| {
            [&c.hp, &c.mp].iter().zip([&d.hp, &d.mp]).all(|(x, y)| {
                event(*x, y, x.revision, y.revision, rev) && y.gain >= x.gain && y.loss >= x.loss
            }) && d.item_gain >= c.item_gain
                && d.damage_loss >= c.damage_loss
        }))?;
        if x.last_move_sequence == y.last_move_sequence {
            ensure(x.heading == y.heading)?;
            if x.position != y.position {
                let tele = q
                    .histories
                    .0
                    .iter()
                    .find(|s| s.lane == Lane::Teleport)
                    .and_then(|s| s.receipt.as_ref())
                    .filter(|r| r.impact.usage.revision > rev);
                let respawn = b
                    .quest
                    .as_ref()
                    .and_then(|q| q.lives.0.iter().find(|l| l.entity == y.entity))
                    .filter(|l| l.respawn_revision > rev)
                    .map(|l| (l.respawn_revision, vec![l.spawn.clone()]))
                    .or_else(|| {
                        b.siege.as_ref().and_then(|s| {
                            s.lives
                                .0
                                .iter()
                                .find(|l| l.entity == y.entity)
                                .filter(|l| l.respawn_revision > rev)
                                .map(|l| {
                                    let team = b.room.members.0[(y.entity - 1) as usize].team;
                                    (
                                        l.respawn_revision,
                                        if team == s.attacker {
                                            s.attack_camp.0.clone()
                                        } else {
                                            s.defense_camp.0.clone()
                                        },
                                    )
                                })
                        })
                    });
                let latest_tele = tele.filter(|r| {
                    respawn
                        .as_ref()
                        .is_none_or(|(v, _)| r.impact.usage.revision > *v)
                });
                ensure(if let Some(t) = latest_tele {
                    matches!(&t.detail,Detail::Teleport{destination} if *destination==y.position)
                } else {
                    respawn.is_some_and(|(_, cells)| cells.contains(&y.position))
                })?;
            }
        }
    }
    ensure(personal(a, b))
}
