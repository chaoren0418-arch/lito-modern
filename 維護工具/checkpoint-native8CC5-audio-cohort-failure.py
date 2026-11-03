"""Preserve the actual Debug readiness failure, completed Release cases and terminal births."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native8CC5-audio-cohort')
c=read(run/'cohort.json');assert c['status']=='failed_preserved' and c['owned_process'] is None and c['active_profile']=='Debug' and c['active_scenario']=='refused'
assert len(c['cases'])==6 and c['actual_sessions']==48 and all(entry['profile']=='Release' for entry in c['cases'].values())
assert c['process_history'][-1]['exit_code']==1 and c['process_history'][-1]['label']=='Debug-refused'
tags=[line[len('NORMAL_AUDIO_RUN='):] for line in(run/'Debug-refused/stdout.log').read_text('utf-8-sig').splitlines() if line.startswith('NORMAL_AUDIO_RUN=')];assert len(tags)==1
failed=Path(tags[0]);r=read(failed/'receipt.json');assert r['status']=='failed_preserved' and r['asset_metadata_unchanged'] and r['owned_process'] is None
case=Path(r['case']);cr=read(case/'receipt.json');runtime=Path(cr['runtime']);assert cr['test_exit_code']==101 and cr.get('server_exit_code') is None and cr['error']=='Server cleanup did not complete'
captures=list((runtime/'captures').glob('*/frames.jsonl'));assert len(captures)==1
frames=[json.loads(line) for line in captures[0].read_text('utf-8-sig').splitlines()]
raw=[bytes.fromhex(f['plain']) for f in frames if f['direction']=='client_to_server' and re.fullmatch('[a-fA-F0-9 ]*',f.get('plain',''))]
assert not any(body==b'\x1f\x44' for body in raw)
npc_actions=[b'\x27'+(1812987996).to_bytes(4,'little')+name.encode('ascii')+bytes(2) for name in ['LT_TELEPORT_B','LT_TELEPORT_C']]
assert not any(body in npc_actions for body in raw)
journals={role:read(runtime/(role+'-presentation-journal.json')) for role in ['a','b']}
first_frames={role:next((row['ms'] for row in j['trace'] if row['state']['frame'] is not None),None) for role,j in journals.items()}
assert first_frames['a'] is None and first_frames['b']==28914
for role,j in journals.items():
    assert j['error']=='existing phase bound at BattlePending' and j['stage']=='initial'
    shutdown=read(runtime/(role+'-presentation-shutdown.json'));assert shutdown['client_quiescent'] and shutdown['Presentation_quiescent'] and shutdown['shutdown_error'] is None
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,h in c['tool_pins'].items():assert sha(n)==h
release=[];jobs=c['process_history']+[read(run/'runner-identity.json')]
for key,entry in c['cases'].items():
    child=Path(entry['run']);rr=read(child/'receipt.json');assert sha(child/'receipt.json')==entry['receipt_sha256'] and rr['status']=='executed_current_normal_body_nonempty_audio_case'
    release.append(dict(key=key,run=str(child),actual_sessions=rr['actual_sessions'],child_exit_code=0,
        retained_independent_qualification=entry.get('retained_qualification'),independently_qualified=bool(entry.get('retained_qualification'))))
    cc=read(Path(rr['case'])/'receipt.json');bb=read(Path(rr['compile'])/'receipt.json')
    jobs+=rr['process_history']+[read(child/'runner-identity.json'),bb['original_runner']]+bb['process_history']+[dict(pid=j['pid'],creation_utc=j['start_utc']) for j in cc['owned_processes']]
bb=read(Path(r['compile'])/'receipt.json');jobs+=r['process_history']+[read(failed/'runner-identity.json'),bb['original_runner']]+bb['process_history']+[dict(pid=j['pid'],creation_utc=j['start_utc']) for j in cr['owned_processes']]
unique={(j['pid'],datetime.fromisoformat(j['creation_utc'].replace('Z','+00:00')).astimezone(timezone.utc)):j for j in jobs};assert len(unique)==43,len(unique)
ids=','.join(str(pid) for pid,_ in unique)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw_cim=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw_cim.decode('utf-8-sig')) if raw_cim.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for(pid,born),j in unique.items():
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-born).total_seconds())>.001
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('failure-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'recorded-process-CIM.json').write_bytes(raw_cim);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
files={}
for family,folder in [('cohort',run),('failed-driver',failed),('failed-case',case),('failed-runtime',runtime)]:
    for p in folder.rglob('*'):
        if p.is_file() and not p.is_relative_to(out) and p.suffix.lower() not in {'.exe','.class','.jar','.zip'}:files[family+'/'+p.relative_to(folder).as_posix()]=p
raw_pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'failure-raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'failure-raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in raw_pins.items())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='failed_Debug_initial_Presentation_readiness_before_any_source_audio_or_handoff_request',
    source_manifest_sha256=c['source_manifest_sha256'],diagnostic_input_manifest_sha256=c['diagnostic_input_manifest_sha256'],
    cohort=str(run),failed_run=str(failed),failed_case=str(case),failed_runtime=str(runtime),original_session_id=a.session,
    original_terminal_exit_code=1,terminal_tool_chunk=a.terminal_chunk,failed_rust_test_exit_code=101,
    original_server_exit_code=None,original_server_exit_code_field_present='server_exit_code' in cr,original_server_cleanup_error=cr['error'],server_identity_observed_terminal=True,
    first_published_frame_from_peer_start_ms=first_frames,existing_first_walk_ready_bound_seconds=8,
    actual_C1F_requests=0,actual_NPC_teleport_requests=0,captured_masked_authentication_frames=sum(f['plain']=='MASKED_C_LOGIN' for f in frames),
    both_Client_and_Presentation_quiescent=True,recorded_owned_identities=43,recorded_owned_all_terminal=True,
    completed_Release_cases=release,completed_Release_sessions=48,Release_independently_qualified_cases=1,
    normal_product_source_modified=False,normal_related_130_each_and_controller_96_sessions_qualifications_retained=True,
    failed_case_audio_retirement_qualification_granted=False,whole_cohort_qualified=False,
    raw_archive_sha256=sha(out/'failure-raw.zip'),raw_files=raw_pins,
    next='Profile initial two-peer asynchronous scene load/draw and source-owner waits under the existing 8-second bound. Independently read the completed Release children using their recorded exit0 while explicitly retaining the parent exit1; do not rerun them or claim this cohort passed.',
    GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'failure.json',record)
work='來源8CC5A560音效整批原'+str(a.session)+'已實際退出1：Release六案48sessions執行完成，僅Release拒絕8sessions已獨立資格，其餘待按各子程序exit0讀回，不冒充整批成功。Debug拒絕在初始Frame就緒8秒界限失敗，未送任何C1F或NPC傳送；A無Frame，B首次Frame為Peer啟動後28914ms。Rust101，原伺服器退出碼未記錄且cleanup失敗；目前43記錄程序身分皆已終止，兩端Client／Presentation靜止，不能回填server exit0。正常各130與控制器96sessions限定資格維持。下一步定位初始場景工作／來源擁有者等待及首張繪圖成本，保留Release成果、不重跑或提高8／60秒／G11預算。正常GUI／實體音效、其他分支與完整G01–G12未完成；相鄰只讀且產物獨立。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        x['current_native8CC5_audio_cohort_failure']=dict(path=str(out/'failure.json'),sha256=sha(out/'failure.json'),source_manifest_sha256=c['source_manifest_sha256'])
        x['current_native8CC5_audio_cohort_active'].update(status='failed_preserved_Debug_initial_readiness',original_terminal_exit_code=1,terminal_tool_chunk=a.terminal_chunk,failure=str(out/'failure.json'),failure_sha256=sha(out/'failure.json'))
        x.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,
            last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'failure.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='Debug-initial-scene-readiness-and-Release-evidence-readback',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),failure_sha256=sha(out/'failure.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
for folder in [run.parent,V/'native8CC5-teleport-audio']:
    path=folder/'current-continuation.json';previous=read(path);(out/(folder.name+'-current-before.json')).write_bytes(path.read_bytes())
    previous.update(status='failed_preserved_Debug_initial_readiness_Release_completed_results_retained',original_session_id=a.session,
        original_terminal_exit_code=1,terminal_tool_chunk=a.terminal_chunk,must_poll_original_session_before_restarting=False,
        failure_checkpoint=str(out),latest_readback=str(out/'readback.json'),failed_run=str(failed),completed_Release_cases=release,
        next_action=record['next'],whole_cohort_qualified=False)
    save(path,previous)
print(json.dumps(dict(checkpoint=str(out),original_exit_code=1,terminal_identities=43,actual_failed_case_C1F=0,completed_Release_sessions=48,whole_goal_complete=False),ensure_ascii=False))
