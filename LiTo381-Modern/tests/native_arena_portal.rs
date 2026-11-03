use anyhow::{ensure,Context,Result};
use lito_modern::modern_lito::{native_connection::Limits,native_login::{LoginController,LoginPhase as P,Snapshot},native_portal::{EntryChoice,EntryFeedback},native_world::{self,World},native_world_scene::{Model,Frame}};
use serde_json::{json,Value};
use std::{path::Path,net::SocketAddr,sync::atomic::{AtomicBool,Ordering},thread,time::{Duration,Instant}};
static ABORT:AtomicBool=AtomicBool::new(false);
fn fixture_aborted()->bool {
 if ABORT.load(Ordering::Acquire){return true}
 if std::env::var_os("LITO_ARENA_GUI_SUPPORT").is_some() && std::env::var_os("LITO_ARENA_PORTAL_ROOT").is_some_and(|p|Path::new(&p).join("gui-abort").exists()) {ABORT.store(true,Ordering::Release);return true}false
}

fn wait(c:&mut LoginController,ok:impl Fn(&Snapshot)->bool)->Result<()>{let end=Instant::now()+Duration::from_secs(if std::env::var_os("LITO_ARENA_GUI_SUPPORT").is_some(){600}else{480});loop{ensure!(!fixture_aborted(),"peer fixture failed");c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"client failed {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(())}ensure!(Instant::now()<end,"wait at {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(10));}}
fn ready_frame(c:&mut LoginController,m:&mut Model,start:Instant)->Result<Frame>{let end=Instant::now()+Duration::from_secs(10);loop{ensure!(!fixture_aborted(),"peer fixture failed");c.poll();let f=m.draw_at(&c.snapshot().world,&AtomicBool::new(false),start.elapsed().as_nanos() as u64)?;if f.movement_ready(&c.snapshot().world){return Ok(f)}if Instant::now()>=end{if let Ok(root)=std::env::var("LITO_ARENA_PORTAL_ROOT"){let id=c.snapshot().world.own_actor().map(|a|a.id).unwrap_or(0);std::fs::write(Path::new(&root).join(format!("frame-not-ready-{id}.json")),serde_json::to_vec_pretty(&json!({"frame":f.receipt,"world":c.snapshot().world}))?)?;}anyhow::bail!("frame not ready");}thread::sleep(Duration::from_millis(10));}}
fn fixture()->World{
 let v:Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=World::default();
 for key in ["world","status","actor","hp","mp"]{if key=="status"{w.accept(&[0xd2]).unwrap();}let h=v["packets"][key].as_str().unwrap();let b:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
 w.map_id=Some(500);let id=w.own_actor().unwrap().id;let a=w.actors.get_mut(&id).unwrap();a.x=32771;a.y=32822;w
}
#[test]
fn original_101_and_102_sources_are_available_without_client_destinations(){
 use lito_modern::modern_lito::{resources::SharedMapSectorKey,world_map::{WorldMap,SectorInput},native_navigation::Navigation,native_room};
 for (id,kind) in [(573,101),(574,102)] {
  let mut input=vec![];
  for e in std::fs::read_dir(format!(r"D:\lineage開發\0-客戶端\LiTo\LTmap\{id}")).unwrap(){let p=e.unwrap().path();let n=p.file_name().unwrap().to_str().unwrap();if !n.ends_with(".s32"){continue}input.push(SectorInput{key:SharedMapSectorKey{map_id:id,sector_x:u16::from_str_radix(&n[..4],16).unwrap(),sector_y:u16::from_str_radix(&n[4..8],16).unwrap()},bytes:std::fs::read(&p).unwrap()});}
  let map=WorldMap::from_native_sector_bytes(input,&Default::default()).unwrap();
  assert!(map.bindings().iter().any(|b|map.sector(b.key).unwrap().portals.iter().any(|p|p.native_kind==Some(kind))));
  let nav=Navigation::from_geometry(&map).unwrap();
  for point in [[32755,32792],[32742,32797]] {assert!(nav.has_portal_source(point));let mut w=fixture();w.map_id=Some(id as u16);let own_id=w.own_actor().unwrap().id;let a=w.actors.get_mut(&own_id).unwrap();a.x=point[0];a.y=point[1];let choice=EntryChoice::capture(4,&w,&nav,point).unwrap();let mut expected=vec![0x73];expected.extend(point[0].to_le_bytes());expected.extend(point[1].to_le_bytes());assert_eq!(choice.request(4,&w,&nav).unwrap(),expected);}
 }
 assert_eq!(native_room::map_selection(3,563).unwrap().as_slice(),[0xd1,3,0x33,2]);
 assert!(native_room::map_selection(3,574).is_err());assert!(native_room::map_selection(255,573).is_err());
 assert!(LoginController::default().select_room_map(573).is_err());
}
fn player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()> {
 let mut c=LoginController::default();let start=Instant::now();let cancel=AtomicBool::new(false);let other=if role=="a"{"b"}else{"a"};let map:u16=std::env::var("LITO_ARENA_PORTAL_MAP").unwrap_or("573".into()).parse()?;ensure!([573,563].contains(&map),"unreviewed arena map");let alternate=if map==573{563}else{573};
 let result=(||->Result<()> {
  c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;wait(&mut c,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;wait(&mut c,|s|s.phase==P::SourcesReady)?;c.select_source(1)?;wait(&mut c,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(20);while c.snapshot().phase!=P::CharactersReady{c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"bootstrap failed");thread::sleep(Duration::from_millis(5));}
  c.select_character(0)?;wait(&mut c,|s|s.phase==P::CharacterSelected)?;c.enter_channel(0)?;wait(&mut c,|s|s.phase==P::LobbyReady)?;
  std::fs::write(root.join(format!("{role}-lobby")),b"ready")?;
  if role=="a" {
   wait(&mut c,|_|root.read_dir().unwrap().filter_map(Result::ok).filter(|e|e.file_name().to_string_lossy().ends_with("-lobby")).count()==30)?;
   wait(&mut c,|s|!s.rooms.is_empty())?;c.join_room(0,"")?;wait(&mut c,|s|s.phase==P::RoomReady)?;
   c.edit_room(&lito_modern::modern_lito::native_room::Settings{name:"原生入口30人",password:"",attributes:0,capacity:30,options:None})?;wait(&mut c,|s|s.phase==P::RoomReady&&s.joined_room.as_ref().is_some_and(|r|r.capacity==30))?;
   for _ in 0..6 {if c.snapshot().joined_room.as_ref().unwrap().mode==3{break}c.step_room_mode(1)?;wait(&mut c,|s|s.phase==P::RoomReady)?;}
   ensure!(c.snapshot().joined_room.as_ref().unwrap().mode==3,"siege mode not reached");
   let old=c.snapshot().joined_room.as_ref().unwrap().clone();
   c.select_room_map(alternate)?;ensure!(c.select_room_map(573).is_err()&&c.set_ready(true).is_err(),"map request competitors accepted");ensure!(c.snapshot().joined_room.as_ref().unwrap().map_id==old.map_id,"predicted map");
   wait(&mut c,|s|s.phase==P::RoomReady&&s.joined_room.as_ref().is_some_and(|r|r.map_id==alternate))?;
   c.select_room_map(map)?;wait(&mut c,|s|s.phase==P::RoomReady&&s.joined_room.as_ref().is_some_and(|r|r.map_id==map))?;
   let room=c.snapshot().joined_room.as_ref().unwrap();std::fs::write(root.join("created-room.json"),serde_json::to_vec(room)?)?;
  }else{
   wait(&mut c,|_|root.join("created-room.json").exists())?;
   if role=="e30" && std::env::var_os("LITO_ARENA_GUI_SUPPORT").is_some(){wait(&mut c,|_|root.join("gui-last-peer-join").exists())?;}
   let room:Value=serde_json::from_slice(&std::fs::read(root.join("created-room.json"))?)?;let id=room["id"].as_u64().unwrap() as u32;
   c.refresh_rooms()?;wait(&mut c,|s|s.phase==P::LobbyReady&&s.rooms.iter().any(|r|r.id==id))?;let at=c.snapshot().rooms.iter().position(|r|r.id==id).unwrap();c.join_room(at,"")?;wait(&mut c,|s|s.phase==P::RoomReady)?;ensure!(c.select_room_map(563).is_err(),"non-owner map selection accepted");
  }
  let desired=if role=="a"||account.strip_prefix("Siege").is_some_and(|n|n.parse::<u8>().unwrap()<=16){0}else{1};let end=Instant::now()+Duration::from_secs(480);
  loop{ensure!(!fixture_aborted(),"peer fixture failed");c.poll();ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"battle setup {:?}",c.snapshot().error);let s=c.snapshot();if s.phase==P::BattlePending&&s.world.ready()&&s.world.hp.is_some(){break}if s.phase==P::RoomReady{let r=s.joined_room.as_ref().context("room")?;let own=r.members.iter().find(|m|Some(m.name.as_str())==s.selected_character.as_deref()).context("self")?;if own.force!=desired{c.set_force(desired)?;}else if own.state==0&&r.members.len()==30&&r.members.iter().filter(|m|m.force==0).count()==15&&r.members.iter().filter(|m|m.force==1).count()==15{c.set_ready(true)?;}}thread::sleep(Duration::from_millis(5));}
  if !["a","b"].contains(&role){
   let battle=c.snapshot().clone();wait(&mut c,|_|root.join("a-report-saved").exists()&&root.join("b-report-saved").exists())?;
   std::fs::write(root.join(format!("{role}-siege-member.json")),serde_json::to_vec_pretty(&json!({"account":account,"battle":battle,"after":c.snapshot()}))?)?;return Ok(());
  }
  let entered=c.snapshot().clone();ensure!(entered.world.map_id==Some(map),"wrong battle map");let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&entered.world,&cancel)?;let nav=model.navigation();
  if role=="b"{wait(&mut c,|_|root.join("a-near-entrance").exists())?;}
  ready_frame(&mut c,&mut model,start)?;
  let own=c.snapshot().world.own_actor().unwrap();let candidates=if map==573{[[32755,32792],[32742,32797]]}else{[[32813,32874],[32799,32910]]};
  let mut routes=vec![];let mut route_errors=vec![];
  for source in candidates{match nav.route_to_map_entrance_avoiding([own.x,own.y],source,2,|x,y|lito_modern::modern_lito::native_obstacles::blocks(&c.snapshot().world,x,y)){Ok(route)=>routes.push((route,source)),Err(e)=>route_errors.push(format!("{source:?}: {e:#}"))}}
  routes.sort_by_key(|(r,_)|r.len());let (route,source)=routes.into_iter().next().ok_or_else(||anyhow::anyhow!("no reachable original entrance: {route_errors:?}"))?;
  let entrance_approach_steps=route.len();

  let mut terrain_residences=vec![];let mut previous_surface:Option<lito_modern::modern_lito::native_world_scene::InputSurface>=None;
  for h in route{let frame=ready_frame(&mut c,&mut model,start)?;
   if terrain_residences.last().is_none_or(|v:&Value|v["native_terrain_center"]!=frame.receipt["native_terrain_center"]){
    let n=terrain_residences.len();terrain_residences.push(json!({"native_terrain_center":frame.receipt["native_terrain_center"],"terrain_reload_count":frame.receipt["terrain_reload_count"],"native_terrain_block":frame.receipt["native_terrain_block"],"native_terrain_block_span":frame.receipt["native_terrain_block_span"],"native_terrain_window_pixels":frame.receipt["native_terrain_window_pixels"],"terrain_resident_sectors":frame.receipt["terrain_resident_sectors"],"view_position":frame.receipt["view_position"],"map_sectors":frame.receipt["map_sectors"],"geometry_sha256":frame.receipt["geometry_sha256"],"budget":frame.receipt["static_resource_budget"]}));
    let mut png=png::Encoder::new(std::fs::File::create(root.join(format!("{role}-terrain-{n}.png")))?,800,600);png.set_color(png::ColorType::Rgba);png.set_depth(png::BitDepth::Eight);png.write_header()?.write_image_data(frame.canvas.rgba())?;
   }
   if let Some(surface)=previous_surface.take(){ensure!(!surface.input_ready(&c.snapshot().world),"pre-walk input survived next authority cell");}previous_surface=frame.input_surface(&c.snapshot().world);
let a=c.snapshot().world.own_actor().unwrap();let target=native_world::destination(a.x,a.y,h)?;c.move_checked(h,&nav)?;wait(&mut c,|s|s.world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;}
  std::fs::write(root.join(format!("{role}-near-entrance")),b"ready")?;wait(&mut c,|_|root.join(format!("{other}-near-entrance")).exists())?;
  let mut transitions=vec![];let mut admitted=false;
  for n in 0..2 {
   let frame=ready_frame(&mut c,&mut model,start)?;let before=c.snapshot().clone();let choice=EntryChoice::nearest(c.transport().snapshot().epoch,&before.world,&nav)?;let request_at=Instant::now();c.enter_map_portal(choice.clone(),&frame)?;
   ensure!(c.enter_map_portal(choice.clone(),&frame).is_err()&&c.move_checked(0,&nav).is_err(),"duplicate portal or move accepted");
   ensure!(c.snapshot().world.generation==before.world.generation,"predicted portal");
   // Rejection has no wire response; wait the real bounded request deadline.
   while c.portal_entry_feedback()==EntryFeedback::Pending {ensure!(!fixture_aborted(),"peer fixture failed");c.poll();ensure!(request_at.elapsed()<Duration::from_secs(10),"portal deadline");thread::sleep(Duration::from_millis(10));}
   if c.portal_entry_feedback()!=EntryFeedback::NoResponse {
    wait(&mut c,|s|s.world.completed_portals>before.world.completed_portals&&s.world.ready()&&s.world.hp.is_some())?;
   }
   let after=c.snapshot().clone();admitted=after.world.completed_portals>before.world.completed_portals;
   ensure!(after.inventory.owner==before.inventory.owner&&serde_json::to_value(&after.inventory.items)?==serde_json::to_value(&before.inventory.items)?,"portal inventory mutation");
   if admitted {ensure!(after.world.map_id==Some(map)&&after.world.generation==before.world.generation+1&&c.portal_entry_feedback()==EntryFeedback::SceneUpdated,"portal generation");model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&after.world,&cancel)?;let f=ready_frame(&mut c,&mut model,start)?;ensure!(c.enter_map_portal(choice,&f).is_err(),"old choice survived");}
   else{ensure!(c.portal_entry_feedback()==EntryFeedback::NoResponse&&after.world.generation==before.world.generation&&after.world.own_actor().unwrap().x==before.world.own_actor().unwrap().x,"refusal changed authority");}
   transitions.push(json!({"before":before,"after":after,"admitted":admitted}));
   if n==0{std::fs::write(root.join(format!("{role}-first-portal")),b"ready")?;wait(&mut c,|_|root.join(format!("{other}-first-portal")).exists())?;}
  }
  let f=ready_frame(&mut c,&mut model,start)?;let own=c.snapshot().world.own_actor().unwrap();let h=(0..8).find(|h|nav.can_step(own.x.into(),own.y.into(),*h)&&native_world::destination(own.x,own.y,*h).is_ok_and(|(x,y)|!c.snapshot().world.actors.values().any(|a|a.id!=own.id&&a.x==x&&a.y==y))).context("post portal move")?;let target=native_world::destination(own.x,own.y,h)?;c.move_checked(h,&f.navigation)?;wait(&mut c,|s|s.world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;
  std::fs::write(root.join(format!("{role}-arena-portal.json")),serde_json::to_vec_pretty(&json!({"role":role,"map":map,"terrain_residences":terrain_residences,"approach_source":source,"approach_steps":entrance_approach_steps,"entered":entered,"transitions":transitions,"admitted":admitted,"after_walk":c.snapshot(),"walk_target":target}))?)?;std::fs::write(root.join(format!("{role}-report-saved")),b"done")?;wait(&mut c,|_|root.join(format!("{other}-report-saved")).exists())?;Ok(())})();
 if let Err(e)=&result{ABORT.store(true,Ordering::Release);std::fs::write(root.join(format!("{role}-arena-portal-error.json")),serde_json::to_vec_pretty(&json!({"error":format!("{e:#}"),"snapshot":c.snapshot()}))?)?;}
 c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(5));}ensure!(c.is_quiescent(),"worker leak");result
}
#[test]
#[ignore="requires owned C02 native arena portal harness"]
fn actual_siege_portal_checks_both_teams_and_retires_scene(){
 ABORT.store(false,Ordering::Release);let root=std::path::PathBuf::from(std::env::var("LITO_ARENA_PORTAL_ROOT").unwrap());assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));let r:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();assert_eq!(r["jar_sha256"],std::env::var("LITO_ARENA_PORTAL_REFERENCE_SHA").unwrap());let peer=format!("127.0.0.1:{}",r["port"].as_u64().unwrap()).parse().unwrap();thread::scope(|s|{let mut handles=vec![];for n in 1..=30{let role=if n==1{"a".to_owned()}else if n==2{"b".to_owned()}else{format!("e{n:02}")};let account=if n==1{"PeerFixture".to_owned()}else if n==2{"PeerFixture2".to_owned()}else{format!("Siege{n:02}")};let root=&root;handles.push(s.spawn(move||player(root,peer,&role,&account)));}for h in handles{let r=h.join().unwrap();assert!(r.is_ok(),"{r:?}");}});
 let a:Value=serde_json::from_slice(&std::fs::read(root.join("a-arena-portal.json")).unwrap()).unwrap();let b:Value=serde_json::from_slice(&std::fs::read(root.join("b-arena-portal.json")).unwrap()).unwrap();assert_ne!(a["admitted"],b["admitted"]);
}



