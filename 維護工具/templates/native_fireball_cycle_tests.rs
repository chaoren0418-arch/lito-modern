
#[cfg(test)]
mod native_fireball_cycle_development_tests {
    use super::*;
    const SOURCE:&str="2 0 0\n#171 8 fireball 0.fly(1 5,0.4:1 0.0:1[98 0.1:1 0.2:1 0.3:1) 104.attr(8) 109.effect(5 218)\n#218 1 impact 0.fire(0 11,0.9:1 0.0:4[99 0.0:1 0.1:1 0.2:1 0.3:1 0.4:1 0.5:1 0.6:1 0.7:1 0.8:1) 104.attr(8)\n";
    fn animation(gfx:u32)->Animation {SpriteActionCatalog::parse_for_inspection(SOURCE.as_bytes(),super::super::sprite_actions::ActionLimits::default()).unwrap().resolve_action(gfx,ActionKey::new(0)).unwrap().animation.clone()}
    fn cell(x:i32,y:i32)->WorldCell {WorldCell{x,y}}
    fn flight(target:WorldCell)->NativeAreaFireballFlight {NativeAreaFireballFlight::new(LightArrowPath::new_area_fireball(cell(100,100),target).unwrap(),Arc::new(animation(171))).unwrap()}
    #[test]
    fn late_dispatch_moves_one_cell_and_reschedules_from_actual_callback() {
        let mut f=flight(cell(104,100));let first=f.advance(0).unwrap();assert_eq!(first.position,cell(101,100));assert_eq!(first.next_callback_ms,Some(60));
        let late=f.advance(1_000_000_000).unwrap();assert_eq!(late.position,cell(102,100));assert_eq!(late.callbacks,2);assert_eq!(late.next_callback_ms,Some(1060));assert!(!late.arrived);
        let same=f.advance(1_000_000_000).unwrap();assert_eq!(same.callbacks,2);assert!(same.flight_cues_after_arrival.is_empty());
        assert_eq!(f.advance(1_059_999_999).unwrap().position,cell(102,100));assert_eq!(f.advance(1_060_000_000).unwrap().position,cell(103,100));
    }
    #[test]
    fn adjacent_arrival_is_reported_before_launch_cues_and_never_replayed() {
        let mut f=flight(cell(101,100));let sample=f.advance(0).unwrap();assert!(sample.arrived_now&&sample.arrived);assert_eq!(sample.next_callback_ms,None);assert_eq!(sample.heading,2);
        let mut secondary=super::super::native_effect_timing::Clip::new(Arc::new(animation(218))).unwrap().start();
        let mut dispatch=secondary.advance(0).unwrap().crossed_cues;dispatch.extend(sample.flight_cues_after_arrival);
        assert_eq!(dispatch.iter().map(|c|c.event.clone()).collect::<Vec<_>>(),vec![FrameEvent::Sound(99),FrameEvent::Sound(98)]);
        let again=f.advance(1_000_000_000).unwrap();assert!(!again.arrived_now&&again.arrived);assert!(again.flight_cues_after_arrival.is_empty());assert_eq!(again.callbacks,1);
    }
    #[test]
    fn zero_distance_has_the_same_immediate_arrival_contract() {
        let mut f=flight(cell(100,100));let s=f.advance(0).unwrap();assert_eq!(s.position,cell(100,100));assert!(s.arrived_now);assert_eq!(s.callbacks,1);assert_eq!(s.flight_cues_after_arrival[0].event,FrameEvent::Sound(98));
    }
    #[test]
    fn secondary_native_terminal_zero_dwells_and_emits_no_hit() {
        let mut p=super::super::native_effect_timing::Clip::new(Arc::new(animation(218))).unwrap().start();
        let first=p.advance(0).unwrap();assert_eq!(first.source_frame,1);assert_eq!(first.crossed_cues[0].event,FrameEvent::Sound(99));assert_eq!(p.next_callback_millis(),Some(166));
        let mut t=166;let mut sample=p.advance(t*1_000_000).unwrap();
        while sample.source_frame!=0 {assert!(sample.crossed_cues.is_empty());t=p.next_callback_millis().unwrap();sample=p.advance(t*1_000_000).unwrap();}
        assert!(!sample.finished);assert_eq!(p.next_callback_millis(),Some(t+41));assert!(!p.advance((t+40)*1_000_000).unwrap().finished);assert!(p.advance((t+41)*1_000_000).unwrap().finished);
    }
    #[test]
    fn fallback_domain_is_scoped_and_frozen_path_does_not_home() {
        assert!(LightArrowPath::new(cell(0,0),cell(-1,-1)).is_err());
        for (from,to) in [(cell(0,0),cell(-1,-1)),(cell(65535,65535),cell(65536,65536))] {let p=LightArrowPath::new_area_fireball(from,to).unwrap();assert_eq!(p.position(Duration::ZERO),to);}
        assert!(LightArrowPath::new_area_fireball(cell(-1,0),cell(0,0)).is_err());assert!(LightArrowPath::new_area_fireball(cell(0,0),cell(-2,0)).is_err());
        let mut f=flight(cell(103,100));f.advance(0).unwrap();let restored=f.clone();assert_eq!(f.advance(60_000_000).unwrap().position,cell(102,100));let mut restored=restored;assert_eq!(restored.advance(60_000_000).unwrap().position,cell(102,100));assert!(restored.advance(59_999_999).is_err());
    }
    #[test]
    fn eight_headings_cycle_one_frame_per_move_and_reenter_sound_source() {
        for (dx,dy,heading) in [(0,-8,0),(8,-8,1),(8,0,2),(8,8,3),(0,8,4),(-8,8,5),(-8,0,6),(-8,-8,7)] {let mut f=flight(cell(100+dx,100+dy));let mut cues=vec![];for n in 0..7 {let s=f.advance(n*60_000_000).unwrap();assert_eq!(s.heading,heading);assert!(!s.arrived);assert_eq!(s.source_frame,((n+1)%5) as usize);cues.extend(s.flight_cues_after_arrival);}assert_eq!(cues.len(),2);assert_eq!(cues.iter().map(|c|(c.tick,c.event.clone())).collect::<Vec<_>>(),vec![(0,FrameEvent::Sound(98)),(5,FrameEvent::Sound(98))]);assert!(f.advance(420_000_000).unwrap().arrived_now);}
    }
    #[test]
    fn unqualified_flight_animation_is_rejected_before_first_callback() {
        let p=LightArrowPath::new_area_fireball(cell(100,100),cell(108,100)).unwrap();
        let mut a=animation(171);a.frames[0].duration_ticks=2;assert!(NativeAreaFireballFlight::new(p.clone(),Arc::new(a)).is_err());
        let mut a=animation(171);a.frames[2].events.push(FrameEvent::Hit);assert!(NativeAreaFireballFlight::new(p.clone(),Arc::new(a)).is_err());
        let mut a=animation(171);a.directionality=Directionality::Undirected;assert!(NativeAreaFireballFlight::new(p,Arc::new(a)).is_err());
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; exact index SPR/WAV reads, no playback or GUI"]
    fn actual_source_frame_api_preserves_eight_banks_terminal_frame_and_profile_fences() {
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");let cancel=AtomicBool::new(false);let a=LightArrowAssets::load_area_fireball_effect_only(Path::new(&root),&cancel).unwrap();
        for heading in 0..8 {for frame in 0..5 {let v=a.native_area_fireball_frame(171,heading,frame,false).unwrap();assert_eq!(v.sprite_name,format!("171-{heading}.spr"));assert_eq!(v.source_frame,frame);assert!(!v.finished);}}
        let terminal=a.native_area_fireball_frame(218,0,0,false).unwrap();assert_eq!(terminal.subframe,9);assert!(!terminal.finished);assert!(a.native_area_fireball_frame(218,0,0,true).unwrap().finished);
        assert!(a.native_area_fireball_frame(171,8,0,false).is_err());assert!(a.native_area_fireball_frame(171,0,5,false).is_err());assert!(a.native_area_fireball_frame(219,0,0,false).is_err());
        let impact=a.native_area_fireball_impact_clip().unwrap();assert_eq!(impact.start().next_callback_millis(),Some(166));assert!(a.start_native_area_fireball(LightArrowPath::new_area_fireball(cell(100,100),cell(101,100)).unwrap()).unwrap().advance(0).unwrap().arrived_now);
        let old=LightArrowAssets::load_effect_only(Path::new(&root),&cancel).unwrap();assert!(old.native_area_fireball_impact_clip().is_err());assert!(old.native_area_fireball_frame(171,0,0,false).is_err());
    }
}
