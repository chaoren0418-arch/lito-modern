use lito_modern::modern_lito::{
    native_city_runtime::{
        login_events::FixtureProfile, login_form::Stage, player_entry_commands::Command, Runtime,
        UiPage,
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
    json!({"kind":"modern_synthetic_loopback_fixture_v1","label":if special{"SYNTHETIC:R502_SPECIAL"}else{"SYNTHETIC:R502_NORMAL"},"endpoint":endpoint.to_string(),"code_page":950,"from_cd_registry_dword":0x123456A5u32,"revision_word_source":0x112345u32,"supplemental":if special{Some("FIXTURE-EXTRA")}else{None},"read_cap":read_cap,"write_cap":write_cap,"connect_delay_ms":delay})
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
        base.join("r502-event-partial.json"),
        serde_json::to_vec_pretty(trace).unwrap(),
    )
    .unwrap();
}
fn selected(request: &str, id: &str) -> bool {
    request == "all" || request == id
}
fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let mut args = std::env::args_os().skip(1);
    let p = PathBuf::from(args.next().unwrap());
    let request = args
        .next()
        .map(|s| s.to_str().unwrap().to_owned())
        .unwrap_or_else(|| "all".into());
    let base = p.join("logs/continuation-20261003");
    let oracle: Value = serde_json::from_slice(&fs::read(base.join("r502-source-oracle.json"))?)?;
    let mut trace = Vec::new();
    if selected(&request, "bounds") {
        let good = profile("127.0.0.1:9".parse()?, false, 1024, 2, 0);
        let mut count = 0;
        for (k, v) in [
            ("kind", json!("production")),
            ("label", json!("R502")),
            ("endpoint", json!("192.0.2.1:9")),
            ("read_cap", json!(0)),
            ("read_cap", json!(1025)),
            ("write_cap", json!(0)),
            ("write_cap", json!(158)),
            ("connect_delay_ms", json!(2001)),
            ("supplemental", json!("A\u{0}B")),
            ("password", json!("dummy")),
        ] {
            let mut bad = good.clone();
            bad[k] = v;
            assert!(FixtureProfile::parse(&serde_json::to_vec(&bad)?).is_err());
            count += 1;
        }
        assert!(FixtureProfile::parse(&vec![b' '; 4097]).is_err());
        record(&base,"bounds","explicit synthetic profile,endpoint,channel IO caps,secret-free schema and oversize fail before IO",json!({"rejections":count+1,"connection_attempts":0}),&mut trace);
    }
    if selected(&request, "normal_local") {
        let mut r = ready(&p)?;
        let mut m = Metrics::default();
        r.handle_player_entry_command(Command::OpenLogin)?;
        edit_submit(&mut r, b"fixtureA", b"dummyPass1")?;
        assert_eq!(r.source_login_snapshot().stage, Stage::AwaitingAuthorities);
        assert!(r.source_send_snapshot().counter.is_none());
        assert_eq!(r.player_fixture_snapshot().workers_started, 0);
        let c = r.source_login_control().unwrap();
        r.handle_player_entry_command(Command::CancelLogin(&c))?;
        wiped(&r);
        closed(&mut r, &mut m)?;
        record(&base,"normal_local","same new Window command bridge without fixture remains local staging and creates no network worker",snapshot(&r,&m),&mut trace);
    }
    for (index, id) in [(0, "normal_roundtrip"), (1, "special_roundtrip")] {
        if !selected(&request, id) {
            continue;
        }
        let v = oracle["profiles"][index].clone();
        let mut r = ready(&p)?;
        let mut m = Metrics::default();
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let endpoint = listener.local_addr()?;
        let sv = v.clone();
        let (tx, rx) = mpsc::channel::<u8>();
        let server = thread::spawn(move || {
            let (mut s, peer) = listener.accept().unwrap();
            assert!(peer.ip().is_loopback());
            s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
            s.set_nodelay(true).unwrap();
            s.write_all(&bytes(&sv["coalesced_handshake"])).unwrap();
            for i in 0..2 {
                expect(&mut s, &sv["send_vectors"][i]);
            }
            assert_eq!(rx.recv_timeout(Duration::from_secs(10)).unwrap(), 1);
            s.write_all(&bytes(&sv["first_roster"]["wire"])).unwrap();
            for i in 2..5 {
                expect(&mut s, &sv["send_vectors"][i]);
            }
            assert_eq!(rx.recv_timeout(Duration::from_secs(10)).unwrap(), 2);
            s.write_all(&bytes(&sv["second_roster"]["wire"])).unwrap();
            expect(&mut s, &sv["send_vectors"][5]);
            let mut one = [0];
            assert_eq!(s.read(&mut one).unwrap(), 0);
        });
        let cfg = profile(
            endpoint,
            index == 1,
            if index == 0 { 1024 } else { 1 },
            if index == 0 { 1 } else { 2 },
            if index == 0 { 300 } else { 0 },
        );
        configure(&mut r, &cfg)?;
        fs::write(
            base.join(format!("r502-{id}-used-synthetic-profile.json")),
            serde_json::to_vec_pretty(&cfg)?,
        )?;
        let at = Instant::now();
        r.handle_player_entry_command(Command::OpenLogin)?;
        let open_us = at.elapsed().as_micros();
        assert!(open_us < 200_000);
        let epoch = r.source_connection_epoch();
        let at = Instant::now();
        edit_submit(&mut r, b"fixtureA", b"dummyPass1")?;
        assert!(at.elapsed() < Duration::from_millis(200));
        let old_stage = r.source_login_control().unwrap();
        until(&mut r, &mut m, |r| {
            r.source_login_snapshot().stage == Stage::Sending
        });
        let view = r.player_fixture_write_view()?.unwrap();
        let before = r.presentation_key();
        assert!(r
            .handle_player_entry_command(Command::SubmitLogin(&old_stage))
            .is_err());
        assert_eq!(before, r.presentation_key());
        until(&mut r, &mut m, |r| {
            r.source_login_snapshot().stage == Stage::AwaitingReply
                && r.source_stream_snapshot().counter == v["receive_after_versions"].as_u64()
        });
        assert!(view.bytes().is_err());
        assert_eq!(
            r.source_send_snapshot().counter,
            v["send_vectors"][1]["after"].as_u64()
        );
        assert!(!r.presentation_key().character_ui.active);
        wiped(&r);
        for _ in 0..8 {
            tick(&mut r, &mut m);
        }
        assert_eq!(r.source_login_snapshot().stage, Stage::AwaitingReply);
        assert_eq!(r.source_flow().phase(), 0);
        assert!(!m.partial.is_empty());
        let connection = r.source_loopback_snapshot().unwrap();
        record(&base,id,"same Window Open/Edit/Submit commands return before delayed connect; worker handshake/partial TX; Complete waits actual packet",json!({"open_callback_us":open_us,"connect_delay_ms":cfg["connect_delay_ms"],"counter":r.source_send_snapshot().counter,"Complete_is_authsuccess":false,"metrics":snapshot(&r,&m)}),&mut trace);
        tx.send(1)?;
        until(&mut r, &mut m, |r| r.presentation_key().character_ui.active);
        assert_eq!(r.source_login_snapshot().stage, Stage::Hidden);
        assert_eq!(
            r.source_stream_snapshot().counter,
            v["first_roster"]["after"].as_u64()
        );
        let role = r.source_character_control(0).unwrap();
        r.handle_player_entry_command(Command::ChooseCharacter {
            control: &role,
            double_click: false,
        })?;
        let submit = r.source_character_submit().unwrap();
        r.handle_player_entry_command(Command::SubmitCharacter(&submit))?;
        until(&mut r, &mut m, |r| {
            r.source_send_snapshot().pending.is_none()
        });
        assert_eq!(
            r.source_send_snapshot().counter,
            v["send_vectors"][2]["after"].as_u64()
        );
        let back = r.source_character_back().unwrap();
        let rx_before = r.source_stream_snapshot();
        let version = r.source_actor_snapshot();
        r.handle_player_entry_command(Command::BackCharacter(&back))?;
        until(&mut r, &mut m, |r| {
            r.source_login_snapshot().stage == Stage::Editing
        });
        assert_eq!(r.source_connection_epoch(), epoch);
        assert_eq!(r.source_stream_snapshot(), rx_before);
        assert_eq!(r.source_actor_snapshot(), version);
        assert_eq!(r.source_flow().phase(), 5);
        assert!(r.presentation_key().awaiting_roster);
        assert_eq!(
            r.source_loopback_snapshot().unwrap().local,
            connection.local
        );
        assert_eq!(r.source_loopback_snapshot().unwrap().peer, connection.peer);
        assert_eq!(r.player_fixture_snapshot().workers_started, 1);
        assert!(r
            .handle_player_entry_command(Command::ChooseCharacter {
                control: &role,
                double_click: false
            })
            .is_err());
        assert!(r
            .handle_player_entry_command(Command::SubmitCharacter(&submit))
            .is_err());
        assert!(r
            .handle_player_entry_command(Command::BackCharacter(&back))
            .is_err());
        record(&base,id,"actual3/203 then shared character select/submit/Back callbacks; complete Back reopens same form and socket without seed reset",json!({"same_epoch":epoch,"same_socket":connection.local.to_string(),"Back_requires_fresh_roster":true,"metrics":snapshot(&r,&m)}),&mut trace);
        edit_submit(&mut r, b"fixtureB", b"dummyPass2")?;
        let old_relogin = r.source_login_control().unwrap();
        until(&mut r, &mut m, |r| {
            r.source_login_snapshot().stage == Stage::AwaitingReply
        });
        assert_eq!(
            r.source_send_snapshot().counter,
            v["send_vectors"][4]["after"].as_u64()
        );
        assert_eq!(r.source_connection_epoch(), epoch);
        assert_eq!(r.source_stream_snapshot(), rx_before);
        assert!(r.presentation_key().awaiting_roster);
        assert!(!r.presentation_key().character_ui.active);
        tx.send(2)?;
        until(&mut r, &mut m, |r| r.presentation_key().character_ui.active);
        assert_eq!(r.source_identity().records()[0].name(), b"Again0");
        assert_eq!(
            r.source_stream_snapshot().counter,
            v["second_roster"]["after"].as_u64()
        );
        assert!(r
            .handle_player_entry_command(Command::SubmitLogin(&old_relogin))
            .is_err());
        let fresh = r.source_character_control(0).unwrap();
        r.handle_player_entry_command(Command::ChooseCharacter {
            control: &fresh,
            double_click: true,
        })?;
        until(&mut r, &mut m, |r| {
            r.source_send_snapshot().pending.is_none()
        });
        assert_eq!(
            r.source_send_snapshot().counter,
            v["send_vectors"][5]["after"].as_u64()
        );
        closed(&mut r, &mut m)?;
        server.join().unwrap();
        assert!(r
            .handle_player_entry_command(Command::ChooseCharacter {
                control: &fresh,
                double_click: false
            })
            .is_err());
        record(&base,id,"retained relogin waits actual new roster; fresh double-click typed201; Close clears all capabilities and joins own socket/resource workers",json!({"exact_source_frames":6,"metrics":snapshot(&r,&m)}),&mut trace);
    }
    if selected(&request, "late_connect") {
        let mut r = ready(&p)?;
        let mut m = Metrics::default();
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let endpoint = listener.local_addr()?;
        listener.set_nonblocking(true)?;
        configure(&mut r, &profile(endpoint, false, 1024, 1, 1000))?;
        let at = Instant::now();
        r.handle_player_entry_command(Command::OpenLogin)?;
        edit_submit(&mut r, b"fixtureA", b"dummyPass1")?;
        assert!(at.elapsed() < Duration::from_millis(200));
        for _ in 0..8 {
            tick(&mut r, &mut m);
        }
        assert!(!r.player_fixture_snapshot().connected);
        closed(&mut r, &mut m)?;
        assert!(listener.accept().is_err());
        record(&base,"late_connect","Close during delayed worker connect stays nonblocking, prevents socket creation and clears staged credentials",snapshot(&r,&m),&mut trace);
    }
    if selected(&request, "late_read") {
        let mut r = ready(&p)?;
        let mut m = Metrics::default();
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let endpoint = listener.local_addr()?;
        let raw = bytes(&oracle["profiles"][0]["coalesced_handshake"]);
        let server = thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            s.write_all(&raw).unwrap();
            let mut one = [0];
            match s.read(&mut one) {
                Ok(0) => {},
                Err(e) if matches!(e.kind(),std::io::ErrorKind::ConnectionReset|std::io::ErrorKind::ConnectionAborted) => {},
                other => panic!("owned close must end peer without auth bytes: {other:?}"),
            }
        });
        configure(&mut r, &profile(endpoint, false, 1, 1, 0))?;
        r.handle_player_entry_command(Command::OpenLogin)?;
        edit_submit(&mut r, b"fixtureA", b"dummyPass1")?;
        let old = r.source_login_control().unwrap();
        pause_until(|| r.player_fixture_snapshot().queued_events == 4);
        assert!(r.source_loopback_snapshot().is_none());
        assert!(r.source_send_snapshot().counter.is_none());
        closed(&mut r, &mut m)?;
        server.join().unwrap();
        assert!(r.player_fixture_snapshot().discarded_late_events >= 4);
        assert!(r
            .handle_player_entry_command(Command::SubmitLogin(&old))
            .is_err());
        record(&base,"late_read","four-event queue backpressure while UI paused; queued Connected/read results rejected after Close with no seed or auth promotion",snapshot(&r,&m),&mut trace);
    }
    if selected(&request, "late_write") {
        let mut r = ready(&p)?;
        let mut m = Metrics::default();
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let endpoint = listener.local_addr()?;
        let seed = bytes(&oracle["seed_wire"]);
        let server = thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            s.write_all(&seed).unwrap();
            let mut partial = Vec::new();
            s.read_to_end(&mut partial).unwrap();
            assert_eq!(partial.len(), 1);
            partial
        });
        configure(&mut r, &profile(endpoint, false, 1024, 1, 0))?;
        r.handle_player_entry_command(Command::OpenLogin)?;
        let c = r.source_login_control().unwrap();
        r.handle_player_entry_command(Command::EditLogin {
            control: &c,
            account: b"fixtureA",
            password: b"dummyPass1",
        })?;
        until(&mut r, &mut m, |r| {
            r.source_send_snapshot().pending.is_some()
        });
        let old = r.source_login_control().unwrap();
        let view = r.player_fixture_write_view()?.unwrap();
        pause_until(|| r.player_fixture_snapshot().write_reports >= 1);
        assert!(view.bytes().is_ok());
        assert_eq!(r.source_send_snapshot().pending.unwrap().2, 0);
        closed(&mut r, &mut m)?;
        assert!(view.bytes().is_err());
        assert!(r.player_fixture_snapshot().discarded_late_events >= 1);
        assert!(r
            .handle_player_entry_command(Command::SubmitLogin(&old))
            .is_err());
        let partial = server.join().unwrap();
        assert_eq!(
            partial,
            bytes(&oracle["profiles"][0]["send_vectors"][0]["wire"])[..1]
        );
        record(&base,"late_write","actual worker partial write report queued before Close; pending ticket/view and secret draft retire before late result is drained",snapshot(&r,&m),&mut trace);
    }
    for (id, header_eof) in [("unsupported59", false), ("header_eof", true)] {
        if !selected(&request, id) {
            continue;
        }
        let mut r = ready(&p)?;
        let mut m = Metrics::default();
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let endpoint = listener.local_addr()?;
        let v = oracle["profiles"][0].clone();
        let (tx, rx) = mpsc::channel::<u8>();
        let server = thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            s.set_nodelay(true).unwrap();
            s.write_all(&bytes(&v["coalesced_handshake"])).unwrap();
            expect(&mut s, &v["send_vectors"][0]);
            expect(&mut s, &v["send_vectors"][1]);
            assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), 1);
            if header_eof {
                s.write_all(&bytes(&v["partial_roster_header"])).unwrap();
                assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
                s.shutdown(Shutdown::Write).unwrap();
            } else {
                s.write_all(&bytes(&v["unsupported59"]["wire"])).unwrap();
            }
            let mut one = [0];
            assert_eq!(s.read(&mut one).unwrap(), 0);
        });
        configure(&mut r, &profile(endpoint, false, 1024, 2, 0))?;
        r.handle_player_entry_command(Command::OpenLogin)?;
        edit_submit(&mut r, b"fixtureA", b"dummyPass1")?;
        let old = r.source_login_control().unwrap();
        until(&mut r, &mut m, |r| {
            r.source_login_snapshot().stage == Stage::AwaitingReply
        });
        tx.send(1)?;
        if header_eof {
            until(&mut r, &mut m, |r| {
                r.source_stream_snapshot().header_bytes == 1
            });
            assert_eq!(
                r.source_stream_snapshot().counter,
                oracle["profiles"][0]["receive_after_versions"].as_u64()
            );
            tx.send(2)?;
        }
        until(&mut r, &mut m, |r| r.source_connection_epoch().is_none());
        assert!(r.source_identity().records().is_empty());
        assert!(!r.presentation_key().character_ui.active);
        assert!(r.source_send_snapshot().counter.is_none());
        wiped(&r);
        if !header_eof {
            assert!(r.status().contains("59"));
        }
        assert!(r
            .handle_player_entry_command(Command::SubmitLogin(&old))
            .is_err());
        closed(&mut r, &mut m)?;
        server.join().unwrap();
        record(
            &base,
            id,
            if header_eof {
                "actual socket EOF with retained one-byte header retires receiver and UI wait"
            } else {
                "actual59 is a controlled unsupported-list result, retires epoch and never becomes roster/auth success"
            },
            snapshot(&r, &m),
            &mut trace,
        );
    }
    if selected(&request, "connect_failure") {
        let mut r = ready(&p)?;
        let mut m = Metrics::default();
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let endpoint = listener.local_addr()?;
        drop(listener);
        configure(&mut r, &profile(endpoint, false, 1024, 2, 150))?;
        r.handle_player_entry_command(Command::OpenLogin)?;
        edit_submit(&mut r, b"fixtureA", b"dummyPass1")?;
        until(&mut r, &mut m, |r| r.source_connection_epoch().is_none());
        assert!(r.status().contains("合成連線失敗"));
        wiped(&r);
        closed(&mut r, &mut m)?;
        record(&base,"connect_failure","own released ephemeral endpoint connect failure is asynchronous and clears staged input",snapshot(&r,&m),&mut trace);
    }
    let details = json!({"state":"R502_SHARED_WINDOW_COMMAND_SAME_RUNTIME_BOUNDED_ASYNC_SYNTHETIC_FINITE_PASS","requested_cases":request,"flow_cases":trace.len(),"trace":trace,"consumer_executions":1,"historical_PASS_reruns":0,"player_executions":0,"native_GUI_or_HWND":false,"source_setters":false,"real_credentials":false,"existing_provider":false,"Java_SQL_server_sharedBIN":false,"formal_authentication_connected":false,"GUI":0,"LOADED":false,"complete_modes":16,"total_modes":40,"focused_seconds":start.elapsed().as_secs_f64()});
    fs::write(
        base.join("r502-event-details.json"),
        serde_json::to_vec_pretty(&details)?,
    )?;
    println!(
        "R502 PASS: {}new shared-command/async-IO flows;player0/GUI0/full16of40;{:.3}s",
        trace.len(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
