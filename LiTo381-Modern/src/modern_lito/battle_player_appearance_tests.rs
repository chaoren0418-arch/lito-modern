//! Engine/CAS -> followed schema11 -> source-bound player pixels. Real shared
//! actor/Tile assets with explicit synthetic geometry, not full VIP LOAD/UI.
use super::*;
use crate::modern_lito::{
    battle_scene,
    compositor::Canvas,
    player_appearance::{Assets, Form, Plan},
    resources::SharedMapSectorKey,
    shared_action_catalog::SharedActionCatalog,
    world_map::{SectorInput, WorldMap},
    world_scene::{BaseAction, SceneSnapshot},
};
use sha2::{Digest, Sha256};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

#[test]
#[ignore = "actual Java schema11/CAS transitions and shared player raster; synthetic terrain, not SQL/TLS/UI or full VIP preflight"]
fn java_public_appearances_render_as_players_and_replace_resources_atomically() {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
    );
    let mut peer = Peer::start_named(Some("vip-appearance-shared"));
    let hello = peer.line();
    let h: Vec<_> = hello.splitn(4, ':').collect();
    assert_eq!(&h[..2], &["SESSION", "240"]);
    let protocol::Message::ExtendedHello(hello, offer) = protocol::decode(&unhex(h[2])).unwrap()
    else {
        panic!("HELLO")
    };
    let mut session = protocol::ModernClientSession::new_extended(hello, offer).unwrap();
    assert_eq!(session.begin().unwrap(), unhex(h[2]));
    let ack = session.accept_ack(&unhex(h[3])).unwrap();
    let p = protocol::extensions::Selected::new(ack, session.extensions().unwrap()).unwrap();
    assert!(p.allows_vip_appearance());
    let mut followers = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut count = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:18");
            break;
        }
        let v: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(v[0], "REPLY");
        let q = unhex(v[2]);
        let bytes = unhex(v[3]);
        let data = decode(&bytes, &q, p).unwrap();
        let f = followers
            .entry(v[1].to_owned())
            .or_insert_with(|| Follower::new_selected(request(&data), p).unwrap());
        f.accept(reply(data)).unwrap();
        let o = f.observation().unwrap().clone();
        assert!(
            battle_scene::baseline_counts(o.clone()).is_err(),
            "full VIP admission must remain closed"
        );
        histories.entry(v[1].to_owned()).or_default().push(o);
        count += 1;
        peer.ack(&bytes);
    }
    peer.finish();
    assert_eq!(count, 18);
    let cancel = AtomicBool::new(false);
    let context = SharedActionCatalog::load(&root, &cancel).unwrap();
    let mut rendered = 0;
    let mut reloads = 0;
    let mut reuses = 0;
    let mut preview = Canvas::new(2400, 600).unwrap();
    let mut preview_panels = 0;
    for (label, trace) in &histories {
        let first = Plan::from_observation(trace[0].clone()).unwrap();
        let sector = &trace[0].data().manifest.sectors.0[0];
        assert_eq!(trace[0].data().manifest.sectors.0.len(), 1);
        let bytes = vec![0; 49170];
        assert_eq!(sector.byte_length as usize, bytes.len());
        assert_eq!(sector.sha256, <[u8; 32]>::from(Sha256::digest(&bytes)));
        let key = SharedMapSectorKey {
            map_id: trace[0].data().manifest.map.try_into().unwrap(),
            sector_x: sector.x.try_into().unwrap(),
            sector_y: sector.y.try_into().unwrap(),
        };
        let map = Arc::new(
            WorldMap::from_sector_bytes(vec![SectorInput { key, bytes }], &cancel).unwrap(),
        );
        assert_eq!(map.canonical_digest(), trace[0].data().manifest.digest);
        let mut assets = Assets::load_with_catalog(&context, map.clone(), &first, &cancel).unwrap();
        let mut previous = first;
        let mut canvas = Canvas::new(800, 600).unwrap();
        for (step, o) in trace.iter().enumerate() {
            if o.data().world.phase == payload::data::Phase::Aborted {
                assert!(Plan::from_observation(o.clone()).is_err());
                continue;
            }
            let plan = Plan::from_observation(o.clone()).unwrap();
            previous.check_successor(&plan).unwrap();
            let snapshot = plan.snapshot();
            let before = canvas.rgba().to_vec();
            if !assets.supports(&plan) {
                assert!(assets
                    .render(&plan, snapshot, &mut canvas, &cancel)
                    .is_err());
                assert_eq!(canvas.rgba(), before);
                assert!(Assets::load_with_catalog(
                    &context,
                    map.clone(),
                    &plan,
                    &AtomicBool::new(true)
                )
                .is_err());
                assets
                    .render(&previous, previous.snapshot(), &mut canvas, &cancel)
                    .unwrap();
                let candidate =
                    Assets::load_with_catalog(&context, map.clone(), &plan, &cancel).unwrap();
                candidate
                    .render(&plan, snapshot, &mut canvas, &cancel)
                    .unwrap();
                assets = candidate;
                reloads += 1;
            } else {
                reuses += 1;
            }
            let result = assets
                .render(&plan, snapshot, &mut canvas, &cancel)
                .unwrap();
            assert_eq!(result.actors.len(), 2);
            assert!(result.npcs.is_empty());
            assert!(result.objectives.is_empty());
            for a in snapshot.actors() {
                let pose = result
                    .actors
                    .iter()
                    .find(|v| v.entity_id == a.entity_id)
                    .unwrap();
                assert_eq!(a.object_id, 3333);
                assert!([7, 42].contains(&a.source_id));
                assert_eq!(pose.cell, a.cell);
                assert!(pose.visibility.is_some());
                match plan.forms()[&a.entity_id] {
                    Form::Polymorph { .. } => {
                        assert_eq!(pose.graphics_id, 240);
                        assert!(pose.sprite_name.starts_with("240-"));
                    }
                    Form::Base(_) => assert_ne!(pose.graphics_id, 240),
                }
                if a.action == BaseAction::Death {
                    assert_eq!(pose.source_frame, 0);
                }
            }
            // Actual native fragment ownership, not just two placement records.
            let mut owned = [0usize; 2];
            for y in 0..600 {
                for x in 0..800 {
                    let a = result.actors[0].visibility.as_ref().unwrap().contains(x, y);
                    let b = result.actors[1].visibility.as_ref().unwrap().contains(x, y);
                    assert!(!(a && b), "one pixel cannot belong to both players");
                    owned[0] += usize::from(a);
                    owned[1] += usize::from(b);
                }
            }
            assert!(
                owned.iter().all(|n| *n > 0),
                "invisible player {label}/{step}: {owned:?}"
            );
            if label == "gear-7" && (1..=3).contains(&step) {
                preview
                    .blit_rgba(canvas.rgba(), 800, 600, ((step - 1) * 800) as i32, 0)
                    .unwrap();
                preview_panels += 1;
            }
            // A same-numbered actor from another source, old revision or owner
            // cannot reuse the current body-frame set or mutate the canvas.
            let before = canvas.rgba().to_vec();
            for mutation in 0..5 {
                let mut actors = snapshot.actors().to_vec();
                let mut revision = snapshot.revision();
                let mut selected = snapshot.selected_entity_id();
                let mut world = snapshot.world_id();
                let mut digest = snapshot.map_digest();
                match mutation {
                    0 => actors[0].source_id = 99,
                    1 => revision += 1,
                    2 => {
                        selected = if selected == actors[0].entity_id {
                            actors[1].entity_id
                        } else {
                            actors[0].entity_id
                        }
                    }
                    3 => world += 1,
                    _ => digest[0] ^= 1,
                }
                let stale = SceneSnapshot::new(world, revision, digest, selected, actors).unwrap();
                assert!(assets.render(&plan, &stale, &mut canvas, &cancel).is_err());
                assert_eq!(canvas.rgba(), before);
            }
            // Animation may change the pose but never identity or confirmed cell.
            let mut actors = snapshot.actors().to_vec();
            for actor in &mut actors {
                if actor.action != BaseAction::Death {
                    actor.action = BaseAction::Attack;
                    actor.action_elapsed = Duration::from_millis(85);
                }
            }
            let posed = snapshot.with_actors(actors).unwrap();
            let posed_result = assets.render(&plan, &posed, &mut canvas, &cancel).unwrap();
            assert_eq!(posed_result.actors.len(), 2);
            previous = plan;
            rendered += 1;
        }
    }
    assert_eq!(rendered, 16);
    assert_eq!(reloads, 4);
    assert_eq!(reuses, 12);
    assert_eq!(preview_panels, 3);
    if let Some(path) = std::env::var_os("LITO_PLAYER_APPEARANCE_TEST_PNG") {
        let file = std::fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(file, preview.width(), preview.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(preview.rgba())
            .unwrap();
    }
    println!("PLAYER_APPEARANCE:18 Java frames;16 player-raster observations;4 transactional reloads/12 reuse; dual sources objid3333; poly->base->poly,expiry,death;80 stale-frame rejections; both players own nonempty disjoint native pixel masks; full VIP gate still closed; no UI/SQL/TLS claim");
}
