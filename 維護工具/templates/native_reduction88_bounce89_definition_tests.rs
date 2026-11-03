
#[cfg(test)]mod reduction88_bounce89_definition_development_tests {
    use super::*;
    fn raw(gfx:u32)->&'static[u8]{match gfx{4824=>include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-reduction88-bounce89-inspection\20261024-180259-031734\4824.definition.txt"),4648=>include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-reduction88-bounce89-inspection\20261024-180259-031734\4648.definition.txt"),_=>panic!()}}
    fn catalog(raw:&[u8])->SpriteActionCatalog{SpriteActionCatalog::parse_for_inspection(&[b"11297 0 0\n".as_slice(),raw].concat(),super::super::sprite_actions::ActionLimits::default()).unwrap()}
    #[test]fn original_frames_are_class_zero_screen_with_overlap_not_child_sprite() {
        for(gfx,count,source,sound)in[(4824,19,3,3089),(4648,22,4,1924)]{let a=qualify_knight88_89(&catalog(raw(gfx)),gfx).unwrap();assert_eq!(a.frames.len(),count);let cues=super::super::combat_animation::classify_native_cues(&a.frames[source].events).unwrap();assert!(cues.non_audio.is_empty());let audio=cues.audio.unwrap();assert_eq!(audio.candidates(),&[sound]);assert_eq!(audio.mode,super::super::combat_animation::NativeAudioMode::AllowOverlappingVoices);}
    }
    #[test]fn wrong_class_blend_sound_or_frame_duration_cannot_use_fallback() {
        for(gfx,sound)in[(4824,3089),(4648,1924)]{let s=String::from_utf8(raw(gfx).to_vec()).unwrap();for(a,b)in[("109.effect(0 -1)".to_owned(),"109.effect(1 -1)".to_owned()),("104.attr(8)".into(),"104.attr(128)".into()),(format!("<{sound}"),format!("[{sound}")),("0.0:1".into(),"0.0:2".into())]{assert!(s.contains(&a));assert!(qualify_knight88_89(&catalog(s.replace(&a,&b).as_bytes()),gfx).is_err());}}
    }
    #[test]fn native_callbacks_keep_overlap_occurrence_and_terminal_dwell_separate_from_status() {
        for(gfx,sound,want_audio,want_end)in[(4824,3089,82,2366),(4648,1924,123,2281)]{let clip=super::super::native_effect_timing::Clip::new(std::sync::Arc::new(qualify_knight88_89(&catalog(raw(gfx)),gfx).unwrap())).unwrap();let mut p=clip.start();let mut sample=p.advance(0).unwrap();let(mut audio,mut at)=(vec![],0);while !sample.finished{for c in &sample.crossed_cues{if c.event==FrameEvent::Effect(sound){audio.push(at);}}at=p.next_callback_millis().unwrap();sample=p.advance(at*1_000_000).unwrap();assert!(p.advance(at*1_000_000).unwrap().crossed_cues.is_empty());}assert_eq!(audio,vec![want_audio]);assert_eq!(at,want_end);assert_eq!(sample.source_frame,0);}
    }
}
