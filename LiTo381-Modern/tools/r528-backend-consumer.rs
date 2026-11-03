use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use zeroize::{Zeroize,Zeroizing};
use std::{fs,io::{Read,Write},path::{Path,PathBuf},time::{Duration,Instant},thread};
use lito_modern::modern_lito::{native_city_runtime::{Runtime,Action,UiPage,login_events::Backend485fProfile,login_bootstrap::{JavaFixtureAdapter,PRODUCER_SHA,CONTRACT_SHA},java_room_entry::{self as room,Intent,Phase},java_room_participants as members,java_room_departure as departure,java_room_tail as tail,java_room_catalog as catalog,java_room_self as self_flow,java_room_settings as settings,java_room_ready as ready_flow,player_entry_commands::Command},native_multisector_resources::Inputs,native_resources::BytePin};
mod countdown {include!("r528-final-countdown-case.rs");}
mod provider {include!("r528-provider.rs");}
const PREPARATION_SCHEMA:&str="modern.r528.provider-supply-request.v1";
const FINAL_PROVIDER_INTERFACE_ACCEPTED:bool=true;
struct ConsumerSpec {fresh_run_id:String,endpoint:String,credentials_file:PathBuf,source_raw_hex:String,guest_alias_raw_hex:String,host_alias_raw_hex:String,channel:u8,room_id:u32,read_cap:usize,write_cap:usize}
struct Provider {consumer:ConsumerSpec,status_file:PathBuf,run_root:PathBuf,helper_pid:u32,port:u16,lease_end_epoch_ms:u64}
struct Credentials {account:String,password:String,room_password:String}
impl Drop for Credentials {fn drop(&mut self){self.account.zeroize();self.password.zeroize();self.room_password.zeroize();}}
fn digest(raw:&[u8])->String{Sha256::digest(raw).iter().map(|v|format!("{v:02X}")).collect()}
fn read_bound(path:&Path,bound:u64)->Result<Vec<u8>>{ensure!(path.is_absolute(),"absolute provider path required");let mut raw=vec![];fs::File::open(path)?.take(bound+1).read_to_end(&mut raw)?;ensure!(raw.len()as u64<=bound,"input exceeds bound");Ok(raw)}
fn decode_hex(s:&str,max:usize)->Result<Vec<u8>>{ensure!(!s.is_empty()&&s.len()%2==0&&s.len()<=max*2,"rawbytes hex bound");s.as_bytes().chunks_exact(2).map(|c|Ok(u8::from_str_radix(std::str::from_utf8(c)?,16)?)).collect()}
fn hex_bytes(raw:&[u8])->String{raw.iter().map(|v|format!("{v:02x}")).collect()}
fn validate_provider(raw:&[u8],accepted_sha:&str)->Result<Provider>{provider::validate(raw,accepted_sha)}
fn synchronized_host_gate(r:&Runtime,p:&Provider)->Result<Option<Value>>{provider::synchronized_host(r,p)}
fn credentials_from_properties(path:&Path)->Result<Credentials>{
 let raw=Zeroizing::new(read_bound(path,4096)?);let text=std::str::from_utf8(&raw).map_err(|_|anyhow::anyhow!("credentials encoding invalid;contents withheld"))?;let mut password=None;
 for line in text.lines(){let line=line.trim();if line.is_empty()||line.starts_with('#')||line.starts_with('!'){continue;}ensure!(line.starts_with("password=")&&password.is_none(),"password-only properties required;contents withheld");let value=&line[9..];ensure!((4..=13).contains(&value.len())&&value.bytes().all(|b|(32..=126).contains(&b))&&!value.contains('\\'),"provider password must be4..13 ASCIIbytes;contents withheld");password=Some(value.to_string());}
 Ok(Credentials{account:"Fx485Guest".into(),password:password.context("providerpassword absent;contents withheld")?,room_password:String::new()})
}
fn profile(p:&Provider,sha:&str)->Result<Backend485fProfile>{Backend485fProfile::parse(&serde_json::to_vec(&json!({"kind":"modern_backend_485F_loopback_v1","label":"BACKEND485F:R528_COUNTDOWN_CANCEL_ACCEPTED_PROVIDER_CANDIDATE","endpoint":p.consumer.endpoint,"producer_sha256":ready_flow::PRODUCER_SHA,"provider_acceptance_sha256":sha,"read_cap":p.consumer.read_cap,"write_cap":p.consumer.write_cap}))?)}
fn inputs(p:&Path,name:&str)->Result<Inputs>{let manifest=p.join("logs/continuation-20261003").join(name);let raw=fs::read(&manifest)?;Ok(Inputs{project:p.into(),manifest,manifest_pin:BytePin{bytes:raw.len()as u64,sha256:Sha256::digest(raw).into()}})}
fn configure(r:&mut Runtime,p:&Provider,sha:&str)->Result<()> {
 r.configure_485f_backend_profile(profile(p,sha)?)?;
 // Historical AD368 adapters are bounded layout candidates; the real485F
 // result must qualify compatibility. Never rename their provenance to485F.
 r.configure_login_producer_fixture(JavaFixtureAdapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_bootstrap_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":CONTRACT_SHA}))?)?)?;
 r.configure_java_room_entry_adapter(room::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_entry_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":room::CONTRACT_SHA}))?)?)?;
 r.configure_java_room_participants_adapter(members::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_participants_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":members::CONTRACT_SHA}))?)?)?;
 r.configure_java_room_departure_adapter(departure::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_departure_v1","producer_sha256":PRODUCER_SHA,"route_sha256":departure::ROUTE_SHA,"native_sha256":departure::NATIVE_SHA,"commit_sha256":departure::COMMIT_SHA}))?)?)?;
 r.configure_java_room_tail_adapter(tail::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_tail_v1","producer_sha256":PRODUCER_SHA,"supplement_sha256":tail::SUPPLEMENT_SHA,"dynamic_sha256":tail::DYNAMIC_SHA,"native_sha256":tail::NATIVE_SHA}))?)?)?;
 r.configure_java_room_catalog_adapter(catalog::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_catalog_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":catalog::CONTRACT_SHA,"lifecycle_sha256":catalog::LIFECYCLE_SHA,"comparison_sha256":catalog::COMPARISON_SHA,"publication_sha256":catalog::PUBLICATION_SHA}))?)?)?;
 r.configure_java_room_self_adapter(self_flow::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_singleton_self_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":self_flow::CONTRACT_SHA}))?)?)?;
 r.configure_java_room_settings_adapter(settings::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_485F_room_settings_v1","producer_sha256":ready_flow::PRODUCER_SHA,"network_sha256":lito_modern::modern_lito::native_city_runtime::java_room_settings_request::SETTINGS_485F_NETWORK_SHA,"layout_sha256":settings::LAYOUT_SHA,"native_sha256":settings::NATIVE_SHA}))?)?)?;
 r.configure_java_room_mode_map_adapter(settings::ModeMapAdapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_485F_room_mode_map_v1","producer_sha256":ready_flow::PRODUCER_SHA,"network_sha256":lito_modern::modern_lito::native_city_runtime::java_room_mode_request::NETWORK_SHA,"mode_step_sha256":lito_modern::modern_lito::native_city_runtime::java_room_mode_request::NATIVE_SHA,"layout_sha256":settings::LAYOUT_SHA,"native_sha256":settings::MODE_MAP_SHA,"scalar_boundary_sha256":settings::SCALAR_BOUNDARY_SHA}))?)?)?;
 r.configure_java_room_ready_adapter(ready_flow::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_485F_participant_ready_v1","producer_sha256":ready_flow::PRODUCER_SHA,"network_sha256":ready_flow::NETWORK_SHA,"native_sha256":ready_flow::NATIVE_SHA}))?)?)?;Ok(())
}
fn run_case(r:&mut Runtime,p:&Provider,credentials:&Credentials,sha:&str,t:&mut countdown::Trace)->Result<()> {
 let alias=decode_hex(&p.consumer.guest_alias_raw_hex,27)?;let host=decode_hex(&p.consumer.host_alias_raw_hex,27)?;t.stage="normal_LOGIN_source_GAME_news_singletonCB_C9_channel_join";
 configure(r,p,sha)?;r.handle_player_entry_command(Command::OpenLogin)?;let c=r.source_login_control().context("login control absent")?;r.handle_player_entry_command(Command::EditLogin{control:&c,account:credentials.account.as_bytes(),password:credentials.password.as_bytes()})?;let c=r.source_login_control().context("login submit absent")?;r.handle_player_entry_command(Command::SubmitLogin(&c))?;
 countdown::wait(r,t,&alias,&host,|r,_|r.source_choice_control(0).is_some())?;let source=decode_hex(&p.consumer.source_raw_hex,63)?;let slot=r.source_bootstrap_snapshot().sources.iter().position(|s|s.name==source).context("provider source raw name absent")?;let c=r.source_choice_control(slot).context("source selection unavailable")?;r.handle_player_entry_command(Command::ChooseSource(&c))?;
 countdown::wait(r,t,&alias,&host,|r,_|r.source_news_control().is_some())?;let c=r.source_news_control().context("news control absent")?;r.handle_player_entry_command(Command::ContinueNews(&c))?;countdown::wait(r,t,&alias,&host,|r,_|r.source_identity().complete())?;
 ensure!(r.source_identity().expected()==Some(1)&&r.source_identity().records().len()==1&&r.source_identity().records()[0].name()==alias,"provider singleton rawGuest roster mismatch");let c=r.source_character_control(0).context("character control absent")?;r.handle_player_entry_command(Command::ChooseCharacter{control:&c,double_click:true})?;
 countdown::wait(r,t,&alias,&host,|r,_|r.source_channel_control(usize::from(p.consumer.channel)).is_some())?;let c=r.source_channel_control(usize::from(p.consumer.channel)).context("channel control absent")?;r.handle_player_entry_command(Command::ChooseChannel(&c))?;countdown::wait(r,t,&alias,&host,|r,_|r.java_character_snapshot().stage==lito_modern::modern_lito::native_city_runtime::java_character_flow::Stage::ChannelReceipts)?;
 let c=r.java_room_entry_control(Intent::Join{room_id:p.consumer.room_id,password:credentials.room_password.as_bytes().to_vec()}).context("join control absent")?;r.handle_java_room_entry_control(&c)?;countdown::wait(r,t,&alias,&host,|r,_|r.java_room_entry_snapshot().phase==Phase::RoomReceipts)?;ensure!(r.java_room_entry_snapshot().room.as_ref().is_some_and(|v|v.id==p.consumer.room_id),"accepted room differs");
 countdown::cancel_case(r,t,&alias,&host,|r|synchronized_host_gate(r,p))?;
 Ok(())
}
fn save_new(path:&Path,v:&Value)->Result<()>{let mut f=fs::OpenOptions::new().write(true).create_new(true).open(path)?;f.write_all(&serde_json::to_vec_pretty(v)?)?;Ok(())}
fn main()->Result<()> {
 let args=std::env::args().skip(1).collect::<Vec<_>>();ensure!(!args.is_empty(),"explicit --check-prepared/--preflight/--connect-once required");
 if args[0]=="--check-prepared"{
  ensure!(args.len()==2,"preparation path required");let v:Value=serde_json::from_slice(&read_bound(Path::new(&args[1]),32768)?)?;
  ensure!(v["schema"]==PREPARATION_SCHEMA&&v["status"]=="COLD_CONSUMER_READY_WAITING_AUTHORIZED_FRESH_PROVIDER"&&v["credentials_contents_in_report"]==false&&v["accepted_cold"]["sha256"]==provider::COLD_ROOT_SHA,"preparation contract invalid");
  println!("R528 CONSUMER_COLD_READY;0socket/0credential/PIDread/0JavaSQLGUI;WAITING_AUTHORIZED_FRESH_PROVIDER");return Ok(());
 }
 ensure!(args.len()==4&&matches!(args[0].as_str(),"--preflight"|"--connect-once"),"absolute root, fresh accepted provider path and SHA required");
 ensure!(FINAL_PROVIDER_INTERFACE_ACCEPTED,"R528 cold provider interface not accepted");
 let root=Path::new(&args[1]);ensure!(root==Path::new("D:/lineage開發/3-lito開發/lito重建版/LiTo381-Modern"),"exact own Modern root required");let raw=read_bound(Path::new(&args[2]),65536)?;let p=validate_provider(&raw,&args[3])?;ensure!(Path::new(&args[2])==p.run_root.join("ready-contract.json"),"exact fresh run READY path required");
 if args[0]=="--preflight"{let credentials=credentials_from_properties(&p.consumer.credentials_file)?;drop(credentials);println!("R528 fresh preflight passed;zero sockets;private credentials zeroized");return Ok(());}
 let output=root.join("logs/continuation-20261003").join(format!("r528-backend-{}",p.consumer.fresh_run_id));fs::create_dir(&output).context("fresh claim exists/output unavailable; first failure stops, no retry")?;
 let credentials=match credentials_from_properties(&p.consumer.credentials_file){Ok(v)=>v,Err(error)=>{
  save_new(&output.join("result.json"),&json!({"schema":"modern.r528.countdown-cancel-result.v1","fresh_run_id":p.consumer.fresh_run_id,"success":false,"stage":"credentials_before_socket","first_failure":error.to_string(),"actual_backend_attempts":0,"credentials_contents_or_hash_recorded":false,"IO_started":0,"IO_joined":0,"provider_stop_required":true,"retry":false}))?;return Err(error);}};
 let mut r=match (||Runtime::start_with_menu_skin(inputs(root,"r486-map-binding.json")?,inputs(root,"r506-native-menu-skin-binding.json")?))(){Ok(r)=>r,Err(error)=>{
  drop(credentials);save_new(&output.join("result.json"),&json!({"schema":"modern.r528.countdown-cancel-result.v1","fresh_run_id":p.consumer.fresh_run_id,"success":false,"stage":"runtime_init_before_socket","first_failure":error.to_string(),"actual_backend_attempts":0,"credentials_contents_or_hash_recorded":false,"provider_stop_required":true,"retry":false}))?;return Err(error);}};
 let mut t=countdown::Trace::new();
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
  countdown::wait(&mut r,&mut t,&[],&[],|r,_|r.page()==UiPage::Menu)?;
  run_case(&mut r,&p,&credentials,&args[3],&mut t)
 }));drop(credentials);
 let case_error=match &result{Ok(Err(e))=>Some(e.to_string()),Err(_)=>Some("panic;contents withheld".into()),_=>None};
 if let Some(error)=&case_error{t.failure(json!({"kind":"first_case_failure","error":error}));}
 let close_result=r.handle(Action::Close).and_then(|_|{let end=Instant::now()+Duration::from_secs(20);while r.page()!=UiPage::Closed{ensure!(Instant::now()<end,"worker close deadline");r.poll();thread::sleep(Duration::from_millis(2));}Ok(())});
 let stats=r.player_fixture_snapshot();let joined=stats.workers_started==stats.workers_joined&&stats.queued_events==0;
 let case_ok=result.as_ref().is_ok_and(|v|v.is_ok());let success=case_ok&&close_result.is_ok()&&joined&&stats.workers_started==2;
 save_new(&output.join("result.json"),&json!({
  "schema":"modern.r528.countdown-cancel-result.v1","fresh_run_id":p.consumer.fresh_run_id,"provider_acceptance_sha256":args[3].to_ascii_uppercase(),
  "success":success,"case_ok":case_ok,"stage":t.stage,"case_error":case_error,"first_failure":t.first_failure,
  "requests":t.requests,"events":t.events,"actual_E103_observed":t.policy.e103,"actual_E100_observed":t.policy.e100,"actual_own_D1one_observed":t.policy.own_d1_one,"actual_own_D1zero_observed":t.policy.own_d1_zero,
  "expected_IO_workers":2,"Ready_complete_write":t.policy.ready_full,"Cancel_complete_write":t.policy.cancel_full,"cancel_queued":t.policy.cancel_queued,
  "E103_observed_us":t.e103_observed_us,"cancel_queued_us":t.cancel_queued_us,"finite_watch_completed":t.watch_completed,"watch_milliseconds":4000,
  "local_room_epoch_label":t.room_epoch,"sameepoch_freshness_proven":false,"queued_tick_drain_proven":false,"WORLD_impossible":false,"server_acceptance_ACK":false,
  "credentials_contents_or_hash_recorded":false,"actual_backend_attempts":if stats.workers_started>0{1}else{0},"synthetic_replay":false,"retry":false,
  "IO_started":stats.workers_started,"IO_joined":stats.workers_joined,"queued_events":stats.queued_events,"discarded_late_events":stats.discarded_late_events,
  "Close_ok":close_result.is_ok(),"Close_error":close_result.as_ref().err().map(|e|e.to_string()),"Modern_starts_or_stops_JavaSQL":false,"GUI":0,"owner_start_WORLD_allowed":false,"provider_stop_required":true
 }))?;
 close_result?;ensure!(joined,"worker/queue not settled; report retained");match result{Ok(v)=>v?,Err(_)=>anyhow::bail!("case panicked; partial/cleanup saved")};
 println!("R528 bounded real countdown/cancellation case ended; server owner must collect wire/GET and stop; no GUI acceptance");Ok(())
}
