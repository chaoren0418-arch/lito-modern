"""Bind two fresh runtime suites to existing current normal/broad evidence.

No preflight build, four-target run, or seven-process ownership is invented.
Those scopes are referenced from their actual upstream qualifications; this
pair independently checks the 64 recorded runtime process identities.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
FAMILY = ROOT/'驗證紀錄/current/native-detection-invisibility'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

def archive(path, digest, files):
    assert sha(path) == digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist()) == len(files) and set(z.namelist()) == set(files)
        for name, expected in files.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == expected, name

ap = argparse.ArgumentParser()
ap.add_argument('--inputs', required=True, type=Path)
ap.add_argument('--debug-cases', required=True, type=Path)
ap.add_argument('--release-cases', required=True, type=Path)
a = ap.parse_args()
inputs_root = a.inputs.resolve(strict=True)
assert inputs_root.is_relative_to(FAMILY)
binding = read(inputs_root/'inputs.json')
assert binding['actual_processes_launched_here'] == binding['actual_casts'] == 0
normal_q = Path(binding['normal_qualification']).resolve(strict=True)
assert normal_q.is_relative_to(ROOT/'驗證紀錄/current/native-selector-integration')
assert sha(normal_q) == binding['normal_qualification_sha256']
n = read(normal_q)
assert n['status'] == 'independently_qualified_current_normal_selector_both_profiles'
assert n['observed_terminal_exit_code'] == 0 and n['recorded_owned_processes_all_terminal']
base = Path(binding['default_cohort']).resolve(strict=True)
assert base.is_relative_to(ROOT/'驗證紀錄/current/native-default111')
bc = read(base/'cohort.json')
bt = read(base/'completion-observation.json')
assert sha(base/'completion-observation.json') == binding['default_completion_sha256']
assert bc['status'] == 'independently_qualified_current_default111_both_profiles'
assert bt['observed_terminal_exit_code'] == 0 and bt['recorded_original_owned_processes_all_terminal']
assert sha(normal_q) == bc['normal_qualification_sha256']
source_sha = binding['source_manifest_sha256']
assert source_sha == n['source_manifest_sha256'] == bc['source_manifest_sha256'] == bt['source_manifest_sha256']
pins = read(normal_q.parent/'source-inputs.json')
compiled = read(normal_q.parent/'compiled-inputs.json')
assert len(pins) == 1028 and len(compiled) == 1029
assert sha(normal_q.parent/'source-inputs.json') == source_sha
assert all(sha(ROOT/name) == digest for name, digest in compiled.items())
assert sha(ROOT/'啟動現代客戶端.ps1') == n['entry_sha256']
profiles, identities, files = {}, [], {'inputs.json':inputs_root/'inputs.json', 'binder.py':inputs_root/'binder.py'}
for profile in ['Debug', 'Release']:
    preflight = Path(binding['profiles'][profile]['input']).resolve(strict=True)
    assert preflight == inputs_root/profile
    pq_path = preflight/'qualified/qualified.json'
    pq = read(pq_path)
    assert sha(pq_path) == binding['profiles'][profile]['qualification_sha256']
    assert pq['status'] == 'independently_bound_existing_normal_for_visibility_runtime'
    assert pq['actual_casts'] == pq['actual_processes_launched_here'] == pq['actual_test_executables_run_here'] == 0
    assert pq['profile'] == profile and pq['source_manifest_sha256'] == source_sha
    assert pq['normal_qualification_sha256'] == sha(normal_q)
    assert sha(pq['broad_qualification']) == pq['broad_qualification_sha256'] == bt['profiles'][profile]['qualification_sha256']
    archive(preflight/'qualified/binding-raw.zip', pq['raw_archive_sha256'], pq['raw_files'])
    run = getattr(a, profile.lower()+'_cases').resolve(strict=True)
    assert run.parent == inputs_root
    c = read(run/'cohort.json')
    q_path = run/'qualified/qualified.json'
    q = read(q_path)
    assert Path(c['preflight']).resolve() == preflight and c['preflight_qualification_sha256'] == sha(pq_path)
    assert c['build_profile'] == q['profile'] == profile
    assert c['status'] == f'executed_normal_{profile}_visibility_eight_cases'
    assert q['status'] == f'independently_qualified_normal_{profile}_visibility_eight_cases'
    assert q['observed_terminal_exit_code'] == 0 and q['original_session_id'] > 0 and q['terminal_tool_chunk_id']
    assert q['source_manifest_sha256'] == source_sha and q['normal_sha256'] == pq['normal_sha256'] == n['profiles'][profile]['normal_sha256']
    assert q['actual_client_sessions'] == 32 and q['actual_cast_requests'] == 14
    assert q['accepted_casts'] == 8 and q['negative_cases'] == 6
    assert q['recorded_owned_identities'] == 32 and q['recorded_owned_all_terminal'] and len(c['cases']) == 8
    assert q['source_and_helpers_unchanged'] and q['visually_inspected_diagnostic_PNGs'] == 4
    assert c['owned_process'] is None
    normal = read(normal_q.parent/'cohort.json')['profiles'][profile]['normal']
    assert sha(normal['path']) == q['normal_sha256']
    archive(run/'qualified/raw.zip', q['raw_archive_sha256'], q['raw_files'])
    for i, case in enumerate(c['cases']):
        assert case['wrapper']['exit_code'] == 0
        identities.append(case['wrapper'])
        folder = Path(case['case']).resolve(strict=True)
        assert folder.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers')
        receipt = read(folder/'receipt.json')
        assert sha(folder/'receipt.json') == case['receipt_sha256'] == q['raw_files'][f'case{i}/output/receipt.json']
        assert receipt['exe_sha256'] == q['normal_sha256']
        assert receipt['jar_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
        assert receipt['server']['actual_client_sessions'] == 4 and receipt['server']['authority_mutations'] == 0
        assert not receipt['server']['sql_access'] and not receipt['server']['state_persistent']
        for name in ['a-launch-identity.json', 'b-launch-identity.json', 'server-launch-identity.json']:
            assert sha(folder/name) == q['raw_files'][f'case{i}/output/{name}']
            identities.append(read(folder/name))
    profiles[profile] = dict(input=str(preflight), cases=str(run), qualification=str(q_path), qualification_sha256=sha(q_path),
        normal_sha256=q['normal_sha256'], actual_client_sessions=32, actual_cast_requests=14, accepted_casts=8, negative_cases=6)
    files[profile+'/runtime-qualified.json'] = q_path
    files[profile+'/normal-input-qualified.json'] = pq_path
    files[profile+'/visual-observations.json'] = run/'visual-observations.json'
assert len(identities) == 64 and len({(p['pid'], p['creation_utc']) for p in identities}) == 64
ids = ','.join(str(pid) for pid in sorted({p['pid'] for p in identities}))
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
result = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert result.returncode == 0 and not result.stderr
live = json.loads(result.stdout.decode('utf-8-sig')) if result.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for row in live:
    for job in identities:
        if row['ProcessId'] == job['pid']:
            assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds()) > .001
out = FAMILY/('qualified-current-normal-pair-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
(out/'CIM.json').write_bytes(result.stdout)
save(out/'recorded-runtime-process-identities.json', identities)
for name in ['source-inputs.json', 'compiled-inputs.json']:
    (out/name).write_bytes((normal_q.parent/name).read_bytes())
q = dict(status='independently_qualified_current_normal_visibility_both_profiles',
    source_manifest_sha256=source_sha, source_files=1028, compiled_input_files=1029,
    profiles=profiles, normal_qualification=str(normal_q), normal_qualification_sha256=sha(normal_q),
    default_cohort=str(base), default_completion_sha256=sha(base/'completion-observation.json'),
    recorded_runtime_owned_identities=64, recorded_runtime_owned_all_terminal=True, PID_reuse_candidates=live,
    upstream_normal_recorded_identities=n['recorded_owned_identities'], upstream_normal_recorded_all_terminal=True,
    upstream_broad_recorded_identities=bt['recorded_original_owned_identities'], upstream_broad_recorded_all_terminal=True,
    actual_client_sessions=64, actual_cast_requests=28, accepted_casts=16, negative_cases=12,
    original_client_tick_clock_source_verified=False, GUI_verified=False, physical_audio_verified=False,
    whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
save(out/'pair-evidence.json', q)
for name in ['qualifier.py', 'CIM.json', 'recorded-runtime-process-identities.json', 'source-inputs.json', 'compiled-inputs.json', 'pair-evidence.json']:
    files[name] = out/name
hashes = {name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(out/'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for name,p in files.items():
        z.write(p, name)
archive(out/'raw.zip', sha(out/'raw.zip'), hashes)
q.update(raw_archive_sha256=sha(out/'raw.zip'), raw_files=hashes)
save(out/'qualified.json', q)
print(json.dumps(dict(output=str(out), actual_client_sessions=64, accepted_casts=16, negative_cases=12,
    recorded_runtime_owned_identities=64), ensure_ascii=False))
