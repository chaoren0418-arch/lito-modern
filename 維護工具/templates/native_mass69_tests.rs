
#[cfg(test)] mod component_tests {
    use super::*;
    fn snapshot()->Snapshot {
        let mut s=super::super::native_login::LoginController::default().snapshot().clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();s.world.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
        s.phase=LoginPhase::WorldReady;s.game_connected=true;s.selected_source=Some("isolated".into());s.selected_character=Some("fixture".into());
        s.inventory.owner=Some(native_inventory::Owner{connection:7,selection:1,character:"fixture".into()});s.world.hp=Some([1,100]);s.world.mp=Some([1,100]);s.world.learned_skills.clear();s.world.learned_skills.insert(69);s
    }
    fn mark(name:&str,d:Destination)->Vec<u8>{let mut b=vec![48];b.extend(name.as_bytes());b.push(0);b.extend(d.x.to_le_bytes());b.extend(d.y.to_le_bytes());b.extend(d.map.to_le_bytes());b}
    fn summon(s:&Snapshot)->Vec<u8>{let mut b=vec![85,0,64];b.extend(s.world.own_actor().unwrap().id.to_le_bytes());b}
    fn grant(q:&mut Invitation,s:&Snapshot,now:Instant)->Token{assert!(q.receive(s,7,now,&[36,236,2,0]).unwrap());q.current(s,7,now).unwrap().token()}
    #[test] fn bookmark_wire_is_name_xy_map_not_request_map_xy_and_bad_tail_is_atomic(){
        let s=snapshot();let mut book=Bookmarks::default();let d=Destination{map:4,x:32123,y:32654};let b=mark("town",d);book.receive(&s,7,&b).unwrap();assert_eq!(book.rows(&s,7)[0].destination,d);let revision=book.revision;
        for bad in [b[..b.len()-1].to_vec(),{let mut b=b.clone();b.push(0);b},mark("",d),mark(&"a".repeat(28),d)]{assert!(book.receive(&s,7,&bad).is_err());assert_eq!(book.revision,revision);assert_eq!(book.rows(&s,7)[0].destination,d);}
    }
    #[test] fn fifty_bookmarks_bound_is_unique_names_and_identical_replay_does_not_revoke(){
        let s=snapshot();let mut book=Bookmarks::default();for i in 0..50 {book.receive(&s,7,&mark(&format!("m{i}"),Destination{map:4,x:32000+i,y:32000})).unwrap();}let rev=book.revision;book.receive(&s,7,&mark("m0",Destination{map:4,x:32000,y:32000})).unwrap();assert_eq!(book.revision,rev);assert!(book.receive(&s,7,&mark("overflow",Destination{map:4,x:1,y:1})).is_err());assert_eq!(book.rows(&s,7).len(),50);
    }
    #[test] fn random_and_stored_requests_are_exact_nine_bytes_without_spending(){
        let s=snapshot();let mut book=Bookmarks::default();let d=Destination{map:4,x:32123,y:32654};book.receive(&s,7,&mark("town",d)).unwrap();let before=serde_json::to_value(&s.world).unwrap();let mut expected=vec![20,8,4];expected.extend(s.world.map_id.unwrap().to_le_bytes());expected.extend([0,0,0,0]);assert_eq!(Choice::random(&s,7).unwrap().request(&s,7,&book).unwrap(),expected);assert_eq!(Choice::stored(&s,7,&book,0).unwrap().request(&s,7,&book).unwrap(),vec![20,8,4,4,0,123,125,142,127]);assert_eq!(serde_json::to_value(&s.world).unwrap(),before);
    }
    #[test] fn bookmark_choice_cannot_revive_after_record_aba_or_character_owner_aba(){
        let s=snapshot();let mut book=Bookmarks::default();let a=Destination{map:4,x:10,y:11};let b=Destination{map:4,x:12,y:13};book.receive(&s,7,&mark("town",a)).unwrap();let choice=Choice::stored(&s,7,&book,0).unwrap();book.receive(&s,7,&mark("town",b)).unwrap();book.receive(&s,7,&mark("town",a)).unwrap();assert!(choice.request(&s,7,&book).is_err());let choice=Choice::stored(&s,7,&book,0).unwrap();let mut changed=s.clone();changed.inventory.owner.as_mut().unwrap().selection+=1;book.synchronize(&changed,7).unwrap();book.synchronize(&s,7).unwrap();book.receive(&s,7,&mark("town",a)).unwrap();assert!(choice.request(&s,7,&book).is_err());
    }
    #[test] fn choice_rejects_old_scene_life_book_and_insufficient_cast_state(){
        let s=snapshot();let book=Bookmarks::default();let c=Choice::random(&s,7).unwrap();for n in 0..6 {let mut s=s.clone();let id=s.world.own_actor().unwrap().id;match n{0=>s.world.generation+=1,1=>s.world.actors.get_mut(&id).unwrap().incarnation+=1,2=>s.world.learned_skills.clear(),3=>s.world.mp=Some([0,100]),4=>s.observer=true,_=>s.game_connected=false};assert!(c.request(&s,7,&book).is_err());}assert!(c.request(&s,8,&book).is_err());
    }
    #[test] fn recipient_without_spell_or_mp_can_consent_once_and_abandon_only_locally(){
        let mut s=snapshot();s.world.learned_skills.clear();s.world.mp=Some([0,100]);let now=Instant::now();let mut q=Invitation::default();let t=grant(&mut q,&s,now);assert_eq!(q.answer(&s,7,now,&t,true).unwrap(),Some(vec![35,236,2,1,0]));assert!(q.current(&s,7,now).unwrap().waiting());assert!(q.answer(&s,7,now,&t,true).is_err());assert_eq!(q.answer(&s,7,now,&t,false).unwrap(),None);assert!(q.current(&s,7,now).is_none());let t=grant(&mut q,&s,now);assert_eq!(q.answer(&s,7,now,&t,false).unwrap(),Some(vec![35,236,2,0,0]));
    }
    #[test] fn exact_748_empty_string_and_replacement_ids_reject_queued_old_decisions(){
        let s=snapshot();let now=Instant::now();let mut q=Invitation::default();let old=grant(&mut q,&s,now);for bad in [&[36,236,2][..],&[36,236,2,0,0][..],&[36,236,2,65,0][..]]{assert!(q.receive(&s,7,now,bad).is_err());assert_eq!(q.current(&s,7,now).unwrap().token(),old);}let new=grant(&mut q,&s,now);assert_ne!(old.command_id(true),new.command_id(true));assert!(q.answer(&s,7,now,&old,true).is_err());assert!(q.answer(&s,7,now,&new,true).is_ok());
    }
    #[test] fn invitation_display_ceiling_is_thirty_seconds_and_never_server_authority(){
        let s=snapshot();let now=Instant::now();let mut q=Invitation::default();let t=grant(&mut q,&s,now);assert!(q.current(&s,7,now+Duration::from_secs(29)).is_some());assert!(q.answer(&s,7,now+Duration::from_secs(30),&t,true).is_err());q.synchronize(&s,7,now+Duration::from_secs(30));assert!(q.current(&s,7,now).is_none());
    }
    #[test] fn invitation_owner_death_source_and_new_question_revoke_without_auto_answer(){
        let s=snapshot();let now=Instant::now();let mut q=Invitation::default();let t=grant(&mut q,&s,now);let mut dead=s.clone();dead.world.hp=Some([0,100]);assert!(q.answer(&dead,7,now,&t,true).is_err());let mut other=s.clone();other.selected_source=Some("other".into());assert!(q.answer(&other,7,now,&t,true).is_err());assert!(!q.receive(&s,7,now,&[36,65,1]).unwrap());assert!(q.current(&s,7,now).is_none());
    }
    #[test] fn explicit_own_s55_grants_one_ack_then_one_server_destination_without_local_cast(){
        let mut s=snapshot();s.world.learned_skills.clear();s.world.mp=None;let mut p=PortalGrant::default();assert_eq!(p.receive(&s,7,&summon(&s)).unwrap(),Some([78,0]));assert!(p.receive(&s,7,&summon(&s)).is_err());assert_eq!(p.take(&s,7,Destination{map:999,x:1,y:2}).unwrap(),LoginPhase::WorldReady);assert!(p.take(&s,7,Destination{map:999,x:1,y:2}).is_err());
    }
    #[test] fn malformed_or_wrong_actor_s55_cannot_replace_a_valid_grant(){
        let s=snapshot();let mut p=PortalGrant::default();let good=summon(&s);p.receive(&s,7,&good).unwrap();for mut bad in [good[..6].to_vec(),{let mut b=good.clone();b.push(0);b},{let mut b=good.clone();b[1]=1;b}]{assert!(p.receive(&s,7,&bad).is_err());assert!(p.current(&s,7));bad.clear();}let mut wrong=good;wrong[3]^=1;assert!(p.receive(&s,7,&wrong).is_err());assert!(p.current(&s,7));
    }
    #[test] fn summon_grant_ignores_same_life_appearance_but_not_new_life_scene_or_connection(){
        let s=snapshot();let mut p=PortalGrant::default();p.receive(&s,7,&summon(&s)).unwrap();let mut appearance=s.clone();let id=s.world.own_actor().unwrap().id;appearance.world.actors.get_mut(&id).unwrap().target_revision+=1;assert!(p.current(&appearance,7));assert!(!p.current(&s,8));let mut life=s.clone();life.world.actors.get_mut(&id).unwrap().incarnation+=1;p.synchronize(&life,7);assert!(!p.current(&s,7));p.receive(&s,7,&summon(&s)).unwrap();let mut scene=s.clone();scene.world.generation+=1;assert!(p.take(&scene,7,Destination{map:4,x:1,y:2}).is_err());assert!(!p.current(&s,7));
    }
}
