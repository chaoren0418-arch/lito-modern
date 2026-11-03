use lito_modern::modern_lito::{
    native_city_runtime::{
        login_events::FixtureProfile, login_form::Stage, player_entry_commands::Command, Runtime,
        UiPage, entry_navigation::{Scene,decode}, source_send::WriteOutcome,
    },
    native_multisector_resources::Inputs,
    native_resources::BytePin,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
fn bytes(v: &Value) -> Vec<u8> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_u64().unwrap() as u8)
        .collect()
}
fn inputs(p: &Path, name: &str) -> Inputs {
    let manifest = p.join("logs/continuation-20261003").join(name);
    let b = fs::read(&manifest).unwrap();
    Inputs {
        project: p.into(),
        manifest,
        manifest_pin: BytePin {
            bytes: b.len() as u64,
            sha256: Sha256::digest(b).into(),
        },
    }
}
fn ready(p: &Path) -> anyhow::Result<Runtime> {
    let mut rt = Runtime::start_with_menu_skin(
        inputs(p, "r486-map-binding.json"),
        inputs(p, "r498-native-menu-skin-binding.json"),
    )?;
    let end = Instant::now() + Duration::from_secs(60);
    while rt.page() != UiPage::Menu {
        assert!(Instant::now() < end);
        rt.poll();
        thread::sleep(Duration::from_millis(2));
    }
    Ok(rt)
}
#[derive(Default)]
struct Metrics {
    calls: usize,
    max_us: u128,
    partial: BTreeSet<usize>,
}
fn tick(r: &mut Runtime, m: &mut Metrics) {
    let at = Instant::now();
    r.poll();
    m.calls += 1;
    m.max_us = m.max_us.max(at.elapsed().as_micros());
    if let Some((_, n, offset)) = r.source_send_snapshot().pending {
        if offset > 0 && offset < n {
            m.partial.insert(offset);
        }
    }
    assert!(r.player_fixture_snapshot().queued_events <= 4);
}
fn until(r: &mut Runtime, m: &mut Metrics, goal: impl Fn(&Runtime) -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !goal(r) {
        assert!(
            Instant::now() < end,
            "bounded new fixture wait expired; page={:?},stage={:?},status={}",
            r.page(),
            r.source_login_snapshot().stage,
            r.status()
        );
        tick(r, m);
        thread::sleep(Duration::from_millis(2));
    }
}
fn pause_until(mut goal: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(5);
    while !goal() {
        assert!(Instant::now() < end);
        thread::sleep(Duration::from_millis(2));
    }
}
fn profile(
    endpoint: SocketAddr,
    special: bool,
    read_cap: usize,
    write_cap: usize,
    delay: u64,
) -> Value {
    json!({"kind":"modern_synthetic_loopback_fixture_v1","label":if special{"SYNTHETIC:R503_SPECIAL"}else{"SYNTHETIC:R503_NORMAL"},"endpoint":endpoint.to_string(),"code_page":950,"from_cd_registry_dword":0x123456A5u32,"revision_word_source":0x112345u32,"supplemental":if special{Some("FIXTURE-EXTRA")}else{None},"read_cap":read_cap,"write_cap":write_cap,"connect_delay_ms":delay})
}
fn configure(r: &mut Runtime, v: &Value) -> anyhow::Result<()> {
    r.configure_synthetic_player_fixture(FixtureProfile::parse(&serde_json::to_vec(v)?)?)
}
fn edit_submit(r: &mut Runtime, a: &[u8], p: &[u8]) -> anyhow::Result<()> {
    let c = r.source_login_control().unwrap();
    r.handle_player_entry_command(Command::EditLogin {
        control: &c,
        account: a,
        password: p,
    })?;
    let c = r.source_login_control().unwrap();
    r.handle_player_entry_command(Command::SubmitLogin(&c))
}
fn expect(s: &mut TcpStream, v: &Value) {
    let mut h = [0; 2];
    s.read_exact(&mut h).unwrap();
    let n = u16::from_le_bytes(h) as usize;
    assert!((3..=157).contains(&n));
    let mut raw = h.to_vec();
    raw.resize(n, 0);
    s.read_exact(&mut raw[2..]).unwrap();
    assert_eq!(raw, bytes(&v["wire"]));
}
fn wiped(r: &Runtime) {
    let s = r.source_login_snapshot();
    assert_eq!(
        (
            s.account_len,
            s.password_len,
            s.staged_account_len,
            s.staged_password_len
        ),
        (0, 0, 0, 0)
    );
}
fn closed(r: &mut Runtime, m: &mut Metrics) -> anyhow::Result<()> {
    let at = Instant::now();
    r.handle_player_entry_command(Command::Close)?;
    assert!(at.elapsed() < Duration::from_millis(200));
    wiped(r);
    assert!(r.source_connection_epoch().is_none());
    until(r, m, |r| r.page() == UiPage::Closed);
    let s = r.player_fixture_snapshot();
    assert_eq!(s.workers_started, s.workers_joined);
    assert_eq!(s.retiring_workers, 0);
    assert!(!s.active);
    Ok(())
}
fn snapshot(r: &Runtime, m: &Metrics) -> Value {
    let s = r.player_fixture_snapshot();
    json!({"poll_calls":m.calls,"max_poll_us":m.max_us,"partial_offsets":m.partial,"workers_started":s.workers_started,"workers_joined":s.workers_joined,"late_events_discarded":s.discarded_late_events,"max_event_queue":s.max_queued_events,"max_receiver_tail":s.max_receiver_tail_bytes,"actual_read_events":s.read_events,"actual_write_reports":s.write_reports,"actual_socket_read_bytes":s.bytes_read})
}
fn record(base: &Path, id: &str, name: &str, details: Value, trace: &mut Vec<Value>) {
    trace.push(json!({"case":id,"name":name,"passed":true,"details":details}));
    fs::write(
        base.join("r503-event-partial.json"),
        serde_json::to_vec_pretty(trace).unwrap(),
    )
    .unwrap();
}
enum Step { Send(Vec<u8>), Expect(Value), Eof, Finish }
struct Endpoint { steps:mpsc::Sender<Step>, done:mpsc::Receiver<()>, thread:thread::JoinHandle<usize> }
fn endpoint(r:&mut Runtime,o:&Value,read_cap:usize,write_cap:usize)->anyhow::Result<Endpoint> {
 let listener=TcpListener::bind("127.0.0.1:0")?;let addr=listener.local_addr()?;
 let setup=o["setup"].clone();let (steps,rx)=mpsc::channel();let (done_tx,done)=mpsc::channel();
 let server=thread::spawn(move||{
  let(mut s,peer)=listener.accept().unwrap();assert!(peer.ip().is_loopback());s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();s.set_nodelay(true).unwrap();
  s.write_all(&bytes(&setup["coalesced_handshake"])).unwrap();
  expect(&mut s,&setup["send_vectors"][0]);expect(&mut s,&setup["send_vectors"][1]);
  s.write_all(&bytes(&setup["first_roster"]["wire"])).unwrap();let mut frames=2;
  loop {match rx.recv_timeout(Duration::from_secs(10)).unwrap(){
   Step::Send(b)=>s.write_all(&b).unwrap(),
   Step::Expect(v)=>{expect(&mut s,&v);frames+=1;},
   Step::Eof=>{s.shutdown(Shutdown::Write).unwrap();},
   Step::Finish=>{let mut b=[0];assert_eq!(s.read(&mut b).unwrap(),0);break;},
  }done_tx.send(()).unwrap();}frames
 });
 configure(r,&profile(addr,false,read_cap,write_cap,0))?;r.handle_player_entry_command(Command::OpenLogin)?;edit_submit(r,b"fixtureA",b"dummyPass1")?;
 Ok(Endpoint{steps,done,thread:server})
}
fn sent(e:&Endpoint){e.done.recv_timeout(Duration::from_secs(10)).unwrap();}
fn finish(e:Endpoint)->usize {e.steps.send(Step::Finish).unwrap();e.thread.join().unwrap()}
fn choose(r:&mut Runtime,slot:usize)->anyhow::Result<()> {let c=r.source_character_control(slot).unwrap();r.handle_player_entry_command(Command::ChooseCharacter{control:&c,double_click:false})}
fn hold(r:&mut Runtime,m:&mut Metrics){for _ in 0..8{tick(r,m);thread::sleep(Duration::from_millis(2));}}
fn main()->anyhow::Result<()> {
 let at=Instant::now();let p=PathBuf::from(std::env::args_os().nth(1).unwrap());let base=p.join("logs/continuation-20261003");let o:Value=serde_json::from_slice(&fs::read(base.join("r503-source-oracle.json"))?)?;
 let mut trace=Vec::new();let mut frames=0;
 //New refusal/reselection/typed success sequence through the shared Window command path.
 let mut r=ready(&p)?;let mut m=Metrics::default();let e=endpoint(&mut r,&o,1,1)?;
 until(&mut r,&mut m,|r|r.source_character_control(0).is_some());let epoch=r.source_connection_epoch();let roster=r.presentation_key().roster_revision;
 choose(&mut r,0)?;let old_control=r.source_character_control(0).unwrap();let old_back=r.source_character_back().unwrap();let submit=r.source_character_submit().unwrap();
 e.steps.send(Step::Expect(o["normal_tx"][0].clone()))?;r.handle_player_entry_command(Command::SubmitCharacter(&submit))?;
 assert_eq!(r.source_navigation_snapshot().scene,Scene::AwaitCharacterReply);assert!(!r.presentation_key().character_ui.active);
 assert!(r.handle_player_entry_command(Command::SubmitCharacter(&submit)).is_err());assert!(r.handle_player_entry_command(Command::BackCharacter(&old_back)).is_err());assert!(r.handle_player_entry_command(Command::ChooseCharacter{control:&old_control,double_click:false}).is_err());
 let counter=r.source_send_snapshot().counter;until(&mut r,&mut m,|r|r.source_navigation_snapshot().write_complete);sent(&e);
 assert_eq!(r.source_flow().phase(),0);assert_eq!(r.source_send_snapshot().counter,counter);assert_eq!(r.source_navigation_snapshot().scene,Scene::AwaitCharacterReply);
 record(&base,"write_wait","write completion locks character tokens and does not enter lobby or fabricate channel",snapshot(&r,&m),&mut trace);
 //Partial success/refusal framing preserves scene until full decoded reply.
 let wire=bytes(&o["normal_rx"][0]["wire"]);let rx=r.source_stream_snapshot().counter;e.steps.send(Step::Send(wire[..1].to_vec()))?;sent(&e);
 until(&mut r,&mut m,|r|r.source_stream_snapshot().header_bytes==1);assert_eq!(r.source_stream_snapshot().counter,rx);assert_eq!(r.source_navigation_snapshot().scene,Scene::AwaitCharacterReply);
 e.steps.send(Step::Send(wire[1..].to_vec()))?;sent(&e);until(&mut r,&mut m,|r|r.source_character_control(0).is_some());
 assert!(r.source_identity().selected().is_none());assert!(r.handle_player_entry_command(Command::SubmitCharacter(&submit)).is_err());assert!(r.handle_player_entry_command(Command::BackCharacter(&old_back)).is_err());
 assert_eq!(r.source_connection_epoch(),epoch);assert_eq!(r.presentation_key().roster_revision,roster);
 record(&base,"refusal_retry","actual S_C9/1 refusal restores new character scene revision without cipher/epoch/roster reset",snapshot(&r,&m),&mut trace);
 //Previous-scene C8 replies are ignored while no request exists; cipher still advances.
 e.steps.send(Step::Send(bytes(&o["normal_rx"][1]["wire"])))?;sent(&e);until(&mut r,&mut m,|r|r.source_navigation_snapshot().ignored_replies==1);
 record(&base,"stale_scene_reply","previous S_C8/8 while character scene idle is ignored without transitions",snapshot(&r,&m),&mut trace);
 choose(&mut r,1)?;let second=r.source_character_submit().unwrap();e.steps.send(Step::Expect(o["normal_tx"][1].clone()))?;r.handle_player_entry_command(Command::SubmitCharacter(&second))?;until(&mut r,&mut m,|r|r.source_navigation_snapshot().write_complete);sent(&e);
 e.steps.send(Step::Send(bytes(&o["normal_rx"][2]["wire"])))?;sent(&e);until(&mut r,&mut m,|r|r.source_character_control(2).is_some());
 record(&base,"c8_refusal","actual S_C8/7 refuses pending character submit without entering channel",snapshot(&r,&m),&mut trace);
 e.steps.send(Step::Send(bytes(&o["normal_rx"][3]["wire"])))?;sent(&e);until(&mut r,&mut m,|r|r.source_navigation_snapshot().ignored_replies==1);
 choose(&mut r,2)?;let third=r.source_character_submit().unwrap();e.steps.send(Step::Expect(o["normal_tx"][2].clone()))?;r.handle_player_entry_command(Command::SubmitCharacter(&third))?;until(&mut r,&mut m,|r|r.source_navigation_snapshot().write_complete);sent(&e);
 let phase=r.source_flow().phase();let mode=r.source_flow().mode();let reply=bytes(&o["normal_rx"][4]["wire"]);e.steps.send(Step::Send(reply[..5].to_vec()))?;sent(&e);until(&mut r,&mut m,|r|r.source_stream_snapshot().body_bytes==3);
 assert_eq!(r.source_flow().phase(),phase);assert_eq!(r.source_navigation_snapshot().scene,Scene::AwaitCharacterReply);
 e.steps.send(Step::Send(reply[5..].to_vec()))?;sent(&e);until(&mut r,&mut m,|r|r.source_navigation_snapshot().scene==Scene::Lobby);
 let k=r.source_navigation_snapshot();assert_eq!(r.source_flow().phase(),3);assert_eq!(r.source_flow().mode(),mode);assert_eq!((k.return_mode,k.return_state),(1,0));assert_eq!(k.channel,Some(2));assert_eq!(k.fields.unwrap().at_64ad20,0x12345678);assert_eq!(r.source_identity().selected().unwrap().slot(),2);
 record(&base,"lobby_success","full actual S_C9 cc+dhd alone enters Modern lobby shell and source phase3, independent Native mode1/return state0",snapshot(&r,&m),&mut trace);
 let back=r.source_lobby_back().unwrap();let before=r.presentation_key();assert!(r.handle_player_entry_command(Command::BackLobby(&back)).is_err());assert!(r.handle_player_entry_command(Command::BackChannel).is_err());assert!(r.handle_player_entry_command(Command::BackRoom).is_err());assert_eq!(r.presentation_key(),before);
 record(&base,"backs_refused","distinct lobby/channel/room callbacks refuse absent sources atomically, emit no cCB/cCA/c0F",snapshot(&r,&m),&mut trace);
 e.steps.send(Step::Send([bytes(&o["normal_rx"][5]["wire"]),bytes(&o["normal_rx"][6]["wire"])].concat()))?;sent(&e);until(&mut r,&mut m,|r|r.source_navigation_snapshot().ignored_replies==2);
 assert_eq!(r.source_navigation_snapshot().scene,Scene::Lobby);assert_eq!(r.source_send_snapshot().counter,o["normal_tx"][2]["after"].as_u64());
 record(&base,"duplicate_lobby","duplicate S_C9 and late S_C8 after lobby success are ignored and cannot resend",snapshot(&r,&m),&mut trace);
 closed(&mut r,&mut m)?;assert!(r.handle_player_entry_command(Command::BackLobby(&back)).is_err());assert_eq!(r.source_navigation_snapshot().scene,Scene::Dormant);frames+=finish(e);
 record(&base,"close_lobby","Close retires lobby tokens and joins existing IO/resource workers",snapshot(&r,&m),&mut trace);
 //BSS return-state zero source path: S_C8 must send automatic CA100, not show page40.
 let mut r=ready(&p)?;let mut m=Metrics::default();let e=endpoint(&mut r,&o,1024,2)?;until(&mut r,&mut m,|r|r.source_character_control(0).is_some());choose(&mut r,0)?;
 let submit=r.source_character_submit().unwrap();e.steps.send(Step::Expect(o["auto_tx"][0].clone()))?;r.handle_player_entry_command(Command::SubmitCharacter(&submit))?;until(&mut r,&mut m,|r|r.source_navigation_snapshot().write_complete);sent(&e);
 e.steps.send(Step::Send(bytes(&o["auto_rx"][0]["wire"])))?;sent(&e);e.steps.send(Step::Expect(o["auto_tx"][1].clone()))?;until(&mut r,&mut m,|r|r.source_navigation_snapshot().scene==Scene::AwaitAutomaticLobbyReply&&r.source_navigation_snapshot().write_complete);sent(&e);
 assert_eq!(r.source_flow().phase(),0);assert!(r.source_character_control(0).is_none());assert!(r.source_lobby_back().is_none());
 record(&base,"automatic_ca100","S_C8/0 B8zero sends exact source cc202,100 and still waits for actual lobby reply",snapshot(&r,&m),&mut trace);
 e.steps.send(Step::Send(bytes(&o["auto_rx"][1]["wire"])))?;sent(&e);until(&mut r,&mut m,|r|r.source_navigation_snapshot().ignored_replies==1);assert_eq!(r.source_navigation_snapshot().scene,Scene::AwaitAutomaticLobbyReply);
 record(&base,"stale_c8_while_auto","duplicate previous C8 while awaiting C9 cannot reserve another automatic request",snapshot(&r,&m),&mut trace);
 let coalesced=o["auto_rx"].as_array().unwrap()[2..].iter().flat_map(|v|bytes(&v["wire"])).collect();e.steps.send(Step::Send(coalesced))?;sent(&e);until(&mut r,&mut m,|r|r.source_navigation_snapshot().ignored_replies==11);
 assert_eq!(r.source_navigation_snapshot().scene,Scene::Lobby);assert_eq!(r.source_navigation_snapshot().channel,Some(3));assert_eq!(r.source_stream_snapshot().counter,Some(o["auto_rx"].as_array().unwrap().last().unwrap()["after"].as_u64().unwrap()));
 record(&base,"coalesced_tail","11 coalesced C9 frames use same bounded8 dispatcher/tail; one success and ten duplicate rejections",snapshot(&r,&m),&mut trace);
 closed(&mut r,&mut m)?;frames+=finish(e);
 //New malformed navigation body actually arrives through the socket and retires all owners.
 let mut r=ready(&p)?;let mut m=Metrics::default();let e=endpoint(&mut r,&o,1024,1)?;until(&mut r,&mut m,|r|r.source_character_control(0).is_some());choose(&mut r,0)?;let t=r.source_character_submit().unwrap();e.steps.send(Step::Expect(o["auto_tx"][0].clone()))?;r.handle_player_entry_command(Command::SubmitCharacter(&t))?;until(&mut r,&mut m,|r|r.source_navigation_snapshot().write_complete);sent(&e);
 e.steps.send(Step::Send(bytes(&o["malformed_rx"][0]["wire"])))?;sent(&e);until(&mut r,&mut m,|r|r.source_connection_epoch().is_none());assert_eq!(r.source_navigation_snapshot().scene,Scene::Dormant);closed(&mut r,&mut m)?;frames+=finish(e);
 record(&base,"malformed_nav","actual short S_C9 success retires epoch/counters/navigation and fully joins workers",snapshot(&r,&m),&mut trace);
 //Partial navigation reply followed by EOF and Close while another reply is pending.
 for id in ["partial_nav_eof","close_waiting_nav"] {
  let mut r=ready(&p)?;let mut m=Metrics::default();let e=endpoint(&mut r,&o,1,1)?;until(&mut r,&mut m,|r|r.source_character_control(0).is_some());choose(&mut r,0)?;let t=r.source_character_submit().unwrap();e.steps.send(Step::Expect(o["auto_tx"][0].clone()))?;r.handle_player_entry_command(Command::SubmitCharacter(&t))?;until(&mut r,&mut m,|r|r.source_navigation_snapshot().write_complete);sent(&e);
  e.steps.send(Step::Send(bytes(&o["auto_rx"][0]["wire"])[..3].to_vec()))?;sent(&e);until(&mut r,&mut m,|r|r.source_stream_snapshot().body_bytes==1);
  if id=="partial_nav_eof" {e.steps.send(Step::Eof)?;sent(&e);until(&mut r,&mut m,|r|r.source_connection_epoch().is_none());}
  closed(&mut r,&mut m)?;frames+=finish(e);record(&base,id,"partial new navigation frame EOF/Close retires pending scene and IO worker",snapshot(&r,&m),&mut trace);
 }
 //Decoded admission and partial-write failure own the same Runtime; no substitute controller.
 let mut r=ready(&p)?;let mut m=Metrics::default();let epoch=r.begin_source_connection_candidate()?;
 for raw in [bytes(&o["setup"]["coalesced_handshake"]),bytes(&o["setup"]["first_roster"]["wire"])] {let mut offset=0;while offset<raw.len(){let b=r.feed_source_stream_candidate(&epoch,&raw[offset..])?;offset+=b.consumed;assert!(b.terminal.is_none());}}
 choose(&mut r,0)?;let t=r.source_character_submit().unwrap();r.handle_player_entry_command(Command::SubmitCharacter(&t))?;let ticket=r.source_pending_write_ticket(&epoch)?.unwrap();let early=epoch.decoded_packet(&bytes(&o["normal_rx"][4]["plain"]))?;
 assert_eq!(r.consume_source_navigation_event(&early)?,lito_modern::modern_lito::native_city_runtime::entry_navigation::Change::Ignored);assert_eq!(r.source_flow().phase(),0);
 r.report_source_write_candidate(&epoch,&ticket,0,WriteOutcome::Written(1))?;let view=r.source_write_view(&epoch,&ticket)?;r.report_source_write_candidate(&epoch,&ticket,1,WriteOutcome::Failed)?;assert!(view.bytes().is_err());assert!(r.consume_source_navigation_event(&early).is_err());assert_eq!(r.source_navigation_snapshot().scene,Scene::Dormant);
 let new_epoch=r.begin_source_connection_candidate()?;assert_ne!(new_epoch.serial(),epoch.serial());assert!(r.consume_source_navigation_event(&early).is_err());closed(&mut r,&mut m)?;
 record(&base,"write_failure_epoch","early reply cannot enter lobby; partial write failure retires write views, old epoch reply cannot enter replacement",snapshot(&r,&m),&mut trace);
 //Only new parser boundaries, not old profile/login suites.
 let mut rejects=0;for body in [vec![200],vec![200,0,1],vec![201],vec![201,0],vec![201,0,0],vec![201,0,0,0,0],vec![201,1,0,0]]{assert!(decode(&body).is_err());rejects+=1;}
 record(&base,"nav_schema","strict cc/dhd widths and trailing-byte rejection",json!({"new_rejections":rejects}),&mut trace);
 fs::write(base.join("r503-event-details.json"),serde_json::to_vec_pretty(&json!({"flows":trace,"actual_complete_socket_frames":frames,"endpoint_threads_joined":5,"IO_workers_joined":5,"runtime_closed":6,"elapsed_seconds":at.elapsed().as_secs_f64(),"historical_PASS_reruns":0,"GUI":0}))?)?;
 println!("R503 new navigation subset PASS: {} flows / {} actual wire frames / 5 endpoints joined / GUI0",trace.len(),frames);Ok(())
}
