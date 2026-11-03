use super::*;
use serde::Deserialize;
use std::{collections::BTreeSet,time::{SystemTime,UNIX_EPOCH}};
const BUNDLE:&str="D:/lito-dev-runtime/20261003-supervision/server/modern-485-provider-frozen-aa561cbbe3d74e1b80f3d8858c1e576f";
const SERVER_RUNS:&str="D:/lito-dev-runtime/20261003-supervision/server";
pub const COLD_ROOT_SHA:&str="6C2C82073BB3DB18FC2121A7C74FAEF4E7A5DF28376874C171F4888BE99ED4EE";
const HANDOFF_SHA:&str="884F2FE61CEDA47E6C0DB4D75B6EE27BD881BF85DD7D4F78513F58B541C81F92";
const LAUNCH_SHA:&str="214AF07DB957F08082ADA739211AD9A63428A03246EB15CF903F1A4269099B60";
const MANIFEST_SHA:&str="511B44E9FD136A6EBDBD7245D8FEAAA4773E09C6C695E590BD551AB7FD2E8634";
const STATUS_POLL_INTERVAL:Duration=Duration::from_millis(200);
pub(super) fn claim_status_poll(last:&std::cell::Cell<Option<Instant>>,now:Instant)->bool{
 if last.get().is_some_and(|previous|now.saturating_duration_since(previous)<STATUS_POLL_INTERVAL){return false;}
 last.set(Some(now));true
}
fn publication_diagnostic(s:&Value)->Result<Value>{
 let d=&s["status_publication"];
 ensure!(d["maximum_move_attempts"]==6&&d["retry_delay_ms"]==20&&["denied_moves","recovered_publications","exhausted_publications"].iter().all(|k|d[*k].as_u64().is_some()),"accepted publication diagnostic schema required");
 let first=d.get("first_error").context("first_error diagnostic absent")?;
 ensure!(first.is_null()||(first["utc"].as_str().is_some()&&first["type"]=="AccessDeniedException"&&first["message"].as_str().is_some()&&first["cause_confirmed"]==false),"publication first_error diagnostic schema invalid");
 // Nonzero counters and a first error neither admit nor reject Host readiness.
 Ok(d.clone())
}
fn accepted_cold_bytes(handoff:&[u8],integrity:&[u8],payload:&[u8])->Result<Value>{
 ensure!(handoff.len()==6067&&digest(handoff)==HANDOFF_SHA,"accepted status IO handoff changed");
 ensure!(integrity.len()==5120&&digest(integrity)==LAUNCH_SHA,"accepted status IO launch integrity changed");
 ensure!(payload.len()==24434&&digest(payload)==MANIFEST_SHA,"accepted status IO payload manifest changed");
 let h:Value=serde_json::from_slice(handoff)?;
 ensure!(h["bundle_root"].as_str().map(Path::new)==Some(Path::new(BUNDLE))&&h["candidate"]["sha256"]==ready_flow::PRODUCER_SHA,"cold status IO candidate differs");
 let m:Value=serde_json::from_slice(integrity)?;let all:Value=serde_json::from_slice(payload)?;
 let files=m["files"].as_array().context("launch pins absent")?;
 let payload_files=all["files"].as_array().context("payload pins absent")?;
 ensure!(files.len()==26&&payload_files.len()==122,"exact26 launch/122 payload pins required");
 ensure!(files.iter().all(|f|payload_files.iter().filter(|p|*p==f).count()==1),"launch pins must match payload manifest exactly");
 Ok(m)
}
const INITIAL:&str="WAITING_GUEST_JOIN_AND_BASELINE_D1_0";
const ARMED:&str="HOST_READY_BOTH_D1_TWO_MEMBER_WAITING_WITNESSED";
pub(super) fn pre_ready_stage(sync:&Value)->Result<bool>{
 sync_common(sync)?;let stage=sync["stage"].as_str().context("Host sync stage absent")?;
 ensure!(matches!(stage,INITIAL|"GUEST_BASELINE_D1_0_BOTH_WRITTEN_WAITING"|"HOST_PARTICIPANT_STATE1_INTENT_SENT"|ARMED),"first unexpected/failed sync stage before GuestReady");
 Ok(stage==ARMED)
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct Pin {path:PathBuf,bytes:u64,sha256:String}
#[derive(Deserialize)]struct Endpoint {host:String,port:u16}
#[derive(Deserialize)]struct Source {id:u32,name:String}
#[derive(Deserialize)]struct Role {role:String,account:String,raw_selection_alias:String,physical_id:u32}
#[derive(Deserialize)]struct ReadyContract {schema:String,state:String,run_identity:String,run_root:PathBuf,bundle_root:PathBuf,candidate:Pin,ownership_file:PathBuf,status_file:PathBuf,credentials_file:PathBuf,login_endpoint:Endpoint,game_endpoint:Endpoint,source:Source,roles:Vec<Role>,support_host_enabled:bool,host_ready_after_guest_notready:bool,host_sync_profile:Value,room:Value,lease_end_epoch_ms:u64,server_lifecycle_owner:bool,rust_acceptance:bool}
pub(super) fn now_ms()->Result<u64>{Ok(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?)}
fn verify(p:&Pin)->Result<Vec<u8>>{ensure!(p.bytes<=16_000_000,"provider file bound");let raw=read_bound(&p.path,p.bytes)?;ensure!(raw.len()as u64==p.bytes&&digest(&raw)==p.sha256.to_ascii_uppercase(),"provider pin mismatch");Ok(raw)}
fn string_set(v:&Value)->Result<BTreeSet<&str>>{let a=v.as_array().context("required epoch/member array absent")?;let s=a.iter().map(|v|v.as_str().context("epoch/member must be string")).collect::<Result<BTreeSet<_>>>()?;ensure!(a.len()==s.len(),"duplicate epoch/member");Ok(s)}
pub(super) fn sync_common(v:&Value)->Result<()>{ensure!(v["enabled"]==true&&v["requires_normal_Guest_baseline_NotReady0"]==true&&v["Host_owner_opcode_only_sent"]==false&&v["failure_Host_state0_attempted"]==false,"explicit normal baseline/Host synchronization flags required");Ok(())}
fn sync_initial(v:&Value)->Result<()>{sync_common(v)?;ensure!(v["stage"]==INITIAL&&v["Host_participant_state1_sent"]==false,"Host must remain initially notready, never soloReady");for key in ["baseline_D1_0_write_epochs","Host_D1_1_write_epochs","E103_write_epochs","Guest_notready_D1_write_epochs","E100_write_epochs"]{ensure!(string_set(&v[key])?.is_empty(),"initial synchronization already consumed");}Ok(())}
fn static_ready(raw:&[u8],accepted_sha:&str)->Result<ReadyContract>{
 ensure!(accepted_sha.len()==64&&accepted_sha.bytes().all(|b|b.is_ascii_hexdigit())&&digest(raw)==accepted_sha.to_ascii_uppercase(),"trusted freshREADY digest mismatch");
 ensure!(accepted_sha.to_ascii_uppercase()!="667B69B3FF198085D1B1B98AFDF4AC1F6A99FEE0D974D1B54397F54D2890F635","retired R527 READY prohibited");
 ensure!(accepted_sha.to_ascii_uppercase()!="2B45D78545ADF12A0CF2FAD02FB1ABF1E9899E3F2AE5B40A8A9F68BBF900D5F4","retired R528 READY prohibited");
 let c:ReadyContract=serde_json::from_slice(raw)?;
 ensure!(c.schema=="server.modern485.provider.run.v1"&&c.state=="READY"&&c.support_host_enabled&&c.host_ready_after_guest_notready&&c.server_lifecycle_owner&&!c.rust_acceptance,"fresh countdown-mode provider contract required; READY is alive only");
 ensure!(c.run_identity.starts_with("nw-")&&c.run_identity.len()==35&&c.run_identity[3..].bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))&&!matches!(c.run_identity.as_str(),"nw-97cd2af6b99842238b90bcef2ba99803"|"nw-6fdb7420a2bc4b8492dd7cb90ff270ed"|"nw-975e97ed0f74480c9c92edd7d6b7d51e"|"nw-c9afae2f2cd14da08208599b1bf066fb")&&c.run_root.parent()==Some(Path::new(SERVER_RUNS))&&c.run_root.file_name().and_then(|s|s.to_str())==Some(c.run_identity.as_str()),"new exact owned run required; retired runs refused");
 ensure!(c.bundle_root==Path::new(BUNDLE)&&c.candidate.path==Path::new(BUNDLE).join("candidate/Ready485.jar")&&c.candidate.bytes==8906219&&c.candidate.sha256==ready_flow::PRODUCER_SHA,"accepted new frozen bundle/candidate required");
 ensure!(c.credentials_file==c.run_root.join("game-player.properties")&&c.status_file==c.run_root.join("status.json")&&c.ownership_file==c.run_root.join("owned-process-live.json"),"provider files escape owned run");
 ensure!(c.login_endpoint.host=="127.0.0.1"&&c.game_endpoint.host=="127.0.0.2"&&c.login_endpoint.port!=0&&c.login_endpoint.port==c.game_endpoint.port&&c.source.id==72&&c.source.name=="Fx485Moon","endpoint/source mismatch");
 ensure!(c.roles.len()==2,"exact Host and Guest roles required");
 for (role,account,alias,id) in [("Host","Fx485Host","Fx485Host(Fx485Moon)",41001),("Guest","Fx485Guest","Fx485Guest(Fx485Moon)",41002)]{ensure!(c.roles.iter().filter(|r|r.role==role).count()==1&&c.roles.iter().find(|r|r.role==role).is_some_and(|r|r.account==account&&r.raw_selection_alias==alias&&r.physical_id==id),"role/source expectations differ; real CB remains authority");}
 sync_initial(&c.host_sync_profile)?;Ok(c)
}
fn template(g:&Value,room_id:u32)->Result<()>{ensure!(room_id!=0&&g["room_id"]==room_id&&g["room_exists"]==true&&g["settings"]["name"]=="Fx485Room"&&g["settings"]["password"]==""&&g["settings"]["capacity"]==4&&g["settings"]["attributes"]==40&&g["settings"]["mode"]==1&&g["settings"]["options"]==json!([2,0,1,31])&&g["settings"]["selections"]==json!([72])&&g["selected_map"]==564,"accepted room template differs");Ok(())}
fn initial_getter(g:&Value)->Result<u32>{let room_id=u32::try_from(g["room_id"].as_u64().context("room ID absent")?)?;template(g,room_id)?;ensure!(g["match_phase"]=="WAITING"&&g["battle_started"]==false&&g["countdown_value"]==0&&g["countdown_next_nanos"]==0&&g["combatant_count"]==1&&g["hub_owner"]=="Host-GAME"&&string_set(&g["hub_members"])?==BTreeSet::from(["Host-GAME"])&&string_set(&g["hub_ready_members"])?.is_empty()&&g["sessions"]["Host-GAME"]["phase"]=="ROOM"&&g["sessions"]["Host-GAME"]["ready"]==false,"initial Host-only notready WAITING0/0 required; minimum_players is not admission");Ok(room_id)}
pub(super) fn same_live_identity(pid:u32,creation_text:&str)->Result<()> {
 use windows::Win32::{Foundation::{CloseHandle,FILETIME},System::Threading::{OpenProcess,PROCESS_QUERY_LIMITED_INFORMATION,GetProcessTimes,GetExitCodeProcess}};
 ensure!(pid>0&&!matches!(pid,6516|9800|8572|18052|6072|11076|4384),"protected/unqualified PID refused");
 let handle=unsafe{OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,false,pid)}?;
 let checked=(||{let(mut creation,mut exit,mut kernel,mut user)=(FILETIME::default(),FILETIME::default(),FILETIME::default(),FILETIME::default());unsafe{GetProcessTimes(handle,&mut creation,&mut exit,&mut kernel,&mut user)}?;let mut code=0;unsafe{GetExitCodeProcess(handle,&mut code)}?;let ticks=(u64::from(creation.dwHighDateTime)<<32)|u64::from(creation.dwLowDateTime);ensure!(code==259&&ticks.to_string()==creation_text,"owned PID retired or creation changed");Ok(())})();let _=unsafe{CloseHandle(handle)};checked
}
pub(super) fn fresh_status(p:&Provider)->Result<Value>{
 p.status_poll_started.set(Some(Instant::now()));
 let opened=super::status_reader::read_json(&p.status_file,65536,Duration::from_secs(5))?;
 let mut s=opened.json;
 let reader=json!({"method":"same_opened_handle_metadata_and_bytes","modified_epoch_ms":u64::try_from(opened.modified.duration_since(UNIX_EPOCH)?.as_millis())?,"bytes":opened.length,"max_age_ms":5000});
 checked_status_header(&s,p)?;
 s["_modern_status_reader"]=reader;Ok(s)
}
fn checked_status_header(s:&Value,p:&Provider)->Result<()>{
 ensure!(s["schema"]=="server.modern485.provider.status.v1"&&s["state"]=="READY"&&s["run_root"].as_str().map(Path::new)==Some(p.run_root.as_path())&&s["bundle_root"].as_str().map(Path::new)==Some(Path::new(BUNDLE))&&s["candidate_sha256"]==ready_flow::PRODUCER_SHA&&s["helper_pid"]==p.helper_pid&&s["port"]==p.port&&s["login_host"]=="127.0.0.1"&&s["game_host"]=="127.0.0.2"&&s["lease_end_epoch_ms"]==p.lease_end_epoch_ms&&s["support_host_enabled"]==true&&s["background_failure"]==""&&s["unexpected_WORLD_start_result"]==false,"fresh provider header failure/identity mismatch");
 ensure!(p.lease_end_epoch_ms>=now_ms()?+60000,"at least60seconds remaining before last GuestReady");
 publication_diagnostic(s)?;Ok(())
}
pub fn validate(raw:&[u8],accepted_sha:&str)->Result<Provider>{
 let c=static_ready(raw,accepted_sha)?;
 let handoff=read_bound(&Path::new(BUNDLE).join("evidence/Modern-status-io-cold-handoff.json"),6067)?;
 let integrity=read_bound(&Path::new(BUNDLE).join("launch-integrity.json"),5120)?;
 let payload=read_bound(&Path::new(BUNDLE).join("manifest.json"),24434)?;
 let manifest=accepted_cold_bytes(&handoff,&integrity,&payload)?;
 let files=manifest["files"].as_array().context("launch pins absent")?;
 for e in files{let name=Path::new(e["path"].as_str().context("relative launch path absent")?);ensure!(!name.is_absolute()&&name.components().all(|c|matches!(c,std::path::Component::Normal(_))),"launch path escape");verify(&Pin{path:Path::new(BUNDLE).join(name),bytes:e["bytes"].as_u64().context("launch bytes absent")?,sha256:e["sha256"].as_str().context("launch digest absent")?.into()})?;}
 verify(&c.candidate)?;
 ensure!(c.lease_end_epoch_ms>=now_ms()?+180000,"fresh lease needs180seconds remaining");
 let owned:Value=serde_json::from_slice(&read_bound(&c.ownership_file,65536)?)?;
 let launch:Value=serde_json::from_slice(&read_bound(&c.run_root.join("launch.json"),65536)?)?;
 ensure!(owned["run_root"].as_str().map(Path::new)==Some(c.run_root.as_path())&&launch["run_root"].as_str().map(Path::new)==Some(c.run_root.as_path())&&launch["bundle_root"].as_str().map(Path::new)==Some(Path::new(BUNDLE))&&launch["support_host"]==true&&launch["host_ready_after_guest_notready"]==true&&launch["lease_seconds"]==900&&launch["attempts"]==1,"owned launch/mode/900second lease mismatch");
 let datadir=owned["datadir"].as_str().context("datadir absent")?;ensure!(Path::new(datadir).starts_with(&c.run_root)&&Path::new(datadir)!=c.run_root,"owned SQL datadir escape");
 let rows=owned["owned_processes"].as_array().context("owned processes absent")?;ensure!(rows.len()==2,"exact helper/mysql identities required");let mut roles=BTreeSet::new();let mut pids=BTreeSet::new();let mut helper_pid=0;
 // First validate all declared metadata; then query only these two owned PIDs.
 for row in rows {let role=row["role"].as_str().context("role absent")?;ensure!(matches!(role,"helper"|"mysql")&&roles.insert(role),"duplicate/unexpected process role");let pid=u32::try_from(row["pid"].as_u64().context("PID absent")?)?;ensure!(pids.insert(pid),"duplicate PID");let cmd=row["cim"]["CommandLine"].as_str().context("command absent")?;let exe=Path::new(row["cim"]["ExecutablePath"].as_str().context("executable absent")?);
  if role=="helper" {helper_pid=pid;ensure!(exe==Path::new("C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/java.exe")&&cmd.contains("local.lito381.server.Modern485ProviderMain")&&cmd.contains(c.run_root.to_str().unwrap())&&cmd.contains(c.candidate.path.to_str().unwrap()),"helper executable/command/run mismatch");}
  else{ensure!(exe==Path::new("C:/Program Files/MySQL/MySQL Server 5.5/bin/mysqld.exe")&&cmd.replace('\\',"/").to_ascii_lowercase().contains(&datadir.replace('\\',"/").to_ascii_lowercase()),"mysql executable/datadir mismatch");}
  row["creation_FILETIME_100ns"].as_str().context("creation FILETIME absent")?.parse::<u64>()?;
 }
 let initial_room=initial_getter(&c.room)?;
 let p=Provider{consumer:ConsumerSpec{fresh_run_id:c.run_identity,endpoint:format!("127.0.0.1:{}",c.login_endpoint.port),credentials_file:c.credentials_file,source_raw_hex:hex_bytes(b"Fx485Moon"),guest_alias_raw_hex:hex_bytes(b"Fx485Guest(Fx485Moon)"),host_alias_raw_hex:hex_bytes(b"Fx485Host(Fx485Moon)"),channel:0,room_id:initial_room,read_cap:31,write_cap:157},status_file:c.status_file,run_root:c.run_root,helper_pid,port:c.login_endpoint.port,lease_end_epoch_ms:c.lease_end_epoch_ms,status_poll_started:std::cell::Cell::new(None),identities:rows.iter().map(|row|(u32::try_from(row["pid"].as_u64().unwrap()).unwrap(),row["creation_FILETIME_100ns"].as_str().unwrap().to_owned())).collect()};
 let status=fresh_status(&p)?;sync_initial(&status["host_sync"])?;ensure!(initial_getter(&status["getter"])?==initial_room,"current initial room differs");
 for row in rows{same_live_identity(u32::try_from(row["pid"].as_u64().unwrap())?,row["creation_FILETIME_100ns"].as_str().unwrap())?;}
 profile(&p,accepted_sha)?;Ok(p)
}
pub(super) fn armed_getter<'a>(g:&'a Value,sync:&Value,room_id:u32)->Result<&'a str>{
 template(g,room_id)?;sync_common(sync)?;
 ensure!(sync["stage"]==ARMED&&sync["Host_participant_state1_sent"]==true,"actual Host arming marker required");
 let sessions=g["sessions"].as_object().context("sessions absent")?;
 let guests=sessions.iter().filter(|(_,s)|s["physical_character_id"]==41002&&s["phase"]=="ROOM").collect::<Vec<_>>();ensure!(guests.len()==1,"unique actual Guest ROOM observation required");let (guest,gs)=guests[0];let hs=&g["sessions"]["Host-GAME"];let peers=BTreeSet::from(["Host-GAME",guest.as_str()]);
 ensure!(g["combatant_count"]==2&&g["hub_owner"]=="Host-GAME"&&string_set(&g["hub_members"])?==peers&&string_set(&g["connected_GAME_session_tags"])?==peers&&string_set(&g["hub_ready_members"])?==BTreeSet::from(["Host-GAME"])&&g["match_phase"]=="WAITING"&&g["battle_started"]==false&&g["countdown_value"]==0&&g["countdown_next_nanos"]==0,"two actual peers/Host-only readiness/WAITING0/0 required");
 for (row,is_host,physical,character) in [(hs,true,41001,"Fx485HostChar"),(gs,false,41002,"Fx485GuestChar")] {ensure!(row["phase"]=="ROOM"&&row["currentRoomId"]==room_id&&row["ready"]==is_host&&row["observer"]==false&&row["native_source_id"]==72&&row["physical_character_id"]==physical&&row["selectedCharacter"]==character&&row["selectedClassicServerName"]=="Fx485Moon"&&row["currentChannelId"]==0&&row["hubChannelId"]==0&&row["litoCharacterSelectionConsumed"]==true&&row["authenticatedGameAccounts_owns_session"]==true&&row["selectedCharacters_owns_session"]==true,"normal actual Host1/Guest0 source/selection/room observation required");}
 for key in ["baseline_D1_0_write_epochs","Host_D1_1_write_epochs"]{ensure!(string_set(&sync[key])?==peers,"both completed-write witness epochs required");}
 for key in ["E103_write_epochs","Guest_notready_D1_write_epochs","E100_write_epochs"]{ensure!(string_set(&sync[key])?.is_empty(),"countdown/cancellation already consumed before GuestReady");}
 Ok(guest.as_str())
}
#[cfg(test)]mod r530_entry_guards {
 use super::*;
 #[test]fn digest_and_retired_supply_refuse_before_io(){
  assert!(static_ready(b"{}","bad").is_err());
  assert!(static_ready(b"{}","2B45D78545ADF12A0CF2FAD02FB1ABF1E9899E3F2AE5B40A8A9F68BBF900D5F4").is_err());
  assert!(static_ready(b"{}",&digest(b"{}")).is_err());
 }
 #[test]fn health_poll_budget_is_shared_not_callback_wait(){let last=Cell::new(None);let now=Instant::now();assert!(claim_status_poll(&last,now));assert!(!claim_status_poll(&last,now+Duration::from_millis(199)));assert!(claim_status_poll(&last,now+Duration::from_millis(200)));}
}
#[cfg(test)]mod r530_new_health_stage {
 use super::*;
 #[test]fn unknown_or_missing_host_stage_stops_before_ready(){
  let mut sync=json!({"enabled":true,"requires_normal_Guest_baseline_NotReady0":true,"Host_owner_opcode_only_sent":false,"failure_Host_state0_attempted":false,"stage":INITIAL});
  assert!(!pre_ready_stage(&sync).unwrap());
  for stage in ["GUEST_BASELINE_D1_0_BOTH_WRITTEN_WAITING","HOST_PARTICIPANT_STATE1_INTENT_SENT"]{sync["stage"]=json!(stage);assert!(!pre_ready_stage(&sync).unwrap());}
  sync["stage"]=json!(ARMED);assert!(pre_ready_stage(&sync).unwrap());
  sync["stage"]=json!("UNEXPECTED");assert!(pre_ready_stage(&sync).is_err());
  sync.as_object_mut().unwrap().remove("stage");assert!(pre_ready_stage(&sync).is_err());
 }
}
