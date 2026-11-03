
#[cfg(test)]
mod freeze50_request_development_tests {
    use super::*;
    use super::super::native_skill_selection::{Model,Target};
    const OTHER:u32=0xf1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();let b:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
        w.learned_skills.clear();w.learned_skills.insert(50);w.hp=Some([1,200]);w.mp=Some([1,200]);let mut a=w.own_actor().unwrap().clone();a.id=OTHER;a.incarnation+=9;a.x+=1;w.actors.insert(OTHER,a);w
    }
    fn menu(w:&World)->Model {let mut m=Model::default();m.sync(7,w,true);m.select(Some(50)).unwrap();m}
    #[test]
    fn freeze50_sends_actor_xy_without_predicting_cost_or_server_freeze() {
        let w=world();let a=&w.actors[&OTHER];let mut b=vec![20,6,1,0x67,0x45,0x23,0xf1];b.extend(a.x.to_le_bytes());b.extend(a.y.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,50,Some(OTHER)).unwrap(),b);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(50),500);assert_eq!(client_clock_millis(50),1000);assert!(!item_targeted(50));
    }
    #[test]
    fn six_cell_diagonal_boundary_rejects_seven_self_absent_dead_and_hidden() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors.get_mut(&OTHER).unwrap();a.x=own.x+6;a.y=own.y+6;assert!(request(&w,50,Some(OTHER)).is_ok());w.actors.get_mut(&OTHER).unwrap().y+=1;assert!(request(&w,50,Some(OTHER)).is_err());w.actors.get_mut(&OTHER).unwrap().y-=1;
        assert!(request(&w,50,None).is_err());assert!(request(&w,50,Some(own.id)).is_err());w.actors.get_mut(&OTHER).unwrap().dead=true;assert!(request(&w,50,Some(OTHER)).is_err());w.actors.get_mut(&OTHER).unwrap().dead=false;w.actors.get_mut(&OTHER).unwrap().draw_state.invisible=true;assert!(request(&w,50,Some(OTHER)).is_err());
    }
    #[test]
    fn menu_keeps_owner_and_target_lease_fences_across_transfer_and_aba() {
        let w=world();let m=menu(&w);assert_eq!(m.rows().iter().find(|r|r.id==50).unwrap().label,"#50 冰矛圍籬（選取目標，六格內）");let t=Target::capture(&w,OTHER);assert_eq!(m.choice(7,&w,t).unwrap().target,Some(OTHER));assert!(m.choice(8,&w,t).is_err());
        for kind in 0..5 {let mut n=w.clone();match kind{0=>n.generation+=1,1=>n.local_corrections+=1,2=>n.actors.get_mut(&OTHER).unwrap().incarnation+=1,3=>n.actors.get_mut(&OTHER).unwrap().target_revision+=1,_=>{let id=n.own_actor().unwrap().id;n.actors.get_mut(&id).unwrap().incarnation+=1;}}assert!(m.choice(7,&n,t).is_err());}
    }
    #[test]
    fn book_mp_and_server_control_guard_without_local_mp100_gate() {
        let mut w=world();assert!(request(&w,50,Some(OTHER)).is_ok());w.mp=Some([0,200]);assert!(request(&w,50,Some(OTHER)).is_err());w.mp=Some([1,200]);w.accept(&[0x25,2]).unwrap();assert!(request(&w,50,Some(OTHER)).is_err());w.accept(&[0x25,3]).unwrap();w.learned_skills.clear();assert!(request(&w,50,Some(OTHER)).is_err());
    }
    #[test]
    fn S23_flight_does_not_freeze_hurt_or_spend_and_only_paralysis_packet_blocks() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors[&OTHER].clone();let hp=w.hp;let mp=w.mp;
        let mut b=vec![0x23,18];b.extend(own.id.to_le_bytes());b.extend(OTHER.to_le_bytes());b.extend([10,2]);b.extend(77_u32.to_le_bytes());b.extend(756_u16.to_le_bytes());b.push(0);for v in [own.x,own.y,a.x,a.y]{b.extend(v.to_le_bytes());}b.extend([0,0]);w.accept(&b).unwrap();assert!(super::super::native_spell_projectiles::supports_graphics(756));assert_eq!(w.actors[&OTHER].action_sequence,a.action_sequence);assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);assert!(w.movement_blocks.is_empty());
        w.accept(&[0x25,2]).unwrap();assert!(!w.movement_blocks.is_empty());w.accept(&[0x25,3]).unwrap();assert!(w.movement_blocks.is_empty());
    }
    #[test]
    fn no_response_cast_action_and_rejection_never_mean_freeze_success() {
        let mut w=world();let now=Instant::now();let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};assert_eq!(p.observe(&w,now+Duration::from_secs(2)),Feedback::NoResponse);
        let mut b=vec![0x20];b.extend(w.own_actor().unwrap().id.to_le_bytes());b.push(18);w.accept(&b).unwrap();assert_eq!(p.observe(&w,now),Feedback::CastAction);assert!(w.movement_blocks.is_empty());w.message_count+=1;w.last_skill_rejection=Some((w.message_count,280));assert_eq!(p.observe(&w,now),Feedback::Rejected(280));
    }
}
