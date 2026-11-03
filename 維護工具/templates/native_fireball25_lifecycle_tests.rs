
#[cfg(test)]
mod fireball25_development_tests {
    use super::*;
    use super::super::{native_skills,native_skill_selection::{Model,Target as SkillTarget}};
    const CASTER:u32=1;const FIRST:u32=0xf1234567;const OTHER:u32=0xe1234567;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for k in ["world","start","status","actor","bot"] {let hex=f["packets"][k].as_str().unwrap();let b:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
        let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=CASTER;w.actors.insert(CASTER,own);w.status.as_mut().unwrap().object_id=CASTER;w.learned_skills.clear();w.learned_skills.extend([25]);w.hp=Some([50,50]);w.mp=Some([1,200]);
        for (i,id) in [FIRST,OTHER].into_iter().enumerate(){let mut a=w.own_actor().unwrap().clone();a.id=id;a.x+=i as u16+1;a.incarnation+=10+i as u64;w.actors.insert(id,a);}w
    }
    fn packet(w:&World,gfx:u16,heading:u8,targets:&[(u32,bool)])->Vec<u8>{let own=w.own_actor().unwrap();let mut b=vec![0x39,18];b.extend(CASTER.to_le_bytes());b.extend(own.x.to_le_bytes());b.extend(own.y.to_le_bytes());b.push(heading);b.extend(0xf123abcd_u32.to_le_bytes());b.extend(gfx.to_le_bytes());b.extend([8,0,0]);b.extend((targets.len()as u16).to_le_bytes());for &(id,hit) in targets{b.extend(id.to_le_bytes());b.push(if hit{0x20}else{0});}b}
    fn launch(w:&World)->EnteredEvent{EnteredEvent{actor:CASTER,sequence:w.own_actor().unwrap().action_sequence,action:18,frame:1,event:"Hit".into(),sound:None,prepared_audio:None}}

