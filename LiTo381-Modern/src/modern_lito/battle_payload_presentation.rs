//! Schema 2 public visual receipts. No private inventories, inferred events,
//! wall-clock sampling, revision conversion or client-side combat authority.
use super::{combat::require, data::*, invalid, ApplicationError};
use crate::modern_lito::world_protocol::MatchDescriptor as Features;

fn anchor(
    d: &UnvalidatedData,
    a: &Actor,
    revision: i64,
    elapsed: i64,
    life: i64,
    gear: i64,
    cell: &Cell,
) -> Result<(), ApplicationError> {
    let r = a.resources.as_ref().ok_or_else(invalid)?;
    require(
        revision >= 2
            && revision <= d.world.revision
            && elapsed >= 0
            && elapsed <= d.world.loading_elapsed_nanos
            && life >= 1
            && r.life.checked_add(1).is_some_and(|n| life <= n)
            && gear >= 0
            && gear <= r.equipment_revision
            && d.manifest.contains_cell(cell),
    )
}
fn target_entity(t: &Target) -> Option<i64> {
    match t {
        Target::Participant { entity, .. } | Target::Honor { entity, .. } => Some(*entity),
        _ => None,
    }
}
pub(crate) fn check(d: &UnvalidatedData) -> Result<(), ApplicationError> {
    let Some(p) = &d.presentation else {
        return Ok(());
    };
    require(p.0.len() == d.actors.0.len())?;
    let enabled = |f: u32| d.world.descriptor.features as u32 & f == f;
    for (p, a) in p.0.iter().zip(&d.actors.0) {
        require(p.entity == a.entity)?;
        require(d.world.active_elapsed_nanos >= 0 || p.shot.is_none() && p.item.is_none())?;
        if let Some(s) = &p.shot {
            let v = a.vital.as_ref().ok_or_else(invalid)?;
            require(
                enabled(Features::AMMO)
                    && s.count > 0
                    && s.attack_sequence >= s.count
                    && s.attack_sequence < s.revision
                    && s.attack_sequence <= v.attack_sequence
                    && s.revision <= v.attack_revision
                    && (s.revision != v.attack_revision
                        || target_entity(&s.target) == Some(v.target))
                    && s.sprite == 66
                    && s.from != s.to
                    && s.target_life >= 1
                    && d.manifest.contains_cell(&s.to),
            )?;
            anchor(
                d,
                a,
                s.revision,
                s.loading_elapsed_nanos,
                s.life,
                s.equipment_revision,
                &s.from,
            )?;
            match &s.target {
                Target::Participant { entity, identity } => {
                    let t = d
                        .actors
                        .0
                        .iter()
                        .find(|t| t.entity == *entity)
                        .ok_or_else(invalid)?;
                    let m = d
                        .room
                        .members
                        .0
                        .get(usize::try_from(*entity - 1).map_err(|_| invalid())?)
                        .ok_or_else(invalid)?;
                    require(
                        *entity != a.entity
                            && *identity == m.identity
                            && t.resources
                                .as_ref()
                                .and_then(|r| r.life.checked_add(1))
                                .is_some_and(|n| s.target_life <= n),
                    )?;
                }
                Target::Honor { entity, team } => {
                    let t = d
                        .honor
                        .objectives
                        .0
                        .iter()
                        .find(|t| t.entity == *entity)
                        .ok_or_else(invalid)?;
                    require(t.team == *team && t.cell == s.to && s.target_life == 1)?;
                }
                _ => return Err(invalid()),
            }
        }
        if let Some(i) = &p.item {
            require(
                enabled(Features::ITEMS)
                    && i.count > 0
                    && i.count < i.revision
                    && matches!(i.effect, 189 | 194 | 197 | 191 | 750 | 751)
                    && (!matches!(i.effect, 191 | 750 | 751) || enabled(Features::TIMED)),
            )?;
            anchor(
                d,
                a,
                i.revision,
                i.loading_elapsed_nanos,
                i.life,
                i.equipment_revision,
                &i.cell,
            )?;
        }
        if a.entity == d.personal.entity {
            let ammo = d.personal.ammo.as_ref().filter(|s| s.shot_count > 0);
            require(ammo.is_some() == p.shot.is_some())?;
            if let (Some(ammo), Some(s)) = (ammo, &p.shot) {
                require(
                    s.count == ammo.shot_count
                        && s.revision == ammo.shot_revision
                        && s.attack_sequence == ammo.attack_sequence
                        && target_entity(&s.target) == Some(ammo.last_target_entity)
                        && s.from == ammo.from
                        && s.to == ammo.to
                        && s.sprite == ammo.projectile_sprite
                        && s.hit == ammo.hit,
                )?;
            }
            let supplies = d
                .personal
                .supplies
                .as_ref()
                .filter(|s| s.history.last.is_some());
            require(supplies.is_some() == p.item.is_some())?;
            if let (Some(s), Some(i)) = (supplies, &p.item) {
                let u = s.history.last.as_ref().ok_or_else(invalid)?;
                require(
                    i.count == s.history.uses
                        && i.revision == u.revision
                        && i.life == u.before.life
                        && i.equipment_revision == u.before.equipment_revision
                        && i.cell == u.before.cell
                        && i.effect == s.effect,
                )?;
            }
        }
    }
    Ok(())
}
fn event<T: PartialEq>(
    a: Option<&T>,
    b: Option<&T>,
    old: &UnvalidatedData,
    next: &UnvalidatedData,
    facts: impl Fn(&T) -> (i64, i64, i64, i64, i64),
) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(_), None) => false,
        (a, Some(b)) => {
            if a == Some(b) {
                return true;
            }
            let (count, revision, elapsed, life, gear) = facts(b);
            let (prior_count, prior_revision, _, prior_life, prior_gear) =
                a.map(&facts).unwrap_or((0, 0, 0, 0, 0));
            count > prior_count
                && revision > prior_revision
                && revision > old.world.revision
                && elapsed >= old.world.loading_elapsed_nanos
                && life >= prior_life
                && gear >= prior_gear
                && i128::from(count) - i128::from(prior_count)
                    <= i128::from(next.world.revision) - i128::from(old.world.revision)
        }
    }
}
pub(crate) fn follows(a: &UnvalidatedData, b: &UnvalidatedData) -> Result<(), ApplicationError> {
    match (&a.presentation, &b.presentation) {
        (None, None) => Ok(()),
        (Some(x), Some(y)) => {
            require(x.0.len() == y.0.len())?;
            for (x, y) in x.0.iter().zip(&y.0) {
                if let (Some(x), Some(y)) = (&x.shot, &y.shot) {
                    require(
                        x == y
                            || y.attack_sequence > x.attack_sequence
                                && i128::from(y.attack_sequence) - i128::from(x.attack_sequence)
                                    >= i128::from(y.count) - i128::from(x.count),
                    )?;
                }
                require(
                    x.entity == y.entity
                        && event(x.shot.as_ref(), y.shot.as_ref(), a, b, |s| {
                            (
                                s.count,
                                s.revision,
                                s.loading_elapsed_nanos,
                                s.life,
                                s.equipment_revision,
                            )
                        })
                        && event(x.item.as_ref(), y.item.as_ref(), a, b, |s| {
                            (
                                s.count,
                                s.revision,
                                s.loading_elapsed_nanos,
                                s.life,
                                s.equipment_revision,
                            )
                        }),
                )?;
            }
            Ok(())
        }
        _ => Err(invalid()),
    }
}
