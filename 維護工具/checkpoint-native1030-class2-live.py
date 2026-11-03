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
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native1030-default111')
b=read(run/'cohort.json');assert b['status'] in ['executing_current_default111_profile','independently_reading_current_default111_profile'] and b['active_profile']=='Release'
debug=b['profiles']['Debug'];dq=read(Path(debug['qualification']));assert debug['qualified'] and sha(Path(debug['qualification']))==debug['qualification_sha256'] and dq['counts']['failed']==0 and dq['actual_test_executables']==112
c=dict(profile='Release',source_manifest_sha256=b['source_manifest_sha256']);driver=dict(pid=b['pipeline_pid'],creation_utc=b['pipeline_creation_utc']);k=ctypes.WinDLL('kernel32',use_last_error=True)
k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.CloseHandle.argtypes=[wintypes.HANDLE]
k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
handle=k.OpenProcess(0x100000|0x1000,False,driver['pid']);assert handle
try:
    assert k.WaitForSingleObject(handle,0)==258
    times=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in times])
    ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32);created=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
    assert abs((created-datetime.fromisoformat(driver['creation_utc'])).total_seconds())<.001
finally:k.CloseHandle(handle)
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=run/('running-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(run=str(run),original_session_id=a.session,source_manifest_sha256=c['source_manifest_sha256'],driver=driver,verified_live_utc=datetime.now(timezone.utc).isoformat(),qualification_pending=True)
work='來源'+c['source_manifest_sha256'][:8]+'之原Debug10192已actual exit0，112實際映像／111整合目標加main，獨立逐名稱／raw／來源／映像讀回'+str(dq['counts']['passed'])+'過、0敗、'+str(dq['counts']['ignored'])+'既有忽略。這是default整合範圍，未執行忽略本文、完整library或現場GUI。Release沿原'+str(a.session)+'續跑，未重跑Debug；兩主與1031編譯輸入維持釘選，產品及對應QA凍結。既有G06控制器96、非同步跨端點成功12／失敗84、重新登入音效16、同連線WORLD成功8／拒絕8sessions維持各自資格。完整library、必要資源、正常GUI／實體輸入音效、C72來源音效、同端點成功及完整G06／G01–G12仍待；Windows工具失敗不列通過。原時限、CRLF來源識別、Release設定不變，登入器與素材唯讀且產物獨立，寫入限重建版。'

audit_path=ROOT/'驗證紀錄/current/native1030-learned-request-audit/20261017-222459-870750/audit.json'
audit=read(audit_path);assert audit['source_manifest_sha256']==c['source_manifest_sha256'] and audit['observed_learned_ids']==74 and audit['recognized_learned_requests']==33 and audit['missing_learned_requests']==41 and audit['new_casts']==0 and not audit['product_source_modified']
binding_path=ROOT/'驗證紀錄/current/native-class2-call-lightning/20261017-223423-844766/binding.json'
binding=read(binding_path);assert not binding['renderer_qualified'] and not binding['product_source_modified'] and binding['formal_skill34']['effectGfx']==11736
assert sha(Path(binding['native_disintegrate']['path']))==binding['native_disintegrate']['sha256']
static_audit=dict(path=str(audit_path),sha256=sha(audit_path),source_manifest_sha256=c['source_manifest_sha256'],learned=74,request_recognized=33,missing_requests=41,new_casts=0,gameplay_coverage_inferred=False)
class2=dict(path=str(binding_path),sha256=sha(binding_path),ambient_supplement=str(binding_path.parent/'ambient-context-supplement.json'),byte_predicates=7,target_following_confirmed=True,render_environment_callback_integration_pending=True,renderer_qualified=False,new_casts=0)
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
work='來源E15907AC的Debug default回歸510過0敗21既有忽略已獨立讀回；Release原35039已完成112個測試映像編譯，沿原工作執行及讀回，未重跑。正常兩版及1031編譯輸入維持釘選，產品與對應QA凍結。技能34之隔離候選完成六核心加四新模組接線並編入Debug診斷：現行C02成功4sessions／2次canonical11736各扣18MP、Sound34各一次；拒絕4sessions／MP不足278、零特效音效。原始封包、authority、清理及8張PNG已核對，僅列候選診斷；正常技能辨識仍33／74。資源候選兩版各1項、回呼橋接各8項、亮度ledger各10項分別通過。非15亮度整合、動態目標／場景退役、Release診斷與正常GUI／實體音效、完整library、同端點及完整G06／G01–G12仍待。原8／60秒、Release設定及G11預算不變，登入器／素材唯讀且產物獨立，寫入限重建版。'
for n in names[:5]:
    path=ROOT/n;j=read(path);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
    j.update(current_native_class2_live_candidate=live_record,current_native_class2_bridge_candidate=bridges,current_native_class2_resource_candidate=resources,current_native_class2_ambient_candidate=ambient_record,current_native_class2_entry_order=entry_record,current_native1030_learned_request_audit=static_audit,current_native_class2_binding=class2,current_native1030_default111=dict(source_manifest_sha256=c['source_manifest_sha256'],profiles={'Debug':debug},Release_qualification_pending=True,full_library_qualified=False,GUI_verified=False,whole_goal_complete=False),current_native_default111_active=record,active_workloads={'default-native-regression':record},active_workload_sessions={'default-native-regression':a.session},source_and_corresponding_QA_frozen=True,
        next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(run/'cohort.json'),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G03-current-source-default111-regression',work=work,applies_to_current_source=True,goal_complete=False)
    save(path,j)
for n in names[5:]:
    path=ROOT/n;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before_files=before,after_files={n:sha(ROOT/n) for n in names}))
print(json.dumps(dict(checkpoint=str(out),verified_live_driver=driver,source_frozen=True,whole_goal_complete=False),ensure_ascii=False))
