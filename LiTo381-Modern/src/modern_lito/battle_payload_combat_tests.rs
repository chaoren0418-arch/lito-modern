use super::{data::*, *};
use crate::modern_lito::{use_history, vital_balance};

fn reject(
    d: &UnvalidatedData,
    label: &str,
    case: &str,
    edit: impl FnOnce(&mut UnvalidatedData),
) -> usize {
    let mut changed = d.clone();
    edit(&mut changed);
    // Deliberately preserve framing, all typed fields, owner and match context.
    let changed = decode(&encode(&changed).expect("syntactically valid altered payload")).unwrap();
    changed
        .check_context(&d.personal.player, &d.world.identity)
        .unwrap_or_else(|_| panic!("{label}/{case}: mutation must reach semantic validation"));
    assert!(
        changed
            .check_combat_values(&d.personal.player, &d.world.identity)
            .is_err(),
        "{label}/{case}: accepted corrupt combat values"
    );
    1
}
pub(super) fn real_payload_mutations(label: &str, d: &UnvalidatedData) -> usize {
    let mut n = 0;
    for (i, a) in d.actors.0.iter().take(2).enumerate() {
        if a.buffs.is_some() {
            // In-memory callers must not bypass legacy rejection by skipping
            // the codec; expired Slow still carries a nonzero history marker.
            let mut bad = d.clone();
            bad.actors.0[i].buffs.as_mut().unwrap().slow_revision = 1;
            assert!(bad
                .check_combat_values(&d.personal.player, &d.world.identity)
                .is_err());
            n += 1;
        }
        n += ordered_spell_mutations(d, label, i);
        n += reject(d, label, "request sequence", |d| {
            d.actors.0[i].spells.sequence = -1
        });
        n += reject(d, label, "lawful accounting", |d| {
            d.actors.0[i].spells.lawful ^= 1
        });
        n += reject(d, label, "cast interval", |d| {
            d.actors.0[i].cast_interval_nanos = -1
        });
        if a.resources.is_some() {
            n += reject(d, label, "negative flow", |d| {
                d.actors.0[i].resources.as_mut().unwrap().flows.damaged = -1
            });
            n += reject(d, label, "overflow flow", |d| {
                d.actors.0[i].resources.as_mut().unwrap().flows.hp_gain = i64::MAX
            });
            n += reject(d, label, "public HP diverges", |d| {
                d.actors.0[i].vital.as_mut().unwrap().values.hp ^= 1
            });
            n += reject(d, label, "weapon type", |d| {
                d.actors.0[i].vital.as_mut().unwrap().weapon_type = 19
            });
            n += reject(d, label, "negative attack", |d| {
                d.actors.0[i].vital.as_mut().unwrap().attack_sequence = -1
            });
            n += reject(d, label, "future hurt", |d| {
                d.actors.0[i].vital.as_mut().unwrap().hurt_revision = d.world.revision + 1
            });
            n += reject(d, label, "attack cooldown", |d| {
                d.actors.0[i].vital.as_mut().unwrap().attack_interval_nanos = 0
            });
        }
        if a.buffs.is_some() {
            n += reject(d, label, "buff owner", |d| {
                d.actors.0[i].buffs.as_mut().unwrap().entity = 0
            });
            n += reject(d, label, "buff kind", |d| {
                d.actors.0[i].buffs.as_mut().unwrap().brave_kind = 256
            });
        }
        if a.regeneration.is_some() {
            n += reject(d, label, "regen flow", |d| {
                d.actors.0[i].regeneration.as_mut().unwrap().hp.gain = -1
            });
            n += reject(d, label, "future regen", |d| {
                d.actors.0[i].regeneration.as_mut().unwrap().mp.revision = d.world.revision + 1
            });
        }
        if !a.spells.histories.0.is_empty() {
            n += reject(d, label, "missing lane", |d| {
                d.actors.0[i].spells.histories.0.pop();
            });
            n += reject(d, label, "negative spend", |d| {
                d.actors.0[i].spells.histories.0[0].ledger.spent = -1
            });
            n += reject(d, label, "future profile", |d| {
                d.actors.0[i].spells.histories.0[0].ledger.profile_revision = d.world.revision + 1
            });
        }
        for (j, s) in a.spells.histories.0.iter().enumerate() {
            let Some(r) = &s.receipt else { continue };
            n += reject(d, label, "missing receipt", |d| {
                d.actors.0[i].spells.histories.0[j].receipt = None
            });
            n += reject(d, label, "receipt count", |d| {
                d.actors.0[i].spells.histories.0[j]
                    .receipt
                    .as_mut()
                    .unwrap()
                    .casts = 0
            });
            n += reject(d, label, "future receipt clock", |d| {
                d.actors.0[i].spells.histories.0[j]
                    .receipt
                    .as_mut()
                    .unwrap()
                    .loading_elapsed_nanos = d.world.loading_elapsed_nanos + 1
            });
            n += reject(d, label, "MP cast delta", |d| {
                d.actors.0[i].spells.histories.0[j]
                    .receipt
                    .as_mut()
                    .unwrap()
                    .impact
                    .usage
                    .mp_after ^= 1
            });
            if !r.impact.targets.0.is_empty() {
                n += reject(d, label, "effect outside manifest", |d| {
                    d.actors.0[i].spells.histories.0[j]
                        .receipt
                        .as_mut()
                        .unwrap()
                        .impact
                        .targets
                        .0[0]
                        .cell
                        .x = i32::MIN
                });
                n += reject(d, label, "effect arithmetic", |d| {
                    d.actors.0[i].spells.histories.0[j]
                        .receipt
                        .as_mut()
                        .unwrap()
                        .impact
                        .targets
                        .0[0]
                        .hp_after ^= 1
                });
                n += reject(d, label, "effect life", |d| {
                    d.actors.0[i].spells.histories.0[j]
                        .receipt
                        .as_mut()
                        .unwrap()
                        .impact
                        .targets
                        .0[0]
                        .life = 0
                });
                n += reject(d, label, "effect kind", |d| {
                    let t = &mut d.actors.0[i].spells.histories.0[j]
                        .receipt
                        .as_mut()
                        .unwrap()
                        .impact
                        .targets
                        .0[0];
                    t.kind = if t.kind == EffectKind::Damage {
                        EffectKind::Heal
                    } else {
                        EffectKind::Damage
                    };
                });
            }
            n += reject(d, label, "skill detail", |d| {
                let r = d.actors.0[i].spells.histories.0[j]
                    .receipt
                    .as_mut()
                    .unwrap();
                r.detail = if r.impact.lane == Lane::Healing {
                    Detail::Arrow { critical: false }
                } else {
                    Detail::Ordinary
                };
            });
            match &r.detail {
                Detail::Teleport { .. } => {
                    n += reject(d, label, "teleport outside manifest", |d| {
                        if let Detail::Teleport { destination } =
                            &mut d.actors.0[i].spells.histories.0[j]
                                .receipt
                                .as_mut()
                                .unwrap()
                                .detail
                        {
                            destination.x = i32::MAX;
                        }
                    });
                    n += reject(d, label, "stationary teleport", |d| {
                        let r = d.actors.0[i].spells.histories.0[j]
                            .receipt
                            .as_mut()
                            .unwrap();
                        r.detail = Detail::Teleport {
                            destination: r.impact.usage.before.cell.clone(),
                        };
                    })
                }
                Detail::Disintegrate { .. } => {
                    n += reject(d, label, "lawful overflow", |d| {
                        let r = d.actors.0[i].spells.histories.0[j]
                            .receipt
                            .as_mut()
                            .unwrap();
                        if let Detail::Disintegrate {
                            lawful_before,
                            lawful_after,
                            ..
                        } = &mut r.detail
                        {
                            *lawful_before = i64::MAX;
                            *lawful_after = i64::MIN;
                        }
                    })
                }
                Detail::FireStorm { damage_loss_after } if !damage_loss_after.0.is_empty() => {
                    n += reject(d, label, "fire cumulative loss", |d| {
                        if let Detail::FireStorm { damage_loss_after } =
                            &mut d.actors.0[i].spells.histories.0[j]
                                .receipt
                                .as_mut()
                                .unwrap()
                                .detail
                        {
                            damage_loss_after.0[0] = -1;
                        }
                    })
                }
                Detail::NpcFireStorm { .. } => {
                    n += reject(d, label, "structure kind", |d| {
                        if let Detail::NpcFireStorm { quest_kind } =
                            &mut d.actors.0[i].spells.histories.0[j]
                                .receipt
                                .as_mut()
                                .unwrap()
                                .detail
                        {
                            *quest_kind = if quest_kind.is_some() {
                                None
                            } else {
                                Some(QuestKind::FinalSymbol)
                            };
                        }
                    })
                }
                _ => {}
            }
        }
        if a.spells.structure.is_some() {
            n += reject(d, label, "structure cast count", |d| {
                d.actors.0[i].spells.structure.as_mut().unwrap().casts = 0
            });
        }
    }
    if d.personal.history.is_some() {
        n += reject(d, label, "runtime birth", |d| {
            d.personal.spell_birth_revision = 0
        });
        n += reject(d, label, "food unit", |d| {
            d.personal.equipment.as_mut().unwrap().saved.food225 = 226
        });
        n += reject(d, label, "private effective maximum", |d| {
            d.personal.equipment.as_mut().unwrap().effective.maximum_hp ^= 1
        });
    }
    if let Some(s) = &d.personal.supplies {
        n += reject(d, label, "supply sequence", |d| {
            d.personal.supplies.as_mut().unwrap().request_sequence = -1
        });
        n += reject(d, label, "supply effect", |d| {
            d.personal.supplies.as_mut().unwrap().effect = -1
        });
        n += reject(d, label, "supply delay", |d| {
            d.personal.supplies.as_mut().unwrap().delay_millis = -1
        });
        if !s.stocks.0.is_empty() {
            n += reject(d, label, "stock negative", |d| {
                d.personal.supplies.as_mut().unwrap().stocks.0[0].remaining = -1
            });
        }
    }
    n
}

