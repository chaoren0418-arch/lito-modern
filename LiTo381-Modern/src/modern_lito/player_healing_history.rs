//! Complete participant-healing ledgers, not a wire decoder or live UI update.
//! Incoming events omit foreign selection epochs; local receipts retain ours.
use super::{
    equipment_history::{Identity, State},
    player_healing::{Impact, Receipt},
    target_history::{Participant, World},
    use_history::Spell,
    vital_balance,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Incoming {
    pub world: World,
    pub recipient: Participant,
    pub received: i64,
    pub restored: i64,
    pub last: Option<Impact>,
}
impl Incoming {
    pub fn valid(self) -> bool {
        self.world.valid()
            && self.recipient.valid()
            && self.received >= 0
            && self.restored >= 0
            && bounded(self.restored, self.received, 65534)
            && (self.received == 0) == self.last.is_none()
            && (self.received != 0 || self.restored == 0)
            && self.last.map_or(true, |last| {
                last.valid()
                    && last.world == self.world
                    && last.recipient() == Some(self.recipient)
                    && self.received < last.use_receipt.revision
                    && self.restored >= last.target.applied
                    && (self.received != 1 || self.restored == last.target.applied)
            })
    }
    pub fn initial(world: World, recipient: Participant) -> Option<Self> {
        let value = Self {
            world,
            recipient,
            received: 0,
            restored: 0,
            last: None,
        };
        value.valid().then_some(value)
    }
    pub fn receive(self, next: Impact) -> Option<Self> {
        if !self.valid() || !next.valid() {
            return None;
        }
        if let Some(last) = self.last {
            let a = last.target.before;
            let b = next.target.before;
            let same_entity = last.caster.entity_id == next.caster.entity_id;
            let same_identity = last.caster.identity == next.caster.identity;
            if next.use_receipt.revision <= last.use_receipt.revision
                || b.life < a.life
                || b.equipment_revision < a.equipment_revision
                || next.target_effect_revision < last.target_effect_revision
                || same_entity != same_identity
                || same_entity
                    && (next.use_receipt.sequence <= last.use_receipt.sequence
                        || next.use_receipt.before.life < last.use_receipt.before.life
                        || next.use_receipt.before.equipment_revision
                            < last.use_receipt.before.equipment_revision)
            {
                return None;
            }
        }
        let value = Self {
            received: self.received.checked_add(1)?,
            restored: self.restored.checked_add(next.target.applied)?,
            last: Some(next),
            ..self
        };
        value.valid().then_some(value)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outgoing {
    pub spending: Spell,
    pub last: Option<Receipt>,
}
impl Outgoing {
    pub fn valid(self) -> bool {
        self.spending.valid()
            && self.spending.current.cost <= 4
            && bounded(self.spending.current.spent, self.spending.current.casts, 4)
            && bounded(
                self.spending.current.prior_spent,
                self.spending.current.prior_casts,
                4,
            )
            && self.spending.last.is_none() == self.last.is_none()
            && self.last.map_or(true, |r| {
                r.valid() && Some(r.use_receipt) == self.spending.last
            })
    }
    pub fn initial(gear: i64, revision: i64, cost: i32) -> Option<Self> {
        let value = Self {
            spending: Spell::initial(gear, revision, cost)?,
            last: None,
        };
        value.valid().then_some(value)
    }
    pub fn cast(self, next: Receipt) -> Option<Self> {
        if !self.valid() || !next.valid() || self.last.is_some_and(|r| r.identity != next.identity)
        {
            return None;
        }
        let u = next.use_receipt;
        let value = Self {
            spending: self.spending.cast(u.revision, u.sequence, u.before)?,
            last: Some(next),
        };
        value.valid().then_some(value)
    }
    pub fn reprice(self, gear: i64, revision: i64, cost: i32) -> Option<Self> {
        if !self.valid() {
            return None;
        }
        let value = Self {
            spending: self.spending.reprice(gear, revision, cost)?,
            ..self
        };
        value.valid().then_some(value)
    }
}

// Division keeps wide accumulated counters exact without multiplication overflow.
fn bounded(total: i64, count: i64, maximum: i64) -> bool {
    total / maximum < count || total / maximum == count && total % maximum == 0
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub incoming: Incoming,
    pub outgoing: Option<Outgoing>,
}
impl Entry {
    pub fn reprice(self, gear: i64, revision: i64, cost: Option<i32>) -> Option<Self> {
        let outgoing = match (self.outgoing, cost) {
            (None, None) => None,
            (Some(outgoing), Some(cost)) => Some(outgoing.reprice(gear, revision, cost)?),
            _ => return None,
        };
        self.incoming.valid().then_some(Self { outgoing, ..self })
    }
    pub fn valid_for(self, identity: Identity, resources: vital_balance::State) -> bool {
        if !identity.valid()
            || !resources.valid()
            || !self.incoming.valid()
            || self.outgoing.is_some_and(|o| !o.valid())
            || self.incoming.world != identity.world
            || self.incoming.recipient != identity.actor
        {
            return false;
        }
        let received = self.incoming.last;
        let sent = self.outgoing.and_then(|o| o.last);
        if let Some(r) = received {
            let h = r.target.before;
            if r.use_receipt.revision > resources.revision
                || Some(h.life) > resources.life.checked_add(1)
                || h.equipment_revision > resources.equipment_revision
                || h.equipment_revision == resources.equipment_revision
                    && h.maximum_hp != resources.vitals.maximum_hp
            {
                return false;
            }
            if r.is_self()
                && sent.map_or(true, |s| {
                    s.use_receipt.revision < r.use_receipt.revision
                        || s.use_receipt.revision == r.use_receipt.revision && s.impact() != r
                        || s.use_receipt.revision > r.use_receipt.revision
                            && s.use_receipt.sequence <= r.use_receipt.sequence
                })
            {
                return false;
            }
        }
        if let Some(s) = sent {
            if s.identity != identity {
                return false;
            }
            if s.impact().is_self()
                && received.map_or(true, |r| {
                    r.use_receipt.revision < s.use_receipt.revision
                        || r.use_receipt.revision == s.use_receipt.revision && s.impact() != r
                })
            {
                return false;
            }
        }
        true
    }
}

/// Exact complete human/human event. Authorization, effect lifecycle, request
/// binding and skipped-frame following are deliberately NOT inferred here.
pub fn complete(caster: &State, target: &State, receipt: Receipt) -> Option<(State, State)> {
    if !caster.valid()
        || !target.valid()
        || !receipt.valid()
        || caster.identity != receipt.identity
        || caster.world_revision != target.world_revision
        || caster.world_revision.checked_add(1) != Some(receipt.use_receipt.revision)
        || target.identity.world != receipt.identity.world
        || receipt.impact().recipient() != Some(target.identity.actor)
        || caster
            .spells
            .values()
            .any(|s| s.current.last_cast_sequence >= receipt.use_receipt.sequence)
        || receipt.impact().is_self() && caster != target
    {
        return None;
    }
    let c = caster.healing?;
    let t = target.healing?;
    let sent = c.outgoing?.cast(receipt)?;
    let received = t.incoming.receive(receipt.impact())?;
    let next = receipt.use_receipt.revision;
    let caster_resources = if receipt.impact().is_self() {
        caster.resources.restore_and_spend(
            next,
            receipt.target.applied,
            i64::from(receipt.use_receipt.cost),
        )?
    } else {
        caster
            .resources
            .spend(next, i64::from(receipt.use_receipt.cost))?
    };
    let target_resources = if receipt.impact().is_self() {
        caster_resources
    } else {
        target.resources.restore(next, receipt.target.applied)?
    };
    if !receipt.matches_balances(
        caster.resources,
        target.resources,
        caster_resources,
        target_resources,
    ) {
        return None;
    }
    let mut a = caster.clone();
    a.world_revision = next;
    a.resources = caster_resources;
    a.healing = Some(Entry {
        incoming: if receipt.impact().is_self() {
            received
        } else {
            c.incoming
        },
        outgoing: Some(sent),
    });
    let mut b = if receipt.impact().is_self() {
        a.clone()
    } else {
        target.clone()
    };
    b.world_revision = next;
    b.resources = target_resources;
    if !receipt.impact().is_self() {
        b.healing = Some(Entry {
            incoming: received,
            ..t
        });
    }
    (a.valid() && b.valid()).then_some((a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        application_protocol::{CharacterKey, SelectionContext},
        equipment_history::{follows_reprofile, Profile},
        equipment_history_wire,
        equipment_transition::Vitals,
        participant::{MemberKind, ParticipantId},
        player_healing::Effects,
        target_history::{Effect, Health, Kind, Lane, Target},
        use_history::{Anchor, Cast, Items},
        world_map::WorldCell,
    };
    use std::collections::BTreeMap;
    fn initial(source_id: u32, entity_id: u64, can_cast: bool) -> State {
        let key = CharacterKey {
            source_id,
            object_id: 3333,
        };
        let identity = Identity {
            world: World {
                room_id: 1,
                match_id: 2,
                generation: 3,
            },
            source: SelectionContext {
                key,
                epoch: if source_id == 7 { 1 } else { 991 },
            },
            channel_id: 0,
            actor: Participant {
                entity_id,
                identity: ParticipantId::new(MemberKind::Human, key),
            },
        };
        let resources = vital_balance::State::initial(
            1,
            0,
            Vitals {
                hp: 40,
                maximum_hp: 100,
                mp: 100,
                maximum_mp: 100,
            },
        )
        .unwrap();
        State {
            identity,
            world_revision: 1,
            resources,
            spells: if can_cast {
                BTreeMap::from([(Lane::Teleport, Spell::initial(0, 1, 1).unwrap())])
            } else {
                BTreeMap::new()
            },
            items: Some(Items::initial(0, 1, 0).unwrap()),
            targets: BTreeMap::new(),
            healing: Some(Entry {
                incoming: Incoming::initial(identity.world, identity.actor).unwrap(),
                outgoing: can_cast.then(|| Outgoing::initial(0, 1, 4).unwrap()),
            }),
        }
    }
    fn anchor(a: &State) -> Anchor {
        Anchor {
            equipment_revision: a.resources.equipment_revision,
            life: a.resources.life + 1,
            vitals: a.resources.vitals,
            x: 10,
            y: 10,
        }
    }
    fn receipt(a: &State, b: &State, sequence: i64) -> Receipt {
        let before = anchor(a);
        let cost = a.healing.unwrap().outgoing.unwrap().spending.current.cost;
        let applied = 8.min(b.resources.vitals.maximum_hp - b.resources.vitals.hp);
        Receipt {
            identity: a.identity,
            use_receipt: Cast {
                revision: a.world_revision + 1,
                sequence,
                before,
                cost,
                mp_after: before.vitals.mp - i64::from(cost),
            },
            target_effect_revision: 0,
            effects: Effects::default(),
            base_amount: 8,
            adjusted_amount: 8,
            target: Effect {
                kind: Kind::Heal,
                before: Health {
                    target: Target::Participant(b.identity.actor),
                    life: b.resources.life + 1,
                    equipment_revision: b.resources.equipment_revision,
                    cell: WorldCell { x: 10, y: 10 },
                    hp: b.resources.vitals.hp,
                    maximum_hp: b.resources.vitals.maximum_hp,
                },
                raw: 8,
                applied,
                hp_after: b.resources.vitals.hp + applied,
            },
        }
    }
    fn cast(a: &State, b: &State, sequence: i64) -> (State, State) {
        complete(a, b, receipt(a, b, sequence)).unwrap()
    }
    fn resources(a: &mut State, r: vital_balance::State) {
        a.resources = r;
        a.world_revision = r.revision;
        assert!(a.valid());
    }
    fn event(mut r: Impact, revision: i64, sequence: i64, caster: Participant) -> Impact {
        r.use_receipt.revision = revision;
        r.use_receipt.sequence = sequence;
        r.caster = caster;
        r
    }
    #[test]
    fn owner_wire5_preserves_cross_source_zero_cost_and_mixed_life_history() {
        use equipment_history_wire::{
            decode_bound_with_schema, encode_with_schema, PLAYER_HEALING_SCHEMA as WIRE,
        };
        fn round(s: &State) -> State {
            let raw = encode_with_schema(s, WIRE).unwrap();
            let read = decode_bound_with_schema(&raw, s.identity, s.world_revision, WIRE).unwrap();
            assert_eq!(&read, s);
            assert_eq!(raw, encode_with_schema(&read, WIRE).unwrap());
            read
        }
        let a = round(&initial(7, 1, true));
        let b = round(&initial(42, 2, false));
        let (a, b) = cast(&a, &b, 1);
        round(&a);
        round(&b);
        assert_eq!(
            b.healing
                .unwrap()
                .incoming
                .last
                .unwrap()
                .caster
                .identity
                .source_id,
            7
        );
        let mut a = round(
            &a.reprofile_with_healing(
                3,
                &Profile {
                    equipment_revision: 1,
                    maximum_hp: 100,
                    maximum_mp: 100,
                    costs: BTreeMap::from([(Lane::Teleport, 2)]),
                    fixed_recovery: Some(0),
                },
                Some(0),
            )
            .unwrap(),
        );
        let next = a
            .resources
            .regenerate(4, 100 - a.resources.vitals.hp, 0, 0, 0)
            .unwrap();
        resources(&mut a, next);
        let (a, _) = cast(&a, &a, 2);
        let a = round(&a);
        let h = a.healing.unwrap();
        assert_eq!(h.outgoing.unwrap().spending.current.casts, 2);
        assert_eq!(a.resources.flows.spent, 4);
        assert_eq!(h.incoming.received, 1);
        assert_eq!(h.incoming.restored, 0);
        assert_eq!(h.incoming.last.unwrap().use_receipt.cost, 0);
        let mut revived = a.clone();
        let next = a
            .resources
            .damage(6, 100)
            .unwrap()
            .respawn(7, 20, 0)
            .unwrap();
        resources(&mut revived, next);
        round(&revived);
        let (again, _) = cast(&revived, &revived, 3);
        assert_eq!(
            round(&again)
                .healing
                .unwrap()
                .incoming
                .last
                .unwrap()
                .target
                .before
                .life,
            2
        );
    }
    #[test]
    fn owner_wire5_is_mandatory_bound_and_rejects_all_prefixes_and_noncanonical_mutations() {
        use equipment_history_wire::{
            decode_bound_with_schema, decode_with_schema, encode_with_schema,
            PLAYER_HEALING_SCHEMA as WIRE,
        };
        let a = initial(7, 1, true);
        let (s, _) = cast(&a, &a, 1);
        let raw = encode_with_schema(&s, WIRE).unwrap();
        for schema in 1..=4 {
            assert!(encode_with_schema(&s, schema).is_err());
            assert!(decode_with_schema(&raw, schema).is_err());
        }
        for n in 0..raw.len() {
            let mut b = raw[..n].to_vec();
            if n >= 12 {
                b[8..12].copy_from_slice(&(n as u32).to_be_bytes());
            }
            assert!(decode_with_schema(&b, WIRE).is_err(), "prefix {n}");
        }
        let mut b = raw.clone();
        b.push(0);
        let n = b.len() as u32;
        b[8..12].copy_from_slice(&n.to_be_bytes());
        assert!(decode_with_schema(&b, WIRE).is_err());
        let mut rejected = 0;
        for n in 0..raw.len() {
            let mut b = raw.clone();
            b[n] ^= 1;
            match decode_with_schema(&b, WIRE) {
                Ok(s) => assert_eq!(b, encode_with_schema(&s, WIRE).unwrap(), "offset {n}"),
                Err(_) => rejected += 1,
            }
        }
        assert!(rejected > 500);
        let mut id = s.identity;
        id.source.epoch ^= 1;
        assert!(decode_bound_with_schema(&raw, id, s.world_revision, WIRE).is_err());
        assert!(decode_bound_with_schema(&raw, s.identity, s.world_revision + 1, WIRE).is_err());
        let empty = encode_with_schema(&a, WIRE).unwrap();
        assert_eq!(empty.len(), 528);
        for offset in [444, 445, 448, 449, 524, 525] {
            let mut b = empty.clone();
            b[offset] = 128;
            assert!(decode_with_schema(&b, WIRE).is_err());
        }
        let mut missing = empty[..380].to_vec();
        missing[8..12].copy_from_slice(&380u32.to_be_bytes());
        assert!(decode_with_schema(&missing, WIRE).is_err());
    }
    #[test]
    fn self_commit_links_two_histories_at_one_revision_and_old_wires_reject_even_empty_new_history()
    {
        let a = initial(7, 1, true);
        let (after, target) = cast(&a, &a, 1);
        assert_eq!(after, target);
        assert_eq!(
            (48, 96),
            (after.resources.vitals.hp, after.resources.vitals.mp)
        );
        let h = after.healing.unwrap();
        assert_eq!((1, 8), (h.incoming.received, h.incoming.restored));
        assert_eq!(
            h.outgoing.unwrap().last.unwrap().impact(),
            h.incoming.last.unwrap()
        );
        for schema in 1..=4 {
            assert!(equipment_history_wire::encode_with_schema(&a, schema).is_err());
            assert!(equipment_history_wire::encode_with_schema(&after, schema).is_err());
        }
        let mut forged = after.clone();
        forged.healing = None;
        assert!(!forged.valid());
        assert_eq!(40, a.resources.vitals.hp);
        assert_eq!(0, a.healing.unwrap().incoming.received);
    }
    #[test]
    fn cross_source_same_objid_and_foreign_epoch_do_not_rebind_recipient() {
        let a = initial(7, 1, true);
        let b = initial(42, 2, false);
        let (after, target) = cast(&a, &b, 1);
        assert_eq!(
            (40, 96),
            (after.resources.vitals.hp, after.resources.vitals.mp)
        );
        assert_eq!(
            (48, 100),
            (target.resources.vitals.hp, target.resources.vitals.mp)
        );
        assert_eq!(991, target.identity.source.epoch);
        assert!(target.healing.unwrap().outgoing.is_none());
        let r = after.healing.unwrap().outgoing.unwrap().last.unwrap();
        let mut other = r;
        other.identity.source.epoch = 88;
        assert_ne!(r, other);
        assert_eq!(r.impact(), other.impact());
    }
    #[test]
    fn mixed_potion_regeneration_cast_clip_respawn_and_free_heal_preserve_exact_history_corpus() {
        let a = initial(7, 1, true);
        let (mut a, _) = cast(&a, &a, 1);
        a = a
            .cast(Lane::Teleport, 3, 2, WorldCell { x: 10, y: 10 }, vec![])
            .unwrap();
        a.items = Some(
            a.items
                .unwrap()
                .use_item(4, 1, 40010, anchor(&a), 10)
                .unwrap(),
        );
        let r = a.resources.restore(4, 10).unwrap();
        resources(&mut a, r);
        let r = a.resources.regenerate(5, 2, 0, 3, 0).unwrap();
        resources(&mut a, r);
        let profile = Profile {
            equipment_revision: 1,
            maximum_hp: 50,
            maximum_mp: 90,
            costs: BTreeMap::from([(Lane::Teleport, 2)]),
            fixed_recovery: Some(3),
        };
        assert!(a.reprofile(6, &profile).is_none());
        let before = a;
        let mut a = before.reprofile_with_healing(6, &profile, Some(0)).unwrap();
        assert!(follows_reprofile(&before, &a));
        assert_eq!(
            before.healing.unwrap().incoming,
            a.healing.unwrap().incoming
        );
        assert_eq!(
            before.healing.unwrap().outgoing.unwrap().last,
            a.healing.unwrap().outgoing.unwrap().last
        );
        let r = a
            .resources
            .damage(7, 50)
            .unwrap()
            .respawn(8, 20, 0)
            .unwrap();
        resources(&mut a, r);
        let (a, _) = cast(&a, &a, 3);
        let r = a.resources;
        let h = a.healing.unwrap();
        assert_eq!((28, 90), (r.vitals.hp, r.vitals.mp));
        assert_eq!((26, 5), (r.flows.restored, r.flows.spent));
        assert_eq!((10, 8), (r.flows.hp_clipped, r.flows.mp_clipped));
        assert_eq!((2, 16), (h.incoming.received, h.incoming.restored));
        assert_eq!(10, a.items.unwrap().restored);
        assert_eq!(2, h.outgoing.unwrap().last.unwrap().target.before.life);
        println!(
            "player-healing-history-v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            a.world_revision,
            r.equipment_revision,
            r.life,
            r.vitals.hp,
            r.vitals.mp,
            r.flows.restored,
            r.flows.spent,
            r.flows.hp_clipped,
            r.flows.mp_clipped,
            h.incoming.received,
            h.outgoing.unwrap().last.unwrap().use_receipt.sequence
        );
    }
    #[test]
    fn incoming_keeps_foreign_sequences_scoped_and_rejects_replay_aliases_world_change_and_overflow(
    ) {
        let a = initial(7, 1, true);
        let b = initial(42, 2, false);
        let r = receipt(&a, &b, 1).impact();
        let first = event(r, 10, 5, r.caster);
        let incoming = b.healing.unwrap().incoming.receive(first).unwrap();
        let mut other = r.caster;
        other.entity_id = 3;
        other.identity.source_id = 43;
        let next = event(r, 11, 1, other);
        assert_eq!(16, incoming.receive(next).unwrap().restored);
        assert!(incoming.receive(first).is_none());
        assert!(incoming.receive(event(r, 11, 5, r.caster)).is_none());
        other.identity = r.caster.identity;
        assert!(incoming.receive(event(r, 11, 6, other)).is_none());
        let mut bad = incoming;
        bad.world.generation += 1;
        assert!(!bad.valid());
        let overflow = Incoming {
            received: i64::MAX - 3,
            restored: i64::MAX,
            last: Some(event(r, i64::MAX - 2, 5, r.caster)),
            ..incoming
        };
        assert!(overflow.valid());
        assert!(overflow
            .receive(event(r, i64::MAX - 1, 1, next.caster))
            .is_none());
    }
    #[test]
    fn bot_full_health_and_erosion_retain_incoming_event_count_without_fake_damage() {
        let a = initial(7, 1, true);
        let b = initial(42, 2, false);
        let mut r = receipt(&a, &b, 1).impact();
        let mut bot = b.identity.actor;
        bot.identity.kind = MemberKind::ServerBot;
        bot.identity.source_id = 7;
        r.target.before.target = Target::Participant(bot);
        let h = Incoming::initial(r.world, bot).unwrap().receive(r).unwrap();
        assert_eq!(8, h.restored);
        for erosion in [false, true] {
            let mut r = receipt(&a, &b, 1).impact();
            r.effects.water_life = true;
            r.effects.healing_erosion = erosion;
            r.adjusted_amount = if erosion { -16 } else { 16 };
            r.target.raw = r.adjusted_amount.max(0);
            if !erosion {
                r.target.before.hp = 100;
            }
            r.target.applied = 0;
            r.target.hp_after = r.target.before.hp;
            let h = b.healing.unwrap().incoming.receive(r).unwrap();
            assert_eq!((1, 0), (h.received, h.restored));
            assert!(!r.effects.consumed().water_life);
        }
    }
    #[test]
    fn orphaned_self_receipts_and_missing_or_inflated_balances_fail_closed() {
        let a = initial(7, 1, true);
        let (after, _) = cast(&a, &a, 1);
        let mut forged = after.clone();
        forged.healing.as_mut().unwrap().incoming = a.healing.unwrap().incoming;
        assert!(!forged.valid());
        let mut forged = after.clone();
        forged.healing.as_mut().unwrap().outgoing = None;
        assert!(!forged.valid());
        let mut forged = after.clone();
        forged.healing.as_mut().unwrap().incoming.restored += 1;
        assert!(!forged.valid());
        let b = initial(42, 2, false);
        let (_, mut target) = cast(&a, &b, 1);
        target.healing = b.healing;
        assert!(!target.valid());
    }
    #[test]
    fn ordinary_and_healing_casts_share_owner_sequence_not_recipient_foreign_sequence() {
        let a = initial(7, 1, true);
        let (healed, _) = cast(&a, &a, 1);
        assert!(healed
            .cast(Lane::Teleport, 3, 1, WorldCell { x: 10, y: 10 }, vec![])
            .is_none());
        let ordinary = a
            .cast(Lane::Teleport, 2, 1, WorldCell { x: 10, y: 10 }, vec![])
            .unwrap();
        assert!(complete(&ordinary, &ordinary, receipt(&ordinary, &ordinary, 1)).is_none());
        assert_eq!(
            2,
            cast(&ordinary, &ordinary, 2)
                .0
                .healing
                .unwrap()
                .outgoing
                .unwrap()
                .last
                .unwrap()
                .use_receipt
                .sequence
        );
    }
    #[test]
    fn exact_candidate_rejects_changed_resource_world_epoch_or_forged_anchor_without_mutation() {
        let a = initial(7, 1, true);
        let b = initial(42, 2, false);
        let r = receipt(&a, &b, 1);
        let mut changed = b.clone();
        let balance = changed.resources.regenerate(2, 1, 0, 0, 0).unwrap();
        resources(&mut changed, balance);
        assert!(complete(&a, &changed, r).is_none());
        let mut epoch = a.clone();
        epoch.identity.source.epoch += 1;
        assert!(complete(&epoch, &b, r).is_none());
        let mut forged = r;
        forged.use_receipt.before.vitals.hp += 1;
        assert!(forged.valid());
        assert!(complete(&a, &b, forged).is_none());
        assert_eq!(
            0,
            a.healing.unwrap().outgoing.unwrap().spending.current.casts
        );
        assert_eq!(0, b.healing.unwrap().incoming.received);
    }
    #[test]
    fn duplicate_world_event_cannot_masquerade_as_zero_recovery_item() {
        let a = initial(7, 1, true);
        let (mut after, _) = cast(&a, &a, 1);
        let mut full = anchor(&a);
        full.vitals.hp = 100;
        after.items = Some(
            Items::initial(0, 1, 0)
                .unwrap()
                .use_item(2, 1, 40010, full, 10)
                .unwrap(),
        );
        assert_eq!(0, after.items.unwrap().restored);
        assert!(!after.valid());
    }
    #[test]
    fn impossible_prior_costs_recovery_totals_and_regressing_effect_revision_are_rejected() {
        let a = initial(7, 1, true);
        let b = initial(42, 2, false);
        let mut r = receipt(&a, &b, 1);
        let impact = r.impact();
        let impossible = Incoming {
            received: 2,
            restored: 131069,
            last: Some(event(impact, 10, 5, impact.caster)),
            ..b.healing.unwrap().incoming
        };
        assert!(!impossible.valid());
        r.use_receipt.revision = 3;
        r.use_receipt.sequence = 2;
        let ledger = super::super::skill_spending::Ledger {
            equipment_revision: 1,
            profile_revision: 4,
            cost: 0,
            prior_casts: 2,
            prior_spent: 9,
            casts: 2,
            spent: 9,
            last_cast_revision: 3,
            last_cast_sequence: 2,
        };
        let spell = Spell {
            current: ledger,
            last: Some(r.use_receipt),
        };
        assert!(spell.valid());
        assert!(!Outgoing {
            spending: spell,
            last: Some(r)
        }
        .valid());
        let mut first = event(impact, 10, 5, impact.caster);
        first.target_effect_revision = 5;
        let incoming = b.healing.unwrap().incoming.receive(first).unwrap();
        assert!(incoming
            .receive(event(impact, 11, 6, impact.caster))
            .is_none());
    }
}
