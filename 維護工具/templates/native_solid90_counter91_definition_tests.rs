
#[cfg(test)]mod solid90_counter91_definition_development_tests {
    use super::*;
    fn raw(gfx:u32)->&'static[u8]{match gfx{5831=>include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-solid90-counter91-inspection\20261024-181628-946038\5831.definition.txt"),5832=>include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-solid90-counter91-inspection\20261024-181628-946038\5832.definition.txt"),10710=>include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-solid90-counter91-inspection\20261024-181628-946038\10710.definition.txt"),_=>panic!()}}
    fn catalog(raw:&[u8])->SpriteActionCatalog{SpriteActionCatalog::parse_for_inspection(&[b"11297 0 0\n".as_slice(),raw].concat(),super::super::sprite_actions::ActionLimits::default()).unwrap()}
    #[test]fn finite_definitions_keep_original_Hit_and_only_two_overlap_groups() {
        for(gfx,len,hit,sound)in[(5831,14,10,Some((1,3973))),(5832,17,15,Some((2,3974))),(10710,6,5,None)]{let a=qualify_knight90_91(&catalog(raw(gfx)),gfx).unwrap();assert_eq!(a.frames.len(),len);assert_eq!(a.frames[hit].events,vec![FrameEvent::Hit]);let mut groups=0;for f in &a.frames{let classified=super::super::combat_animation::classify_native_cues(&f.events).unwrap();if let Some(group)=classified.audio{groups+=1;assert_eq!(group.mode,super::super::combat_animation::NativeAudioMode::AllowOverlappingVoices);}}assert_eq!(groups,usize::from(sound.is_some()));if let Some((source,sound))=sound{assert_eq!(a.frames[source].events,vec![FrameEvent::Effect(sound)]);}}
    }
    #[test]fn changed_Hit_blend_or_hold_never_borrow_another_finite_definition() {
        for gfx in [5831,5832,10710]{let s=String::from_utf8(raw(gfx).to_vec()).unwrap();for(a,b)in[("!",""),("104.attr(8)","104.attr(128)"),("102.type(0)","102.type(1)"),("0.0:","1.0:")]{assert!(s.contains(a));assert!(qualify_knight90_91(&catalog(s.replace(a,b).as_bytes()),gfx).is_err());}}
    }
    #[test]fn native_Hit_and_audio_callbacks_continue_through_terminal_dwell() {
        for(gfx,audio_at,hit_at,end)in[(5831,Some(0),1494,2324),(5832,Some(83),1993,2491),(10710,None,374,665)]{let clip=super::super::native_effect_timing::Clip::new(std::sync::Arc::new(qualify_knight90_91(&catalog(raw(gfx)),gfx).unwrap())).unwrap();let mut p=clip.start();let mut sample=p.advance(0).unwrap();let(mut audio,mut hits,mut at)=(vec![],vec![],0);while !sample.finished{for c in &sample.crossed_cues{match c.event{FrameEvent::Effect(_)=>audio.push(at),FrameEvent::Hit=>hits.push(at),_=>panic!("unreviewed cue")}}at=p.next_callback_millis().unwrap();sample=p.advance(at*1_000_000).unwrap();assert!(p.advance(at*1_000_000).unwrap().crossed_cues.is_empty());}assert_eq!(audio,audio_at.into_iter().collect::<Vec<_>>());assert_eq!(hits,vec![hit_at]);assert_eq!(at,end);}
    }
}
