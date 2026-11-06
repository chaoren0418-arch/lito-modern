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
        let listener=TcpListener::bind("127.0.0.1:0").unwrap(); listener.set_nonblocking(true).unwrap();
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

#[cfg_attr(test, test)]
pub fn skill5_portal_ack_is_once_and_cancel_retires_the_request() {
    let mut peer=FaultPeer::new();let mut c=seeded(&peer);
    c.teleport=None;c.state.merchant.clear();c.state.joined_room=None;
    c.state.phase=LoginPhase::WorldReady;
    let epoch=c.transport.snapshot().epoch;
    c.state.environment.begin(epoch).unwrap();
    c.state.world.learned_skills.insert(5);
    let map=c.state.world.map_id.unwrap();let own=c.state.world.own_actor().unwrap();
    let destination=super::super::native_portal::Destination{map,x:own.x+1,y:own.y};
    c.teleport_spell=Some((super::super::native_navigation::InputStamp::capture(epoch,&c.state.world),Instant::now(),map));
    let mut portal=vec![0x5a];portal.extend(map.to_le_bytes());portal.extend(destination.x.to_le_bytes());portal.extend(destination.y.to_le_bytes());
    peer.emit(vec![portal.clone()]);
    pump(&mut c,|c|c.state.world.portal.is_some());
    assert!(c.teleport_spell.is_none());assert_eq!(c.state.world.portal.unwrap().destination,destination);
    pump(&mut c,|_|peer.opcodes().contains(&8));
    let before=serde_json::to_value(&c.state).unwrap();
    assert!(c.accept_game(&portal).is_err());assert_eq!(serde_json::to_value(&c.state).unwrap(),before);
    assert_eq!(peer.opcodes().iter().filter(|&&op|op==8).count(),1);
    stop(&mut c);peer.close();assert!(c.teleport_spell.is_none());
}
#[cfg_attr(test, test)]
pub fn skill5_cancel_eof_rejection_and_timeout_never_send_portal_ack() {
    for case in 0..4 {
        let mut peer=FaultPeer::new();let mut c=seeded(&peer);
        c.teleport=None;c.state.merchant.clear();c.state.joined_room=None;
        c.state.phase=LoginPhase::WorldReady;c.state.world.learned_skills.insert(5);
        let epoch=c.transport.snapshot().epoch;let now=Instant::now();
        let since=if case==3 {now-Duration::from_secs(3)} else {now};
        c.teleport_spell=Some((super::super::native_navigation::InputStamp::capture(epoch,&c.state.world),since,c.state.world.map_id.unwrap()));
        c.skill_pending=Some(super::super::native_skills::Pending{since,
            casts:c.state.world.own_cast_packets,messages:c.state.world.message_count,
            generation:c.state.world.generation,incarnation:c.state.world.own_actor().unwrap().incarnation});
        c.state.skill_feedback=super::super::native_skills::Feedback::Pending;
        let position=c.state.world.own_actor().map(|a|(a.x,a.y));
        match case {
            0=>stop(&mut c),
            1=>{peer.write(vec![],None,true);pump(&mut c,|c|c.state.phase==LoginPhase::Failed&&c.is_quiescent());},
            2=>{peer.emit(vec![vec![0x10,0x14,1,0]]);pump(&mut c,|c|c.skill_pending.is_none());
                assert_eq!(c.state.skill_feedback,super::super::native_skills::Feedback::Rejected(276));},
            _=>{c.poll();assert_eq!(c.state.skill_feedback,super::super::native_skills::Feedback::NoResponse);},
        }
        assert!(c.teleport_spell.is_none());assert!(c.state.world.portal.is_none());
        if case>=2 {assert_eq!(c.state.world.own_actor().map(|a|(a.x,a.y)),position);}
        stop(&mut c);peer.close();assert!(!peer.opcodes().contains(&8));
    }
}
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
