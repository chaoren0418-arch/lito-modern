use super::*;

fn material_read() -> Reply {
    let mut r = weighted(read());
    let s = Arc::make_mut(r.snapshot.as_mut().unwrap());
    s.offers = vec![Offer {
        item_id: 40318,
        sort: 0,
        support: Support::Material,
        buy_price: 100,
        sell_price: 50,
        inventory_graphic: 235,
        ground_graphic: 773,
        weight: 2000,
        name: "$508".into(),
        healing: None,
    }];
    s.timed = Some(TimedCatalog {
        owner_class: 1,
        offers: vec![],
    });
    s.materials_enabled = true;
    r.validate().unwrap();
    r
}
fn material_wire(r: &Reply) -> Vec<u8> {
    let mut b = v9frame_version(r, 47);
    let s = r.snapshot.as_ref().unwrap();
    assert!(s.ammo.is_none());
    if let Some(timed) = &s.timed {
        assert!(timed.offers.is_empty());
        b.extend([1, timed.owner_class, 0, 0, 0, 0, 0, 0]);
    } else {
        b.extend([0; 8]);
    }
    b.extend([0; 8]);
    b.push(s.materials_enabled as u8);
    b.extend([0; 7]);
    let len = b.len() as u32;
    b[4..8].copy_from_slice(&len.to_be_bytes());
    b
}
#[test]
fn v47_material_shop_appendix_is_exact_and_never_leaks_into_older_versions() {
    for reply in [material_read(), weighted(read())] {
        let wire = material_wire(&reply);
        assert_eq!(
            app::decode_response_version(&wire, 47).unwrap().response,
            app::Response::Shop(reply)
        );
        for version in 8..47 {
            assert!(app::decode_response_version(&wire, version).is_err());
        }
        for n in 0..wire.len() {
            assert!(app::decode_response_version(&wire[..n], 47).is_err());
        }
        let start = wire.len() - 8;
        for at in start..wire.len() {
            let mut bad = wire.clone();
            bad[at] = if at == start { 2 } else { 1 };
            assert!(app::decode_response_version(&bad, 47).is_err());
        }
        let mut tail = wire;
        tail.push(0);
        let len = tail.len() as u32;
        tail[4..8].copy_from_slice(&len.to_be_bytes());
        assert!(app::decode_response_version(&tail, 47).is_err());
    }
}
#[test]
fn material_shop_needs_qualified_gem_timed_and_weight_state() {
    assert!(material_read().snapshot.unwrap().purchasable(40318));
    for case in 0..8 {
        let mut r = material_read();
        let s = Arc::make_mut(r.snapshot.as_mut().unwrap());
        match case {
            0 => s.materials_enabled = false,
            1 => s.timed = None,
            2 => s.weight = None,
            3 => s.offers[0].item_id = 40319,
            4 => s.offers[0].inventory_graphic += 1,
            5 => s.offers[0].ground_graphic += 1,
            6 => s.offers[0].name = "unqualified".into(),
            _ => s.offers[0].weight = -1,
        }
        assert!(r.validate().is_err(), "case {case}");
    }
}
#[test]
fn successful_purchase_cannot_toggle_material_support_and_overweight_does_not_spend() {
    let (prior, request, after) = buy();
    let mut prior = weighted(prior);
    let mut after = weighted(after);
    for reply in [&mut prior, &mut after] {
        let s = Arc::make_mut(reply.snapshot.as_mut().unwrap());
        s.timed = Some(TimedCatalog {
            owner_class: 1,
            offers: vec![],
        });
        s.materials_enabled = true;
    }
    assert!(after.confirms(&request, Some(&prior), prior.room.as_ref()));
    Arc::make_mut(after.snapshot.as_mut().unwrap()).materials_enabled = false;
    after.validate().unwrap();
    assert!(!after.confirms(&request, Some(&prior), prior.room.as_ref()));
    let mut prior = material_read();
    let w = Arc::make_mut(prior.snapshot.as_mut().unwrap())
        .weight
        .as_mut()
        .unwrap();
    w.source_gear_weight = 999;
    w.total = 999;
    w.stage = 240;
    w.hit_penalty = 0;
    prior.validate().unwrap();
    let mut denied = prior.clone();
    denied.echo = prior
        .echo
        .mutation(prior.snapshot.as_ref().unwrap(), Action::Buy, 40318, 1);
    denied.status = Status::Denied;
    denied.reason = Reason::TooHeavy;
    denied.validate().unwrap();
    assert!(denied.confirms(&denied.echo, Some(&prior), prior.room.as_ref()));
    assert_eq!(denied.snapshot, prior.snapshot);
    assert_eq!(
        app::decode_response_version(&material_wire(&denied), 47)
            .unwrap()
            .response,
        app::Response::Shop(denied)
    );
}

#[test]
fn clearing_a_shop_view_preserves_negotiated_support_and_never_fabricates_a_wallet() {
    for supported in [false, true] {
        let state = State::negotiated(supported);
        assert_eq!(
            state,
            if supported {
                State::NotRequested
            } else {
                State::Unsupported
            }
        );
        assert!(state.reply().is_none() && state.snapshot().is_none());
    }
}

const GOLDEN_BODY: &str = concat!(
    "000000070000000000000d05ffffffffffffffff0200000001000000",
    "00000000000000000000000000000000",
    "00000000000000020000000000000003",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "0000000000009c4a0000000000000003"
);

