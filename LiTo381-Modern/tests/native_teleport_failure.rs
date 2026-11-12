//! Opt-in real-server boundary; no server position, clocks or authority are edited.
use lito_modern::modern_lito::{native_login::{LoginController,LoginPhase as P,Snapshot},native_connection::Limits,native_world_scene::Model,native_world};
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,sync::atomic::AtomicBool,thread,time::{Duration,Instant}};
#[path = "../src/modern_lito/native_teleport_registry.rs"]
mod recovery_registry;
// Server-owned stage/source TTL is 30s. These are observation waits, never
// replacements for a server clock or the client's 8s response deadline.
const STAGED_EXPIRY_OBSERVATION: Duration = Duration::from_secs(33);
const SOURCE_EXPIRY_OBSERVATION: Duration = Duration::from_secs(31);
const PEER_WAIT_SECONDS: u64 = 60;
const REQUESTED_SOURCE: &str = "測試乙";

fn select_requested_source(c: &mut LoginController) -> Result<()> {
 let index=c.snapshot().sources.iter().position(|s|s.name==REQUESTED_SOURCE)
  .context("requested recovery source not advertised")?;
 c.select_source(index)
}

fn observe_source_expiry(c: &mut LoginController) -> Result<u128> {
 let start=Instant::now();let epoch=c.transport().snapshot().epoch;
 while start.elapsed()<SOURCE_EXPIRY_OBSERVATION {
  c.poll();let s=c.snapshot();
  ensure!(s.phase==P::RedirectReady && s.selected_source.as_deref()==Some(REQUESTED_SOURCE)
   && s.selected_character.is_none() && s.inventory.owner.is_none()
   && s.inventory.items.is_empty() && s.world.actors.is_empty() && !s.world.ready()
   && s.joined_room.is_none() && s.merchant.owner.is_none() && s.services.is_none()
   && c.transport().snapshot().epoch==epoch,
   "source expiry wait changed connection or published authority");
  thread::sleep(Duration::from_millis(20));
 }
 Ok(start.elapsed().as_millis())
}
fn wait(c:&mut LoginController,seconds:u64,ok:impl Fn(&Snapshot)->bool)->Result<()>{
 let end=Instant::now()+Duration::from_secs(seconds);
 loop{c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"client failed: {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(())}ensure!(Instant::now()<end,"timeout at {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(10));}
}
fn walk(c:&mut LoginController,model:&mut Model,npc:u32,cancel:&AtomicBool,start:Instant)->Result<Vec<Value>>{
 let w=&c.snapshot().world;let own=w.own_actor().context("own actor")?;let target=w.actors.get(&npc).context("NPC absent")?;
 let nav=model.navigation();let route=nav.route_to_adjacent_avoiding([own.x,own.y],[target.x,target.y],|x,y|w.actors.values().any(|a|a.id!=own.id&&!a.dead&&a.draw_state.visible(false)&&a.x==x&&a.y==y))?;
 let mut steps=vec![];
 for heading in route{
  let end=Instant::now()+Duration::from_secs(8);
  loop{c.poll();ensure!(c.snapshot().phase==P::BattlePending,"left rental world");if model.draw_at(&c.snapshot().world,cancel,start.elapsed().as_nanos() as u64)?.movement_ready(&c.snapshot().world){break}ensure!(Instant::now()<end,"movement ready timeout");thread::sleep(Duration::from_millis(10));}
  let own=c.snapshot().world.own_actor().unwrap();let before=[own.x,own.y];let (x,y)=native_world::destination(own.x,own.y,heading)?;
  c.move_checked(heading,&nav)?;let end=Instant::now()+Duration::from_secs(8);
  loop{c.poll();ensure!(c.snapshot().phase==P::BattlePending,"left rental world");if model.draw_at(&c.snapshot().world,cancel,start.elapsed().as_nanos() as u64)?.movement_ready(&c.snapshot().world){break}ensure!(Instant::now()<end,"movement acknowledgement timeout");thread::sleep(Duration::from_millis(10));}
  let own=c.snapshot().world.own_actor().unwrap();ensure!([own.x,own.y]==[x,y],"movement corrected");steps.push(json!({"before":before,"after":[x,y],"heading":heading}));
 }
 Ok(steps)
}

