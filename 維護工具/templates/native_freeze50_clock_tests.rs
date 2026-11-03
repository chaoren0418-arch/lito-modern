
#[cfg(test)]
mod freeze50_clock_development_tests {
    use super::*;
    fn source()->Vec<u8> {[
        b"11297 0 0\n".as_slice(),
        include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-freeze50-inspection\20261024-103539-283280\756.definition.txt"),
        include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-freeze50-inspection\20261024-103539-283280\219.definition.txt")].concat()}
    fn catalog(bytes:&[u8])->SpriteActionCatalog {SpriteActionCatalog::parse_for_inspection(bytes,super::super::sprite_actions::ActionLimits::default()).unwrap()}
    fn cell(x:i32,y:i32)->WorldCell {WorldCell{x,y}}
    fn clock(to:WorldCell)->NativeSpellClock {let c=catalog(&source());let f=qualify_freeze50_effect(&c).unwrap();let i=qualify_impact(&c).unwrap();NativeSpellClock::new(LightArrowPath::new(cell(100,100),to).unwrap(),756,219,Arc::new(f.sequence.animation.clone()),Arc::new(i.sequence.animation.clone()),true).unwrap()}
    #[test]
    fn raw_eight_image_definition_stays_silent_and_class4_with_exact_frames() {
        let s=source();let c=catalog(&s);let f=qualify_freeze50_effect(&c).unwrap();assert_eq!(c.sprites[&756].image_count,Some(8));assert_eq!(f.sequence.animation.frames.len(),4);assert!(f.sequence.animation.frames.iter().all(|f|f.events.is_empty()));assert!(qualify_impact(&c).unwrap().sequence.animation.frames.iter().all(|f|f.events.is_empty()));
        let text=String::from_utf8(s).unwrap();for(a,b)in[("4 219","5 219"),("#756  8","#756  4"),("0.0:2","0.0:2[97"),("0.1:2","0.1:3")]{assert!(text.contains(a));assert!(qualify_freeze50_effect(&catalog(text.replace(a,b).as_bytes())).is_err());}
    }
    #[test]
    fn callback_cycles_silently_and_late_poll_moves_only_one_cell() {
        let mut c=clock(cell(112,100));for n in 0..8u64{let s=c.advance(n*60_000_000).unwrap();assert_eq!(s.source_frame,((n+1)%4)as usize);assert!(s.cues.is_empty());assert!(!s.arrived);}
        let s=c.advance(2_000_000_000).unwrap();assert_eq!(s.callbacks,9);assert_eq!(s.position,cell(109,100));assert_eq!(s.next_movement_ms,Some(2060));assert!(s.cues.is_empty());assert_eq!(c.advance(2_059_999_999).unwrap().callbacks,9);
    }
    #[test]
    fn adjacent_secondary_keeps_zero_dwell_and_never_creates_audio_or_hit() {
        let mut c=clock(cell(101,100));let mut s=c.advance(0).unwrap();assert!(s.arrived_now);assert_eq!((s.graphics,s.source_frame),(219,1));assert!(s.cues.is_empty());
        while s.source_frame!=0 {let at=c.impact.next_callback_millis().unwrap();s=c.advance(at*1_000_000).unwrap();assert!(s.cues.is_empty());}let at=c.impact.next_callback_millis().unwrap();assert_eq!(at-c.impact.sampled_native_millis().unwrap(),41);assert!(!c.advance((at-1)*1_000_000).unwrap().finished);assert!(c.advance(at*1_000_000).unwrap().finished);
    }
    #[test]
    fn class4_folds_west_to_bank_two_and_cloned_clock_keeps_cursor() {
        let mut c=clock(cell(94,100));let s=c.advance(0).unwrap();assert_eq!(s.heading,2);assert_eq!(s.position,cell(99,100));c.advance(500_000_000).unwrap();let mut copy=c.clone();let a=c.advance(560_000_000).unwrap();let b=copy.advance(560_000_000).unwrap();assert_eq!((a.position,a.source_frame,a.callbacks),(b.position,b.source_frame,b.callbacks));assert!(a.cues.is_empty()&&b.cues.is_empty());assert!(copy.advance(559_999_999).is_err());
    }
    #[test]
    fn silent_clock_refuses_injected_sound_and_wrong_secondary() {
        let c=catalog(&source());let f=qualify_freeze50_effect(&c).unwrap();let i=qualify_impact(&c).unwrap();let p=LightArrowPath::new(cell(100,100),cell(106,100)).unwrap();let mut injected=f.sequence.animation.clone();injected.frames[1].events.push(FrameEvent::Sound(97));assert!(NativeSpellClock::new(p.clone(),756,219,Arc::new(injected),Arc::new(i.sequence.animation.clone()),true).is_err());assert!(NativeSpellClock::new(p,756,218,Arc::new(f.sequence.animation.clone()),Arc::new(i.sequence.animation.clone()),true).is_err());
    }
}
