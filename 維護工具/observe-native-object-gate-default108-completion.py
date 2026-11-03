"""Invoke only after observing original session29540's actual terminal exit0."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, struct, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
NORMAL = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified'
ap = argparse.ArgumentParser()
ap.add_argument('--session-id', type=int, required=True)
ap.add_argument('--terminal-exit-code', type=int, required=True)
a = ap.parse_args()
assert a.session_id == 29540 and a.terminal_exit_code == 0
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
encode = lambda v: json.dumps(v, ensure_ascii=False, indent=2) + '\n'
def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def member_sha(z, name):
    with z.open(name) as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()
state = read(NORMAL / 'default108-current.json')
assert state['status'] == 'qualified_current_source1019_default108_both_profiles'
assert set(state['profiles']) == {'Debug', 'Release'}
dest = NORMAL / 'default108-completion-observation.json'
assert not dest.exists()
command = """[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot='ROOT_LITERAL'
$taskAll=@(Get-CimInstance Win32_Process)
function Describe($p){[pscustomobject]@{pid=$p.ProcessId;name=$p.Name;created=$p.CreationDate.ToUniversalTime().ToString('o');path=$p.ExecutablePath;command=$p.CommandLine}}
$taskOriginal=@($taskAll|Where-Object {$_.ProcessId -eq ORIGINAL_PID}|ForEach-Object {Describe $_})
$taskOwned=@($taskAll|Where-Object {
 $_.Name -in @('cargo.exe','rustc.exe','cl.exe','link.exe') -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($taskRoot)) -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -and
 ($_.Name -match '^(LiToModern|lito_modern-|native_|qualified-client|actual-client|oracle)' -or
 ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))
}|ForEach-Object {Describe $_})
[pscustomobject]@{observed_utc=[DateTime]::UtcNow.ToString('o');original_pid_candidates=$taskOriginal;owned_workloads=$taskOwned}|ConvertTo-Json -Depth 6 -Compress
""".replace('ROOT_LITERAL', str(ROOT).replace("'", "''")).replace('ORIGINAL_PID', str(state['pipeline_pid']))
probe = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command],
    capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert probe.returncode == 0 and not probe.stderr
cim = json.loads(probe.stdout.decode('utf-8-sig'))
raw_cim = NORMAL / ('default108-terminal-CIM-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f') + '.json')
raw_cim.write_bytes(probe.stdout)
parse_time = lambda text: datetime.fromisoformat(text.replace('Z', '+00:00'))
assert isinstance(cim['owned_workloads'], list) and cim['owned_workloads'] == []
assert not any(parse_time(p['created']) == parse_time(state['pipeline_creation_utc'])
    for p in cim['original_pid_candidates']), 'Original pipeline identity remains live.'
pins = read(NORMAL / 'source-inputs.json')
normal = read(NORMAL / 'qualified.json')
assert len(pins) == normal['source_files'] == 1019
assert sha(NORMAL / 'source-inputs.json') == normal['source_manifest_sha256']
for name, h in pins.items():
    assert sha(ROOT / name) == h, name
mains = {p: v['sha256'] for p, v in normal['profiles'].items()}
for item in normal['profiles'].values():
    assert sha(item['path']) == item['sha256']
profiles = {}
for profile in ['Debug', 'Release']:
    recorded = state['profiles'][profile]
    run = Path(recorded['run']).resolve(strict=True)
    assert run.is_relative_to(ROOT / '驗證紀錄/current/default-native-regression')
    q = read(run / 'profile-qualified.json')
    r = read(run / 'receipt.json')
    assert sha(run / 'profile-qualified.json') == recorded['qualification_sha256']
    assert q['status'] == 'qualified_current_default_Rust_results' and q['profile'] == profile
    assert q['integration_targets'] == 108 and q['products_retained'] == 109 and q['features'] == []
    assert r['status'] == 'verified_current_default108_integration_targets_and_main_units' and r['compile_exit_code'] == 0
    assert r['source_files'] == r['compiled_input_files'] == 1019
    assert read(run / 'compiled-inputs.json') == read(run / 'source-inputs.json') == pins
    assert q['normal_mains'] == r['normal_mains_before'] == r['normal_mains_after'] == mains
    metadata = read(run / 'metadata/stdout.log')
    package = next(p for p in metadata['packages'] if Path(p['manifest_path']).resolve() == ROOT / 'LiTo381-Modern/Cargo.toml')
    tests = [t for t in package['targets'] if 'test' in t['kind']]
    selected = {t['name'] for t in tests if not t.get('required-features')}
    excluded = {t['name']: t['required-features'] for t in tests if t.get('required-features')}
    assert len(selected) == 108 and excluded == {'native_teleport_controller': ['teleport-wire-test']}
    assert set(r['suites']) == selected and set(r['results']) == selected | {'main_unit'}
    retained = read(run / 'product-retention.json')
    assert set(retained['products']) == set(r['results'])
    assert sha(Path(retained['archive'])) == retained['archive_sha256'] == q['product_archive_sha256']
    counts = dict(passed=0, failed=0, ignored=0)
    with zipfile.ZipFile(retained['archive']) as z:
        assert set(z.namelist()) == {name + '.exe' for name in retained['products']}
        for name, product in retained['products'].items():
            result = r['results'][name]
            assert member_sha(z, name + '.exe') == sha(Path(product['path'])) == product['sha256'] == result['product_sha256']
            assert result['exit_code'] == 0 and not result['timed_out']
            with Path(product['path']).open('rb') as image:
                header = image.read(64)
                assert header[:2] == b'MZ'
                image.seek(int.from_bytes(header[60:64], 'little'))
                pe = image.read(26)
                assert pe[:4] == b'PE\0\0' and int.from_bytes(pe[4:6], 'little') == 0x8664 and int.from_bytes(pe[24:26], 'little') == 0x20b
            case = run / 'cases' / name
            inventory = (case / 'inventory/stdout.log').read_text(encoding='utf-8-sig')
            names = re.findall(r'^(.+): test$', inventory, re.M)
            runtime_log = case / 'runtime/stdout.log'
            matches = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', runtime_log.read_text(encoding='utf-8-sig'))
            assert len(matches) == 1 and matches[0][0] == 'ok'
            passed, failed, ignored, measured, filtered = map(int, matches[0][1:])
            assert failed == measured == filtered == 0 and len(names) == passed + ignored == result['inventory_count']
            assert sha(runtime_log) == result['runtime_log_sha256'] == q['per_target_pins'][name]['runtime_sha256']
            for key, value in [('passed', passed), ('failed', failed), ('ignored', ignored)]:
                assert value == result[key]
                counts[key] += value
    assert counts == {key: q[key] for key in counts} == {key: r[key] for key in counts}
    assert counts['failed'] == 0 and counts['ignored'] == 21 and counts['passed'] > 0
    assert sha(run / 'compiled-source.zip') == q['compiled_source_archive_sha256']
    with zipfile.ZipFile(run / 'compiled-source.zip') as z:
        assert set(z.namelist()) == set(pins)
        for name, h in pins.items():
            assert member_sha(z, name) == h, name
    fixtures = read(run / 'pet-fixture-inputs.json')
    assert len(fixtures['files']) == 353
    for name, h in fixtures['files'].items():
        assert sha(ROOT / name) == h, name
    raw_archive = run / 'profile-raw-evidence.zip'
    raw_note = run / 'profile-raw-readback.json'
    if not raw_note.exists():
        assert not raw_archive.exists()
        files = {p.relative_to(run).as_posix(): p for p in run.rglob('*') if p.is_file() and p.suffix.lower() != '.zip'}
        for p in files.values():
            assert p.resolve().is_relative_to(run) and not p.is_symlink()
        hashes = {name: sha(p) for name, p in files.items()}
        with zipfile.ZipFile(raw_archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
            for name, p in files.items():
                z.write(p, name)
        note = dict(status='independently_readback_verified_' + profile + '_default108_products_source_and_raw',
            profile=profile, source_files=1019, products=109, **counts,
            qualification_sha256=sha(run / 'profile-qualified.json'), product_archive_sha256=sha(Path(retained['archive'])),
            compiled_source_archive_sha256=sha(run / 'compiled-source.zip'), raw_archive_sha256=sha(raw_archive),
            raw_files=hashes, whole_pipeline_session=a.session_id, whole_pipeline_terminal_observed=True,
            full_library_unit_regression_verified=False, full_current_TCP_regression_verified=False, whole_goal_complete=False)
        raw_note.write_text(encode(note), encoding='utf-8', newline='\n')
    note = read(raw_note)
    assert note['profile'] == profile and note['source_files'] == 1019 and note['products'] == 109
    assert {key: note[key] for key in counts} == counts
    assert sha(raw_archive) == note['raw_archive_sha256']
    with zipfile.ZipFile(raw_archive) as z:
        assert set(z.namelist()) == set(note['raw_files'])
        for name, h in note['raw_files'].items():
            assert member_sha(z, name) == h == sha(run / name), name
    profiles[profile] = dict(run=str(run), qualification=str(run / 'profile-qualified.json'),
        qualification_sha256=sha(run / 'profile-qualified.json'), **counts, products=109,
        raw_files=len(note['raw_files']), raw_archive_sha256=sha(raw_archive),
        source_archive_sha256=q['compiled_source_archive_sha256'], product_archive_sha256=q['product_archive_sha256'])
observation = dict(status='independently_readback_verified_source1019_default108_both_profiles_and_terminal_process',
    observed_utc=datetime.now(timezone.utc).isoformat(), session_id=a.session_id, terminal_exit_code=0,
    actual_terminal_tool_observed=True, pipeline_pid=state['pipeline_pid'], expected_creation_utc=state['pipeline_creation_utc'],
    original_pipeline_identity_absent=True, owned_workloads=[], raw_CIM=str(raw_cim), raw_CIM_sha256=sha(raw_cim),
    source_files=1019, source_manifest_sha256=sha(NORMAL / 'source-inputs.json'), normal_mains=mains,
    profiles=profiles, full_library_unit_regression_verified=False, full_current_TCP_regression_verified=False,
    whole_goal_complete=False)
dest.write_text(encode(observation), encoding='utf-8', newline='\n')
assert read(dest) == observation
state['source_and_corresponding_tests_frozen'] = False
state['completion_observation'] = str(dest)
(NORMAL / 'default108-current.json').write_text(encode(state), encoding='utf-8', newline='\n')
print(encode(observation))
