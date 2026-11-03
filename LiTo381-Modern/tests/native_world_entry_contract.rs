//! Raw, opt-in contract discovery. This does not qualify LoginController WORLD support.
use anyhow::{ensure,Context,Result};
use lito_modern::modern_lito::{native_connection::{Controller,Event,Limits,Phase},native_login,native_game::{self,Directory},native_lobby,native_room,native_world::World};
use serde_json::{json,Value};
use std::{net::SocketAddr,path::Path,thread,time::{Duration,Instant}};
use zeroize::Zeroizing;
fn send(c:&mut Controller,b:Vec<u8>)->Result<()>{c.send(c.snapshot().epoch,Zeroizing::new(b))}
fn pump(c:&mut Controller)->Result<Vec<Vec<u8>>>{let rows=c.poll().into_iter().filter_map(|e|if let Event::Packet{body,..}=e{Some(body)}else{None}).collect();ensure!(c.snapshot().phase!=Phase::Failed,"transport {:?}",c.snapshot().error);Ok(rows)}
fn close(c:&mut Controller)->Result<()>{c.cancel();let end=Instant::now()+Duration::from_secs(3);while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(5));}ensure!(c.is_quiescent(),"worker leak");Ok(())}
fn player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()>{
 let mut c=Controller::default();let mut trace=vec![];let started=Instant::now();
 let result=(||->Result<()>{
 c.start(peer,Limits::default())?;let mut authenticated=false;let mut selected=false;let mut game=None;
 while game.is_none(){
  ensure!(started.elapsed()<Duration::from_secs(15),"LOGIN timeout");
  for b in pump(&mut c)?{match native_login::parse(&b)?{
   native_login::Packet::Sources(_) if !selected=>{c.send(c.snapshot().epoch,native_login::source_request("測試乙")?)?;selected=true;},
   native_login::Packet::Redirect{endpoint,..}=>{ensure!(endpoint.ip().is_loopback(),"nonlocal redirect");game=Some(endpoint);},_=>{}}}
  if !authenticated&&c.snapshot().negotiated.is_some(){c.send(c.snapshot().epoch,native_login::login_request(account,"TestOnlyPassword",std::net::Ipv4Addr::LOCALHOST)?)?;authenticated=true;}
  thread::sleep(Duration::from_millis(5));
 }
 close(&mut c)?;c.start(game.unwrap(),Limits::default())?;authenticated=false;
 let mut directory=Directory::default();let mut selected=false;let mut joined=false;let mut ready=false;let mut forced=false;let mut members=std::collections::BTreeMap::new();
 let mut own_name=String::new();let mut w=World::default();let mut spell_sent=false;let mut portal_seen=false;let mut world_since=None;
 loop{
  ensure!(started.elapsed()<Duration::from_secs(70),"WORLD contract timeout: ready={} spell={} portal={} map={:?} packets={}",w.ready(),spell_sent,portal_seen,w.map_id,trace.len());
  for b in pump(&mut c)?{
   trace.push(json!({"ms":started.elapsed().as_millis(),"body":b.iter().map(|v|format!("{v:02x}")).collect::<String>()}));
   match native_game::parse(&b)?{
    native_game::Packet::Notice(id)=>{let ack=native_game::Notice{message_id:Some(id),text:None}.acknowledgement().context("unsupported notice")?;send(&mut c,vec![ack])?;},
    native_game::Packet::Ready=>send(&mut c,vec![0xe2])?,
    native_game::Packet::Count(n)=>directory.start(n)?,
    native_game::Packet::Row(row)=>{directory.push(row)?;if directory.complete()&&!selected{own_name=directory.view()?[0].name.clone();c.send(c.snapshot().epoch,directory.select(0)?)?;selected=true;}},
    native_game::Packet::Selected(0)=>send(&mut c,native_lobby::enter(0)?)?,_=>{}}
   match native_lobby::parse(&b)?{native_lobby::Packet::Enter{result:0,..} if !joined=>send(&mut c,vec![0xce])?,native_lobby::Packet::Room(room) if !joined=>{c.send(c.snapshot().epoch,native_room::join(room.id,"")?)?;joined=true;},_=>{}}
   match native_room::parse(&b)?{
    native_room::Packet::Joined(room)=>{for m in room.members{members.insert(m.name,m.force);}},
    native_room::Packet::Member{member,..}=>{members.insert(member.name,member.force);},
    native_room::Packet::Force{name,force}=>{members.insert(name,force);},_=>{}}
   if joined&&!ready&&members.len()==2{
    let desired=if role=="a"{0}else{1};
    if members.get(&own_name)!=Some(&desired){if !forced{c.send(c.snapshot().epoch,native_room::force_request(desired)?)?;forced=true;}}
    else if members.values().any(|f|*f!=desired){send(&mut c,vec![0xd2,1])?;ready=true;}
   }
   if b[0]==0x63{send(&mut c,vec![0x5a])?;}
   if b[0]==0x5a{ensure!(spell_sent&&!portal_seen,"unexpected portal");portal_seen=true;send(&mut c,vec![0x08])?;}
   // The production world decoder is exercised independently of login's room guard.
   w.accept(&b)?;if w.portal.is_some(){w.finish_portal_if_ready()?;}
  }
  if !authenticated&&c.snapshot().negotiated.is_some(){c.send(c.snapshot().epoch,native_login::login_request(account,"TestOnlyPassword",std::net::Ipv4Addr::LOCALHOST)?)?;authenticated=true;}
  if w.ready()&&w.integrity_challenge.is_some()&&!spell_sent{send(&mut c,vec![0x14,16,2])?;spell_sent=true;}
  if portal_seen&&w.ready()&&w.map_id==Some(4){if world_since.is_none(){world_since=Some(Instant::now());}if world_since.unwrap().elapsed()>Duration::from_secs(1){break;}}
  thread::sleep(Duration::from_millis(5));
 }
 ensure!(w.own_actor().is_some()&&w.hp.is_some()&&w.completed_portals==1,"incomplete WORLD rebuild");
 std::fs::write(root.join(format!("{role}-world-entry.json")),serde_json::to_vec_pretty(&json!({"role":role,"raw_contract_probe":true,"production_login_qualified":false,"world":w,"trace":trace}))?)?;
 Ok(())})();
 if let Err(e)=&result{std::fs::write(root.join(format!("{role}-world-entry-error.json")),serde_json::to_vec_pretty(&json!({"error":format!("{e:#}"),"trace":trace}))?)?;}
 close(&mut c)?;result
}
#[test]
#[ignore="requires owned C02 formal LOGIN/GAME world-tree contract harness"]
fn actual_world_tree_transition_contract(){
 let root=std::path::PathBuf::from(std::env::var("LITO_WORLD_ENTRY_ROOT").unwrap());
 assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));
 let r:Value=serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();
 assert_eq!(r["jar_sha256"],std::env::var("LITO_WORLD_ENTRY_REFERENCE_SHA").unwrap());
 let peer=format!("127.0.0.1:{}",r["port"].as_u64().unwrap()).parse().unwrap();
 thread::scope(|s|{let a=s.spawn(||player(&root,peer,"a","PeerFixture"));let b=s.spawn(||player(&root,peer,"b","PeerFixture2"));let a=a.join().unwrap();let b=b.join().unwrap();assert!(a.is_ok()&&b.is_ok(),"a={a:?}; b={b:?}");});
}
