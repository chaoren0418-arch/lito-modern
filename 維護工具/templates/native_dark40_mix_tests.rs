
#[cfg(test)]
mod dark40_reviewed_mix_development_tests {
    use super::*;
    use super::super::native_world_effects::{Mix,Target};
    #[test]
    fn reviewed_darkness_route_preserves_other_finite_profiles_and_light_alias() {
        assert!(matches!(reviewed_skill_mix(2175),Mix::Multiply));assert!(matches!(reviewed_skill_mix(2510),Mix::Over));for &gfx in super::super::native_additional_finite_effects::GRAPHICS {if gfx!=2175 {assert!(matches!(reviewed_skill_mix(gfx),Mix::Screen));}}
    }
    #[test]
    fn dark40_submission_multiplies_covered_pixels_preserves_holes_and_rejects_partial_alpha() {
        let mut c=Canvas::new(3,1).unwrap();c.clear([248,128,64]);let mut f=super::super::sprite::SpriteFrame{fragments:None,x_offset:0,y_offset:0,width:3,height:1,frame_type:0,unknown1:0,unknown2:0,rgba:vec![128,248,0,255,0,0,0,0,0,0,0,255]};let view=ViewProjection{anchor_cell:WorldCell{x:0,y:0},anchor_pixel:super::super::world_map::PixelPoint{x:0,y:0}};Target::Canvas(&mut c).submit(&f,WorldCell{x:0,y:0},view,reviewed_skill_mix(2175)).unwrap();assert_eq!(c.rgba(),&[132,132,0,255,248,128,64,255,0,0,0,255]);let before=c.rgba().to_vec();f.rgba[11]=128;assert!(Target::Canvas(&mut c).submit(&f,WorldCell{x:0,y:0},view,reviewed_skill_mix(2175)).is_err());assert_eq!(c.rgba(),before);
    }
}
