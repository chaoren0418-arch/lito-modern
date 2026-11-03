use super::*;
use crate::modern_lito::{
    battle_player, battle_runtime::{BattleRuntime, Status}, battle_vip_scene_preview::Preview,
    battle_window, login_window, resources::SharedMapSectorKey,
    world_map::{SectorBinding, WorldMap},
};
use std::{collections::BTreeMap, path::PathBuf, sync::atomic::AtomicBool, time::{Duration, Instant}};

#[test]
#[ignore = "Java schema11 READ and real shared map514 sector/actor/HUD/status; no actual GUI/TLS/SQL or LOAD"]
fn java_vip_real_shared_map_manifest_reaches_formal_read_only_player_preview() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared Lineage root"));
    let xml = PathBuf::from(std::env::var_os("LITO_EFFECTLIST_REFERENCE").expect("explicit shared effect XML"));
    let mut peer = Peer::start_named(Some("vip-appearance-real-map"));
    let hello = peer.line();
    let h: Vec<_> = hello.splitn(4, ':').collect();
    let protocol::Message::ExtendedHello(hello, offer) = protocol::decode(&unhex(h[2])).unwrap()
    else { panic!("HELLO") };
    let mut session = protocol::ModernClientSession::new_extended(hello, offer).unwrap();
    session.begin().unwrap();
    let ack = session.accept_ack(&unhex(h[3])).unwrap();
    let selected = protocol::extensions::Selected::new(ack, session.extensions().unwrap()).unwrap();
    let mut followers = BTreeMap::new();
    let mut frames = BTreeMap::new();
    loop {
        let line = peer.line();
        if line == "CLOSED:4" { break; }
        let v: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(v.len(), 4);
        assert_eq!(v[0], "REPLY");
        let bytes = unhex(v[3]);
        let d = decode(&bytes, &unhex(v[2]), selected).unwrap();
        let source = v[1].rsplit_once('-').unwrap().1;
        let f = followers.entry(source.to_owned())
            .or_insert_with(|| Follower::new_selected(request(&d), selected).unwrap());
        f.accept(reply(d)).unwrap();
        frames.insert(v[1].to_owned(), f.observation().unwrap().clone());
        peer.ack(&bytes);
    }
    peer.finish();
    assert_eq!(frames.len(), 4);
    let cancel = AtomicBool::new(false);
    let expected = unhex("2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8");
    for o in frames.values() {
        assert_eq!(o.data().manifest.map, 514);
        assert_eq!(o.data().manifest.digest.as_slice(), expected);
        assert_eq!(o.data().manifest.sectors.0.len(), 1);
        let sector = &o.data().manifest.sectors.0[0];
        assert_eq!((sector.x, sector.y, sector.byte_length), (0x7fff, 0x8000, 137784));
        assert_eq!(o.data().vip.as_ref().unwrap().appearances.as_ref().unwrap().0.len(), 2);
        assert!(o.data().vip.is_some());
    }
    let pre = frames.get("real-map-pre-7").unwrap();
    let active = frames.get("real-map-active-7").unwrap();
    let s = &pre.data().manifest.sectors.0[0];
    let binding = SectorBinding {
        key: SharedMapSectorKey { map_id: 514, sector_x: s.x.try_into().unwrap(), sector_y: s.y.try_into().unwrap() },
        byte_length: s.byte_length.try_into().unwrap(), sha256: s.sha256,
    };
    let map = WorldMap::load_shared(&root, &[binding], &cancel).unwrap();
    assert_eq!(map.canonical_digest().as_slice(), expected);
    assert_eq!(map.bindings(), vec![binding]);
    let mut false_binding = binding;
    false_binding.sha256[0] ^= 1;
    assert!(WorldMap::load_shared(&root, &[false_binding], &cancel).is_err(),
        "manifest drift must fail before the preview can paint");
    let mut preview = Preview::load(&root, pre.clone(), Some(&xml), &cancel).unwrap();
    let scene = preview.render(Instant::now(), &cancel).unwrap();
    assert_eq!((scene.width, scene.height, scene.bgra.len()), (800, 600, 800 * 600 * 4));
    assert!(scene.effects_skipped);
    assert!(scene.navigation.as_ref().is_some_and(|n| n.matches(pre)));
    assert!(scene.minimap.as_ref().is_some_and(|p| p.matches(pre)));
    battle_window::validate(&scene).unwrap();
    let view = login_window::tests::vip_preview_view(active);
    let mut runtime = BattleRuntime::with_preferences_and_effects(
        root.clone(), Some(xml), crate::modern_lito::preferences::shared()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let published = loop {
        assert!(Instant::now() < deadline, "real-map production preview worker timed out");
        match battle_player::update(&mut runtime, &view, false).unwrap() {
            Status::Ready(surface) => break surface,
            Status::Failed(reason) => panic!("{reason}"),
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    };
    assert!(battle_player::admitted(&published, &view));
    battle_window::validate(&published).unwrap();
    assert_eq!(published.observation.data().manifest.digest.as_slice(), expected);
    assert!(!runtime.prepared(&published));
    assert!(runtime.loaded_request(&published, active).is_none());
    assert!(runtime.navigation().is_none());
    assert!(!battle_player::controls_ready(&published, &view));
    assert!(!battle_player::admitted_observation(active, &view));
    assert!(battle_player::canonical_loaded_command(&runtime, &published, &view).is_none());
    if let Some(path) = std::env::var_os("LITO_VIP_REAL_MAP_EVIDENCE") {
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();
        let mut png = png::Encoder::new(file, published.width, published.height);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        let mut rgba = published.bgra.to_vec();
        for pixel in rgba.chunks_exact_mut(4) { pixel.swap(0, 2); }
        png.write_header().unwrap().write_image_data(&rgba).unwrap();
    }
    println!("VIP_REAL_MAP:4 Java schema11 READ frames; server+client exact 514/7fff8000 manifest; real shared terrain/actors/status in formal read-only preview; no LOAD/gameplay/HWND/TLS/SQL");
}
