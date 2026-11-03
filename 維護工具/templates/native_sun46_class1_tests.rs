
#[cfg(test)]
mod sun46_class1_history_development_tests {
    use super::*;
    fn catalog()->SpriteActionCatalog {
        let bytes=[b"11297 0 0\n".as_slice(),include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-sun46-weak47-inspection\20261024-100201-695865\1811.definition.txt")].concat();
        SpriteActionCatalog::parse_for_inspection(&bytes,super::super::sprite_actions::ActionLimits::default()).unwrap()
    }
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();
        let f:Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for name in ["world","start","status","actor","bot"] {let h=f["packets"][name].as_str().unwrap();let bytes:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&bytes).unwrap();}w
    }
    fn cue(w:&World,gfx:u16)->ProjectileCue {let own=w.own_actor().unwrap();ProjectileCue{attacker:own.id,
        attacker_incarnation:own.incarnation,target:2,target_incarnation:w.actors.get(&2).map(|a|a.incarnation),
        action_sequence:123,damage_marker:10,heading:2,projectile:super::super::native_combat::Projectile{
            id:77,graphics:gfx,flight_type:0,from:[own.x,own.y],to:[own.x+3,own.y]}}}
    fn event(c:&ProjectileCue)->EnteredEvent {EnteredEvent{actor:c.attacker,sequence:c.action_sequence,
        action:18,frame:1,event:"Hit".into(),sound:None,prepared_audio:None}}
    fn vector()->(Effects,World,ProjectileCue) {
        let mut w=world();let c=cue(&w,1811);w.projectiles.push(c.clone());
        let a=SpriteActionCatalog::parse_for_inspection(b"11297 0 0\n#999 1 vector 0.fire(0 3,0.2:1 0.0:1 0.1:1!)\n",super::super::sprite_actions::ActionLimits::default()).unwrap();
        let mut e=Effects::default();e.clips.insert(1811,Clip::new(Arc::new(a.resolve_action(999,ActionKey::new(0)).unwrap().animation.clone())).unwrap());
        e.update(&w,[&event(&c)].into_iter(),0);(e,w,c)
    }
    #[test]
    fn strict_sunburst_rejects_changed_hit_flags_direction_or_alias() {
        let c=catalog();let a=qualify_sunburst(&c).unwrap();assert_eq!(a[0].2.frames.len(),19);
        let mut altered=c.clone();altered.sprites.get_mut(&1811).unwrap().graphics_id=1;assert!(qualify_sunburst(&altered).is_err());
        let mut altered=c.clone();if let super::super::sprite_actions::ActionDefinition::Animation(a)=altered.sprites.get_mut(&1811).unwrap().actions.get_mut(&ActionKey::new(0)).unwrap(){a.frames[11].events.clear();}else{panic!("original animation expected");}assert!(qualify_sunburst(&altered).is_err());
    }
    #[test]
    fn native_per_frame_milliseconds_hit_at410_and_finish_at1904_without_catchup() {
        let animation=qualify_sunburst(&catalog()).unwrap().remove(0).2;let mut p=Clip::new(Arc::new(animation)).unwrap().start();
        let first=p.advance(0).unwrap();assert_eq!(first.source_frame,1);assert!(first.crossed_cues.iter().any(|c|c.event==FrameEvent::Sound(709)));
        let mut hit=None;let mut terminal=None;let mut end=None;
        for _ in 0..20 {
            let Some(at)=p.next_callback_millis() else{break;};let sample=p.advance(at*1_000_000).unwrap();
            if sample.crossed_cues.iter().any(|c|c.event==FrameEvent::Hit){hit=Some(at);}
            if sample.source_frame==0&&!sample.finished{terminal=Some(at);}
            if sample.finished{end=Some(at);break;}
        }
        assert_eq!((hit,terminal,end),(Some(410),Some(1738),Some(1904)));
        let mut p=Clip::new(Arc::new(qualify_sunburst(&catalog()).unwrap().remove(0).2)).unwrap().start();p.advance(0).unwrap();
        assert_eq!(p.advance(10_000_000_000).unwrap().source_frame,2);
    }
    #[test]
    fn same_source_metadata_keeps_cursor_and_sequence_after_raw_ring_and_book_eviction() {
        let (mut e,mut w,c)=vector();e.update(&w,std::iter::empty(),41_000_000);let h=e.history();
        w.projectiles.clear();w.learned_skills.clear();let mut restored=Effects::default();restored.restore_history(h.clone(),true);
        assert!(restored.banks.is_empty());assert_eq!(restored.history().active_sequences(),BTreeSet::from([c.action_sequence]));
        assert_eq!(restored.active[0].frame,h.active[0].frame);assert_eq!(restored.active[0].position,h.active[0].position);
        // This pure metadata vector has no resource/audio claims. Its Hit must
        // remain diagnostic for1811 until the original hurt family is bound.
        assert!(e.update(&w,std::iter::empty(),82_000_000).is_empty());
    }
    #[test]
    fn retired_source_preserves_admission_but_drops_every_active_class1() {
        let (e,_,c)=vector();let mut next=Effects::default();next.restore_history(e.history(),false);
        assert!(next.active.is_empty());assert!(next.seen.contains(&(c.attacker,c.action_sequence)));assert!(next.history().active_sequences().is_empty());
    }
    #[test]
    fn metadata_admission_merge_is_generation_bound_and_never_copies_graphs_or_resources() {
        let (e,w,c)=vector();let mut h=History{generation:Some(w.generation),seen:BTreeSet::new(),active:vec![],started:0,hits:0,omitted:0};h.merge_admission(&e.history());
        assert!(h.active.is_empty());assert!(h.seen.contains(&(c.attacker,c.action_sequence)));let mut stale=e.history();stale.generation=Some(w.generation+1);stale.seen.insert((9,99));h.merge_admission(&stale);assert!(!h.seen.contains(&(9,99)));
    }
    #[test]
    fn generation_retirement_clears_restored_clock_and_admission() {
        let (e,mut w,c)=vector();let mut restored=Effects::default();restored.restore_history(e.history(),true);w.generation+=1;w.projectiles.clear();
        restored.update(&w,[&event(&c)].into_iter(),50_000_000);assert!(restored.active.is_empty());assert!(restored.seen.is_empty());assert_eq!(restored.started,0);
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; original four class1 banks/layers, typed audio metadata, rebuild and memory fragments only"]
    fn original_four_profiles_rebuild_without_replaying_initial_audio_or_requiring_live_caster() {
        let root:std::path::PathBuf=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root").into();
        for (skill,gfx,sound) in [(10,252,110),(28,236,103),(16,1801,712),(46,1811,709)] {
            let mut w=world();w.learned_skills.clear();w.learned_skills.insert(skill);let c=cue(&w,gfx);w.projectiles.push(c.clone());
            let mut e=Effects::default();e.prepare(&root,&w,&AtomicBool::new(false));assert!(e.error.is_none(),"{:?}",e.error);
            e.update(&w,[&event(&c)].into_iter(),0);assert_eq!(e.events.iter().filter(|a|a.sound==Some(sound)).count(),1);
            assert!(e.events.iter().all(|a|a.actor==0&&a.prepared_audio.is_some()));let h=e.history();
            w.projectiles.clear();w.learned_skills.clear();w.actors.remove(&c.attacker);w.actors.remove(&c.target);
            let mut next=Effects::default();next.restore_history(h,true);next.prepare(&root,&w,&AtomicBool::new(false));assert!(next.error.is_none());
            next.update(&w,std::iter::empty(),50_000_000);assert!(next.events.is_empty());assert_eq!(next.started,1);assert_eq!(next.active[0].position,WorldCell{x:i32::from(c.projectile.to[0]),y:i32::from(c.projectile.to[1])});
            let view=ViewProjection{anchor_cell:next.active[0].position,anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};let mut fragments=vec![];
            let (receipt,_)=next.draw_to(&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,50_000_000);
            assert!(!fragments.is_empty());assert_eq!(receipt["effects"][0]["drawn"],true);
        }
    }
}
