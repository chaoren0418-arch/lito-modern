use super::*;
use crate::modern_lito::{
    item_protocol as item, spell_protocol as spell, weight_protocol as weight,
};

const SECOND: i64 = 1_000_000_000;
const JAVA_GOLDEN: &str = concat!(
    "0001000000000000",
    "000000000000000100000000000000020000000000000003",
    "000000000000000a0000000000000014000000000000001e0300000000000000"
);

/// Pure fixture only: full confirmed ownership, zero imported effects, a wizard
/// and an elf. Production must obtain these values from the authenticated server.
pub(crate) fn initial() -> combat::Reply {
    initial_for_class(3)
}
pub(crate) fn initial_for_class(class: u16) -> combat::Reply {
    let mut reply = item::tests::initial();
    let frame = reply.frame.as_mut().unwrap();
    frame.world.room.members[0].class = class;
    frame.world.room.members[1].class = 2;
    frame.world.casts = frame
        .world
        .actors
        .iter()
        .map(|a| spell::CastState::initial(a.entity_id, 1200))
        .collect();
    for vital in &mut frame.vitals {
        vital.mp = 100;
        vital.max_mp = 100;
        vital.attack_interval_nanos = 700_000_000;
    }
    for actor in &mut frame.world.actors {
        actor.step_interval_nanos = 200_000_000;
    }
    frame.items[0].stocks = [40010, 40011, 40012, 40013, 40014, 40016, 40068]
        .into_iter()
        .map(|item_id| item::Stock {
            item_id,
            initial: 4,
            remaining: 4,
        })
        .collect();
    frame.timed = frame
        .world
        .actors
        .iter()
        .map(|a| State::initial(a.entity_id))
        .collect();
    refresh_weight(frame);
    reply.validate().unwrap();
    reply
}
fn refresh_weight(frame: &mut combat::Frame) {
    frame.weights = frame
        .items
        .iter()
        .map(|items| {
            let units = items
                .stocks
                .iter()
                .map(|s| weight::UnitWeight {
                    item_id: s.item_id,
                    unit_weight: 1000,
                })
                .collect();
            let total = items.stocks.iter().map(|s| s.remaining).sum();
            let stage = weight::stage(total, 1000.0).unwrap();
            weight::ActorWeight {
                entity_id: items.entity_id,
                weight: weight::Snapshot {
                    source_gear_weight: 0,
                    supplies_weight: total,
                    total,
                    maximum_bits: 1000f64.to_bits(),
                    stage,
                    hit_penalty: weight::penalty(stage),
                    units,
                },
            }
        })
        .collect();
}
/// Simulates one complete authority receipt; it never mutates the prior value.
pub(crate) fn use_timed(prior: &combat::Reply, kind: Kind, observed: i64) -> item::Reply {
    let mut frame = prior.frame.as_ref().unwrap().clone();
    assert!(kind.eligible(frame.world.room.members[0].class));
    frame.world.revision += 1;
    let revision = frame.world.revision;
    let state = &mut frame.items[0];
    state.request_sequence += 1;
    state.use_count += 1;
    state.use_revision = revision;
    state.item_id = kind.item_id();
    state.restored = 0;
    state.hp_before = frame.vitals[0].hp;
    state.hp_after = state.hp_before;
    state.effect = kind.effect();
    state.delay_millis = 0;
    state.position = frame.world.actors[0].position;
    state
        .stocks
        .iter_mut()
        .find(|s| s.item_id == kind.item_id())
        .unwrap()
        .remaining -= 1;
    let sequence = state.request_sequence;
    let effects = &mut frame.timed[0];
    assert!(observed >= effects.observed_nanos);
    if effects.haste_until_nanos <= observed {
        effects.haste_until_nanos = 0;
    }
    if effects.brave_until_nanos <= observed {
        effects.brave_until_nanos = 0;
        effects.brave_kind = 0;
    }
    if effects.wisdom_until_nanos <= observed {
        effects.wisdom_until_nanos = 0;
    }
    effects.revision = revision;
    effects.observed_nanos = observed;
    let until = observed + kind.duration_millis() * 1_000_000;
    match kind {
        Kind::Haste => effects.haste_until_nanos = until,
        Kind::Brave => {
            effects.brave_until_nanos = until;
            effects.brave_kind = 1;
        }
        Kind::ElfBrave => {
            effects.brave_until_nanos = until;
            effects.brave_kind = 3;
        }
        Kind::Wisdom => effects.wisdom_until_nanos = until,
    }
    refresh_weight(&mut frame);
    let reply = item::Reply {
        echo: item::Request {
            context: prior.echo.context,
            channel_id: prior.echo.channel_id,
            action: item::Action::Use,
            match_id: frame.world.match_id,
            generation: frame.world.generation,
            sequence,
            item_id: kind.item_id(),
        },
        status: item::Status::Ok,
        reason: item::Reason::None,
        frame: Some(frame),
    };
    reply.validate().unwrap();
    reply
}
fn expire(prior: &combat::Reply, observed: i64) -> combat::Reply {
    let mut next = prior.clone();
    let frame = next.frame.as_mut().unwrap();
    frame.world.revision += 1;
    let state = &mut frame.timed[0];
    state.revision = frame.world.revision;
    state.observed_nanos = observed;
    if state.haste_until_nanos <= observed {
        state.haste_until_nanos = 0;
    }
    if state.brave_until_nanos <= observed {
        state.brave_until_nanos = 0;
        state.brave_kind = 0;
    }
    if state.wisdom_until_nanos <= observed {
        state.wisdom_until_nanos = 0;
    }
    next.validate().unwrap();
    next
}
fn bytes(states: &[State]) -> Vec<u8> {
    let mut result = Vec::new();
    result.extend((states.len() as u16).to_be_bytes());
    result.extend([0; 6]);
    for s in states {
        result.extend(s.entity_id.to_be_bytes());
        for n in [
            s.revision,
            s.observed_nanos,
            s.haste_until_nanos,
            s.brave_until_nanos,
            s.wisdom_until_nanos,
        ] {
            result.extend(n.to_be_bytes());
        }
        result.push(s.brave_kind);
        result.extend([0; 7]);
    }
    result
}
fn hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn exact_java_independent_golden_is_a_56_byte_state_after_8_byte_prefix() {
    let s = State {
        entity_id: 1,
        revision: 2,
        observed_nanos: 3,
        haste_until_nanos: 10,
        brave_until_nanos: 20,
        wisdom_until_nanos: 30,
        brave_kind: 3,
    };
    let golden = hex(JAVA_GOLDEN);
    assert_eq!(golden.len(), 64);
    assert_eq!(bytes(&[s.clone()]), golden);
    assert_eq!(decode_states(&golden, 1).unwrap(), (vec![s], 64));
    assert_eq!(decode_states(&[0; 8], 100).unwrap(), (Vec::new(), 8));
}
#[test]
fn every_truncation_reserved_field_order_and_count_is_rejected() {
    let good = hex(JAVA_GOLDEN);
    for end in 0..good.len() {
        assert!(decode_states(&good[..end], 1).is_err(), "cut {end}");
    }
    for index in (2..8).chain(57..64) {
        let mut bad = good.clone();
        bad[index] = 1;
        assert!(decode_states(&bad, 1).is_err(), "reserved {index}");
    }
    for count in [0, 2, 100, 101, u16::MAX] {
        let mut bad = good.clone();
        bad[..2].copy_from_slice(&count.to_be_bytes());
        if count == 0 {
            assert_eq!(decode_states(&bad, 1).unwrap().1, 8);
        } else {
            assert!(decode_states(&bad, 1).is_err());
        }
    }
    let reversed = bytes(&[State::initial(2), State::initial(1)]);
    assert!(decode_states(&reversed, 2).is_err());
    let duplicate = bytes(&[State::initial(1), State::initial(1)]);
    assert!(decode_states(&duplicate, 2).is_err());
}
#[test]
fn state_rejects_invalid_brave_kind_times_negative_values_and_fake_initial_effect() {
    for which in 0..10 {
        let mut s = State::initial(1);
        match which {
            0 => s.entity_id = 0,
            1 => s.entity_id = u64::MAX,
            2 => s.revision = -1,
            3 => s.observed_nanos = -1,
            4 => s.observed_nanos = MAX_OBSERVED_NANOS + 1,
            5 => s.brave_kind = 2,
            6 => s.brave_kind = 1,
            7 => s.haste_until_nanos = 1,
            8 => {
                s.revision = 1;
                s.haste_until_nanos = -1;
            }
            _ => {
                s.revision = 1;
                s.haste_until_nanos = 300 * SECOND + 1;
            }
        }
        assert!(s.validate().is_err(), "case {which}");
    }
    let mut s = State::initial(1);
    s.revision = 1;
    s.observed_nanos = 10;
    s.haste_until_nanos = 10;
    assert!(s.validate().is_err());
}
#[test]
fn full_initial_roster_is_not_an_implicit_global_buff_and_requires_full_weights() {
    let good = initial();
    let frame = good.frame.as_ref().unwrap();
    assert!(frame.validate().is_ok());
    for case in 0..4 {
        let mut bad = frame.clone();
        match case {
            0 => bad.timed.clear(),
            1 => {
                bad.timed.pop();
            }
            2 => bad.weights.clear(),
            _ => bad.timed.swap(0, 1),
        }
        assert!(bad.validate().is_err(), "shape {case}");
    }
    assert!(frame
        .timed
        .iter()
        .all(|s| *s == State::initial(s.entity_id)));
}
#[test]
fn each_class_mask_and_source_asset_identity_is_explicit() {
    for kind in Kind::ALL {
        assert_eq!(Kind::from_item(kind.item_id()), Some(kind));
        assert_eq!(
            kind.duration_millis(),
            if kind == Kind::ElfBrave {
                480000
            } else {
                300000
            }
        );
        for class in 0..7 {
            assert_eq!(
                kind.eligible(class),
                (kind.class_mask() & (1 << class)) != 0
            );
        }
        assert!(!kind.eligible(7));
        assert!(!kind.eligible(u16::MAX));
    }
    assert_eq!(Kind::Haste.effect(), 191);
    assert_eq!(Kind::Brave.effect(), 751);
    assert_eq!(Kind::ElfBrave.effect(), 751);
    assert_eq!(Kind::Wisdom.effect(), 750);
    assert_eq!(Kind::from_item(40010), None);
}
#[test]
fn successful_use_is_atomic_stock_weight_and_buff_without_hp_mp_or_base_speed_edit() {
    for (kind, class) in [
        (Kind::Haste, 3),
        (Kind::Brave, 3),
        (Kind::Wisdom, 3),
        (Kind::ElfBrave, 2),
    ] {
        let old = initial_for_class(class);
        let unchanged = old.clone();
        let new = use_timed(&old, kind, 3 * SECOND);
        assert!(new.confirms(&new.echo, Some(&old)));
        assert_eq!(old, unchanged);
        let a = old.frame.as_ref().unwrap();
        let b = new.frame.as_ref().unwrap();
        assert_eq!(a.vitals, b.vitals);
        assert_eq!(a.world.actors, b.world.actors);
        assert_eq!(a.world.casts, b.world.casts);
        assert_eq!(b.items[0].use_revision, b.timed[0].revision);
        assert_eq!(b.items[0].restored, 0);
        assert_eq!(
            b.timed[0].deadline(kind),
            3 * SECOND + kind.duration_millis() * 1_000_000
        );
        assert_eq!(b.weights[0].weight.total, a.weights[0].weight.total - 1);
    }
}
#[test]
fn no_receipt_cannot_mutate_old_state_or_authorize_replaying_the_same_use() {
    let old = initial();
    let first = use_timed(&old, Kind::Haste, 0);
    let confirmed = first.combat_reply();
    assert!(combat::follows(Some(&confirmed), &confirmed));
    assert!(
        !first.confirms(&first.echo, Some(&confirmed)),
        "one acknowledgement is not a second consumption"
    );
    assert_eq!(old.frame.as_ref().unwrap().items[0].use_count, 0);
    let mut wrong = first.clone();
    wrong.echo.sequence += 1;
    assert!(!wrong.confirms(&first.echo, Some(&old)));
}
#[test]
fn same_revision_clock_resampling_and_extended_deadline_without_stock_are_forbidden() {
    let first = use_timed(&initial(), Kind::Haste, 0).combat_reply();
    let mut changed = first.clone();
    changed.frame.as_mut().unwrap().timed[0].observed_nanos = SECOND;
    assert!(!combat::follows(Some(&first), &changed));
    let mut fake = first.clone();
    let f = fake.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.timed[0].revision = f.world.revision;
    f.timed[0].observed_nanos = SECOND;
    f.timed[0].haste_until_nanos += SECOND;
    assert!(fake.validate().is_ok());
    assert!(!combat::follows(Some(&first), &fake));
}
#[test]
fn expiry_requires_the_authority_deadline_and_keeps_stock_and_base_intervals() {
    let first = use_timed(&initial(), Kind::Haste, 0).combat_reply();
    let expired = expire(&first, 300 * SECOND);
    assert!(combat::follows(Some(&first), &expired));
    assert!(!expired.frame.as_ref().unwrap().timed[0].haste());
    assert_eq!(
        first.frame.as_ref().unwrap().items,
        expired.frame.as_ref().unwrap().items
    );
    assert_eq!(
        first.frame.as_ref().unwrap().world.actors,
        expired.frame.as_ref().unwrap().world.actors
    );
    let mut too_early = expired.clone();
    too_early.frame.as_mut().unwrap().timed[0].observed_nanos -= 1;
    assert!(too_early.validate().is_ok());
    assert!(!combat::follows(Some(&first), &too_early));
    assert!(!combat::follows(Some(&expired), &first));
}
#[test]
fn refresh_and_coalesced_multiple_consumptions_need_original_stock_proof() {
    let old = initial();
    let first = use_timed(&old, Kind::Haste, 0).combat_reply();
    let refreshed = use_timed(&first, Kind::Haste, 100 * SECOND);
    assert!(refreshed.confirms(&refreshed.echo, Some(&first)));
    assert_eq!(
        refreshed.frame.as_ref().unwrap().timed[0].haste_until_nanos,
        400 * SECOND
    );
    let last = use_timed(&refreshed.combat_reply(), Kind::Wisdom, 101 * SECOND).combat_reply();
    assert!(combat::follows(Some(&old), &last));
    assert!(last.frame.as_ref().unwrap().timed[0].wisdom());
    let mut fake = last.clone();
    let frame = fake.frame.as_mut().unwrap();
    frame.items[0]
        .stocks
        .iter_mut()
        .find(|s| s.item_id == 40016)
        .unwrap()
        .remaining += 1;
    refresh_weight(frame);
    assert!(
        fake.validate().is_err(),
        "cannot retain effect while restoring its consumed bottle"
    );
}
#[test]
fn elf_brave_replaces_regular_brave_with_exact_480_seconds_and_preserves_haste() {
    let old = initial_for_class(2);
    let haste = use_timed(&old, Kind::Haste, 0).combat_reply();
    let brave = use_timed(&haste, Kind::Brave, 0).combat_reply();
    let elf = use_timed(&brave, Kind::ElfBrave, 10 * SECOND);
    assert!(elf.confirms(&elf.echo, Some(&brave)));
    let s = &elf.frame.as_ref().unwrap().timed[0];
    assert_eq!(s.brave_kind, 3);
    assert_eq!(s.brave_until_nanos, 490 * SECOND);
    assert_eq!(s.haste_until_nanos, 300 * SECOND);
    let back = use_timed(&elf.combat_reply(), Kind::Brave, 11 * SECOND);
    assert!(back.confirms(&back.echo, Some(&elf.combat_reply())));
    assert_eq!(back.frame.as_ref().unwrap().timed[0].brave_kind, 1);
}
#[test]
fn early_terminal_or_death_clears_are_valid_but_cannot_resurrect_or_restore_stock() {
    let active = use_timed(&initial(), Kind::Haste, 0).combat_reply();
    let mut terminal = active.clone();
    let f = terminal.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.world.phase = world::Phase::Aborted;
    f.world.abort_reason = world::AbortReason::Disconnected;
    f.timed[0] = State {
        revision: f.world.revision,
        observed_nanos: SECOND,
        ..State::initial(1)
    };
    terminal.validate().unwrap();
    assert!(combat::follows(Some(&active), &terminal));
    assert!(!combat::follows(Some(&terminal), &active));
    let mut fake = terminal.clone();
    fake.frame.as_mut().unwrap().timed[0] = active.frame.as_ref().unwrap().timed[0].clone();
    assert!(fake.validate().is_err());
    let mut death = active.clone();
    let f = death.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.vitals[0].hp = 0;
    f.vitals[0].hurt_revision = f.world.revision;
    f.vitals[0].death_revision = f.world.revision;
    assert!(
        death.validate().is_err(),
        "dead actor cannot retain a live lease"
    );
    let f = death.frame.as_mut().unwrap();
    f.timed[0] = State {
        revision: f.world.revision,
        observed_nanos: SECOND,
        ..State::initial(1)
    };
    death.validate().unwrap();
    assert!(combat::follows(Some(&active), &death));
}
#[test]
fn active_effect_requires_class_and_consumed_stock_not_just_a_known_item_id() {
    let good = use_timed(&initial(), Kind::Wisdom, 0).combat_reply();
    let mut wrong_class = good.clone();
    wrong_class.frame.as_mut().unwrap().world.room.members[0].class = 2;
    assert!(wrong_class.validate().is_err());
    let mut wrong_stock = good.clone();
    let f = wrong_stock.frame.as_mut().unwrap();
    f.items[0].stocks.retain(|s| s.item_id != 40016);
    refresh_weight(f);
    assert!(wrong_stock.validate().is_err());
    let mut invented = initial();
    let f = invented.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.timed[0] = State {
        revision: f.world.revision,
        wisdom_until_nanos: 300 * SECOND,
        ..State::initial(1)
    };
    assert!(invented.validate().is_err());
}
#[test]
fn terminal_clearing_cannot_reuse_the_latest_timed_consumption_revision() {
    let active = use_timed(&initial(), Kind::Haste, 0).combat_reply();
    let mut genuine = active.clone();
    let f = genuine.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.world.phase = world::Phase::Aborted;
    f.world.abort_reason = world::AbortReason::Disconnected;
    f.timed[0] = State {
        revision: f.world.revision,
        observed_nanos: SECOND,
        ..State::initial(1)
    };
    genuine.validate().unwrap();
    assert!(combat::follows(Some(&active), &genuine));
    let mut forged = genuine.clone();
    let f = forged.frame.as_mut().unwrap();
    f.timed[0].revision = f.items[0].use_revision;
    assert!(
        forged.validate().is_err(),
        "a true terminal clear is a later authority event, not the timed use itself"
    );
}
#[test]
fn timing_formula_keeps_each_binary64_millisecond_truncation_and_elf_double_layer() {
    let mut s = State {
        revision: 1,
        haste_until_nanos: 300 * SECOND,
        ..State::initial(1)
    };
    assert_eq!(s.effective_nanos(700_000_000, Pace::Attack), 528_000_000);
    s.brave_kind = 1;
    s.brave_until_nanos = 300 * SECOND;
    assert_eq!(s.effective_nanos(700_000_000, Pace::Attack), 398_000_000);
    s.brave_kind = 3;
    s.brave_until_nanos = 480 * SECOND;
    assert_eq!(s.effective_nanos(700_000_000, Pace::Attack), 306_000_000);
    assert_eq!(s.effective_nanos(700_000_000, Pace::Move), 340_000_000);
    assert_eq!(s.effective_nanos(1_000_000, Pace::Spell), 1_000_000);
    assert_eq!(
        State::initial(1).effective_nanos(200_000_001, Pace::Move),
        200_000_001
    );
    let wizard = use_timed(&initial(), Kind::Wisdom, 0).combat_reply();
    assert_eq!(
        wizard.frame.as_ref().unwrap().timed[0].spell_power_bonus(),
        2
    );
    assert_eq!(
        effective_nanos(wizard.frame.as_ref().unwrap(), 1, 700_000_000, Pace::Attack),
        700_000_000
    );
}
#[test]
fn base_interval_replacement_is_not_an_effect_update() {
    let old = initial();
    let good = use_timed(&old, Kind::Haste, 0).combat_reply();
    for actor in [false, true] {
        let mut bad = good.clone();
        let f = bad.frame.as_mut().unwrap();
        if actor {
            f.world.actors[0].step_interval_nanos = 151_000_000;
        } else {
            f.vitals[0].attack_interval_nanos = 528_000_000;
        }
        assert!(bad.validate().is_ok());
        assert!(!combat::follows(Some(&old), &bad));
    }
    let mut nanos = good.clone();
    nanos.frame.as_mut().unwrap().world.actors[0].step_interval_nanos += 1;
    assert!(nanos.validate().is_err());
}
#[test]
fn capability_presence_is_two_way_and_not_inferred_from_version_or_last_item() {
    let timed = initial();
    let response = app::Response::Combat(timed.clone());
    assert!(negotiated(&response, true));
    assert!(!negotiated(&response, false));
    let old = item::tests::initial();
    let response = app::Response::Combat(old);
    assert!(negotiated(&response, false));
    assert!(!negotiated(&response, true));
    let used = use_timed(&timed, Kind::Haste, 0);
    assert!(negotiated(&app::Response::Item(used.clone()), true));
    assert!(!negotiated(&app::Response::Item(used), false));
}
#[test]
fn old_item_versions_reject_timed_stocks_while_v10_preserves_all_seven() {
    let good = initial();
    let f = good.frame.as_ref().unwrap();
    let bytes = item::tests::state_bytes(&f.items);
    assert_eq!(
        item::decode_states_version(&bytes, 2, 10).unwrap(),
        (f.items.clone(), bytes.len())
    );
    for version in [8, 9] {
        assert!(item::decode_states_version(&bytes, 2, version).is_err());
    }
    let old = item::tests::initial();
    let bytes = item::tests::state_bytes(&old.frame.as_ref().unwrap().items);
    assert_eq!(
        item::decode_states_version(&bytes, 2, 8).unwrap(),
        item::decode_states_version(&bytes, 2, 9).unwrap()
    );
}
#[test]
fn timed_uses_cannot_heal_or_forge_a_delay_and_failed_class_action_keeps_confirmed_state() {
    let old = initial();
    let valid = use_timed(&old, Kind::Haste, 0);
    for which in 0..3 {
        let mut bad = valid.clone();
        let f = bad.frame.as_mut().unwrap();
        match which {
            0 => {
                f.items[0].restored = 1;
                f.items[0].hp_after += 1;
                f.vitals[0].hp += 1;
            }
            1 => f.items[0].delay_millis = 500,
            _ => f.items[0].effect = 189,
        }
        assert!(bad.validate().is_err());
        assert!(!bad.confirms(&bad.echo, Some(&old)));
    }
    let mut denied = valid.clone();
    denied.frame = old.frame.clone();
    denied.status = item::Status::Denied;
    denied.reason = item::Reason::ClassRestricted;
    denied.echo.item_id = 40068;
    let f = denied.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.items[0].request_sequence = 1;
    denied.validate().unwrap();
    assert!(denied.confirms(&denied.echo, Some(&old)));
    assert_eq!(
        old.frame.as_ref().unwrap().items[0].stocks,
        denied.frame.as_ref().unwrap().items[0].stocks
    );
    for item_id in [40010, 40013, 40016] {
        let mut wrong = denied.clone();
        wrong.echo.item_id = item_id;
        assert!(wrong.validate().is_err());
    }
    let mut read = denied.clone();
    read.echo.action = item::Action::Read;
    read.echo.sequence = 0;
    read.echo.item_id = 0;
    assert!(read.validate().is_err());
}
#[test]
fn allocation_budget_is_versioned_only_for_combat_spell_and_item_results() {
    for kind in [23u16, 27, 35] {
        assert_eq!(combat::frame_limit(9), 131072);
        assert_eq!(combat::frame_limit(10), 262144);
        let mut header = app::header(1, kind, 16).unwrap();
        header[4..8].copy_from_slice(&131073u32.to_be_bytes());
        let mut old = std::io::Cursor::new(header.clone());
        let mut new = std::io::Cursor::new(header);
        assert!(app::read_response_version(&mut old, 9).is_err());
        assert_eq!(old.position(), 16);
        assert!(app::read_response_version(&mut new, 10).is_err());
        assert_eq!(new.position(), 16);
    }
    assert!(app::header(1, 7, 65537).is_err());
}
