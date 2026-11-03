//! All twelve LTB1 sections, with no opaque world/mode/personal bytes. Values
//! remain UNTRUSTED after typed parsing. No UI, renderer, command permission or
//! session conversion is provided. Cross-field authority admission is separate.
use super::{
    codec::{record, tags, Reader, Wire, Writer},
    invalid, ApplicationError, Section,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Text<const MAX: usize>(pub String);
impl<const MAX: usize> std::fmt::Debug for Text<MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Text[redacted]")
    }
}
impl<const MAX: usize> Wire for Text<MAX> {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        let n = u16::from_be_bytes(r.take(2)?.try_into().map_err(|_| invalid())?) as usize;
        if n > MAX {
            return Err(invalid());
        }
        let value = std::str::from_utf8(r.take(n)?).map_err(|_| invalid())?;
        if value.chars().any(char::is_control) {
            return Err(invalid());
        }
        Ok(Self(value.to_owned()))
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        if self.0.len() > MAX || self.0.chars().any(char::is_control) {
            return Err(invalid());
        }
        let n = u16::try_from(self.0.len()).map_err(|_| invalid())?;
        w.put(&n.to_be_bytes())?;
        w.put(self.0.as_bytes())
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct List<T, const MAX: usize>(pub Vec<T>);
impl<T, const MAX: usize> std::fmt::Debug for List<T, MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("List[redacted]")
    }
}
impl<T: Wire, const MAX: usize> Wire for List<T, MAX> {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        let n = usize::try_from(r.read::<i32>()?).map_err(|_| invalid())?;
        // Every schema element consumes at least one byte. Check before reserve.
        if n > MAX || n > r.remaining() {
            return Err(invalid());
        }
        let mut values = Vec::new();
        values.try_reserve_exact(n).map_err(|_| invalid())?;
        for _ in 0..n {
            values.push(r.read()?);
        }
        Ok(Self(values))
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        if self.0.len() > MAX {
            return Err(invalid());
        }
        w.write(&i32::try_from(self.0.len()).map_err(|_| invalid())?)?;
        for v in &self.0 {
            w.write(v)?;
        }
        Ok(())
    }
}
tags!(Phase { Loading=0, Countdown=1, Active=2, Aborted=3, Finished=4 });
tags!(Abort { None=0, Disconnected=1, LoadTimeout=2, ActiveTimeout=3, ServerAbort=4, ClockRegressed=5, PreparationCancelled=6 });
tags!(Ruleset { SceneOnly=0, PvpSkirmish=1, DeathmatchElimination=2, HonorTowers=3, PetRace=4, NpcEncounter=5, CooperativeTimeAttack=6, SiegeObjectives=7 });
tags!(MemberKind { Human=0, ServerBot=1 });
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Lane {
    Teleport = 0,
    LightArrow = 1,
    Healing = 2,
    FireStorm = 3,
    Disintegrate = 4,
    NpcLightArrow = 5,
    NpcFireStorm = 6,
    NpcDisintegrate = 7,
    GreaterHaste = 8,
    Haste = 9,
    Slow = 10,
}
impl Wire for Lane {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        match r.read::<u8>()? {
            0 => Ok(Self::Teleport),
            1 => Ok(Self::LightArrow),
            2 => Ok(Self::Healing),
            3 => Ok(Self::FireStorm),
            4 => Ok(Self::Disintegrate),
            5 => Ok(Self::NpcLightArrow),
            6 => Ok(Self::NpcFireStorm),
            7 => Ok(Self::NpcDisintegrate),
            8 if matches!(
                r.schema,
                super::GREATER_HASTE_SCHEMA
                    | super::haste::SCHEMA
                    | super::speed::SCHEMA
                    | super::healing::SCHEMA
                    | super::armor::SCHEMA
            ) =>
            {
                Ok(Self::GreaterHaste)
            }
            9 if matches!(
                r.schema,
                super::haste::SCHEMA
                    | super::speed::SCHEMA
                    | super::healing::SCHEMA
                    | super::armor::SCHEMA
            ) =>
            {
                Ok(Self::Haste)
            }
            10 if super::full_speed(r.schema) => Ok(Self::Slow),
            _ => Err(invalid()),
        }
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        if *self == Self::GreaterHaste
            && !matches!(
                w.schema,
                super::GREATER_HASTE_SCHEMA
                    | super::haste::SCHEMA
                    | super::speed::SCHEMA
                    | super::healing::SCHEMA
                    | super::armor::SCHEMA
            )
            || *self == Self::Haste
                && !matches!(
                    w.schema,
                    super::haste::SCHEMA
                        | super::speed::SCHEMA
                        | super::healing::SCHEMA
                        | super::armor::SCHEMA
                )
            || *self == Self::Slow && !super::full_speed(w.schema)
        {
            return Err(invalid());
        }
        w.write(&(*self as u8))
    }
}
tags!(HasteOrigin { None=0, Potion=1, GreaterHaste=2, Haste=3 });
tags!(EffectKind { Damage=0, Heal=1 });
tags!(QuestKind { RoomSymbol=0, FinalSymbol=1, FinalGuardian=2 });
tags!(QuestFlow { FinalGuardian=0, RoomSymbols=1 });
tags!(QuestStatus { Running=0, Finished=1, Aborted=2 });
tags!(QuestPhase { SpawnPending=0, Fighting=1, Objective=2 });
tags!(TimePhase { WaitingForTower=0, SpawnPending=1, Fighting=2, Finished=3, Aborted=4 });
tags!(TimeEnd { None=0, Cleared=1, Timeout=2, LeaderDefeated=3, PartyDefeated=4 });
tags!(TimeKind { Ordinary=0, FinalBosses=1, Dragon=2 });
tags!(SiegeSlot { OuterLeft=0, OuterRight=1, Inner=2 });
tags!(SiegePhase { Waiting=0, Outer=1, Inner=2, Finished=3, Aborted=4 });
tags!(SiegeEnd { InnerDestroyed=0, Timeout=1 });
tags!(RacePhase { Waiting=0, Running=1, FinishWindow=2, Complete=3, Aborted=4 });
tags!(Dnf { None=0, Left=1, Disconnected=2, WindowClosed=3 });
tags!(NpcAction { Idle=0, Walk=1, Hurt=2, Death=3, Attack=4 });
tags!(Direction { PlayerToNpc=0, NpcToParticipant=1 });
tags!(Verdict { Win=0, Loss=1 });
tags!(ItemKind { Weapon=0, Armor=1, Etc=2 });
tags!(Support { Teleport=0, Unimplemented=1, UnsupportedTemplate=2, LightArrow=3, BasicHeal=4, FireStorm=5, Disintegrate=6 });
tags!(LifetimeState { Unqualified=0, Permanent=1, Active=2, Expired=3, UnstartedOrMissing=4, RelativeUnqualified=5, LightingUnqualified=6 });
tags!(Activity { Attack=0, Move=1, None=2 });

