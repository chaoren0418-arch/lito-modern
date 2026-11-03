"""Independent actual-product/name/result/archive readback of current default116."""
from pathlib import Path
import time
from datetime import datetime, timezone
import argparse, hashlib, json, re, zipfile

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

def archive(path, digest, expected):
    assert sha(path) == digest
    with zipfile.ZipFile(path) as z:
        assert set(z.namelist()) == set(expected) and len(z.namelist()) == len(expected)
        for name, value in expected.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == value, name

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/default-native-regression')
r = read(run / 'receipt.json')
assert r['status'] == 'verified_current_default116_integration_targets_and_main_units'
assert r['features'] == [] and not r['library_unit_tests'] and r['sources_unchanged'] and r['fixture_inputs_unchanged']
assert r['compile_exit_code'] == 0 and r['failed'] == 0 and r['owned_process'] is None
assert all(p['exit_code'] == 0 and not p['timed_out'] for p in r['process_history'])
normal = read(r['normal_qualification'])
assert sha(r['normal_qualification']) == r['normal_qualification_sha256']
g06 = read(r['G06_qualification'])
assert sha(r['G06_qualification']) == r['G06_qualification_sha256']
assert g06['actual_formal_sessions'] == 96 and g06['source_manifest_sha256'] == normal['source_manifest_sha256']
pins = read(run / 'source-inputs.json')
fixtures = read(run / 'fixture-inputs.json')
normal_kind=r.get('normal_input_kind','startup')
assert normal_kind=='ability-finite-normal'
startup=Path(r['normal_qualification']).parent.parent
assert startup.is_relative_to(ROOT/'驗證紀錄/current/native-ability-finite-normal')
normal_products={profile:info['normal'] for profile,info in read(startup/'cohort.json')['profiles'].items()}
assert normal['status']=='independently_qualified_ability_finite_normal_related_scope' and normal['profile']==r['profile'] and normal['passed']==31 and normal['failed']==0
assert normal['recorded_owned_all_terminal']
for selected,product in normal_products.items():
    original=read(next(startup.glob(selected+'-qualified-*/qualified.json')))
    assert original['source_manifest_sha256']==normal['source_manifest_sha256'] and original['passed']==31 and product['sha256']==original['normal']['sha256']
extra=dict(path='LiTo381-Modern/windows/app.manifest',sha256=read(startup/'compiled-inputs.json')['LiTo381-Modern/windows/app.manifest'])
assert fixtures == {extra['path']: extra['sha256']} == r['additional_compile_fixtures']
compiled = read(run / 'compiled-inputs.json')
assert compiled == pins | fixtures and len(compiled) == r['compiled_input_files'] == 1048
assert len(pins) == r['source_files'] == 1047
assert sha(run / 'source-inputs.json') == r['source_manifest_sha256']
seed_path = Path(read(startup/'cohort.json')['application'])/'applied-source-inputs.json'
seed = seed_path.read_bytes()
descriptor = (run / 'source-inputs.json').read_bytes()
assert sha(seed_path) == normal['source_manifest_sha256']
assert read(seed_path) == pins
assert descriptor == seed
assert b'\r\n' not in seed and sha(seed_path)==normal['source_manifest_sha256']
canonical_manifest = normal['source_manifest_sha256']
for name, digest in compiled.items():
    assert sha(ROOT / name) == digest, name
archive(run / 'compiled-source.zip', r['compiled_source_archive_sha256'], compiled)
assert r['normal_mains_before'] == r['normal_mains_after'] == {p: v['sha256'] for p, v in normal_products.items()}
for p, info in normal_products.items():
    assert sha(info['path']) == r['normal_mains_after'][p]
