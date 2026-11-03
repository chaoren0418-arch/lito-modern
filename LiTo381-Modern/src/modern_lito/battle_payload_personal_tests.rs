use super::{data::*, *};

fn reject(
    d: &UnvalidatedData,
    label: &str,
    case: &str,
    edit: impl FnOnce(&mut UnvalidatedData),
) -> usize {
    let mut changed = d.clone();
    edit(&mut changed);
    let changed = decode(&encode(&changed).expect("syntax retained")).unwrap();
    changed
        .check_combat_values(&d.personal.player, &d.world.identity)
        .unwrap_or_else(|_| panic!("{label}/{case}: must reach the new private checks"));
    assert!(
        changed
            .personal_view(&d.personal.player, &d.world.identity)
            .is_err(),
        "{label}/{case}: corrupt personal data accepted"
    );
    1
}

pub(super) fn real_payload_mutations(label: &str, d: &UnvalidatedData) -> usize {
    let mut count = 0;
    if let Some(e) = &d.personal.equipment {
        for case in 0..9 {
            count += reject(d, label, "equipment scalars", |d| {
                let e = d.personal.equipment.as_mut().unwrap();
                match case {
                    0 => e.saved.saved_hp = -1,
                    1 => e.saved.saved_mp = -1,
                    2 => e.saved.strength = -1,
                    3 => e.effective.initial_hp = -1,
                    4 => e.effective.charisma = -1,
                    5 => e.evaluated_at_epoch_millis = -1,
                    6 => e.valid_until_epoch_millis = -2,
                    7 => e.valid_until_epoch_millis = e.evaluated_at_epoch_millis,
                    _ => e.saved.experience = -1,
                }
            });
        }
        if !e.inventory.0.is_empty() {
            for case in 0..10 {
                count += reject(d, label, "inventory instance", |d| {
                    let i = &mut d.personal.equipment.as_mut().unwrap().inventory.0[0];
                    match case {
                        0 => i.object = 0,
                        1 => i.catalog = -1,
                        2 => i.quantity = 0,
                        3 => i.unit_weight = -1,
                        4 => i.durability = -1,
                        5 => i.blessing = 256,
                        6 => i.charge_count = -1,
                        7 => i.inventory_graphic = -1,
                        8 => i.ground_graphic = -1,
                        _ => i.name = Text(" ".into()),
                    }
                });
            }
            count += reject(d, label, "duplicate instance", |d| {
                let e = d.personal.equipment.as_mut().unwrap();
                e.inventory.0.push(e.inventory.0[0].clone());
            });
            count += reject(d, label, "missing flags", |d| {
                d.personal.equipment.as_mut().unwrap().equipped.0.pop();
            });
            count += reject(d, label, "missing lifetimes", |d| {
                d.personal.equipment.as_mut().unwrap().lifetimes.0.pop();
            });
            count += reject(d, label, "foreign equip flag", |d| {
                d.personal.equipment.as_mut().unwrap().equipped.0[0].object = 0;
            });
            count += reject(d, label, "foreign lifetime", |d| {
                d.personal.equipment.as_mut().unwrap().lifetimes.0[0].object = 0;
            });
            for case in 0..4 {
                count += reject(d, label, "lifetime", |d| {
                    let e = d.personal.equipment.as_mut().unwrap();
                    let l = &mut e.lifetimes.0[0];
                    match case {
                        0 => l.expires_at_millis = -2,
                        1 => {
                            l.state = LifetimeState::Permanent;
                            l.expires_at_millis = 0;
                        }
                        2 => {
                            l.state = LifetimeState::Active;
                            l.expires_at_millis = e.evaluated_at_epoch_millis;
                        }
                        _ => {
                            l.state = LifetimeState::Expired;
                            l.expires_at_millis = e.evaluated_at_epoch_millis + 1;
                        }
                    }
                });
            }
        }
    }
    if !d.personal.skills.0.is_empty() {
        for case in 0..6 {
            count += reject(d, label, "learned skill catalog", |d| {
                let s = &mut d.personal.skills.0[0];
                match case {
                    0 => s.id = 0,
                    1 => s.mp_cost = -1,
                    2 => s.reuse_millis = -1,
                    3 => s.name = Text(" ".into()),
                    4 => {
                        s.support = if s.id == 5 {
                            Support::LightArrow
                        } else {
                            Support::Teleport
                        }
                    }
                    _ => s.id = 221,
                }
            });
        }
        count += reject(d, label, "duplicate skill", |d| {
            d.personal.skills.0.push(d.personal.skills.0[0].clone());
        });
    }
    if let Some(w) = &d.personal.weight {
        if d.personal.equipment.as_ref().is_some_and(|e| {
            e.inventory
                .0
                .iter()
                .zip(&e.lifetimes.0)
                .any(|(i, l)| i.kind != ItemKind::Etc && l.state != LifetimeState::Expired)
        }) {
            count += reject(d, label, "source gear multiplication overflow", |d| {
                let e = d.personal.equipment.as_mut().unwrap();
                let (i, _) = e
                    .inventory
                    .0
                    .iter_mut()
                    .zip(&e.lifetimes.0)
                    .find(|(i, l)| i.kind != ItemKind::Etc && l.state != LifetimeState::Expired)
                    .unwrap();
                i.quantity = i64::MAX;
                i.unit_weight = i64::MAX;
            });
        }
        for case in 0..6 {
            count += reject(d, label, "weight", |d| {
                let w = d.personal.weight.as_mut().unwrap();
                match case {
                    0 => w.total_weight ^= 1,
                    1 => w.maximum_weight_bits = f64::NAN.to_bits() as i64,
                    2 => w.maximum_weight_bits = 0,
                    3 => w.stage = 241,
                    4 => w.source_gear_weight = -1,
                    _ => w.supplies_weight = -1,
                }
            });
        }
        if !w.units.0.is_empty() {
            count += reject(d, label, "missing stock weight", |d| {
                d.personal.weight.as_mut().unwrap().units.0.pop();
            });
            count += reject(d, label, "negative stock weight", |d| {
                d.personal.weight.as_mut().unwrap().units.0[0].unit_weight = -1;
            });
        }
    }
    if let Some(a) = &d.personal.ammo {
        for case in 0..6 {
            count += reject(d, label, "ammunition", |d| {
                let a = d.personal.ammo.as_mut().unwrap();
                match case {
                    0 => a.entity = 0,
                    1 => a.request_sequence = -1,
                    2 => a.revision = d.world.revision + 1,
                    3 => a.shot_count = -1,
                    4 => a.selected_item = -1,
                    _ => a.attack_sequence = -1,
                }
            });
        }
        if !a.stocks.0.is_empty() {
            for case in 0..6 {
                count += reject(d, label, "arrow stock", |d| {
                    let s = &mut d.personal.ammo.as_mut().unwrap().stocks.0[0];
                    match case {
                        0 => s.remaining = s.initial + 1,
                        1 => s.small = 0,
                        2 => s.consume = false,
                        3 => s.raw_arrow_sprite = 77,
                        4 => s.material = 23,
                        _ => s.blessing = 1,
                    }
                });
            }
        }
    }
    if d.personal.regen_clock.is_some() {
        for case in 0..3 {
            count += reject(d, label, "private regen clock", |d| {
                let c = d.personal.regen_clock.as_mut().unwrap();
                match case {
                    0 => c.next_second_elapsed_nanos = 1,
                    1 => c.hp_counter = 120,
                    _ => c.mp_counter = 64,
                }
            });
        }
    }
    count
}

