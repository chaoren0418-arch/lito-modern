import pathlib,json,hashlib,shutil,difflib,re,datetime,collections
P=pathlib.Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';T=P/'target/r529-player-entry-candidate'
def read(p):return json.loads(pathlib.Path(p).read_text('utf-8-sig'))
def pin(p):
 p=pathlib.Path(p);raw=p.read_bytes();return dict(path=str(p),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest().upper())
def save(p,v):
 with pathlib.Path(p).open('x',encoding='utf-8',newline='\n')as f:json.dump(v,f,ensure_ascii=False,indent=2);f.write('\n')
now=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
candidate=read(L/'r529-player-candidate.json');entry=read(L/'r529-entry-source-contract.json')
for key in ['executable','production_library','candidate_main','original_main','binding','skin_binding','entry_source_contract']:
 v=candidate[key];assert pin(v['path'])==v
assert candidate['candidate_executions']==candidate['GUI']==0
accounts={}
for label in ['library-build','targeted-tests','player-build']:
 a=read(L/f'r529-{label}-accounting.json');assert a['attempts']==1 and a['exit_code']==0;accounts[label]=a
out=(L/'r529-targeted-tests.stdout.log').read_text('utf-8-sig')
assert '4 passed; 0 failed' in out
filtered=int(re.search(r'(\d+) filtered out',out)[1])
assert not (L/'r529-player-build.stderr.log').read_text('utf-8-sig').strip()
def warnings(p):
 rows=[]
 for s in pathlib.Path(p).read_text('utf-8-sig').splitlines():
  try:v=json.loads(s)
  except json.JSONDecodeError:continue
  if v.get('reason')=='compiler-message'and v['message']['level']=='warning':rows.append(v['message'])
 return rows
new=warnings(L/'r529-library-build.jsonl');old=warnings(L/'r527-library-build.jsonl')
signature=lambda w:(w.get('code',{}).get('code')if w.get('code')else None,w['message'])
existing=collections.Counter(map(signature,old));additional=[]
for w in new:
 k=signature(w)
 if existing[k]:existing[k]-=1
 else:additional.append(w)
assert not additional,additional
testerr=(L/'r529-targeted-tests.stderr.log').read_text('utf-8-sig')
test_warnings=int(re.search(r'\(lib test\) generated (\d+) warnings',testerr)[1]);assert test_warnings==92
new_test_warning_names=['display_uses_last_D1_not_initial_or_queued_intent','countdown_displays_received_BYTE_and_clears_for_all_retired_contexts','unavailable_callbacks_and_Close_use_actual_Runtime_guards']
assert all(f'warning: function `{name}` should have a snake case name'in testerr for name in new_test_warning_names)
test_path=P/re.search(r'Running unittests .*? \(([^)]+)\)',testerr)[1]
frozen_test=T/'r529-new-targeted-tests.exe';shutil.copyfile(test_path,frozen_test)
test_binary=pin(frozen_test)
save(L/'r529-build-warning-accounting.json',dict(previous_regular_warnings=len(old),new_regular_warnings=len(new),new_warning_signatures=len(additional),player_warnings=0,test_harness_warnings=92,historical_test_harness_warnings=89,new_test_harness_warnings=3,new_test_warning_names=new_test_warning_names,test_warning_scope='Test function naming only; first passing run retained, no functional/test replays for cosmetic warning cleanup.',frozen_executed_test_binary=test_binary,new_targeted_tests=4,old_tests_filtered=filtered,old_tests_executed=0,initial_accounts=accounts))
old_snapshot=read(L/'r527-production-source-snapshot.json');old_by_rel={str(pathlib.Path(v['path']).relative_to(P)):v for v in old_snapshot['source_pins']}
assert len(old_by_rel)==639
changes=[];source_pins=[];frozen_old=[];diff='';source_root=T/'source';source_root.mkdir()
for source in sorted((P/'src').rglob('*.rs')):
 rel=source.relative_to(P);current=pin(source);previous=old_by_rel.get(str(rel))
 if previous:
  frozen=pin(previous['snapshot']);assert frozen['bytes']==previous['bytes']and frozen['sha256']==previous['sha256'];frozen_old.append(frozen)
  if current['sha256']!=previous['sha256']:
   changes.append(dict(kind='modified',relative_path=str(rel),before={k:previous[k]for k in ['path','bytes','sha256']},after=current))
   before=pathlib.Path(previous['snapshot']).read_text('utf-8')
  else:before=None
 else:
  changes.append(dict(kind='added',relative_path=str(rel),after=current));before=''
 if before is not None:diff+=''.join(difflib.unified_diff(before.splitlines(True),source.read_text('utf-8').splitlines(True),fromfile='R527-frozen/'+str(rel),tofile='R529/'+str(rel)))
 dest=source_root/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,dest);source_pins.append(dict(**current,snapshot=str(dest)))
