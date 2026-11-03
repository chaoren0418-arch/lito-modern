"""Seal normal-body cost measurements; this is not a gameplay/G11 qualification."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native8CC5-scene-cost')
r=read(run/'receipt.json');assert r['status']=='measured_unchanged_normal_Model_resource_costs_only' and r['owned_process'] is None
assert r['normal_source_bodies_unchanged'] and not r['source_overlays_used'] and not r['diagnostic_API_extension']
assert all(j['exit_code']==0 for j in r['process_history']) and len(r['process_history'])==4
assert sha(r['normal_library']['path'])==r['normal_library']['sha256'] and sha(r['product'])==r['product_sha256']
assert sha(run/'profile.rs')==r['probe_sha256'] and sha(run/'input.json')==r['input_sha256']
compile_args=r['process_history'][0]['arguments'];assert 'lito_modern='+r['normal_library']['path'] in compile_args
for pin in r['libraries'].values():assert sha(pin['path'])==pin['sha256']
with Path(r['product']).open('rb') as f:
    assert f.read(2)==b'MZ';f.seek(60);off=struct.unpack('<I',f.read(4))[0];f.seek(off);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
rb=read(run/'readback.json');assert sha(run/'raw.zip')==rb['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert set(z.namelist())==set(rb['raw_files'])
    for n,h in rb['raw_files'].items():
        assert sha(run/n)==h
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
rows=[]
for mode in ['parallel','a','b']:
    report=read(run/mode/'report.json');assert report['network_sessions']==0 and not report['gameplay_qualification_granted'] and not report['G11_qualified']
    assert len(report['results'])==(2 if mode=='parallel' else 1)
    for row in report['results']:
        assert row['input_ready'] and row['movement_ready'] and row['fixed_time_pixel_identity'] and row['minimap_marker_layout_exact']
        assert not row['minimap_received_counter_exact'] and row['actor_count']==(15 if row['role']=='a' else 14)
        assert row['first_frame_us']>8_000_000 and not row['within_existing_8s']
        assert row['first_draw_us']>8_000_000 and row['warm_draw_us']<300_000
        assert all(actor.get('rendered') for actor in row['first_receipt']['actors'])
        rows.append(dict(mode=mode,role=row['role'],actor_count=row['actor_count'],
            **{k:row[k] for k in ['load_us','first_draw_us','first_frame_us','warm_draw_us','within_existing_8s']},
            first_terrain_us=row['first_receipt']['render_profile_us']['terrain'],
            cache_bytes=row['first_receipt']['cache_bytes'],cache_entries=row['first_receipt']['cache_entries']))
assert len(rows)==4
normal=ROOT/'驗證紀錄/current/native-legacy-audio-normal/validation-20261018-104256-823814'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
jobs=[r['runner_identity']]+r['process_history'];ids=','.join(str(j['pid']) for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
record=dict(status='independently_observed_normal_Model_first_draw_exceeds_8s_in_resource_only_measurement',
    source_manifest_sha256=r['source_manifest_sha256'],original_session_id=a.session,observed_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk,
    receipt_sha256=sha(run/'receipt.json'),rows=rows,recorded_owned_identities=5,recorded_owned_all_terminal=True,
    source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_API_extension=False,normal_product_source_modified=False,
    exact_captured_actor_fields_and_marker_layout=True,minimap_receipt_counters_reproduced=False,
    fixed_elapsed_time_ns=0,live_protocol_and_environment_timeline_reproduced=False,
    inference='Model first-draw work alone exceeds 8s under the measured fixed scene inputs. This does not identify the hot stage or reproduce all live scheduling/environment effects.',
    new_sessions=0,new_requests=0,raw_archive_sha256=sha(run/'raw.zip'),normal_GUI_runtime_inferred=False,
    physical_audio_verified=False,gameplay_qualification_granted=False,G11_qualified=False,whole_goal_complete=False)
assert not(run/'cost-observation.json').exists();(run/'cost-observation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'cost-observer.py').write_bytes(Path(__file__).read_bytes());(run/'cost-process-CIM.json').write_bytes(raw)
print(json.dumps(dict(observation=str(run/'cost-observation.json'),measurements=4,first_draw_over_8s_all=True,terminal_identities=5,gameplay_qualification_granted=False),ensure_ascii=False))
