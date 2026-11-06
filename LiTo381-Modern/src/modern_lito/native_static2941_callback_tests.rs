use super::*;
use std::sync::Mutex;

#[test]
fn fixed_outer_sample_survives_reentrant_enqueue_and_nested_drain() {
    let (mut world, mut queue) = (world(), Queue::default());
    let events = Arc::new(Mutex::new(Vec::new()));
    let log = events.clone();
    queue
        .enqueue_callback(
            &world,
            1,
            0,
            || Ok(10),
            Box::new(move |q, w, c| {
                assert!(q.due.is_empty(), "pop precedes callback");
                log.lock().unwrap().push(10);
                let nested = log.clone();
                q.enqueue_callback(
                    w,
                    c,
                    0,
                    || Ok(11),
                    Box::new(move |q, w, c| {
                        nested.lock().unwrap().push(11);
                        let tail = nested.clone();
                        q.enqueue_callback(
                            w,
                            c,
                            0,
                            || Ok(12),
                            Box::new(move |_, _, _| {
                                tail.lock().unwrap().push(12);
                                Ok(())
                            }),
                        )?;
                        Ok(())
                    }),
                )?;
                q.drain_due(w, c, 11)?;
                log.lock().unwrap().push(100); // resumed outer callback
                Ok(())
            }),
        )
        .unwrap();
    queue.drain_due(&mut world, 1, 10).unwrap();
    assert_eq!(*events.lock().unwrap(), [10, 11, 100]);
    assert_eq!(queue.due.len(), 1);
    queue.drain_due(&mut world, 1, 12).unwrap();
    assert_eq!(*events.lock().unwrap(), [10, 11, 100, 12]);
}

#[test]
fn equal_deadline_insertion_remains_fifo_and_owner_change_discards_producers() {
    let (mut world, mut queue) = (world(), Queue::default());
    let events = Arc::new(Mutex::new(Vec::new()));
    for id in [1, 2] {
        let log = events.clone();
        queue
            .enqueue_callback(
                &world,
                1,
                0,
                || Ok(0),
                Box::new(move |q, w, c| {
                    log.lock().unwrap().push(id);
                    if id == 1 {
                        let tail = log.clone();
                        q.enqueue_callback(
                            w,
                            c,
                            0,
                            || Ok(0),
                            Box::new(move |_, _, _| {
                                tail.lock().unwrap().push(3);
                                Ok(())
                            }),
                        )?;
                    }
                    Ok(())
                }),
            )
            .unwrap();
    }
    queue.drain_due(&mut world, 1, 0).unwrap();
    assert_eq!(*events.lock().unwrap(), [1, 2, 3]);
    queue
        .enqueue_callback(
            &world,
            1,
            0,
            || Ok(1),
            Box::new(|_, _, _| panic!("retired owner")),
        )
        .unwrap();
    queue.drain_due(&mut world, 2, 1).unwrap();
    assert!(queue.due.is_empty());
}

