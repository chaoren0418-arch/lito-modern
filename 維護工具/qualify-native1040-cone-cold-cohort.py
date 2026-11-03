"""Independently seal six cold source-body cases after the original terminal."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,sys,time,zipfile
ROOT=Path(__file__).resolve().parents[1]
T=ROOT/'維護工具'
def read(path):
    for attempt in range(10):
        try:return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
parser=argparse.ArgumentParser();parser.add_argument('run',type=Path);parser.add_argument('--session-id',type=int,required=True);parser.add_argument('--terminal-chunk',required=True);parser.add_argument('--terminal-exit-code',type=int,required=True);args=parser.parse_args()
assert args.session_id>0 and args.terminal_exit_code==0 and args.terminal_chunk not in ['','pending','unknown']
run=args.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-cone-cold-cohort')
c=read(run/'cohort.json')
assert c['status']=='executed_current_cold_C02_cohort_qualification_pending' and c['owned_process'] is None
assert (len(c['cases']),c['actual_sessions'],c['actual_requests'],c['actual_accepted_casts'])==(6,24,10,8)
assert len(c['process_history'])==6 and all(job['exit_code']==0 for job in c['process_history'])
for name,digest in read(run/'compiled-inputs.json').items():assert sha(ROOT/name)==digest,name
for name,digest in c['tool_pins'].items():assert sha(name)==digest,name
records=[c['runner']]+c['process_history']
for name,path in c['builds'].items():
    build=Path(path);b=read(build/'receipt.json');assert sha(build/'receipt.json')==c['build_receipt_pins'][name]
    assert b['status']=='compiled_current1040_cold_source_diagnostic' and b['compiler']['exit_code']==0
    assert b['normal_sources_body_compiled'] and not b['source_overrides_used'] and b['source_manifest_sha256']==c['source_manifest_sha256']
    records += [b['original_runner'],b['compiler']]
for case in c['cases'].values():
    r=read(Path(case['run'])/'receipt.json');assert sha(Path(case['run'])/'receipt.json')==case['receipt_sha256'] and case['actual_runner_exit_code']==0
    assert r['all_started_processes_terminal'] and r['asset_metadata_unchanged']
    records += r['process_history']+[read(Path(case['run'])/'original-runner-process.json')]
unique={(job['pid'],job['creation_utc']):job for job in records}
ids=','.join(str(pid)for pid in sorted({pid for pid,_ in unique}))
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
result=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert result.returncode==0 and not result.stderr
live=json.loads(result.stdout.decode('utf-8-sig'))if result.stdout.strip()else[];live=[live]if isinstance(live,dict)else live
for row in live:
    for pid,created in unique:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())>.001
out=run/'qualified';out.mkdir(exist_ok=True);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'CIM.json').write_bytes(result.stdout)
index=out/'case-qualifications.json';proofs=read(index)if index.exists()else{}
for key,case in c['cases'].items():
    if key not in proofs:
        command=[sys.executable,'-B',str(T/'qualify-native1040-cone-cold-live.py'),case['run'],'--session',str(args.session_id),'--terminal-chunk',args.terminal_chunk]
        child=subprocess.run(command,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
        (out/(key+'-stdout.log')).write_bytes(child.stdout);(out/(key+'-stderr.log')).write_bytes(child.stderr)
        assert child.returncode==0,(key,'retain original cases; independent reader failed')
        paths=[line[len('QUALIFICATION='):]for line in child.stdout.decode('utf-8-sig').splitlines()if line.startswith('QUALIFICATION=')];assert len(paths)==1
        qp=Path(paths[0]);proofs[key]=dict(path=str(qp),sha256=sha(qp));save(index,proofs)
    proof=proofs[key];qp=Path(proof['path']);q=read(qp);assert sha(qp)==proof['sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['compile_profile']==case['profile']
    assert q['actual_sessions']==4 and q['recorded_owned_all_terminal'] and q['normal_sources_body_compiled'] and not q['source_overrides_used']
    assert q['original_session_id']==args.session_id and q['original_exit_code']==0 and q['terminal_tool_chunk']==args.terminal_chunk
    assert not q['normal_EXE_runtime_inferred'] and not q['GUI_verified'] and not q['physical_audio_verified']
    assert q['actual_requests']==(1 if case['scenario']=='refused'else 2) and q['actual_accepted_casts']==(0 if case['scenario']=='refused'else 2)
    if case['scenario']=='moving' and 'movement' not in proof:
        child=subprocess.run([sys.executable,'-B',str(T/'qualify-native1040-cone-cold-moving.py'),str(qp)],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
        (out/(key+'-moving-stdout.log')).write_bytes(child.stdout);(out/(key+'-moving-stderr.log')).write_bytes(child.stderr)
        assert child.returncode==0,(key,'retain movement reader failure')
        lines=child.stdout.decode('utf-8-sig').splitlines();assert len(lines)==1
        mp=Path(json.loads(lines[0])['qualification']);proof['movement']=dict(path=str(mp),sha256=sha(mp));save(index,proofs)
    print(key+' independently qualified',flush=True)
for name,digest in read(run/'compiled-inputs.json').items():assert sha(ROOT/name)==digest,name
files={path.relative_to(run).as_posix():path for path in run.rglob('*')if path.is_file()and'qualified'not in path.relative_to(run).parts};files['reader.py']=Path(__file__)
pins={name:sha(path)for name,path in files.items()};archive=out/'cohort-raw.zip';assert not archive.exists()
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED)as z:
    for name,path in files.items():z.write(path,name)
with zipfile.ZipFile(archive)as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
qualified=dict(status='independently_qualified_current1040_cold_C02_six_source_body_cases',source_manifest_sha256=c['source_manifest_sha256'],original_session_id=args.session_id,original_exit_code=0,terminal_tool_chunk=args.terminal_chunk,actual_sessions=24,actual_cases=6,actual_diagnostic_executables=4,actual_requests=10,actual_accepted_casts=8,actual_refused_requests=2,cases=proofs,recorded_owned_identities=len(unique),recorded_owned_all_terminal=True,recorded_identities=[dict(pid=pid,creation_utc=created)for pid,created in unique],PID_reuse_candidates=live,normal_sources_body_compiled=True,source_overrides_used=False,normal_EXE_runtime_inferred=False,GUI_verified=False,physical_audio_verified=False,raw_archive_sha256=sha(archive),raw_files=pins,full_G06_verified=False,whole_goal_complete=False)
save(out/'qualified.json',qualified);print('COHORT_QUALIFICATION='+str(out/'qualified.json'),flush=True)
