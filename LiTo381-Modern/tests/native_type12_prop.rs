use lito_modern::modern_lito::{native_actor_layers::Bank,native_world::World,native_world_animation::Animator,resources::{SharedArchive,SharedArchiveKind},sprite_actions::{ActionKey,ActionLimits,SpriteActionCatalog}};
use std::{path::Path,sync::atomic::AtomicBool};
use sha2::{Digest,Sha256};
fn bytes(h:&str)->Vec<u8>{(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect()}
#[test]
fn original_type12_is_a_single_image_not_an_invented_breath_action(){
 let root=Path::new(r"D:\lineage開發\0-客戶端");let raw=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let catalog=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
 let row=&catalog.sprites[&869];assert_eq!(row.image_count,Some(1));assert!(row.actions.is_empty());assert!(row.attributes.iter().any(|a|a.key==ActionKey::new(102)&&a.body_bytes==b"12"));assert!(row.attributes.iter().any(|a|a.key==ActionKey::new(104)&&a.body_bytes==b"1"));
 let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let evidence:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-type12-869.json")).unwrap();let mut w=World::default();for key in ["world","start","status","actor"]{w.accept(&bytes(f["packets"][key].as_str().unwrap())).unwrap();}w.accept(&bytes(evidence["captured_actor"].as_str().unwrap())).unwrap();
 let actor=&w.actors[&1677722876];assert_eq!(actor.graphics,869);let before=serde_json::to_value(actor).unwrap();let mut clock=Animator::default();
 for now in [0,1_000_000_000,60_000_000_000]{let pose=clock.pose(&catalog,actor,now).unwrap();assert!(pose.static_prop&&pose.action_complete);assert_eq!((pose.graphics,pose.image,pose.subframe),(869,0,0));assert!(pose.events.is_empty()&&pose.entered_events.is_empty());}
 assert_eq!(serde_json::to_value(actor).unwrap(),before);
 let mut bank=Bank::open(root).unwrap();bank.ensure("869-0.spr",&AtomicBool::new(false)).unwrap();let loaded=bank.get("869-0.spr").unwrap();assert_eq!(loaded.source.sha256,evidence["sprite"]["sha256"]);assert_eq!(loaded.sprite.frames.len(),1);let frame=&loaded.sprite.frames[0];assert_eq!((frame.width,frame.height,frame.x_offset,frame.y_offset),(72,72,-24,-96));assert!(frame.fragments.is_some());assert!(frame.rgba.chunks_exact(4).any(|c|c[3]>0));
 let plan=lito_modern::modern_lito::native_actor_layers::Plan::from_catalog(&catalog,869).unwrap();
 assert!(plan.body_half && plan.bind(0).unwrap().body_half);
 let mut moved=actor.clone();moved.motion_from=Some([actor.x-1,actor.y]);assert!(clock.pose(&catalog,&moved,0).is_err());moved.motion_from=None;moved.walk_action=1;assert!(clock.pose(&catalog,&moved,0).is_err());
 let report=serde_json::json!({"catalog_row":evidence["row"],"body":loaded.source,"rgba_sha256":format!("{:X}",Sha256::digest(&frame.rgba)),"static_source":true,"catalogue_flag1_body_half":true,"original_runtime_framebuffer_parity":false,"native_draw_flags_complete":false});
 std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/type12-869-body.json"),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}

#[test]
fn model_uses_original_catalogue_half_without_mutating_world_or_animation(){
 use lito_modern::modern_lito::native_world_scene::Model;
 let root=Path::new(r"D:\lineage開發\0-客戶端");
 let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
 let evidence:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-type12-869.json")).unwrap();
 let mut w=World::default();for key in ["world","start","status","actor"]{w.accept(&bytes(f["packets"][key].as_str().unwrap())).unwrap();}
 w.accept(&bytes(evidence["captured_actor"].as_str().unwrap())).unwrap();
 // Place the captured prop by the local actor for a deterministic camera fixture.
 // This is rendering coverage, not a server-authoritative WORLD placement claim.
 let own=w.own_actor().unwrap().clone();let npc=w.actors.get_mut(&1677722876).unwrap();npc.x=own.x+1;npc.y=own.y;
 let before=serde_json::to_value(&w).unwrap();let cancel=AtomicBool::new(false);
 let mut model=Model::load(root,&w,&cancel).unwrap();let mut receipts=Vec::new();
 for now in [0,1_000_000_000,60_000_000_000]{
  let frame=model.draw_at(&w,&cancel,now).unwrap();
  let row=frame.receipt["actors"].as_array().unwrap().iter().find(|a|a["id"]==1677722876u32).unwrap();
  assert_eq!(row["rendered"],true);assert_eq!(row["sprite_member"],"869-0.spr");assert_eq!(row["sprite_subframe"],0);
  assert_eq!(row["phase"],"static_prop");assert!(row["layer_error"].is_null());
  assert_eq!(row["body_presentation"]["body_half"],true);assert_eq!(row["body_draw_style"],"Half");
  assert_eq!(row["body_source"]["sha256"],evidence["sprite"]["sha256"]);
  assert!(row["entered_events"].as_array().unwrap().is_empty());receipts.push(row.clone());
 }
 assert_eq!(serde_json::to_value(&w).unwrap(),before);
 std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/type12-869-model-half.json"),serde_json::to_vec_pretty(&serde_json::json!({"actors":receipts,"synthetic_camera_placement":true,"world_unchanged":true,"gui_verified":false})).unwrap()).unwrap();
}
