"""Verify candidate executable identity and observed compiler completion."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,subprocess,struct
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('build',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--chunk',required=True);a=ap.parse_args()
build=a.build.resolve(strict=True);assert build.parent==V/'native44FE-teleport-audio'
r=read(build/'receipt.json');assert r['status']=='compiled_current_normal_body_audio_diagnostics' and r['owned_process'] is None
assert not r['source_overlays_used'] and r['normal_source_bodies_unchanged'] and r['diagnostic_request_API_extension'] and not r['normal_product_source_modified']
assert [j['label'] for j in r['process_history']]==['library','success','failure'] and all(j['exit_code']==0 for j in r['process_history'])
identity=read(build/'diagnostic-inputs.json');assert sha(build/'diagnostic-inputs.json')==r['diagnostic_input_manifest_sha256'] and not identity.get('candidate_source_overlays')
for n,pin in r['candidate_inputs'].items():assert sha(pin['path'])==sha(build/n)==pin['sha256']
prefix=(build/'native_login.original.rs').read_bytes();suffix=b'\n'+(build/'diagnostic_wave_api.rs').read_bytes();assert (build/'native_login.rs').read_bytes()==prefix+suffix
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h
assert sha(build/'source-inputs.json')==r['source_manifest_sha256']
assert sha(r['diagnostic_library'])==r['diagnostic_library_sha256']
for pin in r['libraries'].values():assert sha(pin['path'])==pin['sha256']
for pin in r['products'].values():
    assert sha(pin['path'])==pin['sha256']
    with Path(pin['path']).open('rb') as f:
        assert f.read(2)==b'MZ';f.seek(60);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
if r['profile']=='Debug':
    for job in r['process_history']:
        args=job['arguments'];assert 'opt-level=0' in args and 'debuginfo=1' in args and 'debug-assertions=yes' in args and 'overflow-checks=yes' in args
jobs=[r['original_runner'],*r['process_history']];ids=','.join(str(j['pid']) for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
record=dict(status='observed_current_normal_audio_compile_completed_only',compile_receipt_sha256=sha(build/'receipt.json'),original_session_id=a.session,observed_terminal_exit_code=0,terminal_tool_chunk=a.chunk,recorded_owned_all_terminal=True,recorded_owned_identities=4,source_manifest_sha256=r['source_manifest_sha256'],diagnostic_input_manifest_sha256=r['diagnostic_input_manifest_sha256'],source_overlays_used=False,normal_source_bodies_unchanged=True,candidate_module_overlays=0,diagnostic_request_API_extension=True,normal_product_source_modified=False,gameplay_qualification_granted=False,whole_goal_complete=False)
assert not (build/'compile-terminal-observation.json').exists();(build/'compile-terminal-observation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(build/'compile-process-CIM.json').write_bytes(raw);(build/'compile-observer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(observation=str(build/'compile-terminal-observation.json'),candidate_overlays=0,profile=r['profile'],all_terminal=True),ensure_ascii=False))
