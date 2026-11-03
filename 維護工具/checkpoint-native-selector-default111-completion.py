"""Record the completed broad scope without promoting full-library/GUI gates."""
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
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-default111')
c = read(run / 'cohort.json')
done_path = run / 'completion-observation.json'
done = read(done_path)
assert c['status'] == 'independently_qualified_current_default111_both_profiles'
assert c['owned_process'] is None and all(p['exit_code'] == 0 for p in c['process_history'])
assert done['status'] == 'independently_observed_current_default111_both_profiles_and_terminal_original_pipeline'
assert done['original_session_id'] == a.session_id and done['observed_terminal_exit_code'] == 0
assert done['recorded_original_owned_processes_all_terminal']
binding = read(run / 'original-session-binding.json')
assert binding['original_session_id'] == a.session_id and binding['pipeline_pid'] == c['pipeline_pid']
assert binding['pipeline_creation_utc'] == c['pipeline_creation_utc']
normal_path = Path(c['normal_qualification'])
n = read(normal_path)
assert sha(normal_path) == c['normal_qualification_sha256']
assert n['source_manifest_sha256'] == c['source_manifest_sha256'] == done['source_manifest_sha256']
inputs = read(normal_path.parent / 'compiled-inputs.json')
assert len(inputs) == 1029
for name, digest in inputs.items():
    assert sha(ROOT / name) == digest, name
normal = read(normal_path.parent / 'cohort.json')
for profile, info in normal['profiles'].items():
    assert sha(info['normal']['path']) == done['normal_mains'][profile]
gpath = Path(c['G06_qualification'])
g = read(gpath)
assert sha(gpath) == c['G06_qualification_sha256']
assert g['source_manifest_sha256'] == c['source_manifest_sha256'] and g['actual_formal_sessions'] == 96
formal = ROOT.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(formal) == g['reference_jar_sha256']
profiles = {}
for profile in ['Debug', 'Release']:
    info = c['profiles'][profile]
    q = read(info['qualification'])
    assert info['qualified'] and info['actual_runner_exit_code'] == 0
    assert sha(info['qualification']) == info['qualification_sha256']
    assert q['actual_test_executables'] == 112 and q['integration_targets'] == 111
    assert q['counts']['failed'] == 0 and q['source_manifest_sha256'] == c['source_manifest_sha256']
    profiles[profile] = dict(qualification=info['qualification'], qualification_sha256=info['qualification_sha256'],
        counts=q['counts'], actual_test_executables=112, integration_targets=111)
record = dict(status=done['status'], run=str(run), completion_observation=str(done_path),
    completion_sha256=sha(done_path), original_session_id=a.session_id, terminal_tool_chunk_id=a.terminal_chunk_id,
    observed_terminal_exit_code=0, source_manifest_sha256=c['source_manifest_sha256'], source_files=1028,
    compiled_input_files=1029, profiles=profiles, normal_mains=done['normal_mains'],
    reference_JAR_sha256=g['reference_jar_sha256'], reference_JAR_unchanged_at_checkpoint=True,
    recorded_original_owned_identities=done['recorded_original_owned_identities'],
    recorded_original_owned_processes_all_terminal=True, qualified=True,
    full_library_scope_inferred=False, ignored_bodies_executed=False, normal_EXE_runtime_inferred=False,
    GUI_verified=False, whole_goal_complete=False)
names = ['工作佇列.json', '監製進度.json',
    '驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json',
    'README.md', 'docs/ARCHITECTURE.zh-TW.md', 'docs/HISTORY.md']
for name in names[:5]:
    j = read(ROOT / name)
    assert j['current_product_identity']['source_manifest_sha256'] == c['source_manifest_sha256']
    for key, value in j.get('active_workloads', {}).items():
        assert key == 'selector-default111' and value['original_session_id'] == a.session_id
out = run / ('completion-checkpoint-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {name: sha(ROOT / name) for name in names}
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in names:
        archive.write(ROOT / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    assert set(archive.namelist()) == set(names)
    for name, digest in before.items():
        assert hashlib.sha256(archive.read(name)).hexdigest().upper() == digest
summary = '；'.join(profile + str(info['counts']['passed']) + '過／'
    + str(info['counts']['failed']) + '失敗／' + str(info['counts']['ignored']) + '原忽略'
    for profile, info in profiles.items())
work = ('來源89D4B630廣域原' + str(a.session_id) + '已實際退出0，兩版各112實際EXE（111整合目標＋main）已獨立核對：'
    + summary + '；來源／逐映像／原始結果及' + str(done['recorded_original_owned_identities']) + '組記錄程序身分終止已核對。'
    '同版G06控制器12案96sessions與正常兩版相關範圍各自資格保留。'
    '下一步同來源完整library、map563／必要資源／正常可見性；GUI／實體輸入／音效、同端點、可攜G11G12及完整G01–G12仍待。'
    '忽略本文未計入執行；8秒／60秒／G11預算維持原值。登入器只讀且產物獨立，替換提案撤回。')
for name in names[:5]:
    path = ROOT / name
    j = read(path)
    j.update(current_selector_default111=record, active_workloads={}, active_workload_sessions={},
        source_freeze_for_current_default111_both=dict(frozen=False, run=str(run), completion=str(done_path)),
        source_and_corresponding_QA_frozen=False, source_freeze_while_Cargo=False,
        last_goal_turn_classification='progress', last_goal_turn_evidence=record,
        goal_complete=False, whole_goal_complete=False, independent_projects=True,
        launcher_artifact_replacement_required=False, next_task=work, next_focus=work,
        progress_updated_utc=datetime.now(timezone.utc).isoformat())
    if name == names[4]:
        j['next_action'] = work
    else:
        j['current_focus'] = dict(item='source89D4-library-map-resources-pending', work=work,
                                  applies_to_current_source=True, goal_complete=False)
    for item in j.get('items') or []:
        if item['id'].startswith('G03'):
            item.update(state='source89D4_related_G06_default111_qualified_full_library_pending', complete=False,
                        current_broad_evidence=record, current_source_evidence_scope=work)
    for key, gate in (j.get('current_source_goal_validation') or {}).items():
        if key.startswith('G03'):
            gate.update(state='source89D4_related_G06_default111_qualified_full_library_pending', complete=False)
    save(path, j)
for name in names[5:]:
    path = ROOT / name
    first, separator, rest = path.read_text(encoding='utf-8-sig').partition('\n\n')
    assert first.startswith('2026-10-17') and separator
    path.write_text('2026-10-17：' + work + '\n\n' + rest, encoding='utf-8', newline='\n')
(out / 'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out / 'readback.json', dict(before_files=before, before_archive_sha256=sha(out / 'before.zip'),
    after_files={name: sha(ROOT / name) for name in names}, qualified_broad_scope=record, whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out), counts={p: v['counts'] for p, v in profiles.items()},
                       original_owned_identities=done['recorded_original_owned_identities']), ensure_ascii=False))
