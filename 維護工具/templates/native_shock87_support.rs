
fn qualify_shock87(c:&SpriteActionCatalog)->Result<Animation,HonorAssetError> {
 let row=c.sprites.get(&4434).ok_or(HonorAssetError::Unsupported("shock4434 absent"))?;let resolved=c.resolve_action(4434,ActionKey::new(0))?;let a=resolved.animation;
 let attrs:[(u32,&[u8]);2]=[(104,b"8"),(109,b"1 -1")];
 if row.id!=4434||row.graphics_id!=4434||resolved.definition_sprite_id!=4434||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=2
  ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
  ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=6
  ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{5}else{n as u32-1}||f.duration_ticks!=1||f.events!=if n==2{vec![FrameEvent::Sound(1498)]}else{vec![]}) {
  return Err(HonorAssetError::Unsupported("shock4434 original class/audio/timing changed"));
 }
 Ok(a.clone())
}
