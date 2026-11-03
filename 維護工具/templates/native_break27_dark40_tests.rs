
#[cfg(test)]
mod break27_dark40_development_tests {
    use super::*;
    use super::super::native_skill_selection::{Model,Target};
    const OWN:u32=0xfedcba98;const OTHER:u32=0xf1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();let bytes:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&bytes).unwrap();}
        w.learned_skills.clear();w.learned_skills.extend([27,40]);w.hp=Some([1,200]);w.mp=Some([1,200]);let mut a=w.own_actor().unwrap().clone();a.id=OTHER;a.x+=1;a.incarnation+=10;w.actors.insert(OTHER,a);w
    }
    fn menu(w:&World,skill:u16)->Model {let mut m=Model::default();m.sync(7,w,true);m.select(Some(skill)).unwrap();m}
    fn effect(w:&mut World,id:u32,gfx:u16) {let mut b=vec![0x37];b.extend(id.to_le_bytes());b.extend(gfx.to_le_bytes());w.accept(&b).unwrap();}
    #[test]
    fn break27_is_actor_xy_four_cell_request_and_three_second_gate() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors.get_mut(&OTHER).unwrap();a.x=own.x+4;a.y=own.y+4;let mut expected=vec![20,3,2];expected.extend(OTHER.to_le_bytes());expected.extend(a.x.to_le_bytes());expected.extend(a.y.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,27,Some(OTHER)).unwrap(),expected);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(27),3000);w.actors.get_mut(&OTHER).unwrap().x+=1;assert!(request(&w,27,Some(OTHER)).is_err());
    }
    #[test]
    fn dark40_is_actor_xy_three_cell_request_and_half_second_gate() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors.get_mut(&OTHER).unwrap();a.x=own.x+3;a.y=own.y+3;let mut expected=vec![20,4,7];expected.extend(OTHER.to_le_bytes());expected.extend(a.x.to_le_bytes());expected.extend(a.y.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,40,Some(OTHER)).unwrap(),expected);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(40),500);w.actors.get_mut(&OTHER).unwrap().y+=1;assert!(request(&w,40,Some(OTHER)).is_err());
    }
    #[test]
    fn both_status_spells_need_other_live_learned_actor_and_server_admission() {
        for skill in [27,40] {let mut w=world();assert!(request(&w,skill,None).is_err());assert!(request(&w,skill,Some(OWN)).is_err());w.actors.get_mut(&OTHER).unwrap().dead=true;assert!(request(&w,skill,Some(OTHER)).is_err());let mut w=world();w.learned_skills.remove(&skill);assert!(request(&w,skill,Some(OTHER)).is_err());let mut w=world();w.accept(&[0x25,2]).unwrap();assert!(request(&w,skill,Some(OTHER)).is_err());}
    }
    #[test]
    fn menu_preserves_weapon_break_and_darkness_labels_and_target_revisions() {
        let w=world();for (skill,label) in [(27,"#27 壞物術（選取目標，四格內；消耗 1 顆魔法寶石）"),(40,"#40 黑闇之影（選取目標，三格內；消耗 1 顆魔法寶石）")] {let m=menu(&w,skill);assert_eq!(m.rows().iter().find(|r|r.id==skill).unwrap().label,label);let t=Target::capture(&w,OTHER);assert_eq!(m.choice(7,&w,t).unwrap().target,Some(OTHER));let mut next=w.clone();next.actors.get_mut(&OTHER).unwrap().target_revision+=1;assert!(m.choice(7,&next,t).is_err());let mut next=w.clone();next.actors.get_mut(&OTHER).unwrap().incarnation+=1;assert!(m.choice(7,&next,t).is_err());}
    }
    #[test]
    fn both_menu_intents_retire_on_connection_correction_generation_or_caster_change() {
        let w=world();for skill in [27,40] {let m=menu(&w,skill);let t=Target::capture(&w,OTHER);assert!(m.choice(8,&w,t).is_err());let mut next=w.clone();next.local_corrections+=1;assert!(m.choice(7,&next,t).is_err());let mut next=w.clone();next.generation+=1;assert!(m.choice(7,&next,t).is_err());let mut next=w.clone();next.actors.get_mut(&OWN).unwrap().incarnation+=1;assert!(m.choice(7,&next,t).is_err());}
    }
    #[test]
    fn dark_effect_is_not_blind_state_and_s0a_is_owner_scoped_until_server_clear() {
        let mut w=world();let mut e=super::super::native_environment::State::default();e.begin(7).unwrap();effect(&mut w,OTHER,2175);assert_eq!(e.blind,0);e.accept(7,&[0x0a,1]).unwrap();assert_eq!(e.blind,1);assert!(e.accept(8,&[0x0a,0]).is_err());assert_eq!(e.blind,1);e.accept(7,&[0x0a,0]).unwrap();assert_eq!(e.blind,0);e.accept(7,&[0x0a,1]).unwrap();e.retire();assert_eq!(e.blind,0);assert!(e.accept(7,&[0x0a,1]).is_err());
    }
    #[test]
    fn weapon_break_effect_does_not_change_inventory_and_extended_server_detail_is_authoritative() {
        use super::super::native_inventory::{Inventory,Item};let mut w=world();let mut inv=Inventory::default();inv.begin(7,"fixture").unwrap();inv.items.insert(9,Item{id:9,use_type:1,charge:0,icon:1,blessing:1,count:1,identified:1,name:"weapon".into(),detail:vec![1,2,3]});let before=inv.clone();request(&w,27,Some(OTHER)).unwrap();effect(&mut w,OTHER,172);assert_eq!(inv,before);
        let mut b=vec![0x6f];b.extend(9_u32.to_le_bytes());b.extend(b"damaged weapon\0");b.extend(1_u32.to_le_bytes());b.extend([3,7,8,9]);inv.accept(7,&b).unwrap();assert_eq!(inv.items[&9].name,"damaged weapon");assert_eq!(inv.items[&9].detail,vec![7,8,9]);let after=inv.clone();assert!(inv.accept(8,&b).is_err());assert_eq!(inv,after);
    }
    #[test]
    fn real_failure_and_material_messages_do_not_claim_hit_or_local_spending() {
        let mut w=world();let now=Instant::now();let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};let hp=w.hp;let mp=w.mp;w.accept(&[0x10,0x2b,0x01,0]).unwrap();assert_eq!(p.observe(&w,now),Feedback::Rejected(299));w.accept(&[0x10,0x18,0x01,0]).unwrap();assert_eq!(p.observe(&w,now),Feedback::Rejected(280));assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);w.generation+=1;assert_eq!(p.observe(&w,now),Feedback::Interrupted);
    }
    #[test]
    fn effect_binding_and_bad_tail_never_acquire_a_later_target() {
        let mut w=world();effect(&mut w,OTHER,172);let captured=w.effects.last().unwrap().target;w.actors.get_mut(&OTHER).unwrap().incarnation+=1;assert_ne!(captured,Some((OTHER,w.actors[&OTHER].incarnation)));let before=serde_json::to_value(&w).unwrap();let mut bad=vec![0x37];bad.extend(OTHER.to_le_bytes());bad.push(0x7f);assert!(w.accept(&bad).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
    }
    #[test]
    #[ignore="requires original shared resources; explicit resource phase only"]
    fn original_break_and_darkness_frames_cues_mix_and_finite_retirement() {
        use super::super::{resources::{SharedArchive,SharedArchiveKind},sprite_actions::{SpriteActionCatalog,ActionLimits,ActionKey,FrameEvent},native_skill_effects::Effects,native_world_effects::{Target as RenderTarget,Mix},world_map::{WorldCell,PixelPoint,ViewProjection},combat_animation::NativeAudioMode};use std::{path::Path,sync::atomic::AtomicBool};let root=Path::new(r"D:\lineage開發\0-客戶端");let raw=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();let c=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();super::super::native_additional_finite_effects::qualify(&c).unwrap();
        for (gfx,sound,sound_frame,ticks) in [(172_u16,100,2,vec![5,1,1,1,3,3,3,3,3,3,4,4,4,4,4]),(2175,802,1,vec![4;8])] {let a=c.resolve_action(u32::from(gfx),ActionKey::new(0)).unwrap().animation;assert_eq!(a.frames.iter().map(|f|f.duration_ticks).collect::<Vec<_>>(),ticks);assert_eq!(a.frames.iter().enumerate().flat_map(|(i,f)|f.events.iter().map(move|e|(i,e.clone()))).collect::<Vec<_>>(),vec![(sound_frame,FrameEvent::Sound(sound))]);let mut w=world();effect(&mut w,OTHER,gfx);let own=w.own_actor().unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:PixelPoint{x:400,y:300}};let mut effects=Effects::default();effects.prepare(root,&w,&AtomicBool::new(false));let mut cues=vec![];let mut drawn=false;
            for step in 0..=300_u64 {let mut fragments=vec![];let(report,events)=effects.draw_to(&w,&mut RenderTarget::Fragments(&mut fragments),view,step*10_000_000);assert!(report["load_error"].is_null(),"{report}");for e in &fragments {assert!(if gfx==2175{matches!(e.mix,Mix::Multiply)}else{matches!(e.mix,Mix::Screen)});}drawn|=!fragments.is_empty();cues.extend(events);if step==300{assert_eq!(report["active"],0);}}
            assert!(drawn);assert_eq!(cues.len(),1);assert_eq!((cues[0].frame,cues[0].sound),(sound_frame,Some(sound)));assert_eq!(cues[0].prepared_audio.as_ref().unwrap().group().mode,NativeAudioMode::SuppressWhileSameIdPlaying);w.clear_preserving_generation();let mut fragments=vec![];let(report,events)=effects.draw_to(&w,&mut RenderTarget::Fragments(&mut fragments),view,u64::MAX);assert_eq!(report["active"],0);assert!(events.is_empty());
        }
    }
}
