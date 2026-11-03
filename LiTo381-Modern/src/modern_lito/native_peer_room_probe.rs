//! Room service integration through two real connections; files only coordinate input.
use super::{native_connection::Limits,native_login::{LoginController,LoginPhase as P},native_room::Settings};
use anyhow::{ensure,bail,Context,Result};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,time::{Instant,Duration}};
fn mark(root:&Path,role:&str,label:&str)->Result<()>{std::fs::write(root.join(format!("{role}-room-{label}")),b"ready")?;Ok(())}
fn has(root:&Path,role:&str,label:&str)->bool{root.join(format!("{role}-room-{label}")).exists()}
fn record(c:&LoginController,rows:&mut Vec<Value>,label:&str){rows.push(json!({"stage":label,"state":c.snapshot()}));}
pub(super) fn run(peer:SocketAddr,account:&str,password:&str,source:usize,role:&str,root:&Path)->Result<Value>{
    let mut c=LoginController::default();c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
    let start=Instant::now();let mut trace=vec![];let mut previous=Value::Null;
    let outcome=(||->Result<Value>{
        let mut submitted=false;let mut stage=0;let mut rows=vec![];
        let mut refreshed=Instant::now()-Duration::from_secs(2);let mut hold=Instant::now();
        let draft=Settings{name:"整合密碼房",password:"1234",attributes:4,capacity:12,options:Some(([0;4],&[]))};
        let edit=Settings{name:"整合修改房",password:"",attributes:32,capacity:8,options:Some(([1,1,10,1],&[1,2]))};
        loop{
            c.poll();let s=c.snapshot();
            let observation=json!({"phase":s.phase,"room":s.joined_room,"observer":s.observer,"result":s.room_result,"kick":s.room_kick_result,"error":s.error});
            if observation!=previous{ensure!(trace.len()<4096,"room trace bound");trace.push(json!({"ms":start.elapsed().as_millis(),"stage":stage,"state":observation}));previous=observation;}
            ensure!(start.elapsed()<Duration::from_secs(150),"room admin timeout stage {stage}: {previous}");
            if s.notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
            match c.snapshot().phase{
                P::ReadyForLogin if !submitted=>{c.login(account,password)?;submitted=true;},
                P::ReadyForLogin=>bail!("LOGIN rejected"),P::SourcesReady=>c.select_source(source)?,
                P::RedirectReady=>{ensure!(c.snapshot().redirect.context("GAME redirect")?.ip().is_loopback(),"nonloopback GAME");c.begin_game()?;},
                P::CharactersReady=>c.select_character(0)?,P::CharacterSelected=>c.enter_channel(0)?,
                P::Failed|P::Closed=>bail!("room admin failure stage {stage}: {:?}",c.snapshot().error),_=>{}
            }
            let s=c.snapshot();let phase=s.phase;
            let room=s.joined_room.as_ref();
            let two=room.is_some_and(|r|r.members.len()==2);
            let edited=room.is_some_and(|r|r.name==edit.name&&r.capacity==8&&!r.password_required&&r.members.iter().all(|m|m.state==0));
            let own=s.selected_character.as_deref();
            if role=="a"{match stage{
                0 if phase==P::LobbyReady=>{c.create_room(&draft)?;stage=1;},
                1 if phase==P::RoomReady=>{ensure!(room.unwrap().capacity==12&&room.unwrap().password_required,"created settings mismatch");record(&c,&mut rows,"created");mark(root,role,"created")?;stage=2;},
                2 if phase==P::RoomReady&&two&&has(root,"b","joined")=>{record(&c,&mut rows,"joined");mark(root,role,"joined")?;stage=3;},
                3 if phase==P::RoomReady&&has(root,"b","ready")&&room.unwrap().members.iter().any(|m|m.state!=0)=>{c.edit_room(&edit)?;stage=4;},
                4 if phase==P::RoomReady&&edited=>{record(&c,&mut rows,"edited");mark(root,role,"edited")?;stage=5;},
                5 if has(root,"b","observer-on")=>{record(&c,&mut rows,"observer-on");mark(root,role,"observer-on")?;stage=6;},
                6 if has(root,"b","observer-off")=>{record(&c,&mut rows,"observer-off");mark(root,role,"observer-off")?;hold=Instant::now();stage=7;},
                7 if phase==P::RoomReady&&hold.elapsed()>Duration::from_millis(250)=>{c.leave_room()?;stage=8;},
                8 if phase==P::LobbyReady&&has(root,"b","owner-transfer")=>{record(&c,&mut rows,"owner-transfer");mark(root,role,"owner-transfer")?;stage=9;},
                9 if phase==P::LobbyReady=>{if let Some(i)=s.rooms.iter().position(|r|r.name==edit.name){c.join_room(i,"")?;stage=10;}else if refreshed.elapsed()>Duration::from_millis(250){c.refresh_rooms()?;refreshed=Instant::now();}},
                10 if phase==P::RoomReady&&two=>{record(&c,&mut rows,"rejoined");mark(root,role,"rejoined")?;stage=11;},
                11 if phase==P::LobbyReady=>{ensure!(s.last_room_kick.as_deref()==own,"missing own kick notice");record(&c,&mut rows,"kicked");mark(root,role,"kicked")?;stage=12;},
                12 if phase==P::LobbyReady&&has(root,"b","kicked")=>{if let Some(i)=s.rooms.iter().position(|r|r.name==edit.name){c.join_room(i,"")?;stage=13;}else if refreshed.elapsed()>Duration::from_millis(250){c.refresh_rooms()?;refreshed=Instant::now();}},
                13 if phase==P::LobbyReady=>{ensure!(s.room_result.is_some_and(|x|x!=0),"kicked participant was not rejected");record(&c,&mut rows,"kick-rejoin-rejected");mark(root,role,"kick-rejoin-rejected")?;stage=14;},
                14 if phase==P::LobbyReady&&has(root,"b","finished")=>{record(&c,&mut rows,"finished");mark(root,role,"finished")?;stage=15;},_=>{}
            }}else{match stage{
                0 if phase==P::LobbyReady&&has(root,"a","created")=>{if let Some(i)=s.rooms.iter().position(|r|r.name==draft.name){c.join_room(i,"0000")?;stage=1;}else if refreshed.elapsed()>Duration::from_millis(250){c.refresh_rooms()?;refreshed=Instant::now();}},
                1 if phase==P::LobbyReady=>{ensure!(s.room_result.is_some_and(|x|x!=0),"wrong password not rejected");record(&c,&mut rows,"wrong-password-rejected");let i=s.rooms.iter().position(|r|r.name==draft.name).context("created room missing")?;c.join_room(i,draft.password)?;stage=2;},
                2 if phase==P::RoomReady&&two=>{ensure!(c.edit_room(&edit).is_err(),"nonowner may edit");record(&c,&mut rows,"joined");mark(root,role,"joined")?;stage=3;},
                3 if phase==P::RoomReady&&has(root,"a","joined")=>{c.set_ready(true)?;stage=4;},
                4 if phase==P::RoomReady&&room.unwrap().members.iter().any(|m|Some(m.name.as_str())==own&&m.state!=0)=>{record(&c,&mut rows,"ready-before-edit");mark(root,role,"ready")?;stage=5;},
                5 if phase==P::RoomReady&&edited&&has(root,"a","edited")=>{record(&c,&mut rows,"edited");hold=Instant::now();stage=6;},
                6 if phase==P::RoomReady&&hold.elapsed()>Duration::from_millis(250)=>{c.set_observer(true)?;stage=7;},
                7 if phase==P::RoomReady&&s.observer=>{ensure!(c.set_ready(true).is_err(),"observer ready accepted locally");record(&c,&mut rows,"observer-on");mark(root,role,"observer-on")?;hold=Instant::now();stage=8;},
                8 if phase==P::RoomReady&&has(root,"a","observer-on")&&hold.elapsed()>Duration::from_millis(250)=>{c.set_observer(false)?;stage=9;},
                9 if phase==P::RoomReady&&!s.observer=>{record(&c,&mut rows,"observer-off");mark(root,role,"observer-off")?;stage=10;},
                10 if phase==P::RoomReady&&room.is_some_and(|r|r.members.len()==1&&Some(r.owner.as_str())==own)=>{record(&c,&mut rows,"owner-transfer");mark(root,role,"owner-transfer")?;stage=11;},
                11 if phase==P::RoomReady&&two&&has(root,"a","rejoined")=>{record(&c,&mut rows,"rejoined");hold=Instant::now();stage=12;},
                12 if phase==P::RoomReady&&hold.elapsed()>Duration::from_millis(250)=>{let target=room.unwrap().members.iter().find(|m|Some(m.name.as_str())!=own).context("kick peer missing")?.name.clone();c.kick_member(&target)?;stage=13;},
                13 if phase==P::RoomReady&&room.is_some_and(|r|r.members.len()==1)&&s.room_kick_result==Some(true)=>{record(&c,&mut rows,"kicked");mark(root,role,"kicked")?;stage=14;},
                14 if phase==P::RoomReady&&has(root,"a","kick-rejoin-rejected")=>{c.leave_room()?;stage=15;},
                15 if phase==P::LobbyReady=>{record(&c,&mut rows,"finished");mark(root,role,"finished")?;stage=16;},_=>{}
            }}
            if (role=="a"&&stage==15)||(role=="b"&&stage==16&&has(root,"a","finished")){
                return Ok(json!({"status":"real_peer_room_admin_completed","action_profile":"room-admin","role":role,"character":c.snapshot().selected_character,"stages":rows,"transport":c.transport().snapshot()}));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })();
    c.cancel();let end=Instant::now()+Duration::from_secs(3);
    while !c.is_quiescent()&&Instant::now()<end{c.poll();std::thread::sleep(Duration::from_millis(3));}c.poll();
    std::fs::write(root.join(format!("{role}-client-trace.json")),serde_json::to_vec_pretty(&trace)?)?;
    ensure!(c.is_quiescent(),"room worker shutdown failed");let mut result=outcome?;
    result["worker_quiescent"]=json!(true);result["shutdown"]=json!(c.snapshot().phase);Ok(result)
}