fn gui_sector_route(nav:&lito_modern::modern_lito::native_navigation::Navigation,start:[u16;2],candidates:[[u16;2];2],blocked:impl Fn(u16,u16)->bool)->Result<(Vec<u8>,[u16;2])>{
 let end=|from:[u16;2],route:&[u8]|->Result<[u16;2]>{route.iter().try_fold(from,|p,h|native_world::destination(p[0],p[1],*h).map(|(x,y)|[x,y]))};
 let sector=|p:[u16;2]|[p[0]>>6,p[1]>>6];let mut direct=vec![];
 for target in candidates{if let Ok(route)=nav.route_to_map_entrance_avoiding(start,target,2,|x,y|blocked(x,y)){direct.push((route,target));}}
 direct.sort_by_key(|(r,_)|r.len());let (route,target)=direct.into_iter().next().context("no original source entrance route")?;
 let mut cell=start;let mut crosses=false;for h in &route{cell=end(cell,&[*h])?;crosses|=sector(cell)!=sector(start);}if crosses{return Ok((route,target))}
 // Preserve the original spawn and entrance. Exercise an actual neighboring
 // S32 sector through ordinary navigation and return by real GUI key steps.
 let x=i32::from(start[0]&!63);let y=i32::from(start[1]&!63);let sx=i32::from(start[0]);let sy=i32::from(start[1]);let mut detours=vec![];
 for [wx,wy] in [[x-2,sy],[x+65,sy],[sx,y-2],[sx,y+65]]{
  let (Ok(wx),Ok(wy))=(u16::try_from(wx),u16::try_from(wy))else{continue};
  let Ok(mut lead)=nav.route_to_range_avoiding(start,[wx,wy],1,|x,y|blocked(x,y))else{continue};let turn=end(start,&lead)?;if sector(turn)==sector(start){continue}
  let Ok(tail)=nav.route_to_map_entrance_avoiding(turn,target,2,|x,y|blocked(x,y))else{continue};lead.extend(tail);detours.push(lead);
 }
 detours.sort_by_key(Vec::len);let route=detours.into_iter().next().context("no original neighbor-sector walk and return to entrance")?;Ok((route,target))
}
#[test]
fn close_gui_spawn_crosses_real_original_sector_before_source_entrance(){
 use lito_modern::modern_lito::{resources::SharedMapSectorKey,world_map::{WorldMap,SectorInput},native_navigation::Navigation};
 let map_id=563;let root=Path::new(r"D:\lineage開發\0-客戶端\LiTo\LTmap\563");let mut input=vec![];
 for e in std::fs::read_dir(root).unwrap(){let p=e.unwrap().path();if p.extension().is_none_or(|x|x!="s32"){continue}let n=p.file_stem().unwrap().to_str().unwrap();input.push(SectorInput{key:SharedMapSectorKey{map_id,sector_x:u16::from_str_radix(&n[..4],16).unwrap(),sector_y:u16::from_str_radix(&n[4..],16).unwrap()},bytes:std::fs::read(p).unwrap()});}
 let geometry=WorldMap::from_native_sector_bytes(input,&AtomicBool::new(false)).unwrap();let nav=Navigation::from_geometry(&geometry).unwrap();
 // Actual close spawn and live guardian centers observed in041752-826. These
 // coordinates are a static route regression fixture, never authority writes.
 let start=[32774u16,32939u16];let blocked=|x,y|[[32641,32927],[32774,32939],[32835,32889]].contains(&[x,y]);let (route,target)=gui_sector_route(&nav,start,[[32813,32874],[32799,32910]],blocked).unwrap();let mut cell=start;let mut crossed=false;
 for h in route{assert!(nav.can_step(cell[0].into(),cell[1].into(),h));let (x,y)=native_world::destination(cell[0],cell[1],h).unwrap();assert!(!blocked(x,y));cell=[x,y];crossed|=[cell[0]>>6,cell[1]>>6]!=[start[0]>>6,start[1]>>6];}
 assert!(crossed);assert!(nav.has_portal_source(target));assert!(cell[0].abs_diff(target[0]).max(cell[1].abs_diff(target[1]))<=2);
}

