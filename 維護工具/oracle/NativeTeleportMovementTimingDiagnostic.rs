//! Opt-in real-server boundary; no server position, clocks or authority are edited.
use lito_modern::modern_lito::{native_login::{LoginController,LoginPhase as P,Snapshot},native_connection::Limits,native_world_scene::Model,native_world};
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,sync::atomic::AtomicBool,thread,time::{Duration,Instant}};
fn wait(c:&mut LoginController,seconds:u64,ok:impl Fn(&Snapshot)->bool)->Result<()>{
 let end=Instant::now()+Duration::from_secs(seconds);
 loop{c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"client failed: {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(())}ensure!(Instant::now()<end,"timeout at {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(10));}
}

fn movement_observation(c:&LoginController,model:&mut Model,cancel:&AtomicBool,start:Instant,rows:&mut std::collections::VecDeque<Value>,stage:&str)->Result<bool>{
 let draw_started=Instant::now(); let frame=model.draw_at(&c.snapshot().world,cancel,start.elapsed().as_nanos() as u64)?;
 let ready=frame.movement_ready(&c.snapshot().world);
 if rows.len()==16 {rows.pop_front();}
 rows.push_back(json!({"stage":stage,"elapsed_ns":start.elapsed().as_nanos(),"draw_us":draw_started.elapsed().as_micros(),"movement_ready":ready,"position":c.snapshot().world.own_actor().map(|a|[a.x,a.y]),"phase":c.snapshot().phase,"render":frame.receipt}));
 Ok(ready)
}
fn dump_movement(c:&LoginController,rows:&std::collections::VecDeque<Value>,stage:&str)->Result<()>{
 let root=std::path::PathBuf::from(std::env::var("LITO_TELEPORT_ROOT")?);
 let own=c.snapshot().world.own_actor().map(|a|a.id).unwrap_or(0);
 std::fs::write(root.join(format!("movement-diagnostic-{own}-{stage}.json")),serde_json::to_vec_pretty(&json!({"stage":stage,"state":c.snapshot(),"last16frames":rows,"deadline_seconds_unchanged":8}))?)?;
 Ok(())
}

fn walk(c:&mut LoginController,model:&mut Model,npc:u32,cancel:&AtomicBool,start:Instant)->Result<Vec<Value>>{
 let w=&c.snapshot().world;let own=w.own_actor().context("own actor")?;let target=w.actors.get(&npc).context("NPC absent")?;
 let nav=model.navigation();let route=nav.route_to_adjacent_avoiding([own.x,own.y],[target.x,target.y],|x,y|w.actors.values().any(|a|a.id!=own.id&&!a.dead&&a.draw_state.visible(false)&&a.x==x&&a.y==y))?;
 let mut steps=vec![];let mut rows=std::collections::VecDeque::new();
 for heading in route{
  let end=Instant::now()+Duration::from_secs(8);
  loop{c.poll();if c.snapshot().phase!=P::BattlePending{dump_movement(c,&rows,"left-world")?;}ensure!(c.snapshot().phase==P::BattlePending,"left rental world");if movement_observation(c,model,cancel,start,&mut rows,"before")?{break}if Instant::now()>=end{dump_movement(c,&rows,"before")?;}ensure!(Instant::now()<end,"movement ready timeout");thread::sleep(Duration::from_millis(10));}
  let own=c.snapshot().world.own_actor().unwrap();let before=[own.x,own.y];let (x,y)=native_world::destination(own.x,own.y,heading)?;
  c.move_checked(heading,&nav)?;let end=Instant::now()+Duration::from_secs(8);
  loop{c.poll();if c.snapshot().phase!=P::BattlePending{dump_movement(c,&rows,"left-world")?;}ensure!(c.snapshot().phase==P::BattlePending,"left rental world");if movement_observation(c,model,cancel,start,&mut rows,"after")?{break}if Instant::now()>=end{dump_movement(c,&rows,"after")?;}ensure!(Instant::now()<end,"movement acknowledgement timeout");thread::sleep(Duration::from_millis(10));}
  let own=c.snapshot().world.own_actor().unwrap();ensure!([own.x,own.y]==[x,y],"movement corrected");steps.push(json!({"before":before,"after":[x,y],"heading":heading}));
 }
 dump_movement(c,&rows,"walk-completed")?;
 Ok(steps)
}

