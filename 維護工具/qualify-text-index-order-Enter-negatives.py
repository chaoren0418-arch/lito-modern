"""Read actual owned GUI/wire evidence; missing or interrupted cases fail closed.

This adds no runtime sessions.  A successful source-matched positive hotel-return
qualification is a prerequisite, and its original assertions remain unchanged.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import hashlib
import json
import re
import zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
SOURCE = '3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'


def read(path):
    return json.loads(Path(path).read_text('utf-8-sig'))


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()


def utc(value):
    return datetime.fromisoformat(value.replace('Z', '+00:00'))


def encode(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')


def qualify(case):
    cohort = read(case / 'cohort.json')
    queue = read(R / '工作佇列.json')
    assert queue['current_product_identity']['source_manifest_sha256'] == SOURCE
    assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
    assert cohort['source_manifest_sha256'] == SOURCE
    assert cohort['scenario'] == 'rental-hotel-return-reconnect'
    assert cohort['status'] == 'runtime_exited_original_zero_independent_GUI_qualification_pending'
    plan_path = case / 'quantity-Enter-negative-plan.json'
    plan = read(plan_path)
    assert plan['source_manifest_sha256'] == SOURCE and Path(plan['case']) == case
    assert plan['status'] == 'declared_before_normal_GUI_inputs'
    assert plan['positive_required_quantities'] == dict(a=8, b=1)
    assert plan['positive_key'] == 'Return' and plan['expected_total_C51'] == 2
    assert plan['protocol_or_gameplay_deadlines_changed'] is False
    assert plan['authority_injections'] is False
    assert plan['consumed_request']['held_key_autorepeat_claimed'] is False
    positive_path = case / 'qualified-hotel-return-reconnect-keyboard/qualified.json'
    positive = read(positive_path)
    accepted = queue['current_normal_GUI_hotel_return_keyboard'][cohort['profile']]
    assert Path(accepted['qualification']) == positive_path
    assert sha(positive_path) == accepted['qualification_sha256']
    assert positive['source_manifest_sha256'] == SOURCE and Path(positive['case']) == case
    assert positive['actual_formal_sessions'] == 8 and positive['observed_exit_code'] == 0
    assert positive['recorded_original_owned_processes_terminal']
    assert positive['hotel_owner_return_after_native_ack_timeout_and_fresh_auth_verified']
    positive_raw = positive_path.parent / 'raw.zip'
    assert sha(positive_raw) == positive['raw_archive_sha256']
    with zipfile.ZipFile(positive_raw) as archive:
        assert set(archive.namelist()) == set(positive['raw_files'])
        for name, digest in positive['raw_files'].items():
            assert hashlib.sha256(archive.read(name)).hexdigest().upper() == digest
    for name in ['cohort.json', 'Sky-events.json', 'server-trace.jsonl']:
        assert sha(case / name) == positive['raw_files'][name]

    events = read(case / 'Sky-events.json')
    assert events and utc(plan['declared_utc']) < utc(events[0]['observed_utc'])
    event_times = [utc(row['observed_utc']) for row in events]
    assert event_times == sorted(event_times)
    captures = list((case / 'captures').glob('*/frames.jsonl'))
    assert len(captures) == 1 and sha(captures[0]) == positive['raw_files']['frames.jsonl']
    frames = [json.loads(line) for line in captures[0].read_text('utf-8-sig').splitlines()]
    packets = [(row, bytes.fromhex(row['plain'])) for row in frames if row['plain'] != 'MASKED_C_LOGIN']
    outgoing = [(row, body) for row, body in packets if row['direction'] == 'client_to_server']
    replies = [(row, body) for row, body in outgoing if body[0] == 0x51]
    assert len(replies) == 2
    trace = [json.loads(line)['state'] for line in (case / 'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
    own = {role: [row for row in events if row['role'] == role] for role in ['a', 'b']}

    def event(role, label):
        found = [row for row in own[role] if row['label'] == label]
        assert found, (role, label)
        assert len(found) == 1 or all(row['action'] is None for row in found)
        return found[-1]

    def key(role, label):
        row = event(role, label)
        assert row['action'] == dict(method='press_key', args=dict(key='Return'))
        return row

    def pure(role, label):
        row = event(role, label)
        assert row['action'] is None
        return row

    def ordered(role, labels):
        positions = [own[role].index(event(role, label)) for label in labels]
        assert positions == sorted(positions) and len(set(positions)) == len(positions)

    def no_reply_between(start, end):
        assert start < end
        assert not any(start <= utc(row['time']) <= end for row, _ in replies)

    no_request = plan['no_request']
    assert no_request['role'] == 'a' and no_request['expected_new_C51'] == 0
    start = pure('a', 'world-focus')
    pressed = key('a', no_request['press_label'])
    after = pure('a', no_request['after_label'])
    ordered('a', ['world-focus', no_request['press_label'], no_request['after_label'], 'hotel-rent-request'])
    assert start['focused_element'].startswith('0 視窗')
    assert not re.search(r'\bID:\s*222\b', start['tree'])
    assert '地圖 500' in after['tree'] and '背包（45）' in after['tree']
    no_reply_between(utc(start['observed_utc']), utc(event('a', 'hotel-rent-request')['observed_utc']))

    consumed = plan['consumed_request']
    assert consumed['role'] == 'a' and consumed['expected_additional_C51'] == 0
    duplicate = key('a', consumed['press_label'])
    duplicate_after = pure('a', consumed['after_label'])
    ordered('a', ['hotel-quantity-submit', 'hotel-keys-granted', consumed['press_label'], consumed['after_label'], 'hotel-retalk', 'hotel-room-loaded'])
    assert '背包（53）' in duplicate_after['tree'] and '地圖 500' in duplicate_after['tree']
    valid_a = [(row, body) for row, body in replies if utc(row['time']) < utc(event('b', 'account')['observed_utc'])]
    assert len(valid_a) == 1 and int.from_bytes(valid_a[0][1][5:9], 'little') == 8
    assert utc(valid_a[0][0]['time']) < utc(event('a', 'hotel-keys-granted')['observed_utc']) < utc(duplicate['observed_utc'])
    no_reply_between(utc(event('a', 'hotel-keys-granted')['observed_utc']), utc(event('a', 'hotel-room-loaded')['observed_utc']))

    expired = plan['expired_request']
    assert expired['role'] == 'b' and expired['expected_old_request_C51'] == 0
    assert expired['actual_wait_seconds_minimum'] == 30
    assert expired['fresh_original_NPC_request_required_for_positive_case']
    requested = pure('b', expired['request_label'])
    value = event('b', expired['value_label'])
    assert value['action']['method'] == 'set_value' and value['action']['args']['value'] == '1'
    assert 'ID: 222' in value['focused_element'] and 'disabled' not in value['focused_element']
    wait_start = pure('b', expired['wait_start_label'])
    wait_end = pure('b', expired['wait_finished_label'])
    expired_key = key('b', expired['press_label'])
    expired_after = pure('b', expired['after_label'])
    ordered('b', [expired['request_label'], expired['value_label'], expired['wait_start_label'], expired['wait_finished_label'], expired['press_label'], expired['after_label'], 'hotel-talk', 'hotel-rent-request', 'hotel-quantity-requested', 'hotel-quantity-value', 'hotel-quantity-submit'])
    actual_wait = (utc(wait_end['observed_utc']) - utc(wait_start['observed_utc'])).total_seconds()
    assert actual_wait >= 30
    assert '請輸入 1～8 的數量' in requested['tree']
    assert '地圖 500' in expired_after['tree'] and '背包（43）' in expired_after['tree']
    old_requests = [(row, body) for row, body in packets if row['direction'] == 'server_to_client' and body[0] == 0x5b and utc(event('b', 'account')['observed_utc']) <= utc(row['time']) <= utc(requested['observed_utc'])]
    assert len(old_requests) == 1
    old_time = utc(old_requests[0][0]['time'])
    assert old_time <= utc(requested['observed_utc']) <= utc(wait_start['observed_utc'])
    assert utc(expired_key['observed_utc']) >= utc(wait_end['observed_utc'])
    fresh_request_action = event('b', 'hotel-rent-request')
    no_reply_between(old_time, utc(fresh_request_action['observed_utc']))
    fresh_requests = [(row, body) for row, body in packets if row['direction'] == 'server_to_client' and body[0] == 0x5b and utc(expired_after['observed_utc']) < utc(row['time']) <= utc(event('b', 'hotel-quantity-requested')['observed_utc'])]
    assert len(fresh_requests) == 1
    b_reply = next((row, body) for row, body in replies if utc(row['time']) > utc(event('b', 'account')['observed_utc']))
    assert b_reply[1][1:5] == fresh_requests[0][1][1:5]
    assert 0 <= (utc(b_reply[0]['time']) - utc(fresh_requests[0][0]['time'])).total_seconds() < 30

    # Check every actual authority state before the sole grant and through entry.
    # No trace callback is invented for a key that correctly sends no packet.
    fields = ['native_inventory', 'hp', 'mp', 'shop_points', 'rental_gems', 'character', 'rental_set']
    for role, account in [('a', 'PeerFixture'), ('b', 'PeerFixture2')]:
        states = [row for row in trace if row.get('test_account') == account and row.get('phase') == 'BATTLE' and row.get('rental_town')]
        assert len({row['session_identity'] for row in states}) == 1
        grant = next(i for i, row in enumerate(states) if row.get('rental_hotel') is not None)
        assert grant > 0
        initial = states[0]
        assert all(row.get('rental_hotel') is None and row['rental_inventory'] == initial['rental_inventory'] and all(row[field] == initial[field] for field in fields) for row in states[:grant])
        lease = positive['roles'][role]['lease']
        arrived = next(i for i, row in enumerate(states) if row.get('rental_map') == positive['roles'][role]['map_id'])
        assert all(row['rental_hotel'] == lease and row['rental_hotel_owner_lease_active'] is True and row['rental_inventory'] == states[grant]['rental_inventory'] and all(row[field] == initial[field] for field in fields) for row in states[grant:arrived + 1])

    return dict(status='qualified_physical_Return_no_request_consumed_and_expired_request_limited', source_manifest_sha256=SOURCE,
                case=str(case), profile=cohort['profile'], positive_qualification=str(positive_path), positive_qualification_sha256=sha(positive_path),
                original_session=positive['original_session'], reused_formal_sessions=8, new_formal_sessions=0,
                total_C51=2, no_request_C51=0, additional_consumed_C51=0, expired_request_C51=0,
                actual_expiry_wait_seconds=actual_wait, original_numeric_timeout_seconds=30,
                authority_and_inventory_unchanged_until_valid_reply=True, held_key_autorepeat_verified=False,
                actual_screenshots_qualified=False, GUI_full_flow_verified=False, whole_goal_complete=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('case', type=Path)
    args = parser.parse_args()
    case = args.case.resolve(strict=True)
    assert case.is_relative_to(V / 'native-installed-resource-profile-sky-gui')
    result = qualify(case)  # No output or acceptance mutation before all assertions.
    out = case / 'qualified-quantity-Enter-negatives'
    out.mkdir()
    files = {'qualifier.py': Path(__file__), 'plan.json': case / 'quantity-Enter-negative-plan.json',
             'positive-qualified.json': Path(result['positive_qualification']), 'Sky-events.json': case / 'Sky-events.json',
             'server-trace.jsonl': case / 'server-trace.jsonl', 'cohort.json': case / 'cohort.json'}
    files['frames.jsonl'] = next((case / 'captures').glob('*/frames.jsonl'))
    hashes = {name: sha(path) for name, path in files.items()}
    with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
        for name, path in files.items():
            archive.write(path, name)
    with zipfile.ZipFile(out / 'raw.zip') as archive:
        assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in hashes.items())
    result.update(raw_files=hashes, raw_archive_sha256=sha(out / 'raw.zip'), qualified_utc=datetime.now(timezone.utc).isoformat())
    (out / 'qualified.json').write_bytes(encode(result))
    print(json.dumps(result, ensure_ascii=False))
