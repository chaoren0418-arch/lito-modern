//! Walk to all four received merchants and rent/return every offered item.
use super::{native_login::{LoginController,LoginPhase,Snapshot},native_world_scene::Model,native_merchant::Listing,native_npc_document::Catalog};
use anyhow::{ensure,Context,Result};use serde_json::{json,Value};
use std::{path::Path,sync::atomic::AtomicBool,time::{Instant,Duration}};
fn wait(c:&mut LoginController,seconds:u64,ok:impl Fn(&Snapshot)->bool)->Result<()>{
 let until=Instant::now()+Duration::from_secs(seconds);loop{c.poll();ensure!(c.snapshot().phase==LoginPhase::BattlePending,"merchant probe disconnected: {:?}",c.snapshot().error);if ok(c.snapshot()){return Ok(());}ensure!(Instant::now()<until,"merchant probe response timeout");std::thread::sleep(Duration::from_millis(10));}
}
fn settle(c:&mut LoginController)->Result<()>{let until=Instant::now()+Duration::from_millis(160);wait(c,8,|_|Instant::now()>=until)}
pub(super) fn expiry(c:&mut LoginController,model:&mut Model,root:&Path,role:&str,other:&str)->Result<Value>{
 const GEM:u32=1811939330;let cancel=AtomicBool::new(false);let start=Instant::now();
 let before=c.snapshot().clone();let mut grants=vec![];let mut ids=vec![];
 if role=="b"{wait(c,420,|_|root.join("a-item-expiry-rented").exists())?;}
 for npc in [1812987906u32,1812987914,1812987915,1812987916]{
  let steps=super::native_peer_citizen_probe::walk(c,model,npc,start,&cancel)?;
  c.rental_npc_talk(npc)?;wait(c,8,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;
  c.rental_npc_action(npc,"buy")?;wait(c,8,|s|matches!(s.merchant.listing,Some(Listing::Buy{..})))?;
  let listing=c.snapshot().merchant.clone();let Listing::Buy{rows,..}=listing.listing.as_ref().unwrap()else{unreachable!()};
  let(index,row)=rows.iter().enumerate().min_by_key(|(_,r)|r.price).context("empty expiry catalogue")?;
  let price=row.price;let prior=c.snapshot().inventory.clone();let gems=prior.items.get(&GEM).context("expiry gem balance")?.count;ensure!(gems>=price,"expiry item not affordable");
  let requested_ms=start.elapsed().as_millis();c.rental_merchant_order(listing.revision,index,1)?;
  wait(c,8,|s|s.inventory.items.get(&GEM).is_some_and(|i|i.count==gems-price)&&s.inventory.items.keys().any(|id|!prior.items.contains_key(id)))?;settle(c)?;
  let rented=c.snapshot().clone();let added:Vec<_>=rented.inventory.items.keys().filter(|id|!prior.items.contains_key(id)).copied().collect();ensure!(added.len()==1,"expiry single rental identity");ids.push(added[0]);
  grants.push(json!({"npc":npc,"row_index":index,"price":price,"item":added[0],"requested_ms":requested_ms,"steps":steps,"listing":listing,"rented":rented}));
 }
 std::fs::write(root.join(format!("{role}-item-expiry-rented")),b"rented")?;
 let mut removed=vec![];let mut report_at=Instant::now();let deadline=Instant::now()+Duration::from_secs(3720);
 while removed.len()<ids.len(){
  c.poll();let s=c.snapshot();ensure!(s.phase==LoginPhase::BattlePending&&s.world.map_id==Some(500),"item expiry left rental scene");
  for &id in &ids{if !s.inventory.items.contains_key(&id)&&!removed.iter().any(|v:&Value|v["item"]==json!(id)){
   let elapsed_ms=start.elapsed().as_millis();let grant=grants.iter().find(|g|g["item"]==json!(id)).unwrap();ensure!(elapsed_ms>=grant["requested_ms"].as_u64().unwrap() as u128+3_598_000,"rental item disappeared before natural expiry");
   removed.push(json!({"item":id,"observed_ms":elapsed_ms,"inventory":s.inventory}));
  }}
  ensure!(Instant::now()<deadline,"natural merchant item expiry deadline exceeded");
  if Instant::now()>=report_at{std::fs::write(root.join(format!("{role}-item-expiry-progress.json")),serde_json::to_vec(&json!({"state":"waiting_natural_item_expiry","elapsed_ms":start.elapsed().as_millis(),"expected_seconds":3600,"items":ids,"removed":removed.len()}))?)?;report_at=Instant::now()+Duration::from_secs(60);}
  std::thread::sleep(Duration::from_millis(20));
 }
 settle(c)?;let expired=c.snapshot().clone();let mut expected=before.inventory.items.clone();expected.get_mut(&GEM).unwrap().count-=grants.iter().map(|g|g["price"].as_u64().unwrap() as u32).sum::<u32>();ensure!(expired.inventory.items==expected,"expiry changed non-rental inventory or refunded deposit");
 let frame=model.draw_at(&expired.world,&cancel,start.elapsed().as_nanos() as u64)?;let render=super::native_peer_probe::capture(root,role,2,"merchant-natural-expiry",&frame)?;
 std::fs::write(root.join(format!("{role}-item-expiry-done")),b"expired")?;wait(c,420,|_|root.join(format!("{other}-item-expiry-done")).exists())?;
 Ok(json!({"before":before,"grants":grants,"removed":removed,"expired":expired,"render":render,"expected_seconds":3600,"clock_or_authority_injection":false,"equipped_expiry_verified":false,"gui_verified":false}))
}
pub(super) fn run(c:&mut LoginController,model:&mut Model,root:&Path,role:&str,other:&str)->Result<Value>{
 const GEM:u32=1811939330;let cancel=AtomicBool::new(false);let start=Instant::now();let catalog=Catalog::load(Path::new(r"D:\lineage開發\0-客戶端"),&cancel)?;
 if role=="b"{wait(c,420,|_|root.join("a-merchants-complete").exists())?;}
 let mut shops=vec![];
 for npc in [1812987906u32,1812987914,1812987915,1812987916]{
  let steps=super::native_peer_citizen_probe::walk(c,model,npc,start,&cancel)?;let before=c.snapshot().inventory.clone();
  c.rental_npc_talk(npc)?;wait(c,8,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;
  let dialog=c.snapshot().merchant.dialog.clone().context("merchant original page")?;let page=catalog.get(&dialog.html)?;
  // Some original opening pages (Maya) link to a local trade page first.
  // Follow only an actual catalog link; local navigation sends no wire command.
  let local_page=if page.actions.iter().any(|a|!a.navigate&&a.command=="buy") {dialog.html.clone()} else {
   let link=page.actions.iter().filter(|a|a.navigate).find(|a|catalog.get(&a.command).is_ok_and(|d|d.actions.iter().any(|v|!v.navigate&&v.command=="buy"))).context("original merchant has no linked trade page")?;
   link.command.clone()
  };
  ensure!(catalog.get(&local_page)?.actions.iter().any(|a|!a.navigate&&a.command=="sell"),"original trade page lacks return action");
  c.rental_npc_action(npc,"buy")?;wait(c,8,|s|matches!(s.merchant.listing,Some(Listing::Buy{..})))?;
  let first=c.snapshot().merchant.clone();let Listing::Buy{rows,..}=first.listing.clone().unwrap()else{unreachable!()};ensure!(!rows.is_empty()&&rows.len()<=100,"merchant rows outside bounded catalogue");
  let gems=before.items.get(&GEM).context("rental gem balance")?.count;let(index,row)=rows.iter().enumerate().min_by_key(|(_,r)|r.price).unwrap();
  let messages=c.snapshot().world.system_message_count;c.rental_merchant_order(first.revision,index,gems/row.price+1)?;
  wait(c,8,|s|s.world.system_message_count>messages)?;let refusal=c.snapshot().world.last_system_message.clone();
  ensure!(c.snapshot().inventory.items==before.items&&refusal.as_ref().is_some_and(|(k,t)|*k==9&&t=="光之寶石不足。"),"merchant insufficient funds refusal mismatch");
  let mut transactions=vec![];
  for (index,expected) in rows.iter().enumerate(){
   c.rental_npc_action(npc,"buy")?;wait(c,8,|s|matches!(s.merchant.listing,Some(Listing::Buy{..})))?;
   let list=&c.snapshot().merchant;let Listing::Buy{rows:current,..}=list.listing.as_ref().unwrap()else{unreachable!()};ensure!(current==&rows,"merchant catalogue changed mid-session");
   let prior=c.snapshot().inventory.clone();let gems=prior.items.get(&GEM).unwrap().count;ensure!(gems>=expected.price,"catalogue item not affordable");
   c.rental_merchant_order(list.revision,index,1)?;
   wait(c,8,|s|s.inventory.items.get(&GEM).is_some_and(|i|i.count==gems-expected.price)&&s.inventory.items.keys().any(|id|!prior.items.contains_key(id)))?;settle(c)?;
   let rented=c.snapshot().inventory.clone();let added:Vec<_>=rented.items.keys().filter(|id|!prior.items.contains_key(id)).copied().collect();ensure!(added.len()==1,"single rental did not add one distinct item");let item=added[0];
   c.rental_npc_action(npc,"sell")?;wait(c,8,|s|matches!(s.merchant.listing,Some(Listing::Sell{..})))?;
   let sell=&c.snapshot().merchant;let Listing::Sell{rows:sell_rows,..}=sell.listing.as_ref().unwrap()else{unreachable!()};let(sell_index,sell_row)=sell_rows.iter().enumerate().find(|(_,r)|r.id==item).context("new rental missing from return list")?;let refund=sell_row.price;ensure!(refund==expected.price,"formal rental deposit refund changed");
   c.rental_merchant_order(sell.revision,sell_index,1)?;wait(c,8,|s|!s.inventory.items.contains_key(&item)&&s.inventory.items.get(&GEM).is_some_and(|i|i.count==gems))?;settle(c)?;
   let returned=c.snapshot().inventory.clone();ensure!(returned.items==prior.items,"rent/return did not restore inventory");
   transactions.push(json!({"row_index":index,"order_id":expected.id,"item":item,"price":expected.price,"refund":refund,"rented":rented,"returned":returned}));
  }
  let frame=model.draw_at(&c.snapshot().world,&cancel,start.elapsed().as_nanos() as u64)?;let render=super::native_peer_probe::capture(root,role,1,&format!("merchant-{npc}"),&frame)?;
  shops.push(json!({"npc":npc,"steps":steps,"before":before,"dialog":dialog,"local_page":local_page,"buy_list":first,"refusal":refusal,"transactions":transactions,"render":render}));
 }
 std::fs::write(root.join(format!("{role}-merchants-complete")),b"complete")?;wait(c,420,|_|root.join(format!("{other}-merchants-complete")).exists())?;
 Ok(json!({"shops":shops,"server_authority_injection":false,"gui_verified":false,"capacity_weight_and_expiry_verified":false}))
}
