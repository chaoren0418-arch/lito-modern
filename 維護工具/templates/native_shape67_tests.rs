
#[cfg(test)]
mod shape67_development_tests {
    use super::*;
    fn snapshot()->Snapshot {
        let mut s=super::super::native_login::LoginController::default().snapshot().clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();s.world.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
        s.phase=LoginPhase::BattlePending;s.game_connected=true;s.selected_source=Some("isolated".into());s.selected_character=Some("fixture".into());
        s.inventory.owner=Some(native_inventory::Owner{connection:7,selection:1,character:"fixture".into()});
        s.world.hp=Some([1,100]);s.world.learned_skills.clear();s.world.mp=Some([0,100]);s
    }
    fn dialog(id:u32,html:&str)->Vec<u8> {let mut b=vec![42];b.extend(id.to_le_bytes());b.extend(html.as_bytes());b.extend([0,0,0,0]);b}
    fn grant(q:&mut Session,s:&Snapshot)->Token {let b=dialog(s.world.own_actor().unwrap().id,"monlist");assert!(q.receive(s,7,&b).unwrap());q.current(s,7).unwrap().token()}
    #[test] fn recipient_needs_no_polymorph_book_mp_cast_or_ring_inference() {
        let s=snapshot();let mut q=Session::default();let t=grant(&mut q,&s);let before=serde_json::to_value(&s.world).unwrap();
        let b=q.answer(&s,7,&t,1).unwrap();let mut expected=vec![39];expected.extend(s.world.own_actor().unwrap().id.to_le_bytes());expected.extend(b"re bugbear\0\0");assert_eq!(b,expected);
        assert!(q.answer(&s,7,&t,1).is_err());assert_eq!(serde_json::to_value(&s.world).unwrap(),before);
    }
    #[test] fn catalog_exposes_exact_standard_actions_and_explicit_remove_only() {
        assert_eq!(FORMS.len(),96);let mut seen=std::collections::BTreeSet::new();
        for (i,f) in FORMS.iter().enumerate() {assert!(f.action.is_ascii()&&!f.action.as_bytes().contains(&0)&&f.action.trim()==f.action);assert!(seen.insert(f.action));assert!((1..=80).contains(&f.minimum_level));let s=snapshot();let mut q=Session::default();let t=grant(&mut q,&s);let b=q.answer(&s,7,&t,i+1).unwrap();assert_eq!(&b[5..b.len()-2],f.action.as_bytes());assert_eq!(&b[b.len()-2..],&[0,0]);}
        let s=snapshot();let mut q=Session::default();let t=grant(&mut q,&s);let b=q.answer(&s,7,&t,0).unwrap();assert_eq!(&b[5..],b"none\0\0");
    }
    #[test] fn local_dismiss_and_bad_selection_never_implicitly_send_none() {
        let s=snapshot();let mut q=Session::default();let t=grant(&mut q,&s);assert!(q.answer(&s,7,&t,97).is_err());assert!(q.current(&s,7).is_some());q.dismiss(&s,7,&t).unwrap();assert!(q.current(&s,7).is_none());assert!(q.answer(&s,7,&t,0).is_err());
    }
    #[test] fn replacement_tokens_and_all_three_native_command_ids_are_distinct() {
        let s=snapshot();let mut q=Session::default();let old=grant(&mut q,&s);let new=grant(&mut q,&s);
        for role in [Control::List,Control::Apply,Control::Close] {assert_ne!(old.command_id(role),new.command_id(role));}
        assert!(q.answer(&s,7,&old,1).is_err());assert!(q.dismiss(&s,7,&old).is_err());assert!(q.answer(&s,7,&new,1).is_ok());
    }
    #[test] fn malformed_open_close_or_tail_cannot_replace_an_existing_grant() {
        let s=snapshot();let mut q=Session::default();let t=grant(&mut q,&s);let own=s.world.own_actor().unwrap().id;
        let b=dialog(own,"monlist");for n in 1..b.len() {assert!(q.receive(&s,7,&b[..n]).is_err());assert_eq!(q.current(&s,7).unwrap().token(),t);}
        for html in ["","monlist"] {let mut b=dialog(own,html);b.push(0);assert!(q.receive(&s,7,&b).is_err());assert_eq!(q.current(&s,7).unwrap().token(),t);}
    }
    #[test] fn other_actor_monlist_is_consumed_without_stealing_own_grant() {
        let s=snapshot();let mut q=Session::default();let t=grant(&mut q,&s);let other=s.world.own_actor().unwrap().id^0x4000_0000;
        assert!(q.receive(&s,7,&dialog(other,"monlist")).unwrap());assert_eq!(q.current(&s,7).unwrap().token(),t);assert!(!q.receive(&s,7,&dialog(other,"")).unwrap());assert_eq!(q.current(&s,7).unwrap().token(),t);
        assert!(q.receive(&s,7,&dialog(s.world.own_actor().unwrap().id,"")).unwrap());assert!(q.current(&s,7).is_none());
    }
    #[test] fn owner_epoch_source_map_life_appearance_and_observer_boundaries_revoke() {
        let s=snapshot();let own=s.world.own_actor().unwrap().id;let mut q=Session::default();let t=grant(&mut q,&s);assert!(q.answer(&s,8,&t,1).is_err());
        for n in 0..13 {let mut altered=s.clone();match n {
            0=>altered.world.generation+=1,1=>altered.world.map_id=Some(altered.world.map_id.unwrap()+1),2=>altered.selected_source=Some("other".into()),
            3=>altered.inventory.owner.as_mut().unwrap().selection+=1,4=>altered.world.actors.get_mut(&own).unwrap().incarnation+=1,
            5=>altered.world.actors.get_mut(&own).unwrap().target_revision+=1,6=>altered.world.control_interruptions+=1,
            7=>altered.phase=LoginPhase::WorldReady,8=>altered.game_connected=false,9=>altered.observer=true,
            10=>altered.world.hp=Some([0,100]),11=>altered.world.actors.get_mut(&own).unwrap().dead=true,_=>altered.inventory.owner=None,
        }assert!(q.answer(&altered,7,&t,1).is_err());assert!(q.dismiss(&altered,7,&t).is_err());}
        // Ordinary book/inventory deltas are not a different character owner.
        let mut ordinary=s.clone();ordinary.inventory.revision+=1;ordinary.world.learned_skills.insert(67);
        ordinary.world.actors.get_mut(&own).unwrap().action_sequence+=1;assert_eq!(q.current(&ordinary,7).unwrap().token(),t);
    }
    #[test] fn retirement_and_command_id_exhaustion_never_reuse_old_authority() {
        let s=snapshot();let mut q=Session::default();let t=grant(&mut q,&s);q.retire();assert!(q.answer(&s,7,&t,1).is_err());let t2=grant(&mut q,&s);assert_ne!(t,t2);
        q.serial=19_999;let last=grant(&mut q,&s);assert_eq!(last.command_id(Control::Close),61_002);
        assert!(q.receive(&s,7,&dialog(s.world.own_actor().unwrap().id,"monlist")).is_err());assert!(q.current(&s,7).is_none());assert!(q.answer(&s,7,&last,1).is_err());
    }
    #[test] fn synchronize_retires_lost_owner_and_own_other_dialog_replaces_menu() {
        let s=snapshot();let mut q=Session::default();let t=grant(&mut q,&s);let mut gone=s.clone();gone.game_connected=false;q.synchronize(&gone,7);assert!(q.current(&s,7).is_none());assert!(q.answer(&s,7,&t,1).is_err());
        grant(&mut q,&s);assert!(!q.receive(&s,7,&dialog(s.world.own_actor().unwrap().id,"other-dialog")).unwrap());assert!(q.current(&s,7).is_none());
    }
}
