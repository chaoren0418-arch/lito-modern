//! Explicit shared Quest candidates, not proof of the original LT layouts or NPC rules.
//! No native process, source database, LT archive, or shared-resource write.
use super::{
    npc_assets::{NpcAction, NpcAppearance, NpcAssets},
    resources::SharedMapSectorKey,
    scene::static_draw_list,
    shared_action_catalog::SharedActionCatalog,
    sprite_actions::{ActionDefinition, ActionKey},
    world_map::{SectorBinding, WorldCell, WorldMap, DIRECTIONS},
    world_scene::{SceneLimits, WorldSceneAssets},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

fn root() -> PathBuf {
    std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .expect("explicit shared root required")
}

fn frame_record(index: usize, frame: &super::sprite::SpriteFrame) -> [u8; 56] {
    let mut out = [0; 56];
    for (dst, value) in out[..24].chunks_exact_mut(4).zip([
        index as i32,
        frame.x_offset,
        frame.y_offset,
        frame.width as i32,
        frame.height as i32,
        i32::from(frame.frame_type),
    ]) {
        dst.copy_from_slice(&value.to_be_bytes());
    }
    out[24..].copy_from_slice(&Sha256::digest(&frame.rgba));
    out
}

#[test]
#[ignore = "explicit ten-NPC shared body/shadow/attachment SPR and WAV qualification, not Quest gameplay"]
fn actual_shared_quest_npc_complete_asset_bank() {
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(&root(), &cancel).unwrap();
    // Individual resource qualification, NOT a claim that all ten banks can
    // reside together under the production 128MiB budget. Never raise that cap
    // merely to make this probe green; this probe does not exercise Quest
    // stage ownership or gameplay.
    let mut sprite_records = BTreeMap::new();
    let mut frames = 0;
    let mut encoded = 0;
    let mut rgba = 0;
    let mut flat_rgba = 0;
    let mut compact_atlas_bytes = 0;
    let mut banks = Vec::new();
    let mut samples = 0;
    let mut sample_hash = Sha256::new();
    let mut frame_sets = Vec::new();
    let mut composite_hash = Sha256::new();
    let mut sounds = BTreeMap::new();
    for a in NpcAppearance::ALL {
        let assets = NpcAssets::load_with_catalog(&catalog, &[a], &cancel).unwrap();
        encoded += assets.encoded_bytes();
        rgba += assets.rgba_bytes();
        banks.push(json!({"gfx":a.graphics_id(),"sprites":assets.sprite_sources().len(),"encoded_bytes":assets.encoded_bytes(),"rgba_bytes":assets.rgba_bytes()}));
        for p in assets.sprite_sources() {
            let decoded = assets.sprite_frames(&p.name).unwrap();
            let atlas = &decoded.first().unwrap().fragments.as_ref().unwrap().atlas;
            assert!(
                decoded
                    .iter()
                    .all(|frame| Arc::ptr_eq(&frame.fragments.as_ref().unwrap().atlas, atlas)),
                "one shared compact atlas per SPR"
            );
            flat_rgba += decoded.iter().map(|f| f.rgba.len()).sum::<usize>();
            compact_atlas_bytes += atlas
                .iter()
                .map(|block| block.retained_bytes())
                .sum::<usize>();
            let (palette, metadata) = assets.sprite_metadata(&p.name).unwrap();
            let mut h = Sha256::new();
            for (i, f) in decoded.iter().enumerate() {
                h.update(frame_record(i, f));
            }
            let mut record = p.name.as_bytes().to_vec();
            record.push(0);
            record.extend(p.bytes.to_be_bytes());
            record.extend(p.sha256);
            record.push(u8::from(palette));
            record.extend(metadata.to_be_bytes());
            record.extend((decoded.len() as u32).to_be_bytes());
            record.extend(h.finalize());
            assert!(sprite_records.insert(p.name.clone(), record).is_none());
            frames += decoded.len();
        }
        for k in NpcAction::ALL {
            let q = assets.action(a, k).unwrap();
            frame_sets.push(json!({"gfx":a.graphics_id(),"action":k.key().id,"sources":q.source_frames().len(),"hits":q.source_frames().iter().enumerate().filter_map(|(i,f)|f.events.contains(&super::sprite_actions::FrameEvent::Hit).then_some(i)).collect::<Vec<_>>()}));
            for heading in 0..8 {
                for source in 0..q.source_frames().len() {
                    let s = assets.frame(a, k, heading, source).unwrap();
                    for (layer, s) in [Some(s.body), Some(s.shadow), s.attachment]
                        .into_iter()
                        .enumerate()
                        .filter_map(|(l, s)| s.map(|s| (l, s)))
                    {
                        for value in [
                            a.graphics_id(),
                            k.key().id,
                            u32::from(heading),
                            source as u32,
                            layer as u32,
                            s.asset.image_id,
                        ] {
                            sample_hash.update(value.to_be_bytes());
                        }
                        sample_hash.update(frame_record(s.asset.subframe as usize, s.frame));
                        samples += 1;
                    }
                }
            }
            let mut canvas = super::compositor::Canvas::new(256, 256).unwrap();
            canvas.clear([45, 67, 89]);
            assets
                .draw_frame(
                    &mut canvas,
                    a,
                    k,
                    3,
                    1,
                    super::world_map::PixelPoint { x: 128, y: 160 },
                )
                .unwrap();
            assert!(canvas
                .rgba()
                .chunks_exact(4)
                .any(|p| p != [45, 67, 89, 255]));
            composite_hash.update(canvas.rgba());
        }
        let sound = assets.sound_bank();
        for id in sound.ids() {
            let s = sound.get(id).unwrap();
            let f = s.format();
            let row = json!({"id":id,"bytes":s.file_bytes(),"sha256":hex(&s.sha256()),"pcm_bytes":s.pcm_bytes().len(),"channels":f.channels,"sample_rate":f.sample_rate,"byte_rate":f.byte_rate,"block_align":f.block_align,"bits":f.bits_per_sample});
            if let Some(old) = sounds.insert(id, row.clone()) {
                assert_eq!(old, row);
            }
        }
    }
    let mut sprite_hash = Sha256::new();
    for record in sprite_records.values() {
        sprite_hash.update(record);
    }
    let mut sound_hash = Sha256::new();
    for row in sounds.values() {
        for key in [
            "id",
            "bytes",
            "pcm_bytes",
            "channels",
            "sample_rate",
            "byte_rate",
            "block_align",
            "bits",
        ] {
            sound_hash.update((row[key].as_u64().unwrap() as u32).to_be_bytes());
        }
        sound_hash.update(digest(row["sha256"].as_str().unwrap()));
    }
    let sprite_hash = hex(&sprite_hash.finalize());
    let sample_hash = hex(&sample_hash.finalize());
    let sound_hash = hex(&sound_hash.finalize());
    let composite_hash = hex(&composite_hash.finalize());
    // Independent C# full decode and byte/PCM oracle, 2026-09-16. Empty
    // sentinels are normalized only after matching original bytes in both.
    assert_eq!(
        hex(&catalog.sha256()),
        "523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127"
    );
    assert_eq!(
        (
            sprite_records.len(),
            frames,
            samples,
            encoded,
            flat_rgba,
            sounds.len()
        ),
        (920, 5944, 6304, 10245287, 161281152, 30)
    );
    // Preserve the independent C# flattened-pixel oracle. The world loader
    // additionally retains a shared compact RGB555 atlas for native cell
    // interleaving; count that storage once per SPR, never once per frame.
    assert!(compact_atlas_bytes > 0 && compact_atlas_bytes < 2 * encoded);
    assert_eq!(rgba, flat_rgba + compact_atlas_bytes);
    assert_eq!(
        sprite_hash,
        "62087b373d3a4dd6b1ff5a8c27336f05b7eb34b925ef371cf2bc2d5330df7797"
    );
    assert_eq!(
        sample_hash,
        "985675b3b46e415f6c3348d615b233362e26f03f8b770493392fe495d6f5092c"
    );
    assert_eq!(
        sound_hash,
        "b7353ad0b0e42bf31a03d29343733332d1c6a6d33ff3b35966dd04080446833f"
    );
    // Modern reference regression only, NOT C#/BIN compositing equivalence.
    assert_eq!(
        composite_hash,
        "92f255d749f99c7f63fbb0417c8622406411ba38fe0757a39ea2cd3d3dd5b16b"
    );
    println!(
        "{}",
        json!({"schema":"quest-npc-shared-assets-v1","list_sha256":hex(&catalog.sha256()),"sprites":sprite_records.len(),"decoded_frames":frames,"encoded_bytes":encoded,"rgba_bytes":rgba,"flat_rgba_bytes":flat_rgba,"compact_atlas_bytes":compact_atlas_bytes,"resident_banks":banks,"simultaneous_ten_bank_claim":false,"sprite_collection_sha256":sprite_hash,"samples":samples,"sample_80b_sha256":sample_hash,"sound_64b_sha256":sound_hash,"modern_flat_composites_sha256":composite_hash,"actions":frame_sets,"sounds":sounds.values().collect::<Vec<_>>(),"gameplay":false})
    );
}

#[test]
#[ignore = "explicit shared candidate phase residency; no Quest controller/corpse lifetime or stage transition claim"]
fn actual_shared_npc_candidate_stage_budgets_and_oversized_roster_rejection() {
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(&root(), &cancel).unwrap();
    let mut rows = Vec::new();
    for ids in [
        &[1649][..],
        &[2158, 1082],
        &[1047, 1020, 1578],
        &[1517, 2535, 240, 145],
    ] {
        let roster: Vec<_> = ids
            .iter()
            .map(|&id| NpcAppearance::new(id).unwrap())
            .collect();
        let assets = NpcAssets::load_with_catalog(&catalog, &roster, &cancel).unwrap();
        assert!(assets.rgba_bytes() <= 128 * 1024 * 1024);
        rows.push(json!({"appearances":ids,"encoded_bytes":assets.encoded_bytes(),"rgba_bytes":assets.rgba_bytes(),"sprites":assets.sprite_sources().len(),"sound_bytes":assets.sound_bank().file_bytes()}));
        // Each assets owner (including PCM and all unique sprites) drops here.
    }
    let failure = NpcAssets::load_with_catalog(&catalog, &NpcAppearance::ALL, &cancel)
        .err()
        .expect("ten simultaneous banks must not evade production cap");
    assert!(format!("{failure:#}").contains("SPR exceeds total pixels limit"));
    assert!(
        NpcAssets::load_with_catalog(&catalog, &[NpcAppearance::new(145).unwrap()], &cancel)
            .is_ok(),
        "failed oversize load must not poison later explicit loads"
    );
    println!(
        "{}",
        json!({"schema":"quest-npc-candidate-residency-v1","banks":rows,"ten_bank_roster_rejected":true,"gameplay":false,"corpses_across_stages_implemented":false})
    );
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn digest(text: &str) -> [u8; 32] {
    assert_eq!(text.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).unwrap())
}
pub(crate) fn bindings() -> Vec<SectorBinding> {
    [
        (
            0x7fff,
            0x8000,
            62681,
            "0217a00231ccca7c07f1a8d9cff86ae3de319108e71a7289d4acf65181268510",
        ),
        (
            0x7fff,
            0x8001,
            76468,
            "6470ce783beecf047103076114912a787861d2998219bf4af1b2ea9a2a31c120",
        ),
        (
            0x8000,
            0x8000,
            75400,
            "945b9ff592e28f10a58e26f3fa8053bcc566ad8cf9c255ab6117e6fde52a6537",
        ),
        (
            0x8000,
            0x8001,
            64287,
            "a91f86d5017d1c306f8ec6a545f333918b8d22befec00c287ffcb76b6986f8ae",
        ),
    ]
    .into_iter()
    .map(|(x, y, len, sha)| SectorBinding {
        key: SharedMapSectorKey {
            map_id: 554,
            sector_x: x,
            sector_y: y,
        },
        byte_length: len,
        sha256: digest(sha),
    })
    .collect()
}

#[test]
#[ignore = "explicit shared554 complete S32/Tile candidate; no NPC gameplay or original LT map equivalence"]
fn actual_shared554_complete_geometry_and_static_tile_bank_candidate() {
    let root = root();
    let cancel = AtomicBool::new(false);
    let bindings = bindings();
    let actual: BTreeSet<_> = std::fs::read_dir(root.join("map/554"))
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            assert!(e.file_type().unwrap().is_file());
            e.file_name().into_string().unwrap()
        })
        .collect();
    assert_eq!(actual, bindings.iter().map(|b| b.key.file_name()).collect());
    let map = Arc::new(WorldMap::load_shared(&root, &bindings, &cancel).unwrap());
    let mut rows = Vec::new();
    let mut masks = Vec::new();
    let mut standing = BTreeSet::new();
    let mut refs = BTreeSet::new();
    let mut cross = 0;
    for b in &bindings {
        let s = map.sector(b.key).unwrap();
        let (ox, oy) = b.key.world_origin();
        let start = masks.len();
        for y in oy..oy + 64 {
            for x in ox..ox + 64 {
                let p = WorldCell { x, y };
                masks.push(map.direction_mask(p));
                if map.standing(p) {
                    standing.insert(p);
                }
                for (dx, dy) in DIRECTIONS {
                    let q = WorldCell {
                        x: x + dx,
                        y: y + dy,
                    };
                    if map.can_step(p, q)
                        && (!(ox..ox + 64).contains(&q.x) || !(oy..oy + 64).contains(&q.y))
                    {
                        cross += 1;
                    }
                }
            }
        }
        let draws = static_draw_list(s).unwrap();
        refs.extend(draws.iter().map(|d| (d.archive_entry_id, d.sub_index)));
        let a = map.art_limitations(b.key).unwrap();
        rows.push(json!({"sector":b.key.file_name(),"length":b.byte_length,"sha256":hex(&b.sha256),
            "offsets":s.layer_ranges.iter().map(|r|r.as_ref().map(|r|[r.start,r.end])).collect::<Vec<_>>(),
            "direction_sha256":hex(&Sha256::digest(&masks[start..])),
            "counts":[s.floor.len(),s.overlays.len(),s.attributes.len(),s.object_tile_count(),s.opacity.len(),s.tile_dependencies.len(),s.portals.len(),s.effects.len()],
            "extended_effects":s.effects_extended,"unknown_tail":s.has_unknown_tail(),"draws":draws.len(),
            "portals":s.portals.iter().map(|p|json!({"name_hex":hex(&p.name),"half_x":p.x,"y":p.y,"target_map":p.target_map,"portal_id":p.portal_id})).collect::<Vec<_>>(),
            "unimplemented_art":{"opacity":a.opacity_records,"effects":a.sprite_effects,"portals":a.portals,"negative_object_tiles":a.negative_object_tiles}}));
    }
    let anchor = *standing
        .iter()
        .min_by_key(|p| (p.x - 32768).abs() + (p.y - 32832).abs())
        .unwrap();
    let mut queue = VecDeque::from([anchor]);
    let mut reached = BTreeSet::from([anchor]);
    let mut spawns = Vec::new();
    while let Some(p) = queue.pop_front() {
        if spawns.len() < 100 {
            spawns.push(p);
        }
        for (dx, dy) in DIRECTIONS {
            let q = WorldCell {
                x: p.x + dx,
                y: p.y + dy,
            };
            if map.can_step(p, q) && reached.insert(q) {
                queue.push_back(q);
            }
        }
        assert!(reached.len() <= 16384);
    }
    assert_eq!(spawns.len(), 100);
    map.validate_spawns(&spawns, &cancel).unwrap();
    // Independently recomputed by ModernQuestMapQualificationTest from the
    // complete shared four-sector selection; still not original LT equivalence.
    assert_eq!(
        hex(&map.canonical_digest()),
        "fa44590de41cc8fd77a13fbf0cb1dbbed50e3726a27a2a2aa92fe7be717a29c3"
    );
    assert_eq!(
        hex(&Sha256::digest(&masks)),
        "86abc1c5961f6549a6294d077f1fe9e71466697b45b22a69404799259e54eeb6"
    );
    assert_eq!(standing.len(), 979);
    assert_eq!(reached.len(), 165);
    assert_eq!(cross, 0);
    assert_eq!(anchor, WorldCell { x: 32778, y: 32823 });
    assert_eq!(refs.len(), 2204);
    println!(
        "{}",
        json!({"candidate":554,"native_equivalence":false,"canonical":hex(&map.canonical_digest()),
        "directions":hex(&Sha256::digest(&masks)),"standing":standing.len(),"reachable":reached.len(),"cross_sector_edges":cross,
        "anchor":[anchor.x,anchor.y],"tile_refs":refs.len(),"tile_ids":refs.iter().map(|r|r.0).collect::<BTreeSet<_>>(),"sectors":rows})
    );
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    let assets =
        WorldSceneAssets::load_terrain_with_catalog(&catalog, map, SceneLimits::default(), &cancel)
            .unwrap();
    println!(
        "{}",
        json!({"candidate":554,"static_assets_loaded":true,"encoded_bytes":assets.encoded_bytes(),"texture_bytes":assets.texture_bytes(),"tile_sets":assets.tile_set_count(),"gameplay":false})
    );
}

