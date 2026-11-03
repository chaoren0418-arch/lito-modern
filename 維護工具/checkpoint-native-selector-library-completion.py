"""Publish the completed default library scope; retain the remaining game gates."""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import hashlib
import json
import zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')


ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', required=True, type=int)
ap.add_argument('--terminal-chunk-id', required=True)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-startup-library-default')
c = read(run / 'cohort.json')
done_path = run / 'completion-observation.json'
done = read(done_path)
assert c['status'] == 'executed_both_current_full_library_default_selections'
assert c['owned_process'] is None and len(c['process_history']) == 8
assert all(p['exit_code'] == 0 for p in c['process_history'])
assert done['status'] == 'independently_qualified_current_full_library_both_profiles_and_terminal_original_runner'
assert done['original_session_id'] == a.session_id and done['observed_terminal_exit_code'] == 0
assert done['recorded_owned_processes_all_terminal'] and done['source_files'] == 1028
binding = read(run / 'original-session-binding.json')
assert binding['original_session_id'] == a.session_id
assert binding['runner_identity'] == read(run / 'original-runner-process.json')
inputs = read(run / 'compiled-inputs.json')
assert len(inputs) == 1029
for name, digest in inputs.items():
    assert sha(ROOT / name) == digest, name
npath = Path(c['normal_qualification'])
n = read(npath)
assert sha(npath) == c['normal_qualification_sha256']
normal = read(npath.parent / 'cohort.json')
for profile, info in normal['profiles'].items():
    assert sha(info['normal']['path']) == done['normal_mains'][profile]
profiles = {}
for profile, info in done['profiles'].items():
    q = read(info['qualification'])
    assert sha(info['qualification']) == info['qualification_sha256']
    assert q['source_manifest_sha256'] == c['source_manifest_sha256'] == done['source_manifest_sha256']
    assert q['failed'] == q['filtered_out'] == 0 and q['inventory'] == q['passed'] + q['ignored']
    profiles[profile] = dict(info, diagnostic_PNGs=q['owned_output_PNGs'],
        restored_historical_reports=q['restored_historical_reports'], historical_pet_files=353)
assert set(profiles) == {'Debug', 'Release'}
record = dict(status=done['status'], run=str(run), completion_observation=str(done_path),
    completion_sha256=sha(done_path), original_session_id=a.session_id, terminal_tool_chunk_id=a.terminal_chunk_id,
    observed_terminal_exit_code=0, source_manifest_sha256=c['source_manifest_sha256'], source_files=1028,
    compiled_input_files=1029, normal_mains=done['normal_mains'], profiles=profiles,
    recorded_owned_identities=done['recorded_owned_identities'], recorded_owned_processes_all_terminal=True,
    qualified=True, ignored_bodies_executed=False, full_current_TCP_verified=False,
    normal_EXE_runtime_inferred=False, GUI_verified=False, whole_goal_complete=False)
names = ['工作佇列.json', '監製進度.json',
    '驗證紀錄/current/native-teleport-wire/progress.json', '驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json',
    'README.md', 'docs/ARCHITECTURE.zh-TW.md', 'docs/HISTORY.md']
for name in names[:5]:
    j = read(ROOT / name)
    assert j['current_product_identity']['source_manifest_sha256'] == c['source_manifest_sha256']
    for key, value in j.get('active_workloads', {}).items():
        assert key == 'selector-full-library' and value['original_session_id'] == a.session_id
out = run / ('completion-checkpoint-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {name: sha(ROOT / name) for name in names}
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in names:
        archive.write(ROOT / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    for name, digest in before.items():
        assert hashlib.sha256(archive.read(name)).hexdigest().upper() == digest
counts = '；'.join(profile + str(info['passed']) + '過／' + str(info['failed']) + '失敗／'
    + str(info['ignored']) + '原忽略，清單' + str(info['inventory']) + '項' for profile, info in profiles.items())
work = ('來源89D4B630完整library原' + str(a.session_id) + '已實際退出0並獨立核對兩版：' + counts
    + '；來源／兩份實際測試PE／原始結果與' + str(done['recorded_owned_identities']) + '組記錄身分終止均已核對。'
    '既有報告還原與353份測試資料維持；診斷PNG非正常GUI／physical／G11接受。'
    '同版廣域504／500過及G06控制器12案96sessions維持各自限定資格；忽略本文尚未執行。'
    '下一步map563、必要原資源opt-in與正常可見性，並繼續技能／服務／GUI／音效與可攜G11G12；完整G01–G12未完成。'
    '登入器唯讀且產物獨立，替換提案撤回，原時限／預算不變。')
for name in names[:5]:
    path = ROOT / name
    j = read(path)
    j.update(current_selector_full_library=record, active_workloads={}, active_workload_sessions={},
        source_freeze_for_current_full_library=dict(frozen=False, run=str(run), completion=str(done_path)),
        source_and_corresponding_QA_frozen=False, source_freeze_while_Cargo=False,
        last_goal_turn_classification='progress', last_goal_turn_evidence=record,
        next_task=work, next_focus=work, goal_complete=False, whole_goal_complete=False,
        progress_updated_utc=datetime.now(timezone.utc).isoformat())
    if name == names[4]:
        j['next_action'] = work
    else:
        j['current_focus'] = dict(item='source89D4-map-resources-visibility-pending', work=work,
                                  applies_to_current_source=True, goal_complete=False)
    for item in j.get('items') or []:
        if item['id'].startswith('G03'):
            item.update(state='source89D4_broad_and_default_library_qualified_remaining_regression_pending',
                        complete=False, current_full_library_evidence=record)
    for key, gate in (j.get('current_source_goal_validation') or {}).items():
        if key.startswith('G03'):
            gate.update(state='source89D4_broad_and_default_library_qualified_remaining_regression_pending', complete=False)
    save(path, j)
for name in names[5:]:
    path = ROOT / name
    first, separator, rest = path.read_text(encoding='utf-8-sig').partition('\n\n')
    assert first.startswith('2026-10-17') and separator
    path.write_text('2026-10-17：' + work + '\n\n' + rest, encoding='utf-8', newline='\n')
(out / 'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out / 'readback.json', dict(before_files=before, before_archive_sha256=sha(out / 'before.zip'),
    after_files={name: sha(ROOT / name) for name in names}, qualified_library_scope=record, whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out), profiles={p: v['passed'] for p, v in profiles.items()},
                       original_owned_identities=done['recorded_owned_identities']), ensure_ascii=False))
