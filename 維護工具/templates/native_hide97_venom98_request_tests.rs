
#[cfg(test)]mod hide97_venom98_request_draft_tests {
    use super::*;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
        w.learned_skills.clear();w.learned_skills.extend([97,98]);w.hp=Some([1,100]);w.mp=Some([1,100]);w
    }
    #[test]fn both_selector_only_requests_clear_native_menu_target_without_predicting_status() {
        use super::super::native_skill_selection::{Model,Target};let mut w=world();let mut other=w.own_actor().unwrap().clone();other.id=0x71234567;other.incarnation+=1;other.x+=1;w.actors.insert(other.id,other.clone());let target=Target::capture(&w,other.id).unwrap();w.actors.remove(&other.id);let before=serde_json::to_value(&w).unwrap();let mut m=Model::default();m.sync(7,&w,true);
        for(id,column)in[(97,0),(98,1)]{m.select(Some(id)).unwrap();let intent=m.choice(7,&w,Some(target)).unwrap();assert_eq!(intent.target,None);assert_eq!(request(&w,id,intent.target).unwrap(),vec![20,12,column]);assert!(request(&w,id,Some(other.id)).is_err());}
        assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!((reuse_millis(97),reuse_millis(98)),(9990,0));
    }
    #[test]fn effect_does_not_toggle_hidden_state_or_spend_hp_and_wire34_remains_authoritative() {
        let mut w=world();let own=w.own_actor().unwrap().id;let before=(w.hp,w.mp,w.movement_blocks.clone());let now=Instant::now();let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
        for gfx in [2941_u16,2942]{let mut b=vec![0x37];b.extend(own.to_le_bytes());b.extend(gfx.to_le_bytes());w.accept(&b).unwrap();}
        assert!(!w.own_actor().unwrap().draw_state.invisible);assert_eq!(p.observe(&w,now),Feedback::Pending);assert_eq!((w.hp,w.mp,w.movement_blocks.clone()),before);
        for mode in [1_u8,0]{let mut b=vec![0x34];b.extend(own.to_le_bytes());b.push(mode);w.accept(&b).unwrap();assert_eq!(w.own_actor().unwrap().draw_state.invisible,mode==1);}
    }
}
