//! Persistent native TCP owner. UI polls bounded events; a single worker owns
//! reads/writes and cancellation. Version acceptance is not account login.
use super::{
    native_version::{self, ServerVersion, VersionReply},
    native_wire::{self, CipherState, FrameDecoder},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::VecDeque,
    io::{ErrorKind, Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub connect_timeout: Duration,
    pub handshake_timeout: Duration,
    pub io_slice: Duration,
    pub read_chunk: usize,
    pub write_chunk: usize,
    pub max_frame: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(2),
            handshake_timeout: Duration::from_secs(5),
            io_slice: Duration::from_millis(50),
            read_chunk: 4096,
            write_chunk: 4096,
            max_frame: native_wire::MAX_FRAME,
        }
    }
}
impl Limits {
    fn validate(self) -> Result<Self> {
        ensure!(
            !self.connect_timeout.is_zero() && self.connect_timeout <= Duration::from_secs(5),
            "native connect timeout limit"
        );
        ensure!(
            !self.handshake_timeout.is_zero() && self.handshake_timeout <= Duration::from_secs(30),
            "native handshake timeout limit"
        );
        ensure!(
            !self.io_slice.is_zero() && self.io_slice <= Duration::from_millis(100),
            "native IO slice limit"
        );
        ensure!(
            (1..=4096).contains(&self.read_chunk) && (1..=4096).contains(&self.write_chunk),
            "native IO chunk limit"
        );
        ensure!(
            (32..=native_wire::MAX_FRAME).contains(&self.max_frame),
            "native frame budget"
        );
        Ok(self)
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Negotiated {
    pub peer: SocketAddr,
    pub local: SocketAddr,
    pub seed: u32,
    #[serde(serialize_with = "decimal_u64")]
    pub rx_counter: u64,
    #[serde(serialize_with = "decimal_u64")]
    pub tx_counter: u64,
    pub server: ServerVersion,
}
fn decimal_u64<S: serde::Serializer>(
    value: &u64,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}
struct Session {
    stream: TcpStream,
    limits: Limits,
    decoder: FrameDecoder,
    pending: Vec<u8>,
    cursor: usize,
    rx: CipherState,
    tx: CipherState,
    negotiated: Negotiated,
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "native connection cancelled"
    );
    Ok(())
}
fn timeout(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
    )
}
impl Session {
    fn send_body(
        &mut self,
        mut body: zeroize::Zeroizing<Vec<u8>>,
        cancel: &AtomicBool,
    ) -> Result<()> {
        ensure!(
            !body.is_empty() && body.len() + 2 <= self.limits.max_frame,
            "native outgoing frame budget"
        );
        active(cancel)?;
        let mut next = self.tx.clone();
        next.encrypt(&mut body);
        let frame = native_wire::frame(&body)?;
        write_frame(
            &self.stream,
            &frame,
            self.limits,
            Instant::now() + self.limits.handshake_timeout,
            cancel,
        )?;
        self.tx = next;
        self.negotiated.tx_counter = self.tx.counter();
        Ok(())
    }
    fn connect(peer: SocketAddr, limits: Limits, cancel: &AtomicBool) -> Result<Self> {
        let limits = limits.validate()?;
        active(cancel)?;
        let stream = TcpStream::connect_timeout(&peer, limits.connect_timeout)
            .context("native TCP connect")?;
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(limits.io_slice))?;
        stream.set_write_timeout(Some(limits.io_slice))?;
        let local = stream.local_addr()?;
        let actual = stream.peer_addr()?;
        ensure!(actual == peer, "native TCP peer changed");
        active(cancel)?;
        let mut decoder = FrameDecoder::new(limits.max_frame)?;
        let mut pending = Vec::new();
        let mut cursor = 0;
        let deadline = Instant::now() + limits.handshake_timeout;
        let seed_body = wait_frame(
            &stream,
            &mut decoder,
            &mut pending,
            &mut cursor,
            limits,
            deadline,
            cancel,
        )?;
        let seed = native_version::seed(&seed_body)?;
        let mut tx = CipherState::new(seed);
        let mut rx = CipherState::new(seed);
        let mut request = native_version::version_request(0).to_vec();
        tx.encrypt(&mut request);
        write_frame(
            &stream,
            &native_wire::frame(&request)?,
            limits,
            deadline,
            cancel,
        )?;
        let mut response = wait_frame(
            &stream,
            &mut decoder,
            &mut pending,
            &mut cursor,
            limits,
            deadline,
            cancel,
        )?;
        rx.decrypt(&mut response);
        let server = match native_version::parse_version(&response)? {
            VersionReply::Accepted(server) => server,
            VersionReply::Rejected => anyhow::bail!("native version rejected"),
        };
        active(cancel)?;
        let negotiated = Negotiated {
            peer: actual,
            local,
            seed,
            rx_counter: rx.counter(),
            tx_counter: tx.counter(),
            server,
        };
        Ok(Self {
            stream,
            limits,
            decoder,
            pending,
            cursor,
            rx,
            tx,
            negotiated,
        })
    }
    fn poll_body(&mut self, cancel: &AtomicBool) -> Result<Option<Vec<u8>>> {
        self.negotiated.tx_counter = self.tx.counter();
        let body = poll_frame(
            &self.stream,
            &mut self.decoder,
            &mut self.pending,
            &mut self.cursor,
            self.limits,
            cancel,
        )?;
        if let Some(mut body) = body {
            ensure!(!body.is_empty(), "native packet missing opcode");
            self.rx.decrypt(&mut body);
            self.negotiated.rx_counter = self.rx.counter();
            Ok(Some(body))
        } else {
            Ok(None)
        }
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

fn poll_frame(
    stream: &TcpStream,
    decoder: &mut FrameDecoder,
    pending: &mut Vec<u8>,
    cursor: &mut usize,
    limits: Limits,
    cancel: &AtomicBool,
) -> Result<Option<Vec<u8>>> {
    active(cancel)?;
    loop {
        if *cursor < pending.len() {
            let (used, body) = decoder.feed(&pending[*cursor..])?;
            *cursor += used;
            if let Some(body) = body {
                return Ok(Some(body));
            }
            ensure!(used > 0, "native decoder stalled");
        }
        if *cursor == pending.len() {
            pending.clear();
            *cursor = 0;
            let mut buffer = [0u8; 4096];
            let mut reader = stream;
            match reader.read(&mut buffer[..limits.read_chunk]) {
                Ok(0) => anyhow::bail!("native peer EOF ({:?})", decoder.eof_state()),
                Ok(n) => pending.extend_from_slice(&buffer[..n]),
                Err(error) if timeout(&error) => return Ok(None),
                Err(error) => return Err(error).context("native TCP read"),
            }
        }
        // An incomplete packet returns to cancellation/deadline checks after
        // each read. Continuous hostile fragments cannot starve those checks.
        active(cancel)?;
        if *cursor == 0 && !pending.is_empty() {
            let (used, body) = decoder.feed(pending)?;
            *cursor = used;
            return Ok(body);
        }
    }
}
fn wait_frame(
    stream: &TcpStream,
    decoder: &mut FrameDecoder,
    pending: &mut Vec<u8>,
    cursor: &mut usize,
    limits: Limits,
    deadline: Instant,
    cancel: &AtomicBool,
) -> Result<Vec<u8>> {
    loop {
        active(cancel)?;
        ensure!(
            Instant::now() < deadline,
            "native handshake deadline exceeded"
        );
        if let Some(body) = poll_frame(stream, decoder, pending, cursor, limits, cancel)? {
            active(cancel)?;
            ensure!(
                Instant::now() < deadline,
                "native handshake deadline exceeded"
            );
            return Ok(body);
        }
    }
}
fn write_frame(
    stream: &TcpStream,
    frame: &[u8],
    limits: Limits,
    deadline: Instant,
    cancel: &AtomicBool,
) -> Result<()> {
    let mut offset = 0;
    let mut writer = stream;
    while offset < frame.len() {
        active(cancel)?;
        ensure!(Instant::now() < deadline, "native write deadline exceeded");
        let end = (offset + limits.write_chunk).min(frame.len());
        match writer.write(&frame[offset..end]) {
            Ok(0) => anyhow::bail!("native zero-byte write"),
            Ok(n) => offset += n,
            Err(error) if timeout(&error) => {}
            Err(error) => return Err(error).context("native TCP write"),
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Disconnected,
    Connecting,
    VersionAccepted,
    Closing,
    Closed,
    Failed,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Snapshot {
    #[serde(serialize_with = "decimal_u64")]
    pub epoch: u64,
    pub phase: Phase,
    pub negotiated: Option<Negotiated>,
    pub error: Option<String>,
    pub game_timers: super::native_game_timers::Snapshot,
}
pub enum Event {
    Sent {
        epoch: u64,
        tx_counter: u64,
    },
    Negotiated {
        epoch: u64,
        value: Negotiated,
    },
    Packet {
        epoch: u64,
        body: Vec<u8>,
        rx_counter: u64,
        received_ms: u64,
    },
}
pub struct Controller {
    state: Snapshot,
    cancel: Arc<AtomicBool>,
    worker: Option<JoinHandle<std::result::Result<(), String>>>,
    events: Option<Receiver<Event>>,
    commands: Option<Arc<Mutex<Outgoing>>>,
    max_body: usize,
}
struct Batch {
    bodies: Vec<zeroize::Zeroizing<Vec<u8>>>,
    end_counter: u64,
    timer: Option<super::native_game_timers::Kind>,
}
#[derive(Default)]
struct Outgoing {
    queue: VecDeque<Batch>,
    manual: usize,
    queued_counter: Option<u64>,
    written_counter: Option<u64>,
    retiring: bool,
    timers: super::native_game_timers::Schedule,
}
impl Outgoing {
    fn timers_due(&mut self) -> Result<()> {
        for kind in self.timers.due(Instant::now()) {
            let end=self.queued_counter.context("native timer counter owner missing")?.wrapping_add(1);
            self.queue.push_back(Batch { bodies:vec![zeroize::Zeroizing::new(vec![kind.opcode()])],end_counter:end,timer:Some(kind) });
            self.queued_counter=Some(end);
        }
        Ok(())
    }
}
impl Default for Controller {
    fn default() -> Self {
        Self {
            state: Snapshot {
                epoch: 0,
                phase: Phase::Disconnected,
                negotiated: None,
                error: None,
                game_timers: Default::default(),
            },
            cancel: Arc::new(AtomicBool::new(false)),
            worker: None,
            events: None,
            commands: None,
            max_body: native_wire::MAX_BODY,
        }
    }
}
fn emit(tx: &SyncSender<Event>, cancel: &AtomicBool, mut event: Event) -> Result<()> {
    loop {
        active(cancel)?;
        match tx.try_send(event) {
            Ok(()) => return Ok(()),
            Err(mpsc::TrySendError::Full(pending)) => {
                event = pending;
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                anyhow::bail!("native event receiver closed")
            }
        }
    }
}
fn write_batch(session:&mut Session, queue:&Mutex<Outgoing>, batch:Batch, cancel:&AtomicBool)->Result<Option<u64>> {
    for body in batch.bodies { session.send_body(body,cancel)?; }
    ensure!(session.tx.counter()==batch.end_counter,"native outgoing reservation order changed");
    let mut q=queue.lock().map_err(|_|anyhow::anyhow!("native outgoing owner poisoned"))?;
    q.written_counter=Some(session.tx.counter());
    if let Some(kind)=batch.timer {q.timers.sent(kind,Instant::now());Ok(None)} else {Ok(Some(batch.end_counter))}
}
/// Under event backpressure, idle GAME timers can still write in order. Never
/// overtake an ordinary queued command or drop an authoritative inbound event.
fn emit_running(tx:&SyncSender<Event>,cancel:&AtomicBool,mut event:Event,session:&mut Session,queue:&Mutex<Outgoing>)->Result<()> {
    loop {
        active(cancel)?;
        match tx.try_send(event) {
            Ok(())=>return Ok(()),
            Err(mpsc::TrySendError::Full(pending))=>{
                event=pending;
                let batch={let mut q=queue.lock().map_err(|_|anyhow::anyhow!("native outgoing owner poisoned"))?;
                    q.timers_due()?;
                    if q.queue.front().is_some_and(|b|b.timer.is_some()){q.queue.pop_front()}else{None}};
                if let Some(batch)=batch {ensure!(write_batch(session,queue,batch,cancel)?.is_none(),"ordinary command during timer-only wait");}
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(mpsc::TrySendError::Disconnected(_))=>anyhow::bail!("native event receiver closed"),
        }
    }
}
impl Controller {
    pub fn send(&mut self, epoch: u64, body: zeroize::Zeroizing<Vec<u8>>) -> Result<()> {
        self.send_batch(epoch, vec![body])
    }
    /// Atomically enqueue at most two ordered frames; a partial socket write retires the owner.
    pub fn send_batch(
        &mut self,
        epoch: u64,
        bodies: Vec<zeroize::Zeroizing<Vec<u8>>>,
    ) -> Result<()> {
        self.enqueue(epoch,bodies,false).map(|_|())
    }
    /// The returned barrier belongs to this exact batch, even if an automatic
    /// callback reserves another byte before the caller observes the result.
    pub fn send_final(&mut self,epoch:u64,body:zeroize::Zeroizing<Vec<u8>>)->Result<u64>{
        self.enqueue(epoch,vec![body],true)
    }
    fn enqueue(&mut self,epoch:u64,bodies:Vec<zeroize::Zeroizing<Vec<u8>>>,final_batch:bool)->Result<u64>{
        ensure!(
            epoch == self.state.epoch
                && self.state.phase == Phase::VersionAccepted
                && !self.cancel.load(Ordering::Acquire),
            "native send owner is not current/ready"
        );
        ensure!(
            !bodies.is_empty()
                && bodies.len() <= 2
                && bodies
                    .iter()
                    .all(|body| !body.is_empty() && body.len() <= self.max_body),
            "native outgoing body bound"
        );
        let mut q=self.commands.as_ref().context("native send queue missing")?.lock().map_err(|_|anyhow::anyhow!("native outgoing owner poisoned"))?;
        ensure!(!q.retiring && q.manual<4,"native send queue full or retiring");
        let bytes=bodies.iter().map(|body|body.len() as u64).sum::<u64>();
        let next=q.queued_counter.context("native queued TX owner missing")?.wrapping_add(bytes);
        if final_batch {q.retiring=true;q.timers.stop();}
        q.queue.push_back(Batch{bodies,end_counter:next,timer:None});q.manual+=1;q.queued_counter=Some(next);
        Ok(next)
    }
    /// Called only after explicit GAME character admission. LOGIN/version
    /// negotiation alone never enables callbacks. Same-epoch calls are idempotent.
    pub fn start_game_timers(&mut self,epoch:u64)->Result<()> {
        ensure!(epoch==self.state.epoch && self.state.phase==Phase::VersionAccepted && !self.cancel.load(Ordering::Acquire),"native game timer owner is not current/ready");
        let mut q=self.commands.as_ref().context("native timer queue missing")?.lock().map_err(|_|anyhow::anyhow!("native outgoing owner poisoned"))?;
        ensure!(!q.retiring && q.queued_counter.is_some(),"native game timer owner retiring");
        q.timers.begin(Instant::now());q.timers_due()
    }
    /// Counter after every accepted queued body, including not-yet-written commands.
    pub fn queued_tx_counter(&self)->Option<u64>{self.commands.as_ref()?.lock().ok()?.queued_counter}
    pub fn game_timers(&self)->super::native_game_timers::Snapshot{self.commands.as_ref().and_then(|q|q.lock().ok().map(|q|q.timers.snapshot())).unwrap_or_default()}
    pub fn snapshot(&self) -> &Snapshot {
        &self.state
    }
    pub fn is_quiescent(&self) -> bool {
        self.worker.is_none()
    }
    pub fn start(&mut self, peer: SocketAddr, limits: Limits) -> Result<u64> {
        ensure!(
            self.is_quiescent(),
            "native worker still owns previous connection"
        );
        let limits = limits.validate()?;
        let epoch = self
            .state
            .epoch
            .checked_add(1)
            .context("native connection epoch exhausted")?;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (tx, rx) = mpsc::sync_channel(8);
        let commands = Arc::new(Mutex::new(Outgoing::default()));
        let command_rx=commands.clone();
        let worker = std::thread::Builder::new()
            .name("lito-native-network".into())
            .spawn(move || {
                (|| -> Result<()> {
                    let mut session = Session::connect(peer, limits, &worker_cancel)?;
                    {let mut q=command_rx.lock().map_err(|_|anyhow::anyhow!("native outgoing owner poisoned"))?;
                        q.queued_counter=Some(session.tx.counter());q.written_counter=q.queued_counter;}
                    emit(
                        &tx,
                        &worker_cancel,
                        Event::Negotiated {
                            epoch,
                            value: session.negotiated.clone(),
                        },
                    )?;
                    loop {
                        if worker_cancel.load(Ordering::Acquire) {
                            return Ok(());
                        }
                        for _ in 0..4 {
                            let batch={let mut q=command_rx.lock().map_err(|_|anyhow::anyhow!("native outgoing owner poisoned"))?;
                                q.timers_due()?;let batch=q.queue.pop_front();if batch.as_ref().is_some_and(|b|b.timer.is_none()){q.manual-=1;}batch};
                            match batch {
                                Some(batch)=>{
                                    if let Some(tx_counter)=write_batch(&mut session,&command_rx,batch,&worker_cancel)? {
                                        emit_running(&tx,&worker_cancel,Event::Sent{epoch,tx_counter},&mut session,&command_rx)?;
                                    }
                                }
                                None=>break,
                            }
                        }
                        if let Some(body) = session.poll_body(&worker_cancel)? {
                            emit_running(
                                &tx,
                                &worker_cancel,
                                Event::Packet {
                                    epoch,
                                    body,
                                    rx_counter: session.rx.counter(),
                                    received_ms: super::native_status4183::now_ms(),
                                },
                                &mut session,
                                &command_rx,
                            )?;
                        }
                    }
                })()
                .map_err(|error| format!("{error:#}"))
            })?;
        self.cancel = cancel;
        self.events = Some(rx);
        self.commands = Some(commands);
        self.max_body = limits.max_frame - 2;
        self.worker = Some(worker);
        self.state = Snapshot {
            epoch,
            phase: Phase::Connecting,
            negotiated: None,
            error: None,
            game_timers: Default::default(),
        };
        Ok(epoch)
    }
    /// Signal only. Normal UI close keeps polling until is_quiescent().
    pub fn cancel(&mut self) {
        if self.worker.is_some() {
            self.cancel.store(true, Ordering::Release);
            if let Some(q)=&self.commands {if let Ok(mut q)=q.lock(){q.timers.stop();q.retiring=true;}}
            self.state.phase = Phase::Closing;
        }
    }
    fn drain_events(&mut self, out: &mut Vec<Event>) {
        if let Some(rx) = &self.events {
            for _ in 0..16 {
                match rx.try_recv() {
                    Ok(event) => {
                        match &event {
                            Event::Sent { epoch, tx_counter } if *epoch == self.state.epoch => {
                                if let Some(info) = &mut self.state.negotiated {
                                    info.tx_counter = *tx_counter;
                                }
                            }
                            Event::Negotiated { epoch, value }
                                if *epoch == self.state.epoch
                                    && self.state.phase != Phase::Closing =>
                            {
                                self.state.negotiated = Some(value.clone());
                                self.state.phase = Phase::VersionAccepted;
                            }
                            Event::Packet {
                                epoch, rx_counter, ..
                            } if *epoch == self.state.epoch => {
                                if let Some(info) = &mut self.state.negotiated {
                                    info.rx_counter = *rx_counter;
                                }
                            }
                            _ => {}
                        }
                        if self.state.phase != Phase::Closing {
                            out.push(event);
                        }
                    }
                    Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
                }
            }
        }
    }
    pub fn poll(&mut self) -> Vec<Event> {
        let mut out = Vec::new();
        self.drain_events(&mut out);
        self.sync_outgoing();
        if self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.is_finished())
        {
            let result = self.worker.take().unwrap().join();
            // The worker may enqueue its final events after the first drain.
            // It is joined now, so the bounded queue cannot grow again.
            self.drain_events(&mut out);
            if let Some(q)=&self.commands {if let Ok(mut q)=q.lock(){q.timers.stop();q.retiring=true;}}
            self.sync_outgoing();
            self.events = None;
            self.commands = None;
            if self.state.phase == Phase::Closing {
                self.state.phase = Phase::Closed;
                self.state.error = None;
            } else {
                self.state.phase = Phase::Failed;
                self.state.error = Some(match result {
                    Ok(Err(error)) => error,
                    Ok(Ok(())) => "native worker ended".into(),
                    Err(_) => "native worker panic".into(),
                });
            }
        }
        out
    }
    fn sync_outgoing(&mut self) {
        if let Some(q)=&self.commands {if let Ok(q)=q.lock(){
            self.state.game_timers=q.timers.snapshot();
            if let (Some(info),Some(counter))=(&mut self.state.negotiated,q.written_counter){info.tx_counter=counter;}
        }}
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        // Last-resort owner cleanup, never detach. UI should first cancel/poll;
        // when connecting, OS connect_timeout bounds this fallback wait.
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Explicit read-only loopback diagnostic used by the same executable.
pub fn probe_loopback(peer: SocketAddr) -> Result<serde_json::Value> {
    ensure!(
        peer.ip().is_loopback() && peer.port() != 0,
        "native probe requires explicit loopback endpoint"
    );
    let mut controller = Controller::default();
    let limits = Limits {
        read_chunk: 2,
        write_chunk: 3,
        ..Limits::default()
    };
    controller.start(peer, limits)?;
    let deadline = Instant::now() + Duration::from_secs(9);
    loop {
        controller.poll();
        if controller.snapshot().phase == Phase::VersionAccepted {
            break;
        }
        if controller.is_quiescent() {
            anyhow::bail!(
                "{}",
                controller
                    .snapshot()
                    .error
                    .as_deref()
                    .unwrap_or("native probe ended")
            );
        }
        if Instant::now() >= deadline {
            controller.cancel();
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let accepted = controller.snapshot().clone();
    controller.cancel();
    while !controller.is_quiescent() {
        controller.poll();
        std::thread::sleep(Duration::from_millis(5));
    }
    ensure!(
        accepted.phase == Phase::VersionAccepted,
        "native probe timed out"
    );
    Ok(
        serde_json::json!({"schema":1,"status":"native_version_verified","pointer_bits":usize::BITS,
        "connection":accepted,"shutdown":controller.snapshot(),"worker_quiescent":true,
        "account_authenticated":false,"sql_access":false,"native_bin_started":false,"playable_game":false}),
    )
}
