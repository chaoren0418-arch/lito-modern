//! Prepared native groups retain mode/weighted alternatives through publication.
//! Binding is once-only; cloned event/frame payloads share the delivery latch.
use super::{
    combat_animation::{NativeAudioGroup,NativeAudioMode},
    combat_audio::{CombatVoicePool,PlayOutcome,PoolError,SoundBank,MAX_BANK_BYTES},
};
use anyhow::{ensure,Result};
use std::sync::{Arc,OnceLock,atomic::{AtomicBool,Ordering}};

#[derive(Clone,Debug)]
pub struct Scope {
    connection:u64,
    generation:u64,
    publication:Arc<()>,
}
impl Scope {
    pub fn new(connection:u64,generation:u64,publication:Arc<()>)->Self {
        Self{connection,generation,publication}
    }
    pub fn same_context(&self,other:&Self)->bool {
        self.connection==other.connection && self.generation==other.generation
            && Arc::ptr_eq(&self.publication,&other.publication)
    }
}
/// A finite native event is identified by the received S37/S23 sequence and its
/// catalog source slot. This is not a SPR subframe, wall clock or sound ID.
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) struct FiniteEventKey {
    pub sequence:u64,
    pub graphics:u16,
    pub action:u8,
    pub source_frame:usize,
}
#[derive(Debug)]
struct Payload {
    bank:Arc<SoundBank>,
    group:NativeAudioGroup,
    selection:u32,
    finite_event:Option<FiniteEventKey>,
    occurrence:Option<u64>,
    scope:OnceLock<Scope>,
    consumed:AtomicBool,
    status_lease:OnceLock<Arc<AtomicBool>>,
}
#[derive(Clone,Debug)]
pub struct PreparedCue(Arc<Payload>);

