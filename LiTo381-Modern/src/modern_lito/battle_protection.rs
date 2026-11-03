//! Confirmed armor status strip, not a proc effect, skill grant or LOAD proof.
//! The complete Observation remains intact. A local clock may hide an expired
//! icon but never expires authority, changes HP or extends protection. Missing
//! Required proc graphics are qualified separately by battle_armor_effects.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    compositor::Canvas,
    effect_catalog::{EffectCatalog, EffectId},
    immune_to_harm_icon::ImmuneToHarmIcon,
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "protection strip cancelled"
    );
    Ok(())
}
pub(crate) struct Assets {
    icon: Option<ImmuneToHarmIcon>,
}
impl Assets {
    pub(crate) fn load(root: &Path, o: &Observation, cancel: &AtomicBool) -> Result<Self> {
        Self::load_with_catalog(root, o, None, cancel)
    }
    pub(crate) fn load_with_catalog(
        root: &Path,
        o: &Observation,
        effect_catalog: Option<&EffectCatalog>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let data = o.data();
        let graphic = effect_catalog
            .map(|catalog| {
                Ok::<i64, anyhow::Error>(i64::from(catalog.get(EffectId::from_wire(41)?)?.icon))
            })
            .transpose()?
            .unwrap_or(super::immune_to_harm_icon::GRAPHIC);
        let icon = data
            .armor_actors()
            .filter(|_| {
                data.armor_presentation()
                    .is_none_or(|p| p.required.icons.0.contains(&1562))
            })
            .map(|_| ImmuneToHarmIcon::load_graphic(root, graphic, cancel))
            .transpose()?;
        active(cancel)?;
        Ok(Self { icon })
    }
    pub(crate) fn compatible(&self, old: &Self) -> bool {
        match (&self.icon, &old.icon) {
            (None, None) => true,
            (Some(a), Some(b)) => a.source() == b.source() && a.rgba() == b.rgba(),
            _ => false,
        }
    }
}
#[derive(Clone)]
struct Clock {
    life: i64,
    until: i64,
    deadline: Instant,
}
impl Clock {
    fn observed(
        life: i64,
        until: i64,
        clock: i64,
        now: Instant,
        old: Option<&Self>,
    ) -> Result<Self> {
        let nanos = until
            .checked_sub(clock)
            .filter(|n| *n > 0)
            .context("invalid confirmed protection duration")?;
        let mut deadline = now
            .checked_add(Duration::from_nanos(nanos.try_into()?))
            .context("protection deadline overflow")?;
        if let Some(old) = old.filter(|old| old.life == life && old.until == until) {
            // A repeated snapshot, a render, a later damage receipt or a
            // resource handoff must not restart an already observed duration.
            deadline = deadline.min(old.deadline);
        }
        Ok(Self {
            life,
            until,
            deadline,
        })
    }
    fn remaining(&self, now: Instant) -> Option<Duration> {
        self.deadline
            .checked_duration_since(now)
            .filter(|d| !d.is_zero())
    }
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    observed: Instant,
    clocks: BTreeMap<i64, Clock>,
}
impl Timeline {
    pub(crate) fn new(o: Observation, now: Instant, cancel: &AtomicBool) -> Result<Self> {
        let clocks = Self::clocks(&o, now, &BTreeMap::new(), cancel)?;
        Ok(Self {
            observation: o,
            observed: now,
            clocks,
        })
    }
    fn clocks(
        o: &Observation,
        now: Instant,
        old: &BTreeMap<i64, Clock>,
        cancel: &AtomicBool,
    ) -> Result<BTreeMap<i64, Clock>> {
        active(cancel)?;
        let data = o.data();
        let mut clocks = BTreeMap::new();
        // Finished/dead/departed/respawned status is supplied by the server.
        // BOTs have no human armor state; private Quest lanes stay private.
        if let Some(armor) = data
            .armor_actors()
            .filter(|_| data.world.phase == d::Phase::Active)
        {
            for a in &armor.0 {
                active(cancel)?;
                let Some(s) = &a.state else { continue };
                if s.until_nanos <= data.world.active_elapsed_nanos {
                    continue;
                }
                let actor = data
                    .actors
                    .0
                    .iter()
                    .find(|a| a.entity == s.entity)
                    .context("protection actor absent")?;
                let resources = actor
                    .resources
                    .as_ref()
                    .context("protection resources absent")?;
                ensure!(
                    resources.life == s.life && resources.vitals.hp > 0,
                    "protection life mismatch"
                );
                clocks.insert(
                    s.entity,
                    Clock::observed(
                        s.life,
                        s.until_nanos,
                        data.world.active_elapsed_nanos,
                        now,
                        old.get(&s.entity),
                    )?,
                );
            }
        }
        active(cancel)?;
        Ok(clocks)
    }
    pub(crate) fn advance(
        &self,
        next: Observation,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(now >= self.observed, "protection clock reversed");
        self.observation.check_successor(&next)?;
        let clocks = Self::clocks(&next, now, &self.clocks, cancel)?;
        Ok(Self {
            observation: next,
            observed: now,
            clocks,
        })
    }
    pub(crate) fn remaining(&self, entity: i64, now: Instant) -> Result<Option<Duration>> {
        ensure!(now >= self.observed, "protection sample reversed");
        Ok(self.clocks.get(&entity).and_then(|c| c.remaining(now)))
    }
    /// Self status only, in logical HUD pixels, never at a world/camera anchor.
    /// The caller clears/composes each frame so expiry leaves no stale pixels.
    pub(crate) fn draw(
        &self,
        canvas: &mut Canvas,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
        origin: (i32, i32),
    ) -> Result<()> {
        active(cancel)?;
        if self
            .remaining(self.observation.data().personal.entity, now)?
            .is_some()
        {
            assets
                .icon
                .as_ref()
                .context("protection icon not loaded")?
                .draw(canvas, origin.0, origin.1)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_receipts_cannot_extend_or_resurrect_elapsed_status() {
        let at = Instant::now();
        let c = Clock::observed(0, 3_000_000_000, 0, at, None).unwrap();
        let later = at + Duration::from_secs(2);
        let repeated = Clock::observed(0, c.until, 0, later, Some(&c)).unwrap();
        assert_eq!(repeated.remaining(later), Some(Duration::from_secs(1)));
        let ended = at + Duration::from_secs(3);
        let repeated = Clock::observed(0, c.until, 0, ended, Some(&repeated)).unwrap();
        assert_eq!(repeated.remaining(ended), None);
        assert!(repeated.remaining(ended + Duration::from_secs(1)).is_none());
        // A later server clock may shorten, never stretch, the same grant.
        let shortened = Clock::observed(0, c.until, 2_500_000_000, later, Some(&c)).unwrap();
        assert_eq!(shortened.remaining(later), Some(Duration::from_millis(500)));
        // Only a new authoritative life/deadline can establish a new clock.
        let refreshed = Clock::observed(0, 6_000_000_000, 3_000_000_000, ended, Some(&c)).unwrap();
        assert_eq!(refreshed.remaining(ended), Some(Duration::from_secs(3)));
        let revived = Clock::observed(1, c.until, 0, ended, Some(&c)).unwrap();
        assert_eq!(revived.remaining(ended), Some(Duration::from_secs(3)));
        assert!(Clock::observed(0, i64::MAX, i64::MIN, at, None).is_err());
        assert!(Clock::observed(0, 1, 1, at, None).is_err());
    }
    #[test]
    fn ordinary_scene_has_no_status_or_asset_reads() {
        let o = super::super::battle_scene::tests::observation(
            super::super::battle_payload::data_tests::fixture(),
        );
        let cancel = AtomicBool::new(false);
        let at = Instant::now();
        let t = Timeline::new(o.clone(), at, &cancel).unwrap();
        let assets = Assets::load(Path::new("not-a-resource-root"), &o, &cancel).unwrap();
        assert!(assets.icon.is_none());
        let t = t.advance(o.clone(), at, &cancel).unwrap();
        let mut canvas = Canvas::new(40, 40).unwrap();
        let before = canvas.rgba().to_vec();
        t.draw(&mut canvas, &assets, at, &cancel, (0, 0)).unwrap();
        assert_eq!(canvas.rgba(), before);
        assert!(t.advance(o.clone(), at, &AtomicBool::new(true)).is_err());
        assert!(t.advance(o, at - Duration::from_nanos(1), &cancel).is_err());
        assert!(t.remaining(1, at - Duration::from_nanos(1)).is_err());
        assert!(t.clocks.is_empty());
    }
}
