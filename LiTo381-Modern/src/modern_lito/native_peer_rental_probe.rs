//! Formal direct-room bootstrap, including loadout refusal and original map rendering.
use super::{native_connection::Limits,native_login::{LoginController,LoginPhase as P},native_world_scene::Model};
use anyhow::{ensure,bail,Context,Result};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,sync::atomic::AtomicBool,time::{Duration,Instant}};
pub(super) fn run(peer:SocketAddr,account:&str,password:&str,source:usize,role:&str,root:&Path)->Result<Value>{
    let mut c=LoginController::default();c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
    let start=Instant::now();let mut trace=vec![];let mut prior=Value::Null;
    let outcome=(||->Result<Value>{
        let mut logged=false;let mut stage=0;let mut refused=Value::Null;let mut selected=Value::Null;
        let mut model=None;let cancel=AtomicBool::new(false);let mut rendered=Value::Null;
        let mut captured=None;let mut captured_state=Value::Null;let (own,other_id,other)=if role=="a"{(1,3,"b")}else{(3,1,"a")};
        loop{
            c.poll();let s=c.snapshot();let row=json!({"phase":s.phase,"rental_set":s.rental_set,"channel_result":s.channel_result,"room":s.joined_room,"map":s.world.map_id,"actors":s.world.actors.len(),"error":s.error});
            if row!=prior{ensure!(trace.len()<4096,"rental trace bound");trace.push(json!({"ms":start.elapsed().as_millis(),"state":row}));prior=row;}
            ensure!(start.elapsed()<Duration::from_secs(150),"rental entry timeout stage {stage}: {prior}");
            if s.notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
            match c.snapshot().phase{
                P::ReadyForLogin if !logged=>{c.login(account,password)?;logged=true;},P::ReadyForLogin=>bail!("rental LOGIN rejected"),
                P::SourcesReady=>c.select_source(source)?,P::RedirectReady=>{ensure!(c.snapshot().redirect.context("GAME redirect")?.ip().is_loopback(),"nonloopback GAME");c.begin_game()?;},
                P::CharactersReady=>c.select_character(0)?,
                // T055-2：超出職業配裝表的 C_CA 由客戶端本地拒絕、不送出；伺服器端拒絕路徑不再由本探針觸發。
                P::CharacterSelected if stage==0=>{ensure!(c.select_rental_set(3).is_err()&&c.snapshot().phase==P::CharacterSelected&&c.snapshot().rental_set.is_none(),"invalid rental set not refused locally");refused=json!(c.snapshot());c.select_rental_set(0)?;stage=2;},
                P::RentalReady if stage==2=>{selected=json!(c.snapshot());c.join_rental_room(0)?;stage=3;},
                P::BattlePending if stage==3=>{
                    let w=&c.snapshot().world;
                    // Actor creation and S_E8 arrive as separate frames. Capture only
                    // once the peer marker has arrived, not the first actor-only frame.
                    if w.ready()&&w.actors.contains_key(&other_id)&&w.minimap.markers.contains_key(&other_id){
                        ensure!(w.own_actor().context("rental own")?.id==own,"rental identity");
                        ensure!(c.snapshot().joined_room.as_ref().is_some_and(|r|r.mode==6&&Some(r.map_id)==w.map_id),"rental mode/map mismatch");
                        if model.is_none(){model=Some(Model::load(Path::new(r"D:\lineage開發\0-客戶端"),w,&cancel)?);}
                        if rendered.is_null(){let f=model.as_mut().unwrap().draw_at(w,&cancel,0)?;rendered=super::native_peer_probe::capture(root,role,1,"rental-entry",&f)?;
                            captured_state=json!(c.snapshot());captured=Some(Instant::now());std::fs::write(root.join(format!("{role}-rental-rendered")),b"rendered")?;}
                    }
                },
                P::Failed|P::Closed=>bail!("rental entry failed: {:?}",c.snapshot().error),_=>{}
            }
            // Completion uses the captured frame's state, even if the other peer has already disconnected.
            if captured.is_some_and(|at|at.elapsed()>Duration::from_millis(500))&&root.join(format!("{other}-rental-rendered")).exists(){
                let profile=std::env::var("LITO_NATIVE_PEER_PROFILE")?;
                let return_services=profile=="rental-return"&&std::env::var("LITO_NATIVE_RETURN_SERVICES").ok().as_deref()==Some("1");
                let return_shared=profile=="rental-return"&&std::env::var("LITO_NATIVE_RETURN_SHARED").ok().as_deref()==Some("1");
                ensure!(!(return_services&&return_shared),"return probe variants must be separate");
                let shop=if profile=="rental-shop"||return_services{shop(&mut c,root,role,other,return_services)?}else{Value::Null};
                let citizen=if profile=="rental-citizen"{super::native_peer_citizen_probe::run(&mut c,model.as_mut().context("citizen world model")?,root,role,other)?}else{Value::Null};
                let hotel=if matches!(profile.as_str(),"rental-hotel"|"rental-hotel-entry"|"rental-hotel-middle-entry"|"rental-hotel-expiry"|"rental-hotel-middle-expiry")||return_services{super::native_peer_hotel_probe::run(&mut c,model.as_mut().context("hotel world model")?,root,role,other)?}else{Value::Null};
                let hotel_share=if profile=="rental-hotel-share"||return_shared{super::native_peer_hotel_share_probe::run(&mut c,model.as_mut().context("hotel share model")?,root,role,other)?}else{Value::Null};
                let merchants=if profile=="rental-merchants"{super::native_peer_merchants_probe::run(&mut c,model.as_mut().context("merchant world model")?,root,role,other)?}else{Value::Null};
                let equipped_expiry=if profile=="rental-equipped-expiry"{super::native_peer_equipped_expiry_probe::run(&mut c,model.as_mut().context("equipped expiry world model")?,root,role,other)?}else{Value::Null};
                let item_expiry=if profile=="rental-item-expiry"{super::native_peer_merchants_probe::expiry(&mut c,model.as_mut().context("item expiry world model")?,root,role,other)?}else{Value::Null};
                let weight=if profile=="rental-weight"{weight_refusal(&mut c,root,role,other)?}else{Value::Null};
                let returned=if return_shared{hotel_share["return_observation"].clone()}else if profile=="rental-return"{observe_return(&mut c,root,role,other)?}else{Value::Null};
                return Ok(json!({"status":if profile=="rental-return"{"real_peer_rental_return_observed"}else{"real_peer_rental_entry_completed"},"action_profile":profile,"role":role,"refused":refused,"selected":selected,
                    "end":captured_state,"render":rendered,"transport":c.transport().snapshot(),"shop":shop,"citizen":citizen,"hotel":hotel,"hotel_share":hotel_share,"merchants":merchants,"weight":weight,"item_expiry":item_expiry,"equipped_expiry":equipped_expiry,"return_observation":returned,"return_services_requested":return_services,"return_shared_requested":return_shared,"normal_return_verified":returned.get("reentry_verified").and_then(Value::as_bool).unwrap_or(false)}));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })();
    c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();std::thread::sleep(Duration::from_millis(3));}c.poll();
    std::fs::write(root.join(format!("{role}-client-trace.json")),serde_json::to_vec_pretty(&trace)?)?;
    ensure!(c.is_quiescent(),"rental worker did not stop");let mut v=outcome?;v["worker_quiescent"]=json!(true);v["shutdown"]=json!(c.snapshot().phase);Ok(v)
}
pub(super) fn observe_return(c:&mut LoginController,root:&Path,role:&str,other:&str)->Result<Value>{
    let before=c.snapshot().clone();c.return_from_rental()?;let start=Instant::now();let mut phases=vec![json!(c.snapshot().phase)];
    while start.elapsed()<Duration::from_secs(6){
        c.poll();let phase=json!(c.snapshot().phase);if phases.last()!=Some(&phase){phases.push(phase);}
        ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"rental return failed: {:?}",c.snapshot().error);
        if c.snapshot().phase==P::CharactersReady {break;}
        std::thread::sleep(Duration::from_millis(10));
    }
    let after=c.snapshot().clone();let observed_ms=start.elapsed().as_millis();
    if after.phase!=P::CharactersReady {
        return Ok(json!({"before":before,"after":after,"phases":phases,"observed_ms":observed_ms,"characters_ready":false,"reentry_verified":false}));
    }
    ensure!(after.rental_return_outcome==Some(super::native_login::RentalReturnOutcome::Normal),
        "reconnected directory is recovery, not verified normal rental return");
    ensure!(after.joined_room.is_none()&&after.world.map_id.is_none()&&after.inventory.items.is_empty()
        &&after.merchant.dialog.is_none()&&after.rental_set.is_none(),"rental source refresh retained retired state");
    c.select_character(0)?;wait(c,|s|s.phase==P::CharacterSelected)?;
    c.select_rental_set(0)?;wait(c,|s|s.phase==P::RentalReady)?;
    c.join_rental_room(0)?;wait(c,|s|s.phase==P::BattlePending&&s.world.ready())?;
    ensure!(c.snapshot().world.generation>before.world.generation&&c.snapshot().world.map_id==Some(500),"rental reentry reused old world");
    std::fs::write(root.join(format!("{role}-rental-reentered")),b"ready")?;
    let barrier=Instant::now();while !root.join(format!("{other}-rental-reentered")).exists(){
        c.poll();ensure!(barrier.elapsed()<Duration::from_secs(20),"other rental reentry timeout");
        ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"rental reentry disconnected");std::thread::sleep(Duration::from_millis(10));
    }
    let peer=if role=="a"{3}else{1};wait(c,|s|s.world.actors.contains_key(&peer))?;
    let reentered=c.snapshot().clone();let cancel=AtomicBool::new(false);
    let mut model=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),&reentered.world,&cancel)?;
    let frame=model.draw_at(&reentered.world,&cancel,0)?;
    let render=super::native_peer_probe::capture(root,role,2,"rental-return-reentered",&frame)?;
    Ok(json!({"before":before,"after":after,"phases":phases,"observed_ms":observed_ms,"characters_ready":true,"reentry_verified":true,"reentered":reentered,"reentry_render":render}))
}
fn wait(c:&mut LoginController,predicate:impl Fn(&super::native_login::Snapshot)->bool)->Result<()>{
    let deadline=Instant::now()+Duration::from_secs(8);
    loop{c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closed),"rental shop disconnected: {:?}",c.snapshot().error);
        if predicate(c.snapshot()){return Ok(());}ensure!(Instant::now()<deadline,"rental shop response timeout: {:?}",c.snapshot().merchant);std::thread::sleep(Duration::from_millis(10));}
}
fn shop(c:&mut LoginController,root:&Path,role:&str,other:&str,retain_for_return:bool)->Result<Value>{
    use super::native_merchant::Listing;
    const NPC:u32=1812987916;const GEM:u32=1811939330;
    let before=c.snapshot().clone();ensure!(before.world.actors.contains_key(&NPC),"merchant not received");
    c.rental_npc_talk(NPC)?;wait(c,|s|s.merchant.dialog.is_some())?;let dialog=json!(c.snapshot().merchant);
    c.rental_npc_action(NPC,"buy")?;wait(c,|s|matches!(s.merchant.listing,Some(Listing::Buy{..})))?;
    let first_list=c.snapshot().merchant.clone();let Listing::Buy{rows,..}=first_list.listing.as_ref().unwrap()else{unreachable!()};
    let(index,row)=rows.iter().enumerate().min_by_key(|(_,r)|r.price).context("empty rental buy list")?;let price=row.price;
    let gems=before.inventory.items.get(&GEM).context("received gem inventory")?.count;ensure!(price<=gems,"no affordable fixture item");
    let message=c.snapshot().world.system_message_count;c.rental_merchant_order(first_list.revision,index,gems/price+1)?;
    wait(c,|s|s.world.system_message_count>message)?;let refused=c.snapshot().clone();ensure!(refused.inventory.items==before.inventory.items,"refused order changed inventory");
    ensure!(refused.world.last_system_message.as_ref().is_some_and(|(kind,text)|*kind==9&&text=="光之寶石不足。"),"unexpected rental refusal");
    c.rental_npc_action(NPC,"buy")?;wait(c,|s|matches!(s.merchant.listing,Some(Listing::Buy{..})))?;
    let capacity_revision=c.snapshot().merchant.revision;let message=c.snapshot().world.message_count;
    c.rental_merchant_order(capacity_revision,index,257)?;
    wait(c,|s|s.world.message_count>message)?;
    let capacity_refused=c.snapshot().clone();ensure!(capacity_refused.world.last_message==Some((263,vec![]))&&capacity_refused.inventory.items==before.inventory.items,"capacity rejection changed inventory or returned wrong message");
    ensure!(c.rental_merchant_order(capacity_revision,index,1).is_err(),"capacity rejection reused submitted list");
    c.rental_npc_action(NPC,"buy")?;wait(c,|s|matches!(s.merchant.listing,Some(Listing::Buy{..})))?;
    let list=c.snapshot().merchant.clone();let Listing::Buy{rows,..}=list.listing.as_ref().unwrap()else{unreachable!()};ensure!(rows[index].price==price,"fixture list price changed");
    c.rental_merchant_order(list.revision,index,1)?;
    wait(c,|s|s.inventory.items.get(&GEM).is_some_and(|v|v.count==gems-price))?;
    let rented=c.snapshot().clone();let added:Vec<_>=rented.inventory.items.keys().filter(|id|!before.inventory.items.contains_key(id)).copied().collect();ensure!(added.len()==1,"rental distinct inventory item missing");let item=added[0];
    // Hold authority states long enough for the independent passive observer to sample them.
    let until=Instant::now()+Duration::from_millis(150);while Instant::now()<until{c.poll();std::thread::sleep(Duration::from_millis(10));}
    if retain_for_return {
        std::fs::write(root.join(format!("{role}-return-item-rented")),b"rented")?;
        let deadline=Instant::now()+Duration::from_secs(20);
        while !root.join(format!("{other}-return-item-rented")).exists(){c.poll();ensure!(Instant::now()<deadline,"peer retained purchase timeout");std::thread::sleep(Duration::from_millis(10));}
        return Ok(json!({"before":before,"dialog":dialog,"listing":first_list,"refused":refused,"capacity_refused":capacity_refused,"rented":rented,"retained_item":item,"price":price,"retained_for_return":true}));
    }
    c.rental_npc_action(NPC,"sell")?;wait(c,|s|matches!(s.merchant.listing,Some(Listing::Sell{..})))?;
    let sell=c.snapshot().merchant.clone();let Listing::Sell{rows,..}=sell.listing.as_ref().unwrap()else{unreachable!()};let(index,row)=rows.iter().enumerate().find(|(_,r)|r.id==item).context("rented item not returnable")?;let refund=row.price;
    c.rental_merchant_order(sell.revision,index,1)?;
    wait(c,|s|!s.inventory.items.contains_key(&item)&&s.inventory.items.get(&GEM).is_some_and(|v|v.count==gems-price+refund))?;
    let returned=c.snapshot().clone();
    std::fs::write(root.join(format!("{role}-shop-returned")),b"returned")?;
    let until=Instant::now()+Duration::from_millis(250);let deadline=Instant::now()+Duration::from_secs(15);
    while Instant::now()<until||!root.join(format!("{other}-shop-returned")).exists(){c.poll();ensure!(Instant::now()<deadline,"peer shop return barrier");std::thread::sleep(Duration::from_millis(10));}
    Ok(json!({"before":before,"dialog":dialog,"buy_list":first_list,"refused":refused,"capacity_refused":capacity_refused,"capacity_quantity":257,"rented":rented,"sell_list":sell,"returned":returned,"item":item,"price":price,"refund":refund,"server_authority_injection":false}))
}