fn arrow_read() -> Reply {
    let mut r = read();
    let s = Arc::make_mut(r.snapshot.as_mut().unwrap());
    s.offers.clear();
    let mut facts = vec![];
    for (i, id) in crate::modern_lito::ammo_protocol::UI_ITEMS
        .into_iter()
        .enumerate()
    {
        let mut o = offer();
        o.item_id = id;
        o.sort = i as u16;
        o.support = Support::Ammo;
        o.healing = None;
        o.name = format!("Arrow {id}");
        o.inventory_graphic = 96;
        o.buy_price = 2;
        o.sell_price = 1;
        s.offers.push(o);
        facts.push(crate::modern_lito::ammo_protocol::Facts {
            item_id: id,
            small: 3,
            large: 4,
            raw_arrow_sprite: -1,
            consume: true,
            material: i as u8,
            blessing: 0,
        });
    }
    s.ammo = Some(facts);
    r.validate().unwrap();
    r
}
fn arrow_wire(r: &Reply) -> Vec<u8> {
    let mut wire = frame(r);
    wire.extend([0; 16]);
    let ammo = r.snapshot.as_ref().unwrap().ammo.as_ref();
    wire.extend([ammo.is_some() as u8, 0, 0, 0]);
    wire.extend((ammo.map_or(0, Vec::len) as u16).to_be_bytes());
    wire.extend([0; 2]);
    if let Some(facts) = ammo {
        for f in facts {
            for n in [f.item_id, f.small, f.large] {
                wire.extend(n.to_be_bytes());
            }
            wire.extend(f.raw_arrow_sprite.to_be_bytes());
            wire.extend([f.consume as u8, f.material, f.blessing, 0]);
        }
    }
    let n = wire.len() as u32;
    wire[4..8].copy_from_slice(&n.to_be_bytes());
    wire
}
#[test]
fn v11_shop_arrow_facts_exact_order_old_versions_reject_and_tail_is_bounded() {
    let r = arrow_read();
    let wire = arrow_wire(&r);
    let app::Response::Shop(decoded) = app::decode_response_version(&wire, 11).unwrap().response
    else {
        panic!("shop")
    };
    assert_eq!(decoded, r);
    for version in 8..=10 {
        assert!(app::decode_response_version(&wire, version).is_err());
    }
    for n in 0..wire.len() {
        assert!(app::decode_response_version(&wire[..n], 11).is_err());
    }
    let offset = wire.len() - 72;
    for at in [
        offset + 1,
        offset + 2,
        offset + 3,
        offset + 6,
        offset + 7,
        offset + 8 + 31,
    ] {
        let mut bad = wire.clone();
        bad[at] = 1;
        assert!(
            app::decode_response_version(&bad, 11).is_err(),
            "padding {at}"
        );
    }
    for case in 0..4 {
        let mut bad = r.clone();
        let s = Arc::make_mut(bad.snapshot.as_mut().unwrap());
        match case {
            0 => s.ammo.as_mut().unwrap().reverse(),
            1 => {
                s.ammo.as_mut().unwrap().pop();
            }
            2 => s.ammo = None,
            _ => s.ammo.as_mut().unwrap()[0].raw_arrow_sprite = 67,
        }
        assert!(bad.validate().is_err());
    }
}
#[test]
fn v11_arrow_purchase_uses_normal_wallet_cas_and_cannot_change_damage_facts() {
    let before = arrow_read();
    let request = before
        .echo
        .mutation(before.snapshot.as_deref().unwrap(), Action::Buy, 40744, 2);
    let mut after = before.clone();
    after.echo = request;
    let s = Arc::make_mut(after.snapshot.as_mut().unwrap());
    s.revision += 1;
    s.balance -= 4;
    s.stacks.push(Stack {
        item_id: 40744,
        quantity: 2,
        unit_paid_price: 2,
    });
    assert!(after.confirms(&request, Some(&before), before.room.as_ref()));
    assert!(after.snapshot.as_ref().unwrap().purchasable(40744));
    let mut bad = after.clone();
    Arc::make_mut(bad.snapshot.as_mut().unwrap())
        .ammo
        .as_mut()
        .unwrap()[0]
        .small += 1;
    assert!(bad.validate().is_ok());
    assert!(!bad.confirms(&request, Some(&before), before.room.as_ref()));
    let mut failed = State::default();
    failed.receive(before.clone()).unwrap();
    let captured = failed.clone();
    let _request = encode_request(9, request, 11).unwrap();
    assert_eq!(failed, captured);
}

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn context() -> SelectionContext {
    SelectionContext {
        key: app::CharacterKey {
            source_id: 7,
            object_id: 3333,
        },
        epoch: u64::MAX,
    }
}

fn offer() -> Offer {
    Offer {
        item_id: 40010,
        sort: 0,
        support: Support::Healing,
        buy_price: 40,
        sell_price: 40,
        inventory_graphic: 80,
        ground_graphic: 45,
        weight: 3900,
        name: "$26".into(),
        healing: Some(Healing {
            minimum: 10,
            maximum: 20,
            effect: 189,
            delay_group: 2,
            delay_millis: 500,
        }),
    }
}

