//! Pinned Shared terrain diagnostic. A CPU render is not a game screenshot.
use lito_modern::modern_lito::{
    resources::{SharedArchive, SharedArchiveKind, SharedMapSectorKey},
    scene::{render_tiles, static_draw_list, CameraRect},
    world_map::{cell_to_sector_pixel, SectorBinding, WorldCell, WorldMap},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, fs, path::Path, sync::atomic::AtomicBool};
fn digest(text: &str) -> [u8; 32] {
    assert_eq!(text.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&text[i*2..i*2+2], 16).unwrap())
}
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn inspect(row: &Value, root: &Path, output: &Path) -> Result<Value, String> {
    let id = u32::try_from(row["physical_map"].as_u64().unwrap()).unwrap();
    let bindings: Vec<_> = row["sector_pins"].as_array().unwrap().iter().map(|pin| {
        let sector = pin["sector"].as_str().unwrap();
        assert_eq!(sector.len(), 8);
        SectorBinding {
            key: SharedMapSectorKey { map_id:id, sector_x:u16::from_str_radix(&sector[..4],16).unwrap(), sector_y:u16::from_str_radix(&sector[4..],16).unwrap() },
            byte_length:u32::try_from(pin["bytes"].as_u64().unwrap()).unwrap(),
            sha256:digest(pin["sha256"].as_str().unwrap()),
        }
    }).collect();
    let map = WorldMap::load_shared(root, &bindings, &AtomicBool::new(false)).map_err(|e| e.to_string())?;
    if hex(&map.canonical_digest()) != row["manifest_sha256"].as_str().unwrap() { return Err("geometry digest differs".into()); }
    // Select a camera sample only. This is not an authoritative player spawn.
    let mut samples: Vec<_> = bindings.iter().filter_map(|binding| {
        let (ox,oy)=binding.key.world_origin();
        let mut cells: Vec<_>=(0..64).flat_map(|y| (0..64).map(move |x| WorldCell{x:ox+x,y:oy+y})).filter(|&c|map.standing(c)).collect();
        cells.sort_by_key(|c| ((c.x-ox-32).abs()+(c.y-oy-32).abs(),c.y,c.x));
        cells.first().copied().map(|cell|(cells.len(),binding.key,cell))
    }).collect();
    samples.sort_by_key(|(count,key,_)| (std::cmp::Reverse(*count),key.sector_x,key.sector_y));
    let (_,key,cell)=samples.first().copied().ok_or("no standable camera sample")?;
    let anchor=cell_to_sector_pixel(key,cell).ok_or("camera projection failed")?;
    let camera=CameraRect{x:anchor.x-400,y:anchor.y-300,width:800,height:600};
    let sector=map.sector(key).ok_or("sample sector absent")?;
    let draw=static_draw_list(sector).map_err(|e|e.to_string())?;
    let mut archive=SharedArchive::open(root,SharedArchiveKind::Tile).map_err(|e|e.to_string())?;
    let scene=render_tiles(&mut archive,&draw,camera).map_err(|e|format!("map{id} sector{}: {e:#}",key.file_name()))?;
    if scene.visible_tiles==0 || scene.tile_sets_loaded==0 { return Err("empty terrain render".into()); }
    let path=output.join(format!("shared-{id}-static-camera.png"));
    let file=fs::OpenOptions::new().create_new(true).write(true).open(&path).map_err(|e|e.to_string())?;
    let mut encoder=png::Encoder::new(file,scene.canvas.width(),scene.canvas.height());
    encoder.set_color(png::ColorType::Rgba); encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().map_err(|e|e.to_string())?.write_image_data(scene.canvas.rgba()).map_err(|e|e.to_string())?;
    let limits=map.art_limitations(key).ok_or("missing art limits")?;
    Ok(json!({"physical_map":id,"passed":true,"sector":key.file_name(),"camera_sample_cell":[cell.x,cell.y],"camera_is_player_spawn":false,"visible_tiles":scene.visible_tiles,"tile_sets_loaded":scene.tile_sets_loaded,"rgba_sha256":hex(&Sha256::digest(scene.canvas.rgba())),"png":path,"known_art_limits":{"opacity_records":limits.opacity_records,"sprite_effects":limits.sprite_effects,"portals":limits.portals,"negative_object_tiles":limits.negative_object_tiles},"mode_qualified":false,"gui_accepted":false}))
}
fn main() {
    let args:Vec<_>=env::args_os().collect(); assert_eq!(args.len(),4);
    let input:Value=serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let rows=input["rows"].as_array().unwrap(); assert_eq!(rows.len(),9);
    let output=Path::new(&args[3]); assert!(output.is_dir());
    let results:Vec<_>=rows.iter().map(|row|match inspect(row,Path::new(&args[2]),output){Ok(result)=>result,Err(error)=>json!({"physical_map":row["physical_map"],"passed":false,"error":error,"mode_qualified":false,"gui_accepted":false})}).collect();
    let passed=results.iter().filter(|r|r["passed"]==true).count();
    println!("{}",serde_json::to_string_pretty(&json!({"passed":passed,"failed":9-passed,"rows":results,"scope":"Actual Shared Tile archive CPU static camera samples only; no native framebuffer equivalence, actors, animation, conditional roofs or whole-map gameplay acceptance"})).unwrap());
    if passed!=9 { std::process::exit(1); }
}
