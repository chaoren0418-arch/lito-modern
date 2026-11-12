use super::*;
use crate::modern_lito::{native_world_scene::movement_tests, sprite_actions::SpriteActionCatalog};
fn world() -> World {
    let mut w = movement_tests::world();
    let id = w.own_actor().unwrap().id;
    let a = w.actors.get_mut(&id).unwrap();
    a.speed = 0;
    a.brave = 0;
    a.status &= !0x80;
    w
}

fn plan(w: &World) -> Arc<WalkPlan> {
    let a = w.own_actor().unwrap();
    let text = format!(
        "#{} 8 test\n{}.walk(1 4,0.0:4 0.1:4 0.2:4 0.3:4)\n",
        a.graphics, a.walk_action
    );
    let catalog = SpriteActionCatalog::parse(text.as_bytes(), Default::default()).unwrap();
    Arc::new(WalkPlan::load(&catalog, a).unwrap())
}
#[derive(Default)]
struct DriverLog {
    frames: Vec<usize>,
    sent: Vec<Vec<u8>>,
    base_count: usize,
    retire: bool,
}
impl Driver for DriverLog {
    fn draw(
        &mut self,
        q: &mut Queue,
        w: &mut World,
        _: u64,
        _: &mut dyn FnMut() -> Result<u32>,
    ) -> Result<usize> {
        self.frames
            .push(w.player_walk.as_ref().map_or(0, |p| p.frame));
        if self.retire {
            q.retire();
            w.clear_preserving_generation();
        }
        Ok(self.base_count)
    }
    fn send(&mut self, _: u64, body: Zeroizing<Vec<u8>>) -> Result<()> {
        self.sent.push(body.to_vec());
        Ok(())
    }
    fn validate_step(&self, _: &World, _: u8) -> Result<()> {
        Ok(())
    }
}

#[test]
fn source_frames_advance_in_shared_queue_instead_of_elapsed_pose() {
    let mut w = world();
    let p = plan(&w);
    let mut q = Queue::default();
    let mut control = Control::default();
    w.predict_step(2).unwrap();
    control
        .start(&mut q, &mut w, 1, p, &mut || Ok(100))
        .unwrap();
    let mut driver = DriverLog::default();
    let mut clock = || Ok(266);
    let mut host = Host {
        control: &mut control,
        driver: &mut driver,
        clock: &mut clock,
    };
    host.run(&mut q, &mut w, 1, false).unwrap();
    assert_eq!(w.player_walk.as_ref().unwrap().frame, 2);
    assert_eq!(driver.frames, [1, 2]);
}

#[test]
fn raw_normal_tail_uses_unsigned_delta_before_cap() {
    let mut c = Control::default();
    c.normal_tail(u32::MAX - 20, 12);
    c.normal_tail(30, 12);
    assert_eq!(c.raw_draw_delta, 51);
    assert!(c.skip_suffix_draw);
    c.normal_tail(1000, 8);
    assert_eq!(c.capped_draw_delta, 500);
    assert!(!c.skip_suffix_draw);
}

fn started() -> (World, Queue, Control) {
    let mut w = world();
    let p = plan(&w);
    let mut q = Queue::default();
    let mut c = Control::default();
    w.predict_step(2).unwrap();
    c.start(&mut q, &mut w, 1, p, &mut || Ok(0)).unwrap();
    c.dirty = false;
    (w, q, c)
}

#[test]
fn both_suffixes_draw_from_source_index_and_keep_fresh_continuation_outside_outer_cutoff() {
    let (mut w, mut q, mut c) = started();
    c.held(Some(2));
    let mut driver = DriverLog::default();
    let mut clock = || Ok(1000);
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    q.drain_with(&mut w, 1, 498, &mut host).unwrap();
    assert_eq!(host.driver.frames, [2, 3, 0]);
    assert!(host.control.continuation);
    assert_eq!(
        w.predicted_moves, 1,
        "fresh continuation cannot use outer cutoff"
    );
    host.control.held(None);
    q.drain_with(&mut w, 1, 1166, &mut host).unwrap();
    assert!(w.player_walk.as_ref().unwrap().complete);
    assert!(host.driver.sent.is_empty());
}

#[test]
fn cancelled_frame_zero_takes_precedence_over_pending_send() {
    let (mut w, mut q, mut c) = started();
    assert!(c
        .defer(Zeroizing::new(vec![0x14, 12, 0]))
        .unwrap()
        .is_none());
    c.cancel();
    let mut driver = DriverLog::default();
    let mut clock = || Ok(500);
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    q.drain_with(&mut w, 1, 500, &mut host).unwrap();
    assert!(host.driver.sent.is_empty());
    assert!(host.control.pending.is_none());
    assert!(!host.control.action_sent);
    assert!(!host.control.active);
    assert_eq!(
        host.driver.frames,
        [2, 3],
        "cancelled zero suffix does not draw"
    );
}

