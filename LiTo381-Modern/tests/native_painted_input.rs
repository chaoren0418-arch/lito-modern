use lito_modern::modern_lito::{native_world::World,native_world_scene::Model};
use serde_json::Value;
use std::{path::Path,sync::atomic::AtomicBool};
fn world()->World {
 let f:Value=serde_json::from_slice(include_bytes!("fixtures/native-painted-input-current.json")).unwrap();
 let mut w=World::default();
 for s in f["packets"].as_array().unwrap(){let s=s.as_str().unwrap();let b:Vec<u8>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
 assert!(w.ready());w
}
#[test]
fn painted_masks_and_camera_survive_unpainted_idle_ticks_without_copying_canvas() {
 let w=world();let stop=AtomicBool::new(false);
 let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&w,&stop).unwrap();
 let frame=model.draw_at(&w,&stop,0).unwrap();let surface=frame.input_surface(&w).unwrap();
 let own=w.own_actor().unwrap().id;
 let (id,mask)=frame.visibility.iter().find(|(id,_)|*id==3).unwrap();
 let point=(0..600).flat_map(|y|(0..800).map(move |x|(x,y))).find(|&(x,y)|mask.contains(x,y) && frame.pick_current(&w,x as i32,y as i32,own)==Some(*id)).unwrap();
 assert_eq!(surface.pick_current(&w,point.0 as i32,point.1 as i32,own),Some(*id));
 let ground=surface.ground_cell(&w,500,350);
 let newer=model.draw_at(&w,&stop,200_000_000).unwrap();
 assert_ne!(newer.receipt["elapsed_ns"],frame.receipt["elapsed_ns"]);
 drop(frame);drop(newer); // host retains only the Arc-backed painted selection surface
 assert!(surface.input_ready(&w));
 assert_eq!(surface.pick_current(&w,point.0 as i32,point.1 as i32,own),Some(3));
 assert_eq!(surface.ground_cell(&w,500,350),ground);
 assert!(surface.ground_cell(&w,-1,350).is_none());assert!(surface.ground_cell(&w,800,350).is_none());
 assert!(surface.pick_current(&w,700,20,own).is_none()); // original minimap overlay
}
#[test]
fn painted_surface_rejects_portals_updates_actor_life_and_control_interruptions() {
 let original=world();let stop=AtomicBool::new(false);
 let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&original,&stop).unwrap();
 let frame=model.draw_at(&original,&stop,0).unwrap();let surface=frame.input_surface(&original).unwrap();
 for change in 0..9 {
  let mut w=original.clone();let own=w.own_actor().unwrap().id;
  match change {0=>w.generation+=1,1=>w.visual_revision+=1,
   2=>w.actors.get_mut(&own).unwrap().incarnation+=1,3=>w.local_corrections+=1,
   4=>w.movement_locks+=1,5=>w.control_interruptions+=1,
   6=>w.actors.get_mut(&own).unwrap().motion_revision+=1,
   7=>w.actors.get_mut(&own).unwrap().dead=true,_=>w.battle_result=Some(0)}
  assert!(!surface.input_ready(&w),"change {change}");assert!(surface.pick_current(&w,400,300,own).is_none());assert!(surface.ground_cell(&w,500,350).is_none());
 }
 let mut w=original.clone();w.accept(&[0x5a,0x34,2,0xdf,0x7f,0x1f,0x80]).unwrap();
 assert!(!surface.input_ready(&w));assert!(surface.ground_cell(&w,500,350).is_none());
}
