
#[cfg(test)]
mod pending_native_audio_latch_development_tests {
    use super::*;
    use crate::modern_lito::{combat_animation::classify_native_cues,sprite_actions::FrameEvent};
    fn cue()->PreparedCue {let bank=Arc::new(SoundBank::playback_subset(Default::default()).unwrap());let group=classify_native_cues(&[FrameEvent::Sound(0)]).unwrap().audio.unwrap();PreparedCue::from_callback_event(bank,group,0,FiniteEventKey{sequence:42,graphics:167,action:0,source_frame:1},1).unwrap()}
    struct Sink(usize);impl Player for Sink{fn play(&mut self,_:&SoundBank,_:&NativeAudioGroup,_:u32)->std::result::Result<PlayOutcome,PoolError>{self.0+=1;Ok(PlayOutcome::Silent)}}
    fn context()->Scope{Scope::new(1,7,Arc::new(()))}
    #[test]
    fn repeated_same_payload_in_one_frame_does_not_consume_the_kept_item(){
        let original=cue();let scope=context();original.bind(scope.clone()).unwrap();let mut h=FiniteDeliveryHistory::default();h.select_world((1,7,500));let mut frame=vec![original.clone(),original.clone()];let report=h.filter_frame(&mut frame).unwrap();assert_eq!((report.kept,report.duplicates),(1,1));let mut sink=Sink(0);assert!(matches!(frame[0].submit(&scope,&mut sink).unwrap(),DeliveryOutcome::Played(_)));assert_eq!(original.submit(&scope,&mut sink).unwrap(),DeliveryOutcome::AlreadyConsumed);assert_eq!(sink.0,1);
    }
    #[test]
    fn later_frame_duplicates_preserve_pending_original_but_retire_other_payloads(){
        let original=cue();let scope=context();original.bind(scope.clone()).unwrap();let mut h=FiniteDeliveryHistory::default();h.select_world((1,7,500));let mut pending=vec![original.clone()];h.filter_frame(&mut pending).unwrap();let mut clone_frame=vec![original.clone()];assert_eq!(h.filter_frame(&mut clone_frame).unwrap().duplicates,1);assert!(clone_frame.is_empty());let separate=cue();separate.bind(scope.clone()).unwrap();let mut separate_frame=vec![separate.clone()];assert_eq!(h.filter_frame(&mut separate_frame).unwrap().duplicates,1);let mut sink=Sink(0);assert_eq!(separate.submit(&scope,&mut sink).unwrap(),DeliveryOutcome::AlreadyConsumed);assert!(matches!(pending[0].submit(&scope,&mut sink).unwrap(),DeliveryOutcome::Played(_)));assert_eq!(sink.0,1);
    }
    #[test]
    fn weak_history_retains_no_payload_and_expired_key_still_rejects_replay(){
        let original=cue();let mut h=FiniteDeliveryHistory::default();h.select_world((1,7,500));let mut frame=vec![original.clone()];h.filter_frame(&mut frame).unwrap();drop(frame);drop(original);assert!(h.seen.values().next().unwrap().upgrade().is_none());let replay=cue();let scope=context();replay.bind(scope.clone()).unwrap();let mut frame=vec![replay.clone()];assert_eq!(h.filter_frame(&mut frame).unwrap().duplicates,1);assert_eq!(replay.submit(&scope,&mut Sink(0)).unwrap(),DeliveryOutcome::AlreadyConsumed);h.retain_sequences(&Default::default());let mut fresh=vec![cue()];assert_eq!(h.filter_frame(&mut fresh).unwrap().kept,1);
    }
}
