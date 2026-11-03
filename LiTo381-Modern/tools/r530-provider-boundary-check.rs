use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};use sha2::{Digest,Sha256};
use std::{cell::Cell,fs,io::Read,path::{Path,PathBuf},time::{Duration,Instant}};
use lito_modern::modern_lito::native_city_runtime::{Runtime,login_events::Backend485fProfile,java_room_ready as ready_flow};
struct ConsumerSpec {fresh_run_id:String,endpoint:String,credentials_file:PathBuf,source_raw_hex:String,guest_alias_raw_hex:String,host_alias_raw_hex:String,channel:u8,room_id:u32,read_cap:usize,write_cap:usize}
struct Provider {consumer:ConsumerSpec,status_file:PathBuf,run_root:PathBuf,helper_pid:u32,port:u16,lease_end_epoch_ms:u64,status_poll_started:Cell<Option<Instant>>,identities:Vec<(u32,String)>}
fn digest(raw:&[u8])->String{Sha256::digest(raw).iter().map(|v|format!("{v:02X}")).collect()}
fn read_bound(path:&Path,bound:u64)->Result<Vec<u8>>{ensure!(path.is_absolute(),"absolute provider path required");let mut raw=vec![];fs::File::open(path)?.take(bound+1).read_to_end(&mut raw)?;ensure!(raw.len()as u64<=bound,"input exceeds bound");Ok(raw)}
fn hex_bytes(raw:&[u8])->String{raw.iter().map(|v|format!("{v:02x}")).collect()}
fn profile(p:&Provider,sha:&str)->Result<Backend485fProfile>{Backend485fProfile::parse(&serde_json::to_vec(&json!({"kind":"modern_backend_485F_loopback_v1","label":"BACKEND485F:R530_SUPERVISOR_PINNED_PLAYER_CANDIDATE","endpoint":p.consumer.endpoint,"producer_sha256":ready_flow::PRODUCER_SHA,"provider_acceptance_sha256":sha,"read_cap":p.consumer.read_cap,"write_cap":p.consumer.write_cap}))?)}
mod provider {include!("../src/modern_lito/native_backend_485f_provider.rs");
 #[cfg(test)]mod r530_new_retirement_boundary {
 use super::*;
 fn sync()->Value{json!({"enabled":true,"stage":INITIAL,"requires_normal_Guest_baseline_NotReady0":true,"Host_participant_state1_sent":false,"Host_owner_opcode_only_sent":false,"failure_Host_state0_attempted":false,"baseline_D1_0_write_epochs":[],"Host_D1_1_write_epochs":[],"E103_write_epochs":[],"Guest_notready_D1_write_epochs":[],"E100_write_epochs":[]})}
 fn getter()->Value{let mut g=json!({"room_id":1,"room_exists":true,"settings":{"name":"Fx485Room","password":"","capacity":4,"attributes":40,"mode":1,"options":[2,0,1,31],"selections":[72]},"selected_map":564,"hub_owner":"Host-GAME","hub_members":["Host-GAME","Guest"],"connected_GAME_session_tags":["Host-GAME","Guest"],"hub_ready_members":["Host-GAME"],"combatant_count":2,"match_phase":"WAITING","battle_started":false,"countdown_value":0,"countdown_next_nanos":0,"sessions":{}});for (tag,host,id,ch) in [("Host-GAME",true,41001,"Fx485HostChar"),("Guest",false,41002,"Fx485GuestChar")] {g["sessions"][tag]=json!({"phase":"ROOM","currentRoomId":1,"ready":host,"observer":false,"native_source_id":72,"physical_character_id":id,"selectedCharacter":ch,"selectedClassicServerName":"Fx485Moon","currentChannelId":0,"hubChannelId":0,"litoCharacterSelectionConsumed":true,"authenticatedGameAccounts_owns_session":true,"selectedCharacters_owns_session":true});}g}
 #[test]fn new_ui_boundary_rejects_retired_c9af_and_untrusted_metadata_without_io(){
 let root=Path::new(SERVER_RUNS).join(format!("nw-{}","a".repeat(32)));let mut v=json!({"schema":"server.modern485.provider.run.v1","state":"READY","run_identity":root.file_name().unwrap().to_str().unwrap(),"run_root":root,"bundle_root":BUNDLE,"candidate":{"path":Path::new(BUNDLE).join("candidate/Ready485.jar"),"bytes":8906219,"sha256":ready_flow::PRODUCER_SHA},"ownership_file":root.join("owned-process-live.json"),"status_file":root.join("status.json"),"credentials_file":root.join("game-player.properties"),"login_endpoint":{"host":"127.0.0.1","port":12345},"game_endpoint":{"host":"127.0.0.2","port":12345},"source":{"id":72,"name":"Fx485Moon"},"roles":[{"role":"Host","account":"Fx485Host","raw_selection_alias":"Fx485Host(Fx485Moon)","physical_id":41001},{"role":"Guest","account":"Fx485Guest","raw_selection_alias":"Fx485Guest(Fx485Moon)","physical_id":41002}],"support_host_enabled":true,"host_ready_after_guest_notready":true,"host_sync_profile":sync(),"room":getter(),"lease_end_epoch_ms":1,"server_lifecycle_owner":true,"rust_acceptance":false});
 let raw=serde_json::to_vec(&v).unwrap();assert!(static_ready(&raw,&digest(&raw)).is_ok());
 let retired=Path::new(SERVER_RUNS).join("nw-c9afae2f2cd14da08208599b1bf066fb");v["run_identity"]=json!(retired.file_name().unwrap().to_str().unwrap());v["run_root"]=json!(retired);
 let raw=serde_json::to_vec(&v).unwrap();assert!(static_ready(&raw,&digest(&raw)).err().unwrap().to_string().contains("retired"));
 assert!(static_ready(&raw,&"0".repeat(64)).err().unwrap().to_string().contains("digest"));
 }
}
}
mod status_reader{include!("../src/modern_lito/native_backend_485f_status_reader.rs");}
