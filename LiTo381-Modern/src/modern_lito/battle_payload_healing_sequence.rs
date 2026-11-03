//! Explicit schema7 follower. Authenticated transport/command/rendering gates
//! remain closed. Rejection never publishes a partially updated snapshot.
use super::*;

#[derive(Clone)]
pub struct Follower {
    player: Player,
    world: WorldId,
    current: Option<Checked>,
}
impl std::fmt::Debug for Follower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CandidateHealingFollower[redacted]")
    }
}
impl Follower {
    pub fn new(player: Player, world: WorldId) -> Result<Self, ApplicationError> {
        world_value(&world)?;
        require(
            player.source > 0
                && player.object > 0
                && player.epoch != 0
                && (0..=3).contains(&player.channel),
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
    pub fn accept(&mut self, candidate: Candidate) -> Result<(), ApplicationError> {
        let next = candidate.checked(&self.player, &self.world)?;
        if let Some(old) = &self.current {
            old.candidate().check_successor(next.candidate())?;
        }
        self.current = Some(next);
        Ok(())
    }
}
impl Candidate {
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        self.view()?.check_successor(&next.view()?)
    }
}
impl View<'_> {
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        if self.data.armor.is_some() || next.data.armor.is_some() {
            self.check_values(&self.data.personal.player, &self.data.world.identity)?;
            next.check_values(&self.data.personal.player, &self.data.world.identity)?;
            return super::super::armor::follows_attached(self.data, next.data);
        }
        crate::modern_lito::battle_follow::check_healing_view_sequence(self, next)?;
        self.check_successor_body(next)
    }
    /// Only after the complete schema-specific scene and base timeline proof.
    pub(in crate::modern_lito::battle_payload) fn check_successor_body(
        &self,
        next: &Self,
    ) -> Result<(), ApplicationError> {
        let d = &self.data;
        let n = &next.data;
        let rev = d.world.revision;
        let budget = n.world.revision - rev;
        self.status.check_successor(d, next.status, n)?;
        if budget == 0 || matches!(d.world.phase, Phase::Aborted | Phase::Finished) {
            return require(self.actors == next.actors);
        }
        let old_events = self.healing_events()?;
        let healing_delta: i128 = self
            .actors
            .0
            .iter()
            .zip(&next.actors.0)
            .map(|(a, b)| i128::from(b.incoming.received) - i128::from(a.incoming.received))
            .sum();
        require(healing_delta >= 0 && healing_delta <= i128::from(budget))?;
        for (revision, impact) in next.healing_events()? {
            if revision <= rev {
                require(old_events.get(&revision).is_some_and(|old| *old == impact))?;
            }
        }
        for ((a, b), (x, y)) in self
            .actors
            .0
            .iter()
            .zip(&next.actors.0)
            .zip(d.actors.0.iter().zip(&n.actors.0))
        {
            let i = &a.incoming;
            let j = &b.incoming;
            require(a.entity == b.entity && i.world == j.world && i.recipient == j.recipient)?;
            let received = j.received.checked_sub(i.received).ok_or_else(invalid)?;
            let restored = j.restored.checked_sub(i.restored).ok_or_else(invalid)?;
            require(
                (0..=budget).contains(&received)
                    && restored >= 0
                    && i128::from(restored) <= i128::from(received) * 65534,
            )?;
            if received == 0 {
                require(i == j)?;
            } else {
                let last = j.last.as_ref().ok_or_else(invalid)?;
                require(
                    last.usage.revision > rev
                        && restored >= last.target.applied
                        && (received != 1 || restored == last.target.applied),
                )?;
                if let Some(old) = &i.last {
                    require(
                        last.usage.revision > old.usage.revision
                            && last.target.life >= old.target.life
                            && last.target.equipment_revision >= old.target.equipment_revision
                            && last.target_effect_revision >= old.target_effect_revision,
                    )?;
                    if last.caster == old.caster {
                        require(last.usage.sequence > old.usage.sequence)?;
                    }
                }
            }
            let heal_casts = match (&a.outgoing, &b.outgoing) {
                (None, None) => 0,
                (Some(o), Some(p)) => outgoing(
                    o,
                    p,
                    rev,
                    budget,
                    x.spells.sequence,
                    d.world.loading_elapsed_nanos,
                )?,
                _ => return Err(invalid()),
            };
            let ordinary: i128 = x
                .spells
                .histories
                .0
                .iter()
                .zip(&y.spells.histories.0)
                .map(|(a, b)| i128::from(b.ledger.casts) - i128::from(a.ledger.casts))
                .sum();
            require(
                ordinary + i128::from(heal_casts)
                    <= i128::from(y.spells.sequence) - i128::from(x.spells.sequence),
            )?;
        }
        // Event evidence that crosses the old observation must account for both
        // sides, even when its caster/recipient has a newer retained receipt.
        for (revision, impact) in next.healing_events()? {
            if revision <= rev {
                continue;
            }
            let caster = impact.caster.entity;
            let recipient = match impact.target.target {
                Target::Participant { entity, .. } => entity,
                _ => return Err(invalid()),
            };
            let caster_before = d
                .scene_actor(caster)?
                .resources
                .as_ref()
                .ok_or_else(invalid)?;
            let recipient_before = d
                .scene_actor(recipient)?
                .resources
                .as_ref()
                .ok_or_else(invalid)?;
            require(
                impact.usage.before.life
                    >= caster_before.life.checked_add(1).ok_or_else(invalid)?
                    && impact.usage.before.equipment_revision >= caster_before.equipment_revision
                    && impact.target.life
                        >= recipient_before.life.checked_add(1).ok_or_else(invalid)?
                    && impact.target.equipment_revision >= recipient_before.equipment_revision,
            )?;
            let old = self
                .healing_actor(caster)?
                .outgoing
                .as_ref()
                .ok_or_else(invalid)?;
            let new = next
                .healing_actor(caster)?
                .outgoing
                .as_ref()
                .ok_or_else(invalid)?;
            let a = &self.healing_actor(recipient)?.incoming;
            let b = &next.healing_actor(recipient)?.incoming;
            require(
                new.ledger.casts > old.ledger.casts
                    && b.received > a.received
                    && i128::from(new.ledger.spent) - i128::from(old.ledger.spent)
                        >= i128::from(impact.usage.cost)
                    && i128::from(b.restored) - i128::from(a.restored)
                        >= i128::from(impact.target.applied),
            )?;
        }
        Ok(())
    }
}
impl Status {
    pub(in crate::modern_lito::battle_payload) fn check_successor(
        &self,
        d: &UnvalidatedData,
        next: &Self,
        n: &UnvalidatedData,
    ) -> Result<(), ApplicationError> {
        match (self, next) {
            (Self::Motion(a), Self::Motion(b)) => {
                super::super::npc_motion::MotionView { data: d, motion: a }
                    .check_successor(&super::super::npc_motion::MotionView { data: n, motion: b })
            }
            (Self::Haste(a), Self::Haste(b)) => super::super::haste::View {
                data: d,
                effects: a,
            }
            .check_successor(&super::super::haste::View {
                data: n,
                effects: b,
            }),
            (Self::Speed(a), Self::Speed(b)) => super::super::speed::View {
                data: d,
                effects: a,
            }
            .check_successor(&super::super::speed::View {
                data: n,
                effects: b,
            }),
            _ => Err(invalid()),
        }
    }
}
fn outgoing(
    a: &Outgoing,
    b: &Outgoing,
    rev: i64,
    budget: i64,
    old_sequence: i64,
    old_loading_elapsed: i64,
) -> Result<i64, ApplicationError> {
    let x = &a.ledger;
    let y = &b.ledger;
    let casts = y.casts.checked_sub(x.casts).ok_or_else(invalid)?;
    let spent = y.spent.checked_sub(x.spent).ok_or_else(invalid)?;
    require(
        (0..=budget).contains(&casts)
            && spent >= 0
            && i128::from(spent) <= i128::from(casts) * 4
            && (0..=budget).contains(
                &y.equipment_revision
                    .checked_sub(x.equipment_revision)
                    .ok_or_else(invalid)?,
            )
            && y.profile_revision >= x.profile_revision
            && y.prior_casts >= x.prior_casts
            && y.prior_spent >= x.prior_spent,
    )?;
    if x.equipment_revision == y.equipment_revision {
        require(
            x.profile_revision == y.profile_revision
                && x.cost == y.cost
                && x.prior_casts == y.prior_casts
                && x.prior_spent == y.prior_spent,
        )?;
    } else {
        require(y.profile_revision > rev && y.prior_casts >= x.casts && y.prior_spent >= x.spent)?;
    }
    if casts == 0 {
        require(spent == 0 && a.receipt == b.receipt)?;
    } else {
        let last = b.receipt.as_ref().ok_or_else(invalid)?;
        require(
            last.impact.usage.revision > rev
                && last.impact.usage.sequence > old_sequence
                && last.loading_elapsed_nanos >= old_loading_elapsed
                && spent >= i64::from(last.impact.usage.cost)
                && (casts != 1 || spent == i64::from(last.impact.usage.cost)),
        )?;
        if let Some(old) = &a.receipt {
            require(
                last.loading_elapsed_nanos >= old.loading_elapsed_nanos
                    && last.impact.usage.before.life >= old.impact.usage.before.life
                    && last.impact.usage.before.equipment_revision
                        >= old.impact.usage.before.equipment_revision,
            )?;
        }
    }
    Ok(casts)
}
