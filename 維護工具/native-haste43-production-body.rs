use anyhow::{ensure,Context,Result};
use lito_modern::modern_lito::native_login::{LoginController,LoginPhase};
use lito_modern::modern_lito::{native_connection::Limits,native_world_scene::Model};
use serde_json::{json,Value};
use std::{path::{Path,PathBuf},time::{Instant,Duration},sync::atomic::AtomicBool};
fn save(root:&Path,name:&str,value:&Value)->Result<()> {
    std::fs::write(root.join(name),serde_json::to_vec_pretty(value)?)?;Ok(())
}
fn run(peer:std::net::SocketAddr,root:&Path,scenario:&str)->Result<()> {
    let refused=scenario=="mp-refused";let cancel_retirement=scenario.starts_with("cancel-");
    ensure!(peer.ip().is_loopback()&&peer.port()!=0,"isolated loopback required");
    let mut clients=[LoginController::default(),LoginController::default()];
    let outcome=(||->Result<()> {
        for c in &mut clients {c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;}
        let start=Instant::now();let mut submitted=[false;2];let mut force=[false;2];
        let mut models:[Option<Model>;2]=[None,None];let clock=Instant::now();
        let cancel=AtomicBool::new(false);let mut stage=0usize;let mut expected_mp=None;
        let mut before_casts=0;let mut observations=Vec::new();let mut previous=[Value::Null,Value::Null];
        let mut selection=lito_modern::modern_lito::native_skill_selection::Model::default();
        let mut request_time:Option<Instant>=None;
        loop {
            ensure!(start.elapsed()<Duration::from_secs(120),"live haste probe exceeded original declared120s");
            for i in 0..2 {
                let c=&mut clients[i];c.poll();
                let s=c.snapshot();let o=json!({"role":i,"phase":s.phase,"error":s.error,"room":s.joined_room,"world_generation":s.world.generation,"mp":s.world.mp,"casts":s.world.own_cast_packets,"actors":s.world.actors,"learned":s.world.learned_skills});
                if o!=previous[i] {ensure!(observations.len()<1024,"trace bound");observations.push(json!({"ms":start.elapsed().as_millis(),"state":o}));previous[i]=o;save(root,"client-trace.json",&json!(observations))?;}
                ensure!(s.error.is_none(),"client failed: {:?}",s.error);
                if s.notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
                match c.snapshot().phase {
                    LoginPhase::ReadyForLogin if !submitted[i]=>{c.login(if i==0{"PeerFixture"}else{"PeerFixture2"},"TestOnlyPassword")?;submitted[i]=true;},
                    LoginPhase::ReadyForLogin=>anyhow::bail!("test account rejected"),
                    LoginPhase::SourcesReady=>c.select_source(1)?,
                    LoginPhase::RedirectReady=>{ensure!(c.snapshot().redirect.context("redirect")?.ip().is_loopback(),"external redirect");c.begin_game()?;},
                    LoginPhase::CharactersReady=>c.select_character(0)?,
                    LoginPhase::CharacterSelected=>c.enter_channel(0)?,
                    LoginPhase::LobbyReady=>{if !c.snapshot().rooms.is_empty(){c.join_room(0,"")?;}},
                    LoginPhase::RoomReady=>{
                        let s=c.snapshot();let room=s.joined_room.as_ref().context("room")?;
                        let own=room.members.iter().find(|m|Some(m.name.as_str())==s.selected_character.as_deref()).context("membership")?;
                        if own.force!=i as u8&&!force[i]{c.set_force(i as u8)?;force[i]=true;}
                        else if own.force==i as u8&&own.state==0&&room.members.len()==2&&room.members.iter().any(|m|m.force!=i as u8){c.set_ready(true)?;}
                    },
                    LoginPhase::BattlePending=>{},
                    _=>{},
                }
            }
            if !clients.iter().all(|c|c.snapshot().phase.active_world()&&c.snapshot().world.ready()&&c.snapshot().world.integrity_challenge.is_some()&&[1,3].iter().all(|id|c.snapshot().world.actors.contains_key(id))){std::thread::sleep(Duration::from_millis(4));continue;}
            for i in 0..2 {if models[i].is_none(){models[i]=Some(Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&clients[i].snapshot().world,&cancel)?);}}
            let frames=[models[0].as_mut().unwrap().draw_at(&clients[0].snapshot().world,&cancel,clock.elapsed().as_nanos() as u64)?,models[1].as_mut().unwrap().draw_at(&clients[1].snapshot().world,&cancel,clock.elapsed().as_nanos() as u64)?];
            if cancel_retirement&&stage==2 {
                if frames.iter().all(|f|f.receipt["skill_effects"]["active"]==0&&f.receipt["skill_effects"]["started"]==1) {
                    save(root,"finite-effect-before-cancel.json",&json!({"caster":clients[1].snapshot(),"observer":clients[0].snapshot(),"render_receipts":[frames[0].receipt,frames[1].receipt],"elapsed_ms_since_cast":request_time.unwrap().elapsed().as_millis()}))?;
                    save(root,"client-result.json",&json!({"status":"current_haste43_cancel_retirement_exercised","actual_casts":1,"actual_requests":1,"finite755_retired_while_speed1":true,"normal_product_modified":false,"GUI_verified":false}))?;return Ok(());
                }
                std::thread::sleep(Duration::from_millis(8));continue;
            }
            if stage==0||stage==2 {
                let w=&clients[1].snapshot().world;ensure!(w.own_actor().context("caster")?.id==3&&w.learned_skills.contains(&43),"original Mage book must contain43");
                selection.sync(clients[1].transport().snapshot().epoch,w,true);selection.select(Some(43))?;
                ensure!(selection.rows().iter().any(|r|r.id==43&&r.label.contains("加速術")),"current production label");
                let target=if stage==0{None}else{Some(lito_modern::modern_lito::native_skill_selection::Target::capture(w,1).context("peer target")?)};
                let choice=selection.choice(clients[1].transport().snapshot().epoch,w,target)?;
                let before=serde_json::to_value(w)?;let mp=w.mp.context("authority mana")?[0];let casts=w.own_cast_packets;
                if refused {ensure!(stage==0&&mp==2&&w.mp.unwrap()[1]==2,"original low-MP fixture required");}
                if !frames[1].movement_ready(w){std::thread::sleep(Duration::from_millis(8));continue;}
                let cast_started=Instant::now();let cast=clients[1].cast(choice.skill,choice.target,&frames[1]);
                if let Err(e)=cast {if e.to_string().contains("等待施法動作完成"){std::thread::sleep(Duration::from_millis(8));continue;}return Err(e);}
                ensure!(serde_json::to_value(&clients[1].snapshot().world)?==before,"request predicted authority");
                let queued=clients[1].transport().queued_tx_counter();
                ensure!(clients[1].cast(43,choice.target,&frames[1]).is_err(),"pending duplicate accepted");
                ensure!(clients[1].transport().queued_tx_counter()==queued,"pending duplicate queued wire");
                request_time=Some(cast_started);
                save(root,if refused{"mp-request.json"}else if stage==0{"self-request.json"}else{"peer-request.json"},&json!({"target":choice.target,"before":before,"immediate":clients[1].snapshot(),"request_only_no_authority_prediction":true,"immediate_duplicate_refused_without_new_tx":true}))?;
                expected_mp=Some(mp);before_casts=casts;stage+=1;
            } else {
                if refused {
                    let caster=clients[1].snapshot();
                    ensure!(caster.world.mp==Some([2,2])&&caster.world.own_cast_packets==before_casts,"low-MP request altered mana or cast count");
                    ensure!(clients.iter().all(|c|c.snapshot().world.actors.get(&3).is_some_and(|a|a.speed==0)&&!c.snapshot().world.effects.iter().any(|e|e.graphics==755)),"low-MP request produced haste");
                    if caster.skill_feedback==lito_modern::modern_lito::native_skills::Feedback::NoResponse {
                        let elapsed=request_time.unwrap().elapsed().as_millis();ensure!(elapsed>=2000,"original2s pending budget not observed");
                        save(root,"mp-confirmed.json",&json!({"caster":caster,"observer":clients[0].snapshot(),"elapsed_ms_since_request":elapsed,"feedback":"no_response","MP_unchanged":[2,2],"speed_unchanged":true,"no755":true}))?;
                        save(root,"client-result.json",&json!({"status":"current_haste43_low_mp_no_response_observed","actual_casts":0,"actual_requests":1,"pending_duplicate_refused":true,"prediction_absent":true,"normal_product_modified":false,"GUI_verified":false}))?;return Ok(());
                    }
                    std::thread::sleep(Duration::from_millis(8));continue;
                }
                let target=if stage==1{3}else{1};let caster=&clients[1].snapshot().world;
                let ready=clients.iter().all(|c|{let w=&c.snapshot().world;w.actors.get(&target).is_some_and(|a|a.speed==1)&&w.effects.iter().any(|e|e.graphics==755&&e.target.is_some_and(|t|t.0==target))});
                if ready&&caster.mp.context("mana")?[0]<expected_mp.unwrap()&&caster.own_cast_packets>before_casts {
                    for (i,c) in clients.iter().enumerate(){let actor=&c.snapshot().world.actors[&target];ensure!(actor.speed_seconds==if (i==0&&target==1)||(i==1&&target==3){1200}else{0},"formal per-recipient haste duration");}
                    save(root,if stage==1{"self-confirmed.json"}else{"peer-confirmed.json"},&json!({"target":target,"mana_before":expected_mp,"mana_after":caster.mp,"caster":clients[1].snapshot(),"observer":clients[0].snapshot(),"render_receipts":[frames[0].receipt,frames[1].receipt]}))?;
                    stage+=1;if stage==4{save(root,"client-result.json",&json!({"status":"current_haste43_two_formal_casts_observed","self_and_peer":true,"original_Mage_book_retained":true,"pending_duplicate_refused_twice":true,"prediction_absent_twice":true,"per_recipient_speed_duration":true,"actual_casts":2,"normal_product_modified":false,"GUI_verified":false}))?;return Ok(());}
                }
            }
            std::thread::sleep(Duration::from_millis(8));
        }
    })();
    for c in &mut clients {c.cancel();}
    let end=Instant::now()+Duration::from_secs(5);
    while !clients.iter().all(|c|c.is_quiescent())&&Instant::now()<end{for c in &mut clients{c.poll();}std::thread::sleep(Duration::from_millis(5));}
    ensure!(clients.iter().all(|c|c.is_quiescent()),"owned client transport not terminal");
    save(root,"after-cancel.json",&json!({"caster":clients[1].snapshot(),"observer":clients[0].snapshot(),"both_quiescent":true}))?;
    if cancel_retirement {ensure!(clients.iter().all(|c|{let s=c.snapshot();s.world.map_id.is_none()&&s.world.actors.is_empty()&&s.world.learned_skills.is_empty()&&s.world.effects.is_empty()&&s.joined_room.is_none()&&s.services.is_none()&&s.selected_character.is_none()&&s.inventory.owner.is_none()}),"cancel retained previous world/room/skills after owned transport quiescence");}
    outcome
}
fn main(){let args:Vec<_>=std::env::args().collect();let root=PathBuf::from(&args[2]);let result=run(args[1].parse().unwrap(),&root,args.get(3).map_or("self-peer",String::as_str));if let Err(e)=result{let _=save(&root,"error.json",&json!({"error":format!("{e:#}"),"GUI_verified":false}));eprintln!("{e:#}");std::process::exit(1);}}
