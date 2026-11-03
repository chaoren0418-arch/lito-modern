
#[cfg(test)]mod solid90_counter91_request_development_tests {
    use super::*;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
        w.learned_skills.clear();w.learned_skills.extend([90,91]);let own=w.own_actor().unwrap().id;w.actors.get_mut(&own).unwrap().walk_action=50;w.hp=Some([1,200]);w.mp=Some([1,100]);w
    }
    #[test]fn selector_only_requests_preserve_authority_even_below_formal_hp_cost() {
        let w=world();let before=serde_json::to_value(&w).unwrap();
        assert_eq!(request(&w,90,None).unwrap(),vec![20,11,1]);assert_eq!(request(&w,91,None).unwrap(),vec![20,11,2]);
        assert_eq!(serde_json::to_value(&w).unwrap(),before);
        assert_eq!((reuse_millis(90),reuse_millis(91)),(3000,3000));assert_eq!((client_clock_millis(90),client_clock_millis(91)),(0,0));
    }
    #[test]fn supplied_target_and_invalid_local_caster_are_rejected() {
        for id in [90,91]{let mut w=world();assert!(request(&w,id,Some(w.own_actor().unwrap().id)).is_err());assert!(request(&w,id,Some(u32::MAX)).is_err());
            w.learned_skills.remove(&id);assert!(request(&w,id,None).is_err());w.learned_skills.insert(id);w.mp=Some([0,100]);assert!(request(&w,id,None).is_err());
            w.mp=Some([1,100]);w.movement_blocks.insert(1);assert!(request(&w,id,None).is_err());w.movement_blocks.clear();w.hp=Some([0,200]);assert!(request(&w,id,None).is_err());}
    }
    #[test]fn actual_native_menu_discards_stale_target_for_self_only_skills() {
        use super::super::native_skill_selection::{Model,Target};let mut w=world();let mut a=w.own_actor().unwrap().clone();a.id=0x71234567;a.incarnation+=1;a.x+=1;w.actors.insert(a.id,a.clone());let stale=Target::capture(&w,a.id).unwrap();w.actors.remove(&a.id);
        let mut m=Model::default();m.sync(7,&w,true);
        for(id,name)in[(90,"堅固防護"),(91,"反擊屏障")]{assert!(m.rows().iter().any(|r|r.id==id&&r.label.contains(name)));m.select(Some(id)).unwrap();assert_eq!(m.choice(7,&w,Some(stale)).unwrap().target,None);}
    }
    #[test]fn real_279_packet_is_a_skill_rejection_and_does_not_spend_hp() {
        let mut w=world();let before=(w.hp,w.mp);let now=Instant::now();let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
        w.accept(&[0x10,0x17,0x01,0]).unwrap();assert_eq!(p.observe(&w,now),Feedback::Rejected(279));assert_eq!(Feedback::Rejected(279).label(),"HP 不足");assert_eq!((w.hp,w.mp),before);
    }
    #[test]fn effect_is_not_hp_mp_or_status_ack_and_authoritative_hp_mp_still_apply() {
        let mut w=world();let before=(w.hp,w.mp,w.movement_blocks.clone());let now=Instant::now();let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
        for gfx in [5831_u16,5832,10710]{let mut b=vec![0x37];b.extend(w.own_actor().unwrap().id.to_le_bytes());b.extend(gfx.to_le_bytes());w.accept(&b).unwrap();}
        assert_eq!(p.observe(&w,now),Feedback::Pending);assert_eq!((w.hp,w.mp,w.movement_blocks.clone()),before);
        w.accept(&[0x0d,50,0,200,0]).unwrap();w.accept(&[0x4d,20,0,100,0]).unwrap();assert_eq!(w.hp,Some([50,200]));assert_eq!(w.mp,Some([20,100]));
    }
    #[test]fn counter_requires_exact_walk50_and_does_not_claim_actual_weapon_authority() {
        for walk in [0,4,46,50,54]{let mut w=world();let own=w.own_actor().unwrap().id;w.actors.get_mut(&own).unwrap().walk_action=walk;assert_eq!(request(&w,91,None).is_ok(),walk==50);assert!(request(&w,90,None).is_ok());}
    }

}
