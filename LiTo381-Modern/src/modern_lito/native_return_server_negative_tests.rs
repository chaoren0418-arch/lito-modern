//! Opt-in live negative protocol test. Raw requests exist only in the test binary.
use super::*;
use serde_json::{json,Value};
use std::{path::Path,thread};

fn wait(c:&mut LoginController, condition:impl Fn(&Snapshot)->bool)->Result<()> {
    let end=Instant::now()+Duration::from_secs(15);
    while !condition(c.snapshot()) {
        c.poll();
        ensure!(!matches!(c.snapshot().phase,LoginPhase::Failed|LoginPhase::Closed),"bootstrap: {:?}",c.snapshot().error);
        ensure!(Instant::now()<end,"bootstrap timeout: {:?}",c.snapshot().phase);
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}
fn observe(root:&Path,role:&str,label:&str)->Result<Value>{
    std::fs::write(root.join(format!("{role}-observe-request")),label)?;
    let path=root.join(format!("{role}-observe-{label}.json"));
    let end=Instant::now()+Duration::from_secs(3);
    loop {
        if let Ok(bytes)=std::fs::read(&path){if let Ok(value)=serde_json::from_slice(&bytes){return Ok(value)}}
        ensure!(Instant::now()<end,"passive observer timeout {role}/{label}");
        thread::sleep(Duration::from_millis(10));
    }
}
fn raw_step(c:&mut LoginController,root:&Path,role:&str,label:&str,body:&[u8],expected:&str,rows:&mut Vec<Value>)->Result<()> {
    let before=observe(root,role,&format!("{label}-before"))?;
    c.transport.send(c.transport.snapshot().epoch,Zeroizing::new(body.to_vec()))?;
    let end=Instant::now()+Duration::from_millis(650);let mut received=vec![];let mut sent=false;
    while Instant::now()<end {
        for event in c.transport.poll(){match event{
            Event::Packet{body,..}=>received.push(body),Event::Sent{..}=>sent=true,_=>{}
        }}
        ensure!(c.transport.snapshot().phase==Phase::VersionAccepted,"negative session closed: {:?}",c.transport.snapshot().error);
        thread::sleep(Duration::from_millis(10));
    }
    let after=observe(root,role,&format!("{label}-after"))?;
    rows.push(json!({"label":label,"sent":body,"send_confirmed":sent,"received":received,"before":before,"after":after}));
    std::fs::write(root.join(format!("{role}-negative.json")),serde_json::to_vec_pretty(&rows)?)?;
    ensure!(sent,"request not flushed");
    ensure!(after["pending_transition"]==expected,"{label}: unexpected transition {}",after["pending_transition"]);
    if label!="return" && label!="duplicate-return" {
        for key in ["room","rental_town","rental_set","rental_map","rental_inventory","rental_hotel","character","phase"] {
            ensure!(before[key]==after[key],"{label}: mutated {key}");
        }
    }
    let identity=format!("{}({})",before["test_account"].as_str().context("account")?,before["source"].as_str().context("source")?);
    let mut controls=vec![];
    for packet in &received {
        if packet.first()==Some(&0xd0) {
            // Concurrent players may receive each other's legitimate departure.
            let super::super::native_room::Packet::Departed{room,name}=super::super::native_room::parse(packet)? else{bail!("departure parser")};
            ensure!(room==60001,"unexpected departed room");
            if name!=identity {ensure!(name==format!("{}(測試乙)",if role=="a"{"PeerFixture2"}else{"PeerFixture"}),"unknown departing peer");continue;}
        }
        if let Some(op)=packet.first().copied().filter(|o|matches!(o,0x65|0xf0|0xd0|0xf5)){controls.push(op);}
    }
    match label {
        "restart"=>ensure!(controls==[0x65],"restart responses: {controls:?}"),
        "quit"=>ensure!(controls==[0xf0],"quit responses: {controls:?}"),
        "return"=>{
            ensure!(controls==[0xd0,0xf5],"return responses: {controls:?}");
            ensure!(after["room"]==0 && after["rental_town"]==false && after["rental_inventory"].as_array().is_some_and(Vec::is_empty),"return retained rental ownership");
        },
        _=>ensure!(controls.is_empty(),"{label}: unexpected control acknowledgement: {controls:?}"),
    }
    Ok(())
}
fn player(peer:SocketAddr,root:&Path,role:&str,account:&str)->Result<()> {
    let mut c=LoginController::default();let mut rows=vec![];
    let result=(||->Result<()>{
        c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
        wait(&mut c,|s|s.phase==LoginPhase::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;
        wait(&mut c,|s|s.phase==LoginPhase::SourcesReady)?;c.select_source(1)?;
        wait(&mut c,|s|s.phase==LoginPhase::RedirectReady)?;
        ensure!(c.snapshot().redirect.context("GAME endpoint")?.ip().is_loopback(),"nonloopback GAME rejected");c.begin_game()?;
        let end=Instant::now()+Duration::from_secs(15);
        while c.snapshot().phase!=LoginPhase::CharactersReady {
            c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
            ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,LoginPhase::Failed|LoginPhase::Closed),"GAME bootstrap: {:?}",c.snapshot().error);
            thread::sleep(Duration::from_millis(10));
        }
        c.select_character(0)?;wait(&mut c,|s|s.phase==LoginPhase::CharacterSelected)?;
        c.select_rental_set(0)?;wait(&mut c,|s|s.phase==LoginPhase::RentalReady)?;
        c.join_rental_room(0)?;wait(&mut c,|s|s.phase==LoginPhase::BattlePending&&s.world.ready())?;
        // Do not run application auto-acknowledgement while testing invalid wire order.
        for (label,op,expected) in [
            ("early-quit",0xd9,"NONE"),("early-return",0xd0,"NONE"),
            ("restart",0x10,"RESTART_AWAIT_QUIT_FIGHT"),
            ("duplicate-restart",0x10,"RESTART_AWAIT_QUIT_FIGHT"),
            ("return-before-quit",0xd0,"RESTART_AWAIT_QUIT_FIGHT"),
            ("quit",0xd9,"RESTART_AWAIT_RETURN"),
            ("duplicate-quit",0xd9,"RESTART_AWAIT_RETURN"),
            ("restart-after-quit",0x10,"RESTART_AWAIT_RETURN"),
            ("return",0xd0,"RESTART_AWAIT_SOURCE_REFRESH"),
            ("duplicate-return",0xd0,"RESTART_AWAIT_SOURCE_REFRESH"),
        ] {raw_step(&mut c,root,role,label,&[op],expected,&mut rows)?;}
        Ok(())
    })();
    c.cancel();let end=Instant::now()+Duration::from_secs(3);
    while !c.is_quiescent()&&Instant::now()<end {c.poll();thread::sleep(Duration::from_millis(10));}
    result
}
#[test]
#[ignore = "requires explicitly started owned loopback rental review harness"]
fn actual_server_rejects_out_of_order_and_duplicate_return_requests(){
    let root=std::path::PathBuf::from(std::env::var("LITO_NEGATIVE_ROOT").expect("owned test root"));
    let allowed=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap();
    assert!(root.canonicalize().unwrap().starts_with(allowed));
    let ready:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();
    assert_eq!(ready["jar_sha256"],"6887a337e347d561366e0eb7eb8e93a2b7ea03add7a3888fc4be2b70c3479da7");
    let peer:SocketAddr=format!("127.0.0.1:{}",ready["port"].as_u64().unwrap()).parse().unwrap();
    thread::scope(|scope|{
        let a=scope.spawn(||player(peer,&root,"a","PeerFixture"));
        let b=scope.spawn(||player(peer,&root,"b","PeerFixture2"));
        let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");
    });
}