#[test]
fn nested_4857b0_has_fresh_samples_and_draws_after_each_drain_across_wrap() {
    let (mut world, mut queue) = (world(), Queue::default());
    let calls = Arc::new(Mutex::new(Vec::new()));
    let log = calls.clone();
    queue
        .enqueue_callback(
            &world,
            1,
            0,
            || Ok(0),
            Box::new(move |_, _, _| {
                log.lock().unwrap().push("callback");
                Ok(())
            }),
        )
        .unwrap();
    // start MAX-99 -> 0 (100 ms) -> 400 (500 ms total).
    let mut samples = [u32::MAX - 99, 0, 0, 400, 400].into_iter();
    queue
        .drain_4857b0(
            &mut world,
            1,
            || Ok(samples.next().expect("extra native sample")),
            |_, _, _| {
                calls.lock().unwrap().push("draw");
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(*calls.lock().unwrap(), ["callback", "draw", "draw"]);
    assert!(samples.next().is_none());
}

#[test]
fn failed_shared_callback_is_not_replayed_and_leaves_later_work_queued() {
    let (mut world, mut queue) = (world(), Queue::default());
    queue
        .enqueue_callback(
            &world,
            1,
            0,
            || Ok(4),
            Box::new(|_, _, _| anyhow::bail!("producer failed")),
        )
        .unwrap();
    let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let next = count.clone();
    queue
        .enqueue_callback(
            &world,
            1,
            0,
            || Ok(4),
            Box::new(move |_, _, _| {
                next.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }),
        )
        .unwrap();
    assert!(queue.drain_due(&mut world, 1, 4).is_err());
    queue.drain_due(&mut world, 1, 4).unwrap();
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[test]
#[ignore = "LITO_STATIC2941_ROOT; original fragments and shared callback redraw"]
fn original_pixels_publish_inside_callback_window_and_are_revoked_after_retirement() {
    use super::super::super::{
        compositor::Canvas,
        native_world_effects::{Mix, Target},
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    let root =
        PathBuf::from(std::env::var_os("LITO_STATIC2941_ROOT").expect("explicit original root"));
    let (mut world, mut queue) = (world(), Queue::default());
    let lease = receive(&mut queue, &mut world, 1);
    let mut stage = Stage::default();
    let mut proofs = stage
        .prepare(&root, &world, &AtomicBool::new(false), &mut None)
        .unwrap();
    assert_eq!(proofs.len(), 1);
    let cue = world.effects.last().unwrap();
    let view = ViewProjection {
        anchor_cell: WorldCell {
            x: cue.position[0].into(),
            y: cue.position[1].into(),
        },
        anchor_pixel: PixelPoint { x: 48, y: 48 },
    };
    let mut fragments = vec![];
    let guard = stage
        .draw_to(
            &world,
            &mut Target::Fragments(&mut fragments),
            view,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(fragments.len(), 1);
    assert!(matches!(fragments[0].mix, Mix::Over));
    assert_eq!(
        fragments[0]
            .frame
            .fragments
            .as_ref()
            .unwrap()
            .references
            .len(),
        4
    );
    assert!(
        !guard.current(&world, 1),
        "speculative pixels cannot publish before enqueue"
    );
    let mut canvas = Canvas::new(96, 96).unwrap();
    stage
        .draw_to(
            &world,
            &mut Target::Canvas(&mut canvas),
            view,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(canvas
        .rgba()
        .chunks_exact(4)
        .any(|pixel| pixel[0..3] != [0, 0, 0]));
    let before = lease.schedule();
    assert!(before.is_none(), "composition does not sample or enqueue");
    let observed = Arc::new(Mutex::new(Vec::new()));
    let log = observed.clone();
    let during = guard.clone();
    queue
        .enqueue_callback(
            &world,
            1,
            0,
            || Ok(5),
            Box::new(move |q, w, c| {
                assert!(during.current(w, c));
                log.lock().unwrap().push("visible");
                // Nested drain consumes the later equal-deadline retirement before
                // this producer returns. A retained raster is no longer publishable.
                q.drain_due(w, c, 5)?;
                assert!(!during.current(w, c));
                log.lock().unwrap().push("revoked");
                Ok(())
            }),
        )
        .unwrap();
    queue
        .enqueue_prepared(&mut world, 1, &root, proofs.pop().unwrap(), || Ok(5))
        .unwrap();
    let snapshot = world.clone();
    assert!(guard.current(&snapshot, 1));
    // A synthetic scene shell isolates the actual frame/input/cache guards;
    // pixel evidence above comes from the real bank, not this shell.
    let mut frame = super::super::super::native_world_scene::movement_tests::frame(&world, false);
    frame.static2941 = guard.clone();
    let input = frame.input_surface(&snapshot).unwrap();
    let mut presentation = super::super::super::native_world_view::Presentation::default();
    presentation.frame = Some(frame);
    queue.drain_due(&mut world, 1, 5).unwrap();
    assert_eq!(*observed.lock().unwrap(), ["visible", "revoked"]);
    assert!(!guard.current(&snapshot, 1));
    assert!(!input.input_ready(&snapshot));
    assert!(!presentation.frame.as_ref().unwrap().input_ready(&snapshot));
    assert!(presentation.prune_static_frame(&snapshot, 1));
    assert!(presentation.frame.is_none());
    let mut after = vec![];
    assert!(stage
        .draw_to(
            &snapshot,
            &mut Target::Fragments(&mut after),
            view,
            &AtomicBool::new(false)
        )
        .unwrap()
        .sequences()
        .is_empty());
    assert!(after.is_empty());
}
