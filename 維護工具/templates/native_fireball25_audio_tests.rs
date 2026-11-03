
#[cfg(test)]
mod callback_audio_development_tests {
    use super::*;
    use crate::modern_lito::{combat_animation::classify_native_cues,sprite_actions::FrameEvent};
    fn bank()->Arc<SoundBank>{Arc::new(SoundBank::playback_subset(Default::default()).unwrap())}
    fn group()->NativeAudioGroup{classify_native_cues(&[FrameEvent::Sound(0)]).unwrap().audio.unwrap()}
    fn key()->FiniteEventKey{FiniteEventKey{sequence:19,graphics:171,action:0,source_frame:1}}
    fn cue(n:u64)->PreparedCue{PreparedCue::from_callback_event(bank(),group(),0,key(),n).unwrap()}
    #[test]
    fn callback_occurrences_survive_history_while_same_occurrence_does_not_replay(){
        let mut h=FiniteDeliveryHistory::default();h.select_world((1,2,3));let mut inputs=vec![cue(1),cue(1),cue(6),PreparedCue::from_finite_event(bank(),group(),0,key()).unwrap()];let r=h.filter_frame(&mut inputs).unwrap();assert_eq!((r.kept,r.duplicates,r.retained_keys),(3,1,3));assert_eq!(inputs.len(),3);
        let mut again=vec![cue(6)];assert_eq!(h.filter_frame(&mut again).unwrap().duplicates,1);h.retain_sequences(&Default::default());let mut fresh=vec![cue(6)];assert_eq!(h.filter_frame(&mut fresh).unwrap().kept,1);
    }
    #[test]
    fn callback_payload_clones_share_latch_and_replaced_publication_stays_stale(){
        struct Sink(usize);impl Player for Sink{fn play(&mut self,_:&SoundBank,_:&NativeAudioGroup,_:u32)->std::result::Result<PlayOutcome,PoolError>{self.0+=1;Ok(PlayOutcome::Silent)}}
        let a=cue(1);let scope=Scope::new(1,2,Arc::new(()));a.bind(scope.clone()).unwrap();let mut sink=Sink(0);assert!(matches!(a.submit(&scope,&mut sink).unwrap(),DeliveryOutcome::Played(_)));assert_eq!(a.clone().submit(&scope,&mut sink).unwrap(),DeliveryOutcome::AlreadyConsumed);
        let b=cue(6);b.bind(scope).unwrap();assert_eq!(b.submit(&Scope::new(1,2,Arc::new(())),&mut sink).unwrap(),DeliveryOutcome::Stale);assert_eq!(sink.0,1);
    }
    #[test]
    fn zero_callback_identity_is_rejected_and_scene_change_clears_history(){
        assert!(PreparedCue::from_callback_event(bank(),group(),0,key(),0).is_err());let mut h=FiniteDeliveryHistory::default();h.select_world((1,2,3));let mut first=vec![cue(1)];h.filter_frame(&mut first).unwrap();h.select_world((1,3,3));let mut new_scene=vec![cue(1)];assert_eq!(h.filter_frame(&mut new_scene).unwrap().kept,1);
    }
}
