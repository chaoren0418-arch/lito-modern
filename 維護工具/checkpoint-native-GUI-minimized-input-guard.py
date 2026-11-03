"""Preserve the observed Sky recovery failure without changing an active case."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import ast
import hashlib
import json
import subprocess
import sys
import zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()


def data(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')


w = read(R / '工作佇列.json')
assert w['active_workload_sessions'] == {'normal_gui': 77873}
case = Path(w['current_native_catalog_gui_active']['case'])
assert case.name == 'case-20261020-070332-388943-release-rental-hotel-return-reconnect'
c = read(case / 'cohort.json')
source = c['source_manifest_sha256']
assert source == w['current_product_identity']['source_manifest_sha256'] == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert c['status'] == 'normal_windows_running_for_Sky_inputs'
p = read(Path(c['prepared']) / 'prepared.json')
assert sha(Path(c['prepared']) / 'prepared.json') == c['prepared_sha256']
assert all(sha(R / name) == digest for name, digest in p['compiled_inputs'].items())
assert all(sha(row['path']) == sha(row['original_path']) == row['sha256'] for row in p['products']['Release'].values())
events = read(case / 'Sky-events.json')
assert events and all(row['action'] is None for row in events)
assert all(not path.read_bytes() for path in (case / 'captures').glob('*/frames.jsonl'))
assert not (case / 'server-trace.jsonl').exists() or not (case / 'server-trace.jsonl').read_bytes()
plan = case / 'quantity-Enter-negative-plan.json'
assert sha(plan) == '16AA8CACA48E7279EBE969E2F10F7C4D6D9A123E07180D9DB7619A61D4F091CB'
before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
out = V / 'native-quantity-enter-failure-tooling' / ('minimized-input-guard-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
reader = R / '維護工具/qualify-native-quantity-Enter-GUI-negatives.py'
ast.parse(reader.read_text('utf-8'))
rejections = []
for candidate in [case, V / 'native-installed-resource-profile-sky-gui/case-20261020-065512-667537-release-rental-hotel-return-reconnect', V / 'native-installed-resource-profile-sky-gui/case-20261020-024821-662397-debug-rental-hotel-return-reconnect']:
    target = candidate / 'qualified-quantity-Enter-negatives'
    assert not target.exists()
    run = subprocess.run([sys.executable, '-B', '-X', 'utf8', str(reader), str(candidate)], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
    assert run.returncode != 0 and b'AssertionError' in run.stderr and not target.exists()
    name = candidate.name + '.rejection.stderr'
    (out / name).write_bytes(run.stderr)
    rejections.append(dict(case=str(candidate), actual_exit_code=run.returncode, rejection_stderr=name, qualification_created=False))
assert all(sha(R / name) == digest for name, digest in before.items())
assert sha(plan) == '16AA8CACA48E7279EBE969E2F10F7C4D6D9A123E07180D9DB7619A61D4F091CB'
record = dict(status='Sky_minimized_and_input_guard_recovery_unavailable_active_case_preserved', source_manifest_sha256=source,
              case=str(case), original_session=77873, recorded_utc=datetime.now(timezone.utc).isoformat(),
              exact_state_error='window is minimized; call activate_window, refresh with get_window, then retry get_window_state',
              exact_activation_error='user input was detected in this window; call get_window_state before continuing',
              observations=['A returned current window; pure state minimized; activation input guard; pure recovery still minimized.',
                            'B separately returned current window; pure state minimized; activation input guard; pure recovery still minimized.'],
              cause_proven=False, user_attribution_proven=False, desktop_lock_observed=False,
              GUI_input_or_login_completed=False, actual_formal_sessions_observed=0,
              original_runtime_still_owns_cleanup=True, original_deadlines_changed=False,
              product_or_corresponding_active_QA_modified=False, new_negative_reader=str(reader), negative_reader_sha256=sha(reader),
              negative_reader_syntax_verified=True, actual_unqualified_cases_rejected=rejections,
              negative_reader_runtime_qualification_granted=False, GUI_full_flow_verified=False, whole_goal_complete=False)
files = {'negative-reader.py': reader, 'checkpoint.py': Path(__file__), 'cohort-at-observation.json': case / 'cohort.json', 'Sky-events.json': case / 'Sky-events.json', 'negative-plan.json': plan}
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name, path in files.items():
        archive.write(path, name)
with zipfile.ZipFile(out / 'raw.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == sha(path) for name, path in files.items())
record.update(raw_archive_sha256=sha(out / 'raw.zip'))
(out / 'evidence.json').write_bytes(data(record))
work = '原GUI77873尚未登入；A/B純觀測皆最小化，還原收到input guard，刷新仍最小化，停止同條件UI重試並保留原1800+30秒程序與清理責任；未證明原因或歸責使用者。另備Enter無請求／已消耗／逾時負向reader，三個實際未合格批次均拒授資格，尚無實跑通過。已完成當版library與81資源及返回API16不重跑；接續不依賴桌面的前提／契約檢查，GUI全流程與完整G01–G12未完成，不交人工。'
staged = {}
date = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        value['current_GUI_input_runtime_obstruction'] = dict(**record, evidence=str(out / 'evidence.json'), evidence_sha256=sha(out / 'evidence.json'))
        value.update(next_task=work, next_focus=work, last_goal_turn_evidence=str(out / 'evidence.json'), last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
        value['current_focus'] = dict(item='GUI_automation_recovery_and_independent_contract_work', work=work, applies_to_current_source=True, goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        staged[name] = data(value)
    else:
        first, sep, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert sep
        staged[name] = (date + '：' + work + '\n\n' + rest).encode('utf-8')
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(R / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in before.items())
assert all(sha(R / name) == digest for name, digest in before.items())
for name, payload in staged.items():
    (R / name).write_bytes(payload)
after = {name: sha(R / name) for name in before}
assert all(after[name] == hashlib.sha256(payload).hexdigest().upper() for name, payload in staged.items())
(out / 'readback.json').write_bytes(data(dict(before_files=before, after_files=after, before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json'))))
print(json.dumps(dict(evidence=str(out / 'evidence.json'), original_session_preserved=77873, negative_reader_unqualified_cases_rejected=len(rejections), GUI_verified=False), ensure_ascii=False))
