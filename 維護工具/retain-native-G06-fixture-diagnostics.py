"""Seal terminal diagnostic evidence without promoting it to G06 or G11 acceptance."""
from pathlib import Path
import hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / '驗證紀錄/current/native-G06-controller/20261017-030505-804788'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def save(p, v):
    p.write_text(json.dumps(v, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
def seal(out, files, record):
    out.mkdir()
    pins = {n: sha(p) for n, p in files.items()}
    with zipfile.ZipFile(out/'raw.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        for n, p in files.items(): z.write(p, n)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert set(z.namelist()) == set(pins)
        for n, h in pins.items():
            with z.open(n) as f: assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == h
    save(out/'readback.json', dict(record, raw_archive_sha256=sha(out/'raw.zip'), raw_files=pins,
        G06_whole_pipeline_qualified=False, performance_accepted=False, GUI_verified=False, whole_goal_complete=False))
    print(json.dumps(dict(output=str(out), raw_sha256=sha(out/'raw.zip')), ensure_ascii=False))

run = BASE/'town-load-replay/20261017-033321-769292'
c = read(run/'cohort.json')
assert c['status']=='executed_both_offline_captured_town_load_diagnostics' and c['owned_process'] is None
assert len(c['process_history'])==4 and all(p['exit_code']==0 for p in c['process_history'])
observed = read(run/'independent-process-observation.json')
assert len(observed)==4 and not any(p['same_recorded_identity_alive'] for p in observed)
files = {'run/'+p.relative_to(run).as_posix(): p for p in run.rglob('*') if p.is_file()}
summary = {}
for profile, info in c['profiles'].items():
    assert sha(info['executable'])==info['executable_sha256']
    result = run/profile/'results.json'
    assert sha(result)==info['results_sha256']
    d = read(result)
    assert d['OS_cache_not_flushed'] and d['other_host_load_not_excluded'] and not d['performance_accepted']
    assert len(d['records'])==2 and all(v['actors']==14 for v in d['records'])
    summary[profile] = [{k:v[k] for k in ['role','load_us','first_draw_us']} for v in d['records']]
    files['binaries/'+profile+'.exe'] = Path(info['executable'])
seal(run/'independent-readback', files, dict(status='retained_offline_captured_town_load_diagnostic',
    original_session_id=36698, observed_terminal_exit_code=0, terminal_tool_chunk_id='013ef6',
    source_manifest_sha256=c['source_manifest_sha256'], recorded_processes_all_terminal=True,
    actual_new_sessions=0, results=summary, original_Debug_eight_second_failure_root_cause_unproven=True))

case = ROOT/'驗證紀錄/current/native-teleport-handoff/20261017-034244-239'
r = read(case/'receipt.json'); comparison = read(case/'teleport-handoff-comparison.json')
assert r['test_exit_code']==r['server_exit_code']==0 and r['inputs_unchanged']
assert comparison['receipt_sha256']==sha(case/'receipt.json') and r['server']['actual_client_sessions']==6
assert comparison['authority_mutations']==0 and not comparison['sql_access']
assert len(read(case/'independent-process-observation.json'))==2
assert not any(p['same_recorded_identity_alive'] for p in read(case/'independent-process-observation.json'))
files = {'case/'+p.relative_to(case).as_posix(): p for p in case.rglob('*') if p.is_file()}
runtime = Path(r['runtime'])
files.update({'runtime/'+p.relative_to(runtime).as_posix():p for p in runtime.rglob('*') if p.is_file()})
seal(BASE/'phase-gate-candidate-readback', files, dict(status='verified_phase_gated_fixture_original_Debug_test_control_only',
    original_session_id=10052, observed_terminal_exit_code=0, terminal_tool_chunk_id='123133',
    independent_verifier_tool_chunk_id='e85ad4', independent_verifier_exit_code=0,
    source_manifest_sha256=c['source_manifest_sha256'], test_binary_sha256=r['test_binary_sha256'],
    harness_source=r['harness_source'], harness_source_sha256=r['harness_source_sha256'],
    actual_formal_sessions=6, original_Debug_failure_not_relabelled=True,
    deadline_seconds=dict(movement=8, accept=60, fixture=120)))
