"""Current C02 capture and passive staged-state checks; original expiry assertions retained."""
from pathlib import Path
import hashlib, json, sys

p = Path(sys.argv[1]).resolve()
read = lambda f: json.loads(f.read_text(encoding='utf-8-sig'))
sha = lambda f: hashlib.sha256(f.read_bytes()).hexdigest().upper()
r = read(p / 'receipt.json')
s = r['server']
runtime = Path(r['runtime'])
assert r['scenario'] == 'cancel-expiry'
assert r['status'] == 'verified_cancel_expiry_and_fresh_selection'
assert r['test_exit_code'] == r['server_exit_code'] == 0 and r['inputs_unchanged']
assert r['jar_sha256'] == '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert r['reference_is_current'] and r['observed_source_before_sha256'] == r['observed_source_after_sha256'] == r['jar_sha256']
owned_root = Path(__file__).resolve().parents[1]
assert p.is_relative_to(owned_root / '驗證紀錄/current/native-teleport-failure')
formal = owned_root.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(formal) == r['jar_sha256']
assert runtime.resolve().is_relative_to(owned_root / 'LiTo381-Modern/target/current')
assert sha(Path(r['immutable_test_binary'])) == r['test_binary_sha256']
assert s['actual_client_sessions'] == 8 and s['memberships_empty']
assert s['authority_mutations'] == 0 and not s['sql_access'] and not s['state_persistent']
assert s['staged_loadouts_after_reentry'] == 0 and s['full_game_reentry_verified']
for name, expected in r['source_pins'].items():
    assert sha(p / 'source-snapshot' / name) == expected
cap = next((runtime / 'captures').glob('*/frames.jsonl'))
frames = [json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['plain'] == 'MASKED_C_LOGIN' for x in frames) == 8
packets = [(x, bytes.fromhex(x['plain'])) for x in frames if x['plain'] != 'MASKED_C_LOGIN']
outgoing = [b for x, b in packets if x['direction'] == 'client_to_server']
assert sum(b[0] == 0xc9 for b in outgoing) == 4  # Both fresh sessions select again.
assert outgoing.count(bytes([0xd9])) == 6
assert sum(x['direction'] == 'server_to_client' and b[0] == 0xf4 and 'rental-teleport' in x['note'] for x, b in packets) == 2
trace = [json.loads(x) for x in (p / 'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
stages = [json.loads(x) for x in (p / 'staged-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks = []
for role, account in [('a', 'PeerFixture'), ('b', 'PeerFixture2')]:
    d = read(p / f'{role}-failure.json')
    before, cancelled, continued, reentered = [d[k] for k in ['before', 'failed', 'continued', 'reentered']]
    assert d['scenario'] == 'cancel-expiry' and d['manual_selection'] and d['closed_samples'] >= 100
    assert d['elapsed_ms'] < 5000 and d['expiry_wait_ms'] >= 33000
    assert [d['failed_epoch'], d['recovery_epoch'], d['reentered_epoch']] == [d['old_epoch'] + n for n in [1, 2, 3]]
    assert cancelled['phase'] == 'closed' and cancelled['error'] is None
    assert cancelled['world']['actors'] == {}
    assert all(cancelled[k]['owner'] is None for k in ['inventory', 'merchant'])
    assert cancelled['selected_character'] is None and cancelled['joined_room'] is None
    assert d['recovery']['phase'] == 'sources_ready' and d['recovery']['world']['actors'] == {}
    assert before['selected_character'] == continued['selected_character'] == reentered['selected_character']
    assert continued['phase'] == 'character_selected' and continued['inventory']['owner']['connection'] == d['reentered_epoch']
    old = next(x for x in trace if x['test_account'] == account and x['rental_town'] and x['rental_inventory'])
    old_ids = {i['id'] for i in old['rental_inventory']}
    staged = [(x['observed_now_nanos'], row) for x in stages for row in x['staged'] if set(row['inventory_ids']) == old_ids]
    assert staged and len({row['expires_at_nanos'] for _, row in staged}) == 1
    first_now, first = staged[0]
    assert not first['expired'] and 0 < first['expires_at_nanos'] - first_now <= 30_000_000_000
    expired = [(now, row) for now, row in staged if row['expired']]
    assert expired and expired[0][0] >= first['expires_at_nanos']
    assert set(map(int, continued['inventory']['items'])).isdisjoint(old_ids)
    assert reentered['phase'] == 'battle_pending' and reentered['world']['map_id'] == 500
    assert reentered['world']['generation'] > before['world']['generation']
    live = [x for x in trace if x['test_account'] == account and x['rental_town'] and x['phase'] == 'BATTLE' and x['session_identity'] != old['session_identity']]
    assert live and any([x['x'], x['y']] == d['recovery_move']['after'] for x in live)
    assert all(x['character'] == old['character'] for x in live)
    checks.append(dict(role=role, cancellation_ms=d['elapsed_ms'], natural_wait_ms=d['expiry_wait_ms'], closed_samples=d['closed_samples'], epochs=[d['old_epoch'], d['failed_epoch'], d['recovery_epoch'], d['reentered_epoch']], expiry_observed_without_pruning=True, fresh_character_selection=True, stale_inventory_absent=True, authoritative_move=d['recovery_move']))
result = dict(status='verified_cancel_natural_staged_expiry_and_fresh_selection', receipt_sha256=sha(p/'receipt.json'), capture_sha256=sha(cap), staged_trace_sha256=sha(p/'staged-trace.jsonl'), server_trace_sha256=sha(p/'server-trace.jsonl'), validator_sha256=sha(Path(__file__)), test_binary_sha256=r['test_binary_sha256'], jar_sha256=r['jar_sha256'], checks=checks, actual_client_sessions=8, authority_mutations=0, clock_mutations=0, gui_verified=False, immediate_retry_before_expiry_verified=False, full_product_verified=False)
(p/'teleport-cancel-expiry-comparison.json').write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps({'status': result['status'], 'checks': checks}))
