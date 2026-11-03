//! Isolated protocol probe: original local pages, real walking and formal rewards.
use super::{native_login::{LoginController,LoginPhase},native_world_scene::Model,native_npc_document::Catalog};
use anyhow::{ensure,Context,Result};use serde_json::{json,Value};
use std::{path::Path,sync::atomic::AtomicBool,time::{Instant,Duration}};
fn poll(c:&mut LoginController)->Result<()>{c.poll();ensure!(c.snapshot().phase==LoginPhase::BattlePending,"citizen left rental world: {:?}",c.snapshot().error);Ok(())}
fn response(c:&mut LoginController)->Result<()>{let until=Instant::now()+Duration::from_secs(8);loop{poll(c)?;if !c.snapshot().merchant.waiting&&c.snapshot().merchant.dialog.is_some(){return Ok(());}ensure!(Instant::now()<until,"citizen dialog timeout");std::thread::sleep(Duration::from_millis(10));}}
pub(super) fn walk(c:&mut LoginController,model:&mut Model,npc:u32,start:Instant,cancel:&AtomicBool)->Result<Vec<Value>>{
    let w=&c.snapshot().world;let own=w.own_actor().context("rental own actor")?;let target=w.actors.get(&npc).context("rental NPC not received")?;
    let nav=model.navigation();let route=nav.route_to_adjacent_avoiding([own.x,own.y],[target.x,target.y],|x,y|w.actors.values().any(|a|a.id!=own.id&&!a.dead&&a.draw_state.visible(false)&&a.x==x&&a.y==y))?;
    let mut steps=vec![];
    for heading in route{
        let until=Instant::now()+Duration::from_secs(8);
        loop{poll(c)?;let frame=model.draw_at(&c.snapshot().world,cancel,start.elapsed().as_nanos() as u64)?;
            if frame.movement_ready(&c.snapshot().world){break;}ensure!(Instant::now()<until,"rental walk animation timeout");std::thread::sleep(Duration::from_millis(10));}
        let a=c.snapshot().world.own_actor().unwrap();let before=[a.x,a.y];let (x,y)=super::native_world::destination(a.x,a.y,heading)?;
        c.move_checked(heading,&nav)?;
        let until=Instant::now()+Duration::from_secs(8);
        loop{poll(c)?;let frame=model.draw_at(&c.snapshot().world,cancel,start.elapsed().as_nanos() as u64)?;
            if frame.movement_ready(&c.snapshot().world){break;}ensure!(Instant::now()<until,"rental walk arrival timeout");std::thread::sleep(Duration::from_millis(10));}
        let a=c.snapshot().world.own_actor().unwrap();ensure!([a.x,a.y]==[x,y],"formal correction rejected rental move");steps.push(json!({"heading":heading,"before":before,"after":[x,y]}));
    }Ok(steps)
}
pub(super) fn run(c:&mut LoginController,model:&mut Model,root:&Path,role:&str,other:&str)->Result<Value>{
    let cancel=AtomicBool::new(false);let started=Instant::now();let catalog=Catalog::load(Path::new(r"D:\lineage開發\0-客戶端"),&cancel)?;
    // Move one endpoint at a time so a player, never fixture teleportation, owns each route.
    if role=="b"{let deadline=Instant::now()+Duration::from_secs(130);while !root.join("a-citizen-complete").exists(){poll(c)?;ensure!(Instant::now()<deadline,"citizen peer start barrier");std::thread::sleep(Duration::from_millis(10));}}
    let before=c.snapshot().clone();let mut stages=vec![];
    for (npc,command) in [(1812987905,"request LTC emblem of authority"),(1812987906,"request LTC emblem of craftsman"),(1812987907,"request LTC emblem of delight"),(1812987908,"request LTC emblem of peace"),(1812987904,"request LTC light jewel")]{
        let steps=walk(c,model,npc,started,&cancel)?;
        c.rental_npc_talk(npc)?;response(c)?;
        let opening=c.snapshot().merchant.dialog.clone().context("NPC opening dialog")?;
        let page=catalog.get(&opening.html)?;let link=page.actions.iter().find(|a|a.navigate&&a.command.to_ascii_lowercase().ends_with("_citizen")).context("original citizenship page link")?;
        let leaf=catalog.get(&link.command)?;ensure!(leaf.actions.iter().any(|a|!a.navigate&&a.command==command),"original page lacks qualified request");
        let inventory_before=c.snapshot().inventory.clone();c.rental_npc_action(npc,command)?;response(c)?;
        let granted=c.snapshot().clone();let returned=catalog.get(&granted.merchant.dialog.as_ref().context("NPC result dialog")?.html)?;
        ensure!(!returned.text.is_empty(),"empty original reward page");
        c.rental_npc_action(npc,command)?;response(c)?;let duplicate=c.snapshot().clone();ensure!(duplicate.inventory.items==granted.inventory.items,"duplicate NPC request changed inventory");
        let frame=model.draw_at(&c.snapshot().world,&cancel,started.elapsed().as_nanos() as u64)?;
        let image=super::native_peer_probe::capture(root,role,1,&format!("citizen-{npc}"),&frame)?;
        stages.push(json!({"npc":npc,"command":command,"opening":opening,"local_page":link.command,"steps":steps,"inventory_before":inventory_before,"granted":granted,"duplicate":duplicate,"render":image}));
        let until=Instant::now()+Duration::from_millis(160);while Instant::now()<until{poll(c)?;std::thread::sleep(Duration::from_millis(10));}
    }
    let end=c.snapshot().clone();std::fs::write(root.join(format!("{role}-citizen-complete")),b"complete")?;
    let deadline=Instant::now()+Duration::from_secs(130);while !root.join(format!("{other}-citizen-complete")).exists(){poll(c)?;ensure!(Instant::now()<deadline,"citizen peer completion barrier");std::thread::sleep(Duration::from_millis(10));}
    Ok(json!({"before":before,"stages":stages,"end":end,"server_authority_injection":false,"gui_verified":false}))
}
