#[cfg(test)]
mod blizzard59_request_development_tests {
    use super::*;
    fn world()->World {let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}w.learned_skills.clear();w.learned_skills.insert(59);w.mp=Some([1,200]);w.hp=Some([1,200]);w}
    #[test]
    fn selector_is_exactly_three_bytes_and_never_predicts_hp_mp_or_targets() {let w=world();let before=serde_json::to_value(&w).unwrap();assert_eq!(request(&w,59,None).unwrap(),vec![20,7,2]);assert!(request(&w,59,Some(w.own_actor().unwrap().id)).is_err());assert_eq!(reuse_millis(59),1500);assert_eq!(serde_json::to_value(&w).unwrap(),before);}
    #[test]
    fn menu_ignores_old_target_but_keeps_skill_owner_fences() {let mut w=world();let mut m=super::super::native_skill_selection::Model::default();m.sync(7,&w,true);m.select(Some(59)).unwrap();let t=super::super::native_skill_selection::Target::capture(&w,w.own_actor().unwrap().id);assert_eq!(m.choice(7,&w,t).unwrap().target,None);assert!(m.choice(8,&w,t).is_err());w.generation+=1;assert!(m.choice(7,&w,t).is_err());}
    #[test]
    fn book_zero_mp_and_control_gate_precede_selector_serialization() {let mut w=world();w.learned_skills.clear();assert!(request(&w,59,None).is_err());w.learned_skills.insert(59);w.mp=Some([0,200]);assert!(request(&w,59,None).is_err());w.mp=Some([1,200]);w.accept(&[0x25,2]).unwrap();assert!(request(&w,59,None).is_err());}
    #[test]
    fn effect_never_acknowledges_hp_cost_or_damage_and_rejection_wins() {let mut w=world();let now=Instant::now();let p=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};assert_eq!(p.observe(&w,now),Feedback::Pending);assert_eq!(p.observe(&w,now+Duration::from_secs(2)),Feedback::NoResponse);w.message_count+=1;w.last_skill_rejection=Some((w.message_count,278));assert_eq!(p.observe(&w,now),Feedback::Rejected(278));}
}
