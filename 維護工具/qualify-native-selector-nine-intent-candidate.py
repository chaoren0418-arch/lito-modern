"""Independently read back the isolated nine-selector candidate, without applying it."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, subprocess, zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path)
ap.add_argument('--session-id',type=int,required=True)
ap.add_argument('--terminal-exit-code',type=int,required=True)
ap.add_argument('--terminal-chunk',required=True)
a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-selector-nine-intents')
assert a.terminal_exit_code==0
terminal=dict(original_session_id=a.session_id,handle_closed=True,terminal_exit_code=a.terminal_exit_code,
    terminal_tool_chunk_id=a.terminal_chunk,observed_utc=datetime.now(timezone.utc).isoformat())
save(run/'original-session-observation.json',terminal)
r=read(run/'receipt.json');rb=read(run/'readback.json');m=read(run/'candidate.json')
assert r['status']==rb['status']=='passed_isolated_selector_candidate'
assert (r['passed'],r['failed'],r['ignored'])==(4,0,0)
assert not r['product_source_changed'] and not r['applied']
assert r['actual_new_sessions']==r['actual_new_casts']==0
assert sha(run/'raw.zip')==rb['raw_archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert set(z.namelist())==set(rb['raw_files'])
    for n,h in rb['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
        p=Path(r['executable']) if n=='candidate.exe' else run/n
        if p.exists():assert sha(p)==h
base=Path(r['base_related_run']);pins=read(base/'compiled-inputs.json')
assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
assert r['source_manifest_sha256']==m['base_source_manifest_sha256']==read(base/'completion-observation.json')['source_manifest_sha256']
for p,h in r['candidate_pins'].items():assert sha(p)==h
for name,h in m['after'].items():assert sha(run/Path(name).name)==h
with zipfile.ZipFile(run/'before.zip') as z:
    assert set(z.namelist())==set(m['before'])
    for n,h in m['before'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
        assert sha(ROOT/n)==h
for v in r['libraries'].values():assert sha(v['path'])==v['sha256']
assert sha(run/'c02-concentration-target.javap.txt')==m['concentration_target_contract_sha256']
names=re.findall(r'^(.+): test$',(run/'inventory/stdout.log').read_text(encoding='utf-8-sig'),re.M)
text=(run/'runtime/stdout.log').read_text(encoding='utf-8-sig')
assert len(names)==len(set(names))==4
assert set(re.findall(r'^test (.+?) \.\.\. ok$',text,re.M))==set(names)
assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)==[('4','0','0')]
assert not (run/'runtime/stderr.log').read_bytes().strip()
assert len(r['process_history'])==3 and r['owned_process'] is None
assert all(p['exit_code']==0 for p in r['process_history'])
ids=','.join(str(p['pid']) for p in r['process_history'])
script=f"$ids=@({ids}); @(Get-CimInstance Win32_Process | Where-Object {{$_.ProcessId -in $ids}} | ForEach-Object {{[pscustomobject]@{{pid=$_.ProcessId;creation_utc=$_.CreationDate.ToUniversalTime().ToString('o');command=$_.CommandLine}}}}) | ConvertTo-Json -Depth 3 -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',script],encoding='utf-8-sig').strip()
live=json.loads(raw) if raw else []
if isinstance(live,dict):live=[live]
for p in r['process_history']:
    assert not any(v['pid']==p['pid'] and datetime.fromisoformat(v['creation_utc'].replace('Z','+00:00'))==datetime.fromisoformat(p['creation_utc']) for v in live)
save(run/'independent-readback.json',dict(status='independently_qualified_isolated_nine_selector_candidate',
    original_session=terminal,source_manifest_sha256=r['source_manifest_sha256'],compiled_inputs_checked=1025,
    raw_archive_sha256=sha(run/'raw.zip'),executable_sha256=sha(r['executable']),
    passed=4,failed=0,ignored=0,recorded_direct_processes=3,recorded_direct_processes_all_terminal=True,
    observed_pid_reuse=live,product_source_changed=False,applied=False,
    actual_new_sessions=0,actual_new_casts=0,graphics_and_native_audio_pipeline_connected=False,
    GUI_verified=False,whole_goal_complete=False))
print(json.dumps(dict(status='independently_qualified_isolated_nine_selector_candidate',passed=4,failed=0,ignored=0,
    output=str(run/'independent-readback.json')),ensure_ascii=False))
