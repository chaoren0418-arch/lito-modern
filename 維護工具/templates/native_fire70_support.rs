
fn qualify_fire70(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&1819).ok_or(HonorAssetError::Unsupported("firestorm1819 absent"))?;
    let resolved=c.resolve_action(1819,ActionKey::new(0))?;let a=resolved.animation;
    let ticks=[6,3,3,3,4,4,4,4,4,4,4,4,5,5];let attrs:[(u32,&[u8]);2]=[(104,b"8"),(109,b"0 -1")];
    if row.id!=1819||row.graphics_id!=1819||resolved.definition_sprite_id!=1819||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=2
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=14
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{13}else{n as u32-1}||f.duration_ticks!=ticks[n]
            ||f.events!=match n{1=>vec![FrameEvent::Sound(710)],7=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("firestorm1819 original class/audio/timing changed"));
    }
    Ok(vec![(1819,0,a.clone())])
}
