
#[cfg(test)]mod reduction88_render_development_tests {
 use super::*;
 #[test]
 #[ignore="LITO_SHARED_RESOURCE_ROOT; original4824/3089, received anchor and overlap; memory fragments only"]
 fn original_reduction88_survives_target_departure_and_history_with_one_overlap_cue(){
  let root:std::path::PathBuf=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root").into();let cancel=AtomicBool::new(false);let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
  let mut other=w.own_actor().unwrap().clone();other.id=0x71234567;other.incarnation+=10;other.x+=2;w.actors.insert(other.id,other.clone());let mut b=vec![55];b.extend(other.id.to_le_bytes());b.extend(4824_u16.to_le_bytes());w.accept(&b).unwrap();w.actors.remove(&other.id);let authority=(w.hp,w.mp,w.movement_blocks.clone());let mut e=Effects::default();e.prepare(&root,&w,&cancel);assert!(e.additional_error.is_none());let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(other.x),y:i32::from(other.y)},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};
  let mut fragments=vec![];let(_,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,0);assert_eq!(fragments.len(),1);assert!(matches!(fragments[0].mix,super::super::native_world_effects::Mix::Screen));assert_eq!(fragments[0].position,view.anchor_cell);assert!(audio.is_empty());drop(fragments);
  w.effects.clear();
  for step in 1..=3-1 {let history=e.history();let mut restored=Effects::default();restored.restore_history(history);restored.prepare(&root,&w,&cancel);e=restored;let mut fragments=vec![];let(_,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,step*41_000_000);assert_eq!(fragments.len(),1);assert_eq!(fragments[0].position,view.anchor_cell);if step==3-1{assert_eq!(audio.iter().filter(|v|v.actor==0&&v.sound==Some(3089)&&v.prepared_audio.is_some()&&v.event=="EffectAudioOverlap(3089)").count(),1);}else{assert!(audio.is_empty());}drop(fragments);}
  let history=e.history();let mut e=Effects::default();e.restore_history(history);e.prepare(&root,&w,&cancel);let mut fragments=vec![];let(_,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,(3-1)*41_000_000);assert!(audio.is_empty());assert_eq!((w.hp,w.mp,w.movement_blocks.clone()),authority);
 }
}


#[cfg(test)]mod bounce89_render_development_tests {
 use super::*;
 #[test]
 #[ignore="LITO_SHARED_RESOURCE_ROOT; original4648/1924, received anchor and overlap; memory fragments only"]
 fn original_bounce89_survives_target_departure_and_history_with_one_overlap_cue(){
  let root:std::path::PathBuf=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root").into();let cancel=AtomicBool::new(false);let mut w=super::super::native_login::LoginController::default().snapshot().world.clone();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
  let mut other=w.own_actor().unwrap().clone();other.id=0x71234567;other.incarnation+=10;other.x+=2;w.actors.insert(other.id,other.clone());let mut b=vec![55];b.extend(other.id.to_le_bytes());b.extend(4648_u16.to_le_bytes());w.accept(&b).unwrap();w.actors.remove(&other.id);let authority=(w.hp,w.mp,w.movement_blocks.clone());let mut e=Effects::default();e.prepare(&root,&w,&cancel);assert!(e.additional_error.is_none());let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(other.x),y:i32::from(other.y)},anchor_pixel:super::super::world_map::PixelPoint{x:400,y:300}};
  let mut fragments=vec![];let(_,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,0);assert_eq!(fragments.len(),1);assert!(matches!(fragments[0].mix,super::super::native_world_effects::Mix::Screen));assert_eq!(fragments[0].position,view.anchor_cell);assert!(audio.is_empty());drop(fragments);
  w.effects.clear();
  for step in 1..=4-1 {let history=e.history();let mut restored=Effects::default();restored.restore_history(history);restored.prepare(&root,&w,&cancel);e=restored;let mut fragments=vec![];let(_,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,step*41_000_000);assert_eq!(fragments.len(),1);assert_eq!(fragments[0].position,view.anchor_cell);if step==4-1{assert_eq!(audio.iter().filter(|v|v.actor==0&&v.sound==Some(1924)&&v.prepared_audio.is_some()&&v.event=="EffectAudioOverlap(1924)").count(),1);}else{assert!(audio.is_empty());}drop(fragments);}
  let history=e.history();let mut e=Effects::default();e.restore_history(history);e.prepare(&root,&w,&cancel);let mut fragments=vec![];let(_,audio)=e.draw_to(&w,&mut super::super::native_world_effects::Target::Fragments(&mut fragments),view,(4-1)*41_000_000);assert!(audio.is_empty());assert_eq!((w.hp,w.mp,w.movement_blocks.clone()),authority);
 }
}
