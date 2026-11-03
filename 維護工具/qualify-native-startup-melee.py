"""Read back current normal EXEs, passive authority, raw frames and owned exits."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def local_path(path):
    raw = Path(path).resolve(strict=True)
    text = str(raw)
    if text.startswith('\\\\?\\') and len(text) > 7 and text[5:7] == ':\\':
        normalized = Path(text[4:]).resolve(strict=True)
        assert raw.samefile(normalized), 'Namespace normalization must preserve file identity.'
        return normalized
    return raw

def verify_archive(path, digest, hashes):
    assert sha(path) == digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist()) == len(hashes) and set(z.namelist()) == set(hashes)
        for name, h in hashes.items():
            with z.open(name) as stream:
                assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == h

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', type=int, required=True)
ap.add_argument('--terminal-exit-code', type=int, required=True)
a = ap.parse_args()
assert a.terminal_exit_code == 0
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-startup-melee')
c = read(run / 'cohort.json')
assert c['status'] == 'executed_both_current_normal_TW_extended_two_melee_rounds'
assert c['owned_process'] is None and all(p['exit_code'] == 0 for p in c['process_history'])
npath = Path(c['normal_qualification'])
assert sha(npath) == c['normal_qualification_sha256']
n = read(npath)
build = npath.parent.parent
normal = read(build / 'cohort.json')
pins = read(run / 'source-inputs.json')
assert pins == read(build / 'source-inputs.json') and len(pins) == 1024
inputs = read(build / 'compiled-inputs.json')
assert len(inputs) == 1025 and all(sha(ROOT / name) == h for name, h in inputs.items())
verify_archive(build / 'compiled-source.zip', n['compiled_source_archive_sha256'], inputs)
assert sha(ROOT / '啟動現代客戶端.ps1') == n['entry_sha256']
for name, h in c['validation_pins'].items():
    assert sha(run / name) == sha(ROOT / '維護工具' / name) == h
files = {'cohort.json': run / 'cohort.json', 'source-inputs.json': run / 'source-inputs.json'}
identities = [(p['pid'], p['creation_utc']) for p in c['process_history']]
profiles = {}
for profile in ['Debug', 'Release']:
    info = c['profiles'][profile]
    assert info['helper_exit_code'] == 0
    binary = normal['profiles'][profile]['normal']
    digest = n['profiles'][profile]['normal_sha256']
    assert sha(binary['path']) == binary['sha256'] == info['exe_sha256'] == digest
    verify_archive(binary['archive']['path'], n['profiles'][profile]['normal_product_archive_sha256'], {'LiToModern.exe': digest})
    case = Path(info['case']).resolve(strict=True)
    assert case.is_relative_to(ROOT / '驗證紀錄/current/native-real-peers')
    r = read(case / 'receipt.json')
    runtime = Path(r['runtime']).resolve(strict=True)
    assert runtime.is_relative_to(ROOT / 'LiTo381-Modern/target/current/native-real-peers')
    assert sha(case / 'receipt.json') == info['receipt_sha256']
    assert sha(case / 'validation-completed.json') == info['validation_marker_sha256']
    assert r['status'] == 'verified_two_real_peer_rounds' and r['profile'] == profile
    assert r['resource_profile'] == 'tw13081901-v1' and r['extended_item_details']
    assert r['common_catalog_sha256'] == c['common_catalog_sha256']
    assert r['exe_sha256'] == sha(runtime / 'qualified-client.exe') == digest
    assert r['jar_sha256'] == r['source_sha256_at_end'] == c['jar_sha256'] and r['source_matches_snapshot']
    server = r['server']
    assert server['actual_client_sessions'] == 4 and server['game_sessions'] == 2
    assert server['memberships_empty'] and server['authority_mutations'] == 0
    assert not server['sql_access'] and not server['state_persistent']
    marker = read(case / 'validation-completed.json')
    assert marker['binder_sha256'] == c['validation_pins']['owned-peer-validation-binding.ps1']
    assert marker['receipt_sha256'] == info['receipt_sha256']
    for name, h in marker['comparison_pins'].items():
        assert sha(case / name) == h
        assert read(case / name)['receipt_sha256'] == info['receipt_sha256']
    comp = read(case / 'authority-comparison.json')
    assert comp['exe_sha256'] == digest and comp['jar_sha256'] == c['jar_sha256']
    assert comp['trace_sha256'] == sha(case / 'server-trace.jsonl') and len(comp['checks']) == 4
    images = {}
    for check in comp['checks']:
        assert check['winner'] == ((check['role'] == 'a') == (check['round'] == 1))
        for image in check['images']:
            p = local_path(image['path'])
            assert p.is_relative_to(runtime) and sha(p) == image['sha256']
            images[p.name] = image['sha256']
    assert len(images) == 8
    assert all(v['worker_quiescent'] and v['shutdown'] == 'closed' and len(v['rounds']) == 2 for v in r['clients'].values())
    observations = list((run / profile).glob('owned-original-live-*.json'))
    assert observations, 'Original live process identities must be retained before inferring cleanup.'
    names = set()
    for observation in observations:
        o = read(observation)
        assert o['profile'] == profile
        owner = o['helper_identity']
        observed_creation = datetime.fromisoformat(owner['creation_utc'])
        assert observed_creation.tzinfo is not None
        assert any(p['pid'] == owner['pid'] and datetime.fromisoformat(p['creation_utc']) == observed_creation
            for p in c['process_history'])
        for process in o['processes']:
            assert process['ProcessId'] == owner['pid'] or process['ParentProcessId'] == owner['pid']
            if process['Name'] == 'java.exe' or process['Name'] == 'qualified-client.exe':
                assert str(runtime).replace('/', '\\') in process['CommandLine'].replace('/', '\\')
                names.add((process['Name'], process['ProcessId']))
            identities.append((process['ProcessId'], process['creation_utc']))
    assert len([x for x in names if x[0] == 'java.exe']) == 1 and len([x for x in names if x[0] == 'qualified-client.exe']) == 2
    for folder, prefix in [(run / profile, profile + '/console/'), (case, profile + '/case/'), (runtime, profile + '/runtime/')]:
        for p in folder.rglob('*'):
            if p.is_file() and p.suffix.lower() not in ['.exe', '.zip']:
                assert p.resolve().is_relative_to(folder) and not p.is_symlink()
                files[prefix + p.relative_to(folder).as_posix()] = p
    profiles[profile] = dict(exe_sha256=digest, case=str(case), actual_formal_sessions=4, full_melee_rounds=2,
        independent_authority=comp, original_rendered_PNGs=images, original_owned_java_and_two_clients_recorded=True)
identities = sorted(set(identities))
query = ' OR '.join('ProcessId=' + str(pid) for pid in sorted({p for p, _ in identities}))
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); @(Get-CimInstance Win32_Process -Filter '" + query + "'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode == 0 and not p.stderr
live = json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for row in live:
    for pid, created in identities:
        if row['ProcessId'] == pid:
            assert abs((datetime.fromisoformat(row['created']) - datetime.fromisoformat(created)).total_seconds()) > 0.001
out = run / 'qualified'
out.mkdir()
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
(out / 'owned-CIM.json').write_bytes(p.stdout)
files.update({'independent-qualifier.py': out / 'independent-qualifier.py', 'owned-CIM.json': out / 'owned-CIM.json'})
for name in c['validation_pins']:
    files[name] = run / name
hashes = {name: sha(p) for name, p in files.items()}
raw = out / 'raw.zip'
with zipfile.ZipFile(raw, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, p in files.items():
        z.write(p, name)
verify_archive(raw, sha(raw), hashes)
result = dict(status='independently_qualified_both_current_normal_TW_extended_two_melee_rounds',
    original_session_id=a.session_id, observed_terminal_exit_code=0, source_files=1024, compiled_input_files=1025,
    source_manifest_sha256=c['source_manifest_sha256'], profiles=profiles, actual_formal_sessions=8,
    full_melee_rounds=4, original_world_PNGs=16, recorded_owned_identities=len(identities),
    recorded_owned_processes_terminal=True, raw_archive_sha256=sha(raw), raw_files=hashes,
    normal_EXE_diagnostic_mode_verified=True, default_GUI_flow_verified=False, physical_input_verified=False,
    whole_G02_accepted=False, whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'qualified.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: result[k] for k in ['status', 'actual_formal_sessions', 'full_melee_rounds', 'original_world_PNGs']}, ensure_ascii=False))
