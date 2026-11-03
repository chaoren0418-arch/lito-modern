use anyhow::{ensure,Context,Result};
use lito_modern::modern_lito::{native_connection::Limits,native_login::{LoginController,LoginPhase as P,Snapshot},native_portal::{EntryChoice,EntryFeedback,EntryPending},native_world::{self,World},native_world_scene::{Model,Frame}};
use serde_json::{json,Value};
use std::{path::Path,net::SocketAddr,sync::atomic::AtomicBool,thread,time::{Duration,Instant}};
fn wait(c:&mut LoginController,ok:impl Fn(&Snapshot)->bool)->Result<()>{let end=Instant::now()+Duration::from_secs(15);loop{c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"client failed {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(())}ensure!(Instant::now()<end,"wait at {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(10));}}
fn ready_frame(c:&mut LoginController,m:&mut Model,start:Instant)->Result<Frame>{let end=Instant::now()+Duration::from_secs(10);loop{c.poll();let f=m.draw_at(&c.snapshot().world,&AtomicBool::new(false),start.elapsed().as_nanos() as u64)?;if f.movement_ready(&c.snapshot().world){return Ok(f)}ensure!(Instant::now()<end,"frame not ready");thread::sleep(Duration::from_millis(10));}}
fn fixture()->World{
 let v:Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=World::default();
 for key in ["world","status","actor","hp","mp"]{if key=="status"{w.accept(&[0xd2]).unwrap();}let h=v["packets"][key].as_str().unwrap();let b:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
 w.map_id=Some(500);let id=w.own_actor().unwrap().id;let a=w.actors.get_mut(&id).unwrap();a.x=32771;a.y=32822;w
}
#[test]
fn original_native_entrances_encode_source_only_and_guard_every_owner_change(){
 let w=fixture();let model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&w,&AtomicBool::new(false)).unwrap();let nav=model.navigation();
 assert!(nav.has_portal_source([32771,32824]));let choice=EntryChoice::nearest(4,&w,&nav).unwrap();assert_eq!(choice.source,[32771,32824]);
 assert_eq!(choice.request(4,&w,&nav).unwrap(),[0x73,0x03,0x80,0x38,0x80]);
 assert!(choice.request(5,&w,&nav).is_err());assert!(EntryChoice::capture(4,&w,&nav,[1,1]).is_err());
 for change in 0..9 {let mut changed=w.clone();let id=changed.own_actor().unwrap().id;match change{0=>changed.generation+=1,1=>changed.local_corrections+=1,2=>changed.control_interruptions+=1,3=>changed.movement_locks+=1,4=>changed.actors.get_mut(&id).unwrap().incarnation+=1,5=>changed.actors.get_mut(&id).unwrap().x+=1,6=>changed.observer_mode=true,7=>changed.hp=Some([0,100]),_=>changed.map_id=Some(501)};assert!(choice.request(4,&changed,&nav).is_err(),"change {change}");}
 let now=Instant::now();let p=EntryPending{choice,since:now};assert_eq!(p.observe(4,&w,now+Duration::from_millis(3999)),EntryFeedback::Pending);assert_eq!(p.observe(4,&w,now+Duration::from_secs(4)),EntryFeedback::NoResponse);assert_eq!(p.observe(5,&w,now),EntryFeedback::Interrupted);
}
fn player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()>{
 let mut c=LoginController::default();let start=Instant::now();let cancel=AtomicBool::new(false);
 let result=(||->Result<()>{
 c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;wait(&mut c,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;wait(&mut c,|s|s.phase==P::SourcesReady)?;c.select_source(1)?;wait(&mut c,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
 let end=Instant::now()+Duration::from_secs(15);while c.snapshot().phase!=P::CharactersReady{c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"bootstrap failed");thread::sleep(Duration::from_millis(10));}
 c.select_character(0)?;wait(&mut c,|s|s.phase==P::CharacterSelected)?;c.select_rental_set(1)?;wait(&mut c,|s|s.phase==P::RentalReady)?;c.join_rental_room(0)?;wait(&mut c,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.hp.is_some())?;
 let entered=c.snapshot().clone();let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&entered.world,&cancel)?;let nav=model.navigation();ensure!(nav.has_portal_source([32771,32824]),"original source entrance absent");
 let own=c.snapshot().world.own_actor().unwrap();let route=nav.route_to_range_avoiding([own.x,own.y],[32771,32824],2,|x,y|c.snapshot().world.actors.values().any(|a|a.id!=own.id&&a.x==x&&a.y==y))?;
 for h in route{ready_frame(&mut c,&mut model,start)?;let a=c.snapshot().world.own_actor().unwrap();let target=native_world::destination(a.x,a.y,h)?;c.move_checked(h,&nav)?;wait(&mut c,|s|s.world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;}
 let mut transitions=vec![];
 for n in 0..2{
  let frame=ready_frame(&mut c,&mut model,start)?;let before=c.snapshot().clone();let choice=EntryChoice::nearest(c.transport().snapshot().epoch,&before.world,&nav)?;let source=choice.source;
  c.enter_map_portal(choice.clone(),&frame)?;ensure!(c.enter_map_portal(choice.clone(),&frame).is_err()&&c.move_checked(0,&nav).is_err(),"duplicate or competing move accepted");
  ensure!(c.snapshot().world.generation==before.world.generation&&c.snapshot().world.own_actor().unwrap().x==before.world.own_actor().unwrap().x,"predicted portal authority");
  wait(&mut c,|s|s.world.completed_portals==n+1&&s.world.ready()&&s.world.hp.is_some())?;let after=c.snapshot().clone();ensure!(after.world.generation==before.world.generation+1&&after.selected_character==entered.selected_character,"scene/character mismatch");
  ensure!(after.inventory.owner==before.inventory.owner&&serde_json::to_value(&after.inventory.items)?==serde_json::to_value(&before.inventory.items)?,"portal changed character inventory");
  ensure!(c.portal_entry_feedback()==EntryFeedback::SceneUpdated,"missing portal feedback");model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&after.world,&cancel)?;let new_frame=ready_frame(&mut c,&mut model,start)?;ensure!(c.enter_map_portal(choice,&new_frame).is_err(),"old entrance choice survived scene rebuild");
  transitions.push(json!({"source":source,"before":before,"after":after}));
 }
 let frame=ready_frame(&mut c,&mut model,start)?;let own=c.snapshot().world.own_actor().unwrap();let h=(0..8).find(|h|frame.navigation.can_step(i32::from(own.x),i32::from(own.y),*h)&&native_world::destination(own.x,own.y,*h).is_ok_and(|(x,y)|!c.snapshot().world.actors.values().any(|a|a.id!=own.id&&a.x==x&&a.y==y))).context("no post-portal step")?;let target=native_world::destination(own.x,own.y,h)?;c.move_checked(h,&frame.navigation)?;wait(&mut c,|s|s.world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;ready_frame(&mut c,&mut model,start)?;
 std::fs::write(root.join(format!("{role}-map-portal.json")),serde_json::to_vec_pretty(&json!({"role":role,"entered":entered,"transitions":transitions,"after_walk":c.snapshot(),"walk_target":target,"epoch":c.transport().snapshot().epoch,"original_entrances":nav.portal_sources().collect::<Vec<_>>()}))?)?;
 Ok(())})();
 if let Err(error)=&result{std::fs::write(root.join(format!("{role}-map-portal-error.json")),serde_json::to_vec_pretty(&json!({"error":format!("{error:#}"),"snapshot":c.snapshot()}))?)?;}
 c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(10));}ensure!(c.is_quiescent(),"worker leak");result
}
#[test]
#[ignore="requires owned real formal LOGIN/GAME original-map entrance harness"]
fn actual_original_map_entrances_roundtrip_and_retire_old_scene(){
 let root=std::path::PathBuf::from(std::env::var("LITO_MAP_PORTAL_ROOT").unwrap());assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));let ready:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();assert_eq!(ready["jar_sha256"],std::env::var("LITO_MAP_PORTAL_REFERENCE_SHA").unwrap());let peer=format!("127.0.0.1:{}",ready["port"].as_u64().unwrap()).parse().unwrap();
 thread::scope(|scope|{let a=scope.spawn(||player(&root,peer,"a","PeerFixture"));let b=scope.spawn(||player(&root,peer,"b","PeerFixture2"));let a=a.join().unwrap();let b=b.join().unwrap();assert!(a.is_ok()&&b.is_ok(),"a={a:?}; b={b:?}");});
}
