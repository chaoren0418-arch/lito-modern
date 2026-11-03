fn qualify_storm65(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&3924).ok_or(HonorAssetError::Unsupported("storm3924 absent"))?;
    let resolved=c.resolve_action(3924,ActionKey::new(0))?;let a=resolved.animation;
    let ticks=[4,4,4,4,4,4,4,4,4,4,5,5,6];let attrs:[(u32,&[u8]);3]=[(102,b"0"),(104,b"8"),(109,b"1 -1")];
    if row.id!=3924||row.graphics_id!=3924||resolved.definition_sprite_id!=3924||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=attrs.len()
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=13
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{12}else{n as u32-1}||f.duration_ticks!=ticks[n]
            ||f.events!=match n{2=>vec![FrameEvent::Effect(1348)],5=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("storm3924 original class/audio/timing changed"));
    }
    Ok(vec![(3924,0,a.clone())])
}
