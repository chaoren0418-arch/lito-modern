//! Read-only consumer audit using the existing Modern WorldMap parser.
//! Geometry does not qualify player spawns, objectives or Native aliases.
use lito_modern::modern_lito::{
    resources::SharedMapSectorKey,
    world_map::{SectorBinding, WorldCell, WorldMap},
};
use serde_json::{json, Value};
use std::{env, fs, path::Path, sync::atomic::AtomicBool};

fn digest(text: &str) -> [u8; 32] {
    assert_eq!(text.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).unwrap())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn main() {
    let args: Vec<_> = env::args_os().collect();
    assert_eq!(args.len(), 3, "explicit component JSON and Shared root required");
    let input: Value = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let mut results = Vec::new();
    let rows = input["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 9);
    for row in rows {
        assert_eq!(row["mode_qualified"], false);
        let id = u32::try_from(row["physical_map"].as_u64().unwrap()).unwrap();
        let bindings: Vec<_> = row["sector_pins"].as_array().unwrap().iter().map(|pin| {
            let sector = pin["sector"].as_str().unwrap();
            assert_eq!(sector.len(), 8);
            SectorBinding {
                key: SharedMapSectorKey {
                    map_id: id,
                    sector_x: u16::from_str_radix(&sector[..4], 16).unwrap(),
                    sector_y: u16::from_str_radix(&sector[4..], 16).unwrap(),
                },
                byte_length: u32::try_from(pin["bytes"].as_u64().unwrap()).unwrap(),
                sha256: digest(pin["sha256"].as_str().unwrap()),
            }
        }).collect();
        let map = WorldMap::load_shared(Path::new(&args[2]), &bindings, &AtomicBool::new(false)).unwrap();
        assert_eq!(hex(&map.canonical_digest()), row["manifest_sha256"].as_str().unwrap());
        let standing = bindings.iter().map(|binding| {
            let (ox, oy) = binding.key.world_origin();
            (0..64).flat_map(|y| (0..64).map(move |x| WorldCell { x: ox+x, y: oy+y }))
                .filter(|&cell| map.standing(cell)).count()
        }).sum::<usize>();
        assert_eq!(standing as u64, row["standable_cells"].as_u64().unwrap());
        results.push(json!({"physical_map": id, "sectors": bindings.len(), "geometry_sha256": hex(&map.canonical_digest()), "standable_cells": standing, "mode_qualified": false}));
    }
    println!("{}", serde_json::to_string_pretty(&json!({"rows":results,"scope":"Independent Modern WorldMap parsing/canonical digest/standing count, without Native lookup or spawn/objective/race qualification"})).unwrap());
}