fn snapshot() -> Snapshot {
    Snapshot {
        round_id: 1,
        revision: 0,
        allowance: 1000,
        balance: 1000,
        token: [7; 32],
        offers: vec![offer()],
        stacks: vec![],
        weight: None,
        timed: None,
        ammo: None,
        materials_enabled: false,
    }
}

fn read() -> Reply {
    Reply {
        echo: Request::read(context(), 0, None),
        status: Status::Ok,
        reason: Reason::None,
        snapshot: Some(Arc::new(snapshot())),
        room: Some(rooms::tests::sample(context(), false)),
    }
}

fn buy() -> (Reply, Request, Reply) {
    let prior = read();
    let request = prior
        .echo
        .mutation(prior.snapshot.as_deref().unwrap(), Action::Buy, 40010, 2);
    let mut next = snapshot();
    next.revision = 1;
    next.balance = 920;
    next.stacks.push(Stack {
        item_id: 40010,
        quantity: 2,
        unit_paid_price: 40,
    });
    let after = Reply {
        echo: request,
        status: Status::Ok,
        reason: Reason::None,
        snapshot: Some(Arc::new(next)),
        room: prior.room.clone(),
    };
    (prior, request, after)
}

/// Fixture serializer only. Echo has a separate verbatim Java golden below;
/// nested rooms use their already-independent typed fixture grammar.
fn frame(reply: &Reply) -> Vec<u8> {
    frame_version(reply, 8)
}
fn frame_version(reply: &Reply, version: u16) -> Vec<u8> {
    let mut out = vec![];
    put_request(&mut out, reply.echo);
    out.extend([
        reply.status as u8,
        reply.reason as u8,
        reply.snapshot.is_some() as u8,
        0,
    ]);
    if let Some(s) = &reply.snapshot {
        for n in [s.round_id, s.revision, s.allowance, s.balance] {
            out.extend(n.to_be_bytes());
        }
        out.extend(s.token);
        out.extend((s.offers.len() as u16).to_be_bytes());
        out.extend((s.stacks.len() as u16).to_be_bytes());
        for o in &s.offers {
            out.extend(o.item_id.to_be_bytes());
            out.extend(o.sort.to_be_bytes());
            out.extend([o.support as u8, 0]);
            for n in [
                o.buy_price,
                o.sell_price,
                o.inventory_graphic,
                o.ground_graphic,
                o.weight,
            ] {
                out.extend(n.to_be_bytes());
            }
            out.extend((o.name.len() as u16).to_be_bytes());
            out.extend(o.name.as_bytes());
            out.extend([o.healing.is_some() as u8, 0, 0, 0]);
            if let Some(h) = &o.healing {
                for n in [
                    h.minimum,
                    h.maximum,
                    h.effect,
                    h.delay_group,
                    h.delay_millis,
                ] {
                    out.extend(n.to_be_bytes());
                }
            }
        }
        for s in &s.stacks {
            for n in [s.item_id, s.quantity, s.unit_paid_price] {
                out.extend(n.to_be_bytes());
            }
        }
        let nested = rooms::tests::frame_version(reply.room.as_ref().unwrap(), version);
        out.extend(((nested.len() - 16) as u32).to_be_bytes());
        out.extend(&nested[16..]);
    }
    let mut header = app::header(9, RESULT_KIND, 16 + out.len()).unwrap();
    header.extend(out);
    header
}

fn decoded(bytes: &[u8]) -> Result<Reply, ApplicationError> {
    match app::decode_response_version(bytes, 8)?.response {
        app::Response::Shop(reply) => Ok(reply),
        _ => panic!("not a shop fixture"),
    }
}