pub trait Player {
    fn play(&mut self,bank:&SoundBank,group:&NativeAudioGroup,selection:u32)
        ->std::result::Result<PlayOutcome,PoolError>;
}
impl Player for CombatVoicePool {
    fn play(&mut self,bank:&SoundBank,group:&NativeAudioGroup,selection:u32)
        ->std::result::Result<PlayOutcome,PoolError> {
        CombatVoicePool::play(self,bank,group,selection)
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum DeliveryOutcome {
    Unbound,
    Stale,
    AlreadyConsumed,
    Played(PlayOutcome),
}
impl PreparedCue {
    pub(crate) fn from_status_event(bank:Arc<SoundBank>,group:NativeAudioGroup,event:FiniteEventKey,lease:Arc<AtomicBool>)->Result<Self> {
        let cue=Self::from_finite_event(bank,group,0,event)?;
        let _=cue.0.status_lease.set(lease);
        Ok(cue)
    }
    pub(crate) fn status_current(&self)->bool {
        self.0.status_lease.get().is_none_or(|lease|lease.load(Ordering::Acquire))
    }
    pub(crate) fn consumed(&self)->bool {self.0.consumed.load(Ordering::Acquire)}
    /// No files, device, RNG, or world authority are accessed here. Every
    /// alternative must already exist in the immutable qualified bank.
    pub fn new(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32)->Result<Self> {
        Self::build(bank,group,selection,None)
    }
    pub(crate) fn from_finite_event(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32,event:FiniteEventKey)->Result<Self> {
        ensure!(event.sequence!=0 && event.graphics!=0,"finite event identity missing");
        Self::build(bank,group,selection,Some(event))
    }
    fn build(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32,finite_event:Option<FiniteEventKey>)->Result<Self> {
        Self::build_occurrence(bank,group,selection,finite_event,None)
    }
    pub(crate) fn from_callback_event(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32,event:FiniteEventKey,occurrence:u64)->Result<Self> {
        ensure!(event.sequence!=0 && event.graphics!=0 && occurrence!=0,"callback event identity missing");
        Self::build_occurrence(bank,group,selection,Some(event),Some(occurrence))
    }
    fn build_occurrence(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32,finite_event:Option<FiniteEventKey>,occurrence:Option<u64>)->Result<Self> {
        ensure!(group.mode!=NativeAudioMode::LegacyCaretDistanceMode,"unported distance audio");
        ensure!((1..=3).contains(&group.candidates().len()),"native audio candidate count");
        ensure!(group.candidates().iter().all(|id|*id<10_000),"native shared audio namespace");
        ensure!(group.candidates().iter().all(|id|*id==0 || bank.get(*id).is_some()),"incomplete native audio group bank");
        ensure!(bank.file_bytes()<=MAX_BANK_BYTES && bank.pcm_bytes()<=MAX_BANK_BYTES,"native prepared audio budget");
        Ok(Self(Arc::new(Payload{bank,group,selection,finite_event,occurrence,scope:OnceLock::new(),consumed:AtomicBool::new(false),status_lease:OnceLock::new()})))
    }
    pub(crate) fn finite_event(&self)->Option<FiniteEventKey> {self.0.finite_event}
    pub fn group(&self)->&NativeAudioGroup {&self.0.group}
    pub fn selection(&self)->u32 {self.0.selection}
    pub fn prepared_sound(&self)->Option<&super::combat_audio::PreparedSound> {
        self.0.group.selected(self.0.selection).and_then(|id|self.0.bank.get(id))
    }
    /// The render completion binds its captured scope before publication. A
    /// cue can never be rebound to a new connection/world/publication identity.
    pub fn bind(&self,scope:Scope)->Result<()> {
        if let Some(existing)=self.0.scope.get() {
            ensure!(existing.same_context(&scope),"native audio scope changed");
            return Ok(());
        }
        if let Err(scope)=self.0.scope.set(scope) {
            ensure!(self.0.scope.get().is_some_and(|s|s.same_context(&scope)),"native audio scope bind raced");
        }
        Ok(())
    }
    /// Focus/invisibility/drop policy consumes the event without playback.
    pub fn discard(&self)->bool {
        self.0.consumed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_ok()
    }
    pub fn submit(&self,current:&Scope,player:&mut impl Player)
        ->std::result::Result<DeliveryOutcome,PoolError> {
        let Some(bound)=self.0.scope.get() else {return Ok(DeliveryOutcome::Unbound)};
        if !self.discard(){return Ok(DeliveryOutcome::AlreadyConsumed)}
        if !self.status_current(){return Ok(DeliveryOutcome::Stale)}
        if !bound.same_context(current){return Ok(DeliveryOutcome::Stale)}
        player.play(&self.0.bank,&self.0.group,self.0.selection).map(DeliveryOutcome::Played)
    }
}

/// The World retains at most128 inputs per finite effect/S23 ring; one render owns its
/// own captured input ring. Keep admission history for their union, so a new
/// Model cannot replay a previously published event. Do not clear on cancel
/// or resource-root ABA within the same connection/world/map identity.
#[derive(Default)]
pub(crate) struct FiniteDeliveryHistory {
    world:Option<(u64,u64,u16)>,
    /// Admitted key -> the payload that was kept for it. Weak: the history
    /// never keeps a bank or PCM alive, and an expired identity still refuses
    /// a replay of its key.
    seen:std::collections::BTreeMap<(FiniteEventKey,Option<u64>),std::sync::Weak<Payload>>,
}
pub(crate) const MAX_FINITE_DELIVERY_HISTORY_KEYS:usize=4096;
#[cfg(test)]
mod batch1_callback_tests {
    use super::*;
    #[test]
    fn distinct_callback_occurrences_survive_but_republished_occurrences_do_not() {
        let bank=Arc::new(SoundBank::playback_subset(Default::default()).unwrap());
        let group=crate::modern_lito::combat_animation::classify_native_cues(
            &[crate::modern_lito::sprite_actions::FrameEvent::Sound(0)]).unwrap().audio.unwrap();
        let key=FiniteEventKey{sequence:9,graphics:171,action:0,source_frame:1};
        let cue=|n|PreparedCue::from_callback_event(bank.clone(),group.clone(),0,key,n).unwrap();
        let mut history=FiniteDeliveryHistory::default();history.select_world((1,2,3));
        let mut first=vec![cue(1),cue(1),cue(6)];
        let report=history.filter_frame(&mut first).unwrap();assert_eq!((report.kept,report.duplicates),(2,1));
        let mut again=vec![cue(6)];assert_eq!(history.filter_frame(&mut again).unwrap().duplicates,1);
        assert!(PreparedCue::from_callback_event(bank.clone(),group.clone(),0,key,0).is_err());
        history.select_world((1,3,3));let mut next=vec![cue(1)];
        assert_eq!(history.filter_frame(&mut next).unwrap().kept,1);
    }
}
/// T028-4 (S23 clock latch): a duplicate that is the canonical payload itself
/// must not consume the latch the canonical cue is still waiting to use.
#[cfg(test)]
mod canonical_latch_tests {
    use super::*;
    fn cue(occurrence:u64)->PreparedCue {
        let bank=Arc::new(SoundBank::playback_subset(Default::default()).unwrap());
        let group=crate::modern_lito::combat_animation::classify_native_cues(
            &[crate::modern_lito::sprite_actions::FrameEvent::Sound(0)]).unwrap().audio.unwrap();
        PreparedCue::from_callback_event(bank,group,0,FiniteEventKey{sequence:9,graphics:167,action:0,source_frame:1},occurrence).unwrap()
    }
    #[test]
    fn republished_canonical_payload_keeps_its_latch() {
        let canonical=cue(1);
        let mut history=FiniteDeliveryHistory::default();history.select_world((1,2,3));
        let mut first=vec![canonical.clone()];
        assert_eq!(history.filter_frame(&mut first).unwrap().kept,1);
        // The same Arc again: twice in one frame and in a later frame.
        let mut again=vec![canonical.clone(),canonical.clone()];
        let report=history.filter_frame(&mut again).unwrap();
        assert_eq!((report.kept,report.duplicates,report.retained_keys),(0,2,1));
        assert!(again.is_empty());
        assert!(!canonical.consumed(),"removing a duplicate schedule consumed the pending canonical cue");
        // The canonical cue is still deliverable exactly once.
        assert!(canonical.discard());
        assert!(!canonical.discard());
    }
    #[test]
    fn different_payload_under_the_same_key_is_discarded_and_weak_identity_holds_nothing() {
        let canonical=cue(5);
        let weak=Arc::downgrade(&canonical.0);
        let mut history=FiniteDeliveryHistory::default();history.select_world((1,2,3));
        let mut first=vec![canonical.clone()];
        assert_eq!(history.filter_frame(&mut first).unwrap().kept,1);
        let other=cue(5);
        let mut second=vec![other.clone()];
        assert_eq!(history.filter_frame(&mut second).unwrap().duplicates,1);
        assert!(other.consumed()&&!canonical.consumed());
        // The history holds no strong reference to the payload (bank/PCM).
        drop(first);drop(canonical);
        assert!(weak.upgrade().is_none());
        // An expired identity still refuses a replay of its key.
        let late=cue(5);
        let mut third=vec![late.clone()];
        assert_eq!(history.filter_frame(&mut third).unwrap().duplicates,1);
        assert!(third.is_empty()&&late.consumed());
        // A distinct movement occurrence of the same source slot is a new cue.
        let reentered=cue(9);
        let mut fourth=vec![reentered.clone()];
        assert_eq!(history.filter_frame(&mut fourth).unwrap().kept,1);
        assert!(!reentered.consumed());
        // Releasing the sequence releases every occurrence key.
        history.retain_sequences(&std::collections::BTreeSet::new());
        let mut fifth=vec![cue(5)];
        assert_eq!(history.filter_frame(&mut fifth).unwrap().kept,1);
    }
}
#[derive(Clone,Copy,Debug,Default,serde::Serialize)]
pub(crate) struct FiniteHistoryReport {pub kept:usize,pub duplicates:usize,pub retained_keys:usize}
impl FiniteDeliveryHistory {
    pub(crate) fn select_world(&mut self,world:(u64,u64,u16)) {
        if self.world!=Some(world) {self.world=Some(world);self.seen.clear();}
    }
    pub(crate) fn is_world(&self,world:(u64,u64,u16))->bool {self.world==Some(world)}
    pub(crate) fn retain_sequences(&mut self,sequences:&std::collections::BTreeSet<u64>) {
        self.seen.retain(|(event,_),_|sequences.contains(&event.sequence));
    }
    pub(crate) fn filter_frame(&mut self,cues:&mut Vec<PreparedCue>)->Result<FiniteHistoryReport> {
        let keys=cues.iter().map(|cue|cue.finite_event().map(|event|(event,cue.0.occurrence)).ok_or_else(||anyhow::anyhow!("finite native cue provenance absent")))
            .collect::<Result<Vec<_>>>()?;
        let unique:std::collections::BTreeSet<_>=keys.iter().copied().collect();
        let additional=unique.iter().filter(|key|!self.seen.contains_key(*key)).count();
        ensure!(self.seen.len().checked_add(additional).is_some_and(|n|n<=MAX_FINITE_DELIVERY_HISTORY_KEYS),"finite native audio history budget exceeded");
        let incoming=std::mem::take(cues);let mut kept=Vec::with_capacity(incoming.len());let mut report=FiniteHistoryReport::default();
        for (cue,key) in incoming.into_iter().zip(keys) {
            if let Some(canonical)=self.seen.get(&key) {
                report.duplicates+=1;
                // A republished frame can carry the SAME payload (same Arc) as
                // the canonical cue that is still waiting to be played.
                // Removing that duplicate schedule must not consume the shared
                // latch; a different payload under the same key is discarded.
                let same_payload=canonical.upgrade().is_some_and(|kept|Arc::ptr_eq(&kept,&cue.0));
                // Status snapshots share the original payload with an already
                // deferred frame. Dropping a duplicate must not consume it.
                if !same_payload && cue.0.status_lease.get().is_none(){cue.discard();}
            } else {
                self.seen.insert(key,Arc::downgrade(&cue.0));report.kept+=1;kept.push(cue);
            }
        }
        *cues=kept;report.retained_keys=self.seen.len();Ok(report)
    }
}

// Scoped ordinary WAV payload; voice policy remains play_legacy.
pub mod legacy {
//! Scope ordinary prepared WAVs without changing their legacy voice policy.
use crate::modern_lito::{
    combat_audio::{CombatVoicePool, PlayOutcome, PoolError, PreparedSound},
    native_audio_delivery::{DeliveryOutcome, Scope},
    native_combat_animation::EnteredEvent,
};
use anyhow::{ensure, Result};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, OnceLock,
};

pub trait LegacyPlayer {
    fn play_legacy(&mut self, sound: &PreparedSound)
        -> std::result::Result<PlayOutcome, PoolError>;
}
impl LegacyPlayer for CombatVoicePool {
    fn play_legacy(
        &mut self,
        sound: &PreparedSound,
    ) -> std::result::Result<PlayOutcome, PoolError> {
        CombatVoicePool::play_legacy(self, sound)
    }
}
#[derive(Debug)]
struct Payload {
    sound: PreparedSound,
    event: Option<EnteredEvent>,
    scope: OnceLock<Scope>,
    consumed: AtomicBool,
}
/// Each entered sound receives a fresh payload. Clones of that event share its
/// latch; a cached WAV reused by another entered event does not share the latch.
#[derive(Clone, Debug)]
pub struct PreparedLegacyCue(Arc<Payload>);
impl PreparedLegacyCue {
    pub fn new(sound: PreparedSound) -> Self {
        Self(Arc::new(Payload {
            sound,
            event: None,
            scope: OnceLock::new(),
            consumed: AtomicBool::new(false),
        }))
    }
    pub fn sound(&self) -> &PreparedSound {
        &self.0.sound
    }
    /// Retain the actual entered event through environment deferral; a later
    /// frame receipt may describe different events from the deferred payload.
    pub fn from_event(sound: PreparedSound, event: &EnteredEvent) -> Result<Self> {
        ensure!(
            event.sound == Some(sound.sound_id()) && event.prepared_audio.is_none(),
            "legacy sound event does not match its prepared WAV"
        );
        ensure!(event.event.len() <= 128, "legacy event label budget");
        Ok(Self(Arc::new(Payload {
            sound,
            event: Some(event.clone()),
            scope: OnceLock::new(),
            consumed: AtomicBool::new(false),
        })))
    }
    pub fn event(&self) -> Option<&EnteredEvent> {
        self.0.event.as_ref()
    }
    pub fn bind(&self, scope: Scope) -> Result<()> {
        if let Some(existing) = self.0.scope.get() {
            ensure!(existing.same_context(&scope), "legacy audio scope changed");
            return Ok(());
        }
        if let Err(scope) = self.0.scope.set(scope) {
            ensure!(
                self.0
                    .scope
                    .get()
                    .is_some_and(|existing| existing.same_context(&scope)),
                "legacy audio scope bind raced"
            );
        }
        Ok(())
    }
    pub fn discard(&self) -> bool {
        self.0
            .consumed
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    pub fn submit(
        &self,
        current: &Scope,
        player: &mut impl LegacyPlayer,
    ) -> std::result::Result<DeliveryOutcome, PoolError> {
        let Some(bound) = self.0.scope.get() else {
            return Ok(DeliveryOutcome::Unbound);
        };
        if !self.discard() {
            return Ok(DeliveryOutcome::AlreadyConsumed);
        }
        if !bound.same_context(current) {
            return Ok(DeliveryOutcome::Stale);
        }
        player
            .play_legacy(&self.0.sound)
            .map(DeliveryOutcome::Played)
    }
}

}
