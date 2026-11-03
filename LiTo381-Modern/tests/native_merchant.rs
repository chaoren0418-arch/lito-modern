use lito_modern::modern_lito::native_merchant::{self as m,Packet,Listing};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn fixture()->serde_json::Value{serde_json::from_slice(include_bytes!("fixtures/native-merchant-current.json")).unwrap()}
#[test]
fn formal_merchant_rows_and_dialog_are_exact_bounded_and_unsigned(){
    let f=fixture();for key in ["dialog","list","sell","boundary_buy"]{let b=bytes(f[key].as_str().unwrap());let p=m::parse(&b).unwrap();
        match p{Packet::Dialog(d)=>{assert_eq!(d.html,"Love_Start");assert_eq!(d.arguments,["中文",""]);},Packet::List(Listing::Buy{rows,..}) if key=="list"=>{assert_eq!(rows.len(),17);assert_eq!(rows[0].id,0);},Packet::List(Listing::Buy{rows,..})=>{assert_eq!(rows[0].id,u32::MAX);assert_eq!(rows[0].price,u32::MAX);assert_eq!(rows[0].detail.len(),255);},Packet::List(Listing::Sell{rows,..})=>{assert_eq!(rows[0].id,u32::MAX);assert_eq!(rows[0].price,u32::MAX);}}
        for n in 0..b.len(){assert!(m::parse(&b[..n]).is_err(),"{key} truncated {n}");}let mut tail=b;tail.push(0);assert!(m::parse(&tail).is_err());
    }
    let mut bad=bytes(f["list"].as_str().unwrap());bad[5..7].copy_from_slice(&4097u16.to_le_bytes());assert!(m::parse(&bad).is_err());
}
#[test]
fn received_zero_buy_order_is_not_a_zero_inventory_object(){
    let f=fixture();let npc=f["npc"].as_u64().unwrap() as u32;
    assert_eq!(m::talk(npc).unwrap(),bytes(f["talk"].as_str().unwrap()));assert_eq!(m::action(npc,"buy").unwrap(),bytes(f["action"].as_str().unwrap()));
    assert_eq!(m::transaction(npc,false,&[(0,1)]).unwrap(),bytes(f["buy"].as_str().unwrap()));
    assert!(m::transaction(npc,true,&[(0,1)]).is_err());assert!(m::transaction(npc,false,&[(0,0)]).is_err());assert!(m::transaction(npc,false,&[(0,u32::MAX)]).is_err());assert!(m::transaction(npc,false,&[(0,1),(0,1)]).is_err());
    assert!(m::talk(0).is_err());assert!(m::action(npc," buy").is_err());assert!(m::action(npc,"buy\0sell").is_err());
}
#[test]
fn shop_list_is_single_use_and_expires_with_character_world_and_time(){
    use std::time::{Instant,Duration};use lito_modern::modern_lito::native_inventory;
    let f=fixture();let npc=f["npc"].as_u64().unwrap() as u32;let now=Instant::now();
    let owner=m::Owner{connection:2,generation:4,incarnation:1,control_interruptions:0,inventory:native_inventory::Owner{connection:2,selection:1,character:"玩家".into()}};
    let mut s=m::Session::default();s.begin(owner.clone(),npc,now).unwrap();assert!(s.begin(owner.clone(),npc,now).is_err());
    let p=m::parse(&bytes(f["list"].as_str().unwrap())).unwrap();s.accept(p.clone(),now).unwrap();let rev=s.revision;assert!(s.order(rev,0,1,now).is_ok());assert!(s.order(rev+1,0,1,now).is_err());
    assert!(s.order(rev,0,1,now+Duration::from_secs(30)).is_err());s.submitted();assert!(s.order(rev,0,1,now).is_err());
    s.begin(owner.clone(),npc,now).unwrap();s.accept(p,now).unwrap();let mut changed=owner.clone();changed.generation+=1;s.tick(Some(&changed),now);assert!(s.listing.is_none()&&s.owner.is_none());
    s.begin(owner.clone(),npc,now).unwrap();s.tick(Some(&owner),now+Duration::from_secs(8));assert!(s.timed_out&&!s.waiting);
    s.clear();assert!(s.accept(m::parse(&bytes(f["dialog"].as_str().unwrap())).unwrap(),now).is_err());
    for life in [true,false]{s.begin(owner.clone(),npc,now).unwrap();let mut changed=owner.clone();if life{changed.incarnation+=1;}else{changed.control_interruptions+=1;}s.tick(Some(&changed),now);assert!(s.owner.is_none());}
}
#[test]
fn captured_system_refusal_is_not_a_template_message_and_invalid_text_is_atomic(){
    use lito_modern::modern_lito::native_world::World;
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-system-message-current.json")).unwrap();
    let b=bytes(&f["messages"][0]["plain"].as_str().unwrap().replace(' ',""));
    let mut w=World::default();w.map_id=Some(500);w.started=true;assert!(w.accept(&b).unwrap());
    assert_eq!(w.last_system_message,Some((9,"光之寶石不足。".into())));assert_eq!(w.system_message_count,1);assert_eq!(w.message_count,0);
    let before=serde_json::to_value(&w).unwrap();let mut invalid:Vec<_>=(0..b.len()).map(|n|b[..n].to_vec()).collect();
    let mut tail=b.clone();tail.push(0);invalid.push(tail);invalid.extend([vec![15,9,0],vec![15,9,0x81,0],vec![15,9,10,0]]);
    for b in invalid{let mut actual=w.clone();assert!(actual.accept(&b).is_err());assert_eq!(serde_json::to_value(actual).unwrap(),before);}
    let mut max=vec![15,255];max.extend([b'a';255]);max.push(0);w.accept(&max).unwrap();assert_eq!(w.last_system_message.as_ref().unwrap().0,255);
    w.clear_preserving_generation();assert_eq!(w.system_message_count,0);assert!(w.last_system_message.is_none());
}
#[test]
fn formal_inventory_rejections_are_separate_from_text_messages(){
    use lito_modern::modern_lito::native_world::World;
    let mut w=World::default();w.map_id=Some(500);w.started=true;
    for (code,label) in [(263u16,"背包空間不足"),(82,"負重已達上限")]{
        let mut b=vec![0x10];b.extend(code.to_le_bytes());b.push(0);
        assert!(w.accept(&b).unwrap());assert_eq!(w.system_message_count,0);
        assert!(m::inventory_refusal(w.last_message.as_ref()).unwrap().starts_with(label));
    }
    assert!(m::inventory_refusal(Some(&(263,vec!["unexpected".into()]))).is_none());
    assert!(m::inventory_refusal(Some(&(999,vec![]))).is_none());
    w.clear_preserving_generation();assert!(m::inventory_refusal(w.last_message.as_ref()).is_none());
}
