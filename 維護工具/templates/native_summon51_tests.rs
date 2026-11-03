
#[cfg(test)]
mod summon51_development_tests {
    use super::*;
    use super::super::native_login::{Snapshot,LoginController,LoginPhase};
    fn snapshot()->Snapshot {
        let mut s=LoginController::default().snapshot().clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();let b:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();s.world.accept(&b).unwrap();}
        s.phase=LoginPhase::WorldReady;s.game_connected=true;s.world.learned_skills.clear();s.world.learned_skills.insert(51);s.world.hp=Some([1,200]);s.world.mp=Some([1,200]);s.inventory.begin(7,"SummonFixture").unwrap();s
    }
    fn pending(s:&Snapshot,now:Instant)->Pending {Pending{since:now,casts:s.world.own_cast_packets,messages:s.world.message_count,generation:s.world.generation,incarnation:s.world.own_actor().unwrap().incarnation}}
    fn action(s:&mut Snapshot){let mut b=vec![0x20];b.extend(s.world.own_actor().unwrap().id.to_le_bytes());b.push(19);s.world.accept(&b).unwrap();}
    fn message(s:&mut Snapshot,code:u16){let mut b=vec![0x10];b.extend(code.to_le_bytes());b.push(0);s.world.accept(&b).unwrap();}
    #[test]
    fn all_catalog_selections_have_seven_byte_wire_and_never_predict_server_costs() {
        let s=snapshot();let before=serde_json::to_value(&s.world).unwrap();assert_eq!(SUMMON_SELECTIONS.len(),31);for row in SUMMON_SELECTIONS {let t=SummonTarget::capture(&s,7,row.selection).unwrap();let mut expected=vec![20,6,2];expected.extend(row.selection.to_le_bytes());assert_eq!(summon_request(&s,7,&t).unwrap(),expected);}assert_eq!(serde_json::to_value(&s.world).unwrap(),before);assert!(s.inventory.items.is_empty());assert_eq!(reuse_millis(51),1000);
    }
    #[test]
    fn invalid_selection_and_npc_or_actor_identifiers_are_not_accepted_as_choice() {
        let s=snapshot();for n in [0,1,6,81210,81240,u32::MAX,s.world.own_actor().unwrap().id]{assert!(SummonTarget::capture(&s,7,n).is_err());}assert_eq!(SummonTarget::capture(&s,7,519).unwrap().selection(),519);
    }
    #[test]
    fn inventory_update_or_owner_selection_aba_invalidates_an_explicit_choice() {
        let s=snapshot();let t=SummonTarget::capture(&s,7,7).unwrap();for kind in 0..4{let mut n=s.clone();match kind{0=>n.inventory.revision+=1,1=>n.inventory.owner.as_mut().unwrap().selection+=1,2=>n.inventory.owner.as_mut().unwrap().character.push('X'),_=>n.inventory.owner=None};assert!(summon_request(&n,7,&t).is_err());}assert!(summon_request(&s,8,&t).is_err());
    }
    #[test]
    fn transfer_actor_replacement_and_apply_release_between_polls_revoke_choice() {
        let s=snapshot();let t=SummonTarget::capture(&s,7,263).unwrap();for kind in 0..5{let mut n=s.clone();match kind{0=>n.world.generation+=1,1=>n.world.local_corrections+=1,2=>{let id=n.world.own_actor().unwrap().id;n.world.actors.get_mut(&id).unwrap().incarnation+=1;},3=>n.world.map_id=n.world.map_id.map(|x|x.wrapping_add(1)),_=>{n.world.accept(&[0x25,2]).unwrap();n.world.accept(&[0x25,3]).unwrap();}}assert!(summon_request(&n,7,&t).is_err());}
    }
    #[test]
    fn observer_missing_book_mp_or_game_and_dead_caster_are_rejected() {
        for kind in 0..6{let mut s=snapshot();match kind{0=>s.observer=true,1=>s.world.learned_skills.clear(),2=>s.world.mp=Some([0,200]),3=>s.game_connected=false,4=>s.phase=LoginPhase::BattleResult,_=>{let id=s.world.own_actor().unwrap().id;s.world.actors.get_mut(&id).unwrap().dead=true;}}assert!(SummonTarget::capture(&s,7,7).is_err());}
    }
    #[test]
    fn local_hp_one_and_no_material_or_ring_do_not_replace_server_admission() {
        let s=snapshot();let t=SummonTarget::capture(&s,7,274).unwrap();assert_eq!(summon_request(&s,7,&t).unwrap(),vec![20,6,2,18,1,0,0]);assert!(s.inventory.items.is_empty());assert_eq!(s.world.hp.unwrap()[0],1);assert_eq!(s.world.mp.unwrap()[0],1);
    }
    #[test]
    fn generic_actor_item_and_ground_paths_cannot_impersonate_summon_domain() {
        let s=snapshot();let id=s.world.own_actor().unwrap().id;assert!(request(&s.world,51,None).is_err());assert!(request(&s.world,51,Some(id)).is_err());assert!(ground_request(&s.world,51,[0,0]).is_err());assert!(!item_targeted(51)&&summon_targeted(51));let mut m=super::super::native_skill_selection::Model::default();m.sync(7,&s.world,true);m.select(Some(51)).unwrap();assert!(m.rows().iter().find(|r|r.id==51).unwrap().label.contains("召喚種類"));assert_eq!(m.selected_skill(7,&s.world).unwrap(),51);assert!(m.choice(7,&s.world,None).is_err());
    }
    #[test]
    fn map_and_level_rejections_following_caster_action_override_only_summon_feedback() {
        for code in [353,743]{let mut s=snapshot();let now=Instant::now();let p=pending(&s,now);action(&mut s);message(&mut s,code);assert_eq!(p.observe(&s.world,now),Feedback::CastAction);assert_eq!(p.observe_summon(&s.world,now),Feedback::Rejected(code));}let mut s=snapshot();let now=Instant::now();let p=pending(&s,now);message(&mut s,319);assert_eq!(p.observe_summon(&s.world,now),Feedback::Rejected(319));
    }
    #[test]
    fn effects_and_action_are_not_companion_ack_or_local_lifetime() {
        let mut s=snapshot();let now=Instant::now();let p=pending(&s,now);let actors=s.world.actors.len();let hp=s.world.hp;let mp=s.world.mp;for gfx in [2510u16,755]{let mut b=vec![0x37];b.extend(s.world.own_actor().unwrap().id.to_le_bytes());b.extend(gfx.to_le_bytes());s.world.accept(&b).unwrap();}assert_eq!(p.observe_summon(&s.world,now),Feedback::Pending);assert_eq!(p.observe_summon(&s.world,now+Duration::from_secs(2)),Feedback::NoResponse);action(&mut s);assert_eq!(p.observe_summon(&s.world,now),Feedback::CastAction);assert_eq!(s.world.actors.len(),actors);assert_eq!(s.world.hp,hp);assert_eq!(s.world.mp,mp);
    }
    #[test]
    fn retired_world_dominates_late_summon_rejection() {
        let mut s=snapshot();let now=Instant::now();let p=pending(&s,now);s.world.generation+=1;message(&mut s,743);assert_eq!(p.observe_summon(&s.world,now),Feedback::Interrupted);
    }
}