fn gui_original_routes(root:&Path,map_id:u32)->Result<()> {
 use lito_modern::modern_lito::{resources::SharedMapSectorKey,world_map::{WorldMap,SectorInput},native_navigation::{self,Navigation}};
 use sha2::{Digest,Sha256};use std::collections::BTreeSet;
 let deadline=Instant::now()+Duration::from_secs(120);let starts=root.join("gui-starts.json");
 while !starts.exists(){ensure!(!fixture_aborted(),"GUI setup aborted");ensure!(Instant::now()<deadline,"GUI starts deadline");thread::sleep(Duration::from_millis(10));}
 let state:Value=serde_json::from_slice(&std::fs::read(starts)?)?;ensure!(state["map"].as_u64()==Some(map_id.into()),"GUI map mismatch");
 let source=Path::new(r"D:\lineage開發\0-客戶端\LiTo\LTmap").join(map_id.to_string());let mut files:Vec<_>=std::fs::read_dir(source)?.map(|e|e.map(|e|e.path())).collect::<std::io::Result<_>>()?;files.retain(|p|p.extension().is_some_and(|x|x=="s32"));files.sort();ensure!(!files.is_empty()&&files.len()<=64,"original sector bound");
 let mut inputs=vec![];let mut sector_pins=vec![];
 for p in files{let n=p.file_stem().unwrap().to_str().context("sector filename")?;ensure!(n.len()==8,"sector name length");let bytes=std::fs::read(&p)?;sector_pins.push(json!({"file":p.file_name().unwrap().to_str().unwrap(),"sha256":format!("{:X}",Sha256::digest(&bytes))}));inputs.push(SectorInput{key:SharedMapSectorKey{map_id,sector_x:u16::from_str_radix(&n[..4],16)?,sector_y:u16::from_str_radix(&n[4..],16)?},bytes});}
 let geometry=WorldMap::from_native_sector_bytes(inputs,&AtomicBool::new(false))?;let nav=Navigation::from_geometry(&geometry)?;
 let mut blocked=BTreeSet::new();
 for role in ["a","b"]{
  let a=&state[role];ensure!(a["phase"]=="BATTLE","GUI actor not in natural battle");
  for tower in a["guardian_towers"].as_array().context("passive guardian list")?{let id=tower["id"].as_u64().context("guardian ID")?;let gfx=tower["gfx"].as_u64().context("guardian gfx")?;if (0x6b000000..=0x6bffffff).contains(&id)&&[2665,2667,2669,2671].contains(&gfx)&&tower["action"]!=35 {blocked.insert([u16::try_from(tower["x"].as_u64().context("guardian x")?)?,u16::try_from(tower["y"].as_u64().context("guardian y")?)?]);}}
  if let Some(door)=a["door"].as_object(){if door.get("passable").and_then(Value::as_bool)==Some(false){blocked.insert([u16::try_from(door["x"].as_u64().context("door x")?)?,u16::try_from(door["y"].as_u64().context("door y")?)?]);}}
 }
 let candidates=if map_id==573{[[32755,32792],[32742,32797]]}else{ensure!(map_id==563,"unreviewed GUI map");[[32813,32874],[32799,32910]]};let keys=[0x68usize,0x26,0x66,0x27,0x62,0x28,0x64,0x25];
 let step=|from:[u16;2],h:u8|->Result<Value>{ensure!(native_navigation::heading(keys[usize::from(h)])==Some(h),"GUI key mapping");let (x,y)=native_world::destination(from[0],from[1],h)?;Ok(json!({"before":from,"after":[x,y],"key":keys[usize::from(h)]}))};
 let mut plan=json!({"map_id":map_id,"geometry_sha256":geometry.canonical_digest().iter().map(|b|format!("{b:02X}")).collect::<String>(),"sectors":sector_pins,"a":null,"b":null,"post_walks":{},"source_authority_modified":false});let mut posts:BTreeSet<[u16;2]>=candidates.into_iter().collect();
 for role in ["a","b"]{let a=&state[role];let start=[u16::try_from(a["x"].as_u64().context("GUI x")?)?,u16::try_from(a["y"].as_u64().context("GUI y")?)?];
  let (route,target)=gui_sector_route(&nav,start,candidates,|x,y|blocked.contains(&[x,y]))?;let mut current=start;let mut steps=vec![];
  for h in route{let v=step(current,h)?;current=[v["after"][0].as_u64().unwrap() as u16,v["after"][1].as_u64().unwrap() as u16];steps.push(v);}posts.insert(current);plan[role]=json!({"source":target,"start":start,"steps":steps});
 }
 for from in posts{let h=(0u8..8).find(|h|nav.can_step(from[0].into(),from[1].into(),*h)&&native_world::destination(from[0],from[1],*h).is_ok_and(|(x,y)|!blocked.contains(&[x,y]))).context("no original post-portal move")?;plan["post_walks"][format!("{},{}",from[0],from[1])]=step(from,h)?;}
 let tmp=root.join("gui-routes.tmp");std::fs::write(&tmp,serde_json::to_vec_pretty(&plan)?)?;std::fs::rename(tmp,root.join("gui-routes.json"))?;Ok(())
}
#[test]
#[ignore="requires two owned native GUI windows and original C02 thirty-player siege"]
fn actual_siege_gui_supports_twenty_eight_players(){
 ABORT.store(false,Ordering::Release);assert_eq!(std::env::var("LITO_ARENA_GUI_SUPPORT").unwrap(),"1");let root=std::path::PathBuf::from(std::env::var("LITO_ARENA_PORTAL_ROOT").unwrap());assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));let r:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();assert_eq!(r["jar_sha256"],std::env::var("LITO_ARENA_PORTAL_REFERENCE_SHA").unwrap());let map:u32=std::env::var("LITO_ARENA_PORTAL_MAP").unwrap().parse().unwrap();assert!([573,563].contains(&map));let peer=format!("127.0.0.1:{}",r["port"].as_u64().unwrap()).parse().unwrap();
 let result=thread::scope(|s|->Result<()>{let mut handles=vec![];for n in 3..=30{let root=&root;handles.push(s.spawn(move||player(root,peer,&format!("e{n:02}"),&format!("Siege{n:02}"))));}let plan=gui_original_routes(&root,map);if let Err(e)=&plan{ABORT.store(true,Ordering::Release);std::fs::write(root.join("gui-planner-error.json"),serde_json::to_vec_pretty(&json!({"error":format!("{e:#}")}))?)?;}let mut errors=vec![];for h in handles{match h.join(){Ok(Ok(()))=>{},other=>{ABORT.store(true,Ordering::Release);errors.push(format!("{other:?}"));}}}plan?;ensure!(errors.is_empty(),"GUI support failures {errors:?}");Ok(())});assert!(result.is_ok(),"{result:?}");
}
