"""Read back completed library profiles and actual original child exit receipt."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def archive(path,digest,files):
    assert sha(path)==digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(files) and set(z.namelist())==set(files)
        for name,digest in files.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--launch-exit',type=Path,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-startup-library-default')
launch=a.launch_exit.resolve(strict=True);assert launch.is_relative_to(run.parent)
terminal=read(launch);assert terminal['observed_terminal_exit_code']==0
c=read(run/'cohort.json');assert c['status']=='executed_both_current_full_library_default_selections'
assert c['owned_process'] is None and len(c['process_history'])==8 and all(p['exit_code']==0 for p in c['process_history'])
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1029
assert sha(run/'source-inputs.json')==c['source_manifest_sha256'] and len(read(run/'source-inputs.json'))==1028
for name,digest in inputs.items():assert sha(ROOT/name)==digest
archive(c['compiled_source_archive'],c['compiled_source_archive_sha256'],inputs)
npath=Path(c['normal_qualification']);assert sha(npath)==c['normal_qualification_sha256']
n=read(npath);assert sha(ROOT/'啟動現代客戶端.ps1')==n['entry_sha256']
original=read(run/'original-runner-process.json')
binding=read(run/'original-session-binding.json')
assert binding['runner_identity']==original
assert binding['original_session_id']==terminal['original_session_id']
identities=[(original['pid'],original['creation_utc'])]+[(p['pid'],p['creation_utc']) for p in c['process_history']]
profiles={}
for profile in ['Debug','Release']:
    qpath=run/profile/'full-default-qualified/qualified.json';q=read(qpath)
    assert q['status']=='independently_qualified_current_full_library_default_selection'
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['failed']==q['filtered_out']==0
    archive(qpath.parent/'raw.zip',q['raw_archive_sha256'],q['raw_files'])
    product=read(run/profile/'product-retention.json')
    assert sha(product['product'])==q['actual_test_sha256']
    archive(product['archive'],q['actual_test_archive_sha256'],{'lito_modern.exe':q['actual_test_sha256']})
    r=read(run/profile/'runtime-full-current/receipt.json');identities.append((r['pid'],r['creation_utc']))
    profiles[profile]=dict(qualification=str(qpath),qualification_sha256=sha(qpath),
        passed=q['passed'],failed=q['failed'],ignored=q['ignored'],inventory=q['inventory'],actual_test_sha256=q['actual_test_sha256'])
base=Path(c['input_cohort'])
normal_root=(npath.parent.parent if npath.parent.name=='qualified' else npath.parent) if c['input_kind']=='default111' else base
normal=read(normal_root/'cohort.json')
for profile,info in normal['profiles'].items():assert sha(info['normal']['path'])==c['normal_mains'][profile]
identities=sorted(set(identities));ids=','.join(str(pid) for pid,_ in identities)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for pid,created in identities:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(created)).total_seconds())>.001
assert not (run/'completion-observation.json').exists()
(run/'completion-CIM.json').write_bytes(p.stdout)
files={'observer.py':Path(__file__),'cohort.json':run/'cohort.json','original-runner-process.json':run/'original-runner-process.json','original-session-binding.json':run/'original-session-binding.json','launch-exit.json':launch,'completion-CIM.json':run/'completion-CIM.json'}
hashes={name:sha(p) for name,p in files.items()};raw=run/'completion-observation-raw.zip'
with zipfile.ZipFile(raw,'x',zipfile.ZIP_DEFLATED) as z:
    for name,p in files.items():z.write(p,name)
archive(raw,sha(raw),hashes)
q=dict(status='independently_qualified_current_full_library_both_profiles_and_terminal_original_runner',
    original_session_id=binding['original_session_id'],observed_terminal_exit_code=0,profiles=profiles,
    source_files=1028,compiled_input_files=1029,source_manifest_sha256=c['source_manifest_sha256'],
    normal_mains=c['normal_mains'],recorded_owned_identities=len(identities),recorded_owned_processes_all_terminal=True,
    raw_archive_sha256=sha(raw),raw_files=hashes,ignored_bodies_executed=False,
    broad_default_integrations_verified=False,full_current_TCP_verified=False,GUI_verified=False,whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
(run/'completion-observation.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(status=q['status'],profiles=profiles,recorded_owned_identities=len(identities)),ensure_ascii=False))
