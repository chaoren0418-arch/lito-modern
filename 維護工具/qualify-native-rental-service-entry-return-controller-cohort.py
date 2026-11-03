"""Seal both profiles only after all twelve current formal controller cases."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, zipfile, time

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def verify_zip(path, digest, files):
    assert sha(path) == digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist()) == len(files) and set(z.namelist()) == set(files)
        for name, expected in files.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == expected, name

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id',type=int,required=True)
ap.add_argument('--terminal-exit-code',type=int,required=True)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-rental-service-entry-return-controller')
c = read(run / 'cohort.json')
assert c['status'] == 'executed_current_both_profiles_G06_controller_TW_extended_six_cases'
assert a.terminal_exit_code == 0
assert c['owned_process'] is None and all(p['exit_code'] == 0 for p in c['process_history'])
assert set(c['profiles']) == {'Debug', 'Release'}
assert c['scenarios'] == ['cross-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-expiry']
pins = read(run / 'source-inputs.json')
assert len(pins) == c['source_files'] == 1048
assert sha(run / 'source-inputs.json') == c['source_manifest_sha256']
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
retained = c['compiled_source_archive']
inputs=read(Path(c['startup_prerequisite'])/'compiled-inputs.json')
assert len(inputs)==1049 and inputs==retained['files']
verify_zip(retained['path'],retained['sha256'],inputs)
extra=c['additional_resource_input'];assert sha(ROOT/extra['path'])==extra['sha256']
normal=read(c['startup_qualification'])
assert sha(c['startup_qualification'])==c['startup_qualification_sha256']
assert normal['source_manifest_sha256']==c['source_manifest_sha256']
startup=read(Path(c['startup_prerequisite'])/'cohort.json')
for profile,product in startup['profiles'].items():
    assert sha(product['normal']['path'])==product['normal']['sha256']==c['normal_mains'][profile]==normal['profiles'][profile]['normal']['sha256']
for name, digest in c['validation_pins'].items():
    assert sha(run / name) == sha(ROOT / '維護工具' / name) == digest
proofs, identities = {}, []
sessions = 0
for profile, info in c['profiles'].items():
    d = run / profile
    assert set(info['cases']) == set(c['scenarios'])
    products = read(d / 'products.json')
    assert products == info['products'] and len(products) == 2
    verify_zip(d / 'test-products.zip', info['test_archive_sha256'], {name + '.exe': v['sha256'] for name, v in products.items()})
    messages = [json.loads(line) for line in (d / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
    assert any(m.get('reason') == 'build-finished' and m['success'] for m in messages)
    artifacts = [m for m in messages if m.get('reason') == 'compiler-artifact' and m.get('executable') and m['profile']['test']]
    assert len(artifacts) == 2
    for m in artifacts:
        product = products[m['target']['name']]
        assert Path(m['executable']).resolve() == Path(product['path']).resolve()
        assert m['features'] == [] and m['profile'] == product['compiler_profile']
        assert sha(product['path']) == product['sha256']
        with Path(product['path']).open('rb') as f:
            h = f.read(64)
            assert h[:2] == b'MZ'
            f.seek(int.from_bytes(h[60:64], 'little'))
            h = f.read(26)
            assert h[:4] == b'PE\0\0' and h[4:6] == b'\x64\x86' and h[24:26] == b'\x0b\x02'
    proofs[profile] = {}
    for scenario in c['scenarios']:
        qualified = Path(info['cases'][scenario].get('retained_qualification',d / ('qualified-' + scenario) / 'qualified.json'))
        q = read(qualified)
        assert q['status'] == 'independently_readback_qualified_current_G06_TW_extended_controller_case'
        if scenario=='cross-endpoint':
            assert q.get('cross_endpoint_fixture_override')==c.get('cross_endpoint_fixture_override')
        assert q['source_manifest_sha256'] == c['source_manifest_sha256'] and q['profile'] == profile and q['scenario'] == scenario
        assert q['resource_profile'] == 'tw13081901-v1' and q['extended_item_details'] and q['original_owned_processes_terminal']
        verify_zip(qualified.parent / 'raw.zip', q['raw_archive_sha256'], q['raw_files'])
        case = Path(info['cases'][scenario]['directory'])
        r = read(case / 'receipt.json')
        assert r['jar_sha256'] == c['reference_jar_sha256']
        assert q['actual_formal_sessions'] == r['server']['actual_client_sessions'] == (6 if scenario == 'cross-endpoint' else 10 if scenario == 'wrong-service' else 8)
        sessions += q['actual_formal_sessions']
        for role in ['a', 'b']:
            cross = scenario == 'cross-endpoint'
            report = read(case / (role + ('-handoff.json' if cross else '-failure.json')))
            source = report['before']['inventory']['items']
            destination = report['destination' if cross else 'continued']['inventory']['items']
            if scenario != 'cancel-expiry':
                assert set(destination) <= set(source)
                assert destination == {name: source[name] for name in destination}, (profile, scenario, role)
                identities.append(dict(profile=profile, scenario=scenario, role=role, retained=len(destination), all_fields_unchanged=True))
            else:
                assert report['manual_selection'] and report['expiry_wait_ms'] >= 33000
                assert report['failed']['phase'] == 'closed' and report['failed']['inventory']['owner'] is None
        proofs[profile][scenario] = dict(path=str(qualified), sha256=sha(qualified), sessions=q['actual_formal_sessions'], raw_archive_sha256=q['raw_archive_sha256'])
assert sessions == 96 and len(identities) == 20
out = run / 'qualified'
out.mkdir()
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
files = {'independent-qualifier.py': out / 'independent-qualifier.py', 'cohort.json': run / 'cohort.json', 'source-inputs.json': run / 'source-inputs.json', 'runner.py': run / 'runner.py'}
for profile in ['Debug', 'Release']:
    d = run / profile
    files[profile + '/products.json'] = d / 'products.json'
    for f in (d / 'build').iterdir():
        if f.is_file():
            files[profile + '/build/' + f.name] = f
    for scenario in c['scenarios']:
        files[profile + '/' + scenario + '/qualified.json'] = Path(c['profiles'][profile]['cases'][scenario].get('retained_qualification',d / ('qualified-' + scenario) / 'qualified.json'))
for name in c['validation_pins']:
    files['validation/' + name] = run / name
hashes = {name: sha(p) for name, p in files.items()}
archive = out / 'cohort-raw.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, p in files.items():
        z.write(p, name)
verify_zip(archive, sha(archive), hashes)
q = dict(status='independently_qualified_both_current_G06_controller_profiles_six_cases_each',
    cross_endpoint_fixture_override=c.get('cross_endpoint_fixture_override'),
    profiles=proofs, original_session_id=a.session_id,observed_terminal_exit_code=0,source_files=1048, source_manifest_sha256=c['source_manifest_sha256'], additional_resource_input=extra,
    retained_initial_failed_cohort=c.get('retained_initial_failed_cohort'),retained_initial_failure_readback_sha256=c.get('retained_initial_failure_readback_sha256'),
    failed_attempts_not_counted_as_passed_cases=True,performance_acceptance_claimed=False,
    actual_test_executables=4, actual_cases=12, actual_formal_sessions=96,
    source_to_continued_all_item_fields_unchanged=identities, normal_mains=c['normal_mains'],
    startup_qualification=c['startup_qualification'],startup_qualification_sha256=c['startup_qualification_sha256'],full_current_library_qualified=False,
    reference_jar_sha256=c['reference_jar_sha256'], common_catalog_sha256=c['common_catalog_sha256'],
    resource_profile='tw13081901-v1', extended_item_details=True,
    raw_archive_sha256=sha(archive), raw_files=hashes, GUI_verified=False, physical_input_verified=False,
    same_endpoint_verified=False, normal_EXE_runtime_scope_inferred=False, whole_G06_complete=False, whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'qualified.json').write_text(json.dumps(q, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: q[k] for k in ['status', 'actual_test_executables', 'actual_cases', 'actual_formal_sessions']}, ensure_ascii=False))
