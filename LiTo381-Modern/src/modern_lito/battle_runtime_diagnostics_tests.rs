//! Test-only, numeric progress evidence. Sampling never waits, polls, joins,
//! publishes or changes admission. Mailboxes are sampled independently: this
//! is not an atomic snapshot, a wire revision, or proof that pixels are ready.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mailbox {
    Empty,
    Ordinal(u64),
    Busy,
    Poisoned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Worker {
    pub current_attachment: bool,
    pub cancelled: bool,
    pub finished: bool,
    pub input: Mailbox,
    pub output: Mailbox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Progress {
    pub desired: Option<u64>,
    pub pending: Option<u64>,
    pub published: Option<u64>,
    pub active: Option<Worker>,
    pub retiring: Option<Worker>,
    pub closing: bool,
    pub failed: bool,
}

impl BattleRuntime {
    pub(crate) fn diagnostic_progress(&self) -> Progress {
        let worker = |job: &Job| Worker {
            current_attachment: self
                .desired
                .as_ref()
                .is_some_and(|d| Arc::ptr_eq(&d.attachment, &job.attachment)),
            cancelled: job.cancel.load(Ordering::Acquire),
            finished: job.finished(),
            input: match job.input.try_lock() {
                Ok(input) => Mailbox::Ordinal(input.ordinal),
                Err(TryLockError::WouldBlock) => Mailbox::Busy,
                Err(TryLockError::Poisoned(_)) => Mailbox::Poisoned,
            },
            output: match job.output.try_lock() {
                Ok(output) => output
                    .as_ref()
                    .map_or(Mailbox::Empty, |o| Mailbox::Ordinal(o.input.ordinal)),
                Err(TryLockError::WouldBlock) => Mailbox::Busy,
                Err(TryLockError::Poisoned(_)) => Mailbox::Poisoned,
            },
        };
        Progress {
            desired: self.desired.as_ref().map(|d| d.ordinal),
            pending: self.pending.as_ref().map(|d| d.ordinal),
            published: self.published.as_ref().map(|p| p.input.ordinal),
            active: self.active.as_ref().map(worker),
            retiring: self.retiring.as_ref().map(worker),
            closing: self.closing,
            failed: self.failed.is_some(),
        }
    }
}
