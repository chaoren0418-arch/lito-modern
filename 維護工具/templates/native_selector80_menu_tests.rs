
#[cfg(test)]mod selector80_menu_development_tests {
    use super::*;
    const OTHER:u32=0x71234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
        w.learned_skills.clear();w.learned_skills.extend([46,80]);w.hp=Some([1,100]);w.mp=Some([1,100]);let mut a=w.own_actor().unwrap().clone();a.id=OTHER;a.incarnation+=1;a.x+=1;w.actors.insert(OTHER,a);w
    }
    #[test]fn actual80_menu_clears_live_and_expired_target_without_spending_authority() {
        let mut w=world();let selected=Target::capture(&w,OTHER).unwrap();let mut m=Model::default();m.sync(7,&w,true);m.select(Some(80)).unwrap();
        for expired in [false,true]{if expired{w.actors.remove(&OTHER);}let before=serde_json::to_value(&w).unwrap();let intent=m.choice(7,&w,Some(selected)).unwrap();assert_eq!(intent.skill,80);assert_eq!(intent.target,None);assert_eq!(native_skills::request(&w,intent.skill,intent.target).unwrap(),vec![20,9,7]);assert_eq!(serde_json::to_value(&w).unwrap(),before);}
        assert!(native_skills::request(&w,80,Some(OTHER)).is_err());assert_eq!((native_skills::reuse_millis(80),native_skills::client_clock_millis(80)),(3000,0));
    }
    #[test]fn targeted_spell_and_stale_menu_owner_keep_their_original_rejection_guards() {
        let mut w=world();let selected=Target::capture(&w,OTHER).unwrap();let mut m=Model::default();m.sync(7,&w,true);m.select(Some(46)).unwrap();assert_eq!(m.choice(7,&w,Some(selected)).unwrap().target,Some(OTHER));
        w.actors.get_mut(&OTHER).unwrap().target_revision+=1;assert!(m.choice(7,&w,Some(selected)).is_err());
        m.select(Some(80)).unwrap();w.generation+=1;assert!(m.choice(7,&w,Some(selected)).is_err());m.sync(7,&w,true);m.select(Some(80)).unwrap();w.learned_skills.remove(&80);assert!(m.choice(7,&w,Some(selected)).is_err());
    }
}