    #[test]
    fn fireball_request_is_actor_xy_six_cells_with_server_owned_costs(){
        let mut w=world();let own=w.own_actor().unwrap();let x=own.x+6;let y=own.y;w.actors.get_mut(&FIRST).unwrap().x=x;let before=serde_json::to_value(&w).unwrap();let mut expected=vec![20,3,0];expected.extend(FIRST.to_le_bytes());expected.extend(x.to_le_bytes());expected.extend(y.to_le_bytes());assert_eq!(native_skills::request(&w,25,Some(FIRST)).unwrap(),expected);assert_eq!(native_skills::reuse_millis(25),500);assert_eq!(serde_json::to_value(&w).unwrap(),before);assert!(native_skills::request(&w,25,None).is_err());assert!(native_skills::request(&w,25,Some(CASTER)).is_err());w.actors.get_mut(&FIRST).unwrap().x+=1;assert!(native_skills::request(&w,25,Some(FIRST)).is_err());
    }
    #[test]
    fn fireball_menu_reuses_target_owner_revision_and_connection_fences(){
        let w=world();let mut m=Model::default();m.sync(7,&w,true);m.select(Some(25)).unwrap();assert_eq!(m.rows().iter().find(|r|r.id==25).unwrap().label,"#25 火球術（選取目標，六格內）");let t=SkillTarget::capture(&w,FIRST).unwrap();assert_eq!(m.choice(7,&w,Some(t)).unwrap().target,Some(FIRST));let mut replaced=w.clone();replaced.actors.get_mut(&FIRST).unwrap().incarnation+=1;assert!(m.choice(7,&replaced,Some(t)).is_err());assert!(m.choice(8,&w,Some(t)).is_err());
    }
    #[test]
    fn fireball_s39_profile_is_exact_and_bad_tail_is_atomic(){
        let mut w=world();let b=packet(&w,171,2,&[(FIRST,true)]);assert_eq!(decode(&b).unwrap().unwrap().graphics,171);let mut bad=b.clone();bad[17]=0;assert!(decode(&bad).unwrap().is_none());bad=b.clone();bad[1]=19;assert!(decode(&bad).unwrap().is_none());let before=serde_json::to_value(&w).unwrap();assert!(w.accept(&b[..b.len()-1]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
    }
    #[test]
    fn fireball_launch_rechecks_only_first_captured_incarnation_then_freezes_path(){
        let mut w=world();w.accept(&packet(&w,171,2,&[(FIRST,false),(OTHER,true)])).unwrap();let c=w.many_attacks.last().unwrap().clone();let saved=c.target_origin.unwrap();w.actors.get_mut(&FIRST).unwrap().x+=3;let path=fireball_path(&c,&w).unwrap();assert_eq!(path.target().x,saved[0]+3);w.actors.get_mut(&FIRST).unwrap().x+=4;assert_eq!(path.target().x,saved[0]+3);w.actors.get_mut(&FIRST).unwrap().incarnation+=1;assert_eq!(fireball_path(&c,&w).unwrap().target(),WorldCell{x:saved[0],y:saved[1]});w.actors.remove(&FIRST);assert_eq!(fireball_path(&c,&w).unwrap().target(),WorldCell{x:saved[0],y:saved[1]});
    }
    #[test]
    fn fireball_arrival_keeps_miss_dead_and_duplicates_but_rejects_unknown_replacements(){
        let mut w=world();w.actors.get_mut(&OTHER).unwrap().dead=true;w.accept(&packet(&w,171,2,&[(FIRST,false),(OTHER,true),(FIRST,true),(0xd1234567,true)])).unwrap();let c=w.many_attacks.last().unwrap().clone();let before=serde_json::to_value(&w).unwrap();let(r,h)=fireball_recipients(&c,&w);assert_eq!(r.iter().map(|r|r.id).collect::<Vec<_>>(),vec![FIRST,OTHER,FIRST]);assert_eq!(h.iter().map(|r|r.target).collect::<Vec<_>>(),vec![FIRST]);assert_eq!(serde_json::to_value(&w).unwrap(),before);w.actors.get_mut(&FIRST).unwrap().incarnation+=1;let(r,h)=fireball_recipients(&c,&w);assert_eq!(r.len(),1);assert_eq!(r[0].id,OTHER);assert!(h.is_empty());
    }
    #[test]
    fn fireball_unknown_at_receipt_never_acquires_a_new_first_target(){
        let mut w=world();let actor=w.actors.remove(&FIRST).unwrap();w.accept(&packet(&w,171,0,&[(FIRST,true),(OTHER,true)])).unwrap();let c=w.many_attacks.last().unwrap().clone();let saved=c.target_origin.unwrap();w.actors.insert(FIRST,actor);assert_eq!(fireball_path(&c,&w).unwrap().target(),WorldCell{x:saved[0],y:saved[1]});assert_eq!(fireball_recipients(&c,&w).0.iter().map(|r|r.id).collect::<Vec<_>>(),vec![OTHER]);
        let mut edge=world();edge.actors.get_mut(&CASTER).unwrap().x=0;edge.actors.get_mut(&CASTER).unwrap().y=0;edge.accept(&packet(&edge,171,7,&[])).unwrap();assert_eq!(fireball_path(edge.many_attacks.last().unwrap(),&edge).unwrap().target(),WorldCell{x:-1,y:-1});
    }
    #[test]
    fn fireball_group_budget_and_every_anchor_are_checked_before_any_submission(){
        let frame=super::super::sprite::SpriteFrame{fragments:Some(super::super::sprite::SpriteFrameFragments{references:vec![],atlas:Arc::new(vec![])}),x_offset:0,y_offset:0,width:0,height:0,frame_type:0,unknown1:0,unknown2:0,rgba:vec![]};let origin=WorldCell{x:0,y:0};let v=ViewProjection{anchor_cell:origin,anchor_pixel:super::super::world_map::PixelPoint{x:0,y:0}};let mut queue=vec![];for _ in 0..511{queue.push(super::super::native_world_effects::Effect{frame:&frame,position:origin,mix:Mix::Screen});}assert!(submit_fireball_group(&mut Target::Fragments(&mut queue),&frame,&[origin,origin],v).is_err());assert_eq!(queue.len(),511);queue.clear();assert!(submit_fireball_group(&mut Target::Fragments(&mut queue),&frame,&[origin,WorldCell{x:i32::MAX,y:i32::MIN}],v).is_err());assert!(queue.is_empty());submit_fireball_group(&mut Target::Fragments(&mut queue),&frame,&[origin,origin],v).unwrap();assert_eq!(queue.len(),2);
    }
    #[test]
    fn fireball_known_no_hurt_graphics_still_receive_secondary_visuals(){
        for gfx in [1245,1590,2001,5620,2665,2667,2669,2671,2674]{let mut w=world();w.actors.get_mut(&FIRST).unwrap().graphics=gfx;w.accept(&packet(&w,171,2,&[(FIRST,true)])).unwrap();let before=serde_json::to_value(&w).unwrap();let(r,h)=fireball_recipients(w.many_attacks.last().unwrap(),&w);assert_eq!(r.len(),1);assert!(h.is_empty());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; class5 full metadata/SPRs/WAVs and memory draw; no device/GUI"]
    fn actual_fireball_flow_fans_out_audio_order_frozen_anchors_history_and_generation(){
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");let cancel=AtomicBool::new(false);
        for distance in [1,6]{let mut w=world();let own=w.own_actor().unwrap().clone();w.actors.get_mut(&FIRST).unwrap().x=own.x+distance;w.accept(&packet(&w,171,2,&[(FIRST,true),(OTHER,false),(FIRST,true)])).unwrap();let event=launch(&w);let mut e=Effects::default();e.prepare(Path::new(&root),&w,&cancel);assert!(e.error.is_none(),"{:?}",e.error);let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};let mut audio=vec![];let mut hurts=0;let mut arrival_positions=None;let mut restored=false;
            for step in 0..=250u64{if distance==6&&step==1{w.actors.get_mut(&FIRST).unwrap().x+=1;}let now=step*10_000_000;let before=serde_json::to_value(&w).unwrap();hurts+=e.update(&w,if step==0{Some(&event)}else{None}.into_iter(),now).len();assert_eq!(serde_json::to_value(&w).unwrap(),before);if e.non_audio.iter().any(|r|r["event"]=="Class5Arrival"){assert_eq!(e.events.iter().map(|v|v.sound).collect::<Vec<_>>(),vec![Some(99),Some(99),Some(99),Some(98)]);}
                let mut queue=vec![];let(report,events)=e.draw_to(&mut Target::Fragments(&mut queue),view,now);assert!(report["load_error"].is_null(),"{report}");if let Some(row)=report["effects"].as_array().unwrap().iter().find(|r|r["phase"]=="secondary"&&r["drawn"]==true){assert_eq!(queue.len(),3);let positions=row["positions"].clone();if let Some(old)=&arrival_positions{assert_eq!(old,&positions);}else{arrival_positions=Some(positions);}}audio.extend(events);drop(queue);
                if distance==6&&step==20&&!restored{let history=e.history();let mut next=Effects::default();next.restore_history(history);next.prepare(Path::new(&root),&w,&cancel);assert!(next.error.is_none());e=next;restored=true;}
            }
            assert_eq!(hurts,2);assert!(e.active.is_empty());assert_eq!(e.started,1);assert_eq!(audio.iter().filter(|v|v.sound==Some(99)).count(),3);assert_eq!(audio.iter().filter(|v|v.sound==Some(98)).count(),if distance==1{1}else{2});let mut payloads=audio.iter().map(|v|v.prepared_audio.clone().unwrap()).collect::<Vec<_>>();let mut history=super::super::native_audio_delivery::FiniteDeliveryHistory::default();history.select_world((7,w.generation,w.map_id.unwrap()));assert_eq!(history.filter_frame(&mut payloads).unwrap().kept,audio.len());w.actors.get_mut(&FIRST).unwrap().x=own.x+8;w.accept(&packet(&w,171,2,&[(FIRST,true)])).unwrap();let next=launch(&w);e.update(&w,[&next].into_iter(),3_000_000_000);w.actors.get_mut(&CASTER).unwrap().incarnation+=1;assert!(e.update(&w,std::iter::empty(),3_060_000_000).is_empty());assert!(e.error.is_none());assert!(e.active.iter().all(|a|a.finished));w.generation+=1;w.many_attacks.clear();assert!(e.update(&w,[&event].into_iter(),3_070_000_000).is_empty());assert!(e.active.is_empty());assert!(e.seen.is_empty());
        }
    }
}
