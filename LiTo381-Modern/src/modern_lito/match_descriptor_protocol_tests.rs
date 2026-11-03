use super::*;
use crate::modern_lito::{combat_protocol as combat, protocol as p, regen_protocol as regen};

fn policy(ruleset: MatchRuleset, features: u32) -> MatchDescriptor {
    MatchDescriptor {
        ruleset,
        features,
        preparation_timeout_millis: 30_000,
        load_timeout_millis: 30_000,
        countdown_nanos: 3_000_000_000,
        active_timeout_millis: 300_000,
    }
}
fn scene() -> MatchDescriptor {
    policy(MatchRuleset::SceneOnly, 0)
}
#[test]
fn haste_candidate_policy_preserves_legacy_rejection_and_exact_mode_dependencies() {
    use MatchDescriptor as D;
    let base = D::COMBAT | D::SPELLS | D::ITEMS | D::WEIGHT | D::TIMED | D::HASTE_CANDIDATE;
    for (rule, mode, features) in [
        (MatchRuleset::PvpSkirmish, 1, base),
        (MatchRuleset::PvpSkirmish, 3, base),
        (MatchRuleset::PvpSkirmish, 5, base),
        (MatchRuleset::DeathmatchElimination, 1, base),
        (
            MatchRuleset::HonorTowers,
            5,
            base | D::HONOR | D::REGEN | D::LIGHT_ARROW,
        ),
        (MatchRuleset::NpcEncounter, 2, base | D::NPC),
    ] {
        let d = policy(rule, features);
        d.validate_haste_candidate(mode).unwrap();
        assert!(d.validate(mode).is_err());
        assert!(d.validate_canonical(mode).is_err());
        for missing in [
            D::COMBAT,
            D::SPELLS,
            D::ITEMS,
            D::WEIGHT,
            D::TIMED,
            D::HASTE_CANDIDATE,
        ] {
            assert!(policy(rule, features & !missing)
                .validate_haste_candidate(mode)
                .is_err());
        }
        assert!(policy(rule, features | (1 << 29))
            .validate_haste_candidate(mode)
            .is_err());
    }
    for (rule, mode, features) in [
        (MatchRuleset::SceneOnly, 1, base),
        (MatchRuleset::PetRace, 0, base | D::PET_RACE),
        (
            MatchRuleset::CooperativeTimeAttack,
            4,
            base | D::NPC | D::TIME_ATTACK | D::GREATER_HASTE,
        ),
        (MatchRuleset::SiegeObjectives, 3, base | D::SIEGE),
        (
            MatchRuleset::NpcEncounter,
            2,
            base | D::NPC | D::GREATER_HASTE,
        ),
    ] {
        let mut d = policy(rule, features);
        d.active_timeout_millis = 60_000;
        assert!(d.validate_haste_candidate(mode).is_err());
    }
    let old_npc = policy(
        MatchRuleset::NpcEncounter,
        (base & !(D::TIMED | D::HASTE_CANDIDATE)) | D::NPC,
    );
    old_npc.validate_canonical(2).unwrap();
    assert!(
        policy(MatchRuleset::NpcEncounter, old_npc.features | D::TIMED)
            .validate_canonical(2)
            .is_err()
    );
}
fn ack(capabilities: u32) -> p::Ack {
    p::Ack {
        request_id: 1,
        version: 15,
        numeric_schema: 1,
        capabilities,
    }
}
fn hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn parsed(bytes: &[u8]) -> Result<MatchDescriptor, app::ApplicationError> {
    let mut reader = Reader { bytes, pos: 0 };
    let descriptor = MatchDescriptor::decode(&mut reader)?;
    if reader.pos != bytes.len() {
        return Err(invalid());
    }
    descriptor.validate(1)?;
    Ok(descriptor)
}
fn scene_reply() -> Reply {
    let mut reply = tests::sample();
    reply.frame.as_mut().unwrap().descriptor = Some(scene());
    reply.validate().unwrap();
    reply
}