record!(WorldId {
    room: i64,
    match_id: i64,
    generation: i64
});
record!(Player {
    source: i32,
    object: i64,
    epoch: i64,
    channel: i32
});
record!(Identity {
    kind: MemberKind,
    source: i32,
    id: i64
});
record!(Cell { x: i32, y: i32 });
record!(Vitals {
    hp: i64,
    maximum_hp: i64,
    mp: i64,
    maximum_mp: i64
});
record!(Descriptor {
    ruleset: Ruleset,
    features: i32,
    preparation_timeout_millis: i32,
    load_timeout_millis: i32,
    countdown_nanos: i64,
    active_timeout_millis: i32
});
record!(World {
    identity: WorldId,
    revision: i64,
    phase: Phase,
    abort: Abort,
    lane: i32,
    descriptor: Descriptor,
    loading_elapsed_nanos: i64,
    active_elapsed_nanos: i64
});
record!(Member { identity:Identity, name:Text<256>, source_name:Text<256>, level:i64, class:i32, sex:i32, team:i32, ready:bool });
record!(Room { id:i64, revision:i64, name:Text<64>, capacity:i32, mode:i32, map:i32, attributes:i32, users:i32, ready:i32, satisfied:bool, owner_source:i32, owner_object:i64, members:List<Member,100> });
record!(Sector {
    x: i32,
    y: i32,
    byte_length: i32,
    sha256: [u8; 32]
});
record!(Manifest { map:i32, digest:[u8;32], sectors:List<Sector,64> });
record!(Vital {
    values: Vitals,
    ac: i64,
    weapon_type: i32,
    attack_sequence: i64,
    attack_revision: i64,
    hurt_revision: i64,
    death_revision: i64,
    target: i64,
    attack_interval_nanos: i64
});
record!(Flows {
    hp_gain: i64,
    hp_loss: i64,
    restored: i64,
    damaged: i64,
    hp_granted: i64,
    mp_gain: i64,
    mp_loss: i64,
    spent: i64,
    mp_granted: i64,
    hp_clipped: i64,
    mp_clipped: i64
});
record!(Resources {
    revision: i64,
    equipment_revision: i64,
    life: i64,
    birth_hp: i64,
    birth_mp: i64,
    vitals: Vitals,
    flows: Flows
});
record!(Ledger {
    equipment_revision: i64,
    profile_revision: i64,
    cost: i32,
    prior_casts: i64,
    prior_spent: i64,
    casts: i64,
    spent: i64,
    last_cast_revision: i64,
    last_cast_sequence: i64
});
record!(Anchor {
    equipment_revision: i64,
    life: i64,
    vitals: Vitals,
    cell: Cell
});
record!(Cast {
    revision: i64,
    sequence: i64,
    before: Anchor,
    cost: i32,
    mp_after: i64
});
record!(QuestId {
    scope: WorldId,
    slot: i32
});

