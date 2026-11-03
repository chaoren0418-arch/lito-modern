//! Canonical bounded history section, not an application opcode or a complete
//! authoritative world. The containing negotiated envelope must bind the owner,
//! revision, roster and inventory before publishing any UI/gameplay change.
use super::{
    application_protocol::{self as app, ApplicationError, CharacterKey, SelectionContext},
    equipment_history::{Identity, State},
    equipment_transition::Vitals,
    npc_protocol as npc,
    participant::{MemberKind, ParticipantId},
    quest_protocol,
    skill_spending::Ledger,
    target_history::{Batch, Effect, Health, History, Kind, Lane, Participant, Target, World},
    time_attack_protocol,
    use_history::{Anchor, Cast, Items, Spell, Use},
    vital_balance::{self, Flows},
    world_map::WorldCell,
};
use std::collections::BTreeMap;

pub const MAGIC: u32 = 0x4c484931;
pub const SCHEMA: u16 = 1;
pub const GREATER_HASTE_SCHEMA: u16 = 2;
pub const HASTE_SCHEMA: u16 = 3;
pub const SLOW_SCHEMA: u16 = 4;
/// Candidate owner section; no negotiated application route selects it yet.
pub const PLAYER_HEALING_SCHEMA: u16 = 5;
pub const HEADER_BYTES: usize = 256;
pub const MAX_BYTES: usize = 32 * 1024;
pub const EFFECT_BYTES: usize = 112;
const LANES: [Lane; 11] = [
    Lane::Teleport,
    Lane::LightArrow,
    Lane::Healing,
    Lane::FireStorm,
    Lane::Disintegrate,
    Lane::NpcLightArrow,
    Lane::NpcFireStorm,
    Lane::NpcDisintegrate,
    Lane::GreaterHaste,
    Lane::Haste,
    Lane::Slow,
];

