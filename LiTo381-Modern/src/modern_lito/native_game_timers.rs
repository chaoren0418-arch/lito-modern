//! Original 47B720/47B750 GAME callbacks. Deadlines use monotonic elapsed time;
//! they never advance the server's game calendar or predict world authority.
use std::time::{Duration, Instant};

pub const ALIVE_INTERVAL: Duration = Duration::from_secs(60);
pub const SAVE_INTERVAL: Duration = Duration::from_secs(300);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Alive,
    Save,
}
impl Kind {
    pub fn opcode(self) -> u8 {
        match self {
            Self::Alive => 0x20,
            Self::Save => 0x1d,
        }
    }
    fn interval(self) -> Duration {
        match self {
            Self::Alive => ALIVE_INTERVAL,
            Self::Save => SAVE_INTERVAL,
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Alive => 0,
            Self::Save => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Snapshot {
    pub enabled: bool,
    pub alive_sent: u64,
    pub save_sent: u64,
}

#[derive(Default)]
pub struct Schedule {
    begun: bool,
    snapshot: Snapshot,
    deadlines: [Option<Instant>; 2],
    pending: [bool; 2],
}
impl Schedule {
    /// Original world entry calls Alive immediately and schedules Save later.
    /// Re-entering a scene on the same connection does not duplicate the owner.
    pub fn begin(&mut self, now: Instant) -> bool {
        if self.begun {
            return false;
        }
        self.begun = true;
        self.snapshot.enabled = true;
        self.deadlines = [Some(now), None];
        self.pending = [false; 2];
        true
    }
    pub fn stop(&mut self) {
        self.begun = true;
        self.snapshot.enabled = false;
        self.deadlines = [None; 2];
    }
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot
    }
    /// At most one outstanding instance of each callback. A late callback is
    /// sent once, then rearmed from its actual completion; no catch-up burst.
    pub fn due(&mut self, now: Instant) -> Vec<Kind> {
        if !self.snapshot.enabled {
            return vec![];
        }
        let mut due: Vec<_> = [Kind::Save, Kind::Alive]
            .into_iter()
            .filter(|kind| {
                let i = kind.index();
                !self.pending[i] && self.deadlines[i].is_some_and(|d| d <= now)
            })
            .collect();
        // Save was registered earlier when equal 300s deadlines coincide.
        due.sort_by_key(|kind| self.deadlines[kind.index()]);
        for kind in &due {
            self.pending[kind.index()] = true;
        }
        due
    }
    pub fn sent(&mut self, kind: Kind, now: Instant) {
        let count = match kind {
            Kind::Alive => &mut self.snapshot.alive_sent,
            Kind::Save => &mut self.snapshot.save_sent,
        };
        *count = count.saturating_add(1);
        self.pending[kind.index()] = false;
        self.deadlines[kind.index()] = self.snapshot.enabled.then(|| now + kind.interval());
        if self.snapshot.enabled && kind == Kind::Alive && self.snapshot.alive_sent == 1 {
            self.deadlines[Kind::Save.index()] = Some(now + SAVE_INTERVAL);
        }
    }
}