assert len(source_pins)==640 and len(changes)==3
assert {c['relative_path']for c in changes}=={'src\\modern_lito\\native_city_runtime.rs','src\\modern_lito\\native_city_window.rs','src\\modern_lito\\native_java_room_player.rs'}
(L/'r529-production-source-difference.patch').write_text(diff,'utf-8',newline='\n')
save(L/'r529-production-source-snapshot.json',dict(source_count=640,source_pins=source_pins,changes=changes,preserved_old_source_count=639,frozen_old_pins=frozen_old,difference=pin(L/'r529-production-source-difference.patch'),original_main_unchanged=pin(P/'src/main.rs'),production_library=candidate['production_library']))
preserved=[]
for name,expected in [
 ('r528-status-io-real-backend-consumer-handoff.json','BC0763BC30249A6FD216F729C85B6D812147B96C52069123F0A8BC0868F5E597'),
 ('r528-status-io-consumer-ready-handoff.json','7F85409CE66C261833BB145E32214C7273F2B52C26B8BA94CC7F887E0060F4E9'),
 ('r528-countdown-cancel-consumer-ready-handoff.json','28BC666D4512DA1BFF5D9584BD1C4C7A9F85FC7DDF02F13C5648B2C6F84B9E9F')]:
 v=pin(L/name);assert v['sha256']==expected;preserved.append(v)
for name in ['r528-final-candidate.json','r528-status-io-candidate.json']:
 v=read(L/name)
 for key in ['executable','source','countdown_module','provider_module','production_library']:
  if key in v:
   original=v[key];assert pin(original['path'])==original;preserved.append(original)
