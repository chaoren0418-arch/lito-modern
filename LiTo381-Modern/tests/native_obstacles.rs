use lito_modern::modern_lito::{native_obstacles as obstacles,native_world::World};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn rows()->Vec<serde_json::Value>{serde_json::from_slice::<serde_json::Value>(include_bytes!("fixtures/native-guardian-current.json")).unwrap()["states"].as_array().unwrap().clone()}
fn world()->World {let mut w=World::default();w.accept(&[0x28,0xfe,1,0]).unwrap();w.accept(&[0xd2]).unwrap();w}
#[test]
fn formal_guardian_occupancy_includes_one_hp_and_excludes_destroyed_or_removed_objects(){
    let mut w=world();
    for row in rows(){
        w.actors.clear();w.accept(&bytes(row["body"].as_str().unwrap())).unwrap();
        let id=row["id"].as_u64().unwrap() as u32;
        assert!(obstacles::guardian(&w.actors[&id]));
        assert_eq!(obstacles::blocks(&w,32765,32810),row["blocks"].as_bool().unwrap());
        assert!(!obstacles::blocks(&w,32766,32810));
        let mut remove=vec![0x15];remove.extend(id.to_le_bytes());w.accept(&remove).unwrap();
        assert!(!obstacles::blocks(&w,32765,32810));
    }
}
#[test]
fn guardian_state_updates_hold_authority_without_turning_players_into_obstacles(){
    let mut w=world();w.accept(&bytes(rows()[0]["body"].as_str().unwrap())).unwrap();
    let id=rows()[0]["id"].as_u64().unwrap() as u32;
    let fixture:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    for name in ["status","actor","hp"] {w.accept(&bytes(fixture["packets"][name].as_str().unwrap())).unwrap();}
    let own=w.status.unwrap().object_id;
    w.actors.get_mut(&own).unwrap().x=32765;w.actors.get_mut(&own).unwrap().y=32809;
    for action in [32,33,34,35,0] {
        let mut b=vec![0x20];b.extend(id.to_le_bytes());b.push(action);w.accept(&b).unwrap();
        assert_eq!(w.actors[&id].walk_action,action);assert_eq!(obstacles::blocks(&w,32765,32810),action!=35);
        assert_eq!(w.move_request(4).is_ok(),action==35);
    }
    let mut ordinary=w.actors.remove(&id).unwrap();ordinary.id=3;w.actors.insert(3,ordinary);
    assert!(!obstacles::blocks(&w,32765,32810));
    let mut other=w.actors.remove(&3).unwrap();other.id=id;other.graphics=61;w.actors.insert(id,other);
    assert!(!obstacles::blocks(&w,32765,32810));
}
#[test]
fn original_guardian_assets_render_healthy_damaged_and_held_destroyed_states(){
    use lito_modern::modern_lito::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{SpriteActionCatalog,ActionLimits},native_world_animation::Animator};
    use sha2::{Digest,Sha256};
    let root=std::path::Path::new(r"D:\lineage開發\0-客戶端");
    let raw=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let catalog=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
    let catalog_sha=format!("{:X}",Sha256::digest(&raw));let mut evidence=Vec::new();
    let mut sprites=SharedArchive::open(root,SharedArchiveKind::Sprite).unwrap();
    for row in rows(){
        let mut w=world();w.accept(&bytes(row["body"].as_str().unwrap())).unwrap();
        let actor=w.actors.values().next().unwrap();let mut animator=Animator::default();
        for time in [0,10_000_000_000,20_000_000_000] {
            let pose=animator.pose(&catalog,actor,time).unwrap();
            assert_eq!(pose.action,actor.walk_action);assert_eq!(pose.phase,"object_action");
            let raw=sprites.read_entry(&format!("{}-{}.spr",pose.graphics,pose.image)).unwrap();
            let decoded=lito_modern::modern_lito::sprite::decode_classic_spr_with_limits(&raw,Default::default()).unwrap();
            assert!((pose.subframe as usize)<decoded.frames.len());
            evidence.push(serde_json::json!({"gfx":actor.graphics,"wire_action":actor.walk_action,"time_ns":time,"pose":pose,"sprite_sha256":format!("{:X}",Sha256::digest(&raw))}));
            if actor.walk_action==0 {assert_eq!(pose.subframe,0);}
            if actor.walk_action==35&&time>0 {assert_eq!(pose.subframe,19);assert!(pose.action_complete);}
        }
    }
    let out=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/guardian-assets.json");
    std::fs::write(out,serde_json::to_vec_pretty(&serde_json::json!({"catalog_sha256":catalog_sha,"rows":evidence,"gui_verified":false})).unwrap()).unwrap();
}

