//! Current formal S_E8 markers, independent of the historical Modern protocol.
use super::{compositor::Canvas, native_navigation::Navigation, native_world::World};
use anyhow::{ensure, Result};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::atomic::{AtomicBool, Ordering}};

pub const MAX_MARKERS: usize = 2048;
pub const RECT: [i32; 4] = [610, 10, 180, 180];
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Marker { pub id: u32, pub x: u16, pub y: u16, pub allied: bool }
pub fn parse(b: &[u8]) -> Result<Marker> {
    ensure!(b.len() == 13 && b[0] == 0xe8, "minimap packet envelope");
    let id = u32::from_le_bytes(b[1..5].try_into()?);
    ensure!(id != 0 && b[5..7] == [0, 0] && b[11] == 1 && b[12] <= 1,
        "unsupported minimap marker fields");
    Ok(Marker { id, x: u16::from_le_bytes(b[7..9].try_into()?),
        y: u16::from_le_bytes(b[9..11].try_into()?), allied: b[12] == 1 })
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct State { pub markers: BTreeMap<u32, Marker>, pub received: u64 }
impl State {
    pub fn accept(&mut self, b: &[u8]) -> Result<bool> {
        let marker = parse(b)?;
        ensure!(self.markers.contains_key(&marker.id) || self.markers.len() < MAX_MARKERS,
            "minimap marker budget");
        let changed = self.markers.get(&marker.id) != Some(&marker);
        self.markers.insert(marker.id, marker);
        self.received = self.received.saturating_add(1);
        Ok(changed)
    }
    pub fn remove(&mut self, id: u32) -> bool { self.markers.remove(&id).is_some() }
}
pub fn contains(x: i32, y: i32) -> bool {
    (RECT[0]..RECT[0] + RECT[2]).contains(&x) && (RECT[1]..RECT[1] + RECT[3]).contains(&y)
}
/// Same isometric orientation as the world; fixed two pixels per east-west cell.
pub fn project(center: [u16; 2], position: [u16; 2]) -> Option<[i32; 2]> {
    let dx = i32::from(position[0]) - i32::from(center[0]);
    let dy = i32::from(position[1]) - i32::from(center[1]);
    let p = [90 + (dx - dy) * 2, 91 + dx + dy];
    ((6..174).contains(&p[0]) && (24..158).contains(&p[1])).then_some(p)
}
fn pixel(rgba: &mut [u8], x: i32, y: i32, c: [u8; 4]) {
    if (0..180).contains(&x) && (0..180).contains(&y) {
        let at = (y as usize * 180 + x as usize) * 4;
        rgba[at..at + 4].copy_from_slice(&c);
    }
}
pub fn draw(world: &World, navigation: &Navigation, canvas: &mut Canvas,
    text: &mut super::native_world_text::Cache, cancel: &AtomicBool) -> Result<Value> {
    ensure!(!cancel.load(Ordering::Acquire), "minimap draw cancelled");
    if !world.view_ready() {return Ok(json!({"visible":false}));}
    let own=world.own_actor().filter(|_|!world.observer_mode);
    let center=if world.observer_mode {world.observer_camera.unwrap()} else {let own=own.unwrap();[own.x,own.y]};
    ensure!(world.map_id.map(u32::from) == Some(navigation.map_id), "minimap geometry mismatch");
    let mut rgba = vec![0; 180 * 180 * 4];
    for y in 0..180 { for x in 0..180 {
        let border = x == 0 || x == 179 || y == 0 || y == 179 || y == 21 || y == 160;
        pixel(&mut rgba, x, y, if border {[118, 109, 88, 255]} else {[12, 19, 24, 235]});
    }}
    let mut terrain_cells = 0;
    // The local diamond samples the original LTmap directional-edge geometry.
    // It does not imply dynamic doors/actors are passable or offer click-to-walk.
    for dy in -64..=64 { for dx in -64..=64 {
        let (x,y) = (i32::from(center[0])+dx, i32::from(center[1])+dy);
        if !(0..=65535).contains(&x) || !(0..=65535).contains(&y) {continue;}
        let Some(p) = project(center,[x as u16,y as u16]) else {continue;};
        if !navigation.contains(x,y) {continue;}
        let c = if navigation.standing(x,y) {[74,92,87,255]} else {[32,43,43,255]};
        pixel(&mut rgba,p[0],p[1],c); pixel(&mut rgba,p[0]+1,p[1],c);terrain_cells+=1;
    }}
    let mut markers = Vec::new();
    for m in world.minimap.markers.values() {
        if own.is_some_and(|a|m.id==a.id) {continue;}
        // A marker can precede its actor packet; keep it but never expose a
        // missing/suppressed/invisible actor until current world state permits it.
        if !world.actor_visible(m.id) {continue;}
        let Some(p) = project(center,[m.x,m.y]) else {continue;};
        let color = if m.allied {[86,224,126,255]} else {[255,159,72,255]};
        for dy in -2i32..=2 { for dx in -2i32..=2 {
            if m.allied || dx.abs()+dy.abs()<=2 {pixel(&mut rgba,p[0]+dx,p[1]+dy,color);}
        }}
        markers.push(json!({"id":m.id,"world":[m.x,m.y],"allied":m.allied,
            "kind":if m.allied {"ally"} else {"other"},"pixel":[p[0]+RECT[0],p[1]+RECT[1]]}));
    }
    // Own marker follows the same displayed local position as the main scene;
    // self movement may be explicitly predicted until corrected by the server.
    for d in -3..=3 {pixel(&mut rgba,90+d,91,[255,255,255,255]);pixel(&mut rgba,90,91+d,[255,255,255,255]);}
    canvas.blit_rgba(&rgba,180,180,RECT[0],RECT[1])?;
    text.get("周邊地圖",cancel)?.draw(canvas,RECT[0]+8,RECT[1]+5,0x7fff,0)?;
    for (x,label,color) in [(8,if world.observer_mode {"鏡頭"} else {"自己"},0x7fff),(60,"同隊",0x2f8f),(112,"其他",0x7e69)] {
        text.get(label,cancel)?.draw(canvas,RECT[0]+x,RECT[1]+165,color,0)?;
    }
    Ok(json!({"visible":true,"rect":RECT,"map_id":navigation.map_id,"generation":world.generation,
        "geometry":"original LTmap directional edges","center":center,"terrain_cells":terrain_cells,
        "own_id":own.map(|a|a.id),"own_position_predicted":own.is_some_and(|a|a.position_predicted),"observer_camera":world.observer_mode,"markers":markers,
        "received":world.minimap.received,"cached":world.minimap.markers.len(),
        "click_through":false,"dynamic_obstacles_shown":false,"offscreen_markers_clamped":false}))
}