#[test]
#[ignore = "explicit shared Text action inventory for ten Quest NPC candidates; inspection only, no pixels/AI qualification"]
fn actual_shared_quest_npc_action_inventory() {
    let root = root();
    let cancel = AtomicBool::new(false);
    let shared = SharedActionCatalog::load(&root, &cancel).unwrap();
    for id in [
        1649, 2158, 1082, 1047, 1020, 1578, 1517, 2535, 240, 145, 1582, 1519, 242, 1650, 2159,
        1083, 1048, 1021, 1579, 1518, 2536, 241, 128,
    ] {
        let sprite = shared
            .catalog()
            .sprites
            .get(&id)
            .expect("candidate NPC definition absent");
        let mut actions = Vec::new();
        for action in [0, 1, 2, 3, 8] {
            let key = ActionKey::new(action);
            let raw = match sprite.actions.get(&key) {
                Some(ActionDefinition::Animation(a)) => {
                    json!({"kind":"animation","fps":a.frames_per_second,"directionality":format!("{:?}",a.directionality),
                    "frames":a.frames.iter().map(|f|json!({"image":f.image_id,"sub":f.subframe,"ticks":f.duration_ticks,"events":format!("{:?}",f.events)})).collect::<Vec<_>>()})
                }
                other => json!({"definition":format!("{other:?}")}),
            };
            let resolved = match shared.catalog().resolve_action(id, key) {
                Ok(a) => {
                    json!({"definition":a.definition_sprite_id,"frames":a.animation.frames.len(),"ticks":a.animation.total_ticks()})
                }
                Err(e) => json!({"error":format!("{:?}",e.kind)}),
            };
            actions.push(json!({"key":action,"raw":raw,"resolve":resolved}));
        }
        println!(
            "{}",
            json!({"candidate":id,"graphics":sprite.graphics_id,"images":sprite.image_count,"header_issue":format!("{:?}",sprite.header_issue),
            "catalog":hex(&shared.sha256()),"attributes":sprite.attributes.iter().map(|a|json!({"key":a.key.id,"body":String::from_utf8_lossy(&a.body_bytes)})).collect::<Vec<_>>(),"actions":actions,"qualified":false})
        );
    }
}
