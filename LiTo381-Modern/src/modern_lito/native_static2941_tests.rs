use super::*;

fn world() -> World {
    super::super::native_world_scene::movement_tests::world()
}
fn packet(world: &World, graphics: u16) -> Vec<u8> {
    let mut body = vec![0x37];
    body.extend(world.own_actor().unwrap().id.to_le_bytes());
    body.extend(graphics.to_le_bytes());
    body
}
fn receive(queue: &mut Queue, world: &mut World, counter: u64) -> Lease {
    let ingress = IngressReceipt::for_test(1, counter);
    world
        .accept_received(&packet(world, 2941), ingress)
        .unwrap();
    assert!(queue
        .admit_received(world, 1, ingress, Instant::now())
        .unwrap());
    world
        .static_effect_receipts
        .values()
        .next_back()
        .unwrap()
        .lease()
        .unwrap()
        .clone()
}

#[test]
fn untimed_accept_does_not_fabricate_ingress_or_deadline() {
    let mut world = world();
    world.accept(&packet(&world, 2941)).unwrap();
    assert!(world.static_effect_receipts.is_empty());
    let mut queue = Queue::default();
    assert!(!queue
        .admit_received(
            &mut world,
            1,
            IngressReceipt::for_test(1, 1),
            Instant::now()
        )
        .unwrap());
    assert!(queue.pending.is_empty());
}

#[test]
fn only_valid_static_s37_gets_a_receipt() {
    let mut world = world();
    for body in [
        packet(&world, 2942),
        vec![0x37, 0xff, 0xff, 0xff, 0x7f, 0x7d, 0x0b],
        vec![0x53, 1, 0, 1, 0, 0x7d, 0x0b],
    ] {
        world
            .accept_received(&body, IngressReceipt::for_test(1, 1))
            .unwrap();
        assert!(world.static_effect_receipts.is_empty());
    }
    assert!(world
        .accept_received(&[0x37, 1], IngressReceipt::for_test(1, 2))
        .is_err());
    assert!(world
        .accept_received(&packet(&world, 2941), IngressReceipt::for_test(0, 3))
        .is_err());
    assert!(world.static_effect_receipts.is_empty());
}

#[test]
fn receive_waits_without_a_deadline_and_keeps_distinct_times() {
    let mut world = world();
    let mut queue = Queue::default();
    let lease = receive(&mut queue, &mut world, 2);
    assert!(lease.waiting());
    assert!(!lease.drawable());
    assert!(lease.schedule().is_none());
    assert!(lease.ingress().decoded_at() <= lease.dispatched_at());
    assert!(lease.dispatched_at() <= lease.constructed_at());
    assert!(queue.drain_due(&mut world, 1, u32::MAX).unwrap().is_empty());
    assert!(lease.waiting());
    let json = serde_json::to_value(&world).unwrap();
    assert!(json.get("static_effect_receipts").is_none());
}

#[test]
fn mismatched_receipt_and_duplicate_admission_are_rejected() {
    let mut world = world();
    let mut queue = Queue::default();
    let ingress = IngressReceipt::for_test(1, 2);
    world
        .accept_received(&packet(&world, 2941), ingress)
        .unwrap();
    assert!(queue
        .admit_received(&mut world, 2, ingress, Instant::now())
        .is_err());
    assert!(!queue
        .admit_received(
            &mut world,
            1,
            IngressReceipt::for_test(1, 3),
            Instant::now()
        )
        .unwrap());
    assert!(queue
        .admit_received(&mut world, 1, ingress, Instant::now())
        .unwrap());
    assert!(!queue
        .admit_received(&mut world, 1, ingress, Instant::now())
        .unwrap());
    assert_eq!(queue.pending.len(), 1);
}

#[test]
fn world_clear_and_map_replacement_revoke_cloned_observers() {
    for map_packet in [false, true] {
        let mut world = world();
        let mut queue = Queue::default();
        let lease = receive(&mut queue, &mut world, 1);
        let snapshot = world.clone();
        if map_packet {
            world.accept(&[0x28, 4, 0, 0]).unwrap();
        } else {
            world.clear_preserving_generation();
        }
        assert!(!lease.active());
        assert!(!snapshot.static_effect_receipts[&lease.sequence()]
            .lease()
            .unwrap()
            .active());
        queue.synchronize(&world, 1);
        assert!(queue.pending.is_empty());
    }
}

#[test]
fn ring_eviction_is_bounded_and_cannot_restore_old_token() {
    let mut world = world();
    let mut queue = Queue::default();
    let first = receive(&mut queue, &mut world, 1);
    let snapshot = world.clone();
    for counter in 2..=140 {
        receive(&mut queue, &mut world, counter);
    }
    assert_eq!(world.effects.len(), 128);
    assert_eq!(world.static_effect_receipts.len(), 128);
    assert_eq!(queue.pending.len(), 128);
    assert!(!first.active());
    queue.synchronize(&snapshot, 1);
    assert!(!first.active());
}

#[test]
fn connection_generation_and_target_incarnation_revoke_tokens() {
    for change in 0..3 {
        let mut world = world();
        let mut queue = Queue::default();
        let lease = receive(&mut queue, &mut world, 1);
        let connection = if change == 0 { 2 } else { 1 };
        if change == 1 {
            world.generation += 1;
        }
        if change == 2 {
            let id = world.own_actor().unwrap().id;
            world.actors.get_mut(&id).unwrap().incarnation += 1;
        }
        queue.synchronize(&world, connection);
        assert!(!lease.active());
        assert!(queue.pending.is_empty());
    }
}

