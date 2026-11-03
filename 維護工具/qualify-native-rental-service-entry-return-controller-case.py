"""Retain one terminal 75DA/TW controller case and independently decode inventory."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, itertools, json, re, subprocess, zipfile, time

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

def decode(raw):
    try:
        return raw.decode('utf-8-sig')
    except UnicodeDecodeError:
        text = raw.decode('cp950')
        assert text.encode('cp950') == raw
        return text

def inventory(body):
    assert body[0] == 0x7c
    at = 2
    records = {}
    def take(count):
        nonlocal at
        assert 0 <= count <= len(body) - at
        result = body[at:at + count]
        at += count
        return result
    def number(count):
        return int.from_bytes(take(count), 'little')
    for _ in range(body[1]):
        object_id = number(4)
        assert object_id > 0 and str(object_id) not in records
        use_type, charge, icon, blessing, count, identified = [number(n) for n in [1, 1, 2, 1, 4, 1]]
        assert count > 0 and blessing <= 3
        end = body.find(b'\0', at, at + 256)
        assert end >= at
        name = take(end - at).decode('cp950')
        assert take(1) == b'\0'
        detail = list(take(number(1)))
        records[str(object_id)] = dict(id=object_id, use_type=use_type, charge=charge,
            icon=icon, blessing=blessing, count=count, identified=identified, name=name, detail=detail)
    assert at == len(body)
    return records

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--profile', required=True, choices=['Debug', 'Release'])
ap.add_argument('--scenario', required=True, choices=['cross-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-expiry'])
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-rental-service-entry-return-controller')
c = read(run / 'cohort.json')
case_info = c['profiles'][a.profile]['cases'][a.scenario]
assert case_info['exit_code'] == 0
d = run / a.profile
case = Path(case_info['directory']).resolve(strict=True)
cross = a.scenario == 'cross-endpoint'
family = 'native-teleport-handoff' if cross else 'native-teleport-failure'
assert case.is_relative_to(ROOT / '驗證紀錄/current' / family)
for folder in [d / a.scenario, d / (a.scenario + '-comparison')]:
    if folder == d / a.scenario and case_info.get('adopted_terminal_observation'):
        observation=read(case_info['adopted_terminal_observation'])
        assert observation['observed_terminal_exit_code']==0 and observation['original_session_id']==2584
        assert Path(observation['case']).resolve()==case
        assert observation['receipt_sha256']==sha(case/'receipt.json')
        assert observation['same_immutable_test_as_failed_attempt']
        continue
    jobs = [p for p in c['process_history'] if Path(p['log_directory']) == folder]
    assert len(jobs) == 1 and jobs[0].get('exit_code') == 0, folder
r = read(case / 'receipt.json')
runtime = Path(r['runtime']).resolve(strict=True)
assert runtime.is_relative_to(PROJECT / 'target/current' / family)
assert r['scenario'] == a.scenario and r['inputs_unchanged']
assert r['test_exit_code'] == r['server_exit_code'] == 0
assert r['jar_sha256'] == c['reference_jar_sha256'] == '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(ROOT.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar') == r['jar_sha256']
assert c['resource_profile'] == 'tw13081901-v1' and c['extended_item_details']
assert 'Picked up JAVA_TOOL_OPTIONS: ' + c['JAVA_TOOL_OPTIONS_for_owned_children_only'] in decode((case / 'server.stderr.txt').read_bytes())
assert c['common_catalog_sha256'] == sha(PROJECT / 'target/current/shared-profile-inputs/common_catalog-B64ED920.bin')
expected_sessions = 6 if cross else 10 if a.scenario == 'wrong-service' else 8
assert r['server']['actual_client_sessions'] == expected_sessions and r['server']['memberships_empty']
assert r['server']['authority_mutations'] == 0 and not r['server']['sql_access'] and not r['server']['state_persistent']
pins = read(run / 'source-inputs.json')
assert len(pins) == c['source_files'] == 1048 and sha(run / 'source-inputs.json') == c['source_manifest_sha256']
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
for name, digest in c['validation_pins'].items():
    assert sha(run / name) == sha(ROOT / '維護工具' / name) == digest, name
for name, digest in r['source_pins'].items():
    assert sha(case / 'source-snapshot' / name) == digest, name
if cross and c.get('cross_endpoint_fixture_override'):
    fixture=c['cross_endpoint_fixture_override']
    assert Path(r['harness_source']).resolve()==Path(fixture['path']).resolve()
    assert r['harness_source_sha256']==fixture['sha256']==c['validation_pins'][fixture['validation_pin']]
    assert sha(run/fixture['validation_pin'])==fixture['sha256']
product_name = 'native_teleport_handoff' if cross else 'native_teleport_failure'
product = c['profiles'][a.profile]['products'][product_name]
assert sha(product['path']) == sha(r['immutable_test_binary']) == r['test_binary_sha256'] == product['sha256'] == case_info['product_sha256']
assert sha(d / 'test-products.zip') == c['profiles'][a.profile]['test_archive_sha256']
with zipfile.ZipFile(d / 'test-products.zip') as z:
    with z.open(product_name + '.exe') as f:
        assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == product['sha256']
assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',
                  (case / 'test.log').read_text(encoding='utf-8-sig')) == [('ok', '1', '0', '0', '0', '0' if cross else '4')]
comparison_name = 'teleport-handoff-comparison.json' if cross else {
    'cancel-expiry': 'teleport-cancel-expiry-comparison.json', 'cancel-retry': 'teleport-cancel-retry-comparison.json',
    'refused': 'teleport-refused-comparison.json', 'wrong-service': 'teleport-wrong-service-comparison.json',
    'timeout': 'teleport-failure-comparison.json'}[a.scenario]
comparison = read(case / comparison_name)
assert comparison['receipt_sha256'] == sha(case / 'receipt.json') and comparison['jar_sha256'] == r['jar_sha256']
captures = list((runtime / 'captures').glob('*/frames.jsonl'))
assert len(captures) == 1
capture = captures[0]
assert sha(capture) == comparison['capture_sha256']
frames = [json.loads(line) for line in capture.read_text(encoding='utf-8-sig').splitlines()]
assert sum(f['plain'] == 'MASKED_C_LOGIN' for f in frames) == expected_sessions
source_requests=[bytes.fromhex(f['plain'])for f in frames if f['direction']=='client_to_server'and f['plain']!='MASKED_C_LOGIN'and bytes.fromhex(f['plain'])[0]==0xc8]
assert len(source_requests)>=4 and all(b[1:].rstrip(b'\0').decode('cp950')=='測試乙'for b in source_requests)
if cross:assert len(source_requests)==4
batch_frames = [i for i, f in enumerate(frames)
                if f['direction'] == 'server_to_client' and f['plain'].startswith('7C ')]
batches = [inventory(bytes.fromhex(frames[i]['plain'])) for i in batch_frames]
assert batches and any(item['detail'] for batch in batches for item in batch.values())
checks = []
for role in ['a', 'b']:
    report = read(case / (role + ('-handoff.json' if cross else '-failure.json')))
    before = report['before']['inventory']['items']
    continued = report['destination' if cross else 'continued']['inventory']['items']
    action = bytes([0x27]) + (1812987996).to_bytes(4, 'little') + report['action'].encode('ascii') + bytes(2)
    action_frames = [i for i, f in enumerate(frames) if f['direction'] == 'client_to_server'
                     and f['plain'] != 'MASKED_C_LOGIN' and bytes.fromhex(f['plain']) == action]
    assert len(action_frames) == 1
    matching = [i for i, batch in enumerate(batches) if batch_frames[i] > action_frames[0] and batch == continued]
    assert len(matching) == 1, ('entire continued inventory must match one original S7C', role, matching)
    initial = []
    for i, j in itertools.combinations([i for i in range(len(batches)) if batch_frames[i] < action_frames[0]], 2):
        if not (set(batches[i]) & set(batches[j])) and batches[i] | batches[j] == before:
            initial.append([i, j])
    assert len(initial) == 1, ('entire original inventory must match disjoint original S7C batches', role, initial)
    assert report['old_epoch'] + 1 == report['new_epoch' if cross else 'failed_epoch']
    if cross:
        assert report['old_frame_rejected'] and report['old_peer'] != report['new_peer']
    checks.append(dict(role=role, original_inventory_items=len(before), continued_inventory_items=len(continued),
        original_S7C_batches=initial[0], continued_S7C_batch=matching[0], all_item_fields_exact=True))

# Query only the exact two owned PIDs, and distinguish PID reuse by creation time.
owned_pids = [p['pid'] for p in r['owned_processes']]
query = ' OR '.join('ProcessId=' + str(pid) for pid in owned_pids)
script = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); @(Get-CimInstance Win32_Process -Filter '" + query + "'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed = subprocess.run(['powershell.exe', '-NoProfile', '-Command', script], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert observed.returncode == 0 and not observed.stderr
live = json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for p in live:
    original = next(x for x in r['owned_processes'] if x['pid'] == p['ProcessId'])
    assert abs((datetime.fromisoformat(p['created']) - datetime.fromisoformat(original['start_utc'])).total_seconds()) > 0.01, 'Original owned process still alive.'

out = d / ('qualified-' + a.scenario)
out.mkdir()
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
files = {'independent-qualifier.py': out / 'independent-qualifier.py'}
for folder, prefix in [(case, 'case/'), (runtime, 'runtime/'), (d / a.scenario, 'console/'), (d / (a.scenario + '-comparison'), 'comparison-console/')]:
    for f in folder.rglob('*'):
        if f.is_file() and f.suffix.lower() not in ['.exe', '.zip']:
            assert f.resolve().is_relative_to(folder) and not f.is_symlink()
            files[prefix + str(f.relative_to(folder)).replace('\\', '/')] = f
hashes = {name: sha(f) for name, f in files.items()}
archive = out / 'raw.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, f in files.items():
        z.write(f, name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist()) == set(hashes) and len(z.namelist()) == len(hashes)
    for name, digest in hashes.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
q = dict(status='independently_readback_qualified_current_G06_TW_extended_controller_case', profile=a.profile, scenario=a.scenario,
    cross_endpoint_fixture_override=c.get('cross_endpoint_fixture_override') if cross else None,
    actual_formal_sessions=expected_sessions, actual_test_sha256=product['sha256'], source_manifest_sha256=c['source_manifest_sha256'],
    reference_jar_sha256=r['jar_sha256'], resource_profile='tw13081901-v1', extended_item_details=True,
    original_owned_processes_terminal=True, inventory_all_fields_from_original_S7C=checks,
    public_source_C8_requests_observed=len(source_requests),rental_service_entry_return_frames_present=True,
    capture_sha256=sha(capture), raw_archive_sha256=sha(archive), raw_files=hashes,
    GUI_verified=False, physical_input_verified=False, same_endpoint_verified=False, whole_G06_complete=False,
    whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'qualified.json').write_text(json.dumps(q, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: q[k] for k in ['status', 'profile', 'scenario', 'actual_formal_sessions']}, ensure_ascii=False))
