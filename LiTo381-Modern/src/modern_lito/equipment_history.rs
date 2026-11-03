//! Complete scoped equipment/history candidate. Not a public wire capability
//! or proof of permission to equip; old world followers remain strict.
use super::{
    application_protocol::SelectionContext,
    target_history::{Batch, Effect, History, Lane, Participant, World},
    use_history::{Anchor, Items, Spell},
    vital_balance,
    world_map::WorldCell,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub world: World,
    pub source: SelectionContext,
    pub channel_id: u8,
    pub actor: Participant,
}
impl Identity {
    pub fn valid(self) -> bool {
        self.world.valid()
            && self.source.epoch != 0
            && self.channel_id <= 3
            && self.actor.valid()
            && self.actor.identity.matches_character(self.source.key)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub equipment_revision: i64,
    pub maximum_hp: i64,
    pub maximum_mp: i64,
    pub costs: BTreeMap<Lane, i32>,
    pub fixed_recovery: Option<i64>,
}
impl Profile {
    pub fn valid(&self) -> bool {
        self.equipment_revision >= 0
            && self.maximum_hp > 0
            && self.maximum_mp >= 0
            && self.costs.values().all(|&c| c >= 0)
            && self.fixed_recovery.map_or(true, |n| n >= 0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub identity: Identity,
    pub world_revision: i64,
    pub resources: vital_balance::State,
    pub spells: BTreeMap<Lane, Spell>,
    pub items: Option<Items>,
    pub targets: BTreeMap<Lane, History>,
    pub healing: Option<super::player_healing_history::Entry>,
}
impl State {
    pub fn valid(&self) -> bool {
        if !self.identity.valid()
            || !self.resources.valid()
            || self.world_revision < self.resources.revision
            || self.targets.keys().any(|k| !self.spells.contains_key(k))
        {
            return false;
        }
        let mut spent = 0i64;
        let mut profile_revision = None;
        let mut revisions = BTreeSet::new();
        let mut sequences = BTreeSet::new();
        for (&lane, spell) in &self.spells {
            let ledger = spell.current;
            if !spell.valid()
                || ledger.equipment_revision != self.resources.equipment_revision
                || ledger.profile_revision > self.resources.revision
                || profile_revision.is_some_and(|r| r != ledger.profile_revision)
            {
                return false;
            }
            profile_revision = Some(ledger.profile_revision);
            let Some(total) = spent.checked_add(ledger.spent) else {
                return false;
            };
            spent = total;
            let target = self.targets.get(&lane);
            let Some(last) = spell.last else {
                if target.is_some() {
                    return false;
                }
                continue;
            };
            let Some(target) = target else {
                return false;
            };
            if !anchor_valid(self.resources, last.before, last.revision)
                || !revisions.insert(last.revision)
                || !sequences.insert(last.sequence)
                || !target.valid()
                || target.last.use_receipt != last
            {
                return false;
            }
            let b = &target.last;
            if b.world != self.identity.world
                || b.source != self.identity.source
                || b.channel_id != self.identity.channel_id
                || b.caster != self.identity.actor
                || b.lane != lane
            {
                return false;
            }
        }
        let mut restored = 0i64;
        if let Some(healing) = self.healing {
            if !healing.valid_for(self.identity, self.resources) {
                return false;
            }
            restored = healing.incoming.restored;
            if let Some(outgoing) = healing.outgoing {
                let ledger = outgoing.spending.current;
                if ledger.equipment_revision != self.resources.equipment_revision
                    || ledger.profile_revision > self.resources.revision
                    || profile_revision.is_some_and(|r| r != ledger.profile_revision)
                {
                    return false;
                }
                profile_revision = Some(ledger.profile_revision);
                let Some(total) = spent.checked_add(ledger.spent) else {
                    return false;
                };
                spent = total;
                if let Some(sent) = outgoing.last {
                    let u = sent.use_receipt;
                    if !anchor_valid(self.resources, u.before, u.revision)
                        || !revisions.insert(u.revision)
                        || !sequences.insert(u.sequence)
                    {
                        return false;
                    }
                }
            }
            if let Some(incoming) = healing.incoming.last {
                let same = healing
                    .outgoing
                    .and_then(|o| o.last)
                    .is_some_and(|s| s.impact() == incoming);
                if !same && !revisions.insert(incoming.use_receipt.revision)
                    || incoming.is_self()
                        && !same
                        && !sequences.insert(incoming.use_receipt.sequence)
                {
                    return false;
                }
            }
        }
        if spent != self.resources.flows.spent {
            return false;
        }
        match self.items {
            None => self.resources.flows.restored == restored,
            Some(items) => {
                items.valid()
                    && items.equipment_revision == self.resources.equipment_revision
                    && items.profile_revision <= self.resources.revision
                    && profile_revision.map_or(true, |r| r == items.profile_revision)
                    && restored.checked_add(items.restored) == Some(self.resources.flows.restored)
                    && items.last.map_or(true, |last| {
                        anchor_valid(self.resources, last.before, last.revision)
                            && revisions.insert(last.revision)
                    })
            }
        }
    }
    pub fn profile(&self) -> Option<Profile> {
        self.valid().then(|| Profile {
            equipment_revision: self.resources.equipment_revision,
            maximum_hp: self.resources.vitals.maximum_hp,
            maximum_mp: self.resources.vitals.maximum_mp,
            costs: self
                .spells
                .iter()
                .map(|(&lane, h)| (lane, h.current.cost))
                .collect(),
            fixed_recovery: self.items.map(|i| i.fixed_recovery),
        })
    }
    pub fn reprofile(&self, next: i64, profile: &Profile) -> Option<Self> {
        self.reprofile_with_healing(next, profile, None)
    }
    /// The caller must explicitly supply a newly qualified healing cost.
    pub fn reprofile_with_healing(
        &self,
        next: i64,
        profile: &Profile,
        healing_cost: Option<i32>,
    ) -> Option<Self> {
        if !self.valid()
            || !profile.valid()
            || next <= self.world_revision
            || !self.spells.keys().eq(profile.costs.keys())
            || self.items.is_some() != profile.fixed_recovery.is_some()
            || self.healing.is_none() && healing_cost.is_some()
        {
            return None;
        }
        let resources = self.resources.equip(
            next,
            profile.equipment_revision,
            profile.maximum_hp,
            profile.maximum_mp,
        )?;
        let spells = self
            .spells
            .iter()
            .map(|(&lane, s)| {
                Some((
                    lane,
                    s.reprice(profile.equipment_revision, next, profile.costs[&lane])?,
                ))
            })
            .collect::<Option<BTreeMap<_, _>>>()?;
        let items = match self.items {
            None => None,
            Some(items) => {
                Some(items.reprofile(profile.equipment_revision, next, profile.fixed_recovery?)?)
            }
        };
        let value = Self {
            identity: self.identity,
            world_revision: next,
            resources,
            spells,
            items,
            targets: self.targets.clone(),
            healing: match self.healing {
                None => None,
                Some(h) => Some(h.reprice(profile.equipment_revision, next, healing_cost)?),
            },
        };
        value.valid().then_some(value)
    }
    /// Exact self-resource/target receipt commit, not target authorization.
    pub fn cast(
        &self,
        lane: Lane,
        next: i64,
        sequence: i64,
        origin: WorldCell,
        effects: Vec<Effect>,
    ) -> Option<Self> {
        if !self.valid()
            || next <= self.world_revision
            || self
                .healing
                .and_then(|h| h.outgoing)
                .is_some_and(|o| sequence <= o.spending.current.last_cast_sequence)
            || self
                .spells
                .values()
                .any(|s| sequence <= s.current.last_cast_sequence)
        {
            return None;
        }
        let spell = self.spells.get(&lane)?.cast(
            next,
            sequence,
            Anchor {
                equipment_revision: self.resources.equipment_revision,
                life: self.resources.life.checked_add(1)?,
                vitals: self.resources.vitals,
                x: origin.x,
                y: origin.y,
            },
        )?;
        let use_receipt = spell.last?;
        let resources = self.resources.spend(next, i64::from(use_receipt.cost))?;
        let batch = Batch {
            world: self.identity.world,
            source: self.identity.source,
            channel_id: self.identity.channel_id,
            caster: self.identity.actor,
            lane,
            use_receipt,
            targets: effects,
        };
        let history = match self.targets.get(&lane) {
            Some(old) => old.commit(batch)?,
            None => History::first(batch)?,
        };
        let mut value = self.clone();
        value.world_revision = next;
        value.resources = resources;
        value.spells.insert(lane, spell);
        value.targets.insert(lane, history);
        value.valid().then_some(value)
    }
}
fn anchor_valid(resources: vital_balance::State, anchor: Anchor, revision: i64) -> bool {
    revision <= resources.revision
        && anchor.life <= resources.life + 1
        && anchor.equipment_revision <= resources.equipment_revision
        && (anchor.equipment_revision != resources.equipment_revision
            || anchor.vitals.maximum_hp == resources.vitals.maximum_hp
                && anchor.vitals.maximum_mp == resources.vitals.maximum_mp)
}
pub fn follows_reprofile(before: &State, after: &State) -> bool {
    after
        .profile()
        .and_then(|p| {
            before.reprofile_with_healing(
                after.world_revision,
                &p,
                after
                    .healing
                    .and_then(|h| h.outgoing)
                    .map(|o| o.spending.current.cost),
            )
        })
        .as_ref()
        == Some(after)
}
pub fn follows_cast(before: &State, after: &State, lane: Lane) -> bool {
    if !after.valid() {
        return false;
    }
    let Some(history) = after.targets.get(&lane) else {
        return false;
    };
    let use_receipt = history.last.use_receipt;
    before
        .cast(
            lane,
            after.world_revision,
            use_receipt.sequence,
            WorldCell {
                x: use_receipt.before.x,
                y: use_receipt.before.y,
            },
            history.last.targets.clone(),
        )
        .as_ref()
        == Some(after)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::modern_lito::{
        application_protocol::CharacterKey,
        equipment_transition::Vitals,
        npc_protocol as npc,
        participant::{MemberKind, ParticipantId},
        quest_protocol,
        target_history::{Batch, Effect, Health, Kind, Target},
        world_map::WorldCell,
    };
    const LANES: [Lane; 8] = [
        Lane::Teleport,
        Lane::LightArrow,
        Lane::Healing,
        Lane::FireStorm,
        Lane::Disintegrate,
        Lane::NpcLightArrow,
        Lane::NpcFireStorm,
        Lane::NpcDisintegrate,
    ];
    fn identity() -> Identity {
        Identity {
            world: World {
                room_id: 10,
                match_id: 20,
                generation: 30,
            },
            source: SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 4,
            },
            channel_id: 0,
            actor: Participant {
                entity_id: 1,
                identity: ParticipantId {
                    kind: MemberKind::Human,
                    source_id: 7,
                    id: 3333,
                },
            },
        }
    }
    fn anchor(r: vital_balance::State) -> Anchor {
        Anchor {
            equipment_revision: r.equipment_revision,
            life: r.life + 1,
            vitals: r.vitals,
            x: 32770,
            y: 32771,
        }
    }
    pub(crate) fn impacts(lane: Lane) -> Vec<Effect> {
        let scope = npc::Scope {
            room_id: 10,
            match_id: 20,
            generation: 30,
        };
        let target = match lane {
            Lane::Teleport | Lane::GreaterHaste | Lane::Haste | Lane::Slow => return vec![],
            Lane::Healing => Target::Honor {
                entity_id: 3,
                team: 0,
            },
            Lane::LightArrow | Lane::FireStorm | Lane::Disintegrate => {
                Target::Participant(Participant {
                    entity_id: 2,
                    identity: ParticipantId {
                        kind: MemberKind::Human,
                        source_id: 42,
                        id: 3333,
                    },
                })
            }
            Lane::NpcLightArrow | Lane::NpcDisintegrate => {
                Target::Npc(npc::Id { scope, serial: 1 })
            }
            Lane::NpcFireStorm => Target::Quest {
                id: quest_protocol::Id { scope, slot: 1 },
                born_revision: 1,
            },
        };
        let healing = lane == Lane::Healing;
        vec![Effect {
            kind: if healing { Kind::Heal } else { Kind::Damage },
            before: Health {
                target,
                life: if matches!(target, Target::Participant(_) | Target::Npc(_)) {
                    2
                } else {
                    1
                },
                equipment_revision: 0,
                cell: WorldCell { x: 32770, y: 32771 },
                hp: if healing { 100 } else { 50 },
                maximum_hp: 100,
            },
            raw: if healing { 5 } else { 100 },
            applied: if healing { 0 } else { 50 },
            hp_after: if healing { 100 } else { 0 },
        }]
    }
    pub(crate) fn populated() -> State {
        let id = identity();
        let mut r = vital_balance::State::initial(
            1,
            0,
            Vitals {
                hp: 1000,
                maximum_hp: 1000,
                mp: 1000,
                maximum_mp: 1000,
            },
        )
        .unwrap()
        .damage(2, 100)
        .unwrap();
        let mut spells = BTreeMap::new();
        let mut targets = BTreeMap::new();
        for (i, lane) in LANES.into_iter().enumerate() {
            let revision = i as i64 + 3;
            let spell = Spell::initial(0, 1, 10)
                .unwrap()
                .cast(revision, i as i64 + 1, anchor(r))
                .unwrap();
            let batch = Batch {
                world: id.world,
                source: id.source,
                channel_id: 0,
                caster: id.actor,
                lane,
                use_receipt: spell.last.unwrap(),
                targets: impacts(lane),
            };
            targets.insert(lane, History::first(batch).unwrap());
            spells.insert(lane, spell);
            r = r.spend(revision, 10).unwrap();
        }
        let item = Items::initial(0, 1, 5)
            .unwrap()
            .use_item(11, 1, 40010, anchor(r), 10)
            .unwrap();
        r = r.restore(11, item.last.unwrap().restored).unwrap();
        let state = State {
            identity: id,
            world_revision: 11,
            resources: r,
            spells,
            items: Some(item),
            targets,
            healing: None,
        };
        assert!(state.valid());
        state
    }
    pub(crate) fn next(s: &State, hp: i64, mp: i64) -> Profile {
        Profile {
            equipment_revision: s.resources.equipment_revision + 1,
            maximum_hp: hp,
            maximum_mp: mp,
            costs: LANES
                .into_iter()
                .enumerate()
                .map(|(i, l)| (l, i as i32))
                .collect(),
            fixed_recovery: Some(7),
        }
    }
    #[test]
    fn all_lanes_reprice_without_rewriting_old_capacities_costs_or_structure_receipts() {
        let a = populated();
        let b = a.reprofile(12, &next(&a, 100, 50)).unwrap();
        assert!(follows_reprofile(&a, &b));
        assert_eq!(
            (80, 15, 815, 870),
            (
                b.resources.flows.spent,
                b.resources.flows.restored,
                b.resources.flows.hp_clipped,
                b.resources.flows.mp_clipped
            )
        );
        assert_eq!(a.targets, b.targets);
        assert_eq!(a.items.unwrap().last, b.items.unwrap().last);
        for (i, lane) in LANES.into_iter().enumerate() {
            assert_eq!(a.spells[&lane].last, b.spells[&lane].last);
            assert_eq!(
                (i as i32, 10, 12),
                (
                    b.spells[&lane].current.cost,
                    b.spells[&lane].current.spent,
                    b.spells[&lane].current.profile_revision
                )
            );
        }
        let c = b.reprofile(13, &next(&b, 2000, 2000)).unwrap();
        assert_eq!((100, 50), (c.resources.vitals.hp, c.resources.vitals.mp));
        assert_eq!(1000, c.resources.birth_hp);
        assert!(follows_reprofile(&b, &c));
        assert!(!follows_reprofile(&a, &c));
        assert!(!follows_reprofile(&a, &a));
    }
    #[test]
    fn partial_capabilities_skipped_epochs_and_invalid_raw_values_fail_closed() {
        let a = populated();
        let p = next(&a, 100, 50);
        let mut missing = p.clone();
        missing.costs.remove(&Lane::Teleport);
        assert!(a.reprofile(12, &missing).is_none());
        let mut no_items = p.clone();
        no_items.fixed_recovery = None;
        assert!(a.reprofile(12, &no_items).is_none());
        assert!(a.reprofile(11, &p).is_none());
        let mut skipped = p.clone();
        skipped.equipment_revision = 2;
        assert!(a.reprofile(12, &skipped).is_none());
        let mut invalid = p.clone();
        invalid.costs.insert(Lane::Teleport, -1);
        assert!(a.reprofile(12, &invalid).is_none());
        let mut malformed = a.clone();
        malformed.resources.birth_hp -= 1;
        assert!(!malformed.valid());
        assert!(malformed.reprofile(12, &p).is_none());
    }
    #[test]
    fn resources_and_individually_valid_histories_must_agree() {
        let a = populated();
        let mut b = a.clone();
        b.resources = vital_balance::State::initial(11, 0, a.resources.vitals).unwrap();
        assert!(!b.valid());
        let mut b = a.clone();
        b.items = None;
        assert!(!b.valid());
        let mut b = a.clone();
        b.world_revision = 10;
        assert!(!b.valid());
        let mut b = a.clone();
        b.resources.vitals.maximum_hp += 1;
        assert!(b.resources.valid());
        assert!(!b.valid());
        let mut b = a.clone();
        b.spells.insert(
            Lane::Teleport,
            a.spells[&Lane::Teleport].reprice(1, 12, 10).unwrap(),
        );
        assert!(!b.valid());
    }
    #[test]
    fn world_selection_channel_and_typed_actor_cannot_borrow_target_batches() {
        let a = populated();
        for which in 0..6 {
            let mut b = a.clone();
            match which {
                0 => b.identity.world.match_id += 1,
                1 => b.identity.source.epoch += 1,
                2 => b.identity.channel_id += 1,
                3 => b.identity.actor.entity_id += 1,
                4 => b.identity.actor.identity.kind = MemberKind::ServerBot,
                _ => b.identity.source.key.source_id = 42,
            };
            assert!(!b.valid());
            assert!(!follows_reprofile(&a, &b));
        }
    }
    #[test]
    fn duplicate_casts_across_lanes_and_rewritten_valid_targets_are_not_equipment_changes() {
        let a = populated();
        let mut b = a.clone();
        let first = a.spells[&Lane::Teleport];
        b.spells.insert(Lane::LightArrow, first);
        b.targets
            .get_mut(&Lane::LightArrow)
            .unwrap()
            .last
            .use_receipt = first.last.unwrap();
        assert!(b.targets[&Lane::LightArrow].valid());
        assert!(!b.valid());
        let mut b = a.reprofile(12, &next(&a, 100, 50)).unwrap();
        let effect = &mut b.targets.get_mut(&Lane::LightArrow).unwrap().last.targets[0];
        effect.before.hp = 40;
        effect.applied = 40;
        assert!(b.valid());
        assert!(!follows_reprofile(&a, &b));
    }
    #[test]
    fn empty_history_cannot_rebind_world_or_replace_birth_with_current_resources() {
        let a = State {
            identity: identity(),
            world_revision: 1,
            resources: vital_balance::State::initial(
                1,
                0,
                Vitals {
                    hp: 100,
                    maximum_hp: 100,
                    mp: 50,
                    maximum_mp: 50,
                },
            )
            .unwrap(),
            spells: BTreeMap::new(),
            items: None,
            targets: BTreeMap::new(),
            healing: None,
        };
        let b = a
            .reprofile(
                2,
                &Profile {
                    equipment_revision: 1,
                    maximum_hp: 10,
                    maximum_mp: 5,
                    costs: BTreeMap::new(),
                    fixed_recovery: None,
                },
            )
            .unwrap();
        let mut other = b.clone();
        other.identity.world.room_id += 1;
        assert!(other.valid());
        assert!(!follows_reprofile(&a, &other));
        let mut reset = b.clone();
        reset.resources = vital_balance::State::initial(2, 1, b.resources.vitals).unwrap();
        assert!(reset.valid());
        assert!(!follows_reprofile(&a, &reset));
        assert_eq!(
            (90, 45),
            (b.resources.flows.hp_clipped, b.resources.flows.mp_clipped)
        );
    }
    #[test]
    fn old_structure_cannot_be_erased_by_repricing_after_a_later_empty_cast() {
        let mut a = populated();
        let lane = Lane::NpcFireStorm;
        let spell = a.spells[&lane].cast(12, 9, anchor(a.resources)).unwrap();
        let id = a.identity;
        let batch = Batch {
            world: id.world,
            source: id.source,
            channel_id: id.channel_id,
            caster: id.actor,
            lane,
            use_receipt: spell.last.unwrap(),
            targets: vec![],
        };
        let history = a.targets[&lane].commit(batch).unwrap();
        a.targets.insert(lane, history);
        a.spells.insert(lane, spell);
        a.resources = a.resources.spend(12, 10).unwrap();
        a.world_revision = 12;
        assert!(a.valid());
        let b = a.reprofile(13, &next(&a, 100, 50)).unwrap();
        assert!(follows_reprofile(&a, &b));
        let mut erased = b.clone();
        erased.targets.get_mut(&lane).unwrap().structure = None;
        assert!(erased.valid());
        assert!(!follows_reprofile(&a, &erased));
    }
}