#[test]
fn dropping_queue_retires_waiting_snapshots() {
    let mut world = world();
    let mut queue = Queue::default();
    let lease = receive(&mut queue, &mut world, 1);
    drop(queue);
    assert!(!lease.active());
}

#[test]
fn resource_error_latches_without_clock_or_deadline() {
    let mut world = world();
    let mut queue = Queue::default();
    let lease = receive(&mut queue, &mut world, 1);
    let mut stage = Stage::default();
    let mut shared = None;
    let missing =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/issue-142-nonexistent-assets");
    assert!(
        !missing.exists(),
        "negative fixture path unexpectedly exists"
    );
    assert!(stage
        .prepare(&missing, &world, &AtomicBool::new(false), &mut shared)
        .unwrap()
        .is_empty());
    let error = stage.error().unwrap().to_owned();
    assert!(stage
        .prepare(&missing, &world, &AtomicBool::new(false), &mut shared)
        .unwrap()
        .is_empty());
    assert_eq!(stage.error(), Some(error.as_str()));
    assert!(lease.schedule().is_none());
    assert!(lease.waiting());
    assert!(stage
        .prepare(&missing, &world, &AtomicBool::new(true), &mut shared)
        .is_err());
}

#[test]
#[ignore = "requires explicit LITO_STATIC2941_ROOT read-only original resources"]
fn original_bank_ready_gate_fifo_rollover_and_retirement() {
    let root = PathBuf::from(
        std::env::var_os("LITO_STATIC2941_ROOT").expect("set original resource root"),
    );
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load_for_model(&root, &cancel).unwrap();
    let bank = Arc::new(Static2941Bank::load(&catalog, &cancel).unwrap());
    assert!(!bank.used_missing_spr_fallback());
    assert_eq!(bank.provenance().name, "2941-0.spr");
    assert_eq!(bank.provenance().bytes, 865);
    assert_eq!(
        bank.frame_zero()
            .fragments
            .as_ref()
            .unwrap()
            .references
            .len(),
        4
    );
    let mut world = world();
    let mut queue = Queue::default();
    let first = receive(&mut queue, &mut world, 1);
    assert!(Prepared::from_bank(bank.clone(), &first, &AtomicBool::new(true)).is_err());
    let proof = Prepared::from_bank(bank.clone(), &first, &cancel)
        .unwrap()
        .unwrap();
    assert!(!queue
        .enqueue_prepared(&mut world, 1, &root.join("wrong-root"), proof, || panic!(
            "wrong root read clock"
        ))
        .unwrap());
    assert!(first.waiting());
    let proof = Prepared::from_bank(bank.clone(), &first, &cancel)
        .unwrap()
        .unwrap();
    assert!(queue
        .enqueue_prepared(&mut world, 1, &root, proof, || Err(anyhow::anyhow!(
            "clock import failed"
        )))
        .is_err());
    assert!(first.schedule().is_none());
    let proof = Prepared::from_bank(bank.clone(), &first, &cancel)
        .unwrap()
        .unwrap();
    let duplicate = Prepared::from_bank(bank.clone(), &first, &cancel)
        .unwrap()
        .unwrap();
    assert!(queue
        .enqueue_prepared(&mut world, 1, &root, proof, || Ok(u32::MAX))
        .unwrap());
    let schedule = first.schedule().unwrap();
    assert!(first.drawable());
    assert!(
        first.constructed_at() <= schedule.ready_at && schedule.ready_at <= schedule.enqueued_at
    );
    assert_eq!(schedule.deadline_ms, u32::MAX);
    assert!(!queue
        .enqueue_prepared(&mut world, 1, &root, duplicate, || panic!(
            "duplicate read clock"
        ))
        .unwrap());
    let second = receive(&mut queue, &mut world, 2);
    let proof = Prepared::from_bank(bank.clone(), &second, &cancel)
        .unwrap()
        .unwrap();
    queue
        .enqueue_prepared(&mut world, 1, &root, proof, || Ok(0))
        .unwrap();
    let third = receive(&mut queue, &mut world, 3);
    let proof = Prepared::from_bank(bank.clone(), &third, &cancel)
        .unwrap()
        .unwrap();
    queue
        .enqueue_prepared(&mut world, 1, &root, proof, || Ok(0))
        .unwrap();
    assert_eq!(
        queue.drain_due(&mut world, 1, 0).unwrap(),
        [second.sequence(), third.sequence()]
    );
    assert!(first.drawable());
    assert!(!second.active() && !third.active());
    assert_eq!(
        queue.drain_due(&mut world, 1, u32::MAX).unwrap(),
        [first.sequence()]
    );
    assert!(Prepared::from_bank(bank.clone(), &first, &cancel)
        .unwrap()
        .is_none());
    let fourth = receive(&mut queue, &mut world, 4);
    let in_flight = Prepared::from_bank(bank.clone(), &fourth, &cancel)
        .unwrap()
        .unwrap();
    assert!(!queue
        .enqueue_prepared(&mut world, 1, &root, in_flight, || {
            fourth.retire();
            Ok(7)
        })
        .unwrap());
    assert!(!fourth.active());
    assert!(fourth.schedule().is_none());
    assert!(queue.due.is_empty());
    let fifth = receive(&mut queue, &mut world, 5);
    let late = Prepared::from_bank(bank, &fifth, &cancel).unwrap().unwrap();
    world.clear_preserving_generation();
    assert!(!queue
        .enqueue_prepared(&mut world, 1, &root, late, || panic!(
            "late proof read clock"
        ))
        .unwrap());
    assert!(!fifth.active());
}
