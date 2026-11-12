//! G06 generic finite History, four profiles (Skills/Items/Reagent/Travel).
//! Pure admission state: no sprite, PCM, resource root or server clock is used.
use super::*;
use crate::modern_lito::{
    native_audio_delivery::FiniteDeliveryHistory,
    native_world_scene::FiniteHistory,
    sprite_actions::ActionFrame,
};

fn clip() -> Clip {
    let frame = ActionFrame { image_id: 0, subframe: 0, duration_ticks: 2, events: vec![] };
    Clip::new(Arc::new(Animation {
        label_bytes: Vec::new(), directionality: Directionality::Undirected,
        frames_per_second: 24, frames: vec![frame.clone(), frame],
    })).unwrap()
}

fn active(sequence: u64, graphics: u16) -> Active {
    Active { cue: Effect { sequence, graphics, target: None, position: [32800, 32800] }, start: 0, playback: clip().start() }
}

/// Profile-specific graphics: 744 Skills, 189 Items, 2231 Reagent, 169 Travel.
fn profile(generation: u64, base: u64, graphics: u16) -> History {
    History {
        generation: Some(generation), seen: [base, base + 1, base + 2].into(),
        active: vec![active(base + 1, graphics), active(base + 2, graphics)],
        started: 3, omitted: 1,
    }
}

fn four(generation: u64) -> [History; 4] {
    [profile(generation, 10, 744), profile(generation, 20, 189), profile(generation, 30, 2231), profile(generation, 40, 169)]
}

fn assert_admission_only(h: &History, generation: u64, base: u64) {
    let (g, seen, active, started, omitted) = h.admission();
    assert_eq!(g, Some(generation));
    assert_eq!(seen, &BTreeSet::from([base, base + 1, base + 2]));
    assert_eq!((active, started, omitted), (0, 3, 1));
}

#[test]
fn source_retirement_clears_graph_but_keeps_generation_seen_and_counters() {
    let stored = FiniteHistory::from_profiles(four(7));
    let same = stored.for_source(true);
    assert!(same.restores_other_effects());
    assert_eq!(same.active_finite_sequences(), BTreeSet::from([11, 12, 21, 22, 31, 32, 41, 42]));
    let other = stored.for_source(false);
    assert!(!other.restores_other_effects());
    assert!(other.active_finite_sequences().is_empty(), "retired source kept active audio");
    for (h, base) in other.profiles().iter().zip([10, 20, 30, 40]) { assert_admission_only(h, 7, base); }
}

#[test]
fn root_aba_cannot_revive_a_retired_graph() {
    let mut stored = FiniteHistory::from_profiles(four(7));
    stored.retire_source(); // A -> B, before B's worker completes
    let back = stored.for_source(true); // B -> A
    assert!(!back.restores_other_effects());
    assert!(back.active_finite_sequences().is_empty());
    for (h, base) in back.profiles().iter().zip([10, 20, 30, 40]) { assert_admission_only(h, 7, base); }
}

#[test]
fn late_same_generation_worker_merges_admission_without_its_graph() {
    let mut stored = FiniteHistory::from_profiles(four(7));
    stored.retire_source();
    let mut late = four(7);
    for (h, extra) in late.iter_mut().zip([15, 25, 35, 45]) {
        h.seen.insert(extra); h.active.push(active(extra, 744)); h.started = 4;
    }
    stored.merge_disintegrate_admission(&FiniteHistory::from_profiles(late));
    assert!(stored.active_finite_sequences().is_empty(), "late worker imported its graph");
    for (h, base) in stored.profiles().iter().zip([10, 20, 30, 40]) {
        let (g, seen, active, started, _) = h.admission();
        assert_eq!((g, active, started), (Some(7), 0, 4));
        assert!(seen.contains(&base) && seen.contains(&(base + 5)), "late admission lost");
    }
}

#[test]
fn other_generation_late_worker_contributes_nothing() {
    let mut stored = FiniteHistory::from_profiles(four(7));
    let before = stored.active_finite_sequences();
    let mut late = four(6);
    for h in &mut late { h.seen.insert(99); h.started = 50; }
    stored.merge_disintegrate_admission(&FiniteHistory::from_profiles(late));
    assert_eq!(stored.active_finite_sequences(), before);
    for h in stored.profiles() {
        let (g, seen, _, started, _) = h.admission();
        assert_eq!((g, started), (Some(7), 3));
        assert!(!seen.contains(&99));
    }
}

#[test]
fn retired_admission_restores_into_each_profile_without_relaunch() {
    let retired = FiniteHistory::from_profiles(four(7)).for_source(false);
    let mut effects = [Effects::default(), Effects::items(), Effects::reagent(), Effects::travel()];
    for ((effect, h), base) in effects.iter_mut().zip(retired.profiles().iter().cloned()).zip([10, 20, 30, 40]) {
        effect.restore_history(h);
        assert_admission_only(&effect.history(), 7, base);
        assert!(effect.active.is_empty());
    }
}

#[test]
fn active_sequences_keep_audio_keys_after_raw_ring_eviction() {
    let bank = Arc::new(crate::modern_lito::combat_audio::SoundBank::playback_subset(Default::default()).unwrap());
    let group = crate::modern_lito::combat_animation::classify_native_cues(&[FrameEvent::Sound(0)]).unwrap().audio.unwrap();
    let cue = |sequence| crate::modern_lito::native_audio_delivery::PreparedCue::from_finite_event(bank.clone(), group.clone(), 0,
        crate::modern_lito::native_audio_delivery::FiniteEventKey { sequence, graphics: 169, action: 0, source_frame: 1 }).unwrap();
    let stored = FiniteHistory::from_profiles(four(7));
    let mut audio = FiniteDeliveryHistory::default(); audio.select_world((1, 7, 500));
    let mut first = vec![cue(41)];
    assert_eq!(audio.filter_frame(&mut first).unwrap().kept, 1);
    // The World ring no longer holds 41; the active Travel cue still does.
    audio.retain_sequences(&stored.active_finite_sequences());
    let mut again = vec![cue(41)];
    assert_eq!(audio.filter_frame(&mut again).unwrap().duplicates, 1, "active cue audio key released early");
    // After source retirement no active graph remains, so the key is released.
    audio.retain_sequences(&stored.for_source(false).active_finite_sequences());
    let mut after = vec![cue(41)];
    assert_eq!(audio.filter_frame(&mut after).unwrap().kept, 1);
}