#[derive(Clone, PartialEq, Eq)]
pub enum Target {
    Participant { entity: i64, identity: Identity },
    Npc { scope: WorldId, serial: i64 },
    Honor { entity: i64, team: i32 },
    Quest { id: QuestId, born_revision: i64 },
    TimeTower(WorldId),
}
impl std::fmt::Debug for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Target[redacted]")
    }
}
impl Wire for Target {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        Ok(match r.read::<u8>()? {
            0 => Self::Participant {
                entity: r.read()?,
                identity: r.read()?,
            },
            1 => Self::Npc {
                scope: r.read()?,
                serial: r.read()?,
            },
            2 => Self::Honor {
                entity: r.read()?,
                team: r.read()?,
            },
            3 => Self::Quest {
                id: r.read()?,
                born_revision: r.read()?,
            },
            4 => Self::TimeTower(r.read()?),
            _ => return Err(invalid()),
        })
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        match self {
            Self::Participant { entity, identity } => {
                w.write(&0u8)?;
                w.write(entity)?;
                w.write(identity)
            }
            Self::Npc { scope, serial } => {
                w.write(&1u8)?;
                w.write(scope)?;
                w.write(serial)
            }
            Self::Honor { entity, team } => {
                w.write(&2u8)?;
                w.write(entity)?;
                w.write(team)
            }
            Self::Quest { id, born_revision } => {
                w.write(&3u8)?;
                w.write(id)?;
                w.write(born_revision)
            }
            Self::TimeTower(scope) => {
                w.write(&4u8)?;
                w.write(scope)
            }
        }
    }
}
record!(Effect {
    target: Target,
    kind: EffectKind,
    life: i64,
    equipment_revision: i64,
    cell: Cell,
    hp: i64,
    maximum_hp: i64,
    raw: i64,
    applied: i64,
    hp_after: i64
});
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Impact {
    pub lane: Lane,
    pub usage: Cast,
    pub targets: List<Effect, 101>,
}
impl Impact {
    fn bounded(&self) -> bool {
        self.targets.0.len()
            <= match self.lane {
                Lane::Teleport | Lane::GreaterHaste | Lane::Haste | Lane::Slow => 0,
                Lane::FireStorm => 101,
                Lane::NpcFireStorm => 48,
                _ => 1,
            }
    }
}
impl Wire for Impact {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        let v = Self {
            lane: r.read()?,
            usage: r.read()?,
            targets: r.read()?,
        };
        if !v.bounded() {
            return Err(invalid());
        }
        Ok(v)
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        if !self.bounded() {
            return Err(invalid());
        }
        w.write(&self.lane)?;
        w.write(&self.usage)?;
        w.write(&self.targets)
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Detail {
    Slow(super::speed::SlowDetail),
    GreaterHasteNeutralized(super::speed::Neutralized),
    Haste(super::haste::Detail),
    GreaterHaste {
        active_elapsed_nanos: i64,
        haste_until_nanos: i64,
    },
    Ordinary,
    Arrow {
        critical: bool,
    },
    FireStorm {
        damage_loss_after: List<i64, 101>,
    },
    NpcFireStorm {
        quest_kind: Option<QuestKind>,
    },
    Teleport {
        destination: Cell,
    },
    Disintegrate {
        critical: bool,
        lawful_before: i64,
        lawful_after: i64,
        damage_loss_after: i64,
    },
}
impl Wire for Detail {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        Ok(match r.read::<u8>()? {
            0 => Self::Ordinary,
            1 => Self::Arrow {
                critical: r.read()?,
            },
            2 => Self::FireStorm {
                damage_loss_after: r.read()?,
            },
            3 => Self::NpcFireStorm {
                quest_kind: r.read()?,
            },
            4 => Self::Teleport {
                destination: r.read()?,
            },
            5 => Self::Disintegrate {
                critical: r.read()?,
                lawful_before: r.read()?,
                lawful_after: r.read()?,
                damage_loss_after: r.read()?,
            },
            6 if matches!(
                r.schema,
                super::GREATER_HASTE_SCHEMA
                    | super::haste::SCHEMA
                    | super::speed::SCHEMA
                    | super::healing::SCHEMA
                    | super::armor::SCHEMA
            ) =>
            {
                Self::GreaterHaste {
                    active_elapsed_nanos: r.read()?,
                    haste_until_nanos: r.read()?,
                }
            }
            7 if matches!(
                r.schema,
                super::haste::SCHEMA
                    | super::speed::SCHEMA
                    | super::healing::SCHEMA
                    | super::armor::SCHEMA
            ) =>
            {
                Self::Haste(r.read()?)
            }
            8 if super::full_speed(r.schema) => Self::Slow(r.read()?),
            9 if super::full_speed(r.schema) => Self::GreaterHasteNeutralized(r.read()?),
            _ => return Err(invalid()),
        })
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        match self {
            Self::Slow(value) => {
                if !super::full_speed(w.schema) {
                    return Err(invalid());
                }
                w.write(&8u8)?;
                w.write(value)
            }
            Self::GreaterHasteNeutralized(value) => {
                if !super::full_speed(w.schema) {
                    return Err(invalid());
                }
                w.write(&9u8)?;
                w.write(value)
            }
            Self::Haste(value) => {
                if !matches!(
                    w.schema,
                    super::haste::SCHEMA
                        | super::speed::SCHEMA
                        | super::healing::SCHEMA
                        | super::armor::SCHEMA
                ) {
                    return Err(invalid());
                }
                w.write(&7u8)?;
                w.write(value)
            }
            Self::GreaterHaste {
                active_elapsed_nanos,
                haste_until_nanos,
            } => {
                if !matches!(
                    w.schema,
                    super::GREATER_HASTE_SCHEMA
                        | super::haste::SCHEMA
                        | super::speed::SCHEMA
                        | super::healing::SCHEMA
                        | super::armor::SCHEMA
                ) {
                    return Err(invalid());
                }
                w.write(&6u8)?;
                w.write(active_elapsed_nanos)?;
                w.write(haste_until_nanos)
            }
            Self::Ordinary => w.write(&0u8),
            Self::Arrow { critical } => {
                w.write(&1u8)?;
                w.write(critical)
            }
            Self::FireStorm { damage_loss_after } => {
                w.write(&2u8)?;
                w.write(damage_loss_after)
            }
            Self::NpcFireStorm { quest_kind } => {
                w.write(&3u8)?;
                w.write(quest_kind)
            }
            Self::Teleport { destination } => {
                w.write(&4u8)?;
                w.write(destination)
            }
            Self::Disintegrate {
                critical,
                lawful_before,
                lawful_after,
                damage_loss_after,
            } => {
                w.write(&5u8)?;
                w.write(critical)?;
                w.write(lawful_before)?;
                w.write(lawful_after)?;
                w.write(damage_loss_after)
            }
        }
    }
}
record!(SpellReceipt {
    impact: Impact,
    casts: i64,
    spent: i64,
    loading_elapsed_nanos: i64,
    detail: Detail
});
record!(Spell { lane:Lane, ledger:Ledger, last:Option<Cast>, receipt:Option<SpellReceipt> });
record!(Structure { casts:i64, impact:Impact, quest_kind:Option<QuestKind> });
record!(Spells { sequence:i64, request_revision:i64, initial_lawful:i64, lawful:i64, histories:List<Spell,11>, structure:Option<Structure> });
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffs {
    pub entity: i64,
    pub revision: i64,
    pub observed_nanos: i64,
    pub haste_until_nanos: i64,
    pub brave_until_nanos: i64,
    pub wisdom_until_nanos: i64,
    pub brave_kind: i32,
    pub haste_origin: HasteOrigin,
    pub slow_until_nanos: i64,
    pub slow_revision: i64,
}
impl Wire for Buffs {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        let mut b = Self {
            entity: r.read()?,
            revision: r.read()?,
            observed_nanos: r.read()?,
            haste_until_nanos: r.read()?,
            brave_until_nanos: r.read()?,
            wisdom_until_nanos: r.read()?,
            brave_kind: r.read()?,
            haste_origin: HasteOrigin::None,
            slow_until_nanos: 0,
            slow_revision: 0,
        };
        b.haste_origin = if matches!(
            r.schema,
            super::GREATER_HASTE_SCHEMA
                | super::haste::SCHEMA
                | super::speed::SCHEMA
                | super::healing::SCHEMA
                | super::armor::SCHEMA
        ) {
            r.read()?
        } else if b.haste_until_nanos == 0 {
            HasteOrigin::None
        } else {
            HasteOrigin::Potion
        };
        if b.haste_origin == HasteOrigin::Haste
            && !matches!(
                r.schema,
                super::haste::SCHEMA
                    | super::speed::SCHEMA
                    | super::healing::SCHEMA
                    | super::armor::SCHEMA
            )
        {
            return Err(invalid());
        }
        if super::full_speed(r.schema) {
            b.slow_until_nanos = r.read()?;
            b.slow_revision = r.read()?;
        }
        Ok(b)
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        if !super::full_speed(w.schema) && (self.slow_until_nanos != 0 || self.slow_revision != 0) {
            return Err(invalid());
        }
        if self.haste_origin == HasteOrigin::Haste
            && !matches!(
                w.schema,
                super::haste::SCHEMA
                    | super::speed::SCHEMA
                    | super::healing::SCHEMA
                    | super::armor::SCHEMA
            )
        {
            return Err(invalid());
        }
        if !matches!(
            w.schema,
            super::GREATER_HASTE_SCHEMA
                | super::haste::SCHEMA
                | super::speed::SCHEMA
                | super::healing::SCHEMA
                | super::armor::SCHEMA
        ) && self.haste_origin
            != if self.haste_until_nanos == 0 {
                HasteOrigin::None
            } else {
                HasteOrigin::Potion
            }
        {
            return Err(invalid());
        }
        w.write(&self.entity)?;
        w.write(&self.revision)?;
        w.write(&self.observed_nanos)?;
        w.write(&self.haste_until_nanos)?;
        w.write(&self.brave_until_nanos)?;
        w.write(&self.wisdom_until_nanos)?;
        w.write(&self.brave_kind)?;
        if matches!(
            w.schema,
            super::GREATER_HASTE_SCHEMA
                | super::haste::SCHEMA
                | super::speed::SCHEMA
                | super::healing::SCHEMA
                | super::armor::SCHEMA
        ) {
            w.write(&self.haste_origin)?;
        }
        if super::full_speed(w.schema) {
            w.write(&self.slow_until_nanos)?;
            w.write(&self.slow_revision)?;
        }
        Ok(())
    }
}
record!(RegenAxis {
    gain: i64,
    loss: i64,
    revision: i64,
    before: i64,
    after: i64
});
record!(Regeneration {
    hp: RegenAxis,
    mp: RegenAxis,
    item_gain: i64,
    damage_loss: i64
});
record!(Actor [super::ACTOR_LIMIT] { entity:i64, position:Cell, heading:i32, loaded:bool, last_move_sequence:i64, step_interval_nanos:i64, vital:Option<Vital>, resources:Option<Resources>, spells:Spells, cast_interval_nanos:i64, buffs:Option<Buffs>, regeneration:Option<Regeneration> });

