
#[cfg(test)]mod venom98_definition_draft_tests {
    use super::*;
    const RAW:&[u8]=include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-hide97-venom98-inspection\20261024-183937-685740\2942.definition.txt");
    fn catalog(raw:&[u8])->SpriteActionCatalog{SpriteActionCatalog::parse_for_inspection(&[b"11297 0 0\n".as_slice(),raw].concat(),super::super::sprite_actions::ActionLimits::default()).unwrap()}
    #[test]fn venom_original_sound_and_finite_lifetime_are_separate_from_server320seconds() {
        let a=qualify_venom98(&catalog(RAW)).unwrap();assert_eq!(a.frames[1].events,vec![FrameEvent::Sound(1505)]);let mut p=super::super::native_effect_timing::Clip::new(std::sync::Arc::new(a)).unwrap().start();let mut s=p.advance(0).unwrap();assert_eq!(s.crossed_cues[0].event,FrameEvent::Sound(1505));let mut at=0;while !s.finished{at=p.next_callback_millis().unwrap();s=p.advance(at*1_000_000).unwrap();}assert_eq!(at,1496);
        let text=String::from_utf8(RAW.to_vec()).unwrap();for(a,b)in[("[1505","<1505"),("109.effect(1 -1)","109.effect(0 -1)"),("0.0:7","0.0:6")]{assert!(text.contains(a));assert!(qualify_venom98(&catalog(text.replace(a,b).as_bytes())).is_err());}
    }
}