#[test]
fn destroyed_guardian_retires_attacks_spells_and_selection_even_if_repaired_between_polls(){
 use lito_modern::modern_lito::{native_skill_selection::Target,native_combat::{HeldAttack,Feedback},native_skills};
 let mut w=world();let row=&rows()[0];w.accept(&bytes(row["body"].as_str().unwrap())).unwrap();let id=row["id"].as_u64().unwrap() as u32;
 let fixture:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
 for name in ["status","actor","hp"]{w.accept(&bytes(fixture["packets"][name].as_str().unwrap())).unwrap();}
 let own=w.status.unwrap().object_id;w.actors.get_mut(&own).unwrap().x=32765;w.actors.get_mut(&own).unwrap().y=32809;w.mp=Some([30,30]);w.learned_skills.extend([1,4,6]);
 let old=Target::capture(&w,id).unwrap();let mut held=HeldAttack::default();held.press(id,9,&w);let mut delayed=HeldAttack::default();delayed.press(id,9,&w);
 assert!(w.attack_request(id).is_ok());assert!(native_skills::request(&w,6,Some(id)).is_ok());
 for action in [32,33,34]{let mut b=vec![0x20];b.extend(id.to_le_bytes());b.push(action);w.accept(&b).unwrap();w.actors.get_mut(&id).unwrap().hp_ratio=1;assert!(w.attack_request(id).is_ok());assert!(obstacles::blocks(&w,32765,32810));}
 let mut death=vec![0x20];death.extend(id.to_le_bytes());death.push(35);w.accept(&death).unwrap();let retired=w.actors[&id].target_revision;
 assert!(!w.actors[&id].dead,"tower must retain its dedicated object-action35 renderer");assert!(w.attack_request(id).is_err());assert!(native_skills::request(&w,6,Some(id)).is_err());assert!(native_skills::request(&w,1,Some(id)).is_err());assert!(old.resolve(&w).is_none());
 assert!(held.ready(9,&w,Feedback::Idle,true,true).is_none());assert!(!held.active());assert!(!obstacles::blocks(&w,32765,32810));
 w.accept(&death).unwrap();assert_eq!(w.actors[&id].target_revision,retired);
 let mut repair=death;*repair.last_mut().unwrap()=0;w.accept(&repair).unwrap();assert!(w.attack_request(id).is_ok());assert!(old.resolve(&w).is_none());assert!(delayed.ready(9,&w,Feedback::Idle,true,true).is_none());assert!(!delayed.active());assert!(Target::capture(&w,id).unwrap().resolve(&w).is_some());
}

#[test]
fn repeated_guardian_hits_never_queue_character_hurt_or_hide_original_states(){
 use lito_modern::modern_lito::{native_world_animation::Animator,resources::{SharedArchive,SharedArchiveKind},sprite_actions::{SpriteActionCatalog,ActionLimits}};
 let raw=SharedArchive::open(std::path::Path::new(r"D:\lineage開發\0-客戶端"),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
 let catalog=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
 let fixture:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-combat-current.json")).unwrap();
 for row in rows().iter().filter(|r|r["action"]==0){
  let mut w=world();w.accept(&bytes(row["body"].as_str().unwrap())).unwrap();let id=row["id"].as_u64().unwrap() as u32;let mut animator=Animator::default();
  let mut hit=bytes(fixture["packets"]["hit"].as_str().unwrap());hit[6..10].copy_from_slice(&id.to_le_bytes());
  let mut now=0;
  for action in [0,32,33,34,35]{
   let mut state=vec![0x20];state.extend(id.to_le_bytes());state.push(action);w.accept(&state).unwrap();
   let mut sounds=vec![];
   for _ in 0..40 {w.accept(&hit).unwrap();now+=1_000_000_000;let a=&w.actors[&id];assert!(a.actions.is_empty());let pose=animator.pose(&catalog,a,now).unwrap();sounds.extend(pose.entered_events.iter().filter_map(|e|e.sound));assert_eq!(pose.action,action);assert_eq!(pose.phase,"object_action");assert!(!animator.impact(a,now));}
   if action==35 {assert_eq!(sounds,vec![360]);let pose=animator.pose(&catalog,&w.actors[&id],now+10_000_000_000).unwrap();assert_eq!(pose.subframe,19);assert!(pose.entered_events.is_empty());} else {assert!(sounds.is_empty());}
  }
 }
}
