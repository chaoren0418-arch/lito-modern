
#[cfg(test)]
mod eruption45_entry_development_tests {
    use super::*;
    use super::super::native_skill_selection::{Model,Target};
    const OWN:u32=0xfedcba98;const OTHER:u32=0xf1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {
            let h=f["packets"][key].as_str().unwrap();let packet:Vec<_>=(0..h.len()).step_by(2)
                .map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&packet).unwrap();
        }
        w.learned_skills.clear();w.learned_skills.insert(45);w.hp=Some([1,200]);w.mp=Some([1,200]);
        let mut a=w.own_actor().unwrap().clone();a.id=OTHER;a.incarnation+=10;a.x+=3;w.actors.insert(OTHER,a);w
    }
    fn menu(w:&World)->Model {let mut m=Model::default();m.sync(7,w,true);m.select(Some(45)).unwrap();m}
    fn s23(w:&World,marker:u8)->Vec<u8> {
        let own=w.own_actor().unwrap();let target=&w.actors[&OTHER];
        let mut b=vec![0x23,18];b.extend(OWN.to_le_bytes());b.extend(OTHER.to_le_bytes());
        b.extend([marker,2]);b.extend(77_u32.to_le_bytes());b.extend(129_u16.to_le_bytes());b.push(0);
        for p in [own.x,own.y,target.x,target.y] {b.extend(p.to_le_bytes());}b.extend([0,0]);b
    }
    fn pending(w:&World,now:Instant)->Pending {Pending{since:now,casts:w.own_cast_packets,
        messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation}}
    #[test]
    fn entry_is_unsigned_actor_xy_eleven_bytes_without_local_cost_or_damage() {
        let w=world();let a=&w.actors[&OTHER];let mut expected=vec![20,5,4,0x67,0x45,0x23,0xf1];
        expected.extend(a.x.to_le_bytes());expected.extend(a.y.to_le_bytes());
        let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,45,Some(OTHER)).unwrap(),expected);
        assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(45),100);
        assert_eq!(client_clock_millis(45),0);assert!(!item_targeted(45));
    }
    #[test]
    fn eight_cell_diagonal_is_inclusive_and_ninth_cell_is_rejected() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors.get_mut(&OTHER).unwrap();
        a.x=own.x+8;a.y=own.y+8;assert!(request(&w,45,Some(OTHER)).is_ok());
        w.actors.get_mut(&OTHER).unwrap().x+=1;assert!(request(&w,45,Some(OTHER)).is_err());
    }
    #[test]
    fn absent_self_dead_hidden_or_removed_target_never_becomes_a_ground_cast() {
        let mut w=world();assert!(request(&w,45,None).is_err());assert!(request(&w,45,Some(OWN)).is_err());
        w.actors.get_mut(&OTHER).unwrap().dead=true;assert!(request(&w,45,Some(OTHER)).is_err());
        w.actors.get_mut(&OTHER).unwrap().dead=false;
        w.actors.get_mut(&OTHER).unwrap().draw_state.invisible=true;
        assert!(request(&w,45,Some(OTHER)).is_err());w.actors.remove(&OTHER);
        assert!(request(&w,45,Some(OTHER)).is_err());assert!(ground_request(&w,45,[32768,32768]).is_err());
    }
    #[test]
    fn learned_menu_and_target_choice_use_the_actual_actor_lease() {
        let w=world();let m=menu(&w);assert_eq!(m.rows()[0].label,"#45 地裂術（選取目標，八格內）");
        let t=Target::capture(&w,OTHER).unwrap();let c=m.choice(7,&w,Some(t)).unwrap();
        assert_eq!((c.skill,c.target,c.ground),(45,Some(OTHER),None));
        let mut next=w.clone();next.learned_skills.clear();let mut m=m;m.sync(7,&next,true);
        assert!(m.selected().is_none());assert!(m.rows().is_empty());
    }
    #[test]
    fn stale_target_revision_incarnation_and_death_cannot_reauthorize() {
        let w=world();let m=menu(&w);let t=Target::capture(&w,OTHER);
        let mut next=w.clone();next.actors.get_mut(&OTHER).unwrap().target_revision+=1;
        assert!(m.choice(7,&next,t).is_err());let mut next=w.clone();
        next.actors.get_mut(&OTHER).unwrap().incarnation+=1;assert!(m.choice(7,&next,t).is_err());
        let mut next=w.clone();next.actors.get_mut(&OTHER).unwrap().dead=true;assert!(m.choice(7,&next,t).is_err());
    }
    #[test]
    fn connection_world_correction_and_caster_replacement_fence_the_choice() {
        let w=world();let m=menu(&w);let t=Target::capture(&w,OTHER);assert!(m.choice(8,&w,t).is_err());
        let mut next=w.clone();next.generation+=1;assert!(m.choice(7,&next,t).is_err());
        let mut next=w.clone();next.local_corrections+=1;assert!(m.choice(7,&next,t).is_err());
        let mut next=w.clone();next.actors.get_mut(&OWN).unwrap().incarnation+=1;assert!(m.choice(7,&next,t).is_err());
    }
    #[test]
    fn client_checks_live_control_but_leaves_effective_mp_to_the_server() {
        let mut w=world();assert!(request(&w,45,Some(OTHER)).is_ok()); // MP1 is not predicted MP20.
        w.mp=Some([0,200]);assert!(request(&w,45,Some(OTHER)).is_err());w.mp=Some([1,200]);
        w.accept(&[0x25,2]).unwrap();assert!(request(&w,45,Some(OTHER)).is_err());
        w.accept(&[0x25,3]).unwrap();assert!(request(&w,45,Some(OTHER)).is_ok());
        w.learned_skills.clear();assert!(request(&w,45,Some(OTHER)).is_err());
    }
    #[test]
    fn same_incarnation_position_correction_is_encoded_from_current_world() {
        let mut w=world();let m=menu(&w);let t=Target::capture(&w,OTHER);let before=request(&w,45,Some(OTHER)).unwrap();
        w.actors.get_mut(&OTHER).unwrap().x+=1;
        let c=m.choice(7,&w,t).unwrap();let after=request(&w,c.skill,c.target).unwrap();
        assert_ne!(&before[7..9],&after[7..9]);assert_eq!(&after[7..9],&w.actors[&OTHER].x.to_le_bytes());
    }
    #[test]
    fn S23_cast_receipt_is_not_success_prediction_or_immediate_hurt() {
        let mut w=world();let now=Instant::now();let p=pending(&w,now);let hp=w.hp;let mp=w.mp;
        let target_sequence=w.actors[&OTHER].action_sequence;w.accept(&s23(&w,10)).unwrap();
        assert_eq!(p.observe(&w,now),Feedback::CastAction);assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);
        assert_eq!(w.actors[&OTHER].action_sequence,target_sequence); // original child Hit is later, still pending its guard binding.
        let c=w.projectiles.last().unwrap();assert_eq!((c.projectile.graphics,c.projectile.flight_type),(129,0));
        w.message_count+=1;w.last_skill_rejection=Some((w.message_count,278));
        assert_eq!(p.observe(&w,now),Feedback::Rejected(278));
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; #45 choice->wire->S23->class7 dual-layer memory render; no device or GUI"]
    fn original_shared_resources_connect_the_local_choice_to_received_class7() {
        let mut w=world();let c=menu(&w).choice(7,&w,Target::capture(&w,OTHER)).unwrap();
        assert_eq!(request(&w,c.skill,c.target).unwrap().len(),11);w.accept(&s23(&w,0)).unwrap();
        let cue=w.projectiles.last().unwrap().clone();let root:std::path::PathBuf=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root").into();
        let mut effects=super::super::native_spell_projectiles::Projectiles::default();
        effects.prepare(&root,&w,&std::sync::atomic::AtomicBool::new(false));
        let event=super::super::native_combat_animation::EnteredEvent{actor:OWN,sequence:cue.action_sequence,
            action:18,frame:1,event:"Hit".into(),sound:None,prepared_audio:None};
        assert!(effects.update(&w,[&event].into_iter(),0).is_empty());
        let own=w.own_actor().unwrap();let projection=super::super::world_map::ViewProjection{
            anchor_cell:super::super::world_map::WorldCell{x:i32::from(own.x),y:i32::from(own.y)},
            anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};
        let mut fragments=vec![];let(receipt,audio)=effects.draw_to(&mut super::super::native_world_effects::Target::Fragments(&mut fragments),projection,0);
        assert_eq!(fragments.len(),2);assert!(receipt["eruption129"]["load_error"].is_null());
        assert_eq!(receipt["eruption129"]["casts"][0]["nodes"][0]["graphics"],129);
        assert_eq!(audio.iter().filter(|e|e.sound==Some(128)).count(),1);
    }
}
