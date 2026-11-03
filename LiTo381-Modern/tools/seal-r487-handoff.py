from pathlib import Path
import json,hashlib,datetime
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r487-menu-source-study'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):
 p=Path(p);h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(8<<20),b''):h.update(b)
 return dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');t.replace(p)
d=read(L/'r487-event-details.json');proof=read(L/'r487-independent-contract.json');player=read(L/'r487-player-candidate.json');c=read(L/'r487-candidate.json');scope=read(L/'r487-source-scope.json');findings=read(L/'r487-source-findings.json')
assert c['validation'].startswith('PASS')and read(L/'r487-consumer-exit.json')['exit_code']==0 and d['events']==53 and player['candidate_executions']==0
previous=pin(L/'r486-bounded-city563-handoff.json');assert previous['sha256']=='990A6A0042F20C5D43D2A259148BE15C446CEB16D4C9C99959501839D1EFEF5D'
history=dict(state='R487_ONE_FINITE_NEW_EVENT_CONSUMER_ATTEMPTS_RETAINED',production_library_builds=1,production_library_json=pin(L/'r487a-library-build.jsonl'),production_library_stderr=pin(L/'r487a-library-build.stderr.log'),player_build_attempts=1,player_candidate_executions=0,player_version_verified_by_execution=False,finite_consumer_build_attempts=1,finite_consumer_executions=1,finite_exit=pin(L/'r487-consumer-exit.json'),finite_seconds=d['focused_seconds'],preparation_parse_failures=1,failure=pin(L/'r487-preparation-failure.json'),failure_scope='One Python source syntax error before any preparation mutation;following shell invocations found absent scripts and did not compile. Frozen failed preparation source retained,parse repaired before generating candidate/build scripts. No product test failure and no failed Rust build.',historical_PASS_batches_rerun=0,full_test_suite_executed=False,additional_library_test_compile=False,GUI=0,LOADED=False)
save(L/'r487-execution-history.json',history)
remaining=['Thumb/track精確幾何生命週期：共用child64B41C的+8/+10/+24所有aliased寫入、active-panel reset和DD continuation須追溯，現有建構42與draw10/range41不能直接猜成拖曳規則。','word64ACCA原版選定mode寫入者與完整選擇/網路生命週期尚未本輪閉合；Modern combobox trigger仍是適配。','GUI/DPI/font/focus/select/hold/Back/close/normal network route實機驗收須另協調；本輪玩家EXE零次執行。','原版framebuffer/streaming insertion、spawn/network/mode、actors/effects/roof/admission仍待驗；40靜態CPU預覽不表示40完整可玩模式。']
gui=dict(state='R487_SOURCE_DERIVED_ARROW_CORE_AND_HEADLESS_LIFECYCLE_ONLY_GUI_PENDING',player_candidate_executions=0,GUI=0,LOADED=False,complete_modes=16,total_modes=40,static_CPU_previews=40,previous40_finite_acceptance='T325',Native_delay_numeric_values_proven=[700,125],milliseconds_interpretation='Clock1000 scaling and UI timing convention;imported backend not dynamically identified.',Modern_poll_timer_ms=40,wall_clock_jitter_or_original_scheduler_parity_verified=False,focus_captureloss_Back_mode_close_cancellation='Modern lifecycle safety policy;core finite events and actual worker cancellation verified,window routing statically compiled only.',thumb_display_only=True,track_geometry_unresolved=True,remaining_required_player_flow=remaining)
save(L/'r487-unverified-player-flow-checklist.json',gui)
tool_names=['prepare-r487-source-evidence.py','prepare-r487-events.py','prepare-r487-candidate.py','build-r487-player-candidate.ps1','r487-arrow-hold-consumer.rs','build-r487-arrow-consumer.ps1','audit-r487-source-scope.py','verify-r487-segment.py','seal-r487-handoff.py','verify-r487-sealed-links.py','evidence-json.ps1']
evidence_names=['r487-source-evidence.json','r487-scroll-base-source-evidence.json','r487-repeat-callback-source-evidence.json','r487-source-findings.json','r487-preparation-failure.json','r487-event-input.json','r487-entry-source-contract.json','r487-player-candidate.json','r487-consumer-candidate.json','r487-consumer-exit.json','r487-event-details.json','r487-production-source-snapshot.json','r487-source-scope.json','r487-independent-contract.json','r487-candidate.json','r487-execution-history.json','r487-unverified-player-flow-checklist.json']
h=dict(state='R487_FLAG_DIRECT_WRITER_PROVEN_ARROW_HOLD700_125_CORE_53EVENTS_LIFECYCLE_FINITE_PASS_GUI_PENDING',previous_R486_handoff=previous,previous40_static_CPU_finite_acceptance='T325',new_player_flow_review='PENDING',player=player['executable'],player_candidate_executions=0,player_executed_flags=[],production_library=player['production_library'],candidate_main=player['candidate_main'],original_main=player['original_main'],binding_unchanged_from_R486=player['binding'],skin_binding=player['skin_binding'],focused_consumer={k:c[k]for k in ['path','bytes','sha256']},source_findings=findings,implemented=dict(scope='Arrow hold source-derived core plus Modern lifecycle cancellation;no new launch flag policy or guessed thumb geometry.',hold_initial_delay_ms=700,hold_subsequent_delay_ms=125,no_missed_tick_catchup_loop=True,valid_release_action_after_repeat=True,pointer_exit_cancels_repeat=True,pointer_reenter_rearms_initial_delay=True,existing_boundary_disabled_policy_retained=True,focus_captureloss_Back_mode_close_cancel=True,UI_poll40ms_timing_adaptation=True,thumb_track_unimplemented=True,all40_previous_plans_resources_cache_decoder_flow_preserved=True),tests=d,source_scope=scope,all_source_snapshot=pin(L/'r487-production-source-snapshot.json'),independent_contract=proof,source_difference=pin(L/'r487-source-difference.patch'),execution_history=history,tools=[pin(P/'tools'/n)for n in tool_names],evidence=[pin(L/n)for n in evidence_names],build_and_run_logs=[pin(p)for p in sorted(L.glob('r487*.log'))],build_jsonl=[pin(p)for p in sorted(L.glob('r487*.jsonl'))],remaining_required_player_flow=remaining,unverified_GUI=gui,complete_modes=16,total_modes=40,static_CPU_previews=40,GUI=0,LOADED=False,root_main_written=False,R408_rebuilt_or_replaced=False,original_BIN_or_resources_written=False,Java_SQL_Server_used=False,provider_or_observer_used=False,R472_timers_advanced=False,full_original_UI_or_mode_acceptance=False)
path=L/'r487-native-arrow-hold-player-flow-handoff.json';assert not path.exists();save(path,h);hp=pin(path)
text=P/'Native-R487_原版箭頭長按與玩家流程交接.txt'
text.write_text(f'''R487：原版箭頭長按與玩家流程（離線事件已驗，GUI未啟動）

64AF9F兩個直接寫入者已追溯：fn_ClientApplication_Run的/lteventpsvr或-lteventpsvr、/lteventcsvr或-lteventcsvr分支設1；後者也設64AFA0。
46B7D0先比exact mode；非零排除547/548；零的mode3排除549/550及偶數ID。
這是啟動分支旗標，Modern離線ordinary0保持；没有新增伺服器/啟動參數切換。

原版箭頭repeat建構byte1->414BD0延遲700->41AA00每125，回呼477930/477990->474350/474360每次一列。
修正Modern長按；離開取消、回到箭頭重新等700、有效放開仍加一次、邊界不越界。
Back/模式/close清除pending；WM_KILLFOCUS/captureloss與window時間路由靜態編譯，沒有GUI輸入驗收。
原版延遲數值700/125來源已證實；ms依clock1000換算與慣例，匯入clock backend與實際jitter未知。
Modern既有40ms UI poll、一個到期回呼後從當下重新排程，以及u64 checked overflow均明列適配。

一個有限consumer：12組／53固定事件、3種縮放指標序列、3個actual worker生命週期；{d['focused_seconds']:.7f}秒PASS。
切模式／active563 load Back取消／close退場已驗，未產預覽frame；沒有重跑歷史PASS批次或全套測試。
固定166consumer來源對與全部602production來源快照；只改menu/runtime/window三個來源。
保留一次準備Python語法失敗，發生在候選產出前；Rust library/player/consumer各一次編譯成功。

Thumb/track來源的state1/2/3、graboffset、capture、page8及inverse公式已固定。
但共享child的建構rect566,323,579,365／height42，與draw IMG10／range41及inverse height-10仍缺完整active reset/aliased write鏈。
沒有猜測拖曳幾何；thumb仍display-only、track未實作，下一段須追這個確切缺口。
word64ACCA mode writer與GUI/fullgame規則也未本輪閉合。

R486/T325已有限接受40靜態CPU預覽；16/40完整模式、GUI0、LOADEDfalse維持。
本輪玩家候選零次執行，連--version也未跑，未競爭原版3366崩潰調查的使用者輸入。
候選：{player['executable']['path']}
SHA256：{player['executable']['sha256']}
交接：{path}
SHA256：{hp['sha256']}
R486 handoff SHA256：{previous['sha256']}
''',encoding='utf-8')
progress_path=P.parent/'監製進度.json';progress=read(progress_path);progress['updated_at']=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat();progress['phase']='R487_native_flag_writer_arrow_hold53events_lifecycle_finite_pass_new_flow_review_pending_GUI0';progress['current_task']='R487 sealed:64AF9Ftwo direct launch-token writers and46B7D0mode filter proven;source-derived arrow700/125 hold/reenter/release implemented,Modern lifecycle cancel.12cases53events/3scaled/3actualworker lifecycle onefinitePASS;166consumer+602production source snapshots.Thumb/track state machine proven butshared42-vs10/range41geometry lifecycle unresolved,no guessed drag.40staticCPU T325finiteaccepted;16of40/GUI0/LOADEDfalse. Newplayer0executions.'
progress['r486_T325_finite_acceptance']=dict(handoff=previous,accepted_static_CPU_previews=40,map563_complete_sectors=19,qualified_slots=31444,cache_limit=16777216,CPU_reference_pairs=54,scope='offline finite source/qualification/cache/reference/cancel/eviction only',complete_modes=16,GUI=0,LOADED=False)
if 'r486'in progress:progress['r486']['new563_CPU_review']='T325_FINITE_OFFLINE_ACCEPTED'
progress['r487']=dict(handoff=hp,text=pin(text),player=player['executable'],player_candidate_executions=0,focused_consumer=h['focused_consumer'],source_scope=pin(L/'r487-source-scope.json'),independent_contract=pin(L/'r487-independent-contract.json'),events=53,source_fragments=46,flag_named_direct_writers=2,new_player_flow_review='PENDING',remaining_thumb_track='exact shared child64B41C active geometry reset/aliased write chain',static_CPU_previews=40,complete_modes=16,GUI=0,LOADED=False)
progress.setdefault('completed',[]).append('R487：64AF9F直接啟動旗標寫入者與mode篩選閉合；原版箭頭700/125長按、離開/返回/放開及Modern取消實作；12組53事件/3scaled/3actualworker生命週期有限PASS；thumb/track幾何缺口明列、未猜測，玩家EXE0次執行，GUI0。')
save(progress_path,progress);save(L/'r487-final-audit.json',dict(state='R487_SEALED_PROGRESS_UPDATED_FINITE_NEW_PLAYER_FLOW_REVIEW_PENDING_GUI0',handoff=hp,text=pin(text),progress=pin(progress_path),independent_contract=pin(L/'r487-independent-contract.json'),complete_modes=16,GUI=0,LOADED=False));print('R487 sealed',hp['sha256'],'53events/GUI0/player executions0')
