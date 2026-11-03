"""Register exact original GUI Return negatives and partial tool recovery."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse
import hashlib
import json
import zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()
data = lambda value: (json.dumps(value, ensure_ascii=False, indent=2)+'\n').encode('utf-8')
ap = argparse.ArgumentParser()
ap.add_argument('case', type=Path)
ap.add_argument('--negative-terminal-chunk', required=True)
a = ap.parse_args()
case = a.case.resolve(strict=True)
assert case.is_relative_to(V / 'native-installed-resource-profile-sky-gui')
assert a.negative_terminal_chunk not in ['', 'pending', 'unknown']
w = read(R / '工作佇列.json')
source = w['current_product_identity']['source_manifest_sha256']
assert source == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
negative = case / 'qualified-quantity-Enter-negatives/qualified.json'
q = read(negative)
positive = Path(q['positive_qualification'])
p = read(positive)
assert q['source_manifest_sha256'] == p['source_manifest_sha256'] == source
assert q['profile'] == p['profile'] and Path(q['case']) == Path(p['case']) == case
assert sha(positive) == q['positive_qualification_sha256']
accepted = w['current_normal_GUI_hotel_return_keyboard'][q['profile']]
assert Path(accepted['qualification']) == positive and sha(positive) == accepted['qualification_sha256']
assert p['actual_formal_sessions'] == q['reused_formal_sessions'] == 8
assert q['new_formal_sessions'] == 0 and q['total_C51'] == 2
assert q['no_request_C51'] == q['additional_consumed_C51'] == q['expired_request_C51'] == 0
assert q['actual_expiry_wait_seconds'] >= 30 and q['original_numeric_timeout_seconds'] == 30
assert q['authority_and_inventory_unchanged_until_valid_reply']
assert p['recorded_original_owned_processes_terminal'] and p['observed_exit_code'] == 0
for path, proof in [(negative, q), (positive, p)]:
    assert sha(path.parent / 'raw.zip') == proof['raw_archive_sha256']
    with zipfile.ZipFile(path.parent / 'raw.zip') as archive:
        assert set(archive.namelist()) == set(proof['raw_files'])
        assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in proof['raw_files'].items())
    assert not proof['GUI_full_flow_verified']
before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
out = case / ('checkpoint-Enter-recovery-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
record = dict(status='current_source_GUI_Return_three_negatives_qualified_limited', source_manifest_sha256=source,
              profile=q['profile'], case=str(case), qualification=str(negative), qualification_sha256=sha(negative),
              raw_archive=str(negative.parent / 'raw.zip'), raw_archive_sha256=q['raw_archive_sha256'],
              original_session=q['original_session'], negative_reader_actual_terminal_chunk=a.negative_terminal_chunk,
              negative_reader_actual_exit_code=0, reused_formal_sessions=8, new_formal_sessions=0,
              total_C51=2, no_request_C51=0, consumed_request_C51=0, expired_request_C51=0,
              actual_expiry_wait_seconds=q['actual_expiry_wait_seconds'], held_key_autorepeat_verified=False,
              actual_screenshots_qualified=False, GUI_full_flow_verified=False, whole_goal_complete=False)
capture = case / 'fresh-desktop-capture-attempt.json'
recovery = dict(status='actual_owned_keyboard_accessibility_recovered_capture_still_pending', source_manifest_sha256=source,
                case=str(case), original_session=q['original_session'], normal_keyboard_and_accessibility_worked=True,
                qualified_hotel_return=str(positive), qualified_hotel_return_sha256=sha(positive),
                visual_capture_recovered=False, GUI_full_flow_verified=False, whole_goal_complete=False)
if capture.exists():
    attempt = read(capture)
    assert not attempt['success'] and 'FrameArrived timed out' in attempt['error']
    recovery.update(capture_attempt=str(capture), capture_attempt_sha256=sha(capture), capture_error=attempt['error'])
else:
    previous = w.get('current_GUI_input_runtime_recovery', {})
    if previous.get('source_manifest_sha256') == source and previous.get('capture_attempt'):
        assert sha(previous['capture_attempt']) == previous['capture_attempt_sha256']
        recovery.update({key: previous[key] for key in ['capture_attempt', 'capture_attempt_sha256', 'capture_error']})
        recovery['capture_observation_inherited_from_case'] = previous['case']
recovery['checkpoint_evidence'] = str(out / 'evidence.json')
record.update(tool_recovery=recovery)
(out / 'evidence.json').write_bytes(data(record))
(out / 'checkpoint.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(R / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in before.items())
extra = [path for path in [capture, case / 'reader-ordering-error.json'] if path.exists()]
with zipfile.ZipFile(out / 'tool-observations.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for path in extra:
        archive.write(path, path.name)
with zipfile.ZipFile(out / 'tool-observations.zip') as archive:
    assert all(hashlib.sha256(archive.read(path.name)).hexdigest().upper() == sha(path) for path in extra)
profiles = sorted(set(w.get('current_normal_GUI_quantity_Enter_negatives', {})) | {q['profile']})
state = 'current_source_hotel_return_Enter_negatives_qualified_'+'_'.join(profiles)+'_other_GUI_pending'
pending = [profile for profile in ['Debug', 'Release'] if profile not in profiles]
work = q['profile']+'旅館GUI原'+str(q['original_session'])+'正常exit0，8 sessions與8／1鑰匙、map503／501、返回重新認證及租約退役均獨立核對；Enter無請求／已消耗／自然失效三負向均無額外C51。限定通過版本：'+'／'.join(profiles)+'。鍵盤與可及性已實測恢復，畫面仍FrameArrived逾時。接續'+('／'.join(pending)+'旅館、'if pending else'')+'房間／租裝入口、EOF清理及其餘GUI；已核對library、廣域及GUI不重跑，未交人工。'
staged = {}
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        value.setdefault('current_normal_GUI_quantity_Enter_negatives', {})[q['profile']] = record
        value['current_GUI_input_runtime_recovery'] = recovery
        value['current_source_goal_validation'].setdefault('G08-player-gui', {}).update(state=state, complete=False)
        for item in value.get('items', []):
            if item.get('id') == 'G08-player-gui':
                item.update(state=state, complete=False)
        value.update(next_task=work, next_focus=work, last_goal_turn_evidence=str(out / 'evidence.json'),
                     last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        staged[name] = data(value)
    else:
        first, sep, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert sep
        staged[name] = (datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
assert all(sha(R / name) == digest for name, digest in before.items())
for name, payload in staged.items():
    (R / name).write_bytes(payload)
(out / 'readback.json').write_bytes(data(dict(before_files=before, after_files={name: sha(R / name) for name in before},
                                            evidence_sha256=sha(out / 'evidence.json'), before_archive_sha256=sha(out / 'before.zip'),
                                            tool_observations_archive_sha256=sha(out / 'tool-observations.zip'))))
print(json.dumps(dict(checkpoint=str(out), profile=q['profile'], qualified_Enter_negatives=3, GUI_full_flow_verified=False), ensure_ascii=False))
