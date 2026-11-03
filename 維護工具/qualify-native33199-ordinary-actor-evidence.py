"""Seal exact resource-only baseline or isolated candidate evidence."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,hashlib,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--kind',choices=['baseline','candidate','map-failure','map-control'],required=True);ap.add_argument('--session',type=int,required=True);ap.add_argument('--chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V);assert a.terminal_exit_code==0 and a.chunk.lower() not in ['','pending','unknown']
r=read(run/'receipt.json');assert r['owned_process'] is None and not r['normal_product_source_modified'] and r['live_server_sessions']==0
normal=V/'native-scoped-catalog-normal/validation-20261018-132210-585372';inputs=read(normal/'compiled-inputs.json')
assert r['source_manifest_sha256']=='33199F4ED6584FD4D19FBF1EC69C3EC69AB6E62B27ABA64CC59FE0AD2C986ED4'
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for n,pin in r['libraries'].items():assert sha(pin['path'])==pin['sha256'],n
assert sha(run/'tests.rs')==r['probe_sha256'] and sha(r['product'])==r['product_sha256']
if a.kind=='baseline':
    assert r['status']=='reproduced_same_World_new_Model_ordinary_actor_audio_replay_failure' and r['baseline_child_exit_code']==101 and not r['baseline_regression_passed']
    assert [(p['label'],p['exit_code']) for p in r['process_history']]==[('compile',0),('probe',101)]
    assert 'ordinary actor cue replayed after same-World Model recreation' in (run/'probe.stderr.txt').read_text('utf-8-sig')
    o=read(run/'observation.json');assert o['same_authority_world_unchanged'] and o['old_deliveries']==0 and o['old_results']==['Stale','AlreadyConsumed']
    assert len(o['observations'])==2 and o['observations'][0]['event']==o['observations'][1]['event']
    assert o['observations'][0]['sound']==o['observations'][1]['sound']==1534
    scope=dict(baseline_regression_passed=False,reproduced_actor_replay=True,Rust_exit_code=101)
elif a.kind in ['map-failure','map-control']:
    failure=a.kind=='map-failure';expected=101 if failure else 0
    assert [(p['label'],p['exit_code'])for p in r['process_history']]==[('compile',0),('probe',expected)]
    o=read(run/'observation.json');assert o['authority_unchanged'] and o['actor_alive'] and o['old_world_generation']<o['new_world_generation'] and o['old_map']!=o['new_map']
    if failure:
        assert r['status']=='reproduced_isolated_candidate_v2_cross_world_death_pose_leak' and o['new_actor_phase']=='death'
        assert 'previous world death leaked into new alive actor after map change' in (run/'probe.stderr.txt').read_text('utf-8-sig')
        parent=V/'native33199-ordinary-admission-v2-ready/20261018-145825-685763';p=read(parent/'receipt.json');assert r['normal_library']['sha256']==p['diagnostic_library_sha256'] and sha(p['diagnostic_library'])==p['diagnostic_library_sha256']
    else:assert r['status']=='normal_direct_Model_map_change_resets_previous_world_death_as_expected' and o['new_actor_phase']=='action' and o['new_actor_action']==68
    scope=dict(Rust_exit_code=expected,previous_world_death_leak=failure,normal_library_control_passed=not failure,synthetic_authority_resource_fixture=True,candidate_applied=False)
else:
    count=r['passed'];assert count in [4,5] and r['failed']==r['ignored']==0 and r['status']=='isolated_S4A_model_recreation_pending_retirement_and_fresh_connection_regressions_passed'
    assert [(p['label'],p['exit_code']) for p in r['process_history']]==[('library',0),('compile',0),('regressions',0)]
    text=(run/'regressions.stdout.txt').read_text('utf-8-sig');assert f'{count} passed; 0 failed; 0 ignored' in text
    if count==5:assert 'ordinary_direct_model_map_change_cannot_keep_previous_world_death ... ok' in text
    assert len(r['source_overlays'])==5
    profile=r.get('profile','Debug');assert profile in ['Debug','Release']
    if profile=='Release':
        assert tuple(r['release_configuration'][k]for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
        for job in r['process_history'][:2]:
            for flag in ['opt-level=z','lto=thin','codegen-units=16','strip=symbols']:assert flag in job['arguments']
    for n,pin in r['source_overlays'].items():assert sha(run/n)==pin['sha256'],n
    assert sha(r['diagnostic_library'])==r['diagnostic_library_sha256']
    for n,pending in [('actor-published.json',False),('actor-pending.json',True)]:
        o=read(run/n);assert o['pending']==pending and o['authority_unchanged'] and o['retired_deliveries']==0 and o['fresh_deliveries']==[1534] and o['fresh_sequence']>o['retired_sequence'] and o['recreated_distinct_frames']>=3
    o=read(run/'observation.json');assert o['same_authority_world_unchanged'] and o['old_deliveries']==o['new_Model_legacy_cues']==o['new_Model_raw_sounds']==0
    scope=dict(profile=profile,passed=count,failed=0,ignored=0,S4A_and_S20_published_and_pending_same_world_verified=True,new_world_direct_Model_map_reset_verified=count==5,candidate_applied=False,Release_candidate_verified=profile=='Release',source_ABA_verified=False,actor_reincarnation_verified=False,death_pose_verified=False,guardian_motion_verified=False,classic_terrain_reload_verified=False)
identities=[(r['runner_identity']['pid'],r['runner_identity']['creation_utc'])]+[(p['pid'],p['creation_utc']) for p in r['process_history']]
ids=','.join(str(p) for p,_ in identities)
cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
c=subprocess.run(['powershell.exe','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert c.returncode==0 and not c.stderr
live=json.loads(c.stdout.decode('utf-8-sig')) if c.stdout.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for pid,birth in identities:
        if pid==row['ProcessId']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(birth)).total_seconds())>.001
out=run/('failure-observation.json' if a.kind in ['baseline','map-failure'] else 'qualified.json');assert not out.exists()
files={p.name:p for p in run.iterdir() if p.is_file()};files['actual-test.exe']=Path(r['product'])
pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(run/'evidence.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(run/'evidence.zip') as z:
    assert set(z.namelist())==set(pins)
    for n,h in pins.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
record=dict(status='independently_sealed_resource_only_ordinary_actor_'+a.kind,original_session_id=a.session or None,original_tool_completed_synchronously=a.session==0,observed_terminal_exit_code=0,terminal_tool_chunk=a.chunk,source_manifest_sha256=r['source_manifest_sha256'],receipt_sha256=sha(run/'receipt.json'),archive_sha256=sha(run/'evidence.zip'),archive_files=pins,recorded_owned_identities=len(identities),recorded_owned_all_terminal=True,normal_sources_modified=False,GUI_verified=False,physical_audio_verified=False,live_server_sessions=0,whole_goal_complete=False,scope=scope,observed_utc=datetime.now(timezone.utc).isoformat())
(run/'completion-CIM.json').write_bytes(c.stdout);(run/'evidence-reader.py').write_bytes(Path(__file__).read_bytes());out.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(evidence=str(out),scope=scope,recorded_owned_identities=len(identities)),ensure_ascii=False))
