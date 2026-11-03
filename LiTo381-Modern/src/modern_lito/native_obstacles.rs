//! Dynamic occupancy qualified against the deployed arena object contracts.
use super::native_world::{Actor, World};
pub fn guardian(a:&Actor)->bool {
    (0x6b000000..=0x6bffffff).contains(&a.id)
        && matches!(a.graphics,2665|2667|2669|2671)
        && matches!(a.walk_action,0|32|33|34|35)
}
pub fn destroyed_guardian(a:&Actor)->bool{guardian(a)&&a.walk_action==35}
pub fn blocks(w:&World,x:u16,y:u16)->bool {
    super::native_doors::blocks(w,x,y) || w.actors.values().any(|a|
        guardian(a) && a.walk_action!=35 && a.x==x && a.y==y)
}
