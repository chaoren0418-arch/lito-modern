//! Ordered laps and terminal ranking, without synthesizing a legacy race
//! Frame, shifted revision, persistence receipt or combat state.
use super::{combat::require, data::*, scene::initial, ApplicationError};
use sha2::{Digest, Sha256};
const WINDOW: i64 = 10_000_000_000;
fn contains(g: &Gate, c: &Cell) -> bool {
    (g.min_x..=g.max_x).contains(&c.x) && (g.min_y..=g.max_y).contains(&c.y)
}
pub(super) fn course_hash(d: &UnvalidatedData, r: &Race) -> [u8; 32] {
    let mut b = Vec::new();
    let name = b"NEW_MATCH_PET_RACE_V1";
    b.extend((name.len() as u16).to_be_bytes());
    b.extend(name);
    b.extend(d.room.mode.to_be_bytes());
    b.extend(d.room.map.to_be_bytes());
    b.extend(d.manifest.map.to_be_bytes());
    b.extend(d.manifest.digest);
    b.extend((d.manifest.sectors.0.len() as i32).to_be_bytes());
    for s in &d.manifest.sectors.0 {
        b.extend(s.x.to_be_bytes());
        b.extend(s.y.to_be_bytes());
        b.extend(s.byte_length.to_be_bytes());
        b.extend(s.sha256);
    }
    b.extend(
        r.runners
            .0
            .first()
            .map_or(0, |r| r.base_graphics)
            .to_be_bytes(),
    );
    b.extend(
        d.actors
            .0
            .first()
            .map_or(0, |a| a.step_interval_nanos)
            .to_be_bytes(),
    );
    b.extend(4i32.to_be_bytes());
    b.extend(WINDOW.to_be_bytes());
    b.extend((r.gates.0.len() as i32).to_be_bytes());
    for g in &r.gates.0 {
        b.extend(g.min_x.to_be_bytes());
        b.extend(g.max_x.to_be_bytes());
        b.extend(g.min_y.to_be_bytes());
        b.extend(g.max_y.to_be_bytes());
    }
    Sha256::digest(b).into()
}
impl UnvalidatedData {
    pub(super) fn check_race(&self) -> Result<(), ApplicationError> {
        let Some(r) = &self.race else {
            return Ok(());
        };
        let n = self.actors.0.len();
        require(
            self.room.mode == 0
                && (2..=10).contains(&n)
                && (2..=64).contains(&r.gates.0.len())
                && self.result.is_none()
                && (0..=self.world.revision).contains(&r.revision)
                && r.active_limit_nanos
                    == i64::from(self.world.descriptor.active_timeout_millis) * 1_000_000
                && (50_000_000..=1_800_000_000_000).contains(&r.active_limit_nanos)
                && (0..=self.world.active_elapsed_nanos.max(0)).contains(&r.observed_elapsed_nanos)
                && (0..=n as i64).contains(&r.first_finisher_entity)
                && r.finish_window_deadline_elapsed_nanos >= 0
                && (0..=r.revision).contains(&r.terminal_revision)
                && (r.terminal_revision == 0 || r.terminal_revision == self.world.revision)
                && r.phase
                    == match self.world.phase {
                        Phase::Loading | Phase::Countdown => RacePhase::Waiting,
                        Phase::Active => {
                            if r.first_finisher_entity == 0 {
                                RacePhase::Running
                            } else {
                                RacePhase::FinishWindow
                            }
                        }
                        Phase::Finished => RacePhase::Complete,
                        Phase::Aborted => RacePhase::Aborted,
                    }
                && matches!(r.phase, RacePhase::Complete | RacePhase::Aborted)
                    == (r.terminal_revision > 0)
                && (r.phase == RacePhase::Complete) == r.result_id.is_some()
                && r.result_id.is_none_or(|id| id != [0; 16])
                && r.course_sha256 == course_hash(self, r),
        )?;
        for (i, g) in r.gates.0.iter().enumerate() {
            require(g.min_x <= g.max_x && g.min_y <= g.max_y && !r.gates.0[..i].contains(g))?;
        }
        let mut places = std::collections::BTreeMap::new();
        let mut occupied = std::collections::BTreeSet::new();
        for (a, v) in self.actors.0.iter().zip(&r.runners.0) {
            require(
                self.scene_member(a)?.identity.kind == MemberKind::Human
                    && a.step_interval_nanos == self.actors.0[0].step_interval_nanos
                    && a.vital.is_none()
                    && a.resources.is_none()
                    && (0..=r.revision).contains(&v.revision)
                    && v.progress_move_sequence >= 0
                    && v.progress_move_sequence <= a.last_move_sequence
                    && (0..=4).contains(&v.completed_laps)
                    && (0..r.gates.0.len() as i64).contains(&v.next_checkpoint)
                    && (0..=n as i64).contains(&v.placement)
                    && (0..r.active_limit_nanos).contains(&v.finish_elapsed_nanos)
                    && v.finish_elapsed_nanos <= r.observed_elapsed_nanos
                    && [5065, 938, 2145, 3182].contains(&v.base_graphics)
                    && v.base_graphics == r.runners.0[0].base_graphics
                    && (v.placement > 0) == (v.completed_laps == 4)
                    && (v.placement == 0
                        || v.next_checkpoint == 0
                            && v.finish_elapsed_nanos > 0
                            && v.dnf == Dnf::None)
                    && (v.placement != 0 || v.finish_elapsed_nanos == 0)
                    && (v.revision != 0
                        || v.progress_move_sequence == 0
                            && v.completed_laps == 0
                            && v.next_checkpoint == 0
                            && v.dnf == Dnf::None)
                    && (v.progress_move_sequence == 0)
                        == (v.completed_laps == 0 && v.next_checkpoint == 0)
                    && (v.revision == 0) == (v.progress_move_sequence == 0 && v.dnf == Dnf::None)
                    && v.progress_move_sequence
                        >= v.completed_laps * r.gates.0.len() as i64 + v.next_checkpoint
                    && (!initial(self.world.phase) || a.last_move_sequence == 0),
            )?;
            if v.placement > 0 {
                require(
                    places
                        .insert(v.placement, (v.entity, v.finish_elapsed_nanos))
                        .is_none(),
                )?;
            }
            if v.placement == 0 && v.dnf == Dnf::None {
                require(occupied.insert((a.position.x, a.position.y)))?;
            }
            if v.revision == self.world.revision
                && v.dnf == Dnf::None
                && v.progress_move_sequence > 0
            {
                let gate = if v.next_checkpoint == 0 {
                    r.gates.0.len() - 1
                } else {
                    v.next_checkpoint as usize - 1
                };
                require(
                    a.last_move_sequence == v.progress_move_sequence
                        && contains(&r.gates.0[gate], &a.position),
                )?;
            }
        }
        let mut previous = 0;
        for (i, (place, (_, finish))) in places.iter().enumerate() {
            require(*place == i as i64 + 1 && *finish >= previous)?;
            previous = *finish;
        }
        let first = places.get(&1).map_or(0, |v| v.0);
        require(
            first == r.first_finisher_entity
                && (first == 0) == (r.finish_window_deadline_elapsed_nanos == 0),
        )?;
        if let Some((_, time)) = places.get(&1) {
            require(time.checked_add(WINDOW) == Some(r.finish_window_deadline_elapsed_nanos))?;
        }
        if r.phase == RacePhase::Waiting {
            require(r.revision == 0 && r.observed_elapsed_nanos == 0 && first == 0)?;
        }
        if r.phase == RacePhase::Complete {
            require(
                first > 0
                    && r.finish_window_deadline_elapsed_nanos <= r.active_limit_nanos
                    && r.observed_elapsed_nanos >= r.finish_window_deadline_elapsed_nanos
                    && r.runners
                        .0
                        .iter()
                        .all(|r| r.placement > 0 || r.dnf != Dnf::None),
            )?;
        }
        // This digest checks bytes/config identity only. Standing cells and
        // directed reachability still require the qualified shared-map loader.
        Ok(())
    }
}
