impl Presentation {
    /// Called by the native window with disjoint controller presentation state.
    /// The old immutable-world tick remains a resource-only inspection API.
    pub(crate) fn tick_environment_with_static(
        &mut self,root:&Path,connection:u64,world:&mut World,
        environment:&mut super::native_environment::State,
        queue:&mut super::native_static2941::Queue,active:bool,
    )->bool {
        let mut changed=self.tick_environment(root,connection,world,environment,active);
        let ready=std::mem::take(&mut self.pending_static_ready);
        let canonical=root.canonicalize().ok();let checked=canonical.as_deref().unwrap_or(root);
        let result=(||->anyhow::Result<()> {
            if active {for proof in ready {changed|=queue.enqueue_prepared(world,connection,checked,proof)?;}}
            let report=queue.drain_due(world,connection,std::time::Instant::now())?;
            changed|=!report.retired_sequences.is_empty();Ok(())
        })();
        if let Err(e)=result {self.error=Some(format!("static resource handoff: {e:#}"));changed=true;}
        changed|=self.prune_static_pixels(world,connection);changed
    }

    fn prune_static_pixels(&mut self,world:&World,connection:u64)->bool {
        if !self.frame.as_ref().is_some_and(|f|!f.static_current(world,connection)){return false;}
        let mut frame=self.frame.take().expect("checked frame");
        self.rendered_revision=None;self.retired_static_frames=self.retired_static_frames.saturating_add(1);
        if self.deferred_sounds.len()+self.deferred_native_audio.len()+self.deferred_legacy_audio.len()
            +frame.sounds.len()+frame.native_audio.len()+frame.legacy_audio.len()>4096 {
            for cue in &frame.legacy_audio{cue.discard();}for cue in &frame.native_audio{cue.discard();}
            self.error=Some("static stale-frame audio backlog exceeded".into());
        }else{
            self.deferred_sounds.append(&mut frame.sounds);self.deferred_legacy_audio.append(&mut frame.legacy_audio);
            self.deferred_native_audio.append(&mut frame.native_audio);
        }
        if self.deferred_finite_hurt.len()+frame.finite_hurt_callbacks.len()<=4096 {
            self.deferred_finite_hurt.append(&mut frame.finite_hurt_callbacks);
        }else{self.error=Some("static stale-frame callback backlog exceeded".into());}
        true
    }
}
