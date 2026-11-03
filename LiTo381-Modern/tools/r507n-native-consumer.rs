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
        base.join("r507n-event-partial.json"),
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
  let(mut s,peer)=listener.accept().unwrap();assert!(peer.ip().is_loopback());s.set_read_timeout(Some(Duration::from_secs(20))).unwrap();s.set_nodelay(true).unwrap();
  s.write_all(&bytes(&setup["coalesced_handshake"])).unwrap();
  expect(&mut s,&setup["send_vectors"][0]);expect(&mut s,&setup["send_vectors"][1]);
  s.write_all(&bytes(&setup["first_roster"]["wire"])).unwrap();let mut frames=2;
  loop {match rx.recv_timeout(Duration::from_secs(20)).unwrap(){
   Step::Send(b)=>s.write_all(&b).unwrap(),
   Step::Expect(v)=>{expect(&mut s,&v);frames+=1;},
   Step::Eof=>{s.shutdown(Shutdown::Write).unwrap();},
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
fn main()->anyhow::Result<()>{let p=PathBuf::from(std::env::args_os().nth(1).unwrap());let java=PathBuf::from(std::env::args_os().nth(2).unwrap());let base=p.join("logs/continuation-20261003");let mut trace=Vec::new(); // Genuine separate Java Main + owned SQL. This records the first real
 // missing protocol authority; it is not upgraded to a full path PASS.
 let state:Value=serde_json::from_slice(&fs::read(java.join("r507-ready.json"))?)?;let addr:SocketAddr=format!("127.0.0.1:{}",state["port"].as_u64().unwrap()).parse()?;
 let mut r=ready(&p)?;let mut m=Metrics::default();let mut native_profile=profile(addr,false,1024,157,0);native_profile["revision_word_source"]=json!(0x14f8);native_profile["from_cd_registry_dword"]=json!(2);configure(&mut r,&native_profile)?;r.handle_player_entry_command(Command::OpenLogin)?;edit_submit(&mut r,b"WireSend",b"WireOnly1")?;until(&mut r,&mut m,|r|r.source_connection_epoch().is_none());let status=r.status().to_owned();let details=json!({"status":status,"snapshot":snapshot(&r,&m),"full_login_channel_path":false,"fixture_environment":"explicit synthetic registry/version inputs against actual isolated Java Main"});closed(&mut r,&mut m)?;record(&base,"actual_Java_Main_boundary","actual loopback Java/SQL attempted once; first unsupported/rejected producer retained as dependency, no packet drop or translated namespace",details,&mut trace);
fs::write(base.join("r507n-event-details.json"),serde_json::to_vec_pretty(&trace)?)?;println!("actual Java boundary recorded");Ok(())}