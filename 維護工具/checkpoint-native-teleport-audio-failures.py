"""Reconcile preserved failed diagnostics; never infer audio qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE
k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.CloseHandle.argtypes=[wintypes.HANDLE]
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def terminal(j):
    h=k.OpenProcess(0x100000|0x1000,False,j['pid'])
    if not h:
        assert ctypes.get_last_error()==87,(j,ctypes.get_last_error())
        return dict(**j,original_identity_terminal=True,observation='PID absent')
    try:
        ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t) for t in ts])
        ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
        born=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
        expected=datetime.fromisoformat(j.get('creation_utc',j.get('start_utc')).replace('Z','+00:00'))
        reused=abs((born-expected).total_seconds())>.001
        assert reused or k.WaitForSingleObject(h,0)==0,j
        return dict(**j,original_identity_terminal=True,observation='PID reused' if reused else 'original process signalled')
    finally:k.CloseHandle(h)
runs=[ROOT/'驗證紀錄/current/native-teleport-audio/run-20261017-210827-960246',ROOT/'驗證紀錄/current/native-teleport-item-audio/run-20261017-211905-869284']
records=[];identities=[]
for run in runs:
    c=read(run/'receipt.json');f=read(run/'failure-preserved.json');assert c['status']=='failed_preserved' and f['original_exit_code']==1
    assert sha(run/'failed-runtime.zip')==f['raw_archive_sha256']
    with zipfile.ZipFile(run/'failed-runtime.zip') as z:
        assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in f['raw_files'].items())
    case=read(Path(c['case'])/'receipt.json')
    identities += [terminal(read(run/'runner-identity.json'))]+[terminal(j) for j in c['process_history']]+[terminal(j) for j in case['owned_processes']]
    records.append(dict(run=str(run),original_session_id=f['original_session_id'],original_exit_code=1,terminal_tool_chunk=f['terminal_tool_chunk'],failure_receipt_sha256=sha(run/'failure-preserved.json'),observation=f['observed'],qualified=False))
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=ROOT/'驗證紀錄/current/native-teleport-item-audio'/('terminal-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='來源E15907AC既有正常兩主／G06控制器96sessions／非同步成功12及失敗84sessions保留原範圍資格。音效來源嘗試20419與18535皆已退出1並封存：租裝村忽略C14；合法C1C消耗加速藥水但未產生S37／音效。兩案均無音效資格，記錄程序身分已核對終止，清除過期執行中狀態。下一步以普通戰鬥真實音效檢查連線／場景退役，與C72跨端點音效待驗範圍分開。重建版與登入器獨立，只讀參考線索，撤回替換登入器JAR提案；修改及產物限重建版。正常GUI／實體輸入音效、同端點及完整G06／G01–G12仍待，全部時限預算不變。'
for n in names[:5]:
    p=ROOT/n;j=read(p);assert j['current_product_identity']['source_manifest_sha256'].startswith('E15907AC')
    j.update(current_native_G06_audio_active=None,current_native_G06_audio_failed_attempts=records,active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'readback.json'),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G06-audio-lifecycle-supported-source',work=work,applies_to_current_source=True,goal_complete=False)
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(failed_attempts=records,recorded_identities=identities,all_original_identities_terminal=True,before=before,after={n:sha(ROOT/n) for n in names},whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),failed_attempts=len(records),terminal_identities=len(identities)),ensure_ascii=False))
