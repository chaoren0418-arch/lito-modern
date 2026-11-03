
#[cfg(test)]
mod area_fireball_development_tests {
    use super::*;
    const SOURCE:&str="2 0 0\n#171 8 fireball 0.fly(1 5,0.4:1 0.0:1[98 0.1:1 0.2:1 0.3:1) 104.attr(8) 109.effect(5 218)\n#218 1 fireball impact 0.fire(0 11,0.9:1 0.0:4[99 0.0:1 0.1:1 0.2:1 0.3:1 0.4:1 0.5:1 0.6:1 0.7:1 0.8:1) 104.attr(8)\n";
    fn catalog(source:&str)->SpriteActionCatalog { SpriteActionCatalog::parse_for_inspection(source.as_bytes(),super::super::sprite_actions::ActionLimits::default()).unwrap() }
    #[test]
    fn original_fireball_class5_binds_eight_banks_and_its_own_launch_sound() {
        let c=catalog(SOURCE);let flight=qualify_area_fireball_effect(&c).unwrap();let impact=qualify_fire_impact(&c).unwrap();
        assert_eq!(flight.gfx,171);assert_eq!(impact.gfx,218);assert_eq!(flight.sequence.animation.frames.len(),5);assert_eq!(impact.sequence.animation.frames.len(),11);
        assert_eq!(flight.sequence.animation.frames[1].events,vec![FrameEvent::Sound(98)]);assert_eq!(impact.sequence.animation.frames[1].events,vec![FrameEvent::Sound(99)]);
        for heading in 0..8 {let asset=flight.sequence.animation.frame_asset(171,1,heading).unwrap();assert_eq!(asset.image_id,u32::from(heading));assert_eq!(asset.subframe,0);}
    }
    #[test]
    fn fireball_rejects_changed_class_secondary_direction_timing_and_audio() {
        for source in [SOURCE.replace("#171 8","#171 4"),SOURCE.replace("5 218","4 218"),SOURCE.replace("5 218","5 219"),SOURCE.replace("[98","[97"),SOURCE.replace("0.2:1","0.2:2"),SOURCE.replace("0.1:1","0.1:1!")] {
            assert!(qualify_area_fireball_effect(&catalog(&source)).is_err());
        }
    }
    #[test]
    fn fireball_secondary_preserves_original_terminal_dwell_and_rejects_forged_hit() {
        let c=catalog(SOURCE);let impact=qualify_fire_impact(&c).unwrap();let frames=&impact.sequence.animation.frames;
        assert_eq!(frames[0].subframe,9);assert_eq!(frames[0].duration_ticks,1);assert_eq!(frames[1].subframe,0);assert_eq!(frames[1].duration_ticks,4);
        assert!(!frames.iter().any(|f|f.events.contains(&FrameEvent::Hit)));
        for source in [SOURCE.replace("[99","[98"),SOURCE.replace("0.9:1","0.9:2"),SOURCE.replace("0.0:4[99","0.0:4[99!"),SOURCE.replace("#218 1","#218 8")] {
            assert!(qualify_fire_impact(&catalog(&source)).is_err());
        }
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; exact shared SPR/WAV reads, no playback"]
    fn actual_fireball_loads_every_native_heading_and_both_original_wavs() {
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");let assets=LightArrowAssets::load_area_fireball_effect_only(Path::new(&root),&AtomicBool::new(false)).unwrap();
        assert!(assets.casters.is_empty());assert_eq!(assets.sound.ids().collect::<Vec<_>>(),vec![98,99]);assert!(assets.encoded_bytes()<=MAX_ENCODED&&assets.rgba_bytes()<=MAX_RGBA);
        for heading in 0..8 {let flight=assets.frame(&assets.effect,heading,Duration::ZERO).unwrap();assert_eq!(flight.sprite_name,format!("171-{heading}.spr"));assert_eq!(flight.graphics_id,171);assert!(flight.frame.width>0&&flight.frame.height>0);}
        let impact=assets.frame(&assets.impact,0,Duration::ZERO).unwrap();assert_eq!(impact.sprite_name,"218-0.spr");assert_eq!(impact.graphics_id,218);
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; five profiles share a reader, all outputs stay in memory"]
    fn actual_scoped_reader_keeps_class4_and_both_class5_heading_rules_distinct() {
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");let cancel=AtomicBool::new(false);let context=SharedActionCatalog::load(Path::new(&root),&cancel).unwrap();let mut slot=None;
        for gfx in [171,167,1797,1583,1799,171] {
            let assets=LightArrowAssets::load_effect_profile_with_reader(&context,gfx,&cancel,&mut slot).unwrap();
            let expected=if matches!(gfx,171|1797){8}else{4};let loaded=assets.sprites.keys().filter(|key|key.gfx==gfx).count();assert_eq!(loaded,expected);
            if gfx==171 {for (dx,dy) in [(0,-4),(4,-4),(4,0),(4,4),(0,4),(-4,4),(-4,0),(-4,-4)] {
                let path=LightArrowPath::new(super::super::world_map::WorldCell{x:32768,y:32768},super::super::world_map::WorldCell{x:32768+dx,y:32768+dy}).unwrap();
                let frame=assets.flight_frame(&path,Duration::ZERO).unwrap();assert_eq!(frame.sprite_name,format!("171-{}.spr",path.heading()));
                let mut playback=assets.start_effect(&path);let first=playback.advance(Duration::ZERO).unwrap();assert!(first.crossed_cues.iter().any(|c|c.event==FrameEvent::Sound(98)));
            }}
        }
        assert!(slot.is_some());cancel.store(true,Ordering::Release);assert!(LightArrowAssets::load_effect_profile_with_reader(&context,171,&cancel,&mut slot).is_err());assert!(slot.is_none());
    }
}
