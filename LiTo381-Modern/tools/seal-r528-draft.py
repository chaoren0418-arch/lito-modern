"""Cold preparation only; no sockets, credential reads, process queries or replay."""
from r491_common import *
import shutil
candidate=read(L/'r528-draft-v2-candidate.json')
first=read(L/'r528-draft-candidate.json')
checked=[]
for c in [first,candidate]:
 for v in c.values():
  if isinstance(v,dict)and {'path','bytes','sha256'}<=v.keys():
   assert pin(v['path'])==v;checked.append(v)
assert candidate['final_provider_interface_accepted'] is False
assert candidate['preflight_connect_disabled_before_any_providerIO']
assert pin(P/'tools/r528-backend-consumer-draft.rs')['sha256']==candidate['source']['sha256']
assert pin(P/'tools/r528-countdown-case.rs')['sha256']==candidate['countdown_module']['sha256']
assert not (L/'r528-draft-v2-build.stderr.log').read_text('utf-8-sig').strip()
assert not (L/'r528-cold-sync-boundaries-build.stderr.log').read_text('utf-8-sig').strip()
for prefix,expected in [('r528-new-boundaries',2),('r528-cold-sync-boundaries',3)]:
 a=read(L/(prefix+'-accounting.json'));assert a['attempts']==1 and a['exit_code']==0 and a['old_tests_executed']==0 and not a['synthetic_Runtime_replay'] and not a['production_library_rebuilt']
 assert f'{expected} passed; 0 failed' in (L/(prefix+'.stdout.log')).read_text('utf-8-sig')
for prefix in ['r528-draft-preparation-check','r528-draft-v2-preparation-check']:
 a=read(L/(prefix+'-accounting.json'));assert a['attempts']==1 and a['exit_code']==0 and a['network_cases']==a['credentials_reads']==a['PIDqueries']==a['JavaSQL']==a['GUI']==0

source_snapshot=read(L/'r527-production-source-snapshot.json');sources=[]
for e in source_snapshot['source_pins']:
 for path in [e['path'],e['snapshot']]:
  v=pin(path);assert v['bytes']==e['bytes']and v['sha256']==e['sha256'];sources.append(v)
assert len(source_snapshot['source_pins'])==639
prior=pin(L/'r527-485F-real-backend-consumer-handoff.json')
assert prior['sha256']=='D05E390E50ACF543B202D700EF33726424FA9AB8DD445D5D2C795FEDA6DEE14E'

cold_path=P.parent.parent/'lito伺服器/server-modern-485F-countdown-cancel-cold-supply.json'
cold=read(cold_path);cold_pin=pin(cold_path)
assert cold['schema']=='server.modern485.countdown-cancel.cold-supply.v1'
assert cold['new_provider_runtime_attempts']==cold['new_auth_attempts']==0 and not cold['currently_running']
observed=L/'r528-observed-cold-provider';observed.mkdir(exist_ok=True)
cold_checked=[]
for key in ['handoff','launcher','launch_integrity','candidate','actual_gate_Code_source']:
 v=cold[key];assert pin(v['path'])==v;cold_checked.append(v)
 # Only cold textual contracts/code are copied. Never instantiate the launcher.
 if key!='candidate':shutil.copyfile(v['path'],observed/Path(v['path']).name)
