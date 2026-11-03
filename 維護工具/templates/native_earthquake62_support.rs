fn finite_area_profile(gfx:u16)->bool {class1_area_profile(gfx)||gfx==1812}
fn finite_area_mix(gfx:u16)->Mix {if matches!(gfx,1805|1812){Mix::Over}else{Mix::Screen}}
fn qualify_earthquake(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&1812).ok_or(HonorAssetError::Unsupported("earthquake1812 absent"))?;let resolved=c.resolve_action(1812,ActionKey::new(0))?;let a=resolved.animation;
    let ticks=[2,2,3,3,4,5,5,5,5,5,5,5,48,48,48,48];let attrs:[(u32,&[u8]);3]=[(101,b"1813"),(105,b"1 1814"),(109,b"0 -1")];
    if row.id!=1812||row.graphics_id!=1812||resolved.definition_sprite_id!=1812||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=attrs.len()
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=16
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=n as u32||f.duration_ticks!=ticks[n]||f.events!=match n{1=>vec![FrameEvent::Sound(716)],5=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("earthquake1812 original controls changed"));
    }
    for (id,attrs) in [(1813,vec![(102,b"0".as_slice())]),(1814,vec![(102,b"0".as_slice()),(104,b"8")])] {
        let row=c.sprites.get(&id).ok_or(HonorAssetError::Unsupported("earthquake layer absent"))?;
        if row.id!=id||row.graphics_id!=id||row.image_count!=Some(1)||row.header_issue.is_some()||!row.actions.is_empty()||row.attributes.len()!=attrs.len()
            ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1) {return Err(HonorAssetError::Unsupported("earthquake original layers changed"));}
    }
    Ok(vec![(1812,0,a.clone())])
}
