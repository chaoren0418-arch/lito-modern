use lito_modern::modern_lito::{native_city_runtime::{Runtime,UiPage,Action,mode_selection::snapshot::Snapshot,java_room_entry as room},native_multisector_resources::Inputs,native_resources::BytePin};
use anyhow::{ensure,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{fs,path::Path,thread,time::{Instant,Duration}};
fn hex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|v|u8::from_str_radix(std::str::from_utf8(v).unwrap(),16).unwrap()).collect()}
fn inputs(p:&Path,n:&str)->Inputs{let manifest=p.join("logs/continuation-20261003").join(n);let bytes=fs::read(&manifest).unwrap();Inputs{project:p.into(),manifest,manifest_pin:BytePin{bytes:bytes.len()as u64,sha256:Sha256::digest(bytes).into()}}}
fn wait(r:&mut Runtime,page:UiPage){let end=Instant::now()+Duration::from_secs(15);while r.page()!=page{assert!(Instant::now()<end);r.poll();thread::sleep(Duration::from_millis(2));}}
fn main()->Result<()> {
 let arg=std::env::args().nth(1).unwrap();let p=Path::new(&arg);let base=p.join("logs/continuation-20261003");let data:Value=serde_json::from_slice(&fs::read(base.join("r514-test-inputs.json"))?)?;
 let mut runtime=Runtime::start_with_menu_skin(inputs(p,"r486-map-binding.json"),inputs(p,"r506-native-menu-skin-binding.json"))?;wait(&mut runtime,UiPage::Menu);let epoch=runtime.begin_source_connection_candidate()?;
 let before=runtime.presentation_key();let mut native_zero=vec![0;27];native_zero[0]=206;
 assert!(Snapshot::decode(&native_zero).is_ok());let event=epoch.decoded_packet(&native_zero)?;
 assert!(room::decode(&native_zero).is_err(),"AD368 room identity must remain separate from generic structure");
 assert!(runtime.consume_java_room_entry_candidate(&event).is_err());assert_eq!(runtime.presentation_key(),before);
 let mut malformed=0;for n in 0..native_zero.len(){assert!(epoch.decoded_packet(&native_zero[..n]).is_err());malformed+=1;}
 for golden in data["golden_CE"].as_array().unwrap(){let body=hex(golden["body_hex"].as_str().unwrap());let structural=Snapshot::decode(&body)?;let event=epoch.decoded_packet(&body)?;assert_eq!(event.body(),body);let room::Reply::Accepted(receipt)=room::decode(&body)?else{panic!("accepted expected")};assert_eq!(structural.identifier(),receipt.id);assert_eq!(structural.mode_byte(),receipt.mode);}
 assert!(epoch.decoded_packet(&[206,11]).is_ok());assert!(epoch.decoded_packet(&[206,11,0]).is_err());assert_eq!(runtime.presentation_key(),before);
 runtime.disconnect_source_connection_candidate(&epoch)?;let retired=runtime.presentation_key();assert!(runtime.consume_java_room_entry_candidate(&event).is_err());assert_eq!(runtime.presentation_key(),retired);
 runtime.handle(Action::Close)?;wait(&mut runtime,UiPage::Closed);let stats=runtime.player_fixture_snapshot();ensure!(stats.workers_started==0 && stats.workers_joined==0,"final policy check must not create socket fixture");
 fs::write(base.join("r514b-constructor-policy-results.json"),serde_json::to_vec_pretty(&json!({"pass":true,"case":"final library preserves generic Native structural success without room identity; explicit AD368 decoder enforces identity, four saved success shapes remain equivalent, exact short failures and malformed/stale inputs stay guarded","new_final_policy_group":1,"truncations":malformed,"socket_workers":0,"old_PASS_reruns":0,"Java_SQL":0,"GUI":0,"Native_handler_runs":0,"scene_ready":false}))?)?;
 println!("1 final constructor-policy group passed;no socket/Java/old fixture/Native operations");Ok(())
}
