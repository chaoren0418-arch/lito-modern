//! Synthetic encrypted loopback faults against the production controller.
//! Rental scope is seeded from pinned C02 captures, not an authenticated server.
use super::*;
use crate::modern_lito::{native_room, native_wire::{self, CipherState, FrameDecoder}};
use sha2::{Digest, Sha256};
use std::{io::{Read, Write}, net::TcpListener, path::Path,
    sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc}, thread};

const FIXTURE: &str = include_str!("../../tests/fixtures/native-teleport-controller-current.json");
const NPC: u32 = 1812987996;
fn fixture() -> serde_json::Value { serde_json::from_str(FIXTURE).unwrap() }
fn bytes(name: &str) -> Vec<u8> {
    let f = fixture(); let text = f["vectors"][name]["hex"].as_str().unwrap();
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i+2], 16).unwrap()).collect()
}
fn endpoint(peer: SocketAddr) -> Vec<u8> {
    let std::net::IpAddr::V4(ip) = peer.ip() else { panic!("IPv4 fixture") };
    let mut b=vec![0xf4,1]; b.extend(ip.octets()); b.extend(peer.port().to_le_bytes()); b
}
struct Emission { bodies: Vec<Vec<u8>>, raw: Option<Vec<u8>>, close: bool, done: mpsc::Sender<()> }
struct FaultPeer {
    peer: SocketAddr, commands: Arc<Mutex<Vec<Vec<u8>>>>, accepted: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>, tx: mpsc::Sender<Emission>, worker: Option<thread::JoinHandle<anyhow::Result<()>>>,
}
impl FaultPeer {
    fn new() -> Self {
        Self::at("127.0.0.1:0".parse().unwrap())
    }
    fn at(address: SocketAddr) -> Self {
        let listener=TcpListener::bind(address).unwrap(); listener.set_nonblocking(true).unwrap();
        let peer=listener.local_addr().unwrap(); let (tx, rx)=mpsc::channel::<Emission>();
        let commands=Arc::new(Mutex::new(Vec::new())); let log=commands.clone();
        let accepted=Arc::new(AtomicUsize::new(0)); let count=accepted.clone();
        let stop=Arc::new(AtomicBool::new(false)); let ending=stop.clone();
        let worker=thread::spawn(move || -> anyhow::Result<()> {
            let deadline=Instant::now()+Duration::from_secs(5);
            let mut socket=loop {
                if ending.load(Ordering::Acquire) { return Ok(()) }
                match listener.accept() {
                    Ok((s,_)) => { count.fetch_add(1,Ordering::AcqRel); break s }
                    Err(e) if e.kind()==std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(2)),
                    Err(e) => return Err(e.into()),
                }
                ensure!(Instant::now()<deadline,"synthetic peer accept deadline");
            };
            // Windows accepted sockets inherit listener nonblocking mode.
            socket.set_nonblocking(false)?;
            socket.set_nodelay(true)?; socket.set_read_timeout(Some(Duration::from_secs(2)))?;
            socket.set_write_timeout(Some(Duration::from_secs(2)))?;
            let seed=0xfffffffeu32; let mut seed_body=vec![1]; seed_body.extend(seed.to_le_bytes());
            socket.write_all(&native_wire::frame(&seed_body)?)?;
            let mut head=[0;2]; socket.read_exact(&mut head)?;
            let n=usize::from(u16::from_le_bytes(head)); ensure!((2..=65535).contains(&n),"version frame size");
            let mut version=vec![0;n-2]; socket.read_exact(&mut version)?;
            let mut incoming=CipherState::new(seed); incoming.decrypt(&mut version);
            ensure!(version==crate::modern_lito::native_version::version_request(0),"version request changed");
            let mut outgoing=CipherState::new(seed); let mut accepted=bytes("version"); outgoing.encrypt(&mut accepted);
            socket.write_all(&native_wire::frame(&accepted)?)?;
            socket.set_read_timeout(Some(Duration::from_millis(10)))?;
            let mut decoder=FrameDecoder::new(native_wire::MAX_FRAME)?;
            while !ending.load(Ordering::Acquire) {
                while let Ok(e)=rx.try_recv() {
                    let mut wire=Vec::new();
                    for mut body in e.bodies { outgoing.encrypt(&mut body); wire.extend(native_wire::frame(&body)?); }
                    if let Some(raw)=e.raw { wire.extend(raw); }
                    socket.write_all(&wire)?; let _=e.done.send(());
                    if e.close { return Ok(()) }
                }
                let mut buf=[0;512];
                match socket.read(&mut buf) {
                    Ok(0) => return Ok(()),
                    Ok(n) => {
                        let mut at=0;
                        while at<n {
                            let (used, body)=decoder.feed(&buf[at..n])?; at+=used;
                            if let Some(mut body)=body { incoming.decrypt(&mut body); log.lock().unwrap().push(body); }
                            ensure!(used>0,"synthetic frame decoder stalled");
                        }
                    }
                    Err(e) if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::TimedOut|std::io::ErrorKind::Interrupted) => {}
                    Err(e) if matches!(e.kind(),std::io::ErrorKind::ConnectionReset|std::io::ErrorKind::ConnectionAborted) => return Ok(()),
                    Err(e) => return Err(e.into()),
                }
            }
            Ok(())
        });
        Self { peer, commands, accepted, stop, tx, worker:Some(worker) }
    }
    fn emit(&self, bodies:Vec<Vec<u8>>) { self.write(bodies,None,false) }
    fn write(&self,bodies:Vec<Vec<u8>>,raw:Option<Vec<u8>>,close:bool) {
        let (done,rx)=mpsc::channel(); self.tx.send(Emission{bodies,raw,close,done}).unwrap();
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    fn opcodes(&self)->Vec<u8> { self.commands.lock().unwrap().iter().map(|b|b[0]).collect() }
    fn close(&mut self) { self.stop.store(true,Ordering::Release); if let Some(h)=self.worker.take(){h.join().unwrap().unwrap();} }
}

