
fn qualify_meteor74(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&762).ok_or(HonorAssetError::Unsupported("meteor762 absent"))?;
    let resolved=c.resolve_action(762,ActionKey::new(0))?;let a=resolved.animation;
    let ticks=[3,1,1,1,1,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3];let attrs:[(u32,&[u8]);2]=[(104,b"8"),(109,b"1 -1")];
    if row.id!=762||row.graphics_id!=762||resolved.definition_sprite_id!=762||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=2
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=20
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{19}else{n as u32-1}||f.duration_ticks!=ticks[n]
            ||f.events!=match n{1=>vec![FrameEvent::Sound(130)],4=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("meteor762 original class/audio/timing changed"));
    }
    Ok(vec![(762,0,a.clone())])
}
