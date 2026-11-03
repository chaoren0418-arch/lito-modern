use lito_modern::modern_lito::{world_map::{SectorInput,WorldMap},resources::SharedMapSectorKey,scene::static_draw_list};
use std::{path::Path,sync::atomic::AtomicBool,collections::BTreeSet};
#[test]
fn original_rental_city_has_bounded_complete_geometry(){
    let root=Path::new(r"D:\lineage開發\0-客戶端\LiTo\LTmap\500");let mut files:Vec<_>=std::fs::read_dir(root).unwrap().map(|e|e.unwrap().path()).filter(|p|p.extension().is_some_and(|x|x=="s32")).collect();files.sort();
    let inputs=files.iter().map(|p|{let name=p.file_stem().unwrap().to_str().unwrap();SectorInput{key:SharedMapSectorKey{map_id:500,sector_x:u16::from_str_radix(&name[..4],16).unwrap(),sector_y:u16::from_str_radix(&name[4..],16).unwrap()},bytes:std::fs::read(p).unwrap()}}).collect();
    let map=WorldMap::from_native_sector_bytes(inputs,&AtomicBool::new(false)).unwrap();let mut draws=0;let mut unique=BTreeSet::new();
    for b in map.bindings(){let tiles=static_draw_list(map.sector(b.key).unwrap()).unwrap();draws+=tiles.len();for t in tiles {unique.insert((t.archive_entry_id,t.sub_index));}}
    assert_eq!(map.bindings().len(),18);assert!(draws>262_144&&draws<=1_048_576);
    let result=serde_json::json!({"map":500,"sectors":map.bindings().len(),"draw_references":draws,"unique_tile_slots":unique.len(),"canonical_sha256":map.canonical_digest(),"files":files});
    use lito_modern::modern_lito::{native_resources as nr,native_multisector_scene::Scene};
    let cancel=AtomicBool::new(false);let source=Path::new(r"D:\lineage開發\0-客戶端\LiTo");
    let observed=nr::NativeObservedArchives::open(source,nr::Limits::default(),&cancel).unwrap();
    let plan=nr::NativeAssetPlan{namespace:nr::ResourceNamespace::NativeLitoV1,map_id:500,canonical_sha256:map.canonical_digest(),archives:observed.pins().to_vec(),
        sectors:map.bindings().iter().map(|b|nr::SectorPin{key:nr::NativeSectorKey{map_id:500,x:b.key.sector_x,y:b.key.sector_y},pin:nr::BytePin{bytes:u64::from(b.byte_length),sha256:b.sha256}}).collect()};
    let sources=nr::NativeSources{map_root:source.join("LTmap"),archive_root:source.to_path_buf()};
    let assets=nr::NativeResourceAdapter::pinned_world().load_static_observed(&sources,&plan,nr::Limits::default(),observed,&cancel).unwrap();
    assert_eq!(assets.draw_references(),draws);assert_eq!(assets.draw_reference_limit(),1_048_576);assert_eq!(assets.decoded_slots(),unique.len());assert!(assets.decoded_bytes()<=64<<20);
    let scene=Scene::new_cancel(assets,&cancel).unwrap();assert_eq!(scene.assets().draw_references(),draws);
    let output=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/rental-city-geometry.json");std::fs::create_dir_all(output.parent().unwrap()).unwrap();std::fs::write(output,serde_json::to_vec_pretty(&result).unwrap()).unwrap();println!("{result}");
}
