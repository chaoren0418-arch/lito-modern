from r491_common import *
source=P/'target/r527-fixed-backend-consumer-revision2/r527-backend-consumer.rs'
assert pin(source)['sha256']=='83EA350A33002A1948568DE2F618859E7ADDBAB789D061374569B843F6576BBE'
s=source.read_text(encoding='utf-8-sig').replace('use serde::Deserialize;\n','')
s=s.replace('const PREPARATION_SCHEMA:&str="modern.r527.provider-supply-request.v1";', 'mod countdown {include!("r528-countdown-case.rs");}\nconst PREPARATION_SCHEMA:&str="modern.r528.provider-supply-request.v1";\nconst FINAL_PROVIDER_INTERFACE_ACCEPTED:bool=false;')
start=s.index('#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct Pin')
end=s.index('struct Credentials ')
s=s[:start]+'''struct ConsumerSpec {fresh_run_id:String,endpoint:String,credentials_file:PathBuf,source_raw_hex:String,guest_alias_raw_hex:String,host_alias_raw_hex:String,channel:u8,room_id:u32,read_cap:usize,write_cap:usize}
struct Provider {consumer:ConsumerSpec}
'''+s[end:]
start=s.index('fn verify(');end=s.index('fn decode_hex(');s=s[:start]+s[end:]
start=s.index('fn now()');end=s.index('fn credentials_from_properties(')
s=s[:start]+'''// This draft cannot read a provider/credential/PID or connect. The final
// validator must be replaced only after supervisor accepts the new cold schema,
// exact bundle pins and normal Host baseline -> state1 synchronization contract.
fn validate_provider(_raw:&[u8],_accepted_sha:&str)->Result<Provider>{
 anyhow::bail!("R528 accepted new cold schema/Host sync contract pending; draft refuses all preflight/connect")
}
fn synchronized_host_gate(_r:&Runtime,_p:&Provider)->Result<Value>{
 anyhow::bail!("R528 accepted two-peer WAITING Host1/Guest0 baseline gate pending; no guessed getter schema")
}
'''+s[end:]
s=s.replace('R527_ACCEPTED_PROVIDER_CANDIDATE','R528_COUNTDOWN_CANCEL_ACCEPTED_PROVIDER_CANDIDATE')
start=s.index('fn wait(');end=s.index('fn configure(');s=s[:start]+s[end:]
start=s.index('fn run_case(');end=s.index('fn save_new(')
case=s[start:end]
case=case.replace('records:&mut Vec<Value>', 't:&mut countdown::Trace')
case=case.replace(' configure(r,p,sha)?;', ' let alias=decode_hex(&p.consumer.guest_alias_raw_hex,27)?;let host=decode_hex(&p.consumer.host_alias_raw_hex,27)?;t.stage="normal_LOGIN_source_GAME_news_singletonCB_C9_channel_join";\n configure(r,p,sha)?;')
case=case.replace('wait(r,|r|','countdown::wait(r,t,&alias,&host,|r,_|')
case=case.replace(';let alias=decode_hex(&p.consumer.guest_alias_raw_hex,27)?;', ';')
cut=case.index(';let host=decode_hex(&p.consumer.host_alias_raw_hex,27)?;',case.index('let c=r.java_room_entry_control'))
case=case[:cut]+''';
 countdown::cancel_case(r,t,&alias,&host,|r|synchronized_host_gate(r,p))?;
 Ok(())
}
'''
s=s[:start]+case+s[end:]
s=s.replace('use std::{fs,io::{Read,Write},path::{Path,PathBuf},time::{Duration,Instant,SystemTime,UNIX_EPOCH},thread};','use std::{fs,io::{Read,Write},path::{Path,PathBuf},time::{Duration,Instant},thread};')
start=s.index('fn main()')
s=s[:start]+'''fn main()->Result<()> {
 let args=std::env::args().skip(1).collect::<Vec<_>>();ensure!(!args.is_empty(),"explicit --check-prepared/--preflight/--connect-once required");
 if args[0]=="--check-prepared"{
  ensure!(args.len()==2,"preparation path required");let v:Value=serde_json::from_slice(&read_bound(Path::new(&args[1]),32768)?)?;
  ensure!(v["schema"]==PREPARATION_SCHEMA&&v["status"]=="WAITING_ACCEPTED_COLD_SCHEMA_AND_HOST_SYNC"&&v["credentials_contents_in_report"]==false,"preparation contract invalid");
  println!("R528 DRAFT_PREPARED_ONLY;0socket/0credential/PIDread/0JavaSQLGUI;final provider interface pending");return Ok(());
 }
 ensure!(args.len()==4&&matches!(args[0].as_str(),"--preflight"|"--connect-once"),"absolute root, fresh accepted provider path and SHA required");
 ensure!(FINAL_PROVIDER_INTERFACE_ACCEPTED,"R528 cold schema/Host synchronization not accepted; draft stops before reading provider, credentials or PIDs; zero sockets");
 let root=Path::new(&args[1]);ensure!(root.is_absolute(),"absolute Modern root required");let raw=read_bound(Path::new(&args[2]),65536)?;let p=validate_provider(&raw,&args[3])?;
 if args[0]=="--preflight"{let credentials=credentials_from_properties(&p.consumer.credentials_file)?;drop(credentials);println!("R528 fresh preflight passed;zero sockets;private credentials zeroized");return Ok(());}
 let output=root.join("logs/continuation-20261003").join(format!("r528-backend-{}",p.consumer.fresh_run_id));fs::create_dir(&output).context("fresh claim exists/output unavailable; first failure stops, no retry")?;
 let credentials=match credentials_from_properties(&p.consumer.credentials_file){Ok(v)=>v,Err(error)=>{
  save_new(&output.join("result.json"),&json!({"schema":"modern.r528.countdown-cancel-result.v1","fresh_run_id":p.consumer.fresh_run_id,"success":false,"stage":"credentials_before_socket","first_failure":error.to_string(),"actual_backend_attempts":0,"credentials_contents_or_hash_recorded":false,"IO_started":0,"IO_joined":0,"provider_stop_required":true,"retry":false}))?;return Err(error);}};
 let mut r=match Runtime::start_with_menu_skin(inputs(root,"r486-map-binding.json")?,inputs(root,"r506-native-menu-skin-binding.json")?){Ok(r)=>r,Err(error)=>{
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
 let case_ok=result.as_ref().is_ok_and(|v|v.is_ok());let success=case_ok&&close_result.is_ok()&&joined;
 save_new(&output.join("result.json"),&json!({
  "schema":"modern.r528.countdown-cancel-result.v1","fresh_run_id":p.consumer.fresh_run_id,"provider_acceptance_sha256":args[3].to_ascii_uppercase(),
  "success":success,"case_ok":case_ok,"stage":t.stage,"case_error":case_error,"first_failure":t.first_failure,
  "requests":t.requests,"events":t.events,"actual_E103_observed":t.policy.e103,"actual_E100_observed":t.policy.e100,"actual_own_D1zero_observed":t.policy.own_d1_zero,
  "Ready_complete_write":t.policy.ready_full,"Cancel_complete_write":t.policy.cancel_full,"cancel_queued":t.policy.cancel_queued,
  "E103_observed_us":t.e103_observed_us,"cancel_queued_us":t.cancel_queued_us,"finite_watch_completed":t.watch_completed,"watch_milliseconds":4000,
  "local_room_epoch_label":t.room_epoch,"sameepoch_freshness_proven":false,"queued_tick_drain_proven":false,"WORLD_impossible":false,"server_acceptance_ACK":false,
  "credentials_contents_or_hash_recorded":false,"actual_backend_attempts":if stats.workers_started>0{1}else{0},"synthetic_replay":false,"retry":false,
  "IO_started":stats.workers_started,"IO_joined":stats.workers_joined,"queued_events":stats.queued_events,"discarded_late_events":stats.discarded_late_events,
  "Close_ok":close_result.is_ok(),"Close_error":close_result.as_ref().err().map(|e|e.to_string()),"Modern_starts_or_stops_JavaSQL":false,"GUI":0,"owner_start_WORLD_allowed":false,"provider_stop_required":true
 }))?;
 close_result?;ensure!(joined,"worker/queue not settled; report retained");match result{Ok(v)=>v?,Err(_)=>anyhow::bail!("case panicked; partial/cleanup saved")};
 println!("R528 bounded real countdown/cancellation case ended; server owner must collect wire/GET and stop; no GUI acceptance");Ok(())
}
'''
(P/'tools/r528-backend-consumer-draft.rs').write_text(s,encoding='utf-8')
save(L/'r528-provider-supply-request.json',dict(
 schema='modern.r528.provider-supply-request.v1',status='WAITING_ACCEPTED_COLD_SCHEMA_AND_HOST_SYNC',credentials_contents_in_report=False,
 accepted_new_cold_bundle=None,expected_ready_schema=None,final_provider_interface_accepted=False,
 flow=['Normal LOGIN/source/GAME/news/actualsingletonCB/currentrawC9/acceptedC8/channel0/join actual accepted room/privatequalifiedself.',
 'Initial Host notready. Never Host-onlyReady: mode1 nativeRosterReady may true before2players; SQL minimum_players is insufficient admission guard.',
 'Guest sends typed NotReady0 baseline once; full4-byte write and actual ownD1zero. Supplier waits bothpeer baselineD1/GET and two actual participants, then sends normal Host state1 once.',
 'Modern waits actual Host D1one; accepted supplier status/handshake must confirm two-peer WAITING/Hosttrue/Guestfalse before GuestReady. Schema/guard not guessed.',
 'Guest typed Ready1 fullwrite, then actual E103; immediately typedCancel0 with noD1wait/extrapoll/timer/sleep.',
 'Cancel fullwrite, actual ownD1zero and E100 independent receipts; E100 clears countdown only, never D1/Host readiness.',
 '4second bounded observation, Close/join ownIO, queue and discardedlate separate; write firstfailure and partial queued/fullwrite/event records; no automatic retry.',
 'Unsupported/WORLD/start/result/timeout/source retirement wins over samepoll success predicate. No sameepochfreshness/allpeeratomic/queuedtickdrain/noWORLD guarantee.'],
 provider_dependencies=['Supervisor accepts new cold schema/source/binary pins and baseline-to-HostReady synchronization contract before finalexe sealing.',
 'Fresh acceptedREADY digest; never reuse retired667B69 or consumednw-6fdb7420a2bc4b8492dd7cb90ff270ed claim.',
 'Loopback exact endpoints/role/raw source expectations/new bundle/currentstatus<=5sec/lease>=180sec/declared helper+mysqlcreationidentity only, after accepted schema alignment.',
 'Private password-only properties unchanged4..13printableASCII/noescapes; never print/copy/hash secret; no Modern JavaSQL start/stop or serverwrites.',
 'Supply explicit baseline capturedD1/getter guard, HostSocket1 once provenance and stable gate/safe status fields; no assumption HostonlyReady is safe.'],
 final_commands_pending=True,new_network_cases=0,new_JavaSQL=0,GUI=0,old_PASS_replays=0))
print('R528 draft source/request written; provider schema remains closed, zero runtime')
