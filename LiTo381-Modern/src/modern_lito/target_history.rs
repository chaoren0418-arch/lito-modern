//! Event-time target health, independent of subsequent equipment or lives.
//! This candidate contract is not yet a wire capability or skill permission.
use super::{
    application_protocol::SelectionContext, npc_protocol as npc, participant::ParticipantId,
    quest_protocol, time_attack_protocol, use_history::Cast, world_map::WorldCell,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct World {
    pub room_id: u64,
    pub match_id: u64,
    pub generation: u64,
}
fn positive(n: u64) -> bool {
    n > 0 && n <= i64::MAX as u64
}
impl World {
    pub fn valid(self) -> bool {
        positive(self.room_id) && positive(self.match_id) && positive(self.generation)
    }
    fn contains(self, s: npc::Scope) -> bool {
        self.valid()
            && self.room_id == s.room_id
            && self.match_id == s.match_id
            && self.generation == s.generation
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Participant {
    pub entity_id: u64,
    pub identity: ParticipantId,
}
impl Participant {
    pub fn valid(self) -> bool {
        (1..=100).contains(&self.entity_id) && self.identity.valid()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Participant(Participant),
    Npc(npc::Id),
    Honor {
        entity_id: u64,
        team: u8,
    },
    Quest {
        id: quest_protocol::Id,
        born_revision: i64,
    },
    TimeTower(time_attack_protocol::TowerId),
}
impl Target {
    fn valid(self) -> bool {
        let scope = |s: npc::Scope| {
            World {
                room_id: s.room_id,
                match_id: s.match_id,
                generation: s.generation,
            }
            .valid()
        };
        match self {
            Self::Participant(p) => p.valid(),
            Self::Npc(n) => scope(n.scope) && positive(n.serial),
            Self::Honor { entity_id, team } => (3..=102).contains(&entity_id) && team <= 1,
            Self::Quest { id, born_revision } => {
                scope(id.scope) && (1..=8).contains(&id.slot) && born_revision >= 1
            }
            Self::TimeTower(t) => scope(t.scope),
        }
    }
    fn structure(self) -> bool {
        matches!(self, Self::Quest { .. } | Self::TimeTower(_))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Health {
    pub target: Target,
    pub life: i64,
    pub equipment_revision: i64,
    pub cell: WorldCell,
    pub hp: i64,
    pub maximum_hp: i64,
}
impl Health {
    pub fn valid(self) -> bool {
        self.target.valid()
            && self.life >= 1
            && self.equipment_revision >= 0
            && self.maximum_hp >= 1
            && self.hp >= 1
            && self.hp <= self.maximum_hp
            && (matches!(self.target, Target::Participant(_)) || self.equipment_revision == 0)
            && (matches!(self.target, Target::Participant(_) | Target::Npc(_)) || self.life == 1)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Damage,
    Heal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effect {
    pub kind: Kind,
    pub before: Health,
    pub raw: i64,
    pub applied: i64,
    pub hp_after: i64,
}
impl Effect {
    pub fn valid(self) -> bool {
        if !self.before.valid() || self.raw < 0 || self.applied < 0 {
            return false;
        }
        let expected = match self.kind {
            Kind::Damage => self.raw.min(self.before.hp),
            Kind::Heal => self.raw.min(self.before.maximum_hp - self.before.hp),
        };
        self.applied == expected
            && match self.kind {
                Kind::Damage => self.hp_after == self.before.hp - expected,
                Kind::Heal => self.before.hp.checked_add(expected) == Some(self.hp_after),
            }
    }
    pub fn death(self) -> bool {
        self.valid() && self.kind == Kind::Damage && self.hp_after == 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lane {
    Teleport,
    LightArrow,
    Healing,
    FireStorm,
    Disintegrate,
    NpcLightArrow,
    NpcFireStorm,
    NpcDisintegrate,
    GreaterHaste,
    Haste,
    Slow,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub world: World,
    pub source: SelectionContext,
    pub channel_id: u8,
    pub caster: Participant,
    pub lane: Lane,
    pub use_receipt: Cast,
    pub targets: Vec<Effect>,
}
impl Batch {
    pub fn valid(&self) -> bool {
        if !self.world.valid()
            || self.source.epoch == 0
            || self.channel_id > 3
            || !self.caster.valid()
            || !self.caster.identity.matches_character(self.source.key)
            || !self.use_receipt.valid()
        {
            return false;
        }
        let maximum = match self.lane {
            Lane::Teleport | Lane::GreaterHaste | Lane::Haste | Lane::Slow => 0,
            Lane::FireStorm => 101,
            Lane::NpcFireStorm => 48,
            _ => 1,
        };
        if self.targets.len() > maximum || maximum == 1 && self.targets.len() != 1 {
            return false;
        }
        let mut unique = Vec::new();
        let mut entities = Vec::new();
        let mut participants = Vec::new();
        let mut structures = 0;
        for effect in &self.targets {
            let t = effect.before.target;
            if !effect.valid()
                || unique.contains(&t)
                || effect.kind
                    != if self.lane == Lane::Healing {
                        Kind::Heal
                    } else {
                        Kind::Damage
                    }
            {
                return false;
            }
            unique.push(t);
            let admitted = match self.lane {
                Lane::Teleport | Lane::GreaterHaste | Lane::Haste | Lane::Slow => false,
                Lane::LightArrow | Lane::FireStorm => {
                    matches!(t, Target::Participant(_) | Target::Honor { .. })
                }
                Lane::Healing => matches!(t, Target::Honor { .. }),
                Lane::Disintegrate => matches!(t, Target::Participant(_)),
                Lane::NpcLightArrow | Lane::NpcDisintegrate => matches!(t, Target::Npc(_)),
                Lane::NpcFireStorm => matches!(
                    t,
                    Target::Npc(_) | Target::Quest { .. } | Target::TimeTower(_)
                ),
            };
            if !admitted {
                return false;
            }
            match t {
                Target::Participant(p) => {
                    if p.entity_id == self.caster.entity_id
                        || p.identity == self.caster.identity
                        || entities.contains(&p.entity_id)
                        || participants.contains(&p.identity)
                    {
                        return false;
                    }
                    entities.push(p.entity_id);
                    participants.push(p.identity);
                }
                Target::Honor { entity_id, .. } => {
                    if entity_id == self.caster.entity_id || entities.contains(&entity_id) {
                        return false;
                    }
                    entities.push(entity_id);
                }
                Target::Npc(n) => {
                    if !self.world.contains(n.scope) {
                        return false;
                    }
                }
                Target::Quest { id, born_revision } => {
                    if !self.world.contains(id.scope) || born_revision >= self.use_receipt.revision
                    {
                        return false;
                    }
                    structures += 1;
                }
                Target::TimeTower(t) => {
                    if !self.world.contains(t.scope) {
                        return false;
                    }
                    structures += 1;
                }
            }
        }
        structures <= 1
    }
    fn has_structure(&self) -> bool {
        self.targets.iter().any(|t| t.before.target.structure())
    }
    fn same_authority(&self, other: &Self) -> bool {
        self.world == other.world
            && self.source == other.source
            && self.channel_id == other.channel_id
            && self.caster == other.caster
            && self.lane == other.lane
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    pub last: Batch,
    pub structure: Option<Batch>,
}
impl History {
    pub fn valid(&self) -> bool {
        if !self.last.valid() {
            return false;
        }
        if self.last.has_structure() {
            return self.structure.as_ref() == Some(&self.last);
        }
        self.structure.as_ref().map_or(true, |s| {
            s.valid()
                && s.has_structure()
                && s.same_authority(&self.last)
                && s.use_receipt.revision < self.last.use_receipt.revision
                && s.use_receipt.sequence < self.last.use_receipt.sequence
                && s.use_receipt.before.life <= self.last.use_receipt.before.life
                && s.use_receipt.before.equipment_revision
                    <= self.last.use_receipt.before.equipment_revision
        })
    }
    pub fn first(batch: Batch) -> Option<Self> {
        if !batch.valid() {
            return None;
        }
        let structure = batch.has_structure().then(|| batch.clone());
        Some(Self {
            last: batch,
            structure,
        })
    }
    pub fn commit(&self, next: Batch) -> Option<Self> {
        if !self.valid()
            || !next.valid()
            || !self.last.same_authority(&next)
            || next.use_receipt.revision <= self.last.use_receipt.revision
            || next.use_receipt.sequence <= self.last.use_receipt.sequence
            || next.use_receipt.before.life < self.last.use_receipt.before.life
            || next.use_receipt.before.equipment_revision
                < self.last.use_receipt.before.equipment_revision
        {
            return None;
        }
        for before in &self.last.targets {
            for after in &next.targets {
                let a = before.before;
                let b = after.before;
                if a.target == b.target
                    && (b.life < a.life || b.equipment_revision < a.equipment_revision)
                {
                    return None;
                }
                if let (Target::Participant(p), Target::Participant(q)) = (a.target, b.target) {
                    if p.entity_id == q.entity_id && p.identity != q.identity
                        || p.identity == q.identity && p.entity_id != q.entity_id
                    {
                        return None;
                    }
                }
                let entity = |target| match target {
                    Target::Participant(p) => p.entity_id,
                    Target::Honor { entity_id, .. } => entity_id,
                    _ => 0,
                };
                if entity(a.target) != 0
                    && entity(a.target) == entity(b.target)
                    && std::mem::discriminant(&a.target) != std::mem::discriminant(&b.target)
                {
                    return None;
                }
            }
        }
        let structure = if next.has_structure() {
            Some(next.clone())
        } else {
            self.structure.clone()
        };
        Some(Self {
            last: next,
            structure,
        })
    }
    pub fn follows(&self, after: &Self) -> bool {
        self.valid()
            && after.valid()
            && (self == after || self.commit(after.last.clone()).as_ref() == Some(after))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        application_protocol::CharacterKey, equipment_transition::Vitals, participant::MemberKind,
        use_history::Anchor,
    };
    const WORLD: World = World {
        room_id: 10,
        match_id: 20,
        generation: 30,
    };
    const SCOPE: npc::Scope = npc::Scope {
        room_id: 10,
        match_id: 20,
        generation: 30,
    };
    fn participant(entity_id: u64, source_id: u32, id: u64, kind: MemberKind) -> Participant {
        Participant {
            entity_id,
            identity: ParticipantId {
                kind,
                source_id,
                id,
            },
        }
    }
    fn caster() -> Participant {
        participant(1, 7, 3333, MemberKind::Human)
    }
    fn use_receipt(revision: i64, sequence: i64, gear: i64, life: i64) -> Cast {
        Cast {
            revision,
            sequence,
            before: Anchor {
                equipment_revision: gear,
                life,
                vitals: Vitals {
                    hp: 100,
                    maximum_hp: 100,
                    mp: 300,
                    maximum_mp: i64::MAX,
                },
                x: 32768,
                y: 32770,
            },
            cost: 48,
            mp_after: 252,
        }
    }
    fn health(target: Target, life: i64, gear: i64, hp: i64, cap: i64) -> Health {
        Health {
            target,
            life,
            equipment_revision: gear,
            cell: WorldCell { x: 32770, y: 32771 },
            hp,
            maximum_hp: cap,
        }
    }
    fn hit(target: Target) -> Effect {
        Effect {
            kind: Kind::Damage,
            before: health(
                target,
                if matches!(target, Target::Participant(_) | Target::Npc(_)) {
                    2
                } else {
                    1
                },
                0,
                50,
                100,
            ),
            raw: 100,
            applied: 50,
            hp_after: 0,
        }
    }
    fn batch(lane: Lane, revision: i64, sequence: i64, targets: Vec<Effect>) -> Batch {
        Batch {
            world: WORLD,
            source: SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 4,
            },
            channel_id: 0,
            caster: caster(),
            lane,
            use_receipt: use_receipt(revision, sequence, 0, 1),
            targets,
        }
    }
    #[test]
    fn historical_capacities_remain_exact_after_new_life_and_equipment() {
        let id = Target::Participant(participant(2, 42, 3333, MemberKind::Human));
        let first = History::first(batch(Lane::LightArrow, 3, 1, vec![hit(id)])).unwrap();
        let next = Effect {
            kind: Kind::Damage,
            before: health(id, 3, 5, 3, 3),
            raw: 1,
            applied: 1,
            hp_after: 2,
        };
        let after = first
            .commit(batch(Lane::LightArrow, 9, 2, vec![next]))
            .unwrap();
        assert_eq!(first.last.targets[0].before.maximum_hp, 100);
        assert_eq!(after.last.targets[0].before.maximum_hp, 3);
        assert!(first.follows(&after));
        for (life, gear) in [(2, 5), (3, 4)] {
            let invalid = Effect {
                before: health(id, life, gear, 3, 3),
                ..next
            };
            assert!(after
                .commit(batch(Lane::LightArrow, 10, 3, vec![invalid]))
                .is_none());
        }
    }
    #[test]
    fn wide_damage_full_heal_and_raw_struct_forgery_are_checked() {
        let id = Target::Participant(participant(2, 42, 3333, MemberKind::Human));
        let before = health(id, 1, 0, i64::MAX, i64::MAX);
        let damage = Effect {
            kind: Kind::Damage,
            before,
            raw: i64::MAX,
            applied: i64::MAX,
            hp_after: 0,
        };
        assert!(damage.valid() && damage.death());
        let heal = Effect {
            kind: Kind::Heal,
            before,
            raw: i64::MAX,
            applied: 0,
            hp_after: i64::MAX,
        };
        assert!(heal.valid() && !heal.death());
        assert!(!Effect { applied: 1, ..heal }.valid());
        assert!(!Effect { raw: -1, ..damage }.valid());
        let forged = batch(
            Lane::LightArrow,
            3,
            1,
            vec![Effect {
                before: Health { hp: 0, ..before },
                ..damage
            }],
        );
        assert!(!forged.valid());
        assert!(History::first(forged).is_none());
    }
    #[test]
    fn participant_namespaces_and_duplicate_entity_aliases_stay_distinct() {
        let human = participant(2, 42, 3333, MemberKind::Human);
        let bot = participant(3, 7, 3333, MemberKind::ServerBot);
        assert!(batch(
            Lane::FireStorm,
            3,
            1,
            vec![
                hit(Target::Participant(human)),
                hit(Target::Participant(bot))
            ]
        )
        .valid());
        for other in [
            human,
            participant(2, 7, 2, MemberKind::ServerBot),
            participant(3, 42, 3333, MemberKind::Human),
        ] {
            assert!(!batch(
                Lane::FireStorm,
                3,
                1,
                vec![
                    hit(Target::Participant(human)),
                    hit(Target::Participant(other))
                ]
            )
            .valid());
        }
        assert!(!batch(
            Lane::LightArrow,
            3,
            1,
            vec![hit(Target::Participant(caster()))]
        )
        .valid());
        assert!(!batch(
            Lane::FireStorm,
            3,
            1,
            vec![
                hit(Target::Participant(bot)),
                hit(Target::Honor {
                    entity_id: 3,
                    team: 1
                })
            ]
        )
        .valid());
        let first = History::first(batch(
            Lane::FireStorm,
            3,
            1,
            vec![hit(Target::Participant(bot))],
        ))
        .unwrap();
        for t in [
            Target::Participant(participant(3, 42, 3, MemberKind::Human)),
            Target::Participant(Participant {
                entity_id: 4,
                ..bot
            }),
            Target::Honor {
                entity_id: 3,
                team: 1,
            },
        ] {
            assert!(first
                .commit(batch(Lane::FireStorm, 4, 2, vec![hit(t)]))
                .is_none());
        }
    }
    #[test]
    fn world_selection_channel_and_typed_target_admission_are_not_interchangeable() {
        let first = History::first(batch(Lane::Teleport, 3, 1, vec![])).unwrap();
        for kind in 0..3 {
            let mut next = batch(Lane::Teleport, 4, 2, vec![]);
            match kind {
                0 => next.world.generation += 1,
                1 => next.source.epoch += 1,
                _ => next.channel_id = 1,
            };
            assert!(first.commit(next).is_none());
        }
        let pc = hit(Target::Participant(participant(
            2,
            42,
            3333,
            MemberKind::Human,
        )));
        let n = hit(Target::Npc(npc::Id {
            scope: SCOPE,
            serial: 1,
        }));
        assert!(!batch(Lane::Teleport, 3, 1, vec![pc]).valid());
        assert!(!batch(Lane::LightArrow, 3, 1, vec![]).valid());
        assert!(!batch(Lane::Disintegrate, 3, 1, vec![n]).valid());
        assert!(!batch(Lane::NpcLightArrow, 3, 1, vec![pc]).valid());
        let mut forged = batch(Lane::Teleport, 4, 2, vec![]);
        forged.caster.identity.kind = MemberKind::ServerBot;
        assert!(!forged.valid());
        forged.caster = caster();
        forged.use_receipt.mp_after = 300;
        assert!(!forged.valid());
    }
    #[test]
    fn target_scope_lifetime_and_npc_equipment_forgery_are_rejected() {
        let foreign = npc::Scope {
            match_id: 21,
            ..SCOPE
        };
        assert!(!batch(
            Lane::NpcFireStorm,
            3,
            1,
            vec![hit(Target::Npc(npc::Id {
                scope: foreign,
                serial: 1
            }))]
        )
        .valid());
        assert!(!batch(
            Lane::NpcFireStorm,
            3,
            1,
            vec![hit(Target::Quest {
                id: quest_protocol::Id {
                    scope: SCOPE,
                    slot: 1
                },
                born_revision: 3
            })]
        )
        .valid());
        assert!(!batch(
            Lane::NpcFireStorm,
            3,
            1,
            vec![hit(Target::TimeTower(time_attack_protocol::TowerId {
                scope: foreign
            }))]
        )
        .valid());
        assert!(!health(
            Target::Npc(npc::Id {
                scope: SCOPE,
                serial: 1
            }),
            1,
            1,
            1,
            1
        )
        .valid());
        assert!(!health(
            Target::Honor {
                entity_id: 3,
                team: 0
            },
            2,
            0,
            1,
            1
        )
        .valid());
    }
    #[test]
    fn retained_structure_is_not_erased_by_npc_only_or_empty_cast() {
        let structure = hit(Target::Quest {
            id: quest_protocol::Id {
                scope: SCOPE,
                slot: 1,
            },
            born_revision: 1,
        });
        let first = History::first(batch(Lane::NpcFireStorm, 3, 1, vec![structure])).unwrap();
        let mut next = batch(
            Lane::NpcFireStorm,
            8,
            3,
            vec![hit(Target::Npc(npc::Id {
                scope: SCOPE,
                serial: 1,
            }))],
        );
        next.use_receipt = use_receipt(8, 3, 2, 2);
        let changed = first.commit(next).unwrap();
        assert_eq!(changed.structure.as_ref(), Some(&first.last));
        assert!(first.follows(&changed));
        let mut empty = batch(Lane::NpcFireStorm, 9, 4, vec![]);
        empty.use_receipt = use_receipt(9, 4, 2, 2);
        let later = changed.commit(empty.clone()).unwrap();
        assert_eq!(later.structure, changed.structure);
        let erased = History {
            last: empty,
            structure: None,
        };
        assert!(erased.valid());
        assert!(!changed.follows(&erased));
        let mut invalid = changed.clone();
        invalid.last.use_receipt.mp_after = -1;
        assert!(!invalid.valid());
        assert!(invalid.commit(later.last).is_none());
        assert!(!changed.follows(&invalid));
    }
    #[test]
    fn replay_or_same_revision_rewrite_and_unbounded_targets_fail() {
        let p = Target::Participant(participant(2, 42, 3333, MemberKind::Human));
        let first = History::first(batch(Lane::Disintegrate, 3, 1, vec![hit(p)])).unwrap();
        let mut forged = first.clone();
        forged.last.targets[0].before.hp = 40;
        forged.last.targets[0].applied = 40;
        assert!(forged.valid());
        assert!(!first.follows(&forged));
        assert!(first.commit(first.last.clone()).is_none());
        let targets = (1..=48)
            .map(|serial| {
                hit(Target::Npc(npc::Id {
                    scope: SCOPE,
                    serial,
                }))
            })
            .collect();
        let mut valid = batch(Lane::NpcFireStorm, 3, 1, targets);
        assert!(valid.valid());
        valid.targets.push(hit(Target::Npc(npc::Id {
            scope: SCOPE,
            serial: 49,
        })));
        assert!(!valid.valid());
    }
    #[test]
    fn fixed_java_rust_target_corpus_includes_retained_structure_and_wide_capacity() {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        let mut history: Option<History> = None;
        for i in 0..512_i64 {
            let revision = i + 3;
            let sequence = i + 1;
            let life = 1 + i / 23;
            let gear = i / 7;
            let hp = 1 + i * 17 % 1000;
            let maximum = i64::MAX - i;
            let raw = i * 29 % 1200;
            let mut tag = 1;
            let mut id = i % 3 + 1;
            let mut target_life = 1 + i / 17;
            let mut target = Target::Npc(npc::Id {
                scope: SCOPE,
                serial: id as u64,
            });
            if i % 7 == 0 {
                target_life = 1;
                tag = if i % 2 == 0 { 2 } else { 3 };
                id = if tag == 2 { i % 8 + 1 } else { 0 };
                target = if tag == 2 {
                    Target::Quest {
                        id: quest_protocol::Id {
                            scope: SCOPE,
                            slot: id as u8,
                        },
                        born_revision: 1,
                    }
                } else {
                    Target::TimeTower(time_attack_protocol::TowerId { scope: SCOPE })
                };
            }
            let hit = Effect {
                kind: Kind::Damage,
                before: health(target, target_life, 0, hp, maximum),
                raw,
                applied: raw.min(hp),
                hp_after: hp - raw.min(hp),
            };
            let mut next = batch(Lane::NpcFireStorm, revision, sequence, vec![hit]);
            next.use_receipt = use_receipt(revision, sequence, gear, life);
            history = Some(match history {
                None => History::first(next).unwrap(),
                Some(ref old) => old.commit(next).unwrap(),
            });
            let retained = history.as_ref().unwrap().structure.as_ref().unwrap();
            let prior = retained.targets[0].before;
            for value in [
                10,
                20,
                30,
                revision,
                sequence,
                life,
                gear,
                tag,
                id,
                target_life,
                hp,
                maximum,
                raw,
                hit.applied,
                hit.hp_after,
                retained.use_receipt.revision,
                prior.hp,
                prior.maximum_hp,
            ] {
                hash.update(value.to_be_bytes());
            }
        }
        assert_eq!(
            format!("{:x}", hash.finalize()),
            "55bbc7b08be37db19a23431c87e45395707134d05b3e14f1fc913c4effb4c863"
        );
    }
}