fn ordered_spell_mutations(d: &UnvalidatedData, label: &str, index: usize) -> usize {
    let a = &d.actors.0[index];
    let mut receipts = a
        .spells
        .histories
        .0
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.receipt.as_ref().map(|r| (i, r)))
        .collect::<Vec<_>>();
    receipts.sort_by_key(|(_, r)| r.impact.usage.sequence);
    if receipts.len() < 2 || d.world.loading_elapsed_nanos == 0 {
        return 0;
    }
    let earlier = receipts[0].0;
    let later = receipts[1].0;
    for field in 0..3 {
        let change = |bad: &mut UnvalidatedData| {
            let spells = &mut bad.actors.0[index].spells.histories.0;
            let before = spells[earlier].last.as_ref().unwrap().clone();
            if field == 0 {
                spells[later].ledger.last_cast_sequence = before.sequence;
                spells[later].last.as_mut().unwrap().sequence = before.sequence;
                spells[later]
                    .receipt
                    .as_mut()
                    .unwrap()
                    .impact
                    .usage
                    .sequence = before.sequence;
            } else if field == 1 {
                spells[later].ledger.last_cast_revision = before.revision;
                spells[later].last.as_mut().unwrap().revision = before.revision;
                spells[later]
                    .receipt
                    .as_mut()
                    .unwrap()
                    .impact
                    .usage
                    .revision = before.revision;
            } else {
                spells[earlier]
                    .receipt
                    .as_mut()
                    .unwrap()
                    .loading_elapsed_nanos = bad.world.loading_elapsed_nanos;
                spells[later]
                    .receipt
                    .as_mut()
                    .unwrap()
                    .loading_elapsed_nanos = 0;
            }
            // Each individual history remains legal; only the combined event
            // stream is inconsistent. Do not rely on private duplicate fields.
            assert!(spells.iter().all(|s| use_history::Spell::from(s).valid()));
        };
        let mut bad = d.clone();
        change(&mut bad);
        assert!(
            bad.spell_values(&bad.actors.0[index]).is_err(),
            "{label}: duplicate/out-of-order public events"
        );
        reject(d, label, "cross-lane event order", change);
    }
    3
}

