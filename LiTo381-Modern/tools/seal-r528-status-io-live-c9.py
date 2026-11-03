import pathlib,json,hashlib,datetime
P=pathlib.Path(__file__).resolve().parent.parent
L=P/'logs/continuation-20261003'
RUN='nw-c9afae2f2cd14da08208599b1bf066fb'
E=L/('r528-status-io-live-'+RUN)
def read(p):return json.loads(pathlib.Path(p).read_text('utf-8-sig'))
def pin(p):
 p=pathlib.Path(p);raw=p.read_bytes();return dict(path=str(p),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest().upper())
def save(p,v):
 with pathlib.Path(p).open('x',encoding='utf-8',newline='\n')as f:json.dump(v,f,ensure_ascii=False,indent=2);f.write('\n')
now=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
result_path=L/('r528-backend-'+RUN)/'result.json';r=read(result_path)
pre=read(E/'preflight-accounting.json');conn=read(E/'connect-accounting.json');claim=read(E/'authorized-run-claim.json')
assert pre['attempts']==conn['attempts']==1 and pre['exit_code']==conn['exit_code']==0
assert r['success'] and r['case_ok'] and r['first_failure']is None and r['case_error']is None
assert r['fresh_run_id']==RUN and r['actual_backend_attempts']==1 and not r['retry'] and not r['synthetic_replay']
assert r['Close_ok'] and r['Close_error']is None and r['IO_started']==r['IO_joined']==2 and r['queued_events']==r['discarded_late_events']==0
assert r['finite_watch_completed'] and r['watch_milliseconds']==4000 and r['actual_E103_observed'] and r['actual_E100_observed'] and r['actual_own_D1one_observed'] and r['actual_own_D1zero_observed']
assert not r['credentials_contents_or_hash_recorded'] and not r['Modern_starts_or_stops_JavaSQL'] and r['GUI']==0 and not r['owner_start_WORLD_allowed']
assert not r['WORLD_impossible'] and not r['queued_tick_drain_proven'] and not r['sameepoch_freshness_proven'] and not r['server_acceptance_ACK']
assert r['provider_stop_required']
assert r['provider_acceptance_sha256']=='2B45D78545ADF12A0CF2FAD02FB1ABF1E9899E3F2AE5B40A8A9F68BBF900D5F4'
for key in ['consumer','ready','supply','request']:
 v=claim[key];actual=pin(v['path']);assert actual['bytes']==v['bytes']and actual['sha256']==v['sha256']
assert claim['consumer']['sha256']=='FAD83BE02142260C29DC24184580ED0283E69325AE0D6C4C4FB1DA1F22377182'
assert claim['request']['sha256']=='7089029DAECDE224DE59AE1301EA1CE30D378D3D9AA60166734C717B859A2ED5'
assert pin(E/'accepted-ready-contract.json')['sha256']==claim['ready']['sha256']
assert pin(E/'accepted-live-supply.json')['sha256']==claim['supply']['sha256']
requests=r['requests'];assert len(requests)==3
for q,kind,intent in zip(requests,['Baseline','Ready','Cancel'],[0,1,0]):
 assert q['purpose']==kind and q['intent_byte']==intent and q['body_hex']==f'd2{intent:02x}' and q['phase']=='CompletedWrite'
 assert q['frame_bytes']==q['written_bytes']==4 and q['counter_after']==q['counter_before']+2 and q['retirement']is None and not q['server_acceptance_ack']
