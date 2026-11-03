"""Independently read candidate scope tests; no normal or network qualification."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,re,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',required=True,type=int);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
d=a.run.resolve(strict=True);assert d.is_relative_to(ROOT/'驗證紀錄/current/native-legacy-audio-delivery')
assert a.session>0 and a.terminal_exit_code==0 and a.terminal_chunk not in ['','unknown','pending']
x=read(d/'receipt.json');assert x['status']=='executed_both_profiles_candidate_legacy_audio_scope_contracts'and x['owned_process']is None
rb=read(d/'readback.json');assert sha(d/'evidence.zip')==rb['archive_sha256']
with zipfile.ZipFile(d/'evidence.zip')as z:
    assert set(z.namelist())==set(rb['files'])
    for name,h in rb['files'].items():
        with z.open(name)as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h
for name,h in x['candidate_pins'].items():assert sha(d/name)==h
for profile,p in x['profiles'].items():
    assert profile in ['Debug','Release']and(p['passed'],p['failed'],p['ignored'])==(8,0,0)
    assert sha(p['product'])==p['product_sha256']
    with Path(p['product']).open('rb')as f:
        assert f.read(2)==b'MZ';f.seek(60);off=struct.unpack('<I',f.read(4))[0];f.seek(off)
        assert f.read(4)==b'PE\0\0'and struct.unpack('<H',f.read(2))[0]==0x8664
    for v in p['libraries'].values():assert sha(v['path'])==v['sha256']
    inv=re.findall(r'^(.+): test$',(d/(profile+'-inventory.stdout.txt')).read_text('utf-8-sig'),re.M)
    log=(d/(profile+'-runtime.stdout.txt')).read_text('utf-8-sig')
    assert set(re.findall(r'^test (.+) \.\.\. ok$',log,re.M))==set(inv)==set(p['test_names'])and len(inv)==8
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)==[('ok','8','0','0','0','0')]
assert set(x['profiles'])=={'Debug','Release'}
jobs=[x['runner_identity']]+x['process_history'];assert len(jobs)==7 and all(j.get('exit_code',0)==0 for j in jobs)
ids=','.join(str(j['pid'])for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
b=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(b.decode('utf-8-sig'))if b.strip()else [];live=[live]if isinstance(live,dict)else live
for row in live:
    for j in jobs:
        if row['ProcessId']==j['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
for n,h in read(ROOT/'驗證紀錄/current/native-retired-join-normal/validation-20261018-063154-317211/compiled-inputs.json').items():assert sha(ROOT/n)==h
q=dict(status='independently_qualified_candidate_legacy_scope_and_event_component_only',source_manifest_sha256=x['source_manifest_sha256'],
    original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,
    profiles={p:dict(passed=8,failed=0,ignored=0,actual_product_sha256=v['product_sha256'])for p,v in x['profiles'].items()},
    candidate_pins=x['candidate_pins'],recorded_owned_identities=7,recorded_owned_all_terminal=True,
    raw_archive_sha256=sha(d/'evidence.zip'),component_only=True,normal_integration_executed=False,new_sessions=0,
    GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
assert not(d/'qualified.json').exists()
(d/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(d/'qualified-process-CIM.json').write_bytes(b);(d/'qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(qualification=str(d/'qualified.json'),passed_each=8,terminal_identities=7,normal_integration_executed=False),ensure_ascii=False))
