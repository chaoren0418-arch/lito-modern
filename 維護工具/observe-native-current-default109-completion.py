"""Use only after the original continuation handle reports an actual exit0."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
ap = argparse.ArgumentParser()
ap.add_argument('cohort', type=Path)
ap.add_argument('--session-id', required=True, type=int)
ap.add_argument('--terminal-exit-code', required=True, type=int)
a = ap.parse_args()
out = a.cohort.resolve(strict=True)
assert out.is_relative_to(ROOT / '驗證紀錄/current/native-default109') and a.terminal_exit_code == 0

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def verify_zip(path, expected_hash, files):
    assert sha(path) == expected_hash
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist()) == len(files) and set(z.namelist()) == set(files)
        for name, digest in files.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest

s = read(out / 'cohort.json')
assert s['status'] == 'independently_qualified_current_default109_both_profiles_with_retained_Debug_and_exact_descriptor_pair'
assert not s['Debug_tests_rerun'] and set(s['profiles']) == {'Debug', 'Release'}
normal = read(s['normal_qualification'])
source = read(Path(s['normal_qualification']).parent / 'source-inputs.json')
assert len(source) == 1021
for name, digest in source.items():
    assert sha(ROOT / name) == digest, name
for info in normal['profiles'].values():
    assert sha(info['path']) == info['sha256']
processes = [(s['pipeline_pid'], s['pipeline_creation_utc'])]
profiles = {}
for profile, item in s['profiles'].items():
    run = Path(item['run']).resolve(strict=True)
    assert run.is_relative_to(ROOT / '驗證紀錄/current/default-native-regression')
    q = read(item['qualification'])
    assert sha(item['qualification']) == item['qualification_sha256']
    assert q['status'] == 'independently_qualified_current_default109_integrations_and_main'
    assert q['profile'] == profile and q['source_manifest_sha256'] == s['source_manifest_sha256']
    assert q['counts'] == item['counts'] and q['counts']['failed'] == 0
    assert q['integration_targets'] == 109 and q['actual_test_executables'] == 110
    r = read(run / 'receipt.json')
    assert r['status'] == 'verified_current_default109_integration_targets_and_main_units' and r['owned_process'] is None
    assert all(p['exit_code'] == 0 and not p['timed_out'] for p in r['process_history'])
    processes += [(p['pid'], p['creation_utc']) for p in r['process_history']]
    descriptor = (run / 'source-inputs.json').read_bytes()
    canonical = (Path(s['normal_qualification']).parent / 'source-inputs.json').read_bytes()
    assert descriptor.replace(b'\r\n', b'\n') == canonical and read(run / 'source-inputs.json') == source
    assert sha(run / 'source-inputs.json') == q['generated_source_descriptor_sha256']
    inputs = read(run / 'compiled-inputs.json')
    assert len(inputs) == 1022 and inputs == source | read(run / 'fixture-inputs.json')
    verify_zip(run / 'compiled-source.zip', q['compiled_source_archive_sha256'], inputs)
    retained = read(run / 'product-retention.json')
    verify_zip(retained['archive'], q['product_archive_sha256'], {name + '.exe': v['sha256'] for name, v in retained['products'].items()})
    for product in retained['products'].values():
        assert sha(product['path']) == product['sha256']
    verify_zip(Path(item['qualification']).parent / 'raw.zip', q['raw_archive_sha256'], q['raw_files'])
    profiles[profile] = dict(qualification=item['qualification'], qualification_sha256=item['qualification_sha256'],
        counts=q['counts'], actual_test_executables=110, generated_descriptor_sha256=q['generated_source_descriptor_sha256'],
        product_archive_sha256=q['product_archive_sha256'], raw_archive_sha256=q['raw_archive_sha256'])
ids = ','.join(str(pid) for pid in sorted({p for p, _ in processes}))
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@(" + ids + ");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode == 0 and not p.stderr
live = json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for process in live:
    for pid, created in processes:
        if process['ProcessId'] == pid:
            assert abs((datetime.fromisoformat(process['created']) - datetime.fromisoformat(created)).total_seconds()) > 0.001, 'Original owned workload still live.'
dest = out / 'completion-observation.json'
assert not dest.exists()
(out / 'completion-CIM.json').write_bytes(p.stdout)
(out / 'completion-observer.py').write_bytes(Path(__file__).read_bytes())
result = dict(status='independently_observed_current_default109_both_profiles_and_terminal_continuation',
    original_session_id=a.session_id, observed_terminal_exit_code=0, profiles=profiles,
    source_files=1021, compiled_input_files=1022, canonical_source_manifest_sha256=s['source_manifest_sha256'],
    normal_mains={p: v['sha256'] for p, v in normal['profiles'].items()}, Debug_tests_rerun=False,
    exact_owned_PIDs_and_creation_times_all_terminal=True, observed_PID_reuse_candidates=live,
    original_descriptor_readback_failure_preserved=s['original_failed_cohort'],
    GUI_verified=False, full_current_TCP_verified=False, ignored_bodies_executed=False, whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
dest.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps(dict(status=result['status'], profiles={p: v['counts'] for p, v in profiles.items()}, terminal_exit_code=0), ensure_ascii=False))
