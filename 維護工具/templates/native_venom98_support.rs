
fn qualify_venom98(c:&SpriteActionCatalog)->Result<Animation,HonorAssetError> {
    let row=c.sprites.get(&2942).ok_or(HonorAssetError::Unsupported("venom2942 absent"))?;
    let resolved=c.resolve_action(2942,ActionKey::new(0))?;let a=resolved.animation;
    let ticks=[4,7,6,2,2,3,4,4,4];let attrs:[(u32,&[u8]);3]=[(102,b"0"),(104,b"8"),(109,b"1 -1")];
    if row.id!=2942||row.graphics_id!=2942||resolved.definition_sprite_id!=2942||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=3
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=9
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{8}else{n as u32-1}||f.duration_ticks!=ticks[n]
            ||f.events!=if n==1{vec![FrameEvent::Sound(1505)]}else{vec![]}) {
        return Err(HonorAssetError::Unsupported("venom2942 original class/audio/timing changed"));
    }
    Ok(a.clone())
}
