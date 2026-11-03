//! Actual v47 HWND acceptance, never a synthetic click or optimistic UI audit.
//! Setup qualifies Main/SQL/resources; after handoff only operator input casts.
use super::*;

#[derive(Clone, Copy, Debug)]
struct Sample {
    stamp: Stamp,
    remaining: i64,
    uses: i64,
    casts: i64,
    spent: i64,
    mp: i64,
    sequence: i64,
    cell: WorldCell,
}
#[derive(Clone, Debug, Default)]
pub(super) struct Audit {
    initial: Option<Stamp>,
    depleted: bool,
    move_origin: Option<(Stamp, i64, WorldCell)>,
    moved: bool,
    returned: bool,
    close_before_deadline: bool,
    closed: bool,
    receipt: Option<(u64, &'static str, Outcome)>,
    phase: Option<LoginPhase>,
    world_lifecycle: Option<(bool, bool)>,
    owner_status_seen: bool,
    owner_status_removed: bool,
}
fn same_life(a: Stamp, b: Stamp) -> bool {
    a.context == b.context
        && a.channel == b.channel
        && a.world == b.world
        && a.entity == b.entity
        && a.life == b.life
}
impl Audit {
    fn observe(&mut self, s: Sample) {
        if self.initial.is_none() {
            if s.stamp.context.key == SUN
                && (s.remaining, s.uses, s.casts, s.spent, s.mp) == (4, 0, 0, 0, 300)
            {
                self.initial = Some(s.stamp);
            }
            return;
        }
        let initial = self.initial.unwrap();
        if !same_life(initial, s.stamp) || s.stamp.revision <= initial.revision {
            return;
        }
        if (s.remaining, s.uses, s.casts, s.spent, s.mp) != (0, 4, 4, 60, 240) {
            return;
        }
        self.depleted = true;
        let Some((before, sequence, cell)) = self.move_origin else {
            self.move_origin = Some((s.stamp, s.sequence, s.cell));
            return;
        };
        self.moved |= same_life(before, s.stamp)
            && s.stamp.revision > before.revision
            && s.sequence > sequence
            && s.cell != cell;
    }
    pub(super) fn complete(&self) -> bool {
        self.initial.is_some()
            && self.depleted
            && self.moved
            && self.returned
            && self.close_before_deadline
            && self.closed
    }
    pub(super) fn complete_with_status(&self, require_status: bool) -> bool {
        self.complete()
            && (!require_status || self.owner_status_seen && self.owner_status_removed)
    }
}
struct Controller {
    owner: SessionController,
    audit: Arc<std::sync::Mutex<Audit>>,
    started: Instant,
    operator_deadline: Instant,
}
impl LoginController for Controller {
    fn command(&mut self, q: LoginCommand) -> Result<(), String> {
        let kind = match &q {
            LoginCommand::Canonical(c) => c.audit_kind(),
            LoginCommand::Shutdown => "shutdown",
            LoginCommand::Disconnect => "disconnect",
            _ => "other-login-command",
        };
        let closing =
            matches!(&q, LoginCommand::Shutdown) && Instant::now() < self.operator_deadline;
        let result = self.owner.command(q);
        if closing && result.is_ok() {
            let mut audit = self.audit.lock().unwrap();
            audit.close_before_deadline |= audit.returned;
        }
        eprintln!(
            "SLOW_HWND_COMMAND ms={} kind={kind} admitted={}",
            self.started.elapsed().as_millis(),
            result.is_ok()
        );
        result
    }
    fn poll(&mut self) {
        self.owner.poll();
    }
    fn is_quiescent(&self) -> bool {
        self.owner.is_quiescent()
    }
    fn snapshot(&self) -> LoginSnapshot {
        let view = self.owner.snapshot();
        let mut audit = self.audit.lock().unwrap();
        let receipt = view.canonical.receipt_audit();
        if audit.receipt != receipt {
            eprintln!(
                "SLOW_HWND_RECEIPT ms={} receipt={receipt:?}",
                self.started.elapsed().as_millis()
            );
            audit.receipt = receipt;
        }
        if audit.phase != Some(view.phase) {
            eprintln!(
                "SLOW_HWND_PHASE ms={} phase={:?}",
                self.started.elapsed().as_millis(),
                view.phase
            );
            audit.phase = Some(view.phase);
        }
        let world_lifecycle = (view.canonical.attached(), view.world_pending);
        if audit.world_lifecycle != Some(world_lifecycle) {
            eprintln!(
                "SLOW_HWND_WORLD ms={} attached={} pending={}",
                self.started.elapsed().as_millis(),
                world_lifecycle.0,
                world_lifecycle.1
            );
            audit.world_lifecycle = Some(world_lifecycle);
        }
        if let Some(o) = view.canonical.observation() {
            if o.data().world.phase == data::Phase::Active && view.canonical.slow_supported() {
                let a = o
                    .data()
                    .actors
                    .0
                    .iter()
                    .find(|a| a.entity == o.data().personal.entity)
                    .unwrap();
                let slow_visible = a.buffs.as_ref().is_some_and(|b| b.slow_until_nanos > b.observed_nanos);
                if slow_visible && !audit.owner_status_seen {
                    audit.owner_status_seen = true;
                    eprintln!("SLOW_HWND_OWNER_STATUS gained revision={}", o.data().world.revision);
                }
                if audit.owner_status_seen && !slow_visible && !audit.owner_status_removed {
                    audit.owner_status_removed = true;
                    eprintln!("SLOW_HWND_OWNER_STATUS removed revision={}", o.data().world.revision);
                }
                if let (Some(materials), Some(lane), Some(vital)) = (
                    o.data().personal.materials.as_ref(),
                    a.spells
                        .histories
                        .0
                        .iter()
                        .find(|h| h.lane == data::Lane::Slow),
                    a.vital.as_ref(),
                ) {
                    if let Some(stock) = materials
                        .stocks
                        .0
                        .iter()
                        .find(|s| s.item == 40318 && s.initial == 4)
                    {
                        if lane.ledger.cost == 15 {
                            audit.observe(Sample {
                                stamp: Stamp::of(o),
                                remaining: stock.remaining,
                                uses: materials.uses,
                                casts: lane.ledger.casts,
                                spent: lane.ledger.spent,
                                mp: vital.values.mp,
                                sequence: a.last_move_sequence,
                                cell: WorldCell {
                                    x: a.position.x,
                                    y: a.position.y,
                                },
                            });
                        }
                    }
                }
            }
        }
        audit.returned |= audit.depleted
            && audit.moved
            && !view.world_pending
            && !view.canonical.attached()
            && view.phase == LoginPhase::InLobby;
        audit.closed |= audit.returned
            && audit.close_before_deadline
            && view.phase == LoginPhase::Disconnected
            && self.owner.is_quiescent();
        view
    }
}
struct StopOnDrop(mpsc::SyncSender<()>);
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        let _ = self.0.try_send(());
    }
}
pub(super) fn run(
    mut cs: [Client; 2],
    profile: &crate::modern_lito::profile::ModernProfile,
    d: &Deadline,
) -> (anyhow::Result<()>, Audit) {
    for c in &mut cs {
        c.runtime.shutdown();
    }
    while cs.iter().any(|c| !c.runtime.is_quiescent()) {
        for c in &mut cs {
            c.runtime.poll();
        }
        d.pause("retire Slow preflight renderers");
    }
    let [owner, mut guest] = cs;
    let (stop, receive) = mpsc::sync_channel(1);
    let audit = Arc::new(std::sync::Mutex::new(Audit::default()));
    let until = d.0;
    let operator_deadline =
        (Instant::now() + Duration::from_secs(540)).min(until - Duration::from_secs(10));
    let result = thread::scope(|scope| {
        let worker = scope.spawn(move || {
            while Instant::now() < until
                && receive.recv_timeout(Duration::from_millis(100))
                    == Err(mpsc::RecvTimeoutError::Timeout)
            {
                guest.controller.poll();
            }
            guest.controller.command(LoginCommand::Shutdown).unwrap();
            let d = Deadline(until);
            while !guest.controller.is_quiescent() {
                guest.controller.poll();
                d.pause("Slow interactive guest shutdown");
            }
        });
        // Drop inside the scope also wakes the guest if the UI unwinds.
        let stop = StopOnDrop(stop);
        let result = crate::modern_lito::login_window::run_connected_test(
            profile,
            Box::new(Controller {
                owner: owner.controller,
                audit: audit.clone(),
                started: Instant::now(),
                operator_deadline,
            }),
            operator_deadline,
        );
        drop(stop);
        worker.join().unwrap();
        result
    });
    let observed = audit.lock().unwrap().clone();
    eprintln!("SLOW_HWND_AUDIT {observed:?}");
    (result, observed)
}

