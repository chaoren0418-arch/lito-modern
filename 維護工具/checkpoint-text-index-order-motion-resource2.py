"""Reserve or independently qualify only the exact two resource-body executions."""
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
parser.add_argument('--session', type=int)
parser.add_argument('--mode', choices=['active', 'terminal'], required=True)
parser.add_argument('--terminal-chunk')
parser.add_argument('--exit-code', type=int)
args = parser.parse_args()
run = args.run.resolve(strict=True)
assert run.is_relative_to(V / 'native-current-library-motion-resource2')
assert args.session is None or args.session > 0
c = read(run / 'cohort.json')
source = c['source_manifest_sha256']
w = read(R / '工作佇列.json')
assert source == w['current_product_identity']['source_manifest_sha256'] == '3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
plan_path = Path(c['plan'])
assert sha(plan_path) == c['plan_sha256']
plan = read(plan_path)
names = set(plan['names'])
assert len(names) == 2 and plan['expected_tests_per_profile'] == c['expected_tests_per_profile'] == 2
assert sha(plan['test_bodies']) == plan['test_bodies_sha256'] and sha(run / 'runner.py') == plan['runner_sha256']
library = Path(c['library'])
assert sha(library / 'completion-observation.json') == c['library_completion_sha256'] == plan['library_completion_sha256']
assert all(sha(R / name) == digest for name, digest in read(library / 'compiled-inputs.json').items())
rows = [c['pipeline'], *c['profiles'].values()]
if c.get('retained_Debug'):
    retained=c['retained_Debug'];proof=read(retained['failure_evidence']);assert sha(retained['failure_evidence'])==retained['failure_evidence_sha256'] and proof['recorded_original_processes_terminal'] and sha(proof['raw_archive'])==retained['failure_archive_sha256']
    assert sha(Path(retained['original_run'])/'cohort.json')==retained['original_cohort_sha256'] and not retained['tests_rerun']
    rows.append(retained['original_pipeline'])
ids = ','.join(str(row['pid']) for row in rows)
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @(" + ids + ")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed = subprocess.run(['pwsh', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert observed.returncode == 0 and not observed.stderr
live = json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
live = [live] if isinstance(live, dict) else live
same = lambda current, original: current['ProcessId'] == original['pid'] and abs((datetime.fromisoformat(current['created'].replace('Z', '+00:00')) - datetime.fromisoformat(original['creation_utc'])).total_seconds()) < .001
if args.mode == 'active':
    assert args.session is not None
    assert args.terminal_chunk is None and args.exit_code is None
    assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
    assert any(same(current, c['pipeline']) for current in live)
else:
    assert args.terminal_chunk not in [None, '', 'pending', 'unknown'] and args.exit_code == 0
    assert c['status'] == 'executed_current_two_motion_resource_tests_both_profiles'
    assert c['resource_metadata_unchanged'] and c['owned_process'] is None
    assert not any(same(current, original) for current in live for original in rows)
    assert not w['active_workloads'] or w['active_workload_sessions'] == {'resource_motion2': args.session}
    assert set(c['profiles']) == {'Debug', 'Release'}
    for profile, row in c['profiles'].items():
        assert row['exit_code'] == row['failed'] == row['ignored'] == 0 and row['passed'] == 2
        assert row['filtered_out'] == 2461 and not row.get('original_owned_handle_timeout_cleanup')
        target = run / profile
        assert sha(target / 'tests.log') == row['stdout_sha256'] and sha(target / 'stderr.log') == row['stderr_sha256']
        results = re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$', (target / 'tests.log').read_text('utf-8-sig'), re.M)
        assert len(results) == 2 and {name for name, _ in results} == names and all(result == 'ok' for _, result in results)
        diagnostic = (target / 'stderr.log').read_text('utf-8-sig')
        assert 'shared footsteps: 144 qualified families' in diagnostic and '3768 walk/attack source' in diagnostic
        retained = read(library / profile / 'product-retention.json')
        assert row['product_sha256'] == retained['product_sha256'] == sha(retained['product'])
    metadata = read(run / 'resource-metadata-before.json')
    for name, value in metadata.items():
        stat = Path(name).stat()
        assert [stat.st_size, stat.st_mtime_ns] == value
before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
out = run / ('checkpoint-' + args.mode + '-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'owned-CIM.json').write_bytes(observed.stdout)
(out / 'reader.py').write_bytes(Path(__file__).read_bytes())
qualified = args.mode == 'terminal'
record = dict(status='qualified_current_two_motion_resource_tests_both_profiles' if qualified else 'running_current_two_motion_resource_tests',
              run=str(run), source_manifest_sha256=source, original_session=args.session, terminal_tool_chunk=args.terminal_chunk,
              original_execution_kind='resumable_exec_session' if args.session is not None else 'immediate_exec_terminal',
              original_exit_code=args.exit_code, names=sorted(names), profiles=c['profiles'], qualified=qualified,
              additional_ignored_bodies_executed=2 if qualified else 0, all_ignored_bodies_executed=False,
              prior_mutually_exclusive_resource_tests=81, total_mutually_exclusive_resource_tests=83 if qualified else 81,
              remaining_ignored_bodies_unexecuted=405 if qualified else 407, full_default_library_counts_unchanged=True,
              original_identities_terminal=qualified, new_sessions=0, GUI_verified=False, audio_device_verified=False, whole_goal_complete=False)
if qualified:
    files = {path.relative_to(run).as_posix(): path for path in run.rglob('*') if path.is_file() and 'checkpoint-' not in path.relative_to(run).parts[0]}
    files.update({'reader.py': Path(__file__), 'owned-CIM.json': out / 'owned-CIM.json', 'original-plan.json': plan_path, 'test-bodies.json': Path(plan['test_bodies'])})
    pins = {name: sha(path) for name, path in files.items()}
    with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
        for name, path in files.items():
            archive.write(path, name)
    with zipfile.ZipFile(out / 'raw.zip') as archive:
        assert set(archive.namelist()) == set(pins)
        assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in pins.items())
    record.update(raw_files=pins, raw_archive_sha256=sha(out / 'raw.zip'))
    (out / 'qualified.json').write_bytes(data(record))
    record.update(qualification=str(out / 'qualified.json'), qualification_sha256=sha(out / 'qualified.json'))
(out / 'evidence.json').write_bytes(data(record))
work = ('當版兩項額外原資源library本文兩版各2過0敗，144玩家族腳步時序／PCM與十種NPC3768影格／cue／圖層均依原斷言通過；原批次及程序終止、來源／PE／原資源唯讀已核對，互斥資源合計83、餘405忽略未執行。' if qualified else '當版原資源library兩項新審查本文兩版執行中，僅沿原session續跑；來源及對應QA固定，尚未授資格。') + '完整library預設1975過0敗488忽略結果不改，GUI工具阻礙及G01–G12未完成，先自行自動測試後才交人工。'
staged = {}
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        value['current_library_motion_resource2'] = record
        value.update(active_workloads={} if qualified else {'resource_motion2': record}, active_workload_sessions={} if qualified else {'resource_motion2': args.session},
                     source_and_corresponding_QA_frozen=not qualified, next_task=work, next_focus=work,
                     last_goal_turn_evidence=str(out / 'evidence.json'), last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
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
print(json.dumps(dict(checkpoint=str(out), qualified=qualified, tests_each=2, source_manifest_sha256=source), ensure_ascii=False))
