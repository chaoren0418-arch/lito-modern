
#[cfg(test)]mod solid90_counter91_render_development_tests {
    use super::*;
    fn original(gfx:u16,sound:Option<u32>,hit_source:usize,end:u64) {
        let root:std::path::PathBuf=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root").into();let cancel=AtomicBool::new(false);
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
        let mut other=w.own_actor().unwrap().clone();other.id=0x71234567;other.incarnation+=1;other.x+=2;w.actors.insert(other.id,other.clone());let mut b=vec![55];b.extend(other.id.to_le_bytes());b.extend(gfx.to_le_bytes());w.accept(&b).unwrap();w.actors.remove(&other.id);
        let authority=(w.hp,w.mp,w.movement_blocks.clone());let mut e=Effects::default();e.prepare(&root,&w,&cancel);assert!(e.additional_error.is_none());
        let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(other.x),y:i32::from(other.y)},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};
        let(mut at,mut sounds,mut hits)=(0,vec![],vec![]);
        loop {
            let mut fragments=vec![];let(receipt,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,at*1_000_000);
            for row in receipt["effects"].as_array().unwrap(){assert!(row.get("error").is_none(),"{row}");for h in row["diagnostic_hits"].as_array().unwrap(){assert_eq!(h["authority_callback"],false);hits.push(h["source_frame"].as_u64().unwrap()as usize);}}
            if at<end{assert_eq!(fragments.len(),1);assert_eq!(fragments[0].position,view.anchor_cell);assert!(matches!(fragments[0].mix,super::super::native_world_effects::Mix::Screen));}else{assert!(fragments.is_empty());}drop(fragments);
            for a in audio{assert_eq!(a.actor,0);assert!(a.prepared_audio.is_some());assert_eq!(a.sound,sound);sounds.push(a.sound.unwrap());}
            if e.active.is_empty(){break;}
            at=e.active[0].playback.next_callback_millis().unwrap();let history=e.history();w.effects.clear();let mut restored=Effects::default();restored.restore_history(history);restored.prepare(&root,&w,&cancel);e=restored;
        }
        assert_eq!(at,end);assert_eq!(hits,vec![hit_source]);assert_eq!(sounds,sound.into_iter().collect::<Vec<_>>());assert_eq!((w.hp,w.mp,w.movement_blocks.clone()),authority);
        let mut fragments=vec![];let(_,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,at*1_000_000);assert!(audio.is_empty());assert!(fragments.is_empty());
    }
    #[test]#[ignore="LITO_SHARED_RESOURCE_ROOT; original5831/3973, full Hit/History/terminal; no playback"]
    fn solid_original_effect_survives_Hit_and_owner_departure(){original(5831,Some(3973),10,2324);}
    #[test]#[ignore="LITO_SHARED_RESOURCE_ROOT; original5832/3974, full Hit/History/terminal; no playback"]
    fn counter_cast_original_effect_survives_Hit_and_owner_departure(){original(5832,Some(3974),15,2491);}
    #[test]#[ignore="LITO_SHARED_RESOURCE_ROOT; silent10710 trigger, full Hit/History/terminal"]
    fn counter_trigger_is_silent_and_Hit_does_not_retire_it_early(){original(10710,None,5,665);}
}