fn recovery_wait(peer: &FaultPeer) -> LoginController {
    let mut c = LoginController::default();
    c.start(peer.peer, Limits::default()).unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::ReadyForLogin);
    c.game = true; c.readiness_acked = true; c.state.game_connected = true;
    c.state.phase = LoginPhase::CharacterPending;
    c.state.selected_source = Some("測試乙".into());
    c.state.sources = vec![Source { name: "測試乙".into(), endpoint: peer.peer }];
    c.state.selected_character = Some("TestKnight".into());
    c.recovery_character = Some("TestKnight".into());
    c.state.inventory.begin(c.transport.snapshot().epoch, "TestKnight").unwrap();
    c.deadline = Some(Instant::now() - Duration::from_millis(1));
    c
}

fn game_packet(key: &str) -> Vec<u8> {
    let f: serde_json::Value = serde_json::from_str(include_str!("../../tests/fixtures/native-game-current.json")).unwrap();
    let hex = f["packets"][key].as_str().unwrap();
    (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i+2], 16).unwrap()).collect()
}

#[cfg_attr(test, test)]
pub fn recovery_c8_waits_for_complete_directory_and_never_replays_c9() {
    let mut peer = FaultPeer::new(); let mut c = recovery_wait(&peer);
    let epoch = c.transport.snapshot().epoch;
    c.poll();
    assert_eq!(c.state.phase, LoginPhase::LoadingCharacters);
    assert!(c.state.characters.is_empty() && c.state.selected_character.is_none());
    assert!(c.recovery_character.is_none() && c.state.inventory.owner.is_none());
    assert!(c.select_character(0).is_err());
    pump(&mut c, |_| peer.opcodes().contains(&0xc8));
    assert_eq!(*peer.commands.lock().unwrap(), vec![source_request("測試乙").unwrap().to_vec()]);
    let deadline = c.deadline.unwrap();
    assert!(deadline > Instant::now() + Duration::from_secs(7));
    assert!(deadline <= Instant::now() + Duration::from_secs(8));
    let received = c.transport.snapshot().negotiated.as_ref().unwrap().rx_counter;
    peer.emit(vec![vec![3, 1]]);
    pump(&mut c, |c| c.transport.snapshot().negotiated.as_ref().unwrap().rx_counter != received);
    assert!(c.select_character(0).is_err());
    assert_eq!(c.deadline, Some(deadline));
    peer.emit(vec![game_packet("row")]);
    pump(&mut c, |c| c.state.phase == LoginPhase::CharactersReady);
    assert_eq!(c.state.characters[0].name, "重建角色");
    assert_eq!(c.state.directory_generation, 1);
    assert_eq!(c.transport.snapshot().epoch, epoch);
    assert_eq!(peer.opcodes(), vec![0xc8]);
    c.select_character(0).unwrap();
    pump(&mut c, |_| peer.opcodes().contains(&0xc9));
    assert_eq!(*peer.commands.lock().unwrap(), vec![source_request("測試乙").unwrap().to_vec(), game_packet("select")]);
    stop(&mut c); peer.close();
}

