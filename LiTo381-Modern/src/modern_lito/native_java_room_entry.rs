//! Explicit AD368 ordinary room-entry adapter. These are receipt scenes,
//! not Native UI/owner readiness or numeric phase/dispatch qualification.
use anyhow::{ensure, Context, Result};
use super::{Runtime, SourceEpoch, SourceDecodedPacket, entry_navigation::{Scene, Request}};
pub const CONTRACT_SHA: &str = "11D9483483A6863BF680927748BB7592DE5ADCB7D8AA73C77E5AD3863F04D5C6";
#[derive(Clone)] pub struct Adapter;
impl Adapter {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let v: serde_json::Value = serde_json::from_slice(bytes)?;
        ensure!(v["kind"] == "modern_synthetic_AD368_room_entry_v1"
            && v["producer_sha256"] == super::login_bootstrap::PRODUCER_SHA
            && v["contract_sha256"] == CONTRACT_SHA, "explicit pinned AD368 room adapter required");
        Ok(Self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Options {
    pub values: [u8; 4], pub selections: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Settings {
    /// Caller supplies already encoded MS950 bytes. No source/physical identity.
    pub name: Vec<u8>, pub attributes: u8, pub password: Vec<u8>,
    pub capacity: u16, pub options: Option<Options>,
}
#[derive(Clone, Debug, PartialEq, Eq)] pub enum Intent {
    Create(Settings), Join { room_id: u32, password: Vec<u8> }, RefreshList,
}
fn string_bytes(bytes: &[u8], min: usize, max: usize) -> Result<()> {
    ensure!((min..=max).contains(&bytes.len()) && !bytes.contains(&0)
        && !bytes.iter().any(|b| *b < 32 || *b == 127), "room CString outside adapter byte policy");
    Ok(())
}
fn z(body: &mut Vec<u8>, bytes: &[u8]) { body.extend_from_slice(bytes); body.push(0); }
/// Strict ordinary adapter request envelope. Admission remains server owned.
/// Server MS950 roundtrip validation may still reject caller-supplied raw text.
pub fn encode(intent: &Intent) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    match intent {
        Intent::Create(s) => {
            string_bytes(&s.name, 1, 89)?; string_bytes(&s.password, 0, 4)?;
            ensure!(s.attributes & !0x3c == 0 && (s.attributes & 4 != 0) == !s.password.is_empty(), "room attributes/password mismatch");
            // AD368 admission narrows the older parser's 2..200 envelope.
            ensure!((2..=100).contains(&s.capacity), "ordinary AD368 room capacity outside 2..100");
            body.push(204); z(&mut body, &s.name); body.push(s.attributes);
            z(&mut body, &s.password); body.extend(s.capacity.to_le_bytes());
            if let Some(o) = &s.options {
                ensure!(o.selections.len() <= 8 && !o.selections.contains(&0), "room selection bytes outside envelope");
                ensure!((o.values == [0; 4] && o.selections.is_empty())
                    || (o.values[0] <= 7 && o.values[1] <= 2 && (1..=127).contains(&o.values[2]) && o.values[3] <= 31), "room options outside producer domain");
                body.extend(o.values); body.push(o.selections.len() as u8); body.extend(&o.selections);
            }
        },
        Intent::Join { room_id, password } => {
            ensure!(*room_id != 0, "explicit nonzero room ID required"); string_bytes(password, 0, 4)?;
            body.push(207); body.extend(room_id.to_le_bytes()); z(&mut body, password);
        },
        Intent::RefreshList => body.push(206),
    }
    Ok(body)
}
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Room {
    pub id: u32, pub name: Vec<u8>, pub attributes: u8, pub password: Vec<u8>,
    /// max(2,combatants): neither a wire row count nor roster terminator.
    pub users_summary: u16, pub capacity: u16, pub mode: u8, pub map_id: u16,
    pub summary: [u8; 5], pub owner_alias: Vec<u8>, pub options: [u8; 4], pub selections: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)] pub enum Reply { Accepted(Room), Refused(u8) }