shutil.copyfile(cold_path,observed/'observed-cold-supply.json')
helper=Path(cold['evidence_root'])/'source/Modern485ProviderMain.java'
shutil.copyfile(helper,observed/'Modern485ProviderMain.java')
save(L/'r528-cold-interface-proposal.json',dict(
 state='OBSERVED_COLD_SUPPLIER_NOT_YET_SUPERVISOR_ACCEPTED_NOT_IMPLEMENTED_IN_DRAFT_VALIDATOR',
 observed_root=cold_pin,observed_root_refs=cold_checked,
 READY_schema='server.modern485.provider.run.v1',status_schema='server.modern485.provider.status.v1',
 required_READY_extensions=dict(host_ready_after_guest_notready=True,host_sync_profile=dict(enabled=True,stage='WAITING_GUEST_JOIN_AND_BASELINE_D1_0',requires_normal_Guest_baseline_NotReady0=True,Host_participant_state1_sent=False,Host_owner_opcode_only_sent=False)),
 fresh_bundle_root=cold['evidence_root'],root_acceptance_pending=True,
 initial_GET='Host ROOM/notready, WAITING value0/deadline0, readyempty; no Host-onlyReady; exact fixed template/source/normal roles.',
 required_pre_GuestReady_sync_marker='HOST_READY_BOTH_D1_TWO_MEMBER_WAITING_WITNESSED',
 required_pre_GuestReady_GET=['Actual HostD1one in currentqualifiedroster, actualGuestbaseline0fullwrite+ownD1zero first.','Statusage<=5sec/providerREADY/no backgroundfailure/no WORLD, matching acceptedfreshrun/source/room.','host_sync baseline_D1_0_write_epochs and Host_D1_1_write_epochs each exactly Host and actualGuest session.','GET two combatants, hubmembers exactly Host+Guest, Hostowner/Hosttrue/Guestfalse/ROOM/source72/WAITING/countdown0/deadline0/hubreadyonlyHost.','Await safe armedmarker while holding Guest0/IOalive; no timer-triggered Ready; strict status missing/changed/expired/failed means stoponce.'],
 immediate_cancel_has_no_supplier_disk_barrier=True,
 strict_positive_E102_E101='Actual observations preserved then first-case failure; no retry/predicate repair.',
 residual_observation='4second Runtime poll only; no queuedtickdrain/freshness/WORLD exclusion proof.',
 final_validator_pending=['Replace intentionally closed draft validator after exact cold-root/sourcepins/synchronization contract accepted.','Retain bounded acceptedREADY digest/newrun/no reused667B69 or consumedclaim/currentstatus/lease>=180sec/declaredPIDFILETIME checks.','Strictly bind new exactbundle/candidate/source/roles/privateproperties format and initial Hostsync fields.','Implement fresh synchronized marker/GET read before GuestReady; no supplier file read between E103 and typedCancel0.','Seal final EXE only after this alignment; preparation itself does not authorize preflight/connect.'],
 credentials_reads=0,PIDqueries=0,provider_runs=0,network_cases=0))
save(L/'r528-draft-revision-accounting.json',dict(
 build_attempts=2,compile_failures=0,
 first_draft=pin(L/'r528-draft-candidate.json'),first_tests_passed=2,
 first_draft_warning='1 dead-code digest warning, retained in original build log; draft validator intentionally closed.',
 revision2=pin(L/'r528-draft-v2-candidate.json'),revision2_tests_passed=3,revision2_warnings=0,
 reason='New server cold coordination information requires Guest0baseline/actualHostD1+marker and strict tick2/1 stop; revised consumer additionally records fullwrite observation time/retiredpartial writes and requires actual ownD1one beforeE103.',
 old_R527_or_historical_synthetic_replays=0,production_library_rebuilds=0,
 prep_checks_per_revision=[1,1],prep_check_socket_credential_PID_counts=0,
 current_test_filter='countdown::r528_cold_sync_boundaries',
 first_positive_tick_policy_not_promoted_to_current=True,
 scratch_directories_retained=[first['scratch'],candidate['scratch']]))