#[test]
fn scene_is_still_typed_unadmitted_data_and_rejects_unbacked_spell_values() {
    let mut d = data_tests::fixture();
    d.check_combat_values(&d.personal.player, &d.world.identity)
        .unwrap();
    d.actors.0[0].spells.request_revision = 1;
    assert!(d
        .check_combat_values(&d.personal.player, &d.world.identity)
        .is_err());
}
#[test]
fn full_width_balances_allow_i64_max_without_narrowing_or_overflow() {
    let mut r = Resources {
        revision: 1,
        equipment_revision: 0,
        life: 0,
        birth_hp: i64::MAX,
        birth_mp: i64::MAX,
        vitals: Vitals {
            hp: i64::MAX,
            maximum_hp: i64::MAX,
            mp: i64::MAX,
            maximum_mp: i64::MAX,
        },
        flows: Flows {
            hp_gain: 0,
            hp_loss: 0,
            restored: 0,
            damaged: 0,
            hp_granted: 0,
            mp_gain: 0,
            mp_loss: 0,
            spent: 0,
            mp_granted: 0,
            hp_clipped: 0,
            mp_clipped: 0,
        },
    };
    assert!(vital_balance::State::from(&r).valid());
    r.flows.hp_gain = i64::MAX;
    r.flows.hp_loss = i64::MAX;
    assert!(
        vital_balance::State::from(&r).valid(),
        "intermediate sum uses i128, not wrapping i64"
    );
    r.flows.hp_loss -= 1;
    assert!(!vital_balance::State::from(&r).valid());
    r.flows.hp_loss = i64::MAX;
    r.flows.mp_loss = 1;
    r.vitals.mp -= 1;
    assert!(vital_balance::State::from(&r).valid());
    r.life = i64::MAX;
    assert!(!vital_balance::State::from(&r).valid());
}
#[test]
fn free_casts_and_old_equipment_receipts_remain_legal() {
    let mut s = Spell {
        lane: Lane::Healing,
        ledger: Ledger {
            equipment_revision: 0,
            profile_revision: 1,
            cost: 0,
            prior_casts: 0,
            prior_spent: 0,
            casts: 1,
            spent: 0,
            last_cast_revision: 2,
            last_cast_sequence: 1,
        },
        last: Some(Cast {
            revision: 2,
            sequence: 1,
            before: Anchor {
                equipment_revision: 0,
                life: 1,
                vitals: Vitals {
                    hp: 1,
                    maximum_hp: i64::MAX,
                    mp: 0,
                    maximum_mp: 0,
                },
                cell: Cell { x: 32768, y: 32768 },
            },
            cost: 0,
            mp_after: 0,
        }),
        receipt: None,
    };
    assert!(use_history::Spell::from(&s).valid());
    // Repricing does not rewrite the historical zero-MP cast.
    s.ledger.equipment_revision = 1;
    s.ledger.profile_revision = 3;
    s.ledger.prior_casts = 1;
    s.ledger.cost = 48;
    assert!(use_history::Spell::from(&s).valid());
    s.last.as_mut().unwrap().before.equipment_revision = 1;
    assert!(!use_history::Spell::from(&s).valid());
}
#[test]
fn target_clamping_and_life_domains_are_not_truncated() {
    let mut e = Effect {
        target: Target::Honor { entity: 3, team: 0 },
        kind: EffectKind::Damage,
        life: 1,
        equipment_revision: 0,
        cell: Cell { x: 32768, y: 32768 },
        hp: i64::MAX,
        maximum_hp: i64::MAX,
        raw: i64::MAX,
        applied: i64::MAX,
        hp_after: 0,
    };
    assert!(values::effect(&e).unwrap().valid());
    e.life = 2;
    assert!(!values::effect(&e).unwrap().valid());
    e.life = 1;
    e.hp = 1;
    e.applied = 1;
    assert!(values::effect(&e).unwrap().valid());
    e.kind = EffectKind::Heal;
    e.raw = i64::MAX;
    e.applied = i64::MAX - 1;
    e.hp_after = i64::MAX;
    assert!(values::effect(&e).unwrap().valid());
    e.applied = i64::MAX;
    assert!(!values::effect(&e).unwrap().valid());
    e.target = Target::Npc {
        scope: WorldId {
            room: 1,
            match_id: 2,
            generation: 3,
        },
        serial: -1,
    };
    assert!(values::effect(&e).is_none());
}
