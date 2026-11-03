from r491_common import *
draft=P/'target/r528-countdown-draft-v2'
assert pin(draft/'r528-backend-consumer-draft.rs')['sha256']=='509A6132330D09826081298163BEEEBDFF13E893FFAC486964F50741763ACF8A'
assert pin(draft/'r528-countdown-case.rs')['sha256']=='258473BEA2190F03EADD325964D47044120005F0DFCEBEEC084FE6079D4E8388'
s=(draft/'r528-backend-consumer-draft.rs').read_text('utf-8-sig')
s=s.replace('mod countdown {include!("r528-countdown-case.rs");}','mod countdown {include!("r528-final-countdown-case.rs");}\nmod provider {include!("r528-provider.rs");}')
s=s.replace('const FINAL_PROVIDER_INTERFACE_ACCEPTED:bool=false;','const FINAL_PROVIDER_INTERFACE_ACCEPTED:bool=true;')
s=s.replace('struct Provider {consumer:ConsumerSpec}','struct Provider {consumer:ConsumerSpec,status_file:PathBuf,run_root:PathBuf,helper_pid:u32,port:u16,lease_end_epoch_ms:u64}')
start=s.index('// This draft cannot');end=s.index('fn credentials_from_properties(')
s=s[:start]+'''fn validate_provider(raw:&[u8],accepted_sha:&str)->Result<Provider>{provider::validate(raw,accepted_sha)}
fn synchronized_host_gate(r:&Runtime,p:&Provider)->Result<Option<Value>>{provider::synchronized_host(r,p)}
'''+s[end:]
s=s.replace('provider password must be4..13 ASCIIbytes;current coldlauncher token_urlsafe32 incompatible;contents withheld','provider password must be4..13 ASCIIbytes;contents withheld')
s=s.replace('v["status"]=="WAITING_ACCEPTED_COLD_SCHEMA_AND_HOST_SYNC"&&v["credentials_contents_in_report"]==false','v["status"]=="COLD_CONSUMER_READY_WAITING_AUTHORIZED_FRESH_PROVIDER"&&v["credentials_contents_in_report"]==false&&v["accepted_cold"]["sha256"]==provider::COLD_ROOT_SHA')
s=s.replace('R528 DRAFT_PREPARED_ONLY;0socket/0credential/PIDread/0JavaSQLGUI;final provider interface pending','R528 CONSUMER_COLD_READY;0socket/0credential/PIDread/0JavaSQLGUI;WAITING_AUTHORIZED_FRESH_PROVIDER')
s=s.replace('R528 cold schema/Host synchronization not accepted; draft stops before reading provider, credentials or PIDs; zero sockets','R528 cold provider interface not accepted')
s=s.replace('ensure!(root.is_absolute(),"absolute Modern root required");','ensure!(root==Path::new("D:/lineage開發/3-lito開發/lito重建版/LiTo381-Modern"),"exact own Modern root required");')
s=s.replace('let p=validate_provider(&raw,&args[3])?;','let p=validate_provider(&raw,&args[3])?;ensure!(Path::new(&args[2])==p.run_root.join("ready-contract.json"),"exact fresh run READY path required");')
s=s.replace('let mut r=match Runtime::start_with_menu_skin(inputs(root,"r486-map-binding.json")?,inputs(root,"r506-native-menu-skin-binding.json")?){','let mut r=match (||Runtime::start_with_menu_skin(inputs(root,"r486-map-binding.json")?,inputs(root,"r506-native-menu-skin-binding.json")?))(){')
s=s.replace('let success=case_ok&&close_result.is_ok()&&joined;','let success=case_ok&&close_result.is_ok()&&joined&&stats.workers_started==2;')
s=s.replace('"Ready_complete_write":t.policy.ready_full','"expected_IO_workers":2,"Ready_complete_write":t.policy.ready_full')
(P/'tools/r528-backend-consumer.rs').write_text(s,'utf-8')
module=(draft/'r528-countdown-case.rs').read_text('utf-8-sig')
module=module.replace('        if predicate(r,t){return Ok(());}\n        if Instant::now()>=end', '        if Instant::now()>=end')
module=module.replace('anyhow::bail!("bounded stage timeout; no retry");}\n        r.poll();','anyhow::bail!("bounded stage timeout; no retry");}\n        if predicate(r,t){return Ok(());}\n        r.poll();')
module=module.replace('        t.observe(r,own,host)?;\n        if predicate(r,t){return Ok(());}\n        thread::sleep', '        t.observe(r,own,host)?;\n        if Instant::now()>=end {t.failure(json!({"kind":"bounded_stage_timeout"}));anyhow::bail!("bounded stage timeout after poll; no retry");}\n        if predicate(r,t){return Ok(());}\n        thread::sleep')
(P/'tools/r528-final-countdown-case.rs').write_text(module,'utf-8')
base=(P/'tools/build-r528-draft-v2.ps1').read_text('utf-8')
b=base.replace('r528-countdown-draft-v2','r528-final-backend-consumer').replace('r528-draft-v2-build-','r528-final-build-').replace('r528-backend-consumer-draft','r528-backend-consumer').replace("'r528-countdown-case.rs'","'r528-final-countdown-case.rs','r528-provider.rs'")
b=b.replace("@('anyhow','serde_json','sha2','zeroize')","@('anyhow','serde_json','sha2','zeroize','serde','windows')")
b=b.replace('r528-draft-v2-build.stdout','r528-final-build.stdout').replace('r528-draft-v2-build.stderr','r528-final-build.stderr')
b=b.replace('r528-cold-sync-boundaries','r528-final-interface-boundaries').replace("'countdown::r528_cold_sync_boundaries'","'r528_final_provider_boundaries'")
b=b.replace('r528-draft-v2-preparation-check','r528-final-preparation-check').replace('r528-provider-supply-request.json','r528-final-provider-supply-request.json').replace('r528-draft-v2-candidate.json','r528-final-candidate.json')
b=b.replace("status='DRAFT_ONLY_WAITING_ACCEPTED_COLD_SCHEMA_HOST_SYNC'","status='COLD_CONSUMER_READY_FRESH_PROVIDER_EXECUTION_NOT_AUTHORIZED'")
b=b.replace('final_provider_interface_accepted=$false;preflight_connect_disabled_before_any_providerIO=$true','final_provider_interface_accepted=$true;actual_backend_runs=0;provider_module=Pin (Join-Path $root \'r528-provider.rs\')')
b=b.replace('Draft','Final cold consumer').replace('draft','cold consumer')
(P/'tools/build-r528-final.ps1').write_text(b,'utf-8')
exe=P/'target/r528-final-backend-consumer/r528-backend-consumer.exe'
commands=dict(check_prepared=[str(exe),'--check-prepared',str(L/'r528-final-provider-supply-request.json')],preflight=[str(exe),'--preflight',str(P),'<supervisor-accepted fresh ready-contract.json>','<trusted fresh READY SHA256>'],connect_once=[str(exe),'--connect-once',str(P),'<same fresh ready-contract.json>','<same trusted SHA256>'])
save(L/'r528-final-provider-supply-request.json',dict(schema='modern.r528.provider-supply-request.v1',status='COLD_CONSUMER_READY_WAITING_AUTHORIZED_FRESH_PROVIDER',accepted_cold=pin(L/'r528-accepted-final-cold-provider/accepted-final-cold-supply.json'),credentials_contents_in_report=False,commands=commands,
provider_schema='server.modern485.provider.run.v1',new_mode_required=True,provider_run_or_connect_authorized=False,fresh_READY_accepted=False,
fresh_conditions=['New nw lowercase32hex UUID under exactserver namespace, exact accepted7f8b bundle and485F candidate; reject retired667B69/oldnw97/nw6f consumedrun.','Trust supervisor accepted newREADY digest; schemaREADY meansprovideralive only; supportHost/mode/sync enabled, initialstage WAITING_GUEST_JOIN_AND_BASELINE_D1_0/emptywriteepoch arrays/Hostfalse; exactroles/source/template.','Exactlaunch attempts1/supportHost/sync/900secondlease; lease>=180s atpreflight/connect; currentatomicstatus mtime<=5s/header/lease/no backgroundfailure/noWORLD.','Verify acceptedhandoff758D0768 andlaunchAC320ACB plus24launchfilepins; verifyJar485F bytes8906219.','Readown launch/ownershipmetadata; inspectonlydeclaredhelper/mysql protectedPIDrefusal/livePIDcreationFILETIME/executable/command/datadir. No processoperation ordatadircontentread/hash.','Privatepassword-only properties4..13ASCII/noescapes; actualgenerator12; keepzeroize/noprintcopyhash.'],
flow=['normalentry/privatequalifiedself→GuestBaseline0fullwrite+actualD1zero','actualHostD1one plusfresharmedstatus bothbaseline/HostD1writeepochwitnesses andtwo actualROOM peers/WAITING0/0/HosttrueGuestfalse','GuestReady1fullwrite+actualownD1one→actualE103→immediatetypedCancel0(no diskbarrier,extrapoll,sleep,timer)','Cancel0fullwrite+actualownD1zero+E100→4s actualRuntimepoll watch→Close/join/queue/late'],
armed_stage='HOST_READY_BOTH_D1_TWO_MEMBER_WAITING_WITNESSED',armed_remaining_lease_ms=60000,
failure_policy='Samepoll rejection/retirement/timeout outrankscompletion; actualtick2/1 orWORLD/start/result/other deviation stopfirst, partialresult/Close retained,no retry.',
result_path=str(L/'r528-backend-<fresh-nw-UUID>/result.json'),Modern_server_start_stop=False,GUI=0,
limits=['3/2/1 remainwire observations','0clearscountdown only','GET separatefield/hubnotatomic','No sameepochfreshness/queuedtickdrain/WORLDexclusion/ACK','Newcoldacceptance notruntimeacceptance; pending separateauthorization']))
print('Final source/new provider interface prepared; frozen drafts unchanged; no runtime')
