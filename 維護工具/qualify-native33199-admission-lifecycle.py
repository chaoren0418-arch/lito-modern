"""Independently qualify the exact five-body candidate's extended Debug scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,hashlib,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native33199-admission-lifecycle');assert a.terminal_exit_code==0 and a.chunk.lower() not in ['','pending','unknown']
r=read(run/'receipt.json');assert r['status']=='candidate_lifecycle_original_animation_combat_and_source_owner_passed' and r['owned_process'] is None and r['passed']==30 and r['failed']==r['ignored']==0
assert not r['normal_source_modified'] and len(r['process_history'])==7 and all(p['exit_code']==0 for p in r['process_history'])
base=Path(r['base']);b=read(base/'receipt.json');q=read(base/'qualified.json');assert sha(base/'qualified.json')==r['base_qualification_sha256'] and q['scope']['passed']==4 and q['recorded_owned_all_terminal']
profile=r.get('profile','Debug');assert profile in ['Debug','Release']
if profile=='Release':assert q['scope']['profile']=='Release' and q['scope']['Release_candidate_verified']
assert r['diagnostic_library']==b['diagnostic_library'] and sha(r['diagnostic_library'])==r['diagnostic_library_sha256']==b['diagnostic_library_sha256']
for n,pin in b['source_overlays'].items():assert sha(base/n)==pin['sha256'],n
for n,pin in b['libraries'].items():assert sha(pin['path'])==pin['sha256'],n
normal=V/'native-scoped-catalog-normal/validation-20261018-132210-585372'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
expected={'lifecycle-test':3,'native_animation-test':4,'native_combat-test':11,'source-owner-test':12}
assert set(r['results'])==set(expected)
for label,count in expected.items():
    text=(run/(label+'.stdout.txt')).read_text('utf-8-sig');assert f'test result: ok. {count} passed; 0 failed; 0 ignored' in text;assert r['results'][label]==dict(passed=count,ignored=0)
for target,pin in r['products'].items():
    assert sha(pin['path'])==pin['sha256'],target
    if target!='source-owner':assert sha(run/(target+'.rs'))==pin['source_sha256']
assert r['products']['source-owner']['sha256']==b['product_sha256']
for target in ['native_animation','native_combat']:assert sha(run/(target+'.original.rs'))==sha(ROOT/'LiTo381-Modern/tests'/(target+'.rs'))
life=(run/'lifecycle.rs').read_bytes();assert life==(base/'tests.rs').read_bytes()+b'\n'+(run/'lifecycle-regressions.rs').read_bytes()
identities=[(r['runner_identity']['pid'],r['runner_identity']['creation_utc'])]+[(p['pid'],p['creation_utc'])for p in r['process_history']]
ids=','.join(str(pid)for pid,_ in identities);cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
c=subprocess.run(['powershell.exe','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert c.returncode==0 and not c.stderr
live=json.loads(c.stdout.decode('utf-8-sig'))if c.stdout.strip()else []
if isinstance(live,dict):live=[live]
for row in live:
    for pid,birth in identities:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(birth)).total_seconds())>.001
out=run/'qualified.json';assert not out.exists();files={p.name:p for p in run.iterdir()if p.is_file()}
for target,pin in r['products'].items():files[target+'.exe']=Path(pin['path'])
pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(run/'evidence.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(run/'evidence.zip')as z:
    assert set(z.namelist())==set(pins)
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
record=dict(status='independently_qualified_extended_candidate_'+profile+'_lifecycle',profile=profile,source_manifest_sha256=r['source_manifest_sha256'],original_session_id=a.session,observed_terminal_exit_code=0,terminal_tool_chunk=a.chunk,base_qualification=str(base/'qualified.json'),base_qualification_sha256=sha(base/'qualified.json'),passed=30,failed=0,ignored=0,results=r['results'],candidate_applied=False,invalid_source_ABA_verified=True,actor_reincarnation_and_new_connection_verified=True,pending_death_final_pose_verified=True,original_animation_combat_and_source_owner_verified=True,valid_second_source_verified=False,guardian_motion_verified=False,classic_terrain_reload_verified=False,Release_lifecycle_verified=profile=='Release',GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,recorded_owned_identities=len(identities),recorded_owned_all_terminal=True,archive_sha256=sha(run/'evidence.zip'),archive_files=pins,observed_utc=datetime.now(timezone.utc).isoformat())
(run/'completion-CIM.json').write_bytes(c.stdout);(run/'evidence-reader.py').write_bytes(Path(__file__).read_bytes());out.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(qualification=str(out),passed=30,owned_terminal_identities=len(identities)),ensure_ascii=False))
