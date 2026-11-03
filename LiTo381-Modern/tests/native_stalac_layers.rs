use lito_modern::modern_lito::{native_actor_layers::{Bank,Plan,Mix},
    resources::{SharedArchive,SharedArchiveKind},sprite_actions::{SpriteActionCatalog,ActionLimits,ActionKey}};
use std::{path::Path,sync::atomic::AtomicBool};
const ROOT:&str=r"D:\lineage開發\0-客戶端";
#[test]
fn original_stalac_body_shadow_and_smoke_share_every_subframe() {
    let raw=SharedArchive::open(Path::new(ROOT),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let catalog=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
    let a=catalog.resolve_action(1801,ActionKey::new(0)).unwrap().animation;
    let plan=Plan::from_catalog(&catalog,1801).unwrap();
    assert_eq!(plan.shadow.as_ref().unwrap().graphics,1802);
    assert_eq!(plan.attachments[0].graphics,1803);assert_eq!(plan.attachments[0].mix,Mix::Screen);
    let cancel=AtomicBool::new(false);let mut bank=Bank::open(Path::new(ROOT)).unwrap();bank.ensure("1801-0.spr",&cancel).unwrap();
    let mut samples=Vec::new();
    for f in &a.frames {
        let p=bank.prepare(&plan.bind(f.image_id).unwrap(),f.subframe as usize,&cancel).unwrap();
        assert!(p.missing_shadow.is_none()&&p.missing_attachments.is_empty());
        let mut counts=Vec::new();
        for name in ["1801-0.spr","1803-0.spr","1802-0.spr"] {
            let frame=bank.frame(name,f.subframe as usize).unwrap();assert!(frame.fragments.is_some());
            counts.push(serde_json::json!({"name":name,"subframe":f.subframe,"size":[frame.width,frame.height],"source":bank.get(name).unwrap().source}));
        }
        samples.push(counts);
    }
    std::fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/stalac-layer-preflight.json"),serde_json::to_vec_pretty(&samples).unwrap()).unwrap();
}

fn world()->World {
    let d:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=World::default();
    for k in ["world","start","status","actor","bot"] {let s=d["packets"][k].as_str().unwrap();let b:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
    let mut a=w.actors.remove(&0xfedcba98).unwrap();a.id=1;a.x=32729;a.y=32797;w.actors.insert(1,a);w.status.as_mut().unwrap().object_id=1;
    let a=w.actors.get_mut(&2).unwrap();a.x=32730;a.y=32797;w.hp=Some([70,100]);w.mp=Some([24,24]);w.learned_skills.insert(16);w
}
fn attack(heading:u8,marker:u8,target:u32)->Vec<u8> {
    let mut b=vec![0x23,18];b.extend(1u32.to_le_bytes());b.extend(target.to_le_bytes());b.extend([marker,heading]);b.extend(77u32.to_le_bytes());b.extend(1801u16.to_le_bytes());b.push(0);
    for v in [32729u16,32797,32730,32797]{b.extend(v.to_le_bytes());}b.extend([0,0]);b
}
fn hit(w:&World)->EnteredEvent {EnteredEvent{actor:1,sequence:w.projectiles.last().unwrap().action_sequence,action:18,frame:1,event:"Hit".into(),sound:None, prepared_audio:None,}}
fn projection()->ViewProjection {ViewProjection{anchor_cell:WorldCell{x:32730,y:32797},anchor_pixel:PixelPoint{x:400,y:300}}}

#[path = "support/native_fragment_reference.rs"]
mod reference;
use lito_modern::modern_lito::{native_world::World,native_skills,native_class1::Effects,
    native_combat_animation::EnteredEvent,world_map::{WorldCell,PixelPoint,ViewProjection},sprite,
    native_world_effects::{Target,Mix as EffectMix},native_world_layers};
fn original(source:&lito_modern::modern_lito::native_actor_layers::Source)->Vec<u8> {
    let kind=if source.archive=="Sprite"{SharedArchiveKind::Sprite}else{SharedArchiveKind::SpriteShard(source.archive.strip_prefix("Sprite").unwrap().parse().unwrap())};
    SharedArchive::open(Path::new(ROOT),kind).unwrap().read_entry(&source.member).unwrap()
}
#[test]
fn stalac_packet_range_clock_and_no_predicted_authority() {
    let mut w=world();let before=serde_json::to_value(&w).unwrap();
    assert_eq!(native_skills::request(&w,16,Some(2)).unwrap(),vec![0x14,1,7,2,0,0,0,0xda,0x7f,0x1d,0x80]);
    assert_eq!(native_skills::client_clock_millis(16),10);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    w.actors.get_mut(&2).unwrap().x=32740;assert!(native_skills::request(&w,16,Some(2)).is_err());
    w.actors.get_mut(&2).unwrap().x=32730;w.accept(&attack(2,10,2)).unwrap();
    assert!(w.actors[&2].actions.is_empty());assert_eq!(w.hp,Some([70,100]));assert_eq!(w.mp,Some([24,24]));
}
#[test]
fn all_stalac_frames_match_independent_fragment_compositor_and_cues_once() {
    let cancel=AtomicBool::new(false);let mut w=world();w.accept(&attack(0,10,2)).unwrap();let event=hit(&w);
    let before=serde_json::to_value(&w).unwrap();
    let model=reference::full_native_fixture(Path::new(ROOT),u32::from(w.map_id.unwrap()));
    let view=projection();let mut fx=Effects::default();fx.prepare(Path::new(ROOT),&w,&cancel);
    let mut bank=Bank::open(Path::new(ROOT)).unwrap();let mut raw=Vec::new();
    for name in ["1801-0.spr","1803-0.spr","1802-0.spr"] {
        bank.ensure(name,&cancel).unwrap();raw.push(sprite::decode_classic_spr_layered(&original(&bank.get(name).unwrap().source),Default::default()).unwrap());
    }
    let times=[0,166_666_667,291_666_667,416_666_667,583_333_334,791_666_667,1_000_000_000,1_208_333_334,1_416_666_667];
    let mut sound=0;let mut hits=0;
    for (index,now) in times.into_iter().enumerate() {
        let impacts=fx.update(&w,std::iter::once(&event),now);hits+=impacts.len();
        let mut queue=Vec::new();let (receipt,events)=fx.draw_to(&mut Target::Fragments(&mut queue),view,now);
        assert!(receipt["load_error"].is_null());assert_eq!(receipt["omitted"],0);assert_eq!(queue.len(),3);
        assert!(matches!(queue[0].mix,EffectMix::Over)&&matches!(queue[1].mix,EffectMix::Screen)&&matches!(queue[2].mix,EffectMix::Shadow));
        let visual=&receipt["effects"][0];assert_eq!(visual["source_frame"],if index==8{0}else{index+1});
        assert_eq!(visual["subframe"],index);assert_eq!(visual["layers"][0]["subframe"],index);assert_eq!(visual["layers"][1]["subframe"],index);
        assert!(queue.iter().all(|v|v.position==queue[0].position));
        let actual=native_world_layers::compose_with_effects(model.scene(),view,800,600,&[],&Default::default(),&queue,&cancel).unwrap();
        let center=view.cell_to_pixel(queue[0].position).unwrap();
        let refs=vec![(0,center,&raw[0],index,0),(0,center,&raw[1],index,2),(0,center,&raw[2],index,1)];
        let expected=reference::reference_layers(model.scene(),view,&refs,800,600,true);
        assert_eq!(actual.canvas.rgba(),expected.0,"stalac subframe {index}");assert!(actual.visibility.is_empty());
        assert!(expected.1.iter().all(|id|*id==0));
        for e in events {assert_eq!(e.sound,Some(712));sound+=1;}
    }
    assert_eq!((sound,hits),(1,1));
    assert!(fx.update(&w,std::iter::once(&event),1_625_000_000).is_empty());let mut queue=Vec::new();
    let (r,e)=fx.draw_to(&mut Target::Fragments(&mut queue),view,1_625_000_000);assert_eq!(r["active"],0);assert!(e.is_empty()&&queue.is_empty());
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
}

#[test]
fn layered_submission_budget_rejects_the_whole_group() {
    let cancel=AtomicBool::new(false);let mut bank=Bank::open(Path::new(ROOT)).unwrap();bank.ensure("1801-0.spr",&cancel).unwrap();
    let f=bank.frame("1801-0.spr",0).unwrap();let pos=projection().anchor_cell;
    let mut queue:Vec<_>=(0..510).map(|_|lito_modern::modern_lito::native_world_effects::Effect{frame:f,position:pos,mix:EffectMix::Over}).collect();
    let batch=[(f,EffectMix::Over),(f,EffectMix::Screen),(f,EffectMix::Shadow)];
    assert!(Target::Fragments(&mut queue).submit_batch(&batch,pos,projection()).is_err());assert_eq!(queue.len(),510);
    queue.pop();Target::Fragments(&mut queue).submit_batch(&batch,pos,projection()).unwrap();assert_eq!(queue.len(),512);
}
#[test]
fn stalac_miss_replaced_target_and_map_retire_without_hurt_authority() {
    for variant in 0..3 {
        let mut w=world();w.accept(&attack(0,if variant==0{0}else{10},2)).unwrap();let event=hit(&w);
        let mut fx=Effects::default();fx.prepare(Path::new(ROOT),&w,&AtomicBool::new(false));fx.update(&w,std::iter::once(&event),0);
        if variant==1 {w.actors.get_mut(&2).unwrap().incarnation+=1;}
        if variant==2 {w.accept(&[0x28,0x34,2,0]).unwrap();}
        assert!(fx.update(&w,std::iter::empty(),416_666_667).is_empty());
        let mut queue=Vec::new();let (r,_)=fx.draw_to(&mut Target::Fragments(&mut queue),projection(),416_666_667);
        assert_eq!(r["active"],if variant==2{0}else{1});assert_eq!(queue.len(),if variant==2{0}else{3});
    }
}
