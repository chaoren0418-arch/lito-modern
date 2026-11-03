use super::*;
use crate::modern_lito::{
    application_protocol::{CharacterKey, SelectionContext},
    compositor::Canvas,
    effect_catalog::{EffectCatalog, EffectId},
    vip_inventory_preview, vip_status, vip_status_preview,
};
use std::sync::atomic::AtomicBool;
#[test]
#[ignore = "actual Java schema11 equipment/expiry/death owner icons and explicit Lineage effect XML/shared TBT; no full scene/UI"]
fn java_vip_status_icons_follow_source_owned_equipment_without_invented_xml_timers() {
    let mut peer = Peer::start_named(Some("vip-appearance-shared"));
    let hello = peer.line();
    let h: Vec<_> = hello.splitn(4, ':').collect();
    let protocol::Message::ExtendedHello(hello, offer) = protocol::decode(&unhex(h[2])).unwrap()
    else {
        panic!("HELLO")
    };
    let mut session = protocol::ModernClientSession::new_extended(hello, offer).unwrap();
    session.begin().unwrap();
    let ack = session.accept_ack(&unhex(h[3])).unwrap();
    let selected = protocol::extensions::Selected::new(ack, session.extensions().unwrap()).unwrap();
    let cancel = AtomicBool::new(false);
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let xml = std::path::PathBuf::from(std::env::var_os("LITO_EFFECTLIST_REFERENCE").unwrap());
    let catalog = EffectCatalog::load(&xml, &cancel).unwrap();
    let mut histories: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut followers = BTreeMap::new();
    loop {
        let line = peer.line();
        if line == "CLOSED:18" {
            break;
        }
        let v: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(v[0], "REPLY");
        let bytes = unhex(v[3]);
        let d = decode(&bytes, &unhex(v[2]), selected).unwrap();
        let f = followers
            .entry(v[1].to_owned())
            .or_insert_with(|| Follower::new_selected(request(&d), selected).unwrap());
        f.accept(reply(d)).unwrap();
        histories
            .entry(v[1].to_owned())
            .or_default()
            .push(f.observation().unwrap().clone());
        peer.ack(&bytes);
    }
    peer.finish();
    let ids: Vec<_> = histories
        .values()
        .flatten()
        .flat_map(|o| o.data().vip.as_ref().unwrap().owner.icons.0.iter().copied())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    assert!(!ids.is_empty());
    let assets = vip_status::Assets::load(&root, catalog.clone(), &ids, &cancel).unwrap();
    let empty = vip_status::Assets::load(&root, catalog, &[], &cancel).unwrap();
    let mut count = 0;
    let mut visible = 0;
    let mut cleared = 0;
    for (label, trace) in &histories {
        for o in trace {
            assert_eq!(o.data().personal.player.object, 3333);
            let expected = vip_status::required(o).unwrap();
            let mut frame = Canvas::new(128, 64).unwrap();
            frame.clear([12, 23, 34]);
            let blank = frame.rgba().to_vec();
            let mut cancelled = Canvas::new(128, 64).unwrap();
            cancelled.copy_from(&frame).unwrap();
            assert!(assets
                .draw(o, &mut cancelled, (0, 0), 4, &AtomicBool::new(true))
                .is_err());
            assert_eq!(cancelled.rgba(), blank);
            let placements = assets.draw(o, &mut frame, (0, 0), 4, &cancel).unwrap();
            assert_eq!(placements.iter().map(|p| p.0).collect::<Vec<_>>(), expected);
            let once = frame.rgba().to_vec();
            assets.draw(o, &mut frame, (0, 0), 4, &cancel).unwrap();
            assert_eq!(frame.rgba(), once);
            if expected.is_empty() {
                cleared += 1;
                assert_eq!(frame.rgba(), blank);
            } else {
                visible += 1;
                assert_ne!(frame.rgba(), blank);
                assert!(!empty.supports(o).unwrap());
                assert!(empty.draw(o, &mut cancelled, (0, 0), 4, &cancel).is_err());
                assert_eq!(cancelled.rgba(), blank);
                assert!(assets
                    .draw(o, &mut cancelled, (127, 63), 4, &cancel)
                    .is_err());
                assert_eq!(cancelled.rgba(), blank);
                // Independently map and copy opaque shared pixels, not IDs.
                let mut reference = blank.clone();
                for (id, x, y) in &placements {
                    let p = assets.picture(*id).unwrap();
                    assert_eq!(p.graphic, assets.catalog().get(*id).unwrap().icon);
                    for sy in 0..p.height as usize {
                        for sx in 0..p.width as usize {
                            let src = (sy * p.width as usize + sx) * 4;
                            if p.rgba()[src + 3] == 255 {
                                let dst = ((sy + *y as usize) * 128 + sx + *x as usize) * 4;
                                reference[dst..dst + 3].copy_from_slice(&p.rgba()[src..src + 3]);
                            }
                        }
                    }
                }
                assert_eq!(frame.rgba(), reference);
            }
            assert!(crate::modern_lito::battle_scene::baseline_counts(o.clone()).is_err());
            count += 1;
        }
        if label == "gear-7" {
            assert!(!vip_status::required(&trace[0]).unwrap().is_empty());
            assert!(vip_status::required(&trace[2]).unwrap().is_empty());
            assert!(!vip_status::required(&trace[3]).unwrap().is_empty());
            use crate::modern_lito::{
                player_appearance::{Assets as PlayerAssets, Plan, Playback},
                resources::SharedMapSectorKey,
                shared_action_catalog::SharedActionCatalog,
                world_map::{SectorInput, WorldMap},
            };
            use sha2::{Digest, Sha256};
            let o = &trace[1];
            let player = &o.data().personal.player;
            let context = SelectionContext {
                key: CharacterKey {
                    source_id: player.source.try_into().unwrap(),
                    object_id: player.object.try_into().unwrap(),
                },
                epoch: player.epoch.try_into().unwrap(),
            };
            let key = vip_status_preview::Key::from_observation(
                context, player.channel.try_into().unwrap(), o).unwrap().unwrap();
            let inventory_model = vip_inventory_preview::Model::from_observation(
                context, player.channel.try_into().unwrap(), o,
                |token| token.to_owned()).unwrap();
            assert_eq!(inventory_model.rows.len(),
                o.data().personal.equipment.as_ref().unwrap().inventory.0.len());
            let mut inventory_preview = vip_inventory_preview::Preview::default();
            inventory_preview.reconcile(Some(inventory_model.clone()));
            let icon_deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                inventory_preview.poll(&root, false);
                assert!(std::time::Instant::now() < icon_deadline,
                    "owned shared inventory art worker did not finish");
                if inventory_preview.is_quiescent() { break; }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let mut ready_inventory_icons = 0;
            for graphic in &inventory_model.key.graphics {
                match inventory_preview.art(*graphic).expect("each requested icon has explicit status") {
                    Ok(art) => {
                        ready_inventory_icons += 1;
                        assert_eq!(art.bgra.len(),
                            (vip_inventory_preview::ICON_SIDE * vip_inventory_preview::ICON_SIDE * 4) as usize);
                    }
                    Err(reason) => assert!(!reason.is_empty()),
                }
            }
            assert!(ready_inventory_icons > 0, "real source inventory must display at least one shared icon");
            inventory_preview.reconcile(None);
            assert!(inventory_preview.model().is_none());
            assert!(inventory_model.key.graphics.iter().all(|id| inventory_preview.art(*id).is_none()));
            assert_eq!(key.icons, vip_status::required(o).unwrap().iter()
                .map(|id| i64::from(id.value())).collect::<Vec<_>>());
            let mut over_capacity = key.clone();
            over_capacity.icons.resize(vip_status_preview::COLUMNS + 1, 505);
            assert!(vip_status_preview::prepare(&root, &xml, over_capacity, o, &cancel)
                .err().unwrap().to_string().contains("capacity"));
            let mut unconfigured = vip_status_preview::Preview::default();
            unconfigured.update(Some((key.clone(), o)), &root, None, false);
            assert!(unconfigured.image().is_none());
            assert!(unconfigured.error().unwrap().contains("effectlist2.xml"));
            let mut preview = vip_status_preview::Preview::default();
            preview.update(Some((key.clone(), o)), &root, Some(&xml), false);
            preview.update(None, &root, Some(&xml), false);
            assert!(preview.image().is_none(), "retired owner must clear before old I/O completes");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            while !preview.is_quiescent() {
                assert!(std::time::Instant::now() < deadline, "retired status worker did not finish");
                std::thread::sleep(std::time::Duration::from_millis(10));
                preview.update(None, &root, Some(&xml), false);
            }
            preview.update(Some((key.clone(), o)), &root, Some(&xml), false);
            while preview.image().is_none() && preview.error().is_none() {
                assert!(std::time::Instant::now() < deadline, "status worker did not finish");
                std::thread::sleep(std::time::Duration::from_millis(10));
                preview.update(Some((key.clone(), o)), &root, Some(&xml), false);
            }
            assert!(preview.error().is_none(), "{:?}", preview.error());
            let image = preview.image().unwrap();
            assert_eq!(image.key, key);
            assert_eq!(image.bgra.len(),
                (vip_status_preview::WIDTH * vip_status_preview::HEIGHT * 4) as usize);
            assert!(image.bgra.chunks_exact(4).any(|pixel| pixel[..3] != [24, 24, 24]));
            preview.update(None, &root, Some(&xml), false);
            assert!(preview.image().is_none());
            let m = &o.data().manifest;
            let s = &m.sectors.0[0];
            let bytes = vec![0; 49170];
            assert_eq!(s.sha256, <[u8; 32]>::from(Sha256::digest(&bytes)));
            let map = std::sync::Arc::new(
                WorldMap::from_sector_bytes(
                    vec![SectorInput {
                        key: SharedMapSectorKey {
                            map_id: m.map.try_into().unwrap(),
                            sector_x: s.x.try_into().unwrap(),
                            sector_y: s.y.try_into().unwrap(),
                        },
                        bytes,
                    }],
                    &cancel,
                )
                .unwrap(),
            );
            let plan = Plan::from_observation(o.clone()).unwrap();
            let shared = SharedActionCatalog::load(&root, &cancel).unwrap();
            let players = PlayerAssets::load_with_catalog(&shared, map.clone(), &plan, &cancel).unwrap();
            let now = std::time::Instant::now();
            let playback = Playback::new(plan, now).unwrap();
            let mut reference = Canvas::new(800, 600).unwrap();
            players
                .render_playback(&playback, now, &mut reference, &cancel)
                .unwrap();
            assets
                .draw(o, &mut reference, (224, 8), 16, &cancel)
                .unwrap();
            let mut combined = Canvas::new(800, 600).unwrap();
            combined.clear([12, 23, 34]);
            let blank = combined.rgba().to_vec();
            assert!(players
                .render_playback_with_status(
                    &playback,
                    &empty,
                    now,
                    &mut combined,
                    &cancel,
                    (224, 8),
                    16
                )
                .is_err());
            assert_eq!(combined.rgba(), blank);
            players
                .render_playback_with_status(
                    &playback,
                    &assets,
                    now,
                    &mut combined,
                    &cancel,
                    (224, 8),
                    16,
                )
                .unwrap();
            assert_eq!(combined.rgba(), reference.rgba());
            let mut preview = crate::modern_lito::battle_vip_scene_preview::Preview::load_map(
                &root, o.clone(), map.clone(), Some(&xml), &cancel).unwrap();
            let scene = preview.render(std::time::Instant::now(), &cancel).unwrap();
            assert_eq!(scene.observation.data(), o.data());
            let owner = o.data().actors.0.iter().find(|a| a.entity == o.data().personal.entity).unwrap();
            assert_eq!(crate::modern_lito::battle_scene::owner_hud_vitals(o.data()).unwrap(),
                &owner.resources.as_ref().unwrap().vitals);
            assert_eq!((scene.width, scene.height, scene.bgra.len()), (800, 600, 800 * 600 * 4));
            assert!(scene.navigation.as_ref().is_some_and(|nav| nav.matches(o))
                && scene.effects_skipped);
            assert!(scene.minimap.as_ref().is_some_and(|panel| panel.matches(o)));
            assert!(scene.bgra.chunks_exact(4).any(|pixel| pixel[..3] != [0, 0, 0]));
            crate::modern_lito::battle_window::validate(&scene).unwrap();
            let mut view = crate::modern_lito::login_window::tests::vip_preview_view(o);
            assert!(view.canonical.observation().is_none());
            assert!(view.canonical.vip_observation().is_some());
            let mut runtime = crate::modern_lito::battle_runtime::BattleRuntime::with_vip_preview_fixture(
                root.clone(), map, xml.clone());
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            let published = loop {
                assert!(std::time::Instant::now() < deadline,
                    "normal VIP player binding did not reach the preview worker");
                match crate::modern_lito::battle_player::update(&mut runtime, &view, false).unwrap() {
                    crate::modern_lito::battle_runtime::Status::Ready(surface) => break surface,
                    crate::modern_lito::battle_runtime::Status::Failed(reason) => panic!("{reason}"),
                    _ => std::thread::sleep(std::time::Duration::from_millis(10)),
                }
            };
            assert!(crate::modern_lito::battle_player::admitted(&published, &view));
            assert!(!crate::modern_lito::battle_player::admitted_observation(o, &view));
            crate::modern_lito::battle_window::validate(&published).unwrap();
            assert!(!runtime.prepared(&published));
            assert!(runtime.loaded_request(&published, o).is_none());
            assert!(runtime.navigation().is_none());
            assert!(!crate::modern_lito::battle_player::controls_ready(&published, &view));
            let step = crate::modern_lito::battle_window::Kind::Local(
                crate::modern_lito::battle_window::Local::Step(0));
            assert!(!crate::modern_lito::battle_input::ready(step, &view));
            assert!(crate::modern_lito::battle_input::command(&published,
                crate::modern_lito::battle_window::Intent {
                    stamp: crate::modern_lito::battle_window::Stamp::of(o),
                    kind: step,
                }, &view).is_err());
            assert!(crate::modern_lito::battle_player::canonical_loaded_command(
                &runtime, &published, &view).is_none());
            view.selected_source = Some(u32::try_from(player.source).unwrap() + 1);
            crate::modern_lito::battle_player::update(&mut runtime, &view, false).unwrap();
            assert!(runtime.observation().is_none());
            assert!(!crate::modern_lito::battle_player::admitted(&published, &view));
            view.selected_source = Some(u32::try_from(player.source).unwrap());
            crate::modern_lito::battle_player::update(&mut runtime, &view, false).unwrap();
            assert!(runtime.observation().is_some());
            let mut departed = view.clone();
            departed.canonical = Default::default();
            crate::modern_lito::battle_player::update(&mut runtime, &departed, false).unwrap();
            assert!(runtime.observation().is_none());
            assert!(!crate::modern_lito::battle_player::admitted(&published, &departed));
            if let Some(path) = std::env::var_os("LITO_VIP_SCENE_EVIDENCE") {
                let file = std::fs::OpenOptions::new().write(true).create_new(true)
                    .open(path).unwrap();
                let mut png = png::Encoder::new(file, scene.width, scene.height);
                png.set_color(png::ColorType::Rgba);
                png.set_depth(png::BitDepth::Eight);
                let mut rgba = scene.bgra.to_vec();
                for pixel in rgba.chunks_exact_mut(4) { pixel.swap(0, 2); }
                png.write_header().unwrap().write_image_data(&rgba).unwrap();
            }
            if let Some(path) = std::env::var_os("LITO_VIP_STATUS_EVIDENCE") {
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .unwrap();
                let mut png = png::Encoder::new(file, 800, 600);
                png.set_color(png::ColorType::Rgba);
                png.set_depth(png::BitDepth::Eight);
                png.write_header()
                    .unwrap()
                    .write_image_data(combined.rgba())
                    .unwrap();
            }
        }
        if label == "gear-42" {
            assert_eq!(
                vip_status::required(&trace[0]).unwrap(),
                vip_status::required(&trace[2]).unwrap()
            );
        }
        if label.starts_with("expiry-") {
            assert!(vip_status::required(&trace[1]).unwrap().is_empty());
        }
    }
    assert_eq!(count, 18);
    assert!(visible > 0 && cleared > 0);
    assert!(assets.picture(EffectId::from_wire(41).unwrap()).is_none()); // no unrelated grant
    println!("VIP_STATUS:18 real schema11 frames;ids={ids:?};visible={visible}/cleared={cleared};source7/42 same3333,actual XML-to-TBT,owned preview worker retire/reload,unequip/re-equip/expiry/death/abort,atomic cancel/missing/layout,repaint;no invented timer/fullscene/HWND");
}