#[cfg_attr(test, test)]
pub fn recovery_missing_character_refreshes_without_publishing_default_source() {
    let mut peer = FaultPeer::new(); let mut c = recovery_wait(&peer);
    c.state.phase = LoginPhase::LoadingCharacters;
    c.state.selected_character = None; c.state.inventory.clear();
    c.deadline = Some(Instant::now() + Duration::from_secs(8));
    peer.emit(vec![vec![3, 0]]);
    pump(&mut c, |_| peer.opcodes().contains(&0xc8));
    assert_eq!(c.state.phase, LoginPhase::LoadingCharacters);
    assert_eq!(c.state.directory_generation, 0);
    assert!(c.state.characters.is_empty() && !c.directory.complete());
    assert!(c.select_character(0).is_err());
    peer.emit(vec![vec![3, 0]]);
    pump(&mut c, |c| c.state.phase == LoginPhase::CharactersReady);
    assert_eq!(c.state.directory_generation, 1);
    assert_eq!(peer.opcodes(), vec![0xc8]);
    stop(&mut c); peer.close();
}

#[cfg_attr(test, test)]
pub fn recovery_c8_timeout_is_bounded_and_cancel_cannot_restart_it() {
    for cancel in [false, true] {
        let mut peer = FaultPeer::new(); let mut c = recovery_wait(&peer);
        c.poll(); pump(&mut c, |_| peer.opcodes().contains(&0xc8));
        if cancel { c.cancel(); c.cancel(); }
        else { c.deadline = Some(Instant::now() - Duration::from_millis(1)); }
        pump(&mut c, |c| c.is_quiescent());
        assert_eq!(c.state.phase, if cancel { LoginPhase::Closed } else { LoginPhase::Failed });
        retired(&c);
        assert!(c.teleport_recovery.is_none() && c.recovery_character.is_none());
        assert_eq!(c.transport.snapshot().epoch, 1);
        assert_eq!(peer.opcodes(), vec![0xc8]);
        peer.close();
    }
}

#[cfg_attr(test, test)]
pub fn recovery_partial_inventory_fails_without_c8_retry() {
    let mut peer = FaultPeer::new(); let mut c = recovery_wait(&peer);
    c.state.inventory.batches = 1;
    pump(&mut c, |c| c.state.phase == LoginPhase::Failed && c.is_quiescent());
    retired(&c); peer.close();
    assert!(peer.opcodes().is_empty());
}

#[cfg_attr(test, test)]
pub fn recovery_refresh_is_not_available_to_a_direct_teleport() {
    let mut old = FaultPeer::new(); let mut target = FaultPeer::new();
    let mut c = seeded(&old);
    old.emit(vec![bytes("availability"), endpoint(target.peer)]);
    pump(&mut c, |c| c.state.phase == LoginPhase::GameLoginPending);
    target.emit(vec![game_packet("ready"), vec![3, 0]]);
    pump(&mut c, |c| c.state.phase == LoginPhase::Failed && c.is_quiescent());
    retired(&c); old.close(); target.close();
    assert!(c.state.error.as_deref().unwrap().contains("teleport character absent"));
    assert!(!target.opcodes().contains(&0xc8) && !target.opcodes().contains(&0xc9));
}

