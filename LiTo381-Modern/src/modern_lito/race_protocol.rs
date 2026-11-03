//! v16 NEW_MATCH_PET_RACE_V1 receipts. These are explicit modern rules, not
//! recovered native race timers. No receipt grants movement, rewards or a win.
use super::application_protocol::{self as app, ApplicationError};
use super::world_protocol::{Frame, MatchDescriptor, MatchRuleset, MemberKind, Phase};
use sha2::{Digest, Sha256};

pub const LAPS: i64 = 4;
pub const FINISH_WINDOW_NANOS: i64 = 10_000_000_000;
pub const MAX_GATES: usize = 64;
pub const MAX_RUNNERS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RaceState {
    Waiting,
    Running,
    FinishWindow,
    Complete,
    Aborted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Settlement {
    None,
    Pending,
    Saved,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i64)]
pub enum DnfReason {
    None,
    Left,
    Disconnected,
    WindowClosed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Gate {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}
impl Gate {
    pub fn contains(&self, cell: super::world_map::WorldCell) -> bool {
        (self.min_x..=self.max_x).contains(&cell.x) && (self.min_y..=self.max_y).contains(&cell.y)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Runner {
    pub entity_id: u64,
    pub revision: i64,
    pub progress_move_sequence: u64,
    pub completed_laps: i64,
    pub next_checkpoint: i64,
    pub placement: i64,
    pub finish_elapsed_nanos: i64,
    pub dnf: DnfReason,
    pub base_graphics_id: i64,
}
impl Runner {
    pub fn competing(&self) -> bool {
        self.placement == 0 && self.dnf == DnfReason::None
    }
    fn progress(&self, gates: usize) -> i64 {
        self.completed_laps * gates as i64 + self.next_checkpoint
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceData {
    pub state: RaceState,
    pub settlement: Settlement,
    pub course_sha256: [u8; 32],
    pub first_finisher_entity_id: u64,
    pub finish_window_deadline_elapsed_nanos: i64,
    pub terminal_revision: i64,
    pub result_id: [u8; 16],
    pub result_content_sha256: [u8; 32],
    pub gates: Vec<Gate>,
    pub runners: Vec<Runner>,
}
impl RaceData {
    pub fn runner(&self, entity_id: u64) -> Option<&Runner> {
        self.runners.iter().find(|r| r.entity_id == entity_id)
    }
    pub fn competing(&self, entity_id: u64) -> bool {
        self.runner(entity_id).is_some_and(Runner::competing)
    }
    pub fn validate(&self, frame: &Frame) -> Result<(), ApplicationError> {
        if !frame.descriptor.is_some_and(|d| {
            d.ruleset == MatchRuleset::PetRace && d.features == MatchDescriptor::PET_RACE
        }) || frame.room.summary.settings.mode != 0
            || frame.completion.is_some()
            || !frame.casts.is_empty()
            || !(2..=MAX_GATES).contains(&self.gates.len())
            || !(2..=MAX_RUNNERS).contains(&self.runners.len())
            || self.runners.len() != frame.actors.len()
            || self.terminal_revision < 0
            || self.terminal_revision > frame.revision
            || self.first_finisher_entity_id > self.runners.len() as u64
            || self.finish_window_deadline_elapsed_nanos < 0
        {
            return Err(invalid());
        }
        let phase_matches = match self.state {
            RaceState::Waiting => matches!(
                frame.phase,
                Phase::Preparing | Phase::Loading | Phase::Countdown
            ),
            RaceState::Running | RaceState::FinishWindow => frame.phase == Phase::Active,
            RaceState::Complete => frame.phase == Phase::Finished,
            RaceState::Aborted => frame.phase == Phase::Aborted,
        };
        let terminal = matches!(self.state, RaceState::Complete | RaceState::Aborted);
        if !phase_matches
            || terminal != (self.terminal_revision > 0)
            || (self.state == RaceState::Complete) != (self.settlement != Settlement::None)
            || (self.state == RaceState::Complete) != (self.result_id != [0; 16])
            || (self.state == RaceState::Complete) != (self.result_content_sha256 != [0; 32])
        {
            return Err(invalid());
        }
        let mut unique = std::collections::HashSet::new();
        for gate in &self.gates {
            if gate.min_x > gate.max_x
                || gate.min_y > gate.max_y
                || !unique.insert(*gate)
                || !frame.manifest.sectors.iter().any(|s| {
                    let x = 32704 + (i64::from(s.x) - 0x7fff) * 64;
                    let y = 32704 + (i64::from(s.y) - 0x7fff) * 64;
                    i64::from(gate.min_x) < x + 64
                        && i64::from(gate.max_x) >= x
                        && i64::from(gate.min_y) < y + 64
                        && i64::from(gate.max_y) >= y
                })
            {
                return Err(invalid());
            }
        }
        let mut places = std::collections::BTreeMap::new();
        let active_limit = i64::from(frame.descriptor.unwrap().active_timeout_millis) * 1_000_000;
        let gfx = self.runners[0].base_graphics_id;
        let interval = frame.actors[0].step_interval_nanos;
        for (index, (runner, actor)) in self.runners.iter().zip(&frame.actors).enumerate() {
            if runner.entity_id != index as u64 + 1
                || runner.entity_id != actor.entity_id
                || actor.kind != MemberKind::Human
                || actor.step_interval_nanos != interval
                || !matches!(runner.base_graphics_id, 5065 | 938 | 2145 | 3182)
                || runner.base_graphics_id != gfx
                || runner.revision < 0
                || runner.revision > frame.revision
                || terminal && runner.revision > self.terminal_revision
                || !(0..=LAPS).contains(&runner.completed_laps)
                || !(0..self.gates.len() as i64).contains(&runner.next_checkpoint)
                || !(0..=self.runners.len() as i64).contains(&runner.placement)
                || runner.progress_move_sequence > actor.sequence
                || runner.progress_move_sequence < runner.progress(self.gates.len()) as u64
                || !(0..active_limit).contains(&runner.finish_elapsed_nanos)
                || (runner.placement > 0) != (runner.completed_laps == LAPS)
                || runner.placement > 0
                    && (runner.next_checkpoint != 0
                        || runner.finish_elapsed_nanos == 0
                        || runner.dnf != DnfReason::None)
                || runner.placement == 0 && runner.finish_elapsed_nanos != 0
                || (runner.progress_move_sequence == 0) != (runner.progress(self.gates.len()) == 0)
                || (runner.revision == 0)
                    != (runner.progress_move_sequence == 0 && runner.dnf == DnfReason::None)
                || self.state == RaceState::Waiting && (runner.revision != 0 || actor.sequence != 0)
                || self.state == RaceState::Complete && runner.competing()
                || runner.dnf == DnfReason::WindowClosed
                    && (self.state != RaceState::Complete
                        || runner.revision != self.terminal_revision)
            {
                return Err(invalid());
            }
            if runner.revision == frame.revision
                && runner.dnf == DnfReason::None
                && runner.progress_move_sequence > 0
            {
                let index = if runner.next_checkpoint == 0 {
                    self.gates.len() - 1
                } else {
                    runner.next_checkpoint as usize - 1
                };
                let gate = self.gates[index];
                if actor.sequence != runner.progress_move_sequence
                    || !(gate.min_x..=gate.max_x).contains(&actor.position.x)
                    || !(gate.min_y..=gate.max_y).contains(&actor.position.y)
                {
                    return Err(invalid());
                }
            }
            if runner.placement > 0 && places.insert(runner.placement, runner).is_some() {
                return Err(invalid());
            }
        }
        let mut last_finish = 0;
        for (index, (place, runner)) in places.iter().enumerate() {
            if *place != index as i64 + 1 || runner.finish_elapsed_nanos < last_finish {
                return Err(invalid());
            }
            last_finish = runner.finish_elapsed_nanos;
        }
        let first = places.get(&1).copied();
        if first.map_or(0, |r| r.entity_id) != self.first_finisher_entity_id
            || first.map_or(Some(0), |r| {
                r.finish_elapsed_nanos.checked_add(FINISH_WINDOW_NANOS)
            }) != Some(self.finish_window_deadline_elapsed_nanos)
            || matches!(self.state, RaceState::Waiting | RaceState::Running) && first.is_some()
            || matches!(self.state, RaceState::FinishWindow | RaceState::Complete)
                && first.is_none()
            || self.state == RaceState::Complete
                && self.finish_window_deadline_elapsed_nanos > active_limit
            || first.is_some() && last_finish > self.finish_window_deadline_elapsed_nanos
            || self.course_sha256 != self.course_hash(frame)
            || self.state == RaceState::Complete
                && self.result_content_sha256 != self.result_hash(frame)
            || self.state == RaceState::Complete
                && match self.settlement {
                    Settlement::Pending => frame.revision != self.terminal_revision,
                    Settlement::Saved | Settlement::Failed => {
                        self.terminal_revision.checked_add(1) != Some(frame.revision)
                    }
                    Settlement::None => true,
                }
        {
            return Err(invalid());
        }
        Ok(())
    }
    /// Independent canonical SHA; ASCII domains use Java DataOutput.writeUTF's
    /// u16 byte length, followed by explicitly sized big-endian integers.
    pub fn course_hash(&self, frame: &Frame) -> [u8; 32] {
        let mut out = Vec::new();
        utf(&mut out, b"NEW_MATCH_PET_RACE_V1");
        context(&mut out, frame);
        out.extend(frame.manifest.digest);
        out.extend((frame.manifest.sectors.len() as i32).to_be_bytes());
        for sector in &frame.manifest.sectors {
            out.extend(i32::from(sector.x).to_be_bytes());
            out.extend(i32::from(sector.y).to_be_bytes());
            out.extend(sector.byte_length.to_be_bytes());
            out.extend(sector.sha256);
        }
        out.extend(
            self.runners
                .first()
                .map_or(0, |r| r.base_graphics_id)
                .to_be_bytes(),
        );
        out.extend(
            frame
                .actors
                .first()
                .map_or(0, |a| a.step_interval_nanos)
                .to_be_bytes(),
        );
        out.extend((LAPS as i32).to_be_bytes());
        out.extend(FINISH_WINDOW_NANOS.to_be_bytes());
        out.extend((self.gates.len() as i32).to_be_bytes());
        for gate in &self.gates {
            put_gate(&mut out, gate);
        }
        Sha256::digest(out).into()
    }
    pub fn result_hash(&self, frame: &Frame) -> [u8; 32] {
        let mut out = Vec::new();
        utf(&mut out, b"LITO_MODERN_PET_RACE_RESULT_V1");
        out.extend(self.result_id);
        out.extend(frame.match_id.to_be_bytes());
        out.extend(frame.generation.to_be_bytes());
        out.extend(self.terminal_revision.to_be_bytes());
        context(&mut out, frame);
        out.extend(frame.manifest.digest);
        out.extend(self.course_sha256);
        out.extend(self.first_finisher_entity_id.to_be_bytes());
        out.extend(self.finish_window_deadline_elapsed_nanos.to_be_bytes());
        out.extend((self.runners.len() as i32).to_be_bytes());
        for (runner, actor) in self.runners.iter().zip(&frame.actors) {
            out.extend(runner.entity_id.to_be_bytes());
            out.extend(actor.key.source_id.to_be_bytes());
            out.extend(actor.key.object_id.to_be_bytes());
            put_runner_tail(&mut out, runner);
        }
        Sha256::digest(out).into()
    }
}

pub fn follows(old: &Frame, new: &Frame) -> bool {
    let (Some(a), Some(b)) = (&old.race, &new.race) else {
        return old.race.is_none() && new.race.is_none();
    };
    if a.validate(old).is_err()
        || b.validate(new).is_err()
        || a.course_sha256 != b.course_sha256
        || a.gates != b.gates
        || a.runners.len() != b.runners.len()
        || (b.state as u8) < a.state as u8
        || a.first_finisher_entity_id != 0
            && (a.first_finisher_entity_id != b.first_finisher_entity_id
                || a.finish_window_deadline_elapsed_nanos != b.finish_window_deadline_elapsed_nanos)
        || a.terminal_revision != 0
            && (a.terminal_revision != b.terminal_revision
                || a.result_id != b.result_id
                || a.result_content_sha256 != b.result_content_sha256)
    {
        return false;
    }
    a.runners
        .iter()
        .zip(&b.runners)
        .zip(old.actors.iter().zip(&new.actors))
        .all(|((a, b), (x, y))| {
            let progress = b.progress(old.race.as_ref().unwrap().gates.len())
                - a.progress(old.race.as_ref().unwrap().gates.len());
            let single_crossing = progress != 1
                || y.sequence.checked_sub(x.sequence) != Some(1)
                || a.next_checkpoint >= 0
                    && old
                        .race
                        .as_ref()
                        .unwrap()
                        .gates
                        .get(a.next_checkpoint as usize)
                        .is_some_and(|g| !g.contains(x.position) && g.contains(y.position));
            a.entity_id == b.entity_id
                && single_crossing
                && a.base_graphics_id == b.base_graphics_id
                && b.revision >= a.revision
                && b.progress_move_sequence >= a.progress_move_sequence
                && progress >= 0
                && y.sequence
                    .checked_sub(x.sequence)
                    .is_some_and(|d| progress as u64 <= d)
                && (a.revision != b.revision || a == b)
                && (a.revision == b.revision
                    || progress > 0
                    || a.dnf == DnfReason::None && b.dnf != DnfReason::None)
                && (a == b || b.revision > old.revision)
                && (progress == 0 || b.progress_move_sequence > x.sequence)
                && (progress != 0 || b.progress_move_sequence == a.progress_move_sequence)
                && (a.competing()
                    || a == b
                        && x.sequence == y.sequence
                        && x.position == y.position
                        && x.heading == y.heading)
        })
}
pub fn terminal_receipt_follows(old: &Frame, new: &Frame) -> bool {
    let (Some(a), Some(b)) = (&old.race, &new.race) else {
        return false;
    };
    let mut receipt = a.clone();
    receipt.settlement = b.settlement;
    a.state == RaceState::Complete
        && new.phase == Phase::Finished
        && a.settlement == Settlement::Pending
        && matches!(b.settlement, Settlement::Saved | Settlement::Failed)
        && receipt == *b
        && old.actors == new.actors
        && old.casts == new.casts
        && old.revision.checked_add(1) == Some(new.revision)
}
fn context(out: &mut Vec<u8>, f: &Frame) {
    out.extend(i32::from(f.room.summary.settings.mode).to_be_bytes());
    out.extend(f.room.summary.settings.map_id.to_be_bytes());
    out.extend(f.manifest.map_id.to_be_bytes());
}
fn utf(out: &mut Vec<u8>, domain: &[u8]) {
    out.extend((domain.len() as u16).to_be_bytes());
    out.extend(domain);
}
fn put_gate(out: &mut Vec<u8>, g: &Gate) {
    for v in [g.min_x, g.max_x, g.min_y, g.max_y] {
        out.extend(v.to_be_bytes());
    }
}
fn put_runner_tail(out: &mut Vec<u8>, r: &Runner) {
    out.extend(r.revision.to_be_bytes());
    out.extend(r.progress_move_sequence.to_be_bytes());
    for v in [
        r.completed_laps,
        r.next_checkpoint,
        r.placement,
        r.finish_elapsed_nanos,
        r.dnf as i64,
        r.base_graphics_id,
    ] {
        out.extend(v.to_be_bytes());
    }
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid pet race receipt")
}

pub(crate) fn decode(bytes: &[u8]) -> Result<(Option<RaceData>, usize), ApplicationError> {
    let mut r = Reader { bytes, pos: 0 };
    let enabled = r.byte()?;
    if enabled == 0 {
        if r.take(7)?.iter().any(|b| *b != 0) {
            return Err(invalid());
        }
        return Ok((None, r.pos));
    }
    if enabled != 1 {
        return Err(invalid());
    }
    let state = match r.byte()? {
        0 => RaceState::Waiting,
        1 => RaceState::Running,
        2 => RaceState::FinishWindow,
        3 => RaceState::Complete,
        4 => RaceState::Aborted,
        _ => return Err(invalid()),
    };
    let settlement = match r.byte()? {
        0 => Settlement::None,
        1 => Settlement::Pending,
        2 => Settlement::Saved,
        3 => Settlement::Failed,
        _ => return Err(invalid()),
    };
    if r.byte()? != 0 {
        return Err(invalid());
    }
    let gates = u16::from_be_bytes(r.take(2)?.try_into().unwrap()) as usize;
    let runners = r.byte()? as usize;
    if r.byte()? != LAPS as u8
        || !(2..=MAX_GATES).contains(&gates)
        || !(2..=MAX_RUNNERS).contains(&runners)
        || bytes.len() < 112 + gates * 16 + runners * 72
    {
        return Err(invalid());
    }
    let mut data = RaceData {
        state,
        settlement,
        course_sha256: r.take(32)?.try_into().unwrap(),
        first_finisher_entity_id: r.long()? as u64,
        finish_window_deadline_elapsed_nanos: r.long()?,
        terminal_revision: r.long()?,
        result_id: r.take(16)?.try_into().unwrap(),
        result_content_sha256: r.take(32)?.try_into().unwrap(),
        gates: Vec::with_capacity(gates),
        runners: Vec::with_capacity(runners),
    };
    for _ in 0..gates {
        data.gates.push(Gate {
            min_x: r.int()?,
            max_x: r.int()?,
            min_y: r.int()?,
            max_y: r.int()?,
        });
    }
    for _ in 0..runners {
        let mut row = Runner {
            entity_id: r.long()? as u64,
            revision: r.long()?,
            progress_move_sequence: r.long()? as u64,
            completed_laps: r.long()?,
            next_checkpoint: r.long()?,
            placement: r.long()?,
            finish_elapsed_nanos: r.long()?,
            dnf: DnfReason::None,
            base_graphics_id: 0,
        };
        row.dnf = match r.long()? {
            0 => DnfReason::None,
            1 => DnfReason::Left,
            2 => DnfReason::Disconnected,
            3 => DnfReason::WindowClosed,
            _ => return Err(invalid()),
        };
        row.base_graphics_id = r.long()?;
        data.runners.push(row);
    }
    Ok((Some(data), r.pos))
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(invalid)?;
        let out = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
        self.pos = end;
        Ok(out)
    }
    fn byte(&mut self) -> Result<u8, ApplicationError> {
        Ok(self.take(1)?[0])
    }
    fn long(&mut self) -> Result<i64, ApplicationError> {
        Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn int(&mut self) -> Result<i32, ApplicationError> {
        Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
}
#[cfg(test)]
pub(crate) fn encode(out: &mut Vec<u8>, race: Option<&RaceData>) {
    let Some(r) = race else {
        out.extend([0; 8]);
        return;
    };
    out.extend([1, r.state as u8, r.settlement as u8, 0]);
    out.extend((r.gates.len() as u16).to_be_bytes());
    out.extend([r.runners.len() as u8, LAPS as u8]);
    out.extend(r.course_sha256);
    out.extend(r.first_finisher_entity_id.to_be_bytes());
    out.extend(r.finish_window_deadline_elapsed_nanos.to_be_bytes());
    out.extend(r.terminal_revision.to_be_bytes());
    out.extend(r.result_id);
    out.extend(r.result_content_sha256);
    for gate in &r.gates {
        put_gate(out, gate);
    }
    for runner in &r.runners {
        out.extend(runner.entity_id.to_be_bytes());
        put_runner_tail(out, runner);
    }
}

#[cfg(test)]
#[path = "race_protocol_tests.rs"]
pub(crate) mod tests;
