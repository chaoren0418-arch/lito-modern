
fn qualify_freezing80(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
 let row=c.sprites.get(&3933).ok_or(HonorAssetError::Unsupported("freezing3933 absent"))?;let resolved=c.resolve_action(3933,ActionKey::new(0))?;let a=resolved.animation;
 let attrs:[(u32,&[u8]);2]=[(102,b"0"),(104,b"8")];let ticks=[4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,5,5,5,6,6];
 if row.id!=3933||row.graphics_id!=3933||resolved.definition_sprite_id!=3933||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=2
  ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
  ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=31
  ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{30}else{n as u32-1}||f.duration_ticks!=ticks[n]||f.events!=if n==1{vec![FrameEvent::Effect(1353)]}else{vec![]}) {
  return Err(HonorAssetError::Unsupported("freezing3933 original class/audio/timing changed"));
 }
 Ok(vec![(3933,0,a.clone())])
}
