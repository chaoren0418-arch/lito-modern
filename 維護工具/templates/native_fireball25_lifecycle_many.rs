
#[derive(Clone)]
struct FireballRecipient {id:u32,incarnation:u64,position:WorldCell}
#[derive(Clone)]
struct FireballState {
    flight:super::light_arrow_effect::NativeAreaFireballFlight,
    position:WorldCell,heading:u8,source_frame:usize,callbacks:u64,
    arrival_ns:Option<u64>,recipients:Vec<FireballRecipient>,
}
fn fireball_path(c:&Cue,w:&World)->Result<super::light_arrow_effect::LightArrowPath> {
    let saved=c.target_origin.context("fireball receipt target origin absent")?;
    // Rebind ONLY the original first incarnation, including a miss. Missing or
    // replaced first targets keep the receipt fallback; never choose second.
    let target=c.targets.first().and_then(|&(id,_,inc)|w.actors.get(&id).filter(|a|Some(a.incarnation)==inc))
        .map(|a|[i32::from(a.x),i32::from(a.y)]).unwrap_or(saved);
    let from=WorldCell{x:i32::from(c.attack.position[0]),y:i32::from(c.attack.position[1])};
    Ok(super::light_arrow_effect::LightArrowPath::new_area_fireball(from,WorldCell{x:target[0],y:target[1]})?)
}
fn fireball_launch(c:&Cue,w:&World,assets:&super::light_arrow_effect::LightArrowAssets,clip:&Clip,now:u64)->Result<Active> {
    let path=fireball_path(c,w)?;
    let position=path.position(std::time::Duration::ZERO);let heading=path.heading();
    Ok(Active{cue:c.clone(),start:now,playback:clip.start(),frame:1,finished:false,
        fireball:Some(FireballState{flight:assets.start_native_area_fireball(path)?,position,heading,source_frame:1,callbacks:0,arrival_ns:None,recipients:vec![]})})
}
fn fireball_recipients(c:&Cue,w:&World)->(Vec<FireballRecipient>,Vec<Impact>) {
    let mut recipients=vec![];let mut impacts=vec![];
    for &(id,hit,incarnation) in &c.targets {
        let Some(actor)=w.actors.get(&id).filter(|a|Some(a.incarnation)==incarnation) else{continue;};
        recipients.push(FireballRecipient{id,incarnation:actor.incarnation,position:WorldCell{x:i32::from(actor.x),y:i32::from(actor.y)}});
        // Shared animator.impact additionally fences active/queued death,
        // transformation and guardian presentation before accepting hurt.
        if hit&&!actor.dead&&actor.walk_action!=8&&!matches!(actor.graphics,1245|1590|2001|5620|2665|2667|2669|2671|2674) {
            impacts.push(Impact{target:id,sequence:c.action_sequence,projectile:c.attack.sequence});
        }
    }
    (recipients,impacts)
}
fn fireball_audio(assets:&super::light_arrow_effect::LightArrowAssets,c:&Cue,gfx:u16,source_frame:usize,occurrence:u64,sound:u32)->Result<EnteredEvent> {
    ensure!(source_frame==1&&occurrence!=0&&matches!((gfx,sound),(171,98)|(218,99)),"fireball cue changed");
    let group=super::combat_animation::classify_native_cues(&[FrameEvent::Sound(sound)])?.audio.context("fireball audio group absent")?;
    let payload=super::native_audio_delivery::PreparedCue::from_callback_event(assets.sound_bank(),group,0,
        super::native_audio_delivery::FiniteEventKey{sequence:c.action_sequence,graphics:gfx,action:0,source_frame},occurrence)?;
    Ok(EnteredEvent{prepared_audio:Some(payload),actor:c.attack.caster,sequence:c.action_sequence,action:0,frame:source_frame,event:format!("ManySound({sound})"),sound:Some(sound)})
}
fn update_fireball(a:&mut Active,w:&World,assets:&super::light_arrow_effect::LightArrowAssets,now:u64,events:&mut Vec<EnteredEvent>,receipts:&mut Vec<Value>)->Result<Vec<Impact>> {
    if !w.actors.get(&a.cue.attack.caster).is_some_and(|actor|actor.incarnation==a.cue.caster_incarnation) {
        a.finished=true;return Ok(vec![]); // Normal retirement must not poison asset preparation.
    }
    let state=a.fireball.as_mut().context("fireball state absent")?;
    let sample=state.flight.advance(now.saturating_sub(a.start))?;
    state.position=sample.position;state.heading=sample.heading;state.source_frame=sample.source_frame;state.callbacks=sample.callbacks;
    let mut impacts=vec![];
    if sample.arrived_now {
        let resolved=fireball_recipients(&a.cue,w);state.recipients=resolved.0;impacts=resolved.1;state.arrival_ns=Some(now);
        receipts.push(json!({"sequence":a.cue.action_sequence,"event":"Class5Arrival","recipients":state.recipients.iter().map(|r|json!({"id":r.id,"incarnation":r.incarnation,"position":[r.position.x,r.position.y]})).collect::<Vec<_>>(),"hurt_candidates":impacts.len(),"authority_callback":false}));
    }
    if let Some(arrival)=state.arrival_ns {
        if state.recipients.is_empty(){a.finished=true;}
        else {
            let impact=a.playback.advance((now/1_000_000).saturating_sub(arrival/1_000_000)*1_000_000)?;a.frame=impact.source_frame;a.finished=impact.finished;
            for cue in impact.crossed_cues {
                ensure!(cue.event==FrameEvent::Sound(99)&&cue.source_frame==1,"fireball secondary event changed");
                // Original 42F4CB starts each recipient separately. Preserve
                // duplicate wire entries and distinct latches, then let the
                // native same-ID voice policy suppress overlapping playback.
                for (n,_) in state.recipients.iter().enumerate(){events.push(fireball_audio(assets,&a.cue,218,cue.source_frame,n as u64+1,99)?);}
            }
        }
    }
    // 42F500 follows every secondary start, including adjacent arrival.
    for cue in sample.flight_cues_after_arrival {
        ensure!(cue.event==FrameEvent::Sound(98),"fireball flight event changed");
        events.push(fireball_audio(assets,&a.cue,171,cue.source_frame,sample.callbacks,98)?);
    }
    Ok(impacts)
}
fn submit_fireball_group<'s>(target:&mut Target<'_,'s>,frame:&'s super::sprite::SpriteFrame,positions:&[WorldCell],view:ViewProjection)->Result<()> {
    ensure!(positions.len()<=128,"fireball target budget");
    for &position in positions {let anchor=view.cell_to_pixel(position).context("fireball group projection")?;anchor.x.checked_add(frame.x_offset).context("fireball group X")?;anchor.y.checked_add(frame.y_offset).context("fireball group Y")?;}
    if let Target::Fragments(queue)=target {ensure!(queue.len().checked_add(positions.len()).is_some_and(|n|n<=512),"fireball group submission budget");ensure!(frame.fragments.is_some(),"fireball group lacks source fragments");}
    for &position in positions{target.submit(frame,position,view,Mix::Screen)?;}
    Ok(())
}
