//! Bounded offline5350 wire composition. Reuses the existing Native Receiver;
//! no new cipher, socket login or provider. Fixed trace delivery is a separate
//! construction policy; generic Read ingress always uses untagged observations.
use super::native_city_runtime::source_send::{Progress, Sender, Ticket, WriteOutcome, WriteView};
use super::native_city_runtime::source_stream::{
    Frame, Receiver, Snapshot as RxSnapshot, MAX_BATCH_EVENTS,
};
use super::native_pet_race_component::Cell;
use super::native_pet_race_quit_session::{
    ActiveQuit, CandidateSession, Delivery, Snapshot as SessionSnapshot, View, RETURN_SOURCE_SHA,
};
use super::native_pet_race_session::{Command, PredictedMove, ReceiptScope, ResultConfirm};
use super::native_pet_race_start_component::{
    MilestoneKind, Snapshot as StartSnapshot, StartBinding, StartComponent,
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{ErrorKind, Read};

const BINDING_SHA: &str = "3D485A02461C2267AEE06B58E8F22FD5C5A3924FB4CC2661A3CEBDD9A1D531AA";
const LOOPBACK_BINDING_SHA: &str =
    "849589D6E5337A02EC76AFB9C72D1393071250B7EDDFFA5CB19FAF74B48EC13C";
const RESULT_BINDING_SHA: &str = "8044DC25FFCC07900B9431B536CBCDF6421BFE732E0BC6745F969D1A5729F18E";
const START_BINDING_SHA: &str = "E8B1E5110C759271BE567741ADE01E90F4CED27B02529188EEDC51759C6BC847";
pub const MAX_READ_BYTES: usize = 4096;
#[derive(Deserialize)]
struct Pin {
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
struct LaneRow {
    id: String,
    session_binding: String,
    source: String,
    seed_frame_hex: String,
    wire: Pin,
    #[serde(default)]
    fixture: Option<Fixture>,
    #[serde(default)]
    start: Option<StartBinding>,
}
#[derive(Clone, Deserialize)]
struct ClientFrame {
    opcode: u8,
    frame_hex: String,
    counter_before: u64,
    counter_after: u64,
    replay_after_rx_offset: usize,
}
#[derive(Clone, Deserialize)]
struct Fixture {
    room_id: u32,
    alias_hex: String,
    initial_result: bool,
    client_frames: Vec<ClientFrame>,
}
pub(super) fn from_hex(hex: &str) -> Result<Vec<u8>> {
    ensure!(
        hex.len() % 2 == 0 && hex.is_ascii(),
        "bounded hexadecimal source bytes"
    );
    Ok((0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<std::result::Result<Vec<_>, _>>()?)
}
pub struct WireBindings {
    rows: Vec<LaneRow>,
}
pub struct Lane {
    id: String,
    session_binding: String,
    seed: Vec<u8>,
    wire: Vec<u8>,
    fixture: Option<Fixture>,
    start: Option<StartBinding>,
}
impl WireBindings {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= 16384, "wire binding bound");
        let digest = format!("{:X}", Sha256::digest(bytes));
        let expected_lanes = if digest == BINDING_SHA {
            3
        } else if digest == LOOPBACK_BINDING_SHA {
            6
        } else if digest == RESULT_BINDING_SHA {
            6
        } else if digest == START_BINDING_SHA {
            3
        } else {
            anyhow::bail!("unadopted offline wire/key binding");
        };
        #[derive(Deserialize)]
        struct Document {
            schema: String,
            lanes: Vec<LaneRow>,
            offline_only: bool,
        }
        let doc: Document = serde_json::from_slice(bytes)?;
        ensure!(
            doc.schema
                == if digest == START_BINDING_SHA {
                    "MODERN_R540_PINNED_5350_ROOM_START_COMPONENT_V1"
                } else if digest == RESULT_BINDING_SHA {
                    "MODERN_R539_PINNED_5350_RESULT_START_WIRE_V1"
                } else {
                    "MODERN_R538_PINNED_5350_OFFLINE_WIRE_BINDINGS_V1"
                }
                && doc.offline_only
                && doc.lanes.len() == expected_lanes,
            "explicit5350 offline trace lanes"
        );
        Ok(Self { rows: doc.lanes })
    }
    /// The document pins this new connection's actual fixture phase and alias.
    /// This is an offline initial condition, not admission through prior combat.
    pub fn result_start_fixture(&self, id: &str) -> Result<CandidateSession> {
        let row = self
            .rows
            .iter()
            .find(|r| r.id == id)
            .context("unknown RESULT-start lane")?;
        let f = row
            .fixture
            .as_ref()
            .context("lane has no RESULT-start fixture")?;
        CandidateSession::from_result_start_fixture(
            &row.session_binding,
            f.room_id,
            &from_hex(&f.alias_hex)?,
            f.initial_result,
        )
    }
    pub fn lane(&self, id: &str, wire: &[u8]) -> Result<Lane> {
        let row = self
            .rows
            .iter()
            .find(|r| r.id == id)
            .context("missing exact receiver/key/stream lane")?;
        ensure!(
            row.source == RETURN_SOURCE_SHA
                && wire.len() <= 1024 * 1024
                && wire.len() as u64 == row.wire.bytes
                && format!("{:X}", Sha256::digest(wire)) == row.wire.sha256,
            "wire pin or5350 source mismatch"
        );
        ensure!(
            row.seed_frame_hex.len() == 14,
            "actual emitted S_KEY frame required"
        );
        let seed = (0..14)
            .step_by(2)
            .map(|n| u8::from_str_radix(&row.seed_frame_hex[n..n + 2], 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            seed.starts_with(&[7, 0, 1]) && wire.starts_with(&seed),
            "receiver trace must start with its own emitted S_KEY"
        );
        Ok(Lane {
            id: row.id.clone(),
            session_binding: row.session_binding.clone(),
            seed,
            wire: wire.to_vec(),
            fixture: row.fixture.clone(),
            start: row.start.clone(),
        })
    }
    pub fn room_start_component_fixture(&self, id: &str) -> Result<CandidateSession> {
        let row = self
            .rows
            .iter()
            .find(|r| r.id == id)
            .context("unknown ROOM start component lane")?;
        let start = row
            .start
            .as_ref()
            .context("not a ROOM start component profile")?;
        CandidateSession::from_room_start_component_fixture(
            &row.session_binding,
            start.room_id,
            &start.local_alias()?,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Policy {
    FixedOfflineTrace,
    UntaggedReadBoundary,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReceiverState {
    pub counter: Option<u64>,
    pub header_bytes: usize,
    pub body_bytes: usize,
    pub body_expected: Option<usize>,
}
impl From<RxSnapshot> for ReceiverState {
    fn from(s: RxSnapshot) -> Self {
        Self {
            counter: s.counter,
            header_bytes: s.header_bytes,
            body_bytes: s.body_bytes,
            body_expected: s.body_expected,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Terminal {
    pub reason: String,
    pub before: ReceiverState,
    pub raw_pending_or_rejected_frame: Vec<u8>,
    pub sender_before: Option<SenderState>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SenderState {
    pub counter: Option<u64>,
    pub pending: Option<(u64, usize, usize)>,
}
#[derive(Clone)]
pub struct ResultWrite {
    ticket: Ticket,
}
#[derive(Clone)]
pub struct ReadyWrite {
    ticket: Ticket,
}
pub struct OfflineWriteView {
    inner: WriteView,
}
impl OfflineWriteView {
    pub fn bytes(&self) -> Result<&[u8]> {
        self.inner.bytes()
    }
    pub fn offset(&self) -> usize {
        self.inner.offset()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub lane_id: String,
    pub receiver_session_binding: String,
    pub policy: Policy,
    pub wire_bytes_consumed: usize,
    pub complete_frames: usize,
    pub application_bodies_dispatched: usize,
    pub receiver: Option<ReceiverState>,
    pub terminal: Option<Terminal>,
    pub session: SessionSnapshot,
    pub client_receipt_proven: bool,
    pub wire_generation_known: bool,
    pub queued_wire_bytes: usize,
    pub sender: Option<SenderState>,
    pub source_client_frames_completed: usize,
    pub source_client_wire_bytes_completed: usize,
    pub start_component: Option<StartSnapshot>,
}
#[derive(Default, Debug)]
pub struct Batch {
    pub consumed: usize,
    pub complete_frames: usize,
    pub application_bodies: Vec<Vec<u8>>,
    pub terminal: bool,
}
pub struct ReadStep {
    pub socket_or_reader_bytes: usize,
    pub batch: Batch,
    pub eof: bool,
    pub no_data: bool,
}
pub struct OfflineWireSession {
    lane: Lane,
    policy: Policy,
    session: CandidateSession,
    scope: ReceiptScope,
    receiver: Option<Receiver>,
    offset: usize,
    frames: usize,
    bodies: usize,
    pending: Vec<u8>,
    terminal: Option<Terminal>,
    queued: Vec<u8>,
    sender: Option<Sender>,
    client_index: usize,
    client_written: usize,
    pending_confirm: Option<ResultConfirm>,
    start_component: Option<StartComponent>,
}
impl OfflineWireSession {
    pub fn fixed_trace(lane: Lane, session: CandidateSession) -> Result<Self> {
        Self::begin(lane, session, Policy::FixedOfflineTrace)
    }
    pub fn untagged_boundary(lane: Lane, session: CandidateSession) -> Result<Self> {
        Self::begin(lane, session, Policy::UntaggedReadBoundary)
    }
    fn begin(lane: Lane, session: CandidateSession, policy: Policy) -> Result<Self> {
        ensure!(
            session.snapshot().candidate_source == RETURN_SOURCE_SHA
                && session.snapshot().view != View::Disconnected,
            "new5350 offline session required"
        );
        ensure!(
            session.fixture_binding()
                == if lane.fixture.is_some() || lane.start.is_some() {
                    Some(lane.session_binding.as_str())
                } else {
                    None
                },
            "fixture phase/alias must belong to this exact lane"
        );
        let sender = (lane.fixture.is_some() || lane.start.is_some()).then(Sender::default);
        let start_component = lane.start.clone().map(StartComponent::begin).transpose()?;
        ensure!(
            lane.start.is_none() || session.snapshot().view == View::Room,
            "ROOM client phase required for start components"
        );
        let scope = session.receipt_scope();
        Ok(Self {
            lane,
            policy,
            scope,
            session,
            receiver: Some(Receiver::default()),
            offset: 0,
            frames: 0,
            bodies: 0,
            pending: Vec::new(),
            terminal: None,
            queued: Vec::new(),
            sender,
            client_index: 0,
            client_written: 0,
            pending_confirm: None,
            start_component,
        })
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            lane_id: self.lane.id.clone(),
            receiver_session_binding: self.lane.session_binding.clone(),
            policy: self.policy,
            wire_bytes_consumed: self.offset,
            complete_frames: self.frames,
            application_bodies_dispatched: self.bodies,
            receiver: self.receiver.as_ref().map(|r| r.snapshot().into()),
            terminal: self.terminal.clone(),
            session: self.session.snapshot(),
            client_receipt_proven: false,
            wire_generation_known: false,
            queued_wire_bytes: self.queued.len(),
            sender: self.sender_state(),
            source_client_frames_completed: self.client_index,
            source_client_wire_bytes_completed: self.client_written,
            start_component: self.start_component.as_ref().map(StartComponent::snapshot),
        }
    }
    pub fn command(&self) -> Option<Command> {
        self.session.command()
    }
    pub fn predict(&mut self, c: &Command, to: Cell, now: i64) -> Result<PredictedMove> {
        self.session.predict(c, to, now)
    }
    pub fn prepare_active_quit(&mut self) -> Result<ActiveQuit> {
        self.session.prepare_active_quit(&self.scope)
    }
    pub fn note_offline_active_quit_written(&mut self, t: &ActiveQuit) -> Result<[u8; 1]> {
        self.session.note_offline_active_quit_written(t)
    }
    pub fn result_ack(&self) -> Result<[u8; 1]> {
        self.session.result_ack(&self.scope)
    }
    pub fn prepare_result_confirm(&mut self) -> Result<ResultConfirm> {
        self.session.prepare_result_confirm(&self.scope)
    }
    pub fn note_offline_confirm_written(&mut self, t: &ResultConfirm) -> Result<[u8; 1]> {
        ensure!(
            self.lane.fixture.is_none() && self.lane.start.is_none(),
            "keyed RESULT fixture requires complete Sender write report"
        );
        self.session.note_offline_confirm_written(t)
    }
    fn sender_state(&self) -> Option<SenderState> {
        self.sender.as_ref().map(|s| {
            let v = s.snapshot();
            SenderState {
                counter: v.counter,
                pending: v.pending,
            }
        })
    }
    /// Prepare the next pinned D9/D0 using the original Sender. Positions are
    /// chosen fixed-replay windows, not observations of Modern socket timing.
    pub fn prepare_result_wire(&mut self) -> Result<ResultWrite> {
        ensure!(self.receiver.is_some(), "retired RESULT wire scope");
        let f = self
            .lane
            .fixture
            .as_ref()
            .context("not a keyed RESULT-start fixture")?
            .client_frames
            .get(self.client_index)
            .context("no next source client command")?;
        ensure!(
            self.offset == f.replay_after_rx_offset && self.pending.is_empty(),
            "source-qualified offline command window"
        );
        ensure!(
            self.session.snapshot().view == View::Result,
            "RESULT-only keyed command"
        );
        let sender = self.sender.as_ref().context("retired source Sender")?;
        let prepared = match f.opcode {
            0xD9 => {
                self.session.result_ack(&self.scope)?;
                sender.prepare_notice_d9()?
            }
            0xD0 => sender.prepare_exit_d0()?,
            _ => anyhow::bail!("unqualified RESULT-start command"),
        };
        ensure!(
            prepared.reservation.counter_step.before == f.counter_before
                && prepared.reservation.counter_step.after == f.counter_after,
            "own-key independent outgoing counter binding"
        );
        let expected = from_hex(&f.frame_hex)?;
        let confirm = if f.opcode == 0xD0 {
            Some(self.session.prepare_result_confirm(&self.scope)?)
        } else {
            None
        };
        let reservation = self.sender.as_mut().unwrap().commit(prepared);
        let ticket = reservation.ticket;
        if self.sender.as_ref().unwrap().view(&ticket)?.bytes()? != expected {
            self.close("SourceSendBindingMismatch".into(), expected)?;
            anyhow::bail!("source Sender cipher bytes differ from this connection's command");
        }
        self.pending_confirm = confirm;
        Ok(ResultWrite { ticket })
    }
    pub fn result_write_view(&self, write: &ResultWrite) -> Result<OfflineWriteView> {
        Ok(OfflineWriteView {
            inner: self
                .sender
                .as_ref()
                .context("retired Sender")?
                .view(&write.ticket)?,
        })
    }
    /// Reports assumed offline writes only; no socket write or server ACK.
    /// D0 is committed to the lifecycle after the complete framed write.
    pub fn note_offline_result_wire_written(
        &mut self,
        write: &ResultWrite,
        offset: usize,
        bytes: usize,
    ) -> Result<bool> {
        ensure!(
            self.lane.fixture.is_some(),
            "RESULT-start write ticket required"
        );
        self.report_offline_write(&write.ticket, offset, bytes)
    }
    fn report_offline_write(
        &mut self,
        ticket: &Ticket,
        offset: usize,
        bytes: usize,
    ) -> Result<bool> {
        let frame_length = self
            .sender
            .as_ref()
            .context("retired Sender")?
            .snapshot()
            .pending
            .context("no pending source write")?
            .1;
        let progress = self.sender.as_mut().context("retired Sender")?.report(
            ticket,
            offset,
            WriteOutcome::Written(bytes),
        )?;
        match progress {
            Progress::Partial { .. } => Ok(false),
            Progress::Complete => {
                if let Some(confirm) = self.pending_confirm.take() {
                    self.session.note_offline_confirm_written(&confirm)?;
                }
                self.client_index += 1;
                self.client_written += frame_length;
                if let Some(start) = self.start_component.as_mut() {
                    start.ready_written();
                }
                Ok(true)
            }
            Progress::Retired { reason, .. } => {
                self.close(format!("SourceSendFailure: {reason:?}"), Vec::new())?;
                Ok(false)
            }
        }
    }
    pub fn prepare_source_ready_wire(&mut self) -> Result<ReadyWrite> {
        ensure!(
            self.receiver.is_some()
                && self.offset == 7
                && self.pending.is_empty()
                && self.client_index == 0,
            "own complete S_KEY and offline READY window required"
        );
        let start = self
            .lane
            .start
            .as_ref()
            .context("not a ROOM start component lane")?;
        let prepared = self
            .sender
            .as_ref()
            .context("retired Sender")?
            .prepare_participant_ready(1)?;
        ensure!(
            prepared.reservation.counter_step.before
                == self
                    .receiver
                    .as_ref()
                    .unwrap()
                    .snapshot()
                    .counter
                    .context("seed absent")?
                && prepared.reservation.counter_step.after
                    == prepared.reservation.counter_step.before + 2,
            "independent READY source counter"
        );
        let expected = from_hex(&start.ready_frame_hex)?;
        let reservation = self.sender.as_mut().unwrap().commit(prepared);
        if self
            .sender
            .as_ref()
            .unwrap()
            .view(&reservation.ticket)?
            .bytes()?
            != expected
        {
            self.close("SourceReadyBindingMismatch".into(), expected)?;
            anyhow::bail!("READY bytes mismatch own source key");
        }
        Ok(ReadyWrite {
            ticket: reservation.ticket,
        })
    }
    pub fn ready_write_view(&self, write: &ReadyWrite) -> Result<OfflineWriteView> {
        Ok(OfflineWriteView {
            inner: self
                .sender
                .as_ref()
                .context("retired Sender")?
                .view(&write.ticket)?,
        })
    }
    pub fn note_offline_ready_wire_written(
        &mut self,
        write: &ReadyWrite,
        offset: usize,
        bytes: usize,
    ) -> Result<bool> {
        ensure!(
            self.start_component.is_some(),
            "ROOM start component write required"
        );
        self.report_offline_write(&write.ticket, offset, bytes)
    }
    /// Pinned owner method observations are separate from wire-derived state.
    /// Generic/untagged Read cannot import these hypothetical replay milestones.
    pub fn replay_next_start_milestone(&mut self) -> Result<MilestoneKind> {
        ensure!(
            self.policy == Policy::FixedOfflineTrace
                && self.receiver.is_some()
                && self.pending.is_empty(),
            "active fixed component replay only"
        );
        self.start_component
            .as_mut()
            .context("not a ROOM start component")?
            .next_milestone(self.offset)
    }
    pub fn begin_restricted_start_clock(
        &mut self,
        contract: &super::native_pet_race_component::Contract,
    ) -> Result<()> {
        ensure!(
            self.policy == Policy::FixedOfflineTrace && self.receiver.is_some(),
            "restricted offline clock requires fixed active scope"
        );
        self.start_component
            .as_mut()
            .context("not a start component")?
            .begin_restricted_clock(contract)
    }
    pub fn advance_restricted_start_elapsed(&mut self, elapsed_nanos: i64) -> Result<()> {
        ensure!(
            self.policy == Policy::FixedOfflineTrace && self.receiver.is_some(),
            "restricted offline clock scope"
        );
        self.start_component
            .as_mut()
            .context("not a start component")?
            .advance_elapsed(elapsed_nanos)
    }
    pub fn feed(&mut self, input: &[u8]) -> Result<Batch> {
        ensure!(self.receiver.is_some(), "retired wire receiver");
        ensure!(
            self.queued.is_empty(),
            "consume retained reader tail before manual ingress"
        );
        ensure!(
            input.len() <= MAX_READ_BYTES,
            "bounded ingress slice required"
        );
        let mut batch = Batch::default();
        let end = self
            .offset
            .checked_add(input.len())
            .context("wire offset overflow")?;
        if end > self.lane.wire.len() || self.lane.wire[self.offset..end] != *input {
            self.close("BindingMismatch".into(), input.to_vec())?;
            batch.terminal = true;
            return Ok(batch);
        }
        while batch.consumed < input.len() && batch.complete_frames < MAX_BATCH_EVENTS {
            let (n, result) = self
                .receiver
                .as_mut()
                .unwrap()
                .next(&input[batch.consumed..]);
            self.pending
                .extend_from_slice(&input[batch.consumed..batch.consumed + n]);
            self.offset += n;
            batch.consumed += n;
            match result {
                Ok(None) => break,
                Err(reason) => {
                    let raw = std::mem::take(&mut self.pending);
                    self.close(format!("{reason:?}"), raw)?;
                    batch.terminal = true;
                    break;
                }
                Ok(Some(frame)) => {
                    self.frames += 1;
                    batch.complete_frames += 1;
                    let raw = std::mem::take(&mut self.pending);
                    match frame {
                        Frame::Seed { counter } => {
                            if raw != self.lane.seed || self.frames != 1 {
                                self.close("ForeignOrRepeatedSeed".into(), raw)?;
                                batch.terminal = true;
                                break;
                            }
                            if let Some(sender) = self.sender.as_mut() {
                                sender.seed_from_source(counter)?;
                            }
                        }
                        Frame::Body { bytes, .. } => {
                            let delivery = match self.policy {
                                Policy::FixedOfflineTrace => Delivery::HypotheticalFixedReplay,
                                Policy::UntaggedReadBoundary => Delivery::Untagged,
                            };
                            let observed = if let Some(start) = self.start_component.as_mut() {
                                if matches!(bytes.first(), Some(0xD1 | 0xE1 | 0x0B)) {
                                    start.observe(&bytes)
                                } else {
                                    self.session
                                        .observe_offline(&self.scope, &bytes, delivery)
                                        .map(|_| start.hud_observed(&bytes))
                                }
                            } else {
                                self.session.observe_offline(&self.scope, &bytes, delivery)
                            };
                            if let Err(error) = observed {
                                self.close(format!("DispatchRefused: {error}"), raw)?;
                                batch.terminal = true;
                                break;
                            }
                            self.bodies += 1;
                            batch.application_bodies.push(bytes);
                        }
                    }
                }
            }
        }
        Ok(batch)
    }
    fn close(&mut self, reason: String, raw: Vec<u8>) -> Result<()> {
        let sender_before = self.sender_state();
        let before = self
            .receiver
            .as_ref()
            .context("wire receiver already retired")?
            .snapshot()
            .into();
        if self.session.snapshot().view != View::Disconnected {
            self.session.disconnect(&self.scope)?;
        }
        self.receiver = None;
        self.sender = None;
        self.pending_confirm = None;
        if let Some(start) = self.start_component.as_mut() {
            start.close();
        }
        self.queued.clear();
        self.terminal = Some(Terminal {
            reason,
            before,
            raw_pending_or_rejected_frame: raw,
            sender_before,
        });
        Ok(())
    }
    pub fn finish_eof(&mut self) -> Result<()> {
        let end = self.receiver.as_ref().context("retired wire EOF")?.end();
        let raw = std::mem::take(&mut self.pending);
        if let Some(expected) = end.before.body_expected {
            if raw.len() >= 2 && raw.len() < expected + 2 {
                self.session.note_offline_partial_failure(
                    &self.scope,
                    &raw,
                    (expected + 2) as u16,
                )?;
            }
        }
        self.close(format!("{:?}", end.terminal), raw)
    }
    /// A generic reader cannot use the fixed hypothetical trace policy.
    /// No socket is created; EOF/error closes only this owned offline boundary.
    pub fn read_untagged_from(&mut self, reader: &mut impl Read, cap: usize) -> Result<ReadStep> {
        ensure!(
            self.policy == Policy::UntaggedReadBoundary && self.receiver.is_some(),
            "untagged Read boundary required"
        );
        ensure!(
            (1..=MAX_READ_BYTES).contains(&cap),
            "reader cap outside bound"
        );
        if !self.queued.is_empty() {
            let queued = std::mem::take(&mut self.queued);
            let batch = self.feed(&queued)?;
            if !batch.terminal {
                self.queued.extend_from_slice(&queued[batch.consumed..]);
            }
            return Ok(ReadStep {
                socket_or_reader_bytes: 0,
                batch,
                eof: false,
                no_data: false,
            });
        }
        let mut raw = [0u8; MAX_READ_BYTES];
        match reader.read(&mut raw[..cap]) {
            Ok(0) => {
                self.finish_eof()?;
                Ok(ReadStep {
                    socket_or_reader_bytes: 0,
                    batch: Batch::default(),
                    eof: true,
                    no_data: false,
                })
            }
            Ok(n) => {
                let batch = self.feed(&raw[..n])?;
                if !batch.terminal {
                    self.queued.extend_from_slice(&raw[batch.consumed..n]);
                }
                Ok(ReadStep {
                    socket_or_reader_bytes: n,
                    batch,
                    eof: false,
                    no_data: false,
                })
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                Ok(ReadStep {
                    socket_or_reader_bytes: 0,
                    batch: Batch::default(),
                    eof: false,
                    no_data: true,
                })
            }
            Err(e) => {
                let pending = std::mem::take(&mut self.pending);
                self.close(format!("IoFailure: {:?}", e.kind()), pending)?;
                Err(e.into())
            }
        }
    }
}
#[cfg(test)]
#[path = "native_pet_race_result_wire_tests.rs"]
mod result_wire_tests;
#[cfg(test)]
#[path = "native_pet_race_wire_boundary_tests.rs"]
mod tests;
