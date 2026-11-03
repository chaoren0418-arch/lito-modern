"""Read back the full library selection for current startup/retained archive source."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, subprocess, tomllib, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def member(z, name):
    with z.open(name) as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def verify_zip(path, expected, files=None):
    assert sha(path) == expected, path
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist()) == len(set(z.namelist()))
        if files is not None:
            assert set(z.namelist()) == set(files)
            for name, digest in files.items():
                assert member(z, name) == digest, name

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--profile', required=True, choices=['Debug', 'Release'])
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native1030-library-default')
d = run / a.profile
runtime = d / 'runtime-full-current'
c = read(run / 'cohort.json')
r = read(runtime / 'receipt.json')
build = read(d / 'receipt.json')
product = read(d / 'product-retention.json')
pins = read(run / 'source-inputs.json')
assert r['status'] == 'passed_pinned_library_tests'
assert r['exit_code'] == 0 and r['filter'] == '' and r['test_threads'] == 1
assert r['sources_unchanged'] and build['sources_unchanged'] and build['exit_code'] == 0
assert len(pins) == r['source_files'] == build['source_files'] == 1030
manifest = sha(run / 'source-inputs.json')
assert manifest == r['source_manifest_sha256'] == build['source_manifest_sha256'] == c['source_manifest_sha256']
assert sha(d / 'source-inputs.json') == manifest
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
actual = {str(p.relative_to(ROOT)).replace('\\', '/') for folder in ['src', 'tests']
          for p in (PROJECT / folder).rglob('*') if p.is_file()}
assert actual == {n for n in pins if n.startswith(('LiTo381-Modern/src/', 'LiTo381-Modern/tests/'))}
inputs = read(run / 'compiled-inputs.json')
assert len(inputs) == 1031 and all(inputs[name] == digest for name,digest in pins.items())
for name,digest in inputs.items(): assert sha(ROOT/name) == digest
verify_zip(c['compiled_source_archive'], c['compiled_source_archive_sha256'], inputs)
npath = Path(c['normal_qualification'])
assert sha(npath) == c['normal_qualification_sha256']
normal = read(npath)
base = Path(c['input_cohort'])
assert sha(base/'completion-observation.json') == c['input_completion_sha256']
assert normal['source_manifest_sha256'] == manifest
assert c['input_kind']=='default111'
normal_root=npath.parent.parent;normal_cohort=read(normal_root/'cohort.json')
assert sha(ROOT/'啟動現代客戶端.ps1')==normal_cohort['entry_sha256']
assert normal['status']=='independently_qualified_native77_request_normal_related_profile' and normal['passed']==31 and normal['failed']==0
assert sha(Path(normal['identity_manifest']))==manifest and read(Path(normal['identity_manifest']))==pins
broad=read(base/'completion-observation.json');assert broad['status']=='independently_qualified_current1030_default111_both_profiles_and_terminal_processes' and broad['recorded_original_owned_processes_all_terminal']
for profile,info in normal_cohort['profiles'].items():
    nq=read(next(normal_root.glob(profile+'-qualified-*/qualified.json')))
    assert nq['source_manifest_sha256']==manifest and nq['passed']==31 and nq['failed']==0
    assert sha(info['normal']['path'])==nq['normal']['sha256']==c['normal_mains'][profile]
    retained_normal=info['normal']['archive']
    verify_zip(retained_normal['path'],retained_normal['sha256'],{'LiToModern.exe':info['normal']['sha256']})
for name,digest in c['validation_pins'].items(): assert sha(run/name)==sha(ROOT/'維護工具'/name)==digest

args = build['build_arguments']
assert args[1:] == ['test', '--no-run', '--locked', '--offline', '-j1', '--target',
                    'x86_64-pc-windows-msvc', '--target-dir', 'target/current',
                    '--message-format=json', '--lib'] + (['--release'] if a.profile == 'Release' else [])
assert build['features'] == [] and build['profile'] == a.profile
release = tomllib.loads((PROJECT / 'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert (release['opt-level'], release['lto'], release['codegen-units'], release['strip']) == ('z', 'thin', 16, 'symbols')
messages = [json.loads(line) for line in (d / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(m.get('reason') == 'build-finished' and m['success'] for m in messages)
artifacts = [m for m in messages if m.get('reason') == 'compiler-artifact' and m.get('executable') and m['profile']['test']]
assert len(artifacts) == 1 and artifacts[0]['target']['name'] == 'lito_modern' and artifacts[0]['features'] == []
exe = Path(product['product']).resolve(strict=True)
assert exe.is_relative_to(PROJECT / 'target/current') and Path(artifacts[0]['executable']).resolve() == exe
assert artifacts[0]['profile'] == build['compiler_artifact_profile']
assert sha(exe) == product['product_sha256'] == r['product_sha256'] == c['profiles'][a.profile]['product_sha256']
with exe.open('rb') as f:
    h = f.read(64)
    assert h[:2] == b'MZ'
    f.seek(int.from_bytes(h[60:64], 'little'))
    h = f.read(26)
    assert h[:4] == b'PE\0\0' and h[4:6] == b'\x64\x86' and h[24:26] == b'\x0b\x02'
verify_zip(product['archive'], product['archive_sha256'], {'lito_modern.exe': product['product_sha256']})

inventory = read(d / 'inventory-audit/inventory.json')
for name, digest in inventory['raw_pins'].items():
    assert sha(d / 'inventory-audit' / name) == digest
assert inventory['product_sha256'] == product['product_sha256']
all_names = re.findall(r'^(.+): test$', (d / 'inventory-audit/all.stdout.txt').read_text(encoding='utf-8-sig'), re.M)
ignored_names = re.findall(r'^(.+): test$', (d / 'inventory-audit/ignored.stdout.txt').read_text(encoding='utf-8-sig'), re.M)
assert all_names == inventory['all_test_names'] and ignored_names == inventory['ignored_test_names']
assert len(all_names) == len(set(all_names)) == inventory['total_tests']
assert len(ignored_names) == len(set(ignored_names)) == inventory['ignored_tests']
log = (runtime / 'tests.log').read_text(encoding='utf-8-sig')
rows = re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$', log, re.M)
assert len(rows) == len({n for n, _ in rows}) == len(all_names)
assert {n for n, _ in rows} == set(all_names)
assert {n for n, status in rows if status == 'ignored'} == set(ignored_names)
terminal = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', log)
passed = len(all_names)-len(ignored_names)
assert passed > 0
assert terminal == [('ok', str(passed), '0', str(len(ignored_names)), '0', '0')]
assert [sum(status == key for _, status in rows) for key in ['ok', 'FAILED', 'ignored']] == [passed, 0, len(ignored_names)]
assert (r['passed'], r['failed'], r['ignored'], r['filtered_out']) == (passed, 0, len(ignored_names), 0)
audit = read(d / 'inventory-audit/ignored-source-reasons.json')
assert audit['mapped_unique'] == len(audit['tests']) == len(ignored_names) and audit['mapped_ambiguous'] == audit['unmapped'] == 0
assert {row['test'] for row in audit['tests']} == set(ignored_names)
for row in audit['tests']:
    assert not row['body_executed'] and row['source_mapping'] == 'unique_source_attribute'
    assert len(row['candidates']) == 1
    candidate = row['candidates'][0]
    assert pins[candidate['source']] == candidate['source_sha256']
    line = (ROOT / candidate['source']).read_text(encoding='utf-8-sig').splitlines()[candidate['ignore_line'] - 1]
    assert '#[ignore' in line and (candidate['reason'] is None or candidate['reason'] in line)

fixtures = read(run / 'fixture-input-map.json')
assert len(fixtures['files']) == r['historical_pet_fixture_file_count'] == 353
assert sha(run / 'fixture-input-map.json') == r['historical_pet_fixture_map_sha256']
for name, digest in fixtures['files'].items():
    assert sha(ROOT / name) == digest, name
for info in fixtures['outputs'].values():
    assert (runtime / info['name']).is_file() if info['kind'] == 'file' else (runtime / info['name']).is_dir()
prior = read(runtime / 'legacy-reports-before.json')
restored = read(runtime / 'legacy-report-restoration.json')
assert prior['readback_verified'] and restored['existing_reports_restored']
assert prior['archive_sha256'] == restored['before_archive_sha256']
verify_zip(runtime / 'legacy-reports-before.zip', prior['archive_sha256'], prior['files'])
for name, digest in prior['files'].items():
    assert sha(PROJECT / 'logs/continuation-20261003' / name) == digest, name
changes = {row['name']: row['generated_sha256'] for row in restored['changes']}
verify_zip(runtime / 'legacy-generated-reports.zip', restored['generated_archive_sha256'], changes)

jobs = [p for p in c['process_history'] if Path(p['output']).is_relative_to(d)]
assert len(jobs) == 4 and all(p['exit_code'] == 0 for p in jobs)
identities = sorted(set([(p['pid'],p['creation_utc']) for p in jobs] + [(r['pid'],r['creation_utc'])]))
ids = ','.join(str(pid) for pid,_ in identities)
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed = subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert observed.returncode == 0 and not observed.stderr
live = json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
if isinstance(live,dict): live=[live]
for row in live:
    for pid,created in identities:
        if row['ProcessId']==pid:
            assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(created)).total_seconds()) > .001

out = d / 'full-default-qualified'
assert not out.exists(), 'Retain earlier qualification; do not overwrite.'
out.mkdir()
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
files = {'independent-qualifier.py': out / 'independent-qualifier.py'}
(out/'owned-CIM.json').write_bytes(observed.stdout)
files['owned-CIM.json'] = out/'owned-CIM.json'
for folder in ['build', 'inventory-audit', 'runtime-full-current', 'runtime-console']:
    for f in (d / folder).rglob('*'):
        if f.is_file() and f.suffix.lower() != '.zip':
            files[str(f.relative_to(d)).replace('\\', '/')] = f
for name in ['receipt.json', 'product-retention.json', 'source-inputs.json']:
    files[name] = d / name
files['normal-qualification.json'] = npath
files['cohort.json'] = run / 'cohort.json'
files['compiled-inputs.json'] = run / 'compiled-inputs.json'
outputs = {}
for family in ['native-backend-485f-evidence', 'native-multisector-batch']:
    folders = list((PROJECT / 'target/current/scratch' / family).glob(f"run-{r['pid']}-*"))
    assert len(folders) == 1, (family, folders)
    for f in folders[0].rglob('*'):
        if f.is_file():
            assert f.resolve().is_relative_to(folders[0]) and not f.is_symlink()
            name = 'owned-output/' + family + '/' + str(f.relative_to(folders[0])).replace('\\', '/')
            files[name] = f
            outputs[name] = sha(f)
hashes = {name: sha(f) for name, f in files.items()}
archive = out / 'raw.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, f in files.items():
        z.write(f, name)
verify_zip(archive, sha(archive), hashes)
q = dict(status='independently_qualified_current_full_library_default_selection',
         profile=a.profile, source_files=1030, additional_windows_resource_inputs=1,
         source_manifest_sha256=manifest, compiled_source_archive_sha256=c['compiled_source_archive_sha256'],
         normal_qualification=str(npath), normal_qualification_sha256=sha(npath), normal_mains=c['normal_mains'],
         actual_test_sha256=product['product_sha256'], actual_test_archive_sha256=product['archive_sha256'],
         inventory=len(all_names), passed=passed, failed=0, ignored=len(ignored_names), filtered_out=0, Rust_exit_code=0,
         ignored_bodies_executed=False, historical_pet_fixture_files=353,
         restored_historical_reports=len(prior['files']), changed_reports_retained=len(changes),
         legacy_before_archive_sha256=prior['archive_sha256'], legacy_generated_archive_sha256=restored['generated_archive_sha256'],
         actual_runtime_PID=r['pid'], owned_output_PNGs=sum(n.endswith('.png') for n in outputs),
         recorded_profile_owned_identities=len(identities), recorded_profile_owned_processes_terminal=True,
         raw_archive_sha256=sha(archive), raw_files=hashes,
         GUI_verified=False, full_current_TCP_verified=False, whole_goal_complete=False,
         qualifier_sha256=sha(Path(__file__)), observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'qualified.json').write_text(json.dumps(q, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: q[k] for k in ['status', 'profile', 'passed', 'failed', 'ignored', 'owned_output_PNGs', 'restored_historical_reports']}, ensure_ascii=False))
