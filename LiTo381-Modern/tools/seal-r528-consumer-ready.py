"""Seal the accepted cold interface and compiled consumer. Never run a provider."""
from r491_common import *
import shutil
candidate=read(L/'r528-final-candidate.json')
assert candidate['final_provider_interface_accepted'] and candidate['actual_backend_runs']==0
candidates=[]
for v in candidate.values():
 if isinstance(v,dict)and {'path','bytes','sha256'}<=v.keys():assert pin(v['path'])==v;candidates.append(v)
assert candidate['executable']['sha256']=='AB3B9ABEE0A3212D5BECEEB25E3A65B5CAEB4BA274E3AEDF0791D89775A9AF24'
assert candidate['production_library']['sha256']=='5C9DFD442B6F107B1A44DE1FA882A4517D449BE017F99F4B87E5F56BFA006502'
for key,name in [('source','r528-backend-consumer.rs'),('countdown_module','r528-final-countdown-case.rs'),('provider_module','r528-provider.rs')]:
 assert pin(P/'tools'/name)['sha256']==candidate[key]['sha256']
for name in ['r528-final-build.stderr.log','r528-final-interface-boundaries-build.stderr.log','r528-final-interface-boundaries.stderr.log','r528-final-preparation-check.stderr.log']:
 assert not (L/name).read_text('utf-8-sig').strip()
unit=read(L/'r528-final-interface-boundaries-accounting.json');assert unit['attempts']==1 and unit['exit_code']==0 and unit['network_cases']==unit['old_tests_executed']==0 and not unit['production_library_rebuilt'] and not unit['synthetic_Runtime_replay']
assert '3 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out' in (L/'r528-final-interface-boundaries.stdout.log').read_text('utf-8-sig')
check=read(L/'r528-final-preparation-check-accounting.json');assert check['attempts']==1 and check['exit_code']==0 and check['network_cases']==check['credentials_reads']==check['PIDqueries']==check['JavaSQL']==check['GUI']==0
assert 'CONSUMER_COLD_READY' in (L/'r528-final-preparation-check.stdout.log').read_text('utf-8-sig')
old_module=P/'target/r528-countdown-draft-v2/r528-countdown-case.rs'
new_module=Path(candidate['countdown_module']['path'])
test_marker='#[cfg(test)] mod r528_cold_sync_boundaries'
assert old_module.read_text('utf-8-sig').split(test_marker)[1]==new_module.read_text('utf-8-sig').split(test_marker)[1]
assert old_module.read_text('utf-8-sig').split('pub struct Trace')[0]==new_module.read_text('utf-8-sig').split('pub struct Trace')[0]
draft_handoff=pin(L/'r528-countdown-cancel-draft-preparation-handoff.json')
assert draft_handoff['sha256']=='73F9EA92A2D1EF0D9C9CF3FB2D7EDB869A7A5568C628AE8761E4991D23CED511'
for e in read(draft_handoff['path'])['root_references']:assert pin(e['path'])==e
snapshot=read(L/'r527-production-source-snapshot.json');sources=[]
for e in snapshot['source_pins']:
 for path in [e['path'],e['snapshot']]:
  a=pin(path);assert a['bytes']==e['bytes'] and a['sha256']==e['sha256'];sources.append(a)
assert len(snapshot['source_pins'])==639
adoption=read(L/'r528-accepted-final-cold-adoption.json')
assert adoption['accepted_root']['sha256']=='4646D28B95674D2415D5CBAFBA1A365D67C44C00BD8F5ED1B537D42530E67D73'
assert adoption['accepted_root']['bytes']==4578
for key,count in [('root_references_checked',12),('manifest_references_checked',87),('launch_references_checked',24)]:
 assert len(adoption[key])==count
 for e in adoption[key]:assert pin(e['path'])==e
assert pin(adoption['accepted_root']['path'])==adoption['accepted_root']
assert pin(adoption['own_root']['path'])==adoption['own_root']

