
#[cfg(test)]
mod tornado53_development_tests {
    use super::*;
    use super::super::{native_skills,native_skill_selection::{Model,Target as SkillTarget},sprite_actions::ActionLimits};
    const CASTER:u32=1;const LIVE:u32=0xf1234567;const MISS:u32=0xe1234567;const STALE:u32=0xd1234567;
    const ORIGINAL:&str="1 0 0\n#758 1 Tornado 0.fire(0 20,0.19:3 0.0:3[125! 0.1:3 0.2:3 0.3:3 0.4:3 0.5:3 0.6:3 0.7:3 0.8:3 0.9:3 0.10:3 0.11:3 0.12:3 0.13:3 0.14:3 0.15:3 0.16:3 0.17:3 0.18:3) 104.attr(8)\n";
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let value:Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for name in ["world","start","status","actor","bot"] {let hex=value["packets"][name].as_str().unwrap();let bytes:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();w.accept(&bytes).unwrap();}
        let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=CASTER;w.actors.insert(CASTER,own);w.status.as_mut().unwrap().object_id=CASTER;w.learned_skills.clear();w.learned_skills.insert(53);w.hp=Some([50,50]);w.mp=Some([1,200]);
        for (n,id) in [LIVE,MISS,STALE].into_iter().enumerate(){let mut actor=w.own_actor().unwrap().clone();actor.id=id;actor.x+=n as u16+1;actor.incarnation+=10+n as u64;w.actors.insert(id,actor);}w
    }
    fn packet(w:&World,targets:&[(u32,bool)])->Vec<u8>{let own=w.own_actor().unwrap();let mut b=vec![0x39,18];b.extend(CASTER.to_le_bytes());b.extend(own.x.to_le_bytes());b.extend(own.y.to_le_bytes());b.push(own.heading);b.extend(0xf123abcd_u32.to_le_bytes());b.extend(758_u16.to_le_bytes());b.extend([0,0,0]);b.extend((targets.len()as u16).to_le_bytes());for &(id,hit) in targets{b.extend(id.to_le_bytes());b.push(if hit{0x20}else{0});}b}
    fn launch(w:&World)->EnteredEvent{EnteredEvent{actor:CASTER,sequence:w.own_actor().unwrap().action_sequence,action:18,frame:1,event:"Hit".into(),sound:None,prepared_audio:None}}
    fn silent_model_effects()->Effects{
        // Unit vectors isolate callback ownership; no resource or audio qualification is granted.
        let catalog=SpriteActionCatalog::parse_for_inspection(ORIGINAL.as_bytes(),ActionLimits::default()).unwrap();let mut animation=qualify_tornado(&catalog).unwrap().remove(0).2;
        for frame in &mut animation.frames{frame.events.retain(|event|!matches!(event,FrameEvent::Sound(_)));}
        let mut effects=Effects::default();effects.clips.insert(758,Clip::new(Arc::new(animation)).unwrap());effects
    }
    #[test]
    fn tornado_request_is_selector_only_and_keeps_costs_server_owned(){
        let w=world();let before=serde_json::to_value(&w).unwrap();assert_eq!(native_skills::request(&w,53,None).unwrap(),vec![0x14,6,4]);assert_eq!(native_skills::reuse_millis(53),1500);
        assert!(native_skills::request(&w,53,Some(LIVE)).is_err());assert!(native_skills::ground_request(&w,53,[32768,32768]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
        let mut no_book=w.clone();no_book.learned_skills.remove(&53);assert!(native_skills::request(&no_book,53,None).is_err());no_book.learned_skills.insert(53);no_book.mp=Some([0,200]);assert!(native_skills::request(&no_book,53,None).is_err());
    }
    #[test]
    fn tornado_menu_ignores_unrelated_target_but_keeps_caster_owner_fence(){
        let w=world();let mut menu=Model::default();menu.sync(7,&w,true);assert_eq!(menu.rows().iter().find(|r|r.id==53).unwrap().label,"#53 龍捲風（自己周圍四格）");menu.select(Some(53)).unwrap();let stale=SkillTarget::capture(&w,LIVE).unwrap();let mut changed=w.clone();changed.actors.get_mut(&LIVE).unwrap().incarnation+=1;
        let choice=menu.choice(7,&changed,Some(stale)).unwrap();assert_eq!(choice.target,None);assert_eq!(choice.ground,None);assert!(menu.choice(8,&changed,None).is_err());changed.local_corrections+=1;assert!(menu.choice(7,&changed,None).is_err());
    }
    #[test]
    fn tornado_original_twenty_frames_hit_sound_and_definition_are_strict(){
        let catalog=SpriteActionCatalog::parse_for_inspection(ORIGINAL.as_bytes(),ActionLimits::default()).unwrap();let animation=qualify_tornado(&catalog).unwrap().remove(0).2;assert_eq!(animation.frames.len(),20);
        assert_eq!(animation.frames[1].events,vec![FrameEvent::Sound(125),FrameEvent::Hit]);assert_eq!(animation.frames[0].subframe,19);assert!(animation.frames.iter().all(|f|f.duration_ticks==3));
        for source in [ORIGINAL.replace("#758 1","#758 8"),ORIGINAL.replace("[125!","[124!"),ORIGINAL.replace("0.18:3","0.18:4"),ORIGINAL.replace("0.2:3","0.2:3!"),ORIGINAL.replace("104.attr(8)","104.attr(0)")] {let c=SpriteActionCatalog::parse_for_inspection(source.as_bytes(),ActionLimits::default()).unwrap();assert!(qualify_tornado(&c).is_err());}
    }
    #[test]
    fn tornado_wire_profile_is_exact_and_zero_victims_are_still_a_cast(){
        let w=world();let b=packet(&w,&[]);assert_eq!(b.len(),22);assert!(decode(&b).unwrap().unwrap().targets.is_empty());let mut wrong=b.clone();wrong[17]=8;assert!(decode(&wrong).unwrap().is_none());wrong=b.clone();wrong[1]=19;assert!(decode(&wrong).unwrap().is_none());
        for end in 0..b.len(){assert!(decode(&b[..end]).is_err());}let mut tail=b.clone();tail.push(0);assert!(decode(&tail).is_err());let mut zero=w.clone();zero.accept(&b).unwrap();let e=launch(&zero);let mut effects=silent_model_effects();assert!(effects.update(&zero,[&e].into_iter(),0).is_empty());assert_eq!(effects.started,1);assert_eq!(effects.active.len(),1);
    }
    #[test]
    fn tornado_fans_out_one_group_only_to_current_hit_incarnations(){
        let mut w=world();w.accept(&packet(&w,&[(LIVE,true),(MISS,false),(STALE,true)])).unwrap();w.actors.get_mut(&STALE).unwrap().incarnation+=1;let before=serde_json::to_value(&w).unwrap();let e=launch(&w);let mut effects=silent_model_effects();let impacts=effects.update(&w,[&e].into_iter(),0);
        assert_eq!(impacts.len(),1);assert_eq!(impacts[0].target,LIVE);assert_eq!(effects.started,1);assert_eq!(effects.active.len(),1);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert!(effects.update(&w,[&e].into_iter(),0).is_empty());assert_eq!(effects.started,1);
    }
    #[test]
    fn tornado_unknown_target_cannot_acquire_a_later_actor_or_replacement(){
        let mut w=world();let actor=w.actors.remove(&LIVE).unwrap();w.accept(&packet(&w,&[(LIVE,true)])).unwrap();w.actors.insert(LIVE,actor);let e=launch(&w);let mut effects=silent_model_effects();assert!(effects.update(&w,[&e].into_iter(),0).is_empty());assert_eq!(effects.started,1);
    }
    #[test]
    fn tornado_history_keeps_one_cursor_and_scene_retirement_clears_it(){
        let mut w=world();w.accept(&packet(&w,&[(LIVE,true)])).unwrap();let e=launch(&w);let mut effects=silent_model_effects();assert_eq!(effects.update(&w,[&e].into_iter(),0).len(),1);let history=effects.history();let mut resumed=silent_model_effects();resumed.restore_history(history);assert!(resumed.update(&w,[&e].into_iter(),0).is_empty());assert_eq!(resumed.started,1);
        w.generation+=1;w.many_attacks.clear();assert!(resumed.update(&w,[&e].into_iter(),0).is_empty());assert!(resumed.active.is_empty());assert!(resumed.seen.is_empty());assert_eq!(resumed.started,0);
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; original 758 SPR and Sound125 read/draw in memory, no playback"]
    fn actual_tornado_shared_bank_carries_prepared_audio_and_retires_once(){
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");let mut w=world();w.accept(&packet(&w,&[(LIVE,true)])).unwrap();let before=serde_json::to_value(&w).unwrap();let e=launch(&w);let mut effects=Effects::default();effects.prepare(Path::new(&root),&w,&AtomicBool::new(false));assert!(effects.error.is_none(),"{:?}",effects.error);assert!(effects.tornado_bank.is_some());
        let own=w.own_actor().unwrap();let projection=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};let mut audio=Vec::new();
        for step in 0..=400u64 {let now=step*10_000_000;let impacts=effects.update(&w,if step==0{Some(&e)}else{None}.into_iter(),now);if step==0{assert_eq!(impacts.len(),1);}else{assert!(impacts.is_empty());}let mut fragments=Vec::new();let(report,events)=effects.draw_to(&mut Target::Fragments(&mut fragments),projection,now);assert!(report["load_error"].is_null(),"{report}");audio.extend(events);}
        assert!(effects.active.is_empty());assert_eq!(effects.started,1);assert_eq!(audio.len(),1);assert_eq!(audio[0].sound,Some(125));assert!(audio[0].prepared_audio.is_some());assert_eq!(audio[0].prepared_audio.as_ref().unwrap().group().mode,super::super::combat_animation::NativeAudioMode::SuppressWhileSameIdPlaying);assert_eq!(serde_json::to_value(&w).unwrap(),before);
    }
}
