"""Validate retained synthetic fault evidence separately from formal/GUI acceptance."""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import hashlib
import json
import shutil
import zipfile

root = Path(__file__).resolve().parents[1]
stage = root / '驗證紀錄/current/native-teleport-wire'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
parser = argparse.ArgumentParser()
parser.add_argument('--profile', choices=['Debug', 'Release'], required=True)
profile = parser.parse_args().profile
pins_path = stage / 'green-inputs-before.json'
pins = read(pins_path)
assert len(pins) == 965
for name, expected in pins.items():
    p = (root / name).resolve()
    assert p.is_relative_to(root) and sha(p) == expected, name
fixture_path = root / 'LiTo381-Modern/tests/fixtures/native-teleport-controller-current.json'
fixture = read(fixture_path)
assert sha(Path(fixture['capture_path'])) == fixture['capture_sha256']
assert sha(Path(fixture['source_receipt'])) == fixture['source_receipt_sha256']
assert fixture['reference_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
log_path = stage / f'{profile}-green-feature-tests.jsonl'
log = log_path.read_text(encoding='utf-8-sig')
assert '6 passed; 0 failed; 0 ignored' in log and '... FAILED' not in log
messages = []
for line in log.splitlines():
    if line.startswith('{'):
        messages.append(json.loads(line))
assert any(x.get('reason') == 'build-finished' and x['success'] for x in messages)
products = [x for x in messages if x.get('reason') == 'compiler-artifact'
            and x.get('executable') and x['target']['name'] == 'native_teleport_controller']
assert len(products) == 1 and 'teleport-wire-test' in products[0]['features']
binary = Path(products[0]['executable']).resolve()
assert binary.is_relative_to(root / 'LiTo381-Modern/target/current')
product_sha = sha(binary)
archive = stage / f'{profile}-green-feature-product.zip'
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as z:
    z.write(binary, binary.name)
with zipfile.ZipFile(archive) as z:
    assert hashlib.sha256(z.read(binary.name)).hexdigest().upper() == product_sha
names = {f'status-truncated-{n}' for n in range(14)} | {f'endpoint-truncated-{n}' for n in range(8)}
names |= {'status-trailing', 'status-unknown-word', 'status-duplicate', 'endpoint-before-status',
          'endpoint-wrong-slot', 'endpoint-invalid-index', 'endpoint-zero-port', 'endpoint-trailing',
          'frame-zero', 'frame-one', 'frame-partial', 'retired-duplicate-burst',
          'coalesced-old-socket-burst', 'new-epoch-replayed-endpoint', 'cancel-old-burst-fresh-login'}
verification = root / f'LiTo381-Modern/target/current/verification/native-teleport-wire/{profile}'
files = {p.stem: p for p in verification.glob('*.json')}
assert set(files) == names and len(names) == 37
evidence = stage / f'{profile}-green-evidence'
evidence.mkdir(exist_ok=True)
reports = []
connections = 0
for name in sorted(names):
    p = files[name]
    d = read(p)
    assert d['case'] == name and d['quiescent']
    assert d['synthetic_encrypted_loopback'] and d['authenticated_formal_sessions'] == 0
    assert d['fixture_sha256'] == sha(fixture_path)
    state = d['snapshot']
    assert not state['world']['actors'] and state['world']['map_id'] is None
    assert state['inventory']['owner'] is None and not state['inventory']['items']
    assert state['merchant']['owner'] is None and state['joined_room'] is None
    assert state['selected_character'] is None and state['services'] is None
    assert all(n == 1 for n in d['peer_connection_counts'])
    commands = d['client_command_opcodes']
    assert commands[0].count(0x27) == 1
    if name in {'retired-duplicate-burst', 'coalesced-old-socket-burst', 'new-epoch-replayed-endpoint'}:
        assert d['epoch'] == 2 and len(commands) == 2 and commands[0].count(0xd9) == 1
        assert 0xd9 not in commands[1]
        if name == 'coalesced-old-socket-burst':
            assert commands[1].count(1) == 1 and d['extra']['emitted_bodies'] == 8
    elif name == 'cancel-old-burst-fresh-login':
        assert d['epoch'] == 2 and commands[1] == [] and d['extra']['fresh_samples'] == 24
        assert state['redirect'] is None
    else:
        assert d['phase'] == 'failed' and d['error'] and d['epoch'] == 1
        assert state['redirect'] is None and 0xd9 not in commands[0]
    assert 'SyntheticOnly' not in p.read_text(encoding='utf-8')
    connections += sum(d['peer_connection_counts'])
    shutil.copy2(p, evidence / p.name)
    reports.append({'case': name, 'sha256': sha(p), 'epoch': d['epoch'], 'phase': d['phase']})
assert connections == 41
sources = stage / f'{profile}-green-source'
for name in ['Cargo.toml', 'src/modern_lito/native_login.rs', 'src/modern_lito/native_teleport_controller_tests.rs',
             'tests/native_teleport_controller.rs', 'tests/native_teleport.rs',
             'tests/fixtures/native-teleport-controller-current.json']:
    p = root / 'LiTo381-Modern' / name
    dst = sources / name
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(p, dst)
shutil.copy2(Path(__file__), evidence / Path(__file__).name)
result = {'status': 'qualified_synthetic_controller6_and37_fault_scenarios', 'profile': profile,
          'qualified_utc': datetime.now(timezone.utc).isoformat(), 'source_files': 965,
          'source_manifest_sha256': sha(pins_path), 'source_inputs_unchanged': True,
          'feature': 'teleport-wire-test', 'tests_passed': 6, 'fault_scenarios': 37,
          'synthetic_TCP_connections': connections, 'authenticated_formal_sessions': 0,
          'reports': reports, 'test_binary': str(binary), 'test_binary_sha256': product_sha,
          'archive_readback_verified': True, 'test_log_sha256': sha(log_path),
          'fixture_sha256': sha(fixture_path), 'runtime_change': 'Existing retired-world guard moved before F3/F4; strict quit ACK unchanged.',
          'normal_default_main_runtime_verified': False, 'GUI_verified': False, 'full_library_regression_verified': False,
          'limits': ['Rental controller scope is seeded from captured C02 vectors; not an authenticated formal session.',
                     'Deterministic pre-Sent duplicate case dispatches bodies directly; other scenarios use synthetic encrypted TCP.',
                     'Normal clients exclude the explicit test feature; feature-built main outputs must not be published.',
                     'Default related Rust tests, main builds, real current-reference handoff/recovery and Release need their own receipts.'],
          'whole_goal_complete': False}
(stage / f'{profile}-green-qualified.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({k: result[k] for k in ['status', 'profile', 'tests_passed', 'fault_scenarios', 'synthetic_TCP_connections']}, ensure_ascii=False))
