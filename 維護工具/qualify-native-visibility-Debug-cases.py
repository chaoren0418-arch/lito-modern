"""Bind eight completed cases, immutable normal build, captures and owned exit."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int)
ap.add_argument('--terminal-exit-code',required=True,type=int);ap.add_argument('--tool-chunk-id',required=True);a=ap.parse_args()
assert a.session_id>0 and a.terminal_exit_code==0 and a.tool_chunk_id
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
c=read(run/'cohort.json');profile=c.get('build_profile','Debug');assert profile in ['Debug','Release']
assert c['status']==f'executed_normal_{profile}_visibility_eight_cases' and c['owned_process'] is None
assert len(c['cases'])==8 and c['actual_client_sessions']==32 and c['actual_cast_requests']==14
preflight=Path(c['preflight']);pq=read(preflight/'qualified/qualified.json');assert sha(preflight/'qualified/qualified.json')==c['preflight_qualification_sha256']
assert pq['source_manifest_sha256']==c['source_manifest_sha256'] and pq['normal_sha256']==c['normal_sha256']
pins=read(preflight/'compiled-inputs.json');assert len(pins)==1025
for n,h in pins.items():assert sha(ROOT/n)==h
for n,h in c['helper_pins'].items():assert sha(run/n)==sha(ROOT/'維護工具'/n)==h
expected={(resource,profile) for resource in ['tw13081901-v1','lito-legacy'] for profile in
    ['detection-invisibility','detection-refused','invisibility-refused','invisibility-missing']}
assert {(v['resource'],v['profile']) for v in c['cases']}==expected
jobs=[];files={'runner.py':run/'runner.py','cohort.json':run/'cohort.json','preflight-qualified.json':preflight/'qualified/qualified.json'}
retained=c.get('retained_completed_case_from_failed_batch')
if retained:
    previous=Path(c['cases'][0]['output']).parent
    path=previous/'failure-before-encoding-reader-repair/readback.json'
    assert read(path)==retained and retained['original_session_id']==61582 and retained['observed_terminal_exit_code']==1
    assert retained['first_case_wrapper_exit_code']==0 and sha(path.with_name('raw.zip'))==retained['raw_archive_sha256']
    files['retained-failed-batch-readback.json']=path
cases=[]
for i,case in enumerate(c['cases']):
    assert case['status']=='executed_and_independently_compared' and case['wrapper']['exit_code']==0
    jobs.append(case['wrapper']);folder=Path(case['case']).resolve(strict=True)
    assert folder.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers')
    receipt=read(folder/'receipt.json');comparison=read(folder/'visibility-comparison.json')
    assert sha(folder/'receipt.json')==case['receipt_sha256']==comparison['receipt_sha256']
    assert sha(folder/'visibility-comparison.json')==case['comparison_sha256']
    assert receipt['exe_sha256']==c['normal_sha256'] and comparison['source_manifest_sha256']==c['source_manifest_sha256']
    assert receipt['server']['actual_client_sessions']==4 and receipt['server']['authority_mutations']==0
    assert receipt['server']['memberships_empty'] and not receipt['server']['sql_access'] and not receipt['server']['state_persistent']
    assert comparison['actual_cast_requests']==(4 if case['profile']=='detection-invisibility' else 1)
    logs=(folder/'server.log').read_text(encoding='utf-8-sig').splitlines()
    spells=[line for line in logs if 'C_USE_SPELL' in line]
    if case['profile']=='detection-invisibility':
        hide=[line for line in spells if 'C_USE_SPELL invisibility skill=60 ' in line]
        detect=[line for line in spells if 'C_USE_SPELL Detection skill=13 ' in line]
        assert len(spells)==4 and len(hide)==len(detect)==2
        assert all('mp-cost=15 ' in line for line in spells)
        assert all('explicitly-revealed=1 damage-targets=0 ' in line for line in detect)
        for client in receipt['clients'].values():assert client['start']['world']['mp'][0]-client['end']['world']['mp'][0]==30
        wire_paths=list((Path(receipt['runtime'])/'captures').glob('*/frames.jsonl'));assert len(wire_paths)==1
        ticks=[]
        for line in wire_paths[0].read_text(encoding='utf-8-sig').splitlines():
            row=json.loads(line)
            if row['direction']=='client_to_server' and row.get('plain','').startswith('14 07 03 '):
                body=bytes.fromhex(row['plain']);assert len(body)==7;ticks.append(int.from_bytes(body[3:],'little'))
        assert ticks==[int(re.search(r' client-tick=(\d+) ',line)[1]) for line in hide]
    else:
        assert len(spells)==1
        expected_reason={'detection-refused':'Detection rejected state phase=BATTLE mp=2 mp-cost=15 hp=100 cooldownNanos=0',
            'invisibility-refused':'invisibility rejected state skill=60 mp=2 invisible=false cost=15 cooldownNanos=0',
            'invisibility-missing':'invisibility rejected missing reagent catalog=40318'}[case['profile']]
        assert expected_reason in spells[0]
    binding=read(folder/'validation-completed.json');assert binding['receipt_sha256']==case['receipt_sha256']
    for n,h in binding['comparison_pins'].items():assert sha(folder/n)==h
    for n in ['a-launch-identity.json','b-launch-identity.json','server-launch-identity.json']:jobs.append(read(folder/n))
    runtime=Path(receipt['runtime']).resolve(strict=True)
    assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current/native-real-peers') and sha(runtime/'qualified-client.exe')==c['normal_sha256']
    for p in folder.rglob('*'):
        if p.is_file():files[f'case{i}/output/'+p.relative_to(folder).as_posix()]=p
    for p in runtime.rglob('*'):
        if p.is_file() and p.name!='qualified-client.exe':files[f'case{i}/runtime/'+p.relative_to(runtime).as_posix()]=p
    for n in ['stdout.log','stderr.log']:files[f'case{i}/{n}']=Path(case['output'])/n
    assert not (Path(case['output'])/'stderr.log').read_bytes().strip()
    cases.append(dict(profile=case['profile'],resource=case['resource'],case=str(folder),comparison_sha256=case['comparison_sha256'],
        actual_client_sessions=4,actual_cast_requests=comparison['actual_cast_requests'],accepted_casts=4 if case['profile']=='detection-invisibility' else 0))
assert len(jobs)==32
command='$ids=@('+','.join(str(j['pid']) for j in jobs)+');@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n="created";e={$_.CreationDate.ToUniversalTime().ToString("o")}})|ConvertTo-Json -Compress'
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/'qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'CIM.json').write_bytes(p.stdout)
visual=read(run/'visual-observations.json');assert visual['source_manifest_sha256']==c['source_manifest_sha256'] and len(visual['images'])==4
for row in visual['images']:assert row['viewed'] and sha(Path(row['path']))==row['sha256']
files['visual-observations.json']=run/'visual-observations.json'
for n in c['helper_pins']:files['helpers/'+n]=run/n
files.update({'qualifier.py':out/'qualifier.py','CIM.json':out/'CIM.json'})
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
    for n,h in hashes.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
q=dict(status=f'independently_qualified_normal_{profile}_visibility_eight_cases',profile=profile,original_session_id=a.session_id,
    observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,source_manifest_sha256=c['source_manifest_sha256'],
    normal_sha256=c['normal_sha256'],actual_client_sessions=32,actual_cast_requests=14,accepted_casts=8,
    retained_completed_case_from_failed_batch=retained,
    negative_cases=6,cases=cases,recorded_owned_identities=32,recorded_owned_all_terminal=True,
    visually_inspected_diagnostic_PNGs=4,
    source_and_helpers_unchanged=True,raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,
    Release_verified=profile=='Release',GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({k:q[k] for k in ['status','actual_client_sessions','actual_cast_requests','accepted_casts','negative_cases','raw_archive_sha256']},ensure_ascii=False))
