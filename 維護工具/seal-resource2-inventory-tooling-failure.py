"""Seal the wrapper count mismatch; the recorded Debug Rust tests passed."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';T=R/'維護工具';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper();data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
run=V/'native-current-library-motion-resource2/run-20261020-131217-980406';c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
assert c['status']=='failed_preserved' and c['owned_process'] is None and set(c['profiles'])=={'Debug'} and c['profiles']['Debug']['exit_code']==0 and c['resource_metadata_unchanged']
log=(run/'Debug/tests.log').read_text('utf-8-sig');assert '2 passed; 0 failed; 0 ignored; 0 measured; 2461 filtered out;' in log and 'FAILED' not in log
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
rows=[c['pipeline'],c['profiles']['Debug']];filters=' OR '.join('ProcessId='+str(p['pid']) for p in rows);cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
probe=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert probe.returncode==0 and not probe.stderr;live=json.loads(probe.stdout.decode('utf-8-sig')) if probe.stdout.strip() else[];live=[live] if isinstance(live,dict) else live
assert not any(x['ProcessId']==p['pid'] and abs((datetime.fromisoformat(x['created'])-datetime.fromisoformat(p['creation_utc'])).total_seconds())<.001 for x in live for p in rows)
out=run/('tooling-failure-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'owned-CIM.json').write_bytes(probe.stdout);(out/'sealer.py').write_bytes(Path(__file__).read_bytes())
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and not p.is_relative_to(out)}
for name in ['run-text-index-order-motion-resource2.py','run-text-index-order-haste-resource2.py','checkpoint-text-index-order-motion-resource2.py']:files['QA-before/'+name]=T/name
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(source_manifest_sha256=source,status='original_resource2_wrapper_inventory_mismatch_preserved',run=str(run),qualified=False,original_session=94979,terminal_tool_chunk='7e8613',wrapper_exit_code=1,Debug_Rust_exit_code=0,Debug_passed=2,Debug_failed=0,Debug_filtered_out=2461,Release_executed=False,expected_in_old_wrapper=2460,actual_inventory=2463,selected_tests=2,recorded_original_processes_terminal=True,failed_active_checkpoint_tool_chunk='36744d',active_checkpoint_did_not_write=True,product_source_changed=False,GUI_verified=False,whole_goal_complete=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
(out/'evidence.json').write_bytes(data(record));before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_resource2_tooling_inventory_failure']=record;j.update(last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',whole_goal_complete=False,goal_complete=False);staged[n]=data(j)
    else:staged[n]=p.read_bytes()
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(evidence=str(out/'evidence.json'),Debug_passed=2,Debug_failed=0,Release_executed=False),ensure_ascii=False))
