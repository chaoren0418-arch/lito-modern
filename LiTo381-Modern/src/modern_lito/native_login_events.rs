//! Explicit synthetic player transport. Worker owns IO; Runtime owns all protocol state.
use super::{
    login_form, login_transport::Transport, login_wire, source_send, Runtime, SourceEpoch, UiPage,
};
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use std::{
    io::{ErrorKind, Read, Write},
    net::SocketAddr,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
const EVENTS: usize = 4;
const POLL_BUDGET: usize = 8;
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureProfile {
    kind: String,
    label: String,
    endpoint: SocketAddr,
    code_page: u32,
    from_cd_registry_dword: u32,
    revision_word_source: u32,
    supplemental: Option<String>,
    read_cap: usize,
    write_cap: usize,
    connect_delay_ms: u64,
}
/// Explicit real-backend loopback transport candidate. The digest is a
/// provenance binding; the external consumer must verify supervisor acceptance
/// and freshness before calling this interface. It grants no server authority.
#[derive(Clone)]
pub struct Backend485fProfile { fixture:FixtureProfile,provider_acceptance_sha256:String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Backend485fSpec {kind:String,label:String,endpoint:SocketAddr,producer_sha256:String,provider_acceptance_sha256:String,read_cap:usize,write_cap:usize}
impl Backend485fProfile {
 pub fn parse(raw:&[u8])->Result<Self>{
  ensure!(raw.len()<=4096,"backend profile exceeds bound");let v:Backend485fSpec=serde_json::from_slice(raw)?;
  ensure!(v.kind=="modern_backend_485F_loopback_v1"&&v.label.starts_with("BACKEND485F:")&&v.producer_sha256==super::java_room_ready::PRODUCER_SHA,"explicit485F backend profile required");
  ensure!(v.provider_acceptance_sha256.len()==64&&v.provider_acceptance_sha256.bytes().all(|b|b.is_ascii_hexdigit()),"accepted provider digest required; not an acceptance proof");
  ensure!(v.endpoint.is_ipv4()&&v.endpoint.ip().is_loopback()&&v.endpoint.port()!=0,"backend requires ownedIPv4loopback endpoint");
  ensure!((1..=1024).contains(&v.read_cap)&&(1..=157).contains(&v.write_cap),"backend IO bounds exceeded");
  let fixture=FixtureProfile{kind:v.kind,label:v.label,endpoint:v.endpoint,code_page:950,from_cd_registry_dword:2,revision_word_source:0x14f8,supplemental:None,read_cap:v.read_cap,write_cap:v.write_cap,connect_delay_ms:0};fixture.environment()?;
  Ok(Self{fixture,provider_acceptance_sha256:v.provider_acceptance_sha256.to_ascii_uppercase()})
 }
 pub fn provider_acceptance_sha256(&self)->&str{&self.provider_acceptance_sha256}
}
impl FixtureProfile {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= 4096, "synthetic profile exceeds 4096 bytes");
        let p: Self = serde_json::from_slice(bytes)?;
        ensure!(
            p.kind == "modern_synthetic_loopback_fixture_v1" && p.label.starts_with("SYNTHETIC:"),
            "explicit synthetic profile required"
        );
        ensure!(
            p.endpoint.is_ipv4() && p.endpoint.ip().is_loopback(),
            "synthetic IPv4 loopback endpoint required"
        );
        ensure!(
            (1..=1024).contains(&p.read_cap)
                && (1..=157).contains(&p.write_cap)
                && p.connect_delay_ms <= 2000,
            "fixture IO bounds exceeded"
        );
        p.environment()?;
        Ok(p)
    }
    pub fn load(path: &Path) -> Result<Self> {
        ensure!(
            path.is_absolute(),
            "synthetic profile path must be absolute"
        );
        let mut raw = Vec::new();
        std::fs::File::open(path)?
            .take(4097)
            .read_to_end(&mut raw)?;
        Self::parse(&raw)
    }
    fn environment(&self) -> Result<login_wire::FixtureEnvironment> {
        login_wire::FixtureEnvironment::synthetic(
            &self.label,
            self.code_page,
            self.from_cd_registry_dword,
            self.revision_word_source,
            match &self.supplemental {
                Some(s) => login_wire::FixtureVariant::Special {
                    supplemental: s.as_bytes().to_vec(),
                },
                None => login_wire::FixtureVariant::Normal,
            },
        )
    }
}
#[derive(Default)]
struct Stats {
    connected: AtomicBool,
    retired: AtomicBool,
    queued: AtomicUsize,
    reads: AtomicUsize,
    writes: AtomicUsize,
    bytes_read: AtomicUsize,
    max_queue: AtomicUsize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub configured: bool,
    pub active: bool,
    pub connected: bool,
    pub write_inflight: bool,
    pub queued_events: usize,
    pub max_queued_events: usize,
    pub max_receiver_tail_bytes: usize,
    pub retiring_workers: usize,
    pub workers_started: usize,
    pub workers_joined: usize,
    pub discarded_late_events: usize,
    pub read_events: usize,
    pub write_reports: usize,
    pub bytes_read: usize,
}
enum Job {
    Write {
        ticket: source_send::Ticket,
        offset: usize,
        bytes: Vec<u8>,
    },
}
enum Event {
    Connected(Transport),
    ConnectFailed,
    Read(Vec<u8>),
    Wrote {
        ticket: source_send::Ticket,
        offset: usize,
        outcome: source_send::WriteOutcome,
    },
    Eof,
    ReadFailed,
}
struct Handle {
    epoch: SourceEpoch,
    cancel: Arc<AtomicBool>,
    stats: Arc<Stats>,
    jobs: SyncSender<Job>,
    events: Receiver<Event>,
    worker: Option<JoinHandle<()>>,
    inflight: bool,
}
#[derive(Default)]
pub(crate) struct Session {
    profile: Option<FixtureProfile>,
    active: Option<Handle>,
    retiring: Vec<Handle>,
    pub(crate) submit: Option<login_form::Control>,
    reopen_after_back: bool,
    started: usize,
    joined: usize,
    late: usize,
    reads: usize,
    writes: usize,
    bytes_read: usize,
    max_queue: usize,
    max_rx_tail: usize,
}
impl Session {
    pub(crate) fn retire(&mut self) {
        self.submit = None;
        self.reopen_after_back = false;
        if let Some(h) = self.active.take() {
            h.cancel.store(true, Ordering::Release);
            self.retiring.push(h);
        }
    }
    pub(crate) fn close(&mut self) {
        self.retire();
        self.profile = None;
    }
    pub(crate) fn idle(&self) -> bool {
        self.active.is_none() && self.retiring.is_empty()
    }
    fn reap(&mut self) -> usize {
        let mut late = 0;
        for h in &mut self.retiring {
            while h.events.try_recv().is_ok() {
                h.stats.queued.fetch_sub(1, Ordering::AcqRel);
                late += 1;
            }
        }
        let mut i = 0;
        while i < self.retiring.len() {
            if self.retiring[i]
                .worker
                .as_ref()
                .is_some_and(JoinHandle::is_finished)
            {
                let mut h = self.retiring.remove(i);
                let _ = h.worker.take().unwrap().join(); // is_finished checked: never waits for IO.
                while h.events.try_recv().is_ok() {
                    h.stats.queued.fetch_sub(1, Ordering::AcqRel);
                    late += 1;
                }
                self.reads += h.stats.reads.load(Ordering::Acquire);
                self.writes += h.stats.writes.load(Ordering::Acquire);
                self.bytes_read += h.stats.bytes_read.load(Ordering::Acquire);
                self.max_queue = self
                    .max_queue
                    .max(h.stats.max_queue.load(Ordering::Acquire));
                self.joined += 1;
            } else {
                i += 1;
            }
        }
        self.late += late;
        late
    }
    fn snapshot(&self) -> Snapshot {
        let stats = self.active.as_ref().map(|h| &h.stats);
        Snapshot {
            configured: self.profile.is_some(),
            active: self.active.is_some(),
            connected: stats.is_some_and(|s| s.connected.load(Ordering::Acquire)),
            write_inflight: self.active.as_ref().is_some_and(|h| h.inflight),
            queued_events: stats.map_or(0, |s| s.queued.load(Ordering::Acquire).min(EVENTS)),
            max_queued_events: self
                .max_queue
                .max(stats.map_or(0, |s| s.max_queue.load(Ordering::Acquire))),
            max_receiver_tail_bytes: self.max_rx_tail,
            retiring_workers: self.retiring.len(),
            workers_started: self.started,
            workers_joined: self.joined,
            discarded_late_events: self.late,
            read_events: self.reads + stats.map_or(0, |s| s.reads.load(Ordering::Acquire)),
            write_reports: self.writes + stats.map_or(0, |s| s.writes.load(Ordering::Acquire)),
            bytes_read: self.bytes_read + stats.map_or(0, |s| s.bytes_read.load(Ordering::Acquire)),
        }
    }
}
fn publish(tx: &SyncSender<Event>, mut event: Event, cancel: &AtomicBool, stats: &Stats) -> bool {
    loop {
        if cancel.load(Ordering::Acquire) {
            return false;
        }
        stats.queued.fetch_add(1, Ordering::AcqRel);
        match tx.try_send(event) {
            Ok(()) => {
                stats
                    .max_queue
                    .fetch_max(stats.queued.load(Ordering::Acquire), Ordering::AcqRel);
                return true;
            }
            Err(TrySendError::Full(e)) => {
                stats.queued.fetch_sub(1, Ordering::AcqRel);
                event = e;
            }
            Err(TrySendError::Disconnected(_)) => {
                stats.queued.fetch_sub(1, Ordering::AcqRel);
                return false;
            }
        }
        thread::sleep(Duration::from_millis(2));
    }
}
fn io_worker(
    profile: FixtureProfile,
    commands: Receiver<Job>,
    events: SyncSender<Event>,
    cancel: Arc<AtomicBool>,
    stats: Arc<Stats>,
) {
    struct Retired(Arc<Stats>);
    impl Drop for Retired {
        fn drop(&mut self) {
            self.0.retired.store(true, Ordering::Release);
        }
    }
    let _retired = Retired(stats.clone());
    let delayed_until = Instant::now() + Duration::from_millis(profile.connect_delay_ms);
    while Instant::now() < delayed_until {
        if cancel.load(Ordering::Acquire) {
            return;
        }
        thread::sleep(Duration::from_millis(2));
    }
    if cancel.load(Ordering::Acquire) {
        return;
    }
    let connection = (|| -> Result<_> {
        let mut t = Transport::connect(profile.endpoint, profile.environment()?)?;
        let socket = t.take_stream()?;
        socket.set_nonblocking(true)?;
        Ok((t, socket))
    })();
    let (metadata, mut stream) = match connection {
        Ok(c) => c,
        Err(_) => {
            publish(&events, Event::ConnectFailed, &cancel, &stats);
            return;
        }
    };
    stats.connected.store(true, Ordering::Release);
    if !publish(&events, Event::Connected(metadata), &cancel, &stats) {
        return;
    }
    let mut pending: Option<Job> = None;
    while !cancel.load(Ordering::Acquire) {
        if pending.is_none() {
            match commands.try_recv() {
                Ok(j) => pending = Some(j),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => break,
            }
        }
        if let Some(Job::Write {
            ticket,
            offset,
            bytes,
        }) = pending.take()
        {
            match stream.write(&bytes[..bytes.len().min(profile.write_cap)]) {
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    pending = Some(Job::Write {
                        ticket,
                        offset,
                        bytes,
                    })
                }
                result => {
                    let outcome = match result {
                        Ok(n) => source_send::WriteOutcome::Written(n),
                        Err(_) => source_send::WriteOutcome::Failed,
                    };
                    stats.writes.fetch_add(1, Ordering::AcqRel);
                    if !publish(
                        &events,
                        Event::Wrote {
                            ticket,
                            offset,
                            outcome,
                        },
                        &cancel,
                        &stats,
                    ) {
                        return;
                    }
                }
            }
        }
        let mut raw = vec![0; profile.read_cap];
        match stream.read(&mut raw) {
            Ok(0) => {
                publish(&events, Event::Eof, &cancel, &stats);
                break;
            }
            Ok(n) => {
                raw.truncate(n);
                stats.reads.fetch_add(1, Ordering::AcqRel);
                stats.bytes_read.fetch_add(n, Ordering::AcqRel);
                if !publish(&events, Event::Read(raw), &cancel, &stats) {
                    return;
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(_) => {
                publish(&events, Event::ReadFailed, &cancel, &stats);
                break;
            }
        }
        thread::sleep(Duration::from_millis(2));
    }
}
impl Runtime {
    pub fn player_fixture_snapshot(&self) -> Snapshot {
        self.login_events.snapshot()
    }
    pub fn player_fixture_write_view(&self) -> Result<Option<source_send::WriteView>> {
        let h = self
            .login_events
            .active
            .as_ref()
            .context("owned fixture worker required")?;
        self.source_scope.admit_epoch(&h.epoch)?;
        self.source_send
            .pending_ticket()
            .map(|t| self.source_send.view(&t))
            .transpose()
    }
    pub fn synthetic_player_fixture_configured(&self) -> bool {
        self.login_events.profile.is_some()
    }
    pub fn configure_synthetic_player_fixture(&mut self, profile: FixtureProfile) -> Result<()> {
        ensure!(
            matches!(self.page, UiPage::Booting | UiPage::Menu)
                && self.login_events.idle()
                && self.source_connection_epoch().is_none(),
            "fixture configuration unavailable"
        );
        self.login_events.profile = Some(profile);
        self.status = "合成 loopback 測試；真實認證尚未開放".into();
        Ok(())
    }
    pub fn configure_485f_backend_profile(&mut self,profile:Backend485fProfile)->Result<()> {
        ensure!(matches!(self.page,UiPage::Booting|UiPage::Menu)&&self.login_events.idle()&&self.source_connection_epoch().is_none(),"backend profile configuration unavailable");
        self.login_events.profile=Some(profile.fixture);
        self.status="485F backend 有限實接候選；往返尚未驗證".into();Ok(())
    }
    pub(crate) fn begin_player_fixture_if_needed(&mut self) -> Result<()> {
        if self.login_events.profile.is_none() || self.login_events.active.is_some() {
            return Ok(());
        }
        let late = self.login_events.reap();
        self.stale_results_rejected += late as u64;
        ensure!(
            self.login_events.retiring.len() < 4,
            "fixture worker retirement budget reached"
        );
        let profile = self.login_events.profile.as_ref().unwrap().clone();
        let epoch = self.begin_source_connection_candidate()?;
        self.spawn_owned_player_fixture(profile,epoch)
    }
    fn spawn_owned_player_fixture(&mut self,profile:FixtureProfile,epoch:SourceEpoch)->Result<()> {
        let (tx, rx) = mpsc::sync_channel(1);
        let (out, events) = mpsc::sync_channel(EVENTS);
        let cancel = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(Stats::default());
        let c = cancel.clone();
        let s = stats.clone();
        let worker = match thread::Builder::new()
            .name("modern-synthetic-socket".into())
            .spawn(move || io_worker(profile, rx, out, c, s))
        {
            Ok(w) => w,
            Err(e) => {
                self.disconnect_source_connection_candidate(&epoch)?;
                return Err(e.into());
            }
        };
        self.login_events.active = Some(Handle {
            epoch,
            cancel,
            stats,
            jobs: tx,
            events,
            worker: Some(worker),
            inflight: false,
        });
        self.login_events.started += 1;
        if self.entry_transfer.key.current_address.is_none(){if let SocketAddr::V4(address)=self.login_events.profile.as_ref().context("fixture profile absent")?.endpoint{self.entry_transfer.key.current_address=Some(address);}}
        self.status = "合成端點連線中；可輸入測試資料".into();
        Ok(())
    }
    pub(crate) fn source_current_fixture_endpoint_v4(&self)->Option<std::net::SocketAddrV4>{self.entry_transfer.key.current_address}
    pub(crate) fn reconnect_source_transfer_fixture(&mut self,target:std::net::SocketAddrV4)->Result<()> {
        ensure!(target.ip().is_loopback()&&target.port()!=0&&self.login_events.active.is_some(),"owned loopback redirect required");
        let late=self.login_events.reap();self.stale_results_rejected+=late as u64;
        ensure!(self.login_events.retiring.len()<4,"fixture redirect retirement budget reached");
        let mut profile=self.login_events.profile.as_ref().context("configured fixture absent")?.clone();
        //Wire F4 supplies the target. Profile only supplies synthetic IO/environment.
        profile.endpoint=SocketAddr::V4(target);
        let mut transfer=self.entry_transfer.clone();transfer.reconnect_after_write=None;
        transfer.key.server_rows.clear();transfer.key.rows_revision=transfer.key.rows_revision.checked_add(1).context("row revision exhausted")?;
        let flow=self.source_flow.transfer_connected();let old_return=(self.entry_navigation.key.return_state,self.entry_navigation.key.return_mode);
        let epoch=self.begin_source_connection_candidate()?;
        self.entry_transfer=transfer;self.source_flow=flow;
        self.entry_navigation.key.scene=super::entry_navigation::Scene::AwaitTransferRoster;
        self.entry_navigation.key.return_state=old_return.0;self.entry_navigation.key.return_mode=old_return.1;
        self.spawn_owned_player_fixture(profile,epoch)?;
        self.status="頻道切換連線中，等待伺服器回覆…".into();Ok(())
    }
    /// Explicit fresh GAME adapter resume after cooperative old-worker join.
    /// New authentication and NEWS are required; no automatic credential replay.
    pub fn resume_java_return_fixture(&mut self)->Result<()> {
        ensure!(self.bootstrap_adapter.is_some()&&self.java_return.key.phase==super::java_return::Phase::Retained&&self.java_return.unexpired(),"retained AD368 return unavailable or expired");
        let late=self.login_events.reap();self.stale_results_rejected+=late as u64;
        ensure!(self.login_events.idle()&&self.source_connection_epoch().is_none(),"old socket worker must join before fresh GAME");
        let mut profile=self.login_events.profile.as_ref().context("configured fixture absent")?.clone();
        profile.endpoint=SocketAddr::V4(self.java_return.endpoint.context("retained GAME address absent")?);
        let mut retained=std::mem::take(&mut self.java_return);
        let epoch=self.begin_source_connection_candidate()?;
        retained.key.phase=super::java_return::Phase::Authenticating;
        self.login_bootstrap.key.selected_name=retained.key.source.clone();self.java_return=retained;
        self.spawn_owned_player_fixture(profile,epoch)?;self.open_source_login_form()?;
        self.status="重新輸入同一測試帳號，驗證返回資料".into();Ok(())
    }
    pub(crate) fn request_player_fixture_submit(&mut self) {
        if self.login_events.active.is_some() {
            self.login_events.submit = self.source_login_control();
            self.status = "合成資料已準備，等待連線與版本回包".into();
        }
    }
    pub(crate) fn reconnect_login_bootstrap_fixture(&mut self,target:std::net::SocketAddrV4)->Result<()> {
        ensure!(self.bootstrap_adapter.is_some()&&target.ip().is_loopback()&&target.port()!=0&&self.login_events.active.is_some(),"owned AD368 GAME redirect required");
        let late=self.login_events.reap();self.stale_results_rejected+=late as u64;
        ensure!(self.login_events.retiring.len()<4,"bootstrap retirement budget reached");
        let mut profile=self.login_events.profile.as_ref().context("fixture profile absent")?.clone();profile.endpoint=SocketAddr::V4(target);
        let mut bootstrap=std::mem::take(&mut self.login_bootstrap);
        ensure!(bootstrap.credentials.is_some(),"GAME credential lease absent");
        let epoch=self.begin_source_connection_candidate()?;
        bootstrap.key.stage=super::login_bootstrap::Stage::GameConnecting;bootstrap.key.write_serial=None;
        self.login_bootstrap=bootstrap;
        self.spawn_owned_player_fixture(profile,epoch)?;self.entry_transfer.key.current_address=Some(target);
        self.status="來源已選擇，正在連線 GAME".into();Ok(())
    }
    pub(crate) fn request_player_fixture_reopen(&mut self) {
        if self.login_events.active.is_some() {
            self.login_events.reopen_after_back = true;
        }
    }
    fn drive_player_fixture_write(&mut self) -> Result<()> {
        let Some(epoch) = self.login_events.active.as_ref().map(|h| h.epoch.clone()) else {
            return Ok(());
        };
        self.source_scope.admit_epoch(&epoch)?;
        if self.source_send.snapshot().pending.is_none()
            && self.source_actor.snapshot().version.is_some()
            && self
                .source_loopback_snapshot()
                .is_some_and(|s| s.compatibility_complete)
        {
            if let Some(c) = self.login_events.submit.take() {
                self.prepare_source_login_fixture(&epoch, &c)?;
            } else if self.login_bootstrap.key.stage==super::login_bootstrap::Stage::GameConnecting {
                self.prepare_game_bootstrap_login()?;
            }
        }
        self.drive_source_navigation_sequence()?;
        let h = self.login_events.active.as_mut().unwrap();
        if !h.inflight {
            if let Some(ticket) = self.source_send.pending_ticket() {
                let view = self.source_send.view(&ticket)?;
                let job = Job::Write {
                    ticket,
                    offset: view.offset(),
                    bytes: view.bytes()?.to_vec(),
                };
                match h.jobs.try_send(job) {
                    Ok(()) => h.inflight = true,
                    Err(TrySendError::Full(_)) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        anyhow::bail!("fixture socket worker ended")
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn poll_player_fixture_events(&mut self) -> bool {
        let late = self.login_events.reap();
        self.stale_results_rejected += late as u64;
        let mut changed = late != 0;
        if matches!(self.page, UiPage::Closing | UiPage::Closed) {
            return changed;
        }
        for _ in 0..POLL_BUDGET {
            let Some(epoch) = self.login_events.active.as_ref().map(|h| h.epoch.clone()) else {
                break;
            };
            if self.source_scope.admit_epoch(&epoch).is_err() {
                self.login_events.retire();
                changed = true;
                break;
            }
            if self
                .login_transport
                .as_ref()
                .is_some_and(|t| !t.queued.is_empty())
            {
                let raw = std::mem::take(&mut self.login_transport.as_mut().unwrap().queued);
                match self.feed_source_stream_candidate(&epoch, &raw) {
                    Ok(batch) => {
                        if let Some(t) = self.login_transport.as_mut().filter(|_|self.source_scope.admit_epoch(&epoch).is_ok()) {
                            t.queued.extend_from_slice(&raw[batch.consumed..]);
                            self.login_events.max_rx_tail =
                                self.login_events.max_rx_tail.max(t.queued.len());
                        }
                    }
                    Err(_) => {
                        let _ = self.disconnect_source_connection_candidate(&epoch);
                        self.status = "合成回包處理失敗，連線已結束".into();
                    }
                }
                changed = true;
                continue;
            }
            let event = {
                let h = self.login_events.active.as_ref().unwrap();
                h.events.try_recv()
            };
            let event = match event {
                Ok(e) => {
                    self.login_events
                        .active
                        .as_ref()
                        .unwrap()
                        .stats
                        .queued
                        .fetch_sub(1, Ordering::AcqRel);
                    e
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = self.disconnect_source_connection_candidate(&epoch);
                    self.status = "合成 socket 工作已停止，資料已清除".into();
                    changed = true;
                    break;
                }
            };
            changed = true;
            match event {
                Event::Connected(t) => {
                    self.login_transport = Some(t);
                    self.status = "合成端點已連線，等待 seed 與版本".into();
                }
                Event::ConnectFailed => {
                    let _ = self.disconnect_source_connection_candidate(&epoch);
                    self.status = "合成連線失敗，資料已清除".into();
                }
                Event::Read(raw) => {
                    if let Some(t) = &mut self.login_transport {
                        t.queued = raw;
                    }
                }
                Event::Wrote {
                    ticket,
                    offset,
                    outcome,
                } => {
                    self.login_events.active.as_mut().unwrap().inflight = false;
                    match self.report_source_write_candidate(&epoch, &ticket, offset, outcome) {
                        Ok(source_send::Progress::Complete)
                            if self.login_events.reopen_after_back =>
                        {
                            self.login_events.reopen_after_back = false;
                            if let Err(_) = self.open_source_login_form() {
                                let _ = self.disconnect_source_connection_candidate(&epoch);
                            } else {
                                self.status = "合成測試已返回登入；沿用同一連線".into();
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {
                            let _ = self.disconnect_source_connection_candidate(&epoch);
                            self.status = "過期合成傳送結果已拒收，連線已結束".into();
                        }
                    }
                }
                Event::Eof => {
                    let _ = self.end_source_stream_candidate(&epoch);
                    self.status = "合成端點已結束連線，資料已清除".into();
                }
                Event::ReadFailed => {
                    let _ = self.disconnect_source_connection_candidate(&epoch);
                    self.status = "合成讀取失敗，資料已清除".into();
                }
            }
        }
        if let Err(_) = self.drive_player_fixture_write() {
            if let Some(epoch) = self.login_events.active.as_ref().map(|h| h.epoch.clone()) {
                let _ = self.disconnect_source_connection_candidate(&epoch);
                self.status = "合成傳送工作已停止，資料已清除".into();
            }
            changed = true;
        }
        changed
    }
}

#[cfg(test)]mod r527_backend_profile_tests {
 use super::*;
 fn spec()->serde_json::Value{serde_json::json!({"kind":"modern_backend_485F_loopback_v1","label":"BACKEND485F:R527_PREPARED","endpoint":"127.0.0.1:12345","producer_sha256":super::super::java_room_ready::PRODUCER_SHA,"provider_acceptance_sha256":"A".repeat(64),"read_cap":31,"write_cap":157})}
 #[test]fn explicit_backend_provenance_and_fixed_login_shape(){let value=spec();let raw=serde_json::to_vec(&value).unwrap();assert!(FixtureProfile::parse(&raw).is_err());let p=Backend485fProfile::parse(&raw).unwrap();assert_eq!(p.provider_acceptance_sha256(),"A".repeat(64));assert_eq!(p.fixture.environment().unwrap().provenance(),"BACKEND485F:R527_PREPARED");assert_eq!(p.fixture.environment().unwrap().compatibility_body(),[0,51,128,182,3,0,0,2,248,20]);assert!(matches!(p.fixture.environment().unwrap().variant,login_wire::FixtureVariant::Normal));}
 #[test]fn backend_rejects_wrongproducer_pendingdigest_nonloopback_and_io(){for(key,bad)in[("kind",serde_json::json!("modern_synthetic_loopback_fixture_v1")),("label",serde_json::json!("SYNTHETIC:invalid")),("producer_sha256",serde_json::json!("AD368")),("provider_acceptance_sha256",serde_json::json!("WAITING_PROVIDER")),("endpoint",serde_json::json!("192.168.1.1:12345")),("endpoint",serde_json::json!("127.0.0.1:0")),("read_cap",serde_json::json!(0)),("write_cap",serde_json::json!(158))]{let mut v=spec();v[key]=bad;assert!(Backend485fProfile::parse(&serde_json::to_vec(&v).unwrap()).is_err());}let mut v=spec();v["password"]=serde_json::json!("forbidden");assert!(Backend485fProfile::parse(&serde_json::to_vec(&v).unwrap()).is_err());}
}