record!(HonorObjective {
    entity: i64,
    team: i32,
    action: i32,
    graphic: i32,
    cell: Cell,
    hp: i64,
    maximum_hp: i64,
    revision: i64,
    hurt_revision: i64,
    death_revision: i64,
    damage_loss: i64,
    heal_gain: i64,
    last_actor: i64,
    last_kind: i32
});
record!(HonorLife {
    entity: i64,
    death_count: i64,
    last_death_revision: i64,
    respawn_count: i64,
    respawn_revision: i64,
    respawn_hp_gain: i64,
    pending_due_millis: i64,
    death_cell: Cell
});
record!(Honor { objectives:List<HonorObjective,2>, lives:List<HonorLife,100> });
record!(QuestLane { lane:i32, stage:i32, phase:QuestPhase, living:List<i64,128> });
record!(QuestTarget {
    lane: i32,
    stage: i32,
    cell: Cell,
    graphic: i32,
    maximum_hp: i64
});
record!(QuestObjective {
    id: QuestId,
    target: QuestTarget,
    kind: QuestKind,
    hp: i64,
    born_revision: i64,
    revision: i64,
    hurt_revision: i64,
    death_revision: i64
});
record!(Outcome {
    hit: bool,
    raw_damage: i64,
    applied_damage: i64,
    remaining_hp: i64,
    death: bool,
    maximum_triggered: bool,
    critical_triggered: bool
});
record!(QuestExchange {
    participant: Identity,
    target: QuestId,
    request_sequence: i64,
    revision: i64,
    outcome: Outcome
});
record!(QuestLife {
    entity: i64,
    lane: i32,
    spawn: Cell,
    deaths: i64,
    death_revision: i64,
    respawns: i64,
    respawn_revision: i64,
    hp_gain: i64,
    death_cell: Cell
});
record!(Quest { revision:i64, flow:QuestFlow, status:QuestStatus, lanes:List<QuestLane,2>, targets:List<QuestTarget,8>, objectives:List<QuestObjective,2>, exchanges:List<QuestExchange,100>, lives:List<QuestLife,100>, winning_lane:i32, death_revision:i64 });
record!(TimeTower {
    cell: Cell,
    graphic: i32,
    maximum_hp: i64
});
record!(TimeExchange {
    entity: i64,
    target: WorldId,
    sequence: i64,
    revision: i64,
    outcome: Outcome
});
record!(TimeAttack { revision:i64, phase:TimePhase, end:TimeEnd, waves:List<TimeKind,32>, wave:i32, leader:i64, duration_millis:i32, started_millis:i64, observed_millis:i64, departed:List<i64,60>, living:List<i64,128>, tower:TimeTower, tower_hp:i64, tower_hurt_revision:i64, tower_death_revision:i64, plan_sha256:[u8;32], exchanges:List<TimeExchange,60> });
record!(SiegeTower {
    slot: SiegeSlot,
    cell: Cell,
    graphic: i32,
    maximum_hp: i64
});
record!(SiegeObjective {
    tower: SiegeTower,
    hp: i64,
    damage_loss: i64,
    hit_count: i64,
    revision: i64,
    death_revision: i64,
    last_actor: i64
});
record!(SiegeLife {
    entity: i64,
    deaths: i64,
    death_revision: i64,
    respawns: i64,
    respawn_revision: i64,
    hp_gain: i64,
    due_millis: i64
});
record!(Siege { scope:WorldId, mode:i32, logical_map:i32, towers:List<SiegeTower,3>, gate:List<Cell,64>, attack_camp:List<Cell,50>, defense_camp:List<Cell,50>, portal_outside:Cell, portal_inside:Cell, duration_millis:i64, respawn_millis:i64, attacker:i32, phase:SiegePhase, objectives:List<SiegeObjective,3>, revision:i64, gate_revision:i64, finish_revision:i64, finished_at_millis:i64, winner:i32, lives:List<SiegeLife,100> });
record!(Gate {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32
});
record!(Runner {
    entity: i64,
    revision: i64,
    progress_move_sequence: i64,
    completed_laps: i64,
    next_checkpoint: i64,
    placement: i64,
    finish_elapsed_nanos: i64,
    dnf: Dnf,
    base_graphics: i64
});
record!(Race { revision:i64, phase:RacePhase, course_sha256:[u8;32], gates:List<Gate,64>, active_limit_nanos:i64, observed_elapsed_nanos:i64, first_finisher_entity:i64, finish_window_deadline_elapsed_nanos:i64, terminal_revision:i64, result_id:Option<[u8;16]>, runners:List<Runner,10> });
record!(Npc {
    serial: i64,
    source: i32,
    npc_id: i32,
    graphic: i32,
    lane: i32,
    stage: i32,
    heading: i32,
    action: NpcAction,
    cell: Cell,
    hp: i64,
    maximum_hp: i64,
    mp: i64,
    maximum_mp: i64,
    life: i64,
    born_revision: i64,
    changed_revision: i64,
    corpse_until_millis: i64,
    next_move_millis: i64,
    next_attack_millis: i64
});
record!(MeleeOutcome {
    hit: bool,
    raw_damage: i64,
    damage: i64,
    applied_damage: i64,
    remaining_hp: i64,
    death: bool,
    maximum_triggered: bool,
    critical_triggered: bool,
    material_bonus: i64,
    blessing_bonus: i64
});
record!(NpcExchange {
    direction: Direction,
    participant: Identity,
    participant_life: i64,
    scope: WorldId,
    serial: i64,
    npc_life: i64,
    request_sequence: i64,
    revision: i64,
    outcome: MeleeOutcome
});
record!(NpcName { source:i32, npc_id:i32, graphic:i32, name:Text<256>, name_id:Text<256> });
record!(Npcs { revision:i64, elapsed_millis:i64, spawned:i64, closed:bool, actors:List<Npc,128>, exchanges:List<NpcExchange,228>, graphics:List<i32,10>, names:List<NpcName,128> });

