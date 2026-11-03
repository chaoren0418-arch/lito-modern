//! Diagnostic against the pinned current library; never selects a live map.
use lito_modern::modern_lito::{resources::{SharedMapSectorKey, SharedArchive, SharedArchiveKind},
    world_map::{WorldMap, SectorInput}, tile::{self, TileLimits}};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, BTreeSet}, path::{Path,PathBuf}, sync::atomic::AtomicBool,
    os::windows::fs::MetadataExt};

fn no_reparse(path:&Path)->std::io::Result<()> {
    assert!(path.is_absolute());
    for ancestor in path.ancestors() {
        if std::fs::symlink_metadata(ancestor)?.file_attributes() & 0x400 != 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput,"reparse component refused"));
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    assert_eq!(args.len(), 3);
    let root = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    assert!(root.is_absolute() && output.is_absolute());
    let directory = root.join("map/303");
    no_reparse(&directory)?;
    let mut paths = std::fs::read_dir(&directory)?.map(|e|e.map(|e|e.path())).collect::<Result<Vec<_>,_>>()?;
    paths.retain(|p|p.extension().is_some_and(|e|e=="s32"));
    paths.sort();
    assert_eq!(paths.len(), 30, "Original observed set must remain explicit.");
    let mut inputs = Vec::new();
    let mut total = 0;
    for path in paths {
        no_reparse(&path)?;
        let name = path.file_stem().unwrap().to_str().unwrap();
        assert!(name.len()==8 && name.bytes().all(|b|b.is_ascii_hexdigit()));
        let bytes = std::fs::read(&path)?;
        assert!(bytes.len() <= 8<<20);
        total += bytes.len();
        assert!(total <= 32<<20);
        inputs.push(SectorInput {key: SharedMapSectorKey {map_id:303,
            sector_x:u16::from_str_radix(&name[..4],16)?, sector_y:u16::from_str_radix(&name[4..],16)?},bytes});
    }
    let map = WorldMap::from_sector_bytes(inputs, &AtomicBool::new(false))?;
    let mut needed: BTreeMap<u16,BTreeSet<usize>> = BTreeMap::new();
    let mut object_refs: BTreeMap<(u16,u8),usize> = BTreeMap::new();
    let mut sectors = Vec::new();
    let mut negative_object_references = 0;
    for binding in map.bindings() {
        let s = map.sector(binding.key).unwrap();
        for t in s.floor.iter().copied().chain(s.overlays.iter().map(|v|v.tile)) {
            needed.entry(t.tile_id).or_default().insert(usize::from(t.subtile));
        }
        for g in &s.objects {
            for t in &g.tiles {
                match t.shared_tile_reference() {
                    Some(reference) => {
                        needed.entry(reference.tile_id).or_default().insert(usize::from(reference.subtile));
                        *object_refs.entry((reference.tile_id,reference.subtile)).or_default() += 1;
                    },
                    None => negative_object_references += 1,
                }
            }
        }
        sectors.push(json!({"sector_x":binding.key.sector_x,"sector_y":binding.key.sector_y,
            "bytes":binding.byte_length,"sha256":format!("{:X}",Sha256::digest(std::fs::read(directory.join(format!("{:04x}{:04x}.s32",binding.key.sector_x,binding.key.sector_y)))?)),
            "decoder_input_sha256":binding.sha256.iter().map(|v|format!("{v:02X}")).collect::<String>(),
            "floor_references":s.floor.len(),"overlay_references":s.overlays.len(),
            "object_groups":s.objects.len(),"object_references":s.objects.iter().map(|g|g.tiles.len()).sum::<usize>(),
            "opacity_records":s.opacity.len(),"portal_records":s.portals.len(),"sprite_effects":s.effects.len()}));
    }
    let mut archive = SharedArchive::open(&root,SharedArchiveKind::Tile)?;
    let mut tiles = Vec::new();
    let mut type_counts = BTreeMap::<u8,usize>::new();
    let mut raw_object_references = 0;
    let mut decoded_bytes = 0;
    for (id,slots) in needed {
        let name = format!("{id}.til");
        let bytes = archive.read_entry(&name)?;
        let decoded = tile::decode_classic_til_selected_with_empty_tail(&bytes, TileLimits::default(), &slots)?;
        assert_eq!(decoded.len(), slots.len());
        let mut selected = Vec::new();
        for (index,tile) in decoded {
            assert_eq!((tile.width,tile.height,tile.rgba.len()),(24,24,24*24*4));
            decoded_bytes += tile.rgba.len();
            let kind = tile.tile_type as u8;
            *type_counts.entry(kind).or_default() += 1;
            let objects = object_refs.get(&(id,u8::try_from(index)?)).copied().unwrap_or(0);
            if kind & 2 == 0 {raw_object_references += objects;}
            selected.push(json!({"slot":index,"type":kind,"rgba_sha256":format!("{:X}",Sha256::digest(&tile.rgba)),
                "object_reference_instances":objects,"object_RLE_coverage_candidate":kind&2!=0}));
        }
        tiles.push(json!({"member":name,"encoded_bytes":bytes.len(),"encoded_sha256":format!("{:X}",Sha256::digest(&bytes)),"selected":selected}));
    }
    let report = json!({"status":"decoded_current_library_original_classic_map303_geometry_and_referenced_tiles",
        "map_id":303,"namespace":"BASE map/303 + Tile","sector_count":sectors.len(),"source_bytes":total,
        "geometry_sha256":map.canonical_digest().iter().map(|v|format!("{v:02X}")).collect::<String>(),
        "sectors":sectors,"tile_members":tiles,"selected_type_counts":type_counts,"decoded_selected_RGBA_bytes":decoded_bytes,
        "negative_object_references":negative_object_references,"raw_object_references_requiring_original_RLE_provenance":raw_object_references,
        "normal_scene_map303_enabled":false,"background_scene_adapter_verified":false,"GUI_verified":false,
        "actual_formal_sessions":0,"full_map_rendering_verified":false,"whole_goal_complete":false});
    std::fs::write(output,serde_json::to_vec_pretty(&report)?)?;
    println!("map303: 30 original sectors and referenced tiles decoded; scene integration remains pending");
    Ok(())
}
