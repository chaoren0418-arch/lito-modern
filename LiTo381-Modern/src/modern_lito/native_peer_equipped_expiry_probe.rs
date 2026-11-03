//! Real one-hour equipped rental gloves expiry. Only received client actions; no authority or clock mutation.
use super::{native_login::{LoginController,LoginPhase,Snapshot},native_world_scene::Model,
    native_item_actions::{Action,Choice,Feedback},native_merchant::Listing};
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{path::Path,sync::atomic::AtomicBool,time::{Duration,Instant}};
fn wait(c:&mut LoginController,seconds:u64,ok:impl Fn(&Snapshot)->bool)->Result<()> {
    let deadline=Instant::now()+Duration::from_secs(seconds);
    loop {c.poll();ensure!(c.snapshot().phase==LoginPhase::BattlePending,"equipped expiry left world");
        if ok(c.snapshot()){return Ok(());}ensure!(Instant::now()<deadline,"equipped expiry response timeout");
        std::thread::sleep(Duration::from_millis(10));}
}
fn settle(c:&mut LoginController)->Result<()> {let until=Instant::now()+Duration::from_millis(300);wait(c,8,|_|Instant::now()>=until)}
fn toggle(c:&mut LoginController,id:u32)->Result<Value> {
    let choice=Choice::capture(c.snapshot(),id)?;let before=c.snapshot().clone();
    c.item_action(choice.clone(),Action::Use)?;
    ensure!(c.snapshot().inventory==before.inventory,"equipment request predicted inventory");
    ensure!(c.item_action(choice.clone(),Action::Use).is_err(),"duplicate equipment admitted");
    wait(c,8,|s|s.item_feedback!=Feedback::Pending)?;
    ensure!(c.snapshot().item_feedback==Feedback::InventoryChanged,"equipment not acknowledged");settle(c)?;
    let after=c.snapshot().clone();ensure!(after.inventory.items.get(&id)!=before.inventory.items.get(&id),"equipment item unchanged");
    Ok(json!({"before":before,"after":after,"no_prediction":true,"duplicate_pending_rejected":true}))
}
pub(super) fn run(c:&mut LoginController,model:&mut Model,root:&Path,role:&str,other:&str)->Result<Value> {
    const NPC:u32=1812987914;const GEM:u32=1811939330;
    let cancel=AtomicBool::new(false);let start=Instant::now();
    if role=="b"{wait(c,180,|_|root.join("a-equipped-expiry-ready").exists())?;}
    let steps=super::native_peer_citizen_probe::walk(c,model,NPC,start,&cancel)?;
    let before=c.snapshot().clone();
    let original_glove=before.inventory.items.values().find(|i|i.use_type==20&&i.name.ends_with("($117)")).context("fixture has no received equipped glove")?.id;
    let unequip=toggle(c,original_glove)?;ensure!(!c.snapshot().inventory.items[&original_glove].name.ends_with("($117)"),"original glove still equipped");
    let baseline=c.snapshot().clone();
    c.rental_npc_talk(NPC)?;wait(c,8,|s|!s.merchant.waiting&&s.merchant.dialog.is_some())?;
    c.rental_npc_action(NPC,"buy")?;wait(c,8,|s|matches!(s.merchant.listing,Some(Listing::Buy{..})))?;
    let listing=c.snapshot().merchant.clone();let Listing::Buy{rows,..}=listing.listing.as_ref().unwrap()else{unreachable!()};
    let(index,row)=rows.iter().enumerate().min_by_key(|(_,r)|r.price).context("empty glove shop")?;
    let price=row.price;let gems=baseline.inventory.items[&GEM].count;ensure!(gems>=price,"glove rental unaffordable");
    let requested_ms=start.elapsed().as_millis();c.rental_merchant_order(listing.revision,index,1)?;
    wait(c,8,|s|s.inventory.items.get(&GEM).is_some_and(|i|i.count==gems-price)&&s.inventory.items.keys().any(|id|!baseline.inventory.items.contains_key(id)))?;settle(c)?;
    let rented=c.snapshot().clone();let added:Vec<_>=rented.inventory.items.keys().filter(|id|!baseline.inventory.items.contains_key(id)).copied().collect();
    ensure!(added.len()==1&&rented.inventory.items[&added[0]].use_type==20,"rental did not yield a single glove");let item=added[0];
    let equip=toggle(c,item)?;ensure!(c.snapshot().inventory.items[&item].name.ends_with("($117)"),"rental gloves not equipped");
    let equipped=c.snapshot().clone();let stale=Choice::capture(c.snapshot(),item)?;
    ensure!(i16::from(baseline.world.status.as_ref().unwrap().armor_class)-i16::from(equipped.world.status.as_ref().unwrap().armor_class)==10,"expected received glove AC decrease absent");
    let frame=model.draw_at(&equipped.world,&cancel,start.elapsed().as_nanos() as u64)?;
    let equipped_render=super::native_peer_probe::capture(root,role,1,"rental-glove-equipped",&frame)?;
    std::fs::write(root.join(format!("{role}-equipped-expiry-ready")),b"equipped")?;
    let deadline=Instant::now()+Duration::from_secs(3720);let mut report_at=Instant::now();
    loop {c.poll();let s=c.snapshot();ensure!(s.phase==LoginPhase::BattlePending&&s.world.map_id==Some(500),"equipped expiry left rental scene");
        if !s.inventory.items.contains_key(&item){break;}
        ensure!(s.inventory.items[&item]==equipped.inventory.items[&item],"rental gloves changed before expiry");
        ensure!(Instant::now()<deadline,"equipped natural expiry deadline");
        if Instant::now()>=report_at {
            std::fs::write(root.join(format!("{role}-equipped-expiry-progress.json")),serde_json::to_vec(&json!({"state":"waiting_real_equipped_expiry","item":item,"elapsed_ms":start.elapsed().as_millis(),"expected_seconds":3600}))?)?;
            report_at=Instant::now()+Duration::from_secs(60);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let observed_ms=start.elapsed().as_millis();ensure!(observed_ms>=requested_ms+3_598_000,"equipped glove disappeared early");settle(c)?;
    let expired=c.snapshot().clone();let mut expected=baseline.inventory.items.clone();expected.get_mut(&GEM).unwrap().count-=price;
    ensure!(expired.inventory.items==expected,"expiry changed other inventory or refunded gems");
    ensure!(expired.world.generation==baseline.world.generation,"expiry replaced world");
    ensure!(expired.world.status.as_ref().unwrap().armor_class==baseline.world.status.as_ref().unwrap().armor_class,"expired glove AC retained");
    ensure!(c.item_action(stale,Action::Use).is_err(),"expired choice reused");
    ensure!(c.rental_merchant_order(listing.revision,index,1).is_err(),"old merchant list reused");
    let frame=model.draw_at(&expired.world,&cancel,start.elapsed().as_nanos() as u64)?;
    let expired_render=super::native_peer_probe::capture(root,role,2,"rental-glove-expired",&frame)?;
    let restore=toggle(c,original_glove)?;let restored=c.snapshot().clone();let mut expected=before.inventory.items.clone();expected.get_mut(&GEM).unwrap().count-=price;
    ensure!(restored.inventory.items==expected,"original glove restoration differs");
    std::fs::write(root.join(format!("{role}-equipped-expiry-done")),b"expired")?;wait(c,240,|_|root.join(format!("{other}-equipped-expiry-done")).exists())?;
    Ok(json!({"npc":NPC,"row_index":index,"price":price,"item":item,"original_glove":original_glove,"steps":steps,"before":before,"unequip":unequip,"baseline":baseline,"listing":listing,"rented":rented,"equip":equip,"equipped":equipped,"expired":expired,"restore":restore,"restored":restored,"requested_ms":requested_ms,"observed_ms":observed_ms,"equipped_render":equipped_render,"expired_render":expired_render,"expected_seconds":3600,"stale_item_and_listing_rejected":true,"clock_or_authority_injection":false,"gui_verified":false,"weapon_expiry_verified":false,"merchant_extra_bonus_verified":false}))
}
