
#[cfg(test)]
mod zombie41_presented_input_development_tests {
    use super::*;
    use super::super::{native_skill_selection::Target,native_world::World,world_map::{WorldCell,PixelPoint,ViewProjection},world_raster::{Raster,Blend},sprite::{SpriteFrame,SpriteFrameFragments,SpriteBlockReference,CompactSpriteBlock},world_depth::NativeDrawCell};
    use std::sync::Arc;
    const LIVE:u32=0xe1234567;const FIRST:u32=0xf1234567;const SECOND:u32=0xf2234567;
    fn fixture()->(World,InputSurface,super::super::native_render_environment::Key) {
        let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"] {let h=f["packets"][key].as_str().unwrap();let b:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}w.learned_skills.insert(41);w.hp=Some([100,100]);w.mp=Some([100,100]);let own=w.own_actor().unwrap().clone();w.actors.retain(|id,_|*id==own.id);for (i,id) in [LIVE,FIRST,SECOND].into_iter().enumerate(){let mut a=own.clone();a.id=id;a.x+=(i as u16)*2+1;a.incarnation+=i as u64+10;a.dead=id!=LIVE;a.action_sequence+=10;w.actors.insert(id,a);}
        let projection=ViewProjection{anchor_cell:WorldCell{x:i32::from(own.x),y:i32::from(own.y)},anchor_pixel:PixelPoint{x:50,y:150}};let mut canvas=Canvas::new(300,240).unwrap();let rgba=[248,0,0,255].repeat(576);let sprite=SpriteFrame{fragments:Some(SpriteFrameFragments{references:vec![SpriteBlockReference{half_x:0,y:0,depth:10,block_id:0}],atlas:Arc::new(vec![CompactSpriteBlock::test_rgba(&rgba)])}),x_offset:0,y_offset:0,width:24,height:24,frame_type:10,unknown1:0,unknown2:0,rgba};let mut raster=Raster::new(projection,&canvas);for (i,id) in [LIVE,FIRST,SECOND].into_iter().enumerate(){let a=&w.actors[&id];raster.sprite(NativeDrawCell::from_world(WorldCell{x:i32::from(a.x),y:i32::from(a.y)}).unwrap(),&sprite,i as u16+1,Blend::Over).unwrap();}let masks=raster.paint(&mut canvas,&AtomicBool::new(false),3).unwrap();let visibility=[LIVE,FIRST,SECOND].into_iter().zip(masks).collect::<Vec<_>>();assert_eq!(super::super::world_raster::DrawVisibility::visible_ids(&visibility,None),vec![LIVE,FIRST,SECOND]);
        let mut bytes=vec![0;32768];bytes.extend(0_u16.to_le_bytes());bytes.resize(bytes.len()+16384,0);bytes.extend([0;16]);let geometry=WorldMap::from_sector_bytes(vec![SectorInput{key:super::super::resources::SharedMapSectorKey{map_id:u32::from(w.map_id.unwrap()),sector_x:0x8000,sector_y:0x8000},bytes}],&AtomicBool::new(false)).unwrap();let navigation=Arc::new(super::super::native_navigation::Navigation::from_geometry(&geometry).unwrap());let env=super::super::native_render_environment::Key{epoch:7,map:w.map_id.unwrap(),mask_argument:0,underwater:false,blind:0,ambient_level:Some(15),weather:[0;3]};let surface=InputSurface{identity:FrameIdentity{map:u32::from(w.map_id.unwrap()),generation:w.generation,caster_incarnation:own.incarnation,movement_key:(own.id,own.motion_revision,own.x,own.y,own.speed,own.brave,own.action_sequence),observer_key:None},stamp:super::super::native_navigation::InputStamp::capture(0,&w),revision:w.visual_revision,visibility,ground_ids:vec![],navigation,projection,minimap_visible:false,environment_key:Some(env)};(w,surface,env)
    }
    #[test]
    fn t_cycle_uses_presented_corpse_pixels_only_when_41_is_selected() {
        let(w,s,e)=fixture();assert_eq!(s.next_target(&w,None,e).unwrap().id,LIVE);let first=s.next_target_for_skill(&w,None,e,Some(41)).unwrap();assert_eq!(first.id,FIRST);let second=s.next_target_for_skill(&w,Some(first),e,Some(41)).unwrap();assert_eq!(second.id,SECOND);assert_eq!(s.next_target_for_skill(&w,Some(second),e,Some(41)).unwrap().id,FIRST);assert!(first.resolve(&w).is_none());assert_eq!(first.resolve_for_skill(&w,41),Some(FIRST));
    }
    #[test]
    fn corpse_cycle_requires_book_and_current_frame_environment_and_owner() {
        let(w,s,e)=fixture();let mut n=w.clone();n.learned_skills.remove(&41);assert!(s.next_target_for_skill(&n,None,e,Some(41)).is_none());let mut n=w.clone();n.generation+=1;assert!(s.next_target_for_skill(&n,None,e,Some(41)).is_none());let mut n=w.clone();n.visual_revision+=1;assert!(s.next_target_for_skill(&n,None,e,Some(41)).is_none());let mut n=w.clone();n.local_corrections+=1;assert!(s.next_target_for_skill(&n,None,e,Some(41)).is_none());let mut wrong=e;wrong.epoch+=1;assert!(s.next_target_for_skill(&w,None,wrong,Some(41)).is_none());let mut blind=e;blind.blind=1;assert!(s.next_target_for_skill(&w,None,blind,Some(41)).is_none());
    }
    #[test]
    fn corpse_cycle_never_infers_hidden_or_ground_targets_from_roster() {
        let(w,mut s,e)=fixture();s.ground_ids.push(FIRST);assert_eq!(s.next_target_for_skill(&w,None,e,Some(41)).unwrap().id,SECOND);s.visibility.clear();assert!(s.next_target_for_skill(&w,None,e,Some(41)).is_none());assert!(w.actors.contains_key(&FIRST));
    }
    #[test]
    fn corpse_cycle_cannot_reuse_live_selection_and_later_lease_revisions() {
        let(mut w,s,e)=fixture();let live=Target::capture(&w,LIVE);assert_eq!(s.next_target_for_skill(&w,live,e,Some(41)).unwrap().id,FIRST);let retained=Target::capture(&w,FIRST).unwrap();w.actors.get_mut(&FIRST).unwrap().action_sequence+=1;assert!(retained.resolve_for_skill(&w,41).is_none());let mut n=w.clone();n.actors.get_mut(&FIRST).unwrap().dead=false;assert!(retained.resolve_for_skill(&n,41).is_none());
    }
}