assert requests[1]['counter_before']==requests[0]['counter_after'] and requests[2]['counter_before']==requests[1]['counter_after']
events=r['events'];e1=[e for e in events if e['kind']=='actual_E1']
assert [e['details']['count']for e in e1]==[3,0]
assert [e['details']['body_hex']for e in e1]==['e103','e100']
assert all(not e['details']['wire_room_generation_sequence'] for e in e1)
gate=[e for e in events if e['kind']=='accepted_supplier_synchronized_host_gate'];assert len(gate)==1
g=gate[0]['details'];assert g['host_sync']['stage']=='HOST_READY_BOTH_D1_TWO_MEMBER_WAITING_WITNESSED'
assert g['getter']['combatant_count']==2 and g['getter']['match_phase']=='WAITING' and not g['getter']['battle_started'] and g['getter']['countdown_value']==g['getter']['countdown_next_nanos']==0
assert g['getter']['sessions']['Host-GAME']['ready'] and not g['getter']['sessions'][g['observed_Guest_session']]['ready']
assert g['modern_status_reader']['method']=='same_opened_handle_metadata_and_bytes' and g['status_poll_minimum_interval_ms']==200
participants=[]
for e in events:
 if e['kind']=='participants_snapshot':
  participants.append(dict(observed_us=e['observed_us'],stage=e['stage'],**e['details']))
assert [p['revision']for p in participants]==[1,2,3,4,5,6]
assert [next((row['state']for row in p['rows']if row['role']=='Guest'),None)for p in participants]==[None,0,0,0,1,0]
assert [next(row['state']for row in p['rows']if row['role']=='Host')for p in participants]==[0,0,0,1,1,1]
delta=r['cancel_queued_us']-r['E103_observed_us'];assert delta>=0
summary=dict(schema='modern.r528.status-io.real-countdown-cancel-observations.v1',run_identity=RUN,status='MODERN_BOUNDED_ACTUAL_CASE_PASS_SERVER_FINAL_WIRE_GET_AND_RETIREMENT_PENDING',updated_at=now,
 actual_result=pin(result_path),consumer=claim['consumer'],trusted_ready=claim['ready'],live_supply=claim['supply'],accepted_cold_handoff=pin(L/'r528-status-io-consumer-ready-handoff.json'),
 process_wallclock_windows=dict(preflight_start_utc=pre['start_utc'],preflight_end_utc=pre['end_utc'],connect_start_utc=conn['start_utc'],connect_end_utc=conn['end_utc']),
 timeline_clock='All observed_us/queued_us are the one Rust Trace Instant origin, not wire timestamp or UTC; do not map process wallclock exactly to Trace origin.',
 requests=requests,participants=participants,actual_E1_events=e1,synchronized_host_gate=gate[0],
 E103_to_cancel=dict(E103_observed_us=r['E103_observed_us'],cancel_queued_us=r['cancel_queued_us'],local_observation_to_enqueue_delta_us=delta,request_queued_us=requests[2]['queued_us'],request_complete_write_observed_us=requests[2]['complete_write_observed_us'],request_delta_uses_separate_local_instrumentation=True,no_status_IO_sleep_or_wait_between=True,source_evidence=pin(P/'target/r528-status-io-backend-consumer/r528-status-io-countdown-case.rs'),not_wire_arrival_or_server_ack=True),
 finite_watch=[e for e in events if e['kind']=='finite_watch_completed'],
 cleanup=dict(Close_ok=r['Close_ok'],IO_started=r['IO_started'],IO_joined=r['IO_joined'],queued_events=r['queued_events'],discarded_late_events=r['discarded_late_events'],server_stop_required=True,Modern_server_start_stop=False),
 status_IO_diagnostic_scope='Armed snapshot only; diagnostic counters0/first_error null at that snapshot, not a claim of zero publication failures for the whole server run. 200ms limit and finite Java retries do not prove all-rate compatibility.',
 actual_D1_scope='Participant snapshots contain last actual D1 bodies. Revisions/local epochs are client labels; snapshot sampling does not log every intermediate wire D1.',
 actual_E1_scope='Actual opcode225 + BYTE3 and BYTE0 observations. No wire room/generation/sequence/timestamp; 0 clears countdown only. No inferred missing tick2/1, no guaranteed drain/freshness.',
 limits=['One authorized actual preflight and connect, not stress/timing/race proof.','No globalatomic GET/ACK/sameepochfreshness/queuedtickdrain/WORLD impossibility/GUI acceptance.','Server side final wire/GET observations and helper/mysql retirement owned by server group, pending at sealing.','Historical nw975 failure and bounded-publication stress exhaustion retained; this pass does not establish original cause.'],
 accounting=dict(preflight_attempts=1,connect_attempts=1,actual_backend_attempts=1,IO_started=2,IO_joined=2,queue=0,late=0,retries=0,production_builds=0,old_PASS_replays=0,GUI=0,Modern_JavaSQL_start_stop=0,server_writes=0,private_password_print_copy_hash=False,evidence_seal_attempts=2,evidence_seal_first_failure=pin(E/'seal-first-failure.json'),runtime_replayed_on_seal_repair=False))