result=pin(L/'r528-backend-nw-c9afae2f2cd14da08208599b1bf066fb/result.json');assert result['sha256']=='73DF75B10A8FD259A3991B358C7D5BD4C026AF9461D3096B5A0A6C28FCBD4718';preserved.append(result)
save(L/'r529-prior-evidence-preservation.json',dict(status='AB3_FAD83_28BC_7F854_BC0763_73DF_AND639_FROZEN_PRESERVED',pins=preserved,production_original_sources_modified=2,production_new_source=1,production_total_changes=3,old_library_rebuilt=False))
window=(P/'src/modern_lito/native_city_window.rs').read_text('utf-8');module=(P/'src/modern_lito/native_java_room_player.rs').read_text('utf-8');main=pathlib.Path(entry['candidate_main']['path']).read_text('utf-8')
assert '"--candidate-java-bootstrap"'in main and 'native_city_window::run_synthetic_java_bootstrap' in main
assert 'runtime.configure_login_producer_fixture(adapter)?;room_player::configure(&mut runtime)?;'in window
assert 'ROOM_READY=>Some(RoomOperation::Ready)'in window and 'ROOM_CANCEL=>Some(RoomOperation::Cancel)'in window
assert 'let result=panel.invoke(s.runtime.borrow_mut().as_mut().unwrap(),operation);'in window
assert 'r.handle_java_room_ready_control(token)'in module
assert 'java_room_ready_control(ready::ParticipantIntent::Ready)'in module and 'java_room_ready_control(ready::ParticipantIntent::NotReady)'in module
assert 'request_java_room_owner_start'not in module and 'prepare_participant_ready'not in module
mapping=[
 dict(control='已收到的公開房間清單／加入房間',callback='WM_COMMAND ROOM_LIST selection + ROOM_JOIN BN_CLICKED -> RoomOperation::Join(slot)',qualification='Capture existing private java_room_entry_control(Join) for current observed CC0 public row, then verify current catalog epoch/channel/admission/revision/ID before invoke',typed_action='handle_java_room_entry_control -> existing ordinary C_CF Join envelope',display_effect='No optimistic room change; wait accepted ordinary CE',limitations='Possibly incomplete CC0 list, no protected-room password editor; no room creation'),
 dict(control='準備',callback='WM_COMMAND ROOM_READY BN_CLICKED -> room_operation -> Panel::invoke(Operation::Ready)',qualification='Existing private java_room_ready_control(ParticipantIntent::Ready), re-admitted by Runtime',typed_action='handle_java_room_ready_control -> existing C_D2 [210,1], frame4',display_effect='Keep last actual D1; queued/written intent only appears in existing status',limitations='No owner/start/WORLD grant'),
 dict(control='取消準備',callback='WM_COMMAND ROOM_CANCEL BN_CLICKED -> room_operation -> Panel::invoke(Operation::Cancel)',qualification='Existing private java_room_ready_control(ParticipantIntent::NotReady), re-admitted by Runtime',typed_action='handle_java_room_ready_control -> existing C_D2 [210,0], frame4',display_effect='D1 changes ready state; E100 independently clears countdown',limitations='No automatic timed cancellation or retry'),
 dict(control='倒數',callback='Existing40ms WM_TIMER Runtime.poll -> presentation Gate -> RoomPanel::capture -> ROOM_COUNTDOWN static label',qualification='Current admitted CE/settings scope, same epoch/room/joined context, not invalidated',typed_action='No outgoing request; read existing actual E1 snapshot',display_effect='Show received unsignedBYTE literally; count None/0 clears label; no local timer',limitations='No sameepoch wire freshness/generation/tickdrain guarantee'),
 dict(control='來源／離房失效／Close',callback='Existing Runtime retirement/invalidation -> changed presentation key -> capture clears panel; late invoke delegated to existing private control admission',qualification='Runtime private epoch/context guard unchanged',typed_action='Existing Close/modal/logout only; no new BackRoom/committed leave route',display_effect='Clear old panel/countdown for missing admitted context, Closing/Closed',limitations='Committed leave scene/back-to-lobby remains unavailable; no Native modal owner factory change'),
 dict(control='重新取得清單',callback='WM_COMMAND ROOM_REFRESH -> existing private RefreshList control',qualification='Existing refused-room qualification only',typed_action='handle_java_room_entry_control(RefreshList)',display_effect='Wait real current list receipts; no reset/upsert completion inference',limitations='No refresh from arbitrary unqualified lobby state')]
