//! Formal hotel allocation, return and server-driven room entry. Shared keys/expiry are separate gates.
use super::{native_login::{LoginController,LoginPhase},native_world_scene::Model,native_npc_document::Catalog};
use anyhow::{ensure,Context,Result};use serde_json::{json,Value};
use std::{path::Path,sync::atomic::AtomicBool,time::{Instant,Duration}};
fn poll(c:&mut LoginController)->Result<()>{c.poll();ensure!(c.snapshot().phase==LoginPhase::BattlePending,"hotel left world: {:?}",c.snapshot().error);Ok(())}
fn response(c:&mut LoginController)->Result<()>{let until=Instant::now()+Duration::from_secs(8);loop{poll(c)?;if !c.snapshot().merchant.waiting&&c.snapshot().merchant.dialog.is_some(){break;}ensure!(Instant::now()<until,"hotel response timeout");std::thread::sleep(Duration::from_millis(10));}let until=Instant::now()+Duration::from_millis(250);while Instant::now()<until{poll(c)?;std::thread::sleep(Duration::from_millis(10));}Ok(())}
pub(super) fn run(c:&mut LoginController,model:&mut Model,root:&Path,role:&str,other:&str)->Result<Value>{
 const NPC:u32=1812987995;let cancel=AtomicBool::new(false);let start=Instant::now();let catalog=Catalog::load(&super::native_startup::diagnostic_resource_root()?,&cancel)?;
 let profile=std::env::var("LITO_NATIVE_PEER_PROFILE")?;let expiry=matches!(profile.as_str(),"rental-hotel-expiry"|"rental-hotel-middle-expiry");let return_services=profile=="rental-return"&&std::env::var("LITO_NATIVE_RETURN_SERVICES").ok().as_deref()==Some("1");let enter=profile.ends_with("-entry")||expiry||return_services;
 if role=="b"{let until=Instant::now()+Duration::from_secs(130);while !root.join("a-hotel-complete").exists(){poll(c)?;ensure!(Instant::now()<until,"hotel start barrier");std::thread::sleep(Duration::from_millis(10));}}
 let steps=super::native_peer_citizen_probe::walk(c,model,NPC,start,&cancel)?;let before=c.snapshot().clone();
 c.rental_npc_talk(NPC)?;response(c)?;let opening=c.snapshot().merchant.dialog.clone().context("hotel dialog")?;
 let arguments=super::native_npc_document::resolved_arguments(&opening.html,&opening.arguments,&c.snapshot().world.actors[&NPC].name);
 let text=catalog.get(&opening.html)?.text_with_args(&arguments)?;ensure!(text.contains("席巴"),"hotel original argument substitution");
 c.rental_npc_action(NPC,"room")?;response(c)?;let requested=c.snapshot().clone();let input=requested.merchant.input.as_ref().context("hotel numeric request")?;
 ensure!((input.minimum,input.current,input.maximum,input.mode)==(1,1,8,0),"formal hotel range changed");
 ensure!(catalog.get(&input.html)?.text_with_args(&input.arguments)?.contains("幾個鑰匙"),"original numeric question");
 let revision=requested.merchant.revision;ensure!(c.rental_npc_quantity(revision,0).is_err()&&c.rental_npc_quantity(revision,9).is_err()&&c.rental_npc_quantity(revision+1,1).is_err(),"invalid numeric input accepted");
 let quantity=if matches!(profile.as_str(),"rental-hotel-middle-entry"|"rental-hotel-middle-expiry"){if role=="a"{4}else{2}}else if role=="a"{8}else{1};let lease_requested=Instant::now();c.rental_npc_quantity(revision,quantity)?;response(c)?;let rented=c.snapshot().clone();
 ensure!(rented.merchant.dialog.as_ref().is_some_and(|d|d.html=="inn4"),"hotel allocation reply");
 ensure!(c.rental_npc_quantity(revision,quantity).is_err(),"old quantity request reused");
 c.rental_npc_action(NPC,"room")?;response(c)?;let repeated=c.snapshot().clone();ensure!(rented.inventory.items==repeated.inventory.items&&repeated.merchant.input.is_none(),"duplicate room granted keys");
 let frame=model.draw_at(&c.snapshot().world,&cancel,start.elapsed().as_nanos() as u64)?;let render=super::native_peer_probe::capture(root,role,1,"hotel-keys",&frame)?;
 let mut arrival=Value::Null;let mut room_render=Value::Null;let mut returned=Value::Null;
 if enter{
  c.rental_npc_action(NPC,"enter")?;
  let until=Instant::now()+Duration::from_secs(8);
  loop{poll(c)?;let w=&c.snapshot().world;if w.ready()&&w.completed_portals==before.world.completed_portals+1{break;}ensure!(Instant::now()<until,"hotel portal timeout: {:?}",c.snapshot().world.portal);std::thread::sleep(Duration::from_millis(10));}
  let until=Instant::now()+Duration::from_millis(300);while Instant::now()<until{poll(c)?;std::thread::sleep(Duration::from_millis(10));}
  let s=c.snapshot();let w=&s.world;let own=w.own_actor().context("hotel arrival actor")?;
  let expected=match quantity{1=>(501,32802,32796),2..=4=>(502,32800,32793),_=>(503,32798,32799)};
  ensure!((w.map_id,own.x,own.y)==(Some(expected.0),expected.1,expected.2),"hotel destination mismatch");
  ensure!(w.generation==before.world.generation+1&&own.id==before.world.own_actor().unwrap().id&&w.actors.len()==1,"old scene or peer leaked across hotel lease");
  ensure!(s.joined_room.as_ref().is_some_and(|r|Some(r.map_id)==w.map_id)&&s.inventory.items==rented.inventory.items,"hotel room/inventory mismatch");
  ensure!(s.merchant.owner.is_none()&&s.merchant.dialog.is_none()&&s.merchant.input.is_none()&&!s.merchant.waiting,"old NPC state survived portal");
  arrival=json!(s);*model=Model::load(&super::native_startup::diagnostic_resource_root()?,w,&cancel)?;
  let f=model.draw_at(w,&cancel,start.elapsed().as_nanos() as u64)?;room_render=super::native_peer_probe::capture(root,role,2,"hotel-entry",&f)?;
 }else{
  c.rental_npc_action(NPC,"return")?;response(c)?;let s=c.snapshot();ensure!(s.merchant.dialog.as_ref().is_some_and(|d|d.html=="inn20"),"hotel return page");ensure!(before.inventory.items==s.inventory.items,"returned keys remain in inventory");returned=json!(s);
 }
 std::fs::write(root.join(format!("{role}-hotel-complete")),b"complete")?;let until=Instant::now()+Duration::from_secs(130);while !root.join(format!("{other}-hotel-complete")).exists(){poll(c)?;ensure!(Instant::now()<until,"hotel completion barrier");std::thread::sleep(Duration::from_millis(10));}
 let mut natural_expiry=Value::Null;
 if expiry{
  let seconds=if quantity>=5{7200u64}else{3600u64};let deadline=lease_requested+Duration::from_secs(seconds+120);let mut report_at=Instant::now();
  loop{
   poll(c)?;let s=c.snapshot();
   if s.world.ready()&&s.world.map_id==Some(500)&&s.world.completed_portals==before.world.completed_portals+2&&s.inventory.items==before.inventory.items{break;}
   ensure!(Instant::now()<deadline,"natural hotel expiry deadline exceeded");
   if Instant::now()>=report_at{std::fs::write(root.join(format!("{role}-hotel-expiry-progress.json")),serde_json::to_vec(&json!({"state":"waiting_natural_expiry","quantity":quantity,"expected_seconds":seconds,"elapsed_ms":lease_requested.elapsed().as_millis(),"map":s.world.map_id}))?)?;report_at=Instant::now()+Duration::from_secs(60);}
   std::thread::sleep(Duration::from_millis(20));
  }
  let elapsed=lease_requested.elapsed().as_millis();ensure!(elapsed>=u128::from(seconds-2)*1000,"hotel returned before natural lease expiry");
  let until=Instant::now()+Duration::from_millis(300);while Instant::now()<until{poll(c)?;std::thread::sleep(Duration::from_millis(10));}
  let s=c.snapshot();let w=&s.world;let own=w.own_actor().context("expiry return actor")?;
  ensure!([own.x,own.y]==[32779,32832]&&w.generation==before.world.generation+2&&w.actors.contains_key(&NPC),"expiry village rebuild mismatch");
  ensure!(s.merchant.owner.is_none()&&s.merchant.dialog.is_none(),"expired hotel kept NPC state");
  *model=Model::load(&super::native_startup::diagnostic_resource_root()?,w,&cancel)?;let f=model.draw_at(w,&cancel,start.elapsed().as_nanos() as u64)?;
  natural_expiry=json!({"expected_seconds":seconds,"elapsed_ms":elapsed,"returned":s,"render":super::native_peer_probe::capture(root,role,3,"hotel-natural-expiry",&f)?,"clock_or_authority_injection":false});
  std::fs::write(root.join(format!("{role}-hotel-expired")),b"expired")?;
  let until=Instant::now()+Duration::from_secs(7400);while !root.join(format!("{other}-hotel-expired")).exists(){poll(c)?;ensure!(Instant::now()<until,"peer natural expiry barrier");std::thread::sleep(Duration::from_millis(20));}
 }
 Ok(json!({"steps":steps,"before":before,"opening":opening,"requested":requested,"quantity":quantity,"rented":rented,"repeated":repeated,"returned":returned,"render":render,"arrival":arrival,"room_render":room_render,"natural_expiry":natural_expiry,"server_authority_injection":false,"gui_verified":false,"room_entry_verified":enter}))
}
