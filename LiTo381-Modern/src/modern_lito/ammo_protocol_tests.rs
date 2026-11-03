use super::*;

fn facts(id: i64) -> Facts {
    Facts {
        item_id: id,
        small: 3,
        large: 4,
        raw_arrow_sprite: -1,
        consume: true,
        material: if id == 40744 { 0 } else { 1 },
        blessing: 0,
    }
}
pub(crate) fn initial() -> combat::Reply {
    let mut r = combat::tests::active_reply();
    let f = r.frame.as_mut().unwrap();
    f.vitals[0].weapon_type = 4;
    f.ammo = f
        .world
        .actors
        .iter()
        .map(|a| {
            State::initial(
                a.entity_id,
                if a.entity_id == 1 {
                    UI_ITEMS
                        .into_iter()
                        .map(|id| Stock {
                            facts: facts(id),
                            initial: 2,
                            remaining: 2,
                        })
                        .collect()
                } else {
                    vec![]
                },
            )
        })
        .collect();
    r.validate().unwrap();
    r
}
pub(crate) fn selection(prior: &combat::Reply, id: i64) -> Reply {
    let mut f = prior.frame.as_ref().unwrap().clone();
    f.world.revision += 1;
    let s = &mut f.ammo[0];
    s.request_sequence += 1;
    s.revision = f.world.revision;
    let reason = if !supported(id) {
        Reason::Unsupported
    } else if s.stock(id).is_none_or(|s| s.remaining == 0) {
        Reason::Empty
    } else {
        Reason::None
    };
    if reason == Reason::None {
        s.selected_item_id = id;
        s.selection_revision = s.revision;
    }
    let echo = Request {
        action: Action::Select,
        sequence: s.request_sequence,
        item_id: id,
        ..Request::read(prior.echo.context, prior.echo.channel_id, &f.world)
    };
    Reply {
        echo,
        status: if reason == Reason::None {
            Status::Ok
        } else {
            Status::Denied
        },
        reason,
        frame: Some(f),
    }
}
fn shot(prior: &combat::Reply, hit: bool) -> combat::Reply {
    let mut r = prior.clone();
    let f = r.frame.as_mut().unwrap();
    f.world.revision += 1;
    let rev = f.world.revision;
    let v = &mut f.vitals[0];
    v.attack_sequence += 1;
    v.attack_revision = rev;
    v.last_target_entity_id = 2;
    let s = &mut f.ammo[0];
    s.shot_count += 1;
    s.revision = rev;
    s.shot_revision = rev;
    s.shot_item_id = s.selected_item_id;
    s.attack_sequence = v.attack_sequence;
    s.last_target_entity_id = 2;
    s.from = f.world.actors[0].position;
    s.to = f.world.actors[1].position;
    s.projectile_sprite = 66;
    s.hit = hit;
    s.stocks
        .iter_mut()
        .find(|x| x.facts.item_id == s.selected_item_id)
        .unwrap()
        .remaining -= 1;
    if hit {
        f.vitals[1].hp -= 5;
        f.vitals[1].hurt_revision = rev;
    }
    r.echo.action = combat::Action::Attack;
    r.echo.sequence = f.vitals[0].attack_sequence;
    r.echo.target_entity_id = 2;
    r
}
fn as_read(mut r: combat::Reply) -> combat::Reply {
    r.echo.action = combat::Action::Read;
    r.echo.sequence = 0;
    r.echo.target_entity_id = 0;
    r
}
fn hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}
fn states_bytes(states: &[State]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend((states.len() as u16).to_be_bytes());
    b.extend([0; 6]);
    for s in states {
        for n in [
            s.entity_id,
            s.request_sequence,
            s.revision as u64,
            s.selection_revision as u64,
            s.selected_item_id as u64,
            s.shot_count,
            s.shot_revision as u64,
            s.shot_item_id as u64,
            s.attack_sequence,
            s.last_target_entity_id,
        ] {
            b.extend(n.to_be_bytes());
        }
        for n in [s.from.x, s.from.y, s.to.x, s.to.y, s.projectile_sprite] {
            b.extend(n.to_be_bytes());
        }
        b.extend([s.hit as u8, s.stocks.len() as u8, 0, 0]);
        for t in &s.stocks {
            for n in [
                t.facts.item_id,
                t.initial,
                t.remaining,
                t.facts.small,
                t.facts.large,
            ] {
                b.extend(n.to_be_bytes());
            }
            b.extend(t.facts.raw_arrow_sprite.to_be_bytes());
            b.extend([t.facts.consume as u8, t.facts.material, t.facts.blessing, 0]);
        }
    }
    b
}