fn player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()>{
 let mut c=LoginController::default();let cancel=AtomicBool::new(false);let start=Instant::now();
 let result=(||->Result<()>{
  c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
  wait(&mut c,15,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;
  wait(&mut c,15,|s|s.phase==P::SourcesReady)?;c.select_source(0)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(15);
  while c.snapshot().phase!=P::CharactersReady {c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"GAME bootstrap failed");thread::sleep(Duration::from_millis(10));}
  c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected)?;c.select_rental_set(1)?;
  wait(&mut c,15,|s|s.phase==P::RentalReady)?;c.join_rental_room(0)?;wait(&mut c,15,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&1812987996))?;
  if role=="b"{wait(&mut c,60,|_|root.join("a-transferred").exists())?;}
  let load_started=Instant::now();
  let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&c.snapshot().world,&cancel)?;
  std::fs::write(root.join(format!("{role}-model-load-timing.json")),serde_json::to_vec_pretty(&json!({"load_us":load_started.elapsed().as_micros(),"player_elapsed_ms":start.elapsed().as_millis(),"world_actor_count":c.snapshot().world.actors.len(),"world":c.snapshot().world}))?)?;
  let npc=1812987996;let steps=walk(&mut c,&mut model,npc,&cancel,start)?;
  c.rental_npc_talk(npc)?;wait(&mut c,10,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;
  let same_endpoint=std::env::var("LITO_TELEPORT_SCENARIO").as_deref()==Ok("same-endpoint");
  let before=c.snapshot().clone();let old_epoch=c.transport().snapshot().epoch;
  let old_peer=c.transport().snapshot().negotiated.as_ref().unwrap().peer;
  let old_frame=model.draw_at(&before.world,&cancel,start.elapsed().as_nanos() as u64)?;
  let action=if role=="a"{"LT_TELEPORT_B"}else{"LT_TELEPORT_C"};c.rental_npc_action(npc,action)?;
  let end=Instant::now()+Duration::from_secs(20);let mut transitions=vec![];let mut last=P::BattlePending;
  loop {
   c.poll();let s=c.snapshot();if last!=s.phase{transitions.push(json!({"phase":s.phase,"epoch":c.transport().snapshot().epoch,"world_ready":s.world.ready(),"actors":s.world.actors.len(),"inventory_owner":s.inventory.owner,"services":s.services}));last=s.phase;}
   ensure!(!matches!(s.phase,P::Failed|P::Closed),"transfer failed: {:?}",s.error);
   if s.phase==P::CharacterSelected&&s.inventory.batches>0{break}
   if s.notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
   ensure!(Instant::now()<end,"transfer timeout {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(10));
  }
  let destination=c.snapshot().clone();let new_epoch=c.transport().snapshot().epoch;
  ensure!(new_epoch==old_epoch+1&&destination.selected_character==before.selected_character,"connection or character continuity mismatch");
  ensure!(destination.joined_room.is_none()&&(same_endpoint||destination.services.is_none())&&destination.rental_set.is_none()&&destination.world.actors.is_empty()&&!destination.world.ready(),"old scene retained");
  let new_peer=c.transport().snapshot().negotiated.as_ref().unwrap().peer;
  ensure!((new_peer==old_peer)==same_endpoint,"destination address/scenario mismatch");
  ensure!(!old_frame.input_ready(&destination.world),"old frame authorized destination input");
  ensure!(destination.inventory.owner.as_ref().is_some_and(|o|o.connection==new_epoch)&&!destination.inventory.items.is_empty(),"destination inventory ownership");
  ensure!(c.select_character(0).is_err()&&c.rental_npc_action(npc,action).is_err(),"old selector/NPC replay admitted");
  std::fs::write(root.join(format!("{role}-transferred")),b"done")?;
  let other=if role=="a"{"b"}else{"a"};wait(&mut c,60,|_|root.join(format!("{other}-transferred")).exists())?;
  c.enter_channel(0)?;wait(&mut c,15,|s|s.phase==P::LobbyReady)?;
  if role=="a"{
   c.create_room(&lito_modern::modern_lito::native_room::Settings{name:"傳送接續",password:"",attributes:0,capacity:2,options:None})?;
   wait(&mut c,15,|s|s.phase==P::RoomReady)?;
   std::fs::write(root.join("room-id"),c.snapshot().joined_room.as_ref().unwrap().id.to_string())?;
   wait(&mut c,20,|s|s.joined_room.as_ref().is_some_and(|r|r.members.len()==2))?;
  } else {
   wait(&mut c,20,|_|root.join("room-id").exists())?;
   let id=std::fs::read_to_string(root.join("room-id"))?.parse::<u32>()?;
   wait(&mut c,15,|s|s.rooms.iter().any(|r|r.id==id))?;let index=c.snapshot().rooms.iter().position(|r|r.id==id).unwrap();c.join_room(index,"")?;
   wait(&mut c,15,|s|s.phase==P::RoomReady&&s.joined_room.as_ref().is_some_and(|r|r.members.len()==2))?;
   c.set_force(1)?;wait(&mut c,15,|s|s.phase==P::RoomReady)?;
  }
  wait(&mut c,20,|s|s.joined_room.as_ref().is_some_and(|r|r.members.len()==2&&r.members.iter().any(|m|m.force==0)&&r.members.iter().any(|m|m.force==1)))?;
  c.set_ready(true)?;wait(&mut c,20,|s|s.phase==P::BattlePending&&s.world.ready())?;
  let battle=c.snapshot().clone();ensure!(battle.world.generation>before.world.generation&&battle.world.map_id!=Some(500),"new world generation missing");
  let battle_model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&battle.world,&cancel)?;
  let _=battle_model;
  std::fs::write(root.join(format!("{role}-battle-ready")),b"done")?;
  wait(&mut c,20,|_|root.join(format!("{other}-battle-ready")).exists())?;
  std::fs::write(root.join(format!("{role}-handoff.json")),serde_json::to_vec_pretty(&json!({"role":role,"action":action,"steps":steps,"old_epoch":old_epoch,"new_epoch":new_epoch,"before":before,"destination":destination,"transitions":transitions,"battle":battle,"old_frame_rejected":true,"same_endpoint":same_endpoint,"old_peer":old_peer.to_string(),"new_peer":new_peer.to_string()}))?)?;
  Ok(())
 })();
 c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(10));}
 ensure!(c.is_quiescent(),"client worker not reclaimed");result
}
#[test]
#[ignore="requires owned formal multi-endpoint loopback harness"]
fn actual_two_players_teleport_and_continue_on_new_game_endpoints(){
 let root=std::path::PathBuf::from(std::env::var("LITO_TELEPORT_ROOT").expect("owned root"));assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));
 let ready:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();let expected=std::env::var("LITO_TELEPORT_REFERENCE_SHA").unwrap_or_else(|_|"bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43".into());assert_eq!(ready["jar_sha256"],expected);
 let peer:SocketAddr=format!("127.0.0.1:{}",ready["port"].as_u64().unwrap()).parse().unwrap();
 thread::scope(|scope|{let a=scope.spawn(||player(&root,peer,"a","PeerFixture"));let b=scope.spawn(||player(&root,peer,"b","PeerFixture2"));let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");});
}
