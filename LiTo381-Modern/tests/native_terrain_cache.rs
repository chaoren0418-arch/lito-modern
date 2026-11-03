use lito_modern::modern_lito::{compositor::Canvas, native_terrain_cache::TerrainCache, scene::CameraRect};
use std::sync::atomic::{AtomicBool, Ordering};
fn pattern(r: CameraRect) -> anyhow::Result<Canvas> {
    let mut c = Canvas::new(r.width, r.height)?;
    let mut raw = Vec::new();
    for y in 0..r.height { for x in 0..r.width {
        let wx = i64::from(r.x) + i64::from(x); let wy = i64::from(r.y) + i64::from(y);
        raw.extend_from_slice(&[wx as u8, wy as u8, (wx ^ wy) as u8, 255]);
    }}
    c.blit_rgba(&raw, r.width, r.height, 0, 0)?; Ok(c)
}
#[test]
fn crop_preserves_stride_edges_and_rejects_overflow() {
    let c=pattern(CameraRect{x:-20,y:35,width:19,height:13}).unwrap();
    for (x,y,w,h) in [(0,0,19,13),(3,2,11,7),(18,12,1,1)] {
        let p=c.crop(x,y,w,h).unwrap(); let expected=pattern(CameraRect{x:-20+x as i32,y:35+y as i32,width:w,height:h}).unwrap();
        assert_eq!(p.rgba(),expected.rgba());
    }
    for (x,y,w,h) in [(0,0,0,3),(19,0,1,1),(0,13,1,1),(u32::MAX,0,2,1),(0,0,u32::MAX,1)] { assert!(c.crop(x,y,w,h).is_err()); }
}
#[test]
fn cache_matches_direct_view_across_hits_edges_rebuilds_and_clear() {
    let cancel=AtomicBool::new(false); let mut cache=TerrainCache::default();
    for (x,y,builds) in [(0,0,1),(24,-12,1),(128,128,1),(129,128,2),(-50,-50,3)] {
        let r=CameraRect{x,y,width:80,height:60};let cached=cache.render_with(r,&cancel,pattern).unwrap();
        assert_eq!(cached.rgba(),pattern(r).unwrap().rgba());assert_eq!(cache.builds,builds);
    }
    assert_eq!(cache.hits,2);assert!(cache.bytes() <= 1280*1280*4);
    cache.clear();assert_eq!(cache.bytes(),0);
    cache.render_with(CameraRect{x:-50,y:-50,width:80,height:60},&cancel,pattern).unwrap();assert_eq!(cache.builds,4);
}
#[test]
fn cancelled_or_failed_rebuild_never_publishes_partial_terrain() {
    let cancel=AtomicBool::new(false);let mut cache=TerrainCache::default();let r=CameraRect{x:0,y:0,width:80,height:60};
    cache.render_with(r,&cancel,pattern).unwrap();let distant=CameraRect{x:500,..r};
    assert!(cache.render_with(distant,&cancel, |_|anyhow::bail!("decoder error")).is_err());assert_eq!(cache.builds,1);
    assert!(cache.render_with(distant,&cancel, |r|{cancel.store(true,Ordering::Release);pattern(r)}).is_err());assert_eq!(cache.builds,1);
    assert!(cache.render_with(r,&cancel, |_|panic!("cancelled hit must not render")).is_err());
    cancel.store(false,Ordering::Release);let kept=cache.render_with(r,&cancel, |_|panic!("previous valid region survives")).unwrap();assert_eq!(kept.rgba(),pattern(r).unwrap().rgba());
    for invalid in [CameraRect{width:1025,..r},CameraRect{x:i32::MIN,..r}] {assert!(cache.render_with(invalid,&cancel, |_|panic!("invalid region")).is_err());}
}
