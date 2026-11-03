"""Seal all twelve Presentation cases only after the original cohort tool exits."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,subprocess,sys,time,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
assert a.terminal_exit_code==0 and a.terminal_chunk not in ['','pending','unknown']
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-presentation-cohort')
c=read(run/'cohort.json');assert c['status']=='executed_presentation_cohort_qualification_pending' and c['owned_process'] is None
assert c['actual_sessions']==96 and len(c['cases'])==12 and len(c['process_history'])==16 and all(p['exit_code']==0 for p in c['process_history'])
jobs=c['process_history']+[read(run/'runner-identity.json')];ids=','.join(str(p['pid']) for p in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
result=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert result.returncode==0 and not result.stderr
live=json.loads(result.stdout.decode('utf-8-sig')) if result.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,h in c['tool_pins'].items():assert sha(n)==h
out=run/'qualified';out.mkdir(exist_ok=True);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'CIM.json').write_bytes(result.stdout)
index=out/'case-qualifications.json';proofs=read(index) if index.exists() else {}
for key,case in c['cases'].items():
    child=Path(case['run']);assert sha(child/'receipt.json')==case['receipt_sha256']
    if key not in proofs:
        family='teleport-presentation' if case['scenario']=='cross-endpoint' else 'teleport-failure-presentation'
        command=[sys.executable,'-B',str(ROOT/'維護工具'/('qualify-native1040-'+family+'.py')),str(child),'--session-id',str(a.session_id),'--terminal-chunk',a.terminal_chunk]
        p=subprocess.run(command,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
        (out/(key+'-stdout.log')).write_bytes(p.stdout);(out/(key+'-stderr.log')).write_bytes(p.stderr);assert p.returncode==0,(key,'case reader failed; preserve all runs')
        paths=[s[len('QUALIFICATION='):] for s in p.stdout.decode('utf-8-sig').splitlines() if s.startswith('QUALIFICATION=')];assert len(paths)==1
        qp=Path(paths[0]);proofs[key]=dict(path=str(qp),sha256=sha(qp));save(index,proofs)
    proof=proofs[key];qp=Path(proof['path']);q=read(qp);assert sha(qp)==proof['sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['profile']==case['profile']
    assert q['actual_sessions']==case['actual_sessions'] and q['actual_Presentation_executed'] and q['recorded_owned_all_terminal']
    assert q['original_session_id']==a.session_id and q['terminal_tool_chunk']==a.terminal_chunk and q['original_exit_code']==0
    assert not q['GUI_verified'] and not q['normal_EXE_runtime_inferred'] and not q['physical_audio_verified']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in q['raw_files'].items())
    print(key+' independently qualified',flush=True)
for n,h in inputs.items():assert sha(ROOT/n)==h
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and 'qualified' not in p.relative_to(run).parts};files['reader.py']=Path(__file__)
pins={n:sha(p) for n,p in files.items()};archive=out/'cohort-raw.zip';assert not archive.exists()
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_current1040_Presentation_both_profiles_six_branches',source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session_id,terminal_tool_chunk=a.terminal_chunk,original_exit_code=0,actual_sessions=96,actual_cases=12,actual_diagnostic_executables=4,cases=proofs,normal_mains=c['normal_mains'],recorded_cohort_identities=17,recorded_cohort_identities_all_terminal=True,actual_Presentation_executed=True,unplayed_source_audio_rejection_verified=False,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False,raw_archive_sha256=sha(archive),raw_files=pins)
save(out/'qualified.json',q);print('COHORT_QUALIFICATION='+str(out/'qualified.json'),flush=True)