fn weighted(mut reply: Reply) -> Reply {
    use crate::modern_lito::weight_protocol as weight;
    let s = Arc::make_mut(reply.snapshot.as_mut().unwrap());
    let units: Vec<_> = s
        .stacks
        .iter()
        .map(|s| weight::UnitWeight {
            item_id: s.item_id,
            unit_weight: 3900,
        })
        .collect();
    let supplies = s
        .stacks
        .iter()
        .map(|s| weight::stack_weight(s.quantity, 3900).unwrap())
        .sum::<i64>();
    let total = 100 + supplies;
    let stage = weight::stage(total, 1000.0).unwrap();
    s.weight = Some(weight::Snapshot {
        source_gear_weight: 100,
        supplies_weight: supplies,
        total,
        maximum_bits: 1000f64.to_bits(),
        stage,
        hit_penalty: weight::penalty(stage),
        units,
    });
    reply.validate().unwrap();
    reply
}
fn v9frame(reply: &Reply) -> Vec<u8> {
    v9frame_version(reply, 9)
}
fn v9frame_version(reply: &Reply, version: u16) -> Vec<u8> {
    let mut wire = frame_version(reply, version);
    let weight = reply.snapshot.as_ref().and_then(|s| s.weight.as_ref());
    wire.extend([weight.is_some() as u8, 0, 0, 0, 0, 0, 0, 0]);
    if let Some(w) = weight {
        wire.extend(crate::modern_lito::weight_protocol::tests::encode(w));
    }
    let n = wire.len() as u32;
    wire[4..8].copy_from_slice(&n.to_be_bytes());
    wire
}
#[test]
fn v9_shop_weight_follows_complete_room_body_and_is_required_only_when_negotiated() {
    use crate::modern_lito::weight_protocol as weight;
    let (prior, request, after) = buy();
    let prior = weighted(prior);
    let after = weighted(after);
    assert!(after.confirms(&request, Some(&prior), prior.room.as_ref()));
    let wire = v9frame(&after);
    assert_eq!(
        app::decode_response_version(&wire, 9).unwrap().response,
        app::Response::Shop(after.clone())
    );
    assert!(app::decode_response_version(&wire, 8).is_err());
    assert!(app::decode_response_version(&frame(&after), 9).is_err());
    assert!(weight::negotiated(
        &app::Response::Shop(after.clone()),
        true
    ));
    assert!(!weight::negotiated(&app::Response::Shop(after), false));
    let disabled = read();
    assert!(app::decode_response_version(&v9frame(&disabled), 9).is_ok());
    assert!(!weight::negotiated(&app::Response::Shop(disabled), true));
    for n in wire.len() - 64..wire.len() {
        assert!(app::decode_response_version(&wire[..n], 9).is_err());
    }
}
#[test]
fn purchased_unit_weight_survives_catalog_refresh_and_sale_while_new_buy_uses_offer_weight() {
    let (prior, request, after) = buy();
    let prior = weighted(prior);
    let mut after = weighted(after);
    assert!(after.confirms(&request, Some(&prior), prior.room.as_ref()));
    // Fresh catalog may differ; old owned units remain 3900, not the new 9000.
    {
        let s = Arc::make_mut(after.snapshot.as_mut().unwrap());
        s.revision += 1;
        s.offers[0].weight = 9000;
    }
    after.echo = Request::read(context(), 0, None);
    after.validate().unwrap();
    let request = after
        .echo
        .mutation(after.snapshot.as_ref().unwrap(), Action::Sell, 40010, 1);
    let mut sold = after.clone();
    sold.echo = request;
    {
        let s = Arc::make_mut(sold.snapshot.as_mut().unwrap());
        s.revision += 1;
        s.balance += 40;
        s.stacks[0].quantity = 1;
        let w = s.weight.as_mut().unwrap();
        w.supplies_weight = 3;
        w.total = 103;
        w.stage = 25;
        w.hit_penalty = 0;
    }
    sold.validate().unwrap();
    assert!(sold.confirms(&request, Some(&after), after.room.as_ref()));
    assert_eq!(
        sold.snapshot
            .as_ref()
            .unwrap()
            .weight
            .as_ref()
            .unwrap()
            .units[0]
            .unit_weight,
        3900
    );
    let mut fake = sold;
    let s = Arc::make_mut(fake.snapshot.as_mut().unwrap());
    let w = s.weight.as_mut().unwrap();
    w.units[0].unit_weight = 9000;
    w.supplies_weight = 9;
    w.total = 109;
    w.stage = 26;
    fake.validate().unwrap();
    assert!(!fake.confirms(&request, Some(&after), after.room.as_ref()));
}

#[test]
fn overweight_denial_is_v9_only_and_never_predicts_a_purchase() {
    let mut prior = weighted(read());
    let weight = Arc::make_mut(prior.snapshot.as_mut().unwrap())
        .weight
        .as_mut()
        .unwrap();
    weight.source_gear_weight = 999;
    weight.total = 999;
    weight.stage = 240;
    weight.hit_penalty = 0;
    prior.validate().unwrap();
    let mut denied = prior.clone();
    denied.echo = prior
        .echo
        .mutation(prior.snapshot.as_ref().unwrap(), Action::Buy, 40010, 1);
    denied.status = Status::Denied;
    denied.reason = Reason::TooHeavy;
    denied.validate().unwrap();
    assert!(denied.confirms(&denied.echo, Some(&prior), prior.room.as_ref()));
    assert_eq!(denied.snapshot, prior.snapshot);
    assert!(app::decode_response_version(&v9frame(&denied), 9).is_ok());
    assert!(app::decode_response_version(&frame(&denied), 8).is_err());
    assert!(denied.notice().contains("未扣款"));
    for case in 0..8 {
        let mut bad = denied.clone();
        match case {
            0 => bad.echo.action = Action::Sell,
            1 => Arc::make_mut(bad.snapshot.as_mut().unwrap()).weight = None,
            2 => bad.echo.item_id = 40011,
            3 => {
                Arc::make_mut(bad.snapshot.as_mut().unwrap()).weight =
                    weighted(read()).snapshot.as_ref().unwrap().weight.clone()
            }
            4 => bad.echo.round_id += 1,
            5 => bad.echo.expected_revision += 1,
            6 => bad.echo.token[0] ^= 1,
            _ => {
                let s = Arc::make_mut(bad.snapshot.as_mut().unwrap());
                let o = &mut s.offers[0];
                o.support = Support::Unimplemented;
                o.healing = None;
            }
        }
        assert!(bad.validate().is_err(), "forged overweight case {case}");
    }
    assert!(!crate::modern_lito::weight_protocol::negotiated(
        &app::Response::Shop(denied),
        false
    ));
}

