"""Check adversarial own-window notifications against actual formal wire traffic."""
from pathlib import Path
import hashlib
import json
import sys

p = Path(sys.argv[1]).resolve()
read = lambda f: json.loads(f.read_text(encoding='utf-8-sig'))
sha = lambda f: hashlib.sha256(f.read_bytes()).hexdigest().upper()
r = read(p / 'receipt.json')
base = read(p / 'teleport-recovery-gui-comparison.json')
assert r['scenario'] == 'cancel-retry' and r['replay_stale_commands']
assert base['receipt_sha256'] == sha(p / 'receipt.json')
assert r['inputs_unchanged'] and r['reference_is_current']
assert r['server']['actual_client_sessions'] == 8 and r['server']['memberships_empty']
assert r['server']['staged_loadouts_after_reentry'] == 0 and r['server']['authority_mutations'] == 0
assert not r['server']['sql_access'] and not r['server']['state_persistent']
stages = ['immediate-repeat', 'destination-connecting', 'after-cancel', 'new-world-old-controls']
replays = r['stale_command_replays']
assert len(replays) == 8
checks = []
for role in ['a', 'b']:
    rows = [x for x in replays if x['role'] == role]
    assert [x['stage'] for x in rows] == stages
    for row in rows:
        attempts = row['attempts']
        assert len(attempts) == 8 and all(a['feedback'] for a in attempts)
        expected = [134] * 8 if row['stage'] == 'immediate-repeat' else [134, 135, 131, 132, 133, 127, 128, 130]
        assert [a['id'] for a in attempts] == expected
        if row['stage'] != 'immediate-repeat':
            assert row['before']['inventory'] == row['after']['inventory']
            assert row['before']['connect'] == row['after']['connect']
            assert not row['before']['npc_visible'] and not row['after']['npc_visible']
        if row['stage'] in ['destination-connecting', 'after-cancel']:
            assert row['before']['inventory'] == '背包（0）'
    checks.append(dict(role=role, attempted_notifications=32, stages=stages))
runtime = Path(r['runtime'])
capture = next((runtime / 'captures').glob('*/frames.jsonl'))
frames = [json.loads(x) for x in capture.read_text(encoding='utf-8-sig').splitlines()]
outgoing = [bytes.fromhex(x['plain']) for x in frames if x['direction'] == 'client_to_server' and x['plain'] != 'MASKED_C_LOGIN']
counts = {hex(op): sum(b[0] == op for b in outgoing) for op in [0x27, 0x28, 0x29, 0xc9, 0xd9]}
assert counts == {'0x27': 2, '0x28': 0, '0x29': 2, '0xc9': 2, '0xd9': 6}, counts
actions = [b[5:].split(b'\0', 1)[0].decode('ascii') for b in outgoing if b[0] == 0x27]
assert sorted(actions) == ['LT_TELEPORT_B', 'LT_TELEPORT_C']
result = dict(status='verified_64_stale_and_duplicate_GUI_notifications_without_extra_formal_requests',
              checks=checks, formal_wire_request_counts=counts, formal_sessions=8,
              exe_sha256=r['exe_sha256'], jar_sha256=r['jar_sha256'],
              receipt_sha256=sha(p / 'receipt.json'), capture_sha256=sha(capture),
              recovery_comparison_sha256=sha(p / 'teleport-recovery-gui-comparison.json'),
              validator_sha256=sha(Path(__file__)), product_Rust_modified=False,
              injected_server_authority=False, physical_input_verified=False,
              hostile_wire_reply_replay_verified=False, full_product_verified=False)
(p / 'teleport-stale-command-comparison.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'status': result['status'], 'requests': counts}))
