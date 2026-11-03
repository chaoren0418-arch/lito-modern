
#[cfg(test)]
mod class1_origin_development_tests {
    use super::*;
    use super::super::{native_combat::Projectile,sprite_actions::ActionLimits};
    const CASTER:u32=0xfedcba98;const TARGET:u32=2;
    fn world()->World {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let fixture:Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();
        for name in ["world","start","status","actor","bot"] {let hex=fixture["packets"][name].as_str().unwrap();let bytes:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();w.accept(&bytes).unwrap();}w
    }
    fn cue(w:&World,gfx:u16,heading:u8)->ProjectileCue {ProjectileCue{heading,attacker_incarnation:w.actors[&CASTER].incarnation,target_incarnation:Some(w.actors[&TARGET].incarnation),action_sequence:99,attacker:CASTER,target:TARGET,damage_marker:0x20,projectile:Projectile{id:0xf1234567,flight_type:0,graphics:gfx,from:[32710,32720],to:[32730,32797]}}}
    fn launch()->EnteredEvent{EnteredEvent{actor:CASTER,sequence:99,action:18,frame:1,event:"Hit".into(),sound:None,prepared_audio:None}}
    fn vector_effects(gfx:u16)->Effects {
        // Synthetic client callback clock only; no original resource/Java qualification.
        let catalog=SpriteActionCatalog::parse_for_inspection(b"1 0 0\n#999 1 vector 0.fire(0 3,0.2:1 0.0:1 0.1:1!)\n",ActionLimits::default()).unwrap();let animation=catalog.resolve_action(999,ActionKey::new(0)).unwrap().animation.clone();let mut e=Effects::default();e.clips.insert(gfx,Clip::new(Arc::new(animation)).unwrap());e
    }
    #[test]
    fn current_class1_wire_destination_does_not_enter_special_two_step_branch(){
        for gfx in [252,236,1801]{assert!(supports_graphics(gfx));}
        assert!(!supports_graphics(2483));assert!(!supports_graphics(2484));
        for heading in 0..8 {for wire in [[32730,32797],[0,0],[u16::MAX,u16::MAX],[0,u16::MAX]] {assert_eq!(position(wire,heading).unwrap(),WorldCell{x:i32::from(wire[0]),y:i32::from(wire[1])});}}
        assert!(position([32730,32797],8).is_err());assert!(position([32730,32797],255).is_err());
    }
    #[test]
    fn class1_launch_freezes_wire_position_across_later_actor_motion(){
        for gfx in [252,236,1801] {for heading in 0..8 {let mut w=world();w.projectiles.push(cue(&w,gfx,heading));let event=launch();let before=serde_json::to_value(&w).unwrap();let mut e=vector_effects(gfx);assert!(e.update(&w,[&event].into_iter(),0).is_empty());assert_eq!(e.active[0].position,WorldCell{x:32730,y:32797});assert_eq!(serde_json::to_value(&w).unwrap(),before);
            w.actors.get_mut(&TARGET).unwrap().x+=10;w.actors.get_mut(&CASTER).unwrap().y+=10;e.update(&w,std::iter::empty(),50_000_000);assert_eq!(e.active[0].position,WorldCell{x:32730,y:32797});assert_eq!(e.started,1);
        }}
    }
    #[test]
    fn class1_unshifted_visual_cannot_give_a_replacement_target_old_hurt(){
        for gfx in [252,236,1801] {let mut w=world();w.projectiles.push(cue(&w,gfx,3));let event=launch();let mut e=vector_effects(gfx);e.update(&w,[&event].into_iter(),0);w.actors.get_mut(&TARGET).unwrap().incarnation+=1;let before=serde_json::to_value(&w).unwrap();assert!(e.update(&w,std::iter::empty(),50_000_000).is_empty());assert_eq!(e.active[0].position,WorldCell{x:32730,y:32797});assert_eq!(serde_json::to_value(&w).unwrap(),before);}
    }
    #[test]
    fn class1_scene_retirement_drops_old_fixed_position_and_callback(){
        let mut w=world();w.projectiles.push(cue(&w,252,7));let event=launch();let mut e=vector_effects(252);e.update(&w,[&event].into_iter(),0);assert_eq!(e.active.len(),1);w.generation+=1;w.projectiles.clear();assert!(e.update(&w,[&event].into_iter(),100_000_000).is_empty());assert!(e.active.is_empty());assert!(e.seen.is_empty());assert_eq!(e.started,0);
    }
    #[test]
    #[ignore="LITO_SHARED_RESOURCE_ROOT; original three profiles/layers draw in memory, no playback"]
    fn actual_class1_sprite_layers_are_submitted_at_literal_wire_destination(){
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        for (gfx,skill) in [(252,10),(236,28),(1801,16)] {for heading in 0..8 {let mut w=world();w.learned_skills.clear();w.learned_skills.insert(skill);w.projectiles.push(cue(&w,gfx,heading));let event=launch();let mut e=Effects::default();e.prepare(Path::new(&root),&w,&AtomicBool::new(false));assert!(e.error.is_none(),"{:?}",e.error);e.update(&w,[&event].into_iter(),0);
            let projection=ViewProjection{anchor_cell:WorldCell{x:32730,y:32797},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};let mut fragments=Vec::new();let(report,_)=e.draw_to(&mut super::super::native_world_effects::Target::Fragments(&mut fragments),projection,0);assert!(report["load_error"].is_null(),"{report}");assert_eq!(report["effects"][0]["position"],json!([32730,32797]));assert!(!fragments.is_empty());assert!(fragments.iter().all(|item|item.position==WorldCell{x:32730,y:32797}));
            if gfx==1801 {assert_eq!(report["effects"][0]["layers"].as_array().unwrap().len(),2);}
        }}
    }
}
