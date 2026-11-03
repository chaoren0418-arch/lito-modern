"""Reconcile current selector-source trackers after the original G06 terminal.

Only the completed controller scope is promoted. An optional broad workload is
recorded only after its original live process identity has been observed.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import hashlib
import json
import subprocess
import time
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    for attempt in range(10):
        try:
            return json.loads(Path(path).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if attempt == 9:
                raise
            time.sleep(.05)


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n',
                    encoding='utf-8', newline='\n')


ap = argparse.ArgumentParser()
ap.add_argument('G06_run', type=Path)
ap.add_argument('--session-id', required=True, type=int)
ap.add_argument('--terminal-chunk-id', required=True)
ap.add_argument('--default-run', type=Path)
ap.add_argument('--default-session-id', type=int)
ap.add_argument('--default-dispatch-chunk-id')
a = ap.parse_args()
run = a.G06_run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-G06-controller')
c = read(run / 'cohort.json')
qpath = run / 'qualified/qualified.json'
q = read(qpath)
completion_path = run / 'completion-observation.json'
completion = read(completion_path)
assert c['status'] == 'executed_current_both_profiles_G06_controller_TW_extended_six_cases'
assert c['owned_process'] is None and all(p['exit_code'] == 0 for p in c['process_history'])
assert completion['original_session_id'] == q['original_session_id'] == a.session_id
assert completion['observed_terminal_exit_code'] == q['observed_terminal_exit_code'] == 0
assert completion['recorded_original_owned_processes_all_terminal']
assert completion['qualification_sha256'] == sha(qpath)
assert q['actual_cases'] == 12 and q['actual_formal_sessions'] == 96
assert completion['source_manifest_sha256'] == q['source_manifest_sha256'] == c['source_manifest_sha256']
pins = read(run / 'source-inputs.json')
assert len(pins) == q['source_files'] == 1028
assert sha(run / 'source-inputs.json') == q['source_manifest_sha256']
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
normal = read(c['startup_qualification'])
normal_run = Path(c['startup_prerequisite'])
normal_cohort = read(normal_run / 'cohort.json')
assert normal['status'] == 'independently_qualified_current_normal_selector_both_profiles'
for profile, info in normal_cohort['profiles'].items():
    assert sha(info['normal']['path']) == q['normal_mains'][profile]
    assert normal['profiles'][profile]['normal_sha256'] == q['normal_mains'][profile]

active = {}
broad = None
if a.default_run:
    assert a.default_session_id and a.default_dispatch_chunk_id
    broad_run = a.default_run.resolve(strict=True)
    assert broad_run.is_relative_to(ROOT / '驗證紀錄/current/native-default111')
    broad = read(broad_run / 'cohort.json')
    assert broad['source_manifest_sha256'] == q['source_manifest_sha256']
    assert broad['G06_qualification_sha256'] == sha(qpath)
    assert broad['status'] in ['executing_current_default111_profile',
                               'independently_reading_current_default111_profile']
    command = ('[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);'
               'Get-CimInstance Win32_Process -Filter "ProcessId='
               + str(broad['pipeline_pid']) + '"|Select-Object ProcessId,CommandLine,'
               '@{n="created";e={$_.CreationDate.ToUniversalTime().ToString("o")}}'
               '|ConvertTo-Json -Compress')
    observed = subprocess.run(['pwsh', '-NoProfile', '-NonInteractive', '-Command', command],
                              capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
    assert observed.returncode == 0 and not observed.stderr
    live = json.loads(observed.stdout.decode('utf-8-sig'))
    assert live['ProcessId'] == broad['pipeline_pid']
    assert abs((datetime.fromisoformat(live['created'].replace('Z', '+00:00')) -
                datetime.fromisoformat(broad['pipeline_creation_utc'])).total_seconds()) < .001
    assert str(ROOT / '維護工具/run-native-selector-default111-both.py').casefold() in live['CommandLine'].casefold()
    active['selector-default111'] = dict(run=str(broad_run),
        original_session_id=a.default_session_id,
        initial_tool_chunk_id=a.default_dispatch_chunk_id,
        status=broad['status'], active_profile=broad.get('active_profile'),
        source_manifest_sha256=q['source_manifest_sha256'], CIM=live,
        observed_utc=datetime.now(timezone.utc).isoformat(), qualified=False)
    profiles = {}
    for profile, info in broad['profiles'].items():
        if not info.get('qualified'):
            continue
        qualified = read(info['qualification'])
        assert sha(info['qualification']) == info['qualification_sha256']
        assert qualified['source_manifest_sha256'] == q['source_manifest_sha256']
        assert qualified['actual_test_executables'] == 112 and qualified['counts']['failed'] == 0
        profiles[profile] = dict(qualification=info['qualification'],
            qualification_sha256=info['qualification_sha256'],
            counts=qualified['counts'], actual_test_executables=112)
    active['selector-default111']['qualified_profiles'] = profiles

record = dict(run=str(run), qualification=str(qpath), qualification_sha256=sha(qpath),
    completion_observation=str(completion_path), completion_sha256=sha(completion_path),
    original_session_id=a.session_id, observed_terminal_exit_code=0,
    terminal_tool_chunk_id=a.terminal_chunk_id,
    status=q['status'], source_manifest_sha256=q['source_manifest_sha256'],
    source_files=1028, compiled_input_files=1029, normal_mains=q['normal_mains'],
    actual_cases=12, actual_formal_sessions=96,
    recorded_owned_identities=completion['recorded_original_owned_identities'],
    recorded_owned_processes_all_terminal=True, qualified=True,
    GUI_verified=False, physical_input_verified=False, same_endpoint_verified=False,
    whole_G06_complete=False, whole_goal_complete=False)
names = ['工作佇列.json', '監製進度.json',
    '驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json',
    'README.md', 'docs/ARCHITECTURE.zh-TW.md', 'docs/HISTORY.md']
for name in names[:5]:
    assert read(ROOT / name)['current_product_identity']['source_manifest_sha256'] == q['source_manifest_sha256']
out = run / ('continuation-checkpoint-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {name: sha(ROOT / name) for name in names}
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in names:
        archive.write(ROOT / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    assert set(archive.namelist()) == set(names)
    for name, digest in before.items():
        assert hashlib.sha256(archive.read(name)).hexdigest().upper() == digest
work = ('來源89D4B630正常兩版相關測試已通過（Debug155／Release151，各0失敗3原忽略）；'
        '原G06工作37081已退出0並獨立核對兩版12案／96sessions與全部記錄程序終止。'
        '此為跨端點與五種失敗恢復控制器範圍，GUI／同端點及完整G06仍待。'
        + ('同來源廣域111整合目標與main兩版正在執行，保留原handle及來源凍結。' if broad else
           '下一步同來源廣域111整合目標與main兩版。')
        + '之後全library、地圖／資源／可見性、GUI／實體音效、可攜G11G12與完整G01–G12仍待。'
          '登入器與伺服器唯讀，產物獨立，替換提案撤回；8秒／60秒／G11預算不變。')
if broad and profiles:
    work += ' 已獨立核對廣域版別：' + '；'.join(
        profile + str(info['counts']['passed']) + '過／' + str(info['counts']['failed'])
        + '失敗／' + str(info['counts']['ignored']) + '原忽略，112實際EXE'
        for profile, info in profiles.items()) + '。未完成版別仍待原工作終止，不算兩版整批完成。'


def update_transfer_check(node):
    if isinstance(node, list):
        for child in node:
            update_transfer_check(child)
    elif isinstance(node, dict):
        if node.get('id') == 'G06.3d':
            node.update(state='source89D4_controller_qualified_GUI_and_other_branches_pending',
                current_controller_evidence=record, complete=False,
                current_source_revalidation_note=work,
                remaining='來源89D4B630兩版跨端點及五種失敗恢復控制器12案／96sessions通過；同端點、普通WORLD／其他服務、當版旅館、正常GUI／實體輸入仍待，G06.3d未完成。')
            node.setdefault('retained_GUI_scope_before_source89D4', node.get('current_GUI_scope'))
            node['current_GUI_scope'] = dict(source_files=1028, compiled_inputs=1029,
                source_manifest_sha256=q['source_manifest_sha256'], verified=False)
            node['current_GUI_defined_scope_verified'] = False
            node['current_TW_GUI_defined_scope_verified'] = False
            return
        for child in node.values():
            update_transfer_check(child)


for name in names[:5]:
    path = ROOT / name
    j = read(path)
    j.update(current_selector_G06_controller=record, active_workloads=active,
        active_workload_sessions=active, source_freeze_for_current_G06_controller=False,
        goal_complete=False, whole_goal_complete=False, independent_projects=True,
        launcher_artifact_replacement_required=False,
        source_and_corresponding_QA_frozen=bool(broad),
        last_goal_turn_classification='progress',
        last_goal_turn_evidence=dict(G06=record, current_broad_scope=active.get('selector-default111')),
        progress_updated_utc=datetime.now(timezone.utc).isoformat())
    if isinstance(j.get('source_freeze_for_current_selector_validation'), dict):
        j['source_freeze_for_current_selector_validation'].update(frozen=False,
            terminal_scope='Both original related profiles and unique-entry normal builds already qualified; current freeze belongs to the recorded active workload.')
    if broad:
        j['current_selector_default111'] = active['selector-default111']
        j['source_freeze_for_current_default111_both'] = dict(frozen=True,
            run=str(broad_run), source_manifest_sha256=q['source_manifest_sha256'])
    j['next_task'] = work
    j['next_focus'] = work
    if name != names[4]:
        j['current_focus'] = dict(item='source89D4-default111' if broad else 'source89D4-G06-controller-qualified',
                                  work=work, applies_to_current_source=True, goal_complete=False)
    else:
        j['next_action'] = work
    for item in (j.get('items') or []):
        if item['id'].startswith('G01'):
            item['current_source_evidence_scope'] = '正常兩版、四項雜湊及29相關EXE已核對；僅此基準範圍通過。'
        if item['id'].startswith('G03'):
            item['state'] = 'source89D4_related_and_G06_controller_qualified_broad_pending'
            item['current_source_evidence_scope'] = work
        if item['id'].startswith('G06'):
            item.update(state='source89D4_G06_controller_qualified_full_scope_pending',
                        complete=False, current_evidence=record,
                        current_source_evidence_scope=work)
        if item['id'] in j.get('current_source_goal_validation', {}):
            j['current_source_goal_validation'][item['id']].update(
                state=item['state'], complete=item.get('complete', False))
    update_transfer_check(j.get('items', []))
    gates = j.get('current_source_goal_validation') or {}
    for key, gate in gates.items():
        if key.startswith('G03'):
            gate.update(state='source89D4_related_and_G06_controller_qualified_broad_pending', complete=False)
        if key.startswith('G06'):
            gate.update(state='source89D4_G06_controller_qualified_full_scope_pending', complete=False)
    save(path, j)
paragraph = ('2026-10-17：' + work + ' 正常Debug5D634328／Release1E452012。')
for name in names[5:]:
    path = ROOT / name
    first, sep, rest = path.read_text(encoding='utf-8-sig').partition('\n\n')
    assert first.startswith('2026-10-17') and sep
    path.write_text(paragraph + '\n\n' + rest, encoding='utf-8', newline='\n')
(out / 'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out / 'readback.json', dict(before_files=before,
    before_archive_sha256=sha(out / 'before.zip'), after_files={n: sha(ROOT / n) for n in names},
    G06=record, active_workloads=active, whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out), G06_qualified_sessions=96,
                       active_workloads=list(active)), ensure_ascii=False))
