//! Offline pinned Native asset experiment. No wire admission or Shared fallback.
#![allow(dead_code)]
#[path="native-resource-adapter.rs"] mod native;
#[path="../src/modern_lito/tile.rs"] mod tile;
use lito_modern::modern_lito::{compositor::Canvas,map::parse_s32,resources::SharedMapSectorKey,scene::{project_half_tile,DrawTile,CameraRect},sprite::decode_classic_spr,world_map::{SectorInput,WorldCell,WorldMap}};
use native::{hex,NativeArchive,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{collections::{BTreeMap,BTreeSet},env,fs,path::Path,sync::atomic::AtomicBool};
fn write_png(path:&Path,canvas:&Canvas)->Result<()> {
    let file=fs::OpenOptions::new().create_new(true).write(true).open(path)?;
    let mut encoder=png::Encoder::new(file,canvas.width(),canvas.height());
    encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(canvas.rgba())?;Ok(())
}
fn render(tiles:&[DrawTile],cache:&BTreeMap<(u16,u8),tile::DecodedTile>,camera:CameraRect)->Result<Canvas> {
    if camera.width>4096 || camera.height>4096 {return Err("Native diagnostic camera budget exceeded".into());}
    let mut canvas=Canvas::new(camera.width,camera.height)?;
    canvas.clear([18,25,35]);
    let mut ordered:Vec<_>=tiles.iter().collect();ordered.sort_by_key(|t|t.layer);
    for t in ordered {
        let decoded=cache.get(&(t.archive_entry_id,t.sub_index)).ok_or("required Native tile block absent")?;
        canvas.blit_rgba(&decoded.rgba,decoded.width,decoded.height,t.x-camera.x,t.y-camera.y)?;
    }
    Ok(canvas)
}
fn main()->Result<()> {
    let args:Vec<_>=env::args_os().collect();if args.len()!=3 {return Err("explicit pinned consumer input and new output directory required".into());}
    let input_path=Path::new(&args[1]);let root=input_path.parent().ok_or("Native input has no root")?;
    let input:Value=serde_json::from_slice(&fs::read(input_path)?)?;
    let output=Path::new(&args[2]);if !output.is_dir() {return Err("owned output directory absent".into());}
    let pin=&input["native_map"];
    if input["candidate_id"]!="native-dm564-geometry.20261004.1" || pin["logical_map"]!=564 || pin["sector"]!="7fff8000" {return Err("candidate binding differs from fixed handoff".into());}
    let bytes=native::read_pinned(root,pin,"NATIVE_LITO_LTMAP_V1",8*1024*1024)?;
    let sector=parse_s32(&bytes)?;
    if !sector.unknown_trailing.is_empty() || sector.opacity.len()!=390 || sector.effects.len()!=5 {return Err("Native sector layout differs".into());}
    let key=SharedMapSectorKey{map_id:564,sector_x:0x7fff,sector_y:0x8000};
    // Geometry's numeric coordinate key does not select a resource provider.
    let world=WorldMap::from_sector_bytes(vec![SectorInput{key,bytes}],&AtomicBool::new(false))?;
    if hex(&world.canonical_digest())!=pin["canonical_sha256"].as_str().ok_or("canonical pin absent")? {return Err("Native canonical geometry pin differs".into());}
    let (ox,oy)=key.world_origin();
    let standing=(0..64).flat_map(|y|(0..64).map(move|x|WorldCell{x:ox+x,y:oy+y})).filter(|&c|world.standing(c)).count();
    if standing!=760 {return Err("Native standing count differs".into());}
    let pins=input["archive_pins"].as_array().ok_or("Native archive pins absent")?;
    let tiles=NativeArchive::open(root,pins,false)?;
    let mut draw=Vec::new();
    for (i,t) in sector.floor.iter().enumerate(){let (x,y)=project_half_tile((i%128) as u8,(i/128) as u8);draw.push(DrawTile{half_x:(i%128) as u8,half_y:(i/128) as u8,archive_entry_id:t.tile_id,sub_index:t.subtile,x,y,layer:-2});}
    for t in &sector.overlays{let(x,y)=project_half_tile(t.x,t.y);draw.push(DrawTile{half_x:t.x,half_y:t.y,archive_entry_id:t.tile.tile_id,sub_index:t.tile.subtile,x,y,layer:-1});}
    for g in &sector.objects{for t in &g.tiles{let id=u16::try_from(t.tile_id).map_err(|_|"negative Native object Tile ID")?;let(x,y)=project_half_tile(t.x,t.y);draw.push(DrawTile{half_x:t.x,half_y:t.y,archive_entry_id:id,sub_index:t.subtile,x,y,layer:i32::from(t.layer)});}}
    let mut references:BTreeMap<u16,BTreeSet<usize>>=BTreeMap::new();for t in &draw{references.entry(t.archive_entry_id).or_default().insert(usize::from(t.sub_index));}
    let expected=input["tile_dependencies"].as_array().ok_or("Native dependency list absent")?;
    let declared:BTreeSet<_>=expected.iter().map(|r|r["name"].as_str().unwrap().to_owned()).collect();
    let required:BTreeSet<_>=references.keys().map(|id|format!("{id}.til")).collect();
    if declared.len()!=32 || required!=declared {return Err("Native required tile names differ from full map dependency list".into());}
    let mut cache=BTreeMap::new();let mut tile_rows=Vec::new();let mut negatives=Vec::new();
    let variant=tiles.entry("1026.til")?;
    let variant_sha=hex(&Sha256::digest(variant));
    if variant.len()!=40295 || !variant_sha.eq_ignore_ascii_case(input["empty_tail_variant"]["sha256"].as_str().unwrap()) || variant!=fs::read(input["repro"].as_str().ok_or("Native repro absent")?)? {return Err("exact Native1026 entry/repro differs".into());}
    let count=u32::from_le_bytes(variant[..4].try_into()?);let header=4+(count as usize+1)*4;
    let end=u32::from_le_bytes(variant[header-4..header].try_into()?);
    if count!=256 || header!=1032 || end!=39263 {return Err("Native1026 table identity differs".into());}
    let offsets:Vec<_>=(0..256).map(|i|u32::from_le_bytes(variant[4+i*4..8+i*4].try_into().unwrap())).collect();
    if offsets[..106].iter().any(|o|*o>=end) || offsets[106..].iter().any(|o|*o!=end) || references[&1026].last()!=Some(&105) {return Err("Native1026 selected/terminal-empty contract differs".into());}
    let repro:Value=serde_json::from_slice(&fs::read(root.join("producer/minimal-native-tile-failure.json"))?)?;
    let repro_ids:BTreeSet<_>=repro["required_references"]["objects"]["subtiles"].as_array().unwrap().iter().map(|i|i.as_u64().unwrap() as usize).collect();
    if references[&1026]!=repro_ids || draw.iter().filter(|t|t.archive_entry_id==1026).count()!=583 {return Err("Native1026 full map references differ from repro".into());}
    for (id,selected) in &references {
        let name=format!("{id}.til");let raw=tiles.entry(&name)?;
        let row=expected.iter().find(|r|r["name"]==name).ok_or("required entry pin absent")?;
        if raw.len() as u64!=row["bytes"].as_u64().unwrap() || !hex(&Sha256::digest(raw)).eq_ignore_ascii_case(row["native_payload_sha256"].as_str().unwrap()) {return Err("Native tile payload pin differs".into());}
        let strict=tile::decode_classic_til(raw);
        let decoded=if *id==1026 {
            if strict.is_ok() || tile::decode_classic_til_selected(raw,tile::TileLimits::default(),selected).is_ok(){return Err("strict Native1026 guard changed".into());}
            tile::decode_classic_til_selected_with_empty_tail(raw,tile::TileLimits::default(),selected)?
        }else{
            strict?;tile::decode_classic_til_selected(raw,tile::TileLimits::default(),selected)?
        };
        let mut pixel_digest=Sha256::new();for (sub,t) in &decoded{pixel_digest.update((*sub as u32).to_le_bytes());pixel_digest.update(&t.rgba);}
        tile_rows.push(json!({"name":name,"bytes":raw.len(),"sha256":hex(&Sha256::digest(raw)),"selected_ids":selected,"decoded_slots":decoded.len(),"selected_rgba_with_ids_sha256":hex(&pixel_digest.finalize()),"strict_full_decodes":*id!=1026,"explicit_empty_tail_variant":*id==1026}));
        for(sub,t)in decoded{cache.insert((*id,sub as u8),t);}
    }
    let decoded_bytes:usize=cache.values().map(|t|t.rgba.len()).sum();if decoded_bytes>64*1024*1024 {return Err("Native decoded cache budget exceeded".into());}
    let mut check=|name:&str,rejected:bool|->Result<()>{if !rejected{return Err(format!("negative admission accepted: {name}").into());}negatives.push(json!({"name":name,"rejected":true}));Ok(())};
    check("selected_empty106",tile::decode_classic_til_selected_with_empty_tail(variant,tile::TileLimits::default(),&[106].into()).is_err())?;
    check("selected_empty255",tile::decode_classic_til_selected_with_empty_tail(variant,tile::TileLimits::default(),&[255].into()).is_err())?;
    let mut beyond=variant.to_vec();beyond[4+255*4..8+255*4].copy_from_slice(&(end+1).to_le_bytes());check("unselected_beyond_eof",tile::decode_classic_til_selected_with_empty_tail(&beyond,tile::TileLimits::default(),&references[&1026]).is_err())?;
    let mut interior=variant.to_vec();interior[4+5*4..8+5*4].copy_from_slice(&end.to_le_bytes());check("interior_empty_slot",tile::decode_classic_til_selected_with_empty_tail(&interior,tile::TileLimits::default(),&references[&1026]).is_err())?;
    check("truncated_native1026",tile::decode_classic_til_selected_with_empty_tail(&variant[..variant.len()-1],tile::TileLimits::default(),&references[&1026]).is_err())?;
    let mut wrong_namespace=pin.clone();wrong_namespace["namespace"]=json!("SHARED_WORLD_V1");check("map_namespace",native::read_pinned(root,&wrong_namespace,"NATIVE_LITO_LTMAP_V1",8*1024*1024).is_err())?;
    let mut wrong_sha=pin.clone();wrong_sha["sha256"]=json!("00".repeat(32));check("map_raw_sha",native::read_pinned(root,&wrong_sha,"NATIVE_LITO_LTMAP_V1",8*1024*1024).is_err())?;
    let mut wrong_pins=pins.clone();for p in &mut wrong_pins{p["namespace"]=json!("SHARED_TILE_V1");}check("archive_namespace",NativeArchive::open(root,&wrong_pins,false).is_err())?;
    check("missing_required_entry_no_fallback",tiles.entry("unprovided-native.til").is_err())?;
    let idx_pin=pins.iter().find(|p|p["path"].as_str().unwrap().ends_with("LTtile.idx")).unwrap();let idx=native::read_pinned(root,idx_pin,"NATIVE_LITO_LTTILE_V1",8*1024*1024)?;
    let mut bad_idx=idx.clone();bad_idx[..4].copy_from_slice(&u32::MAX.to_le_bytes());check("index_count_budget",native::parse_index(&bad_idx,148485306).is_err())?;
    let mut bad_offset=idx.clone();let last=bad_offset.len()-28;bad_offset[last..last+4].copy_from_slice(&u32::MAX.to_le_bytes());check("unused_index_entry_bounds",native::parse_index(&bad_offset,148485306).is_err())?;
    let left=draw.iter().map(|t|t.x).min().unwrap();let top=draw.iter().map(|t|t.y).min().unwrap();let right=draw.iter().map(|t|t.x+24).max().unwrap();let bottom=draw.iter().map(|t|t.y+24).max().unwrap();
    let full_camera=CameraRect{x:left,y:top,width:(right-left)as u32,height:(bottom-top)as u32};
    let full=render(&draw,&cache,full_camera)?;let full_path=output.join("native-dm564-static-full.png");write_png(&full_path,&full)?;
    let camera=CameraRect{x:1136,y:484,width:800,height:600};let sample=render(&draw,&cache,camera)?;let sample_path=output.join("native-dm564-static-camera.png");write_png(&sample_path,&sample)?;
    let sprites=NativeArchive::open(root,pins,true)?;let mut sprite_rows=Vec::new();
    // These names are observed in the pinned Native index. Frame0 is an asset
    // inspection sample, not a recovered action/duration binding for Layer8.
    for (id,name) in [(38u16,"38-0.spr"),(123u16,"123-0.spr")] {
        let raw=sprites.entry(name)?;let decoded=decode_classic_spr(raw)?;let mut h=Sha256::new();for f in &decoded.frames{h.update(f.width.to_le_bytes());h.update(f.height.to_le_bytes());h.update(&f.rgba);}
        let frame=decoded.frames.first().ok_or("Native sprite has no frame")?;
        let mut canvas=Canvas::new(frame.width,frame.height)?;canvas.blit_rgba(&frame.rgba,frame.width,frame.height,0,0)?;
        let path=output.join(format!("native-effect-{id}-observed-file-frame0.png"));write_png(&path,&canvas)?;
        sprite_rows.push(json!({"sprite_id":id,"observed_native_entry":name,"bytes":raw.len(),"sha256":hex(&Sha256::digest(raw)),"decoded_frames":decoded.frames.len(),"frames_rgba_sha256":hex(&h.finalize()),"inspection_png":path,"layer8_action_binding_qualified":false}));
    }
    let report=json!({"status":"passed_offline_native_assets_and_static_cpu","candidate":input["candidate_id"],"namespace":pin["namespace"],"tile_namespace":tiles.namespace,"sprite_namespace":sprites.namespace,"canonical_sha256":hex(&world.canonical_digest()),"standable_cells":standing,"native_tile_entries_verified":tile_rows.len(),"tile_rows":tile_rows,"decoded_required_slots":cache.len(),"decoded_cache_bytes":decoded_bytes,"native1026":{"table_slots_preserved":count,"encoded_end":end,"first_empty_slot":106,"required_reference_count":583,"required_ids":references[&1026],"strict_default_still_rejects":true,"bytes_not_clipped_or_rewritten":true},"negative_checks":negatives,"static_floor_object_references":draw.len(),"full_render":{"png":full_path,"width":full.width(),"height":full.height(),"rgba_sha256":hex(&Sha256::digest(full.rgba()))},"camera_render":{"png":sample_path,"rgba_sha256":hex(&Sha256::digest(sample.rgba()))},"opacity":{"records_parsed":390,"actor_condition_or_roof_fade_qualified":0,"coverage":"Static objects retained;390 records not interpreted as actor-driven roof policy"},"effects":{"records_parsed":5,"unique_sprite_ids":[38,123],"native_asset_inspection":sprite_rows,"world_animation_or_action_binding_qualified":0,"coverage":"Two observed Native SPR files decoded and frame0 CPU inspection composed. No guess about Layer8 action, duration, depth or world animation"},"mode_additions":0,"gui_accepted":false,"wire_tls_sql_combat_settlement_accepted":false});
    println!("{}",serde_json::to_string_pretty(&report)?);Ok(())
}