pub(super) fn real_payload_positive_controls(label: &str, d: &UnvalidatedData) {
    let original = d
        .personal_view(&d.personal.player, &d.world.identity)
        .unwrap();
    let mut changed = d.clone();
    if let Some(e) = &mut changed.personal.equipment {
        // Database saved resources are not current match HP/MP, nor necessarily
        // bounded by the source base maxima. Preserve their full signed width.
        e.saved.saved_hp = i64::MAX;
        e.saved.saved_mp = i64::MAX;
        for item in &mut e.inventory.0 {
            // A match-local loadout can differ from the imported equipped bit.
            item.equipped = !item.equipped;
        }
        let mut decoded = decode(&encode(&changed).unwrap()).unwrap();
        let view = decoded
            .personal_view(&d.personal.player, &d.world.identity)
            .unwrap_or_else(|_| panic!("{label}: wide saved values or runtime overrides rejected"));
        let source = view.source().unwrap();
        assert_eq!(source.stats.saved_hp, i64::MAX);
        assert_eq!(source.stats.saved_mp, i64::MAX);
        assert_eq!(view.effective(), original.effective());
        assert_eq!(view.weight(), original.weight());
        assert_eq!(view.ammunition(), original.ammunition());
        for item in &source.items {
            assert_eq!(
                view.equipped(item.object_id as i64),
                original.equipped(item.object_id as i64)
            );
            assert_eq!(
                view.lifetime(item.object_id as i64),
                original.lifetime(item.object_id as i64)
            );
        }
        // Checked views own their data, not a mutable alias into decoded input.
        let source_count = source.items.len();
        decoded
            .personal
            .equipment
            .as_mut()
            .unwrap()
            .inventory
            .0
            .clear();
        decoded.personal.skills.0.clear();
        assert_eq!(
            original,
            d.personal_view(&d.personal.player, &d.world.identity)
                .unwrap()
        );
        assert_eq!(view.source().unwrap().items.len(), source_count);
        assert_eq!(view.skills(), d.personal.skills.0);
        if let Some(template) = d.personal.equipment.as_ref().unwrap().inventory.0.first() {
            // Expired source gear and ETC/source currency remain visible in
            // the inventory, but neither is added to match-carried gear weight.
            let mut retained = d.clone();
            let e = retained.personal.equipment.as_mut().unwrap();
            assert!(e.inventory.0.last().unwrap().object < i64::MAX - 1);
            for (id, kind, state, expiry) in [
                (
                    i64::MAX - 1,
                    ItemKind::Armor,
                    LifetimeState::Expired,
                    e.evaluated_at_epoch_millis,
                ),
                (i64::MAX, ItemKind::Etc, LifetimeState::Permanent, -1),
            ] {
                let mut item = template.clone();
                item.object = id;
                item.kind = kind;
                item.quantity = i64::MAX;
                item.unit_weight = i64::MAX;
                item.equipped = true;
                e.inventory.0.push(item);
                e.equipped.0.push(Equipped {
                    object: id,
                    equipped: false,
                });
                e.lifetimes.0.push(Lifetime {
                    object: id,
                    state,
                    expires_at_millis: expiry,
                });
            }
            let retained = decode(&encode(&retained).unwrap()).unwrap();
            let retained = retained
                .personal_view(&d.personal.player, &d.world.identity)
                .unwrap();
            assert_eq!(retained.source().unwrap().items.len(), source_count + 2);
            assert_eq!(retained.weight(), original.weight());
            assert_eq!(retained.supplies(), original.supplies());
            assert_eq!(retained.ammunition(), original.ammunition());
            assert_eq!(retained.equipped(i64::MAX - 1), Some(false));
        }
    }
    if label == "disintegrate" {
        assert!(!original.skills().iter().any(|s| s.id == 77));
        let actor = d
            .actors
            .0
            .iter()
            .find(|a| a.entity == d.personal.entity)
            .unwrap();
        assert!(actor
            .spells
            .histories
            .0
            .iter()
            .any(|s| s.lane == Lane::Disintegrate && s.receipt.is_some()));
        // In particular, the private UI view has not synthesized skill77 from
        // successful history authorized by the engine's independent catalog.
        assert_eq!(original.skills(), d.personal.skills.0);
    }
}

#[test]
fn scene_personal_is_bound_empty_not_a_fabricated_inventory() {
    let d = data_tests::fixture();
    let v = d
        .personal_view(&d.personal.player, &d.world.identity)
        .unwrap();
    assert_eq!(v.player(), &d.personal.player);
    assert_eq!(v.world(), &d.world.identity);
    assert_eq!(v.revision(), d.world.revision);
    assert!(v.source().is_none());
    assert!(v.effective().is_none());
    assert!(v.weight().is_none());
    assert!(v.ammunition().is_none());
    assert!(v.equipped(1).is_none());
    assert_eq!(format!("{v:?}"), "CheckedBattlePersonal[redacted]");
    let mut changed = d.clone();
    changed.personal.skills.0.push(Skill {
        id: 5,
        name: Text("Teleport".into()),
        mp_cost: 10,
        reuse_millis: 10,
        support: Support::Teleport,
    });
    assert!(changed
        .personal_view(&d.personal.player, &d.world.identity)
        .is_err());
    let mut p = d.personal.player.clone();
    p.epoch ^= 1;
    assert!(d.personal_view(&p, &d.world.identity).is_err());
}