#[test]
fn java_verbatim_buy_and_busy_goldens_preserve_signed_widths_and_exact_echo() {
    let request = Request {
        context: context(),
        channel_id: 2,
        action: Action::Buy,
        room_id: 0,
        expected_room_revision: 0,
        round_id: 2,
        expected_revision: 3,
        token: [0xaa; 32],
        item_id: 40010,
        quantity: 3,
    };
    let expected = hex(&format!("4c544d310000007cfedcba9800200000{GOLDEN_BODY}"));
    assert_eq!(expected.len(), 124);
    assert_eq!(encode_request(0xfedcba98, request, 8).unwrap(), expected);
    let busy = hex(&format!(
        "4c544d3100000080fedcba9800210000{GOLDEN_BODY}030b0000"
    ));
    let parsed = app::decode_response_version(&busy, 8).unwrap();
    assert_eq!(parsed.request_id, 0xfedcba98);
    let app::Response::Shop(parsed) = parsed.response else {
        panic!("wrong response")
    };
    assert_eq!(parsed.echo, request);
    assert_eq!(parsed.status, Status::Busy);
    assert_eq!(parsed.reason, Reason::Busy);
    assert!(parsed.snapshot.is_none());
    assert!(parsed.room.is_none());
}

#[test]
fn java_absent_shop_snapshots_have_no_appendix_in_versions_eight_nine_or_ten() {
    // Independent Java putResult shape: 16-byte LTM1 header, 108-byte echo,
    // then status/reason/present/reserved. No DTO serializer or appendix.
    for (suffix, status, reason) in [
        ("030b0000", Status::Busy, Reason::Busy),
        ("040a0000", Status::Unavailable, Reason::Backend),
        ("05090000", Status::Frozen, Reason::Frozen),
    ] {
        let bytes = hex(&format!(
            "4c544d3100000080fedcba9800210000{GOLDEN_BODY}{suffix}"
        ));
        assert_eq!(bytes.len(), 128);
        for version in [8, 9, 10, 11, 12] {
            let decoded = app::decode_response_version(&bytes, version).unwrap();
            assert_eq!(decoded.request_id, 0xfedcba98);
            let app::Response::Shop(reply) = decoded.response else {
                panic!("not SHOP");
            };
            assert_eq!(reply.status, status);
            assert_eq!(reply.reason, reason);
            assert_eq!(
                reply.echo,
                Request {
                    context: context(),
                    channel_id: 2,
                    action: Action::Buy,
                    room_id: 0,
                    expected_room_revision: 0,
                    round_id: 2,
                    expected_revision: 3,
                    token: [0xaa; 32],
                    item_id: 40010,
                    quantity: 3,
                }
            );
            assert!(reply.snapshot.is_none() && reply.room.is_none());
            for extra in [8usize, 16] {
                let mut invalid = bytes.clone();
                invalid.resize(128 + extra, 0);
                invalid[4..8].copy_from_slice(&((128 + extra) as u32).to_be_bytes());
                assert!(
                    app::decode_response_version(&invalid, version).is_err(),
                    "absent snapshot cannot smuggle {extra} appendix bytes in v{version}"
                );
            }
            for end in 0..bytes.len() {
                assert!(app::decode_response_version(&bytes[..end], version).is_err());
            }
        }
    }
}

#[test]
fn absent_shop_stream_receipts_preserve_confirmed_scope_and_leave_the_next_frame_readable() {
    // Exercise the same versioned stream decoder, negotiated-capability checks
    // and receipt validator as transport.exchange/shop, without opening TLS.
    // The trailing PONG proves no nonexistent appendix was read from the next
    // response; failed refreshes retain the exact prior confirmed allocation.
    for version in [8, 9, 10, 11, 12] {
        let mut confirmed = read();
        if version >= 9 {
            confirmed = weighted(confirmed);
        }
        if version >= 10 {
            Arc::make_mut(confirmed.snapshot.as_mut().unwrap()).timed = Some(TimedCatalog {
                owner_class: 3,
                offers: vec![],
            });
        }
        confirmed.echo.channel_id = 2;
        if version >= 11 {
            Arc::make_mut(confirmed.snapshot.as_mut().unwrap()).ammo = Some(vec![]);
        }
        confirmed.room.as_mut().unwrap().common.channel_id = 2;
        let snapshot = Arc::make_mut(confirmed.snapshot.as_mut().unwrap());
        snapshot.round_id = 2;
        snapshot.revision = 3;
        snapshot.token = [0xaa; 32];
        let request = confirmed.echo.mutation(snapshot, Action::Buy, 40010, 3);
        confirmed.validate().unwrap();
        let mut state = State::default();
        state.receive(confirmed.clone()).unwrap();
        let retained = match &state {
            State::Ready { confirmed, .. } => confirmed.clone(),
            _ => unreachable!(),
        };
        let mut stream = Vec::new();
        for suffix in ["030b0000", "040a0000", "05090000"] {
            stream.extend(hex(&format!(
                "4c544d3100000080fedcba9800210000{GOLDEN_BODY}{suffix}"
            )));
        }
        stream.extend(hex("4c544d3100000010fedcba9900080000"));
        let mut stream = std::io::Cursor::new(stream);
        for (index, status) in [Status::Busy, Status::Unavailable, Status::Frozen]
            .into_iter()
            .enumerate()
        {
            let decoded = app::read_response_version(&mut stream, version).unwrap();
            assert_eq!(decoded.request_id, 0xfedcba98);
            assert_eq!(stream.position(), (index as u64 + 1) * 128);
            assert!(crate::modern_lito::weight_protocol::negotiated(
                &decoded.response,
                version >= 9
            ));
            assert!(crate::modern_lito::timed_protocol::negotiated(
                &decoded.response,
                version >= 10
            ));
            let app::Response::Shop(reply) = decoded.response else {
                panic!("not SHOP");
            };
            assert!(reply.confirms(&request, Some(&confirmed), confirmed.room.as_ref()));
            state.receive(reply).unwrap();
            let State::Ready {
                confirmed: current,
                refresh_failure,
            } = &state
            else {
                panic!("a normal refusal lost confirmed scope");
            };
            assert!(Arc::ptr_eq(current, &retained));
            assert_eq!(current.echo.context, context());
            assert_eq!(*refresh_failure, Some(status));
            assert_eq!(state.snapshot(), confirmed.snapshot.as_deref());
        }
        let pong = app::read_response_version(&mut stream, version).unwrap();
        assert_eq!(pong.request_id, 0xfedcba99);
        assert!(matches!(pong.response, app::Response::Pong));
        assert_eq!(stream.position(), 3 * 128 + 16);
    }
}

