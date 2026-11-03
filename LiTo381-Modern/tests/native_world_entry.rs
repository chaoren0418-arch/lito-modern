use anyhow::{ensure,Context,Result};
use lito_modern::modern_lito::{native_connection::Limits,native_login::{LoginController,LoginPhase as P,Snapshot},native_world,native_world_scene::{Model,Frame}};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,sync::atomic::AtomicBool,thread,time::{Duration,Instant}};
fn wait(c:&mut LoginController,ok:impl Fn(&Snapshot)->bool)->Result<()>{let end=Instant::now()+Duration::from_secs(20);loop{c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"client {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(())}ensure!(Instant::now()<end,"wait {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(5));}}
fn frame(c:&mut LoginController,m:&mut Model,start:Instant)->Result<Frame>{let end=Instant::now()+Duration::from_secs(10);loop{c.poll();ensure!(c.snapshot().phase.active_world(),"frame phase {:?}: {:?}",c.snapshot().phase,c.snapshot().error);let f=m.draw_at(&c.snapshot().world,&AtomicBool::new(false),start.elapsed().as_nanos() as u64)?;if f.movement_ready(&c.snapshot().world){return Ok(f)}ensure!(Instant::now()<end,"frame wait");thread::sleep(Duration::from_millis(5));}}
fn player(root:&Path,peer:SocketAddr,role:&str,account:&str,refused:bool)->Result<()>{
 let mut c=LoginController::default();let start=Instant::now();let cancel=AtomicBool::new(false);
 let result=(||->Result<()>{
  c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;wait(&mut c,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;wait(&mut c,|s|s.phase==P::SourcesReady)?;c.select_source(1)?;wait(&mut c,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(20);while c.snapshot().phase!=P::CharactersReady{c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"bootstrap {:?}",c.snapshot().error);thread::sleep(Duration::from_millis(5));}
  c.select_character(0)?;wait(&mut c,|s|s.phase==P::CharacterSelected)?;c.enter_channel(0)?;wait(&mut c,|s|s.phase==P::LobbyReady&&!s.rooms.is_empty())?;c.join_room(0,"")?;wait(&mut c,|s|s.phase==P::RoomReady)?;
  let desired=if role=="a"{0}else{1};let mut forced=false;let end=Instant::now()+Duration::from_secs(20);
  loop{c.poll();ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"battle setup {:?}",c.snapshot().error);let s=c.snapshot();if s.phase==P::BattlePending&&s.world.ready()&&s.world.hp.is_some(){break;}
   if s.phase==P::RoomReady{let r=s.joined_room.as_ref().context("room")?;let own=r.members.iter().find(|m|Some(m.name.as_str())==s.selected_character.as_deref()).context("self member")?;if own.force!=desired&&!forced{c.set_force(desired)?;forced=true;}else if own.force==desired&&own.state==0&&r.members.len()==2&&r.members.iter().any(|m|m.force!=desired){c.set_ready(true)?;}}
   thread::sleep(Duration::from_millis(5));
  }
  let before=c.snapshot().clone();ensure!(before.world.learned_skills.contains(&131),"fixture book not received");let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&before.world,&cancel)?;let old_frame=frame(&mut c,&mut model,start)?;
  std::fs::write(root.join(format!("{role}-ready-to-cast")),b"ready")?;let other=if role=="a"{"b"}else{"a"};wait(&mut c,|_|root.join(format!("{other}-ready-to-cast")).exists())?;
  if refused {
   let request_start=Instant::now();
   for _ in 0..2 {
    let f=frame(&mut c,&mut model,start)?;let messages=c.snapshot().world.message_count;
    c.cast(131,None,&f)?;ensure!(c.cast(131,None,&f).is_err(),"duplicate refused request accepted");
    wait(&mut c,|s|s.world.message_count>messages&&s.skill_feedback==lito_modern::modern_lito::native_skills::Feedback::Rejected(276))?;
    ensure!(c.snapshot().phase==P::BattlePending&&c.snapshot().world.generation==before.world.generation&&c.snapshot().world.map_id==before.world.map_id&&c.snapshot().world.mp==before.world.mp&&c.snapshot().world.portal.is_none(),"map refusal changed authority");
   }
   let after=c.snapshot().clone();let f=frame(&mut c,&mut model,start)?;let own=c.snapshot().world.own_actor().context("refused caster")?;
   let h=(0..8).find(|h|f.navigation.can_step(own.x.into(),own.y.into(),*h)&&native_world::destination(own.x,own.y,*h).is_ok_and(|(x,y)|!c.snapshot().world.actors.values().any(|a|a.id!=own.id&&a.x==x&&a.y==y))).context("no refused followup step")?;
   let target=native_world::destination(own.x,own.y,h)?;c.move_checked(h,&f.navigation)?;wait(&mut c,|s|s.world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;frame(&mut c,&mut model,start)?;
   ensure!(c.snapshot().world.local_corrections==after.world.local_corrections,"post-refusal correction");
   ensure!(serde_json::to_value(&before.inventory.items)?==serde_json::to_value(&c.snapshot().inventory.items)?,"refusal inventory mutation");
   std::fs::write(root.join(format!("{role}-world-entry.json")),serde_json::to_vec_pretty(&json!({"role":role,"before":before,"after_refused":after,"after_walk":c.snapshot(),"walk_target":target,"two_explicit_requests_elapsed_ms":request_start.elapsed().as_millis(),"classic_world_compatibility_enabled":false,"ordinary_world_entered":false}))?)?;
   std::fs::write(root.join(format!("{role}-refusal-complete")),b"done")?;wait(&mut c,|_|root.join(format!("{other}-refusal-complete")).exists())?;
   return Ok(());
  }
  c.cast(131,None,&old_frame)?;ensure!(c.cast(131,None,&old_frame).is_err(),"duplicate spell accepted");
  wait(&mut c,|s|s.phase==P::WorldReady&&s.world.ready()&&s.world.hp.is_some()&&s.world.actors.contains_key(&if role=="a"{3}else{1}))?;
  let after=c.snapshot().clone();ensure!(after.joined_room.is_none()&&after.channel.is_none()&&after.rooms.is_empty(),"old membership retained");ensure!(after.world.map_id==Some(4)&&after.world.generation==before.world.generation+1&&after.world.completed_portals==1,"WORLD generation");ensure!(after.selected_character==before.selected_character&&after.inventory.owner==before.inventory.owner&&serde_json::to_value(&after.inventory.items)?==serde_json::to_value(&before.inventory.items)?,"character/inventory continuity");
  ensure!(c.cast(131,None,&old_frame).is_err(),"old rendered frame accepted after WORLD");
  model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&after.world,&cancel)?;let f=frame(&mut c,&mut model,start)?;
  let npc=f.receipt["actors"].as_array().context("actor receipt")?.iter().find(|a|a["id"]==1677722876u32).context("type12 NPC receipt")?;
  ensure!(npc["rendered"]==true&&npc["sprite_member"]=="869-0.spr"&&npc["sprite_subframe"]==0&&npc["phase"]=="static_prop","type12 body still omitted");
  ensure!(npc["layer_error"].is_null()&&npc["body_presentation"]["body_half"]==true&&npc["body_draw_style"]=="Half","catalogue flag1 body-half path absent");
  let mut png=png::Encoder::new(std::fs::File::create(root.join(format!("{role}-world-entry.png")))?,f.canvas.width(),f.canvas.height());png.set_color(png::ColorType::Rgba);png.set_depth(png::BitDepth::Eight);png.write_header()?.write_image_data(f.canvas.rgba())?;
  std::fs::write(root.join(format!("{role}-world-loaded")),b"ready")?;wait(&mut c,|_|root.join(format!("{other}-world-loaded")).exists())?;
  // Start in the same server-selected cell; separate peers in opposite directions.
  let own=c.snapshot().world.own_actor().context("WORLD self")?;let h=if role=="a"{2}else{6};ensure!(f.navigation.can_step(own.x.into(),own.y.into(),h),"chosen WORLD step blocked");let target=native_world::destination(own.x,own.y,h)?;c.move_checked(h,&f.navigation)?;
  wait(&mut c,|s|s.world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;frame(&mut c,&mut model,start)?;
  ensure!(c.snapshot().world.local_corrections==after.world.local_corrections,"WORLD step corrected");
  std::fs::write(root.join(format!("{role}-world-walked")),b"ready")?;wait(&mut c,|_|root.join(format!("{other}-world-walked")).exists())?;
  wait(&mut c,|s|s.world.actors.get(&if role=="a"{3}else{1}).is_some_and(|a|[a.x,a.y]!=[33051,32337]))?;
  std::fs::write(root.join(format!("{role}-world-entry.json")),serde_json::to_vec_pretty(&json!({"role":role,"before":before,"after":after,"after_walk":c.snapshot(),"walk_target":target,"render":f.receipt,"product_controller_qualified":true,"classic_world_compatibility_enabled":true}))?)?;
  std::fs::write(root.join(format!("{role}-report-saved")),b"done")?;wait(&mut c,|_|root.join(format!("{other}-report-saved")).exists())?;
  Ok(())})();
 if let Err(e)=&result{std::fs::write(root.join(format!("{role}-world-entry-error.json")),serde_json::to_vec_pretty(&json!({"error":format!("{e:#}"),"snapshot":c.snapshot()}))?)?;}
 c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(5));}ensure!(c.is_quiescent(),"worker leak");result
}
#[test]
#[ignore="requires owned C02 formal world-entry harness and original resources"]
fn actual_world_tree_enters_roomless_world_and_walks(){run(false)}
#[test]
#[ignore="requires owned formal-configuration world-tree refusal harness"]
fn actual_world_tree_disabled_preserves_arena_and_allows_retry(){run(true)}
fn run(refused:bool){
 let root=std::path::PathBuf::from(std::env::var("LITO_WORLD_ENTRY_ROOT").unwrap());assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));let r:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();assert_eq!(r["jar_sha256"],std::env::var("LITO_WORLD_ENTRY_REFERENCE_SHA").unwrap());let peer=format!("127.0.0.1:{}",r["port"].as_u64().unwrap()).parse().unwrap();thread::scope(|s|{let a=s.spawn(||player(&root,peer,"a","PeerFixture",refused));let b=s.spawn(||player(&root,peer,"b","PeerFixture2",refused));let a=a.join().unwrap();let b=b.join().unwrap();assert!(a.is_ok()&&b.is_ok(),"a={a:?}; b={b:?}");});
}

