"""Checkpoint an original production-rlib probe; never infer successful casts."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper();data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--mode',choices=['active','failed'],required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-haste43-live-current')
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
assert all(sha(R/n)==h for n,h in c['compiled_inputs'].items()) and c['production_rlib_used'] and not c['production_source_overlays_used']
rows=[c['pipeline'],*c['process_history']];filters=' OR '.join('ProcessId='+str(p['pid']) for p in rows);cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
probe=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert probe.returncode==0 and not probe.stderr;live=json.loads(probe.stdout.decode('utf-8-sig')) if probe.stdout.strip() else[];live=[live] if isinstance(live,dict) else live
is_live=lambda p:any(x['ProcessId']==p['pid'] and abs((datetime.fromisoformat(x['created'])-datetime.fromisoformat(p['creation_utc'])).total_seconds())<.001 for x in live)
if a.mode=='active':assert is_live(c['pipeline']) and not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'] and a.terminal_chunk is None and a.exit_code is None
else:assert c['status']=='failed_preserved' and a.exit_code not in [None,0] and a.terminal_chunk not in [None,'','unknown','pending'] and not any(is_live(p) for p in rows)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'original-owned-CIM.json').write_bytes(probe.stdout);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='running_current_production_haste43' if a.mode=='active' else'failed_current_production_haste43_preserved',source_manifest_sha256=source,run=str(run),scenario=c['scenario'],original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_exit_code=a.exit_code,qualification_granted=False,production_rlib_used=True,production_source_overlays_used=False,recorded_original_identities=len(rows),recorded_original_identities_terminal=a.mode=='failed',GUI_verified=False,whole_goal_complete=False)
if a.mode=='failed':
    files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and not p.is_relative_to(out)}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
    record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'))
(out/'evidence.json').write_bytes(data(record))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_production_haste43_active_or_failure']=record;j.update(active_workloads={'production_haste43':record} if a.mode=='active' else{},active_workload_sessions={'production_haste43':a.session} if a.mode=='active' else{},source_and_corresponding_QA_frozen=a.mode=='active',source_freeze_while_Cargo=False,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',whole_goal_complete=False,goal_complete=False);staged[n]=data(j)
    else:staged[n]=p.read_bytes()
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(checkpoint=str(out),mode=a.mode,original_session=a.session,qualified=False),ensure_ascii=False))
