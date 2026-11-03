
#[cfg(test)]
mod curse33_drain39_development_tests {
    use super::*;
    use super::super::native_skill_selection::{Model,Target};
    const OWN:u32=0xfedcba98;const OTHER:u32=0xf1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {
            let h=f["packets"][key].as_str().unwrap();let bytes:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&bytes).unwrap();
        }
        w.learned_skills.clear();w.learned_skills.extend([33,39]);w.hp=Some([1,200]);w.mp=Some([1,200]);
        let mut a=w.own_actor().unwrap().clone();a.id=OTHER;a.x+=1;a.incarnation+=10;w.actors.insert(OTHER,a);w
    }
    fn wire(w:&World,skill:u16,id:u32)->Vec<u8> {
        let a=&w.actors[&id];let mut b=vec![20,4,if skill==33{0}else{6}];b.extend(id.to_le_bytes());b.extend(a.x.to_le_bytes());b.extend(a.y.to_le_bytes());b
    }
    fn menu(w:&World,skill:u16)->Model {let mut m=Model::default();m.sync(7,w,true);m.select(Some(skill)).unwrap();m}
    fn pending(w:&World,now:Instant)->Pending {Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation}}

    #[test]
    fn curse33_requires_other_actor_at_two_cell_diagonal_and_preserves_authority() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors.get_mut(&OTHER).unwrap();a.x=own.x+2;a.y=own.y+2;
        let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,33,Some(OTHER)).unwrap(),wire(&w,33,OTHER));assert_eq!(serde_json::to_value(&w).unwrap(),before);
        assert!(request(&w,33,None).is_err());assert!(request(&w,33,Some(OWN)).is_err());w.actors.get_mut(&OTHER).unwrap().y+=1;assert!(request(&w,33,Some(OTHER)).is_err());
        assert_eq!(reuse_millis(33),5000);assert_eq!(client_clock_millis(33),5000);
    }
    #[test]
    fn drain39_unselected_and_explicit_self_send_actor_xy_even_with_one_hp() {
        let w=world();let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,39,None).unwrap(),wire(&w,39,OWN));assert_eq!(request(&w,39,Some(OWN)).unwrap(),wire(&w,39,OWN));assert_eq!(serde_json::to_value(&w).unwrap(),before);
        assert_eq!(reuse_millis(39),500);assert_eq!(client_clock_millis(39),0);
    }
    #[test]
    fn drain39_other_actor_is_four_cells_and_never_falls_back_after_disappearance() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors.get_mut(&OTHER).unwrap();a.x=own.x+4;a.y=own.y+4;
        assert_eq!(request(&w,39,Some(OTHER)).unwrap(),wire(&w,39,OTHER));w.actors.get_mut(&OTHER).unwrap().x+=1;assert!(request(&w,39,Some(OTHER)).is_err());w.actors.remove(&OTHER);assert!(request(&w,39,Some(OTHER)).is_err());
    }
    #[test]
    fn curse_and_drain_reject_dead_target_and_missing_book_or_server_control() {
        for skill in [33,39] {
            let mut w=world();w.actors.get_mut(&OTHER).unwrap().dead=true;assert!(request(&w,skill,Some(OTHER)).is_err());
            let mut w=world();w.learned_skills.remove(&skill);assert!(request(&w,skill,Some(OTHER)).is_err());
            let mut w=world();w.accept(&[0x25,2]).unwrap();assert!(request(&w,skill,Some(OTHER)).is_err());w.accept(&[0x25,3]).unwrap();assert!(request(&w,skill,Some(OTHER)).is_ok());
        }
    }
    #[test]
    fn curse_and_drain_menu_keep_labels_and_distinct_self_policy() {
        let w=world();let c=menu(&w,33);assert_eq!(c.rows().iter().find(|r|r.id==33).unwrap().label,"#33 木乃伊的詛咒（選取目標，兩格內；消耗 2 顆魔法寶石）");assert!(c.choice(7,&w,None).is_err());
        let d=menu(&w,39);assert_eq!(d.rows().iter().find(|r|r.id==39).unwrap().label,"#39 魔力奪取（選取目標／自己，四格內）");assert_eq!(d.choice(7,&w,None).unwrap().target,None);assert_eq!(d.choice(7,&w,Target::capture(&w,OWN)).unwrap().target,Some(OWN));
    }
    #[test]
    fn curse_and_drain_choice_reject_target_revision_or_incarnation_aba() {
        for skill in [33,39] {let w=world();let m=menu(&w,skill);let t=Target::capture(&w,OTHER).unwrap();assert_eq!(m.choice(7,&w,Some(t)).unwrap().target,Some(OTHER));
            let mut next=w.clone();next.actors.get_mut(&OTHER).unwrap().target_revision+=1;assert!(m.choice(7,&next,Some(t)).is_err());
            let mut next=w.clone();next.actors.get_mut(&OTHER).unwrap().incarnation+=1;assert!(m.choice(7,&next,Some(t)).is_err());
        }
    }
    #[test]
    fn curse_and_drain_choice_reject_connection_correction_generation_and_caster_replacement() {
        for skill in [33,39] {let w=world();let m=menu(&w,skill);let t=Target::capture(&w,OTHER);assert!(m.choice(8,&w,t).is_err());
            let mut next=w.clone();next.local_corrections+=1;assert!(m.choice(7,&next,t).is_err());
            let mut next=w.clone();next.generation+=1;assert!(m.choice(7,&next,t).is_err());
            let mut next=w.clone();next.actors.get_mut(&OWN).unwrap().incarnation+=1;assert!(m.choice(7,&next,t).is_err());
        }
    }
    #[test]
    fn curse_action_and_finite_effect_are_not_predicted_paralysis_or_resource_spending() {
        let mut w=world();let now=Instant::now();let p=pending(&w,now);let before_hp=w.hp;let before_mp=w.mp;let mut action=vec![0x20];action.extend(OWN.to_le_bytes());action.push(19);w.accept(&action).unwrap();assert_eq!(p.observe(&w,now),Feedback::CastAction);
        for graphics in [746_u16,2171] {let mut b=vec![0x37];b.extend(OTHER.to_le_bytes());b.extend(graphics.to_le_bytes());w.accept(&b).unwrap();let e=w.effects.last().unwrap();assert_eq!(e.target,Some((OTHER,w.actors[&OTHER].incarnation)));assert_eq!(e.graphics,graphics);}
        assert!(w.movement_blocks.is_empty());assert_eq!(w.hp,before_hp);assert_eq!(w.mp,before_mp);
    }
    #[test]
    fn drain_silent_hp_rejection_stays_no_response_and_real_rejection_precedes_action() {
        let mut w=world();let now=Instant::now();let p=pending(&w,now);request(&w,39,None).unwrap();assert_eq!(p.observe(&w,now+Duration::from_millis(1999)),Feedback::Pending);assert_eq!(p.observe(&w,now+Duration::from_secs(2)),Feedback::NoResponse);
        w.message_count+=1;w.last_skill_rejection=Some((w.message_count,280));w.own_cast_packets+=1;assert_eq!(p.observe(&w,now),Feedback::Rejected(280));w.generation+=1;assert_eq!(p.observe(&w,now),Feedback::Interrupted);
    }
    #[test]
    fn malformed_or_unknown_target_effect_never_creates_actor_or_curse_control() {
        let mut w=world();let mut b=vec![0x37];b.extend(OTHER.to_le_bytes());b.extend(746_u16.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert!(w.accept(&b[..b.len()-1]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
        w.actors.remove(&OTHER);let n=w.effects.len();w.accept(&b).unwrap();assert_eq!(w.effects.len(),n);assert!(!w.actors.contains_key(&OTHER));assert!(w.movement_blocks.is_empty());
    }
}
