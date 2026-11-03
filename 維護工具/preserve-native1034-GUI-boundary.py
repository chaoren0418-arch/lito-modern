"""Preserve an expired owned GUI attempt and close only its recorded process."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--exit-code',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1034-interactive-teleport')
c=read(run/'receipt.json');assert c['status']=='failed_preserved' and c['owned_process'] is None and a.exit_code!=0 and a.terminal_chunk not in ['','pending','unknown']
assert 'expired' in c['error'] or 'aborted' in c['error'];rt=Path(c['runtime']);assert rt.is_relative_to(ROOT/'LiTo381-Modern/target/current')
out=run/'terminal-boundary';out.mkdir();cleanup=[]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.TerminateProcess.argtypes=[wintypes.HANDLE,wintypes.UINT];k.CloseHandle.argtypes=[wintypes.HANDLE]
for role in ['a','b']:
    p=run/('normal-launch-'+role)/'receipt.json'
    if not p.exists():continue
    launch=read(p);owner=launch.get('normal_process')
    if owner is None:continue
    assert sha(launch['normal_product'])==launch['normal_product_sha256'];assert Path(owner['ExecutablePath']).resolve().is_relative_to(ROOT/'LiTo381-Modern/target/current')
    handle=k.OpenProcess(0x100000|0x1000|1,False,owner['ProcessId']);observation=dict(role=role,recorded_process=owner,stopped=False)
    if not handle:
        assert ctypes.get_last_error()==87,'cannot inspect recorded app; do not terminate unknown process';observation['terminal_observation']='PID absent'
    else:
        try:
            ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32);born=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
            if abs((born-datetime.fromisoformat(owner['creation_utc'].replace('Z','+00:00'))).total_seconds())>=.001:observation['terminal_observation']='recorded owner absent; PID reused; no action'
            elif k.WaitForSingleObject(handle,0)==0:observation['terminal_observation']='original handle terminal'
            else:
                # Keep the verified original handle throughout termination.
                assert k.TerminateProcess(handle,1);assert k.WaitForSingleObject(handle,5000)==0;observation.update(stopped=True,terminal_observation='original recorded handle terminated after GUI lease expiry')
        finally:k.CloseHandle(handle)
    cleanup.append(observation)
captures=list((rt/'captures').glob('*/frames.jsonl'));rows=[json.loads(s) for p in captures for s in p.read_text(encoding='utf-8-sig').splitlines()]
assert [v['opcode'] for v in rows[:3]]==['0x01','0x00','0x3A']
record=dict(status='preserved_incomplete_GUI_authentication_and_capture_boundary_after_owned_lease',source_manifest_sha256=c['source_manifest_sha256'],controller_original_session=a.session,controller_observed_exit_code=a.exit_code,terminal_tool_chunk=a.terminal_chunk,controller_receipt_sha256=sha(run/'receipt.json'),normal_cleanup=cleanup,actual_captured_frames=len(rows),captured_opcodes=[v['opcode'] for v in rows],normal_keyboard_version_exchange_verified=True,authentication_automated=False,authentication_request_expired=True,user_authentication_pending=False,GUI_visual_verified=False,GUI_acceptance=False,physical_input_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'boundary.json',record)
files={str(p.relative_to(run)):p for p in [run/'receipt.json',run/'controller-raw.zip',run/'controller-archive-readback.json',run/'computer-use-observations.json'] if p.exists()}
for role in ['a','b']:
    folder=run/('normal-launch-'+role)
    if folder.is_dir():files.update({p.relative_to(run).as_posix():p for p in folder.iterdir() if p.is_file()})
files.update({'runtime/'+p.relative_to(rt).as_posix():p for p in captures});files['boundary.json']=out/'boundary.json';files['preserver.py']=Path(__file__);pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'raw-readback.json',dict(files=pins,archive_sha256=sha(out/'raw.zip')))
before=read(run/'keyboard-connection-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
with zipfile.ZipFile(out/'tracker-before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'tracker-before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='來源86E65C7B控制器與非同步Presentation兩批各兩版六分支12案／96sessions均維持獨立資格。正常Release唯一入口與Space鍵盤版本交換三封包已核對；WGC兩次逾時、登入需由使用者操作，隔離服務原'+str(a.session)+'已租約到期actual exit'+str(a.exit_code)+'並封存，僅清理出生時間符合的自有正常APP，該GUI嘗試未通過且舊測試位址已失效。下一步執行當版兩版廣域111整合＋main，再完整library；GUI／實體輸入音效、40技能請求缺口及完整G06／G01–G12仍待。正常兩版各57相關測試不等於廣域已驗，8／60秒、Release及G11不變，相鄰只讀且產物獨立。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        j=read(p);j['current_native1034_GUI'].update(record,boundary=str(out/'boundary.json'),boundary_sha256=sha(out/'boundary.json'));j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'boundary.json'),goal_complete=False,whole_goal_complete=False);j['current_focus']=dict(item='current-source-broad-default111',work=work,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(p,j)
    else:
        first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'tracker-readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},boundary_sha256=sha(out/'boundary.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'tracker-readback.json')['after_files'].items())
print(json.dumps(dict(boundary=str(out),captured_frames=len(rows),cleanup=cleanup,GUI_acceptance=False,whole_goal_complete=False),ensure_ascii=False),flush=True)