#[test]
fn independent_request_and_absent_reply_literals_keep_56_byte_body_and_old_versions_closed() {
    let request = Request {
        context: SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        },
        channel_id: 2,
        action: Action::Select,
        match_id: 2,
        generation: 3,
        sequence: 4,
        item_id: 40744,
    };
    const BODY: &str = concat!(
        "000000070000000000000d05ffffffffffffffff02010000",
        "0000000000000002000000000000000300000000000000040000000000009f28"
    );
    let expected = hex(&format!("4c544d3100000048fedcba9800240000{BODY}"));
    assert_eq!(encode_request(0xfedcba98, request, 11).unwrap(), expected);
    assert_eq!(expected.len(), 72);
    let reply = hex(&format!("4c544d310000004cfedcba9800250000{BODY}03010000"));
    let app::Response::Ammo(r) = app::decode_response_version(&reply, 11).unwrap().response else {
        panic!("kind")
    };
    assert_eq!(r.echo, request);
    assert_eq!(r.reason, Reason::NotActive);
    assert!(r.frame.is_none());
    for version in 1..=10 {
        assert!(encode_request(1, request, version).is_err());
        assert!(app::decode_response_version(&reply, version).is_err());
    }
    assert_eq!(encode_request(0xfedcba98, request, 12).unwrap(), expected);
    assert_eq!(
        app::decode_response_version(&reply, 12).unwrap().response,
        app::Response::Ammo(r)
    );
    assert_eq!(encode_request(0xfedcba98, request, 13).unwrap(), expected);
    assert!(app::decode_response_version(&reply, 13).is_ok());
    assert_eq!(encode_request(0xfedcba98, request, 14).unwrap(), expected);
    assert_eq!(encode_request(0xfedcba98, request, 15).unwrap(), expected);
    assert!(encode_request(1, request, 16).is_ok());
    assert_eq!(encode_request(0xfedcba98, request, 17).unwrap(), expected);
    assert!(encode_request(1, request, super::super::protocol::MAX_VERSION + 1).is_err());
    assert!(app::decode_response_version(&reply, 14).is_ok());
    assert!(app::decode_response_version(&reply, 15).is_ok());
    assert!(app::decode_response_version(&reply, 16).is_ok());
    assert!(app::decode_response_version(&reply, 17).is_ok());
    assert!(app::decode_response_version(&reply, super::super::protocol::MAX_VERSION + 1).is_err());
    for n in 0..reply.len() {
        assert!(
            app::decode_response_version(&reply[..n], 11).is_err(),
            "truncate {n}"
        );
    }
    for at in [14, 15, 38, 39, 75] {
        let mut b = reply.clone();
        b[at] = 1;
        assert!(
            app::decode_response_version(&b, 11).is_err(),
            "padding {at}"
        );
    }
    let mut tail = reply;
    tail.push(0);
    let n = tail.len() as u32;
    tail[4..8].copy_from_slice(&n.to_be_bytes());
    assert!(app::decode_response_version(&tail, 11).is_err());
}
#[test]
fn exact_state_104_stock_48_counts_padding_and_each_prefix() {
    let states = initial().frame.unwrap().ammo;
    let b = states_bytes(&states);
    assert_eq!(b.len(), 8 + 104 * 2 + 48 * 2);
    assert_eq!(decode_states(&b, 2).unwrap(), (states, b.len()));
    for n in 0..b.len() {
        assert!(decode_states(&b[..n], 2).is_err(), "truncate {n}");
    }
    for at in [2, 7, 8 + 102, 8 + 103, 8 + 104 + 47] {
        let mut bad = b.clone();
        bad[at] = 1;
        assert!(decode_states(&bad, 2).is_err(), "padding {at}");
    }
    let mut bad = b.clone();
    bad[1] = 3;
    assert!(decode_states(&bad, 2).is_err());
    let mut bad = b;
    bad[8 + 101] = 3;
    assert!(decode_states(&bad, 2).is_err());
}
#[test]
fn facts_are_exact_material_consume_and_qualified_sprite_with_wide_damage() {
    for id in UI_ITEMS {
        let mut f = facts(id);
        f.small = i64::MAX;
        f.large = i64::MAX;
        f.validate().unwrap();
        for case in 0..7 {
            let mut bad = f;
            match case {
                0 => bad.small = 0,
                1 => bad.large = -1,
                2 => bad.raw_arrow_sprite = 65,
                3 => bad.consume = false,
                4 => bad.material ^= 1,
                5 => bad.blessing = 1,
                _ => bad.item_id = 40010,
            }
            assert!(bad.validate().is_err());
        }
    }
}
#[test]
fn initial_lowest_arrow_and_exhaustion_do_not_automatically_swap() {
    let a = initial();
    let b = shot(&a, false);
    let c = shot(&b, false);
    assert!(combat::follows(Some(&a), &b));
    assert!(combat::follows(Some(&b), &c));
    let s = &c.frame.as_ref().unwrap().ammo[0];
    assert_eq!(s.selected_item_id, 40744);
    assert!(!s.available());
    assert_eq!(s.stock(40746).unwrap().remaining, 2);
    let mut bad = a.frame.unwrap().ammo[0].clone();
    bad.selected_item_id = 40746;
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.selected_item_id = 40746;
    assert!(bad.validate().is_err());
}
#[test]
fn miss_and_hit_each_consume_exactly_one_arrow_and_only_hit_may_hurt() {
    let old = initial();
    for hit in [false, true] {
        let next = shot(&old, hit);
        next.validate().unwrap();
        assert!(combat::follows(Some(&old), &next));
        assert!(confirms_attack(Some(&old), &next));
        assert_eq!(next.frame.as_ref().unwrap().ammo[0].stocks[0].remaining, 1);
        assert_eq!(
            next.frame.as_ref().unwrap().vitals[1].hp,
            if hit { 45 } else { 50 }
        );
    }
    let mut forged = shot(&old, false);
    let f = forged.frame.as_mut().unwrap();
    f.vitals[1].hp -= 1;
    f.vitals[1].hurt_revision = f.world.revision;
    assert!(forged.validate().is_err());
}
#[test]
fn first_snapshot_without_selection_cannot_hide_consumption_of_the_other_arrow() {
    let accepted = shot(&initial(), false);
    let mut state = accepted.frame.as_ref().unwrap().ammo[0].clone();
    assert_eq!(state.selection_revision, 0);
    state.validate().unwrap();
    state.stocks[1].remaining -= 1;
    state.shot_count += 1;
    state.attack_sequence += 1;
    assert_eq!(
        state
            .stocks
            .iter()
            .map(|s| s.initial - s.remaining)
            .sum::<i64>(),
        state.shot_count as i64
    );
    assert_eq!(state.shot_item_id, state.selected_item_id);
    assert!(
        state.validate().is_err(),
        "a matching latest shot cannot conceal an unselected earlier shot"
    );
}
#[test]
fn selection_changes_no_shot_or_vitals_and_same_id_selection_has_own_revision() {
    let prior = initial();
    for id in UI_ITEMS {
        let reply = selection(&prior, id);
        assert!(reply.confirms(&reply.echo, Some(&prior)));
        assert_eq!(
            reply.frame.as_ref().unwrap().vitals,
            prior.frame.as_ref().unwrap().vitals
        );
    }
    let selected = selection(&prior, 40746).combat_reply();
    let same = selection(&selected, 40746);
    assert!(same.confirms(&same.echo, Some(&selected)));
    assert_eq!(same.frame.as_ref().unwrap().ammo[0].request_sequence, 2);
    assert_eq!(same.frame.as_ref().unwrap().ammo[0].shot_count, 0);
}
#[test]
fn readonly_cannot_consume_an_unselected_arrow_without_intervening_selection() {
    let old = initial();
    let mut next = shot(&old, false);
    let s = &mut next.frame.as_mut().unwrap().ammo[0];
    s.stocks[0].remaining += 1;
    s.stocks[1].remaining -= 1;
    s.shot_item_id = 40746;
    assert!(next.validate().is_err());
    let selected = selection(&old, 40746).combat_reply();
    let mut next = shot(&selected, false);
    let s = &mut next.frame.as_mut().unwrap().ammo[0];
    s.stocks[0].remaining -= 1;
    s.shot_count += 1;
    s.attack_sequence += 1;
    next.frame.as_mut().unwrap().vitals[0].attack_sequence += 1;
    next.echo.sequence += 1;
    assert!(next.validate().is_ok());
    assert!(!combat::follows(Some(&selected), &next));
}
#[test]
fn coalesced_selection_then_other_arrow_shot_is_valid_but_history_is_immutable() {
    let old = initial();
    let one = shot(&old, false);
    let selected = selection(&one, 40746).combat_reply();
    let two = shot(&selected, true);
    assert!(combat::follows(Some(&old), &two));
    assert!(confirms_attack(Some(&selected), &two));
    let mut later = as_read(two.clone());
    later.frame.as_mut().unwrap().world.revision += 1;
    later.frame.as_mut().unwrap().vitals[0].attack_sequence += 1;
    assert!(combat::follows(Some(&two), &later));
    later.frame.as_mut().unwrap().ammo[0].attack_sequence += 1;
    assert!(!combat::follows(Some(&two), &later));
}
#[test]
fn no_ammo_denial_is_not_a_shot_and_requires_live_bow_empty_selected() {
    let old = shot(&shot(&initial(), false), false);
    let mut denied = old.clone();
    denied.status = combat::Status::Denied;
    denied.reason = combat::Reason::NoAmmo;
    denied.echo.sequence += 1;
    let f = denied.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.vitals[0].attack_sequence += 1;
    denied.validate().unwrap();
    assert!(confirms_attack(Some(&old), &denied));
    assert!(combat::follows(Some(&old), &denied));
    let mut bad = denied.clone();
    bad.echo.action = combat::Action::Read;
    bad.echo.sequence = 0;
    bad.echo.target_entity_id = 0;
    assert!(bad.validate().is_err());
    let mut bad = denied.clone();
    bad.frame.as_mut().unwrap().ammo.clear();
    assert!(bad.validate().is_err());
    assert!(!negotiated(&app::Response::Combat(denied), false));
}
#[test]
fn inactive_select_is_explicit_non_consuming_not_active_for_loading_countdown_terminal() {
    for phase in [
        world::Phase::Loading,
        world::Phase::Countdown,
        world::Phase::Aborted,
    ] {
        let mut old = initial();
        let f = old.frame.as_mut().unwrap();
        f.world.phase = phase;
        if phase == world::Phase::Loading {
            for a in &mut f.world.actors {
                a.loaded = false;
            }
        }
        if phase == world::Phase::Aborted {
            f.world.abort_reason = world::AbortReason::ServerAbort;
        }
        let request = Request {
            action: Action::Select,
            sequence: 1,
            item_id: 40746,
            ..Request::read(old.echo.context, old.echo.channel_id, &f.world)
        };
        old.validate().unwrap();
        let reply = Reply {
            echo: request,
            status: Status::Denied,
            reason: Reason::NotActive,
            frame: old.frame.clone(),
        };
        assert!(reply.confirms(&request, Some(&old)), "{phase:?}");
        let mut forged = reply.clone();
        forged.frame.as_mut().unwrap().ammo[0].request_sequence = 1;
        forged.frame.as_mut().unwrap().ammo[0].revision = 1;
        assert!(!forged.confirms(&request, Some(&old)));
    }
}
#[test]
fn retained_loading_finished_and_aborted_ammo_read_is_success_not_a_fake_selection_denial() {
    let mut loading = initial();
    loading.frame.as_mut().unwrap().world.phase = world::Phase::Loading;
    for actor in &mut loading.frame.as_mut().unwrap().world.actors {
        actor.loaded = false;
    }
    let mut aborted = initial();
    aborted.frame.as_mut().unwrap().world.phase = world::Phase::Aborted;
    aborted.frame.as_mut().unwrap().world.abort_reason = world::AbortReason::ServerAbort;
    let mut finished = combat::tests::finished_reply();
    let f = finished.frame.as_mut().unwrap();
    f.ammo = f
        .world
        .actors
        .iter()
        .map(|a| State::initial(a.entity_id, vec![]))
        .collect();
    for old in [loading, finished, aborted] {
        old.validate().unwrap();
        let f = old.frame.as_ref().unwrap();
        let request = Request::read(old.echo.context, old.echo.channel_id, &f.world);
        let reply = Reply {
            echo: request,
            status: Status::Ok,
            reason: Reason::None,
            frame: old.frame.clone(),
        };
        assert!(reply.confirms(&request, Some(&old)), "{:?}", f.world.phase);
        for status in [Status::Denied, Status::Busy, Status::Unavailable] {
            let mut invalid = reply.clone();
            invalid.status = status;
            invalid.reason = Reason::NotActive;
            assert!(
                invalid.validate().is_err(),
                "READ with full retained frame must be OK, {status:?}"
            );
        }
        let mut unavailable = reply;
        unavailable.frame = None;
        unavailable.status = Status::Unavailable;
        unavailable.reason = Reason::NotActive;
        unavailable.validate().unwrap();
    }
    let old = initial();
    let mut not_active = selection(&old, 40744);
    not_active.status = Status::Denied;
    not_active.reason = Reason::NotActive;
    assert!(
        not_active.validate().is_err(),
        "ACTIVE cannot claim NOT_ACTIVE"
    );
}
#[test]
fn denial_reason_is_proved_by_alive_supported_and_remaining_state() {
    let old = initial();
    let unsupported = selection(&old, 40010);
    assert!(unsupported.confirms(&unsupported.echo, Some(&old)));
    let mut forged = unsupported.clone();
    forged.reason = Reason::Empty;
    assert!(forged.validate().is_err());
    forged = unsupported.clone();
    forged.reason = Reason::Dead;
    assert!(forged.validate().is_err());
    forged = unsupported.clone();
    let rev = forged.frame.as_ref().unwrap().world.revision;
    forged.frame.as_mut().unwrap().ammo[0].selection_revision = rev;
    assert!(forged.validate().is_err());
    let exhausted = shot(&shot(&old, false), false);
    let empty = selection(&exhausted, 40744);
    assert!(empty.confirms(&empty.echo, Some(&exhausted)));
    forged = empty.clone();
    forged.reason = Reason::Unsupported;
    assert!(forged.validate().is_err());
    forged = empty;
    forged.frame.as_mut().unwrap().ammo[0].request_sequence -= 1;
    assert!(forged.validate().is_err());
}
#[test]
fn typed_bot_cannot_receive_human_ammo_and_full_roster_counts_are_exact() {
    let mut f = initial().frame.unwrap();
    f.ammo.pop();
    assert!(f.validate().is_err());
    let mut f = initial().frame.unwrap();
    f.ammo[1].entity_id = 1;
    assert!(f.validate().is_err());
    // Pure ledger rejects non-canonical fake bot rows even before renderer use.
    let canonical = State::initial(2, vec![]);
    assert_eq!(canonical.revision, 0);
    assert!(canonical.validate().is_ok());
    let mut invalid = canonical;
    invalid.selected_item_id = 40744;
    assert!(invalid.validate().is_err());
}
#[test]
fn world_weight_uses_union_of_consumables_and_arrows_without_changing_use_count() {
    use crate::modern_lito::{item_protocol as item, weight_protocol as weight};
    let mut old = initial();
    let f = old.frame.as_mut().unwrap();
    f.items = item::tests::initial().frame.unwrap().items;
    for s in &mut f.items {
        s.stocks.clear();
    }
    f.weights = f
        .ammo
        .iter()
        .map(|s| {
            let total = s.stocks.iter().map(|s| s.remaining).sum::<i64>();
            let stage = weight::stage(total, 1000.0).unwrap();
            weight::ActorWeight {
                entity_id: s.entity_id,
                weight: weight::Snapshot {
                    source_gear_weight: 0,
                    supplies_weight: total,
                    total,
                    maximum_bits: 1000f64.to_bits(),
                    stage,
                    hit_penalty: weight::penalty(stage),
                    units: s
                        .stocks
                        .iter()
                        .map(|s| weight::UnitWeight {
                            item_id: s.facts.item_id,
                            unit_weight: 1000,
                        })
                        .collect(),
                },
            }
        })
        .collect();
    old.validate().unwrap();
    let mut next = shot(&old, false);
    assert!(next.validate().is_err());
    let w = &mut next.frame.as_mut().unwrap().weights[0].weight;
    w.supplies_weight -= 1;
    w.total -= 1;
    w.stage = weight::stage(w.total, w.maximum()).unwrap();
    w.hit_penalty = weight::penalty(w.stage);
    next.validate().unwrap();
    assert!(combat::follows(Some(&old), &next));
    assert_eq!(
        next.frame.as_ref().unwrap().items,
        old.frame.as_ref().unwrap().items
    );
    let mut forged = next;
    forged.frame.as_mut().unwrap().weights[0].weight.units.pop();
    assert!(forged.validate().is_err());
}
#[test]
fn cap_v11_dependencies_and_disabled_full_frame_are_not_guessed() {
    use crate::modern_lito::protocol as p;
    for version in 1..=10 {
        assert!(p::encode(p::Message::Ack(p::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities: 8191
        }))
        .is_err());
    }
    assert!(p::encode(p::Message::Ack(p::Ack {
        request_id: 1,
        version: 11,
        numeric_schema: 1,
        capabilities: 8191
    }))
    .is_ok());
    for missing in [p::CAP_TIMED, p::CAP_WEIGHT, p::CAP_SHOP, p::CAP_COMBAT] {
        assert!(p::encode(p::Message::Ack(p::Ack {
            request_id: 1,
            version: 11,
            numeric_schema: 1,
            capabilities: 8191 & !missing
        }))
        .is_err());
    }
    let r = initial();
    assert!(negotiated(&app::Response::Combat(r.clone()), true));
    assert!(!negotiated(&app::Response::Combat(r), false));
    let old = combat::tests::active_reply();
    assert!(negotiated(&app::Response::Combat(old.clone()), false));
    assert!(!negotiated(&app::Response::Combat(old), true));
}