bin_root=P.parent.parent/'lito脫殼資源/Native-provider-writer-contract.json'
bp=pin(bin_root);assert bp['sha256'].startswith('F989981D')
ui_copy=L/'r528-Batch38-Native-provider-writer-contract.json';shutil.copyfile(bin_root,ui_copy)
save(L/'r528-Batch38-future-UI-dependency.json',dict(accepted_root=bp,own_copy=pin(ui_copy),scope='FutureUIsourceonly; actual provider-to-file identity remains unknown; no newresource/UIimplementation.',GUI=0,resource_changes=0))
save(L/'r528-final-build-accounting.json',dict(
 all_R528_consumer_build_attempts=3,compile_failures=0,
 earlier_draft_builds_preserved=2,earlier_draft_new_tests_per_build=[2,3],
 final_build_attempts=1,final_new_interface_tests_passed=3,previous_current_policy_tests_filtered=3,
 final_consumer_warnings=0,final_test_compile_warnings=0,
 earlier_draft1_dead_code_warning_preserved=1,production_library_rebuilds=0,
 R527_regular_library_warning_messages_historical=288,R527_new_product_warnings_historical=0,
 prep_CPU_only_checks=[1,1,1],preflight_runs=0,connect_runs=0,
 old_R527_PASS_or_synthetic_Runtime_replays=0,
 source_delta='Only new consumer sources/interface;639production sources unchanged.',
 final_setup_repair='Runtime/inputs initialization wrapped to save pre-socket failure; deadline checked before completion predicates both before/after poll.',
 scratch_retained=[read(L/'r528-draft-candidate.json')['scratch'],read(L/'r528-draft-v2-candidate.json')['scratch'],candidate['scratch']],
 deletes=0,JavaSQL=0,server_writes=0))
request=read(L/'r528-final-provider-supply-request.json')
human=P/'Native-R528_485F倒數取消consumer就緒交接.txt'
human.write_text('''R528 真正485F COUNTDOWN／E1→typed取消 consumer 就緒（冷準備完成，實接尚未授權）

已嚴格採用監製接受的新供應根4646D28B（4578bytes）、新bundle7f8b1593、handoff758D0768、launch AC320ACB、manifest71DC9ACC。12根／87manifest／24launch引用獨立相符。沿用R527凍結regular rlib5C9DFD44，639份production來源及snapshot未改；不改Java／SQL／resource／Native／default main。

最終EXE：target/r528-final-backend-consumer/r528-backend-consumer.exe
SHA256 AB3B9ABEE0A3212D5BECEEB25E3A65B5CAEB4BA274E3AEDF0791D89775A9AF24
consumer source 1D034364255B5B9091864B869EEE2B95B6348DA8E4B6C5C3309AC84B16B4DFA5
countdown source 9AE57C12F033ED87F7F1FAA8BA98741B690B581166465C8763CA4E89D520E03A
provider source 640E21083FA06A80479D287ABFD250B631142917992F027724445E32C070348B
library SHA 5C9DFD442B6F107B1A44DE1FA882A4517D449BE017F99F4B87E5F56BFA006502

冷驗證：final EXE／test compile均warning0；新provider邊界3tests passed／0failed／3舊countdown policy tests filtered，沒有重播。先前v2的3policy tests及Policy/test正文一致，只保留已有unit邊界證據，不作runtime通過。final CPU-only --check-prepared一次PASS，socket／credentials／PID／JavaSQL／GUI皆0；本批final preflight／connect尚未執行。兩版draft與各build／新test／CPUcheck保留，R528共3consumer build、compile failure0、歷史draft1 dead-code warning1保留、未重建production library、3個自有ASCII scratch保留。

正常流程：真正LOGIN/sourceFx485Moon/GAME/news/完整singletonCB→同rawbytes C9→acceptedC8→channel0→actual roomjoin/privatequalifiedself。Host初始notready；Guest先typedNotReady0基線完整body2/frame4/write4/counter+2、實ownD1zero。只有supplier雙peer baseline D1／GET確認兩位正常participant後，才正常HostSocket一次state1。

Modern須實收HostD1one，並等fresh status HOST_READY_BOTH_D1_TWO_MEMBER_WAITING_WITNESSED：baseline与HostD1的兩peer write-epoch集合精確相同；兩combatants／hub members／connectedGAME、Hostowner／Hosttrue／Guestfalse、真source72／選角已消耗／ROOM/currentroom/channel0、WAITING／countdown0/deadline0／hubreadyonlyHost。這是分開field/hub GET，不是全peer原子性、ACK或client identity authority。physical ID只作backend一致性觀測，真正CB rawalias仍是client選角權限。

Guest最後Ready1完整write与實ownD1one後，實收E103便立即typedCancel0；不多做poll／D1wait／sleep／timer或supplier磁碟barrier。Cancel完整write、實ownD1zero与E100是獨立條件；E100只清countdown，不改D1／Hoststate。四秒Runtime poll觀察、Close、own IO join；queue与discardedlate分開。成功須case_ok／Close_ok／2IO started=joined／queue0；不是GUI或WORLD驗收。

firstfailure：同poll先記partial snapshot／E1／write，再判source rejection／ROOMepoch retirement／owned IO ended／Retired write／tick偏離／timeout，才可接受完成predicate。tick2／1仍是wire observation，但本次嚴格案立即保留並停；WORLD／start／result／unsupported／timeout等皆首次停止，無重試／改predicate。result保留Baseline/Ready/Cancel requestpurpose／sequence／write_serial／cipherframe／counter／Queued/CompletedWrite/Retired／partial bytes／observed時間；E1 bytes／local labels／最後D1／first_failure／stage／case_error與Close_error／join／queue／late。未claim的freshguard失敗由外層exit/stdout/stderr accounting保留；claim後credentials／初始化／case失敗寫result且供應者需收尾。

fresh接受条件：exact newbundle／candidate／handoff／24launch pins、trusted新READY SHA；READY仍表示provider alive，不代表Host已ready。拒絕退休667B69／nw6f與nw97、舊schema缺newmode/sync、非新UUID／逃出runroot／不符roles/source/template。launch須一次supportHost+sync+900s；preflight/connect lease>=180s，status mtime<=5s／candidate/run/bundle/port/helper/lease一致且無backgroundfailure／WORLD；同步gate至少60s lease。只查宣告helper／mysql指定PID／creation FILETIME／STILL_ACTIVE及既有protectedPID／exe／command／datadir守門，不操作程序、不讀hash datadir內容。

private password-only properties仍4..13可列印ASCII／noescapes（供應器生成12bytes），Zeroizing／Drop zeroize，不印／copy／hash秘密。Modern不啟停provider／Java／SQL、不寫server namespace、不讀rawauthcapture。本批只有冷檔案／JSON單元測試與CPUcheck。

命令與完整argv：logs/continuation-20261003/r528-final-provider-supply-request.json
冷檢查（已執行一次）：EXE --check-prepared <該final supply request絕對路徑>
另行授權fresh後：EXE --preflight <Modern絕對root> <監製接受fresh ready-contract.json> <trustedSHA>
隨後一次：EXE --connect-once <相同root> <相同freshREADY> <相同SHA>
每run output logs/continuation-20261003/r528-backend-<freshnwUUID>/result.json；create_new claim禁止重用。server供應者依監製另行900s start／collect wire與GET／normalstop／指定PID退休／ports free／datadir retained。冷就緒不等於實接許可；本批未有freshREADY。

限制：E1沒有wire roomid／generation／sequence，3/2/1僅觀測、0僅countdownclear；不宣稱sameepoch freshness／queuedtickdrain／sendfailure/race／全peer原子性／WORLD不可能先到。Close EOF peerD0不是committed ownleave。GUI／originalart／defaultmain／Native privateUI／WORLD等仍未驗收。Batch36／37／38只記未来UI來源；834BD0 PNG／834BD4 IMG、signedmod16及writer仍缺實體provider-to-file identity，未補未知圖資。整體in_progress。
''',encoding='utf-8')
roots=[draft_handoff,pin(human),pin(L/'r527-production-source-snapshot.json')]
names=['r528-final-candidate.json','r528-accepted-final-cold-adoption.json','r528-final-provider-supply-request.json','r528-final-build-accounting.json','r528-Batch38-future-UI-dependency.json','r528-Batch38-Native-provider-writer-contract.json','r528-future-UI-dependencies.json']
for prefix in ['r528-final-build','r528-final-interface-boundaries-build','r528-final-interface-boundaries','r528-final-preparation-check']:
 names += [prefix+'.stdout.log',prefix+'.stderr.log']