metadata = read(run / 'metadata/stdout.log')
package = next(p for p in metadata['packages'] if Path(p['manifest_path']).resolve() == PROJECT / 'Cargo.toml')
targets = [t for t in package['targets'] if 'test' in t['kind']]
suites = {t['name'] for t in targets if not t.get('required-features')}
excluded = {t['name']: t['required-features'] for t in targets if t.get('required-features')}
assert len(suites) == 116 and {'native_green_clock','native_startup','native_selector_only','native_ability_buffs','native_ability_world','native_additional_finite_resources','native_ability_live'}<=suites
assert excluded == {'native_teleport_controller': ['teleport-wire-test']}
assert set(r['suites']) == suites and set(r['results']) == suites | {'main_unit'}
products = read(run / 'product-retention.json')
assert products['readback_verified'] and set(products['products']) == set(r['results'])
archive(products['archive'], products['archive_sha256'], {name + '.exe': v['sha256'] for name, v in products['products'].items()})
messages = [json.loads(line) for line in (run / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(m.get('reason') == 'build-finished' and m['success'] for m in messages)
artifacts = [m for m in messages if m.get('reason') == 'compiler-artifact' and m.get('executable') and m['profile']['test']]
assert len(artifacts) == 117
by_path = {Path(m['executable']).resolve(): m for m in artifacts}
totals = dict(passed=0, failed=0, ignored=0)
test_names, per_target = {}, {}
for name, result in r['results'].items():
    p = products['products'][name]
    path = Path(p['path']).resolve(strict=True)
    assert path.is_relative_to(PROJECT / 'target/current') and sha(path) == p['sha256'] == result['product_sha256']
    m = by_path[path]
    assert m['features'] == []
    assert m['target']['name'] == ('LiToModern' if name == 'main_unit' else name)
    with path.open('rb') as f:
        h = f.read(64)
        assert h[:2] == b'MZ'
        f.seek(int.from_bytes(h[60:64], 'little'))
        h = f.read(26)
        assert h[:4] == b'PE\0\0' and h[4:6] == b'\x64\x86' and h[24:26] == b'\x0b\x02'
    case = run / 'cases' / name
    names = re.findall(r'^(.+): test$', (case / 'inventory/stdout.log').read_text(encoding='utf-8-sig'), re.M)
    assert len(names) == len(set(names)) == result['inventory_count']
    log = (case / 'runtime/stdout.log').read_text(encoding='utf-8-sig')
    assert sha(case / 'runtime/stdout.log') == result['runtime_log_sha256']
    rows = re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$', log, re.M)
    assert len(rows) == len({n for n, _ in rows}) == len(names)
    assert {n for n, _ in rows} == set(names)
    terminal = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', log)
    assert len(terminal) == 1 and terminal[0][0] == 'ok'
    passed, failed, ignored, measured, filtered = map(int, terminal[0][1:])
    assert result['exit_code'] == failed == measured == filtered == 0 and not result['timed_out']
    assert [passed, failed, ignored] == [sum(s == k for _, s in rows) for k in ['ok', 'FAILED', 'ignored']]
    for key, value in [('passed', passed), ('failed', failed), ('ignored', ignored)]:
        assert value == result[key]
        totals[key] += value
    test_names[name] = dict(all=names, ignored=[n for n, s in rows if s == 'ignored'])
    per_target[name] = dict(product_sha256=p['sha256'], passed=passed, failed=failed, ignored=ignored)
assert totals == {k: r[k] for k in totals}
pet = read(run / 'pet-fixture-inputs.json')
assert len(pet['files']) == 353
for name, digest in pet['files'].items():
    assert sha(ROOT / name) == digest, name
legacy = run / 'cases/native_library_regression'
before = read(legacy / 'legacy-reports-before.json')
restored = read(legacy / 'legacy-report-restoration.json')
assert before['readback_verified'] and restored['existing_reports_restored']
assert before['archive_sha256'] == restored['before_archive_sha256']
archive(legacy / 'legacy-reports-before.zip', before['archive_sha256'], before['files'])
archive(legacy / 'legacy-generated-reports.zip', restored['generated_archive_sha256'], {x['name']: x['generated_sha256'] for x in restored['changes']})
for name, digest in before['files'].items():
    assert sha(PROJECT / 'logs/continuation-20261003' / name) == digest
out = run / 'profile-qualified'
assert not out.exists()
raw_files = {str(p.relative_to(run)).replace('\\', '/'): p for p in run.rglob('*') if p.is_file() and p.suffix.lower() != '.zip'}
out.mkdir()
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
raw_files['independent-qualifier.py'] = out / 'independent-qualifier.py'
hashes = {name: sha(p) for name, p in raw_files.items()}
raw = out / 'raw.zip'
with zipfile.ZipFile(raw, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, p in raw_files.items():
        z.write(p, name)
archive(raw, sha(raw), hashes)
q = dict(status='independently_qualified_current_default116_integrations_and_main', profile=r['profile'],
    source_files=1047, compiled_input_files=1048, source_manifest_sha256=canonical_manifest,
    generated_source_descriptor_sha256=r['source_manifest_sha256'], descriptor_conversion_only='none; original LF identity bytes retained',
    integration_targets=116, actual_test_executables=117, features=[], counts=totals,
    current_target_inventory=sorted(suites), ignored_bodies_executed=False, all_test_names=test_names, per_target=per_target,
    normal_mains=r['normal_mains_after'], compiled_source_archive_sha256=r['compiled_source_archive_sha256'],
    product_archive_sha256=products['archive_sha256'], historical_pet_files=353, historical_reports_restored=len(before['files']),
    raw_archive_sha256=sha(raw), raw_files=hashes, full_library_scope_inferred=False,
    full_current_TCP_verified=False, GUI_verified=False, whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'qualified.json').write_text(json.dumps(q, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: q[k] for k in ['status', 'profile', 'counts', 'actual_test_executables']}, ensure_ascii=False))