save(L/'r529-UI-typed-callback-mapping.json',dict(schema='modern.r529.room-ui-typed-callback-mapping.v1',actual_entry='candidate main Route::CandidateJavaBootstrap -> candidate_java_entry -> native_city_window::run_synthetic_java_bootstrap -> run_inner -> RoomPlayer.configure',predecessor_scope='R526 candidate entry/flag existed, actual native_city_window only login adapter and channel/lobby controls; room_window.rs belongs to different legacy path and was not used as evidence of completion.',mapping=mapping,assets='Existing Lineage map/skin bindings and existing standard Modern/GDI controls/font; zero new images, no provider-to-file inference or BIN factory access',GUI_executions=0,qualified_positive_UI_callback_executed=False,new_CPU_tests=4,old_tests_filtered=filtered))
remaining=[
 'GUI0: do not use Win32 input/screenshot injection to bypass unavailable native computer-use surface.',
 'Build is launchable through the preserved explicit --candidate-java-bootstrap flag with absolute compatible SYNTHETIC fixture profile and pinned AD368 bootstrap adapter; no current live profile/provider is created or started.',
 'Real485F GUI supply still requires separate supervisor authorization, a fresh accepted READY/lease/exact ownership check and explicit Backend485F/UI entry integration. Existing flag consumes FixtureProfile(SYNTHETIC), not Backend485fProfile; do not mislabel retired2B45 or arbitrary endpoint as accepted backend.',
 'In a supported native GUI session, verify login/source/news/actual singleton roster/channel -> observed public room selection -> Join acceptedCE -> private Ready/Cancel buttons, keyboard/tab activation, labels/DPI/focus and E1 literal values/0 clearing.',
 'Verify qualified positive callbacks/full actual writes/D1 updates through the new window and stale held controls after actual room deletion/departure/source switch/Close; this round checks projection and unavailable callbacks, not qualified positive GUI clicks.',
 'No room creation/password-join/editor or committed BackRoom/leave transition added; keep existing refusals and qualified Close/logout route.',
 'No new provider/SQL/socket/live case, old PASS replay or stress test; GUI/ACK/tickdrain/WORLD exclusion/native side effects remain unproven.'
]
save(L/'r529-remaining-acceptance-steps.json',dict(status='COMPILED_CANDIDATE_GUI_AND_POSITIVE_WINDOW_CALLBACK_ACCEPTANCE_PENDING',steps=remaining,example_launch=[candidate['executable']['path'],'--candidate-java-bootstrap','<absolute compatible explicit SYNTHETIC profile>','<absolute pinned AD368 bootstrap adapter>'],executable=candidate['executable'],authorized_now=False))
human=P/'Native-R529_房間準備取消介面候選交接.txt'
human.write_text('R529 房間準備／取消與倒數顯示候選\n\n'
 '已沿實際 --candidate-java-bootstrap 入口確認缺口：R526 candidate main→candidate_java_entry→native_city_window::run_synthetic_java_bootstrap→run_inner，原本只設定登入adapter，無房間加入／Ready／取消／E1呈現；其他room_window並非此路徑。\n\n'
 '這次補在同一既有視窗：顯示已收到的公開房間、加入房間、準備、取消準備、參與者D1狀態与E1倒數label；重新取得清單僅依原refused-room private control。初始化只在顯式Java-bootstrap候選設定既有pinned room/participant/self/catalog/settings/ready adapters，保留AD368 layout與485F writer來源標籤。\n\n'
 '按鈕由目前private Control決定是否可用，callback把快照token交原Runtime重新核epoch／revision／joined/context，不新增送包controller或opcode-only start。準備／取消只送現有typed Ready1／NotReady0（body2/frame4）。準備文字取last actualD1，不以送出或完成寫入改成已準備；E1只顯示收到的BYTE，0清文字，沒有本機計時／WORLD推演。失去current admitted joined scope、房間失效、換來源、Close即停止顯示舊倒數。\n\n'
 f'可啟動成品：{candidate["executable"]["path"]}\nSHA256：{candidate["executable"]["sha256"]}\n候選main SHA256：{candidate["candidate_main"]["sha256"]}\n新的regular library SHA256：{candidate["production_library"]["sha256"]}\n'
 '入口：--candidate-java-bootstrap <絕對SYNTHETIC loopback fixture profile> <絕對pinned AD368 bootstrap adapter>。只編譯，未啟動成品或GUI；這個既有旗標仍不是Backend485F真供應入口，不能拿退休READY或任意真endpoint冒稱合格。真實485F GUI仍需下一步的fresh監製供應與明確backend/UI entry接線。\n\n'
 f'有限新檢查4項首次PASS，舊{filtered}項filtered；新regular library warning signatures0、player warning0（regular歷史warning數{len(new)}）。test harness92warnings=89歷史＋3新測試名稱snake_case警告，明列保留而不重跑PASS。檢查lastD1與初始/queued意圖分離、歧義alias/缺D1保持未知、E1原BYTE/0及epoch/room/CE/nav失效投影、actual CPU Runtime未合格callback拒絕與Close。沒有執行qualified positive GUI callback，不把pure projection資料當server線上證據。\n\n'
 'production有3處變更：native_city_runtime.rs加module、native_city_window.rs接現有控制、新native_java_room_player.rs；總來源640。新source snapshot／diff／library另立，不宣稱仍零變更。639份舊frozen來源、AB3與FAD83 executable、5C9舊library、R528封存與73DF實際結果均保持原樣。原src/main.rs未改，default normal profile路徑不變。\n\n'
 '沿用原合格Lineage map／skin資源與既有Modern/GDI字體控制，無新替代圖資、provider-to-file推論或BIN私有factory解封。密碼房間加入、建立房間、committed BackRoom／離房返回仍不提供；來源已有守門與Close不解封。\n\n'
 'R528新接收紀錄另記server 57FFD604 closure與供應B5F91741→退休862DE4D9索引，BC0763原封存不改；監製已接受雙端及退休，nw-c9af／READY2B45禁止重用。server寫D1zero先於E100，Modern E100事件早於participant snapshot102us不代表網路重排；4秒為有限poll與後續離房GET，不是連續4秒GET檔案。\n\n'
 '剩餘驗收：在支援原生視窗的工具環境檢查實際滑鼠／鍵盤Ready/取消、完整write／D1／E1文字、焦點/DPI/Tab，以及實際離房失效／換來源／Close後舊control與文字退役。GUI0，沒有Win32輸入/截圖繞過，沒有新provider／SQL／socket、舊live案例或壓力測試。\n\n'
 '機器交接：logs/continuation-20261003/r529-room-ui-candidate-handoff.json\ncallback對照：logs/continuation-20261003/r529-UI-typed-callback-mapping.json\n剩餘步驟：logs/continuation-20261003/r529-remaining-acceptance-steps.json\n',encoding='utf-8',newline='\n')