#[test]
fn per_binding_clocks_preserve_every_nano_boundary_on_the_existing_wire() {
    for (nanos, bytes) in [
        (0, "0000000000000000"),
        (1, "0000000000000001"),
        (3_000_000_001, "00000000b2d05e01"),
        (30_000_000_000, "00000006fc23ac00"),
    ] {
        let mut descriptor = scene();
        descriptor.countdown_nanos = nanos;
        descriptor.load_timeout_millis = 25_000;
        descriptor.active_timeout_millis = 6_000;
        let mut encoded = Vec::new();
        descriptor.encode(&mut encoded);
        assert_eq!(&encoded[16..24], hex(bytes).as_slice());
        assert_eq!(parsed(&encoded).unwrap(), descriptor);
        assert_eq!(encoded.len(), 32);
    }
}

#[test]
fn active_timeout_is_terminal_without_results_and_keeps_the_exact_frozen_clock() {
    let mut active = combat::tests::active_reply().world_reply().unwrap();
    active.frame.as_mut().unwrap().descriptor = Some(scene());
    active.validate().unwrap();
    let mut aborted = active.clone();
    let frame = aborted.frame.as_mut().unwrap();
    frame.phase = Phase::Aborted;
    frame.abort_reason = AbortReason::ActiveTimeout;
    frame.revision += 1;
    assert!(frame.completion.is_none());
    assert!(follows(Some(&active), &aborted));
    assert!(follows(Some(&aborted), &aborted));
    assert!(!follows(Some(&aborted), &active));
    let mut restarted = aborted.clone();
    let frame = restarted.frame.as_mut().unwrap();
    frame.revision += 1;
    frame.phase = Phase::Active;
    frame.abort_reason = AbortReason::None;
    restarted.validate().unwrap();
    assert!(!follows(Some(&aborted), &restarted));
    let mut changed = aborted.clone();
    changed
        .frame
        .as_mut()
        .unwrap()
        .descriptor
        .as_mut()
        .unwrap()
        .active_timeout_millis += 1;
    assert!(!follows(Some(&active), &changed));
    let mut fabricated_result = aborted;
    fabricated_result.frame.as_mut().unwrap().completion = tests::finished().completion;
    assert!(fabricated_result.validate().is_err());
}

#[test]
fn exact_independent_descriptor_32_bytes_and_nanos_are_not_milliseconds() {
    let literal = hex(concat!(
        "0001000000000000",
        "0000753000007530",
        "00000000b2d05e00",
        "000493e000000000"
    ));
    assert_eq!(literal.len(), 32);
    let descriptor = parsed(&literal).unwrap();
    assert_eq!(descriptor, scene());
    assert_eq!(descriptor.countdown_nanos, 3_000_000_000);
    assert!(!descriptor.combat() && !descriptor.results());
    let mut encoded = Vec::new();
    descriptor.encode(&mut encoded);
    assert_eq!(encoded, literal);
    for end in 0..literal.len() {
        assert!(parsed(&literal[..end]).is_err(), "prefix {end}");
    }
    for at in [0, 1, 2, 3, 4, 28, 29, 30, 31] {
        let mut bad = literal.clone();
        bad[at] = if at == 1 { 2 } else { 255 };
        assert!(parsed(&bad).is_err(), "schema/reserved {at}");
    }
    let mut tail = literal;
    tail.push(0);
    assert!(parsed(&tail).is_err());
}

#[test]
fn scalar_limits_are_closed_and_rules_never_guess_timeout_winners() {
    for (prepare, load, countdown, active) in
        [(50, 50, 0, 50), (30_000, 30_000, 30_000_000_000, 1_800_000)]
    {
        let mut value = scene();
        value.preparation_timeout_millis = prepare;
        value.load_timeout_millis = load;
        value.countdown_nanos = countdown;
        value.active_timeout_millis = active;
        value.validate(1).unwrap();
    }
    for case in 0..8 {
        let mut value = scene();
        match case {
            0 => value.preparation_timeout_millis = 49,
            1 => value.preparation_timeout_millis = 30_001,
            2 => value.load_timeout_millis = 49,
            3 => value.load_timeout_millis = 30_001,
            4 => value.countdown_nanos = 30_000_000_001,
            5 => value.active_timeout_millis = 49,
            6 => value.active_timeout_millis = 1_800_001,
            _ => value.features = 1024,
        }
        assert!(value.validate(1).is_err(), "scalar {case}");
    }
    for mode in 0..=5 {
        scene().validate(mode).unwrap();
    }
    assert!(scene().validate(6).is_err());
    for mode in 0..=5 {
        assert_eq!(
            policy(MatchRuleset::PvpSkirmish, 1).validate(mode).is_ok(),
            matches!(mode, 1 | 3 | 5)
        );
        assert_eq!(
            policy(MatchRuleset::DeathmatchElimination, 1)
                .validate(mode)
                .is_ok(),
            mode == 1
        );
        assert_eq!(
            policy(MatchRuleset::HonorTowers, 511)
                .validate(mode)
                .is_ok(),
            mode == 5
        );
    }
    assert!(!policy(MatchRuleset::PvpSkirmish, 1).results());
    assert!(policy(MatchRuleset::DeathmatchElimination, 1).results());
    assert!(policy(MatchRuleset::HonorTowers, 511).results());
    assert!(policy(MatchRuleset::SceneOnly, 1).validate(1).is_err());
    assert!(policy(MatchRuleset::PvpSkirmish, 511).validate(5).is_err());
    assert!(policy(MatchRuleset::DeathmatchElimination, 511)
        .validate(1)
        .is_err());
    assert!(policy(MatchRuleset::HonorTowers, 255).validate(5).is_err());
}

