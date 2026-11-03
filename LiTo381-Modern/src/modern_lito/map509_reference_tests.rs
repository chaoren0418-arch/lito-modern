//! Shared map 509 reference checks. Expected data comes from the separately
//! executed, read-only PakViewer C# + strict PowerShell reference pipeline in
//! tools/verify-map509-reference.ps1, not from an LT map or this decoder.
//! Static L1/L2/L4 qualification is not proof of Layer5/Layer8 presentation.

use super::{
    resources::{ResourceLimits, SharedArchive, SharedArchiveKind, SharedMapSectorKey},
    scene::{render_tiles, static_draw_list, CameraRect},
    tile::{decode_classic_til_with_limits, DecodedTile, TileLimits},
    world_map::{
        cell_to_sector_pixel, SectorBinding, WorldCell, WorldMap, WorldMapError, DIRECTIONS,
    },
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, VecDeque};
use std::path::Path;
use std::sync::atomic::AtomicBool;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64);
    std::array::from_fn(|index| u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap())
}

fn binding() -> SectorBinding {
    SectorBinding {
        key: SharedMapSectorKey {
            map_id: 509,
            sector_x: 0x7fff,
            sector_y: 0x8000,
        },
        byte_length: 106_355,
        sha256: digest("8580337a4ad3593d697a416062c4da063f8fd6c2ba311f0b694fcfc59710cb9d"),
    }
}

/// Each entry is exactly 45 bytes: BE tile ID, BE subindex, type, BE source
/// offset, four source-rectangle bytes, then the independently decoded RGBA SHA.
fn tile_entry(id: u16, subindex: u16, tile: &DecodedTile) -> [u8; 45] {
    let mut bytes = [0; 45];
    bytes[..2].copy_from_slice(&id.to_be_bytes());
    bytes[2..4].copy_from_slice(&subindex.to_be_bytes());
    bytes[4] = tile.tile_type as u8;
    bytes[5..9].copy_from_slice(&tile.source_offset.to_be_bytes());
    bytes[9..13].copy_from_slice(&[
        tile.x_offset,
        tile.y_offset,
        tile.content_width,
        tile.content_height,
    ]);
    bytes[13..].copy_from_slice(&Sha256::digest(&tile.rgba));
    bytes
}

fn connected_spawns(map: &WorldMap, anchor: WorldCell) -> (Vec<WorldCell>, usize) {
    assert!(map.standing(anchor));
    let mut queue = VecDeque::from([anchor]);
    let mut seen = BTreeSet::from([anchor]);
    let mut result = Vec::new();
    while let Some(cell) = queue.pop_front() {
        if result.len() < 100 {
            result.push(cell);
        }
        for (dx, dy) in DIRECTIONS {
            let next = WorldCell {
                x: cell.x + dx,
                y: cell.y + dy,
            };
            if map.can_step(cell, next) && seen.insert(next) {
                queue.push_back(next);
            }
        }
        assert!(seen.len() <= 4096, "single-sector BFS bound");
    }
    assert_eq!(result.len(), 100);
    (result, seen.len())
}

