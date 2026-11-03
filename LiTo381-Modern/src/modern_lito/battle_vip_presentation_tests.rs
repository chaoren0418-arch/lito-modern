//! Complete selected engine frames stay intact while shared presentation
//! components consume VIP armor. This is not a whole-scene LOAD/UI test.
use super::*;
use crate::modern_lito::battle_follow::Observation;

#[test]
#[ignore = "real Java full-VIP selected frames and installed shared armor pixels/audio; no full-scene LOAD/UI claim"]
fn java_vip_armor_shared_presentation_keeps_owner_versions_and_expiry() {
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    assert!(root.is_absolute());
    let original = collect("vip-selected");
    crate::modern_lito::battle_armor_effects::tests::verify_vip_missing_assets(&root, &original);
    let frames = collect("vip-selected-shared-armor-gfx0");
    crate::modern_lito::battle_armor_effects::tests::verify_vip(&root, &frames);
    // The existing all-resource gate is deliberately still closed: these
    // components do not prove VIP status-ID mapping or every player's shape.
    for (_, o) in &frames {
        assert!(crate::modern_lito::battle_scene::baseline_counts(o.clone()).is_err());
    }
    println!("VIP_ARMOR_SHARED: 64 original frames retain missing11101 rejection; separate gfx0 fixture 64 full frames/shared228/1562/132; owner-only proc pixels, status expiry, immutable full VIP ledger; no substitution or full-scene LOAD/UI");
}
fn collect(mode: &str) -> Vec<(String, Observation)> {
    let mut peer = Peer::start_named(Some(mode));
    let mut frames = Vec::<(String, Observation)>::new();
    let mut followers = BTreeMap::new();
    let selection = selected(29);
    let mut handshakes = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:64");
            break;
        }
        if line.starts_with("SESSION:") {
            let parts: Vec<_> = line.splitn(4, ':').collect();
            let protocol::Message::ExtendedHello(h, offer) =
                protocol::decode(&unhex(parts[2])).unwrap()
            else {
                panic!("extended HELLO required")
            };
            let mut session = protocol::ModernClientSession::new_extended(h, offer).unwrap();
            session.begin().unwrap();
            session.accept_ack(&unhex(parts[3])).unwrap();
            assert_eq!(session.extensions(), Some(parts[1].parse().unwrap()));
            handshakes += 1;
            continue;
        }
        let parts: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(parts[0], "REPLY");
        let bytes = unhex(parts[3]);
        let data = decode(&bytes, &unhex(parts[2]), selection).unwrap();
        let key = format!(
            "{}-{}",
            parts[1].split('-').next().unwrap(),
            data.personal.player.source
        );
        let follower = followers
            .entry(key)
            .or_insert_with(|| Follower::new_selected(request(&data), selection).unwrap());
        follower.accept(reply(data.clone())).unwrap();
        let observation = follower.observation().unwrap().clone();
        assert_eq!(observation.data(), &data);
        assert!(data.vip.is_some() && data.armor.is_none() && data.healing.is_none());
        assert!(!observation.legacy_commands_supported());
        frames.push((parts[1].into(), observation));
        peer.ack(&bytes);
    }
    peer.finish();
    assert_eq!((handshakes, followers.len(), frames.len()), (5, 18, 64));
    frames
}
