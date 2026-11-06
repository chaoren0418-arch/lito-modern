//! Native timer arithmetic, separate from packet-receipt `Instant`s.
//!
//! 4D6360 retains the imported frequency-query result. 4D69F0 multiplies the
//! counter by 1000 (low 64 bits), performs signed division, and returns EAX.
//! 4D65F0/4D6570 compare the resulting deadlines as *unsigned* dwords.
//! The retained PE has no reconstructed imports: the Windows adapter below is
//! our explicit host binding, not proof of the old executable's thunk names.
use anyhow::{ensure, Result};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Counter { frequency: i64 },
    Milliseconds,
}

/// Initialization is explicit; a failed frequency query selects the fallback.
/// An allegedly successful query with a zero/negative denominator is rejected.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Clock {
    source: Source,
}
impl Clock {
    pub(crate) fn initialize(frequency: Option<i64>) -> Result<Self> {
        let source = match frequency {
            Some(frequency) => {
                ensure!(frequency > 0, "native clock invalid imported frequency");
                Source::Counter { frequency }
            }
            None => Source::Milliseconds,
        };
        Ok(Self { source })
    }

    pub(crate) fn sample(
        &self,
        counter: impl FnOnce() -> Result<i64>,
        milliseconds: impl FnOnce() -> u32,
    ) -> Result<u32> {
        match self.source {
            Source::Counter { frequency } => {
                let count = counter()?;
                ensure!(count >= 0, "native clock negative imported counter");
                // Match the x86 low-64 multiply and signed division, including
                // long uptime. Do not substitute u128 or a process-relative epoch.
                Ok((count.wrapping_mul(1000) / frequency) as u32)
            }
            Source::Milliseconds => Ok(milliseconds()),
        }
    }
}

#[cfg(windows)]
mod imports {
    #[link(name = "kernel32")]
    extern "system" {
        pub(super) fn QueryPerformanceFrequency(value: *mut i64) -> i32;
        pub(super) fn QueryPerformanceCounter(value: *mut i64) -> i32;
        pub(super) fn GetTickCount() -> u32;
    }
}

/// One initialized clock shared by controllers. Never initialized by paint or
/// asset loading; no retry with a new epoch after an import failure.
pub(crate) fn sample() -> Result<u32> {
    #[cfg(windows)]
    {
        static CLOCK: OnceLock<std::result::Result<Clock, String>> = OnceLock::new();
        let clock = CLOCK.get_or_init(|| {
            let mut frequency = 0;
            // SAFETY: a writable, correctly sized LARGE_INTEGER; no retained pointer.
            let ok = unsafe { imports::QueryPerformanceFrequency(&mut frequency) } != 0;
            Clock::initialize(ok.then_some(frequency)).map_err(|e| e.to_string())
        });
        let clock = clock.as_ref().map_err(|e| anyhow::anyhow!("{e}"))?;
        clock.sample(
            || {
                let mut count = 0;
                // SAFETY: same LARGE_INTEGER ABI as the frequency query.
                ensure!(
                    unsafe { imports::QueryPerformanceCounter(&mut count) } != 0,
                    "native clock counter import failed"
                );
                Ok(count)
            },
            // SAFETY: GetTickCount has no pointer arguments or preconditions.
            || unsafe { imports::GetTickCount() },
        )
    }
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!(
            "native clock requires the Windows host binding"
        ))
    }
}

/// No half-range wrap heuristic: this is the original unsigned comparison.
pub(crate) fn due(deadline: u32, cutoff: u32) -> bool {
    deadline <= cutoff
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn initialization_selects_one_import_and_rejects_invalid_frequency() {
        assert!(Clock::initialize(Some(0)).is_err());
        assert!(Clock::initialize(Some(-1)).is_err());
        let fallback = Clock::initialize(None).unwrap();
        assert_eq!(
            fallback
                .sample(|| panic!("counter on fallback"), || u32::MAX)
                .unwrap(),
            u32::MAX
        );
        assert_eq!(
            fallback
                .sample(|| panic!("counter on fallback"), || 0)
                .unwrap(),
            0
        );
        let clock = Clock::initialize(Some(10_000_000)).unwrap();
        assert_eq!(
            clock
                .sample(|| Ok(12_345_678), || panic!("fallback on counter"))
                .unwrap(),
            1234
        );
        assert!(clock
            .sample(
                || Err(anyhow::anyhow!("import failure")),
                || panic!("silent fallback")
            )
            .is_err());
    }

    #[test]
    fn counter_rollover_preserves_low_dword_without_rebasing() {
        let clock = Clock::initialize(Some(10_000_000)).unwrap();
        for millis in [
            u32::MAX as i64 - 1,
            u32::MAX as i64,
            1_i64 << 32,
            (1_i64 << 32) + 1,
        ] {
            assert_eq!(
                clock.sample(|| Ok(millis * 10_000), || 0).unwrap(),
                millis as u32
            );
        }
        assert!(!due(u32::MAX, 0));
        assert!(due(0, u32::MAX));
        assert!(due(0, 0));
    }

    #[test]
    fn signed_multiply_boundary_is_not_widened_or_saturated() {
        let clock = Clock::initialize(Some(1000)).unwrap();
        let count = i64::MAX / 1000 + 1;
        let product = count.wrapping_mul(1000);
        assert!(product < 0);
        assert_eq!(
            clock.sample(|| Ok(count), || 0).unwrap(),
            (product / 1000) as u32
        );
        assert!(clock.sample(|| Ok(-1), || 0).is_err());
    }

    #[test]
    fn host_import_initialization_and_sampling_succeed() {
        sample().context("first host sample").unwrap();
        sample().context("second host sample").unwrap();
    }
}