#[test]
fn versions_one_through_seven_reject_all_shop_and_item_request_response_kinds() {
    use super::super::item_protocol as items;
    let shop = read();
    let shop_bytes = frame(&shop);
    let item = items::Request {
        context: context(),
        channel_id: 0,
        action: items::Action::Read,
        match_id: 3,
        generation: 4,
        sequence: 0,
        item_id: 0,
    };
    let item_request = items::encode_request(9, item, 8).unwrap();
    let mut item_reply = app::header(9, items::RESULT_KIND, 76).unwrap();
    item_reply.extend(&item_request[16..]);
    item_reply.extend([3, 1, 0, 0]);
    assert!(app::decode_response_version(&item_reply, 8).is_ok());
    for version in 1..=7 {
        assert!(encode_request(9, shop.echo, version).is_err());
        assert!(items::encode_request(9, item, version).is_err());
        assert!(app::decode_response_version(&shop_bytes, version).is_err());
        assert!(app::decode_response_version(&item_reply, version).is_err());
    }
}

#[test]
fn reserved_bytes_unknown_tags_and_all_prefix_truncations_are_rejected() {
    let bytes = frame(&read());
    assert_eq!(decoded(&bytes).unwrap(), read());
    for n in 0..bytes.len() {
        assert!(decoded(&bytes[..n]).is_err(), "prefix {n}");
    }
    // Header, echo, result, product and healing reserved bytes, respectively.
    for at in [14, 15, 37, 38, 39, 41, 42, 43, 127, 207, 254, 255, 256] {
        let mut bad = bytes.clone();
        bad[at] = 1;
        assert!(decoded(&bad).is_err(), "reserved {at}");
    }
    for at in [40, 124, 125, 126, 206, 253] {
        let mut bad = bytes.clone();
        bad[at] = 255;
        assert!(decoded(&bad).is_err(), "tag {at}");
    }
    let mut bad = bytes.clone();
    bad.extend([0]);
    assert!(decoded(&bad).is_err());
}

#[test]
fn exact_price_count_text_and_healing_boundaries_cannot_be_coerced() {
    let bytes = frame(&read());
    for at in [208, 216] {
        for price in [0i64, -1, i64::MIN] {
            let mut bad = bytes.clone();
            bad[at..at + 8].copy_from_slice(&price.to_be_bytes());
            assert!(decoded(&bad).is_err());
        }
    }
    for at in [192, 194] {
        let mut bad = bytes.clone();
        bad[at..at + 2].copy_from_slice(&257u16.to_be_bytes());
        assert!(decoded(&bad).is_err());
    }
    for invalid in ["", "\u{00a0}", "\0", "x\n", &"界".repeat(86)] {
        let mut bad = read();
        Arc::make_mut(bad.snapshot.as_mut().unwrap()).offers[0].name = invalid.into();
        assert!(decoded(&frame(&bad)).is_err());
    }
    for (field, n) in [
        (0, 0),
        (1, 9),
        (2, 0),
        (2, i32::MAX as i64 + 1),
        (3, -1),
        (4, 0),
        (4, 10001),
    ] {
        let mut bad = bytes.clone();
        let at = 257 + field * 8;
        bad[at..at + 8].copy_from_slice(&i64::to_be_bytes(n));
        assert!(decoded(&bad).is_err());
    }
    let mut max = read();
    let s = Arc::make_mut(max.snapshot.as_mut().unwrap());
    s.round_id = i64::MAX;
    s.revision = i64::MAX;
    s.allowance = i64::MAX;
    s.balance = i64::MAX;
    let o = &mut s.offers[0];
    o.item_id = i64::MAX;
    o.sort = u16::MAX;
    o.buy_price = i64::MAX;
    o.sell_price = i64::MAX;
    o.weight = i64::MAX;
    o.inventory_graphic = i64::MAX;
    o.ground_graphic = i64::MAX;
    assert_eq!(decoded(&frame(&max)).unwrap(), max);
}

