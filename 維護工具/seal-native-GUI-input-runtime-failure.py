"""Seal the exact owned input-unavailable cleanup; never call it a GUI pass."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib
import json
import zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data = lambda value: (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
w = read(R / '工作佇列.json')
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
obstruction = w['current_GUI_input_runtime_obstruction']
case = Path(obstruction['case'])
assert case.is_relative_to(V / 'native-installed-resource-profile-sky-gui')
c = read(case / 'cohort.json')
assert c['status'] == 'failed_preserved' and c['recorded_owned_processes_terminal']
source = c['source_manifest_sha256']
assert source == w['current_product_identity']['source_manifest_sha256']
terminal_path = next(case.glob('checkpoint-terminal-*/evidence.json'))
terminal = read(terminal_path)
assert terminal['original_session_id'] == 77873 and terminal['terminal_tool_chunk'] == '14775d'
assert terminal['observed_actual_exit_code'] == 1 and terminal['original_identities_terminal']
stop_path = case / 'owned-input-unavailable-stop.json'
stop = read(stop_path)
assert stop['original_session'] == 77873 and stop['original_GUI_normal_close_claimed'] is False
assert len(stop['records']) == len(c['processes']) == 3
for row, observed in zip(c['processes'], stop['records']):
    assert row['role'] == observed['role'] and row['pid'] == observed['pid']
    assert (row['exit_code'] & 0xffffffff) == (observed['exit_code'] & 0xffffffff) == 4294967295
    assert datetime.fromisoformat(row['creation_utc']) == datetime.fromisoformat(observed['creation_utc'])
    assert observed['cleanup_started'] is True
events = read(case / 'Sky-events.json')
assert events and all(row['action'] is None for row in events)
frames = list((case / 'captures').glob('*/frames.jsonl'))
assert all(not path.read_text('utf-8-sig').strip() for path in frames)
assert not (case / 'server-trace.jsonl').read_text('utf-8-sig').strip()
p = read(Path(c['prepared']) / 'prepared.json')
assert sha(Path(c['prepared']) / 'prepared.json') == c['prepared_sha256']
assert all(sha(R / name) == digest for name, digest in p['compiled_inputs'].items())
assert all(sha(row['path']) == sha(row['original_path']) == row['sha256'] for row in p['products']['Release'].values())
paths = V / 'native-current-skill-request-scope/handler-paths-20261020-071630-321475/paths.json'
mapping = read(paths)
assert mapping['source_manifest_sha256'] == source and mapping['IDs_with_static_handler_paths'] == mapping['missing_IDs'] == 37
assert not mapping['request_shape_or_runtime_qualification_granted']
assert sha(mapping['selected_methods']) == mapping['selected_methods_sha256']
before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
out = case / ('input-runtime-failure-analysis-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
files = {path.relative_to(case).as_posix(): path for path in case.iterdir() if path.is_file()}
files.update({'sealer.py': Path(__file__), 'terminal.json': terminal_path, 'prepared.json': Path(c['prepared']) / 'prepared.json',
              'input-obstruction.json': Path(obstruction['evidence']), 'skill-handler-paths.json': paths,
              'skill-selected-methods.json': Path(mapping['selected_methods']),
              'cleanup.ps1': R / '維護工具/stop-owned-GUI-cohort-input-unavailable.ps1'})
files.update({path.relative_to(case).as_posix(): path for path in frames})
pins = {name: sha(path) for name, path in files.items()}
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name, path in files.items():
        archive.write(path, name)
with zipfile.ZipFile(out / 'raw.zip') as archive:
    assert set(archive.namelist()) == set(pins)
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in pins.items())
record = dict(status='owned_GUI_input_runtime_unavailable_failure_preserved_no_acceptance', source_manifest_sha256=source,
              case=str(case), original_session=77873, original_terminal_chunk='14775d', original_exit_code=1,
              actual_formal_sessions=0, original_children_exit_codes=[row['exit_code'] for row in c['processes']],
              original_identities_terminal=True, cleanup_was_explicit_failure_cleanup=True, GUI_normal_close_claimed=False,
              cause_proven=False, user_attribution_proven=False, GUI_qualification_granted=False,
              raw_archive=str(out / 'raw.zip'), raw_archive_sha256=sha(out / 'raw.zip'), raw_files=pins,
              current_missing_skill_handler_paths=str(paths), current_missing_skill_handler_paths_sha256=sha(paths),
              static_handler_link_count=37, skill_runtime_qualification_granted=False, whole_goal_complete=False)
(out / 'evidence.json').write_bytes(data(record))
work = '當版GUI原77873因Sky最小化／input guard無法恢復，0sessions；僅精確owned三個子程序做失敗清理，原wrapper1及四身分終止已核對，不宣稱正常關閉或GUI通過。Enter負向reader僅準備與拒授核對；另已逐名索引37個未接入技能的正式75 handler／讀包路徑，仍須分支／成本／狀態／資源與實跑驗證。接續當版控制器96與廣域Release等獨立工作；GUI需工具狀態實際改變後恢復，不重試同條件；已完成library與81資源及API16不重跑，不交人工。'
staged = {}
date = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        value['current_normal_GUI_input_runtime_failure'] = record
        value['current_missing_skill_handler_paths'] = dict(path=str(paths), sha256=sha(paths), linked_IDs=37, static_only=True, new_sessions=0, new_casts=0)
        value.update(next_task=work, next_focus=work, last_goal_turn_evidence=str(out / 'evidence.json'), last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
        value['current_focus'] = dict(item='controller_broad_regression_and_GUI_obstruction', work=work, applies_to_current_source=True, goal_complete=False)
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
(out / 'readback.json').write_bytes(data(dict(before_files=before, after_files={name: sha(R / name) for name in before}, before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json'))))
print(json.dumps(dict(evidence=str(out / 'evidence.json'), original_exit_code=1, actual_formal_sessions=0, GUI_qualified=False, static_skill_paths=37), ensure_ascii=False))