refs=[pin(human),pin(L/'r529-player-candidate.json'),pin(L/'r529-entry-source-contract.json'),pin(L/'r529-production-source-snapshot.json'),pin(L/'r529-production-source-difference.patch'),pin(L/'r529-prior-evidence-preservation.json'),pin(L/'r529-build-warning-accounting.json'),pin(L/'r529-UI-typed-callback-mapping.json'),pin(L/'r529-remaining-acceptance-steps.json'),pin(L/'r529-R528-server-closure-reception.json')]
refs += [pin(p)for p in sorted((L/'r529-R528-server-closure-reception').iterdir())]
for label in ['library-build','targeted-tests','player-build']:
 refs += [pin(L/f'r529-{label}-accounting.json'),pin(L/f'r529-{label}.stderr.log')]
refs += [pin(L/'r529-library-build.jsonl'),pin(L/'r529-targeted-tests.stdout.log'),pin(L/'r529-player-build.stdout.log')]
refs += [pin(P/'tools'/name)for name in ['prepare-r529-room-ui.py','build-r529-room-ui.ps1','seal-r529-room-ui.py']]
refs += [candidate['executable'],candidate['production_library'],candidate['candidate_main'],candidate['binding'],candidate['skin_binding'],*preserved[:3]]
refs.append(test_binary)
for v in refs:assert pin(v['path'])==v
verification=L/'r529-room-ui-candidate-verification.json'
save(verification,dict(status='CANDIDATE_BUILD_AND_NEW_CPU_BINDING_CHECKS_PASS_GUI_UNVERIFIED',root_references_checked=refs,source_count=640,production_changes=changes,new_tests_passed=4,old_tests_filtered=filtered,new_regular_warning_signatures=0,player_warnings=0,new_test_naming_warnings=3,GUI=0,qualified_positive_UI_callback_executed=False))
refs.append(pin(verification))
handoff=L/'r529-room-ui-candidate-handoff.json'
save(handoff,dict(schema='modern.r529.room-ui-candidate-handoff.v1',updated_at=now,state='ROOM_UI_TYPED_CALLBACK_CANDIDATE_COMPILED_NEW_CPU_CHECKS_PASS_GUI_ACCEPTANCE_PENDING',root_references=refs,candidate=candidate,human=pin(human),verification=pin(verification),callback_mapping=pin(L/'r529-UI-typed-callback-mapping.json'),remaining_steps=pin(L/'r529-remaining-acceptance-steps.json'),production_snapshot=pin(L/'r529-production-source-snapshot.json'),production_difference=pin(L/'r529-production-source-difference.patch'),R528_server_retirement_reception=pin(L/'r529-R528-server-closure-reception.json'),new_tests=4,old_tests_filtered=filtered,new_test_naming_warnings=3,production_changes=3,production_source_count=640,previous_frozen_source_count=639,GUI=0,new_provider_runs=0,old_live_case_replays=0,qualified_positive_window_callback_acceptance=False,overall_complete=False))
shared=P.parent/'監製進度.json';progress=read(shared);progress['updated_at']=now;progress['phase']='R529_room_UI_candidate_compiled_GUI_positive_callback_acceptance_pending';progress['overall_status']='in_progress'
progress['current_task']='R529實際Java-bootstrap→native_city_window缺口已接：公開room Join/private Ready1/NotReady0、lastD1準備狀態/E1數值/0清除／失效退役。3production變更／640sources另封，4新CPUchecks PASS、GUI0；合格真BackendUI入口與positive視窗操作驗收待續。R528双端與server退休57FFD604已接收，nw-c9af/2B45不可重用。'
r528=progress.setdefault('r528',{});r528.update(server_retirement_verified=True,server_closure_pending=False,server_stop_required=False,server_closure_reception=pin(L/'r529-R528-server-closure-reception.json'),fresh_provider_execution_authorized=False,fresh_READY_retired=True,fresh_READY_reuse_prohibited=True,state='BOUNDED_R528_BOTH_PEERS_AND_SERVER_RETIREMENT_ACCEPTED_GUI_UNVERIFIED')
progress['r529']=dict(state='COMPILED_ROOM_UI_CANDIDATE_GUI_PENDING',handoff=pin(handoff),human=pin(human),candidate=pin(L/'r529-player-candidate.json'),verification=pin(verification),production_snapshot=pin(L/'r529-production-source-snapshot.json'),production_changes=3,production_source_count=640,old_frozen_sources=639,new_tests_passed=4,old_tests_filtered=filtered,new_regular_warning_signatures=0,player_warnings=0,new_test_naming_warnings=3,GUI=0,actual_candidate_executions=0,new_provider_runs=0,JavaSQL=0,network=0,server_writes=0,qualified_positive_window_callback_acceptance=False,real_backend_GUI_entry_pending=True,overall_complete=False)
progress.setdefault('completed',[]).append('R529 roomUI新候選：沿實際Java-bootstrap入口接public Join/private Ready/取消/D1狀態/E1原值，3production變更640sources另封；4新CPUchecks PASS/舊filtered、新warning0/player0。GUI與真backendUI入口/positive操作待續；R528雙端/退休57FFD604新接收，不改BC0763。')
with shared.open('w',encoding='utf-8',newline='\n')as f:json.dump(progress,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps(dict(handoff=pin(handoff),executable=candidate['executable'],human=pin(human),new_tests=4,old_tests_filtered=filtered,production_changes=3,GUI=0,remaining_acceptance=pin(L/'r529-remaining-acceptance-steps.json')),ensure_ascii=False))
