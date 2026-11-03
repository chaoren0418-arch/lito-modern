//! Actual engine -> negotiated schema11 READ -> immutable client observation.
//! No graphics ID is a claim that a shared sprite or full scene has loaded.
use super::*;

#[test]
fn vip_public_appearance_requires_an_explicit_dependency_closed_selection() {
    use protocol::extensions as e;
    assert_eq!(e::FULL_VIP.required, 112);
    assert_eq!(e::FULL_VIP_V2.required, 240);
    for bits in [128, 129, 160, 192, 224, 256] {
        assert!(e::Selected::new(selected(112).base(), bits).is_err());
    }
    for bits in [144, 176, 208, 240, 255] {
        let p = selected(bits);
        assert!(p.allows_vip_appearance());
        assert_eq!(
            payload::Format::Selected(p).maximum(),
            vip::MAX_APPEARANCE_BYTES
        );
    }
    assert!(!selected(112).allows_vip_appearance());
    assert_eq!(
        payload::Format::Selected(selected(112)).maximum(),
        vip::MAX_BYTES
    );
}

#[test]
#[ignore = "actual Java VIP appearance/CAS/expiry/death frames and extended handshake; not shared preload/UI"]
fn java_vip_public_appearance_is_source_bound_versioned_and_cannot_be_downgraded() {
    let mut peer = Peer::start_named(Some("vip-appearance"));
    let line = peer.line();
    let s: Vec<_> = line.splitn(4, ':').collect();
    assert_eq!(&s[..2], &["SESSION", "240"]);
    let protocol::Message::ExtendedHello(h, offer) = protocol::decode(&unhex(s[2])).unwrap() else {
        panic!("HELLO");
    };
    let mut client = protocol::ModernClientSession::new_extended(h, offer).unwrap();
    assert_eq!(client.begin().unwrap(), unhex(s[2]));
    let ack = client.accept_ack(&unhex(s[3])).unwrap();
    let p = protocol::extensions::Selected::new(ack, client.extensions().unwrap()).unwrap();
    assert!(p.allows_vip_appearance());
    let mut followers = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<payload::UnvalidatedData>> = BTreeMap::new();
    let mut count = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:18");
            break;
        }
        let s: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(s[0], "REPLY");
        let q = unhex(s[2]);
        let bytes = unhex(s[3]);
        let d = decode(&bytes, &q, p).unwrap_or_else(|e| panic!("{} {e:?}", s[1]));
        assert_eq!(payload::encode(&d).unwrap(), &bytes[72..]);
        assert_eq!(&bytes[76..78], &11u16.to_be_bytes());
        assert_eq!(vip::decode(&bytes[72..]).unwrap().into_data().unwrap(), d);
        for bits in [0, 16, 112, 127] {
            assert!(decode(&bytes, &q, selected(bits)).is_err());
        }
        let v = d.vip.as_ref().unwrap();
        let appearances = &v.appearances.as_ref().unwrap().0;
        assert_eq!(appearances.len(), 2);
        assert_eq!((appearances[0].source, appearances[1].source), (7, 42));
        assert!(appearances.iter().all(|a| a.object == 3333));
        let f = followers
            .entry(s[1].to_owned())
            .or_insert_with(|| Follower::new_selected(request(&d), p).unwrap());
        f.accept(reply(d.clone())).unwrap();
        assert_eq!(f.observation().unwrap().data(), &d);
        f.accept(reply(d.clone())).unwrap(); // Identical repeat, no invented visual transition.
        let own = (v.owner.entity - 1) as usize;
        for mutation in 0..9 {
            let mut bad = d.clone();
            let a = &mut bad.vip.as_mut().unwrap().appearances.as_mut().unwrap().0[own];
            match mutation {
                0 => a.source = if a.source == 7 { 42 } else { 7 },
                1 => a.object += 1,
                2 => a.entity += 1,
                3 => a.life += 1,
                4 => a.runtime += 1,
                5 => a.lifecycle += 1,
                6 => a.graphics += 1,
                7 => a.graphics = -1,
                _ => bad
                    .vip
                    .as_mut()
                    .unwrap()
                    .appearances
                    .as_mut()
                    .unwrap()
                    .0
                    .reverse(),
            }
            reject(f, bad);
        }
        // A remote graphics change can be meaningful data in a later frame,
        // but cannot rewrite the same world/ability/lifecycle revision.
        let mut bad = d.clone();
        bad.vip.as_mut().unwrap().appearances.as_mut().unwrap().0[1 - own].graphics += 1;
        reject(f, bad);
        let mut downgrade = d.clone();
        downgrade.vip.as_mut().unwrap().appearances = None;
        assert!(!read::data_negotiated_selected(&downgrade, p));
        reject(f, downgrade);
        assert!(crate::modern_lito::battle_scene::baseline_counts(
            f.observation().unwrap().clone()
        )
        .is_err());
        if appearances.iter().any(|a| a.graphics > u32::MAX as i64) {
            assert!(
                crate::modern_lito::player_appearance::Plan::from_observation(
                    f.observation().unwrap().clone()
                )
                .is_err(),
                "no truncation or partial player roster"
            );
        }
        histories.entry(s[1].to_owned()).or_default().push(d);
        peer.ack(&bytes);
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 18);
    assert_eq!(histories.len(), 6);
    for source in [7, 42] {
        let gear = &histories[&format!("gear-{source}")];
        let graphics = |d: &payload::UnvalidatedData, i: usize| {
            d.vip.as_ref().unwrap().appearances.as_ref().unwrap().0[i].graphics
        };
        assert_eq!(gear.len(), 5);
        assert_eq!(
            (graphics(&gear[0], 0), graphics(&gear[0], 1)),
            (5641, 4_294_967_301)
        );
        assert_eq!(
            (graphics(&gear[2], 0), graphics(&gear[2], 1)),
            (0, 4_294_967_301)
        );
        assert_eq!(graphics(&gear[3], 0), 5641);
        // A genuine later world frame must not allow an unrelated player's
        // graphics to change without that player's ability/lifecycle advance.
        let mut f = Follower::new_selected(request(&gear[1]), p).unwrap();
        f.accept(reply(gear[1].clone())).unwrap();
        let mut bad = gear[2].clone();
        bad.vip.as_mut().unwrap().appearances.as_mut().unwrap().0[1].graphics += 1;
        reject(&mut f, bad);
        let expiry = &histories[&format!("expiry-{source}")];
        assert_eq!((graphics(&expiry[1], 0), graphics(&expiry[1], 1)), (0, 0));
        let death = &histories[&format!("death-{source}")];
        assert_eq!(
            death[1].actors.0[1].resources.as_ref().unwrap().vitals.hp,
            0
        );
        assert_eq!(graphics(&death[1], 1), 4_294_967_301);
    }
    println!("VIP_APPEARANCE: 18 actual engine READ frames; dual-source same objid; CAS unequip/re-equip, expiry/death/abort; 198 atomic rejections; schema10 unchanged; no resource/LOAD/UI claim");
}
