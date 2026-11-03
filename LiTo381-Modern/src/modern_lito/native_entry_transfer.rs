//! R506 source-owned F4 branches after verified API identity adoption.
//! 62C6A4 fixed primary127.0.0.1; inet_addr/ntoa use network octets carried
//! by an x86 LE d. 64AD74 host authority remains unclosed, separate from
//! the preserved64AD54 selected alias. Profiles never supply that authority.
use anyhow::{ensure, Context, Result};
use std::{net::{Ipv4Addr,SocketAddrV4},time::{Duration,Instant}};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerRow {pub name:Vec<u8>,pub address_word:u32,pub port_word:u16,pub at_10c:u32,pub at_110:u32,pub at_114:u32}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet { Table { values: Vec<u32> }, Request { slot: usize, address_word: u32, port_word: u16 },Rows {first:Vec<ServerRow>,second:Vec<ServerRow>} }
pub fn decode(body: &[u8]) -> Result<Packet> {
    match body.first() {
        Some(243) => {
            ensure!(body.len() >= 2, "S_F3 requires signed c count");
            let signed=body[1] as i8;
            let count=usize::try_from(signed.max(0)).unwrap();
            //64AD2C..64AD50 are exactly ten cells;64AD54 is selected-name storage.
            ensure!(count<=10, "S_F3 would overwrite Native selected-name storage");
            ensure!(body.len()==2+count*4, "S_F3 exact c+d count required");
            Ok(Packet::Table { values:body[2..].chunks_exact(4).map(|v|u32::from_le_bytes(v.try_into().unwrap())).collect() })
        },
        Some(244) => {
            ensure!(body.len()==8, "S_F4 exact cdh required (port is h, not d)");
            let slot=body[1] as i8;
            ensure!((0..10).contains(&slot), "S_F4 signed index outside ten-cell Native table");
            Ok(Packet::Request {slot:slot as usize,address_word:u32::from_le_bytes(body[2..6].try_into().unwrap()),port_word:u16::from_le_bytes(body[6..8].try_into().unwrap())})
        },
        Some(251)=> {
            ensure!(body.len()>=9&&body.len()<=super::source_connection::MAX_DECODED_BODY,"S_FB requires dd counts within Modern frame bound");
            let mut at=9;let mut lists=Vec::new();
            for i in [1,5] {
                let count=i32::from_le_bytes(body[i..i+4].try_into().unwrap()).max(0) as usize;
                // Native signed d <=0 skips that loop. The widget's five
                // visible rows are not a packet count limit. Bound allocation
                // by remaining bytes (NUL + 14 fields is the shortest row).
                ensure!(count<=(body.len()-at)/15,"S_FB count cannot fit remaining row bytes");
                let mut rows=Vec::new();
                for _ in 0..count {
                    let n=body[at..].iter().take(257).position(|b|*b==0).context("S_FB bounded NUL name missing")?;
                    let name=body[at..at+n].to_vec();at+=n+1;
                    let fields=body.get(at..at+14).context("short S_FB sdhdd row")?;at+=14;
                    rows.push(ServerRow{name,address_word:u32::from_le_bytes(fields[..4].try_into().unwrap()),port_word:u16::from_le_bytes(fields[4..6].try_into().unwrap()),at_10c:if i==1{u32::from_le_bytes(fields[6..10].try_into().unwrap())}else{0},at_110:u32::from_le_bytes(fields[if i==1{10}else{6}..if i==1{14}else{10}].try_into().unwrap()),at_114:if i==1{0}else{u32::from_le_bytes(fields[10..14].try_into().unwrap())}});
                }
                lists.push(rows);
            }
            ensure!(at==body.len(),"S_FB trailing bytes");
            Ok(Packet::Rows {second:lists.pop().unwrap(),first:lists.pop().unwrap()})
        },
        _=>anyhow::bail!("not S_F3/S_F4/S_FB transfer body"),
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Key {pub revision:u64,pub table:[u32;10],pub rows_revision:u64,pub server_rows:Vec<ServerRow>,pub primary_return:bool,pub alternate_return:bool,pub replay_sent:bool,pub selected_name:Option<Vec<u8>>,pub byte_storage:super::startup_host::bytes::Buffers,pub current_address:Option<SocketAddrV4>,pub saved_primary:Option<SocketAddrV4>,pub timer_active:bool,pub roster_count:Option<u8>}
#[derive(Clone, Default)]
pub(crate) struct State {pub(crate) key:Key,pub(crate) reconnect_after_write:Option<SocketAddrV4>,timer:Option<Timer>}
#[derive(Clone,Copy)]pub(crate) enum TimerRoute {PrimaryToFallback,FallbackToPrimary}
#[derive(Clone)]struct Timer {start:Instant,complete:bool,route:TimerRoute}
pub fn address_from_word(word:u32)->Ipv4Addr {Ipv4Addr::from(word.to_le_bytes())}
pub const FIXED_PRIMARY_WORD:u32=0x0100007F;
impl State {
    pub(crate) fn retire(&mut self)->Result<()> {let revision=self.key.revision.checked_add(1).context("transfer revision exhausted")?;self.key=Key{revision,..Key::default()};self.reconnect_after_write=None;self.timer=None;Ok(())}
    pub(crate) fn start(&mut self,name:&[u8],target:Option<SocketAddrV4>)->Result<()> {
        ensure!(name.len()<28&&!name.contains(&0),"owned selected alias outside Native64AD54 capacity");
        let revision=self.key.revision.checked_add(1).context("transfer revision exhausted")?;
        let mut bytes=self.key.byte_storage.clone();bytes.alias.copy_payload(name)?;
        if let Some(target)=target {bytes.port.write_bits(target.port());}
        //Source alias is an owned payload projection; never startup authority.
        self.key.selected_name=bytes.alias.payload().map(<[u8]>::to_vec);self.key.byte_storage=bytes;
        self.key.revision=revision;self.key.primary_return=true;
        if let Some(target)=target {self.key.replay_sent=false;self.key.current_address=Some(target);self.key.saved_primary=Some(target);}
        self.reconnect_after_write=target;self.arm(TimerRoute::PrimaryToFallback,target.is_none());Ok(())
    }
    pub(crate) fn alias(&self)->Option<&[u8]>{self.key.byte_storage.alias.payload()}
    pub(crate) fn roster_alternate(&self)->bool {self.key.primary_return||self.key.alternate_return}
    pub(crate) fn arm(&mut self,route:TimerRoute,complete:bool){self.timer=Some(Timer{start:Instant::now(),complete,route});self.key.timer_active=true;}
    pub(crate) fn character_success(&mut self)->Option<u32>{
        let state=if self.key.primary_return {self.key.primary_return=false;Some(2)}else if self.key.alternate_return {self.key.alternate_return=false;Some(0)}else{None};
        if state.is_some(){self.key.replay_sent=false;}state
    }
    pub(crate) fn lobby_success(&mut self)->Option<u32>{let mut state=None;if self.key.primary_return{self.key.primary_return=false;state=Some(1);}if self.key.alternate_return{self.key.alternate_return=false;state=Some(0);}self.complete();state}
    pub(crate) fn complete(&mut self){if let Some(t)=&mut self.timer {t.complete=true;}}
    pub(crate) fn return_one(&mut self){self.key.alternate_return=true;self.arm(TimerRoute::FallbackToPrimary,false);}
    pub(crate) fn return_two(&mut self,target:SocketAddrV4,current:SocketAddrV4){self.key.byte_storage.port.write_bits(target.port());self.key.saved_primary=Some(current);self.key.current_address=Some(target);self.key.alternate_return=true;self.key.replay_sent=false;self.reconnect_after_write=Some(target);self.arm(TimerRoute::FallbackToPrimary,false);}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {TableUpdated {count:usize},SlotUnavailable {slot:usize},RowsUpdated {rows:usize},SameAddressStarted,RedirectStarted {target:SocketAddrV4},RosterCount {count:u8},SelectedAliasReplay,IgnoredReplay}
impl super::Runtime {
    pub fn source_transfer_snapshot(&self)->Key {self.entry_transfer.key.clone()}
    pub fn source_transfer_server_control(&self,slot:usize)->Option<super::SourceTransferServerControl> {
        use super::entry_navigation::Scene;
        if slot>=5||self.page!=super::UiPage::Menu||self.entry_navigation.key.scene!=Scene::Lobby||self.entry_transfer.key.timer_active||self.source_send.snapshot().pending.is_some(){return None;}
        let row=self.entry_transfer.key.server_rows.get(slot)?;
        //475F40 guaranteed-available subset independent of unclosed8316B0/B4/B8 gate.
        //Native cmp/jg is signed. Restrict nonzero-capacity rows to proved
        //nonnegative occupancy/capacity; unknown sign domains stay unavailable.
        if row.at_110==u32::MAX||(row.at_114!=0&&(row.at_110>i32::MAX as u32||row.at_114>i32::MAX as u32||row.at_114<=row.at_110))||row.address_word!=FIXED_PRIMARY_WORD {return None;}
        Some(super::SourceTransferServerControl {epoch:self.source_scope.current_epoch().ok()?,revision:self.entry_transfer.key.rows_revision,navigation_revision:self.entry_navigation.key.revision,slot})
    }
    pub fn handle_source_transfer_server(&mut self,c:&super::SourceTransferServerControl)->Result<()> {
        self.source_scope.admit_epoch(&c.epoch)?;
        ensure!(c.revision==self.entry_transfer.key.rows_revision&&c.navigation_revision==self.entry_navigation.key.revision&&self.source_transfer_server_control(c.slot).is_some(),"retired or unqualified original server row");
        //S_FB ->4AC7D0 second vector ->475960 ->475F40 same-primary release.
        self.start_source_transfer_branch(FIXED_PRIMARY_WORD,0)?;Ok(())
    }
    fn start_source_transfer_branch(&mut self,address_word:u32,port_word:u16)->Result<Change> {
        use super::entry_navigation::{Scene,Request};
        ensure!(matches!(self.entry_navigation.key.scene,Scene::AwaitCharacterReply|Scene::Lobby)&&self.source_send.snapshot().pending.is_none()&&self.entry_navigation.queue.is_empty(),"transfer requires an idle owned selected request or lobby");
        let name=self.entry_transfer.alias().or_else(||self.source_identity.selected().map(|s|s.actor_alias())).context("source-selected64AD54 alias absent")?.to_vec();
        let target=if address_word==FIXED_PRIMARY_WORD {None}else{
            let address=address_from_word(address_word);ensure!(address.is_loopback()&&port_word!=0,"synthetic redirect must be a nonzero-port loopback IPv4");
            ensure!(self.synthetic_player_fixture_configured(),"owned synthetic IO required for F4 redirect");Some(SocketAddrV4::new(address,port_word))
        };
        let requests=if target.is_some(){vec![Request::NoticeD9,Request::Return15]}else{vec![Request::NoticeD9,Request::RosterCB,Request::Channel(0)]};
        let mut transfer=self.entry_transfer.clone();transfer.start(&name,target)?;
        self.begin_source_navigation_sequence(Scene::AwaitTransferReply,requests)?;
        self.entry_transfer=transfer;self.source_flow=self.source_flow.transfer_started();
        self.entry_navigation.key.return_state=if target.is_none(){1}else{self.entry_navigation.key.return_state};
        self.entry_navigation.key.return_mode=if target.is_none(){1}else{self.entry_navigation.key.return_mode};
        self.status="正在切換頻道，等待伺服器回覆…".into();
        Ok(if let Some(target)=target{Change::RedirectStarted {target}}else{Change::SameAddressStarted})
    }
    pub fn consume_source_transfer_event(&mut self,event:&super::SourceDecodedPacket)->Result<Change> {
        self.source_scope.admit(event)?;
        ensure!(self.page==super::UiPage::Menu,"transfer prefix unavailable");
        match decode(event.body())? {
            Packet::Table {values} => {
                let revision=self.entry_transfer.key.revision.checked_add(1).context("transfer revision exhausted")?;
                self.entry_transfer.key.table[..values.len()].copy_from_slice(&values);
                self.entry_transfer.key.revision=revision;
                Ok(Change::TableUpdated {count:values.len()})
            },
            Packet::Request {slot,address_word,port_word} => {
                if self.entry_transfer.key.table[slot]==u32::MAX {return Ok(Change::SlotUnavailable {slot});}
                self.start_source_transfer_branch(address_word,port_word)
            },
            Packet::Rows {second,..}=>{let revision=self.entry_transfer.key.rows_revision.checked_add(1).context("server row revision exhausted")?;let rows=second.len();self.entry_transfer.key.server_rows=second;self.entry_transfer.key.rows_revision=revision;Ok(Change::RowsUpdated {rows})},
        }
    }
    pub(crate) fn consume_source_transfer_roster(&mut self,event:&super::SourceDecodedPacket)->Result<Change> {
        use super::entry_navigation::{Scene,Request};self.source_scope.admit(event)?;
        ensure!(self.entry_transfer.roster_alternate(),"alternate roster path absent");
        if event.body().first()==Some(&3) {
            let super::source_identity::Packet::Start(count)=super::source_identity::decode(event.body())? else{unreachable!()};
            self.source_roster_revision=self.source_roster_revision.checked_add(1).context("roster revision exhausted")?;
            self.source_identity=super::source_identity::State::default();self.character_scene.retire()?;self.entry_transfer.key.roster_count=Some(count);return Ok(Change::RosterCount {count});
        }
        ensure!(event.body().first()==Some(&203),"alternate roster requires203");
        if self.entry_transfer.key.replay_sent{return Ok(Change::IgnoredReplay);}
        let name=self.entry_transfer.alias().context("saved source alias absent")?.to_vec();
        ensure!(self.entry_navigation.queue.len()<3,"bounded source sequence full");
        self.entry_navigation.queue.push_back(Request::Character(name));self.entry_navigation.key.scene=Scene::AwaitTransferReply;self.entry_navigation.key.write_complete=false;
        self.entry_transfer.key.replay_sent=true;self.drive_source_navigation_sequence()?;Ok(Change::SelectedAliasReplay)
    }
    pub(crate) fn poll_source_transfer_timer(&mut self)->bool {
        let Some(t)=&self.entry_transfer.timer else{return false;};
        //Modern monotonic milliseconds; source strict >3s completion precedes
        //strict >10s timeout. Native absolute QPC/u32 wrap is not claimed.
        if t.complete&&t.start.elapsed()>Duration::from_millis(3000) {self.entry_transfer.timer=None;self.entry_transfer.key.timer_active=false;return true;}
        if !t.complete&&t.start.elapsed()>Duration::from_millis(10000) {
            let target=match t.route{TimerRoute::PrimaryToFallback=>self.source_startup_host_fixture_target(),TimerRoute::FallbackToPrimary=>self.entry_transfer.key.saved_primary};
            if let Some(target)=target {
                let mut transfer=self.entry_transfer.clone();
                match t.route {TimerRoute::PrimaryToFallback=>{transfer.key.primary_return=false;transfer.key.alternate_return=true;transfer.arm(TimerRoute::FallbackToPrimary,false);},TimerRoute::FallbackToPrimary=>{transfer.key.alternate_return=false;transfer.key.primary_return=true;transfer.arm(TimerRoute::PrimaryToFallback,false);}}
                // Native timeout does not clear BE; no pending old write may
                // cross the fresh epoch. Synthetic IO owns cancellation.
                transfer.key.current_address=Some(target);self.entry_transfer=transfer;
                if self.reconnect_source_transfer_fixture(target).is_ok(){return true;}
            }
            if let Ok(epoch)=self.source_scope.current_epoch(){let _=self.disconnect_source_connection_candidate(&epoch);}
            self.entry_navigation.key.scene=super::entry_navigation::Scene::ReconnectRequired;
            self.status="頻道切換等待逾時；重連地址尚未可用。連線已結束，可重新連線。".into();return true;
        }
        false
    }
}
