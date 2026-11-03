
#[cfg(test)]
mod weak47_mix_development_tests {
    use super::*;
    #[test]
    fn weakness_attr128_uses_the_same_rgb555_multiply_consumer_as_darkness() {
        let mut canvas=super::super::compositor::Canvas::new(1,1).unwrap();canvas.clear([248,128,64]);
        let frame=super::super::sprite::SpriteFrame{fragments:None,x_offset:0,y_offset:0,width:1,height:1,frame_type:0,unknown1:0,unknown2:0,rgba:vec![128,248,0,255]};
        let view=ViewProjection{anchor_cell:WorldCell{x:0,y:0},anchor_pixel:super::super::world_map::PixelPoint{x:0,y:0}};
        super::super::native_world_effects::Target::Canvas(&mut canvas).submit(&frame,WorldCell{x:0,y:0},view,reviewed_skill_mix(2228)).unwrap();assert_eq!(canvas.rgba(),&[132,132,0,255]);
    }
}
