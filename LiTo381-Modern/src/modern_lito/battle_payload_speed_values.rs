//! Local speed/reagent invariants only. These checks deliberately do NOT admit
//! a scene: callers must use Candidate::check_values/check_successor for the
//! complete candidate proof. Negotiation and controller integration remain separate.
use super::*;
use crate::modern_lito::{battle_payload::data as d, timed_protocol, use_history};
const SECOND: i64 = 1_000_000_000;
const SLOW: i64 = 64 * SECOND;
const HASTE: i64 = 1200 * SECOND;
fn lease(until: i64, now: i64, max: i64) -> Result<(), ApplicationError> {
    require(
        until >= 0
            && (until == 0
                || until > now && i128::from(until) - i128::from(now) <= i128::from(max)),
    )
}
pub(super) fn buff(b: &Buffs) -> Result<(), ApplicationError> {
    require(
        b.entity > 0
            && b.revision >= 0
            && (0..=timed_protocol::MAX_OBSERVED_NANOS).contains(&b.observed_nanos)
            && b.slow_revision >= 0
            && b.slow_revision <= b.revision
            && (b.slow_until_nanos == 0 || b.slow_revision > 0 && b.haste_until_nanos == 0)
            && matches!(b.brave_kind, 0 | 1 | 3)
            && (b.brave_kind == 0) == (b.brave_until_nanos == 0)
            && (b.haste_origin == d::HasteOrigin::None) == (b.haste_until_nanos == 0),
    )?;
    lease(b.slow_until_nanos, b.observed_nanos, SLOW)?;
    lease(
        b.haste_until_nanos,
        b.observed_nanos,
        match b.haste_origin {
            d::HasteOrigin::GreaterHaste => 2400 * SECOND,
            d::HasteOrigin::Haste => HASTE,
            _ => 300 * SECOND,
        },
    )?;
    lease(
        b.brave_until_nanos,
        b.observed_nanos,
        if b.brave_kind == 3 {
            480 * SECOND
        } else {
            300 * SECOND
        },
    )?;
    lease(b.wisdom_until_nanos, b.observed_nanos, 300 * SECOND)?;
    require(
        b.revision != 0
            || (b.observed_nanos == 0
                && b.haste_until_nanos == 0
                && b.brave_until_nanos == 0
                && b.wisdom_until_nanos == 0),
    )
}
pub(super) fn npc(s: &Speed) -> Result<(), ApplicationError> {
    require(
        s.revision >= 0
            && (0..=1_860_000).contains(&s.observed_millis)
            && s.slow_revision >= 0
            && s.slow_revision <= s.revision
            && (s.slow_until_millis == 0 || s.slow_revision > 0 && s.haste_until_millis == 0)
            && (s.revision != 0 || s.observed_millis == 0 && s.haste_until_millis == 0),
    )?;
    lease(s.haste_until_millis, s.observed_millis, HASTE / 1_000_000)?;
    lease(s.slow_until_millis, s.observed_millis, SLOW / 1_000_000)
}
fn alive(until: i64, now: i64) -> i64 {
    if until > now {
        until
    } else {
        0
    }
}
fn changed(
    b: &Buffs,
    time: i64,
    revision: i64,
    slow: bool,
    greater: bool,
) -> Result<Buffs, ApplicationError> {
    buff(b)?;
    require(
        time >= b.observed_nanos
            && time <= timed_protocol::MAX_OBSERVED_NANOS
            && revision > b.revision,
    )?;
    let mut a = b.clone();
    a.observed_nanos = time;
    a.revision = revision;
    a.brave_until_nanos = alive(b.brave_until_nanos, time);
    a.wisdom_until_nanos = alive(b.wisdom_until_nanos, time);
    if a.brave_until_nanos == 0 {
        a.brave_kind = 0;
    }
    if slow {
        a.haste_until_nanos = 0;
        a.haste_origin = d::HasteOrigin::None;
        a.slow_revision = revision;
        a.slow_until_nanos = if alive(b.haste_until_nanos, time) != 0 {
            0
        } else {
            time.checked_add(SLOW).ok_or_else(invalid)?
        };
    } else {
        a.slow_until_nanos = 0;
        a.haste_until_nanos = if alive(b.slow_until_nanos, time) != 0 {
            0
        } else {
            time.checked_add(if greater { 2400 * SECOND } else { HASTE })
                .ok_or_else(invalid)?
        };
        a.haste_origin = if a.haste_until_nanos == 0 {
            d::HasteOrigin::None
        } else if greater {
            d::HasteOrigin::GreaterHaste
        } else {
            d::HasteOrigin::Haste
        };
    }
    buff(&a)?;
    Ok(a)
}
fn npc_changed(b: &Speed, time: i64, revision: i64, slow: bool) -> Result<Speed, ApplicationError> {
    npc(b)?;
    require(time >= b.observed_millis && time <= 1_860_000 && revision > b.revision)?;
    let mut a = b.clone();
    a.observed_millis = time;
    a.revision = revision;
    if slow {
        a.haste_until_millis = 0;
        a.slow_revision = revision;
        a.slow_until_millis = if alive(b.haste_until_millis, time) != 0 {
            0
        } else {
            time.checked_add(SLOW / 1_000_000).ok_or_else(invalid)?
        };
    } else {
        a.slow_until_millis = 0;
        a.haste_until_millis = if alive(b.slow_until_millis, time) != 0 {
            0
        } else {
            time.checked_add(HASTE / 1_000_000).ok_or_else(invalid)?
        };
    }
    npc(&a)?;
    Ok(a)
}
fn missing(m: &MissingEffect) -> Result<(), ApplicationError> {
    require(
        m.required_skill
            == match m.npc_template {
                45912 | 45913 | 45914 | 45915 => 1013,
                45916 => 1014,
                45941 => 1015,
                45752 | 45753 => 4005,
                45675 | 81082 | 45625 | 45674 | 45685 => 4006,
                87000 => 4007,
                45020 => 4008,
                99019 => 4025,
                _ => return Err(invalid()),
            },
    )
}
pub(super) fn receipt(r: &d::SpellReceipt) -> Result<(), ApplicationError> {
    let u = &r.impact.usage;
    let (elapsed, t, applied, slow, max_cost) = match &r.detail {
        d::Detail::Slow(s) => {
            if let Some(m) = &s.missing_effect {
                missing(m)?;
                require(
                    matches!(s.target, StatusTarget::Npc(_))
                        && s.probability == 0
                        && s.roll == 0
                        && s.equipment_haste_items == 0,
                )?;
            } else {
                require((1..=90).contains(&s.probability) && (1..=100).contains(&s.roll))?;
            }
            require(
                s.equipment_haste_items >= 0
                    && (!matches!(s.target, StatusTarget::Npc(_)) || s.equipment_haste_items == 0)
                    && r.impact.lane == d::Lane::Slow,
            )?;
            (
                s.active_elapsed_nanos,
                &s.target,
                s.missing_effect.is_none()
                    && s.roll <= s.probability
                    && s.equipment_haste_items == 0,
                true,
                20,
            )
        }
        d::Detail::Haste(h) => {
            require(r.impact.lane == d::Lane::Haste)?;
            (h.active_elapsed_nanos, &h.target, true, false, 40)
        }
        d::Detail::GreaterHasteNeutralized(n) => {
            require(
                r.impact.lane == d::Lane::GreaterHaste
                    && r.impact.targets.0.is_empty()
                    && use_history::Cast::from(u).valid()
                    && (25..=60).contains(&u.cost)
                    && n.before.slow_until_nanos > n.active_elapsed_nanos
                    && changed(&n.before, n.active_elapsed_nanos, u.revision, false, true)?
                        == n.after,
            )?;
            require(
                r.casts > 0
                    && r.casts <= u.sequence
                    && u.sequence < u.revision
                    && i128::from(r.spent) >= i128::from(r.casts) * 25
                    && i128::from(r.spent) <= i128::from(r.casts) * 60
                    && (r.casts != 1 || r.spent == i64::from(u.cost)),
            )?;
            return Ok(());
        }
        _ => return Err(invalid()),
    };
    require(
        (0..=timed_protocol::MAX_OBSERVED_NANOS).contains(&elapsed)
            && r.impact.targets.0.is_empty()
            && use_history::Cast::from(u).valid()
            && (15..=max_cost).contains(&u.cost)
            && r.casts > 0
            && r.casts <= u.sequence
            && u.sequence < u.revision
            && i128::from(r.spent) >= i128::from(r.casts) * 15
            && i128::from(r.spent) <= i128::from(r.casts) * i128::from(max_cost)
            && (r.casts != 1 || r.spent == i64::from(u.cost)),
    )?;
    let cell = match t {
        StatusTarget::Player(p) => {
            buff(&p.before)?;
            buff(&p.after)?;
            require(
                p.participant.entity > 0
                    && p.participant.entity <= 100
                    && p.participant.identity.source > 0
                    && p.participant.identity.id > 0
                    && p.life > 0
                    && p.equipment_revision >= 0
                    && p.before.entity == p.participant.entity
                    && p.after.entity == p.participant.entity
                    && p.before.revision < u.revision
                    && p.before.observed_nanos <= elapsed,
            )?;
            require(if applied {
                changed(&p.before, elapsed, u.revision, slow, false)? == p.after
            } else {
                p.before == p.after
            })?;
            &p.cell
        }
        StatusTarget::Npc(n) => {
            npc(&n.before)?;
            npc(&n.after)?;
            let time = elapsed / 1_000_000;
            require(
                n.id.world.room > 0
                    && n.id.world.match_id > 0
                    && n.id.world.generation > 0
                    && n.id.serial > 0
                    && n.life > 0
                    && (0..=1).contains(&n.lane)
                    && n.roster_revision >= 1
                    && n.before.revision <= n.roster_revision
                    && n.roster_revision < u.revision
                    && n.after.revision < u.revision
                    && n.before.observed_millis <= time
                    && (n.before.haste_until_millis == 0 || time < n.before.haste_until_millis)
                    && (n.before.slow_until_millis == 0 || time < n.before.slow_until_millis),
            )?;
            require(if applied {
                npc_changed(
                    &n.before,
                    time,
                    n.roster_revision.checked_add(1).ok_or_else(invalid)?,
                    slow,
                )? == n.after
            } else {
                n.before == n.after
            })?;
            &n.cell
        }
    };
    let dx = i64::from(cell.x) - i64::from(u.before.cell.x);
    let dy = i64::from(cell.y) - i64::from(u.before.cell.y);
    let distance = dx.abs() + dy.abs();
    let edge = (dx + 18).abs() + (dy + 18).abs();
    require(distance <= 18 || distance == 19 && (19..=52).contains(&edge))
}
impl View<'_> {
    /// Checks ONLY the new component invariants and direct owner bindings.
    /// Success is not a complete World/Observation proof and cannot enter UI.
    pub fn check_components(&self) -> Result<(), ApplicationError> {
        let d = &self.data;
        let w = &d.world;
        require(
            w.descriptor.features & FEATURE != 0
                && d.haste.is_none()
                && d.npc_motion.is_none()
                && self.effects.world == w.identity
                && self.effects.revision == w.revision,
        )?;
        let m = d.personal.materials.as_ref().ok_or_else(invalid)?;
        require(
            m.world == w.identity
                && m.player == d.personal.player
                && m.caster.entity == d.personal.entity
                && m.caster.identity.kind == d::MemberKind::Human
                && m.caster.identity.source == m.player.source
                && m.caster.identity.id == m.player.object
                && m.player.source > 0
                && m.player.object > 0
                && m.player.epoch != 0
                && (0..=3).contains(&m.player.channel)
                && m.revision >= 0
                && m.revision <= w.revision
                && m.uses >= 0
                && m.last_sequence >= m.uses
                && m.last_life >= 0
                && m.last_equipment_revision >= 0
                && m.stocks.0.len() <= 1,
        )?;
        let mut consumed = 0i128;
        for s in &m.stocks.0 {
            require(
                s.item == 40318
                    && s.unit_weight >= 0
                    && (1..=1_000_000).contains(&s.initial)
                    && (0..=s.initial).contains(&s.remaining),
            )?;
            consumed += i128::from(s.initial) - i128::from(s.remaining);
        }
        require(consumed == i128::from(m.uses))?;
        if m.uses == 0 {
            require(
                m.revision == 0
                    && m.last_sequence == 0
                    && m.last_life == 0
                    && m.last_equipment_revision == 0,
            )?;
        } else {
            require(m.revision >= m.uses && m.last_life > 0 && !m.stocks.0.is_empty())?;
        }
        let owner = d
            .actors
            .0
            .iter()
            .find(|a| a.entity == m.caster.entity)
            .ok_or_else(invalid)?;
        require(d.scene_member(owner)?.identity == m.caster.identity)?;
        let spell = owner
            .spells
            .histories
            .0
            .iter()
            .find(|s| s.lane == d::Lane::Slow)
            .ok_or_else(invalid)?;
        require(spell.ledger.casts == m.uses)?;
        if let Some(r) = &spell.receipt {
            let u = &r.impact.usage;
            require(
                r.casts == m.uses
                    && u.revision == m.revision
                    && u.sequence == m.last_sequence
                    && u.before.life == m.last_life
                    && u.before.equipment_revision == m.last_equipment_revision,
            )?;
        } else {
            require(m.uses == 0)?;
        }
        for a in &d.actors.0 {
            if let Some(b) = &a.buffs {
                buff(b)?;
                require(b.entity == a.entity && b.revision <= w.revision)?;
            }
            for s in &a.spells.histories.0 {
                if let Some(r) = &s.receipt {
                    if matches!(
                        r.detail,
                        d::Detail::Slow(_)
                            | d::Detail::Haste(_)
                            | d::Detail::GreaterHasteNeutralized(_)
                    ) {
                        receipt(r)?;
                        require(s.lane == r.impact.lane)?;
                    }
                }
            }
        }
        let mut last = None;
        for e in &self.effects.targets.0 {
            let key = match &e.target {
                Target::Player(p) => (0, p.entity),
                Target::Npc(n) => (1, n.serial),
            };
            require(last.is_none_or(|v| v < key) && (e.active.is_some() || e.walk.is_some()))?;
            last = Some(key);
            for g in [&e.active, &e.walk].into_iter().flatten() {
                receipt(&g.receipt)?;
                require(g.caster.identity.kind == d::MemberKind::Human)?;
                let a = d
                    .actors
                    .0
                    .iter()
                    .find(|a| a.entity == g.caster.entity)
                    .ok_or_else(invalid)?;
                require(d.scene_member(a)?.identity == g.caster.identity)?;
                let (elapsed, target) = match &g.receipt.detail {
                    d::Detail::Haste(h) => (h.active_elapsed_nanos, &h.target),
                    d::Detail::Slow(s) => {
                        require(
                            s.missing_effect.is_none()
                                && s.roll <= s.probability
                                && s.equipment_haste_items == 0,
                        )?;
                        (s.active_elapsed_nanos, &s.target)
                    }
                    _ => return Err(invalid()),
                };
                require(
                    elapsed <= w.active_elapsed_nanos
                        && g.receipt.impact.usage.revision <= w.revision
                        && i128::from(g.receipt.loading_elapsed_nanos) - i128::from(elapsed)
                            == i128::from(w.loading_elapsed_nanos)
                                - i128::from(w.active_elapsed_nanos),
                )?;
                require(match (&e.target, target) {
                    (Target::Player(a), StatusTarget::Player(b)) => a == &b.participant,
                    (Target::Npc(a), StatusTarget::Npc(b)) => a == &b.id && a.world == w.identity,
                    _ => false,
                })?;
            }
        }
        if let Some(n) = &self.effects.npcs {
            require(
                n.world == w.identity
                    && d.npcs.as_ref().is_some_and(|s| {
                        s.revision == n.revision
                            && s.elapsed_millis == n.elapsed_millis
                            && s.actors.0.len() == n.actors.0.len()
                    }),
            )?;
            for a in &n.actors.0 {
                npc(&a.speed)?;
                npc(&a.walk_speed)?;
            }
        } else {
            require(d.npcs.is_none())?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> Buffs {
        Buffs {
            entity: 1,
            revision: 0,
            observed_nanos: 0,
            haste_until_nanos: 0,
            brave_until_nanos: 0,
            wisdom_until_nanos: 0,
            brave_kind: 0,
            haste_origin: d::HasteOrigin::None,
            slow_until_nanos: 0,
            slow_revision: 0,
        }
    }
    fn encoded<T: Wire>(v: &T, schema: u16) -> Result<Vec<u8>, ApplicationError> {
        let mut w = Writer::count();
        w.schema = schema;
        w.write(v)?;
        let mut out = Writer::allocated(w.pos)?;
        out.schema = schema;
        out.write(v)?;
        Ok(out.bytes.unwrap())
    }
    #[test]
    fn neutralized_greater_haste_requires_its_own_cost_range_not_slow_costs() {
        let before = changed(&base(), 0, 2, true, false).unwrap();
        let after = changed(&before, SECOND, 4, false, true).unwrap();
        let build = |cost: i32| d::SpellReceipt {
            impact: d::Impact {
                lane: d::Lane::GreaterHaste,
                usage: d::Cast {
                    revision: 4,
                    sequence: 1,
                    cost,
                    mp_after: 100 - i64::from(cost),
                    before: d::Anchor {
                        equipment_revision: 0,
                        life: 1,
                        cell: d::Cell { x: 1, y: 1 },
                        vitals: d::Vitals {
                            maximum_hp: 100,
                            maximum_mp: 100,
                            hp: 100,
                            mp: 100,
                        },
                    },
                },
                targets: List(vec![]),
            },
            casts: 1,
            spent: i64::from(cost),
            loading_elapsed_nanos: SECOND,
            detail: d::Detail::GreaterHasteNeutralized(Neutralized {
                active_elapsed_nanos: SECOND,
                before: before.clone(),
                after: after.clone(),
            }),
        };
        for cost in [25, 26, 59, 60] {
            receipt(&build(cost)).unwrap();
        }
        for cost in [15, 20, 24, 61] {
            assert!(receipt(&build(cost)).is_err());
        }
        let mut r = build(25);
        r.casts = 2;
        r.spent = 49;
        r.impact.usage.sequence = 2;
        assert!(receipt(&r).is_err());
        r.spent = 50;
        receipt(&r).unwrap();
    }
    #[test]
    fn speed_descriptor_is_explicit_and_does_not_open_old_canonical_admission() {
        use crate::modern_lito::world_protocol::{MatchDescriptor as F, MatchRuleset};
        let mut d = F {
            ruleset: MatchRuleset::PvpSkirmish,
            features: F::COMBAT | F::SPELLS | F::ITEMS | F::WEIGHT | F::TIMED | F::SLOW_CANDIDATE,
            preparation_timeout_millis: 2000,
            load_timeout_millis: 2000,
            countdown_nanos: 0,
            active_timeout_millis: 60_000,
        };
        d.validate_speed_candidate(1).unwrap();
        assert!(d.validate(1).is_err());
        assert!(d.validate_canonical(1).is_err());
        assert!(d.validate_haste_candidate(1).is_err());
        d.features &= !F::SLOW_CANDIDATE;
        assert!(d.validate_speed_candidate(1).is_err());
        d.validate_canonical(1).unwrap();
    }
    #[test]
    fn full_buff_roundtrip_preserves_expired_slow_history_and_old_layouts() {
        let original = base();
        for schema in 1..=5 {
            let bytes = encoded(&original, schema).unwrap();
            assert_eq!(bytes.len(), if matches!(schema, 3 | 5) { 53 } else { 52 });
            let mut r = Reader::new(&bytes);
            r.schema = schema;
            assert_eq!(r.read::<Buffs>().unwrap(), original);
            r.finish().unwrap();
        }
        let mut b = changed(&original, 0, 2, true, false).unwrap();
        assert_eq!(b.slow_until_nanos, SLOW);
        assert_eq!(encoded(&b, SCHEMA).unwrap().len(), 69);
        for live in [true, false] {
            if !live {
                b.slow_until_nanos = 0;
                b.observed_nanos = SLOW;
                b.revision = 3;
            }
            buff(&b).unwrap();
            for schema in 1..=5 {
                assert!(encoded(&b, schema).is_err());
            }
            let bytes = encoded(&b, SCHEMA).unwrap();
            let mut r = Reader::new(&bytes);
            r.schema = SCHEMA;
            assert_eq!(r.read::<Buffs>().unwrap(), b);
            r.finish().unwrap();
            let mut bad = b.clone();
            bad.slow_revision = bad.revision + 1;
            assert!(buff(&bad).is_err());
        }
    }
    #[test]
    fn legacy_context_rejects_materials_even_without_a_wire_decode() {
        let mut d = crate::modern_lito::battle_payload::data_tests::fixture();
        d.check_context(&d.personal.player, &d.world.identity)
            .unwrap();
        d.personal.materials = Some(Materials {
            world: d.world.identity.clone(),
            player: d.personal.player.clone(),
            caster: Participant {
                entity: d.personal.entity,
                identity: d.room.members.0[0].identity.clone(),
            },
            revision: 0,
            uses: 0,
            last_sequence: 0,
            last_life: 0,
            last_equipment_revision: 0,
            stocks: List(vec![]),
        });
        assert!(d
            .check_context(&d.personal.player, &d.world.identity)
            .is_err());
    }
    #[test]
    fn both_spell_hastes_neutralize_slow_and_recast_is_required() {
        for greater in [false, true] {
            let slow = changed(&base(), 0, 2, true, false).unwrap();
            let neutral = changed(&slow, SECOND, 3, false, greater).unwrap();
            assert_eq!(
                (
                    neutral.haste_until_nanos,
                    neutral.slow_until_nanos,
                    neutral.slow_revision
                ),
                (0, 0, 2)
            );
            let haste = changed(&neutral, 2 * SECOND, 4, false, greater).unwrap();
            assert_eq!(
                haste.haste_until_nanos,
                2 * SECOND + if greater { 2400 * SECOND } else { HASTE }
            );
            let neutral = changed(&haste, 3 * SECOND, 5, true, false).unwrap();
            assert_eq!(
                (
                    neutral.haste_until_nanos,
                    neutral.slow_until_nanos,
                    neutral.slow_revision
                ),
                (0, 0, 5)
            );
            assert!(changed(&neutral, 2 * SECOND, 6, true, false).is_err());
            assert!(changed(&neutral, 4 * SECOND, 5, true, false).is_err());
        }
    }
    #[test]
    fn npc_speed_is_lossless_and_cannot_be_encoded_as_haste_only() {
        let initial = Speed {
            revision: 0,
            observed_millis: 0,
            haste_until_millis: 0,
            slow_until_millis: 0,
            slow_revision: 0,
        };
        let slow = npc_changed(&initial, 150, 3, true).unwrap();
        assert_eq!(slow.slow_until_millis, 64_150);
        let neutral = npc_changed(&slow, 2000, 4, false).unwrap();
        assert_eq!(
            (
                neutral.haste_until_millis,
                neutral.slow_until_millis,
                neutral.slow_revision
            ),
            (0, 0, 3)
        );
        for value in [slow, neutral] {
            for schema in 1..=5 {
                assert!(encoded(&value, schema).is_err());
            }
            let bytes = encoded(&value, SCHEMA).unwrap();
            assert_eq!(bytes.len(), 40);
            let mut r = Reader::new(&bytes);
            r.schema = SCHEMA;
            assert_eq!(r.read::<Speed>().unwrap(), value);
            r.finish().unwrap();
        }
    }
    #[test]
    fn only_candidate_schema_accepts_new_lane_and_material_stock_is_bounded() {
        for schema in 1..=5 {
            assert!(encoded(&d::Lane::Slow, schema).is_err());
            let mut r = Reader::new(&[10]);
            r.schema = schema;
            assert!(r.read::<d::Lane>().is_err());
        }
        assert_eq!(encoded(&d::Lane::Slow, SCHEMA).unwrap(), [10]);
        let mut r = Reader::new(&[0, 0, 0, 2]);
        r.schema = SCHEMA;
        assert!(r.read::<List<Stock, 1>>().is_err());
        let stocks = List::<Stock, 1>(vec![
            Stock {
                item: 40318,
                unit_weight: i64::MAX,
                initial: 1,
                remaining: 0
            };
            2
        ]);
        assert!(encoded(&stocks, SCHEMA).is_err());
    }
    #[test]
    fn missing_native_effect_is_an_exact_pair_not_a_generic_immune_flag() {
        for (ids, skill) in [
            (vec![45912, 45913, 45914, 45915], 1013),
            (vec![45916], 1014),
            (vec![45941], 1015),
            (vec![45752, 45753], 4005),
            (vec![45675, 81082, 45625, 45674, 45685], 4006),
            (vec![87000], 4007),
            (vec![45020], 4008),
            (vec![99019], 4025),
        ] {
            for id in ids {
                missing(&MissingEffect {
                    npc_template: id,
                    required_skill: skill,
                })
                .unwrap();
                assert!(missing(&MissingEffect {
                    npc_template: id,
                    required_skill: skill + 1
                })
                .is_err());
            }
        }
        assert!(missing(&MissingEffect {
            npc_template: 1,
            required_skill: 0
        })
        .is_err());
    }
}
