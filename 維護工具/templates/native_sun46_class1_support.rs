
fn qualify_sunburst(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&1811).ok_or(HonorAssetError::Unsupported("sunburst absent"))?;
    let action=c.resolve_action(1811,ActionKey::new(0))?;let a=action.animation;
    let ticks=[4,1,1,1,1,1,1,1,1,1,1,4,4,4,4,4,4,4,4];
    if row.graphics_id!=1811 || row.image_count!=Some(1) || row.header_issue.is_some()
        ||row.actions.len()!=1||action.definition_sprite_id!=1811||row.attributes.len()!=2
        ||[(104,b"8".as_slice()),(109,b"1 -1")].iter().any(|(key,body)|
            row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=19
        ||a.frames.iter().enumerate().any(|(i,f)|f.image_id!=0
            ||f.subframe!=if i==0{18}else{i as u32-1}||f.duration_ticks!=ticks[i]
            ||f.events!=match i{1=>vec![FrameEvent::Sound(709)],11=>vec![FrameEvent::Hit],_=>vec![]}) {
        return Err(HonorAssetError::Unsupported("sunburst original definition changed"));
    }
    Ok(vec![(1811,0,a.clone())])
}

/// No decoded sprites, PCM, archive or file ownership crosses Model rebuilds.
#[derive(Clone)]
pub(crate) struct History {
    generation:Option<u64>,seen:BTreeSet<(u32,u64)>,active:Vec<Active>,started:u64,hits:u64,omitted:u64,
}
impl History {
    pub(crate) fn active_sequences(&self)->BTreeSet<u64> {
        self.active.iter().filter(|a|!a.finished).map(|a|a.cue.action_sequence).collect()
    }
    pub(crate) fn merge_admission(&mut self,other:&Self) {
        if self.generation==other.generation{self.seen.extend(other.seen.iter().copied());}
    }
}
impl Effects {
    pub(crate) fn history(&self)->History {History{generation:self.generation,seen:self.seen.clone(),
        active:self.active.clone(),started:self.started,hits:self.hits,omitted:self.omitted}}
    pub(crate) fn restore_history(&mut self,h:History,same_source:bool) {
        self.generation=h.generation;self.seen=h.seen;self.active=if same_source{h.active}else{vec![]};
        self.started=h.started;self.hits=h.hits;self.omitted=h.omitted;
    }
}

fn prepared_class1_sound(bank:&SharedEffectBank,graphics:u16,source:usize,sequence:u64)
    ->anyhow::Result<super::native_audio_delivery::PreparedCue> {
    let animation=bank.animation(u32::from(graphics),0)?;
    let frame=animation.frames.get(source).ok_or_else(||anyhow::anyhow!("class1 audio source absent"))?;
    let group=super::combat_animation::classify_native_cues(&frame.events)?.audio
        .ok_or_else(||anyhow::anyhow!("class1 native audio group absent"))?;
    super::native_audio_delivery::PreparedCue::from_finite_event(bank.sound_bank(),group,0,
        super::native_audio_delivery::FiniteEventKey{sequence,graphics,action:0,source_frame:source})
}