#[cfg_attr(test, test)]
pub fn recovery_natural_eight_second_waits_do_not_publish_or_retry() {
    let mut peer = FaultPeer::new(); let mut c = recovery_wait(&peer);
    let start = Instant::now();
    c.deadline = Some(start + Duration::from_secs(8));
    while c.state.phase == LoginPhase::CharacterPending {
        c.poll();
        assert!(start.elapsed() < Duration::from_secs(12), "continuation exceeded bound");
        if start.elapsed() < Duration::from_secs(8) {
            assert_eq!(c.state.phase, LoginPhase::CharacterPending);
            assert!(peer.opcodes().is_empty());
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(start.elapsed() >= Duration::from_secs(8));
    assert_eq!(c.state.phase, LoginPhase::LoadingCharacters);
    let confirmation = Instant::now();
    while c.state.phase == LoginPhase::LoadingCharacters || !c.is_quiescent() {
        c.poll();
        assert!(confirmation.elapsed() < Duration::from_secs(12), "C8 exceeded bound");
        thread::sleep(Duration::from_millis(5));
    }
    assert!(start.elapsed() >= Duration::from_secs(16));
    assert_eq!(c.state.phase, LoginPhase::Failed);
    retired(&c); peer.close();
    assert_eq!(peer.opcodes(), vec![0xc8]);
}

#[cfg_attr(test, test)]
pub fn recovery_after_second_cancel_uses_explicit_login_and_source_once() {
    let mut old = FaultPeer::new(); let mut target = FaultPeer::new();
    let mut c = seeded(&old);
    c.teleport.as_mut().unwrap().character = "重建角色".into();
    old.emit(vec![bytes("availability"), endpoint(target.peer)]);
    pump(&mut c, |c| c.transport.snapshot().epoch == 2 && c.state.phase == LoginPhase::GameLoginPending);
    stop(&mut c); c.cancel(); c.poll(); retired(&c);
    old.close(); target.close();
    assert!(c.teleport_recovery.is_some());
    let mut login = FaultPeer::at(old.peer);
    let mut game = FaultPeer::new();
    c.start(login.peer, Limits::default()).unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::ReadyForLogin);
    assert!(login.opcodes().is_empty(), "cancel must not auto-login");
    c.login("SyntheticAccount", "SyntheticOnly").unwrap();
    let mut sources = vec![0x3b, 1];
    sources.extend(crate::modern_lito::native_login::encode_text("測試乙", 63).unwrap().iter());
    sources.push(0); sources.extend([127, 0, 0, 1]); sources.extend(game.peer.port().to_le_bytes());
    login.emit(vec![vec![0xe7], sources]);
    pump(&mut c, |c| c.state.phase == LoginPhase::SourcesReady);
    c.select_source(0).unwrap();
    assert!(c.teleport_recovery.is_none());
    login.emit(vec![endpoint(game.peer)]);
    pump(&mut c, |c| c.state.phase == LoginPhase::RedirectReady);
    c.begin_game().unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::GameLoginPending);
    game.emit(vec![game_packet("ready"), vec![3, 1], game_packet("row"), vec![0xc8, 0], bytes("inventory")]);
    pump(&mut c, |c| c.state.phase == LoginPhase::CharacterSelected && c.state.inventory.batches > 0);
    assert_eq!(c.state.selected_character.as_deref(), Some("重建角色"));
    assert!(c.recovery_character.is_none());
    stop(&mut c); login.close(); game.close();
    assert_eq!(login.opcodes(), vec![1, 0xc8]);
    assert!(!game.opcodes().contains(&0xc8) && !game.opcodes().contains(&0xc9));
    assert!(c.teleport_recovery.is_none());
}
impl Drop for FaultPeer {
    fn drop(&mut self) { self.stop.store(true,Ordering::Release); if let Some(h)=self.worker.take(){if let Ok(Err(error))=h.join(){eprintln!("synthetic peer {} failed: {error:#}",self.peer);}} }
}
fn pump(c:&mut LoginController, ok:impl Fn(&LoginController)->bool) {
    let until=Instant::now()+Duration::from_secs(3);
    loop { c.poll(); if ok(c){return} assert!(Instant::now()<until,"controller {:?}: {:?}, epoch={}, game={}",c.state.phase,c.state.error,c.transport.snapshot().epoch,c.game); thread::sleep(Duration::from_millis(2)); }
}
fn seeded(peer:&FaultPeer)->LoginController {
    let mut c=LoginController::default();
    c.start(peer.peer,Limits{io_slice:Duration::from_millis(5),write_chunk:3,..Limits::default()}).unwrap();
    pump(&mut c,|c|c.state.phase==LoginPhase::ReadyForLogin);
    c.game=true; c.state.phase=LoginPhase::BattlePending; c.state.game_connected=true;
    c.state.selected_character=Some("TestKnight".into()); c.state.selected_source=Some("測試乙".into());
    c.credentials=Some((Zeroizing::new("SyntheticAccount".into()),Zeroizing::new("SyntheticOnly".into())));
    for key in ["world","start","status","own_actor","npc","hp"] { c.state.world.accept(&bytes(key)).unwrap(); }
    let f=fixture(); let a=c.state.world.actors.get_mut(&1).unwrap();
    a.x=f["own_position"][0].as_u64().unwrap() as u16; a.y=f["own_position"][1].as_u64().unwrap() as u16;
    let native_room::Packet::Joined(room)=native_room::parse(&bytes("joined")).unwrap() else {panic!("joined fixture")};
    c.state.joined_room=Some(room); c.state.rental_set=Some(1); c.state.shop_points=Some(5000);
    c.state.inventory.begin(c.transport.snapshot().epoch,"TestKnight").unwrap();
    c.state.inventory.accept(c.transport.snapshot().epoch,&bytes("inventory")).unwrap();
    let owner=c.merchant_owner().unwrap(); c.state.merchant.begin(owner,NPC,Instant::now()).unwrap();
    c.state.merchant.accept(crate::modern_lito::native_merchant::parse(&bytes("dialog")).unwrap(),Instant::now()).unwrap();
    c.rental_npc_action(NPC,"LT_TELEPORT_B").unwrap();
    pump(&mut c,|_|peer.opcodes().contains(&0x27)); c
}
fn retired(c:&LoginController) {
    assert!(!c.state.world.ready() && c.state.world.actors.is_empty());
    assert!(c.state.inventory.owner.is_none() && c.state.inventory.items.is_empty());
    assert!(c.state.merchant.owner.is_none() && c.state.joined_room.is_none());
    assert!(c.state.selected_character.is_none() && c.state.services.is_none());
}
fn stop(c:&mut LoginController) { c.cancel(); pump(c,|c|c.is_quiescent()); }
fn record(name:&str,c:&LoginController,peers:&[&FaultPeer],extra:serde_json::Value) {
    let profile=if cfg!(debug_assertions){"Debug"}else{"Release"};
    let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-teleport-wire").join(profile);
    std::fs::create_dir_all(&path).unwrap();
    let report=serde_json::json!({"case":name,"phase":c.state.phase,"epoch":c.transport.snapshot().epoch,"error":c.state.error,
        "quiescent":c.is_quiescent(),"snapshot":c.state,"peer_connection_counts":peers.iter().map(|p|p.accepted.load(Ordering::Acquire)).collect::<Vec<_>>(),
        "client_command_opcodes":peers.iter().map(|p|p.opcodes()).collect::<Vec<_>>(),"fixture_sha256":format!("{:X}",Sha256::digest(FIXTURE.as_bytes())),
        "synthetic_encrypted_loopback":true,"authenticated_formal_sessions":0,"seeded_scope_from_C02":true,"extra":extra});
    std::fs::write(path.join(format!("{name}.json")),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}

#[cfg_attr(test, test)]
pub fn retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier() {
    let mut old=FaultPeer::new(); let mut target=FaultPeer::new(); let mut c=seeded(&old);
    c.accept_game(&bytes("availability")).unwrap();
    let destination=endpoint(target.peer); c.accept_game(&destination).unwrap();
    assert_eq!(c.state.phase,LoginPhase::TeleportQuitPending);
    let before=serde_json::to_value(&c.state).unwrap(); let barrier=c.transport.queued_tx_counter();
    for body in [destination.clone(),endpoint(old.peer),bytes("availability"),bytes("world"),bytes("inventory"),bytes("own_actor")] {
        assert!(c.accept_game(&body).is_ok(),"retired packet {:02X} rejected before quit write barrier",body[0]);
        assert_eq!(serde_json::to_value(&c.state).unwrap(),before);
        assert_eq!(c.transport.queued_tx_counter(),barrier);
    }
    pump(&mut c,|c|c.transport.snapshot().epoch==2 && c.state.phase==LoginPhase::GameLoginPending);
    retired(&c); assert_eq!(target.accepted.load(Ordering::Acquire),1);
    assert_eq!(c.state.redirect,Some(target.peer)); stop(&mut c); old.close();target.close();
    assert_eq!(old.opcodes().iter().filter(|&&op|op==0xd9).count(),1);
    record("retired-duplicate-burst",&c,&[&old,&target],serde_json::json!({"quit_requests":1,"old_bodies_ignored":6}));
}

#[cfg_attr(test, test)]
pub fn coalesced_old_socket_burst_cannot_retarget_or_repeat_quit() {
    let mut old=FaultPeer::new(); let mut target=FaultPeer::new(); let mut c=seeded(&old);
    old.emit(vec![bytes("availability"),endpoint(target.peer),endpoint(target.peer),endpoint(old.peer),
        bytes("availability"),bytes("world"),bytes("inventory"),bytes("own_actor")]);
    // Let the bounded receive queue collect the coalesced old-socket burst.
    // The deterministic direct-dispatch case separately forces the pre-Sent boundary.
    thread::sleep(Duration::from_millis(80));
    pump(&mut c,|c|c.transport.snapshot().epoch==2&&c.state.phase==LoginPhase::GameLoginPending);
    retired(&c); assert_eq!(c.state.redirect,Some(target.peer));
    pump(&mut c,|_|target.opcodes().iter().filter(|&&op|op==1).count()==1);
    stop(&mut c);old.close();target.close();
    assert_eq!(old.opcodes().iter().filter(|&&op|op==0xd9).count(),1);
    assert_eq!(target.accepted.load(Ordering::Acquire),1);
    assert_eq!(target.opcodes().iter().filter(|&&op|op==1).count(),1);
    record("coalesced-old-socket-burst",&c,&[&old,&target],serde_json::json!({"emitted_bodies":8,"quit_requests":1,"target_connections":1}));
}

#[cfg_attr(test, test)]
pub fn malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner() {
    let status=bytes("availability"); let good=endpoint("127.0.0.1:1".parse().unwrap()); let mut cases=Vec::new();
    for n in 0..status.len(){cases.push((format!("status-truncated-{n}"),vec![status[..n].to_vec()]));}
    for n in 0..good.len(){cases.push((format!("endpoint-truncated-{n}"),vec![status.clone(),good[..n].to_vec()]));}
    let mut trailing=status.clone();trailing.push(0);cases.push(("status-trailing".into(),vec![trailing]));
    let mut word=status.clone();word[6..10].copy_from_slice(&1u32.to_le_bytes());cases.push(("status-unknown-word".into(),vec![word]));
    cases.push(("status-duplicate".into(),vec![status.clone(),status.clone()]));
    cases.push(("endpoint-before-status".into(),vec![good.clone()]));
    let mut index=good.clone();index[1]=2;cases.push(("endpoint-wrong-slot".into(),vec![status.clone(),index]));
    let mut invalid=good.clone();invalid[1]=10;cases.push(("endpoint-invalid-index".into(),vec![status.clone(),invalid]));
    let mut zero=good.clone();zero[6..8].fill(0);cases.push(("endpoint-zero-port".into(),vec![status.clone(),zero]));
    let mut trailing=good;trailing.push(0);cases.push(("endpoint-trailing".into(),vec![status,trailing]));
    for (name,bodies) in cases {
        let mut peer=FaultPeer::new();let mut c=seeded(&peer);let epoch=c.transport.snapshot().epoch;let queued=c.transport.queued_tx_counter();
        assert!(queued.is_some(),"{name}: active TX owner missing before malformed reply");
        peer.emit(bodies);pump(&mut c,|c|c.state.phase==LoginPhase::Failed&&c.is_quiescent());retired(&c);
        assert_eq!(c.transport.snapshot().epoch,epoch,"{name}: malformed reply started another epoch");
        assert_eq!(c.transport.queued_tx_counter(),None,"{name}: retired TX queue still owned");
        assert_eq!(c.transport.snapshot().negotiated.as_ref().map(|n|n.tx_counter),queued,
            "{name}: malformed reply changed the final written TX counter");
        assert!(c.credentials.is_none()&&c.state.redirect.is_none());peer.close();
        assert_eq!(*peer.commands.lock().unwrap(),vec![crate::modern_lito::native_merchant::action(NPC,"LT_TELEPORT_B").unwrap()],
            "{name}: unexpected body sent after malformed reply");
        assert!(!peer.opcodes().contains(&0xd9));record(&name,&c,&[&peer],serde_json::json!({"quit_requests":0,"new_epoch_started":false,
            "retired_TX_queue_absent":true,"expected_written_TX_counter":queued,
            "final_written_TX_counter":c.transport.snapshot().negotiated.as_ref().map(|n|n.tx_counter),"exact_original_request_only":true}));
    }
}

#[cfg_attr(test, test)]
pub fn new_destination_does_not_accept_previous_endpoint_as_a_new_handoff() {
    let mut old=FaultPeer::new();let mut target=FaultPeer::new();let mut c=seeded(&old);
    old.emit(vec![bytes("availability"),endpoint(target.peer)]);
    pump(&mut c,|c|c.transport.snapshot().epoch==2&&c.state.phase==LoginPhase::GameLoginPending);
    retired(&c);target.emit(vec![endpoint(old.peer)]);
    pump(&mut c,|c|c.state.phase==LoginPhase::Failed&&c.is_quiescent());retired(&c);
    assert_eq!(c.transport.snapshot().epoch,2);old.close();target.close();
    assert_eq!(old.opcodes().iter().filter(|&&op|op==0xd9).count(),1);assert!(!target.opcodes().contains(&0xd9));
    record("new-epoch-replayed-endpoint",&c,&[&old,&target],serde_json::json!({"destination_quit_requests":0,"new_epoch_started_again":false}));
}

#[cfg_attr(test, test)]
pub fn cancel_discards_old_inbound_burst_before_fresh_login() {
    let mut old=FaultPeer::new();let mut fresh=FaultPeer::new();let mut c=seeded(&old);
    old.emit((0..32).map(|_|endpoint(fresh.peer)).collect());stop(&mut c);retired(&c);old.close();
    c.start(fresh.peer,Limits::default()).unwrap();pump(&mut c,|c|c.state.phase==LoginPhase::ReadyForLogin);
    for _ in 0..24 {c.poll();assert_eq!(c.state.phase,LoginPhase::ReadyForLogin);assert!(c.state.redirect.is_none());retired(&c);thread::sleep(Duration::from_millis(2));}
    assert_eq!(c.transport.snapshot().epoch,2);stop(&mut c);fresh.close();
    assert!(!old.opcodes().contains(&0xd9));assert!(fresh.opcodes().is_empty());
    record("cancel-old-burst-fresh-login",&c,&[&old,&fresh],serde_json::json!({"old_body_burst":32,"fresh_samples":24}));
}

#[cfg_attr(test, test)]
pub fn invalid_transport_envelopes_during_teleport_retire_scope() {
    for (name,raw) in [("frame-zero",vec![0,0]),("frame-one",vec![1,0]),("frame-partial",vec![8,0,0,0])] {
        let mut peer=FaultPeer::new();let mut c=seeded(&peer);
        peer.write(vec![],Some(raw),true);pump(&mut c,|c|c.state.phase==LoginPhase::Failed&&c.is_quiescent());
        retired(&c);assert_eq!(c.transport.snapshot().epoch,1);peer.close();
        assert!(!peer.opcodes().contains(&0xd9));record(name,&c,&[&peer],serde_json::json!({"transport_envelope_rejected":true}));
    }
}
