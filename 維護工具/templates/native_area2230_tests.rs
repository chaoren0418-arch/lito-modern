
#[cfg(test)]
mod area2230_development_tests {
    use super::*;
    use super::super::{native_skills,native_skill_selection::{Model,Target as SkillTarget},sprite_actions::ActionLimits};
    const CASTER:u32=1;const FIRST:u32=0xf1234567;const OTHER:u32=0xe1234567;
    const ORIGINAL:&str="4 0 0\n#1804 1 frozen cloud 0.fire(0 21,0.20:5 0.0:4[714 0.1:4 0.2:4 0.3:4 0.4:4 0.5:4 0.6:4 0.7:5! 0.8:5 0.9:5 0.10:5 0.11:5 0.12:5 0.13:5 0.14:5 0.15:5 0.16:5 0.17:5 0.18:5 0.19:5) 104.attr(8) 109.effect(1 -1)\n#1805 1 earthjail 0.fire(0 17,0.16:4 0.0:3 0.1:3[718 0.2:3 0.3:3! 0.4:3 0.5:3 0.6:4! 0.7:4 0.8:4 0.9:4 0.10:5 0.11:5 0.12:5 0.13:5 0.14:5 0.15:4) 101.shadow(1806) 105.clothes(1 1807) 109.effect(1 -1)\n#1806 1 shadow 102.type(0)\n#1807 1 smoke 102.type(0) 104.attr(8)\n";
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for k in ["world","start","status","actor","bot"] {let hex=f["packets"][k].as_str().unwrap();let b:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
        let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=CASTER;w.actors.insert(CASTER,own);w.status.as_mut().unwrap().object_id=CASTER;w.learned_skills.clear();w.learned_skills.extend([22,30]);w.hp=Some([50,50]);w.mp=Some([1,200]);
        for (i,id) in [FIRST,OTHER].into_iter().enumerate(){let mut a=w.own_actor().unwrap().clone();a.id=id;a.x+=i as u16+1;a.incarnation+=10+i as u64;w.actors.insert(id,a);}w
    }
    fn packet(w:&World,gfx:u16,heading:u8,targets:&[(u32,bool)])->Vec<u8>{let own=w.own_actor().unwrap();let mut b=vec![0x39,18];b.extend(CASTER.to_le_bytes());b.extend(own.x.to_le_bytes());b.extend(own.y.to_le_bytes());b.push(heading);b.extend(0xf123abcd_u32.to_le_bytes());b.extend(gfx.to_le_bytes());b.extend([8,0,0]);b.extend((targets.len()as u16).to_le_bytes());for &(id,hit) in targets{b.extend(id.to_le_bytes());b.push(if hit{0x20}else{0});}b}
    fn launch(w:&World)->EnteredEvent{EnteredEvent{actor:CASTER,sequence:w.own_actor().unwrap().action_sequence,action:18,frame:1,event:"Hit".into(),sound:None,prepared_audio:None}}
    fn vectors(gfx:u16)->Effects{let c=SpriteActionCatalog::parse_for_inspection(ORIGINAL.as_bytes(),ActionLimits::default()).unwrap();let mut animation=qualify_area2230(&c,gfx).unwrap().remove(0).2;for f in &mut animation.frames{f.events.retain(|e|!matches!(e,FrameEvent::Sound(_)));}let mut effects=Effects::default();effects.clips.insert(gfx,Clip::new(Arc::new(animation)).unwrap());effects}
    #[test]
    fn area2230_requests_keep_unsigned_actor_coordinates_range_and_server_costs(){
        for (skill,row,column,range,reuse) in [(22,2,5,6,300),(30,3,5,8,500)]{let mut w=world();let own=w.own_actor().unwrap();let x=own.x+range;let y=own.y;let target=w.actors.get_mut(&FIRST).unwrap();target.x=x;target.y=y;
            let mut expected=vec![0x14,row,column];expected.extend(FIRST.to_le_bytes());expected.extend(x.to_le_bytes());expected.extend(y.to_le_bytes());let before=serde_json::to_value(&w).unwrap();assert_eq!(native_skills::request(&w,skill,Some(FIRST)).unwrap(),expected);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert_eq!(native_skills::reuse_millis(skill),reuse);assert!(native_skills::request(&w,skill,None).is_err());assert!(native_skills::request(&w,skill,Some(CASTER)).is_err());w.actors.get_mut(&FIRST).unwrap().x+=1;assert!(native_skills::request(&w,skill,Some(FIRST)).is_err());
        }
    }
    #[test]
    fn area2230_menu_keeps_target_and_caster_staleness_fences(){
        for (skill,label) in [(22,"#22 寒冰氣息（選取目標，六格內）"),(30,"#30 巖牢（選取目標，八格內）")]{let w=world();let mut menu=Model::default();menu.sync(7,&w,true);menu.select(Some(skill)).unwrap();assert_eq!(menu.rows().iter().find(|r|r.id==skill).unwrap().label,label);let target=SkillTarget::capture(&w,FIRST).unwrap();assert_eq!(menu.choice(7,&w,Some(target)).unwrap().target,Some(FIRST));let mut changed=w.clone();changed.actors.get_mut(&FIRST).unwrap().target_revision+=1;assert!(menu.choice(7,&changed,Some(target)).is_err());changed=w.clone();changed.local_corrections+=1;assert!(menu.choice(7,&changed,Some(target)).is_err());assert!(menu.choice(8,&w,Some(target)).is_err());}
    }
    #[test]
    fn area2230_wire_profiles_and_malformed_packets_do_not_partially_mutate_world(){
        let w=world();for gfx in [1804,1805]{let b=packet(&w,gfx,3,&[(FIRST,true)]);assert_eq!(decode(&b).unwrap().unwrap().graphics,gfx);let mut wrong=b.clone();wrong[17]=0;assert!(decode(&wrong).unwrap().is_none());wrong=b.clone();wrong[1]=19;assert!(decode(&wrong).unwrap().is_none());let mut malformed=w.clone();let before=serde_json::to_value(&malformed).unwrap();assert!(malformed.accept(&b[..b.len()-1]).is_err());assert_eq!(serde_json::to_value(&malformed).unwrap(),before);}
    }
    #[test]
    fn area2230_captures_first_target_at_receipt_even_if_its_hit_bit_is_false(){
        for gfx in [1804,1805]{let mut w=world();let first=w.actors[&FIRST].clone();w.accept(&packet(&w,gfx,3,&[(FIRST,false),(OTHER,true)])).unwrap();assert_eq!(w.many_attacks.last().unwrap().target_origin,Some([i32::from(first.x),i32::from(first.y)]));w.actors.get_mut(&FIRST).unwrap().x+=9;let event=launch(&w);let mut effects=vectors(gfx);effects.update(&w,[&event].into_iter(),0);assert_eq!(effects.active[0].cue.target_origin,Some([i32::from(first.x),i32::from(first.y)]));}
    }
    #[test]
    fn area2230_missing_first_uses_one_wire_step_and_never_selects_the_second(){
        let directions=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)];for gfx in [1804,1805]{for (heading,(dx,dy)) in directions.into_iter().enumerate(){let mut w=world();w.actors.remove(&FIRST);let own=w.own_actor().unwrap();let expected=[i32::from(own.x)+dx,i32::from(own.y)+dy];w.accept(&packet(&w,gfx,heading as u8,&[(FIRST,true),(OTHER,true)])).unwrap();assert_eq!(w.many_attacks.last().unwrap().target_origin,Some(expected));let mut empty=world();empty.accept(&packet(&empty,gfx,heading as u8,&[])).unwrap();let own=empty.own_actor().unwrap();assert_eq!(empty.many_attacks.last().unwrap().target_origin,Some([i32::from(own.x)+dx,i32::from(own.y)+dy]));}}
    }
    #[test]
    fn area2230_hit_frames_fan_out_only_current_marked_targets_without_stat_writes(){
        for (gfx,expected_frames) in [(1804,vec![8]),(1805,vec![4,7])]{let mut w=world();w.accept(&packet(&w,gfx,2,&[(FIRST,true),(OTHER,true)])).unwrap();w.actors.get_mut(&OTHER).unwrap().incarnation+=1;let before=serde_json::to_value(&w).unwrap();let event=launch(&w);let mut effects=vectors(gfx);let mut targets=vec![];let mut frames=vec![];
            for step in 0..=600u64 {targets.extend(effects.update(&w,if step==0{Some(&event)}else{None}.into_iter(),step*10_000_000).into_iter().map(|i|i.target));frames.extend(effects.non_audio.iter().filter(|r|r["event"]=="Hit").map(|r|r["source_frame"].as_u64().unwrap() as usize));}
            assert_eq!(frames,expected_frames);assert_eq!(targets,vec![FIRST;expected_frames.len()]);assert_eq!(effects.started,1);assert_eq!(serde_json::to_value(&w).unwrap(),before);
        }
    }
    #[test]
    fn area2230_history_keeps_anchor_and_scene_retirement_drops_the_group(){
        let mut w=world();w.accept(&packet(&w,1805,0,&[(FIRST,true)])).unwrap();let event=launch(&w);let mut effects=vectors(1805);effects.update(&w,[&event].into_iter(),0);let origin=effects.active[0].cue.target_origin;let mut restored=vectors(1805);restored.restore_history(effects.history());restored.update(&w,[&event].into_iter(),0);assert_eq!(restored.active.len(),1);assert_eq!(restored.active[0].cue.target_origin,origin);assert_eq!(restored.started,1);w.generation+=1;w.many_attacks.clear();assert!(restored.update(&w,[&event].into_iter(),0).is_empty());assert!(restored.active.is_empty());assert!(restored.seen.is_empty());
    }
    #[test]
    fn area2230_original_cadence_layers_and_audio_are_exact(){
        let c=SpriteActionCatalog::parse_for_inspection(ORIGINAL.as_bytes(),ActionLimits::default()).unwrap();assert_eq!(qualify_area2230(&c,1804).unwrap()[0].2.frames.len(),21);assert_eq!(qualify_area2230(&c,1805).unwrap()[0].2.frames.len(),17);
        for (gfx,source) in [(1804,ORIGINAL.replace("[714","[718")),(1804,ORIGINAL.replace("0.7:5!","0.7:5")),(1805,ORIGINAL.replace("1806)","1802)")),(1805,ORIGINAL.replace("0.6:4!","0.6:4")),(1805,ORIGINAL.replace("#1807 1 smoke 102.type(0) 104.attr(8)","#1807 1 smoke 102.type(0) 104.attr(0)"))]{let changed=SpriteActionCatalog::parse_for_inspection(source.as_bytes(),ActionLimits::default()).unwrap();assert!(qualify_area2230(&changed,gfx).is_err());}
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; original1804/1805/layers and WAV714/718, memory draw only"]
    fn actual_area2230_uses_frozen_anchor_layers_and_prepared_native_audio(){
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");for (skill,gfx,sound,hit_count) in [(22,1804,714,1),(30,1805,718,2)]{let mut w=world();w.learned_skills.clear();w.learned_skills.insert(skill);let first=w.actors[&FIRST].clone();w.accept(&packet(&w,gfx,3,&[(FIRST,true)])).unwrap();w.actors.get_mut(&FIRST).unwrap().x+=3;let event=launch(&w);let mut effects=Effects::default();effects.prepare(Path::new(&root),&w,&AtomicBool::new(false));assert!(effects.error.is_none(),"{:?}",effects.error);let before=serde_json::to_value(&w).unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(first.x),y:i32::from(first.y)},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};let mut audio=vec![];let mut hits=0;
            for step in 0..=600u64{let now=step*10_000_000;hits+=effects.update(&w,if step==0{Some(&event)}else{None}.into_iter(),now).len();let mut fragments=vec![];let(report,events)=effects.draw_to(&mut Target::Fragments(&mut fragments),view,now);assert!(report["load_error"].is_null(),"{report}");if !fragments.is_empty(){assert!(fragments.iter().all(|f|f.position==WorldCell{x:i32::from(first.x),y:i32::from(first.y)}));assert_eq!(fragments.len(),if gfx==1805{3}else{1});}audio.extend(events);}
            assert_eq!(hits,hit_count);assert_eq!(audio.len(),1);assert_eq!(audio[0].sound,Some(sound));assert!(audio[0].prepared_audio.is_some());assert!(effects.active.iter().all(|a|a.finished));assert_eq!(serde_json::to_value(&w).unwrap(),before);
        }
    }
}
