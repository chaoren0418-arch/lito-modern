//! A body-only phase accumulator. An authority pace change never rewinds source
//! frames/cues. Effect, projectile and PCM clocks must not use this clock.
use anyhow::{ensure, Result};
use std::time::{Duration, Instant};
#[derive(Clone, Debug)]
pub(crate) struct PacedClock {
    last: Instant,
    phase: u128,
    base: u64,
    effective: u64,
}
impl PacedClock {
    pub fn new(now: Instant) -> Self {
        Self {
            last: now,
            phase: 0,
            base: 1,
            effective: 1,
        }
    }
    fn advance(&mut self, now: Instant) -> Result<()> {
        let delta = now.saturating_duration_since(self.last).as_nanos();
        let scaled = delta
            .checked_mul(u128::from(self.base))
            .and_then(|n| n.checked_mul(1u128 << 32))
            .map(|n| n / u128::from(self.effective))
            .ok_or_else(|| anyhow::anyhow!("body phase overflow"))?;
        self.phase = self
            .phase
            .checked_add(scaled)
            .ok_or_else(|| anyhow::anyhow!("body phase overflow"))?;
        self.last = self.last.max(now);
        Ok(())
    }
    pub fn retime(&mut self, now: Instant, base: u64, effective: u64) -> Result<()> {
        ensure!(
            base > 0 && effective > 0 && base <= 10_000_000_000 && effective <= 10_000_000_000,
            "body pace out of bound"
        );
        self.advance(now)?;
        self.base = base;
        self.effective = effective;
        Ok(())
    }
    pub fn restart(&mut self, now: Instant) {
        self.last = now;
        self.phase = 0;
    }
    pub(crate) fn rate(&self) -> (u64, u64) {
        (self.base, self.effective)
    }
    pub fn elapsed(&mut self, now: Instant) -> Result<Duration> {
        self.advance(now)?;
        Ok(Duration::from_nanos(u64::try_from(self.phase >> 32)?))
    }
    pub fn peek(&self, now: Instant) -> Result<Duration> {
        self.clone().elapsed(now)
    }
    pub fn until(&mut self, now: Instant, phase: Duration) -> Result<Instant> {
        self.advance(now)?;
        let target = phase
            .as_nanos()
            .checked_mul(1u128 << 32)
            .ok_or_else(|| anyhow::anyhow!("body phase overflow"))?;
        let left = target.saturating_sub(self.phase);
        let numerator = left
            .checked_mul(u128::from(self.effective))
            .ok_or_else(|| anyhow::anyhow!("body phase overflow"))?;
        let divisor = u128::from(self.base) << 32;
        let nanos = numerator / divisor + u128::from(numerator % divisor != 0);
        self.last
            .checked_add(Duration::from_nanos(u64::try_from(nanos)?))
            .ok_or_else(|| anyhow::anyhow!("body phase deadline overflow"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pace_changes_preserve_elapsed_phase_and_do_not_apply_new_rate_to_history() {
        let t = Instant::now();
        let mut c = PacedClock::new(t);
        assert_eq!(
            c.elapsed(t + Duration::from_millis(100)).unwrap(),
            Duration::from_millis(100)
        );
        c.retime(t + Duration::from_millis(100), 1000, 500).unwrap();
        assert_eq!(
            c.elapsed(t + Duration::from_millis(150)).unwrap(),
            Duration::from_millis(200)
        );
        c.retime(t + Duration::from_millis(150), 1000, 1000)
            .unwrap();
        assert_eq!(
            c.elapsed(t + Duration::from_millis(250)).unwrap(),
            Duration::from_millis(300)
        );
    }
    #[test]
    fn late_ack_timestamp_cannot_rewind_or_double_accrue() {
        let t = Instant::now();
        let mut c = PacedClock::new(t);
        c.elapsed(t + Duration::from_millis(100)).unwrap();
        c.retime(t + Duration::from_millis(50), 1000, 500).unwrap();
        assert_eq!(
            c.elapsed(t + Duration::from_millis(100)).unwrap(),
            Duration::from_millis(100)
        );
        assert_eq!(
            c.elapsed(t + Duration::from_millis(150)).unwrap(),
            Duration::from_millis(200)
        );
    }
    #[test]
    fn accelerated_walk_deadline_retains_progress() {
        let t = Instant::now();
        let mut c = PacedClock::new(t);
        c.retime(t + Duration::from_millis(100), 200000000, 100000000)
            .unwrap();
        assert_eq!(
            c.until(t + Duration::from_millis(100), Duration::from_millis(200))
                .unwrap(),
            t + Duration::from_millis(150)
        );
    }
}
