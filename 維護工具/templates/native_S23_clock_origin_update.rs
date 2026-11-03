        self.events.clear();self.non_audio.clear();self.clock_errors.clear();
        let mut impacts = Vec::new();
        for flight in &mut self.flights {
            if flight.finished{continue;}
            let Some(assets)=self.assets.get(&flight.cue.projectile.graphics) else{continue;};
            let result=(||->anyhow::Result<_>{
                let mut sample=flight.playback.advance(native_clock_elapsed_ns(now,flight.start))?;
                if sample.arrived_now {
                    flight.arrived=true;self.arrived+=1;
                    if flight.cue.projectile.graphics==1797 {flight.impact_position=world.actors.get(&flight.cue.target).map(|a|WorldCell{x:i32::from(a.x),y:i32::from(a.y)});}
                    if flight.cue.damage_marker>0&&world.actors.get(&flight.cue.target).is_some_and(|a|!a.dead) {
                        impacts.push(Impact{target:flight.cue.target,sequence:flight.cue.action_sequence,projectile:flight.cue.projectile.id});
                    }
                }
                for cue in std::mem::take(&mut sample.cues) {
                    if let FrameEvent::Sound(sound)=cue.event {
                        let classified=super::combat_animation::classify_native_cues(assets.native_S23_events(cue.graphics,cue.source_frame)?)?;
                        let group=classified.audio.ok_or_else(||anyhow::anyhow!("native S23 audio group absent"))?;
                        let payload=super::native_audio_delivery::PreparedCue::from_callback_event(assets.sound_bank(),group,0,super::native_audio_delivery::FiniteEventKey{sequence:flight.cue.action_sequence,graphics:cue.graphics,action:0,source_frame:cue.source_frame},cue.occurrence)?;
                        self.events.push(EnteredEvent{prepared_audio:Some(payload),actor:flight.cue.attacker,sequence:flight.cue.action_sequence,action:18,frame:cue.source_frame,event:format!("SpellSound({sound})"),sound:Some(sound)});
                    } else {self.non_audio.push(serde_json::json!({"sequence":flight.cue.action_sequence,"graphics":cue.graphics,"source_frame":cue.source_frame,"event":format!("{:?}",cue.event),"authority_callback":false}));}
                }
                flight.finished=sample.finished;flight.sample=Some(sample);Ok(())
            })();
            if let Err(error)=result {flight.finished=true;self.omitted+=1;self.clock_errors.push(serde_json::json!({"projectile":flight.cue.projectile.id,"error":error.to_string()}));}
        }
        impacts
