//! Actual bootstrap diagnostic; does not qualify the unfinished observer renderer.
use anyhow::{ensure,Context,Result};
use lito_modern::modern_lito::{native_login::{LoginController,LoginPhase as P,Snapshot},native_connection::Limits};
use serde_json::{json,Value};
use std::{path::Path,net::SocketAddr,thread,time::{Instant,Duration},sync::atomic::AtomicBool};
fn capture(root:&Path,label:&str,frame:&lito_modern::modern_lito::native_world_scene::Frame)->Result<()> {let mut enc=png::Encoder::new(std::fs::File::create(root.join(format!("o-observer-{label}.png")))?,800,600);enc.set_color(png::ColorType::Rgba);enc.set_depth(png::BitDepth::Eight);enc.write_header()?.write_image_data(frame.canvas.rgba())?;Ok(())}
fn wait(c:&mut LoginController,secs:u64,ok:impl Fn(&Snapshot)->bool)->Result<()> {let end=Instant::now()+Duration::from_secs(secs);loop{c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"client failure {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(())}ensure!(Instant::now()<end,"timeout {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(10));}}
fn peer(root:&Path,addr:SocketAddr,role:&str,account:&str)->Result<()> {
 let mut c=LoginController::default();let live=std::env::var("LITO_OBSERVER_LIVE").is_ok();let result=(||->Result<()> {
  c.start(addr,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;wait(&mut c,15,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;wait(&mut c,15,|s|s.phase==P::SourcesReady)?;c.select_source(0)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(15);loop{c.poll();if c.snapshot().phase==P::CharactersReady{break}if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"GAME bootstrap");thread::sleep(Duration::from_millis(10));}
  c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected)?;c.enter_channel(0)?;wait(&mut c,15,|s|s.phase==P::LobbyReady)?;
  if role=="o"{c.set_observer(true)?;wait(&mut c,15,|s|s.phase==P::LobbyReady&&s.observer)?;}
  if role=="a" {c.create_room(&lito_modern::modern_lito::native_room::Settings{name:"觀戰實測",password:"",attributes:0x08,capacity:3,options:None})?;wait(&mut c,15,|s|s.phase==P::RoomReady)?;std::fs::write(root.join("observer-room-id"),c.snapshot().joined_room.as_ref().unwrap().id.to_string())?;}
  else {wait(&mut c,20,|_|root.join("observer-room-id").exists())?;let id=std::fs::read_to_string(root.join("observer-room-id"))?.parse::<u32>()?;wait(&mut c,15,|s|s.rooms.iter().any(|r|r.id==id))?;let index=c.snapshot().rooms.iter().position(|r|r.id==id).unwrap();c.join_room(index,"")?;wait(&mut c,15,|s|s.phase==P::RoomReady)?;if role=="b"{c.set_force(1)?;wait(&mut c,15,|s|s.phase==P::RoomReady)?;}}
  wait(&mut c,20,|s|s.joined_room.as_ref().is_some_and(|r|r.members.len()==3&&r.members.iter().any(|m|m.force==1)))?;
  if role=="o"{let tx=c.transport().queued_tx_counter();ensure!(c.set_ready(true).is_err(),"observer readiness accepted");ensure!(c.transport().queued_tx_counter()==tx,"observer readiness sent wire data");std::fs::write(root.join("observer-room-preflight.json"),serde_json::to_vec_pretty(c.snapshot())?)?;}
  else {wait(&mut c,20,|_|root.join("observer-room-preflight.json").exists())?;c.set_ready(true)?;}
  wait(&mut c,25,|s|s.phase==P::BattlePending&&s.world.started&&s.world.status.is_some()&&s.world.observer_bounds.is_some())?;
  let settle=Instant::now()+Duration::from_secs(2);wait(&mut c,5,|_|Instant::now()>=settle)?;let s=c.snapshot().clone();
  ensure!(s.world.map_id==Some(564),"unexpected observer arena");
  std::fs::write(root.join(format!("{role}-observer-bootstrap.json")),serde_json::to_vec_pretty(&json!({"role":role,"snapshot":s,"world_ready":s.world.ready(),"own_actor_present":s.world.own_actor().is_some(),"epoch":c.transport().snapshot().epoch,"renderer_verified":false,"authority_mutated":false}))?)?;
  if role=="o" {
   ensure!(s.world.observer_mode&&s.world.view_ready()&&!s.world.ready()&&s.world.own_actor().is_none(),"observer role/view contract");
   let cancel=AtomicBool::new(false);let clock=Instant::now();let mut model=lito_modern::modern_lito::native_world_scene::Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&s.world,&cancel)?;
   let before=model.draw_at(&s.world,&cancel,clock.elapsed().as_nanos() as u64)?;
   ensure!(before.input_ready(&s.world)&&!before.movement_ready(&s.world),"observer frame permits participant movement");
   for id in [1,3] {ensure!(before.receipt["actors"].as_array().unwrap().iter().any(|a|a["id"]==id&&a["rendered"]==true),"combatant not rendered");}
   capture(root,"before",&before)?;let actors=serde_json::to_value(&c.snapshot().world.actors)?;let tx=c.transport().queued_tx_counter();
   ensure!(c.move_step(0).is_err()&&c.attack(1).is_err()&&c.cast(3,None,&before).is_err(),"observer player input accepted");
   ensure!(c.pan_observer(0)?&&!before.input_ready(&c.snapshot().world),"camera pan did not retire old frame");
   let after=model.draw_at(&c.snapshot().world,&cancel,clock.elapsed().as_nanos() as u64)?;capture(root,"after",&after)?;
   let mut pans=1;while c.pan_observer(0)? {pans+=1;ensure!(pans<=64,"observer camera escaped bounds");}
   ensure!(c.snapshot().world.observer_camera.unwrap()[1]==s.world.observer_bounds.as_ref().unwrap().min[1],"camera did not stop at half-open boundary");
   ensure!(c.transport().queued_tx_counter()==tx&&serde_json::to_value(&c.snapshot().world.actors)?==actors,"camera/player guards mutated wire or actors");
   std::fs::write(root.join("observer-view.json"),serde_json::to_vec_pretty(&json!({"before":before.receipt,"after":after.receipt,"final_camera":c.snapshot().world.observer_camera,"pans":pans,"camera_sent_no_wire":true,"actors_unchanged":true,"player_actions_rejected":true,"old_frame_rejected":true,"own_actor_created":false,"physical_input_verified":false,"gui_verified":false}))?)?;
   std::fs::write(root.join("observer-view-complete"),b"done")?;
  } else {wait(&mut c,90,|_|root.join("observer-view-complete").exists())?;}
  if live {
   let cancel=AtomicBool::new(false);let clock=Instant::now();let mut model=lito_modern::modern_lito::native_world_scene::Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&c.snapshot().world,&cancel)?;
   if role=="o"{while c.snapshot().world.observer_camera!=s.world.observer_camera{ensure!(c.pan_observer(4)?,"camera restore failed");}}
   let mut moves=vec![];let mut frames=vec![];let mut seen_move=false;let mut seen_hurt=false;let mut seen_death=false;let mut attacks=0;
   let mut held=lito_modern::modern_lito::native_combat::HeldAttack::default();
   if role=="a" {
    let w=&c.snapshot().world;let own=w.own_actor().unwrap();let target=&w.actors[&3];let nav=model.navigation();let route=nav.route_to_adjacent_avoiding([own.x,own.y],[target.x,target.y],|x,y|w.actors.values().any(|a|a.id!=own.id&&!a.dead&&a.x==x&&a.y==y))?;
    for h in route {let end=Instant::now()+Duration::from_secs(8);loop{c.poll();if model.draw_at(&c.snapshot().world,&cancel,clock.elapsed().as_nanos() as u64)?.movement_ready(&c.snapshot().world){break}ensure!(Instant::now()<end,"participant movement wait");thread::sleep(Duration::from_millis(10));}let own=c.snapshot().world.own_actor().unwrap();let from=[own.x,own.y];let (x,y)=lito_modern::modern_lito::native_world::destination(own.x,own.y,h)?;c.move_checked(h,&nav)?;wait(&mut c,8,|s|s.world.own_actor().is_some_and(|a|[a.x,a.y]==[x,y]))?;moves.push(json!({"before":from,"after":[x,y],"heading":h}));}
    held.press(3,c.transport().snapshot().epoch,&c.snapshot().world);
   }
   let end=Instant::now()+Duration::from_secs(160);
   loop {c.poll();ensure!(matches!(c.snapshot().phase,P::BattlePending|P::BattleResult),"live observer left world {:?}",c.snapshot().error);let w=&c.snapshot().world;let frame=model.draw_at(w,&cancel,clock.elapsed().as_nanos() as u64)?;
    if role=="o" {
     let a=w.actors.get(&1).context("observed attacker absent")?;let b=w.actors.get(&3).context("observed target absent")?;
     for (label,fresh) in [("move",!seen_move&&[a.x,a.y]!=[s.world.actors[&1].x,s.world.actors[&1].y]),("hurt",!seen_hurt&&b.hp_ratio<100), ("death",!seen_death&&b.dead)] {
      if fresh {capture(root,label,&frame)?;frames.push(json!({"label":label,"world":w,"frame":frame.receipt}));std::fs::write(root.join("observer-live-frames.json"),serde_json::to_vec_pretty(&frames)?)?;match label{"move"=>seen_move=true,"hurt"=>seen_hurt=true,_=>seen_death=true};}
     }
    }
    if c.snapshot().phase==P::BattleResult {break}
    if role=="b"&&frame.death_ready(&c.snapshot().world){c.confirm_death(&frame)?;}
    if role=="a"&&c.continue_attack(&mut held,&frame)? {attacks+=1;ensure!(attacks<150,"observer attack budget");}
    ensure!(Instant::now()<end,"live observer settlement timeout");thread::sleep(Duration::from_millis(20));
   }
   if role=="o"{ensure!(seen_move&&seen_hurt&&seen_death,"observer missed live update: {seen_move}/{seen_hurt}/{seen_death}");}
   let result=c.snapshot().clone();c.return_from_battle()?;wait(&mut c,20,|s|s.phase==P::LobbyReady)?;let lobby=c.snapshot().clone();ensure!(!lobby.world.view_ready()&&lobby.world.observer_camera.is_none(),"observer scene retained after return");
   std::fs::write(root.join(format!("{role}-observer-live.json")),serde_json::to_vec_pretty(&json!({"role":role,"moves":moves,"attacks":attacks,"observed_frames":frames,"result":result,"lobby":lobby,"authority_or_clock_injection":false,"gui_verified":false}))?)?;
   std::fs::write(root.join(format!("{role}-observer-live-done")),b"done")?;wait(&mut c,20,|_|["a","b","o"].iter().all(|r|root.join(format!("{r}-observer-live-done")).exists()))?;
  }
  std::fs::write(root.join(format!("{role}-observer-captured")),b"captured")?;wait(&mut c,20,|_|["a","b","o"].iter().all(|r|root.join(format!("{r}-observer-captured")).exists()))?;Ok(())
 })();if let Err(e)=&result{std::fs::write(root.join(format!("{role}-observer-error.json")),serde_json::to_vec_pretty(&json!({"error":format!("{e:#}"),"snapshot":c.snapshot()}))?)?;}
 c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(10));}ensure!(c.is_quiescent(),"observer worker not reclaimed");result
}
#[test]
#[ignore="requires owned formal three-role observer harness; bootstrap diagnostic"]
fn actual_three_role_observer_bootstrap_preflight(){
 let root=std::path::PathBuf::from(std::env::var("LITO_OBSERVER_ROOT").expect("owned root"));ensure_root(&root);let ready:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();let expected=std::env::var("LITO_OBSERVER_REFERENCE_SHA").unwrap_or_else(|_|"9719f45db324206617726ce06d1a9e2184e9a3b3aefab3d50f3ebbdfbf09cfac".into());assert_eq!(ready["jar_sha256"],expected);let addr:SocketAddr=format!("127.0.0.1:{}",ready["port"].as_u64().context("LOGIN port").unwrap()).parse().unwrap();
 thread::scope(|scope|{let a=scope.spawn(||peer(&root,addr,"a","PeerFixture"));let b=scope.spawn(||peer(&root,addr,"b","PeerFixture2"));let o=scope.spawn(||peer(&root,addr,"o","PeerFixture3"));let ar=a.join().unwrap();let br=b.join().unwrap();let or=o.join().unwrap();assert!(ar.is_ok()&&br.is_ok()&&or.is_ok(),"a={ar:?}; b={br:?}; o={or:?}");});
}
fn ensure_root(root:&Path){assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));}