fn failure_player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()>{
 let scenario=std::env::var("LITO_TELEPORT_SCENARIO").unwrap_or_else(|_|"timeout".into());
 // This entry exercises the source TTL in addition to the staged TTL. The
 // other historical repeated-cancel scenarios remain a separate T023 slice.
 let source_expiry=scenario=="cancel-expiry-source";
 let wrong_service=scenario=="wrong-service";let refused=scenario=="refused";let cancel_expiry=scenario=="cancel-expiry"||source_expiry;let cancel_retry=scenario=="cancel-retry";
 let mut manual_selection=false;let mut closed_samples=0u32;let mut expiry_wait_ms=0u128;let mut source_wait_ms=0u128;
 let mut c=LoginController::default();let cancel=AtomicBool::new(false);let start=Instant::now();
 let result=(||->Result<()>{
  c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
  wait(&mut c,15,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;
  wait(&mut c,15,|s|s.phase==P::SourcesReady)?;select_requested_source(&mut c)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(15);
  while c.snapshot().phase!=P::CharactersReady {c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"GAME bootstrap failed");thread::sleep(Duration::from_millis(10));}
  c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected)?;c.select_rental_set(1)?;
  wait(&mut c,15,|s|s.phase==P::RentalReady)?;c.join_rental_room(0)?;wait(&mut c,15,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&1812987996))?;
  if role=="b"{wait(&mut c,if cancel_expiry{120}else{PEER_WAIT_SECONDS},|_|root.join("a-failure-complete").exists())?;}
  let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&c.snapshot().world,&cancel)?;
  let npc=1812987996;let steps=walk(&mut c,&mut model,npc,&cancel,start)?;
  c.rental_npc_talk(npc)?;wait(&mut c,10,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;

  let before=c.snapshot().clone();let old_epoch=c.transport().snapshot().epoch;
  let action=if role=="a"{"LT_TELEPORT_B"}else{"LT_TELEPORT_C"};let requested=Instant::now();c.rental_npc_action(npc,action)?;
  let end=Instant::now()+Duration::from_secs(15);
  if cancel_expiry||cancel_retry {
   loop{c.poll();if c.snapshot().phase==P::GameConnecting&&c.transport().snapshot().epoch==old_epoch+1{break}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"destination connecting not reached");thread::sleep(Duration::from_millis(10));}
   c.cancel();let end=Instant::now()+Duration::from_secs(3);
   loop{c.poll();if c.snapshot().phase==P::Closed&&c.is_quiescent(){break}ensure!(Instant::now()<end,"cancel did not reclaim destination");thread::sleep(Duration::from_millis(10));}
  } else {
   loop{c.poll();if c.snapshot().phase==P::Failed&&c.is_quiescent(){break}ensure!(Instant::now()<end,"failure wait exceeded");thread::sleep(Duration::from_millis(10));}
  }
  let failed=c.snapshot().clone();let elapsed=requested.elapsed().as_millis();let failed_epoch=c.transport().snapshot().epoch;
  ensure!(failed_epoch==old_epoch+1&&failed.world.actors.is_empty()&&!failed.world.ready()&&failed.inventory.owner.is_none()&&failed.selected_character.is_none()&&failed.joined_room.is_none()&&failed.merchant.owner.is_none(),"failed destination retained previous authority");
  if cancel_expiry||cancel_retry {
   ensure!(failed.phase==P::Closed&&failed.error.is_none()&&elapsed<5000,"cancel became a timeout");
   // Real monotonic time; no server clock or staged authority is rewritten.
   while requested.elapsed()<if cancel_expiry{STAGED_EXPIRY_OBSERVATION}else{Duration::from_millis(600)} {
    c.poll();ensure!(c.snapshot().phase==P::Closed&&c.is_quiescent()&&c.transport().snapshot().epoch==failed_epoch&&c.snapshot().world.actors.is_empty()&&c.snapshot().inventory.owner.is_none(),"cancelled handoff resumed in background");
    closed_samples+=1;thread::sleep(Duration::from_millis(20));
   }
   expiry_wait_ms=requested.elapsed().as_millis();
   if cancel_expiry {std::fs::write(root.join(format!("{role}-expiry-observed")),b"natural wait complete")?;}
  } else if wrong_service {ensure!(failed.error.is_some()&&elapsed<15000,"wrong LOGIN destination did not fail boundedly");}
  else if refused {ensure!(failed.error.as_ref().is_some_and(|e|e.contains("native TCP connect"))&&elapsed<5000,"closed destination was not bounded by TCP connect: {:?}",failed.error);}
  else {ensure!(failed.error.is_some()&&elapsed>=5000&&elapsed<15000,"failure did not remain bounded");}
  ensure!(c.rental_npc_action(npc,action).is_err()&&c.enter_channel(0).is_err(),"failed session accepts gameplay");
  c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;wait(&mut c,15,|s|s.phase==P::ReadyForLogin)?;
  c.login(account,"TestOnlyPassword")?;wait(&mut c,15,|s|s.phase==P::SourcesReady)?;
  let recovery=c.snapshot().clone();ensure!(c.transport().snapshot().epoch==failed_epoch+1,"recovery connection epoch missing");
  let recovery_epoch=c.transport().snapshot().epoch;
  select_requested_source(&mut c)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;
  if source_expiry {source_wait_ms=observe_source_expiry(&mut c)?;}
  c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(15);
  loop {c.poll();
   if cancel_expiry&&c.snapshot().phase==P::CharactersReady&&!manual_selection {
    ensure!(c.snapshot().selected_character.is_none()&&c.snapshot().inventory.items.is_empty(),"expired handoff auto-selected stale authority");
    c.select_character(0)?;manual_selection=true;
   }
   if c.snapshot().phase==P::CharacterSelected&&c.snapshot().inventory.batches>0{break}
   if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
   ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"recovery GAME continuation failed: {:?}",c.snapshot().error);thread::sleep(Duration::from_millis(10));}
  ensure!(c.snapshot().selected_character==before.selected_character&&c.transport().snapshot().epoch==recovery_epoch+1,"recovery character/epoch mismatch");
  ensure!(!cancel_expiry||(manual_selection && expiry_wait_ms>=33000),"expired handoff skipped natural wait or fresh character selection");
  ensure!(!source_expiry||source_wait_ms>=31000,"source handoff skipped natural wait");
  ensure!(c.snapshot().selected_source.as_deref()==Some(REQUESTED_SOURCE),"recovery changed selected source");
  let continued=c.snapshot().clone();c.select_rental_set(1)?;wait(&mut c,15,|s|s.phase==P::RentalReady)?;c.join_rental_room(0)?;
  wait(&mut c,15,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&npc))?;
  let reentered=c.snapshot().clone();ensure!(reentered.world.generation>before.world.generation&&reentered.world.map_id==Some(500),"reentry world generation missing");
  let mut recovered_model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&reentered.world,&cancel)?;
  let end=Instant::now()+Duration::from_secs(8);
  loop {c.poll();let frame=recovered_model.draw_at(&c.snapshot().world,&cancel,start.elapsed().as_nanos() as u64)?;if frame.movement_ready(&c.snapshot().world){break}if Instant::now()>=end{std::fs::write(root.join(format!("{role}-recovery-frame-error.json")),serde_json::to_vec_pretty(&frame.receipt)?)?;anyhow::bail!("recovery initial movement not presented")}thread::sleep(Duration::from_millis(10));}
  let nav=recovered_model.navigation();let own=c.snapshot().world.own_actor().unwrap();
  let heading=(0..8).find(|h|nav.can_step(i32::from(own.x),i32::from(own.y),*h)&&native_world::destination(own.x,own.y,*h).is_ok_and(|(x,y)|!c.snapshot().world.actors.values().any(|a|a.id!=own.id&&a.x==x&&a.y==y))).context("no open recovery step")?;
  let move_before=[own.x,own.y];let (x,y)=native_world::destination(own.x,own.y,heading)?;c.move_checked(heading,&nav)?;wait(&mut c,8,|s|s.world.own_actor().is_some_and(|a|[a.x,a.y]==[x,y]))?;
  let end=Instant::now()+Duration::from_secs(8);loop{c.poll();let frame=recovered_model.draw_at(&c.snapshot().world,&cancel,start.elapsed().as_nanos() as u64)?;if frame.movement_ready(&c.snapshot().world){break}if Instant::now()>=end{std::fs::write(root.join(format!("{role}-recovery-frame-error.json")),serde_json::to_vec_pretty(&frame.receipt)?)?;anyhow::bail!("recovery movement not presented")}thread::sleep(Duration::from_millis(10));}
  std::fs::write(root.join(format!("{role}-failure.json")),serde_json::to_vec_pretty(&json!({"role":role,"action":action,"steps":steps,"before":before,"failed":failed,"recovery":recovery,"continued":continued,"reentered":reentered,"recovery_move":{"before":move_before,"after":[x,y],"heading":heading},"old_epoch":old_epoch,"failed_epoch":failed_epoch,"recovery_epoch":recovery_epoch,"reentered_epoch":c.transport().snapshot().epoch,"elapsed_ms":elapsed,"scenario":scenario,"manual_selection":manual_selection,"closed_samples":closed_samples,"expiry_wait_ms":expiry_wait_ms,"source_wait_ms":source_wait_ms,"requested_source":REQUESTED_SOURCE,"full_game_reentry_verified":true}))?)?;
  std::fs::write(root.join(format!("{role}-failure-complete")),b"done")?;let other=if role=="a"{"b"}else{"a"};wait(&mut c,if cancel_expiry{120}else{PEER_WAIT_SECONDS},|_|root.join(format!("{other}-failure-complete")).exists())?;
  Ok(())
 })();
 if let Err(error)=&result {std::fs::write(root.join(format!("{role}-recovery-error.json")),serde_json::to_vec_pretty(&json!({"error":format!("{error:#}"),"snapshot":c.snapshot(),"transport_epoch":c.transport().snapshot().epoch}))?)?;}
 c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(10));}
 ensure!(c.is_quiescent(),"client worker not reclaimed");result
}
#[test]
#[ignore="requires owned formal no-response destination harness"]
fn destination_timeout_retires_old_authority_and_allows_fresh_login(){
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("timeout"));
 run_owned_harness();
}
#[test]
#[ignore="requires owned harness and natural 33s staged expiry observation"]
fn cancellation_retires_destination_and_expired_handoff_requires_fresh_selection(){
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("cancel-expiry"));
 run_owned_harness();
}
#[test]
#[ignore="requires owned E149 harness and natural 33s staged plus 31s source expiry"]
fn source_handoff_expiry_confirms_c8_before_manual_selection(){
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("cancel-expiry-source"));
 run_owned_harness();
}
#[test]
#[ignore="requires owned harness and immediate authenticated retry"]
fn cancellation_then_immediate_retry_consumes_staged_character_once(){
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("cancel-retry"));run_owned_harness();
}
#[test]
#[ignore="requires owned previously reserved loopback port and formal recovery"]
fn refused_destination_retires_authority_and_allows_fresh_login(){
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("refused"));run_owned_harness();
}
#[test]
#[ignore="requires owned real LOGIN destination and formal GAME recovery"]
fn wrong_service_destination_retires_authority_and_allows_fresh_login(){
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("wrong-service"));run_owned_harness();
}
fn run_owned_harness(){
 recovery_registry::checkout_binding().expect("current source/fixture binding");
 let root=std::path::PathBuf::from(std::env::var("LITO_TELEPORT_ROOT").expect("owned root"));assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));
 let ready:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();
 let expected="e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498";
 if let Ok(declared)=std::env::var("LITO_TELEPORT_REFERENCE_SHA"){assert_eq!(declared.to_ascii_lowercase(),expected);}
 assert_eq!(ready["jar_sha256"],expected);
 for name in ["port","town_port"] {let port=ready[name].as_u64().expect("owned listener port");assert!(!matches!(port,2000|3306),"forbidden {name}: {port}");}
 let peer:SocketAddr=format!("127.0.0.1:{}",ready["port"].as_u64().unwrap()).parse().unwrap();
 assert_eq!(ready["scenario"],std::env::var("LITO_TELEPORT_SCENARIO").unwrap());
 assert_eq!(ready["requested_source"],REQUESTED_SOURCE);
 assert_eq!(ready["default_source"],"測試甲");
 thread::scope(|scope|{let a=scope.spawn(||failure_player(&root,peer,"a","PeerFixture"));let b=scope.spawn(||failure_player(&root,peer,"b","PeerFixture2"));let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");});
}

// These run only encrypted synthetic loopback peers. They neither start Java
// nor access a database, resource tree or existing game process.
#[cfg(feature = "teleport-wire-test")]
mod recovery_wire {
 use lito_modern::modern_lito::native_login::teleport_wire_tests as cases;
 #[test]
 fn second_cancel_then_explicit_login() { cases::recovery_after_second_cancel_uses_explicit_login_and_source_once(); }
 #[test]
 fn c8_waits_for_complete_source_directory() { cases::recovery_c8_waits_for_complete_directory_and_never_replays_c9(); }
 #[test]
 fn absent_character_confirms_source() { cases::recovery_missing_character_refreshes_without_publishing_default_source(); }
 #[test]
 fn c8_timeout_and_cancellation_are_terminal() { cases::recovery_c8_timeout_is_bounded_and_cancel_cannot_restart_it(); }
 #[test]
 fn partial_inventory_never_retries_c8() { cases::recovery_partial_inventory_fails_without_c8_retry(); }
 #[test]
 fn natural_eight_second_deadlines() { cases::recovery_natural_eight_second_waits_do_not_publish_or_retry(); }
 #[test]
 fn direct_teleport_cannot_refresh_source() { cases::recovery_refresh_is_not_available_to_a_direct_teleport(); }
 #[test]
 fn natural_source_wait_then_complete_c8_and_manual_selection() {
  cases::recovery_after_source_wait_confirms_directory(super::observe_source_expiry);
 }
}
