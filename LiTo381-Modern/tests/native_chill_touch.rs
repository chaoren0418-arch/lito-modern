use lito_modern::modern_lito::{native_world::World,native_skills,native_class1::Effects,
    native_combat_animation::EnteredEvent,compositor::Canvas,world_map::{WorldCell,PixelPoint,ViewProjection}};
use std::{path::Path,sync::atomic::AtomicBool};
fn world()->World {
    let d:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=World::default();
    for k in ["world","start","status","actor","bot"] {let s=d["packets"][k].as_str().unwrap();let b:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
    let mut a=w.actors.remove(&0xfedcba98).unwrap();a.id=1;a.x=32729;a.y=32797;w.actors.insert(1,a);w.status.as_mut().unwrap().object_id=1;
    let a=w.actors.get_mut(&2).unwrap();a.x=32730;a.y=32797;w.hp=Some([70,100]);w.mp=Some([24,24]);w.learned_skills.insert(10);w
}
fn attack(heading:u8,marker:u8,target:u32)->Vec<u8> {
    let mut b=vec![0x23,18];b.extend(1u32.to_le_bytes());b.extend(target.to_le_bytes());b.extend([marker,heading]);b.extend(77u32.to_le_bytes());b.extend(252u16.to_le_bytes());b.push(0);
    for v in [32729u16,32797,32730,32797]{b.extend(v.to_le_bytes());}b.extend([0,0]);b
}
fn hit(w:&World)->EnteredEvent {EnteredEvent{actor:1,sequence:w.projectiles.last().unwrap().action_sequence,action:18,frame:1,event:"Hit".into(),sound:None, prepared_audio:None,}}
fn projection()->ViewProjection {ViewProjection{anchor_cell:WorldCell{x:32730,y:32797},anchor_pixel:PixelPoint{x:400,y:300}}}
#[test]
fn request_layout_guards_and_server_authority_do_not_predict_healing_or_hurt() {
    let mut w=world();let before=serde_json::to_value(&w).unwrap();
    assert_eq!(native_skills::request(&w,10,Some(2)).unwrap(),vec![0x14,1,1,2,0,0,0,0xda,0x7f,0x1d,0x80]);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    assert!(native_skills::request(&w,10,None).is_err()&&native_skills::request(&w,10,Some(1)).is_err());
    w.actors.get_mut(&2).unwrap().x=32733;assert!(native_skills::request(&w,10,Some(2)).is_err());w.actors.get_mut(&2).unwrap().x=32730;
    w.accept(&attack(2,10,2)).unwrap();assert!(w.actors[&2].actions.is_empty());assert_eq!(w.hp,Some([70,100]));assert_eq!(w.mp,Some([24,24]));
    w.accept(&[0x0d,85,0,100,0]).unwrap();assert_eq!(w.hp,Some([85,100]));
}
#[test]
fn all_eight_banks_frozen_heading_offset_once_sound_and_effect_hit() {
    let cancel=AtomicBool::new(false);
    for heading in 0..8 {
        let mut w=world();w.accept(&attack(heading,10,2)).unwrap();let event=hit(&w);
        w.actors.get_mut(&1).unwrap().heading=(heading+3)%8;
        w.actors.get_mut(&2).unwrap().x+=4;
        let before=serde_json::to_value(&w).unwrap();let mut fx=Effects::default();fx.prepare(Path::new(r"D:\lineage開發\0-客戶端"),&w,&cancel);
        assert!(fx.update(&w,std::iter::once(&event),0).is_empty());let mut canvas=Canvas::new(800,600).unwrap();
        let (r,e)=fx.draw(&mut canvas,projection(),0);assert!(r["load_error"].is_null());assert_eq!(r["omitted"],0);assert_eq!(e[0].sound,Some(110));
        assert_eq!(r["effects"][0]["sprite"],format!("252-{heading}.spr"));
        assert_eq!(r["effects"][0]["position"],serde_json::to_value(lito_modern::modern_lito::native_class1::position([32730,32797],heading).map(|p|[p.x,p.y]).unwrap()).unwrap());
        // Dispatch the original finite callbacks; a late callback starts a fresh dwell.
        for ms in [41,124,249] { assert!(fx.update(&w,std::iter::empty(),ms*1_000_000).is_empty()); }
        assert!(fx.update(&w,std::iter::once(&event),400_000_000).is_empty());
        let hits=fx.update(&w,std::iter::empty(),415_000_000);assert_eq!(hits.len(),1);
        let (r,e)=fx.draw(&mut canvas,projection(),415_000_000);assert!(e.is_empty());assert_eq!(r["non_audio_cues"][0]["source_frame"],5);
        assert!(fx.update(&w,std::iter::empty(),456_000_000).is_empty());let (r,_)=fx.draw(&mut canvas,projection(),456_000_000);assert_eq!(r["effects"][0]["subframe"],5);assert_eq!(r["active"],1);
        fx.update(&w,std::iter::empty(),581_000_000);assert_eq!(fx.draw(&mut canvas,projection(),581_000_000).0["active"],0);
        assert_eq!(serde_json::to_value(&w).unwrap(),before);
    }
}
#[test]
fn miss_replacement_and_map_change_cannot_hurt_a_new_target() {
    for variant in 0..3 {
        let mut w=world();w.accept(&attack(0,if variant==0{0}else{10},if variant==0{0}else{2})).unwrap();let event=hit(&w);
        let mut fx=Effects::default();fx.prepare(Path::new(r"D:\lineage開發\0-客戶端"),&w,&AtomicBool::new(false));fx.update(&w,std::iter::once(&event),0);
        if variant==1 {w.actors.get_mut(&2).unwrap().incarnation+=1;}
        if variant==2 {w.accept(&[0x28,0x34,2,0]).unwrap();}
        for ms in [41,124,249,415,456,581,1000] { assert!(fx.update(&w,std::iter::empty(),ms*1_000_000).is_empty()); }
        let mut canvas=Canvas::new(800,600).unwrap();let (r,e)=fx.draw(&mut canvas,projection(),1_000_000_000);assert!(e.is_empty());assert_eq!(r["active"],0);
    }
}

#[test]
fn common_finite_clock_quantizes_birth_and_dispatches_late_hit_once() {
    let mut w = world();
    w.accept(&attack(0, 10, 2)).unwrap();
    let event = hit(&w);
    let before = serde_json::to_value(&w).unwrap();
    let mut fx = Effects::default();
    fx.prepare(Path::new(r"D:\lineage開發\0-客戶端"), &w, &AtomicBool::new(false));
    assert!(fx.update(&w, std::iter::once(&event), 900_000).is_empty());
    let mut canvas = Canvas::new(800, 600).unwrap();
    let (first, sound) = fx.draw(&mut canvas, projection(), 900_000);
    assert_eq!(first["effects"][0]["source_frame"], 1);
    assert_eq!(sound.iter().filter_map(|e| e.sound).collect::<Vec<_>>(), [110]);
    let mut samples = vec![first];
    for (now, source, impacts, callbacks, due) in [
        (41_000_000, 2, 0, 2, Some(124)),
        (400_000_000, 3, 0, 3, Some(525)),
        (525_000_000, 4, 0, 4, Some(691)),
        (691_000_000, 5, 1, 5, Some(732)),
        (732_000_000, 0, 0, 6, Some(857)),
        (857_000_000, 0, 0, 7, None),
    ] {
        let hit = fx.update(&w, std::iter::empty(), now);
        assert_eq!(hit.len(), impacts);
        if impacts == 1 { assert_eq!(hit[0].target, 2); }
        let (r, sound) = fx.draw(&mut canvas, projection(), now);
        assert_eq!(r["effects"][0]["source_frame"], source, "native common clock at {now}ns");
        assert_eq!(r["effects"][0]["native_clock_elapsed_ms"], now / 1_000_000);
        assert_eq!(r["effects"][0]["callback_count"], callbacks);
        assert_eq!(r["effects"][0]["next_callback_ms"], serde_json::json!(due));
        assert!(sound.is_empty());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
        samples.push(r);
    }
    assert_eq!(samples.last().unwrap()["active"], 0);
    assert_eq!(samples.last().unwrap()["hits"], 1);
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-common-finite-class1-clock.json"), serde_json::to_vec_pretty(&samples).unwrap()).unwrap();
}