pub fn encode(s: &State) -> Result<Vec<u8>, ApplicationError> {
    encode_with_schema(s, SCHEMA)
}
pub fn encode_with_schema(s: &State, schema: u16) -> Result<Vec<u8>, ApplicationError> {
    if !s.valid()
        || s.healing.is_some() != (schema == PLAYER_HEALING_SCHEMA)
        || !matches!(
            schema,
            SCHEMA | GREATER_HASTE_SCHEMA | HASTE_SCHEMA | SLOW_SCHEMA | PLAYER_HEALING_SCHEMA
        )
        || schema < SLOW_SCHEMA && s.spells.contains_key(&Lane::Slow)
        || schema < HASTE_SCHEMA && s.spells.contains_key(&Lane::Haste)
        || schema == SCHEMA && s.spells.contains_key(&Lane::GreaterHaste)
    {
        return Err(invalid());
    }
    let mut b = Vec::with_capacity(HEADER_BYTES);
    b.extend(MAGIC.to_be_bytes());
    b.extend(schema.to_be_bytes());
    zero(&mut b, 6);
    put_world(&mut b, s.identity.world);
    b.extend(s.identity.source.key.source_id.to_be_bytes());
    b.extend(s.identity.source.key.object_id.to_be_bytes());
    b.extend(s.identity.source.epoch.to_be_bytes());
    b.push(s.identity.channel_id);
    zero(&mut b, 3);
    put_participant(&mut b, s.identity.actor);
    b.extend(s.world_revision.to_be_bytes());
    put_resources(&mut b, s.resources);
    b.push(s.spells.len() as u8);
    b.push(u8::from(s.items.is_some()));
    zero(&mut b, 2);
    for (index, lane) in LANES.into_iter().enumerate() {
        let Some(spell) = s.spells.get(&lane) else {
            continue;
        };
        b.push(index as u8);
        zero(&mut b, 3);
        put_ledger(&mut b, spell.current);
        let history = s.targets.get(&lane);
        let retained = match history.and_then(|h| h.structure.as_ref()) {
            None => 0,
            Some(st) if history.is_some_and(|h| *st == h.last) => 1,
            Some(_) => 2,
        };
        b.push(u8::from(spell.last.is_some()));
        b.push(retained);
        zero(&mut b, 2);
        if let Some(last) = spell.last {
            put_cast(&mut b, last);
            put_effects(&mut b, &history.ok_or_else(invalid)?.last.targets);
        }
        if retained == 2 {
            let receipt = history
                .and_then(|h| h.structure.as_ref())
                .ok_or_else(invalid)?;
            put_cast(&mut b, receipt.use_receipt);
            put_effects(&mut b, &receipt.targets);
        }
    }
    if let Some(items) = s.items {
        put_items(&mut b, items);
    }
    if let Some(healing) = s.healing {
        healing::put(&mut b, healing);
    }
    if b.len() > MAX_BYTES {
        return Err(invalid());
    }
    let size = b.len() as u32;
    b[8..12].copy_from_slice(&size.to_be_bytes());
    Ok(b)
}
pub fn decode(bytes: &[u8]) -> Result<State, ApplicationError> {
    decode_with_schema(bytes, SCHEMA)
}
pub fn decode_with_schema(bytes: &[u8], schema: u16) -> Result<State, ApplicationError> {
    if !matches!(
        schema,
        SCHEMA | GREATER_HASTE_SCHEMA | HASTE_SCHEMA | SLOW_SCHEMA | PLAYER_HEALING_SCHEMA
    ) {
        return Err(invalid());
    }
    if !(HEADER_BYTES..=MAX_BYTES).contains(&bytes.len()) {
        return Err(invalid());
    }
    let mut r = Reader { bytes, pos: 0 };
    if r.u32()? != MAGIC || r.u16()? != schema {
        return Err(invalid());
    }
    r.zeros(2)?;
    if r.u32()? as usize != bytes.len() {
        return Err(invalid());
    }
    let world = r.world()?;
    let source = SelectionContext {
        key: CharacterKey {
            source_id: r.u32()?,
            object_id: r.u64()?,
        },
        epoch: r.u64()?,
    };
    let channel_id = r.u8()?;
    r.zeros(3)?;
    let identity = Identity {
        world,
        source,
        channel_id,
        actor: r.participant()?,
    };
    let world_revision = r.i64()?;
    let resources = r.resources()?;
    let count = r.u8()?;
    let has_items = r.flag()?;
    r.zeros(2)?;
    let maximum = if schema == SCHEMA {
        8
    } else if schema == GREATER_HASTE_SCHEMA {
        9
    } else if schema == HASTE_SCHEMA {
        10
    } else {
        11
    };
    if count > maximum {
        return Err(invalid());
    }
    let mut spells = BTreeMap::new();
    let mut targets = BTreeMap::new();
    let mut previous = None;
    for _ in 0..count {
        let index = r.u8()?;
        r.zeros(3)?;
        if index >= maximum || previous.is_some_and(|p| index <= p) {
            return Err(invalid());
        }
        previous = Some(index);
        let lane = LANES[index as usize];
        let current = r.ledger()?;
        let used = r.flag()?;
        let retained = r.u8()?;
        r.zeros(2)?;
        if retained > 2 || !used && retained != 0 {
            return Err(invalid());
        }
        let last = if used {
            let use_receipt = r.cast()?;
            let latest = batch(identity, lane, use_receipt, r.effects(lane)?);
            let structure = match retained {
                0 => None,
                1 => Some(latest.clone()),
                2 => {
                    if latest.targets.iter().any(|e| {
                        matches!(e.before.target, Target::Quest { .. } | Target::TimeTower(_))
                    }) {
                        return Err(invalid());
                    }
                    Some(batch(identity, lane, r.cast()?, r.effects(lane)?))
                }
                _ => return Err(invalid()),
            };
            targets.insert(
                lane,
                History {
                    last: latest,
                    structure,
                },
            );
            Some(use_receipt)
        } else {
            None
        };
        spells.insert(lane, Spell { current, last });
    }
    let items = if has_items { Some(r.items()?) } else { None };
    let healing = if schema == PLAYER_HEALING_SCHEMA {
        Some(healing::get(&mut r, identity)?)
    } else {
        None
    };
    let state = State {
        identity,
        world_revision,
        resources,
        spells,
        items,
        targets,
        healing,
    };
    if r.pos != bytes.len() || !state.valid() {
        return Err(invalid());
    }
    Ok(state)
}
pub fn decode_bound(
    bytes: &[u8],
    expected: Identity,
    revision: i64,
) -> Result<State, ApplicationError> {
    decode_bound_with_schema(bytes, expected, revision, SCHEMA)
}
pub fn decode_bound_with_schema(
    bytes: &[u8],
    expected: Identity,
    revision: i64,
    schema: u16,
) -> Result<State, ApplicationError> {
    let state = decode_with_schema(bytes, schema)?;
    if state.identity != expected || state.world_revision != revision {
        return Err(invalid());
    }
    Ok(state)
}
fn batch(id: Identity, lane: Lane, use_receipt: Cast, targets: Vec<Effect>) -> Batch {
    Batch {
        world: id.world,
        source: id.source,
        channel_id: id.channel_id,
        caster: id.actor,
        lane,
        use_receipt,
        targets,
    }
}
fn zero(b: &mut Vec<u8>, n: usize) {
    b.resize(b.len() + n, 0);
}
fn put_world(b: &mut Vec<u8>, w: World) {
    b.extend(w.room_id.to_be_bytes());
    b.extend(w.match_id.to_be_bytes());
    b.extend(w.generation.to_be_bytes());
}
fn put_scope(b: &mut Vec<u8>, s: npc::Scope) {
    put_world(
        b,
        World {
            room_id: s.room_id,
            match_id: s.match_id,
            generation: s.generation,
        },
    );
}
fn put_participant(b: &mut Vec<u8>, p: Participant) {
    b.extend(p.entity_id.to_be_bytes());
    b.push(p.identity.kind as u8);
    zero(b, 3);
    b.extend(p.identity.source_id.to_be_bytes());
    b.extend(p.identity.id.to_be_bytes());
}
fn put_vitals(b: &mut Vec<u8>, v: Vitals) {
    for n in [v.hp, v.maximum_hp, v.mp, v.maximum_mp] {
        b.extend(n.to_be_bytes());
    }
}
fn put_resources(b: &mut Vec<u8>, r: vital_balance::State) {
    for n in [
        r.revision,
        r.equipment_revision,
        r.life,
        r.birth_hp,
        r.birth_mp,
    ] {
        b.extend(n.to_be_bytes());
    }
    put_vitals(b, r.vitals);
    let f = r.flows;
    for n in [
        f.hp_gain,
        f.hp_loss,
        f.restored,
        f.damaged,
        f.hp_granted,
        f.mp_gain,
        f.mp_loss,
        f.spent,
        f.mp_granted,
        f.hp_clipped,
        f.mp_clipped,
    ] {
        b.extend(n.to_be_bytes());
    }
}
fn put_ledger(b: &mut Vec<u8>, l: Ledger) {
    b.extend(l.equipment_revision.to_be_bytes());
    b.extend(l.profile_revision.to_be_bytes());
    b.extend(l.cost.to_be_bytes());
    zero(b, 4);
    for n in [
        l.prior_casts,
        l.prior_spent,
        l.casts,
        l.spent,
        l.last_cast_revision,
        l.last_cast_sequence,
    ] {
        b.extend(n.to_be_bytes());
    }
}
fn put_anchor(b: &mut Vec<u8>, a: Anchor) {
    b.extend(a.equipment_revision.to_be_bytes());
    b.extend(a.life.to_be_bytes());
    put_vitals(b, a.vitals);
    b.extend(a.x.to_be_bytes());
    b.extend(a.y.to_be_bytes());
}
fn put_cast(b: &mut Vec<u8>, c: Cast) {
    b.extend(c.revision.to_be_bytes());
    b.extend(c.sequence.to_be_bytes());
    put_anchor(b, c.before);
    b.extend(c.cost.to_be_bytes());
    zero(b, 4);
    b.extend(c.mp_after.to_be_bytes());
}
fn put_items(b: &mut Vec<u8>, i: Items) {
    for n in [
        i.equipment_revision,
        i.profile_revision,
        i.fixed_recovery,
        i.uses,
        i.restored,
    ] {
        b.extend(n.to_be_bytes());
    }
    b.push(u8::from(i.last.is_some()));
    zero(b, 3);
    if let Some(u) = i.last {
        for n in [u.revision, u.sequence, u.item_id] {
            b.extend(n.to_be_bytes());
        }
        put_anchor(b, u.before);
        for n in [u.fixed_recovery, u.rolled_base, u.restored, u.hp_after] {
            b.extend(n.to_be_bytes());
        }
    }
}
fn put_effects(b: &mut Vec<u8>, effects: &[Effect]) {
    b.extend((effects.len() as u16).to_be_bytes());
    zero(b, 2);
    for e in effects {
        let h = e.before;
        let tag = match h.target {
            Target::Participant(_) => 0,
            Target::Npc(_) => 1,
            Target::Honor { .. } => 2,
            Target::Quest { .. } => 3,
            Target::TimeTower(_) => 4,
        };
        b.push(tag);
        b.push(if e.kind == Kind::Damage { 0 } else { 1 });
        zero(b, 6);
        let start = b.len();
        match h.target {
            Target::Participant(p) => put_participant(b, p),
            Target::Npc(n) => {
                put_scope(b, n.scope);
                b.extend(n.serial.to_be_bytes());
            }
            Target::Honor { entity_id, team } => {
                b.extend(entity_id.to_be_bytes());
                b.push(team);
            }
            Target::Quest { id, born_revision } => {
                put_scope(b, id.scope);
                b.extend(u32::from(id.slot).to_be_bytes());
                zero(b, 4);
                b.extend(born_revision.to_be_bytes());
            }
            Target::TimeTower(t) => put_scope(b, t.scope),
        }
        zero(b, 40 - (b.len() - start));
        b.extend(h.life.to_be_bytes());
        b.extend(h.equipment_revision.to_be_bytes());
        b.extend(h.cell.x.to_be_bytes());
        b.extend(h.cell.y.to_be_bytes());
        for n in [h.hp, h.maximum_hp, e.raw, e.applied, e.hp_after] {
            b.extend(n.to_be_bytes());
        }
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(invalid)?;
        let bytes = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
        self.pos = end;
        Ok(bytes)
    }
    fn u8(&mut self) -> Result<u8, ApplicationError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, ApplicationError> {
        Ok(u16::from_be_bytes(
            self.take(2)?.try_into().map_err(|_| invalid())?,
        ))
    }
    fn u32(&mut self) -> Result<u32, ApplicationError> {
        Ok(u32::from_be_bytes(
            self.take(4)?.try_into().map_err(|_| invalid())?,
        ))
    }
    fn i32(&mut self) -> Result<i32, ApplicationError> {
        Ok(self.u32()? as i32)
    }
    fn u64(&mut self) -> Result<u64, ApplicationError> {
        Ok(u64::from_be_bytes(
            self.take(8)?.try_into().map_err(|_| invalid())?,
        ))
    }
    fn i64(&mut self) -> Result<i64, ApplicationError> {
        Ok(self.u64()? as i64)
    }
    fn flag(&mut self) -> Result<bool, ApplicationError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid()),
        }
    }
    fn zeros(&mut self, n: usize) -> Result<(), ApplicationError> {
        if self.take(n)?.iter().any(|&b| b != 0) {
            Err(invalid())
        } else {
            Ok(())
        }
    }
    fn world(&mut self) -> Result<World, ApplicationError> {
        Ok(World {
            room_id: self.u64()?,
            match_id: self.u64()?,
            generation: self.u64()?,
        })
    }
    fn scope(&mut self) -> Result<npc::Scope, ApplicationError> {
        let w = self.world()?;
        Ok(npc::Scope {
            room_id: w.room_id,
            match_id: w.match_id,
            generation: w.generation,
        })
    }
    fn participant(&mut self) -> Result<Participant, ApplicationError> {
        let entity_id = self.u64()?;
        let kind = match self.u8()? {
            0 => MemberKind::Human,
            1 => MemberKind::ServerBot,
            _ => return Err(invalid()),
        };
        self.zeros(3)?;
        Ok(Participant {
            entity_id,
            identity: ParticipantId {
                kind,
                source_id: self.u32()?,
                id: self.u64()?,
            },
        })
    }
    fn vitals(&mut self) -> Result<Vitals, ApplicationError> {
        Ok(Vitals {
            hp: self.i64()?,
            maximum_hp: self.i64()?,
            mp: self.i64()?,
            maximum_mp: self.i64()?,
        })
    }
    fn resources(&mut self) -> Result<vital_balance::State, ApplicationError> {
        Ok(vital_balance::State {
            revision: self.i64()?,
            equipment_revision: self.i64()?,
            life: self.i64()?,
            birth_hp: self.i64()?,
            birth_mp: self.i64()?,
            vitals: self.vitals()?,
            flows: Flows {
                hp_gain: self.i64()?,
                hp_loss: self.i64()?,
                restored: self.i64()?,
                damaged: self.i64()?,
                hp_granted: self.i64()?,
                mp_gain: self.i64()?,
                mp_loss: self.i64()?,
                spent: self.i64()?,
                mp_granted: self.i64()?,
                hp_clipped: self.i64()?,
                mp_clipped: self.i64()?,
            },
        })
    }
    fn ledger(&mut self) -> Result<Ledger, ApplicationError> {
        let equipment_revision = self.i64()?;
        let profile_revision = self.i64()?;
        let cost = self.i32()?;
        self.zeros(4)?;
        Ok(Ledger {
            equipment_revision,
            profile_revision,
            cost,
            prior_casts: self.i64()?,
            prior_spent: self.i64()?,
            casts: self.i64()?,
            spent: self.i64()?,
            last_cast_revision: self.i64()?,
            last_cast_sequence: self.i64()?,
        })
    }
    fn anchor(&mut self) -> Result<Anchor, ApplicationError> {
        Ok(Anchor {
            equipment_revision: self.i64()?,
            life: self.i64()?,
            vitals: self.vitals()?,
            x: self.i32()?,
            y: self.i32()?,
        })
    }
    fn cast(&mut self) -> Result<Cast, ApplicationError> {
        let revision = self.i64()?;
        let sequence = self.i64()?;
        let before = self.anchor()?;
        let cost = self.i32()?;
        self.zeros(4)?;
        Ok(Cast {
            revision,
            sequence,
            before,
            cost,
            mp_after: self.i64()?,
        })
    }
    fn items(&mut self) -> Result<Items, ApplicationError> {
        let equipment_revision = self.i64()?;
        let profile_revision = self.i64()?;
        let fixed_recovery = self.i64()?;
        let uses = self.i64()?;
        let restored = self.i64()?;
        let used = self.flag()?;
        self.zeros(3)?;
        let last = if used {
            Some(Use {
                revision: self.i64()?,
                sequence: self.i64()?,
                item_id: self.i64()?,
                before: self.anchor()?,
                fixed_recovery: self.i64()?,
                rolled_base: self.i64()?,
                restored: self.i64()?,
                hp_after: self.i64()?,
            })
        } else {
            None
        };
        Ok(Items {
            equipment_revision,
            profile_revision,
            fixed_recovery,
            uses,
            restored,
            last,
        })
    }
    fn effects(&mut self, lane: Lane) -> Result<Vec<Effect>, ApplicationError> {
        let count = usize::from(self.u16()?);
        self.zeros(2)?;
        let maximum = match lane {
            Lane::Teleport | Lane::GreaterHaste | Lane::Haste | Lane::Slow => 0,
            Lane::FireStorm => 101,
            Lane::NpcFireStorm => 48,
            _ => 1,
        };
        if count > maximum || self.bytes.len() - self.pos < count * EFFECT_BYTES {
            return Err(invalid());
        }
        let mut effects = Vec::with_capacity(count);
        for _ in 0..count {
            let tag = self.u8()?;
            let kind = match self.u8()? {
                0 => Kind::Damage,
                1 => Kind::Heal,
                _ => return Err(invalid()),
            };
            self.zeros(6)?;
            let start = self.pos;
            let target = match tag {
                0 => Target::Participant(self.participant()?),
                1 => Target::Npc(npc::Id {
                    scope: self.scope()?,
                    serial: self.u64()?,
                }),
                2 => Target::Honor {
                    entity_id: self.u64()?,
                    team: self.u8()?,
                },
                3 => {
                    let scope = self.scope()?;
                    let slot = u8::try_from(self.u32()?).map_err(|_| invalid())?;
                    self.zeros(4)?;
                    Target::Quest {
                        id: quest_protocol::Id { scope, slot },
                        born_revision: self.i64()?,
                    }
                }
                4 => Target::TimeTower(time_attack_protocol::TowerId {
                    scope: self.scope()?,
                }),
                _ => return Err(invalid()),
            };
            self.zeros(40 - (self.pos - start))?;
            let before = Health {
                target,
                life: self.i64()?,
                equipment_revision: self.i64()?,
                cell: WorldCell {
                    x: self.i32()?,
                    y: self.i32()?,
                },
                hp: self.i64()?,
                maximum_hp: self.i64()?,
            };
            effects.push(Effect {
                kind,
                before,
                raw: self.i64()?,
                applied: self.i64()?,
                hp_after: self.i64()?,
            });
        }
        Ok(effects)
    }
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid complete equipment history section",
    )
}

#[path = "equipment_history_healing_wire.rs"]
mod healing;

#[cfg(test)]
#[path = "equipment_history_wire_interop_tests.rs"]
mod interop_tests;
#[cfg(test)]
#[path = "equipment_history_wire_tests.rs"]
mod tests;
