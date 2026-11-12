//! Original 758: one callback cursor and four simultaneous images per cast.
use super::super::{native_attack_many::Active, native_world_effects::{Target,Mix},
    sprite_actions::{SpriteActionCatalog,ActionKey,Animation,Directionality,FrameEvent},
    tower_assets::{SharedEffectBank,HonorAssetError}, world_map::{WorldCell,ViewProjection}};
use anyhow::{ensure,Context,Result};

pub(super) fn qualify(c:&SpriteActionCatalog)->super::super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&758).ok_or(HonorAssetError::Unsupported("tornado missing"))?;
    let a=c.resolve_action(758,ActionKey::new(0))?.animation;
    if row.graphics_id!=758 || row.image_count!=Some(1) || row.header_issue.is_some()
        || row.actions.len()!=1 || row.attributes.len()!=1
        || row.attributes[0].key!=ActionKey::new(104) || row.attributes[0].body_bytes.trim_ascii()!=b"8"
        || a.directionality!=Directionality::Undirected || a.frames_per_second!=24 || a.frames.len()!=20
        || a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0 || f.subframe!=if n==0{19}else{n as u32-1}
            || f.duration_ticks!=3 || f.events!=if n==1{vec![FrameEvent::Sound(125),FrameEvent::Hit]}else{vec![]}) {
        return Err(HonorAssetError::Unsupported("tornado definition changed"));
    }
    Ok(vec![(758,0,a.clone())])
}
pub(super) fn draw<'s>(bank:&'s SharedEffectBank,a:&Active,target:&mut Target<'_,'s>,view:ViewProjection)->Result<serde_json::Value> {
    let visual=bank.frame(758,0,a.frame,a.finished)?;
    let [x,y]=a.cue.attack.position.map(i32::from);
    let positions=[(2,2),(-2,-2),(-2,2),(2,-2)].map(|(dx,dy)|WorldCell{x:x+dx,y:y+dy});
    if !a.finished {
        if let Target::Fragments(queue)=target {ensure!(queue.len()+4<=512,"tornado group budget");ensure!(visual.frame.fragments.is_some(),"tornado fragments missing");}
        for position in positions {
            let p=view.cell_to_pixel(position).context("tornado projection")?;
            p.x.checked_add(visual.frame.x_offset).context("tornado x")?;
            p.y.checked_add(visual.frame.y_offset).context("tornado y")?;
        }
        for position in positions {target.submit(visual.frame,position,view,Mix::Screen)?;}
    }
    Ok(serde_json::json!({"graphics":758,"sequence":a.cue.action_sequence,"positions":positions.iter().map(|p|[p.x,p.y]).collect::<Vec<_>>(),"source_frame":a.frame,"drawn":!a.finished,"finished":a.finished}))
}