#[test]
fn pending_action_is_sent_once_at_frame_zero_and_sets_169_gate() {
    let (mut w, mut q, mut c) = started();
    let body = vec![0x14, 12, 0];
    c.defer(Zeroizing::new(body.clone())).unwrap();
    let mut driver = DriverLog::default();
    let mut clock = || Ok(500);
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    q.drain_with(&mut w, 1, 332, &mut host).unwrap();
    assert!(host.driver.sent.is_empty());
    q.drain_with(&mut w, 1, 500, &mut host).unwrap();
    assert_eq!(host.driver.sent, [body]);
    assert!(host.control.action_sent);
    q.drain_with(&mut w, 1, 1000, &mut host).unwrap();
    assert_eq!(host.driver.sent.len(), 1);
}

#[test]
fn twelve_frame_normal_tail_suppresses_both_suffix_draws_but_not_source_advance() {
    let (mut w, mut q, mut c) = started();
    c.held(Some(2));
    c.normal_tail(0, 12);
    c.normal_tail(41, 12);
    let mut driver = DriverLog::default();
    let mut clock = || Ok(1000);
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    q.drain_with(&mut w, 1, 498, &mut host).unwrap();
    assert!(host.driver.frames.is_empty());
    assert_eq!(w.player_walk.as_ref().unwrap().frame, 0);
    assert!(host.control.continuation);
}

#[test]
fn separate_integer_duration_stages_and_character_override() {
    let mut w = movement_tests::world();
    let p = plan(&w);
    let id = w.own_actor().unwrap().id;
    let a = w.actors.get_mut(&id).unwrap();
    a.speed = 1;
    a.brave = 1;
    assert_eq!(p.duration(1, a, false).unwrap(), 93); // floor(floor(166*3/4)*3/4)
    a.brave = 3;
    assert_eq!(p.duration(1, a, true).unwrap(), 63); // floor(72*7/8)
    a.status |= 0x80;
    let mut c = Control::default();
    c.character_timing(7, 1);
    assert!(!c.special(a));
    c.character_timing(0, 80);
    assert!(c.special(a));
    a.status &= !0x80;
    c.character_timing(1, 50);
    assert!(c.special(a));
    c.character_timing(1, 51);
    assert!(!c.special(a));
}

fn effect(w: &World, graphics: u16, target: Option<u32>) -> Vec<u8> {
    let mut body = vec![0x37];
    body.extend(target.unwrap_or(w.own_actor().unwrap().id).to_le_bytes());
    body.extend(graphics.to_le_bytes());
    body
}

#[test]
fn s37_pump_requires_successful_action_send_169_and_current_own_target() {
    let mut w = movement_tests::world();
    let mut q = Queue::default();
    let mut c = Control::default();
    c.synchronize(&mut q, &mut w, 1);
    c.accepted(&effect(&w, 169, None), &w);
    assert!(!c.pump);
    c.sent();
    c.accepted(&effect(&w, 2941, None), &w);
    assert!(!c.pump);
    c.accepted(&effect(&w, 169, Some(u32::MAX)), &w);
    assert!(!c.pump);
    c.accepted(&effect(&w, 169, None), &w);
    assert!(c.pump);
    c.pump = false;
    c.accepted(&[0x10, 0x49, 0, 0], &w);
    assert!(!c.action_sent);
    c.accepted(&effect(&w, 169, None), &w);
    assert!(!c.pump);
}

#[test]
fn accepted_169_cancels_camera_and_nested_drain_draws_before_completion_across_rollover() {
    let mut w = movement_tests::world();
    let mut q = Queue::default();
    let mut c = Control::default();
    c.synchronize(&mut q, &mut w, 1);
    c.camera_step = 4;
    c.camera_actor = Some(2);
    c.sent();
    q.enqueue_native(&w, 1, 0, 0, Event::Camera).unwrap();
    let body = effect(&w, 169, None);
    w.accept(&body).unwrap();
    c.accepted(&body, &w);
    let mut values = [u32::MAX - 99, 0, 0, 400].into_iter();
    let mut clock = || Ok(values.next().expect("extra pump clock"));
    let mut driver = DriverLog::default();
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    host.run(&mut q, &mut w, 1, true).unwrap();
    assert!(host.control.pump_completed);
    assert_eq!(host.control.camera_step, -1);
    assert!(host.control.camera_actor.is_none());
    assert_eq!(host.driver.frames, [0]);
    assert!(values.next().is_none());
}

