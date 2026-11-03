"""Independent C14 candidate readback; do not promote it to normal/live gameplay."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-request');r=read(run/'receipt.json');p=r['preparation']
assert r['status']=='executed_isolated_native77_request_candidate' and r['owned_process'] is None and not r['production_source_modified']
assert [j['label'] for j in r['process_history']]==['compile','inventory','ignored-inventory','unit'] and all(j['exit_code']==0 for j in r['process_history'])
assert read(run/'actual-tool-terminal-readback.json')==dict(original_session_id=76339,exit_code=0,terminal_tool_chunk='f9c9fc')
for n,h in r['candidate_pins'].items():assert sha(n)==h==sha(run/Path(n).name)
src=ROOT/'LiTo381-Modern/src/modern_lito';prepared=ROOT/'維護工具/candidates/native77-request'
for n,h in p['before'].items():assert sha(src/n)==h
for n,h in p['after'].items():assert sha(prepared/n)==h
before=(src/'native_skills.rs').read_text(encoding='utf-8-sig');after=(prepared/'native_skills.rs').read_text(encoding='utf-8-sig')
extended='4 | 6 | 7 | 10 | 11 | 15 | 16 | 17 | 18 | 28 | 77 => {';old='4 | 6 | 7 | 10 | 11 | 15 | 16 | 17 | 18 | 28 => {'
assert after.count(extended)==after.count('match skill {77=>5000,60=>0,')==1
assert after.replace(extended,old).replace('match skill {77=>5000,60=>0,','match skill {60=>0,')==before
label='                        77 => "#77 究極光裂術（選取目標，十格內）".into(),\n'
after=(prepared/'native_skill_selection.rs').read_text(encoding='utf-8-sig');assert after.count(label)==1 and after.replace(label,'')==(src/'native_skill_selection.rs').read_text(encoding='utf-8-sig')
qp=Path(r['normal_qualification']);assert sha(qp)==r['normal_qualification_sha256'];q=read(qp)
assert q['source_manifest_sha256']==r['source_manifest_sha256']=='C831F9BCC6518FFC43ACFE6309FE946A9C3F603D07AAB3C9C099174C90BC0614'
inputs=read(qp.parent.parent/'compiled-inputs.json');assert len(inputs)==1031
for n,h in inputs.items():assert sha(ROOT/n)==h
for v in r['libraries'].values():assert sha(v['path'])==v['sha256']
for v in r['normal_mains_before'].values():assert sha(v['path'])==v['sha256']
lo,hi=map(datetime.fromisoformat,r['library_emission_window']);assert lo<=datetime.fromtimestamp(Path(r['libraries']['lito_modern']['path']).stat().st_mtime,tz=timezone.utc)<=hi
oracle=r['oracle'];assert sha(oracle['path'])==oracle['sha256'];vectors=[json.loads(s) for s in Path(oracle['path']).read_text(encoding='utf-8-sig').splitlines() if json.loads(s)['skill']==77]
assert vectors==oracle['vectors'] and len(vectors)==2 and {v['request'] for v in vectors}=={'140904c800000002800180'}
assert {v['reuse_ms'] for v in vectors}=={5000} and {v['maximum_range'] for v in vectors}=={10}
assert sha(r['product']['path'])==r['product']['sha256']
with Path(r['product']['path']).open('rb') as f:
    assert f.read(2)==b'MZ';f.seek(0x3c);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
def log(label):return (run/label/'stdout.log').read_text(encoding='utf-8-sig')
names=set(re.findall(r'^(.+): test$',log('inventory'),re.M));assert names=={'current_c02_exact_object_coordinates_and_readonly_authority','invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests','mp_and_lawful_are_server_permission_inputs_and_never_locally_spent','range_ten_boundary_and_target_changes_use_current_received_coordinates','selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection'}
assert not set(re.findall(r'^(.+): test$',log('ignored-inventory'),re.M))
assert set(re.findall(r'^test (.+?) \.\.\. ok$',log('unit'),re.M))==names
assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log('unit'))==[('ok','5','0','0','0','0')]
ids=','.join(str(j['pid']) for j in r['process_history']);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for job in r['process_history']:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files=[f for f in run.iterdir() if f.is_file()]+[f for label in ['compile','inventory','ignored-inventory','unit'] for f in (run/label).iterdir() if f.is_file()]
pins={f.relative_to(run).as_posix():sha(f) for f in files}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for f in files:z.write(f,f.relative_to(run).as_posix())
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
record=dict(status='independently_qualified_isolated_native77_C14_request_candidate',run=str(run),source_manifest_sha256=r['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,
    candidate_pins=r['candidate_pins'],product=r['product'],passed=5,failed=0,ignored=0,original_session_id=76339,original_exit_code=0,original_terminal_tool_chunk='f9c9fc',
    recorded_owned_processes=r['process_history'],recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),exact_C02_request='140904c800000002800180',range=10,reuse_ms=5000,
    preserved_original_request_and_selection_logic=True,production_source_modified=False,full_ClientSession_executed=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(out/'qualified.json',record);print(json.dumps(dict(qualification=str(out/'qualified.json'),sha256=sha(out/'qualified.json'),passed=5,production_source_modified=False,new_casts=0),ensure_ascii=False))
