//! Opt-in real-server boundary; no server position, clocks or authority are edited.
use lito_modern::modern_lito::{native_login::{LoginController,LoginPhase as P,Snapshot},native_connection::Limits,native_world_scene::Model,native_world};
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,sync::atomic::AtomicBool,thread,time::{Duration,Instant}};
fn wait(c:&mut LoginController,seconds:u64,ok:impl Fn(&Snapshot)->bool)->Result<()>{
 let end=Instant::now()+Duration::from_secs(seconds);
 loop{c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"client failed: {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(())}ensure!(Instant::now()<end,"timeout at {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(10));}
}
fn observe(root:&Path,role:&str,label:&str)->Result<Value>{
 std::fs::write(root.join(format!("{role}-observe-request")),label)?;
 let file=root.join(format!("{role}-observe-{label}.json"));let end=Instant::now()+Duration::from_secs(3);
 loop{if let Ok(b)=std::fs::read(&file){if let Ok(v)=serde_json::from_slice(&b){return Ok(v)}}ensure!(Instant::now()<end,"passive observer timeout");thread::sleep(Duration::from_millis(10));}
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
fn player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()>{
 let mut c=LoginController::default();let mut stages=vec![];let cancel=AtomicBool::new(false);let start=Instant::now();
 let result=(||->Result<()>{
  c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
  wait(&mut c,15,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;
  wait(&mut c,15,|s|s.phase==P::SourcesReady)?;c.select_source(1)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;
  ensure!(c.snapshot().redirect.context("redirect")?.ip().is_loopback(),"nonlocal GAME rejected");c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(15);
  while c.snapshot().phase!=P::CharactersReady {c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"GAME bootstrap failed");thread::sleep(Duration::from_millis(10));}
  c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected)?;c.select_rental_set(0)?;
  wait(&mut c,15,|s|s.phase==P::RentalReady)?;c.join_rental_room(0)?;wait(&mut c,15,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&1812987999))?;
  if role=="b"{wait(&mut c,170,|_|root.join("a-teleport-complete").exists())?;}
  let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&c.snapshot().world,&cancel)?;
  let npcs=if role=="a"{[1812987996,1812987997]}else{[1812987998,1812987999]};
  for (ordinal,npc) in npcs.into_iter().enumerate(){
   let steps=walk(&mut c,&mut model,npc,&cancel,start)?;
   for (action,suffix) in [("LT_TELEPORT_B","b"),("LT_TELEPORT_C","c")]{
    c.rental_npc_talk(npc)?;wait(&mut c,10,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;
    let dialog=c.snapshot().merchant.dialog.clone().unwrap();ensure!(dialog.html=="TeleportMe"&&dialog.npc==npc,"unexpected source page");
    let label=format!("{}-{suffix}",if ordinal==0{"first"}else{"second"});
    let before=c.snapshot().clone();let authority_before=observe(root,role,&format!("{label}-before"))?;
    let request=Instant::now();c.rental_npc_action(npc,action)?;
    ensure!(c.rental_npc_action(npc,action).is_err(),"duplicate pending NPC action admitted");
    wait(&mut c,11,|s|s.merchant.timed_out&&!s.merchant.waiting)?;
    let elapsed=request.elapsed().as_millis();let after=c.snapshot().clone();let authority_after=observe(root,role,&format!("{label}-after"))?;
    ensure!(elapsed>=8000&&after.phase==P::BattlePending&&after.world.generation==before.world.generation&&after.inventory.items==before.inventory.items&&after.selected_character==before.selected_character,"unconfigured action changed ownership");
    ensure!(authority_before==authority_after,"unconfigured action changed server state");
    // A fresh valid dialogue must recover from the timeout without reconnecting.
    c.rental_npc_talk(npc)?;wait(&mut c,10,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;
    ensure!(!c.snapshot().merchant.timed_out,"dialogue did not recover");
    stages.push(json!({"npc":npc,"action":action,"dialog":dialog,"steps":steps,"elapsed_ms":elapsed,"before":before,"after":after,"authority_before":authority_before,"authority_after":authority_after,"reopened":c.snapshot().merchant}));
    std::fs::write(root.join(format!("{role}-teleport-boundary.json")),serde_json::to_vec_pretty(&stages)?)?;
   }
  }
  std::fs::write(root.join(format!("{role}-teleport-complete")),b"done")?;
  let other=if role=="a"{"b"}else{"a"};wait(&mut c,170,|_|root.join(format!("{other}-teleport-complete")).exists())?;
  Ok(())
 })();
 c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(10));}
 result
}
#[test]
#[ignore="requires owned formal loopback harness with no teleport destinations"]
fn actual_unconfigured_teleport_wait_is_bounded_and_dialogue_recovers(){
 let root=std::path::PathBuf::from(std::env::var("LITO_NEGATIVE_ROOT").expect("owned root"));assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));
 let ready:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();assert_eq!(ready["jar_sha256"],"bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43");
 let peer:SocketAddr=format!("127.0.0.1:{}",ready["port"].as_u64().unwrap()).parse().unwrap();
 thread::scope(|scope|{let a=scope.spawn(||player(&root,peer,"a","PeerFixture"));let b=scope.spawn(||player(&root,peer,"b","PeerFixture2"));let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");});
}