#[test]
fn invalid_utf8_and_room_body_length_or_context_are_fail_closed() {
    let bytes = frame(&read());
    let mut invalid = bytes.clone();
    invalid[250] = 0xff;
    assert!(decoded(&invalid).is_err());
    let nested_length_at = 297; // One three-byte-name healing offer, zero owned stacks.
    for length in [0, u32::MAX] {
        let mut bad = bytes.clone();
        bad[nested_length_at..nested_length_at + 4].copy_from_slice(&length.to_be_bytes());
        assert!(decoded(&bad).is_err());
    }
    let mut bad = read();
    bad.room.as_mut().unwrap().common.context.key.source_id = 42;
    assert!(decoded(&frame(&bad)).is_err());
    let mut bad = read();
    bad.room.as_mut().unwrap().status = rooms::Status::Denied;
    bad.room.as_mut().unwrap().reason = rooms::Reason::InvalidState;
    assert!(decoded(&frame(&bad)).is_err());
}

#[test]
fn shop_256k_limit_does_not_relax_regular_room_or_item_limits() {
    use super::super::item_protocol as items;
    assert_eq!(MAX_FRAME_BYTES, 256 * 1024);
    assert!(app::header(9, RESULT_KIND, MAX_FRAME_BYTES).is_ok());
    assert!(app::header(9, RESULT_KIND, MAX_FRAME_BYTES + 1).is_err());
    assert!(app::header(9, 4, 65_537).is_err());
    assert!(app::header(9, rooms::RESULT_KIND, rooms::MAX_FRAME_BYTES + 1).is_err());
    // Header encoding uses the current version. The negotiated v9 stream
    // must still reject an oversized item frame before reading its body.
    let oversized_item = app::header(9, items::RESULT_KIND, items::MAX_FRAME_BYTES + 1).unwrap();
    let mut old_stream = std::io::Cursor::new(oversized_item);
    let old_error = app::read_response_version(&mut old_stream, 9).unwrap_err();
    assert_eq!(old_stream.position(), 16);
    assert!(old_error
        .to_string()
        .contains("response exceeds negotiated version budget"));
    assert!(decode_body(&vec![0; MAX_FRAME_BYTES - 15], 8).is_err());
    let mut max = read();
    let s = Arc::make_mut(max.snapshot.as_mut().unwrap());
    s.offers.clear();
    for n in 1..=MAX_ROWS {
        let mut o = offer();
        o.item_id = n as i64;
        o.sort = n as u16;
        o.name = "x".repeat(256);
        s.offers.push(o);
    }
    let bytes = frame(&max);
    assert!(bytes.len() > 65_536);
    assert!(bytes.len() <= MAX_FRAME_BYTES);
    assert_eq!(decoded(&bytes).unwrap(), max);
}

#[test]
fn buy_confirmation_checks_cost_quantity_other_stacks_token_and_revision_before_state_change() {
    let (before, request, after) = buy();
    assert!(after.confirms(&request, Some(&before), before.room.as_ref()));
    for scenario in 0..7 {
        let mut wrong = after.clone();
        let s = Arc::make_mut(wrong.snapshot.as_mut().unwrap());
        match scenario {
            0 => s.balance += 1,
            1 => s.stacks[0].quantity += 1,
            2 => s.stacks[0].unit_paid_price += 1,
            3 => s.token[0] ^= 1,
            4 => s.revision += 1,
            5 => s.offers[0].buy_price += 1,
            _ => s.stacks.push(Stack {
                item_id: 50000,
                quantity: 1,
                unit_paid_price: 1,
            }),
        }
        assert!(
            !wrong.confirms(&request, Some(&before), before.room.as_ref()),
            "scenario {scenario}"
        );
        if scenario == 3 {
            assert!(decoded(&frame(&wrong)).is_err());
        }
    }
    let mut wrong = after.clone();
    wrong.echo.context.epoch -= 1;
    assert!(!wrong.confirms(&request, Some(&before), before.room.as_ref()));
    assert!(!after.confirms(&request, None, before.room.as_ref()));
}

#[test]
fn sell_confirmation_caps_refund_to_original_paid_and_never_mints_allowance() {
    let (_, _, mut before) = buy();
    Arc::make_mut(before.snapshot.as_mut().unwrap()).offers[0].sell_price = i64::MAX;
    let request = Request::read(context(), 0, None).mutation(
        before.snapshot.as_deref().unwrap(),
        Action::Sell,
        40010,
        2,
    );
    let mut after = before.clone();
    after.echo = request;
    let s = Arc::make_mut(after.snapshot.as_mut().unwrap());
    s.revision += 1;
    s.balance = 1000;
    s.stacks.clear();
    assert!(after.confirms(&request, Some(&before), before.room.as_ref()));
    let mut wrong = after.clone();
    Arc::make_mut(wrong.snapshot.as_mut().unwrap()).balance = 999;
    assert!(!wrong.confirms(&request, Some(&before), before.room.as_ref()));
    let request = Request::read(context(), 0, None).mutation(
        before.snapshot.as_deref().unwrap(),
        Action::Sell,
        40010,
        3,
    );
    after.echo = request;
    assert!(!after.confirms(&request, Some(&before), before.room.as_ref()));
}