#[test]
fn independent_feature_dependencies_and_binding_modes_are_exact() {
    use MatchDescriptor as M;
    for (feature, dependency) in [
        (M::SPELLS, M::COMBAT),
        (M::LIGHT_ARROW, M::SPELLS),
        (M::ITEMS, M::COMBAT),
        (M::WEIGHT, M::ITEMS),
        (M::TIMED, M::WEIGHT),
        (M::AMMO, M::TIMED),
        (M::REGEN, M::WEIGHT),
        (M::HONOR, M::REGEN),
        (M::HONOR, M::LIGHT_ARROW),
        (M::FIRE_STORM, M::REGEN),
        (M::FIRE_STORM, M::LIGHT_ARROW),
    ] {
        let mut value = policy(MatchRuleset::HonorTowers, 1023);
        assert!(value.has(feature));
        value.features &= !dependency;
        assert!(
            value.validate(5).is_err(),
            "feature {feature} dependency {dependency}"
        );
    }
    policy(MatchRuleset::HonorTowers, 1023).validate(5).unwrap();
    policy(MatchRuleset::DeathmatchElimination, 1023 & !M::HONOR)
        .validate(1)
        .unwrap();
    assert!(policy(MatchRuleset::PvpSkirmish, 1023 & !M::HONOR)
        .validate(3)
        .is_err());
    assert!(policy(MatchRuleset::HonorTowers, 1023).validate(1).is_err());
}

