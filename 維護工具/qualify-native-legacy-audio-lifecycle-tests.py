"""Independent raw-output, candidate-library and terminal-identity verification."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,re,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p): return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
d=a.run.resolve(strict=True);assert d.is_relative_to(ROOT/'驗證紀錄/current/native-legacy-audio-lifecycle-tests')
assert a.session>0 and re.fullmatch('[a-f0-9]+',a.terminal_chunk)
x=read(d/'receipt.json');assert x['status']=='executed_both_profiles_candidate_real_resource_lifecycle_regressions' and x['owned_process'] is None
rb=read(d/'readback.json');assert sha(d/'evidence.zip')==rb['archive_sha256']
with zipfile.ZipFile(d/'evidence.zip') as z:
    assert set(z.namelist())==set(rb['files'])
    for n,h in rb['files'].items():
        assert sha(d/n)==h
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
assert sha(d/'tests.rs')==x['candidate_test_sha256']
assert set(x['profiles'])=={'Debug','Release'}
for profile,p in x['profiles'].items():
    assert (p['passed'],p['failed'],p['ignored'])==(2,0,0)
    assert sha(p['product'])==p['product_sha256']
    assert sha(p['diagnostic_library'])==p['diagnostic_library_sha256']
    assert sha(Path(p['compile'])/'receipt.json')==p['compile_receipt_sha256']
    assert sha(p['previous_qualification'])==p['previous_qualification_sha256']
    with Path(p['product']).open('rb') as f:
        assert f.read(2)==b'MZ';f.seek(60);off=struct.unpack('<I',f.read(4))[0];f.seek(off)
        assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
    inv=re.findall(r'^(.+): test$',(d/(profile+'-inventory.stdout.txt')).read_text('utf-8-sig'),re.M)
    selected={n for n in inv if 'legacy_' in n};assert len(selected)==2
    log=(d/(profile+'-runtime.stdout.txt')).read_text('utf-8-sig')
    assert set(re.findall(r'^test (.+) \.\.\. ok$',log,re.M))==selected==set(p['test_names'])
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)==[('ok','2','0','0','0',str(len(inv)-2))]
    jobs=[j for j in x['process_history'] if j['label'].startswith(profile)]
    assert [j['label'] for j in jobs]==[profile+'-compile',profile+'-inventory',profile+'-runtime']
    assert jobs[-1]['arguments']==[p['product'],'legacy_','--test-threads=1']
    args=jobs[0]['arguments'];assert args[args.index('--extern')+1]=='lito_modern='+p['diagnostic_library']
jobs=[x['runner_identity']]+x['process_history'];assert len(jobs)==7 and all(j.get('exit_code',0)==0 for j in jobs)
ids=','.join(str(j['pid']) for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
b=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(b.decode('utf-8-sig')) if b.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for j in jobs:
        if row['ProcessId']==j['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
for n,h in read(ROOT/'驗證紀錄/current/native-retired-join-normal/validation-20261018-063154-317211/compiled-inputs.json').items():assert sha(ROOT/n)==h
q=dict(status='independently_qualified_both_profiles_candidate_real_resource_lifecycle_regressions',
       original_session_id=a.session,observed_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk,
       profiles=x['profiles'],candidate_test_sha256=x['candidate_test_sha256'],recorded_owned_identities=7,recorded_owned_all_terminal=True,
       raw_archive_sha256=sha(d/'evidence.zip'),normal_product_source_modified=False,normal_GUI_runtime_inferred=False,
       new_sessions=0,physical_audio_verified=False,whole_goal_complete=False)
assert not (d/'qualified.json').exists()
(d/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(d/'qualified-process-CIM.json').write_bytes(b);(d/'qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(qualification=str(d/'qualified.json'),passed_each=2,terminal_identities=7),ensure_ascii=False))