save(E/'actual-observations.json',summary)
human=P/'Native-R528_485F真實倒數取消Modern交接.txt'
human.write_text('R528 實際485F倒數取消：Modern端有限案例通過\n\n'
 f'唯一run：{RUN}\n唯一READY SHA256：{claim["ready"]["sha256"]}\nConsumer SHA256：{claim["consumer"]["sha256"]}\n\n'
 f'一次preflight：{pre["start_utc"]}～{pre["end_utc"]}，exit0／zero sockets。\n一次connect-once：{conn["start_utc"]}～{conn["end_utc"]}，exit0／success true，無重試。\n\n'
 '正常LOGIN→source Fx485Moon→GAME→news→actual singletonGuest CB/C9→channel0→current room1。Guest Baseline0(d200)完整4byte写入，actual ownD1=0 revision3；actual HostD1=1 revision4。新samehandle reader取得雙peer WAITING0/0／HosttrueGuestfalse armed status，兩端baseline／HostD1寫入epoch一致，來源角色／房間／qualified private Ready仍走原守門。\n\n'
 'Guest Ready1(d201)完整4byte写入，actual ownD1=1 revision5；觀察actual E103後立即typedCancel0(d200)，完整4byte寫入，再觀察actual E100與ownD1=0 revision6。三個counter各+2，write serial7/8/9。Host保持state1；actual E1事件只有3、0，沒有在結果中補造2/1。\n'
 f'E103_observed_us={r["E103_observed_us"]}，cancel_queued_us={r["cancel_queued_us"]}，本地觀察至enqueue差{delta}微秒；request queued_us={requests[2]["queued_us"]}是另一次本地記錄。這是同一Trace單調時鐘測量，不是server收到時間或ACK，不是一般延遲保證。既有來源9AE57C12保持E103→Cancel0無額外status讀檔／等待／sleep／timer。\n\n'
 '4秒實際Runtime pollwatch完成，first_failure／case_error皆null，Close成功，IO started2／joined2，queued0／late0。沒有owneropcode-only start、GUI、JavaSQL啟停、密碼輸出／複製／hash，也沒有重編production或重跑PASS。armed status的publication counters0/first_error null只描述該快照，不推論整段server無出版失敗。\n\n'
 '界限：actual D1是snapshot中的last receipt，不涵蓋所有中間包；revision／room epoch為local labels。E1沒有wire room/generation/sequence/time，0只清倒數；未證明sameepochfreshness／tickdrain／globalatomicGET／ACK／WORLD不可能。server兩端final wire/GET與helper/mysql退休由server組收尾，目前仍待該組交接，Modern不代為停止。\n\n'
 f'實際結果：{result_path}\n觀察封存：{E/"actual-observations.json"}\n機器交接：{L/"r528-status-io-real-backend-consumer-handoff.json"}\n',encoding='utf-8',newline='\n')
