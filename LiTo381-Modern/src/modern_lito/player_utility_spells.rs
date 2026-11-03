//! Player utility-spell composition. Reuses confirmed effect timelines and adds
//! logical-form action19 bodies; never derives authority from a visual Hit.
use super::{d, Form, Plan, PlayerFrame, PolymorphAssets, PolymorphForm};
use crate::modern_lito::{
    battle_follow::Observation,
    battle_haste, battle_healing,
    battle_motion::{
        self,
        finish::{IdleReturn, QualifiedReturn},
    },
    battle_slow_visual, battle_target_haste_visual, battle_teleport,
    compositor::Canvas,
    paced_animation::PacedClock,
    polymorph_assets::cast,
    shared_action_catalog::SharedActionCatalog,
    teleport_effect::TeleportAssets,
    world_audio::AudioFrame,
    world_map::{ViewProjection, WorldMap},
    world_scene::{ActorOverride, BaseAction, SceneSnapshot},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "player utility presentation cancelled"
    );
    Ok(())
}
pub(super) fn check_map(o: &Observation, map: &WorldMap) -> Result<()> {
    battle_teleport::check_map(o, map)?;
    battle_healing::check_map(o, map)?;
    battle_haste::check_map(o, map)?;
    battle_target_haste_visual::check_map(o, map)?;
    battle_slow_visual::check_map(o, map)
}
pub(super) struct Assets {
    teleport: Option<TeleportAssets>,
    healing: battle_healing::Assets,
    haste: battle_haste::Assets,
    target_haste: battle_target_haste_visual::Assets,
    slow: battle_slow_visual::Assets,
    polymorph: Option<cast::Assets>,
}
impl Assets {
    pub(super) fn load(c: &SharedActionCatalog, plan: &Plan, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let o = plan.observation();
        let enabled = battle_teleport::enabled(o)
            || battle_healing::enabled(o)
            || battle_haste::enabled(o)
            || battle_target_haste_visual::enabled(o)
            || battle_slow_visual::enabled(o);
        let forms: Vec<_> = plan
            .snapshot()
            .actors()
            .iter()
            .map(|a| a.appearance)
            .collect();
        let teleport = if enabled {
            Some(TeleportAssets::load_with_catalog(c, &forms, cancel)?)
        } else {
            None
        };
        let form = plan.forms().values().find_map(|f| match f {
            Form::Polymorph { form, .. } => Some(*form),
            _ => None,
        });
        let polymorph = if enabled {
            form.map(|f| cast::Assets::load_kind(c, f, cast::Kind::Undirected, cancel))
                .transpose()?
        } else {
            None
        };
        Ok(Self {
            teleport,
            polymorph,
            healing: battle_healing::Assets::load(c, o, cancel)?,
            haste: battle_haste::Assets::load(c, o, cancel)?,
            target_haste: battle_target_haste_visual::Assets::load(c, o, cancel)?,
            slow: battle_slow_visual::Assets::load(c, o, cancel)?,
        })
    }
    pub(super) fn cache_bytes(&self) -> (usize, usize) {
        self.polymorph
            .as_ref()
            .map_or((0, 0), |b| (b.encoded_bytes(), b.rgba_bytes()))
    }
}
#[derive(Clone)]
struct Cast {
    usage: d::Cast,
    form: Form,
    sequence: i64,
    position: d::Cell,
    heading: u8,
    body: PacedClock,
    tail: IdleReturn,
}
fn current(plan: &Plan, a: &d::Actor, c: &Cast) -> Result<bool> {
    let d = plan.observation().data();
    Ok(plan.forms().get(&(a.entity as u64)) == Some(&c.form)
        && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
        && battle_motion::life(a) == Some(c.usage.before.life)
        && a.resources
            .as_ref()
            .is_some_and(|r| r.equipment_revision == c.usage.before.equipment_revision)
        && a.last_move_sequence == c.sequence
        && a.position == c.position
        && battle_motion::latest_body_event(d, a, d.scene_member(a)?) <= c.usage.revision)
}
fn recent(receipt: i64, now: i64) -> bool {
    now.checked_sub(receipt)
        .is_some_and(|n| (0..=500_000_000).contains(&n))
}
fn utility(lane: d::Lane) -> bool {
    matches!(
        lane,
        d::Lane::Teleport
            | d::Lane::Healing
            | d::Lane::GreaterHaste
            | d::Lane::Haste
            | d::Lane::Slow
    )
}
// Different wire domains remain distinct. This short borrowed tuple is only
// a body start, not an invented SpellReceipt or an outgoing healing command.
fn receipts<'a>(
    data: &'a d::UnvalidatedData,
    a: &'a d::Actor,
) -> Vec<(&'a d::Cast, i64, &'a d::Cell, bool)> {
    let mut result = Vec::new();
    for h in &a.spells.histories.0 {
        if !utility(h.lane) {
            continue;
        }
        if let Some(r) = &h.receipt {
            let (position, teleport) = match &r.detail {
                d::Detail::Teleport { destination } => (destination, true),
                _ => (&r.impact.usage.before.cell, false),
            };
            result.push((&r.impact.usage, r.loading_elapsed_nanos, position, teleport));
        }
    }
    if let Some(r) = data
        .healing_actors()
        .and_then(|h| h.0.iter().find(|h| h.entity == a.entity))
        .and_then(|h| h.outgoing.as_ref())
        .and_then(|h| h.receipt.as_ref())
    {
        result.push((
            &r.impact.usage,
            r.loading_elapsed_nanos,
            &r.impact.usage.before.cell,
            false,
        ));
    }
    result.sort_by_key(|r| r.0.revision);
    result
}
#[derive(Clone)]
pub(super) struct Timeline {
    plan: Plan,
    observed: Instant,
    casts: BTreeMap<i64, Cast>,
    teleport: battle_teleport::Timeline,
    healing: battle_healing::Timeline,
    haste: battle_haste::Timeline,
    target_haste: battle_target_haste_visual::Timeline,
    slow: battle_slow_visual::Timeline,
}
impl Timeline {
    pub(super) fn new(plan: Plan, now: Instant) -> Self {
        let o = plan.observation();
        Self {
            teleport: battle_teleport::Timeline::new(o.clone(), now),
            healing: battle_healing::Timeline::new(o.clone(), now),
            haste: battle_haste::Timeline::new(o.clone(), now),
            target_haste: battle_target_haste_visual::Timeline::new(o.clone(), now),
            slow: battle_slow_visual::Timeline::new(o.clone(), now),
            plan,
            observed: now,
            casts: BTreeMap::new(),
        }
    }
    pub(super) fn advance(
        &self,
        plan: Plan,
        now: Instant,
        assets: &Assets,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(now >= self.observed, "utility clock reversed");
        self.plan.check_successor(&plan)?;
        let o = plan.observation();
        let mut out = Self {
            teleport: self.teleport.advance(o.clone(), now, cancel)?,
            healing: self
                .healing
                .advance(o.clone(), now, &assets.healing, cancel)?,
            haste: self.haste.advance(o.clone(), now, &assets.haste, cancel)?,
            target_haste: self.target_haste.advance(
                o.clone(),
                now,
                &assets.target_haste,
                cancel,
            )?,
            slow: self.slow.advance(o.clone(), now, &assets.slow, cancel)?,
            casts: self.casts.clone(),
            plan: plan.clone(),
            observed: now,
        };
        let data = o.data();
        if data.world.phase != d::Phase::Active {
            out.casts.clear();
            return Ok(out);
        }
        for a in &data.actors.0 {
            active(cancel)?;
            if let Some(form @ Form::Polymorph { .. }) = plan.forms().get(&(a.entity as u64)) {
                let old = self
                    .plan
                    .observation()
                    .data()
                    .actors
                    .0
                    .iter()
                    .find(|old| old.entity == a.entity)
                    .context("utility actor absent")?;
                for (usage, elapsed, position, teleport) in receipts(data, a) {
                    if usage.revision <= self.plan.observation().data().world.revision
                        || !recent(elapsed, data.world.loading_elapsed_nanos)
                        || (!teleport
                            && old.last_move_sequence != a.last_move_sequence
                            && usage.revision != data.world.revision)
                    {
                        continue;
                    }
                    let bank = assets
                        .polymorph
                        .as_ref()
                        .context("utility polymorph bank absent")?;
                    let mut body = PacedClock::new(now);
                    let base = u64::try_from(a.cast_interval_nanos)?;
                    body.retime(now, base, battle_motion::paced_nanos(a, base, false))?;
                    let tail = IdleReturn::start(
                        QualifiedReturn::polymorph_cast(bank.action()),
                        &body,
                        now,
                    )?;
                    let cast = Cast {
                        usage: usage.clone(),
                        form: *form,
                        sequence: a.last_move_sequence,
                        position: position.clone(),
                        heading: a.heading.try_into()?,
                        body,
                        tail,
                    };
                    if current(&plan, a, &cast)? {
                        out.casts.insert(a.entity, cast);
                    }
                }
            }
            if let Some(c) = out.casts.get_mut(&a.entity) {
                if !current(&plan, a, c)? {
                    out.casts.remove(&a.entity);
                } else {
                    let base = u64::try_from(a.cast_interval_nanos)?;
                    c.body
                        .retime(now, base, battle_motion::paced_nanos(a, base, false))?;
                    c.tail.retime(&c.body, now)?;
                }
            }
        }
        active(cancel)?;
        Ok(out)
    }
    pub(super) fn overrides<'a>(
        &self,
        snapshot: &SceneSnapshot,
        assets: &'a Assets,
        now: Instant,
    ) -> Result<Vec<ActorOverride<'a>>> {
        ensure!(now >= self.observed, "utility sample reversed");
        let Some(bank) = &assets.teleport else {
            return Ok(Vec::new());
        };
        let mut draws = self.teleport.overrides(snapshot, bank, now)?;
        draws.extend(self.healing.overrides(snapshot, bank, now)?);
        draws.extend(self.haste.overrides(snapshot, bank, now)?);
        draws.extend(self.target_haste.overrides(snapshot, bank, now)?);
        draws.extend(self.slow.overrides(snapshot, bank, now)?);
        // Never put a base-class body over a logical polymorph.
        draws.retain(|d| matches!(self.plan.forms().get(&d.entity_id), Some(Form::Base(_))));
        Ok(draws)
    }
    pub(super) fn frames<'a>(
        &self,
        snapshot: &SceneSnapshot,
        assets: &'a Assets,
        bodies: &'a BTreeMap<PolymorphForm, PolymorphAssets>,
        now: Instant,
    ) -> Result<Vec<(u64, PlayerFrame<'a>)>> {
        ensure!(
            now >= self.observed && self.plan.matches(snapshot),
            "stale utility sample"
        );
        let mut draws = Vec::new();
        for a in &self.plan.observation().data().actors.0 {
            let Some(c) = self.casts.get(&a.entity) else {
                continue;
            };
            if snapshot.hidden(a.entity as u64) || !current(&self.plan, a, c)? {
                continue;
            }
            let Form::Polymorph { form, .. } = c.form else {
                unreachable!()
            };
            let frame = if now >= c.tail.deadline() {
                let bank = bodies.get(&form).context("utility idle bank absent")?;
                let elapsed = now.duration_since(c.tail.deadline());
                let source = bank.action(BaseAction::Idle)?.sample(elapsed, c.heading)?.0;
                PlayerFrame::from(bank.frame(BaseAction::Idle, c.heading, source)?)
            } else {
                let frame = assets
                    .polymorph
                    .as_ref()
                    .context("utility caster bank absent")?
                    .frame(c.body.peek(now)?, c.heading)?;
                ensure!(frame.form == form, "utility cast form mismatch");
                PlayerFrame {
                    form,
                    action: None,
                    source: frame.source,
                    body: frame.body,
                    shadow: frame.shadow,
                    attachment: frame.attachment,
                }
            };
            draws.push((a.entity.try_into()?, frame));
        }
        Ok(draws)
    }
    pub(super) fn draw(
        &self,
        canvas: &mut Canvas,
        projection: ViewProjection,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        if let Some(bank) = &assets.teleport {
            self.teleport.draw(canvas, projection, bank, now, cancel)?;
        }
        self.healing
            .draw(canvas, projection, &assets.healing, now, cancel)?;
        self.haste
            .draw(canvas, projection, &assets.haste, now, cancel)?;
        self.target_haste
            .draw(canvas, projection, &assets.target_haste, now, cancel)?;
        self.slow
            .draw(canvas, projection, &assets.slow, now, cancel)?;
        Ok(())
    }
    pub(super) fn audio(
        &mut self,
        assets: &Assets,
        now: Instant,
    ) -> Result<[Option<AudioFrame>; 2]> {
        Ok([
            assets
                .teleport
                .as_ref()
                .map(|a| self.teleport.audio(a, now))
                .transpose()?,
            self.healing.audio(&assets.healing, now),
        ])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn body_age_never_accepts_future_overflow_or_stale_receipts() {
        assert!(recent(0, 0));
        assert!(recent(3, 500_000_003));
        assert!(!recent(3, 2));
        assert!(!recent(3, 500_000_004));
        assert!(!recent(i64::MIN, i64::MAX));
    }
}
