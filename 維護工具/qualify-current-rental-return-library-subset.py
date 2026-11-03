"""Qualify ten actual current regression results within the completed full library.

No test is rerun, and no historical product or standalone filtered run is claimed.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, zipfile
R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
SOURCE = 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()
def save(p, value):
    Path(p).write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')
ap = argparse.ArgumentParser()
ap.add_argument('library', type=Path)
ap.add_argument('--publish', action='store_true')
a = ap.parse_args()
base = a.library.resolve(strict=True)
assert base.is_relative_to(V / 'native-installed-resource-profile-library-default')
w = read(R / '工作佇列.json')
assert w['current_product_identity']['source_manifest_sha256'] == SOURCE
op = base / 'completion-observation.json'
o = read(op)
assert o['source_manifest_sha256'] == SOURCE and o['observed_terminal_exit_code'] == 0
assert o['recorded_original_owned_processes_all_terminal']
assert w['current_native_catalog_library']['completion_sha256'] == sha(op)
inputs = read(base / 'compiled-inputs.json')
assert len(inputs) == 1049
for name, digest in inputs.items():
    assert sha(R/name) == digest, name
src = R/'LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs'
prefix = 'modern_lito::native_login::rental_return_tests::'
names = sorted(prefix+n for n in re.findall(r'#\[test\]\s*fn\s+(\w+)\s*\(', src.read_text('utf-8-sig')))
assert len(names) == len(set(names)) == 10
assert '#[path = "native_rental_return_tests.rs"]\nmod rental_return_tests;' in (src.parent/'native_login.rs').read_text('utf-8-sig')
files = {'completion-observation.json': op, 'compiled-inputs.json': base/'compiled-inputs.json', 'regression-source.rs': src, 'reader.py': Path(__file__)}
profiles = {}
for profile in ['Debug', 'Release']:
    qp = base/profile/'full-default-qualified/qualified.json'
    q = read(qp)
    assert sha(qp) == o['profiles'][profile]['qualification_sha256']
    assert q['source_manifest_sha256'] == SOURCE and q['profile'] == profile
    assert q['passed'] == 1974 and q['failed'] == q['filtered_out'] == q['Rust_exit_code'] == 0 and q['ignored'] == 488
    archive = qp.parent/'raw.zip'
    assert sha(archive) == q['raw_archive_sha256']
    logname = 'runtime-full-current/tests.log'
    with zipfile.ZipFile(archive) as z:
        assert set(z.namelist()) == set(q['raw_files'])
        for n, digest in q['raw_files'].items():
            with z.open(n) as stream:
                assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == digest
        body = z.read(logname)
    log = base/profile/logname
    assert sha(log) == q['raw_files'][logname] == hashlib.sha256(body).hexdigest().upper()
    results = re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)(?:, [^\r\n]*)?\r?$', body.decode('utf-8-sig'), re.M)
    selected = [(n, result) for n, result in results if n.startswith(prefix)]
    assert sorted(n for n, _ in selected) == names and all(result == 'ok' for _, result in selected)
    assert len(results) == 2462 and len(set(n for n, _ in results)) == 2462
    profiles[profile] = dict(passed=10, failed=0, ignored=0, selection='actual subset within full default run', standalone_filtered_run=False,
        enclosing_passed=1974, enclosing_ignored=488, enclosing_filtered_out=0, actual_test_sha256=q['actual_test_sha256'],
        qualification=str(qp), qualification_sha256=sha(qp), tests_log_sha256=sha(log))
    files[profile+'/qualified.json'] = qp
    files[profile+'/tests.log'] = log
out = V/'native-rental-return-reconnect-focused'/('full-library-subset-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True)
hashes = {n: sha(p) for n, p in files.items()}
with zipfile.ZipFile(out/'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for n, p in files.items(): z.write(p, n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper() == h for n, h in hashes.items())
record = dict(status='current_rental_return_ten_qualified_within_actual_full_library_both_profiles', run=str(base),
    source_manifest_sha256=SOURCE, qualified=True, profiles=profiles, exact_tests=names,
    original_session=o['original_session_id'], terminal_tool_chunk=o['terminal_tool_chunk'], observed_exit_code=0,
    completion=str(op), completion_sha256=sha(op), recorded_original_owned_processes_terminal=True,
    standalone_filtered_run=False, tests_rerun=False, historical_results_promoted=False, actual_formal_sessions=0,
    GUI_verified=False, whole_goal_complete=False, raw_files=hashes, raw_archive_sha256=sha(out/'raw.zip'))
save(out/'qualified.json', record)
record.update(qualification=str(out/'qualified.json'), qualification_sha256=sha(out/'qualified.json'))
if a.publish:
    assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
    before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
    assert len(before) == 8 and all(sha(R/n) == h for n, h in before.items())
    with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
        for n in before: z.write(R/n, n)
    with zipfile.ZipFile(out/'before.zip') as z:
        assert all(hashlib.sha256(z.read(n)).hexdigest().upper() == h for n, h in before.items())
    save(out/'evidence.json', record)
    staged = {}
    for n in before:
        p = R/n
        if p.suffix == '.json':
            j = read(p)
            j['current_native_rental_return_reconnect_focused'] = record
            j.update(last_goal_turn_evidence=str(out/'evidence.json'), last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
            staged[n] = (json.dumps(j, ensure_ascii=False, indent=2)+'\n').encode('utf-8')
        else:
            staged[n] = p.read_bytes()
    assert len(staged) == 8 and all(sha(R/n) == h for n, h in before.items())
    for n, payload in staged.items(): (R/n).write_bytes(payload)
    save(out/'readback.json', dict(before_files=before, after_files={n: sha(R/n) for n in before}, before_archive_sha256=sha(out/'before.zip'), evidence_sha256=sha(out/'evidence.json')))
    assert all(sha(R/n) == h for n, h in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(qualification=record['qualification'], profiles=profiles, published=a.publish, tests_rerun=False), ensure_ascii=False))
