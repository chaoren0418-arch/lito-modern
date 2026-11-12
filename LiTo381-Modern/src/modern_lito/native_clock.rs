//! Native timer arithmetic, separate from packet-receipt `Instant`s.
//!
//! 4D6360 retains the imported frequency-query result. 4D69F0 multiplies the
//! counter by 1000 (low 64 bits), performs signed division, and returns EAX.
//! 4D65F0/4D6570 compare the resulting deadlines as *unsigned* dwords.
//! The retained PE has no reconstructed imports: the Windows adapter below is
//! our explicit host binding, not proof of the old executable's thunk names.
//! Its startup path does prove 482537 -> 480A60 -> 480A92 -> 4D6360;
//! initialization belongs before controller activity, never in `sample()`.
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

/// The process keeps both successful initialization and errors. An uninitialized
/// clock is distinct from an initialized clock whose frequency query failed.
#[derive(Default)]
struct ProcessClock {
    initialized: OnceLock<std::result::Result<Clock, String>>,
}
impl ProcessClock {
    fn initialize(&self, frequency: impl FnOnce() -> Option<i64>) {
        self.initialized
            .get_or_init(|| Clock::initialize(frequency()).map_err(|e| e.to_string()));
    }

    fn clock(&self) -> Result<&Clock> {
        self.initialized
            .get()
            .ok_or_else(|| anyhow::anyhow!("native clock not initialized"))?
            .as_ref()
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

#[cfg(windows)]
static PROCESS_CLOCK: ProcessClock = ProcessClock {
    initialized: OnceLock::new(),
};

/// Called when creating the controller, before receiving or painting. Errors
/// remain latched and are returned when timing is needed, not silently retried.
/// Multiple controllers/reconnects share the original process clock source.
pub(crate) fn initialize() {
    #[cfg(windows)]
    {
        PROCESS_CLOCK.initialize(|| {
            let mut frequency = 0;
            // SAFETY: a writable, correctly sized LARGE_INTEGER; no retained pointer.
            let ok = unsafe { imports::QueryPerformanceFrequency(&mut frequency) } != 0;
            ok.then_some(frequency)
        });
    }
}

/// Reading the clock must never initialize it or change its source/epoch.
pub(crate) fn sample() -> Result<u32> {
    #[cfg(windows)]
    {
        let clock = PROCESS_CLOCK.clock()?;
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
        initialize();
        sample().context("first host sample").unwrap();
        sample().context("second host sample").unwrap();
    }

    #[test]
    fn startup_is_explicit_and_latches_success_fallback_and_invalid_frequency() {
        for frequency in [Some(10_000_000), None, Some(0), Some(-1)] {
            let process = ProcessClock::default();
            assert!(process
                .clock()
                .unwrap_err()
                .to_string()
                .contains("not initialized"));
            let queries = std::cell::Cell::new(0);
            process.initialize(|| {
                queries.set(queries.get() + 1);
                frequency
            });
            process.initialize(|| panic!("controller recreation must not query again"));
            assert_eq!(queries.get(), 1);
            match frequency {
                Some(10_000_000) => {
                    let clock = process.clock().unwrap();
                    assert_eq!(
                        clock.sample(|| Ok(20_000), || panic!("fallback")).unwrap(),
                        2
                    );
                    assert!(clock
                        .sample(
                            || Err(anyhow::anyhow!("counter failed")),
                            || panic!("fallback")
                        )
                        .is_err());
                    // A sample failure cannot reset initialization or rebase time.
                    process.initialize(|| panic!("counter failure must not reinitialize"));
                    assert_eq!(
                        clock.sample(|| Ok(40_000), || panic!("fallback")).unwrap(),
                        4
                    );
                }
                None => {
                    let clock = process.clock().unwrap();
                    for ticks in [u32::MAX, 0, 1] {
                        assert_eq!(clock.sample(|| panic!("counter"), || ticks).unwrap(), ticks);
                    }
                }
                _ => assert!(process
                    .clock()
                    .unwrap_err()
                    .to_string()
                    .contains("invalid imported frequency")),
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn controller_initializes_clock_before_any_queue_sample() {
        // A separate process-clock instance above tests the empty state without
        // depending on parallel tests' access to the shared Windows clock.
        let controller = super::super::native_login::LoginController::default();
        let first = PROCESS_CLOCK.clock().unwrap();
        drop(controller);
        let _next_controller = super::super::native_login::LoginController::default();
        assert!(std::ptr::eq(first, PROCESS_CLOCK.clock().unwrap()));
    }

    /// Static source evidence only. Does not execute the PE, resolve its missing
    /// external pages, or qualify the original import names/pre-startup state.
    #[test]
    #[ignore = "requires LITO_NATIVE_CLOCK_PE pointing to the pinned read-only PE"]
    fn original_pe_clock_startup_sites_and_snapshot_match() {
        use sha2::{Digest, Sha256};
        let path = std::env::var_os("LITO_NATIVE_CLOCK_PE").expect("LITO_NATIVE_CLOCK_PE");
        let bytes = std::fs::read(path).expect("read original PE without executing it");
        assert_eq!(
            format!("{:X}", Sha256::digest(&bytes)),
            "4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9"
        );
        // The exact hash pins this rebuilt image's RVA == raw offset mapping.
        let at = |va: usize, len: usize| &bytes[va - 0x400000..va - 0x400000 + len];
        let u32_at =
            |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let header = u32_at(0x3c) as usize;
        assert_eq!(&bytes[header..header + 4], b"PE\0\0");
        assert_eq!(u32_at(header + 24 + 28), 0x400000); // PE32 ImageBase
        assert_eq!(u32_at(header + 24 + 16), 0xf162a); // reconstructed entry RVA
        for (site, target) in [
            (0x4f1759, 0x4823a0), // entry -> wrapper
            (0x482537, 0x480a60), // wrapper -> application startup
            (0x480a88, 0x4d39f0), // ordered startup calls before clock init
            (0x480a8d, 0x48e470),
            (0x480a92, 0x4d6360),
            (0x4d636a, 0x5010a68), // unresolved frequency-like external thunk
            (0x4d6a04, 0x5010863), // unresolved counter-like external thunk
            (0x4d6a37, 0x501095f), // unresolved fallback external thunk
        ] {
            let call = at(site, 5);
            assert_eq!(call[0], 0xe8, "call at {site:08X}");
            let displacement = i32::from_le_bytes(call[1..5].try_into().unwrap());
            assert_eq!((site as u32 + 5).wrapping_add(displacement as u32), target);
        }
        // The initializer retains EAX and the two output dwords, in this order.
        assert_eq!(at(0x4d6376, 5), [0xa3, 0xf0, 0x4b, 0x83, 0]);
        assert_eq!(at(0x4d637b, 6), [0x89, 0x0d, 0xe8, 0x4b, 0x83, 0]);
        assert_eq!(at(0x4d6381, 6), [0x89, 0x15, 0xec, 0x4b, 0x83, 0]);
        // sample: read retained flag, test it, jump to fallback on zero.
        assert_eq!(
            at(0x4d69f6, 9),
            [0xa1, 0xf0, 0x4b, 0x83, 0, 0x85, 0xc0, 0x74, 0x37]
        );
        assert_eq!(at(0x834be8, 12), [0; 12]); // snapshot, NOT a pre-init oracle
        let image_end = 0x400000 + u32_at(header + 24 + 56);
        assert_eq!(image_end, 0xac4000);
        assert!([0x5010a68, 0x5010863, 0x501095f]
            .iter()
            .all(|&va| va >= image_end));
    }
}
