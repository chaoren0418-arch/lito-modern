"""Record only a verified live driver; retain frozen production/QA closure."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,zipfile,time
ROOT=Path(__file__).resolve().parents[1]
def read(p):
    for i in range(10):
        try:return json.loads(p.read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--library-run',type=Path,required=True);ap.add_argument('--library-session',type=int,required=True);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native1030-default111')
b=read(run/'cohort.json');assert b['status']=='independently_qualified_current_default111_both_profiles' and b['owned_process'] is None
debug=b['profiles']['Debug'];dq=read(Path(debug['qualification']));assert debug['qualified'] and sha(Path(debug['qualification']))==debug['qualification_sha256'] and dq['counts']['failed']==0 and dq['actual_test_executables']==112
c=dict(profile='Release',source_manifest_sha256=b['source_manifest_sha256']);driver=dict(pid=b['pipeline_pid'],creation_utc=b['pipeline_creation_utc'])
completion_path=run/'completion-observation.json';completion=read(completion_path)
assert completion['cohort_sha256']==sha(run/'cohort.json') and completion['recorded_original_owned_processes_all_terminal'] and completion['observed_terminal_exit_code']==0
for profile in ['Debug','Release']:
    info=b['profiles'][profile];q=read(Path(info['qualification']));assert info['qualified'] and sha(Path(info['qualification']))==info['qualification_sha256'] and q['counts']['failed']==0 and q['actual_test_executables']==112
library=a.library_run.resolve(strict=True);assert library.is_relative_to(ROOT/'驗證紀錄/current/native1030-library-default')
lc=read(library/'cohort.json');assert lc['source_manifest_sha256']==c['source_manifest_sha256'] and lc['status'] in ['prepared_current_full_library_default_both_profiles','compiling_current_full_library_test_image','running_current_full_library_default_selection']
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.CloseHandle.argtypes=[wintypes.HANDLE];k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
handle=k.OpenProcess(0x100000|0x1000,False,lc['pipeline_pid']);assert handle
try:
    assert k.WaitForSingleObject(handle,0)==258
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    created=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10);assert abs((created-datetime.fromisoformat(lc['pipeline_creation_utc'])).total_seconds())<.001
finally:k.CloseHandle(handle)
library_record=dict(run=str(library),original_session_id=a.library_session,driver=dict(pid=lc['pipeline_pid'],creation_utc=lc['pipeline_creation_utc']),source_manifest_sha256=lc['source_manifest_sha256'],verified_live_utc=datetime.now(timezone.utc).isoformat(),full_library_qualification_pending=True)
dq_library_path=library/'Debug/full-default-qualified/qualified.json';dq_library=read(dq_library_path)
assert dq_library['profile']=='Debug' and dq_library['source_manifest_sha256']==c['source_manifest_sha256'] and dq_library['passed']==1943 and dq_library['failed']==0 and dq_library['ignored']==483
assert dq_library['restored_historical_reports']==1719 and dq_library['recorded_profile_owned_processes_terminal']
library_record.update(profiles={'Debug':dict(qualification=str(dq_library_path),qualification_sha256=sha(dq_library_path),passed=1943,failed=0,ignored=483,restored_historical_reports=1719)},Release_qualification_pending=True,full_library_both_profiles_qualified=False,GUI_verified=False)
ui_path=ROOT/'驗證紀錄/current/native1030-ui-capture-startup/20261017-235558-675385/observation.json';ui=read(ui_path)
assert ui['screenshots_obtained']==0 and ui['native_config_Escape_cancel_observed'] and not ui['GUI_acceptance']
ui_record=dict(path=str(ui_path),sha256=sha(ui_path),native_config_accessibility_observed=True,native_config_Escape_cancel_observed=True,capture_errors=[ui['first_capture_error'],ui['retry_capture_error']],GUI_verified=False,physical_game_input_verified=False)
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=run/('checkpoint-library-running-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(run=str(run),original_session_id=a.session,source_manifest_sha256=c['source_manifest_sha256'],original_driver=driver,completion=str(completion_path),completion_sha256=sha(completion_path),recorded_original_owned_identities=completion['recorded_original_owned_identities'],actual_terminal_exit_code=0,qualification_pending=False)
work='來源'+c['source_manifest_sha256'][:8]+'之原Debug10192已actual exit0，112實際映像／111整合目標加main，獨立逐名稱／raw／來源／映像讀回'+str(dq['counts']['passed'])+'過、0敗、'+str(dq['counts']['ignored'])+'既有忽略。這是default整合範圍，未執行忽略本文、完整library或現場GUI。Release沿原'+str(a.session)+'續跑，未重跑Debug；兩主與1031編譯輸入維持釘選，產品及對應QA凍結。既有G06控制器96、非同步跨端點成功12／失敗84、重新登入音效16、同連線WORLD成功8／拒絕8sessions維持各自資格。完整library、必要資源、正常GUI／實體輸入音效、C72來源音效、同端點成功及完整G06／G01–G12仍待；Windows工具失敗不列通過。原時限、CRLF來源識別、Release設定不變，登入器與素材唯讀且產物獨立，寫入限重建版。'

audit_path=ROOT/'驗證紀錄/current/native1030-learned-request-audit/20261017-222459-870750/audit.json'
audit=read(audit_path);assert audit['source_manifest_sha256']==c['source_manifest_sha256'] and audit['observed_learned_ids']==74 and audit['recognized_learned_requests']==33 and audit['missing_learned_requests']==41 and audit['new_casts']==0 and not audit['product_source_modified']
binding_path=ROOT/'驗證紀錄/current/native-class2-call-lightning/20261017-223423-844766/binding.json'
binding=read(binding_path);assert not binding['renderer_qualified'] and not binding['product_source_modified'] and binding['formal_skill34']['effectGfx']==11736
assert sha(Path(binding['native_disintegrate']['path']))==binding['native_disintegrate']['sha256']
static_audit=dict(path=str(audit_path),sha256=sha(audit_path),source_manifest_sha256=c['source_manifest_sha256'],learned=74,request_recognized=33,missing_requests=41,new_casts=0,gameplay_coverage_inferred=False)
class2=dict(path=str(binding_path),sha256=sha(binding_path),ambient_supplement=str(binding_path.parent/'ambient-context-supplement.json'),byte_predicates=7,target_position_resolved_at_virtual_entry=True,refresh_cadence_verified=False,render_environment_callback_integration_pending=True,renderer_qualified=False,new_casts=0)
work+=' 當版實際技能書與請求程式已靜態重核74種／辨識33種／缺41種；不代表施法通過。技能34的原始class2目標追蹤、共享狀態及現有環境callback未接線已有PE位元組證據，不能直接沿用固定落點的77渲染器；尚未套用或增加技能實測資格。'
ambient_paths=[ROOT/'驗證紀錄/current/native-class2-ambient/run-20261017-225645-032014/qualified-20261017-225732-969519/qualified.json',ROOT/'驗證紀錄/current/native-class2-ambient/run-20261017-225739-299242/qualified-20261017-225750-597913/qualified.json']
ambient={}
for path in ambient_paths:
    q=read(path);assert q['status']=='qualified_isolated_ambient_ledger_only' and q['passed']==10 and q['failed']==0 and q['source_manifest_sha256']==c['source_manifest_sha256'] and not q['renderer_verified']
    ambient[q['profile']]=dict(path=str(path),sha256=sha(path),passed=10,candidate_module_sha256=q['candidate_module_sha256'])
assert set(ambient)=={'Debug','Release'} and ambient['Debug']['candidate_module_sha256']==ambient['Release']['candidate_module_sha256']
entry_path=ROOT/'驗證紀錄/current/native-class2-entry-order/20261017-225903-318354/binding.json';entry=read(entry_path);assert not entry['renderer_qualified']
ambient_record=dict(profiles=ambient,scope='isolated environment lease lifecycle only',actual_sessions=0,real_casts_verified=False,renderer_verified=False,product_registered=False)
entry_record=dict(path=str(entry_path),sha256=sha(entry_path),first_callback_frame=1,transient_source0_compositor_visibility_qualified=False,product_modified=False)
work+=' 亮度覆寫候選兩版各10項生命週期測試經raw／來源／PE讀回，重疊還原、跨場景與連線、重複回呼及有界歷史通過；只列候選元件，未註冊產品或增加施法／渲染資格。原版actor marker→class2虛擬入口→來源0推進1的證據已封存；首次實際合成可見性仍待。canonical11736資源載入候選準備中。'
live_paths=[ROOT/'驗證紀錄/current/native-class2-live/20261017-232124-047352/qualified-20261017-232351-286825/qualified.json',ROOT/'驗證紀錄/current/native-class2-live/20261017-232245-549554/qualified-20261017-232406-293868/qualified.json']
live={}
for path in live_paths:
    q=read(path);assert q['status']=='independently_qualified_class2_C02_candidate_controller_presentation_scope' and q['base_product_source_manifest_sha256']==c['source_manifest_sha256'] and q['recorded_owned_all_terminal'] and not q['GUI_verified'] and not q['normal_product_registered']
    pictures=path.parent.parent/'picture-readback.json';picture=read(pictures);assert len(picture['verified_all_pictures'])==(5 if q['profile']=='disintegrate' else 3)
    live[q['profile']]=dict(path=str(path),sha256=sha(path),accepted=q['actual_accepted_casts'],sessions=q['actual_sessions'],requests=q['actual_requests'],picture_readback=str(pictures),picture_readback_sha256=sha(pictures))
assert live['disintegrate']['accepted']==2 and live['disintegrate-refused']['accepted']==0
bridge_paths=[ROOT/'驗證紀錄/current/native-class2-bridge/run-20261017-231123-443925/qualified-20261017-231157-985109/qualified.json',ROOT/'驗證紀錄/current/native-class2-bridge/run-20261017-231248-371327/qualified-20261017-231258-225931/qualified.json']
bridges={}
for path in bridge_paths:
    q=read(path);assert q['passed']==8 and q['failed']==0 and not q['renderer_verified'];bridges[q['profile']]=dict(path=str(path),sha256=sha(path),passed=8)
resource_paths=[ROOT/'驗證紀錄/current/native-class2-presentation/run-20261017-230702-368925/qualified-20261017-230729-481084/qualified.json',ROOT/'驗證紀錄/current/native-class2-presentation/run-20261017-231000-502537/qualified-20261017-231053-229065/qualified.json']
resources={}
for path in resource_paths:
    q=read(path);assert q['passed']==1 and q['failed']==0 and not q['renderer_verified'];resources[q['profile']]=dict(path=str(path),sha256=sha(path),passed=1)
live_record=dict(Debug=live,actual_sessions=8,accepted_casts=2,rejected_requests=1,graphics=11736,MP_cost=18,actual_Release_diagnostic_pending=True,normal_product_registered=False,non15_brightness_change_verified=False,dynamic_target_following_verified=False,GUI_verified=False,physical_audio_verified=False,full_G06_verified=False,whole_goal_complete=False)
live_record.update(qualified_snapshot_scope_only=True,latest_Model_source_owner_fence_draft_typecheck_pending=True,controlled_non15_local_S21_replay_prepared_not_executed=True)
work='來源E15907AC正常兩版與1031編譯輸入維持釘選；default兩版Debug510／Release506、皆0敗21忽略、各112實際映像已獨立核對，460記錄身分終止。完整library沿原93199續驗：保留Debug映像，已獨立核對1943過0敗483忽略及1719歷史報告還原；Release仍待完成與讀回。產品與對應QA仍凍結。技能34隔離快照對C02共8sessions、2次成功各18MP及Sound34一次／1次MP不足拒絕，正常產品未註冊。最新草稿增加Model來源身分與同圖號世代切換亮度重設；明示本地S21夜間重播與退役preflight待編譯。正常Release設定視窗可及性及Escape取消已觀察，兩次擷取逾時，不列GUI通過。正常GUI／實體輸入音效、同端點、完整G06／G01–G12仍待；8／60秒、G11與Release設定不變，唯讀來源及獨立產物邊界維持。'
for n in names[:5]:
    path=ROOT/n;j=read(path);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
    j.update(current_native_class2_live_candidate=live_record,current_native_class2_bridge_candidate=bridges,current_native_class2_resource_candidate=resources,current_native_class2_ambient_candidate=ambient_record,current_native_class2_entry_order=entry_record,current_native1030_learned_request_audit=static_audit,current_native_class2_binding=class2,current_native1030_default111=dict(source_manifest_sha256=c['source_manifest_sha256'],profiles=b['profiles'],Release_qualification_pending=False,full_library_qualified=False,GUI_verified=False,whole_goal_complete=False),current_native_default111_completion=record,active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,
        next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(run/'cohort.json'),goal_complete=False,whole_goal_complete=False)
    j.update(current_native1030_ui_capture_observation=ui_record,current_native1030_library=library_record,active_workloads={'native1030-library-default':library_record},active_workload_sessions={'native1030-library-default':a.library_session},last_goal_turn_evidence=str(library/'cohort.json'))
    j.pop('current_native_default111_active',None)
    if 'items' in j:
        for item in j['items']:
            if item['id']=='G03-current-regression':
                item.setdefault('state_before_E159_default111',item['state']);item.update(state='E159_default111_and_Debug_library_qualified_Release_library_GUI_pending',complete=False)
                evidence=str(completion_path.relative_to(ROOT)).replace('\\','/');item.setdefault('evidence',[])
                if evidence not in item['evidence']:item['evidence'].append(evidence)
    scope=j.setdefault('current_source_goal_validation',{});scope['G03-current-regression']=dict(state='current_E159_default111_and_Debug_library_qualified_Release_library_GUI_pending',complete=False,completion=record,full_library_qualified=False,GUI_verified=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G03-current-source-full-library',work=work,applies_to_current_source=True,goal_complete=False)
    save(path,j)
for n in names[5:]:
    path=ROOT/n;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before_files=before,after_files={n:sha(ROOT/n) for n in names}))
print(json.dumps(dict(checkpoint=str(out),terminal_original_identities=completion['recorded_original_owned_identities'],verified_live_library=library_record,source_frozen=True,whole_goal_complete=False),ensure_ascii=False))
