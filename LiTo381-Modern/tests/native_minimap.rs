use lito_modern::modern_lito::{native_minimap as m,native_world::World,native_world_scene::Model};
use std::{path::Path,sync::atomic::AtomicBool};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn vectors()->serde_json::Value{serde_json::from_slice(include_bytes!("fixtures/native-minimap-current.json")).unwrap()}
fn world()->World{
 let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
 let mut w=World::default();for k in ["world","start","status","actor","bot","hp","mp"]{w.accept(&bytes(f["packets"][k].as_str().unwrap())).unwrap();}w
}
fn marker(id:u32,x:u16,y:u16,allied:bool)->Vec<u8>{let mut b=vec![0xe8];b.extend(id.to_le_bytes());b.extend([0,0]);b.extend(x.to_le_bytes());b.extend(y.to_le_bytes());b.extend([1,u8::from(allied)]);b}
#[test]
fn eighteen_formal_vectors_preserve_full_unsigned_identity_and_coordinates(){
 let f=vectors();assert_eq!(f["vectors"].as_array().unwrap().len(),18);
 for v in f["vectors"].as_array().unwrap(){let p=m::parse(&bytes(v["hex"].as_str().unwrap())).unwrap();assert_eq!(serde_json::to_value(p).unwrap(),serde_json::json!({"id":v["id"],"x":v["x"],"y":v["y"],"allied":v["allied"]}));}
}
#[test]
fn malformed_packets_never_partially_update_world_or_marker_state(){
 let b=marker(2,32721,32781,true);let mut w=world();w.accept(&b).unwrap();let before=serde_json::to_value(&w).unwrap();
 let mut bad:Vec<Vec<u8>>=(0..b.len()).map(|n|b[..n].to_vec()).collect();let mut tail=b.clone();tail.push(0);bad.push(tail);
 for (at,v) in [(1,0),(5,1),(6,1),(11,0),(11,2),(12,2)] {let mut p=b.clone();p[at]=v;bad.push(p);}
 for p in bad{assert!(w.accept(&p).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
 assert!(!w.unhandled_opcodes.contains(&0xe8));
}
#[test]
fn bounded_marker_replacement_and_delete_are_not_actor_predictions(){
 let mut s=m::State::default();for id in 1..=m::MAX_MARKERS as u32{s.accept(&marker(id,1,2,false)).unwrap();}
 let before=serde_json::to_value(&s).unwrap();assert!(s.accept(&marker(999999,1,2,false)).is_err());assert_eq!(before,serde_json::to_value(&s).unwrap());
 assert!(s.accept(&marker(1,3,4,true)).unwrap());assert!(!s.accept(&marker(1,3,4,true)).unwrap());assert!(s.remove(1));s.accept(&marker(999999,1,2,false)).unwrap();
 let mut w=world();let pos=[w.actors[&2].x,w.actors[&2].y];w.accept(&marker(2,0,65535,true)).unwrap();assert_eq!([w.actors[&2].x,w.actors[&2].y],pos);
 let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
 w.accept(&bytes(f["packets"]["bot"].as_str().unwrap())).unwrap();assert!(w.minimap.markers.is_empty());
 w.accept(&marker(2,0,65535,true)).unwrap();
 let mut remove=vec![0x15];remove.extend(2u32.to_le_bytes());w.accept(&remove).unwrap();assert!(w.minimap.markers.is_empty());
}
#[test]
fn new_map_same_map_portal_and_disconnect_retire_markers(){
 for mode in 0..3{let mut w=world();w.accept(&marker(2,32721,32781,true)).unwrap();let g=w.generation;
  match mode{0=>{w.accept(&[0x28,52,2,0]).unwrap();},1=>{let a=w.own_actor().unwrap();let mut p=vec![0x5a];p.extend(w.map_id.unwrap().to_le_bytes());p.extend(a.x.to_le_bytes());p.extend(a.y.to_le_bytes());w.accept(&p).unwrap();},_=>w.clear_preserving_generation()}
  assert!(w.minimap.markers.is_empty());assert_eq!(w.minimap.received,0);assert!(!w.ready());assert!(w.generation>=g);
 }
 let mut w=World::default();assert!(!w.accept(&marker(2,0,0,false)).unwrap());assert!(w.minimap.markers.is_empty());
}
#[test]
fn original_geometry_overlay_filters_hidden_markers_and_blocks_world_hit_tests(){
 let mut w=world();let a=w.own_actor().unwrap();let center=[a.x,a.y];let own=a.id;
 let ally=marker(2,center[0]+3,center[1],true);w.accept(&ally).unwrap();w.accept(&marker(999,center[0]+5,center[1],false)).unwrap();
 let cancel=AtomicBool::new(false);let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&w,&cancel).unwrap();
 let mut f=model.draw_at(&w,&cancel,0).unwrap();let mini=&f.receipt["minimap"];
 assert_eq!(mini["visible"],true);assert!(mini["terrain_cells"].as_u64().unwrap()>100);assert_eq!(mini["markers"].as_array().unwrap().len(),1);
 assert_eq!(mini["markers"][0]["pixel"],serde_json::json!([706,104]));assert_eq!(mini["markers"][0]["kind"],"ally");
 for (x,y) in [(610,10),(700,100),(789,189)]{assert!(f.overlay_contains(x,y));assert!(f.pick(x,y,own).is_none());assert!(f.ground_cell(&w,x,y).is_none());}
 for (x,y) in [(609,10),(610,9),(790,189),(789,190)]{assert!(!f.overlay_contains(x,y));}
 let out=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-minimap");std::fs::create_dir_all(&out).unwrap();
 let file=std::fs::File::create(out.join("original-geometry-ally.png")).unwrap();let mut encoder=png::Encoder::new(file,800,600);encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.write_header().unwrap().write_image_data(f.canvas.rgba()).unwrap();
 w.actors.get_mut(&2).unwrap().draw_state.invisible=true;w.visual_revision+=1;
 f=model.draw_at(&w,&cancel,1_000_000).unwrap();assert!(f.receipt["minimap"]["markers"].as_array().unwrap().is_empty());
 w.actors.get_mut(&2).unwrap().draw_state.invisible=false;w.accept(&marker(2,center[0]+3,center[1],false)).unwrap();
 f=model.draw_at(&w,&cancel,2_000_000).unwrap();assert_eq!(f.receipt["minimap"]["markers"][0]["kind"],"other");
 assert!(m::project([0,0],[65535,65535]).is_none());assert_eq!(m::project([65535,65535],[65535,65535]),Some([90,91]));
}
