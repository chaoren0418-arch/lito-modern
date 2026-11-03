"""Record off-tree components separately from normal product qualifications."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse
import hashlib
import json
import re
import subprocess
import zipfile
R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
data = lambda value: (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
parser = argparse.ArgumentParser()
parser.add_argument('run', type=Path)
parser.add_argument('--mode', choices=['active', 'terminal'], required=True)
parser.add_argument('--session', type=int)
parser.add_argument('--terminal-chunk')
parser.add_argument('--exit-code', type=int)
args = parser.parse_args()
run = args.run.resolve(strict=True)
assert run.is_relative_to(V / 'native-haste43-components')
w = read(R / '工作佇列.json')
c = read(run / 'cohort.json')
source = c['base_source_manifest_sha256']
assert source == w['current_product_identity']['source_manifest_sha256'] == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert not c['normal_source_tree_modified'] and not c['candidate_applied_to_product']
assert c['candidate_included_as_off_tree_modules'] and c['actual_formal75_casts'] == 0
candidate_path = Path(c['candidate'])
candidate = read(candidate_path)
assert sha(candidate_path) == c['candidate_sha256']
assert all(sha(R / name) == digest for name, digest in candidate['before'].items())
assert all(sha(candidate['candidate_files'][name]) == digest for name, digest in candidate['after'].items())
prep_path = Path(c['preparation'])
prep = read(prep_path)
assert sha(prep_path) == c['preparation_sha256'] and prep['candidate_sha256'] == sha(candidate_path)
normal = w['current_native_installed_resource_profile_normal_qualification']
assert all(sha(R / name) == digest for name, digest in read(Path(normal['run']) / 'compiled-inputs.json').items())
normal_q = read(normal['qualification'])
for profile, product_sha in normal_q['normal_mains'].items():
    assert sha(normal_q['profiles'][profile]['normal']['path']) == product_sha
for profile, info in c['profiles'].items():
    assert all(sha(record['path']) == record['sha256'] for record in info['extern'].values())
    assert sha(info['broad_build_log']) == info['broad_build_log_sha256']
for info in prep['probe_sources'].values():
    assert sha(info['path']) == info['sha256']
assert all(sha(name) == digest for name, digest in prep['compatibility_test_sources'].items())
assert all(sha(name) == info['sha256'] == sha(info['source']) for name, info in prep['fixtures'].items())
rows = [c['pipeline'], *c['process_history']]
ids = ','.join(str(row['pid']) for row in rows)
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @(" + ids + ")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
query = subprocess.run(['pwsh', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert query.returncode == 0 and not query.stderr
live = json.loads(query.stdout.decode('utf-8-sig')) if query.stdout.strip() else []
live = [live] if isinstance(live, dict) else live
same = lambda current, original: current['ProcessId'] == original['pid'] and abs((datetime.fromisoformat(current['created'].replace('Z', '+00:00')) - datetime.fromisoformat(original['creation_utc'])).total_seconds()) < .001
terminal = args.mode == 'terminal'
if not terminal:
    assert args.session is not None and args.session > 0 and args.exit_code is None and args.terminal_chunk is None
    assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
    assert any(same(current, c['pipeline']) for current in live)
else:
    assert args.exit_code == 0 and args.terminal_chunk not in [None, '', 'unknown', 'pending']
    assert c['status'] == 'executed_off_tree_haste43_baseline_and_candidate_components_both_profiles'
    assert c['owned_process'] is None and len(c['process_history']) == 8
    assert [row['exit_code'] for row in c['process_history']] == [0, 101, 0, 0] * 2
    assert all(not row.get('exact_owned_handle_timeout_cleanup') for row in c['process_history'])
    assert not any(same(current, original) for current in live for original in rows)
    assert not w['active_workloads'] or w['active_workload_sessions'] == {'haste43_components': args.session}
    assert set(c['profiles']) == {'Debug', 'Release'}
    for profile, info in c['profiles'].items():
        assert set(info['modes']) == {'baseline', 'candidate'}
        for mode, proof in info['modes'].items():
            assert sha(proof['executable']) == proof['sha256']
            log = (run / profile / (mode + '-run/stdout.log')).read_text('utf-8-sig')
            results = re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$', log, re.M)
            assert dict(results) == proof['results']
            if mode == 'baseline':
                assert len(results) == 3 and set(proof['results']) == set(prep['baseline_selected_expected_failures'])
                assert all(result == 'FAILED' for _, result in results)
                assert '0 passed; 3 failed; 0 ignored; 0 measured; 17 filtered out' in log
            else:
                assert len(results) == 20 and all(result == 'ok' for _, result in results)
                assert '20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in log
before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
out = run / ('checkpoint-' + args.mode + '-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'owned-CIM.json').write_bytes(query.stdout)
(out / 'reader.py').write_bytes(Path(__file__).read_bytes())
record = dict(status='qualified_off_tree_haste43_components_candidate_unapplied' if terminal else 'running_off_tree_haste43_components_candidate_unapplied',
              run=str(run), base_source_manifest_sha256=source, source_manifest_sha256=source,
              source_manifest_field_scope='Current unchanged normal product tree only; candidate modules are separately pinned, not normal product PEs.',
              candidate=str(candidate_path), candidate_sha256=sha(candidate_path), preparation=str(prep_path), preparation_sha256=sha(prep_path),
              original_session=args.session, terminal_tool_chunk=args.terminal_chunk, observed_exit_code=args.exit_code,
              qualified=terminal, profiles=c['profiles'], baseline_expected_failures_each=3, candidate_tests_each=20,
              recorded_original_identities=len(rows), recorded_original_identities_terminal=terminal,
              normal_product_source_modified=False, candidate_applied=False, normal_product_qualification_granted=False,
              actual_formal75_casts=0, live_skill43_qualified=False, GUI_verified=False, physical_audio_verified=False, whole_goal_complete=False)
if terminal:
    # A dependency-only preparation can reuse pinned assets in its predecessor.
    artifact_root = Path(prep.get('supersedes_preparation', prep_path)).parent.resolve(strict=True)
    if 'supersedes_preparation' in prep:
        assert sha(prep['supersedes_preparation']) == prep['supersedes_sha256']
    assert artifact_root.is_relative_to(candidate_path.parent)
    files = {path.relative_to(run).as_posix(): path for path in run.rglob('*') if path.is_file() and not path.relative_to(run).parts[0].startswith('checkpoint-')}
    files.update({'reader.py': Path(__file__), 'owned-CIM.json': out / 'owned-CIM.json', 'candidate.json': candidate_path, 'preparation/preparation.json': prep_path})
    for name, path in candidate['candidate_files'].items():
        files['candidate/' + name] = Path(path)
    for name in candidate['before']:
        files['baseline/' + name] = R / name
    for proof in prep['probe_sources'].values():
        path = Path(proof['path'])
        files['preparation/' + path.resolve(strict=True).relative_to(artifact_root).as_posix()] = path
    for name in [*prep['compatibility_test_sources'], *prep['fixtures']]:
        path = Path(name)
        files['preparation/' + path.resolve(strict=True).relative_to(artifact_root).as_posix()] = path
    for profile, info in c['profiles'].items():
        files['normal-libraries/' + profile + '/lito_modern.rlib'] = Path(info['extern']['lito_modern']['path'])
    pins = {name: sha(path) for name, path in files.items()}
    with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as archive:
        for name, path in files.items():
            archive.write(path, name)
    with zipfile.ZipFile(out / 'raw.zip') as archive:
        assert set(archive.namelist()) == set(pins)
        assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in pins.items())
    record.update(raw_files=pins, raw_archive_sha256=sha(out / 'raw.zip'))
    (out / 'qualified.json').write_bytes(data(record))
    record.update(qualification=str(out / 'qualified.json'), qualification_sha256=sha(out / 'qualified.json'))
(out / 'evidence.json').write_bytes(data(record))
resource_total = 85 if w.get('current_library_haste_resource2',{}).get('qualified') else 83
work = ('#43加速術隔離元件兩版已核對：每版原版3個預期反例及候選20過0敗；只在獨立wrapper引用候選兩模組，正常來源仍A695，未套用產品、正式75施放0、GUI未驗。下一步正式75原始技能書、自用／對人／MP及目標拒絕、動作／S29／755／回復生命週期，再決定套用與重驗。' if terminal else '#43加速術隔離元件沿原批次執行；正常來源與對應QA固定，原版反例及候選結果尚待實測與獨立核對，不授正常產品或GUI資格。') + '已通過當版正常／完整library／'+str(resource_total)+'資源／API16／控制器96與廣域兩版依各自資格保留；其他G01–G12與完整GUI仍待，先自行測試後才交人工。'
staged = {}
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        value['current_native_haste43_components'] = record
        value.update(active_workloads={} if terminal else {'haste43_components': record}, active_workload_sessions={} if terminal else {'haste43_components': args.session},
                     source_and_corresponding_QA_frozen=not terminal, next_task=work, next_focus=work,
                     last_goal_turn_evidence=str(out / 'evidence.json'), last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
        value['current_focus'] = dict(item='off_tree_haste43_components', work=work, applies_to_current_source=False, candidate_applied=False, goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        staged[name] = data(value)
    else:
        first, sep, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert sep
        staged[name] = (datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d') + '：' + work + '\n\n' + rest).encode('utf-8')
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(R / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in before.items())
assert all(sha(R / name) == digest for name, digest in before.items())
for name, payload in staged.items():
    (R / name).write_bytes(payload)
(out / 'readback.json').write_bytes(data(dict(before_files=before, after_files={name: sha(R / name) for name in before}, before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out), qualified=terminal, candidate_tests_each=20, normal_product_qualified=False, formal75_casts=0), ensure_ascii=False))
