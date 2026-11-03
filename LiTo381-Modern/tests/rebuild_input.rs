use lito_modern::modern_lito::compositor::Viewport;

#[test]
fn input_matches_letterboxed_rendering_at_varied_scales() {
    for (cw,ch) in [(800,600),(1200,900),(1920,1080),(640,480),(320,900)] {
        let v=Viewport::fit(cw,ch,800,600).unwrap();
        assert_eq!(v.canvas_point(v.x,v.y,800,600),Some((0,0)));
        assert!(v.canvas_point(v.x-1,v.y,800,600).is_none());
        assert!(v.canvas_point(v.x,v.y-1,800,600).is_none());
        assert!(v.canvas_point(v.x+v.width,v.y,800,600).is_none());
        assert!(v.canvas_point(v.x,v.y+v.height,800,600).is_none());
        let p=v.canvas_point(v.x+v.width-1,v.y+v.height-1,800,600).unwrap();
        assert!((0..800).contains(&p.0) && (0..600).contains(&p.1));
    }
}

#[test]
fn minimized_and_extreme_signed_coordinates_cannot_wrap_into_a_hit() {
    assert!(Viewport::fit(0,0,800,600).is_none());
    let v=Viewport{x:i32::MIN,y:i32::MIN,width:i32::MAX,height:i32::MAX};
    assert_eq!(v.canvas_point(i32::MIN,i32::MIN,800,600),Some((0,0)));
    assert!(v.canvas_point(i32::MAX,i32::MAX,800,600).is_none());
    assert!(v.canvas_point(i32::MIN,i32::MIN,u32::MAX,600).is_none());
    assert!(Viewport{x:0,y:0,width:0,height:0}.canvas_point(0,0,800,600).is_none());
}
