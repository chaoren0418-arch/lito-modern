//! Reuse a production controller after an actual owned-server socket failure.
use super::{native_connection::Limits,native_login::{LoginController,LoginPhase},native_world_scene::{Frame,Model}};
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::{Path,PathBuf},sync::atomic::AtomicBool,time::{Duration,Instant}};
pub fn run(peer:SocketAddr,account:&str,password:&str,source:usize,mode:&str)->Result<Value>{
    ensure!(peer.ip().is_loopback()&&peer.port()!=0&&matches!(mode,"room"|"world"),"invalid recovery fixture");
    let root=PathBuf::from(std::env::var("LITO_NATIVE_RECOVERY_ROOT")?).canonicalize()?;
    ensure!(root.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/native-recovery").canonicalize()?),"recovery root");
    let limits=Limits{read_chunk:2,write_chunk:3,..Limits::default()};
    let mut c=LoginController::default();c.start(peer,limits)?;
    let start=Instant::now();let mut submitted=false;let mut restarted=false;let mut fault_requested=false;
    let mut before=Value::Null;let mut failure=Value::Null;let mut after=Value::Null;
    let mut old_frame:Option<Frame>=None;let cancel=AtomicBool::new(false);let mut first_epoch=0;
    let outcome=(||->Result<Value>{loop{
        c.poll();ensure!(start.elapsed()<Duration::from_secs(110),"recovery deadline {:?}",c.snapshot().phase);
        if fault_requested && !restarted && c.snapshot().phase==LoginPhase::Failed && c.is_quiescent()
            && root.join("drop-closed").exists(){
            ensure!(c.snapshot().error.is_some()&&c.snapshot().inventory.items.is_empty(),"failure not retired");
            ensure!(c.attack(1).is_err()&&c.join_room(0,"").is_err()&&c.set_ready(true).is_err(),"failed controller accepts gameplay");
            failure=json!(c.snapshot());c.start(peer,limits)?;submitted=false;restarted=true;
            let s=c.snapshot();ensure!(s.world.actors.is_empty()&&s.joined_room.is_none()&&s.selected_character.is_none()
                &&s.inventory.items.is_empty()&&s.sources.is_empty()&&s.error.is_none(),"reconnect retained authority");
        }
        if !after.is_null(){
            if root.join("recovery-observed").exists(){return Ok(json!({"status":"recovery_completed","mode":mode,
                "before":before,"failure":failure,"after":after,"first_epoch":first_epoch,
                "last_epoch":c.transport().snapshot().epoch,"stale_frame_rejected":mode=="world"}));}
            std::thread::sleep(Duration::from_millis(5));continue;
        }
        if fault_requested&&!restarted{std::thread::sleep(Duration::from_millis(5));continue;}
        if c.snapshot().notice.is_some(){c.acknowledge_notice()?;}
        match c.snapshot().phase{
            LoginPhase::ReadyForLogin if !submitted=>{c.login(account,password)?;submitted=true;}
            LoginPhase::ReadyForLogin=>anyhow::bail!("recovery authentication rejected"),
            LoginPhase::SourcesReady=>c.select_source(source)?,
            LoginPhase::RedirectReady=>{ensure!(c.snapshot().redirect.context("redirect")?.ip().is_loopback(),"remote redirect");c.begin_game()?;}
            LoginPhase::CharactersReady=>c.select_character(0)?,
            LoginPhase::CharacterSelected=>c.enter_channel(0)?,
            LoginPhase::LobbyReady=>c.join_room(0,"")?,
            LoginPhase::RoomReady if mode=="world"=>{
                let s=c.snapshot();let room=s.joined_room.as_ref().context("recovery room")?;
                if room.members.iter().any(|m|Some(m.name.as_str())==s.selected_character.as_deref()&&m.state==0){c.set_ready(true)?;}
            }
            LoginPhase::RoomReady|LoginPhase::BattlePending=>{
                let w=&c.snapshot().world;
                if mode=="world"&&(!w.ready()||w.integrity_challenge.is_none()){std::thread::sleep(Duration::from_millis(5));continue;}
                if mode=="world"{
                    if restarted{
                        let old=old_frame.as_ref().context("old frame")?;
                        ensure!(w.generation>before["world"]["generation"].as_u64().unwrap(),"world generation reused after reconnect");
                        ensure!(!old.movement_ready(w)&&!old.input_ready(w)&&!old.death_ready(w),"stale frame authorizes reconnected world");
                    }
                    let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),w,&cancel)?;
                    let frame=model.draw_at(w,&cancel,0)?;
                    ensure!(frame.movement_ready(w),"fresh frame unavailable");
                    super::native_peer_probe::capture(&root,"recovery",if restarted{2}else{1},"world",&frame)?;
                    if !restarted{old_frame=Some(frame);}
                }
                ensure!(c.snapshot().selected_source.as_deref()==Some("測試乙"),"recovery selected source");
                if restarted{
                    ensure!(c.transport().snapshot().epoch>first_epoch,"transport epoch reused");
                    after=json!(c.snapshot());std::fs::write(root.join("recovered"),b"recovered")?;
                }else{
                    first_epoch=c.transport().snapshot().epoch;before=json!(c.snapshot());
                    std::fs::write(root.join("drop-ready"),b"drop owned GAME socket")?;fault_requested=true;
                }
            }
            LoginPhase::Failed|LoginPhase::Closed=>anyhow::bail!("unexpected recovery failure {:?}",c.snapshot().error),
            _=>{}
        }
        std::thread::sleep(Duration::from_millis(5));
    }})();
    c.cancel();let until=Instant::now()+Duration::from_secs(3);
    while !c.is_quiescent()&&Instant::now()<until{c.poll();std::thread::sleep(Duration::from_millis(3));}c.poll();
    ensure!(c.is_quiescent(),"recovery worker leaked");let mut value=outcome?;value["worker_quiescent"]=json!(true);Ok(value)
}