#[test]
fn tile_reference_aggregate_has_explicit_width_and_source_offset() {
    let tile = DecodedTile {
        tile_type: super::tile::ClassicTileType::RleOddHalfOpacity,
        source_offset: 0x1020_3040,
        x_offset: 1,
        y_offset: 2,
        content_width: 3,
        content_height: 4,
        width: 24,
        height: 24,
        rgba: vec![],
    };
    let entry = tile_entry(0x1234, 0x5678, &tile);
    assert_eq!(hex(&entry[..13]), "12345678071020304001020304");
    assert_eq!(
        hex(&entry[13..]),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
#[ignore = "requires LITO_SHARED_RESOURCE_ROOT; entire shared509 sector/Tile qualification and independent memory-only C# pixel oracle"]
fn actual_shared509_full_static_references_geometry_and_connected_spawns_match_independent_oracle()
{
    let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
    let root = Path::new(&root);
    let cancel = AtomicBool::new(false);
    let binding = binding();
    let map = WorldMap::load_shared(root, &[binding], &cancel).unwrap();
    assert_eq!(map.map_id(), 509);
    assert_eq!(map.bindings(), vec![binding]);
    assert_eq!(map.canonical_bytes().len(), 4171);
    assert_eq!(
        hex(&map.canonical_digest()),
        "1178261c2d6a968251f2eadfb2029bd0febade95b7151c15872f355784df87f8"
    );
    let mut masks = Vec::with_capacity(4096);
    for y in 32768..32832 {
        for x in 32704..32768 {
            masks.push(map.direction_mask(WorldCell { x, y }));
        }
    }
    assert_eq!(
        hex(&Sha256::digest(&masks)),
        "e112237a0209d41d229797782c90a29eff5c20e35a9acb4970ac01799c9a76d1"
    );
    assert_eq!(masks.iter().filter(|mask| **mask & 0x55 != 0).count(), 324);
    let anchor = WorldCell { x: 32740, y: 32796 };
    assert_eq!(map.direction_mask(anchor), 255);
    let (spawns, component_count) = connected_spawns(&map, anchor);
    assert_eq!(component_count, 324);
    assert_eq!(spawns[1], WorldCell { x: 32740, y: 32795 });
    map.validate_spawns(&spawns, &cancel).unwrap();
    let mut spawn_bytes = Vec::with_capacity(800);
    for cell in &spawns {
        spawn_bytes.extend_from_slice(&cell.x.to_be_bytes());
        spawn_bytes.extend_from_slice(&cell.y.to_be_bytes());
    }
    assert_eq!(
        hex(&Sha256::digest(&spawn_bytes)),
        "0e3394021b9e2da1454bcbc5d8b9b58fafe9a4e7913d1ed0f85dada600d59c55"
    );
    let sector = map.sector(binding.key).unwrap();
    assert_eq!(
        sector
            .layer_ranges
            .iter()
            .map(|range| range.as_ref().unwrap().start)
            .collect::<Vec<_>>(),
        [0, 32768, 32770, 49154, 102823, 106117, 106237, 106239]
    );
    assert_eq!(sector.layer_ranges[7].as_ref().unwrap().end, 106355);
    assert_eq!(sector.floor.len(), 8192);
    assert!(sector.overlays.is_empty());
    assert_eq!(sector.attributes.len(), 4096);
    assert_eq!(sector.objects.len(), 139);
    assert_eq!(sector.object_tile_count(), 7587);
    assert_eq!(sector.opacity.len(), 658);
    assert!(sector.portals.is_empty());
    assert_eq!(sector.effects.len(), 19);
    assert!(!sector.effects_extended);
    assert_eq!(
        sector
            .effects
            .iter()
            .map(|effect| (effect.sprite_id, effect.x, effect.y, effect.extended))
            .collect::<Vec<_>>(),
        [
            (4860, 32752, 32792),
            (4860, 32752, 32800),
            (4860, 32743, 32808),
            (4860, 32736, 32808),
            (4860, 32727, 32799),
            (4860, 32727, 32792),
            (4860, 32736, 32782),
            (4860, 32743, 32782),
            (4861, 32746, 32786),
            (4863, 32733, 32786),
            (4862, 32749, 32789),
            (4864, 32749, 32802),
            (4865, 32746, 32805),
            (4865, 32733, 32806),
            (4866, 32729, 32802),
            (4866, 32729, 32789),
            (1906, 32728, 32811),
            (1892, 32755, 32789),
            (1892, 32733, 32782)
        ]
        .into_iter()
        .map(|(id, x, y)| (id, x, y, None))
        .collect::<Vec<_>>()
    );
    assert!(!sector.has_unknown_tail());
    let ids: BTreeSet<u16> = [18, 29, 34, 36, 37, 38, 41, 1514, 1515, 2110]
        .into_iter()
        .chain(2967..=2985)
        .collect();
    assert_eq!(ids.len(), 29);
    assert_eq!(
        sector.tile_dependencies,
        ids.iter().map(|id| i32::from(*id)).collect::<Vec<_>>()
    );
    let limitations = map.art_limitations(binding.key).unwrap();
    assert_eq!(limitations.opacity_records, 658);
    assert_eq!(limitations.sprite_effects, 19);
    assert_eq!(limitations.portals, 0);
    assert_eq!(limitations.negative_object_tiles, 0);
    assert!(limitations.has_known_presentation_gaps());

    // Enumerate ALL L1/L2/L4 references before any viewport culling. This is
    // deliberately stronger than a render_tiles-only first-camera smoke test.
    let draws = static_draw_list(sector).unwrap();
    assert_eq!(draws.len(), 15779);
    let refs: BTreeSet<(u16, u8)> = draws
        .iter()
        .map(|draw| (draw.archive_entry_id, draw.sub_index))
        .collect();
    assert_eq!(refs.len(), 4341);
    assert_eq!(
        draws
            .iter()
            .map(|draw| draw.archive_entry_id)
            .collect::<BTreeSet<_>>(),
        ids
    );
    assert_eq!(
        [
            draws.iter().map(|draw| draw.x).min().unwrap(),
            draws.iter().map(|draw| draw.y).min().unwrap(),
            draws.iter().map(|draw| draw.x + 24).max().unwrap(),
            draws.iter().map(|draw| draw.y + 24).max().unwrap()
        ],
        [0, 0, 3072, 1536]
    );
    let mut draw_hash = Sha256::new();
    for draw in &draws {
        draw_hash.update(draw.archive_entry_id.to_be_bytes());
        draw_hash.update(u16::from(draw.sub_index).to_be_bytes());
        draw_hash.update(draw.x.to_be_bytes());
        draw_hash.update(draw.y.to_be_bytes());
        draw_hash.update(draw.layer.to_be_bytes());
    }
    assert_eq!(
        hex(&draw_hash.finalize()),
        "ed0d7f817e53d619e9e2d8fb1ad93b3476b7be2a2f7c54a519349dcb4275351c"
    );
    let mut archive = SharedArchive::open_with_limits(
        root,
        SharedArchiveKind::Tile,
        ResourceLimits {
            max_entry_bytes: 16 * 1024 * 1024,
            ..Default::default()
        },
    )
    .unwrap();
    let mut all_hash = Sha256::new();
    let mut used_hash = Sha256::new();
    let mut encoded_bytes = 0usize;
    let mut all_blocks = 0usize;
    let mut referenced_blocks = 0usize;
    for id in &ids {
        let bytes = archive.read_entry(&format!("{id}.til")).unwrap();
        encoded_bytes = encoded_bytes.checked_add(bytes.len()).unwrap();
        assert!(encoded_bytes <= 64 * 1024 * 1024);
        let decoded = decode_classic_til_with_limits(
            &bytes,
            TileLimits {
                max_input_bytes: 16 * 1024 * 1024,
                ..Default::default()
            },
        )
        .unwrap_or_else(|error| panic!("full shared Tile/{id}.til qualification: {error}"));
        assert_eq!(
            decoded.tiles.len(),
            256,
            "shared Tile/{id}.til full block count"
        );
        all_blocks += decoded.tiles.len();
        assert!(all_blocks * 24 * 24 * 4 <= 64 * 1024 * 1024);
        for (subindex, tile) in decoded.tiles.iter().enumerate() {
            assert_eq!((tile.width, tile.height, tile.rgba.len()), (24, 24, 2304));
            let entry = tile_entry(*id, subindex.try_into().unwrap(), tile);
            all_hash.update(entry);
            if refs.contains(&(*id, subindex.try_into().unwrap())) {
                used_hash.update(entry);
                referenced_blocks += 1;
            }
        }
    }
    assert_eq!(encoded_bytes, 3_318_334);
    assert_eq!(all_blocks, 7424);
    assert_eq!(referenced_blocks, 4341);
    assert_eq!(
        hex(&all_hash.finalize()),
        "a7808700f27e8a1270b137fa82c367d09dc7e64ed6eafcb83913bc4c56eb23f8"
    );
    assert_eq!(
        hex(&used_hash.finalize()),
        "40b486ff24365f18365535e9fac35c6d0296307059093141ac93541b5b99f255"
    );

    let anchor_pixel = cell_to_sector_pixel(binding.key, anchor).unwrap();
    let camera = CameraRect {
        x: anchor_pixel.x - 400,
        y: anchor_pixel.y - 300,
        width: 800,
        height: 600,
    };
    assert_eq!((camera.x, camera.y), (1160, 372));
    let rendered = render_tiles(&mut archive, &draws, camera).unwrap();
    assert_eq!(rendered.visible_tiles, 4692);
    assert_eq!(rendered.tile_sets_loaded, 23);
    // C# coverage-preserving tile oracle + separately implemented RGBA8 alpha
    // composition. This is NOT native RGB555 frame equivalence or a GUI shot.
    assert_eq!(
        hex(&Sha256::digest(rendered.canvas.rgba())),
        "c79a635df962e27ba578a669b07675bb2fb40287e37c9211d44373d40eba27cc"
    );
    if std::env::var("LITO_WORLD_MAP_WRITE_REFERENCE").as_deref() == Ok("1") {
        // Explicit diagnostic render, never a live game/GUI screenshot.
        let directory =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("target/modern-lito-render-tests");
        std::fs::create_dir_all(&directory).unwrap();
        let output = directory.join("shared-map509-static-reference.png");
        let mut encoder = png::Encoder::new(std::fs::File::create(&output).unwrap(), 800, 600);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(rendered.canvas.rgba())
            .unwrap();
        eprintln!("Static shared 509 diagnostic: {}", output.display());
    }
    let mut wrong = binding;
    wrong.sha256[0] ^= 1;
    assert!(matches!(
        WorldMap::load_shared(root, &[wrong], &cancel),
        Err(WorldMapError::HashMismatch(_))
    ));
    // Same numeric-map or sector aliases are not accepted as a resource bind.
    let mut wrong_sector = binding.key;
    wrong_sector.sector_x = 0x8000;
    assert!(map.sector(wrong_sector).is_none());
}
