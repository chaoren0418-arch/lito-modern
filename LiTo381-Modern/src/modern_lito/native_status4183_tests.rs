use super::*;
use crate::modern_lito::{
    native_world_animation::Animator,
    sprite_actions::{ActionLimits, SpriteActionCatalog},
};

const ID: u32 = 0x71234567;
// Independently transcribed original Text/list.spr definition, not a fake bank.
const RAW:&str="#4183 1 character stun\n3.animation(0 6,0.5:4 0.0:4<1453 0.1:4 0.2:4 0.3:4 0.4:4)\n102.type(0)\n104.attr(8)\n";
fn catalog(raw: &str) -> SpriteActionCatalog {
    SpriteActionCatalog::parse_for_inspection(
        format!("1 0 0\n{raw}").as_bytes(),
        ActionLimits::default(),
    )
    .unwrap()
}
fn world() -> World {
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-world-current.json"
    ))
    .unwrap();
    let mut w = World::default();
    for key in ["world", "start", "status", "actor"] {
        let h = fixture["packets"][key].as_str().unwrap();
        let b: Vec<_> = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect();
        w.accept_at(&b, 0, 0).unwrap();
    }
    w
}
fn put(w: &World) -> Vec<u8> {
    let actor = w.own_actor().unwrap();
    let mut b = vec![0x0b];
    b.extend(actor.x.to_le_bytes());
    b.extend(actor.y.to_le_bytes());
    b.extend(ID.to_le_bytes());
    b.extend(4183u16.to_le_bytes());
    b.extend([0; 4]);
    b.extend(0u32.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    b.extend([0, 0, 0]);
    b.extend(0u32.to_le_bytes());
    b.extend([0, 0, 0, 255, 0, 0, 0]);
    b.extend(0u16.to_le_bytes());
    b.push(0);
    b
}
fn start() -> World {
    let mut w = world();
    w.accept_at(&put(&w), 0, 0).unwrap();
    w
}
fn due(w: &World) -> u64 {
    w.status4183.live[&ID]
        .callbacks
        .first_key_value()
        .unwrap()
        .0
         .0
}
fn frame(w: &World) -> usize {
    w.status4183.live[&ID].frame
}
fn remove(w: &mut World, now: u64) {
    let mut b = vec![0x15];
    b.extend(ID.to_le_bytes());
    assert!(w.accept_at(&b, now, now).unwrap());
}

#[test]
fn ingress_and_dispatch_are_distinct_and_model_delay_never_restarts_warmup() {
    let mut w = world();
    w.accept_at(&put(&w), 20, 100).unwrap();
    assert_eq!(due(&w), 684);
    assert_eq!(w.status4183.live[&ID].received_ms, 20);
    w.poll_status4183(683).unwrap();
    assert!(!w.status4183.live[&ID].active_action);
    w.poll_status4183(684).unwrap();
    assert_eq!(frame(&w), 1);
    let snapshot = w.clone();
    let _model = Stage::default();
    assert_eq!(due(&snapshot), 852);
    assert_eq!(snapshot.status4183.sounds.len(), 1);
    w.poll_status4183(852).unwrap();
    assert_eq!(frame(&w), 2);
    assert_eq!(frame(&snapshot), 1);
}
#[test]
fn canonical_callbacks_follow_584_168_and_terminal_166_without_local_expiry() {
    let mut w = start();
    for (t, f, next) in [
        (584, 1, 752),
        (752, 2, 920),
        (920, 3, 1088),
        (1088, 4, 1256),
        (1256, 5, 1424),
        (1424, 0, 1590),
        (1590, 1, 1758),
    ] {
        w.poll_status4183(t).unwrap();
        assert_eq!(frame(&w), f);
        assert_eq!(due(&w), next);
    }
    let authority = (w.hp, w.mp, w.movement_blocks.clone());
    w.poll_status4183(60_000).unwrap();
    assert_eq!(frame(&w), 2);
    assert_eq!(due(&w), 60_003);
    assert!(w.actors.contains_key(&ID));
    assert_eq!((w.hp, w.mp, w.movement_blocks.clone()), authority);
}
#[test]
fn repeated_poll_at_same_sample_does_not_replay_callbacks() {
    let mut w = start();
    w.poll_status4183(584).unwrap();
    let revision = w.visual_revision;
    w.poll_status4183(584).unwrap();
    assert_eq!(w.visual_revision, revision);
    assert_eq!(w.status4183.sounds.len(), 1);
}
#[test]
fn resend_while_idle_queued_uses_one_ms_and_keeps_original_callback() {
    let mut w = start();
    let old = w.status4183.leases().pop().unwrap();
    w.accept_at(&put(&w), 100, 100).unwrap();
    assert!(!old.load(Ordering::Acquire));
    assert_eq!(due(&w), 101);
    assert_eq!(w.status4183.live[&ID].callbacks.len(), 2);
    w.poll_status4183(101).unwrap();
    assert_eq!(frame(&w), 1);
    w.poll_status4183(269).unwrap();
    assert_eq!(frame(&w), 2);
    w.poll_status4183(437).unwrap();
    assert_eq!(frame(&w), 3);
    w.poll_status4183(584).unwrap();
    assert_eq!(frame(&w), 3);
    assert_eq!(due(&w), 605);
}
#[test]
fn resend_during_animation_preserves_old_frame_and_pending_idle_interleaving() {
    let mut w = start();
    w.poll_status4183(584).unwrap();
    let stale = w.clone();
    w.accept_at(&put(&w), 600, 600).unwrap();
    assert_eq!(due(&w), 752);
    assert!(!w.status4183.live[&ID].active_action);
    assert!(!stale.status4183.leases()[0].load(Ordering::Acquire));
    // Old action0 callback returns0; the queued idle is drained in this sample.
    w.poll_status4183(752).unwrap();
    assert_eq!(frame(&w), 1);
    assert_eq!(due(&w), 920);
    assert_eq!(w.status4183.live[&ID].callbacks.len(), 2);
    assert_eq!(w.status4183.sounds.len(), 1);
}
#[test]
fn resend_at_deadline_is_applied_before_the_controller_batch_drain() {
    let mut w = start();
    w.accept_at(&put(&w), 584, 584).unwrap();
    w.poll_status4183(584).unwrap();
    assert_eq!(frame(&w), 1);
    assert_eq!(due(&w), 585);
    w.poll_status4183(585).unwrap();
    assert_eq!(frame(&w), 1);
    assert_eq!(due(&w), 752);
}
#[test]
fn removal_and_reuse_retire_snapshots_and_start_a_new_queue() {
    let mut w = start();
    w.poll_status4183(584).unwrap();
    let old = w.clone();
    remove(&mut w, 600);
    assert!(w.status4183.live.is_empty());
    assert!(w.status4183.sounds.is_empty());
    assert!(!old.status4183.leases()[0].load(Ordering::Acquire));
    w.accept_at(&put(&w), 700, 700).unwrap();
    assert_eq!(due(&w), 1284);
    w.poll_status4183(752).unwrap();
    assert!(!w.status4183.live[&ID].active_action);
}
#[test]
fn map_reset_retires_old_leases() {
    let mut w = start();
    let old = w.clone();
    w.accept_at(&[0x28, 4, 0, 0], 10, 10).unwrap();
    assert!(w.status4183.live.is_empty());
    assert!(!old.status4183.leases()[0].load(Ordering::Acquire));
}
#[test]
fn malformed_resend_preserves_existing_clock_and_lease() {
    let mut w = start();
    let old = w.status4183.leases()[0].clone();
    let mut b = put(&w);
    b.pop();
    assert!(w.accept_at(&b, 100, 100).is_err());
    assert_eq!(due(&w), 584);
    assert!(old.load(Ordering::Acquire));
    assert!(w.accept_at(&put(&w), 200, 100).is_err());
    assert_eq!(due(&w), 584);
}
#[test]
fn noncanonical_speed_retires_status_without_changing_general_idle_guard() {
    let mut w = start();
    let old = w.status4183.leases()[0].clone();
    let mut b = put(&w);
    b[14] = 1;
    w.accept_at(&b, 100, 100).unwrap();
    assert!(w.status4183.live.is_empty());
    assert!(!old.load(Ordering::Acquire));
    let mut animator = Animator::default();
    assert!(animator
        .pose(&catalog(RAW), &w.actors[&ID], 0)
        .unwrap_err()
        .to_string()
        .contains("idle action family"));
}
#[test]
fn exact_original_action_is_qualified_without_relaxing_other_idle_labels() {
    let a = qualify(&catalog(RAW)).unwrap().remove(0).2;
    assert_eq!(a.frames[0].subframe, 5);
    assert_eq!(a.frames[1].subframe, 0);
    for (from, to) in [
        ("<1453", "[1453"),
        ("104.attr(8)", "104.attr(128)"),
        ("0.4:4", "0.4:5"),
    ] {
        assert!(qualify(&catalog(&RAW.replace(from, to))).is_err());
    }
    let w = start();
    let mut animator = Animator::default();
    assert!(animator.pose(&catalog(RAW), &w.actors[&ID], 0).is_err());
}
#[test]
fn callback_budget_fails_closed_and_clock_regression_is_rejected() {
    let mut w = start();
    let first = w.status4183.leases()[0].clone();
    for _ in 0..64 {
        w.accept_at(&put(&w), 1, 1).unwrap();
    }
    // The overflowing object is isolated: its lease is retired, and it no
    // longer contributes a lease that would reject every later frame.
    assert!(!first.load(Ordering::Acquire));
    assert!(!w.status4183.live.contains_key(&ID));
    assert!(w.status4183.leases().iter().all(|l| l.load(Ordering::Acquire)));
    let fault = &w.status4183.faults[&ID];
    assert!(fault.error.contains("4183 callback budget"), "{fault:?}");
    let mut w = start();
    w.poll_status4183(100).unwrap();
    assert!(w.poll_status4183(99).is_err());
}
fn second_put(w: &World) -> Vec<u8> {
    let mut b = put(w);
    b[5..9].copy_from_slice(&(ID - 1).to_le_bytes());
    b
}
fn overflow(w: &mut World, at: u64) {
    while w.status4183.live.contains_key(&ID) {
        w.accept_at(&put(w), at, at).unwrap();
    }
}
#[test]
fn budget_fault_does_not_reject_current_frames_or_freeze_other_objects() {
    let mut w = start();
    w.accept_at(&second_put(&w), 0, 0).unwrap();
    // A snapshot taken before the fault must still be retired.
    let stale = w.status4183.leases();
    overflow(&mut w, 1);
    assert!(stale.iter().any(|l| !l.load(Ordering::Acquire)));
    // Leases a freshly drawn Frame captures are all current (Frame::status4183_current).
    let current = w.status4183.leases();
    assert_eq!(current.len(), 1);
    assert!(current.iter().all(|l| l.load(Ordering::Acquire)));
    // The other 4183 keeps its own native clock; nothing is silently frozen.
    w.poll_status4183(584).unwrap();
    let other = &w.status4183.live[&(ID - 1)];
    assert!(other.active_action);
    assert_eq!(other.frame, 1);
    assert_eq!(w.status4183.sounds.len(), 1);
    w.poll_status4183(752).unwrap();
    assert_eq!(w.status4183.live[&(ID - 1)].frame, 2);
    assert!(w.status4183.faults.contains_key(&ID));
    assert!(current.iter().all(|l| l.load(Ordering::Acquire)));
}
#[test]
fn faulted_object_is_rebuilt_by_a_new_put_or_cleared_by_removal() {
    let mut w = start();
    overflow(&mut w, 1);
    // Isolated object is excluded from ordinary actor drawing and from 4183 drawing.
    assert!(is_object(&w, &w.actors[&ID]));
    w.poll_status4183(10).unwrap();
    assert!(w.status4183.faults.contains_key(&ID));
    // Same ID rebuilt by a fresh S0B: explicit recovery, new queue, fault cleared.
    w.accept_at(&put(&w), 20, 20).unwrap();
    assert!(w.status4183.faults.is_empty());
    assert_eq!(w.status4183.recovered, 1);
    assert_eq!(due(&w), 604);
    assert_eq!(w.status4183.live[&ID].callbacks.len(), 1);
    w.poll_status4183(604).unwrap();
    assert_eq!(frame(&w), 1);
    // Fault on removal: removing the actor clears the fault without a recovery.
    overflow(&mut w, 700);
    assert!(w.status4183.faults.contains_key(&ID));
    remove(&mut w, 800);
    assert!(w.status4183.faults.is_empty());
    assert_eq!(w.status4183.recovered, 1);
    w.accept_at(&put(&w), 900, 900).unwrap();
    assert_eq!(due(&w), 1484);
    assert_eq!(w.status4183.recovered, 1);
}
#[test]
fn callback_fault_during_poll_isolates_only_that_object() {
    let mut w = start();
    w.accept_at(&second_put(&w), 0, 0).unwrap();
    w.poll_status4183(584).unwrap();
    // A poll callback pops before it schedules, so the budget cannot trip
    // here; drive the arithmetic guard of one object instead.
    w.status4183.live.get_mut(&ID).unwrap().ideal = u64::MAX - 100;
    let stale = w.status4183.live[&ID].lease.clone();
    let revision = w.visual_revision;
    // Runtime-wide Ok: the fault is per object, not a silent global stop.
    w.poll_status4183(752).unwrap();
    assert!(w.visual_revision > revision);
    assert!(!stale.load(Ordering::Acquire));
    assert!(!w.status4183.live.contains_key(&ID));
    assert!(w.status4183.faults[&ID].error.contains("overflow"));
    assert!(w.status4183.sounds.iter().all(|s| s.lease.load(Ordering::Acquire)));
    assert_eq!(w.status4183.live[&(ID - 1)].frame, 2);
    w.poll_status4183(920).unwrap();
    assert_eq!(w.status4183.live[&(ID - 1)].frame, 3);
    assert!(w.status4183.leases().iter().all(|l| l.load(Ordering::Acquire)));
}

#[test]
fn equal_deadline_objects_keep_ingress_order_instead_of_actor_id_order() {
    let mut w = start();
    let mut second = put(&w);
    second[5..9].copy_from_slice(&(ID - 1).to_le_bytes());
    w.accept_at(&second, 0, 0).unwrap();
    w.poll_status4183(584).unwrap();
    assert_eq!(w.status4183.sounds.len(), 2);
    assert!(Arc::ptr_eq(
        &w.status4183.sounds[0].lease,
        &w.status4183.live[&ID].lease
    ));
    assert!(Arc::ptr_eq(
        &w.status4183.sounds[1].lease,
        &w.status4183.live[&(ID - 1)].lease
    ));
}
#[test]
fn source_retirement_and_late_snapshot_cannot_readmit_same_object() {
    let mut w = start();
    let late = w.clone();
    let first = Arc::new(());
    assert!(admit(&w.status4183.live[&ID].presentation, &first));
    let second = Arc::new(());
    assert!(!admit(&late.status4183.live[&ID].presentation, &second));
    drop(first);
    let returned = Arc::new(());
    assert!(!admit(&w.status4183.live[&ID].presentation, &returned));
    w.accept_at(&put(&w), 10, 10).unwrap();
    assert!(admit(&w.status4183.live[&ID].presentation, &returned));
    assert!(!late.status4183.leases()[0].load(Ordering::Acquire));
}

#[test]
#[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; read-only SPR/WAV decode, no playback or GUI"]
fn original_bank_screen_fragments_audio_and_refresh_lease() {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit resource root"),
    );
    let mut w = start();
    let mut stage = Stage::default();
    let cancel = AtomicBool::new(false);
    stage.prepare(&root, &w, &cancel, &mut None);
    assert!(stage.error.is_none(), "{:?}", stage.error);
    let actor = w.own_actor().unwrap();
    let view = super::super::world_map::ViewProjection {
        anchor_cell: super::super::world_map::WorldCell {
            x: actor.x.into(),
            y: actor.y.into(),
        },
        anchor_pixel: super::super::world_map::PixelPoint { x: 400, y: 300 },
    };
    w.poll_status4183(584).unwrap();
    let mut fragments = Vec::new();
    let (_, audio) = stage
        .draw_to(
            &w,
            &mut super::super::native_world_effects::Target::Fragments(&mut fragments),
            view,
        )
        .unwrap();
    assert_eq!(fragments.len(), 1);
    assert!(matches!(
        fragments[0].mix,
        super::super::native_world_effects::Mix::Screen
    ));
    assert_eq!(audio.len(), 1);
    assert_eq!(audio[0].group().candidates(), &[1453]);
    let scope = super::super::native_audio_delivery::Scope::new(1, w.generation, Arc::new(()));
    audio[0].bind(scope.clone()).unwrap();
    let mut history = super::super::native_audio_delivery::FiniteDeliveryHistory::default();
    let mut first = audio.clone();
    let mut duplicate = audio.clone();
    assert_eq!(history.filter_frame(&mut first).unwrap().kept, 1);
    assert_eq!(history.filter_frame(&mut duplicate).unwrap().duplicates, 1);
    assert!(!first[0].consumed());
    w.accept_at(&put(&w), 600, 600).unwrap();
    assert!(!first[0].status_current());
    assert_eq!(
        first[0].submit(&scope, &mut NeverPlay).unwrap(),
        super::super::native_audio_delivery::DeliveryOutcome::Stale
    );
    assert!(!audio[0].discard());
}

