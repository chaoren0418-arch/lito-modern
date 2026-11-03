//! Diagnose C05 from the exact native directory, without inventing a world scene.
use super::{native_connection::Limits,native_login::{LoginController,LoginPhase as P}};
use anyhow::{ensure,bail,Context,Result};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,time::{Duration,Instant}};
pub(super) fn run(peer:SocketAddr,account:&str,password:&str,source:usize,role:&str,root:&Path)->Result<Value>{
    let mut c=LoginController::default();c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
    let start=Instant::now();let mut trace=vec![];let mut prior=Value::Null;
    let outcome=(||->Result<Value>{
        let mut logged=false;let mut requested=false;let mut observed=None;let mut returning=false;
        let mut directory=Value::Null;let mut boundary=Value::Null;
        loop{
            c.poll();let s=c.snapshot();let row=json!({"phase":s.phase,"check":s.classic_entry_result,"selected":s.selected_character,"map":s.world.map_id,"error":s.error});
            if row!=prior{trace.push(json!({"ms":start.elapsed().as_millis(),"state":row}));prior=row;}
            ensure!(start.elapsed()<Duration::from_secs(35),"classic entry timeout {prior}");
            if s.notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
            match c.snapshot().phase{
                P::ReadyForLogin if !logged=>{c.login(account,password)?;logged=true;},
                P::ReadyForLogin=>bail!("classic LOGIN rejected"),P::SourcesReady=>c.select_source(source)?,
                P::RedirectReady=>{ensure!(c.snapshot().redirect.context("GAME redirect")?.ip().is_loopback(),"nonloopback redirect");c.begin_game()?;},
                P::CharactersReady if !requested=>{directory=json!(c.snapshot().characters);c.request_classic_entry(0)?;requested=true;},
                P::Failed|P::Closed=>bail!("classic entry failed: {:?}",c.snapshot().error),_=>{}
            }
            if requested&&!returning&&c.snapshot().classic_entry_result.is_some(){
                let since=observed.get_or_insert_with(Instant::now);
                if since.elapsed()>=Duration::from_secs(2){
                    boundary=json!(c.snapshot());
                    std::fs::write(root.join(format!("{role}-classic-observed")),b"observed")?;
                    let other=if role=="a"{"b"}else{"a"};
                    if root.join(format!("{other}-classic-observed")).exists(){
                        if c.snapshot().phase==P::ClassicCharacterSelected{c.return_characters()?;}
                        returning=true;
                    }
                }
            }
            if returning&&c.snapshot().phase==P::CharactersReady{
                return Ok(json!({"status":"real_peer_classic_entry_observed","action_profile":"classic-entry","role":role,
                    "directory":directory,"boundary":boundary,"return_directory":c.snapshot().characters,"transport":c.transport().snapshot(),"observation_ms":observed.unwrap().elapsed().as_millis()}));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })();
    c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();std::thread::sleep(Duration::from_millis(3));}c.poll();
    std::fs::write(root.join(format!("{role}-client-trace.json")),serde_json::to_vec_pretty(&trace)?)?;
    ensure!(c.is_quiescent(),"classic worker did not stop");let mut v=outcome?;v["worker_quiescent"]=json!(true);v["shutdown"]=json!(c.snapshot().phase);Ok(v)
}
