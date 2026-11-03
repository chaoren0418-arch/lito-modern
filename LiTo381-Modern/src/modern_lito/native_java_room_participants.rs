//! AD368 incremental room observations. Raw aliases never authorize physical,
//! selected-source, self, Native table, owner, readiness command or UI objects.
use anyhow::{ensure, Context, Result};
use super::{Runtime, SourceEpoch, SourceDecodedPacket};
pub const CONTRACT_SHA: &str="EF3FF32941FF07DD0276FCB885A67560ECC47CBE0B660AB778EC5F60CA46C65F";
pub const MAX_ROWS: usize=256; // Modern observation budget, not room capacity.
pub const MAX_RECEIPTS: usize=1024; // Preserve each wire arrival within a separate budget.
#[derive(Clone)] pub struct Adapter;
impl Adapter {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let v: serde_json::Value=serde_json::from_slice(bytes)?;
        ensure!(v["kind"]=="modern_synthetic_AD368_room_participants_v1"
            && v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA
            && v["contract_sha256"]==CONTRACT_SHA, "explicit pinned AD368 participant adapter required");
        Ok(Self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum MemberState { NotReady, Ready, ShopList }
impl MemberState {
    fn decode(value: u8) -> Result<Self> { match value {0=>Ok(Self::NotReady),1=>Ok(Self::Ready),2=>Ok(Self::ShopList),_=>anyhow::bail!("AD368 participant state outside 0/1/2")} }
    pub fn wire(self) -> u8 {match self {Self::NotReady=>0,Self::Ready=>1,Self::ShopList=>2}}
}
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Participant {
    pub room_id: u32, pub display_alias: Vec<u8>, pub force: u8, pub character_class: u8, pub sex: u8,
    pub decided_matches: u32, pub wins: u32, pub losses: u32, pub initial_room_state: u8,
    pub affiliation: Vec<u8>, pub reserved: u8, pub ready_label: Vec<u8>, pub disconnects: u32,
    pub channel_selector_raw: u8, pub initial_preview_state: u8, pub channel_rank: u32, pub grade_selector: u8,
    pub legacy_certificates: [u8;12], pub victory_badge_editions: [u8;5], pub lito_master: u32,
    pub raw_body: Vec<u8>,
}
impl Participant { pub fn channel_selector_signed(&self) -> i8 {self.channel_selector_raw as i8} }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct StateReceipt {pub display_alias: Vec<u8>, pub state: MemberState, pub raw_body: Vec<u8>}
#[derive(Clone, Debug, PartialEq, Eq)] pub enum Packet {Participant(Participant), State(StateReceipt), Departure(super::java_room_departure::Receipt),Owner(super::java_room_tail::OwnerReceipt)}
struct Cursor<'a> {body: &'a [u8], at: usize}
impl Cursor<'_> {
    fn take(&mut self,n:usize)->Result<&[u8]>{let end=self.at.checked_add(n).context("participant offset overflow")?;let v=self.body.get(self.at..end).context("short participant field")?;self.at=end;Ok(v)}
    fn c(&mut self)->Result<u8>{Ok(self.take(1)?[0])}
    fn d(&mut self)->Result<u32>{Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))}
    fn z(&mut self,cap:usize)->Result<Vec<u8>>{let n=self.body[self.at..].iter().take(cap).position(|b|*b==0).context("bounded participant CString missing")?;Ok(self.take(n+1)?[..n].to_vec())}
}
/// Exact AD368 schema/domain policy; generic Native version decoding remains
/// separate. Strings are raw MS950 bytes; no alias parsing or case folding.
pub fn decode(body:&[u8])->Result<Packet>{
    if body.first()==Some(&213){return Ok(Packet::Owner(super::java_room_tail::decode_owner(body)?));}
    if body.first()==Some(&208){return Ok(Packet::Departure(super::java_room_departure::decode(body)?));}
    ensure!(!body.is_empty(),"participant opcode absent");let mut c=Cursor{body,at:1};
    if body[0]==209 {
        ensure!(body.len()<=30,"AD368 S_D1 exceeds bounded alias envelope");
        let display_alias=c.z(28)?;ensure!(!display_alias.is_empty(),"S_D1 alias absent");
        let state=MemberState::decode(c.c()?)?;ensure!(c.at==body.len(),"S_D1 trailing bytes");
        return Ok(Packet::State(StateReceipt{display_alias,state,raw_body:body.to_vec()}));
    }
    ensure!(body[0]==205 && body.len()<=105,"AD368 S_CD outside full schema envelope");
    let room_id=c.d()?;let display_alias=c.z(28)?;let force=c.c()?;let character_class=c.c()?;let sex=c.c()?;
    let decided_matches=c.d()?;let wins=c.d()?;let losses=c.d()?;let initial_room_state=c.c()?;
    let affiliation=c.z(17)?;let reserved=c.c()?;let ready_label=c.z(17)?;let disconnects=c.d()?;
    let channel_selector_raw=c.c()?;let initial_preview_state=c.c()?;let channel_rank=c.d()?;let grade_selector=c.c()?;
    let legacy_certificates=c.take(12)?.try_into().unwrap();let victory_badge_editions:[u8;5]=c.take(5)?.try_into().unwrap();let lito_master=c.d()?;
    ensure!(c.at==body.len(),"S_CD trailing bytes");
    ensure!(room_id!=0 && !display_alias.is_empty() && force<=1 && character_class<=4 && sex<=1,"AD368 S_CD identity/display domain mismatch");
    ensure!(initial_room_state<=1 && initial_preview_state<=1 && reserved==0 && ready_label==b"Ready","AD368 S_CD serialized state/label domain mismatch");
    ensure!((channel_selector_raw<=4 || channel_selector_raw==255) && grade_selector<=14
        && victory_badge_editions.iter().all(|v|*v<=127) && lito_master<=1,"AD368 S_CD selector/award domain mismatch");
    Ok(Packet::Participant(Participant{room_id,display_alias,force,character_class,sex,decided_matches,wins,losses,initial_room_state,affiliation,reserved,ready_label,disconnects,channel_selector_raw,initial_preview_state,channel_rank,grade_selector,legacy_certificates,victory_badge_editions,lito_master,raw_body:body.to_vec()}))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Origin {ExplicitObservation, JoinedCeReceipt}
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Completeness {PossiblyIncomplete}
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum StateSource {CdArrival, D1Arrival}
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Row {
    pub participant: Participant, pub state: MemberState, pub state_source: StateSource,
    pub last_arrival: u64, pub last_cd_arrival: u64, pub last_state_receipt: Option<StateReceipt>,
}
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Snapshot {
    pub epoch: u64, pub room_id: u32, pub origin: Origin, pub ce_revision: Option<u64>,
    pub revision: u64, pub completeness: Completeness, pub rows: Vec<Row>,
    pub ignored_unknown_states: u64, pub last_unknown_state: Option<StateReceipt>,
    pub ignored_duplicate_rows: u64, pub ignored_ambiguous_states: u64,
    pub last_ambiguous_state: Option<StateReceipt>, pub receipts: Vec<Receipt>,
    pub owner_display:Option<super::java_room_tail::OwnerReceipt>, pub owner_link:Option<super::java_room_tail::OwnerLink>,
    pub joined_room_invalidated:bool, pub synthetic_self_expectation_qualified: bool, pub stream_arrivals:u64, pub last_stream_change:Option<Change>,
    pub authenticated_singleton_self_qualified:bool,
}
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Receipt {pub sequence:u64, pub packet:Packet}
/// Standalone observations can examine CHANNEL preview data without claiming
/// a joined room. Runtime's owned adapter only installs JoinedCeReceipt books.
#[derive(Clone)] pub struct Book {epoch:SourceEpoch, key:Snapshot, own_expectation:Option<Vec<u8>>}
#[derive(Clone, Debug, PartialEq, Eq)] pub enum Change {Added, IgnoredDuplicateRow, StateUpdated, IgnoredUnknownAlias, IgnoredAmbiguousAlias, DepartureRemoved{rows:usize}, DepartureAbsentNoop, IgnoredDepartureOtherRoom, IgnoredUnqualifiedDeparture, IgnoredOwnDepartureWithoutCompletedIntent,OwnerUpdated{link:super::java_room_tail::OwnerLink},IgnoredOwnerOtherRoom,SelfExitNotification{write_serial:u64}}
impl Book {
    pub fn for_observation(epoch:&SourceEpoch,room_id:u32)->Result<Self>{Self::new(epoch,room_id,Origin::ExplicitObservation,None)}
    fn new(epoch:&SourceEpoch,room_id:u32,origin:Origin,ce_revision:Option<u64>)->Result<Self>{
        ensure!(room_id!=0,"observation room ID absent");
        Ok(Self{epoch:epoch.clone(),own_expectation:None,key:Snapshot{epoch:epoch.serial(),room_id,origin,ce_revision,revision:0,
            completeness:Completeness::PossiblyIncomplete,rows:vec![],ignored_unknown_states:0,last_unknown_state:None,authenticated_singleton_self_qualified:false,
            ignored_duplicate_rows:0,ignored_ambiguous_states:0,last_ambiguous_state:None,receipts:vec![],synthetic_self_expectation_qualified:false,stream_arrivals:0,last_stream_change:None,owner_display:None,owner_link:None,joined_room_invalidated:false}})
    }
    pub fn snapshot(&self)->Snapshot{self.key.clone()}
    fn owner_link(&self,alias:&[u8])->super::java_room_tail::OwnerLink {
        let rows=self.key.rows.iter().filter(|r|r.participant.display_alias==alias).collect::<Vec<_>>();
        match rows.len(){0=>super::java_room_tail::OwnerLink::Absent,1=>super::java_room_tail::OwnerLink::Unique{force:rows[0].participant.force},n=>super::java_room_tail::OwnerLink::Ambiguous{rows:n}}
    }
    pub(crate) fn invalidate_joined(&mut self,runtime:&Runtime)->Result<()> {
        runtime.source_scope.admit_epoch(&self.epoch)?;
        ensure!(self.key.origin==Origin::JoinedCeReceipt && self.key.ce_revision==Some(runtime.java_room.key.revision) && runtime.java_room.key.room.as_ref().is_some_and(|r|r.id==self.key.room_id) && runtime.java_room.key.phase==super::java_room_entry::Phase::RoomReceipts,"retired joined catalog invalidation");
        self.key.joined_room_invalidated=true;Ok(())
    }
    /// Scope must still be current in this Runtime. Equal serials belonging
    /// to another Runtime, retired epochs and another room cannot authorize it.
    /// A duplicate CD of the same kind/raw alias preserves its existing row
    /// and newer D1 state. Every receipt is retained separately. Cross-kind
    /// rows remain distinct; D1 only updates a unique known raw alias. Unknown
    /// or ambiguous D1 is ignored, never buffered or merged into an identity.
    pub fn observe(&mut self,runtime:&Runtime,event:&SourceDecodedPacket)->Result<Change>{
        runtime.source_scope.admit_epoch(&self.epoch)?;runtime.source_scope.admit(event)?;
        let packet=decode(event.body())?;let revision=self.key.revision.checked_add(1).context("participant arrival sequence exhausted")?;
        ensure!(self.key.receipts.len()<MAX_RECEIPTS,"participant receipt budget reached; no projection update");
        let change=match packet.clone() {
            Packet::Participant(participant)=>{
                ensure!(participant.room_id==self.key.room_id,"S_CD room differs from scoped observation");
                let row=Row{state:MemberState::decode(participant.initial_room_state)?,participant,
                    state_source:StateSource::CdArrival,last_arrival:revision,last_cd_arrival:revision,last_state_receipt:None};
                if self.key.rows.iter().any(|old|old.participant.force==row.participant.force && old.participant.display_alias==row.participant.display_alias){
                    let count=self.key.ignored_duplicate_rows.checked_add(1).context("duplicate row counter exhausted")?;
                    self.key.ignored_duplicate_rows=count;Change::IgnoredDuplicateRow
                }
                else {ensure!(self.key.rows.len()<MAX_ROWS,"participant observation budget reached; completeness remains unknown");self.key.rows.push(row);Change::Added}
            },
            Packet::Owner(receipt)=>{
                if receipt.room_id!=self.key.room_id{Change::IgnoredOwnerOtherRoom}
                else {let link=self.owner_link(&receipt.display_alias);self.key.owner_display=Some(receipt);self.key.owner_link=Some(link.clone());Change::OwnerUpdated{link}}
            },
            Packet::Departure(receipt)=>{
                if receipt.room_id!=self.key.room_id {Change::IgnoredDepartureOtherRoom}
                else if self.own_expectation.as_deref()==Some(receipt.display_alias.as_slice()) {
                    // Even this explicit synthetic expectation cannot replace
                    // a completed same-join C_D0 write. Retain every own row.
                    Change::IgnoredOwnDepartureWithoutCompletedIntent
                } else if self.own_expectation.is_none() && receipt.display_alias!=b"ArenaBot" {
                    // CE owner/CD/selected physical name cannot qualify self.
                    Change::IgnoredUnqualifiedDeparture
                } else {
                    let mut removed=0;
                    // D0 carries no kind: first exact raw alias in EACH kind.
                    // Observation uniqueness limits each kind to one row.
                    for kind in [0,1] {if let Some(at)=self.key.rows.iter().position(|r|r.participant.force==kind&&r.participant.display_alias==receipt.display_alias){self.key.rows.remove(at);removed+=1;}}
                    if removed==0{Change::DepartureAbsentNoop}else{Change::DepartureRemoved{rows:removed}}
                }
            },
            Packet::State(receipt)=>{
                let matches=self.key.rows.iter().filter(|row|row.participant.display_alias==receipt.display_alias).count();
                if matches==1 {let row=self.key.rows.iter_mut().find(|row|row.participant.display_alias==receipt.display_alias).unwrap();row.state=receipt.state;row.state_source=StateSource::D1Arrival;row.last_arrival=revision;row.last_state_receipt=Some(receipt);Change::StateUpdated}
                else if matches==0 {let ignored=self.key.ignored_unknown_states.checked_add(1).context("unknown alias counter exhausted")?;self.key.ignored_unknown_states=ignored;self.key.last_unknown_state=Some(receipt);Change::IgnoredUnknownAlias}
                else {let ignored=self.key.ignored_ambiguous_states.checked_add(1).context("ambiguous alias counter exhausted")?;self.key.ignored_ambiguous_states=ignored;self.key.last_ambiguous_state=Some(receipt);Change::IgnoredAmbiguousAlias}
            },
        };if let Some(owner)=&self.key.owner_display {self.key.owner_link=Some(self.owner_link(&owner.display_alias));}
        self.key.receipts.push(Receipt{sequence:revision,packet});self.key.revision=revision;Ok(change)
    }
}
#[derive(Default)] pub(crate) struct State {pub(crate) adapter:Option<Adapter>, pub(crate) book:Option<Book>, pub(crate) departure_adapter:Option<super::java_room_departure::Adapter>}
impl State {
    pub(crate) fn retire(&mut self){self.book=None;}
    fn joined_valid(&self)->bool{self.book.as_ref().is_some_and(|b|!b.key.joined_room_invalidated)}
    pub(crate) fn note_stream(&mut self,change:&Change){if let Some(book)=self.book.as_mut(){book.key.stream_arrivals+=1;book.key.last_stream_change=Some(change.clone());}}
    pub(crate) fn prepare_joined(&self,epoch:&SourceEpoch,room_id:u32,ce_revision:u64,own_expectation:Option<Vec<u8>>,actual_singleton:Option<Vec<u8>>)->Result<Option<Book>>{
        if self.adapter.is_some(){let mut book=Book::new(epoch,room_id,Origin::JoinedCeReceipt,Some(ce_revision))?;book.key.synthetic_self_expectation_qualified=own_expectation.is_some();book.key.authenticated_singleton_self_qualified=actual_singleton.is_some();book.own_expectation=actual_singleton.or(own_expectation);Ok(Some(book))}else{Ok(None)}
    }
}
impl Runtime {
    pub fn configure_java_room_participants_adapter(&mut self,adapter:Adapter)->Result<()>{
        ensure!(self.java_room.key.phase==super::java_room_entry::Phase::Idle && self.java_participants.book.is_none()
            && self.source_send.snapshot().pending.is_none(),"participant adapter change during room observation");
        self.java_participants.adapter=Some(adapter);Ok(())
    }
    pub fn java_room_participants_snapshot(&self)->Option<Snapshot>{self.java_participants.book.as_ref().map(Book::snapshot)}
    pub fn last_java_room_participants_checkpoint(&self)->Option<Snapshot>{self.last_participants_checkpoint.clone()}
    pub(crate) fn java_room_participants_route_enabled(&self)->bool{
        self.bootstrap_adapter.is_some() && self.java_room.adapter.is_some() && self.java_participants.adapter.is_some()
            && self.java_room.key.phase==super::java_room_entry::Phase::RoomReceipts
            && self.entry_navigation.key.scene==super::entry_navigation::Scene::RoomReceipts && self.java_participants.joined_valid()
    }
    pub fn consume_java_room_participants_candidate(&mut self,event:&SourceDecodedPacket)->Result<Change>{
        self.source_scope.admit(event)?;
        if event.body().first()==Some(&213){ensure!(self.java_room_owner_route_enabled(),"AD368 owner adapter unqualified");}
        if event.body().first()==Some(&208){ensure!(self.java_room_departure_route_enabled(),"AD368 D0 adapter unqualified");}
        ensure!(self.java_room_participants_route_enabled(),"AD368 participant outside explicit joined room receipt scope");
        let room=self.java_room.key.room.as_ref().context("joined room receipt absent")?;
        let book=self.java_participants.book.as_ref().context("joined participant book absent")?;
        ensure!(book.key.room_id==room.id && book.key.ce_revision==Some(self.java_room.key.revision)
            && book.key.origin==Origin::JoinedCeReceipt,"retired participant room revision");
        let mut next=book.clone();let mut change=next.observe(self,event)?;
        let (self_state,notification)=if event.body().first()==Some(&208){self.prepare_self_notification(&super::java_room_departure::decode(event.body())?)}else{(self.java_self.clone(),None)};
        if let Some(write_serial)=notification{change=Change::SelfExitNotification{write_serial};}
        self.java_participants.book=Some(next);self.java_self=self_state;
        self.status=if notification.is_some(){"已收到相符自身離房通知；伺服器完成離房仍未確認"}else{"已收到房間成員資料；名單仍可能不完整"}.into();Ok(change)
    }
}
