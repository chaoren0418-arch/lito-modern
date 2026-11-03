"""Bind both broad selections to exact original terminal processes and inputs."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('cohort',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
base=a.cohort.resolve(strict=True);assert base.is_relative_to(ROOT/'驗證紀錄/current/native-installed-resource-profile-default117')
b=read(base/'cohort.json');assert b['status']=='independently_qualified_current_default117_both_profiles' and b['owned_process'] is None
assert a.terminal_exit_code==0
assert a.terminal_chunk not in {'pending','unknown',''}
normal=Path(b['normal_qualification']);startup=normal.parent.parent;identity=Path(read(startup/'cohort.json')['application'])/'applied-source-inputs.json';source=sha(identity)
assert source==b['source_manifest_sha256']=='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
source_bytes=identity.read_bytes();assert b'\r\n' not in source_bytes
startup=normal.parent.parent;inputs=read(startup/'compiled-inputs.json');assert len(inputs)==1049 and len(read(identity))==1048
for n,h in inputs.items():assert sha(ROOT/n)==h,n
records=[dict(pid=b['pipeline_pid'],creation_utc=b['pipeline_creation_utc'])]+b['process_history'];profiles={}
for profile in ['Debug','Release']:
    info=b['profiles'][profile];run=Path(info['run']);r=read(run/'receipt.json');q=read(info['qualification'])
    assert info['qualified'] and info['actual_runner_exit_code']==0 and sha(info['qualification'])==info['qualification_sha256']
    assert r['status']=='verified_current_default117_integration_targets_and_main_units' and r['owned_process'] is None
    assert q['source_manifest_sha256']==source and q['actual_test_executables']==118 and q['counts']['failed']==0
    assert q['per_target']['native_attack_skills']['passed']==16 and q['per_target']['native_skill_selection']['passed']==7 and q['per_target']['native_skills']['passed']==13
    assert not q['full_library_scope_inferred'] and not q['GUI_verified'] and not q['whole_goal_complete']
    records += [read(run/'runner-identity.json')]+r['process_history']
    profiles[profile]=dict(run=str(run),qualification=info['qualification'],qualification_sha256=sha(info['qualification']),counts=q['counts'],integration_targets=117,actual_test_executables=118,ignored_tests={k:v['ignored'] for k,v in q['all_test_names'].items() if v['ignored']},normal_mains=q['normal_mains'])
assert profiles['Debug']['normal_mains']==profiles['Release']['normal_mains']
for profile,digest in profiles['Debug']['normal_mains'].items():assert sha(read(normal)['profiles'][profile]['normal']['path'])==digest
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.CloseHandle.argtypes=[wintypes.HANDLE];k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
unique={}
for j in records:
    if 'exit_code' in j:assert j['exit_code']==0 and not j.get('timed_out')
    key=(j['pid'],j['creation_utc'])
    if key in unique:continue
    handle=k.OpenProcess(0x100000|0x1000,False,j['pid'])
    fallback=None
    if not handle:
        error=ctypes.get_last_error()
        if error==87:observation='PID absent'
        else:
            assert error==5,dict(pid=j['pid'],error=error)
            command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter 'ProcessId="+str(j['pid'])+"'|Select-Object ProcessId,Name,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
            output=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);fallback=json.loads(output.decode('utf-8-sig')) if output.strip() else []
            fallback=[fallback] if isinstance(fallback,dict) else fallback
            if not fallback:observation='PID absent in CIM after OpenProcess AccessDenied'
            else:
                assert len(fallback)==1 and fallback[0]['ProcessId']==j['pid']
                actual=datetime.fromisoformat(fallback[0]['created'].replace('Z','+00:00'))
                assert abs((actual-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001,dict(original=j,current=fallback)
                observation='PID reused; authoritative CIM creation time differs'
    else:
        try:
            ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
            actual=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
            reused=abs((actual-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
            assert reused or k.WaitForSingleObject(handle,0)==0,j;observation='PID reused' if reused else 'original signalled'
        finally:k.CloseHandle(handle)
    unique[key]=dict(pid=j['pid'],creation_utc=j['creation_utc'],original_terminal=True,observation=observation,CIM_fallback=fallback)
assert sum(p['actual_test_executables'] for p in profiles.values())==236
for name,digest in b['validation_pins'].items():assert sha(ROOT/'維護工具'/name)==sha(base/name)==digest,name
out=base/'completion-observation.json';assert not out.exists()
result=dict(status='independently_qualified_current1048_default117_both_profiles_and_terminal_processes',cohort=str(base),cohort_sha256=sha(base/'cohort.json'),original_session_id=a.session,observed_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk,Debug_external_cohort_session_id=a.session,Debug_child_runner_exit_code=b['profiles']['Debug']['actual_runner_exit_code'],source_manifest_sha256=source,source_files=1048,compiled_input_files=1049,profiles=profiles,actual_test_executables=sum(p['actual_test_executables'] for p in profiles.values()),recorded_original_owned_processes_all_terminal=True,recorded_original_owned_identities=len(unique),recorded_identities=list(unique.values()),source_descriptor_line_endings='LF',profile_descriptor_text_clarification='Original current LF descriptor bytes retained and independently hashed; no conversion.',full_library_qualified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(base/'completion-observer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(completion=str(out),counts={p:j['counts'] for p,j in profiles.items()},terminal_identities=len(unique)),ensure_ascii=False))
