
#[cfg(test)]mod shock87_definition_development_tests {
 use super::*;const RAW:&[u8]=include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-freezing80-shock87-inspection\20261024-172131-694095\4434.definition.txt");
 fn catalog(raw:&[u8])->SpriteActionCatalog{SpriteActionCatalog::parse_for_inspection(&[b"11297 0 0\n".as_slice(),raw].concat(),super::super::sprite_actions::ActionLimits::default()).unwrap()}
 #[test]fn six_frames_require_sound1498_source_two_and_original_class(){let a=qualify_shock87(&catalog(RAW)).unwrap();assert_eq!(a.frames.len(),6);assert_eq!(a.frames[2].events,vec![FrameEvent::Sound(1498)]);let s=String::from_utf8(RAW.to_vec()).unwrap();for(a,b)in[("[1498","<1498"),("109.effect(1 -1)","109.effect(0 -1)"),("0.4:1","0.4:2")]{assert!(s.contains(a));assert!(qualify_shock87(&catalog(s.replace(a,b).as_bytes())).is_err());}}
 #[test]fn visual_sound_at41_and_terminal246ms_do_not_determine_stun_seconds(){let a=qualify_shock87(&catalog(RAW)).unwrap();let mut p=super::super::native_effect_timing::Clip::new(std::sync::Arc::new(a)).unwrap().start();let mut s=p.advance(0).unwrap();let(mut sound,mut at)=(vec![],0);while !s.finished{for c in &s.crossed_cues{if c.event==FrameEvent::Sound(1498){sound.push(at);}}at=p.next_callback_millis().unwrap();s=p.advance(at*1_000_000).unwrap();}assert_eq!(sound,vec![41]);assert_eq!(at,246);}
}
