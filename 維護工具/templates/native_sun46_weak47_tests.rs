
#[cfg(test)]
mod sun46_weak47_development_tests {
    use super::*;
    use super::super::native_skill_selection::{Model,Target};
    const OWN:u32=0xfedcba98;const OTHER:u32=0xf1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();let p:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&p).unwrap();}
        w.learned_skills.clear();w.learned_skills.extend([46,47]);w.hp=Some([1,200]);w.mp=Some([1,200]);
        let mut a=w.own_actor().unwrap().clone();a.id=OTHER;a.incarnation+=10;a.x+=1;w.actors.insert(OTHER,a);w
    }
    fn menu(w:&World,id:u16)->Model {let mut m=Model::default();m.sync(7,w,true);m.select(Some(id)).unwrap();m}
    fn effect(w:&mut World){let mut b=vec![0x37];b.extend(OTHER.to_le_bytes());b.extend(2228_u16.to_le_bytes());w.accept(&b).unwrap();}
    fn pending(w:&World,now:Instant)->Pending {Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation}}
    #[test]
    fn both_typed_requests_preserve_high_actor_id_xy_and_server_cost_authority() {
        let w=world();for (skill,column) in [(46,5),(47,6)] {let a=&w.actors[&OTHER];let mut expected=vec![20,5,column,0x67,0x45,0x23,0xf1];expected.extend(a.x.to_le_bytes());expected.extend(a.y.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,skill,Some(OTHER)).unwrap(),expected);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(skill),500);assert!(!item_targeted(skill));}
    }
    #[test]
    fn three_cell_diagonal_boundary_does_not_accept_four_cells_or_self() {
        for skill in [46,47] {let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors.get_mut(&OTHER).unwrap();a.x=own.x+3;a.y=own.y+3;assert!(request(&w,skill,Some(OTHER)).is_ok());w.actors.get_mut(&OTHER).unwrap().x+=1;assert!(request(&w,skill,Some(OTHER)).is_err());assert!(request(&w,skill,None).is_err());assert!(request(&w,skill,Some(OWN)).is_err());}
    }
    #[test]
    fn menu_target_and_owner_lease_do_not_reauthorize_after_aba_or_transfer() {
        for (skill,label) in [(46,"#46 烈炎術（選取目標，三格內）"),(47,"#47 弱化術（選取目標，三格內；消耗 1 顆魔法寶石）")] {
            let w=world();let m=menu(&w,skill);assert_eq!(m.rows().iter().find(|r|r.id==skill).unwrap().label,label);let t=Target::capture(&w,OTHER);assert_eq!(m.choice(7,&w,t).unwrap().target,Some(OTHER));assert!(m.choice(8,&w,t).is_err());
            for kind in 0..4 {let mut next=w.clone();match kind{0=>next.generation+=1,1=>next.local_corrections+=1,2=>next.actors.get_mut(&OTHER).unwrap().incarnation+=1,_=>next.actors.get_mut(&OTHER).unwrap().target_revision+=1};assert!(m.choice(7,&next,t).is_err());}
        }
    }
    #[test]
    fn live_book_visibility_and_control_are_required_without_guessing_mp20_or_materials() {
        for skill in [46,47] {let mut w=world();assert!(request(&w,skill,Some(OTHER)).is_ok());w.mp=Some([0,200]);assert!(request(&w,skill,Some(OTHER)).is_err());w.mp=Some([1,200]);w.actors.get_mut(&OTHER).unwrap().dead=true;assert!(request(&w,skill,Some(OTHER)).is_err());w.actors.get_mut(&OTHER).unwrap().dead=false;w.actors.get_mut(&OTHER).unwrap().draw_state.invisible=true;assert!(request(&w,skill,Some(OTHER)).is_err());w.actors.get_mut(&OTHER).unwrap().draw_state.invisible=false;w.accept(&[0x25,2]).unwrap();assert!(request(&w,skill,Some(OTHER)).is_err());w.accept(&[0x25,3]).unwrap();w.learned_skills.remove(&skill);assert!(request(&w,skill,Some(OTHER)).is_err());}
    }
    #[test]
    fn S23_sunburst_freezes_wire_target_and_never_applies_immediate_hurt_or_hp() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let target=w.actors[&OTHER].clone();let hp=w.hp;let mp=w.mp;
        let mut b=vec![0x23,18];b.extend(OWN.to_le_bytes());b.extend(OTHER.to_le_bytes());b.extend([10,2]);b.extend(77_u32.to_le_bytes());b.extend(1811_u16.to_le_bytes());b.push(0);for v in [own.x,own.y,target.x,target.y]{b.extend(v.to_le_bytes());}b.extend([0,0]);w.accept(&b).unwrap();
        assert_eq!(w.actors[&OTHER].action_sequence,target.action_sequence);assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);let cue=w.projectiles.last().unwrap();assert_eq!(cue.projectile.to,[target.x,target.y]);assert_eq!(cue.projectile.flight_type,0);assert!(super::super::native_class1::supports_graphics(1811));
    }
    #[test]
    fn S37_weakness_visual_never_predicts_server_stats_or_sixty_four_second_status() {
        let mut w=world();let before=w.actors[&OTHER].clone();let hp=w.hp;let mp=w.mp;let blocks=w.movement_blocks.clone();effect(&mut w);
        assert_eq!(w.actors[&OTHER],before);assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);assert_eq!(w.movement_blocks,blocks);assert_eq!(w.effects.last().unwrap().graphics,2228);
    }
    #[test]
    fn nine_frame_weakness_clock_ends_at1166ms_and_is_not_the_server_status_timer() {
        let raw=include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-sun46-weak47-inspection\20261024-100201-695865\2228.definition.txt");let bytes=[b"11297 0 0\n".as_slice(),raw].concat();
        let c=super::super::sprite_actions::SpriteActionCatalog::parse_for_inspection(&bytes,super::super::sprite_actions::ActionLimits::default()).unwrap();let a=c.resolve_action(2228,super::super::sprite_actions::ActionKey::new(0)).unwrap().animation.clone();assert_eq!(a.frames.len(),9);
        let mut p=super::super::native_effect_timing::Clip::new(std::sync::Arc::new(a)).unwrap().start();let first=p.advance(0).unwrap();assert!(first.crossed_cues.iter().any(|c|c.event==super::super::sprite_actions::FrameEvent::Sound(843)));
        let mut end=None;for _ in 0..10{let Some(at)=p.next_callback_millis() else{break;};if p.advance(at*1_000_000).unwrap().finished{end=Some(at);break;}}assert_eq!(end,Some(1166));
    }
    #[test]
    fn effect_or_caster_action_is_not_a_weakness_success_ack_and_rejection_wins() {
        let mut w=world();let now=Instant::now();let p=pending(&w,now);effect(&mut w);assert_eq!(p.observe(&w,now),Feedback::Pending);
        let mut b=vec![0x20];b.extend(OWN.to_le_bytes());b.push(19);w.accept(&b).unwrap();assert_eq!(p.observe(&w,now),Feedback::CastAction);w.message_count+=1;w.last_skill_rejection=Some((w.message_count,280));assert_eq!(p.observe(&w,now),Feedback::Rejected(280));
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; original2228/WAV843 Multiply memory fragments and prepared audio only"]
    fn original_weakness_profile_loads_and_submits_multiply_without_a_status_timer() {
        let root:std::path::PathBuf=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root").into();let mut w=world();effect(&mut w);let mut e=super::super::native_skill_effects::Effects::default();e.prepare(&root,&w,&std::sync::atomic::AtomicBool::new(false));
        let own=w.own_actor().unwrap();let view=super::super::world_map::ViewProjection{anchor_cell:super::super::world_map::WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};
        let mut fragments=vec![];let(receipt,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,0);
        assert!(receipt["additional_load_error"].is_null(),"{receipt}");assert_eq!(fragments.len(),1);assert!(matches!(fragments[0].mix,super::super::native_world_effects::Mix::Multiply));assert_eq!(audio.iter().filter(|a|a.sound==Some(843)&&a.prepared_audio.is_some()).count(),1);
    }
}
