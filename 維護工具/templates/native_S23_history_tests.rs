
#[cfg(test)]
mod S23_history_development_tests {
    use super::*;
    fn empty(generation:u64)->History {History{generation:Some(generation),seen:BTreeSet::new(),flights:vec![],launched:0,arrived:0,omitted:0}}
    #[test]
    fn metadata_roundtrip_keeps_admission_and_counters_even_when_source_retires(){
        let mut h=empty(7);h.seen.insert((0xf1234567,91));h.launched=5;h.arrived=3;h.omitted=2;let mut p=Projectiles::default();p.restore_history(h,false);let saved=p.history();assert_eq!(saved.generation,Some(7));assert!(saved.seen.contains(&(0xf1234567,91)));assert_eq!((saved.launched,saved.arrived,saved.omitted),(5,3,2));assert!(saved.flights.is_empty());
    }
    #[test]
    fn cancelled_worker_admission_merges_only_with_matching_world_generation(){
        let mut h=empty(7);h.seen.insert((1,10));let mut peer=empty(7);peer.seen.insert((2,20));h.merge_admission(&peer);assert_eq!(h.seen,BTreeSet::from([(1,10),(2,20)]));let mut retired=empty(8);retired.seen.insert((3,30));h.merge_admission(&retired);assert!(!h.seen.contains(&(3,30)));assert!(h.flights.is_empty());
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; original167/219/WAV97 read/draw in memory, no device/GUI"]
    fn actual_S23_model_rebuild_keeps_cursor_and_retired_source_does_not_relaunch(){
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");let cancel=AtomicBool::new(false);let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let fixture:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for key in ["world","start","status","actor","bot"] {let hex=fixture["packets"][key].as_str().unwrap();let packet:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();w.accept(&packet).unwrap();}
        w.learned_skills.clear();w.learned_skills.insert(4);let own=w.own_actor().unwrap().clone();let target=0x71234567;let mut other=own.clone();other.id=target;other.x+=8;other.incarnation+=1;w.actors.insert(target,other.clone());
        let projectile=super::super::native_combat::Projectile{id:77,graphics:167,flight_type:0,from:[own.x,own.y],to:[other.x,other.y]};
        w.projectiles.push(ProjectileCue{heading:2,attacker_incarnation:own.incarnation,target_incarnation:Some(other.incarnation),action_sequence:123,attacker:own.id,target,damage_marker:0,projectile});let event=EnteredEvent{actor:own.id,sequence:123,action:18,frame:1,event:"Hit".into(),sound:None,prepared_audio:None};
        let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};
        let mut initial=Projectiles::default();initial.prepare(Path::new(&root),&w,&cancel);assert!(initial.errors.is_empty());initial.update(&w,[&event].into_iter(),0);let mut fragments=vec![];let(_,audio)=initial.draw_to(&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,0);assert_eq!(audio.iter().filter(|v|v.sound==Some(97)).count(),1);drop(fragments);
        let history=initial.history();assert_eq!(history.flights.len(),1);let mut restored=Projectiles::default();restored.restore_history(history.clone(),true);
        // Active metadata still requires assets after its original ring entry
        // and the local learned-book hint have both left this snapshot.
        let mut ring_evicted=w.clone();ring_evicted.projectiles.clear();ring_evicted.learned_skills.clear();restored.prepare(Path::new(&root),&ring_evicted,&cancel);assert!(restored.assets.contains_key(&167));restored.update(&ring_evicted,std::iter::empty(),60_000_000);let mut fragments=vec![];let(_,audio)=restored.draw_to(&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,60_000_000);assert!(audio.is_empty());drop(fragments);assert_eq!(restored.launched,1);
        let mut retired=Projectiles::default();retired.restore_history(history,false);retired.prepare(Path::new(&root),&w,&cancel);retired.update(&w,[&event].into_iter(),60_000_000);assert!(retired.flights.is_empty());assert_eq!(retired.launched,1);assert_eq!(retired.omitted,0);
        ring_evicted.actors.get_mut(&target).unwrap().incarnation+=1;restored.update(&ring_evicted,std::iter::empty(),70_000_000);assert!(restored.flights.is_empty());ring_evicted.generation+=1;restored.update(&ring_evicted,std::iter::empty(),80_000_000);assert!(restored.seen.is_empty());assert_eq!(restored.launched,0);
    }
}
