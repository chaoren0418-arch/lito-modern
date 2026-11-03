        let mut rows=vec![];
        for f in &self.flights {
            let (Some(assets),Some(sample))=(self.assets.get(&f.cue.projectile.graphics),f.sample.as_ref()) else{continue;};
            let result=(||->anyhow::Result<_>{let secondary=sample.arrived&&f.secondary;
                let visual=assets.native_S23_frame(sample.graphics,if secondary{0}else{sample.heading},sample.source_frame,sample.finished)?;
                let drawn=!f.finished&&!sample.finished;
                let position=if sample.arrived{f.impact_position.unwrap_or(sample.position)}else{sample.position};
                if drawn{target.submit(visual.frame,position,projection,super::native_world_effects::Mix::Screen)?;}
                Ok(serde_json::json!({"sequence":f.cue.action_sequence,"projectile":f.cue.projectile.id,"attacker":f.cue.attacker,"target":f.cue.target,"graphics":visual.graphics_id,"source_frame":sample.source_frame,"sprite":visual.sprite_name,"subframe":visual.subframe,"drawn":drawn,"finished":f.finished||sample.finished,"arrived":sample.arrived,"heading":sample.heading,"position":[position.x,position.y],"secondary":f.secondary,"from":[f.path.from().x,f.path.from().y],"to":[f.path.target().x,f.path.target().y],"elapsed_ns":now.saturating_sub(f.start),"arrival_ns":sample.arrival_ns,"movement_callbacks":sample.callbacks,"next_movement_ms":sample.next_movement_ms}))
            })();rows.push(match result{Ok(row)=>row,Err(error)=>serde_json::json!({"projectile":f.cue.projectile.id,"drawn":false,"error":error.to_string()})});
        }
        self.flights.retain(|f|!f.finished);
        (serde_json::json!({"generation":self.generation,"launched":self.launched,"arrivals":self.arrived,"omitted":self.omitted,"active":self.flights.len(),"non_audio_cues":self.non_audio,"clock_errors":self.clock_errors,"load_error":if self.errors.is_empty(){None}else{Some(&self.errors)},"flights":rows,"entered_events":self.events,"sources":self.assets.values().flat_map(|a|a.sprite_sources()).map(|s|serde_json::json!({"name":s.name,"bytes":s.bytes,"sha256":s.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>()})).collect::<Vec<_>>(),"complete":false}),self.events.clone())
