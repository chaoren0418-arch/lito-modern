
fn qualify_knight90_91(c:&SpriteActionCatalog,gfx:u32)->Result<Animation,HonorAssetError> {
    let(ticks,sound,source,hit):(&[u32],Option<u32>,usize,usize)=match gfx {
        5831=>(&[4,4,4,4,4,4,4,4,4,4,4,4,4,4],Some(3973),1,10),
        5832=>(&[4,2,2,2,3,3,4,4,4,4,4,4,4,4,4,4,4],Some(3974),2,15),
        10710=>(&[3,1,2,3,3,4],None,0,5),
        _=>return Err(HonorAssetError::Unsupported("unreviewed knight90/91 effect")),
    };
    let row=c.sprites.get(&gfx).ok_or(HonorAssetError::Unsupported("knight90/91 effect absent"))?;
    let resolved=c.resolve_action(gfx,ActionKey::new(0))?;let a=resolved.animation;
    let attrs:[(u32,&[u8]);2]=[(102,b"0"),(104,b"8")];
    if row.id!=gfx||row.graphics_id!=gfx||resolved.definition_sprite_id!=gfx||row.image_count!=Some(1)
        ||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=attrs.len()
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=ticks.len()
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{ticks.len()as u32-1}else{n as u32-1}
            ||f.duration_ticks!=ticks[n]||f.events!=if n==hit{vec![FrameEvent::Hit]}else if n==source{sound.map(|s|vec![FrameEvent::Effect(s)]).unwrap_or_default()}else{vec![]}) {
        return Err(HonorAssetError::Unsupported("knight90/91 original class/overlap/Hit/timing changed"));
    }
    Ok(a.clone())
}
