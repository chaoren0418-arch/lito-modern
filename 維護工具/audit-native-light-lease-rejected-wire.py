"""Independently read the retained rejected rental-town spell request."""
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
CASE = ROOT / '驗證紀錄/current/native-light-lease-gui/20261015-221753-691'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
retention = read(CASE / 'failure-preserved.json')
receipt = read(CASE / 'receipt.json')
result = read(CASE / 'result.json')
assert sha(CASE / 'receipt.json') == retention['runtime_receipt_unchanged_sha256']
captures = list((CASE / 'raw-capture').rglob('frames.jsonl'))
assert len(captures) == 1
capture = captures[0]
archive = Path(retention['archive']['path'])
assert sha(archive) == retention['archive']['sha256']
name = 'raw/' + capture.relative_to(CASE).as_posix()
with zipfile.ZipFile(archive) as z:
    assert hashlib.sha256(z.read(name)).hexdigest().upper() == sha(capture)
frames = [json.loads(line) for line in capture.read_text(encoding='utf-8-sig').splitlines()]
for frame in frames:
    if frame['plain'] == 'MASKED_C_LOGIN':
        assert frame['direction'] == 'client_to_server' and frame['opcode'] == '0x01'
    else:
        payload = bytes.fromhex(frame['plain'])
        assert payload and payload[0] == int(frame['opcode'], 16)
keys = [f for f in frames if f['direction'] == 'server_to_client' and f['note'] == 'S_KEY']
logins = [f for f in frames if f['direction'] == 'server_to_client' and f['note'] == 'S_LITO_LOGIN_OK']
assert len(keys) == result['actual_client_sessions'] == 4 and len(logins) == 2
requests = [f for f in frames if f['direction'] == 'client_to_server' and int(f['opcode'], 16) == 0x14]
assert len(requests) == 1 and bytes.fromhex(requests[0]['plain']) == bytes([0x14, 0, 1])
index = frames.index(requests[0])
responses = [f for f in frames[index+1:] if f['direction'] == 'server_to_client']
assert responses
counts = {f'S{op:02X}': sum(int(f['opcode'], 16) == op for f in responses) for op in (0x37, 0x1b, 0x4a)}
assert all(value == 0 for value in counts.values())
assert 'rental-town rejected unsupported gameplay C opcode=0x14 length=3 plain=14 00 01' in (CASE / 'server.log').read_text(encoding='utf-8-sig')
traces = [json.loads(line)['state'] for line in (CASE / 'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
a = [s for s in traces if s['test_account'] == 'PeerFixture' and s['phase'] == 'BATTLE' and s['rental_map'] == 500]
assert a and all(s['light_expires_at_nanos'] == 0 for s in a)
assert a[-1]['mp'] == receipt['before_a']['mp'] == 140
proof = {
    'status': 'rejected_rental_town_request_wire_confirmed_not_gameplay_pass',
    'capture_sha256': sha(capture), 'receipt_sha256': sha(CASE / 'receipt.json'),
    'actual_sessions': 4, 'LOGIN_sessions': 2, 'GAME_sessions': 2,
    'C14': 1, 'exact_request': '14 00 01',
    'server_frames_after_request': len(responses), 'spell_responses_after_request': counts,
    'original_server_rejection_confirmed': True,
    'last_passive_caster_mp': a[-1]['mp'], 'passive_light_expiry_remained_zero': True,
    'natural3600s_expiry_test_executed': False,
    'memberships_empty': result['memberships_empty'],
    'qualifier_sha256': sha(Path(__file__)), 'whole_goal_complete': False,
}
destination = CASE / 'rejected-wire-audit.json'
assert not destination.exists()
destination.write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(proof, ensure_ascii=False))
