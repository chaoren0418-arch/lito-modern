//! Opt-in real-server boundary; no server position, clocks or authority are edited.
use lito_modern::modern_lito::{native_login::{LoginController,LoginPhase as P,Snapshot},native_connection::Limits,native_world_scene::Model,native_world};
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,sync::atomic::AtomicBool,thread,time::{Duration,Instant}};
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


fn capture(root:&Path,role:&str,label:&str,frame:&lito_modern::modern_lito::native_world_scene::Frame)->Result<()> {
 let path=root.join(format!("{role}-guardian-{label}.png"));let mut enc=png::Encoder::new(std::fs::File::create(path)?,800,600);enc.set_color(png::ColorType::Rgba);enc.set_depth(png::BitDepth::Eight);enc.write_header()?.write_image_data(frame.canvas.rgba())?;Ok(())
}
fn guardian_player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()>{
 let mut c=LoginController::default();let cancel=AtomicBool::new(false);let start=Instant::now();
 let result=(||->Result<()>{
  c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;wait(&mut c,15,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;
  wait(&mut c,15,|s|s.phase==P::SourcesReady)?;c.select_source(0)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(15);while c.snapshot().phase!=P::CharactersReady{c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"GAME bootstrap");thread::sleep(Duration::from_millis(10));}
  c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected)?;c.select_rental_set(1)?;wait(&mut c,15,|s|s.phase==P::RentalReady)?;c.join_rental_room(0)?;
  wait(&mut c,15,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&1812987996))?;
  if role=="b"{wait(&mut c,90,|_|root.join("a-guardian-transferred").exists())?;}
  let mut town_model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&c.snapshot().world,&cancel)?;let rental_steps=walk(&mut c,&mut town_model,1812987996,&cancel,start)?;
  c.rental_npc_talk(1812987996)?;wait(&mut c,10,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;
  let rental=c.snapshot().clone();c.rental_npc_action(1812987996,if role=="a"{"LT_TELEPORT_B"}else{"LT_TELEPORT_C"})?;
  let end=Instant::now()+Duration::from_secs(20);loop{c.poll();if c.snapshot().phase==P::CharacterSelected&&c.snapshot().inventory.batches>0{break}if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"guardian loadout handoff: {:?}",c.snapshot().error);thread::sleep(Duration::from_millis(10));}
  let transferred=c.snapshot().clone();ensure!(transferred.inventory.items.values().any(|i|i.use_type==1&&i.name.ends_with("($9)")),"no formally equipped handed-off weapon");
  std::fs::write(root.join(format!("{role}-guardian-transferred")),b"transferred")?;let other=if role=="a"{"b"}else{"a"};wait(&mut c,90,|_|root.join(format!("{other}-guardian-transferred")).exists())?;
  c.enter_channel(0)?;wait(&mut c,15,|s|s.phase==P::LobbyReady)?;
  if role=="a"{
   c.create_room(&lito_modern::modern_lito::native_room::Settings{name:"守護塔實戰",password:"",attributes:0,capacity:2,options:None})?;wait(&mut c,15,|s|s.phase==P::RoomReady)?;
   c.step_room_mode(1)?;wait(&mut c,15,|s|s.phase==P::RoomReady&&s.joined_room.as_ref().is_some_and(|r|r.mode==5))?;std::fs::write(root.join("guardian-room-id"),c.snapshot().joined_room.as_ref().unwrap().id.to_string())?;
  }else{
   wait(&mut c,20,|_|root.join("guardian-room-id").exists())?;let id=std::fs::read_to_string(root.join("guardian-room-id"))?.parse::<u32>()?;
   wait(&mut c,15,|s|s.rooms.iter().any(|r|r.id==id))?;let index=c.snapshot().rooms.iter().position(|r|r.id==id).unwrap();c.join_room(index,"")?;wait(&mut c,15,|s|s.phase==P::RoomReady)?;c.set_force(1)?;wait(&mut c,15,|s|s.phase==P::RoomReady)?;
  }
  wait(&mut c,20,|s|s.joined_room.as_ref().is_some_and(|r|r.mode==5&&r.members.len()==2&&r.members.iter().any(|m|m.force==1)&&r.members.iter().any(|m|m.force==0)))?;
  c.set_ready(true)?;wait(&mut c,20,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.values().any(|a|a.graphics==2669))?;
  let initial=c.snapshot().clone();let target=initial.world.actors.values().find(|a|a.graphics==2669&&lito_modern::modern_lito::native_obstacles::guardian(a)).context("blue guardian")?.id;
  let old=lito_modern::modern_lito::native_skill_selection::Target::capture(&initial.world,target).context("guardian selection")?;
  let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&initial.world,&cancel)?;let steps=walk(&mut c,&mut model,target,&cancel,start)?;
  std::fs::write(root.join(format!("{role}-guardian-near")),b"near")?;let other=if role=="a"{"b"}else{"a"};wait(&mut c,60,|_|root.join(format!("{other}-guardian-near")).exists())?;
  let mut held=lito_modern::modern_lito::native_combat::HeldAttack::default();let mut attacks=0usize;let mut poses=vec![];let mut seen=std::collections::BTreeSet::new();let deadline=Instant::now()+Duration::from_secs(1000);
  if role=="a"{held.press(target,c.transport().snapshot().epoch,&c.snapshot().world);}
  loop {
   c.poll();ensure!(matches!(c.snapshot().phase,P::BattlePending|P::BattleResult),"guardian world unavailable: {:?} {:?}",c.snapshot().phase,c.snapshot().error);let w=&c.snapshot().world;ensure!(w.local_corrections==0,"guardian route corrections");
   let actor=w.actors.get(&target).context("guardian disappeared")?;let action=actor.walk_action;let frame=model.draw_at(w,&cancel,start.elapsed().as_nanos() as u64)?;
   if seen.insert(action){let draw=frame.receipt["actors"].as_array().context("actor render receipt")?.iter().find(|a|a["id"]==target).context("guardian draw absent")?;ensure!(draw["rendered"]==true&&draw["action"]==action&&draw["phase"]=="object_action","guardian render failed: {draw}");capture(root,role,&action.to_string(),&frame)?;poses.push(json!({"action":action,"world":w,"frame":frame.receipt}));}
   if action==35{break}
   if role=="a"{ensure!(held.active(),"guardian held intent ended prematurely");if c.continue_attack(&mut held,&frame)?{attacks+=1;ensure!(attacks<4000,"guardian attack budget");}}
   ensure!(Instant::now()<deadline,"guardian destruction timeout");thread::sleep(Duration::from_millis(20));
  }
  let destroyed=c.snapshot().clone();ensure!(old.resolve(&destroyed.world).is_none(),"old guardian target survived destruction");ensure!(destroyed.world.attack_request(target).is_err(),"destroyed guardian attackable");ensure!(lito_modern::modern_lito::native_skills::request(&destroyed.world,6,Some(target)).is_err(),"destroyed guardian spellable");
  ensure!(!lito_modern::modern_lito::native_obstacles::blocks(&destroyed.world,destroyed.world.actors[&target].x,destroyed.world.actors[&target].y),"destroyed guardian blocks");
  // Capture every sampled death frame, including the input-retirement draw.
  // PNG compression can advance the real clock into sound frame 1 before
  // the five-second hold loop starts; discarding that draw loses the event.
  let mut death_sounds=vec![];
  for pose in poses.iter().filter(|p|p["action"]==35) {
   death_sounds.extend(pose["frame"]["audio"].as_array().unwrap().iter().filter(|s|s["event"]["actor"]==target&&s["event"]["sound"]==360).cloned());
  }
  let frame=model.draw_at(&destroyed.world,&cancel,start.elapsed().as_nanos() as u64)?;ensure!(!c.continue_attack(&mut held,&frame)?&&!held.active(),"old held attack resumed");
  death_sounds.extend(frame.receipt["audio"].as_array().unwrap().iter().filter(|s|s["event"]["actor"]==target&&s["event"]["sound"]==360).cloned());
  let held_until=Instant::now()+Duration::from_secs(5);
  let held_frame=loop {
   c.poll();let frame=model.draw_at(&c.snapshot().world,&cancel,start.elapsed().as_nanos() as u64)?;
   let draw=frame.receipt["actors"].as_array().unwrap().iter().find(|a|a["id"]==target).context("held guardian absent")?;
   ensure!(draw["rendered"]==true&&draw["action"]==35&&draw["phase"]=="object_action","held guardian render: {draw}");
   for sound in frame.receipt["audio"].as_array().unwrap().iter().filter(|s|s["event"]["actor"]==target&&s["event"]["sound"]==360){ensure!(sound["prepared"]==true,"guardian sound failed");death_sounds.push(sound.clone());}
   if Instant::now()>=held_until {ensure!(draw["action_complete"]==true&&draw["sprite_subframe"]==19,"guardian terminal pose incomplete");capture(root,role,"held-destroyed",&frame)?;break frame.receipt;}
   thread::sleep(Duration::from_millis(20));
  };
  ensure!(death_sounds.len()==1&&death_sounds[0]["prepared"]==true,"guardian sound360 must prepare once: {}",death_sounds.len());
  // The pinned formal honor mode settles immediately when this tower falls.
  // Do not invent a post-result move or withdraw to keep the fixture running.
  wait(&mut c,20,|s|s.phase==P::BattleResult)?;
  let tower=&c.snapshot().world.actors[&target];let own=c.snapshot().world.own_actor().unwrap();
  let heading=(0..8).find(|h|native_world::destination(own.x,own.y,*h).is_ok_and(|(x,y)|[x,y]==[tower.x,tower.y])).context("not adjacent to destroyed tower")?;
  ensure!(c.move_checked(heading,&model.navigation()).is_err(),"post-result movement accepted");
  std::fs::write(root.join(format!("{role}-guardian-observed")),b"observed")?;wait(&mut c,20,|_|root.join(format!("{other}-guardian-observed")).exists())?;
  wait(&mut c,20,|s|s.phase==P::BattleResult)?;let result=c.snapshot().clone();c.return_from_battle()?;wait(&mut c,20,|s|s.phase==P::LobbyReady)?;let lobby=c.snapshot().clone();
  std::fs::write(root.join(format!("{role}-guardian-result.json")),serde_json::to_vec_pretty(&json!({"role":role,"target":target,"rental":rental,"rental_steps":rental_steps,"transferred":transferred,"initial":initial,"steps":steps,"poses":poses,"destroyed":destroyed,"held_destroyed_frame":held_frame,"guardian_death_sound":death_sounds,"attacks":attacks,"automatic_tower_settlement":true,"post_result_move_rejected":true,"post_destruction_cell_move_verified":false,"result":result,"lobby":lobby,"authority_or_clock_injection":false,"gui_verified":false}))?)?;
  std::fs::write(root.join(format!("{role}-guardian-complete")),b"done")?;wait(&mut c,20,|_|root.join(format!("{other}-guardian-complete")).exists())?;Ok(())
 })();c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(10));}ensure!(c.is_quiescent(),"guardian client not reclaimed");result
}
#[test]
#[ignore="requires owned formal honor harness; actual3000HP tower"]
fn actual_guardian_cracks_destruction_and_settlement_retire_old_inputs(){
 let root=std::path::PathBuf::from(std::env::var("LITO_GUARDIAN_ROOT").expect("owned root"));assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));let ready:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();let expected=std::env::var("LITO_GUARDIAN_REFERENCE_SHA").unwrap_or_else(|_|"bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43".into());assert_eq!(ready["jar_sha256"],expected);let peer:SocketAddr=format!("127.0.0.1:{}",ready["port"].as_u64().unwrap()).parse().unwrap();
 thread::scope(|scope|{let a=scope.spawn(||guardian_player(&root,peer,"a","PeerFixture"));let b=scope.spawn(||guardian_player(&root,peer,"b","PeerFixture2"));let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");});
}
