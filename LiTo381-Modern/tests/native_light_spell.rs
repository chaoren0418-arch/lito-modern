use lito_modern::modern_lito::{native_skills, native_world::World};

fn world() -> World {
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w=World::default();
    for key in ["world","start","status","actor","bot"] {
        let h=fixture["packets"][key].as_str().unwrap();
        let p:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();
        w.accept(&p).unwrap();
    }
    w.learned_skills.insert(2); w.mp=Some([10,10]); w
}

#[test]
fn light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice() {
    use lito_modern::modern_lito::native_skill_selection::{Model,Target};
    let mut w=world(); let before=serde_json::to_value(&w).unwrap();
    assert_eq!(native_skills::request(&w,2,None).unwrap(),[0x14,0,1]);
    assert!(native_skills::request(&w,2,Some(1)).is_err());
    assert_eq!(native_skills::reuse_millis(2),10);
    assert_eq!(native_skills::client_clock_millis(2),10);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    let mut m=Model::default();m.sync(7,&w,true);m.select(Some(2)).unwrap();
    assert!(m.rows().iter().find(|r|r.id==2).unwrap().label.contains("日光術"));
    let target=Target::capture(&w,w.own_actor().unwrap().id);
    let c=m.choice(7,&w,target).unwrap(); assert_eq!((c.skill,c.target),(2,None));
    assert!(m.choice(8,&w,None).is_err());
    w.learned_skills.remove(&2);assert!(native_skills::request(&w,2,None).is_err());
    w.learned_skills.insert(2);w.hp=Some([0,100]);assert!(native_skills::request(&w,2,None).is_err());
}

