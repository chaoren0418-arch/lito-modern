
#[cfg(test)]mod mass69_entry_development_tests{
    use super::*;
    #[test]fn mass_entry_is_distinct_from_actor_ground_item_and_summon_domains(){assert!(mass_targeted(69));for id in [51,58,63,67,68,70,73]{assert!(!mass_targeted(id));}assert!(!item_targeted(69));assert!(!summon_targeted(69));assert_eq!(reuse_millis(69),10);}
    #[test]fn generic_actor_request_cannot_silently_choose_random_destination(){let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}w.learned_skills.insert(69);w.mp=Some([1,100]);w.hp=Some([1,100]);assert!(request(&w,69,None).is_err());assert!(request(&w,69,Some(w.own_actor().unwrap().id)).is_err());}
}
