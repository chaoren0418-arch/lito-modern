
#[cfg(test)]mod decay71_definition_development_tests {
    use super::*;
    const RAW:&[u8]=include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-decay71-meteor74-inspection\20261024-165259-729672\2232.definition.txt");
    fn catalog(raw:&[u8])->SpriteActionCatalog{SpriteActionCatalog::parse_for_inspection(&[b"11297 0 0\n".as_slice(),raw].concat(),super::super::sprite_actions::ActionLimits::default()).unwrap()}
    #[test]fn original_decay_visual_has_sound838_and_1952ms_independent_of_server_sixteen_seconds(){let a=qualify_one(&catalog(RAW),SPECS.iter().find(|s|s.sprite==2232).unwrap()).unwrap();assert_eq!(a.frames.len(),21);assert_eq!(a.frames[1].events,vec![FrameEvent::Sound(838)]);let mut p=super::super::native_effect_timing::Clip::new(std::sync::Arc::new(a)).unwrap().start();let mut s=p.advance(0).unwrap();let mut at=0;while !s.finished{at=p.next_callback_millis().unwrap();s=p.advance(at*1_000_000).unwrap();}assert_eq!(at,1952);}
    #[test]fn decay_sound_duration_and_mix_changes_are_not_substituted(){let text=String::from_utf8(RAW.to_vec()).unwrap();let spec=SPECS.iter().find(|s|s.sprite==2232).unwrap();for(a,b)in[("[838","<838"),("0.0:3","0.0:4"),("104.attr(8)","104.attr(128)")]{assert!(text.contains(a));assert!(qualify_one(&catalog(text.replace(a,b).as_bytes()),spec).is_err());}}
}
