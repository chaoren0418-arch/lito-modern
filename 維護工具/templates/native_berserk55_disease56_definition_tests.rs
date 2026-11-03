
#[cfg(test)]
mod berserk55_disease56_definition_development_tests {
    use super::*;
    fn raw(gfx:u32)->&'static[u8]{match gfx{3943=>include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-berserk55-disease56-inspection\20261024-111546-041737\3943.definition.txt"),2230=>include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-berserk55-disease56-inspection\20261024-111546-041737\2230.definition.txt"),_=>panic!()}}
    fn catalog(b:&[u8])->SpriteActionCatalog{let bytes=[b"11297 0 0\n".as_slice(),b].concat();SpriteActionCatalog::parse_for_inspection(&bytes,super::super::sprite_actions::ActionLimits::default()).unwrap()}
    fn qualify(gfx:u32)->Animation{let c=catalog(raw(gfx));qualify_one(&c,SPECS.iter().find(|s|s.sprite==gfx).unwrap()).unwrap()}
    #[test]
    fn berserk_requires_original_eight_banks_sound_source_two_and_class_zero() {
        let a=qualify(3943);assert_eq!(a.directionality,Directionality::EightWay);assert_eq!(a.frames.len(),27);assert_eq!(a.frames[2].events,vec![FrameEvent::Sound(1361)]);assert!(a.frames[1].events.is_empty());let text=String::from_utf8(raw(3943).to_vec()).unwrap();for(x,y)in[("#3943 8","#3943 1"),("effect(1 27","effect(0 27"),("[1361","[97"),("109.effect(0 -1)","109.effect(1 -1)")]{assert!(text.contains(x));assert!(qualify_one(&catalog(text.replace(x,y).as_bytes()),SPECS.iter().find(|s|s.sprite==3943).unwrap()).is_err());}
    }
    #[test]
    fn disease_requires_undirected_eleven_frames_sound839_and_attr128() {
        let a=qualify(2230);assert_eq!(a.directionality,Directionality::Undirected);assert_eq!(a.frames.len(),11);assert_eq!(a.frames[1].events,vec![FrameEvent::Sound(839)]);let text=String::from_utf8(raw(2230).to_vec()).unwrap();for(x,y)in[("104.attr(128)","104.attr(8)"),("[839","[843"),("0.3:3","0.3:4")]{assert!(qualify_one(&catalog(text.replace(x,y).as_bytes()),SPECS.iter().find(|s|s.sprite==2230).unwrap()).is_err());}
    }
    #[test]
    fn visual_native_clocks_end_at2829_and1375ms_without_status_lifetimes() {
        for(gfx,end)in[(3943,2829),(2230,1375)]{let mut p=super::super::native_effect_timing::Clip::new(std::sync::Arc::new(qualify(gfx))).unwrap().start();let mut sample=p.advance(0).unwrap();let mut at=0;while !sample.finished{at=p.next_callback_millis().unwrap();sample=p.advance(at*1_000_000).unwrap();}assert_eq!(at,end);}
    }
    #[test]
    fn late_berserk_callback_enters_one_frame_then_reschedules_from_actual_time() {
        let mut p=super::super::native_effect_timing::Clip::new(std::sync::Arc::new(qualify(3943))).unwrap().start();assert_eq!(p.advance(0).unwrap().source_frame,1);let sample=p.advance(1_000_000_000).unwrap();assert_eq!(sample.source_frame,2);assert_eq!(sample.crossed_cues.iter().map(|c|c.event.clone()).collect::<Vec<_>>(),vec![FrameEvent::Sound(1361)]);assert_eq!(p.next_callback_millis(),Some(1125));assert!(p.advance(1_000_000_000).unwrap().crossed_cues.is_empty());
    }
}
