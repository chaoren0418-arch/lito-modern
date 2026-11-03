//! Precisely pinned 5350 ROOM start components. Source milestones and receive
//! observations are separate. Room MatchState RUNNING never admits WORLD,
//! movement, renderer actors or a synchronized Native/server clock.
use super::native_city_runtime::{java_room_participants, java_room_ready};
use super::native_pet_race_component::{
    Cell, Contract, PanelTransition, PanelType, Reducer, Transition, NAMESPACE,
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
pub(super) struct Member {
    pub object_id: u32,
    pub alias_hex: String,
    pub prepared_x: u16,
    pub prepared_y: u16,
    pub prepared_gfx: u16,
}
#[derive(Clone, Copy, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum MilestoneKind {
    CountdownStarted,
    Tick,
    Begin,
    Prepare,
    DirectGate,
    Teardown,
}
#[derive(Clone, Deserialize)]
pub(super) struct Milestone {
    pub kind: MilestoneKind,
    pub after_rx_offset: usize,
    pub value: u8,
    pub source_now: Option<i64>,
    pub source_next: i64,
}
#[derive(Clone, Deserialize)]
pub(super) struct DirectGate {
    object_id: u32,
    from: [i32; 2],
    to: [i32; 2],
    elapsed_nanos: i64,
}
#[derive(Clone, Deserialize)]
pub(super) struct StartBinding {
    pub room_id: u32,
    pub map_id: u32,
    pub members: Vec<Member>,
    pub local_object_id: u32,
    pub generation_initial: u64,
    pub generation_begin: u64,
    pub generation_teardown: u64,
    pub source_start_nanos: i64,
    pub source_deadline_nanos: i64,
    pub milestones: Vec<Milestone>,
    direct_gate: DirectGate,
    pub ready_frame_hex: String,
    minimum_three_is_fixture_precondition: bool,
    scene_prerequisites_qualified: bool,
}
impl StartBinding {
    pub(super) fn local_alias(&self) -> Result<Vec<u8>> {
        super::native_pet_race_wire_boundary::from_hex(
            &self
                .members
                .iter()
                .find(|m| m.object_id == self.local_object_id)
                .context("missing fixture local member")?
                .alias_hex,
        )
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReadyObservation {
    pub alias_raw: Vec<u8>,
    pub state_wire: u8,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ActorObservation {
    pub object_id: u32,
    pub x: u16,
    pub y: u16,
    pub gfx: u16,
    pub name_raw: Vec<u8>,
    pub walk_action: u8,
    pub heading: u8,
    pub light_range: u8,
    pub speed: u8,
    pub lawful: i16,
    pub title_raw: Vec<u8>,
    pub status: u8,
    pub emblem_id: u32,
    pub clan_raw: Vec<u8>,
    pub hp_ratio: u8,
    pub level: u8,
    pub raw_body: Vec<u8>,
    pub ancillary_fields_synthetic: bool,
    pub scene_prerequisites_qualified: bool,
    pub drawable_actor_installed: bool,
}
struct Bytes<'a> {
    body: &'a [u8],
    at: usize,
}
impl Bytes<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self.at.checked_add(n).context("actor offset overflow")?;
        let v = self
            .body
            .get(self.at..end)
            .context("short pinned actor body")?;
        self.at = end;
        Ok(v)
    }
    fn c(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn h(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    fn d(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn z(&mut self) -> Result<Vec<u8>> {
        let n = self.body[self.at..]
            .iter()
            .take(1025)
            .position(|b| *b == 0)
            .context("bounded actor CString absent")?;
        let value = self.take(n)?.to_vec();
        self.take(1)?;
        Ok(value)
    }
}
/// Exact Protocol.battleActorWithHeading layout, qualified solely as component
/// output. This decoder does not install an actor into Native scene tables.
pub(super) fn decode_actor(body: &[u8]) -> Result<ActorObservation> {
    ensure!(body.len() <= 4096, "actor component bound");
    let mut r = Bytes { body, at: 0 };
    ensure!(r.c()? == 0x0B, "actor component opcode");
    let x = r.h()?;
    let y = r.h()?;
    let object_id = r.d()?;
    let gfx = r.h()?;
    let walk_action = r.c()?;
    let heading = r.c()?;
    let light_range = r.c()?;
    let speed = r.c()?;
    ensure!(
        object_id != 0 && heading <= 7 && speed <= 2 && r.d()? == 0,
        "bounded actor source header"
    );
    let lawful = r.h()? as i16;
    let name_raw = r.z()?;
    let title_raw = r.z()?;
    let status = r.c()?;
    let emblem_id = r.d()?;
    let clan_raw = r.z()?;
    ensure!(
        !name_raw.is_empty()
            && (emblem_id == 0 || !clan_raw.is_empty())
            && r.z()?.is_empty()
            && r.c()? == 0,
        "source actor strings/reserved fields"
    );
    let hp_ratio = r.c()?;
    ensure!(r.c()? == 0, "actor source reserved ratio byte");
    let level = r.c()?;
    ensure!(
        r.z()?.is_empty() && r.h()? == 0 && r.c()? == 0 && r.at == body.len(),
        "complete actor source tail"
    );
    Ok(ActorObservation {
        object_id,
        x,
        y,
        gfx,
        name_raw,
        walk_action,
        heading,
        light_range,
        speed,
        lawful,
        title_raw,
        status,
        emblem_id,
        clan_raw,
        hp_ratio,
        level,
        raw_body: body.to_vec(),
        ancillary_fields_synthetic: true,
        scene_prerequisites_qualified: false,
        drawable_actor_installed: false,
    })
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum SourceMatchPhase {
    Waiting,
    Countdown,
    Running,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct OfflineClock {
    pub elapsed_nanos: i64,
    pub deadline_elapsed_nanos: i64,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct PanelObservation {
    pub kind: PanelType,
    pub duration_seconds: u8,
    pub trailing: bool,
}
impl From<PanelTransition> for PanelObservation {
    fn from(value: PanelTransition) -> Self {
        Self {
            kind: value.kind,
            duration_seconds: value.duration_seconds,
            trailing: value.trailing,
        }
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Snapshot {
    pub room_id: u32,
    pub map_id: u32,
    pub client_phase: &'static str,
    pub ready_observations: Vec<ReadyObservation>,
    pub countdown_receipts: Vec<u8>,
    pub actors: Vec<ActorObservation>,
    pub hud_timer_start_observed: bool,
    pub source_match_phase: SourceMatchPhase,
    pub source_local_generation: u64,
    pub source_roster_ids: Vec<u32>,
    pub source_next_nanos: Option<i64>,
    pub source_start_nanos: Option<i64>,
    pub source_deadline_nanos: Option<i64>,
    pub source_milestones: Vec<MilestoneKind>,
    pub source_prepare_observed: bool,
    pub source_direct_gate_observed: bool,
    pub offline_direct_gate: Option<Transition>,
    pub offline_panel: Option<PanelObservation>,
    pub offline_clock: Option<OfflineClock>,
    pub local_clock_synchronized: bool,
    pub own_ready_request_assumed_written: bool,
    pub active: bool,
    pub teardown_observed: bool,
    pub closed: bool,
    pub wire_generation_known: bool,
    pub live_world_admitted: bool,
    pub movement_enabled: bool,
    pub renderer_loaded: bool,
    pub full_player_ack: bool,
    pub battle_rules_whole_class_equivalent: bool,
}
pub(super) struct StartComponent {
    binding: StartBinding,
    state: Snapshot,
    reducer: Option<Reducer>,
}
impl StartComponent {
    pub(super) fn begin(binding: StartBinding) -> Result<Self> {
        ensure!(
            binding.room_id == 2
                && binding.map_id == 566
                && binding.members.len() == 3
                && binding.minimum_three_is_fixture_precondition
                && !binding.scene_prerequisites_qualified,
            "pinned ROOM fixture only"
        );
        binding.local_alias()?;
        let generation = binding.generation_initial;
        Ok(Self {
            state: Snapshot {
                room_id: binding.room_id,
                map_id: binding.map_id,
                client_phase: "ROOM",
                ready_observations: Vec::new(),
                countdown_receipts: Vec::new(),
                actors: Vec::new(),
                hud_timer_start_observed: false,
                source_match_phase: SourceMatchPhase::Waiting,
                source_local_generation: generation,
                source_roster_ids: Vec::new(),
                source_next_nanos: None,
                source_start_nanos: None,
                source_deadline_nanos: None,
                source_milestones: Vec::new(),
                source_prepare_observed: false,
                source_direct_gate_observed: false,
                offline_direct_gate: None,
                offline_panel: None,
                offline_clock: None,
                local_clock_synchronized: false,
                own_ready_request_assumed_written: false,
                active: false,
                teardown_observed: false,
                closed: false,
                wire_generation_known: false,
                live_world_admitted: false,
                movement_enabled: false,
                renderer_loaded: false,
                full_player_ack: false,
                battle_rules_whole_class_equivalent: false,
            },
            binding,
            reducer: None,
        })
    }
    pub(super) fn snapshot(&self) -> Snapshot {
        self.state.clone()
    }
    pub(super) fn observe(&mut self, body: &[u8]) -> Result<()> {
        ensure!(!self.state.closed, "retired start component scope");
        match body.first() {
            Some(0xD1) => {
                let java_room_participants::Packet::State(r) =
                    java_room_participants::decode(body)?
                else {
                    anyhow::bail!("READY observation subset")
                };
                ensure!(
                    self.binding.members.iter().any(|m| {
                        super::native_pet_race_wire_boundary::from_hex(&m.alias_hex)
                            .is_ok_and(|name| name == r.display_alias)
                    }) && self.state.ready_observations.len() < 64,
                    "known fixture member/bounded READY observations"
                );
                self.state.ready_observations.push(ReadyObservation {
                    alias_raw: r.display_alias,
                    state_wire: r.state.wire(),
                });
            }
            Some(0xE1) => {
                let value = java_room_ready::decode_countdown(body)?;
                ensure!(
                    self.state.countdown_receipts.len() < 64,
                    "bounded countdown observations"
                );
                self.state.countdown_receipts.push(value);
            }
            Some(0x0B) => {
                let actor = decode_actor(body)?;
                let member = self
                    .binding
                    .members
                    .iter()
                    .find(|m| m.object_id == actor.object_id)
                    .context("unknown component actor")?;
                ensure!(
                    actor.name_raw
                        == super::native_pet_race_wire_boundary::from_hex(&member.alias_hex)?
                        && (actor.x, actor.y, actor.gfx)
                            == (member.prepared_x, member.prepared_y, member.prepared_gfx)
                        && self.state.actors.len() < 10,
                    "prepared fixture actor fields"
                );
                ensure!(
                    !self
                        .state
                        .actors
                        .iter()
                        .any(|a| a.object_id == actor.object_id),
                    "duplicate actor in fixed component subset"
                );
                self.state.actors.push(actor);
            }
            _ => anyhow::bail!("not a READY/countdown/actor component observation"),
        }
        Ok(())
    }
    pub(super) fn hud_observed(&mut self, body: &[u8]) {
        if body == [0xFD, 0] {
            self.state.hud_timer_start_observed = true;
        }
    }
    pub(super) fn ready_written(&mut self) {
        self.state.own_ready_request_assumed_written = true;
    }
    pub(super) fn next_milestone(&mut self, offset: usize) -> Result<MilestoneKind> {
        ensure!(!self.state.closed, "retired source component milestone");
        let e = self
            .binding
            .milestones
            .get(self.state.source_milestones.len())
            .context("no next source milestone")?
            .clone();
        ensure!(
            e.after_rx_offset == offset,
            "exact offline source milestone window"
        );
        match e.kind {
            MilestoneKind::CountdownStarted => {
                ensure!(
                    self.state.countdown_receipts.last() == Some(&3)
                        && self.binding.members.iter().all(|m| {
                            super::native_pet_race_wire_boundary::from_hex(&m.alias_hex).is_ok_and(
                                |alias| {
                                    self.state
                                        .ready_observations
                                        .iter()
                                        .rev()
                                        .find(|r| r.alias_raw == alias)
                                        .is_some_and(|r| r.state_wire == 1)
                                },
                            )
                        }),
                    "source READY/countdown evidence incomplete"
                );
                self.state.source_match_phase = SourceMatchPhase::Countdown;
                self.state.source_next_nanos = Some(e.source_next);
            }
            MilestoneKind::Tick | MilestoneKind::Begin => {
                ensure!(
                    self.state.source_match_phase == SourceMatchPhase::Countdown
                        && e.source_now.context("source monotonic now absent")?
                            >= self.state.source_next_nanos.context("source next absent")?,
                    "source countdown due guard"
                );
                if e.kind == MilestoneKind::Tick {
                    ensure!(
                        self.state.countdown_receipts.last() == Some(&e.value),
                        "source tick/received count mismatch"
                    );
                    self.state.source_next_nanos = Some(e.source_next);
                } else {
                    ensure!(
                        e.source_now == Some(self.binding.source_start_nanos)
                            && self
                                .binding
                                .source_deadline_nanos
                                .checked_sub(self.binding.source_start_nanos)
                                == Some(300_000_000_000),
                        "source begin monotonic domain/duration"
                    );
                    self.state.source_match_phase = SourceMatchPhase::Running;
                    self.state.source_local_generation = self.binding.generation_begin;
                    self.state.source_roster_ids =
                        self.binding.members.iter().map(|m| m.object_id).collect();
                    self.state.source_start_nanos = Some(self.binding.source_start_nanos);
                    self.state.source_deadline_nanos = Some(self.binding.source_deadline_nanos);
                    self.state.source_next_nanos = None;
                    self.state.active = true;
                }
            }
            MilestoneKind::Prepare => {
                ensure!(
                    self.state.source_match_phase == SourceMatchPhase::Running && self.state.active,
                    "prepare requires source RUNNING"
                );
                self.state.source_prepare_observed = true;
            }
            MilestoneKind::DirectGate => {
                ensure!(
                    self.state.active
                        && self.state.source_prepare_observed
                        && self.state.actors.len() == 3,
                    "source direct gate component prerequisites"
                );
                if self.reducer.is_some() {
                    self.advance_elapsed(self.binding.direct_gate.elapsed_nanos)?;
                    let d = &self.binding.direct_gate;
                    let from = Cell {
                        x: d.from[0],
                        y: d.from[1],
                    };
                    let to = Cell {
                        x: d.to[0],
                        y: d.to[1],
                    };
                    let reducer = self.reducer.as_mut().unwrap();
                    self.state.offline_panel = Some(
                        reducer
                            .panel_moved(i64::from(d.object_id), from, to, d.elapsed_nanos)
                            .into(),
                    );
                    self.state.offline_direct_gate =
                        Some(reducer.moved(i64::from(d.object_id), from, to, d.elapsed_nanos));
                }
                self.state.source_direct_gate_observed = true;
            }
            MilestoneKind::Teardown => {
                ensure!(
                    self.state.active
                        && self.state.source_local_generation == self.binding.generation_begin,
                    "teardown source generation"
                );
                self.state.source_local_generation = self.binding.generation_teardown;
                self.state.source_match_phase = SourceMatchPhase::Waiting;
                self.state.active = false;
                self.state.teardown_observed = true;
                self.state.offline_clock = None;
                self.reducer = None;
            }
        }
        self.state.source_milestones.push(e.kind);
        Ok(e.kind)
    }
    pub(super) fn begin_restricted_clock(&mut self, c: &Contract) -> Result<()> {
        ensure!(
            !self.state.closed
                && self.state.active
                && !self.state.teardown_observed
                && self.state.source_match_phase == SourceMatchPhase::Running
                && self.state.offline_clock.is_none(),
            "source-qualified active component clock only"
        );
        let duration = self
            .binding
            .source_deadline_nanos
            .checked_sub(self.binding.source_start_nanos)
            .context("source clock overflow")?;
        let ids: Vec<i64> = self
            .binding
            .members
            .iter()
            .map(|m| i64::from(m.object_id))
            .collect();
        let reducer = c.begin_component(NAMESPACE, self.binding.map_id, &ids, 0, duration)?;
        self.reducer = Some(reducer);
        self.state.offline_clock = Some(OfflineClock {
            elapsed_nanos: 0,
            deadline_elapsed_nanos: duration,
        });
        Ok(())
    }
    pub(super) fn advance_elapsed(&mut self, elapsed: i64) -> Result<()> {
        ensure!(
            !self.state.closed && self.state.active,
            "retired component clock"
        );
        let clock = self
            .state
            .offline_clock
            .as_mut()
            .context("local clock unknown; explicit restricted offline clock required")?;
        ensure!(
            elapsed >= clock.elapsed_nanos && elapsed <= clock.deadline_elapsed_nanos,
            "offline elapsed clock only, monotonic and bounded"
        );
        clock.elapsed_nanos = elapsed;
        Ok(())
    }
    pub(super) fn close(&mut self) {
        self.reducer = None;
        self.state.offline_clock = None;
        self.state.active = false;
        self.state.closed = true;
    }
}
#[cfg(test)]
#[path = "native_pet_race_start_component_tests.rs"]
mod tests;