#[test]
fn failed_or_missing_ack_does_not_optimistically_clear_the_last_confirmed_wallet() {
    let (_, request, confirmed) = buy();
    let mut state = State::default();
    state.receive(confirmed.clone()).unwrap();
    let before = state.clone();
    let _owned_request = encode_request(11, request, 8).unwrap();
    assert_eq!(state, before); // Encoding/sending is not an ACK or a local mutation.
    for (status, reason) in [
        (Status::Busy, Reason::Busy),
        (Status::Unavailable, Reason::Backend),
        (Status::Frozen, Reason::Frozen),
    ] {
        state
            .receive(Reply {
                echo: Request::read(context(), 0, None),
                status,
                reason,
                snapshot: None,
                room: None,
            })
            .unwrap();
        assert_eq!(state.snapshot(), confirmed.snapshot.as_deref());
        assert!(matches!(&state,State::Ready{refresh_failure:Some(actual),..} if *actual==status));
    }
    let prior = state.clone();
    let mut malformed = confirmed.clone();
    malformed.reason = Reason::Backend;
    assert!(state.receive(malformed).is_err());
    assert_eq!(state, prior);
}

#[test]
fn absent_failure_is_not_fabricated_zero_balance_or_a_cross_source_cached_wallet() {
    let mut state = State::default();
    let fail = Reply {
        echo: Request::read(context(), 0, None),
        status: Status::Unavailable,
        reason: Reason::Backend,
        snapshot: None,
        room: None,
    };
    state.receive(fail.clone()).unwrap();
    assert!(state.snapshot().is_none());
    assert!(matches!(state, State::Unavailable { .. }));
    state.receive(read()).unwrap();
    let mut changed = fail;
    changed.echo.context.key.source_id = 42;
    state.receive(changed).unwrap();
    assert!(state.snapshot().is_none());
    assert!(matches!(state,State::Unavailable{context:c,..} if c.key.source_id==42));
}

#[test]
fn same_revision_mutation_and_round_revision_rollback_are_rejected_without_losing_confirmation() {
    let (_, _, before) = buy();
    let mut state = State::default();
    state.receive(before.clone()).unwrap();
    for scenario in 0..5 {
        let mut invalid = before.clone();
        invalid.echo = Request::read(context(), 0, None);
        let s = Arc::make_mut(invalid.snapshot.as_mut().unwrap());
        match scenario {
            0 => s.balance += 1,
            1 => s.revision = 0,
            2 => {
                s.round_id += 1;
                s.revision = 0;
            }
            3 => s.round_id += 1,
            _ => s.round_id = 0,
        }
        let prior = state.clone();
        assert!(state.receive(invalid).is_err(), "scenario {scenario}");
        assert_eq!(state, prior);
    }
    let mut reset = before.clone();
    reset.echo = Request::read(context(), 0, None);
    let s = Arc::make_mut(reset.snapshot.as_mut().unwrap());
    s.round_id += 1;
    s.revision += 1;
    s.balance = s.allowance;
    s.stacks.clear();
    state.receive(reset.clone()).unwrap();
    assert_eq!(state.snapshot(), reset.snapshot.as_deref());
}

#[test]
fn stale_catalog_removed_offer_keeps_owned_stack_but_cannot_confirm_an_unsupported_purchase() {
    let (_, _, mut prior) = buy();
    prior.echo = Request::read(context(), 0, None);
    let s = Arc::make_mut(prior.snapshot.as_mut().unwrap());
    s.revision += 1;
    s.offers.clear();
    s.token[0] ^= 1;
    assert!(prior.validate().is_ok());
    let mut state = State::default();
    state.receive(prior.clone()).unwrap();
    assert_eq!(state.snapshot().unwrap().stacks[0].quantity, 2);
    let request = prior
        .echo
        .mutation(prior.snapshot.as_deref().unwrap(), Action::Buy, 40010, 1);
    let mut after = prior.clone();
    after.echo = request;
    Arc::make_mut(after.snapshot.as_mut().unwrap()).revision += 1;
    assert!(!after.confirms(&request, Some(&prior), prior.room.as_ref()));
}

#[test]
fn request_and_i64_arithmetic_edges_reject_without_saturating_or_wrapping() {
    let (prior, request, after) = buy();
    assert!(encode_request(0, request, 8).is_err());
    for scenario in 0..7 {
        let mut bad = request;
        match scenario {
            0 => bad.context.key.object_id = i64::MAX as u64 + 1,
            1 => bad.context.epoch = 0,
            2 => bad.channel_id = 4,
            3 => bad.quantity = MAX_QUANTITY + 1,
            4 => bad.expected_revision = -1,
            5 => bad.room_id = i64::MAX as u64 + 1,
            _ => bad.round_id = 0,
        }
        assert!(bad.validate().is_err());
    }
    let mut huge = prior.clone();
    let s = Arc::make_mut(huge.snapshot.as_mut().unwrap());
    s.allowance = i64::MAX;
    s.balance = i64::MAX;
    s.offers[0].buy_price = i64::MAX;
    let huge_request = huge
        .echo
        .mutation(huge.snapshot.as_deref().unwrap(), Action::Buy, 40010, 2);
    let mut result = after.clone();
    result.echo = huge_request;
    let s = Arc::make_mut(result.snapshot.as_mut().unwrap());
    s.allowance = i64::MAX;
    s.offers = huge.snapshot.as_ref().unwrap().offers.clone();
    assert!(!result.confirms(&huge_request, Some(&huge), huge.room.as_ref()));
    let mut overflow = after;
    overflow.echo.expected_revision = i64::MAX;
    assert!(overflow.validate().is_err());
}