record!(SavedStats {
    maximum_hp: i64,
    maximum_mp: i64,
    saved_hp: i64,
    saved_mp: i64,
    ac: i64,
    strength: i64,
    dexterity: i64,
    constitution: i64,
    intelligence: i64,
    wisdom: i64,
    charisma: i64,
    experience: i64,
    lawful: i64,
    original_strength: i64,
    original_dexterity: i64,
    original_constitution: i64,
    original_intelligence: i64,
    original_wisdom: i64,
    food225: i64
});
record!(InventoryItem { object:i64, catalog:i64, kind:ItemKind, name:Text<256>, inventory_graphic:i64, ground_graphic:i64, unit_weight:i64, quantity:i64, equipped:bool, enchantment:i64, durability:i64, identified:bool, blessing:i64, sealed:bool, charge_count:i64 });
record!(Effective {
    maximum_hp: i64,
    maximum_mp: i64,
    initial_hp: i64,
    initial_mp: i64,
    ac: i64,
    strength: i64,
    dexterity: i64,
    constitution: i64,
    intelligence: i64,
    wisdom: i64,
    charisma: i64
});
record!(Equipped {
    object: i64,
    equipped: bool
});
record!(Lifetime {
    object: i64,
    state: LifetimeState,
    expires_at_millis: i64
});
record!(Equipment { source:i32, object:i64, name:Text<256>, class:i32, sex:i32, level:i64, saved:SavedStats, inventory:List<InventoryItem,512>, revision:i64, effective:Effective, equipped:List<Equipped,512>, lifetimes:List<Lifetime,512>, evaluated_at_epoch_millis:i64, valid_until_epoch_millis:i64 });
record!(Skill { id:i32, name:Text<256>, mp_cost:i64, reuse_millis:i64, support:Support });
/// Existing LHI1 subsection is decoded into its typed, locally validated State.
/// It still needs exact outer owner/world/resource binding before admission.
#[derive(Clone, PartialEq, Eq)]
pub struct EquipmentHistory(pub crate::modern_lito::equipment_history::State);
impl std::fmt::Debug for EquipmentHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EquipmentHistory[redacted]")
    }
}
impl Wire for EquipmentHistory {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        let armor_healing = r.schema == super::armor::SCHEMA && r.read::<bool>()?;
        let n = usize::try_from(r.read::<i32>()?).map_err(|_| invalid())?;
        if n > crate::modern_lito::equipment_history_wire::MAX_BYTES {
            return Err(invalid());
        }
        let schema = if r.schema == super::healing::SCHEMA || armor_healing {
            5
        } else if super::full_speed(r.schema) {
            4
        } else if r.schema == super::haste::SCHEMA {
            3
        } else if r.schema == super::GREATER_HASTE_SCHEMA {
            2
        } else {
            1
        };
        Ok(Self(
            crate::modern_lito::equipment_history_wire::decode_with_schema(r.take(n)?, schema)?,
        ))
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        if w.schema == super::armor::SCHEMA {
            w.write(&self.0.healing.is_some())?;
        }
        let bytes = crate::modern_lito::equipment_history_wire::encode_with_schema(
            &self.0,
            if w.schema == super::healing::SCHEMA
                || w.schema == super::armor::SCHEMA && self.0.healing.is_some()
            {
                5
            } else if super::full_speed(w.schema) {
                4
            } else if w.schema == super::haste::SCHEMA {
                3
            } else if w.schema == super::GREATER_HASTE_SCHEMA {
                2
            } else {
                1
            },
        )?;
        w.write(&i32::try_from(bytes.len()).map_err(|_| invalid())?)?;
        w.put(&bytes)
    }
}
record!(ItemUse {
    revision: i64,
    sequence: i64,
    item: i64,
    before: Anchor,
    fixed_recovery: i64,
    rolled_base: i64,
    restored: i64,
    hp_after: i64
});
record!(Items { equipment_revision:i64, profile_revision:i64, fixed_recovery:i64, uses:i64, restored:i64, last:Option<ItemUse> });
record!(Stock {
    item: i64,
    initial: i64,
    remaining: i64
});
record!(Supplies { request_sequence:i64, request_revision:i64, history:Items, stocks:List<Stock,7>, effect:i32, delay_millis:i32 });
record!(UnitWeight {
    item: i64,
    unit_weight: i64
});
record!(Weight { source_gear_weight:i64, supplies_weight:i64, total_weight:i64, maximum_weight_bits:i64, stage:i32, units:List<UnitWeight,256> });
record!(AmmoStock {
    item: i64,
    small: i64,
    large: i64,
    raw_arrow_sprite: i32,
    consume: bool,
    material: i32,
    blessing: i32,
    initial: i64,
    remaining: i64
});
record!(Ammo { entity:i64, request_sequence:i64, revision:i64, selection_revision:i64, selected_item:i64, shot_count:i64, shot_revision:i64, shot_item:i64, attack_sequence:i64, last_target_entity:i64, from:Cell, to:Cell, projectile_sprite:i32, hit:bool, stocks:List<AmmoStock,2> });
record!(RegenClock {
    next_second_elapsed_nanos: i64,
    hp_counter: i32,
    mp_counter: i32,
    hp_activity: Activity,
    mp_activity: Activity
});
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Personal {
    pub player: Player,
    pub entity: i64,
    pub equipment: Option<Equipment>,
    pub skills: List<Skill, 220>,
    pub history: Option<EquipmentHistory>,
    pub spell_birth_revision: i64,
    pub supplies: Option<Supplies>,
    pub weight: Option<Weight>,
    pub ammo: Option<Ammo>,
    pub regen_clock: Option<RegenClock>,
    pub materials: Option<super::speed::Materials>,
}
impl Wire for Personal {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        Ok(Self {
            player: r.read()?,
            entity: r.read()?,
            equipment: r.read()?,
            skills: r.read()?,
            history: r.read()?,
            spell_birth_revision: r.read()?,
            supplies: r.read()?,
            weight: r.read()?,
            ammo: r.read()?,
            regen_clock: r.read()?,
            materials: if matches!(r.schema, super::healing::SCHEMA | super::armor::SCHEMA) {
                r.read()?
            } else if super::full_speed(r.schema) {
                Some(r.read()?)
            } else {
                None
            },
        })
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        if !matches!(w.schema, super::healing::SCHEMA | super::armor::SCHEMA)
            && super::full_speed(w.schema) != self.materials.is_some()
        {
            return Err(invalid());
        }
        w.write(&self.player)?;
        w.write(&self.entity)?;
        w.write(&self.equipment)?;
        w.write(&self.skills)?;
        w.write(&self.history)?;
        w.write(&self.spell_birth_revision)?;
        w.write(&self.supplies)?;
        w.write(&self.weight)?;
        w.write(&self.ammo)?;
        w.write(&self.regen_clock)?;
        if matches!(w.schema, super::healing::SCHEMA | super::armor::SCHEMA) {
            w.write(&self.materials)?;
        } else if let Some(m) = &self.materials {
            w.write(m)?;
        }
        Ok(())
    }
}
// Schema 2 public facts, deliberately no stocks/item IDs/private source epoch.
record!(PublicShot {
    count: i64,
    revision: i64,
    attack_sequence: i64,
    loading_elapsed_nanos: i64,
    life: i64,
    equipment_revision: i64,
    from: Cell,
    target: Target,
    target_life: i64,
    to: Cell,
    sprite: i32,
    hit: bool
});
record!(PublicItem {
    count: i64,
    revision: i64,
    loading_elapsed_nanos: i64,
    life: i64,
    equipment_revision: i64,
    cell: Cell,
    effect: i32
});
record!(PublicPresentation { entity:i64, shot:Option<PublicShot>, item:Option<PublicItem> });
record!(ResultParticipant {
    entity: i64,
    identity: Identity,
    team: i32,
    verdict: Verdict
});
record!(TimeProof {
    room: i64,
    physical_map: i32,
    terrain_sha256: [u8; 32],
    plan_sha256: [u8; 32],
    end: TimeEnd,
    wave: i32,
    wave_count: i32,
    leader: i32,
    duration_millis: i32,
    started_millis: i64,
    observed_millis: i64,
    alive_mask: i64,
    departed_mask: i64,
    tower_death_revision: i64
});
record!(SiegeProof { room:i64, physical_map:i32, terrain_sha256:[u8;32], config_sha256:[u8;32], attacker:i32, winner:i32, end:SiegeEnd, duration_millis:i32, respawn_millis:i32, elapsed_millis:i64, gate_revision:i64, towers:List<SiegeObjective,3> });
record!(MatchResult { result_id:[u8;16], match_id:i64, generation:i64, engine_revision:i64, mode:i32, map:i32, finished_at_millis:i64, participants:List<ResultParticipant,100>, time_attack:Option<TimeProof>, siege:Option<SiegeProof> });