struct NeverPlay;
impl super::super::native_audio_delivery::Player for NeverPlay {
    fn play(
        &mut self,
        _: &super::super::combat_audio::SoundBank,
        _: &super::super::combat_animation::NativeAudioGroup,
        _: u32,
    ) -> std::result::Result<
        super::super::combat_audio::PlayOutcome,
        super::super::combat_audio::PoolError,
    > {
        panic!("retired status audio reached playback")
    }
}

#[test]
fn stale_audio_submit_and_deferred_duplicate_are_safe_without_a_device() {
    use super::super::{
        combat_animation::classify_native_cues,
        combat_audio::SoundBank,
        native_audio_delivery::{DeliveryOutcome, FiniteDeliveryHistory, Scope},
        sprite_actions::FrameEvent,
    };
    // Silent selection is an explicit unit fixture, not the original WAV test.
    let group = classify_native_cues(&[FrameEvent::Effect(0)])
        .unwrap()
        .audio
        .unwrap();
    let bank = Arc::new(SoundBank::playback_subset(BTreeMap::new()).unwrap());
    let lease = Arc::new(AtomicBool::new(true));
    let cue = PreparedCue::from_status_event(
        bank,
        group,
        FiniteEventKey {
            sequence: 1,
            graphics: 4183,
            action: 3,
            source_frame: 1,
        },
        lease.clone(),
    )
    .unwrap();
    let scope = Scope::new(1, 1, Arc::new(()));
    cue.bind(scope.clone()).unwrap();
    let mut history = FiniteDeliveryHistory::default();
    let mut first = vec![cue.clone()];
    let mut duplicate = first.clone();
    assert_eq!(history.filter_frame(&mut first).unwrap().kept, 1);
    assert_eq!(history.filter_frame(&mut duplicate).unwrap().duplicates, 1);
    assert!(!cue.consumed());
    lease.store(false, Ordering::Release);
    assert_eq!(
        cue.submit(&scope, &mut NeverPlay).unwrap(),
        DeliveryOutcome::Stale
    );
    assert_eq!(
        first[0].submit(&scope, &mut NeverPlay).unwrap(),
        DeliveryOutcome::AlreadyConsumed
    );
}
