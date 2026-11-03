"""Independently qualify all twelve branches after the observed original terminal."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,sys,time,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for attempt in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
assert a.session>0 and a.terminal_exit_code==0 and re.fullmatch('[a-f0-9]+',a.terminal_chunk)
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native8CC5-audio-cohort')
c=read(run/'cohort.json');assert c['status']=='executed_current_normal_body_audio_cohort_qualification_pending' and c['owned_process'] is None
assert len(c['cases'])==12 and c['actual_sessions']==96 and len(c['process_history'])==11 and all(j['exit_code']==0 for j in c['process_history'])
assert set(c['cases'])=={p+'-'+s for p in ['Debug','Release'] for s in ['cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-expiry']}
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,h in c['tool_pins'].items():assert sha(n)==h
out=run/'qualified';out.mkdir(exist_ok=True);assert not(out/'qualified.json').exists()
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
index=out/'case-qualifications.json';proofs=read(index) if index.exists() else {}
readers=[];runtime_jobs=c['process_history']+[read(run/'runner-identity.json')];products=set();normal_mains={};retained_count=0
retired_cues=0;wave_requests=0;recovery_wave_deliveries=0;native_heal_deliveries=0
for key,entry in c['cases'].items():
    child=Path(entry['run']);assert sha(child/'receipt.json')==entry['receipt_sha256']
    if key not in proofs:
        if entry.get('retained_without_rerun'):
            qp=Path(entry['retained_qualification']);assert sha(qp)==entry['retained_qualification_sha256']
        else:
            existing=list(child.glob('qualified-*/qualified.json'))
            if existing:assert len(existing)==1;qp=existing[0]
            else:
                command=[sys.executable,'-B','-X','utf8',str(ROOT/'維護工具/qualify-native8CC5-teleport-audio.py'),str(child),'--session-id',str(a.session),'--terminal-chunk',a.terminal_chunk,'--terminal-exit-code','0']
                result=subprocess.run(command,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
                (out/(key+'-stdout.log')).write_bytes(result.stdout);(out/(key+'-stderr.log')).write_bytes(result.stderr)
                readers.append(dict(case=key,command=command,exit_code=result.returncode));save(out/'reader-jobs.json',readers)
                assert result.returncode==0,(key,'Preserve all actual runs and this independent reader failure')
                paths=[line[len('QUALIFICATION='):] for line in result.stdout.decode('utf-8-sig').splitlines() if line.startswith('QUALIFICATION=')];assert len(paths)==1;qp=Path(paths[0])
        proofs[key]=dict(path=str(qp),sha256=sha(qp));save(index,proofs)
    qp=Path(proofs[key]['path']);q=read(qp);assert sha(qp)==proofs[key]['sha256']
    assert q['status']=='independently_qualified_current_normal_bodies_nonempty_audio_branch' and q['profile']==entry['profile'] and q['scenario']==entry['scenario']
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['diagnostic_input_manifest_sha256']==c['diagnostic_input_manifest_sha256']
    assert q['actual_sessions']==entry['actual_sessions'] and q['retained_source_cues']==2 and q['unplayed_source_audio_rejection_verified'] and q['new_scene_own_sound_events_delivered_once_each']
    assert q['actual_Presentation_executed'] and q['recorded_owned_all_terminal'] and q['normal_source_bodies_unchanged'] and not q['source_overlays_used']
    assert q['diagnostic_request_API_extension'] and not q['normal_GUI_API_permission_inferred'] and not q['normal_EXE_runtime_inferred'] and not q['GUI_verified'] and not q['physical_audio_verified']
    assert len(q['wire_audio_proofs'])==2
    retired_cues+=q['retained_source_cues']
    for proof in q['wire_audio_proofs']:
        assert proof['old_first']=='Stale' and proof['old_repeat']=='AlreadyConsumed' and proof['old_deliveries']==0
        assert proof['source']['request_line']>0 and proof['source']['S20_lines'];wave_requests+=1
        if q['scenario']=='cross-endpoint':
            assert proof['new_audio']['kind']=='native_heal1_sound109' and proof['new_audio']['delivered_once'];native_heal_deliveries+=1
        else:
            assert proof['new_audio']['request_line']>0 and proof['new_audio']['S20_lines'];wave_requests+=1;recovery_wave_deliveries+=1
    if entry.get('retained_without_rerun'):
        retained_count+=1;assert q['original_session_id']==entry['retained_original_session_id'] and q['terminal_tool_chunk']==entry['retained_terminal_tool_chunk'] and key=='Release-refused'
    else:assert q['original_session_id']==a.session and q['terminal_tool_chunk']==a.terminal_chunk
    assert q['original_exit_code']==0 and sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:
        assert set(z.namelist())==set(q['raw_files'])
        for n,h in q['raw_files'].items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    rr=read(child/'receipt.json');b=read(Path(rr['compile'])/'receipt.json');cr=read(Path(rr['case'])/'receipt.json')
    runtime_jobs+=rr['process_history']+[read(child/'runner-identity.json'),b['original_runner']]+b['process_history']+[dict(pid=j['pid'],creation_utc=j['start_utc']) for j in cr['owned_processes']]
    products.add(q['actual_product_sha256'])
    normal=read(Path(b['normal_cohort'])/'cohort.json')
    for profile,info in normal['profiles'].items():
        assert sha(info['normal']['path'])==info['normal']['sha256']
        if profile in normal_mains:assert normal_mains[profile]==info['normal']['sha256']
        normal_mains[profile]=info['normal']['sha256']
    print(key+' independently qualified',flush=True)
assert retained_count==1 and len(products)==4 and set(proofs)==set(c['cases'])
assert(retired_cues,wave_requests,recovery_wave_deliveries,native_heal_deliveries)==(24,44,20,4)
unique={}
for j in runtime_jobs:
    if 'exit_code' in j:assert j['exit_code']==0
    born=datetime.fromisoformat(j['creation_utc'].replace('Z','+00:00')).astimezone(timezone.utc)
    unique[(j['pid'],born)]=j
assert len(unique)==69,len(unique)
ids=','.join(str(pid) for pid,_ in unique)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
result=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert result.returncode==0 and not result.stderr
(out/'recorded-runtime-CIM.json').write_bytes(result.stdout)
live=json.loads(result.stdout.decode('utf-8-sig')) if result.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for(pid,born),j in unique.items():
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-born).total_seconds())>.001
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,h in c['tool_pins'].items():assert sha(n)==h
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and not p.is_relative_to(out)}
files['qualifier.py']=Path(__file__);files['case-qualifications.json']=index
pins={n:sha(p) for n,p in files.items()};archive=out/'cohort-raw.zip';assert not archive.exists()
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_current_normal_bodies_nonempty_audio_both_profiles_six_branches',
    source_manifest_sha256=c['source_manifest_sha256'],diagnostic_input_manifest_sha256=c['diagnostic_input_manifest_sha256'],
    original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,actual_cases=12,actual_sessions=96,
    actual_diagnostic_executables=4,retained_qualified_cases_without_rerun=1,newly_executed_cases=11,cases=proofs,normal_mains=normal_mains,
    actual_Presentation_executed=True,retained_source_cues=retired_cues,unplayed_source_audio_rejection_verified=True,
    actual_C1F_wave_requests=wave_requests,new_scene_sound_events_delivered_once=recovery_wave_deliveries+native_heal_deliveries,
    recovered_ordinary_wave_deliveries=recovery_wave_deliveries,destination_native_heal_deliveries=native_heal_deliveries,
    source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,normal_product_source_modified=False,
    normal_GUI_API_permission_inferred=False,recorded_original_owned_identities=len(unique),recorded_original_owned_processes_all_terminal=True,
    original_runtime_identities=list(unique.values()),same_World_new_Model_ABA_event_admission_verified=False,
    normal_EXE_runtime_inferred=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,
    same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False,raw_archive_sha256=sha(archive),raw_files=pins)
save(out/'qualified.json',q);print('COHORT_QUALIFICATION='+str(out/'qualified.json'));print(json.dumps(dict(actual_cases=12,actual_sessions=96,old_cues_rejected=24,new_events_delivered_once=24,terminal_runtime_identities=len(unique)),ensure_ascii=False))
