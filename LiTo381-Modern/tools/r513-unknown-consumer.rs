use lito_modern::modern_lito::{native_city_runtime::{Runtime,UiPage,Action,login_events::FixtureProfile,login_bootstrap::{JavaFixtureAdapter,PRODUCER_SHA,CONTRACT_SHA},status_family::Projection,player_entry_commands::Command,status_receipt},native_multisector_resources::Inputs,native_resources::BytePin};
use anyhow::{ensure,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{fs,path::Path,net::{TcpListener,TcpStream,SocketAddr},io::{Read,Write},thread,time::{Instant,Duration},sync::mpsc};
fn hex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|v|u8::from_str_radix(std::str::from_utf8(v).unwrap(),16).unwrap()).collect()}
fn pin_input(p:&Path,n:&str)->Inputs {let manifest=p.join("logs/continuation-20261003").join(n);let b=fs::read(&manifest).unwrap();Inputs{project:p.into(),manifest,manifest_pin:BytePin{bytes:b.len() as u64,sha256:Sha256::digest(b).into()}}}
fn ready(p:&Path)->Result<Runtime>{let mut r=Runtime::start_with_menu_skin(pin_input(p,"r486-map-binding.json"),pin_input(p,"r506-native-menu-skin-binding.json"))?;until(&mut r,|r|r.page()==UiPage::Menu);Ok(r)}
fn until(r:&mut Runtime,f:impl Fn(&Runtime)->bool){let end=Instant::now()+Duration::from_secs(15);while !f(r){assert!(Instant::now()<end,"new branch wait expired: {} {:?} {:?}",r.status(),r.java_return_snapshot(),r.last_source_rejection());r.poll();thread::sleep(Duration::from_millis(2));}}
fn close(r:&mut Runtime)->Result<()> {r.handle(Action::Close)?;until(r,|r|r.page()==UiPage::Closed);ensure!(r.player_fixture_snapshot().workers_started==r.player_fixture_snapshot().workers_joined,"worker leak");Ok(())}
fn main()->Result<()> {
 let path=std::env::args().nth(1).unwrap();let p=Path::new(&path);let mut r=ready(p)?;let epoch=r.begin_source_connection_candidate()?;
 let old=epoch.decoded_packet(&[80,1,2,3,4,5,6,7])?;r.feed_source_stream_candidate(&epoch,&[7,0,1,0x78,0x56,0x34,0x12])?;
 assert!(epoch.decoded_packet(&[0xfe]).is_err());let batch=r.feed_source_stream_candidate(&epoch,&[3,0,0xfe^0x78])?;
 assert_eq!(batch.terminal.unwrap().terminal,lito_modern::modern_lito::native_city_runtime::source_stream::Terminal::UnsupportedBody);
 assert_eq!(r.last_source_rejection().unwrap().opcode,Some(0xfe));assert!(r.source_connection_epoch().is_none());assert_eq!(r.java_character_snapshot().status_fields,Projection::default());assert!(r.consume_java_status_family_candidate(&old).is_err());
 close(&mut r)?;fs::write(p.join("logs/continuation-20261003/r513-unknown-consumer-results.json"),serde_json::to_vec_pretty(&json!({"case":"synthetic FE unknown stream constructor/retirement preserved after four new opcode additions; cached valid-family old event rejected","pass":true,"unknown":"synthetic negative, not claimed actual producer opcode","actual_next_boundary":"CE in separate saved-tail consumer","new_Java_SQL":0,"new_socket_workers":0,"GUI":0,"historical_PASS_reruns":0}))?)?;println!("1 new unknown-stream guard group passed; synthetic FE retires epoch");Ok(())
}
