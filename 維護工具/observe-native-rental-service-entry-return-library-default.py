"""Observe the original completed library pipeline after both independent readers."""
from pathlib import Path
import time
from datetime import datetime,timezone
import argparse,ctypes,hashlib,json,subprocess,zipfile
from ctypes import wintypes
ROOT=Path(__file__).resolve().parents[1]
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-rental-service-entry-return-library-default')
assert a.terminal_exit_code==0
assert a.terminal_chunk not in ['','pending','unknown']
c=read(run/'cohort.json');assert c['status']=='executed_both_current_full_library_default_selections' and c['owned_process'] is None
assert c['source_files']==1048 and c['compiled_input_files']==1049
assert c['source_manifest_sha256']=='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
for name,digest in read(run/'compiled-inputs.json').items():assert sha(ROOT/name)==digest,name
records=[dict(pid=c['pipeline_pid'],creation_utc=c['pipeline_creation_utc'],role='original pipeline')]+c['process_history'];profiles={}
assert len(c['process_history'])==8 and all(j['exit_code']==0 for j in c['process_history'])
for profile in ['Debug','Release']:
    qpath=run/profile/'full-default-qualified/qualified.json';q=read(qpath)
    assert q['status']=='independently_qualified_current_full_library_default_selection' and q['profile']==profile
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['failed']==q['filtered_out']==q['Rust_exit_code']==0
    assert not q['ignored_bodies_executed'] and not q['GUI_verified'] and not q['whole_goal_complete']
    assert q['passed']==c['profiles'][profile]['counts']['passed'] and q['ignored']==c['profiles'][profile]['counts']['ignored']
    archive=qpath.parent/'raw.zip';assert sha(archive)==q['raw_archive_sha256']
    with zipfile.ZipFile(archive) as z:
        assert set(z.namelist())==set(q['raw_files'])
        for name,digest in q['raw_files'].items():
            with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==digest
    runtime=read(run/profile/'runtime-full-current/receipt.json');records.append(dict(pid=runtime['pid'],creation_utc=runtime['creation_utc'],role=profile+' actual Rust tests'))
    profiles[profile]=dict(qualification=str(qpath),qualification_sha256=sha(qpath),passed=q['passed'],failed=0,ignored=q['ignored'],inventory=q['inventory'],actual_test_sha256=q['actual_test_sha256'])
if c.get('retained_failed_run'):
    old=read(Path(c['retained_failed_run'])/'cohort.json');assert old['status']=='failed_preserved'
    records.append(dict(pid=old['pipeline_pid'],creation_utc=old['pipeline_creation_utc'],role='preserved original failed metadata pipeline'))
    records+=old['process_history']
unique={(j['pid'],j['creation_utc']):j for j in records};ids=','.join(str(pid) for pid,_ in unique)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,Name,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.CloseHandle.argtypes=[wintypes.HANDLE]
observations=[]
for (pid,created),record in unique.items():
    row=next((row for row in live if row['ProcessId']==pid),None);observation='PID absent in authoritative CIM'
    if row:
        same=abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())<.001
        if same:
            handle=k.OpenProcess(0x100000|0x1000,False,pid);assert handle,record
            try:assert k.WaitForSingleObject(handle,0)==0,record
            finally:k.CloseHandle(handle)
            observation='original signalled'
        else:observation='PID reused with different creation time'
    observations.append(dict(pid=pid,creation_utc=created,observation=observation,current_CIM=row,original_terminal=True))
out=run/'completion-observation.json';assert not out.exists()
result=dict(status='independently_qualified_current1048_library_both_profiles_and_terminal_processes',cohort=str(run),cohort_sha256=sha(run/'cohort.json'),source_manifest_sha256=c['source_manifest_sha256'],
    original_session_id=a.session,observed_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk,profiles=profiles,
    recorded_original_owned_processes_all_terminal=True,recorded_original_owned_identities=len(observations),recorded_identities=observations,
    retained_Debug_image_and_inventory=bool(c.get('retained_Debug_compile_and_inventory')),preserved_prior_metadata_failure=bool(c.get('original_failure_preserved')),
    full_library_default_selection_qualified=True,ignored_bodies_executed=False,GUI_verified=False,physical_audio_verified=False,full_G06_verified=False,whole_goal_complete=False)
out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(run/'completion-observer.py').write_bytes(Path(__file__).read_bytes());(run/'completion-owned-CIM.json').write_bytes(raw)
print(json.dumps(dict(completion=str(out),profiles=profiles,terminal_identities=len(observations)),ensure_ascii=False))
