"""Run only after the original both-profile handle reports an actual exit0."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def verify(path,digest,files):
    assert sha(Path(path))==digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(files) and set(z.namelist())==set(files)
        for name,h in files.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
ap=argparse.ArgumentParser();ap.add_argument('cohort',type=Path);ap.add_argument('--session-id',required=True,type=int);ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
out=a.cohort.resolve(strict=True);assert out.is_relative_to(ROOT/'驗證紀錄/current/native-default110') and a.terminal_exit_code==0
c=read(out/'cohort.json');assert c['status']=='independently_qualified_current_default110_both_profiles' and c['owned_process'] is None
assert set(c['profiles'])=={'Debug','Release'} and all(p['exit_code']==0 for p in c['process_history'])
normal_path=Path(c['normal_qualification']);n=read(normal_path)
normal_build=normal_path.parent.parent if c.get('normal_input_kind','startup')=='startup' else normal_path.parent
pins=read(normal_build/'source-inputs.json');assert len(pins)==1024
for name,h in pins.items():assert sha(ROOT/name)==h
normal=read(normal_build/'cohort.json')
for profile,info in normal['profiles'].items():assert sha(info['normal']['path'])==n['profiles'][profile]['normal_sha256']
assert sha(ROOT/'啟動現代客戶端.ps1')==n['entry_sha256']
identities=[(c['pipeline_pid'],c['pipeline_creation_utc'])]+[(p['pid'],p['creation_utc']) for p in c['process_history']]
profiles={}
for profile,info in c['profiles'].items():
    run=Path(info['run']);q=read(info['qualification']);r=read(run/'receipt.json')
    assert info['qualified'] and info['actual_runner_exit_code']==0 and sha(Path(info['qualification']))==info['qualification_sha256']
    assert q['status']=='independently_qualified_current_default110_integrations_and_main' and q['actual_test_executables']==111 and q['integration_targets']==110
    assert q['counts']==info['counts'] and q['counts']['failed']==0 and q['source_manifest_sha256']==c['source_manifest_sha256']
    assert r['owned_process'] is None and all(p['exit_code']==0 and not p['timed_out'] for p in r['process_history'])
    identities.extend((p['pid'],p['creation_utc']) for p in r['process_history'])
    inputs=read(run/'compiled-inputs.json');assert len(inputs)==1025 and inputs==pins|read(run/'fixture-inputs.json')
    assert (run/'source-inputs.json').read_bytes()==(normal_build/'source-inputs.json').read_bytes()
    verify(run/'compiled-source.zip',q['compiled_source_archive_sha256'],inputs)
    products=read(run/'product-retention.json');verify(products['archive'],q['product_archive_sha256'],{name+'.exe':v['sha256'] for name,v in products['products'].items()})
    for product in products['products'].values():assert sha(Path(product['path']))==product['sha256']
    verify(Path(info['qualification']).parent/'raw.zip',q['raw_archive_sha256'],q['raw_files'])
    profiles[profile]=dict(qualification=info['qualification'],qualification_sha256=info['qualification_sha256'],counts=q['counts'],actual_test_executables=111,raw_archive_sha256=q['raw_archive_sha256'])
identities=sorted(set(identities));ids=','.join(str(pid) for pid in sorted({pid for pid,_ in identities}))
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for pid,created in identities:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(created)).total_seconds())>0.001
dest=out/'completion-observation.json';assert not dest.exists();(out/'completion-CIM.json').write_bytes(p.stdout);(out/'completion-observer.py').write_bytes(Path(__file__).read_bytes())
result=dict(status='independently_observed_current_default110_both_profiles_and_terminal_original_pipeline',original_session_id=a.session_id,observed_terminal_exit_code=0,profiles=profiles,source_files=1024,compiled_input_files=1025,source_manifest_sha256=c['source_manifest_sha256'],normal_mains={profile:v['normal_sha256'] for profile,v in n['profiles'].items()},recorded_original_owned_identities=len(identities),recorded_original_owned_processes_all_terminal=True,PID_reuse_candidates=live,GUI_verified=False,full_library_scope_inferred=False,ignored_bodies_executed=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
dest.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(completion=str(dest),profiles={p:v['counts'] for p,v in profiles.items()},owned_identities=len(identities)),ensure_ascii=False))