#[test]
fn correction_retires_player_timers_and_pending_action_without_recreating_source_pose() {
    let (mut w, mut q, mut c) = started();
    c.defer(Zeroizing::new(vec![0x14])).unwrap();
    w.local_corrections += 1;
    let mut driver = DriverLog::default();
    let mut clock = || Ok(1000);
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    host.run(&mut q, &mut w, 1, false).unwrap();
    assert!(w.player_walk.is_none());
    assert!(host.driver.frames.is_empty());
    assert!(host.driver.sent.is_empty());
    assert!(!host.control.active);
}

#[test]
fn a_repeated_painted_frame_does_not_recompute_the_normal_tail() {
    let mut c = Control::default();
    c.observe_frame(1, 12, || Ok(0)).unwrap();
    c.observe_frame(2, 12, || Ok(41)).unwrap();
    assert!(c.skip_suffix_draw);
    c.observe_frame(2, 12, || panic!("paint is not a new native draw"))
        .unwrap();
    assert!(c.skip_suffix_draw);
    assert_eq!(c.raw_draw_delta, 41);
}

#[test]
fn nested_pump_dispatches_player_callbacks_on_the_same_host_before_its_own_draw() {
    let (mut w, mut q, mut c) = started();
    c.sent();
    c.accepted(&effect(&w, 169, None), &w);
    let mut samples = [0, 166, 200, 201, 500].into_iter();
    let mut clock = || Ok(samples.next().expect("extra nested clock"));
    let mut driver = DriverLog::default();
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    host.run(&mut q, &mut w, 1, true).unwrap();
    assert_eq!(host.driver.frames, [2, 2]);
    assert!(host.control.pump_completed);
    assert_eq!(w.player_walk.as_ref().unwrap().frame, 2);
    assert!(samples.next().is_none());
}

#[test]
fn pump_retirement_does_not_sample_normal_tail_or_mark_completion() {
    let mut w = world();
    let mut q = Queue::default();
    let mut c = Control::default();
    c.synchronize(&mut q, &mut w, 1);
    c.sent();
    c.accepted(&effect(&w, 169, None), &w);
    let mut samples = [0, 0].into_iter();
    let mut clock = || Ok(samples.next().expect("sample after retirement"));
    let mut driver = DriverLog {
        retire: true,
        ..Default::default()
    };
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    host.run(&mut q, &mut w, 1, true).unwrap();
    assert!(!host.control.pump_completed);
    assert_eq!(host.driver.frames, [0]);
    assert!(samples.next().is_none());
}

#[test]
fn held_continuation_sends_validated_move_then_owns_the_next_source_walk() {
    let (mut w, mut q, mut c) = started();
    c.held(Some(6));
    let expected = w.move_request(6).unwrap();
    let mut driver = DriverLog::default();
    let mut clock = || Ok(1000);
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    q.drain_with(&mut w, 1, 498, &mut host).unwrap();
    q.drain_with(&mut w, 1, 1166, &mut host).unwrap();
    assert_eq!(host.driver.sent, [expected]);
    assert_eq!(w.predicted_moves, 2);
    assert!(host.control.active);
    assert!(w.player_walk.as_ref().unwrap().current(&w));
    // The first callback of the new walk has the same deadline as this cutoff.
    assert_eq!(w.player_walk.as_ref().unwrap().frame, 2);
}

#[test]
fn received_action_queue_suppresses_held_continuation_until_combat_can_run() {
    let (mut w, mut q, mut c) = started();
    c.held(Some(2));
    let id = w.own_actor().unwrap().id;
    let actor = w.actors.get_mut(&id).unwrap();
    actor.action_sequence += 1;
    actor
        .actions
        .push(crate::modern_lito::native_combat::ActionCue {
            sequence: actor.action_sequence,
            action: 18,
            source: "S_ACTION",
        });
    let mut driver = DriverLog::default();
    let mut clock = || Ok(1000);
    let mut host = Host {
        control: &mut c,
        driver: &mut driver,
        clock: &mut clock,
    };
    q.drain_with(&mut w, 1, 1000, &mut host).unwrap();
    assert!(!host.control.continuation);
    assert!(!host.control.active);
    assert!(host.driver.sent.is_empty());
    assert!(w.player_walk.as_ref().unwrap().complete);
}