human=P/'Native-R528_倒數取消consumer冷準備交接.txt'
human.write_text('''R528 倒數取消 consumer 冷準備交接（草稿，尚非最終實接執行檔）

目前零連線、零Java／SQL啟停、零private credentials／PID讀取；639份production來源未改，沿用R527凍結rlib 5C9DFD44。沒有重跑已通過R527或歷史synthetic Runtime。新供應根已看見，但尚未收到監製接受schema／Host同步契約，因此草稿在讀provider前就拒絕preflight與connect。

流程已準備：正常LOGIN/source/GAME/news/實singletonCB/C9/channel0/join，私有ROOM／qualifiedself；Host初始notready，Guest先typed state0基線完整write及實ownD1zero，供應者雙peer／GET確認後才正常HostSocket一次state1。Modern等實HostD1one及接受契約的雙人WAITING同步marker／GET，再送最後GuestReady1。

GuestReady必須完整write，並觀察實ownD1one与E103；觀察3後不加D1wait／額外poll／sleep／timer／supplier磁碟barrier，立即typedCancel0。Cancel完整write、實ownD1zero與E100為獨立條件；E100只清countdown，不改D1／Host準備狀態。四秒有限poll觀察後Close。tick2／1仍是wire observation，本次嚴格案一旦觀察即保留並停；不能升格sameepochfreshness／queuedtickdrain／全peer原子性或WORLD不可能先到。

result／trace預備欄位：stage／first_failure、Baseline/Ready/Cancel三請求purpose／sequence／write_serial／body／cipherframe／counter／Queued/CompletedWrite/Retired／partial writtenbytes／queued/fullwrite observed時間；實E1 bytes／local context labels、participant snapshot／最後D1、E103→cancel local時間、finite watch是否完成。首次unsupported／WORLD／start／result／timeout／write退役／來源退役優先於同poll成功條件；不重試。case_ok、Close_ok、join／queue合併才success；queue与discardedlate分列。當次partial及cleanup結果保留，server_stop_required由serverowner後續收尾。

兩個草稿build無compile failure。v1新邊界2tests通過，1個digest dead-code warning已保存；冷協調資訊新增後，v2針對獨立D1one與嚴格tick邊界3tests通過、warning0。v1舊寬鬆tickpolicy不採入目前流程。兩版各一次CPU-only --check-prepared，均0socket／credentials／PID／GUI。兩個自有ASCII scratch保留，無刪除。

現用草稿EXE SHA45399129CE1DF84BE6FAE413B7B1D17DD5F767EFD5F1CD98FC842475F248B29C
consumer source SHA509A6132330D09826081298163BEEEBDFF13E893FFAC486964F50741763ACF8A
countdown module SHA258473BEA2190F03EADD325964D47044120005F0DFCEBEEC084FE6079D4E8388

可執行冷檢查命令（本批已執行一次，不需重播）：
target/r528-countdown-draft-v2/r528-backend-consumer-draft.exe --check-prepared logs/continuation-20261003/r528-provider-supply-request.json

尚待：監製接受新冷供應7f8b1593的root／sourcepins／schema與Host同步。預期READY仍是server.modern485.provider.run.v1，但新增host_ready_after_guest_notready=true／host_sync_profile，不能只憑舊schema相同就放行。初始stage WAITING_GUEST_JOIN_AND_BASELINE_D1_0；最後GuestReady前必須實HostD1one且stage HOST_READY_BOTH_D1_TWO_MEMBER_WAITING_WITNESSED／雙人WAITING／Hosttrue Guestfalse。current draft尚未實作此新validator/GET讀取；callback與fail-closed接點已備，待接受後嚴格對齊並封存finalexe，才交實接preflight／connect命令。667B69 READY與舊claim已退休不可重用；冷接受不代表授權啟動或連線。

Batch36／37僅記未来UI依賴：834BD0是PNG、834BD4是IMG，byte-sum/signedmod16不是namespace分類；provider-to-file仍未知。本批無resource/UI擴張、GUI／WORLD／Native權限與整體完成仍未驗收。
''',encoding='utf-8')
roots=[prior,pin(human),pin(L/'r527-production-source-snapshot.json')]
names=['r528-provider-supply-request.json','r528-future-UI-dependencies.json','r528-cold-interface-proposal.json','r528-draft-revision-accounting.json','r528-draft-candidate.json','r528-draft-v2-candidate.json']
for prefix in ['r528-draft-build','r528-new-boundaries-build','r528-new-boundaries','r528-draft-preparation-check','r528-draft-v2-build','r528-cold-sync-boundaries-build','r528-cold-sync-boundaries','r528-draft-v2-preparation-check']:
 names += [prefix+'.stdout.log',prefix+'.stderr.log']
