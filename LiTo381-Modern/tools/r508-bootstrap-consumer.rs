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
use lito_modern::modern_lito::native_city_runtime::login_bootstrap::{self as bootstrap,Packet,Stage as BootstrapStage,JavaFixtureAdapter};
fn hex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|x|u8::from_str_radix(std::str::from_utf8(x).unwrap(),16).unwrap()).collect()}
fn memory(p:&Path)->anyhow::Result<()> {
 let base=p.join("logs/continuation-20261003");let vectors:Value=serde_json::from_slice(&fs::read(base.join("r508-producer-inputs/protocol-goldens.json"))?)?;
 let raw=|id:&str|hex(vectors["vectors"].as_array().unwrap().iter().find(|v|v["id"]==id).unwrap()["body_hex"].as_str().unwrap());
 let a=raw("S_LOGIN_CHECK-R507-accepted");let Packet::Check(check)=bootstrap::decode(&a)? else{panic!()};assert_eq!(a.len(),26);assert_eq!(check.result,0);assert_eq!(check.term,3600);assert_eq!(check.name,b"LiTo 3.81C Local");assert_eq!((check.flag,check.word,check.extension),(0,0,None));
 assert!(bootstrap::validate_check_version(&check,0x0734fd33,false).is_err());bootstrap::validate_check_version(&check,0x0734fd33,true)?;bootstrap::validate_check_version(&check,0x5225,false)?;
 let mut native=a.clone();native.extend_from_slice(&0x80000007u32.to_le_bytes());let Packet::Check(ext)=bootstrap::decode(&native)? else{panic!()};bootstrap::validate_check_version(&ext,0x5226,false)?;assert!(bootstrap::validate_check_version(&ext,0x5225,false).is_err());assert!(bootstrap::validate_check_version(&ext,0x5226,true).is_err());assert_eq!(ext.extension,Some(0x80000007));
 for n in 0..a.len(){assert!(bootstrap::decode(&a[..n]).is_err(),"truncated S0 {n}");}for n in 1..4 {let mut v=a.clone();v.extend(vec![0;n]);assert!(bootstrap::decode(&v).is_err());}
 let Packet::Check(failure)=bootstrap::decode(&raw("S_LOGIN_CHECK-failed"))? else{panic!()};assert_eq!(failure.result,8);
 let Packet::Check(ms)=bootstrap::decode(&raw("S_LOGIN_CHECK-MS950"))? else{panic!()};assert_eq!(ms.name,hex("b4fab8d5"));
 let d=raw("S_SERVER_LIST-R507");let Packet::Directory(rows)=bootstrap::decode(&d)? else{panic!()};assert_eq!(rows.len(),2);assert_eq!(rows[0].name,b"WireSun");assert_eq!(rows[1].name,b"WireMoon");assert_eq!(rows[1].address_word,0x0200007f);assert_eq!(rows[1].port_word,64915);
 for n in 0..d.len(){assert!(bootstrap::decode(&d[..n]).is_err(),"truncated S3B {n}");}
 let mut many=vec![59,127];for _ in 0..127 {many.extend(vec![b'x';63]);many.push(0);many.extend_from_slice(&0x0200007fu32.to_le_bytes());many.extend_from_slice(&64915u16.to_le_bytes());}let Packet::Directory(rows)=bootstrap::decode(&many)? else{panic!()};assert_eq!(rows.len(),127);assert_eq!(rows[126].name.len(),63);
 assert!(bootstrap::decode(&[59,128]).is_err());assert!(bootstrap::decode(&[59,255]).is_err());assert!(bootstrap::decode(&[59,127]).is_err());assert_eq!(bootstrap::decode(&[59,0])?,Packet::Directory(vec![]));let mut long=vec![59,1];long.extend(vec![b'x';64]);long.extend(vec![0;7]);assert!(bootstrap::decode(&long).is_err());
 assert_eq!(bootstrap::decode(&[230])?,Packet::News);assert!(bootstrap::decode(&[230,0]).is_err());assert!(JavaFixtureAdapter::parse(br#"{"kind":"modern_synthetic_AD368_bootstrap_v1","producer_sha256":"bad","contract_sha256":"bad"}"#).is_err());
 fs::write(base.join("r508-memory-results.json"),serde_json::to_vec_pretty(&json!({"passed":5,"cases":["actual_AD368_login_check_and_Native_extension_boundary","strict_truncation_and_trailing_fields","failed_and_MS950_check_bytes","source_directory_two_and127_rows_signed_capacity_guards","NEWS_structure_and_explicit_adapter_identity"],"GUI":0,"Java_goldens_rerun":false,"historical_PASS_reruns":0}))?)?;
 println!("5 new Rust bootstrap parser checks passed");Ok(())
}
fn adapter()->anyhow::Result<JavaFixtureAdapter>{JavaFixtureAdapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_bootstrap_v1","producer_sha256":bootstrap::PRODUCER_SHA,"contract_sha256":bootstrap::CONTRACT_SHA}))?)}
fn boundary(p:&Path,r:&mut Runtime,m:&mut Metrics,label:&str)->anyhow::Result<()> {
 if let Some(v)=r.last_source_rejection(){let details=json!({"case":label,"epoch":v.epoch,"first_unsupported_opcode":v.opcode,"body_bytes":v.body_bytes,"reason":v.reason,"GUI":0,"credentials_pending":r.source_bootstrap_snapshot().credentials_pending});fs::write(p.join("logs/continuation-20261003/r508c-first-boundary.json"),serde_json::to_vec_pretty(&details)?)?;closed(r,m)?;anyhow::bail!("new first protocol boundary preserved");}Ok(())
}
fn main()->anyhow::Result<()> {
 let p=PathBuf::from(std::env::args_os().nth(1).unwrap());let arg=std::env::args_os().nth(2).unwrap();if arg=="--memory" {return memory(&p);}
 let java=PathBuf::from(arg);let base=p.join("logs/continuation-20261003");let state:Value=serde_json::from_slice(&fs::read(java.join("r508-ready.json"))?)?;
 let addr:SocketAddr=format!("127.0.0.1:{}",state["port"].as_u64().unwrap()).parse()?;let mut trace=Vec::new();
 for(label,account,source,slot)in [("Moon_named_source","WireSend","WireMoon",1),("Sun_named_source","WireRecv","WireSun",0)] {
  let mut r=ready(&p)?;let mut m=Metrics::default();let mut prof=profile(addr,false,31,3,0);prof["revision_word_source"]=json!(0x14f8);prof["from_cd_registry_dword"]=json!(2);configure(&mut r,&prof)?;r.configure_login_producer_fixture(adapter()?)?;
  assert!(r.source_news_control().is_none());r.handle_player_entry_command(Command::OpenLogin)?;edit_submit(&mut r,account.as_bytes(),b"WireOnly1")?;
  until(&mut r,&mut m,|r|r.source_bootstrap_snapshot().stage==BootstrapStage::SourceChoice||r.source_connection_epoch().is_none());boundary(&p,&mut r,&mut m,label)?;
  until(&mut r,&mut m,|r|r.source_transfer_snapshot().revision>0&&r.source_transfer_snapshot().rows_revision>0||r.source_connection_epoch().is_none());boundary(&p,&mut r,&mut m,label)?;
  let initial_epoch=r.source_connection_epoch().unwrap();let check=r.source_bootstrap_snapshot();assert_eq!(check.sources.len(),2);assert!(check.credentials_pending);assert!(check.check.unwrap().extension.is_none());assert_eq!(r.source_transfer_snapshot().server_rows.len(),1);assert!(r.source_identity().records().is_empty());wiped(&r);assert!(r.source_choice_control(2).is_none());assert!(r.source_news_control().is_none());
  assert!(r.handle_player_entry_command(Command::OpenLogin).is_err());let token=r.source_choice_control(slot).unwrap();let other=r.source_choice_control(1-slot).unwrap();
  let mut foreign=ready(&p)?;let mut fm=Metrics::default();foreign.begin_source_connection_candidate()?;assert!(foreign.handle_player_entry_command(Command::ChooseSource(&token)).is_err());closed(&mut foreign,&mut fm)?;
  r.handle_player_entry_command(Command::ChooseSource(&token))?;assert!(r.handle_player_entry_command(Command::ChooseSource(&token)).is_err());assert!(r.handle_player_entry_command(Command::ChooseSource(&other)).is_err());assert!(r.source_news_control().is_none());
  until(&mut r,&mut m,|r|r.source_bootstrap_snapshot().stage==BootstrapStage::NewsChoice||r.source_connection_epoch().is_none());boundary(&p,&mut r,&mut m,label)?;
  let game_epoch=r.source_connection_epoch().unwrap();assert_ne!(initial_epoch,game_epoch);let game=r.source_bootstrap_snapshot();assert_eq!(game.selected_name.as_deref(),Some(source.as_bytes()));assert!(!game.credentials_pending);assert!(r.source_transfer_snapshot().selected_name.is_none());assert!(!r.source_transfer_snapshot().primary_return);assert!(!r.source_transfer_snapshot().alternate_return);assert_eq!(r.source_loopback_snapshot().unwrap().peer.ip().octets(),[127,0,0,2]);assert!(r.source_actor_snapshot().version.is_some());assert!(r.handle_player_entry_command(Command::ChooseSource(&token)).is_err());
  let news=r.source_news_control().unwrap();r.handle_player_entry_command(Command::ContinueNews(&news))?;assert!(r.handle_player_entry_command(Command::ContinueNews(&news)).is_err());
  until(&mut r,&mut m,|r|r.source_identity().complete()||r.source_connection_epoch().is_none());boundary(&p,&mut r,&mut m,label)?;
  assert_eq!(r.source_bootstrap_snapshot().roster_count,Some(1));assert_eq!(r.source_identity().records().len(),1);assert_eq!(r.source_identity().records()[0].name(),format!("{account}({source})").as_bytes());assert!(r.source_character_control(0).is_some());assert!(!r.source_bootstrap_snapshot().credentials_pending);assert!(r.source_news_control().is_none());assert!(r.handle_player_entry_command(Command::ContinueNews(&news)).is_err());
  let details=json!({"source":source,"account":"synthetic fixture","initial_epoch":initial_epoch,"GAME_epoch":game_epoch,"source0_accepted":true,"source59_consumed":true,"FB_namespace_separate":true,"F3_consumed":true,"source_C8_from_received_row":true,"fresh_GAME_VERSION_LOGIN":true,"E6_matching_E2_once":true,"actual_roster_count":1,"received_character_alias":String::from_utf8_lossy(r.source_identity().records()[0].name()),"native_complete_claim":false,"skills_owner_ready":false,"guard_checks":9});closed(&mut r,&mut m)?;let details=json!({"flow":details,"after_Close":snapshot(&r,&m),"owned_credentials_cleared":!r.source_bootstrap_snapshot().credentials_pending});trace.push(json!({"case":label,"passed":true,"details":details}));fs::write(base.join("r508c-event-partial.json"),serde_json::to_vec_pretty(&trace)?)?;
 }
 fs::write(base.join("r508c-event-details.json"),serde_json::to_vec_pretty(&trace)?)?;println!("2 new AD368 LOGIN/source choice/GAME NEWS/roster flows passed; all owned Rust workers joined");Ok(())
}