#[test]
fn v15_world_has_only_one_optional_40_byte_appendix_and_old_shapes_stay_exact() {
    let old = tests::sample();
    let v14 = tests::wire_complete_version(&old, 14);
    for version in 7..=14 {
        let wire = tests::wire_complete_version(&old, version);
        assert_eq!(wire, v14);
        assert_eq!(decode_body_version(&wire[16..], version).unwrap(), old);
    }
    let off = tests::wire_complete_version(&old, 15);
    assert_eq!(off.len(), v14.len() + 8);
    assert_eq!(&off[8..v14.len()], &v14[8..]);
    assert_eq!(&off[v14.len()..], &[0; 8]);
    assert_eq!(decode_body_version(&off[16..], 15).unwrap(), old);
    assert!(decode_body_version(&v14[16..], 15).is_err());
    assert!(decode_body_version(&off[16..], 14).is_err());
    let next = scene_reply();
    let wire = tests::wire_complete_version(&next, 15);
    assert_eq!(wire.len(), v14.len() + 40);
    assert_eq!(&wire[v14.len()..v14.len() + 8], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(decode_body_version(&wire[16..], 15).unwrap(), next);
    for end in v14.len()..wire.len() {
        assert!(decode_body_version(&wire[16..end], 15).is_err());
    }
    for offset in 0..8 {
        let mut bad = wire.clone();
        bad[v14.len() + offset] = 2;
        assert!(decode_body_version(&bad[16..], 15).is_err());
    }
    let mut extra = wire;
    extra.push(0);
    assert!(decode_body_version(&extra[16..], 15).is_err());
}

#[test]
fn descriptor_presence_is_negotiated_not_guessed_from_unavailable_or_global_combat() {
    let all = ack(p::KNOWN_CAPABILITIES);
    let off = ack(p::KNOWN_CAPABILITIES & !p::CAP_MATCH_DESCRIPTOR);
    let legacy = p::Ack { version: 14, ..off };
    let old = app::Response::World(tests::sample());
    let current = app::Response::World(scene_reply());
    assert!(!descriptor_negotiated(&old, &all));
    assert!(descriptor_negotiated(&old, &off));
    assert!(descriptor_negotiated(&old, &legacy));
    assert!(descriptor_negotiated(&current, &all));
    assert!(!descriptor_negotiated(&current, &off));
    assert!(!descriptor_negotiated(&current, &legacy));
    let minimal = ack(7 | p::CAP_ROOMS | p::CAP_WORLD | p::CAP_MATCH_DESCRIPTOR);
    assert!(descriptor_negotiated(&current, &minimal));
    let mut combat = combat::tests::active_reply();
    combat.frame.as_mut().unwrap().world.descriptor = Some(policy(MatchRuleset::PvpSkirmish, 1));
    assert!(!descriptor_negotiated(
        &app::Response::Combat(combat.clone()),
        &minimal
    ));
    assert!(descriptor_negotiated(
        &app::Response::Combat(combat.clone()),
        &all
    ));
    let unavailable = combat::Reply {
        frame: None,
        status: combat::Status::Unavailable,
        reason: combat::Reason::NotActive,
        ..combat.clone()
    };
    assert!(descriptor_negotiated(
        &app::Response::Combat(unavailable),
        &all
    ));
    assert!(combat.frame.as_ref().unwrap().world.combat_enabled(false));
    assert!(!scene_reply().frame.unwrap().combat_enabled(true));
}

#[test]
fn same_world_descriptor_cannot_change_or_disappear_at_any_new_revision() {
    let mut base = combat::tests::active_reply().world_reply().unwrap();
    base.frame.as_mut().unwrap().descriptor = Some(scene());
    base.validate().unwrap();
    assert!(follows(Some(&base), &base));
    for case in 0..7 {
        let mut next = base.clone();
        let f = next.frame.as_mut().unwrap();
        f.revision += 1;
        match case {
            0 => f.descriptor = None,
            1 => f.descriptor.as_mut().unwrap().preparation_timeout_millis -= 1,
            2 => f.descriptor.as_mut().unwrap().load_timeout_millis -= 1,
            3 => f.descriptor.as_mut().unwrap().countdown_nanos += 1,
            4 => f.descriptor.as_mut().unwrap().active_timeout_millis += 1,
            5 => f.descriptor = Some(policy(MatchRuleset::PvpSkirmish, 1)),
            _ => f.descriptor = Some(policy(MatchRuleset::DeathmatchElimination, 1)),
        }
        next.validate().unwrap();
        assert!(!follows(Some(&base), &next), "mutation {case}");
    }
    let mut preparing = base.clone();
    preparing.frame.as_mut().unwrap().phase = Phase::Preparing;
    for a in &mut preparing.frame.as_mut().unwrap().actors {
        a.loaded = false;
        a.sequence = 0;
    }
    preparing.validate().unwrap();
    let mut loading = preparing.clone();
    loading.frame.as_mut().unwrap().phase = Phase::Loading;
    loading.frame.as_mut().unwrap().revision += 1;
    assert!(follows(Some(&preparing), &loading));
    let mut active = loading.clone();
    active.frame.as_mut().unwrap().phase = Phase::Active;
    active.frame.as_mut().unwrap().revision += 1;
    for a in &mut active.frame.as_mut().unwrap().actors {
        a.loaded = true;
    }
    active.validate().unwrap();
    // CLOCK and features are policy, never an optimistic cast permission.
    assert_eq!(
        active.frame.as_ref().unwrap().descriptor,
        preparing.frame.as_ref().unwrap().descriptor
    );
}

#[test]
fn only_result_rulesets_admit_existing_completion_and_terminal_policy_stays_fixed() {
    let mut finished = tests::finished();
    finished.descriptor = Some(policy(MatchRuleset::DeathmatchElimination, 1));
    finished.validate(finished.actors[0].key).unwrap();
    for d in [scene(), policy(MatchRuleset::PvpSkirmish, 1)] {
        let mut bad = finished.clone();
        bad.descriptor = Some(d);
        assert!(bad.validate(bad.actors[0].key).is_err());
    }
    let prior = Reply {
        echo: Request::read(tests::sample().echo.context, 3),
        status: Status::Ok,
        reason: Reason::None,
        frame: Some(finished),
    };
    let mut saved = prior.clone();
    let f = saved.frame.as_mut().unwrap();
    f.completion.as_mut().unwrap().state = SettlementState::Saved;
    f.revision += 1;
    assert!(follows(Some(&prior), &saved));
    saved
        .frame
        .as_mut()
        .unwrap()
        .descriptor
        .as_mut()
        .unwrap()
        .active_timeout_millis += 1;
    assert!(!follows(Some(&prior), &saved));
}

#[test]
fn combat_only_with_global_full_capabilities_uses_exact_per_match_empty_appendices() {
    let mut reply = combat::tests::active_reply();
    reply.frame.as_mut().unwrap().world.descriptor = Some(policy(MatchRuleset::PvpSkirmish, 1));
    reply.validate().unwrap();
    let response = app::Response::Combat(reply.clone());
    assert!(crate::modern_lito::weight_protocol::negotiated(
        &response, true
    ));
    assert!(crate::modern_lito::timed_protocol::negotiated(
        &response, true
    ));
    assert!(crate::modern_lito::ammo_protocol::negotiated(
        &response, true
    ));
    assert!(regen::negotiated(&response, true));
    assert!(crate::modern_lito::honor_protocol::negotiated(
        &response, true
    ));
    assert!(crate::modern_lito::fire_storm_protocol::negotiated(
        &response, true
    ));
    let mut off = reply.clone();
    off.frame.as_mut().unwrap().world.descriptor = Some(scene());
    assert!(off.validate().is_err()); // SceneOnly is WORLD, not an empty COMBAT fallback.
    let mut legacy = reply;
    legacy.frame.as_mut().unwrap().world.descriptor = None;
    let response = app::Response::Combat(legacy);
    assert!(!crate::modern_lito::weight_protocol::negotiated(
        &response, true
    ));
    assert!(!regen::negotiated(&response, true)); // legacy homogeneous promises remain strict.
}

#[test]
fn full_feature_vectors_must_match_descriptor_and_v15_nested_world_position_is_exact() {
    use MatchDescriptor as M;
    let mut reply = regen::tests::initial();
    let features = M::COMBAT | M::SPELLS | M::ITEMS | M::WEIGHT | M::TIMED | M::REGEN;
    reply.frame.as_mut().unwrap().world.descriptor =
        Some(policy(MatchRuleset::PvpSkirmish, features));
    reply.validate().unwrap();
    let wire = regen::tests::frame_wire(&reply, 15);
    assert_eq!(
        app::decode_response_version(&wire, 15).unwrap().response,
        app::Response::Combat(reply.clone())
    );
    assert!(app::decode_response_version(&wire, 14).is_err());
    for case in 0..8 {
        let mut bad = reply.clone();
        let f = bad.frame.as_mut().unwrap();
        match case {
            0 => f.world.casts.clear(),
            1 => f.items.clear(),
            2 => f.weights.clear(),
            3 => f.timed.clear(),
            4 => f.regen.clear(),
            5 => f.world.descriptor.as_mut().unwrap().features |= M::LIGHT_ARROW,
            6 => f.world.descriptor.as_mut().unwrap().features |= M::AMMO,
            _ => f.world.descriptor.as_mut().unwrap().features &= !M::REGEN,
        }
        assert!(bad.validate().is_err(), "shape {case}");
    }
    let response = app::Response::Combat(reply.clone());
    assert!(regen::negotiated(&response, true));
    assert!(!regen::negotiated(&response, false));
    let mut changed = reply.clone();
    changed.frame.as_mut().unwrap().world.revision += 1;
    changed
        .frame
        .as_mut()
        .unwrap()
        .world
        .descriptor
        .as_mut()
        .unwrap()
        .features &= !M::REGEN;
    changed.frame.as_mut().unwrap().regen.clear();
    assert!(!combat::follows(Some(&reply), &changed));
    assert_eq!(combat::frame_limit(14), 1024 * 1024);
    assert_eq!(combat::frame_limit(15), 1024 * 1024);
}

#[test]
fn descriptor_capability_requires_only_world_and_is_strictly_v15_optional() {
    let minimal = 7 | p::CAP_ROOMS | p::CAP_WORLD | p::CAP_MATCH_DESCRIPTOR;
    let offered = ack(minimal);
    p::encode(p::Message::Ack(offered)).unwrap();
    assert_eq!(minimal & p::CAP_COMBAT, 0);
    for version in 1..=14 {
        assert!(p::encode(p::Message::Ack(p::Ack { version, ..offered })).is_err());
    }
    assert!(p::encode(p::Message::Ack(ack(minimal & !p::CAP_WORLD))).is_err());
    p::encode(p::Message::Ack(ack(minimal & !p::CAP_MATCH_DESCRIPTOR))).unwrap();
    assert!(p::encode(p::Message::Ack(ack(p::KNOWN_CAPABILITIES | 131072))).is_err());
    let mut client = p::ModernClientSession::new(p::Hello::current(1)).unwrap();
    client.begin().unwrap();
    assert_eq!(
        client
            .accept_ack(&p::encode(p::Message::Ack(offered)).unwrap())
            .unwrap(),
        offered
    );
}

#[test]
fn scene_only_does_not_remove_selection_shop_or_equipment_capability_promises() {
    use crate::modern_lito::{
        ammo_protocol as ammo, equipment, shop_protocol as shop, timed_protocol as timed,
        weight_protocol as weight,
    };
    use std::sync::Arc;
    let world = scene_reply();
    let context = world.echo.context;
    let empty_weight = weight::Snapshot {
        source_gear_weight: 0,
        supplies_weight: 0,
        total: 0,
        maximum_bits: 1000f64.to_bits(),
        stage: 0,
        hit_penalty: 0,
        units: vec![],
    };
    let mut shop = shop::Reply {
        echo: shop::Request::read(context, 0, None),
        status: shop::Status::Ok,
        reason: shop::Reason::None,
        snapshot: Some(Arc::new(shop::Snapshot {
            round_id: 1,
            revision: 0,
            allowance: 1000,
            balance: 1000,
            token: [7; 32],
            offers: vec![],
            stacks: vec![],
            weight: Some(empty_weight.clone()),
            timed: Some(shop::TimedCatalog {
                owner_class: 0,
                offers: vec![],
            }),
            ammo: Some(vec![]),
            materials_enabled: false,
        })),
        room: Some(rooms::tests::sample(context, false)),
    };
    shop.validate().unwrap();
    let response = app::Response::Shop(shop.clone());
    assert!(!world
        .frame
        .as_ref()
        .unwrap()
        .feature_enabled(MatchDescriptor::ITEMS, true));
    assert!(descriptor_negotiated(
        &response,
        &ack(p::KNOWN_CAPABILITIES)
    ));
    for verify in [weight::negotiated, timed::negotiated, ammo::negotiated] {
        assert!(verify(&response, true));
        assert!(!verify(&response, false));
    }
    let snapshot = Arc::make_mut(shop.snapshot.as_mut().unwrap());
    snapshot.weight = None;
    snapshot.timed = None;
    snapshot.ammo = None;
    let response = app::Response::Shop(shop);
    for verify in [weight::negotiated, timed::negotiated, ammo::negotiated] {
        assert!(!verify(&response, true));
        assert!(verify(&response, false));
    }
    let wire = equipment::tests::fixture(
        0,
        equipment::Request::read(context),
        equipment::Status::Ok,
        false,
    );
    let app::Response::Equipment(mut gear) =
        app::decode_response_version(&wire, 1).unwrap().response
    else {
        panic!("equipment fixture");
    };
    let snapshot = Arc::make_mut(gear.snapshot.as_mut().unwrap());
    snapshot.lifetimes = Some(vec![]);
    snapshot.weight = Some(empty_weight);
    gear.validate().unwrap();
    let response = app::Response::Equipment(gear.clone());
    assert!(descriptor_negotiated(
        &response,
        &ack(p::KNOWN_CAPABILITIES)
    ));
    assert!(weight::negotiated(&response, true));
    assert!(!weight::negotiated(&response, false));
    Arc::make_mut(gear.snapshot.as_mut().unwrap()).weight = None;
    assert!(!weight::negotiated(&app::Response::Equipment(gear), true));
}
