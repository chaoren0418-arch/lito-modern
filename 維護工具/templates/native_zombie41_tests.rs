
#[cfg(test)]
mod zombie41_development_tests {
    use super::*;
    use super::super::native_skill_selection::{Model,Target};
    const OWN:u32=0xfedcba98;const CORPSE:u32=0xf1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();let b:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
        w.learned_skills.clear();w.learned_skills.extend([23,27,41]);w.hp=Some([1,200]);w.mp=Some([1,200]);let mut a=w.own_actor().unwrap().clone();a.id=CORPSE;a.x+=1;a.incarnation+=10;a.dead=true;a.action_sequence+=10;w.actors.insert(CORPSE,a);w
    }
    fn menu(w:&World)->Model {let mut m=Model::default();m.sync(7,w,true);m.select(Some(41)).unwrap();m}
    fn effect(w:&mut World) {let mut b=vec![0x37];b.extend(CORPSE.to_le_bytes());b.extend(226_u16.to_le_bytes());w.accept(&b).unwrap();}
    #[test]
    fn zombie41_is_seven_byte_corpse_id_and_leaves_costs_and_companion_state_to_server() {
        let w=world();let mut expected=vec![20,5,0];expected.extend(CORPSE.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,41,Some(CORPSE)).unwrap(),expected);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(41),10);
    }
    #[test]
    fn zombie41_rejects_live_self_missing_unlearned_and_controlled_intent() {
        let mut w=world();assert!(request(&w,41,None).is_err());assert!(request(&w,41,Some(OWN)).is_err());w.actors.get_mut(&CORPSE).unwrap().dead=false;assert!(request(&w,41,Some(CORPSE)).is_err());w.actors.remove(&CORPSE);assert!(request(&w,41,Some(CORPSE)).is_err());let mut w=world();w.learned_skills.remove(&41);assert!(request(&w,41,Some(CORPSE)).is_err());let mut w=world();w.accept(&[0x25,2]).unwrap();assert!(request(&w,41,Some(CORPSE)).is_err());
    }
    #[test]
    fn zombie41_minus_one_range_is_native_screen_not_unlimited() {
        let mut w=world();let own=w.own_actor().unwrap().clone();for (dx,dy) in [(18,0),(-18,0),(10,9),(-10,-9),(10,-9),(-10,9),(19,0),(-19,0),(20,0)] {let a=w.actors.get_mut(&CORPSE).unwrap();a.x=(i32::from(own.x)+dx)as u16;a.y=(i32::from(own.y)+dy)as u16;let valid=in_native_screen([own.x,own.y],[a.x,a.y]);assert_eq!(request(&w,41,Some(CORPSE)).is_ok(),valid);}
    }
    #[test]
    fn mouse_captured_corpse_is_valid_only_for_corpse_skill_and_retained_death_action() {
        let mut w=world();let t=Target::capture(&w,CORPSE).unwrap();assert_eq!(t.resolve_for_skill(&w,41),Some(CORPSE));assert_eq!(t.resolve(&w),None);assert_eq!(t.resolve_for_skill(&w,27),None);w.actors.get_mut(&CORPSE).unwrap().dead=false;assert_eq!(t.resolve_for_skill(&w,41),None);assert_eq!(t.resolve(&w),None);let a=w.actors.get_mut(&CORPSE).unwrap();a.dead=true;a.action_sequence+=1;assert_eq!(t.resolve_for_skill(&w,41),None);
    }
    #[test]
    fn living_selection_never_becomes_corpse_authorization_after_death() {
        let mut w=world();w.actors.get_mut(&CORPSE).unwrap().dead=false;let live=Target::capture(&w,CORPSE).unwrap();assert_eq!(live.resolve(&w),Some(CORPSE));w.actors.get_mut(&CORPSE).unwrap().dead=true;assert_eq!(live.resolve_for_skill(&w,41),None);assert!(menu(&w).choice(7,&w,Some(live)).is_err());let fresh=Target::capture(&w,CORPSE).unwrap();assert_eq!(menu(&w).choice(7,&w,Some(fresh)).unwrap().target,Some(CORPSE));
    }
    #[test]
    fn corpse_choice_rejects_revision_incarnation_connection_correction_and_generation() {
        let w=world();let m=menu(&w);let t=Target::capture(&w,CORPSE);assert!(m.choice(8,&w,t).is_err());let mut n=w.clone();n.actors.get_mut(&CORPSE).unwrap().target_revision+=1;assert!(m.choice(7,&n,t).is_err());let mut n=w.clone();n.actors.get_mut(&CORPSE).unwrap().incarnation+=1;assert!(m.choice(7,&n,t).is_err());let mut n=w.clone();n.local_corrections+=1;assert!(m.choice(7,&n,t).is_err());let mut n=w.clone();n.generation+=1;assert!(m.choice(7,&n,t).is_err());
    }
    #[test]
    fn zombie_menu_requires_explicit_corpse_and_keeps_normal_live_spells_separate() {
        let w=world();let mut m=menu(&w);assert_eq!(m.rows().iter().find(|r|r.id==41).unwrap().label,"#41 造屍術（選取屍體，畫面內；消耗 1 顆魔法寶石）");assert!(m.choice(7,&w,None).is_err());let t=Target::capture(&w,CORPSE);assert!(m.choice(7,&w,t).is_ok());m.select(Some(23)).unwrap();assert!(m.choice(7,&w,t).is_err());m.select(Some(27)).unwrap();assert!(m.choice(7,&w,t).is_err());
    }
    #[test]
    fn capacity_then_226_effect_is_not_created_companion_or_local_HP_spending() {
        let mut w=world();let before_ids:Vec<_>=w.actors.keys().copied().collect();let hp=w.hp;let mp=w.mp;let now=Instant::now();let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};w.accept(&[0x10,0x3f,0x01,0]).unwrap();effect(&mut w);assert_eq!(p.observe(&w,now),Feedback::Rejected(319));assert_eq!(w.actors.keys().copied().collect::<Vec<_>>(),before_ids);assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);assert_eq!(w.effects.last().unwrap().target,Some((CORPSE,w.actors[&CORPSE].incarnation)));
    }
    #[test]
    fn removed_corpse_and_same_id_replacement_cannot_reuse_intent() {
        let mut w=world();let t=Target::capture(&w,CORPSE);let mut a=w.actors[&CORPSE].clone();let mut b=vec![0x08];b.extend(CORPSE.to_le_bytes());w.accept(&b).unwrap();assert!(menu(&w).choice(7,&w,t).is_err());a.incarnation+=1;w.actors.insert(CORPSE,a);assert!(menu(&w).choice(7,&w,t).is_err());
    }
    #[test]
    #[ignore="requires original shared resources; explicit resource phase only"]
    fn original_226_sound139_and_receipt_position_survive_corpse_consumption_once() {
        use super::super::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{SpriteActionCatalog,ActionLimits,ActionKey,FrameEvent},native_skill_effects::Effects,native_world_effects::Target as RenderTarget,world_map::{WorldCell,PixelPoint,ViewProjection}};use std::{path::Path,sync::atomic::AtomicBool};let root=Path::new(r"D:\lineage開發\0-客戶端");let raw=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let c=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();super::super::native_additional_finite_effects::qualify(&c).unwrap();let a=c.resolve_action(226,ActionKey::new(0)).unwrap().animation;assert_eq!(a.frames.iter().map(|f|f.duration_ticks).collect::<Vec<_>>(),vec![4,2,2,4,4,2,2]);assert_eq!(a.frames[1].events,vec![FrameEvent::Sound(139)]);let mut w=world();effect(&mut w);let p=w.effects.last().unwrap().position;let own=w.own_actor().unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:PixelPoint{x:400,y:300}};let mut fx=Effects::default();fx.prepare(root,&w,&AtomicBool::new(false));let mut cues=vec![];let mut drawn_after_remove=false;
        for step in 0..=200_u64 {if step==3 {let mut b=vec![0x08];b.extend(CORPSE.to_le_bytes());w.accept(&b).unwrap();}let mut fragments=vec![];let(report,events)=fx.draw_to(&w,&mut RenderTarget::Fragments(&mut fragments),view,step*10_000_000);assert!(report["load_error"].is_null(),"{report}");for f in &fragments {assert_eq!(f.position,WorldCell{x:i32::from(p[0]),y:i32::from(p[1])});}drawn_after_remove|=step>3&&!fragments.is_empty();cues.extend(events);if step==200{assert_eq!(report["active"],0);}}
        assert!(drawn_after_remove);assert_eq!(cues.len(),1);assert_eq!((cues[0].frame,cues[0].sound),(1,Some(139)));w.clear_preserving_generation();w.generation+=1;let mut fragments=vec![];let(report,events)=fx.draw_to(&w,&mut RenderTarget::Fragments(&mut fragments),view,2_000_000_001);assert_eq!(report["active"],0);assert!(events.is_empty());
    }
}
