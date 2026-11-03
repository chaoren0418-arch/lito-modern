//! Real Java selected READ -> ordinary immutable follower. No TLS owner,
//! source database, resource qualification, LOAD, mutation or UI admission.
use super::{
    healing_admission::{request, selected},
    Peer,
};
use crate::modern_lito::{
    application_protocol as app, battle_follow::Follower, battle_payload as payload,
    battle_read as read, protocol,
};
use payload::vip;
use std::{collections::BTreeMap, io::Cursor};

#[path = "battle_vip_appearance_tests.rs"]
mod appearance;
#[path = "battle_player_appearance_tests.rs"]
mod player_appearance;
#[path = "battle_player_body_tests.rs"]
mod player_body;
#[path = "battle_player_cast_tests.rs"]
mod player_cast;
#[path = "battle_player_utility_tests.rs"]
mod player_utility;
#[path = "battle_player_walking_tests.rs"]
mod player_walking;
#[path = "battle_vip_presentation_tests.rs"]
mod presentation;
#[path = "battle_vip_status_tests.rs"]
mod status;
#[path = "battle_vip_real_map_tests.rs"]
mod real_map;

fn unhex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0 && s.len() <= 2 * (vip::MAX_BYTES + 4096));
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn reply(d: payload::UnvalidatedData) -> read::UnvalidatedReply {
    read::UnvalidatedReply {
        echo: request(&d),
        state: read::State::Data,
        data: Some(d),
    }
}
fn decode(
    b: &[u8],
    q: &[u8],
    p: protocol::extensions::Selected,
) -> Result<payload::UnvalidatedData, app::ApplicationError> {
    let mut cursor = Cursor::new(b);
    let e = app::read_response_selected(&mut cursor, p, true)?;
    let app::Response::BattleData(r) = e.response else {
        panic!("READ expected")
    };
    assert_eq!(read::encode_request(e.request_id, &r.echo).unwrap(), q);
    assert_eq!(cursor.position(), b.len() as u64);
    Ok(r.data.unwrap())
}
fn reject(f: &mut Follower, bad: payload::UnvalidatedData) {
    let saved = f.observation().unwrap().clone();
    assert!(f.accept(reply(bad)).is_err());
    assert_eq!(f.state(), read::State::Data);
    assert_eq!(f.observation().unwrap().data(), saved.data());
    assert_eq!(f.observation().unwrap().personal(), saved.personal());
}
#[test]
fn vip_selection_is_independent_read_only_bounded_and_not_default() {
    use protocol::extensions as e;
    assert_eq!(
        [
            e::HEALING.required,
            e::ARMOR.required,
            e::HEALING_ARMOR.required,
            e::ARMOR_PRESENTATION.required,
            e::HEALING_ARMOR_PRESENTATION.required
        ],
        [3, 4, 7, 12, 15]
    );
    for ext in [16, 17, 28, 29, 31] {
        let p = selected(ext);
        assert!(p.allows_vip_data());
        assert_eq!(p.allows_commands(), ext == 31);
        assert_eq!(payload::Format::Selected(p).maximum(), vip::MAX_BYTES);
        let h = protocol::Hello {
            request_id: 1,
            min_version: 47,
            max_version: 47,
            numeric_schema: 1,
            supported_capabilities: protocol::KNOWN_CAPABILITIES,
            required_capabilities: e::DEPENDENCIES,
        };
        let offer = e::Offer {
            supported: ext,
            required: ext,
        };
        let mut c = protocol::ModernClientSession::new_extended(h, offer).unwrap();
        c.begin().unwrap();
        let ack = protocol::Ack {
            request_id: 1,
            ..p.base()
        };
        c.accept_ack(&protocol::encode(protocol::Message::ExtendedAck(ack, ext)).unwrap())
            .unwrap();
        assert_eq!(c.extensions(), Some(ext));
    }
    assert!(e::Selected::new(selected(29).base(), 32).is_err());
    for ext in [0, 1, 3, 4, 5, 7, 12, 13, 15] {
        assert!(!selected(ext).allows_vip_data());
    }
    for bit in (0..32)
        .map(|n| 1u32 << n)
        .filter(|b| e::DEPENDENCIES & b != 0)
    {
        let ack = protocol::Ack {
            capabilities: protocol::KNOWN_CAPABILITIES & !bit,
            ..selected(29).base()
        };
        assert!(e::Selected::new(ack, 16).is_err());
    }
}
#[test]
#[ignore = "compiled Java full-VIP extended handshake + ordinary READ/follower; not TLS/source/UI"]
fn java_vip_selected_read_retains_complete_state_and_rejects_downgrade_atomically() {
    let mut peer = Peer::start_named(Some("vip-selected"));
    let mut sessions = BTreeMap::new();
    let mut followers = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<payload::UnvalidatedData>> = BTreeMap::new();
    let mut frames = BTreeMap::new();
    let mut count = 0;
    let mut mutations = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:64");
            break;
        }
        if line.starts_with("SESSION:") {
            let s: Vec<_> = line.splitn(4, ':').collect();
            let ext = s[1].parse::<u32>().unwrap();
            let raw = unhex(s[2]);
            let protocol::Message::ExtendedHello(h, offer) = protocol::decode(&raw).unwrap() else {
                panic!("extended HELLO")
            };
            let mut client = protocol::ModernClientSession::new_extended(h, offer).unwrap();
            assert_eq!(client.begin().unwrap(), raw);
            let ack = client.accept_ack(&unhex(s[3])).unwrap();
            assert_eq!(client.extensions(), Some(ext));
            assert!(sessions
                .insert(ext, protocol::extensions::Selected::new(ack, ext).unwrap())
                .is_none());
            continue;
        }
        let s: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(s[0], "REPLY");
        let label = s[1];
        let q = unhex(s[2]);
        let b = unhex(s[3]);
        let selection = sessions[&29];
        let d = decode(&b, &q, selection).unwrap_or_else(|e| panic!("{label}: {e:?}"));
        assert_eq!(payload::encode(&d).unwrap(), &b[72..]);
        assert_eq!(vip::decode(&b[72..]).unwrap().into_data().unwrap(), d);
        let v = d.vip.as_ref().unwrap();
        assert_eq!(v.owner.player.object, 3333);
        assert_eq!(
            v.owner.player.source,
            if label.ends_with("-7") { 7 } else { 42 }
        );
        assert!(d.armor.is_none() && d.healing.is_none() && d.speed.is_none() && d.haste.is_none());
        assert!(d.world.descriptor.features & vip::FEATURE != 0);
        for old in [0, 1, 3, 4, 5, 7, 12, 13, 15] {
            assert!(decode(&b, &q, selected(old)).is_err());
        }
        for ext in [16, 17, 28, 29, 31] {
            let p = sessions[&ext];
            let admitted = (v.healing.is_none() || p.allows_data())
                && (v.presentation.is_none() || p.allows_armor_presentation());
            assert_eq!(decode(&b, &q, p).is_ok(), admitted, "{label} ext{ext}");
            assert!(!read::command_data_negotiated_selected(&d, p));
            assert!(payload::Format::Selected(p)
                .decode_command(&b[72..])
                .is_err());
        }
        for version in 1..=47 {
            assert!(payload::decode_versioned(&b[72..], version).is_err());
        }
        // Ordinary stream framing does not consume the following frame.
        let both = [b.as_slice(), b.as_slice()].concat();
        let mut stream = Cursor::new(both);
        for expected in [b.len(), 2 * b.len()] {
            app::read_response_selected(&mut stream, selection, true).unwrap();
            assert_eq!(stream.position(), expected as u64);
        }
        for size in [0, 15, 16, 71, 72, b.len() - 1] {
            assert!(
                app::read_response_selected(&mut Cursor::new(&b[..size]), selection, true).is_err()
            );
        }
        for length in [16, 71, (72 + vip::MAX_BYTES + 1) as u32, u32::MAX] {
            let mut bad = b.clone();
            bad[4..8].copy_from_slice(&length.to_be_bytes());
            let mut c = Cursor::new(bad);
            assert!(app::read_response_selected(&mut c, selection, true).is_err());
            assert_eq!(c.position(), 16);
        }
        let group = label.split('-').next().unwrap();
        let key = format!("{group}-{}", d.personal.player.source);
        let f = followers
            .entry(key.clone())
            .or_insert_with(|| Follower::new_selected(request(&d), selection).unwrap());
        f.accept(reply(d.clone()))
            .unwrap_or_else(|e| panic!("follow {label}: {e:?}"));
        f.accept(reply(d.clone())).unwrap();
        let o = f.observation().unwrap();
        assert_eq!(o.data(), &d);
        assert!(!o.legacy_commands_supported());
        let detail = crate::modern_lito::battle_details::vip_character(o)
            .expect("selected owner has a bounded read-only VIP character page");
        assert!(detail.contains("完整 VIP 本場權威數值（唯讀）"));
        assert!(detail.contains(&format!("來源 {}／角色 3333", v.owner.player.source)));
        assert!(detail.contains(&format!("能力版本 {}", v.owner.versions.runtime)));
        for (name, value) in vip::FIELD_NAMES.iter().zip(v.owner.abilities.0.iter()) {
            assert!(detail.contains(&format!("（{name}）：{value}")), "missing {name}");
        }
        assert!(!detail.contains("來源保存值不是本場即時素質"));
        crate::modern_lito::login_window::tests::assert_vip_read_only_character_route(o);
        assert!(crate::modern_lito::battle_scene::baseline_counts(o.clone())
            .unwrap_err()
            .to_string()
            .contains("full VIP shared-resource presentation is not implemented"));
        assert_eq!(
            o.personal(),
            &d.personal_view(&d.personal.player, &d.world.identity)
                .unwrap()
        );
        assert!(crate::modern_lito::battle_greater_haste::Intent::from_observation(o).is_none());
        assert!(crate::modern_lito::battle_slow::Intent::from_observation(
            o,
            crate::modern_lito::battle_window::Target::Player { entity: 1, life: 1 }
        )
        .is_none());
        assert!(
            crate::modern_lito::battle_target_haste::Intent::from_observation(
                o,
                crate::modern_lito::battle_window::Target::Player { entity: 1, life: 1 }
            )
            .is_none()
        );
        assert!(
            crate::modern_lito::battle_player_healing::Intent::from_observation(o, 1).is_none()
        );
        assert!(crate::modern_lito::battle_revival::Request::from_observation(o).is_none());
        for mutate in [
            (|x: &mut payload::UnvalidatedData| x.vip = None) as fn(&mut payload::UnvalidatedData),
            |x| x.world.descriptor.features &= !vip::FEATURE,
            |x| x.vip.as_mut().unwrap().owner.player.source = 99,
            |x| x.vip.as_mut().unwrap().owner.versions.runtime += 1,
            |x| {
                x.vip.as_mut().unwrap().owner.versions.saved =
                    x.vip.as_ref().unwrap().owner.versions.runtime + 1
            },
            |x| x.vip.as_mut().unwrap().owner.world.generation += 1,
            |x| x.vip.as_mut().unwrap().owner.life += 1,
            |x| x.vip.as_mut().unwrap().owner.icons.0.push(12345),
            |x| x.vip.as_mut().unwrap().armor.0.clear(),
            |x| x.actors.0[0].position.x = i32::MAX,
        ] {
            let mut bad = d.clone();
            mutate(&mut bad);
            reject(f, bad);
            mutations += 1;
        }
        histories.entry(key).or_default().push(d.clone());
        frames.insert(label.to_owned(), d);
        peer.ack(&b);
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 64);
    assert_eq!(sessions.len(), 5);
    let mut pairs = 0;
    for history in histories.values() {
        for (i, a) in history.iter().enumerate() {
            for b in history.iter().skip(i) {
                let mut f = Follower::new_selected(request(a), sessions[&29]).unwrap();
                f.accept(reply(a.clone())).unwrap();
                f.accept(reply(b.clone())).unwrap();
                if a.world.revision < b.world.revision {
                    reject(&mut f, a.clone());
                }
                pairs += 1;
            }
        }
    }
    for source in [7, 42] {
        let wide = &frames[&format!("wide-{source}")];
        assert!(wide.vip.as_ref().unwrap().owner.abilities.0[7] > u32::MAX as i64);
        let expired = &frames[&format!("durable-expiry-{source}")];
        let saved = &frames[&format!("durable-{source}")];
        assert!(
            expired.vip.as_ref().unwrap().owner.versions.runtime
                > expired.vip.as_ref().unwrap().owner.versions.saved
        );
        if source == 7 {
            assert!(
                saved.vip.as_ref().unwrap().owner.versions.saved
                    > expired.vip.as_ref().unwrap().owner.versions.saved
            );
        }
    }
    assert_eq!(histories.len(), 18);
    assert_eq!(pairs, 166);
    assert_eq!(mutations, 640);
    println!("VIP_SELECTED: 64 complete READ frames; 5 actual extended selections; 18 immutable streams; 166 ordered pairs; 640 atomic rejections; source-scoped read-only stats, inventory and learned-skill routes; no legacy command admission, TLS/SQL, shared LOAD or HWND");
}
