//! The twelve historical ready gates, rewritten against the main queue API.
use super::*;

fn bank() -> (PathBuf, Arc<Static2941Bank>) {
    let root =
        PathBuf::from(std::env::var_os("LITO_STATIC2941_ROOT").expect("explicit original root"));
    let catalog = SharedActionCatalog::load_for_model(&root, &AtomicBool::new(false)).unwrap();
    let bank = Arc::new(Static2941Bank::load(&catalog, &AtomicBool::new(false)).unwrap());
    (root, bank)
}
fn ready(
    queue: &mut Queue,
    world: &mut World,
    lease: &Lease,
    root: &Path,
    bank: Arc<Static2941Bank>,
    sample: u32,
) {
    let proof = Prepared::from_bank(bank, lease, &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert!(queue
        .enqueue_prepared(world, 1, root, proof, || Ok(sample))
        .unwrap());
}

#[test]
#[ignore = "LITO_STATIC2941_ROOT; actual SPR bank"]
fn distinct_decode_dispatch_constructor_enqueue_samples_and_zero_deadline_are_bound() {
    let (root, bank) = bank();
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    assert!(lease.schedule().is_none());
    ready(&mut queue, &mut world, &lease, &root, bank, 73);
    let schedule = lease.schedule().unwrap();
    assert!(lease.ingress().decoded_at() <= lease.dispatched_at());
    assert!(lease.dispatched_at() <= lease.constructed_at());
    assert!(
        lease.constructed_at() <= schedule.ready_at && schedule.ready_at <= schedule.enqueued_at
    );
    assert_eq!(schedule.deadline_ms, 73); // fresh native sample + zero, not Instant
    assert!(queue.drain_due(&mut world, 1, 72).unwrap().is_empty());
    assert!(lease.drawable());
    assert_eq!(
        queue.drain_due(&mut world, 1, 73).unwrap(),
        [lease.sequence()]
    );
}

#[test]
#[ignore = "LITO_STATIC2941_ROOT; actual SPR bank"]
fn copied_world_observer_is_revoked_by_controller_drain_without_mutating_the_copy() {
    let (root, bank) = bank();
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    ready(&mut queue, &mut world, &lease, &root, bank, 7);
    let copy = world.clone();
    let guard = DrawGuard(vec![lease.clone()]);
    assert!(guard.current(&copy, 1));
    let revision = copy.visual_revision;
    assert_eq!(
        queue.drain_due(&mut world, 1, 7).unwrap(),
        [lease.sequence()]
    );
    assert_eq!(copy.visual_revision, revision);
    assert_eq!(world.visual_revision, revision + 1);
    assert!(!guard.current(&copy, 1));
}

#[test]
#[ignore = "LITO_STATIC2941_ROOT; actual SPR bank"]
fn equal_or_earlier_deadlines_keep_admission_order_in_the_callback_receipt() {
    let (root, bank) = bank();
    let (mut world, mut queue) = (world(), Queue::default());
    let a = receive(&mut queue, &mut world, 1);
    let b = receive(&mut queue, &mut world, 2);
    let c = receive(&mut queue, &mut world, 3);
    ready(&mut queue, &mut world, &a, &root, bank.clone(), 7);
    ready(&mut queue, &mut world, &b, &root, bank.clone(), 7);
    ready(&mut queue, &mut world, &c, &root, bank, 8);
    assert_eq!(
        queue.drain_due(&mut world, 1, 9).unwrap(),
        [a.sequence(), b.sequence(), c.sequence()]
    );
}

#[test]
fn ring_eviction_revokes_only_removed_observers_and_keeps_queue_bounded() {
    let (mut world, mut queue) = (world(), Queue::default());
    let first = receive(&mut queue, &mut world, 1);
    let second = receive(&mut queue, &mut world, 2);
    for counter in 3..=129 {
        receive(&mut queue, &mut world, counter);
    }
    assert!(!first.active() && second.active());
    assert_eq!(world.static_effect_receipts.len(), 128);
    assert_eq!(queue.pending.len(), 128);
    assert!(queue.drain_due(&mut world, 1, u32::MAX).unwrap().is_empty());
}

#[test]
fn owner_change_and_queue_drop_revoke_old_snapshots() {
    for change_owner in [false, true] {
        let (mut world, mut queue) = (world(), Queue::default());
        let lease = receive(&mut queue, &mut world, 1);
        let snapshot = world.clone();
        if change_owner {
            queue.synchronize(&world, 2);
        }
        drop(queue);
        assert!(!lease.current(&snapshot, 1));
    }
}

#[test]
fn retired_or_already_scheduled_receipt_cannot_create_another_lifetime() {
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    assert!(!queue
        .admit_received(&mut world, 1, lease.ingress(), Instant::now())
        .unwrap());
    queue.retire();
    assert!(!queue
        .admit_received(&mut world, 1, lease.ingress(), Instant::now())
        .unwrap());
    assert!(!lease.active());
}

#[test]
fn wrong_connection_is_rejected_before_revoking_the_current_owner() {
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    assert!(queue
        .admit_received(&mut world, 2, lease.ingress(), Instant::now())
        .is_err());
    assert!(lease.current(&world, 1));
}

#[test]
fn untimed_world_accept_does_not_create_a_controller_clock_or_deadline() {
    let (mut world, mut queue) = (world(), Queue::default());
    world.accept(&packet(&world, 2941)).unwrap();
    assert!(!queue
        .admit_received(
            &mut world,
            1,
            IngressReceipt::for_test(1, 1),
            Instant::now()
        )
        .unwrap());
    assert!(world.static_effect_receipts.is_empty());
    assert!(queue.drain_due(&mut world, 1, u32::MAX).unwrap().is_empty());
}

#[test]
fn pending_resource_never_acquires_a_deadline_from_poll_or_paint() {
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    let mut canvas = super::super::super::compositor::Canvas::new(96, 96).unwrap();
    let stage = Stage::default();
    let view = super::super::super::world_map::ViewProjection {
        anchor_cell: super::super::super::world_map::WorldCell { x: 0, y: 0 },
        anchor_pixel: super::super::super::world_map::PixelPoint { x: 0, y: 0 },
    };
    for _ in 0..3 {
        let mut target = super::super::super::native_world_effects::Target::Canvas(&mut canvas);
        assert!(stage
            .draw_to(&world, &mut target, view, &AtomicBool::new(false))
            .unwrap()
            .sequences()
            .is_empty());
        assert!(queue.drain_due(&mut world, 1, u32::MAX).unwrap().is_empty());
    }
    assert!(lease.waiting() && lease.schedule().is_none());
}

#[test]
fn source_history_cannot_reset_a_retired_token() {
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    let history = History::for_world(&world);
    let mut other = history.for_source(false);
    queue.retire();
    other.merge(&history);
    let mut stage = Stage::default();
    stage.restore(&other);
    assert!(stage.history.leases.is_empty());
    assert!(!lease.active());
}

#[test]
#[ignore = "LITO_STATIC2941_ROOT; actual SPR bank"]
fn wrong_root_resource_proof_does_not_start_the_callback_clock() {
    let (root, bank) = bank();
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    let proof = Prepared::from_bank(bank, &lease, &AtomicBool::new(false))
        .unwrap()
        .unwrap();
    assert!(!queue
        .enqueue_prepared(&mut world, 1, &root.join("wrong-root"), proof, || panic!(
            "wrong root must not sample clock"
        ))
        .unwrap());
    assert!(lease.waiting() && lease.schedule().is_none());
}

#[test]
fn missing_spr_source_keeps_pending_and_does_not_create_a_fake_timer() {
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/current/issue-143-missing-source");
    assert!(!root.exists(), "negative fixture must not exist");
    let mut stage = Stage::default();
    assert!(stage
        .prepare(&root, &world, &AtomicBool::new(false), &mut None)
        .unwrap()
        .is_empty());
    let history = stage.history();
    let error = stage.error().unwrap().to_owned();
    let mut restored = Stage::default();
    restored.restore(&history.for_source(true));
    assert_eq!(restored.error(), Some(error.as_str()));
    assert!(restored
        .prepare(&root, &world, &AtomicBool::new(false), &mut None)
        .unwrap()
        .is_empty());
    assert!(queue.drain_due(&mut world, 1, u32::MAX).unwrap().is_empty());
    assert!(lease.waiting() && lease.schedule().is_none());
}
