
#[cfg(test)]
mod reveal23_tame36_development_tests {
    use super::*;
    use super::super::native_skill_selection::{Model,Target};
    const OWN:u32=0xfedcba98;const OTHER:u32=0xf1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();let bytes:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&bytes).unwrap();}
        w.learned_skills.clear();w.learned_skills.extend([23,36]);w.hp=Some([1,200]);w.mp=Some([1,200]);let mut a=w.own_actor().unwrap().clone();a.id=OTHER;a.x+=1;a.incarnation+=10;w.actors.insert(OTHER,a);w
    }
    fn menu(w:&World,skill:u16)->Model {let mut m=Model::default();m.sync(7,w,true);m.select(Some(skill)).unwrap();m}
    fn pending(w:&World,now:Instant)->Pending {Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation}}
    fn effect(w:&mut World,id:u32,gfx:u16) {let mut b=vec![0x37];b.extend(id.to_le_bytes());b.extend(gfx.to_le_bytes());w.accept(&b).unwrap();}
    #[test]
    fn reveal23_wire_is_seven_bytes_and_has_no_coordinates() {
        let w=world();let mut expected=vec![20,2,6];expected.extend(OTHER.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,23,Some(OTHER)).unwrap(),expected);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(23),10);
    }
    #[test]
    fn tame36_wire_is_eleven_bytes_and_never_spends_or_creates_a_companion_locally() {
        let w=world();let a=&w.actors[&OTHER];let mut expected=vec![20,4,3];expected.extend(OTHER.to_le_bytes());expected.extend(a.x.to_le_bytes());expected.extend(a.y.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,36,Some(OTHER)).unwrap(),expected);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(36),10);
    }
    #[test]
    fn both_minus_one_ranges_use_native_asymmetric_screen_and_reject_self() {
        for skill in [23,36] {let mut w=world();assert!(request(&w,skill,None).is_err());assert!(request(&w,skill,Some(OWN)).is_err());let own=w.own_actor().unwrap().clone();
            for (dx,dy) in [(18,0),(-18,0),(0,18),(0,-18),(10,9),(-10,-9),(10,-9),(-10,9),(19,0),(-19,0),(20,0)] {let a=w.actors.get_mut(&OTHER).unwrap();a.x=(i32::from(own.x)+dx)as u16;a.y=(i32::from(own.y)+dy)as u16;let expected=in_native_screen([own.x,own.y],[a.x,a.y]);assert_eq!(request(&w,skill,Some(OTHER)).is_ok(),expected,"skill={skill} offset={dx},{dy}");}
        }
    }
    #[test]
    fn menu_preserves_monster_intent_and_rejects_retired_selection() {
        let w=world();for (skill,label) in [(23,"#23 能量感測（選取怪物，畫面內）"),(36,"#36 迷魅術（選取怪物，畫面內；消耗 1 顆魔法寶石）")] {
            let m=menu(&w,skill);assert_eq!(m.rows().iter().find(|r|r.id==skill).unwrap().label,label);let t=Target::capture(&w,OTHER);assert_eq!(m.choice(7,&w,t).unwrap().target,Some(OTHER));assert!(m.choice(8,&w,t).is_err());
            let mut next=w.clone();next.actors.get_mut(&OTHER).unwrap().incarnation+=1;assert!(m.choice(7,&next,t).is_err());let mut next=w.clone();next.local_corrections+=1;assert!(m.choice(7,&next,t).is_err());let mut next=w.clone();next.generation+=1;assert!(m.choice(7,&next,t).is_err());
        }
    }
    #[test]
    fn spell_intent_rejects_dead_missing_unlearned_and_server_controlled_actors() {
        for skill in [23,36] {let mut w=world();w.actors.get_mut(&OTHER).unwrap().dead=true;assert!(request(&w,skill,Some(OTHER)).is_err());w.actors.remove(&OTHER);assert!(request(&w,skill,Some(OTHER)).is_err());let mut w=world();w.learned_skills.remove(&skill);assert!(request(&w,skill,Some(OTHER)).is_err());let mut w=world();w.accept(&[0x25,2]).unwrap();assert!(request(&w,skill,Some(OTHER)).is_err());}
    }
    #[test]
    fn weakness_keeps_all_wire_elements_and_never_relabels_english_catalog_comments() {
        let mut w=world();let hp=w.hp;let mp=w.mp;for gfx in [2510,2169,2167,2166,2168] {effect(&mut w,OTHER,gfx);}
        assert_eq!(w.effects.iter().map(|e|e.graphics).collect::<Vec<_>>(),vec![2510,2169,2167,2166,2168]);assert!(w.effects.iter().all(|e|e.target==Some((OTHER,w.actors[&OTHER].incarnation))));assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);assert!(w.movement_blocks.is_empty());
    }
    #[test]
    fn tame_capacity_message_is_real_rejection_not_companion_success() {
        let mut w=world();let now=Instant::now();let p=pending(&w,now);let mut b=vec![0x20];b.extend(OWN.to_le_bytes());b.push(19);w.accept(&b).unwrap();assert_eq!(p.observe(&w,now),Feedback::CastAction);let count=w.actors.len();w.accept(&[0x10,0x3f,0x01,0]).unwrap();assert_eq!(p.observe(&w,now),Feedback::Rejected(319));assert_eq!(Feedback::Rejected(319).label(),"可控制的同伴數量已達上限");assert_eq!(w.actors.len(),count);w.generation+=1;assert_eq!(p.observe(&w,now),Feedback::Interrupted);
    }
    #[test]
    fn tame_effect_and_removed_monster_never_bind_to_a_future_replacement() {
        let mut w=world();let old=w.actors[&OTHER].clone();let t=Target::capture(&w,OTHER);effect(&mut w,OTHER,228);let captured=w.effects.last().unwrap().target;let mut b=vec![0x08];b.extend(OTHER.to_le_bytes());w.accept(&b).unwrap();assert!(!w.actors.contains_key(&OTHER));let count=w.effects.len();effect(&mut w,OTHER,228);assert_eq!(w.effects.len(),count);let mut replacement=old;replacement.incarnation+=1;w.actors.insert(OTHER,replacement);assert_ne!(captured,Some((OTHER,w.actors[&OTHER].incarnation)));assert!(menu(&w,36).choice(7,&w,t).is_err());
    }
    #[test]
    fn old_capacity_message_or_malformed_new_message_cannot_reject_new_intent() {
        let mut w=world();w.accept(&[0x10,0x3f,0x01,0]).unwrap();let now=Instant::now();let p=pending(&w,now);assert_eq!(p.observe(&w,now),Feedback::Pending);let before=serde_json::to_value(&w).unwrap();assert!(w.accept(&[0x10,0x3f,0x01,1]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(p.observe(&w,now+Duration::from_secs(2)),Feedback::NoResponse);
    }
    #[test]
    #[ignore="requires original shared resources; explicit resource test phase only"]
    fn original_reveal_and_tame_images_audio_cadence_and_finite_retirement() {
        use super::super::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{SpriteActionCatalog,ActionLimits,ActionKey,FrameEvent},native_skill_effects::Effects,native_world_effects::Target as RenderTarget,world_map::{WorldCell,PixelPoint,ViewProjection},combat_animation::NativeAudioMode};
        use std::{path::Path,sync::atomic::AtomicBool};let root=Path::new(r"D:\lineage開發\0-客戶端");let raw=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let catalog=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();super::super::native_additional_finite_effects::qualify(&catalog).unwrap();super::super::native_skill_effects::qualify_skills(&catalog).unwrap();
        for (gfx,sound,ticks) in [(228_u16,Some(132),vec![4,2,2,4,4,2,2]),(2166,Some(809),vec![4;11]),(2167,Some(810),vec![4;11]),(2168,Some(811),vec![4;11]),(2169,Some(812),vec![4;11]),(2510,None,vec![4])] {
            let a=catalog.resolve_action(u32::from(gfx),ActionKey::new(0)).unwrap().animation;assert_eq!(a.frames.iter().map(|f|f.duration_ticks).collect::<Vec<_>>(),ticks);assert_eq!(a.frames.iter().enumerate().flat_map(|(i,f)|f.events.iter().map(move|e|(i,e.clone()))).collect::<Vec<_>>(),sound.map(|id|vec![(1,FrameEvent::Sound(id))]).unwrap_or_default());
            let mut w=world();effect(&mut w,OTHER,gfx);let before=serde_json::to_value(&w).unwrap();let own=w.own_actor().unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:PixelPoint{x:400,y:300}};let mut effects=Effects::default();effects.prepare(root,&w,&AtomicBool::new(false));let mut cues=vec![];let mut drawn=false;
            for step in 0..=300_u64 {let mut fragments=vec![];let(report,events)=effects.draw_to(&w,&mut RenderTarget::Fragments(&mut fragments),view,step*10_000_000);assert!(report["load_error"].is_null(),"{report}");drawn|=!fragments.is_empty();cues.extend(events);if step==300{assert_eq!(report["active"],0);}}
            assert!(drawn);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(cues.len(),usize::from(sound.is_some()));if let Some(id)=sound {assert_eq!((cues[0].frame,cues[0].sound),(1,Some(id)));assert_eq!(cues[0].prepared_audio.as_ref().unwrap().group().mode,NativeAudioMode::SuppressWhileSameIdPlaying);}
            w.clear_preserving_generation();let mut fragments=vec![];let(report,events)=effects.draw_to(&w,&mut RenderTarget::Fragments(&mut fragments),view,u64::MAX);assert_eq!(report["active"],0);assert!(events.is_empty());
        }
    }
}
