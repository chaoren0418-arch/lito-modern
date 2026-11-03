use lito_modern::modern_lito::{
    compositor::Canvas, native_skill_effects::{Effects, qualify_skills},
    native_skill_selection::{Model, Target}, native_skills,
    native_world::World, resources::{SharedArchive, SharedArchiveKind},
    sprite_actions::{ActionKey, ActionLimits, FrameEvent, SpriteActionCatalog},
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::{path::Path, sync::atomic::AtomicBool};
const ROOT: &str = r"D:\lineage開發\0-客戶端";
fn world() -> World {
    let data: serde_json::Value = serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        let s = data["packets"][key].as_str().unwrap();
        let b: Vec<_> = (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
        w.accept(&b).unwrap();
    }
    w.learned_skills.insert(8);
    w
}
#[test]
fn object_selector_screen_and_owner_guards_preserve_authority() {
    let mut w = world();
    let own = w.status.as_ref().unwrap().object_id;
    let before = serde_json::to_value(&w).unwrap();
    let mut expected = vec![0x14, 0, 7]; expected.extend(own.to_le_bytes());
    assert_eq!(native_skills::request(&w,8,None).unwrap(),expected);
    assert_eq!(native_skills::request(&w,8,Some(2)).unwrap(),vec![0x14,0,7,2,0,0,0]);
    let mut m = Model::default(); m.sync(7,&w,true); m.select(Some(8)).unwrap();
    assert!(m.rows()[0].label.contains("神聖武器"));
    let target = Target::capture(&w,2).unwrap();
    assert_eq!(m.choice(7,&w,Some(target)).unwrap().target,Some(2));
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    let own_pos = {let a=w.own_actor().unwrap();[a.x,a.y]};
    w.actors.get_mut(&2).unwrap().x=own_pos[0]+19;
    w.actors.get_mut(&2).unwrap().y=own_pos[1]+1;
    assert!(native_skills::request(&w,8,Some(2)).is_err());
    w.actors.get_mut(&2).unwrap().x=own_pos[0];
    w.actors.get_mut(&2).unwrap().dead=true;
    assert!(m.choice(7,&w,Some(target)).is_err());
    w.set_observer_mode(true);
    assert!(native_skills::request(&w,8,None).is_err());
}
#[test]
fn original_holy_definition_is_exact_and_changed_controls_fail_closed() {
    let bytes=SharedArchive::open(Path::new(ROOT),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let c=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits::default()).unwrap();
    let rows=qualify_skills(&c).unwrap();
    assert_eq!(rows.iter().map(|v|v.0).collect::<Vec<_>>(),[221,744,745,749,2510,829,830,831,761,2165,871,754]);
    let a=&rows.iter().find(|v|v.0==2165).unwrap().2;
    assert_eq!(a.frames.len(),13);
    assert_eq!(a.frames[1].events,vec![FrameEvent::Sound(804)]);
    let mut altered=c.clone();
    altered.sprites.get_mut(&2165).unwrap().attributes[0].body_bytes=b"5".to_vec();
    assert!(qualify_skills(&altered).is_err());
    assert_eq!(c.resolve_action(2165,ActionKey::new(0)).unwrap().animation.frames[0].subframe,12);
}
#[test]
fn holy_sound_terminal_dwell_and_map_retirement_do_not_predict_a_buff() {
    let mut w=world();
    w.accept(&[0x37,2,0,0,0,0x75,8]).unwrap();
    let before=serde_json::to_value(&w).unwrap();
    let mut fx=Effects::default();fx.prepare(Path::new(ROOT),&w,&AtomicBool::new(false));
    let mut canvas=Canvas::new(800,600).unwrap();
    let projection=ViewProjection{anchor_cell:WorldCell{x:32721,y:32781},anchor_pixel:PixelPoint{x:400,y:300}};
    let (r,e)=fx.draw(&w,&mut canvas,projection,0);
    assert_eq!(r["load_error"],serde_json::Value::Null);assert_eq!(r["started"],1);
    assert_eq!(e.len(),1);assert_eq!(e[0].sound,Some(804));
    assert_eq!(r["effects"][0]["subframe"],0);
    assert!(fx.draw(&w,&mut canvas,projection,0).1.is_empty());
    for step in 1..12 { assert!(fx.draw(&w,&mut canvas,projection,step*166_000_000).1.is_empty()); }
    let (r,e)=fx.draw(&w,&mut canvas,projection,1_992_000_000);
    assert!(e.is_empty());assert_eq!(r["effects"][0]["subframe"],12);assert_eq!(r["active"],1);
    assert_eq!(fx.draw(&w,&mut canvas,projection,2_158_000_000).0["active"],0);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    w.accept(&[0x28,0x34,2,0]).unwrap();
    let (r,e)=fx.draw(&w,&mut canvas,projection,3_000_000_000);
    assert_eq!(r["started"],0);assert_eq!(r["active"],0);assert!(e.is_empty());
}
