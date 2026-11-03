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
        inputs(p, "r506-native-menu-skin-binding.json"),
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
    let end = Instant::now() + Duration::from_secs(20);
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
    json!({"kind":"modern_synthetic_loopback_fixture_v1","label":if special{"SYNTHETIC:R507_SPECIAL"}else{"SYNTHETIC:R507_NORMAL"},"endpoint":endpoint.to_string(),"code_page":950,"from_cd_registry_dword":0x123456A5u32,"revision_word_source":0x112345u32,"supplemental":if special{Some("FIXTURE-EXTRA")}else{None},"read_cap":read_cap,"write_cap":write_cap,"connect_delay_ms":delay})
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
        base.join("r507t-event-partial.json"),
        serde_json::to_vec_pretty(trace).unwrap(),
    )
    .unwrap();
}
enum Step { Send(Vec<u8>), Expect(Value), Eof, Finish,Reaccept }
struct Endpoint { steps:mpsc::Sender<Step>, done:mpsc::Receiver<()>, thread:thread::JoinHandle<usize> }
fn endpoint(r:&mut Runtime,o:&Value,read_cap:usize,write_cap:usize)->anyhow::Result<Endpoint> {
 let listener=TcpListener::bind("127.0.0.1:0")?;let addr=listener.local_addr()?;
 let setup=o["setup"].clone();let (steps,rx)=mpsc::channel();let (done_tx,done)=mpsc::channel();
 let server=thread::spawn(move||{
  let(mut s,peer)=listener.accept().unwrap();assert!(peer.ip().is_loopback());s.set_read_timeout(Some(Duration::from_secs(20))).unwrap();s.set_nodelay(true).unwrap();
  s.write_all(&bytes(&setup["coalesced_handshake"])).unwrap();
  expect(&mut s,&setup["send_vectors"][0]);expect(&mut s,&setup["send_vectors"][1]);
  s.write_all(&bytes(&setup["first_roster"]["wire"])).unwrap();let mut frames=2;
  loop {match rx.recv_timeout(Duration::from_secs(20)).unwrap(){
   Step::Send(b)=>s.write_all(&b).unwrap(),
   Step::Expect(v)=>{expect(&mut s,&v);frames+=1;},
   Step::Reaccept=>panic!("only the new timeout target can reaccept"),Step::Eof=>{s.shutdown(Shutdown::Write).unwrap();},
   Step::Finish=>{let mut b=[0];assert_eq!(s.read(&mut b).unwrap(),0);break;},
  }done_tx.send(()).unwrap();}frames
 });
 configure(r,&profile(addr,false,read_cap,write_cap,0))?;r.configure_source_startup_host_fixture(lito_modern::modern_lito::native_city_runtime::startup_host::FixtureStartupHost::parse(br#"{"kind":"modern_synthetic_startup_host_v1","host":"127.0.0.2"}"#)?)?;r.handle_player_entry_command(Command::OpenLogin)?;edit_submit(r,b"fixtureA",b"dummyPass1")?;
 Ok(Endpoint{steps,done,thread:server})
}
fn sent(e:&Endpoint){e.done.recv_timeout(Duration::from_secs(20)).unwrap();}
fn finish(e:Endpoint)->usize {e.steps.send(Step::Finish).unwrap();e.thread.join().unwrap()}
fn choose(r:&mut Runtime,slot:usize)->anyhow::Result<()> {let c=r.source_character_control(slot).unwrap();r.handle_player_entry_command(Command::ChooseCharacter{control:&c,double_click:false})}
fn hold(r:&mut Runtime,m:&mut Metrics){for _ in 0..8{tick(r,m);thread::sleep(Duration::from_millis(2));}}

fn input_wire(body:&[u8],counter:u64)->Vec<u8>{let mut out=((body.len()+2)as u16).to_le_bytes().to_vec();let key=counter.to_le_bytes();let mut previous=0;for(i,b)in body.iter().enumerate(){previous=*b^key[i&7]^previous;out.push(previous);}out}
fn target(listener:TcpListener,o:&Value)->Endpoint{
 let setup=o["setup"].clone();let(steps,rx)=mpsc::channel();let(done_tx,done)=mpsc::channel();let server=thread::spawn(move||{
  let(mut s,peer)=listener.accept().unwrap();assert!(peer.ip().is_loopback());s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();s.set_nodelay(true).unwrap();s.write_all(&bytes(&setup["coalesced_handshake"])).unwrap();expect(&mut s,&setup["send_vectors"][0]);let mut frames=1;done_tx.send(()).unwrap();
  loop{match rx.recv_timeout(Duration::from_secs(30)).unwrap(){Step::Send(b)=>s.write_all(&b).unwrap(),Step::Expect(v)=>{expect(&mut s,&v);frames+=1;},Step::Reaccept=>{let mut b=[0];assert_eq!(s.read(&mut b).unwrap(),0);let(next,peer)=listener.accept().unwrap();assert!(peer.ip().is_loopback());s=next;s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();s.set_nodelay(true).unwrap();s.write_all(&bytes(&setup["coalesced_handshake"])).unwrap();expect(&mut s,&setup["send_vectors"][0]);frames+=1;},Step::Eof=>{s.shutdown(Shutdown::Write).unwrap();},Step::Finish=>{let mut b=[0];assert_eq!(s.read(&mut b).unwrap(),0);break;}}done_tx.send(()).unwrap();}frames});Endpoint{steps,done,thread:server}
}
fn main()->anyhow::Result<()>{
 let p=PathBuf::from(std::env::args_os().nth(1).unwrap());let base=p.join("logs/continuation-20261003");let o:Value=serde_json::from_slice(&fs::read(base.join("r507-timeout-oracle.json"))?)?;let listener=TcpListener::bind("127.0.0.3:0")?;let addr=listener.local_addr()?;let fallback_listener=TcpListener::bind("127.0.0.2:2000")?;let mut r=ready(&p)?;let mut m=Metrics::default();let e=endpoint(&mut r,&o,1,1)?;let primary=target(listener,&o);let fallback=target(fallback_listener,&o);let mut trace=Vec::new();let start=Instant::now();
 until(&mut r,&mut m,|r|r.source_character_control(0).is_some());choose(&mut r,0)?;let submit=r.source_character_submit().unwrap();e.steps.send(Step::Expect(o["timeout_tx"][0].clone()))?;r.handle_player_entry_command(Command::SubmitCharacter(&submit))?;until(&mut r,&mut m,|r|r.source_navigation_snapshot().write_complete);sent(&e);
 let mut f4=vec![244,0];f4.extend(0x0300007fu32.to_le_bytes());f4.extend(addr.port().to_le_bytes());e.steps.send(Step::Send(input_wire(&f4,o["setup"]["first_roster"]["after"].as_u64().unwrap())))?;sent(&e);for i in 1..3{e.steps.send(Step::Expect(o["timeout_tx"][i].clone()))?;}
 until(&mut r,&mut m,|r|r.player_fixture_snapshot().workers_started==2&&r.source_loopback_snapshot().is_some_and(|s|s.compatibility_complete));sent(&e);sent(&e);sent(&primary);
 primary.steps.send(Step::Expect(o["target_tx"][0].clone()))?;primary.steps.send(Step::Send([bytes(&o["target_rx"][0]["wire"]),bytes(&o["target_rx"][1]["wire"]),bytes(&o["target_rx"][2]["wire"])].concat()))?;sent(&primary);until(&mut r,&mut m,|r|r.source_transfer_snapshot().replay_sent&&r.source_navigation_snapshot().write_complete);sent(&primary);assert!(r.source_transfer_snapshot().primary_return);let epoch2=r.source_connection_epoch();
 until(&mut r,&mut m,|r|r.player_fixture_snapshot().workers_started==3&&r.source_loopback_snapshot().is_some_and(|s|s.compatibility_complete));sent(&fallback);assert_ne!(r.source_connection_epoch(),epoch2);assert_eq!(r.source_transfer_snapshot().current_address.unwrap().to_string(),"127.0.0.2:2000");assert!(r.source_transfer_snapshot().alternate_return&&!r.source_transfer_snapshot().primary_return&&r.source_transfer_snapshot().replay_sent);assert!(start.elapsed()>Duration::from_secs(10));
 record(&base,"primary_timeout_to_independent_host","strict10s Modern monotonic timeout creates fresh socket/seed and BC0 BD1 while preserving BE1 and selected alias",snapshot(&r,&m),&mut trace);
 fallback.steps.send(Step::Send([bytes(&o["timeout_rx"][0]["wire"]),bytes(&o["timeout_rx"][1]["wire"])].concat()))?;sent(&fallback);until(&mut r,&mut m,|r|r.source_transfer_snapshot().roster_count==Some(1));assert!(r.source_transfer_snapshot().replay_sent);assert!(r.source_send_snapshot().pending.is_none());primary.steps.send(Step::Reaccept)?;let epoch3=r.source_connection_epoch();
 until(&mut r,&mut m,|r|r.player_fixture_snapshot().workers_started==4&&r.source_loopback_snapshot().is_some_and(|s|s.compatibility_complete));sent(&primary);assert_ne!(r.source_connection_epoch(),epoch3);assert_eq!(r.source_transfer_snapshot().current_address.unwrap().to_string(),addr.to_string());assert!(r.source_transfer_snapshot().primary_return&&!r.source_transfer_snapshot().alternate_return&&r.source_transfer_snapshot().replay_sent);
 primary.steps.send(Step::Send([bytes(&o["timeout_rx"][0]["wire"]),bytes(&o["timeout_rx"][1]["wire"]),bytes(&o["timeout_rx"][2]["wire"])].concat()))?;sent(&primary);until(&mut r,&mut m,|r|r.source_channel_back().is_some());assert!(!r.source_transfer_snapshot().replay_sent);assert_eq!(r.source_navigation_snapshot().return_state,2);closed(&mut r,&mut m)?;let frames=finish(e)+finish(primary)+finish(fallback);
 record(&base,"alternate_timeout_to_saved_primary","second strict10s fresh epoch/socket reenters saved F4 primary, retains BE1 until actual C8 success clears it, then normal Close joins all4 IO owners",snapshot(&r,&m),&mut trace);
 fs::write(base.join("r507t-event-details.json"),serde_json::to_vec_pretty(&json!({"trace":trace,"actual_socket_frames":frames,"seconds":start.elapsed().as_secs_f64(),"historical_PASS_reruns":0,"GUI":0}))?)?;println!("new timer directions passed frames={frames}");Ok(())
}
