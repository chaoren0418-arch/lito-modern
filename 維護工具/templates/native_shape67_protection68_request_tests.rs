
#[cfg(test)] mod shape67_protection68_request_development_tests {
    use super::*;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
        w.learned_skills.clear();w.learned_skills.extend([67,68]);w.hp=Some([1,100]);w.mp=Some([1,100]);
        let mut other=w.own_actor().unwrap().clone();other.id=0x71234567;other.incarnation+=1;other.x+=1;w.actors.insert(other.id,other);w
    }
    fn wire(skill:u16,id:u32)->Vec<u8> {let mut b=vec![20,8,(skill-65) as u8];b.extend(id.to_le_bytes());b}
    #[test] fn self_and_other_requests_are_seven_bytes_and_never_spend_or_infer_eligibility() {
        let w=world();let own=w.own_actor().unwrap().id;let before=serde_json::to_value(&w).unwrap();
        for skill in [67,68] {assert_eq!(request(&w,skill,None).unwrap(),wire(skill,own));assert_eq!(request(&w,skill,Some(own)).unwrap(),wire(skill,own));assert_eq!(request(&w,skill,Some(0x71234567)).unwrap(),wire(skill,0x71234567));}
        assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(67),10);assert_eq!(reuse_millis(68),1500);
    }
    #[test] fn screen_edge_stale_and_dead_targets_do_not_fall_back_to_self() {
        for skill in [67,68] {let mut w=world();let a=w.own_actor().unwrap().clone();w.actors.get_mut(&0x71234567).unwrap().x=a.x+10;w.actors.get_mut(&0x71234567).unwrap().y=a.y+9;
            assert_eq!(request(&w,skill,Some(0x71234567)).is_ok(),in_native_screen([a.x,a.y],[a.x+10,a.y+9]));w.actors.get_mut(&0x71234567).unwrap().y=a.y+10;assert!(request(&w,skill,Some(0x71234567)).is_err());
            w.actors.get_mut(&0x71234567).unwrap().x=a.x+1;w.actors.get_mut(&0x71234567).unwrap().y=a.y;w.actors.get_mut(&0x71234567).unwrap().dead=true;assert!(request(&w,skill,Some(0x71234567)).is_err());w.actors.remove(&0x71234567);assert!(request(&w,skill,Some(0x71234567)).is_err());}
    }
    #[test] fn missing_book_zero_mp_and_dead_caster_are_rejected_before_packet() {
        for skill in [67,68] {let mut w=world();w.learned_skills.remove(&skill);assert!(request(&w,skill,None).is_err());w.learned_skills.insert(skill);w.mp=Some([0,100]);assert!(request(&w,skill,None).is_err());w.mp=Some([1,100]);let id=w.own_actor().unwrap().id;w.actors.get_mut(&id).unwrap().dead=true;assert!(request(&w,skill,None).is_err());}
    }
    #[test] fn learned_native_rows_select_both_intents_and_preserve_actor_lease() {
        use super::super::native_skill_selection::{Model,Target};let mut w=world();let mut m=Model::default();m.sync(7,&w,true);
        for skill in [67,68] {m.select(Some(skill)).unwrap();let choice=m.choice(7,&w,None).unwrap();assert_eq!((choice.skill,choice.target),(skill,None));let t=Target::capture(&w,0x71234567).unwrap();assert_eq!(m.choice(7,&w,Some(t)).unwrap().target,Some(0x71234567));w.actors.get_mut(&0x71234567).unwrap().target_revision+=1;assert!(m.choice(7,&w,Some(t)).is_err());}
    }
}
