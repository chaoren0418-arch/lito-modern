//! A qualified body tail, not a network action queue or combat authority.
//!
//! Native4E5B50 loads source0 and returns false;4E3CE9 calls4E02C0.
//! With no pending command,4E0399 queues stance+3 after0x248ms via
//! 4E63C0/4E9300.4E46DA refuses to replace Death8. Modern observations
//! supersede the tail when a newer confirmed action arrives; no legacy
//! command backlog, scheduler+2ms overhead or prediction is fabricated.
use crate::modern_lito::{
    combat_animation::QualifiedCombatAnimation, paced_animation::PacedClock,
    polymorph_assets::QualifiedPolymorphAction, world_scene::BaseAction,
};
use anyhow::{ensure, Context, Result};
use std::time::{Duration, Instant};

const IDLE_WAIT: Duration = Duration::from_millis(0x248);

/// Minted only from the loaded body's qualified source0 threshold. The base
/// renderer retains its existing tick clock; polymorphs use native WORD-ms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct QualifiedReturn {
    action: BaseAction,
    terminal: Duration,
}
impl QualifiedReturn {
    pub(crate) fn base(action: BaseAction, animation: &QualifiedCombatAnimation) -> Result<Self> {
        ensure!(
            matches!(action, BaseAction::Attack | BaseAction::Damage)
                && action.combat() == Some(animation.provenance().action),
            "body return action mismatch"
        );
        Ok(Self {
            action,
            terminal: animation.duration()?,
        })
    }
    pub(crate) fn polymorph(animation: &QualifiedPolymorphAction) -> Result<Self> {
        ensure!(
            matches!(animation.action(), BaseAction::Attack | BaseAction::Damage),
            "polymorph return action unqualified"
        );
        let terminal = animation
            .duration()
            .checked_sub(animation.source_duration(0)?)
            .context("polymorph terminal threshold underflow")?;
        ensure!(!terminal.is_zero(), "zero active body duration");
        Ok(Self {
            action: animation.action(),
            terminal,
        })
    }
    pub(crate) fn action(self) -> BaseAction {
        self.action
    }
    pub(crate) fn polymorph_cast(
        animation: &crate::modern_lito::polymorph_assets::cast::QualifiedCast,
    ) -> Self {
        Self {
            action: BaseAction::Idle,
            terminal: animation.terminal(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IdleReturn {
    proof: QualifiedReturn,
    terminal_at: Instant,
    idle_at: Instant,
    rate: (u64, u64),
}
impl IdleReturn {
    pub(crate) fn start(proof: QualifiedReturn, clock: &PacedClock, now: Instant) -> Result<Self> {
        // A late resource attachment cannot reconstruct an unowned deadline.
        // Initial Playback suppresses history and only new events enter here.
        ensure!(clock.peek(now)?.is_zero(), "unanchored player completion");
        let terminal_at = clock.clone().until(now, proof.terminal)?;
        let idle_at = terminal_at
            .checked_add(IDLE_WAIT)
            .context("idle deadline overflow")?;
        Ok(Self {
            proof,
            terminal_at,
            idle_at,
            rate: clock.rate(),
        })
    }
    pub(crate) fn matches(&self, proof: QualifiedReturn) -> bool {
        self.proof == proof
    }
    pub(crate) fn retime(&mut self, clock: &PacedClock, now: Instant) -> Result<()> {
        // The native post-terminal wait is real time, not attack-speed time.
        // Once source0 has been reached, no pace update extends/restarts it.
        let rate = clock.rate();
        // Repeated sampling at an unchanged rational rate must retain the
        // owned deadline. Re-solving from Q32 phase truncation can otherwise
        // add one nanosecond on every sample, even with identical authority.
        let changed = u128::from(rate.0) * u128::from(self.rate.1)
            != u128::from(rate.1) * u128::from(self.rate.0);
        if now < self.terminal_at && changed {
            self.terminal_at = clock.clone().until(now, self.proof.terminal)?;
            self.idle_at = self
                .terminal_at
                .checked_add(IDLE_WAIT)
                .context("idle deadline overflow")?;
        }
        self.rate = rate;
        Ok(())
    }
    pub(crate) fn deadline(&self) -> Instant {
        self.idle_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unchanged_fractional_pace_cannot_move_a_previously_owned_deadline() {
        let now = Instant::now();
        let mut clock = PacedClock::new(now);
        clock.retime(now, 1_200_000_000, 906_000_000).unwrap();
        let mut tail = IdleReturn::start(
            QualifiedReturn {
                action: BaseAction::Idle,
                terminal: Duration::from_millis(395),
            },
            &clock,
            now,
        )
        .unwrap();
        let original = now + Duration::from_micros(882225);
        assert_eq!(tail.deadline(), original);
        for nanos in [
            1,
            2,
            4,
            78_519_996,
            78_520_004,
            157_040_004,
            235_560_004,
            298_224_999,
        ] {
            let at = now + Duration::from_nanos(nanos);
            let scale = if nanos % 2 == 0 { 2 } else { 1 };
            clock
                .retime(at, 1_200_000_000 / scale, 906_000_000 / scale)
                .unwrap();
            tail.retime(&clock, at).unwrap();
            assert_eq!(
                tail.deadline(),
                original,
                "repeated same pace at {nanos}ns must not drift"
            );
        }
    }
    fn proof() -> QualifiedReturn {
        QualifiedReturn {
            action: BaseAction::Attack,
            terminal: Duration::from_millis(268),
        }
    }
    #[test]
    fn qualified_tail_wait_is_after_terminal_not_after_source_zero_duration() {
        let now = Instant::now();
        let clock = PacedClock::new(now);
        let tail = IdleReturn::start(proof(), &clock, now).unwrap();
        assert_eq!(tail.terminal_at, now + Duration::from_millis(268));
        assert_eq!(tail.deadline(), now + Duration::from_millis(852));
        assert!(IdleReturn::start(proof(), &clock, now + Duration::from_nanos(1)).is_err());
        assert!(!tail.matches(QualifiedReturn {
            terminal: Duration::from_millis(269),
            ..proof()
        }));
    }
    #[test]
    fn rate_changes_retime_only_remaining_active_phase_and_never_the_fixed_tail() {
        let now = Instant::now();
        let mut clock = PacedClock::new(now);
        let mut tail = IdleReturn::start(proof(), &clock, now).unwrap();
        let changed = now + Duration::from_millis(100);
        clock.retime(changed, 200_000_000, 151_000_000).unwrap();
        tail.retime(&clock, changed).unwrap();
        assert_eq!(tail.terminal_at, now + Duration::from_micros(226840));
        assert_eq!(tail.deadline(), now + Duration::from_micros(810840));
        let fixed = tail.deadline();
        for ms in [227, 300, 810, 999] {
            let at = now + Duration::from_millis(ms);
            clock.retime(at, 200_000_000, 400_000_000).unwrap();
            tail.retime(&clock, at).unwrap();
            assert_eq!(tail.deadline(), fixed);
        }
    }
    #[test]
    fn slow_then_fast_retains_phase_and_does_not_replay_history() {
        let now = Instant::now();
        let mut clock = PacedClock::new(now);
        let mut tail = IdleReturn::start(proof(), &clock, now).unwrap();
        clock.retime(now, 1, 2).unwrap();
        tail.retime(&clock, now).unwrap();
        assert_eq!(tail.terminal_at, now + Duration::from_millis(536));
        let change = now + Duration::from_millis(200);
        clock.retime(change, 2, 1).unwrap();
        tail.retime(&clock, change).unwrap();
        assert_eq!(tail.terminal_at, now + Duration::from_millis(284));
        assert_eq!(tail.deadline(), now + Duration::from_millis(868));
        assert!(
            clock
                .peek(tail.terminal_at - Duration::from_nanos(1))
                .unwrap()
                < proof().terminal
        );
        assert_eq!(clock.peek(tail.terminal_at).unwrap(), proof().terminal);
    }
}
