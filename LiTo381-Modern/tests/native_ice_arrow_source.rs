use lito_modern::modern_lito::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{ActionKey,ActionLimits,SpriteActionCatalog}};
use std::path::Path;
use sha2::{Sha256,Digest};
#[test]
fn inspect_original_ice_arrow_chain(){
 let mut archive=SharedArchive::open(Path::new(r"D:\lineage開發\0-客戶端"),SharedArchiveKind::Text).unwrap();assert!(archive.entry("list.spr").unwrap().size<=8*1024*1024);
 let bytes=archive.read_entry("list.spr").unwrap();let digest=Sha256::digest(&bytes).iter().map(|b|format!("{b:02x}")).collect::<String>();let catalog=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits{max_bytes:8*1024*1024,..Default::default()}).unwrap();
 let mut rows=vec![];
 for gfx in [1797,1798]{
  let source=&catalog.sprites[&gfx];let action=catalog.resolve_action(gfx,ActionKey::new(0)).unwrap();
  assert!(source.header_issue.is_none());assert!(!action.animation.has_uninterpreted_events());
  rows.push(serde_json::json!({"gfx":gfx,"definition_gfx":action.definition_sprite_id,"image_count":source.image_count,"direction":format!("{:?}",action.animation.directionality),"fps":action.animation.frames_per_second,"attributes":source.attributes.iter().map(|a|serde_json::json!({"key":a.key.id,"body":String::from_utf8_lossy(&a.body_bytes)})).collect::<Vec<_>>(),"frames":action.animation.frames.iter().map(|f|serde_json::json!({"image":f.image_id,"subframe":f.subframe,"duration_ticks":f.duration_ticks,"events":f.events.iter().map(|e|format!("{e:?}")).collect::<Vec<_>>()})).collect::<Vec<_>>() }));
 }
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/ice-arrow-draft");std::fs::create_dir_all(&root).unwrap();
 std::fs::write(root.join("resource-chain.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"original_action_metadata_inspected_only","list_spr_sha256":digest,"rows":rows,"full_skill_verified":false})).unwrap()).unwrap();
}
