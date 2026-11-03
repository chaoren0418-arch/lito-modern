
#[cfg(test)]
mod berserk55_disease56_request_development_tests {
    use super::*;
    use super::super::native_skill_selection::{Model,Target};
    const OTHER:u32=0xf1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();let b:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}w.learned_skills.clear();w.learned_skills.extend([55,56]);w.hp=Some([1,200]);w.mp=Some([1,200]);let mut a=w.own_actor().unwrap().clone();a.id=OTHER;a.incarnation+=9;a.x+=1;a.heading=6;w.actors.insert(OTHER,a);w
    }
    fn effect(w:&mut World,id:u32,gfx:u16){let mut b=vec![0x37];b.extend(id.to_le_bytes());b.extend(gfx.to_le_bytes());w.accept(&b).unwrap();}
    #[test]
    fn berserk_object_envelope_keeps_self_and_other_ids_without_stat_or_cost_prediction() {
        let w=world();let own=w.own_actor().unwrap().id;let before=serde_json::to_value(&w).unwrap();for id in [None,Some(own),Some(OTHER)]{let mut b=vec![20,6,6];b.extend(id.unwrap_or(own).to_le_bytes());assert_eq!(request(&w,55,id).unwrap(),b);}assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(reuse_millis(55),10);assert_eq!(client_clock_millis(55),0);
    }
    #[test]
    fn disease_actor_xy_is_three_cells_and_rejects_self_or_no_target() {
        let mut w=world();let own=w.own_actor().unwrap().clone();let a=w.actors.get_mut(&OTHER).unwrap();a.x=own.x+3;a.y=own.y+3;let a=&w.actors[&OTHER];let mut b=vec![20,6,7];b.extend(OTHER.to_le_bytes());b.extend(a.x.to_le_bytes());b.extend(a.y.to_le_bytes());assert_eq!(request(&w,56,Some(OTHER)).unwrap(),b);assert!(request(&w,56,None).is_err());assert!(request(&w,56,Some(own.id)).is_err());w.actors.get_mut(&OTHER).unwrap().x+=1;assert!(request(&w,56,Some(OTHER)).is_err());assert_eq!(reuse_millis(56),500);
    }
    #[test]
    fn berserk_negative_range_uses_native_asymmetric_screen_and_stale_target_never_falls_back() {
        let mut w=world();let own=w.own_actor().unwrap().clone();w.actors.get_mut(&OTHER).unwrap().x=own.x-19;assert!(request(&w,55,Some(OTHER)).is_ok());w.actors.get_mut(&OTHER).unwrap().x=own.x+19;assert!(request(&w,55,Some(OTHER)).is_err());w.actors.remove(&OTHER);assert!(request(&w,55,Some(OTHER)).is_err());assert!(request(&w,55,None).is_ok());
    }
    #[test]
    fn explicit_target_and_skill_owner_are_revoked_by_aba_and_transfer() {
        for skill in [55,56]{let w=world();let mut m=Model::default();m.sync(7,&w,true);m.select(Some(skill)).unwrap();let t=Target::capture(&w,OTHER);assert_eq!(m.choice(7,&w,t).unwrap().target,Some(OTHER));assert!(m.choice(8,&w,t).is_err());for kind in 0..4{let mut n=w.clone();match kind{0=>n.generation+=1,1=>n.local_corrections+=1,2=>n.actors.get_mut(&OTHER).unwrap().incarnation+=1,_=>n.actors.get_mut(&OTHER).unwrap().target_revision+=1};assert!(m.choice(7,&n,t).is_err());}}
    }
    #[test]
    fn live_book_mp_and_control_are_required_without_guessing_clan_material_or_probability() {
        for skill in [55,56]{let mut w=world();assert!(request(&w,skill,Some(OTHER)).is_ok());w.mp=Some([0,200]);assert!(request(&w,skill,Some(OTHER)).is_err());w.mp=Some([1,200]);w.actors.get_mut(&OTHER).unwrap().dead=true;assert!(request(&w,skill,Some(OTHER)).is_err());w.actors.get_mut(&OTHER).unwrap().dead=false;w.accept(&[0x25,2]).unwrap();assert!(request(&w,skill,Some(OTHER)).is_err());w.accept(&[0x25,3]).unwrap();w.learned_skills.remove(&skill);assert!(request(&w,skill,Some(OTHER)).is_err());}
    }
    #[test]
    fn S37_snapshots_berserk_heading_and_position_without_following_actor_or_predicting_status() {
        let mut w=world();let a=w.actors[&OTHER].clone();let hp=w.hp;let mp=w.mp;effect(&mut w,OTHER,3943);let e=w.effects.last().unwrap().clone();assert_eq!(w.finite_effect_headings.get(&e.sequence),Some(&6));w.actors.get_mut(&OTHER).unwrap().heading=2;w.actors.get_mut(&OTHER).unwrap().x+=3;w.actors.remove(&OTHER);assert_eq!(w.finite_effect_headings.get(&e.sequence),Some(&6));assert_eq!(e.position,[a.x,a.y]);assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);assert!(w.movement_blocks.is_empty());
    }
    #[test]
    fn heading_provenance_is_bounded_with_effect_ring_and_cleared_on_generation() {
        let mut w=world();for _ in 0..130{effect(&mut w,OTHER,3943);}assert_eq!(w.effects.len(),128);assert_eq!(w.finite_effect_headings.len(),128);assert_eq!(w.finite_effect_headings.keys().next().copied(),Some(w.effects[0].sequence));let before=w.finite_effect_headings.len();effect(&mut w,0x71234567,3943);assert_eq!(w.finite_effect_headings.len(),before);w.clear_preserving_generation();assert!(w.finite_effect_headings.is_empty());
    }
    #[test]
    fn effect_receipts_do_not_ack_berserk_or_disease_status_and_rejection_has_priority() {
        let mut w=world();let now=Instant::now();let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};for gfx in [3943,2230]{effect(&mut w,OTHER,gfx);}assert_eq!(p.observe(&w,now),Feedback::Pending);assert_eq!(p.observe(&w,now+Duration::from_secs(2)),Feedback::NoResponse);w.message_count+=1;w.last_skill_rejection=Some((w.message_count,280));assert_eq!(p.observe(&w,now),Feedback::Rejected(280));
    }
}