names += [prefix+'-accounting.json'for prefix in ['r528-new-boundaries','r528-draft-preparation-check','r528-cold-sync-boundaries','r528-draft-v2-preparation-check']]
roots += [pin(L/name)for name in names]
roots += [pin(p)for p in sorted(observed.iterdir())if p.is_file()]
roots += [pin(P/'tools'/name)for name in ['r528-backend-consumer-draft.rs','r528-countdown-case.rs','prepare-r528-consumer.py','build-r528-draft.ps1','build-r528-draft-v2.ps1','seal-r528-draft.py']]
verification=L/'r528-draft-preparation-verification.json'
save(verification,dict(status='PASS_DRAFT_EVIDENCE_NO_RUNTIME_FINAL_INTERFACE_PENDING',root_references_checked=roots,candidates_checked=checked,source_pins_checked=sources,source_files=639,source_changes=0,production_library_rebuilds=0,current_draft_warnings=0,new_current_tests_passed=3,earlier_draft_tests_passed=2,old_tests_run=0,network_cases=0,credentials_reads=0,PIDqueries=0,server_scripts_run=0))
roots.append(pin(verification))
handoff=L/'r528-countdown-cancel-draft-preparation-handoff.json'
save(handoff,dict(schema='modern.r528.countdown-cancel-draft-preparation-handoff.v1',updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
 state='DRAFT_FLOW_AND_NEW_BOUNDARIES_PREPARED_WAITING_ACCEPTED_COLD_SCHEMA_HOST_SYNC_FINAL_VALIDATOR',root_references=roots,
 previous_accepted_R527=prior,human=pin(human),candidate=candidate,earlier_draft=first,
 source_files=639,production_source_changes=0,production_library_rebuilds=0,
 final_executable_sealed=False,final_provider_interface_accepted=False,
 observed_supplier_root=cold_pin,observed_supplier_is_not_acceptance=True,
 interface_alignment_proposal=pin(L/'r528-cold-interface-proposal.json'),
 new_current_boundary_tests_passed=3,earlier_draft_tests_passed=2,current_draft_warnings=0,
 compile_attempts=2,compile_failures=0,old_PASS_replays=0,synthetic_Runtime_replay=0,
 commands=dict(check_prepared=[candidate['executable']['path'],'--check-prepared',str(L/'r528-provider-supply-request.json')],
 preflight_and_connect='Disabled before any provider IO. Final argv pending accepted new provider validator and final binary.'),
 first_failure_policy='Persist queued/partial/fullwrite/event/stage/firstfailure; close own Runtime; no retries; rejection/retirement outranks samepoll predicate.',
 normal_case_request_order=['GuestBaseline0','HostSocket1 by server after bothpeer baseline','GuestReady1','GuestCancel0 immediately after actualE103'],
 observation_limits=['actual3/2/1/0 wire only','E100 countdown clear only; separateD1zero required','No sameepochfreshness/queuedtickdrain/allpeeratomic/serverACK/WORLD exclusion','Strictcase tick2/1 firstfailure, no changedpredicate retry'],
 boundaries=dict(network_cases=0,preflight_runs=0,connect_runs=0,credentials_reads=0,PIDqueries=0,JavaSQL=0,server_writes=0,player_runs=0,GUI=0,oldPASS_replays=0),
 future_UI_only=pin(L/'r528-future-UI-dependencies.json'),verification=pin(verification),
 next='Supervisor accepts new cold schema/sourcepins/Host sync contract; replace closed validator/GET callback strictly and seal finalexe before one separately authorized freshprovider integration.',overall_completed=False))
for e in read(handoff)['root_references']:assert pin(e['path'])==e
v=read(P.parent/'監製進度.json');v['updated_at']=read(handoff)['updated_at'];v['phase']='R528_cold_draft_sealed_final_validator_pending_accepted_supplier_contract';v['overall_status']='in_progress';v['current_task']='R528正常entry／Guest0基線／實HostD1與同步callback／GuestReady fullwrite＋D1one／E103立即Cancel0／D1zero＋E100／有限watch／Close草稿冷封存。v2新3tests PASS/warning0，未重跑R527／歷史synthetic。觀測新cold根但監製acceptance與finalvalidator／GET對齊待續，finalexe未封、preflight/connect先封閉；0網路／JavaSQL／credentials／PID。'
v.setdefault('r528',{}).update(state=read(handoff)['state'],handoff=pin(handoff),human=pin(human),verification=pin(verification),candidate=pin(L/'r528-draft-v2-candidate.json'),new_tests=3,earlier_draft_tests=2,final_validator_pending=True,accepted_new_supplier_contract=False,final_executable_sealed=False,actual_backend_executions=0,overall_completed=False)
save(P.parent/'監製進度.json',v)
print(json.dumps(dict(status='DRAFT_SEALED_FINAL_INTERFACE_PENDING',handoff=pin(handoff),human=pin(human),roots=len(roots),current_new_tests=3,earlier_draft_tests=2,network_cases=0),ensure_ascii=False))
