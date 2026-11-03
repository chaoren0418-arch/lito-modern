//! Candidate-only atomic following. Saved equipment, runtime abilities and VIP
//! lifecycle have distinct counters. None is a durable command receipt.
use super::*;

#[derive(Clone)]
pub struct Follower {
    player: Player,
    world: WorldId,
    current: Option<Checked>,
}
impl std::fmt::Debug for Follower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CandidateVipFollower[redacted]")
    }
}
impl Follower {
    pub fn new(player: Player, world: WorldId) -> Result<Self, ApplicationError> {
        require(
            player.source > 0
                && player.object > 0
                && player.epoch != 0
                && (0..=3).contains(&player.channel)
                && world.room > 0
                && world.match_id > 0
                && world.generation > 0,
        )?;
        Ok(Self {
            player,
            world,
            current: None,
        })
    }
    pub fn current(&self) -> Option<&Checked> {
        self.current.as_ref()
    }
    /// No historical replay on first attachment. Rejected state cannot change
    /// the current scene or emit effects. Skipped event history is not invented.
    pub fn accept(
        &mut self,
        candidate: Candidate,
    ) -> Result<Vec<armor::EffectEvent>, ApplicationError> {
        let next = candidate.checked(&self.player, &self.world)?;
        let events = if let Some(old) = &self.current {
            old.candidate().check_successor(next.candidate())?;
            armor::retained_value_events(
                &next.candidate().data,
                &next.candidate().armor,
                old.candidate().data.world.revision,
            )?
        } else {
            Vec::new()
        };
        self.current = Some(next);
        Ok(events)
    }
}
impl View<'_> {
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        crate::modern_lito::battle_follow::check_vip_view_sequence(self, next)?;
        // Borrow the actual optional sections; no synthetic armor candidate or
        // removed VIP flag. This helper proves only shared temporal values.
        view(self).check_successor_body(&view(next))?;
        let a = &self.owner;
        let b = &next.owner;
        let budget = b
            .world_revision
            .checked_sub(a.world_revision)
            .ok_or_else(invalid)?;
        require(self.appearances.is_some() == next.appearances.is_some())?;
        if let (Some(old), Some(new)) = (self.appearances, next.appearances) {
            require(old.0.len() == new.0.len())?;
            for (a, b) in old.0.iter().zip(&new.0) {
                require(
                    a.entity == b.entity
                        && a.source == b.source
                        && a.object == b.object
                        && b.life >= a.life,
                )?;
                let runtime = delta(a.runtime, b.runtime, budget)?;
                let lifecycle = delta(a.lifecycle, b.lifecycle, budget)?;
                if budget == 0 || matches!(self.data.world.phase, Phase::Aborted | Phase::Finished)
                {
                    require(a == b)?;
                } else if runtime == 0 || lifecycle == 0 {
                    require(a.graphics == b.graphics)?;
                }
            }
        }
        require(
            a.world == b.world && a.player == b.player && a.entity == b.entity && b.life >= a.life,
        )?;
        if budget == 0 || matches!(self.data.world.phase, Phase::Aborted | Phase::Finished) {
            return require(a == b);
        }
        let saved = delta(a.versions.saved, b.versions.saved, budget)?;
        let runtime = delta(a.versions.runtime, b.versions.runtime, budget)?;
        let lifecycle = delta(a.versions.lifecycle, b.versions.lifecycle, budget)?;
        require(
            next.data
                .personal
                .equipment
                .as_ref()
                .ok_or_else(invalid)?
                .evaluated_at_epoch_millis
                >= self
                    .data
                    .personal
                    .equipment
                    .as_ref()
                    .ok_or_else(invalid)?
                    .evaluated_at_epoch_millis,
        )?;
        require(saved <= runtime && saved <= lifecycle)?;
        if runtime == 0 || lifecycle == 0 {
            require(
                a.items == b.items
                    && a.abilities == b.abilities
                    && a.icons == b.icons
                    && a.shape == b.shape,
            )?;
        }
        if saved == 0 {
            require(
                b.items
                    .0
                    .iter()
                    .all(|i| a.items.0.iter().any(|old| old.object == i.object)),
            )?;
            let now = next
                .data
                .personal
                .equipment
                .as_ref()
                .ok_or_else(invalid)?
                .evaluated_at_epoch_millis;
            for old in &a.items.0 {
                if !b.items.0.iter().any(|i| i.object == old.object) {
                    require(
                        old.lifetime.state == LifetimeState::Active
                            && old.lifetime.expires_at_millis <= now,
                    )?;
                }
            }
            require(b.shape.is_none() || b.shape == a.shape)?;
        }
        if saved <= 1 {
            // Retained templates are pinned. Two or more skipped saved changes
            // may legitimately remove and re-equip an item from a new catalog.
            for old in &a.items.0 {
                if let Some(new) = b.items.0.iter().find(|i| i.object == old.object) {
                    require(
                        new.applied == old.applied
                            && new.lifetime.state == old.lifetime.state
                            && new.lifetime.expires_at_millis <= old.lifetime.expires_at_millis,
                    )?;
                }
            }
        }
        Ok(())
    }
}
fn view<'a>(c: &View<'a>) -> armor::View<'a> {
    armor::View {
        data: c.data,
        status: c.status,
        healing: c.healing,
        actors: c.armor,
        presentation: c.presentation,
    }
}
fn delta(a: i64, b: i64, budget: i64) -> Result<i64, ApplicationError> {
    let value = b.checked_sub(a).ok_or_else(invalid)?;
    require((0..=budget).contains(&value))?;
    Ok(value)
}
