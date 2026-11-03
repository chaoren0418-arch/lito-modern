//! Programmatic loopback development test; no GUI login or authority injection.
use anyhow::{ensure,Context,Result};
use lito_modern::modern_lito as m;
use serde_json::{json,Value};
use std::{net::SocketAddr,path::{Path,PathBuf},thread,time::{Duration,Instant}};
fn authority(c:&m::native_login::LoginController)->Value {
    json!({"world":c.snapshot().world,"inventory":c.snapshot().inventory})
}
fn picture(root:&Path,role:&str,gfx:u16,frame:&m::native_world_scene::Frame)->Result<Value> {
    let path=root.join(format!("{role}-effect-{gfx}.png"));let mut e=png::Encoder::new(std::fs::File::create(&path)?,800,600);
    e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header()?.write_image_data(frame.canvas.rgba())?;
    Ok(json!({"png":path,"receipt":frame.receipt}))
}
fn peer(role:&str,endpoint:SocketAddr,root:&Path)->Result<()> {
    use m::native_login::{LoginController,LoginPhase as Phase};
    let mut c=LoginController::default();let mut view=m::native_world_view::Presentation::default();
    c.start(endpoint,m::native_connection::Limits{read_chunk:2,write_chunk:3,..Default::default()})?;
    let mut login_submitted=false;let mut force_sent=false;let mut refresh=Instant::now()-Duration::from_secs(2);
    let started=Instant::now();let mut previous=Value::Null;let mut trace=Vec::new();let mut initial=None;
    let mut seen=std::collections::BTreeMap::new();let mut actions=Vec::new();let mut next=0usize;
    let mut pending:Option<(usize,Instant,Value,Value,String)>=None;let mut finished=false;
    let mut recovery_walk:Option<(Instant,Value)>=None;
    let outcome=(||->Result<()>{loop {
        c.poll();ensure!(c.snapshot().error.is_none(),"controller error {:?}",c.snapshot().error);
        let state=json!({"phase":c.snapshot().phase,"world_generation":c.snapshot().world.generation,"casts":c.snapshot().world.own_cast_packets,"mp":c.snapshot().world.mp,"status":c.snapshot().world.status,"feedback":c.snapshot().skill_feedback});
        if state!=previous {ensure!(trace.len()<4096,"trace budget");trace.push(json!({"ms":started.elapsed().as_millis(),"state":state}));previous=state;}
        ensure!(started.elapsed()<Duration::from_secs(210),"existing210s peer functional deadline: {previous}");
        if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
        match c.snapshot().phase {
            Phase::ReadyForLogin if !login_submitted=>{c.login(if role=="a"{"PeerFixture"}else{"PeerFixture2"},"TestOnlyPassword")?;login_submitted=true;},
            Phase::ReadyForLogin=>anyhow::bail!("isolated memory authentication rejected"),
            Phase::SourcesReady=>c.select_source(1)?,
            Phase::RedirectReady=>{ensure!(c.snapshot().redirect.context("GAME endpoint")?.ip().is_loopback(),"nonloopback redirect");c.begin_game()?;},
            Phase::CharactersReady=>c.select_character(0)?,Phase::CharacterSelected=>c.enter_channel(0)?,
            Phase::LobbyReady=>{if c.snapshot().rooms.is_empty(){if refresh.elapsed()>Duration::from_secs(1){c.refresh_rooms()?;refresh=Instant::now();}}else{c.join_room(0,"")?;force_sent=false;}},
            Phase::RoomReady=>{
                let room=c.snapshot().joined_room.as_ref().context("joined room")?;
                let own=room.members.iter().find(|a|Some(a.name.as_str())==c.snapshot().selected_character.as_deref()).context("own member")?;
                let desired=if role=="a"{0}else{1};
                if own.force!=desired&&!force_sent{c.set_force(desired)?;force_sent=true;}
                else if own.force==desired&&room.members.len()==2&&room.members.iter().any(|a|a.force!=desired)&&own.state==0{c.set_ready(true)?;}
            },
            Phase::BattlePending=>{
                let s=c.snapshot();if !s.world.ready()||s.world.integrity_challenge.is_none()||!s.world.actors.contains_key(&if role=="a"{3}else{1}){thread::sleep(Duration::from_millis(3));continue;}
                let epoch=c.transport().snapshot().epoch;view.tick(Path::new(r"D:\lineage開發\0-客戶端"),epoch,&s.world,&s.environment,true);
                ensure!(view.error.is_none(),"candidate render error {:?}",view.error);
                let Some(frame)=view.frame.as_ref().filter(|f|f.input_ready(&c.snapshot().world))else{thread::sleep(Duration::from_millis(3));continue;};
                if initial.is_none(){
                    ensure!(c.snapshot().world.own_actor().context("own actor")?.id==if role=="a"{1}else{3},"memory player identity");
                    ensure!(c.snapshot().world.actors[&1].relations.clan.is_empty()&&c.snapshot().world.actors[&3].relations.clan.is_empty(),"non-clan rejection fixture changed");
                    if role=="b"{ensure!([26,42].iter().all(|s|c.snapshot().world.learned_skills.contains(s)),"actual book lacks abilities");ensure!(c.snapshot().world.mp.context("MP")?[0]>=100,"default caster MP below two cast costs");}
                    initial=Some(authority(&c));std::fs::write(root.join(format!("{role}-ready")),b"rendered")?;
                }
                for gfx in [750u16,6342] {
                    if !seen.contains_key(&gfx)&&frame.receipt["skill_effects"]["effects"].as_array().context("effect receipt")?.iter().any(|e|e["graphics"]==gfx&&e["drawn"]==true){seen.insert(gfx,picture(root,role,gfx,frame)?);}
                }
                if role=="a" {
                    if root.join("recovery-walk-target.json").exists()&&!root.join("a-walk-observed").exists(){
                        let plan:Value=serde_json::from_slice(&std::fs::read(root.join("recovery-walk-target.json"))?)?;
                        let actor=c.snapshot().world.actors.get(&3).context("remote recovering caster")?;
                        if json!([actor.x,actor.y])==plan["target"] {std::fs::write(root.join("a-walk-observed"),serde_json::to_vec(&json!({"actor":actor.id,"position":[actor.x,actor.y]}))?)?;}
                    }
                    if root.join("b-finished").exists()&&seen.len()==2 {std::fs::write(root.join("a-finished"),b"observed")?;return Ok(());}
                }else if root.join("a-ready").exists(){
                    if let Some((index,sent,before,immediate,duplicate))=&pending {
                        ensure!(sent.elapsed()<Duration::from_secs(60),"60s skill application stage: {previous}");
                        let answered=if *index==0 {c.snapshot().skill_feedback==m::native_skills::Feedback::NoResponse}
                            else if *index==3 {c.snapshot().skill_feedback==m::native_skills::Feedback::Rejected(278)}
                            else {c.snapshot().skill_feedback==m::native_skills::Feedback::CastAction&&seen.contains_key(&if *index==1{750}else{6342})};
                        if answered&&frame.movement_ready(&c.snapshot().world) {
                            let after=authority(&c);let status=&after["world"]["status"];
                            if matches!(*index,0|3) {
                                ensure!(before["world"]["mp"]==after["world"]["mp"]&&before["world"]["own_cast_packets"]==after["world"]["own_cast_packets"],"rejected target/MP changed MP/action authority");
                                ensure!(before["world"]["status"]==after["world"]["status"]&&before["world"]["ability_buffs"]==after["world"]["ability_buffs"],"rejected cast changed abilities");
                            }else {
                                let old_mp=before["world"]["mp"][0].as_i64().context("before MP")?;let new_mp=after["world"]["mp"][0].as_i64().context("after MP")?;
                                ensure!(old_mp-new_mp==50,"actual ability MP cost changed");
                                ensure!(after["world"]["own_cast_packets"].as_u64()==before["world"]["own_cast_packets"].as_u64().map(|n|n+1),"ability cast action count");
                                let ability=if *index==1{"dexterity"}else{"strength"};
                                ensure!(status[ability].as_u64()==before["world"]["status"][ability].as_u64().map(|n|n+5),"server ability score did not rise by5");
                                let update=&after["world"]["ability_buffs"][if *index==1{"108"}else{"107"}];
                                ensure!(update["score"]==status[ability]&&update["received_seconds"]==1200&&update["kind"]==5,"actual S6B/S6C receipt missing");
                            }
                            ensure!(before["inventory"]==after["inventory"],"ability changed inventory");
                            actions.push(json!({"index":index,"skill":if *index==2{42}else{26},"target":if *index==0{1}else{3},"before":before,"immediately_after_queue":immediate,"duplicate_error":duplicate,"after":after,"feedback":c.snapshot().skill_feedback,"elapsed_ms":sent.elapsed().as_millis()}));
                            next+=1;pending=None;
                        }
                    }else if next<4&&frame.movement_ready(&c.snapshot().world) {
                        let skill=if next==2{42}else{26};let target=if next==0{Some(1)}else{None};
                        let choice=m::native_skill_selection::choice(&c.snapshot().world,skill,target.and_then(|id|m::native_skill_selection::Target::capture(&c.snapshot().world,id)))?;
                        let before=authority(&c);c.cast(choice.skill,choice.target,frame)?;let immediate=authority(&c);ensure!(before==immediate,"client predicted ability/MP changes");
                        let duplicate=c.cast(choice.skill,choice.target,frame).expect_err("duplicate ability admitted").to_string();
                        pending=Some((next,Instant::now(),before,immediate,duplicate));
                    }else if next==4&&seen.len()==2 {
                        if recovery_walk.is_none(){
                            let w=&c.snapshot().world;let own=w.own_actor().context("recovering caster")?;
                            let (heading,target)=[2u8,4,6,0,1,3,5,7].into_iter().find_map(|heading|{
                                let target=m::native_world::destination(own.x,own.y,heading).ok()?;
                                (frame.navigation.can_step(own.x.into(),own.y.into(),heading)&&!m::native_obstacles::blocks(w,target.0,target.1)
                                    &&!w.actors.values().any(|a|a.id!=own.id&&!a.dead&&(a.x,a.y)==target)).then_some((heading,target))
                            }).context("no free recovery step")?;
                            let plan=json!({"origin":[own.x,own.y],"target":[target.0,target.1],"heading":heading,"local_corrections":w.local_corrections});
                            std::fs::write(root.join("recovery-walk-target.json"),serde_json::to_vec_pretty(&plan)?)?;
                            c.move_checked(heading,&frame.navigation)?;recovery_walk=Some((Instant::now(),plan));
                        }else{
                            let (sent,plan)=recovery_walk.as_ref().unwrap();ensure!(sent.elapsed()<Duration::from_secs(8),"8s refusal recovery movement bound");
                            if root.join("a-walk-observed").exists()&&frame.movement_ready(&c.snapshot().world){
                                let own=c.snapshot().world.own_actor().context("walked caster")?;
                                ensure!(json!([own.x,own.y])==plan["target"]&&json!(c.snapshot().world.local_corrections)==plan["local_corrections"],"recovery move corrected");
                                if !finished{std::fs::write(root.join("b-finished"),b"complete")?;finished=true;}
                                if root.join("a-finished").exists(){return Ok(());}
                            }
                        }
                    }
                }
            },
            Phase::Failed|Phase::Closed=>anyhow::bail!("controller ended early: {previous}"),_=>{}
        }
        thread::sleep(Duration::from_millis(3));
    }})();
    std::fs::write(root.join(format!("{role}-result.json")),serde_json::to_vec_pretty(&json!({"role":role,"initial":initial,"actions":actions,"final":authority(&c),"pictures":seen,"trace":trace,"recovery_walk":recovery_walk.as_ref().map(|(_,plan)|plan),"error":outcome.as_ref().err().map(|e|format!("{e:#}")),"actual_cast_requests":if role=="b"{next}else{0},"normal_GUI_verified":false,"physical_audio_verified":false}))?)?;
    view.cancel();c.cancel();let end=Instant::now()+Duration::from_secs(3);
    while (!c.is_quiescent()||!view.quiescent())&&Instant::now()<end {
        c.poll();let s=c.snapshot();view.tick(Path::new(r"D:\lineage開發\0-客戶端"),c.transport().snapshot().epoch,&s.world,&s.environment,false);thread::sleep(Duration::from_millis(3));
    }
    ensure!(c.is_quiescent()&&view.quiescent(),"existing3s shutdown bound");
    std::fs::write(root.join(format!("{role}-shutdown.json")),serde_json::to_vec_pretty(&json!({"controller_quiescent":c.is_quiescent(),"renderer_quiescent":view.quiescent()}))?)?;
    outcome
}
#[test]#[ignore="requires owned current75 loopback harness and original resources"]
fn actual_two_ability_buffs_self_and_nonclan_target(){
    let root=PathBuf::from(std::env::var("LITO_ABILITY_BUFF_LIVE_ROOT").unwrap()).canonicalize().unwrap();
    assert!(root.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/native44FE-extra-finite-effects").canonicalize().unwrap()));
    let endpoint:SocketAddr=std::fs::read_to_string(root.join("endpoint.txt")).unwrap().trim().parse().unwrap();assert!(endpoint.ip().is_loopback()&&endpoint.port()!=0);
    let results=thread::scope(|scope|{let a=scope.spawn(||peer("a",endpoint,&root));let b=scope.spawn(||peer("b",endpoint,&root));(a.join().unwrap(),b.join().unwrap())});results.0.unwrap();results.1.unwrap();
}
