"""Seal actual same-map generation failure or exact isolated generation-fence cases."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current';normal=V/'native-ordinary-admission-normal/validation-20261018-152011-520801'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--kind',choices=['failure','candidate'],required=True);ap.add_argument('--session',type=int,required=True);ap.add_argument('--chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V)and a.terminal_exit_code==0 and a.chunk.lower()not in['','pending','unknown'];r=read(run/'receipt.json')
assert r['source_manifest_sha256']=='CBC7FC73A2049B0B240587EBDB5A80201E95C3DCF5E839DC3EE08172ED29D4C7'and r['owned_process']is None
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
for n,pin in r['libraries'].items():assert sha(pin['path'])==pin['sha256'],n
assert sha(run/'tests.rs')==r['probe_sha256']and sha(r['product'])==r['product_sha256']
if a.kind=='failure':
    assert r['status']=='reproduced_CBC7_same_map_new_generation_actor_death_leak'and not r['normal_product_source_modified']and r['baseline_child_exit_code']==101 and not r['baseline_regression_passed']
    assert[(p['label'],p['exit_code'])for p in r['process_history']]==[('compile',0),('probe',101)]
    o=read(run/'observation.json');assert o['authority_unchanged']and o['actor_alive']and o['old_map']==o['new_map']and o['new_world_generation']==o['old_world_generation']+1 and o['new_actor_phase']=='death'
    assert 'previous world death leaked into new alive actor after same-map generation change'in(run/'probe.stderr.txt').read_text('utf-8-sig')
    scope=dict(actual_Rust_exit_code=101,same_map_new_generation_death_leak_reproduced=True,regression_passed=False,synthetic_authority_resource_fixture=True)
else:
    assert r['status']=='isolated_Model_generation_direct_job_and_cancel_regressions_passed'and r['passed']==3 and r['failed']==r['ignored']==0 and not r['normal_product_source_modified']
    assert[(p['label'],p['exit_code'])for p in r['process_history']]==[('library',0),('compile',0),('regressions',0)]
    assert '3 passed; 0 failed; 0 ignored'in(run/'regressions.stdout.txt').read_text('utf-8-sig')
    assert len(r['source_overlays'])==1
    for n,pin in r['source_overlays'].items():assert sha(run/n)==pin['sha256'],n
    assert sha(r['diagnostic_library'])==r['diagnostic_library_sha256']
    profile=r.get('profile','Debug');assert profile in['Debug','Release']
    if profile=='Release':
        for job in r['process_history'][:2]:
            for flag in['opt-level=z','lto=thin','codegen-units=16','strip=symbols']:assert flag in job['arguments']
    scope=dict(profile=profile,passed=3,failed=0,ignored=0,same_map_new_generation_direct_Model_and_async_Job_verified=True,cancellation_preserves_old_Model_then_new_world_succeeds=True,candidate_applied=False,GUI_verified=False,physical_audio_verified=False,connection_change_without_world_generation_change_verified=False)
identities=[(r['runner_identity']['pid'],r['runner_identity']['creation_utc'])]+[(p['pid'],p['creation_utc'])for p in r['process_history']];ids=','.join(str(pid)for pid,_ in identities);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
c=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert c.returncode==0 and not c.stderr;live=json.loads(c.stdout.decode('utf-8-sig'))if c.stdout.strip()else[]
if isinstance(live,dict):live=[live]
for row in live:
    for pid,birth in identities:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(birth)).total_seconds())>.001
out=run/('failure-observation.json'if a.kind=='failure'else'qualified.json');assert not out.exists();files={p.name:p for p in run.iterdir()if p.is_file()};files['actual-test.exe']=Path(r['product']);pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(run/'evidence.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(run/'evidence.zip')as z:
    assert set(z.namelist())==set(pins)
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
record=dict(status='independently_sealed_CBC7_Model_generation_'+a.kind,source_manifest_sha256=r['source_manifest_sha256'],original_session_id=a.session or None,original_tool_completed_synchronously=a.session==0,observed_terminal_exit_code=0,terminal_tool_chunk=a.chunk,receipt_sha256=sha(run/'receipt.json'),archive_sha256=sha(run/'evidence.zip'),archive_files=pins,recorded_owned_identities=len(identities),recorded_owned_all_terminal=True,scope=scope,normal_source_modified=False,live_server_sessions=0,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(run/'completion-CIM.json').write_bytes(c.stdout);(run/'evidence-reader.py').write_bytes(Path(__file__).read_bytes());out.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(evidence=str(out),scope=scope,terminal_identities=len(identities)),ensure_ascii=False))
