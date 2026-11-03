"""Verify instrumented cost evidence without inventing an unavailable wrapper exit."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,struct,subprocess,zipfile,collections
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--terminal-exit',type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.parent in [V/'native8CC5-resource-open-profile',V/'native8CC5-scene-stage-profile',V/'native8CC5-scoped-catalog',V/'native8CC5-scoped-catalog-v2',V/'native8CC5-scoped-catalog-v3']
assert (a.terminal_chunk is None)==(a.terminal_exit is None)
r=read(run/'receipt.json');assert r['status']=='measured_instrumented_resource_open_costs_only' and r['owned_process'] is None
assert r['source_overlays_used'] and not r['normal_source_bodies_unchanged'] and not r['normal_product_source_modified']
assert r['new_sessions']==r['new_requests']==0
assert all(j['exit_code']==0 for j in r['process_history'])
for path,expected in [(r['product'],r['product_sha256']),(r['diagnostic_library'],r['diagnostic_library_sha256']),(run/'input.json',r['input_sha256']),(run/'profile.rs',r['probe_sha256'])]:assert sha(path)==expected
for pin in [r['normal_library_reference'],*r['libraries'].values()]:assert sha(pin['path'])==pin['sha256']
assert sha(run/'resources.rs')==r['resource_overlay']['sha256']
candidate=Path(r['resource_overlay']['path']).parent;prep=read(candidate/'preparation.json')
assert sha(prep['original_resource']['path'])==prep['original_resource']['sha256']
for key in ['original_files','candidate_files']:
    for name,pin in prep.get(key,{}).items():
        assert sha(pin['path'])==pin['sha256']
        if key=='candidate_files':assert sha(run/name)==pin['sha256']
if 'original_Model' in prep:
    for key in ['original_Model','candidate_Model']:assert sha(prep[key]['path'])==prep[key]['sha256']
    assert sha(run/'native_world_scene.rs')==prep['candidate_Model']['sha256']
with Path(r['product']).open('rb') as f:
    assert f.read(2)==b'MZ';f.seek(60);off=struct.unpack('<I',f.read(4))[0];f.seek(off);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
rb=read(run/'readback.json');assert sha(run/'raw.zip')==rb['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert set(z.namelist())==set(rb['raw_files'])
    for n,h in rb['raw_files'].items():
        assert sha(run/n)==h
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
baseline=V/'native8CC5-scene-cost/measurement-20261018-123916-229041'
assert read(baseline/'cost-observation.json')['observed_terminal_exit_code']==0
assert sha(run/'input.json')==sha(baseline/'input.json')
rows=[]
for mode in r['modes']:
    d=read(run/mode/'report.json');b=read(baseline/mode/'report.json')
    assert d['network_sessions']==0 and not d['gameplay_qualification_granted'] and not d['G11_qualified']
    for row in d['results']:
        reference=next(x for x in b['results'] if x['role']==row['role'])
        assert row['input_ready'] and row['movement_ready'] and row['fixed_time_pixel_identity'] and row['minimap_marker_layout_exact']
        assert row['first_receipt']['rgba_sha256']==reference['first_receipt']['rgba_sha256']
        assert all(x.get('rendered') for x in row['first_receipt']['actors'])
        events=[e for e in d['resource_open_profile'] if e['thread_name']=='scene-cost-'+row['role']]
        totals=collections.defaultdict(lambda:dict(count=0,elapsed_us=0))
        preparations=collections.defaultdict(list)
        for e in events:
            if e['stage']=='model_prepare':preparations[e['kind']].append(e['elapsed_us']);continue
            assert e['stage'] in ['archive_open','index_parse']
            if e['stage']=='archive_open':assert e['success'] and e['error'] is None
            k=e['stage']+'/'+('SpriteShard' if e['kind'].startswith('SpriteShard') else e['kind'])
            totals[k]['count']+=1;totals[k]['elapsed_us']+=e['elapsed_us']
        if 'original_Model' in prep:assert len(preparations)==12 and all(len(v)==2 for v in preparations.values())
        rows.append(dict(mode=mode,role=row['role'],**{k:row[k] for k in ['actor_count','load_us','first_draw_us','first_frame_us','warm_draw_us']},rgba_sha256=row['first_receipt']['rgba_sha256'],unchanged_normal_baseline_pixels=True,resource_totals=dict(totals),first_and_warm_prepare_us=dict(preparations)))
normal=V/'native-legacy-audio-normal/validation-20261018-104256-823814'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
jobs=[r['runner_identity']]+r['process_history'];ids=','.join(str(j['pid']) for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
record=dict(status='instrumented_scene_cost_evidence_verified_no_gameplay_qualification',measurement=str(run),receipt_sha256=sha(run/'receipt.json'),source_manifest_sha256=r['source_manifest_sha256'],original_session_id=a.session,observed_wrapper_terminal_exit_code=a.terminal_exit,observed_wrapper_terminal_tool_chunk=a.terminal_chunk,original_wrapper_handle_available=a.terminal_exit is not None,recorded_children_exit_codes=[j['exit_code'] for j in r['process_history']],recorded_owned_identities=len(jobs),recorded_owned_all_terminal=True,rows=rows,normal_product_source_modified=False,source_overlays_used=True,normal_source_bodies_unchanged=False,new_sessions=0,new_requests=0,raw_archive_sha256=sha(run/'raw.zip'),minimap_receipt_counters_reproduced=False,live_protocol_timeline_reproduced=False,normal_GUI_verified=False,physical_audio_verified=False,gameplay_qualification_granted=False,G11_qualified=False,whole_goal_complete=False)
assert not (run/'cost-observation.json').exists()
(run/'cost-observation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'cost-observer.py').write_bytes(Path(__file__).read_bytes());(run/'cost-process-CIM.json').write_bytes(raw)
print(json.dumps(dict(observation=str(run/'cost-observation.json'),rows=len(rows),wrapper_terminal_exit_code=a.terminal_exit,all_pixels_identical_to_normal=True,normal_product_source_modified=False),ensure_ascii=False))
