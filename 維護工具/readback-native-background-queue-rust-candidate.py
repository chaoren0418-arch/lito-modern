"""Bind every Rust queue/state field to independently qualified original outputs."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,csv,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--exec-chunk-id',required=True)
ap.add_argument('--terminal-exit-code',required=True,type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-background-queue-rust-candidate')
assert a.terminal_exit_code==0 and a.exec_chunk_id
r=read(run/'receipt.json');assert r['status']=='unapplied_background_queue_candidate_matches_native_selected_lifecycle'
assert r['owned_process'] is None and len(r['processes'])==2 and all(v['exit_code']==0 for v in r['processes'])
ar=read(run/'archive-readback.json');assert sha(run/'raw.zip')==ar['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert len(z.namelist())==len(ar['files']) and set(z.namelist())==set(ar['files'])
    for name,value in ar['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
        if not name.startswith('libraries/') and name!='Rust-x64-probe.exe':assert sha(run/name)==value
    binary=z.read('Rust-x64-probe.exe');at=int.from_bytes(binary[60:64],'little')
    assert binary[:2]==b'MZ' and binary[at+4:at+6]==b'\x64\x86' and binary[at+24:at+26]==b'\x0b\x02'
    assert hashlib.sha256(binary).hexdigest().upper()==r['diagnostic_sha256']
native_path=Path(r['native_qualification']);native=read(native_path)
assert sha(native_path)==r['native_qualification_sha256']==sha(run/'native-qualification.json')
assert native['status']=='independently_qualified_original_background_queue_and_selected_lifecycle_slices'
assert native['original_session_id']>0 and native['observed_terminal_exit_code']==0
ready=native.get('distinct_ready_reuse_filter_executed',False)
expected_snapshots,expected_rows=(18,90) if ready else (16,74)
assert native['original_scheduler_cancel_dispatch_and_filters_executed'] and native['original_code_bytes']==(1336 if ready else 1217)
assert sha(native_path.parent/'raw.zip')==native['raw_archive_sha256']
with zipfile.ZipFile(native_path.parent/'raw.zip') as z:
    assert len(z.namelist())==len(native['raw_files']) and set(z.namelist())==set(native['raw_files'])
    for name,value in native['raw_files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
prior_path=Path(r['prior_candidate_qualification']);prior=read(prior_path)
assert sha(prior_path)==r['prior_candidate_qualification_sha256']==sha(run/'prior-candidate-qualification.json')
assert prior['status']=='independently_read_back_unapplied_shared_background_clock_candidate'
assert all(r['candidate_files'][name]==value for name,value in prior['candidate_files'].items())
assert set(r['candidate_files'])==set(prior['candidate_files'])|{'ready_filter.rs'}
for name,value in r['candidate_files'].items():
    assert sha(run/name)==sha(ROOT/'維護工具/candidates/native-background-clock'/name)==value
assert sha(ROOT/'LiTo381-Modern/src/modern_lito/native_water_clock.rs')==r['candidate_files']['base-native_water_clock.rs']
assert sha(run/'probe.rs')==r['probe_sha256']==sha(ROOT/'維護工具/oracle/NativeBackgroundQueueRustProbe.rs')
for prefix in ['state','queue']:
    original=native_path.parent/(prefix+'-vectors.csv');copied=run/('native-'+prefix+'-vectors.csv');rust=run/('Rust-'+prefix+'-vectors.csv')
    assert sha(original)==sha(copied)==native['raw_files'][prefix+'-vectors.csv']
    assert copied.read_text(encoding='ascii').replace('\r\n','\n')==rust.read_text(encoding='ascii').replace('\r\n','\n')
with (run/'Rust-state-vectors.csv').open(encoding='ascii',newline='') as f:states=list(csv.DictReader(f))
with (run/'Rust-queue-vectors.csv').open(encoding='ascii',newline='') as f:queues=list(csv.DictReader(f))
expected=['initial','initial_again','rearm','stop','other_owner_due','map_before','map_after','scene_before','scene_after',
    'late','late_early','late_due','duplicate','mixed','wrap','stopped_idle']
if ready:expected[9:9]=['ready_before','ready_after']
assert len(states)==expected_snapshots and len(queues)==expected_rows and [v['scenario'] for v in states]==expected
result=read(run/'candidate-results.json');assert result['scenarios']==expected and (result['snapshots'],result['queue_rows'])==(expected_snapshots,expected_rows)
assert sha(run/'candidate-results.json')==r['report_sha256']
assert f'{expected_snapshots} original snapshots and {expected_rows} queue rows' in (run/'vectors-stdout.log').read_text(encoding='utf-8')
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for name,value in pins.items():assert sha(ROOT/name)==value
for value in r['libraries'].values():assert sha(value['path'])==value['sha256']
q=dict(status='independently_read_back_unapplied_background_queue_candidate_against_original_lifecycle',
    original_session_id=None,original_exec_tool_chunk_id=a.exec_chunk_id,observed_terminal_exit_code=0,
    source_manifest_sha256=r['source_manifest_sha256'],candidate_sha256=r['candidate_files']['native_water_clock.rs'],
    native_qualification=str(native_path),native_qualification_sha256=sha(native_path),
    original_code_bytes=native['original_code_bytes'],original_functions_and_slices=native['original_functions_and_slices'],
    snapshots=expected_snapshots,queue_rows=expected_rows,ready_filter_candidate_sha256=r['candidate_files']['ready_filter.rs'],
    distinct_ready_reuse_filter_executed=ready,ready_reuse_filter_removes_background=ready,
    complete_state_and_queue_fields_equal=True,native_scheduler_cancel_dispatch_and_filters_executed=True,
    clock_source_stubbed=True,CRT_memory_operations_stubbed=True,audio_dispatch_tail_stubbed=True,
    entire_entry_cleanup_or_packet_handler_executed=False,modern_scene_binding_verified=False,
    GUI_or_real_clock_timing_verified=False,candidate_applied_to_product=False,product_source_changed=False,
    actual_formal_sessions=0,whole_goal_complete=False,raw_archive_sha256=ar['archive_sha256'],raw_files=ar['files'],
    observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'independent-readback.json').exists()
(run/'independent-readback.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','snapshots','queue_rows','complete_state_and_queue_fields_equal']},ensure_ascii=False))
