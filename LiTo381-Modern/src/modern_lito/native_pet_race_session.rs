//! Offline Native PetRace session composition for ClientSession013ADEF1 /
//! GameHub98E03203. This profile is not the 485F overlay or a live admission.
//! Prediction and wire observations stay separate; commands carry only private
//! local epochs, which cannot resolve untagged delayed packets on the wire.
use super::{
    native_city_runtime::java_room_catalog,
    native_pet_race_component::{Cell, Contract, NAMESPACE},
    native_pet_race_offline::{OfflineRace, Step},
    native_pet_race_terrain::NativeTerrain,
    world_map::DIRECTIONS,
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
const OUTER_SHA: &str = "DD31F3D3DADD8AC42F26D91A8808A255AA6C407B442110E010E05C8ADAF8020B";
const MAX_REPORTS: usize = 64;
pub struct OuterContract {
    _sealed: (),
}
impl OuterContract {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() <= 32768 && format!("{:X}", Sha256::digest(bytes)) == OUTER_SHA,
            "unadopted Native outer contract"
        );
        let v: serde_json::Value = serde_json::from_slice(bytes)?;
        ensure!(
            v["schema"] == "NATIVE_PET_RACE_OUTER_LIFECYCLE_V1"
                && v["namespace"] == NAMESPACE
                && v["maps"] == serde_json::json!([566, 569, 570]),
            "Native outer source profile"
        );
        Ok(Self { _sealed: () })
    }
}
#[derive(Clone)]
pub struct ReceiptScope {
    owner: Arc<()>,
}
#[derive(Clone)]
pub struct Command {
    wire: Arc<()>,
    game: Arc<()>,
}
#[derive(Clone)]
pub struct ResultConfirm {
    wire: Arc<()>,
    intent: Arc<()>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum View {
    Room,
    Seeding,
    Prediction,
    Interrupted,
    Result,
    Returning,
    Channel,
    Disconnected,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct RosterRow {
    pub value: u8,
    pub name_raw: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Roster {
    pub rows: Vec<RosterRow>,
    pub local_index: Option<u16>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NamedTime {
    pub name_raw: Vec<u8>,
    pub elapsed_milliseconds: u32,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ResultRow {
    pub name_raw: Vec<u8>,
    pub points: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub force_bytes: [u8; 2],
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Snapshot {
    pub view: View,
    pub room_id: u32,
    pub roster: Option<Roster>,
    pub lap_values: Option<(u16, u16)>,
    pub hud_start_observed: bool,
    pub named_times: Vec<NamedTime>,
    pub winning_force: Option<i32>,
    pub result_rows: Vec<ResultRow>,
    pub quit_reply_observed: bool,
    pub self_exit_notified: bool,
    pub room_list_row_observed: bool,
    pub commands_enabled: bool,
}
pub struct Session {
    wire: Arc<()>,
    open: bool,
    game: Option<Arc<()>>,
    predicted: Option<OfflineRace>,
    local_player: i64,
    local_alias: Vec<u8>,
    confirm: Option<Arc<()>>,
    confirm_written: bool,
    snapshot: Snapshot,
}
pub struct PredictedMove {
    pub step: Step,
    pub application_body: [u8; 6],
}
enum Event {
    Roster(Roster),
    Lap(u16, u16),
    Hud,
    Time(NamedTime),
    Result(i32),
    Stats(ResultRow),
    Quit,
    Exit(u32, Vec<u8>),
    Catalog,
}
struct Bytes<'a> {
    b: &'a [u8],
    at: usize,
}
impl<'a> Bytes<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(n)
            .context("Native wire length overflow")?;
        ensure!(end <= self.b.len(), "Native wire truncated");
        let b = &self.b[self.at..end];
        self.at = end;
        Ok(b)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn name(&mut self) -> Result<Vec<u8>> {
        let n = self.b[self.at..]
            .iter()
            .position(|b| *b == 0)
            .context("Native MS950 CString absent")?;
        ensure!((1..=1024).contains(&n), "Native name bounded subset");
        let v = self.take(n)?.to_vec();
        self.take(1)?;
        Ok(v)
    }
}
fn decode(b: &[u8]) -> Result<Event> {
    ensure!(!b.is_empty() && b.len() <= 16384, "Native race body bound");
    let mut r = Bytes { b, at: 0 };
    let e = match r.u8()? {
        0xFD => match r.u8()? {
            0 => Event::Hud,
            1 => {
                let count = r.u16()?;
                let local = r.u16()?;
                ensure!(
                    count <= 10 && (local == 0xFFFF || local < count),
                    "bounded PetRace roster/local index"
                );
                let mut rows = Vec::new();
                for _ in 0..count {
                    rows.push(RosterRow {
                        value: r.u8()?,
                        name_raw: r.name()?,
                    });
                }
                Event::Roster(Roster {
                    rows,
                    local_index: (local != 0xFFFF).then_some(local),
                })
            }
            2 => Event::Lap(r.u16()?, r.u16()?),
            3 => Event::Time(NamedTime {
                name_raw: r.name()?,
                elapsed_milliseconds: r.u32()?,
            }),
            _ => anyhow::bail!("unqualified Native FD subtype"),
        },
        0xD3 => Event::Result(r.u32()? as i32),
        0xDB => Event::Stats(ResultRow {
            name_raw: r.name()?,
            points: r.u32()?,
            kills: r.u32()?,
            deaths: r.u32()?,
            assists: r.u32()?,
            force_bytes: [r.u8()?, r.u8()?],
        }),
        0xF0 => {
            ensure!(r.u8()? == 0, "Native source F0 compatibility zero");
            Event::Quit
        }
        0xD0 => Event::Exit(r.u32()?, r.name()?),
        0xCC => {
            java_room_catalog::decode(b)?;
            r.at = b.len();
            Event::Catalog
        }
        _ => anyhow::bail!("unqualified Native race wire event"),
    };
    ensure!(r.at == b.len(), "Native race trailing body bytes");
    Ok(e)
}
impl Session {
    pub fn begin_offline(
        _outer: &OuterContract,
        c: &Contract,
        terrain: Arc<NativeTerrain>,
        seats: &[(i64, Cell)],
        local_player: i64,
        room_id: u32,
        local_alias: &[u8],
        start: i64,
        deadline: i64,
    ) -> Result<Self> {
        ensure!(
            room_id != 0
                && (1..=1024).contains(&local_alias.len())
                && !local_alias.contains(&0)
                && seats.iter().any(|(id, _)| *id == local_player),
            "offline Native session identity"
        );
        let predicted = OfflineRace::begin(c, terrain, seats, start, deadline)?;
        Ok(Self {
            wire: Arc::new(()),
            open: true,
            game: None,
            predicted: Some(predicted),
            local_player,
            local_alias: local_alias.to_vec(),
            confirm: None,
            confirm_written: false,
            snapshot: Snapshot {
                view: View::Seeding,
                room_id,
                roster: None,
                lap_values: None,
                hud_start_observed: false,
                named_times: Vec::new(),
                winning_force: None,
                result_rows: Vec::new(),
                quit_reply_observed: false,
                self_exit_notified: false,
                room_list_row_observed: false,
                commands_enabled: false,
            },
        })
    }
    pub fn receipt_scope(&self) -> ReceiptScope {
        ReceiptScope {
            owner: self.wire.clone(),
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.clone()
    }
    pub fn predicted_position(&self) -> Result<Cell> {
        self.predicted
            .as_ref()
            .context("no fixture race prediction")?
            .position(self.local_player)
    }
    /// Only the separately pinned RESULT-start wire fixture constructor calls
    /// this crate-local entry. No terrain, prior combat, roster or D3 is made up.
    pub(super) fn result_start_fixture(room_id: u32, alias: &[u8], result: bool) -> Result<Self> {
        ensure!(
            if result {
                room_id != 0 && (1..=1024).contains(&alias.len()) && !alias.contains(&0)
            } else {
                room_id == 0 && alias.is_empty()
            },
            "exact RESULT peer or CHANNEL watcher fixture context"
        );
        Ok(Self {
            wire: Arc::new(()),
            open: true,
            game: None,
            predicted: None,
            local_player: 0,
            local_alias: alias.to_vec(),
            confirm: None,
            confirm_written: false,
            snapshot: Snapshot {
                view: if result { View::Result } else { View::Channel },
                room_id,
                roster: None,
                lap_values: None,
                hud_start_observed: false,
                named_times: Vec::new(),
                winning_force: None,
                result_rows: Vec::new(),
                quit_reply_observed: false,
                self_exit_notified: false,
                room_list_row_observed: false,
                commands_enabled: false,
            },
        })
    }
    pub(super) fn room_start_component_fixture(room_id: u32, alias: &[u8]) -> Result<Self> {
        let mut session = Self::result_start_fixture(room_id, alias, true)?;
        session.snapshot.view = View::Room;
        Ok(session)
    }
    pub fn command(&self) -> Option<Command> {
        self.game.as_ref().map(|game| Command {
            wire: self.wire.clone(),
            game: game.clone(),
        })
    }
    pub(super) fn scope(&self, s: &ReceiptScope) -> Result<()> {
        ensure!(
            self.open && Arc::ptr_eq(&s.owner, &self.wire),
            "retired/foreign Native receipt scope"
        );
        Ok(())
    }
    pub(super) fn local_exit_matches(&self, body: &[u8]) -> Result<bool> {
        match decode(body)? {
            Event::Exit(room, alias) => {
                Ok(room == self.snapshot.room_id && alias == self.local_alias)
            }
            _ => Ok(false),
        }
    }
    pub(super) fn close_commands(&mut self) {
        self.game = None;
        self.snapshot.commands_enabled = false;
        if let Some(predicted) = self.predicted.as_mut() {
            predicted.retire();
        }
    }
    fn seed(&mut self) {
        if self.snapshot.view == View::Seeding
            && self.predicted_position().is_ok()
            && self
                .snapshot
                .roster
                .as_ref()
                .is_some_and(|r| r.local_index.is_some())
            && self
                .snapshot
                .lap_values
                .is_some_and(|(max, current)| max == 4 && (1..=4).contains(&current))
            && self.snapshot.hud_start_observed
        {
            self.game = Some(Arc::new(()));
            self.snapshot.commands_enabled = true;
            self.snapshot.view = View::Prediction;
        }
    }
    /// Offline prediction plus existing plain C_MOVE body; no framing, cipher,
    /// write, server position acknowledgement or authoritative gate transition.
    pub fn predict(&mut self, c: &Command, to: Cell, now: i64) -> Result<PredictedMove> {
        ensure!(
            self.open
                && Arc::ptr_eq(&c.wire, &self.wire)
                && self.game.as_ref().is_some_and(|g| Arc::ptr_eq(g, &c.game)),
            "retired/foreign Native command epoch"
        );
        let from = self.predicted_position()?;
        let dx = i64::from(to.x) - i64::from(from.x);
        let dy = i64::from(to.y) - i64::from(from.y);
        let heading = DIRECTIONS
            .iter()
            .position(|&(x, y)| i64::from(x) == dx && i64::from(y) == dy)
            .context("Native C_MOVE requires one heading edge")?;
        let x = u16::try_from(from.x)?;
        let y = u16::try_from(from.y)?;
        let xb = x.to_le_bytes();
        let yb = y.to_le_bytes();
        let step = self
            .predicted
            .as_mut()
            .context("no fixture race prediction")?
            .step(self.local_player, to, now)?;
        Ok(PredictedMove {
            step,
            application_body: [0x0A, xb[0], xb[1], yb[0], yb[1], heading as u8],
        })
    }
    pub fn observe(&mut self, s: &ReceiptScope, body: &[u8]) -> Result<()> {
        self.scope(s)?;
        let e = decode(body)?;
        match e {
            Event::Roster(r) => {
                ensure!(
                    matches!(
                        self.snapshot.view,
                        View::Room | View::Seeding | View::Prediction
                    ),
                    "late roster requires a new local race owner"
                );
                if self.snapshot.view == View::Prediction {
                    self.close_commands();
                    self.snapshot.view = View::Seeding;
                }
                self.snapshot.roster = Some(r);
                self.snapshot.lap_values = None;
                self.snapshot.hud_start_observed = false;
            }
            Event::Lap(max, current) => {
                ensure!(
                    matches!(
                        self.snapshot.view,
                        View::Room | View::Seeding | View::Prediction
                    ),
                    "late lap display"
                );
                self.snapshot.lap_values = Some((max, current));
                if self.snapshot.view == View::Prediction
                    && !(max == 4 && (1..=4).contains(&current))
                {
                    self.close_commands();
                    self.snapshot.view = View::Seeding;
                }
            }
            Event::Hud => {
                ensure!(
                    matches!(
                        self.snapshot.view,
                        View::Room | View::Seeding | View::Prediction
                    ),
                    "late HUD start"
                );
                self.snapshot.hud_start_observed = true;
            }
            Event::Time(t) => {
                ensure!(
                    matches!(self.snapshot.view, View::Seeding | View::Prediction)
                        && self.snapshot.named_times.len() < MAX_REPORTS,
                    "late/overbound named time"
                );
                self.snapshot.named_times.push(t);
            }
            Event::Result(force) => {
                ensure!(
                    matches!(
                        self.snapshot.view,
                        View::Seeding | View::Prediction | View::Interrupted | View::Result
                    ),
                    "late result display"
                );
                self.close_commands();
                self.snapshot.winning_force = Some(force);
                self.snapshot.view = View::Result;
            }
            Event::Stats(r) => {
                ensure!(
                    self.snapshot.view == View::Result
                        && self.snapshot.result_rows.len() < MAX_REPORTS,
                    "result row phase/bound"
                );
                self.snapshot.result_rows.push(r);
            }
            Event::Quit => {
                self.snapshot.quit_reply_observed = true;
            }
            Event::Exit(room, alias) => {
                if room == self.snapshot.room_id
                    && alias == self.local_alias
                    && self.snapshot.view != View::Channel
                {
                    self.close_commands();
                    if self.confirm_written
                        && matches!(self.snapshot.view, View::Result | View::Returning)
                    {
                        self.snapshot.self_exit_notified = true;
                        self.snapshot.view = View::Returning;
                        self.snapshot.room_list_row_observed = false;
                    } else if matches!(self.snapshot.view, View::Seeding | View::Prediction) {
                        self.snapshot.view = View::Interrupted;
                    }
                }
            }
            Event::Catalog => {
                self.snapshot.room_list_row_observed = true;
                if self.snapshot.self_exit_notified && self.snapshot.view == View::Returning {
                    self.snapshot.view = View::Channel;
                }
            }
        }
        self.seed();
        Ok(())
    }
    /// Result ACK retains room membership. This is a prepared offline body only.
    pub fn result_ack(&self, s: &ReceiptScope) -> Result<[u8; 1]> {
        self.scope(s)?;
        ensure!(
            self.snapshot.view == View::Result,
            "Native result ACK phase"
        );
        Ok([0xD9])
    }
    /// Only RESULT confirm is offered. Active C_D0 producer remains unqualified.
    pub fn prepare_result_confirm(&mut self, s: &ReceiptScope) -> Result<ResultConfirm> {
        self.scope(s)?;
        ensure!(
            self.snapshot.view == View::Result,
            "Native result confirm phase"
        );
        let intent = Arc::new(());
        self.confirm = Some(intent.clone());
        self.confirm_written = false;
        Ok(ResultConfirm {
            wire: self.wire.clone(),
            intent,
        })
    }
    pub fn note_offline_confirm_written(&mut self, c: &ResultConfirm) -> Result<[u8; 1]> {
        ensure!(
            self.open
                && self.snapshot.view == View::Result
                && Arc::ptr_eq(&c.wire, &self.wire)
                && self
                    .confirm
                    .as_ref()
                    .is_some_and(|i| Arc::ptr_eq(i, &c.intent)),
            "retired/foreign result confirm"
        );
        self.confirm = None;
        self.confirm_written = true;
        Ok([0xD0])
    }
    pub fn disconnect(&mut self, s: &ReceiptScope) -> Result<()> {
        self.scope(s)?;
        self.close_commands();
        self.open = false;
        self.snapshot.view = View::Disconnected;
        self.confirm = None;
        Ok(())
    }
}
#[cfg(test)]
#[path = "native_pet_race_session_tests.rs"]
mod tests;
