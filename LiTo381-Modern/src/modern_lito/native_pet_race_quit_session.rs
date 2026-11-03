//! Offline consumer of the isolated ClientSession00FC5E67 quit candidate.
//! The predecessor 013A session remains its own profile. A successful server
//! flush supplies replay bytes, never client delivery or wire generation.
use super::native_pet_race_component::Cell;
use super::native_pet_race_session::{
    Command, NamedTime, PredictedMove, ReceiptScope, ResultConfirm, ResultRow, Roster, Session,
    View as PriorView,
};
use anyhow::{ensure, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub const CANDIDATE_SOURCE_SHA: &str =
    "00FC5E67441BDAE88CAF40BDA3560F91B5254B694CA6A85A921C1E42D61603B2";
pub const RETURN_SOURCE_SHA: &str =
    "5350B36DD93F2EA1FC9F00639073A1C990C759B26F424BCE704481B646BC959E";
pub struct QuitContract {
    _sealed: (),
}
impl QuitContract {
    pub fn parse(order: &[u8], versions: &[u8]) -> Result<Self> {
        ensure!(
            order.len() <= 8192 && versions.len() <= 16384,
            "quit candidate contract bound"
        );
        ensure!(
            format!("{:X}", Sha256::digest(order))
                == "E954D4F58048058C20F59F2DD08A040D97DE3EF935E9E056C6AD1AE270D25A79"
                && format!("{:X}", Sha256::digest(versions))
                    == "E1692FE8F679EB671D63E16B72923668B0360C32A93AE9CE4C06DE5470639566",
            "unadopted quit candidate profile"
        );
        let v: serde_json::Value = serde_json::from_slice(versions)?;
        ensure!(
            v["isolated_candidate_source"]["sha256"] == CANDIDATE_SOURCE_SHA,
            "quit candidate source binding"
        );
        Ok(Self { _sealed: () })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum View {
    Room,
    Seeding,
    Prediction,
    Result,
    ReturnUnknown,
    Returning,
    Channel,
    Disconnected,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ReturnFlow {
    None,
    ActiveQuit,
    ResultConfirm,
}
/// An explicit hypothetical delivery assumption used only by offline replay.
/// Untagged real/late bytes do not establish which intent generated a reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    HypotheticalFixedReplay,
    Untagged,
}
#[derive(Clone)]
pub struct ActiveQuit {
    owner: Arc<()>,
    intent: Arc<()>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Notice {
    pub opcode: u8,
    pub code: u16,
    pub text_raw: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoomObservation {
    pub room_id: u32,
    pub name_raw: Vec<u8>,
    pub attributes: u8,
    pub displayed_users_wire: u16,
    pub capacity: u16,
    pub mapword_wire: u16,
    pub mode_wire: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OwnerObservation {
    pub room_id: u32,
    pub display_name_raw: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PartialTransport {
    pub raw_encrypted_prefix: Vec<u8>,
    pub declared_wire_length: u16,
    pub complete_application_body: bool,
    pub client_receipt_proven: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub candidate_source: &'static str,
    pub view: View,
    pub return_flow: ReturnFlow,
    pub origin_room_id: u32,
    pub roster: Option<Roster>,
    pub lap_values: Option<(u16, u16)>,
    pub hud_start_observed: bool,
    pub named_times: Vec<NamedTime>,
    pub winning_force: Option<i32>,
    pub result_rows: Vec<ResultRow>,
    pub notices: Vec<Notice>,
    pub quit_reply_observed: bool,
    pub current_reply_assumed: bool,
    pub current_room_list_assumed: bool,
    pub self_exit_notified: bool,
    pub room_list_row_observed: bool,
    pub commands_enabled: bool,
    pub ambiguous_receipts: bool,
    pub wire_generation_known: bool,
    pub room_rows: Vec<RoomObservation>,
    pub owner_updates: Vec<OwnerObservation>,
    pub deleted_room_ids: Vec<u32>,
    pub partial_transport: Option<PartialTransport>,
    pub actual_room_users_known: bool,
    pub complete_catalog_known: bool,
}
pub struct CandidateSession {
    source: &'static str,
    fixture_binding: Option<String>,
    base: Session,
    owner: Arc<()>,
    pending_quit: Option<Arc<()>>,
    committed: bool,
    reply: bool,
    room_row: bool,
    ambiguous: bool,
    flow: ReturnFlow,
    view: View,
    notices: Vec<Notice>,
    room_rows: Vec<RoomObservation>,
    owner_updates: Vec<OwnerObservation>,
    deleted_room_ids: Vec<u32>,
    partial_transport: Option<PartialTransport>,
}
impl CandidateSession {
    /// Reuses the source-pinned predecessor's common race/wire component,
    /// then adds only the separately pinned candidate quit lifecycle.
    pub fn from_offline_predecessor(_contract: &QuitContract, base: Session) -> Result<Self> {
        Self::from_profile_predecessor(base, CANDIDATE_SOURCE_SHA)
    }
    pub(super) fn from_return_predecessor(base: Session) -> Result<Self> {
        Self::from_profile_predecessor(base, RETURN_SOURCE_SHA)
    }
    pub(super) fn from_result_start_fixture(
        binding: &str,
        room: u32,
        alias: &[u8],
        result: bool,
    ) -> Result<Self> {
        let base = Session::result_start_fixture(room, alias, result)?;
        let mut session = Self::compose(
            base,
            RETURN_SOURCE_SHA,
            if result { View::Result } else { View::Channel },
        );
        session.fixture_binding = Some(binding.to_owned());
        Ok(session)
    }
    pub(super) fn fixture_binding(&self) -> Option<&str> {
        self.fixture_binding.as_deref()
    }
    pub(super) fn from_room_start_component_fixture(
        binding: &str,
        room: u32,
        alias: &[u8],
    ) -> Result<Self> {
        let base = Session::room_start_component_fixture(room, alias)?;
        let mut session = Self::compose(base, RETURN_SOURCE_SHA, View::Room);
        session.fixture_binding = Some(binding.to_owned());
        Ok(session)
    }
    fn from_profile_predecessor(base: Session, source: &'static str) -> Result<Self> {
        let initial = base.snapshot();
        ensure!(
            initial.view == PriorView::Seeding
                && initial.roster.is_none()
                && initial.lap_values.is_none()
                && !initial.hud_start_observed
                && base.predicted_position().is_ok(),
            "candidate requires new unseeded offline owner"
        );
        Ok(Self::compose(base, source, View::Seeding))
    }
    fn compose(base: Session, source: &'static str, view: View) -> Self {
        Self {
            source,
            fixture_binding: None,
            base,
            owner: Arc::new(()),
            pending_quit: None,
            committed: false,
            reply: false,
            room_row: false,
            ambiguous: false,
            flow: ReturnFlow::None,
            view,
            notices: Vec::new(),
            room_rows: Vec::new(),
            owner_updates: Vec::new(),
            deleted_room_ids: Vec::new(),
            partial_transport: None,
        }
    }
    pub fn receipt_scope(&self) -> ReceiptScope {
        self.base.receipt_scope()
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.base.snapshot();
        Snapshot {
            candidate_source: self.source,
            view: self.view,
            return_flow: self.flow,
            origin_room_id: s.room_id,
            roster: s.roster,
            lap_values: s.lap_values,
            hud_start_observed: s.hud_start_observed,
            named_times: s.named_times,
            winning_force: s.winning_force,
            result_rows: s.result_rows,
            notices: self.notices.clone(),
            quit_reply_observed: s.quit_reply_observed,
            current_reply_assumed: self.reply,
            current_room_list_assumed: self.room_row,
            self_exit_notified: s.self_exit_notified,
            room_list_row_observed: s.room_list_row_observed,
            commands_enabled: self.view == View::Prediction && s.commands_enabled,
            ambiguous_receipts: self.ambiguous,
            wire_generation_known: false,
            room_rows: self.room_rows.clone(),
            owner_updates: self.owner_updates.clone(),
            deleted_room_ids: self.deleted_room_ids.clone(),
            partial_transport: self.partial_transport.clone(),
            actual_room_users_known: false,
            complete_catalog_known: false,
        }
    }
    pub fn command(&self) -> Option<Command> {
        if self.view == View::Prediction {
            self.base.command()
        } else {
            None
        }
    }
    pub fn predict(&mut self, c: &Command, to: Cell, now: i64) -> Result<PredictedMove> {
        ensure!(
            self.view == View::Prediction,
            "candidate prediction retired"
        );
        self.base.predict(c, to, now)
    }
    pub fn prepare_active_quit(&mut self, scope: &ReceiptScope) -> Result<ActiveQuit> {
        self.base.scope(scope)?;
        ensure!(
            self.view == View::Prediction && self.base.command().is_some(),
            "active quit requires seeded candidate prediction"
        );
        let intent = Arc::new(());
        self.pending_quit = Some(intent.clone());
        self.base.close_commands();
        self.view = View::ReturnUnknown;
        self.flow = ReturnFlow::ActiveQuit;
        self.committed = false;
        self.reply = false;
        self.room_row = false;
        self.ambiguous = false;
        Ok(ActiveQuit {
            owner: self.owner.clone(),
            intent,
        })
    }
    /// Records an offline request assumption; prepares C_D9 only, no write.
    pub fn note_offline_active_quit_written(&mut self, token: &ActiveQuit) -> Result<[u8; 1]> {
        ensure!(
            self.view == View::ReturnUnknown
                && self.flow == ReturnFlow::ActiveQuit
                && Arc::ptr_eq(&self.owner, &token.owner)
                && self
                    .pending_quit
                    .as_ref()
                    .is_some_and(|i| Arc::ptr_eq(i, &token.intent)),
            "retired/foreign/used active quit intent"
        );
        self.pending_quit = None;
        self.committed = true;
        Ok([0xD9])
    }
    pub fn result_ack(&self, scope: &ReceiptScope) -> Result<[u8; 1]> {
        ensure!(self.view == View::Result, "candidate result ACK phase");
        self.base.result_ack(scope)
    }
    pub fn prepare_result_confirm(&mut self, scope: &ReceiptScope) -> Result<ResultConfirm> {
        ensure!(self.view == View::Result, "candidate RESULT-only confirm");
        self.base.prepare_result_confirm(scope)
    }
    pub fn note_offline_confirm_written(&mut self, token: &ResultConfirm) -> Result<[u8; 1]> {
        ensure!(
            self.view == View::Result,
            "candidate RESULT-only confirm write"
        );
        let body = self.base.note_offline_confirm_written(token)?;
        self.flow = ReturnFlow::ResultConfirm;
        self.view = View::ReturnUnknown;
        self.committed = true;
        self.reply = false;
        self.room_row = false;
        self.ambiguous = false;
        Ok(body)
    }
    /// Ordinary untagged ingress never establishes a current return reply.
    pub fn observe(&mut self, scope: &ReceiptScope, body: &[u8]) -> Result<()> {
        self.observe_offline(scope, body, Delivery::Untagged)
    }
    pub fn observe_offline(
        &mut self,
        scope: &ReceiptScope,
        body: &[u8],
        delivery: Delivery,
    ) -> Result<()> {
        self.base.scope(scope)?;
        let successor = self.source == RETURN_SOURCE_SHA;
        if successor && body.first() == Some(&0xCF) {
            ensure!(body.len() == 5, "CF exact room deletion body");
            let room = u32::from_le_bytes(body[1..5].try_into()?);
            ensure!(
                room != 0 && self.deleted_room_ids.len() < 64,
                "bounded room deletion"
            );
            self.deleted_room_ids.push(room);
            self.room_rows.retain(|r| r.room_id != room);
            return Ok(()); // deletion is not a self-exit or complete catalog ACK
        }
        if successor && body.first() == Some(&0xD5) {
            ensure!(
                body.len() >= 8 && body.len() <= 1031 && body[1] == 5,
                "bounded D5 owner update subset"
            );
            let room = u32::from_le_bytes(body[2..6].try_into()?);
            let name = &body[6..];
            ensure!(
                room != 0
                    && name.last() == Some(&0)
                    && !name[..name.len() - 1].contains(&0)
                    && self.owner_updates.len() < 64,
                "exact owner display-name update"
            );
            self.owner_updates.push(OwnerObservation {
                room_id: room,
                display_name_raw: name[..name.len() - 1].to_vec(),
            });
            return Ok(()); // a display-name owner change has no own return authority
        }
        let catalog = if successor && body.first() == Some(&0xCC) {
            let row = super::native_city_runtime::java_room_catalog::decode(body)?;
            ensure!(
                self.room_rows.len() < 64
                    || self.room_rows.iter().any(|r| r.room_id == row.room_id),
                "bounded observed room rows"
            );
            Some(RoomObservation {
                room_id: row.room_id,
                name_raw: row.name,
                attributes: row.attributes,
                displayed_users_wire: row.display_users,
                capacity: row.capacity,
                mapword_wire: row.native_mapword,
                mode_wire: row.mode,
            })
        } else {
            None
        };
        let own_exit = if successor && body.first() == Some(&0xD0) {
            self.base.local_exit_matches(body)?
        } else {
            true
        };
        if body.first().is_some_and(|op| *op == 0x72 || *op == 0x0F) {
            let notice = decode_notice(body)?;
            ensure!(self.notices.len() < 64, "candidate notices bounded");
            self.notices.push(notice);
            if self.view == View::Prediction {
                self.base.close_commands();
                self.view = View::ReturnUnknown;
            }
            return Ok(());
        }
        // Existing exact bounded decoder validates before either model mutates.
        self.base.observe(scope, body)?;
        if let Some(row) = catalog {
            if let Some(old) = self.room_rows.iter_mut().find(|r| r.room_id == row.room_id) {
                *old = row;
            } else {
                self.room_rows.push(row);
            }
        }
        let s = self.base.snapshot();
        let ordered = delivery == Delivery::HypotheticalFixedReplay;
        match body[0] {
            0xD3 => {
                self.view = View::Result;
                self.flow = ReturnFlow::None;
                self.pending_quit = None;
                self.committed = false;
                self.reply = false;
                self.room_row = false;
            }
            0xF0 if self.flow == ReturnFlow::ActiveQuit
                && self.committed
                && matches!(self.view, View::ReturnUnknown | View::Returning) =>
            {
                if ordered && !self.ambiguous {
                    self.reply = true;
                    self.view = View::Returning;
                } else {
                    self.unknown();
                }
            }
            0xD0 if self.flow == ReturnFlow::ResultConfirm
                && self.committed
                && own_exit
                && s.self_exit_notified
                && self.view != View::Channel =>
            {
                if ordered && !self.ambiguous {
                    self.reply = true;
                    self.view = View::Returning;
                } else {
                    self.unknown();
                }
            }
            0xCC if self.committed
                && self.flow != ReturnFlow::None
                && matches!(self.view, View::ReturnUnknown | View::Returning) =>
            {
                if !ordered {
                    self.unknown();
                } else if self.reply && !self.ambiguous {
                    self.room_row = true;
                    self.view = View::Channel;
                }
            }
            _ => {
                if self.view == View::Seeding && s.view == PriorView::Prediction {
                    self.view = View::Prediction;
                } else if self.view == View::Prediction && s.view != PriorView::Prediction {
                    self.base.close_commands();
                    self.view = View::ReturnUnknown;
                }
            }
        }
        Ok(())
    }
    fn unknown(&mut self) {
        self.ambiguous = true;
        self.reply = false;
        self.room_row = false;
        self.base.close_commands();
        self.view = View::ReturnUnknown;
    }
    /// Ends a fixed offline receipt window. Missing replies/zero-row lists have
    /// no completion marker or clock promise; they cannot imply CHANNEL.
    pub fn finish_offline_receipt_window(&mut self, scope: &ReceiptScope) -> Result<()> {
        self.base.scope(scope)?;
        if matches!(
            self.view,
            View::Prediction | View::ReturnUnknown | View::Returning
        ) {
            self.unknown();
        }
        Ok(())
    }
    pub fn disconnect(&mut self, scope: &ReceiptScope) -> Result<()> {
        self.base.disconnect(scope)?;
        self.view = View::Disconnected;
        self.pending_quit = None;
        self.committed = false;
        self.reply = false;
        Ok(())
    }
    /// Preserves a source-produced partial encrypted prefix as transport data.
    /// The offline failure assumption closes this owner without decrypting or
    /// padding it into an application body, or claiming client receipt.
    pub fn note_offline_partial_failure(
        &mut self,
        scope: &ReceiptScope,
        prefix: &[u8],
        declared: u16,
    ) -> Result<()> {
        self.base.scope(scope)?;
        ensure!(
            self.source == RETURN_SOURCE_SHA
                && declared >= 4
                && prefix.len() >= 2
                && prefix.len() < usize::from(declared)
                && u16::from_le_bytes(prefix[..2].try_into()?) == declared,
            "bounded incomplete encrypted frame required for new return profile"
        );
        self.disconnect(scope)?;
        self.partial_transport = Some(PartialTransport {
            raw_encrypted_prefix: prefix.to_vec(),
            declared_wire_length: declared,
            complete_application_body: false,
            client_receipt_proven: false,
        });
        Ok(())
    }
}
fn decode_notice(b: &[u8]) -> Result<Notice> {
    ensure!(b.len() <= 1030, "candidate notice bound");
    let (op, code, start) = match b {
        [0x72, 0xA6, 0, 1, ..] => (0x72, 166, 4),
        [0x0F, 9, ..] => (0x0F, 9, 2),
        _ => anyhow::bail!("only observed candidate result warning/message subset"),
    };
    let tail = &b[start..];
    ensure!(
        (2..=1025).contains(&tail.len())
            && tail.last() == Some(&0)
            && !tail[..tail.len() - 1].contains(&0),
        "candidate notice exact raw MS950 CString"
    );
    Ok(Notice {
        opcode: op,
        code,
        text_raw: tail[..tail.len() - 1].to_vec(),
    })
}
#[cfg(test)]
#[path = "native_pet_race_quit_session_tests.rs"]
mod tests;
