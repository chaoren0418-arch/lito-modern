//! Actual key transfer and owner-initiated lease invalidation; no authority injection.
use super::{native_login::{LoginController,LoginPhase,Snapshot},native_world_scene::Model};
use anyhow::{ensure,Context,Result};use serde_json::{json,Value};
use std::{path::Path,sync::atomic::AtomicBool,time::{Instant,Duration}};
const NPC:u32=1812987995;
fn wait(c:&mut LoginController,seconds:u64,ok:impl Fn(&Snapshot)->bool)->Result<()>{
 let until=Instant::now()+Duration::from_secs(seconds);
 loop{c.poll();ensure!(c.snapshot().phase==LoginPhase::BattlePending,"hotel share world closed: {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(());}ensure!(Instant::now()<until,"hotel share wait timed out");std::thread::sleep(Duration::from_millis(10));}
}
fn response(c:&mut LoginController)->Result<()>{wait(c,8,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())}
fn settle(c:&mut LoginController)->Result<()>{let until=Instant::now()+Duration::from_millis(250);wait(c,8,|_|Instant::now()>=until)}
pub(super) fn run(c:&mut LoginController,model:&mut Model,root:&Path,role:&str,other:&str)->Result<Value>{
 let owner_exit=std::env::var("LITO_NATIVE_RETURN_SHARED").ok().as_deref()==Some("1");
 let cancel=AtomicBool::new(false);let start=Instant::now();let before=c.snapshot().clone();let mut proof=json!({"before":before});
 if role=="b"{wait(c,100,|_|root.join("hotel-key-dropped.json").exists())?;}
 let steps=super::native_peer_citizen_probe::walk(c,model,NPC,start,&cancel)?;proof["steps"]=json!(steps);
 c.rental_npc_talk(NPC)?;response(c)?;
 if role=="a"{
  c.rental_npc_action(NPC,"room")?;response(c)?;
  let revision=c.snapshot().merchant.revision;c.rental_npc_quantity(revision,2)?;response(c)?;settle(c)?;
  let rented=c.snapshot().clone();let keys:Vec<u32>=rented.inventory.items.keys().filter(|id|!before.inventory.items.contains_key(id)).copied().collect();ensure!(keys.len()==2,"shared rental key allocation");
  proof["rented"]=json!(rented);let key=keys[0];
  let choice=super::native_item_actions::Choice::capture(c.snapshot(),key)?;
  c.item_action(choice,super::native_item_actions::Action::Drop(1))?;
  wait(c,8,|s|!s.inventory.items.contains_key(&key)&&s.world.actors.values().any(|a|!rented.world.actors.contains_key(&a.id)&&a.experience==1))?;settle(c)?;
  let ground=c.snapshot().world.actors.values().find(|a|!rented.world.actors.contains_key(&a.id)&&a.experience==1).context("dropped ground key")?.clone();
  let frame=model.draw_at(&c.snapshot().world,&cancel,start.elapsed().as_nanos() as u64)?;
  ensure!(frame.ground_items.iter().any(|i|i.id==ground.id),"key not rendered as ground item");
  proof["dropped"]=json!(c.snapshot());proof["drop_render"]=super::native_peer_probe::capture(root,role,1,"hotel-shared-key",&frame)?;
  std::fs::write(root.join("hotel-key-dropped.json"),serde_json::to_vec(&json!({"inventory_key":key,"ground_id":ground.id,"keys":keys}))?)?;
  wait(c,100,|_|root.join("b-hotel-shared-entered").exists())?;
  if owner_exit {
   proof["before_owner_exit"]=json!(c.snapshot());
   proof["return_observation"]=super::native_peer_rental_probe::observe_return(c,root,role,other)?;
   ensure!(proof["return_observation"]["reentry_verified"]==true,"owner restart did not reenter");
  } else {
  // Reopen the received NPC so a long guest walk never reuses an expired dialog.
  c.rental_npc_talk(NPC)?;response(c)?;c.rental_npc_action(NPC,"return")?;response(c)?;
  wait(c,8,|s|s.inventory.items==before.inventory.items)?;settle(c)?;
  proof["returned"]=json!(c.snapshot());
  }
 }else{
  let transfer:Value=serde_json::from_slice(&std::fs::read(root.join("hotel-key-dropped.json"))?)?;
  let key=u32::try_from(transfer["inventory_key"].as_u64().context("inventory key")?)?;
  let ground=u32::try_from(transfer["ground_id"].as_u64().context("ground key")?)?;
  wait(c,8,|s|s.world.actors.contains_key(&ground))?;
  let frame=model.draw_at(&c.snapshot().world,&cancel,start.elapsed().as_nanos() as u64)?;
  let choice=super::native_ground_pickup::Choice::capture(c.snapshot(),&frame,ground)?;
  proof["pickup_render"]=super::native_peer_probe::capture(root,role,1,"hotel-shared-key",&frame)?;
  c.pickup(choice,1,&frame)?;wait(c,8,|s|s.inventory.items.contains_key(&key)&&!s.world.actors.contains_key(&ground))?;settle(c)?;
  proof["picked_up"]=json!(c.snapshot());
  c.rental_npc_talk(NPC)?;response(c)?;c.rental_npc_action(NPC,"enter")?;
  wait(c,8,|s|s.world.ready()&&s.world.map_id==Some(502)&&s.world.completed_portals==before.world.completed_portals+1)?;settle(c)?;
  let w=&c.snapshot().world;let own=w.own_actor().context("guest arrival")?;
  ensure!(w.actors.len()==1&&[own.x,own.y]==[32800,32793],"guest room arrival");
  proof["entered"]=json!(c.snapshot());*model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),w,&cancel)?;
  let frame=model.draw_at(w,&cancel,start.elapsed().as_nanos() as u64)?;proof["entry_render"]=super::native_peer_probe::capture(root,role,2,"hotel-shared-entry",&frame)?;
  std::fs::write(root.join("b-hotel-shared-entered"),b"entered")?;
  wait(c,12,|s|s.world.ready()&&s.world.map_id==Some(500)&&s.world.completed_portals==before.world.completed_portals+2&&s.inventory.items==before.inventory.items)?;settle(c)?;
  let w=&c.snapshot().world;let own=w.own_actor().context("guest returned")?;
  ensure!([own.x,own.y]==[32779,32832]&&w.generation==before.world.generation+2,"guest forced return destination/generation");
  ensure!(w.actors.contains_key(&NPC)&&(owner_exit||w.actors.contains_key(&1))&&c.snapshot().merchant.owner.is_none(),"village did not rebuild or old NPC survived");
  proof["returned"]=json!(c.snapshot());*model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),w,&cancel)?;
  let frame=model.draw_at(w,&cancel,start.elapsed().as_nanos() as u64)?;proof["return_render"]=super::native_peer_probe::capture(root,role,3,"hotel-shared-return",&frame)?;
  if owner_exit {
   proof["return_observation"]=super::native_peer_rental_probe::observe_return(c,root,role,other)?;
   ensure!(proof["return_observation"]["reentry_verified"]==true,"guest restart did not reenter");
  }
 }
 std::fs::write(root.join(format!("{role}-hotel-share-complete")),b"complete")?;
 wait(c,100,|_|root.join(format!("{other}-hotel-share-complete")).exists())?;
 proof["owner_restart_requested"]=json!(owner_exit);
 proof["server_authority_injection"]=json!(false);proof["gui_verified"]=json!(false);proof["natural_expiry_verified"]=json!(false);Ok(proof)
}
