//! Candidate-only timeline and retained-event admission. No transport,
//! shared-resource, rendering or command permission follows from this proof.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectId {
    pub world: WorldId,
    pub entity: i64,
    /// Zero-based resource life; not a graphics or equipment revision.
    pub life: i64,
    pub revision: i64,
    pub index: usize,
}
#[derive(Clone, PartialEq, Eq)]
pub struct EffectEvent {
    pub id: EffectId,
    pub observed_nanos: i64,
    pub effect: Effect,
}
impl std::fmt::Debug for EffectEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CandidateArmorEffect[redacted]")
    }
}
#[derive(Clone)]
pub struct Follower {
    player: Player,
    world: WorldId,
    current: Option<Checked>,
}
impl std::fmt::Debug for Follower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CandidateArmorFollower[redacted]")
    }
}
impl Follower {
    pub fn new(player: Player, world: WorldId) -> Result<Self, ApplicationError> {
        require(world.room > 0 && world.match_id > 0 && world.generation > 0)?;
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
    /// First attachment suppresses retained historical effects. Subsequent
    /// snapshots report only newly accepted retained events; missing frames
    /// cannot be reconstructed. Rejection publishes neither state nor events.
    pub fn accept(&mut self, candidate: Candidate) -> Result<Vec<EffectEvent>, ApplicationError> {
        let next = candidate.checked(&self.player, &self.world)?;
        let events = if let Some(old) = &self.current {
            old.candidate().check_successor(next.candidate())?;
            retained_events(next.candidate(), old.candidate().data.world.revision)?
        } else {
            Vec::new()
        };
        self.current = Some(next);
        Ok(events)
    }
}
impl View<'_> {
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        crate::modern_lito::battle_follow::check_armor_view_sequence(self, next)?;
        self.check_successor_body(next)
    }
    /// Shared only after both complete scenes and their base timeline pass.
    pub(in crate::modern_lito::battle_payload) fn check_successor_body(
        &self,
        next: &Self,
    ) -> Result<(), ApplicationError> {
        super::super::armor_presentation::check_successor(self, next)?;
        match (&self.healing, &next.healing) {
            (Some(a), Some(b)) => healing::View {
                data: &self.data,
                status: &self.status,
                actors: a,
            }
            .check_successor_body(&healing::View {
                data: &next.data,
                status: &next.status,
                actors: b,
            })?,
            (None, None) => self
                .status
                .check_successor(&self.data, &next.status, &next.data)?,
            _ => return Err(invalid()),
        }
        let old = &self.data.world;
        let new = &next.data.world;
        if old.revision == new.revision || matches!(old.phase, Phase::Aborted | Phase::Finished) {
            return require(self.actors == next.actors);
        }
        for (a, b) in self.actors.0.iter().zip(&next.actors.0) {
            require(a.entity == b.entity)?;
            match (&a.state, &b.state) {
                (None, None) => (),
                (Some(a), Some(b)) => state_follows(
                    a,
                    b,
                    old.revision,
                    new.revision,
                    old.active_elapsed_nanos.max(0),
                    can_clear(&next.data, b.entity)?,
                )?,
                _ => return Err(invalid()),
            }
        }
        Ok(())
    }
}
fn can_clear(d: &UnvalidatedData, entity: i64) -> Result<bool, ApplicationError> {
    let actor = d.scene_actor(entity)?;
    Ok(matches!(d.world.phase, Phase::Aborted | Phase::Finished)
        || actor.resources.as_ref().ok_or_else(invalid)?.vitals.hp == 0
        || d.time_attack
            .as_ref()
            .is_some_and(|t| t.departed.0.contains(&entity))
        || d.race.as_ref().is_some_and(|r| {
            r.runners
                .0
                .iter()
                .any(|r| r.entity == entity && (r.dnf != Dnf::None || r.placement != 0))
        }))
}
/// Called only after both complete scenes and the base timeline are proved.
fn state_follows(
    a: &State,
    b: &State,
    old_revision: i64,
    new_revision: i64,
    old_clock: i64,
    clearing: bool,
) -> Result<(), ApplicationError> {
    require(a.entity == b.entity && b.life >= a.life && b.revision >= a.revision)?;
    if a == b {
        return Ok(());
    }
    require(b.revision > old_revision)?;
    let budget = new_revision.checked_sub(old_revision).ok_or_else(invalid)?;
    if b.life > a.life {
        require(b.sequence <= budget && b.life - a.life <= budget)?;
        return require(b.sequence == 0 || b.observed_nanos >= old_clock);
    }
    let delta = b.sequence.checked_sub(a.sequence).ok_or_else(invalid)?;
    require(
        (0..=budget).contains(&delta)
            && b.observed_nanos >= a.observed_nanos
            && b.observed_nanos >= old_clock,
    )?;
    if delta == 0 {
        return require(
            clearing
                && (a.until_nanos != 0 || !a.effects.0.is_empty())
                && b.until_nanos == 0
                && b.effects.0.is_empty(),
        );
    }
    // Damage absorbed by protection may still publish effects. Do not infer
    // event identity or hit count from HP loss, equipment or graphics IDs.
    if a.until_nanos > b.observed_nanos {
        require(if b.until_nanos != 0 {
            b.until_nanos == a.until_nanos && b.effects.0.iter().all(|e| !e.applied_protection)
        } else {
            clearing
        })?;
    } else if b.until_nanos != 0 && delta == 1 {
        require(b.effects.0.iter().any(|e| e.applied_protection))?;
    }
    Ok(())
}
fn retained_events(
    next: &Candidate,
    old_revision: i64,
) -> Result<Vec<EffectEvent>, ApplicationError> {
    retained_value_events(&next.data, &next.actors, old_revision)
}
pub(in crate::modern_lito::battle_payload) fn retained_value_events(
    data: &UnvalidatedData,
    actors: &List<Actor, 100>,
    old_revision: i64,
) -> Result<Vec<EffectEvent>, ApplicationError> {
    let count: usize = actors
        .0
        .iter()
        .filter_map(|a| a.state.as_ref())
        .filter(|s| s.revision > old_revision)
        .map(|s| s.effects.0.len())
        .sum();
    let mut events = Vec::new();
    events.try_reserve_exact(count).map_err(|_| invalid())?;
    for state in actors.0.iter().filter_map(|a| a.state.as_ref()) {
        if state.revision <= old_revision {
            continue;
        }
        for (index, effect) in state.effects.0.iter().enumerate() {
            events.push(EffectEvent {
                id: EffectId {
                    world: data.world.identity.clone(),
                    entity: state.entity,
                    life: state.life,
                    revision: state.revision,
                    index,
                },
                observed_nanos: state.observed_nanos,
                effect: effect.clone(),
            });
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn initial() -> State {
        State {
            entity: 1,
            life: 0,
            sequence: 0,
            revision: 1,
            observed_nanos: 0,
            until_nanos: 0,
            effects: List(vec![]),
        }
    }
    fn hit() -> State {
        State {
            sequence: 1,
            revision: 5,
            observed_nanos: SECOND,
            until_nanos: 4 * SECOND,
            effects: List(vec![Effect {
                graphics: 11101,
                graphics_mode: 1,
                applied_protection: true,
                duration_seconds: 3,
            }]),
            ..initial()
        }
    }
    fn follow(a: &State, b: &State, old: i64, new: i64, clock: i64, clear: bool) -> bool {
        a.shape().unwrap();
        b.shape().unwrap();
        state_follows(a, b, old, new, clock, clear).is_ok()
    }
    #[test]
    fn hit_absorption_expiry_clear_and_new_life_keep_distinct_sequences() {
        let a = initial();
        let b = hit();
        assert!(follow(&a, &b, 4, 5, 0, false));
        let mut c = b.clone();
        c.sequence += 1;
        c.revision += 1;
        c.observed_nanos = 2 * SECOND;
        c.effects.0[0].applied_protection = false;
        assert!(follow(&b, &c, 5, 6, SECOND, false));
        let mut expire = c.clone();
        expire.sequence += 1;
        expire.revision += 1;
        expire.observed_nanos = 4 * SECOND;
        expire.until_nanos = 0;
        expire.effects.0.clear();
        assert!(follow(&c, &expire, 6, 7, 2 * SECOND, false));
        let mut cleared = c.clone();
        cleared.revision += 1;
        cleared.until_nanos = 0;
        cleared.effects.0.clear();
        assert!(follow(&c, &cleared, 6, 7, 2 * SECOND, true));
        assert!(!follow(&c, &cleared, 6, 7, 2 * SECOND, false));
        let reset = State {
            life: 1,
            revision: 9,
            ..initial()
        };
        assert!(follow(&expire, &reset, 7, 9, 4 * SECOND, false));
        let mut next_hit = hit();
        next_hit.life = 1;
        next_hit.revision = 10;
        next_hit.observed_nanos = 5 * SECOND;
        next_hit.until_nanos = 8 * SECOND;
        assert!(follow(&expire, &next_hit, 7, 10, 4 * SECOND, false));
    }
    #[test]
    fn timeline_rejects_old_revisions_replayed_grants_refresh_and_regression() {
        let a = hit();
        for mutation in 0..8 {
            let mut b = a.clone();
            b.sequence = 2;
            b.revision = 7;
            b.observed_nanos = 2 * SECOND;
            b.effects.0[0].applied_protection = false;
            match mutation {
                0 => b.revision = 6, // already crossed this observation
                1 => {
                    b.sequence = 1;
                    b.effects.0.clear();
                }
                2 => b.sequence = 4, // one world revision cannot accept three hits
                3 => b.observed_nanos = 0,
                4 => b.until_nanos += SECOND,
                5 => {
                    b.effects.0[0].applied_protection = true;
                    b.until_nanos = 5 * SECOND;
                }
                6 => {
                    b.until_nanos = 0;
                    b.effects.0.clear();
                }
                _ => b.entity = 2,
            }
            assert!(!follow(&a, &b, 6, 7, SECOND, false), "{mutation}");
        }
        let mut false_grant = hit();
        false_grant.effects.0[0].applied_protection = false;
        assert!(!follow(&initial(), &false_grant, 4, 5, 0, false));
        let mut reset = initial();
        reset.life = 1;
        reset.revision = 8;
        let mut old_life = reset.clone();
        old_life.life = 0;
        old_life.revision = 9;
        assert!(!follow(&reset, &old_life, 8, 9, SECOND, false));
    }
}
