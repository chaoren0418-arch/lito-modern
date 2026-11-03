//! Fixed 4903D0/48FAA0/48F580 receiving and source-choice subset.
//! AD368 interoperability is separately enabled; it supplies no Native private
//! version extension, startup authority, phase, NEWS flag, or skill readiness.
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use std::{io::Read, path::Path};
use zeroize::Zeroizing;
use super::{Runtime, SourceDecodedPacket, SourceEpoch, UiPage};

pub const PRODUCER_SHA: &str = "AD368F49CF47E37A76A954AAFAAE1962AB2AB761057AAD4EE3145F7BBFD71219";
pub const CONTRACT_SHA: &str = "BF143F967C0064BBB52C8EFB5DD2F42EBA30EAB50653C2A6448E607319D13241";
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JavaFixtureAdapter { kind: String, producer_sha256: String, contract_sha256: String }
impl JavaFixtureAdapter {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= 512, "bootstrap adapter exceeds bound");
        let a: Self = serde_json::from_slice(bytes)?;
        ensure!(a.kind == "modern_synthetic_AD368_bootstrap_v1" && a.producer_sha256 == PRODUCER_SHA && a.contract_sha256 == CONTRACT_SHA, "explicit pinned AD368 adapter required");
        Ok(a)
    }
    pub fn load(path: &Path) -> Result<Self> {
        ensure!(path.is_absolute(), "absolute adapter path required");
        let mut bytes = Vec::new(); std::fs::File::open(path)?.take(513).read_to_end(&mut bytes)?;
        Self::parse(&bytes)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginCheck { pub result: i8, pub term: i32, pub name: Vec<u8>, pub flag: i8, pub word: u16, pub extension: Option<u32> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRow { pub name: Vec<u8>, pub address_word: u32, pub port_word: u16 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet { Check(LoginCheck), Directory(Vec<SourceRow>), News }
struct Reader<'a> { body: &'a [u8], at: usize }
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> { let end=self.at.checked_add(n).context("bootstrap offset overflow")?; let v=self.body.get(self.at..end).context("short bootstrap field")?; self.at=end; Ok(v) }
    fn string(&mut self, cap: usize) -> Result<Vec<u8>> { let n=self.body[self.at..].iter().take(cap).position(|b|*b==0).context("bootstrap bounded CString missing")?; Ok(self.take(n+1)?[..n].to_vec()) }
}
/// Wire structure only. Current-epoch version requirements are enforced below.
pub fn decode(body: &[u8]) -> Result<Packet> {
    ensure!(body.len() <= super::source_connection::MAX_DECODED_BODY, "bootstrap body exceeds frame bound");
    let mut r=Reader{body,at:1};
    let packet=match body.first() {
        Some(0) => { let result=r.take(1)?[0] as i8; let term=i32::from_le_bytes(r.take(4)?.try_into().unwrap()); let name=r.string(41)?; let flag=r.take(1)?[0] as i8; let word=u16::from_le_bytes(r.take(2)?.try_into().unwrap());
            let extension=if body.len()-r.at==4 {Some(u32::from_le_bytes(r.take(4)?.try_into().unwrap()))} else {None};
            Packet::Check(LoginCheck{result,term,name,flag,word,extension}) }
        Some(59) => { let count=r.take(1)?[0] as i8; ensure!(count>=0,"negative source count would request unsafe Native allocation"); ensure!(count as usize <= (body.len()-r.at)/7,"source rows cannot fit body"); let mut rows=Vec::new();
            for _ in 0..count { let name=r.string(64)?; let address_word=u32::from_le_bytes(r.take(4)?.try_into().unwrap()); let port_word=u16::from_le_bytes(r.take(2)?.try_into().unwrap()); rows.push(SourceRow{name,address_word,port_word}); }
            Packet::Directory(rows) }
        Some(230) => Packet::News,
        _ => anyhow::bail!("not LOGIN_CHECK/source-directory/NEWS trigger"),
    }; ensure!(r.at==body.len(),"trailing bootstrap bytes"); Ok(packet)
}
pub fn validate_check_version(check: &LoginCheck, version: i32, java_adapter: bool) -> Result<()> {
    if java_adapter { ensure!(check.extension.is_none(), "AD368 has no Native extension DWORD"); }
    else { ensure!(check.extension.is_some()==(version>=0x5226), "Native LOGIN_CHECK version extension DWORD mismatch"); }
    Ok(())
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Stage { #[default] Idle, AwaitLoginCheck, AwaitDirectory, SourceChoice, SendingSource, AwaitRedirect, GameConnecting, SendingGameLogin, AwaitNews, NewsChoice, SendingNews, AwaitRoster, Roster }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Key { pub stage: Stage, pub revision: u64, pub check: Option<LoginCheck>, pub sources: Vec<SourceRow>, pub source_count: usize, pub selected_name: Option<Vec<u8>>, pub write_serial: Option<u64>, pub credentials_pending: bool, pub roster_count: Option<u8> }
pub(crate) struct Credentials { pub(crate) account: Zeroizing<Vec<u8>>, pub(crate) password: Zeroizing<Vec<u8>> }
#[derive(Default)]
pub(crate) struct State { pub(crate) key: Key, pub(crate) credentials: Option<Credentials> }
impl State {
    pub(crate) fn retire(&mut self) { self.credentials=None; self.key=Key::default(); }
    fn revise(&mut self) -> Result<()> { self.key.revision=self.key.revision.checked_add(1).context("bootstrap revision exhausted")?; Ok(()) }
    pub(crate) fn written(&mut self, serial: u64) {
        if self.key.write_serial!=Some(serial) {return;}
        if self.key.stage==Stage::SendingGameLogin {self.credentials=None;self.key.credentials_pending=false;}
        self.key.stage=match self.key.stage {Stage::SendingSource=>Stage::AwaitRedirect,Stage::SendingGameLogin=>Stage::AwaitNews,Stage::SendingNews=>Stage::AwaitRoster,s=>s};
        self.key.write_serial=None;
    }
}
#[derive(Clone)]
pub struct SourceControl { epoch: SourceEpoch, revision: u64, slot: usize }
#[derive(Clone)]
pub struct NewsControl { epoch: SourceEpoch, revision: u64 }
impl SourceControl { pub fn slot(&self)->usize {self.slot} }
#[derive(Clone, Debug)]
pub enum Change { Checked{result:i8,java_without_native_extension:bool}, Directory{count:usize}, NewsPending, GameRedirect{target:std::net::SocketAddrV4} }
impl Runtime {
    pub fn configure_login_producer_fixture(&mut self, adapter: JavaFixtureAdapter)->Result<()> {
        ensure!(self.synthetic_player_fixture_configured() && self.source_connection_epoch().is_none() && matches!(self.page,UiPage::Booting|UiPage::Menu) && self.bootstrap_adapter.is_none(),"bootstrap fixture configuration unavailable");
        self.bootstrap_adapter=Some(adapter); Ok(())
    }
    pub fn source_bootstrap_snapshot(&self)->Key {self.login_bootstrap.key.clone()}
    pub fn source_choice_control(&self,slot:usize)->Option<SourceControl> {
        if self.page!=UiPage::Menu || self.login_bootstrap.key.stage!=Stage::SourceChoice || self.source_send.snapshot().pending.is_some() {return None;}
        if slot>=self.login_bootstrap.key.source_count {return None;}
        let row=self.login_bootstrap.key.sources.get(slot)?; if row.name.is_empty(){return None;}
        Some(SourceControl{epoch:self.source_scope.current_epoch().ok()?,revision:self.login_bootstrap.key.revision,slot})
    }
    pub fn handle_source_choice(&mut self,c:&SourceControl)->Result<()> {
        self.source_scope.admit_epoch(&c.epoch)?;
        ensure!(c.revision==self.login_bootstrap.key.revision && self.source_choice_control(c.slot).is_some(),"retired or foreign source choice");
        let name=self.login_bootstrap.key.sources[c.slot].name.clone();
        let prepared=self.source_send.prepare_source_choice(&name)?; let serial=prepared.reservation.ticket.serial();
        self.login_bootstrap.revise()?; self.login_bootstrap.key.selected_name=Some(name); self.login_bootstrap.key.write_serial=Some(serial); self.login_bootstrap.key.stage=Stage::SendingSource;
        self.source_send.commit(prepared); self.status="來源選擇等待傳送".into(); Ok(())
    }
    pub fn source_news_control(&self)->Option<NewsControl> {
        if self.page!=UiPage::Menu || self.bootstrap_adapter.is_none() || self.login_bootstrap.key.stage!=Stage::NewsChoice || self.source_send.snapshot().pending.is_some() {return None;}
        Some(NewsControl{epoch:self.source_scope.current_epoch().ok()?,revision:self.login_bootstrap.key.revision})
    }
    pub fn handle_source_news_continue(&mut self,c:&NewsControl)->Result<()> {
        self.source_scope.admit_epoch(&c.epoch)?; ensure!(c.revision==self.login_bootstrap.key.revision && self.source_news_control().is_some(),"retired or duplicate NEWS continuation");
        let prepared=self.source_send.prepare_news_continue()?; let serial=prepared.reservation.ticket.serial(); self.login_bootstrap.revise()?;
        if self.java_return.key.phase==super::java_return::Phase::Authenticating {ensure!(self.java_return.unexpired(),"retained return expired before NEWS");self.java_return.key.phase=super::java_return::Phase::ResumeRoster;}
        self.login_bootstrap.key.stage=Stage::SendingNews;self.login_bootstrap.key.write_serial=Some(serial);self.source_send.commit(prepared);self.status="等待角色名單".into();Ok(())
    }
    pub fn consume_source_bootstrap_event(&mut self,event:&SourceDecodedPacket)->Result<Change> {
        self.source_scope.admit(event)?; ensure!(self.page==UiPage::Menu,"bootstrap unavailable");
        match decode(event.body())? {
            Packet::Check(check)=>{
                ensure!(self.login_form.snapshot().stage==super::login_form::Stage::AwaitingReply,"LOGIN_CHECK requires completed login write");
                ensure!(!self.entry_transfer.roster_alternate(),"Native alternate LOGIN_CHECK suppression unimplemented");
                let version=self.source_actor.snapshot().version.context("LOGIN_CHECK requires current version")?.value();let adapted=self.bootstrap_adapter.is_some(); validate_check_version(&check,version,adapted)?;
                if adapted {ensure!(self.login_bootstrap.key.stage==Stage::AwaitLoginCheck,"duplicate or GAME LOGIN_CHECK");}
                let result=check.result;
                // Error dialogs and private locale/time-limit globals stay unqualified.
                ensure!(result==0,"LOGIN_CHECK refused: Native modal error effects unqualified");
                self.login_bootstrap.revise()?;self.login_bootstrap.key.check=Some(check);self.login_bootstrap.key.stage=Stage::AwaitDirectory;
                Ok(Change::Checked{result,java_without_native_extension:adapted&&version>=0x5226})
            },
            Packet::Directory(rows)=>{
                ensure!(matches!(self.login_bootstrap.key.stage,Stage::AwaitDirectory|Stage::SourceChoice),"source directory requires accepted LOGIN_CHECK");
                ensure!(!self.entry_transfer.roster_alternate(),"Native cached alternate source directory branch unqualified");
                let count=rows.len();
                self.login_bootstrap.revise()?;self.login_bootstrap.key.source_count=count;
                // Native zero count returns before replacing rows or opening UI.
                if count!=0 {self.login_bootstrap.key.sources=rows;self.login_bootstrap.key.stage=Stage::SourceChoice;self.login_form.retire()?;self.status="請選擇登入來源".into();}else{self.status="目前沒有可選擇的登入來源".into();}
                Ok(Change::Directory{count})
            },
            Packet::News=>{
                ensure!(self.bootstrap_adapter.is_some() && self.login_bootstrap.key.stage==Stage::AwaitNews,"NEWS requires fresh GAME version and completed reauthentication");
                self.login_form.retire()?;self.login_bootstrap.revise()?;self.login_bootstrap.key.stage=Stage::NewsChoice;self.status="按繼續取得角色名單".into();Ok(Change::NewsPending)
            },
        }
    }
    pub(crate) fn redirect_login_bootstrap(&mut self,slot:usize,address_word:u32,port_word:u16)->Result<Change> {
        ensure!(self.bootstrap_adapter.is_some() && self.login_bootstrap.key.stage==Stage::AwaitRedirect && self.source_send.snapshot().pending.is_none(),"GAME redirect requires completed source choice");
        ensure!(self.entry_transfer.key.table[slot]!=u32::MAX,"source GAME slot unavailable");
        let target=std::net::SocketAddrV4::new(super::entry_transfer::address_from_word(address_word),port_word);
        ensure!(target.ip().is_loopback() && target.port()!=0,"bootstrap GAME requires owned loopback target");
        self.reconnect_login_bootstrap_fixture(target)?;Ok(Change::GameRedirect{target})
    }
}