names += ['r528-final-interface-boundaries-accounting.json','r528-final-preparation-check-accounting.json']
roots += [pin(L/name)for name in names]
roots += [pin(p)for p in sorted((L/'r528-accepted-final-cold-provider').iterdir())if p.is_file()]
roots += [pin(P/'tools'/name)for name in ['r528-backend-consumer.rs','r528-final-countdown-case.rs','r528-provider.rs','finalize-r528-source.py','build-r528-final.ps1','seal-r528-consumer-ready.py']]
verification=L/'r528-final-consumer-ready-verification.json'
save(verification,dict(status='PASS_COLD_CONSUMER_READY_NOT_RUNTIME_ACCEPTANCE',root_references_checked=roots,candidates_checked=candidates,source_pins_checked=sources,source_files=639,production_source_changes=0,
 new_tests_passed=3,previous_policy_tests_filtered=3,previous_policy_and_test_body_unchanged=True,
 final_exe_test_warnings=0,new_network_cases=0,credential_reads=0,PIDqueries=0,JavaSQL=0,server_writes=0,old_PASS_replays=0))
roots.append(pin(verification))
handoff=L/'r528-countdown-cancel-consumer-ready-handoff.json'
save(handoff,dict(schema='modern.r528.countdown-cancel-consumer-ready-handoff.v1',updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
 state='COLD_CONSUMER_READY_ACCEPTED_NEW_PROVIDER_SCHEMA_FRESH_RUNTIME_NOT_AUTHORIZED',root_references=roots,human=pin(human),candidate=candidate,
 library=candidate['production_library'],source_manifest=pin(L/'r527-production-source-snapshot.json'),source_files=639,production_source_changes=0,production_library_rebuilds=0,
 adopted_provider=pin(L/'r528-accepted-final-cold-adoption.json'),accepted_cold_root=adoption['own_root'],cold_root_reference_count=12,manifest_reference_count=87,launch_reference_count=24,
 draft_preparation_history=draft_handoff,final_interface_accepted=True,final_executable_sealed=True,
 new_provider_interface_unit_tests=dict(passed=3,failed=0,filtered_previous_policy_tests=3,attempts=1,network_cases=0),
 previous_countdown_policy_tests='3 saved unit tests, unchangedPolicy/testbody; not rerun or promoted to truebackend proof.',
 warning_count=0,build_accounting=pin(L/'r528-final-build-accounting.json'),
 commands=request['commands'],supply_request=pin(L/'r528-final-provider-supply-request.json'),result_path=request['result_path'],
 event_order=request['flow'],fresh_requirements=request['fresh_conditions'],armed_stage=request['armed_stage'],
 first_failure_policy=request['failure_policy'],
 result_fields=['stage/first_failure/case_error/Close_error','Baseline/Ready/Cancel purpose/sequence/write_serial/body/cipher/counter/queued/fullwrite/retired/partialbytes/times','Actual ownD1one/zero and E103/E100 observations/localcontextlabels','finite4s watch/2IOstartedjoined/queue/discardedlate/Close','provider_stop_required/zeroModernJavaSQLGUI/zeroautoretry'],
 result_success='case_ok AND Close_ok AND2IOstarted=joined ANDqueue0; late separately recorded.',
 execution=dict(preparation_CPU_check_attempts=1,preflight_runs=0,connect_runs=0,actual_backend_cases=0,provider_runs=0,JavaSQL=0,credentials_reads=0,PIDqueries=0,server_writes=0,GUI=0,player_runs=0,old_PASS_replays=0),
 fresh_READY_accepted=False,fresh_provider_execution_authorized=False,
 retired_R527_READY_or_claim_reused=False,
 future_UI_dependencies=[pin(L/'r528-future-UI-dependencies.json'),pin(L/'r528-Batch38-future-UI-dependency.json')],
 remaining=request['limits']+['Actual new485F COUNTDOWN/E1/cancel runtime acceptance pending separately authorized fresh900s run.','NativeGUI/originalart/defaultmain/WORLD/committedownleave/partialwrite/race/resourcephysicalfileidentity remainopen.'],
 verification=pin(verification),awaiting_supervisor_consumer_ready_acceptance=True,overall_completed=False))