struct Cursor<'a> { body: &'a [u8], at: usize }
impl Cursor<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> { let end=self.at.checked_add(n).context("room offset overflow")?; let v=self.body.get(self.at..end).context("short room result field")?; self.at=end; Ok(v) }
    fn c(&mut self) -> Result<u8> { Ok(self.take(1)?[0]) }
    fn h(&mut self) -> Result<u16> { Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap())) }
    fn d(&mut self) -> Result<u32> { Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap())) }
    fn z(&mut self, cap: usize) -> Result<Vec<u8>> { let n=self.body[self.at..].iter().take(cap).position(|v| *v==0).context("bounded room CString missing")?; Ok(self.take(n+1)?[..n].to_vec()) }
}
pub fn decode(body: &[u8]) -> Result<Reply> {
    ensure!(body.first()==Some(&206) && (2..=155).contains(&body.len()), "S_CE outside bounded adapter envelope");
    if body[1]!=0 { ensure!(body.len()==2, "S_CE refusal requires exact two bytes"); return Ok(Reply::Refused(body[1])); }
    let mut c=Cursor {body, at:2};
    let id=c.d()?; let name=c.z(90)?; let attributes=c.c()?; let password=c.z(5)?;
    let users_summary=c.h()?; let capacity=c.h()?; let mode=c.c()?; let map_id=c.h()?;
    let summary=c.take(5)?.try_into().unwrap(); let owner_alias=c.z(28)?;
    let options=c.take(4)?.try_into().unwrap(); let count=usize::from(c.c()?);
    ensure!(count<=8, "S_CE selection count exceeds adapter bound");
    let selections=c.take(count)?.to_vec(); ensure!(c.at==body.len(), "S_CE trailing bytes");
    // Strict size/end rules are Modern policy, not original malformed handling.
    ensure!(id!=0 && !name.is_empty(), "S_CE accepted room identity absent");
    Ok(Reply::Accepted(Room {id,name,attributes,password,users_summary,capacity,mode,map_id,summary,owner_alias,options,selections}))
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)] pub enum Phase {
    #[default] Idle, EntryWrite, AwaitReply, RoomReceipts, Refused, RefreshWrite, AwaitRoomList,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)] pub struct Key {
    pub revision: u64, pub phase: Phase, pub request_opcode: Option<u8>,
    pub pending_serial: Option<u64>, pub room: Option<Room>, pub refusal: Option<u8>,
}
#[derive(Clone)] pub struct Control { epoch: SourceEpoch, navigation: u64, roster: u64, room_revision: u64, intent: Intent }
#[derive(Default)] pub(crate) struct State { pub(crate) adapter: Option<Adapter>, pub(crate) key: Key, intent: Option<Intent> }
impl State {
    pub(crate) fn retire(&mut self) -> Result<()> { let revision=self.key.revision.checked_add(1).context("room revision exhausted")?; self.key=Key{revision,..Key::default()}; self.intent=None; Ok(()) }
    pub(crate) fn written(&mut self, serial: u64) {
        if self.key.pending_serial==Some(serial) { self.key.pending_serial=None; self.key.phase=match self.key.phase {Phase::EntryWrite=>Phase::AwaitReply,Phase::RefreshWrite=>Phase::AwaitRoomList,p=>p}; }
    }
}
#[derive(Clone, Debug)] pub enum Change { Accepted { room_id: u32 }, Refused { result: u8 } }
impl Runtime {
    pub fn configure_java_room_entry_adapter(&mut self, adapter: Adapter) -> Result<()> {
        ensure!(self.java_room.key.phase==Phase::Idle && self.source_send.snapshot().pending.is_none(), "room adapter change during request");
        self.java_room.retire()?; self.java_room.adapter=Some(adapter); Ok(())
    }
    pub fn java_room_entry_snapshot(&self) -> Key { self.java_room.key.clone() }
    pub fn last_java_room_entry_checkpoint(&self) -> Option<Key> { self.last_room_checkpoint.clone() }
    pub fn java_room_entry_control(&self, intent: Intent) -> Option<Control> {
        if self.bootstrap_adapter.is_none() || self.java_room.adapter.is_none() || self.page!=super::UiPage::Menu
            || self.java_character.key.stage!=super::java_character_flow::Stage::ChannelReceipts
            || self.java_character.key.channel_info.is_none() || self.java_return.key.phase!=super::java_return::Phase::Idle
            || self.entry_transfer.key.timer_active || self.source_send.snapshot().pending.is_some() || !self.entry_navigation.queue.is_empty()
            || encode(&intent).is_err() { return None; }
        let allowed=match intent {Intent::RefreshList=>self.java_room.key.phase==Phase::Refused && self.entry_navigation.key.scene==Scene::RoomRefused,
            _=>self.java_room.key.phase==Phase::Idle && self.entry_navigation.key.scene==Scene::Lobby};
        if !allowed {return None;}
        Some(Control {epoch:self.source_scope.current_epoch().ok()?, navigation:self.entry_navigation.key.revision,
            roster:self.source_roster_revision, room_revision:self.java_room.key.revision, intent})
    }
    pub fn handle_java_room_entry_control(&mut self, c: &Control) -> Result<()> {
        self.source_scope.admit_epoch(&c.epoch)?;
        ensure!(c.navigation==self.entry_navigation.key.revision && c.roster==self.source_roster_revision
            && c.room_revision==self.java_room.key.revision && self.java_room_entry_control(c.intent.clone()).is_some(), "retired or unavailable room control");
        let body=encode(&c.intent)?; let revision=self.java_room.key.revision.checked_add(1).context("room revision exhausted")?;
        let refresh=matches!(c.intent,Intent::RefreshList);
        self.begin_source_navigation_sequence(if refresh {Scene::AwaitRoomList} else {Scene::AwaitRoomReply},vec![Request::Room(body)])?;
        self.java_room.key=Key {revision,phase:if refresh {Phase::RefreshWrite} else {Phase::EntryWrite},
            request_opcode:Some(if refresh {206} else if matches!(c.intent,Intent::Create(_)) {204} else {207}),
            pending_serial:self.entry_navigation.key.pending_write,..Key::default()};
        self.java_room.intent=Some(c.intent.clone()); self.status=if refresh {"正在重新取得房間清單…"} else {"正在送出房間要求…"}.into(); Ok(())
    }
    pub fn consume_java_room_entry_candidate(&mut self, event: &SourceDecodedPacket) -> Result<Change> {
        self.source_scope.admit(event)?;
        ensure!(self.bootstrap_adapter.is_some() && self.java_room.adapter.is_some(), "explicit AD368 room adapter required");
        ensure!(self.java_room.key.phase==Phase::AwaitReply && self.entry_navigation.key.scene==Scene::AwaitRoomReply
            && self.entry_navigation.key.write_complete && self.source_send.snapshot().pending.is_none()
            && self.entry_navigation.queue.is_empty(), "S_CE outside completed owned room intent");
        let reply=decode(event.body())?;
        if let Reply::Accepted(room)=&reply {
            match self.java_room.intent.as_ref().context("room intent absent")? {
                Intent::Join {room_id,..}=>ensure!(*room_id==room.id, "S_CE joined room ID differs from explicit intent"),
                // Raw create name is an expectation, not character identity.
                Intent::Create(s)=>ensure!(s.name==room.name, "S_CE room name differs from create intent"),
                Intent::RefreshList=>anyhow::bail!("S_CE cannot complete room-list request"),
            }
        }
        let revision=self.java_room.key.revision.checked_add(1).context("room revision exhausted")?;
        let navigation=self.entry_navigation.key.revision.checked_add(1).context("navigation revision exhausted")?;
        let participant_book=if let Reply::Accepted(room)=&reply{self.java_participants.prepare_joined(&self.source_scope.current_epoch()?,room.id,revision,self.java_participants.departure_adapter.as_ref().and_then(|a|a.qualify(self)),self.qualified_singleton_self_alias())?}else{None};
        let settings_book=if let Reply::Accepted(room)=&reply{self.prepare_settings_joined(room.id,revision,navigation)?}else{None};
        let self_state=if let Reply::Accepted(room)=&reply{self.prepare_self_joined(room.id,revision,navigation)}else{self.java_self.clone()};
        // Only the Modern receipt scene changes. Native mode/phase effects
        // continue through their existing guard until separately qualified.
        self.entry_navigation.key.revision=navigation; self.entry_navigation.key.pending_write=None;
        self.entry_navigation.key.write_complete=false; self.java_room.key.revision=revision;
        self.java_room.intent=None; self.java_participants.book=participant_book;self.java_settings.book=settings_book;self.java_self=self_state;
        match reply {
            Reply::Accepted(room)=>{let room_id=room.id;self.java_room.key.room=Some(room);self.java_room.key.phase=Phase::RoomReceipts;
                self.entry_navigation.key.scene=Scene::RoomReceipts;self.status="已收到進房確認，後續房間名單仍待接線".into();Ok(Change::Accepted {room_id})},
            Reply::Refused(result)=>{self.java_room.key.refusal=Some(result);self.java_room.key.phase=Phase::Refused;
                self.entry_navigation.key.scene=Scene::RoomRefused;self.status="進房未成功，可重新取得房間清單".into();Ok(Change::Refused {result})},
        }
    }
}
