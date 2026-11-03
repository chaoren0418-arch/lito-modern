
/// These class-0 actor effects use <sound> overlap cues, not child sprites.
fn qualify_knight88_89(c:&SpriteActionCatalog,gfx:u32)->Result<Animation,HonorAssetError> {
    let (ticks,sound,source):(&[u32],u32,usize)=match gfx {
        4824=>(&[4,1,1,1,2,2,2,3,3,3,3,4,4,4,4,4,4,4,4],3089,3),
        4648=>(&[4,1,1,1,1,1,1,2,2,2,2,2,2,3,3,3,4,4,4,4,4,4],1924,4),
        _=>return Err(HonorAssetError::Unsupported("unreviewed knight finite effect")),
    };
    let row=c.sprites.get(&gfx).ok_or(HonorAssetError::Unsupported("knight finite effect absent"))?;
    let resolved=c.resolve_action(gfx,ActionKey::new(0))?;let a=resolved.animation;
    let attrs:[(u32,&[u8]);2]=[(104,b"8"),(109,b"0 -1")];
    if row.id!=gfx||row.graphics_id!=gfx||resolved.definition_sprite_id!=gfx||row.image_count!=Some(1)
        ||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=attrs.len()
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=ticks.len()
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{ticks.len()as u32-1}else{n as u32-1}
            ||f.duration_ticks!=ticks[n]||f.events!=if n==source{vec![FrameEvent::Effect(sound)]}else{vec![]}) {
        return Err(HonorAssetError::Unsupported("knight finite effect class/overlap/timing changed"));
    }
    Ok(a.clone())
}