for e in read(handoff)['root_references']:assert pin(e['path'])==e
v=read(P.parent/'監製進度.json');v['updated_at']=read(handoff)['updated_at'];v['phase']='R528_final_consumer_cold_ready_awaiting_supervisor_fresh_runtime_coordination';v['overall_status']='in_progress';v['current_task']='R528新冷契約4646D28B已嚴格採用；最終consumer AB3B9ABE／固定rlib／639來源未改，3個新schema/Hostguard單元tests PASS、舊policy3filtered/warning0，CPUcheck一次。baseline0→實HostD1+fresharmed兩人WAITING→GuestReady fullwrite/D1one→實E103立即Cancel0→D1zero/E100→watch/Close封存。未preflight/connect/啟provider，0credentials/PID/JavaSQL，待監製接受consumer-ready後另授權fresh900s一次；GUI/WORLD/freshness等仍open。'
r=v.setdefault('r528',{});r['draft_handoff']=draft_handoff
r.update(state=read(handoff)['state'],handoff=pin(handoff),human=pin(human),verification=pin(verification),candidate=pin(L/'r528-final-candidate.json'),
 accepted_new_supplier_contract=True,final_validator_pending=False,final_executable_sealed=True,new_final_interface_tests=3,previous_policy_filtered=3,
 actual_backend_executions=0,preflight_runs=0,connect_runs=0,fresh_provider_execution_authorized=False,fresh_READY_accepted=False,awaiting_supervisor_consumer_ready_acceptance=True,overall_completed=False)
save(P.parent/'監製進度.json',v)
print(json.dumps(dict(status='FINAL_COLD_CONSUMER_READY_SEALED',handoff=pin(handoff),human=pin(human),root_references=len(roots),executable=candidate['executable'],new_tests=3,old_filtered=3,network_cases=0),ensure_ascii=False))