/// Typed, lossless syntax only. This is not a validated live game snapshot.
#[derive(Clone, PartialEq, Eq)]
pub struct UnvalidatedData {
    pub world: World,
    pub room: Room,
    pub manifest: Manifest,
    pub actors: List<Actor, 100>,
    pub honor: Honor,
    pub quest: Option<Quest>,
    pub time_attack: Option<TimeAttack>,
    pub siege: Option<Siege>,
    pub race: Option<Race>,
    pub npcs: Option<Npcs>,
    pub personal: Personal,
    pub result: Option<MatchResult>,
    /// None is legacy schema 1; Some is the complete visible schema 2 roster.
    pub presentation: Option<List<PublicPresentation, 100>>,
    /// v43/schema4 presence is mandatory even when this world has no NPCs.
    pub npc_motion: Option<super::npc_motion::Envelope>,
    /// v44/schema5 complete status and motion; exclusive with schema4.
    pub haste: Option<super::haste::Envelope>,
    /// v46/schema6 full speed/material state, exclusive with schema4/5.
    pub speed: Option<super::speed::Envelope>,
    /// Explicitly selected schema7: complete status AND healing, never a projection.
    pub healing: Option<super::healing::Envelope>,
    /// Explicit armor selection: full status, optional healing and armor together.
    pub armor: Option<super::armor::Envelope>,
    /// Full VIP selection retains all optional sections and three owner versions.
    pub vip: Option<super::vip::Envelope>,
}
impl std::fmt::Debug for UnvalidatedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UnvalidatedBattleData[redacted]")
    }
}
fn section<T: Wire>(p: &super::UnvalidatedPayload<'_>, id: Section) -> Result<T, ApplicationError> {
    let mut r = Reader::new(p.section(id));
    r.schema = p.layout_schema();
    let value = r.read()?;
    r.finish()?;
    Ok(value)
}
pub fn decode(bytes: &[u8]) -> Result<UnvalidatedData, ApplicationError> {
    let p = super::inspect(bytes)?;
    decode_payload(&p)
}
pub(super) fn decode_payload(
    p: &super::UnvalidatedPayload<'_>,
) -> Result<UnvalidatedData, ApplicationError> {
    let data = decode_sections(p)?;
    if p.layout_schema() != super::armor::SCHEMA
        && p.schema() != super::healing::SCHEMA
        && p.schema() != super::haste::SCHEMA
        && p.schema() != super::speed::SCHEMA
        && (p.schema() == super::GREATER_HASTE_SCHEMA) != data.greater_haste()
    {
        return Err(invalid());
    }
    data.schema_lanes(p.layout_schema())?;
    Ok(data)
}
// Field codecs only. Callers must retain and check their complete envelope.
pub(super) fn decode_sections(
    p: &super::UnvalidatedPayload<'_>,
) -> Result<UnvalidatedData, ApplicationError> {
    let data = UnvalidatedData {
        vip: None,
        armor: None,
        healing: None,
        haste: None,
        speed: None,
        npc_motion: None,
        world: section(&p, Section::World)?,
        room: section(&p, Section::Room)?,
        manifest: section(&p, Section::Manifest)?,
        actors: section(&p, Section::Actors)?,
        honor: section(&p, Section::Honor)?,
        quest: section(&p, Section::Quest)?,
        time_attack: section(&p, Section::TimeAttack)?,
        siege: section(&p, Section::Siege)?,
        race: section(&p, Section::Race)?,
        npcs: section(&p, Section::Npcs)?,
        personal: section(&p, Section::Personal)?,
        result: section(&p, Section::Result)?,
        presentation: if p.schema() >= super::PRESENTATION_SCHEMA {
            Some(section(&p, Section::Presentation)?)
        } else {
            None
        },
    };
    Ok(data)
}
impl UnvalidatedData {
    /// Borrow the complete selected envelope; never copy schema8 into schema7.
    /// Callers still require an admitted Observation before presentation/input.
    pub(crate) fn healing_actors(&self) -> Option<&List<super::healing::Actor, 100>> {
        if let Some(v) = &self.vip {
            return v.healing.as_ref();
        }
        match &self.armor {
            Some(a) => a.healing.as_ref(),
            None => self.healing.as_ref().map(|h| &h.actors),
        }
    }
    /// Borrow public armor state from the selected complete envelope. VIP keeps
    /// its owner ledger and independent versions; no legacy DTO is fabricated.
    pub(crate) fn armor_actors(&self) -> Option<&List<super::armor::Actor, 100>> {
        if let Some(v) = &self.vip {
            return (self.world.descriptor.features & super::armor::FEATURE != 0)
                .then_some(&v.armor);
        }
        self.armor.as_ref().map(|a| &a.actors)
    }
    pub(crate) fn armor_presentation(&self) -> Option<&super::armor_presentation::Presentation> {
        if let Some(v) = &self.vip {
            return v.presentation.as_ref();
        }
        self.armor.as_ref()?.presentation.as_ref()
    }
    /// Status inside schema7 is the same complete state, not a copied legacy envelope.
    pub(crate) fn speed_state(&self) -> Option<&super::speed::State> {
        match self.combined_status() {
            Some(super::healing::Status::Speed(s)) => Some(s),
            Some(_) => None,
            None => self.speed.as_ref().map(|e| &e.state),
        }
    }
    pub(crate) fn haste_state(&self) -> Option<&super::haste::State> {
        match self.combined_status() {
            Some(super::healing::Status::Haste(s)) => Some(s),
            Some(_) => None,
            None => self.haste.as_ref().map(|e| &e.state),
        }
    }
    pub(crate) fn motion_state(&self) -> Option<&Option<super::npc_motion::State>> {
        match self.combined_status() {
            Some(super::healing::Status::Motion(s)) => Some(s),
            Some(_) => None,
            None => self.npc_motion.as_ref().map(|e| &e.state),
        }
    }
    pub(crate) fn greater_haste(&self) -> bool {
        self.world.descriptor.features & (1 << 21) != 0
    }
    fn combined_status(&self) -> Option<&super::healing::Status> {
        if let Some(v) = &self.vip {
            return Some(&v.status);
        }
        self.armor
            .as_ref()
            .map(|a| &a.status)
            .or_else(|| self.healing.as_ref().map(|h| &h.status))
    }
    fn schema_lanes(&self, schema: u16) -> Result<(), ApplicationError> {
        if self.vip.is_some() || self.world.descriptor.features & super::vip::FEATURE != 0 {
            return Err(invalid()); // Never drop the complete VIP envelope.
        }
        let armor = self.world.descriptor.features & super::armor::FEATURE != 0;
        if (schema == super::armor::SCHEMA) != armor {
            return Err(invalid());
        }
        if armor {
            return super::armor::check_base_shape(self);
        }
        let healing = self.world.descriptor.features & super::healing::FEATURE != 0;
        if (schema == super::healing::SCHEMA) != healing {
            return Err(invalid());
        }
        if healing {
            return super::healing::check_base_shape(self);
        }
        if (schema == super::speed::SCHEMA)
            != (self.world.descriptor.features & super::speed::FEATURE != 0)
        {
            return Err(invalid());
        }
        if schema == super::speed::SCHEMA {
            return if self.presentation.is_some() && self.personal.materials.is_some() {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        if self.personal.materials.is_some()
            || self
                .actors
                .0
                .iter()
                .any(|a| a.spells.histories.0.iter().any(|s| s.lane == Lane::Slow))
        {
            return Err(invalid());
        }
        if (schema == super::haste::SCHEMA)
            != (self.world.descriptor.features & super::haste::FEATURE != 0)
        {
            return Err(invalid());
        }
        if schema == super::haste::SCHEMA {
            return if self.presentation.is_some()
                && self
                    .actors
                    .0
                    .iter()
                    .all(|a| a.spells.histories.0.len() <= 10)
            {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        if self.actors.0.iter().any(|a| {
            a.spells.histories.0.len()
                > if schema == super::GREATER_HASTE_SCHEMA {
                    9
                } else {
                    8
                }
        }) {
            return Err(invalid());
        }
        if self
            .actors
            .0
            .iter()
            .any(|a| a.spells.histories.0.iter().any(|s| s.lane == Lane::Haste))
        {
            return Err(invalid());
        }
        if schema != super::GREATER_HASTE_SCHEMA
            && self.actors.0.iter().any(|a| {
                a.spells.histories.0.len() > 8
                    || a.spells
                        .histories
                        .0
                        .iter()
                        .any(|s| s.lane == Lane::GreaterHaste)
            })
        {
            return Err(invalid());
        }
        if schema == super::GREATER_HASTE_SCHEMA && self.presentation.is_none() {
            return Err(invalid());
        }
        Ok(())
    }
}
fn write(w: &mut Writer, d: &UnvalidatedData, forced: Option<u16>) -> Result<(), ApplicationError> {
    w.put(&super::MAGIC.to_be_bytes())?;
    let schema = forced.unwrap_or_else(|| {
        if d.greater_haste() {
            super::GREATER_HASTE_SCHEMA
        } else if d.presentation.is_some() {
            super::PRESENTATION_SCHEMA
        } else {
            super::SCHEMA
        }
    });
    d.schema_lanes(schema)?;
    w.schema = schema;
    w.put(&schema.to_be_bytes())?;
    w.put(&[0; 6])?;
    w.put(&((super::SECTION_COUNT + usize::from(d.presentation.is_some())) as u16).to_be_bytes())?;
    w.put(&[0; 2])?;
    write_sections(w, d)?;
    let n = u32::try_from(w.pos).map_err(|_| invalid())?;
    if let Some(b) = &mut w.bytes {
        b[8..12].copy_from_slice(&n.to_be_bytes());
    }
    Ok(())
}
pub(super) fn write_sections(w: &mut Writer, d: &UnvalidatedData) -> Result<(), ApplicationError> {
    w.section(1, &d.world)?;
    w.section(2, &d.room)?;
    w.section(3, &d.manifest)?;
    w.section(4, &d.actors)?;
    w.section(5, &d.honor)?;
    w.section(6, &d.quest)?;
    w.section(7, &d.time_attack)?;
    w.section(8, &d.siege)?;
    w.section(9, &d.race)?;
    w.section(10, &d.npcs)?;
    w.section(11, &d.personal)?;
    w.section(12, &d.result)?;
    if let Some(p) = &d.presentation {
        w.section(13, p)?;
    }
    Ok(())
}
pub fn encode(data: &UnvalidatedData) -> Result<Vec<u8>, ApplicationError> {
    if data.vip.is_some() {
        return super::vip::encode_attached(data);
    }
    if data.armor.is_some() {
        return super::armor::encode_attached(data);
    }
    if data.healing.is_some() {
        return super::healing::encode_attached(data);
    }
    if data.speed.is_some() {
        return super::speed::encode_attached(data);
    }
    if data.haste.is_some() {
        return super::haste::encode_attached(data);
    }
    if let Some(motion) = &data.npc_motion {
        let mut base = data.clone();
        base.npc_motion = None;
        return super::npc_motion::encode(&super::npc_motion::Candidate {
            data: base,
            motion: motion.state.clone(),
        });
    }
    encode_base(data, None)
}
pub(super) fn encode_base(
    data: &UnvalidatedData,
    schema: Option<u16>,
) -> Result<Vec<u8>, ApplicationError> {
    let mut count = Writer::count();
    write(&mut count, data, schema)?;
    let mut out = Writer::allocated(count.pos)?;
    write(&mut out, data, schema)?;
    if out.pos != count.pos {
        return Err(invalid());
    }
    out.bytes.ok_or_else(invalid)
}