fn weight_refusal(c:&mut LoginController,root:&Path,role:&str,other:&str)->Result<Value>{
    use super::native_merchant::Listing;
    const NPC:u32=1812987916;const GEM:u32=1811939330;
    let before=c.snapshot().clone();let mut refusals=vec![];
    c.rental_npc_talk(NPC)?;wait(c,|s|s.merchant.dialog.is_some())?;
    for (quantity,code) in [(1,82u16),(257,263),(1,82)]{
        c.rental_npc_action(NPC,"buy")?;wait(c,|s|matches!(s.merchant.listing,Some(Listing::Buy{..})))?;
        let list=c.snapshot().merchant.clone();let Listing::Buy{rows,..}=list.listing.as_ref().unwrap()else{unreachable!()};
        let(index,row)=rows.iter().enumerate().min_by_key(|(_,r)|r.price).context("weight list")?;
        ensure!(before.inventory.items.get(&GEM).context("weight gem")?.count>=row.price,"weight single item must be affordable");
        let count=c.snapshot().world.message_count;c.rental_merchant_order(list.revision,index,quantity)?;wait(c,|s|s.world.message_count>count)?;
        let s=c.snapshot().clone();ensure!(s.world.last_message==Some((code,vec![]))&&s.inventory.items==before.inventory.items,"weight/capacity refusal changed inventory or message");
        ensure!(c.rental_merchant_order(list.revision,index,1).is_err(),"weight rejected list reused");
        refusals.push(json!({"quantity":quantity,"expected_code":code,"listing":list,"snapshot":s}));
    }
    std::fs::write(root.join(format!("{role}-weight-complete")),b"complete")?;
    let end=Instant::now()+Duration::from_secs(15);while !root.join(format!("{other}-weight-complete")).exists(){c.poll();ensure!(Instant::now()<end,"weight peer barrier");std::thread::sleep(Duration::from_millis(10));}
    Ok(json!({"before":before,"refusals":refusals,"initial_fixture":"strength/constitution/originalStrength/originalConstitution = 1; original rental loadout retained","live_authority_mutations":false,"normal_character_weight_reachability_verified":false,"gui_verified":false}))
}
