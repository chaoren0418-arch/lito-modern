use lito_modern::modern_lito::{native_world::World,native_skills,native_attack_many::{self,Effects},native_combat_animation::EnteredEvent,
    native_world_effects::Target,native_actor_layers::{Bank,Source},resources::{SharedArchive,SharedArchiveKind},sprite,
    native_world_layers,world_map::{WorldCell,PixelPoint,ViewProjection}};
use std::{path::Path,sync::atomic::AtomicBool};
const ROOT:&str=r"D:\lineage開發\0-客戶端";
fn world()->World {
    let d:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=World::default();
    for k in ["world","start","status","actor","bot"] {let s=d["packets"][k].as_str().unwrap();let b:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
    let mut a=w.actors.remove(&0xfedcba98).unwrap();a.id=1;a.x=32729;a.y=32797;w.actors.insert(1,a);w.status.as_mut().unwrap().object_id=1;
    let a=w.actors.get_mut(&2).unwrap();a.x=32730;a.y=32797;w.hp=Some([70,100]);w.mp=Some([24,24]);w.learned_skills.insert(17);w
}

fn attack(heading:u8,targets:&[(u32,u8)])->Vec<u8> {
    let mut b=vec![0x39,18];b.extend(1u32.to_le_bytes());b.extend(32729u16.to_le_bytes());b.extend(32797u16.to_le_bytes());b.push(heading);
    b.extend(77u32.to_le_bytes());b.extend(170u16.to_le_bytes());b.extend([8,0,0]);b.extend((targets.len() as u16).to_le_bytes());
    for (id,hit) in targets {b.extend(id.to_le_bytes());b.push(*hit);}b
}
fn event(w:&World)->EnteredEvent {EnteredEvent{actor:1,sequence:w.many_attacks.last().unwrap().action_sequence,action:18,frame:1,event:"Hit".into(),sound:None, prepared_audio:None,}}
#[test]
fn many_packet_is_atomic_bounded_and_preserves_each_unsigned_target_marker() {
    let mut w=world();let before=serde_json::to_value(&w).unwrap();let b=attack(2,&[(2,0x20),(0xfedcba98,0)]);
    for n in 0..b.len(){assert!(w.accept(&b[..n]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
    for at in [10,18,26] {let mut bad=b.clone();bad[at]=9;assert!(w.accept(&bad).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
    let decoded=native_attack_many::decode(&b).unwrap().unwrap();assert_eq!(decoded.targets[1].id,0xfedcba98);assert!(!decoded.targets[1].hit);
    w.accept(&b).unwrap();assert_eq!(w.own_cast_packets,1);assert!(w.actors[&2].actions.is_empty());assert_eq!(w.hp,Some([70,100]));assert_eq!(w.mp,Some([24,24]));
    assert!(native_attack_many::decode(&attack(0,&[])).unwrap().is_some());
    let mut large=attack(0,&vec![(2,0);129]);assert!(native_attack_many::decode(&large).is_err());large=attack(0,&[(2,0)]);large[15]=171;assert!(native_attack_many::decode(&large).unwrap().is_none());
}
#[test]
fn request_clock_reuse_range_and_target_guard_preserve_authority() {
    let mut w=world();let before=serde_json::to_value(&w).unwrap();assert_eq!(native_skills::request(&w,17,Some(2)).unwrap(),vec![0x14,2,0,2,0,0,0,0xda,0x7f,0x1d,0x80]);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(native_skills::client_clock_millis(17),100);assert_eq!(native_skills::reuse_millis(17),500);
    w.actors.get_mut(&2).unwrap().x=32736;assert!(native_skills::request(&w,17,Some(2)).is_err());assert!(native_skills::request(&w,17,Some(1)).is_err());
}
fn original(s:&Source)->Vec<u8> {let kind=if s.archive=="Sprite"{SharedArchiveKind::Sprite}else{SharedArchiveKind::SpriteShard(s.archive.strip_prefix("Sprite").unwrap().parse().unwrap())};SharedArchive::open(Path::new(ROOT),kind).unwrap().read_entry(&s.member).unwrap()}
#[path="support/native_fragment_reference.rs"] mod reference;
#[test]
fn all_eight_lightning_banks_match_independent_pixels_origin_and_multi_target_hit_once() {
    let cancel=AtomicBool::new(false);let model=reference::full_native_fixture(Path::new(ROOT),u32::from(world().map_id.unwrap()));
    let mut bank=Bank::open(Path::new(ROOT)).unwrap();let mut sources=Vec::new();
    for heading in 0..8 {
        let name=format!("170-{heading}.spr");bank.ensure(&name,&cancel).unwrap();let source=bank.get(&name).unwrap().source.clone();let raw=sprite::decode_classic_spr_layered(&original(&source),Default::default()).unwrap();sources.push(source);
        let mut w=world();let mut other=w.actors[&2].clone();other.id=3;other.incarnation+=1;w.actors.insert(3,other);w.accept(&attack(heading,&[(2,0x20),(3,0x20),(99,0)])).unwrap();let e=event(&w);
        w.actors.get_mut(&1).unwrap().x+=5;w.actors.get_mut(&1).unwrap().heading=(heading+3)%8;w.actors.get_mut(&2).unwrap().y+=3;
        let before=serde_json::to_value(&w).unwrap();let mut fx=Effects::default();fx.prepare(Path::new(ROOT),&w,&cancel);
        let view=ViewProjection{anchor_cell:WorldCell{x:32729,y:32797},anchor_pixel:PixelPoint{x:160,y:120}};
        let starts=[0u64,1,3,5,7,9,11,13,15,17];let subs=[0usize,1,2,3,4,5,2,4,5,3];let mut hits=0;let mut sounds=0;
        for (n,tick) in starts.into_iter().enumerate() {
            let ns=(tick*1_000_000_000).div_ceil(24);let impacts=fx.update(&w,std::iter::once(&e),ns);hits+=impacts.len();
            assert!(impacts.iter().all(|i|i.target==2||i.target==3));let mut queue=Vec::new();let (r,events)=fx.draw_to(&mut Target::Fragments(&mut queue),view,ns);
            assert!(r["load_error"].is_null());assert_eq!(r["omitted"],0);assert_eq!(queue.len(),1);assert_eq!(r["effects"][0]["sprite"],name);assert_eq!(r["effects"][0]["subframe"],subs[n]);assert_eq!(queue[0].position,view.anchor_cell);
            let actual=native_world_layers::compose_with_effects(model.scene(),view,320,240,&[],&Default::default(),&queue,&cancel).unwrap();
            let expected=reference::reference_layers(model.scene(),view,&[(0,view.anchor_pixel,&raw,subs[n],2)],320,240,true);assert_eq!(actual.canvas.rgba(),expected.0);assert!(actual.visibility.is_empty());
            for event in events {assert_eq!(event.sound,Some(101));sounds+=1;}
        }
        assert_eq!((hits,sounds),(2,1));fx.update(&w,std::iter::once(&e),791_666_667);let mut queue=Vec::new();assert_eq!(fx.draw_to(&mut Target::Fragments(&mut queue),view,791_666_667).0["active"],0);assert!(queue.is_empty());assert_eq!(serde_json::to_value(&w).unwrap(),before);
    }
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/lightning-sprite-sources.json"),serde_json::to_vec_pretty(&sources).unwrap()).unwrap();
}
#[test]
fn many_hits_reject_miss_replacement_removal_death_and_new_world() {
    for variant in 0..5 {
        let mut w=world();w.accept(&attack(0,&[(2,if variant==0{0}else{0x20})])).unwrap();let e=event(&w);let mut fx=Effects::default();fx.prepare(Path::new(ROOT),&w,&AtomicBool::new(false));fx.update(&w,std::iter::once(&e),0);
        match variant {1=>w.actors.get_mut(&2).unwrap().incarnation+=1,2=>{w.actors.remove(&2);},3=>w.actors.get_mut(&2).unwrap().dead=true,4=>{w.accept(&[0x28,0x34,2,0]).unwrap();},_=>{}}
        assert!(fx.update(&w,std::iter::empty(),41_666_667).is_empty());
    }
}

#[test]
fn many_common_clock_quantizes_birth_and_late_dispatch_without_replaying_hits() {
    let mut w=world();
    w.accept(&attack(0,&[(2,0x20),(99,0)])).unwrap();
    let e=event(&w);let before=serde_json::to_value(&w).unwrap();
    let mut fx=Effects::default();fx.prepare(Path::new(ROOT),&w,&AtomicBool::new(false));
    assert!(fx.update(&w,std::iter::once(&e),900_000).is_empty());
    let view=ViewProjection{anchor_cell:WorldCell{x:32729,y:32797},anchor_pixel:PixelPoint{x:160,y:120}};
    let mut queue=Vec::new();let (r,sounds)=fx.draw_to(&mut Target::Fragments(&mut queue),view,900_000);
    assert_eq!(r["effects"][0]["source_frame"],1);
    assert_eq!(sounds.iter().filter_map(|e|e.sound).collect::<Vec<_>>(),[101]);
    let mut samples=vec![r];let mut total_hits=0;
    for (ms,source,callbacks,due) in [(41,2,2,Some(124)),(1000,3,3,Some(1083)),
        (1083,4,4,Some(1166)),(1166,5,5,Some(1249)),(1249,6,6,Some(1332)),
        (1332,7,7,Some(1415)),(1415,8,8,Some(1498)),(1498,9,9,Some(1581)),
        (1581,0,10,Some(1664)),(1664,0,11,None)] {
        let impacts=fx.update(&w,std::iter::once(&e),ms*1_000_000);
        assert_eq!(impacts.len(),usize::from(ms==41));
        assert!(impacts.iter().all(|i|i.target==2));total_hits+=impacts.len();
        let mut queue=Vec::new();let (r,sounds)=fx.draw_to(&mut Target::Fragments(&mut queue),view,ms*1_000_000);
        let frame=&r["effects"][0];assert_eq!(frame["source_frame"],source);
        assert_eq!(frame["native_clock_elapsed_ms"],ms);
        assert_eq!(frame["callback_count"],callbacks);assert_eq!(frame["next_callback_ms"],serde_json::json!(due));
        assert!(sounds.is_empty());assert_eq!(queue.len(),usize::from(ms!=1664));
        assert_eq!(serde_json::to_value(&w).unwrap(),before);samples.push(r);
    }
    assert_eq!(total_hits,1);assert_eq!(samples.last().unwrap()["active"],0);
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-common-finite-many-clock.json"),serde_json::to_vec_pretty(&samples).unwrap()).unwrap();
}
