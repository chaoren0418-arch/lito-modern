use lito_modern::modern_lito::{compositor::Canvas,native_terrain_pixels::{self as terrain,Gate,TILE_BYTES},tile};
use sha2::{Digest,Sha256};
use std::collections::BTreeSet;
fn hex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn source(block:&[u8])->(tile::DecodedTile,tile::DecodedTile){
    let mut data=1u32.to_le_bytes().to_vec();data.extend(0u32.to_le_bytes());data.extend((block.len() as u32).to_le_bytes());data.extend(block);
    let normal=tile::decode_classic_til(&data).unwrap().tiles.remove(0);
    let raw=tile::decode_classic_til_with_policy(&data,Default::default(),tile::TileZeroPolicy::RawRgb555Coverage).unwrap().tiles.remove(0);(normal,raw)
}
#[test]
fn original_floor_dispatch_64_whole_surfaces_match_real_tile_decoder_and_gate(){
    let fixture=include_bytes!("fixtures/native-floor-gate-original.json");
    assert_eq!(format!("{:X}",Sha256::digest(fixture)),"1B0AAD36AD82B594B7AC535EE86A1EBC536788B62A78F82732BF3BE333BBCFC3");
    let v:serde_json::Value=serde_json::from_slice(fixture).unwrap();assert_eq!(v["format"],"LITO_FLOOR_GATE_ORIGINAL1");
    let cases=v["cases"].as_array().unwrap();assert_eq!(cases.len(),64);let mut ids=BTreeSet::new();
    for case in cases {
        let kind=case["type"].as_u64().unwrap() as u8;let parity=case["parity"].as_u64().unwrap() as u8;let map=case["map"].as_u64().unwrap();
        assert!(ids.insert((kind,parity,map)));let block=hex(v["sources"][kind.to_string()].as_str().unwrap());let (normal,raw)=source(&block);
        let mut rgba=[0;TILE_BYTES];let drawn=terrain::floor_pixels(normal.tile_type,&normal.rgba,Some(&raw.rgba),parity,&mut rgba).unwrap();assert_eq!(drawn,kind&8==0);
        let mut canvas=Canvas::new(744,552).unwrap();canvas.clear([239,222,189]);
        let mut gate=Gate::new(744,552).unwrap();
        if drawn {canvas.blit_rgba(&rgba,24,24,180,250).unwrap();if map==303 {gate.mark(&rgba,180,250).unwrap();}}
        let packed:Vec<_>=canvas.rgba().chunks_exact(4).flat_map(|p|(((p[0] as u16>>3)<<10)|((p[1] as u16>>3)<<5)|(p[2] as u16>>3)).to_le_bytes()).collect();
        assert_eq!(format!("{:X}",Sha256::digest(&packed)),case["frame_sha256"].as_str().unwrap(),"type{kind} parity{parity} map{map}");
        assert_eq!(format!("{:X}",Sha256::digest(gate.values())),case["gate_sha256"].as_str().unwrap(),"gate type{kind} parity{parity} map{map}");
        assert_eq!(gate.covered(),if map==303 {case["written_pixels"].as_u64().unwrap() as usize}else{0});
    }
}
#[test]
fn clipped_gate_follows_actual_canvas_black_coverage_and_extreme_positions(){
    let mut rgba=[0u8;TILE_BYTES];for y in 0..24 {for x in 0..24 {if (x+3*y)%5!=0 {rgba[(y*24+x)*4+3]=255;}}}
    for [x,y] in [[-12,-12],[17,19],[-23,4],[4,-23],[i32::MIN,0],[i32::MAX,0],[0,i32::MIN],[0,i32::MAX]] {
        let mut canvas=Canvas::new(32,32).unwrap();canvas.clear([255,255,255]);canvas.blit_rgba(&rgba,24,24,x,y).unwrap();
        let mut gate=Gate::new(32,32).unwrap();gate.mark(&rgba,x,y).unwrap();let first=gate.values().to_vec();gate.mark(&rgba,x,y).unwrap();assert_eq!(first,gate.values());
        for (p,g) in canvas.rgba().chunks_exact(4).zip(gate.values()){assert_eq!(*g,if p[0]==0 {1}else{0});}
        assert_eq!(gate.covered(),gate.values().iter().filter(|v|**v!=0).count());
    }
}
#[test]
fn invalid_floor_or_gate_inputs_fail_before_mutation(){
    let mut output=[71u8;TILE_BYTES];let normal=[0u8;TILE_BYTES];
    assert!(terrain::floor_pixels(tile::ClassicTileType::DiamondRightCloudAlpha,&normal,None,0,&mut output).is_err());assert_eq!(output,[71;TILE_BYTES]);
    assert!(terrain::floor_pixels(tile::ClassicTileType::DiamondRight,&normal,None,2,&mut output).is_err());assert_eq!(output,[71;TILE_BYTES]);
    let mut bad=normal;bad[3]=1;let mut gate=Gate::new(24,24).unwrap();assert!(gate.mark(&bad,0,0).is_err());assert_eq!(gate.covered(),0);assert!(gate.values().iter().all(|x|*x==0));
    assert!(Gate::new(0,24).is_err());assert!(Gate::new(1025,24).is_err());
}