refs=[pin(p)for p in sorted(E.iterdir())if p.is_file()]
refs += [pin(result_path),pin(human),pin(P/'tools/run-r528-status-io-live-c9.ps1'),pin(P/'tools/seal-r528-status-io-live-c9.py'),pin(L/'r528-status-io-consumer-ready-handoff.json'),pin(L/'r528-status-io-candidate.json'),pin(L/'r528-status-io-provider-supply-request.json')]
refs += [claim['consumer'],pin(P/'target/r528-status-io-backend-consumer/r528-status-io-countdown-case.rs')]
for v in refs:assert pin(v['path'])==v
verification=L/'r528-status-io-real-backend-consumer-verification.json'
save(verification,dict(status='MODERN_ACTUAL_BOUNDED_CASE_PASS_SERVER_CLOSURE_PENDING',root_references_checked=refs,observations=pin(E/'actual-observations.json'),actual_result=pin(result_path),server_retirement_verified=False,GUI_accepted=False))
refs.append(pin(verification))
handoff=L/'r528-status-io-real-backend-consumer-handoff.json'
save(handoff,dict(schema='modern.r528.status-io.real-backend-consumer-handoff.v1',updated_at=now,state='MODERN_REAL_COUNTDOWN_CANCEL_CASE_PASS_AWAITING_SERVER_FINAL_WIRE_GET_AND_RETIREMENT',run_identity=RUN,root_references=refs,human=pin(human),actual_result=pin(result_path),observations=pin(E/'actual-observations.json'),verification=pin(verification),consumer=claim['consumer'],accepted_ready=claim['ready'],accepted_live_supply=claim['supply'],accounting=summary['accounting'],E103_to_Cancel_enqueue_delta_us=delta,server_stop_required=True,server_retirement_verified=False,server_lifecycle_owner='server group',limits=summary['limits'],overall_complete=False))
shared=P.parent/'監製進度.json';progress=read(shared)
progress['updated_at']=now;progress['phase']='R528_real_countdown_cancel_Modern_case_pass_server_wire_GET_retirement_pending';progress['overall_status']='in_progress'
progress['current_task']='R528新aa561供應nw-c9af：一次preflight＋一次connect PASS；baseline0/HostD11雙peer armed/Ready1fullwrite+D11/actualE103立即Cancel0fullwrite/actualE100+D10/4秒pollwatch完成。IO2/2 queue0 late0、GUI0；Modern結果已封存，等待server final wire/GET與退休，不代server啟停。'
r528=progress.setdefault('r528',{})
r528['cold_ready_handoff']=r528.get('handoff');r528['cold_ready_human']=r528.get('human');r528['cold_ready_verification']=r528.get('verification')
r528.update(state='MODERN_ACTUAL_CASE_PASS_AWAITING_SERVER_WIRE_GET_RETIREMENT',handoff=pin(handoff),human=pin(human),verification=pin(verification),actual_result=pin(result_path),observations=pin(E/'actual-observations.json'),fresh_run_id=RUN,fresh_READY_accepted=True,fresh_provider_execution_authorized=True,fresh_authorization_consumed=True,preflight_runs=1,connect_runs=1,actual_backend_executions=1,actual_case_success=True,actual_E103_observed=True,actual_E100_observed=True,E103_to_Cancel_enqueue_delta_us=delta,IO_started=2,IO_joined=2,queued_events=0,discarded_late_events=0,Close_ok=True,server_retirement_verified=False,server_stop_required=True,server_closure_pending=True,server_writes=0,JavaSQL=0,GUI=0,overall_completed=False,old_PASS_replays=0,awaiting_supervisor_consumer_ready_acceptance=False)
progress.setdefault('completed',[]).append('R528唯一fresh nw-c9af實際Modern案例PASS：1preflight/1connect、三次0/1/0完整4byte寫入、actualD1与E1=3/0、E103→Cancel本地enqueue154us、4秒pollwatch、Close IO2/2 queue0 late0；server最終wire/GET/退休待收尾，GUI0。')
with shared.open('w',encoding='utf-8',newline='\n')as f:json.dump(progress,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps(dict(handoff=pin(handoff),actual_result=pin(result_path),human=pin(human),observations=pin(E/'actual-observations.json'),accounting=summary['accounting'],server_closure_pending=True),ensure_ascii=False))
