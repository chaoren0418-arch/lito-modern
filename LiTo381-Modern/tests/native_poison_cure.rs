use lito_modern::modern_lito::{
    compositor::Canvas, native_skill_effects::{Effects, qualify_skills},
    native_skills::{self, Feedback, Pending}, native_world::World,
    resources::{SharedArchive, SharedArchiveKind},
    sprite_actions::{ActionKey, ActionLimits, FrameEvent, SpriteActionCatalog},
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::{path::Path, sync::atomic::AtomicBool, time::Instant};
const ROOT: &str = r"D:\lineage開發\0-客戶端";
#[test]
fn verified_numeric_comma_does_not_relax_other_delimiters() {
    let comma=SpriteActionCatalog::parse_for_inspection(b"#1 1\n0.anim(0 2,0,0:1 0.1:2) 104.attr(8)\n",ActionLimits::default()).unwrap();
    let dot=SpriteActionCatalog::parse_for_inspection(b"#1 1\n0.anim(0 2,0.0:1 0.1:2) 104.attr(8)\n",ActionLimits::default()).unwrap();
    assert_eq!(comma.resolve_action(1,ActionKey::new(0)).unwrap().animation,dot.resolve_action(1,ActionKey::new(0)).unwrap().animation);
    for raw in [b"#1 1\n0,anim(0 2,0.0:1 0.1:2)\n".as_slice(),b"#1 1\n0.anim(0 2,0!0:1 0.1:2)\n"] {
        let c=SpriteActionCatalog::parse_for_inspection(raw,ActionLimits::default()).unwrap();
        assert!(c.resolve_action(1,ActionKey::new(0)).is_err());
    }
}
fn world() -> World {
    let data:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w=World::default();
    for key in ["world","start","status","actor","bot"] {
        let s=data["packets"][key].as_str().unwrap();
        let b:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
        w.accept(&b).unwrap();
    }
    w.learned_skills.extend([9,11]);w
}
#[test]
fn object_and_coordinate_envelopes_preserve_authority_and_gate_live_targets() {
    let mut w=world();let own=w.status.as_ref().unwrap().object_id;
    let before=serde_json::to_value(&w).unwrap();
    let mut self_cure=vec![0x14,1,0];self_cure.extend(own.to_le_bytes());
    assert_eq!(native_skills::request(&w,9,None).unwrap(),self_cure);
    assert_eq!(native_skills::request(&w,9,Some(2)).unwrap(),vec![0x14,1,0,2,0,0,0]);
    assert_eq!(native_skills::request(&w,11,Some(2)).unwrap(),vec![0x14,1,2,2,0,0,0,0xd1,0x7f,0x0d,0x80]);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    assert!(native_skills::request(&w,11,None).is_err());
    assert!(native_skills::request(&w,11,Some(own)).is_err());
    w.actors.get_mut(&2).unwrap().x+=1;
    assert!(native_skills::request(&w,11,Some(2)).is_err());
    w.learned_skills.remove(&9);assert!(native_skills::request(&w,9,None).is_err());
    assert_eq!(native_skills::reuse_millis(11),500);assert_eq!(native_skills::reuse_millis(9),10);
}
#[test]
fn reagent_and_probability_refusals_do_not_invent_a_poison_or_expire_one_locally() {
    let mut w=world();let now=Instant::now();
    let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
    w.accept(&[0x10,0x2b,1,0]).unwrap();
    assert_eq!(p.observe(&w,now),Feedback::Rejected(299));assert!(w.effects.is_empty());
    let p=Pending{messages:w.message_count,..p};
    let id=w.own_actor().unwrap().id;let mut action=vec![0x20];action.extend(id.to_le_bytes());action.push(19);w.accept(&action).unwrap();
    w.accept(&[0x10,0x18,1,0]).unwrap();
    assert_eq!(p.observe(&w,now),Feedback::Rejected(280));
    w.accept(&[0x32,2,0,0,0,1,0]).unwrap();assert!(w.actors[&2].draw_state.green);
    assert_eq!(w.hp,Some([100,200]));
    w.accept(&[0x32,2,0,0,0,0,0]).unwrap();assert!(!w.actors[&2].draw_state.green);
    assert_eq!(w.hp,Some([100,200]));
}
#[test]
fn cure_uses_its_own_timing_and_sound_with_direct_192_graphics_bank() {
    let raw=SharedArchive::open(Path::new(ROOT),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let c=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
    let rows=qualify_skills(&c).unwrap();
    let poison=&rows.iter().find(|r|r.0==745).unwrap().2;
    assert_eq!(poison.frames.iter().map(|f|f.duration_ticks).sum::<u32>(),30);
    assert_eq!(poison.frames[1].events,vec![FrameEvent::Sound(117)]);
    assert_eq!(c.sprites[&871].graphics_id,192);
    assert_ne!(c.resolve_action(192,ActionKey::new(0)).unwrap().animation.frames.len(),6);
    let mut changed=c.clone();changed.sprites.get_mut(&871).unwrap().graphics_id=871;assert!(qualify_skills(&changed).is_err());
    let mut w=world();w.accept(&[0x37,2,0,0,0,0x67,3]).unwrap();
    let mut fx=Effects::default();fx.prepare(Path::new(ROOT),&w,&AtomicBool::new(false));
    let mut canvas=Canvas::new(800,600).unwrap();
    let projection=ViewProjection{anchor_cell:WorldCell{x:32721,y:32781},anchor_pixel:PixelPoint{x:400,y:300}};
    let (r,e)=fx.draw(&w,&mut canvas,projection,0);
    assert!(r["load_error"].is_null());assert_eq!(r["omitted"],0);
    assert_eq!(r["effects"][0]["sprite"],"192-0.spr");assert_eq!(r["effects"][0]["subframe"],0);
    assert_eq!(e.len(),1);assert_eq!(e[0].sound,Some(141));
    for step in 1..5 { assert!(fx.draw(&w,&mut canvas,projection,step*41_000_000).1.is_empty()); }
    let (r,e)=fx.draw(&w,&mut canvas,projection,205_000_000);
    assert_eq!(r["effects"][0]["subframe"],5);assert_eq!(r["active"],1);assert!(e.is_empty());
    assert_eq!(fx.draw(&w,&mut canvas,projection,330_000_000).0["active"],0);
    w.accept(&[0x28,0x34,2,0]).unwrap();assert_eq!(fx.draw(&w,&mut canvas,projection,1_000_000_000).0["started"],0);
}