#[test]
fn formal_change_light_is_atomic_unsigned_actor_scoped_and_does_not_change_motion_owner() {
    let mut w=world();let id=w.own_actor().unwrap().id;
    let original=w.actors[&id].clone(); let mut p=vec![0x1b];p.extend(id.to_le_bytes());p.push(14);
    assert!(w.accept(&p).unwrap());assert_eq!(w.actors[&id].light,14);
    let current=&w.actors[&id];assert_eq!(current.incarnation,original.incarnation);assert_eq!(current.motion_revision,original.motion_revision);assert_eq!(current.target_revision,original.target_revision);
    let before=serde_json::to_value(&w).unwrap();
    for end in 0..p.len(){assert!(w.accept(&p[..end]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
    let mut bad=p.clone();bad.push(0);assert!(w.accept(&bad).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
    p[1..5].copy_from_slice(&0u32.to_le_bytes());assert!(w.accept(&p).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
    p[1..5].copy_from_slice(&999u32.to_le_bytes());let orphan=w.orphan_updates;w.accept(&p).unwrap();assert_eq!(w.orphan_updates,orphan+1);assert!(!w.actors.contains_key(&999));
    p[1..5].copy_from_slice(&id.to_le_bytes());p[5]=255;w.accept(&p).unwrap();assert_eq!(w.actors[&id].light,255);p[5]=0;w.accept(&p).unwrap();assert_eq!(w.actors[&id].light,0);
}

#[test]
fn formal_direct_sound_is_bounded_atomic_and_cleared_with_the_world() {
    let mut w=world();let p=[0x4a,0,145,0];assert!(w.accept(&p).unwrap());
    assert_eq!(serde_json::to_value(&w).unwrap()["wire_sounds"][0]["sound"],145);
    let before=serde_json::to_value(&w).unwrap();
    for end in 0..p.len(){assert!(w.accept(&p[..end]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
    let mut bad=p.to_vec();bad.push(0);assert!(w.accept(&bad).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
    for _ in 0..128{w.accept(&p).unwrap();}
    let v=serde_json::to_value(&w).unwrap();assert_eq!(v["wire_sounds"].as_array().unwrap().len(),128);assert_eq!(v["wire_sounds_dropped"],1);
    w.accept(&[0x28,0x34,2,0]).unwrap();assert!(serde_json::to_value(&w).unwrap()["wire_sounds"].as_array().unwrap().is_empty());
}

#[test]
fn direct_sound_cursor_consumes_once_bounds_bursts_and_does_not_replay_across_worlds() {
    use lito_modern::modern_lito::native_wire_audio::Cursor;
    let mut w=world();let mut cursor=Cursor::default();w.accept(&[0x4a,0,145,0]).unwrap();
    let batch=cursor.take(&w);assert_eq!(batch.events.len(),1);assert_eq!(batch.events[0].sound,Some(145));assert_eq!(batch.events[0].actor,0);
    assert!(cursor.take(&w).events.is_empty());
    w.accept(&[0x4a,1,145,0]).unwrap();let batch=cursor.take(&w);assert!(batch.events.is_empty());assert_eq!(batch.unsupported,1);
    for _ in 0..40 {w.accept(&[0x4a,0,145,0]).unwrap();}
    let batch=cursor.take(&w);assert_eq!(batch.events.len(),32);assert_eq!(batch.dropped,8);assert!(cursor.take(&w).events.is_empty());
    w.accept(&[0x28,0x34,2,0]).unwrap();assert!(cursor.take(&w).events.is_empty());assert!(!w.accept(&[0x4a,0,145,0]).unwrap());w.accept(&[0xd2]).unwrap();w.accept(&[0x4a,0,145,0]).unwrap();assert_eq!(cursor.take(&w).events.len(),1);
}

#[test]
fn original_light_single_bitmap_retires_on_166ms_callback_without_embedded_sound() {
    use lito_modern::modern_lito::{native_skill_effects::Effects,native_world_effects::{Target,Mix},world_map::{ViewProjection,WorldCell,PixelPoint}};
    use std::{path::Path,sync::atomic::AtomicBool};use sha2::{Digest,Sha256};
    let mut w=world();let own=w.own_actor().unwrap();let mut p=vec![0x37];p.extend(own.id.to_le_bytes());p.extend(2510u16.to_le_bytes());let pos=WorldCell{x:i32::from(own.x),y:i32::from(own.y)};w.accept(&p).unwrap();
    let before=serde_json::to_value(&w).unwrap();let cancel=AtomicBool::new(false);let mut effects=Effects::default();effects.prepare(Path::new(r"D:\lineage開發\0-客戶端"),&w,&cancel);
    let view=ViewProjection{anchor_cell:pos,anchor_pixel:PixelPoint{x:400,y:300}};
    for ms in [0,165,166] {
        let mut queue=Vec::new();let (r,e)=effects.draw_to(&w,&mut Target::Fragments(&mut queue),view,ms*1_000_000);
        assert!(r["load_error"].is_null(),"{r}");assert_eq!(r["started"],1);assert_eq!(r["omitted"],0);assert!(e.is_empty());assert_eq!(r["effects"][0]["sprite"],"149-0.spr");
        assert_eq!(r["active"],if ms<166{1}else{0});
        if ms<166 {assert_eq!(queue.len(),1);assert!(matches!(queue[0].mix,Mix::Over));assert_eq!(format!("{:X}",Sha256::digest(&queue[0].frame.rgba)),"6DB65FD59FD356F6729140571B5BCD6BB3B83492A16E1BF0A3884442FC3C8A0E");assert_eq!(r["effects"][0]["next_callback_ms"],166);}
        else {assert!(queue.is_empty());assert_eq!(r["effects"][0]["callback_count"],2);}
    }
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
}

#[test]
fn scene_prepares_the_original_direct_145_WAV_once_without_inventing_light_duration() {
    use lito_modern::modern_lito::native_world_scene::Model;
    use std::{path::Path,sync::atomic::AtomicBool};use sha2::{Digest,Sha256};
    let mut w=world();let own=w.own_actor().unwrap().id;let mut light=vec![0x1b];light.extend(own.to_le_bytes());light.push(14);w.accept(&light).unwrap();w.accept(&[0x4a,0,145,0]).unwrap();
    let root=Path::new(r"D:\lineage開發\0-客戶端");let wav=std::fs::read(root.join("Sound/145.wav")).unwrap();let expected=format!("{:x}",Sha256::digest(&wav));
    let cancel=AtomicBool::new(false);let mut model=Model::load(root,&w,&cancel).unwrap();let before=serde_json::to_value(&w).unwrap();
    for ms in [0,166,1000] {
        let f=model.draw_at(&w,&cancel,ms*1_000_000).unwrap();let sound:Vec<_>=f.receipt["audio"].as_array().unwrap().iter().filter(|r|r["event"]["event"]=="WireSound(145)").collect();
        assert_eq!(sound.len(),if ms==0{1}else{0});
        if ms==0 {assert_eq!(sound[0]["prepared"],true);assert_eq!(sound[0]["sha256"],expected);assert_eq!(sound[0]["bytes"],wav.len());}
        assert_eq!(f.receipt["actor_light_ranges"][own.to_string()],14);assert_eq!(serde_json::to_value(&w).unwrap(),before);
    }
}

#[test]
fn original_light_definition_and_149_bitmap_are_inspected_without_alias_guessing() {
    use lito_modern::modern_lito::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{ActionLimits,ActionKey,SpriteActionCatalog},sprite};
    use sha2::{Digest,Sha256};use std::path::Path;
    let root=Path::new(r"D:\lineage開發\0-客戶端");
    let bytes=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let c=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits::default()).unwrap();let row=&c.sprites[&2510];
    let at=bytes.windows(6).position(|s|s==b"#2510 ").unwrap();let end=bytes[at+1..].windows(2).position(|s|s==b"\n#").map(|n|at+1+n).unwrap_or(bytes.len());let original=std::str::from_utf8(&bytes[at..end]).unwrap();
    let spr=SharedArchive::open(root,SharedArchiveKind::SpriteShard(14)).unwrap().read_entry("149-0.spr").unwrap();assert_eq!(spr.len(),23);assert_eq!(format!("{:X}",Sha256::digest(&spr)),"5E675A5A04735B7412096314303B8E2AF92A1CE7C7CAFB38FF7E289FA5531DC7");let image=sprite::decode_classic_spr_with_limits(&spr,Default::default()).unwrap();
    let output=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-light-selected-source.json");
    let a=c.resolve_action(2510,ActionKey::new(0)).unwrap();
    std::fs::write(output,serde_json::to_vec_pretty(&serde_json::json!({"legacy_definition":original,"row_debug":format!("{row:#?}"),"resolved_definition":a.definition_sprite_id,"SPR_bytes":spr.len(),"SPR_sha256":format!("{:X}",Sha256::digest(&spr)),"frames":image.frames.iter().map(|f|serde_json::json!({"x":f.x_offset,"y":f.y_offset,"width":f.width,"height":f.height,"rgba_sha256":format!("{:X}",Sha256::digest(&f.rgba))})).collect::<Vec<_>>()})).unwrap()).unwrap();
    assert_eq!(row.graphics_id,149);assert_eq!(a.animation.frames.len(),1);
}
