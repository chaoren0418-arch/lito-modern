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
    json!({"kind":"modern_synthetic_loopback_fixture_v1","label":if special{"SYNTHETIC:R508_SPECIAL"}else{"SYNTHETIC:R508_NORMAL"},"endpoint":endpoint.to_string(),"code_page":950,"from_cd_registry_dword":0x123456A5u32,"revision_word_source":0x112345u32,"supplemental":if special{Some("FIXTURE-EXTRA")}else{None},"read_cap":read_cap,"write_cap":write_cap,"connect_delay_ms":delay})
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
use lito_modern::modern_lito::native_city_runtime::login_bootstrap::{JavaFixtureAdapter,CONTRACT_SHA,PRODUCER_SHA,Stage as BootstrapStage};
fn h(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|v|u8::from_str_radix(std::str::from_utf8(v).unwrap(),16).unwrap()).collect()}
fn encrypted(body:&[u8],counter:&mut u64)->Vec<u8>{let key=counter.to_le_bytes();let mut previous=0;let mut out=(body.len()as u16+2).to_le_bytes().to_vec();for(i,b)in body.iter().enumerate(){let b=b^key[i&7]^previous;out.push(b);previous=b;}*counter=counter.wrapping_add(body.len()as u64);out}
fn read_body(s:&mut TcpStream,counter:&mut u64)->Vec<u8>{let mut header=[0;2];s.read_exact(&mut header).unwrap();let n=u16::from_le_bytes(header)as usize;assert!((3..=157).contains(&n));let mut raw=vec![0;n-2];s.read_exact(&mut raw).unwrap();let key=counter.to_le_bytes();let body=raw.iter().enumerate().map(|(i,b)|b^if i==0{0}else{raw[i-1]}^key[i&7]).collect();*counter=counter.wrapping_add(raw.len()as u64);body}
fn main()->anyhow::Result<()> {
 let p=PathBuf::from(std::env::args_os().nth(1).unwrap());let base=p.join("logs/continuation-20261003");let v:Value=serde_json::from_slice(&fs::read(base.join("r508-producer-inputs/protocol-goldens.json"))?)?;
 let golden=|id:&str|h(v["vectors"].as_array().unwrap().iter().find(|v|v["id"]==id).unwrap()["body_hex"].as_str().unwrap());let version=golden("S_VERSION-accepted");let check=golden("S_LOGIN_CHECK-R507-accepted");let rows=golden("S_SERVER_LIST-R507");
 let listener=TcpListener::bind("127.0.0.1:0")?;let address=listener.local_addr()?;let (go,rx)=mpsc::channel();let (done,seen)=mpsc::channel();
 let server=thread::spawn(move||{let(mut s,_)=listener.accept().unwrap();s.set_read_timeout(Some(Duration::from_secs(20))).unwrap();let mut out=0x87546ca112345678u64;let mut input=out;s.write_all(&h("07000178563412")).unwrap();assert_eq!(read_body(&mut s,&mut input)[0],0);s.write_all(&encrypted(&version,&mut out)).unwrap();assert_eq!(read_body(&mut s,&mut input)[0],1);let mut bytes=encrypted(&check,&mut out);bytes.extend(encrypted(&rows,&mut out));s.write_all(&bytes).unwrap();rx.recv_timeout(Duration::from_secs(20)).unwrap();s.write_all(&encrypted(&[59,0],&mut out)).unwrap();done.send(()).unwrap();let mut last=[0];assert_eq!(s.read(&mut last).unwrap(),0);2usize});
 let mut r=ready(&p)?;let mut m=Metrics::default();let mut prof=profile(address,false,5,3,0);prof["revision_word_source"]=json!(0x14f8);prof["from_cd_registry_dword"]=json!(2);configure(&mut r,&prof)?;r.configure_login_producer_fixture(JavaFixtureAdapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_bootstrap_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":CONTRACT_SHA}))?)?)?;r.handle_player_entry_command(Command::OpenLogin)?;edit_submit(&mut r,b"fixtureZero",b"dummyPass1")?;
 until(&mut r,&mut m,|r|r.source_bootstrap_snapshot().stage==BootstrapStage::SourceChoice);let before=r.source_bootstrap_snapshot();assert_eq!(before.source_count,2);let c=r.source_choice_control(0).unwrap();let c1=r.source_choice_control(1).unwrap();go.send(())?;seen.recv_timeout(Duration::from_secs(20))?;
 until(&mut r,&mut m,|r|r.source_bootstrap_snapshot().source_count==0);let after=r.source_bootstrap_snapshot();assert_eq!(after.sources,before.sources);assert!(after.revision>before.revision);assert!(r.source_choice_control(0).is_none());assert!(r.source_choice_control(1).is_none());assert!(r.handle_player_entry_command(Command::ChooseSource(&c)).is_err());assert!(r.handle_player_entry_command(Command::ChooseSource(&c1)).is_err());assert!(r.source_send_snapshot().pending.is_none());assert_eq!(r.source_connection_epoch(),Some(1));assert!(after.credentials_pending);closed(&mut r,&mut m)?;assert_eq!(server.join().unwrap(),2);assert!(!r.source_bootstrap_snapshot().credentials_pending);
 fs::write(base.join("r508z-zero-directory-results.json"),serde_json::to_vec_pretty(&json!({"new_checks":1,"passed":true,"case":"positive_then_zero_S3B_preserves_rows_but_disables_current_and_stale_controls","initial_count":2,"new_count":0,"old_rows_retained":true,"stale_controls_rejected":2,"C_C8_sent":0,"pending_credential_lease_cleared_on_Close":true,"after_Close":snapshot(&r,&m),"synthetic_endpoint_actual_TX":2,"producer":"explicit scripted loopback endpoint using sealed body vectors, not Java authentication","previous_7_PASS_reruns":0,"GUI":0}))?)?;println!("1 new zero-directory state/credential-Close guard passed; socket worker joined");Ok(())
}
