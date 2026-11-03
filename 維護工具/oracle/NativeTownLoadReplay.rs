//! Offline replay of selected captured initial packets. No network/GUI acceptance.
use lito_modern::modern_lito::{native_world::World,native_world_scene::Model};
use serde_json::{json,Value};
use std::{path::PathBuf,sync::atomic::AtomicBool,time::Instant};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let args:Vec<_>=std::env::args_os().collect();assert_eq!(args.len(),3);
 let assets=PathBuf::from(&args[1]);let out=PathBuf::from(&args[2]).canonicalize()?;
 assert!(out.starts_with(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize()?));
 let fixture:Value=serde_json::from_slice(&std::fs::read(out.join("fixture.json"))?)?;
 let cancel=AtomicBool::new(false);let mut retained_models=Vec::new();let mut records=Vec::new();
 for role in ["a","b"] {
  let data=&fixture["worlds"][role];let mut world=World::default();
  for row in data["packets"].as_array().unwrap(){let s=row["payload_hex"].as_str().unwrap();
   let bytes:Vec<u8>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
   assert!(world.accept(&bytes)?);
  }
  assert!(world.ready());assert_eq!(world.map_id,Some(500));assert_eq!(world.status.unwrap().object_id,u32::try_from(data["own_id"].as_u64().unwrap())?);
  assert_eq!(world.actors.len(),data["expected_actor_count"].as_u64().unwrap() as usize);
  let value=serde_json::to_value(&world)?;
  for (id,expected) in data["expected_actor_fields"].as_object().unwrap(){
   for (name,v) in expected.as_object().unwrap(){assert_eq!(&value["actors"][id][name],v,"captured actor {id}/{name}");}
  }
  let started=Instant::now();let mut model=Model::load(&assets,&world,&cancel)?;let load_us=started.elapsed().as_micros();
  let draw=Instant::now();let frame=model.draw_at(&world,&cancel,0)?;let draw_us=draw.elapsed().as_micros();
  records.push(json!({"role":role,"actors":world.actors.len(),"load_us":load_us,"first_draw_us":draw_us,
   "total_us":started.elapsed().as_micros(),"load_profile_us":frame.receipt["load_profile_us"],
   "render_profile_us":frame.receipt["render_profile_us"],"first_frame":frame.receipt}));
  retained_models.push(model);
 }
 std::fs::write(out.join("results.json"),serde_json::to_vec_pretty(&json!({"status":"offline_captured_town_Model_load_and_first_draw_diagnostic",
  "records":records,"retained_first_Model_while_loading_second":true,"actual_new_sessions":0,"GUI_verified":false,
  "not_full_controller_replay":true,"OS_cache_not_flushed":true,"other_host_load_not_excluded":true,"performance_accepted":false,"whole_goal_complete":false}))?)?;
 println!("Captured town replay: two Models loaded/drawn; offline diagnostic only");Ok(())
}