#[test]
fn learned_world_tree_is_selector_only_and_cross_map_admission_stays_owned(){
 use lito_modern::modern_lito::{native_portal as p,native_skills,native_skill_selection};
 fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
 let v:Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut s=LoginController::default().snapshot().clone();
 for key in ["world","start","status","actor","bot","hp","mp"]{s.world.accept(&bytes(v["packets"][key].as_str().unwrap())).unwrap();}
 s.phase=P::WorldReady;s.game_connected=true;
 let d=p::Destination{map:4,x:33051,y:32337};assert!(p::admit_world_tree(&s,d).is_err());
 s.world.learned_skills.insert(131);assert_eq!(native_skills::request(&s.world,131,None).unwrap(),[0x14,16,2]);assert!(native_skills::request(&s.world,131,Some(2)).is_err());
 let target=native_skill_selection::Target::capture(&s.world,2);assert_eq!(native_skill_selection::choice(&s.world,131,target).unwrap().target,None);
 assert!(p::admit_world_tree(&s,d).is_ok());assert!(p::admit(&s,d).is_err());
 for change in 0..7{let mut c=s.clone();match change{0=>c.phase=P::WorldEntering,1=>c.game_connected=false,2=>c.observer=true,3=>c.world.hp=Some([0,100]),4=>{c.world.learned_skills.remove(&131);},5=>c.world.observer_mode=true,_=>{let id=c.world.own_actor().unwrap().id;c.world.actors.get_mut(&id).unwrap().dead=true;}}assert!(p::admit_world_tree(&c,d).is_err(),"guard {change}");}
 assert!(p::admit_world_tree(&s,p::Destination{map:4,x:33052,y:32337}).is_err());assert!(p::admit_world_tree(&s,p::Destination{map:5,..d}).is_err());
 assert!(!P::WorldEntering.active_world());assert!(P::WorldReady.active_world());
}

#[test]
fn formal_map_rejection_is_visible_and_does_not_change_world(){
 use lito_modern::modern_lito::native_skills::{Feedback,Pending};
 let v:Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=native_world::World::default();
 for key in ["world","start","status","actor","bot","hp","mp"]{let h=v["packets"][key].as_str().unwrap();w.accept(&(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()).unwrap();}
 let old=w.clone();let now=Instant::now();let pending=Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
 w.accept(&[0x10,0x14,0x01,0]).unwrap();assert_eq!(pending.observe(&w,now),Feedback::Rejected(276));assert_eq!(Feedback::Rejected(276).label(),"目前地圖無法使用此技能");assert_eq!(w.generation,old.generation);assert_eq!(w.map_id,old.map_id);assert_eq!(w.mp,old.mp);assert!(w.portal.is_none());
}
