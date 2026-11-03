
#[cfg(test)]
mod native_S23_clock_development_tests {
    use super::*;
    const DATA:&str="8 0 0\n#167 4 light 0.fly(1 4,0.3:3 0.0:2[97 0.1:3 0.2:2) 104.attr(8) 109.effect(4 219)\n#1583 4 fire 0.fly(1 4,0.3:3 0.0:2[97 0.1:3 0.2:2) 104.attr(8) 109.effect(4 218)\n#1799 4 wind 0.fly(0 5,0.4:2 0.0:2[719 0.1:2 0.2:2 0.3:2) 104.attr(8) 109.effect(4 1800)\n#1797 8 ice 0.fly(1 4,0.3:2 0.0:2[719 0.1:2 0.2:2) 104.attr(8) 109.effect(5 1798)\n#219 1 light impact 0.fire(0 11,0.9:1 0.0:4 0.0:1 0.1:1 0.2:1 0.3:1 0.4:1 0.5:1 0.6:1 0.7:1 0.8:1) 104.attr(8)\n#218 1 fire impact 0.fire(0 11,0.9:1 0.0:4[99 0.0:1 0.1:1 0.2:1 0.3:1 0.4:1 0.5:1 0.6:1 0.7:1 0.8:1) 104.attr(8)\n#1800 1 wind impact 0.fire(0 6,0.5:4 0.0:4[711! 0.1:3 0.2:3 0.3:3 0.4:4) 104.attr(8)\n#1798 1 ice impact 0.fire(0 7,0.6:3 0.0:2[713! 0.1:1 0.2:1 0.3:2 0.4:3 0.5:3) 104.attr(8)\n";
    fn cell(x:i32,y:i32)->WorldCell{WorldCell{x,y}}
    fn sequences(gfx:u16)->(u16,Arc<Animation>,Arc<Animation>){let c=SpriteActionCatalog::parse_for_inspection(DATA.as_bytes(),super::super::sprite_actions::ActionLimits::default()).unwrap();let (f,i)=match gfx {167=>(qualify_effect(&c).unwrap(),qualify_impact(&c).unwrap()),1583=>(qualify_fire_effect(&c).unwrap(),qualify_fire_impact(&c).unwrap()),1799=>(qualify_wind_effect(&c).unwrap(),qualify_wind_impact(&c).unwrap()),1797=>(qualify_ice_effect(&c).unwrap(),qualify_ice_impact(&c).unwrap()),_=>panic!()};(i.gfx as u16,Arc::new(f.sequence.animation.clone()),Arc::new(i.sequence.animation.clone()))}
    fn clock(gfx:u16,to:WorldCell,secondary:bool)->NativeSpellClock{let(i,f,a)=sequences(gfx);NativeSpellClock::new(LightArrowPath::new(cell(100,100),to).unwrap(),gfx,i,f,a,secondary).unwrap()}
    fn sounds(sample:&NativeSpellSample)->Vec<u32>{sample.cues.iter().filter_map(|c|if let FrameEvent::Sound(id)=c.event{Some(id)}else{None}).collect()}
    #[test]
    fn adjacent_native_profiles_keep_class4_and_class5_opposite_cue_order(){
        for(gfx,expected)in[(167,vec![97]),(1583,vec![97,99]),(1799,vec![719,711]),(1797,vec![713,719])]{let mut c=clock(gfx,cell(101,100),true);let s=c.advance(0).unwrap();assert_eq!(sounds(&s),expected);assert!(s.arrived_now&&s.arrived&&!s.finished);assert_eq!(s.source_frame,1);assert!(c.advance(0).unwrap().cues.is_empty());}
    }
    #[test]
    fn late_callback_moves_one_cell_reschedules_now_and_cannot_catch_up(){
        for gfx in [167,1583,1799,1797]{let mut c=clock(gfx,cell(108,100),true);let first=c.advance(0).unwrap();assert_eq!(first.position,cell(101,100));let late=c.advance(1_000_000_000).unwrap();assert_eq!(late.position,cell(102,100));assert_eq!(late.source_frame,2);assert_eq!(late.next_movement_ms,Some(1060));assert_eq!(late.callbacks,2);assert!(!late.arrived);assert_eq!(c.advance(1_059_999_999).unwrap().callbacks,2);assert_eq!(c.advance(1_060_000_000).unwrap().position,cell(103,100));}
    }
    #[test]
    fn flight_selector_cycles_on_movement_not_catalog_tick_durations(){
        for(gfx,count)in[(167,4),(1583,4),(1799,5),(1797,4)]{let mut c=clock(gfx,cell(120,100),true);let mut entries=vec![];for n in 0..10u64{let s=c.advance(n*60_000_000).unwrap();assert_eq!(s.source_frame,((n+1)%count)as usize);assert!(!s.arrived);entries.extend(s.cues.into_iter().map(|v|v.occurrence));}assert_eq!(entries,if count==4{vec![1,5,9]}else{vec![1,6]});}
    }
    #[test]
    fn native_secondaries_keep_source_zero_dwell_before_finishing(){
        for(gfx,dwell)in[(167,41),(1583,41),(1799,166),(1797,125)]{let mut c=clock(gfx,cell(101,100),true);let mut s=c.advance(0).unwrap();while s.source_frame!=0{let due=c.impact.next_callback_millis().unwrap();s=c.advance(due*1_000_000).unwrap();}assert!(!s.finished);let due=c.impact.next_callback_millis().unwrap();assert_eq!(due-c.impact.sampled_native_millis().unwrap(),dwell);assert!(!c.advance((due-1)*1_000_000).unwrap().finished);assert!(c.advance(due*1_000_000).unwrap().finished);}
    }
    #[test]
    fn empty_class5_target_has_no_secondary_or_forged_hit(){let mut c=clock(1797,cell(101,100),false);let s=c.advance(0).unwrap();assert!(s.finished&&s.arrived_now);assert_eq!(s.graphics,1797);assert_eq!(sounds(&s),vec![719]);assert!(!s.cues.iter().any(|v|v.event==FrameEvent::Hit));assert!(c.advance(1_000_000_000).unwrap().cues.is_empty());}
    #[test]
    fn heading_banks_and_cloned_callback_state_survive_late_polls(){
        for gfx in [167,1583,1799,1797]{let mut c=clock(gfx,cell(92,100),true);let a=c.advance(0).unwrap();assert_eq!(a.heading,if gfx==1797{6}else{2});c.advance(500_000_000).unwrap();let mut copy=c.clone();let x=c.advance(560_000_000).unwrap();let y=copy.advance(560_000_000).unwrap();assert_eq!((x.position,x.source_frame,x.callbacks),(cell(97,100),3,3));assert_eq!((x.position,x.source_frame,x.callbacks),(y.position,y.source_frame,y.callbacks));assert!(copy.advance(559_999_999).is_err());}
    }
    #[test]
    fn zero_distance_and_unqualified_profile_are_explicit(){
        let mut c=clock(1583,cell(100,100),true);let s=c.advance(0).unwrap();assert!(s.arrived_now);assert_eq!(sounds(&s),vec![97,99]);let(i,f,a)=sequences(167);assert!(NativeSpellClock::new(LightArrowPath::new(cell(100,100),cell(101,100)).unwrap(),171,i,f,a,true).is_err());
    }
}
