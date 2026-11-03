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
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-failure-presentation')
c=read(run/'receipt.json');driver=read(run/'runner-identity.json');k=ctypes.WinDLL('kernel32',use_last_error=True)
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
work='來源'+c['source_manifest_sha256'][:8]+'正常兩版各31相關測試、77三案12sessions、G06兩版六分支96sessions及成功交接非同步Presentation兩版12sessions維持各自資格。原'+str(a.session)+'正在'+c['profile']+'／'+c['scenario']+'實際非同步失敗恢復診斷：同一Presentation在失敗／取消時撤下畫面並退役，重新登入同map500後驗舊input surface拒絕及實際移動。當案未取得結果；產品來源／QA及探針凍結，8秒／60秒／自然30秒到期與G11預算不變。正常GUI／實體輸入音效、同端點、旅館／其他服務、完整G06與G01–G12仍待；登入器／素材唯讀，產物獨立。'
for n in names[:5]:
    path=ROOT/n;j=read(path);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
    j.update(current_native_G06_failure_presentation_active=record,active_workloads={'native-teleport-failure-presentation':record},active_workload_sessions={'native-teleport-failure-presentation':a.session},source_and_corresponding_QA_frozen=True,
        next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(run/'cohort.json'),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native77-current-live-cast-validation',work=work,applies_to_current_source=True,goal_complete=False)
    save(path,j)
for n in names[5:]:
    path=ROOT/n;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before_files=before,after_files={n:sha(ROOT/n) for n in names}))
print(json.dumps(dict(checkpoint=str(out),verified_live_driver=driver,source_frozen=True,whole_goal_complete=False),ensure_ascii=False))
