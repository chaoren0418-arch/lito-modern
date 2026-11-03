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

fn weapon_player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()>{
 let mut c=LoginController::default();let cancel=AtomicBool::new(false);let start=Instant::now();
 let result=(||->Result<()>{
  c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
  wait(&mut c,15,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;
  wait(&mut c,15,|s|s.phase==P::SourcesReady)?;c.select_source(1)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
  let end=Instant::now()+Duration::from_secs(15);
  while c.snapshot().phase!=P::CharactersReady {c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closed),"GAME bootstrap failed");thread::sleep(Duration::from_millis(10));}
  c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected)?;c.select_rental_set(1)?;
  wait(&mut c,15,|s|s.phase==P::RentalReady)?;c.join_rental_room(0)?;wait(&mut c,15,|s|s.phase==P::BattlePending&&s.world.ready()&&s.world.actors.contains_key(&1812987996))?;
  if role=="b"{wait(&mut c,60,|_|root.join("a-weapon-equipped").exists())?;}
  let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&c.snapshot().world,&cancel)?;
  let npc=1812987906;let steps=walk(&mut c,&mut model,npc,&cancel,start)?;
  c.rental_npc_talk(npc)?;wait(&mut c,10,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;

  let before=c.snapshot().clone();let original=before.inventory.items.values().find(|i|i.use_type==1&&i.name.ends_with("($9)")).context("no received equipped weapon")?.id;
  let unequip=toggle(&mut c,original)?;ensure!(!c.snapshot().inventory.items[&original].name.ends_with("($9)"),"original weapon remained equipped");
  let baseline=c.snapshot().clone();const GEM:u32=1811939330;
  c.rental_npc_talk(npc)?;wait(&mut c,10,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;c.rental_npc_action(npc,"buy")?;
  wait(&mut c,10,|s|matches!(s.merchant.listing,Some(lito_modern::modern_lito::native_merchant::Listing::Buy{..})))?;
  let listing=c.snapshot().merchant.clone();let Some(lito_modern::modern_lito::native_merchant::Listing::Buy{rows,..})=&listing.listing else{unreachable!()};
  let (index,row)=rows.iter().enumerate().find(|(_,row)|row.icon==20015).context("formal dagger19002 missing")?;let price=row.price;let gems=baseline.inventory.items[&GEM].count;ensure!(gems>=price,"rental unaffordable");
  let requested_ms=start.elapsed().as_millis();c.rental_merchant_order(listing.revision,index,1)?;
  wait(&mut c,10,|s|s.inventory.items[&GEM].count==gems-price&&s.inventory.items.keys().any(|id|!baseline.inventory.items.contains_key(id)))?;
  let rented=c.snapshot().clone();let added:Vec<_>=rented.inventory.items.keys().filter(|id|!baseline.inventory.items.contains_key(id)).copied().collect();ensure!(added.len()==1&&rented.inventory.items[&added[0]].use_type==1,"single weapon grant mismatch");let item=added[0];
  let equip=toggle(&mut c,item)?;ensure!(c.snapshot().inventory.items[&item].name.ends_with("($9)"),"rental weapon not equipped");let equipped=c.snapshot().clone();
  let stale=lito_modern::modern_lito::native_item_actions::Choice::capture(&equipped,item)?;
  std::fs::write(root.join(format!("{role}-weapon-preparation.json")),serde_json::to_vec_pretty(&json!({"role":role,"npc":npc,"steps":steps,"before":before,"baseline":baseline,"original_weapon":original,"unequip":unequip,"listing":listing,"row_index":index,"price":price,"catalog_id":19002,"item":item,"rented":rented,"equip":equip,"equipped":equipped,"requested_ms":requested_ms,"expected_seconds":3600,"clock_or_authority_injection":false}))?)?;
  std::fs::write(root.join(format!("{role}-weapon-equipped")),b"equipped")?;
  let deadline=Instant::now()+Duration::from_secs(3720);let mut report_at=Instant::now();
  loop {c.poll();let state=c.snapshot();ensure!(state.phase==P::BattlePending&&state.world.map_id==Some(500),"natural weapon expiry left world");if !state.inventory.items.contains_key(&item){break}
   ensure!(state.inventory.items[&item]==equipped.inventory.items[&item],"equipped weapon changed before expiry");ensure!(Instant::now()<deadline,"natural weapon expiry deadline");
   if Instant::now()>=report_at{std::fs::write(root.join(format!("{role}-weapon-progress.json")),serde_json::to_vec_pretty(&json!({"status":"waiting_real_equipped_weapon_expiry","item":item,"catalog_id":19002,"elapsed_ms":start.elapsed().as_millis(),"expected_seconds":3600}))?)?;report_at=Instant::now()+Duration::from_secs(60);}
   thread::sleep(Duration::from_millis(20));
  }
  let observed_ms=start.elapsed().as_millis();ensure!(observed_ms>=requested_ms+3_598_000,"weapon disappeared early");let until=Instant::now()+Duration::from_millis(300);wait(&mut c,8,|_|Instant::now()>=until)?;
  let expired=c.snapshot().clone();let mut expected=baseline.inventory.items.clone();expected.get_mut(&GEM).unwrap().count-=price;ensure!(expired.inventory.items==expected,"expiry changed unrelated inventory or refunded gems");
  ensure!(expired.world.generation==baseline.world.generation&&expired.world.own_actor().unwrap().walk_action==baseline.world.own_actor().unwrap().walk_action,"expired weapon left stale walk family");
  ensure!(c.item_action(stale,lito_modern::modern_lito::native_item_actions::Action::Use).is_err(),"expired weapon choice accepted");ensure!(c.rental_merchant_order(listing.revision,index,1).is_err(),"old rental listing accepted");
  let restore=toggle(&mut c,original)?;let restored=c.snapshot().clone();let mut expected=before.inventory.items.clone();expected.get_mut(&GEM).unwrap().count-=price;ensure!(restored.inventory.items==expected&&restored.world.own_actor().unwrap().walk_action==before.world.own_actor().unwrap().walk_action,"original weapon not restored");
  let frame=model.draw_at(&restored.world,&cancel,start.elapsed().as_nanos() as u64)?;
  let png_file=root.join(format!("{role}-weapon-restored.png"));let mut enc=png::Encoder::new(std::fs::File::create(&png_file)?,800,600);enc.set_color(png::ColorType::Rgba);enc.set_depth(png::BitDepth::Eight);enc.write_header()?.write_image_data(frame.canvas.rgba())?;
  std::fs::write(root.join(format!("{role}-weapon-result.json")),serde_json::to_vec_pretty(&json!({"expired":expired,"restore":restore,"restored":restored,"observed_ms":observed_ms,"requested_ms":requested_ms,"frame":frame.receipt,"png":png_file,"stale_choices_rejected":true,"clock_or_authority_injection":false}))?)?;
  std::fs::write(root.join(format!("{role}-weapon-done")),b"expired")?;let other=if role=="a"{"b"}else{"a"};wait(&mut c,240,|_|root.join(format!("{other}-weapon-done")).exists())?;
  Ok(())
 })();
 c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(10));}ensure!(c.is_quiescent(),"weapon client not reclaimed");result
}
fn toggle(c:&mut LoginController,id:u32)->Result<Value>{
 use lito_modern::modern_lito::native_item_actions::{Choice,Action,Feedback};let before=c.snapshot().clone();let choice=Choice::capture(&before,id)?;c.item_action(choice.clone(),Action::Use)?;
 ensure!(c.snapshot().inventory==before.inventory,"local equipment prediction");ensure!(c.item_action(choice,Action::Use).is_err(),"duplicate equipment admitted");wait(c,8,|s|s.item_feedback!=Feedback::Pending)?;
 ensure!(c.snapshot().item_feedback==Feedback::InventoryChanged,"equipment not acknowledged");let until=Instant::now()+Duration::from_millis(300);wait(c,8,|_|Instant::now()>=until)?;Ok(json!({"before":before,"after":c.snapshot(),"no_prediction":true,"duplicate_pending_rejected":true}))
}
#[test]
#[ignore="requires owned formal loopback harness and a real one-hour lease"]
fn two_equipped_weapons_expire_after_real_one_hour_and_restore_original_weapon(){
 let root=std::path::PathBuf::from(std::env::var("LITO_WEAPON_ROOT").expect("owned root"));assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));
 let ready:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();assert_eq!(ready["jar_sha256"],"bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43");
 let peer:SocketAddr=format!("127.0.0.1:{}",ready["port"].as_u64().unwrap()).parse().unwrap();thread::scope(|scope|{let a=scope.spawn(||weapon_player(&root,peer,"a","PeerFixture"));let b=scope.spawn(||weapon_player(&root,peer,"b","PeerFixture2"));let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");});
}