#[test]
fn slow_window_audit_requires_paid_material_depletion_then_same_life_movement() {
    let mut audit = Audit::default();
    let mut s = Sample {
        stamp: Stamp {
            context: SelectionContext { key: SUN, epoch: 1 },
            channel: 1,
            world: [1, 2, 3],
            entity: 1,
            life: 1,
            revision: 10,
        },
        remaining: 4,
        uses: 0,
        casts: 0,
        spent: 0,
        mp: 300,
        sequence: 5,
        cell: WorldCell { x: 32768, y: 32768 },
    };
    audit.observe(s);
    assert!(audit.initial.is_some());
    assert!(!audit.complete());
    s.stamp.revision += 1;
    s.remaining = 0;
    s.uses = 4;
    s.casts = 4;
    s.spent = 60;
    s.mp = 240;
    for bad in [
        Sample { uses: 3, ..s },
        Sample { casts: 3, ..s },
        Sample { spent: 59, ..s },
        Sample { mp: 241, ..s },
        Sample { remaining: 1, ..s },
        Sample {
            stamp: Stamp { life: 2, ..s.stamp },
            ..s
        },
        Sample {
            stamp: Stamp {
                world: [1, 2, 4],
                ..s.stamp
            },
            ..s
        },
        Sample {
            stamp: Stamp {
                context: SelectionContext {
                    key: MOON,
                    epoch: 1,
                },
                ..s.stamp
            },
            ..s
        },
    ] {
        audit.observe(bad);
        assert!(!audit.depleted);
    }
    audit.observe(s);
    assert!(audit.depleted);
    assert!(!audit.moved);
    let moved = Sample {
        stamp: Stamp {
            revision: 12,
            ..s.stamp
        },
        sequence: 6,
        cell: WorldCell { x: 32769, y: 32768 },
        ..s
    };
    for bad in [
        Sample {
            sequence: 5,
            ..moved
        },
        Sample {
            cell: s.cell,
            ..moved
        },
        Sample {
            stamp: s.stamp,
            ..moved
        },
        Sample {
            stamp: Stamp {
                life: 2,
                ..moved.stamp
            },
            ..moved
        },
    ] {
        audit.observe(bad);
        assert!(!audit.moved);
    }
    audit.observe(moved);
    assert!(audit.moved);
    assert!(!audit.complete());
    audit.returned = true;
    audit.closed = true;
    assert!(!audit.complete(), "deadline shutdown is not operator close");
    audit.close_before_deadline = true;
    assert!(audit.complete());
    assert!(audit.complete_with_status(false));
    assert!(!audit.complete_with_status(true), "R383 cannot pass without observed owner status");
    audit.owner_status_seen = true;
    assert!(!audit.complete_with_status(true), "status must also be removed by server");
    audit.owner_status_removed = true;
    assert!(audit.complete_with_status(true));
}
#[test]
fn dropping_window_guard_wakes_owned_guest_without_blocking() {
    let (send, receive) = mpsc::sync_channel(1);
    drop(StopOnDrop(send));
    assert_eq!(receive.try_recv(), Ok(()));
    let (send, receive) = mpsc::sync_channel(1);
    send.send(()).unwrap();
    drop(StopOnDrop(send));
    assert_eq!(receive.try_recv(), Ok(()));
}
