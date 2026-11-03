"""Stage all eight continuation files around an owned normal-window GUI case."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, hashlib, json, subprocess, time, zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
def read(path):
    for attempt in range(10):
        try: return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt == 9: raise
            time.sleep(.05)
def sha(path):
    with Path(path).open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def data(obj): return (json.dumps(obj, ensure_ascii=False, indent=2)+'\n').encode('utf-8')

ap = argparse.ArgumentParser()
ap.add_argument('case', type=Path)
ap.add_argument('--session', required=True, type=int)
ap.add_argument('--mode', required=True, choices=['active', 'terminal'])
ap.add_argument('--terminal-chunk')
ap.add_argument('--terminal-exit-code', type=int)
a = ap.parse_args()
case = a.case.resolve(strict=True)
assert case.is_relative_to(V / 'native-rental-service-entry-return-sky-gui') and a.session > 0
c = read(case / 'cohort.json')
p = read(Path(c['prepared']) / 'prepared.json')
assert sha(Path(c['prepared']) / 'prepared.json') == c['prepared_sha256']
assert c['source_manifest_sha256'] == p['source_manifest_sha256'] == 'BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
for name, digest in p['compiled_inputs'].items(): assert sha(R / name) == digest, name
for product in p['products'][c['profile']].values(): assert sha(product['path']) == sha(product['original_path']) == product['sha256']
assert sha(p['jar']) == p['jar_sha256'] and sha(p['catalog']) == p['catalog_sha256']
focus_only = c['scenario'] == 'focus-regression-only'
expected_children = 2 if focus_only else 1 if c['scenario'] == 'capture-probe' else 3
assert len(c['processes']) == expected_children
rows = [c['pipeline']] + c['processes']
ids = ','.join(str(row['pid']) for row in rows)
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,Name,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
result = subprocess.run(['pwsh', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert result.returncode == 0 and not result.stderr
actual = json.loads(result.stdout.decode('utf-8-sig')) if result.stdout.strip() else []
if isinstance(actual, dict): actual = [actual]
live = [row for row in rows if any(x['ProcessId'] == row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z', '+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds()) < .001 for x in actual)]
queue = read(R / '工作佇列.json')
if focus_only:
    assert p['scope'] == 'normal_focus_regression_only'
    normal = read(p['normal_qualification'])
    assert sha(p['normal_qualification']) == p['normal_qualification_sha256']
    assert normal['status'] == 'independently_qualified_keyboard_focus_normal_related_both_profiles'
    assert normal['source_manifest_sha256'] == c['source_manifest_sha256'] and normal['passed_per_profile'] == 16
    assert normal['observed_terminal_exit_code'] == 0 and normal['recorded_owned_processes_all_terminal']
else:
    assert queue['current_full_default_library_verified']
if a.mode == 'active':
    assert c['status'] == 'normal_windows_running_for_Sky_inputs' and len(live) == expected_children+1
    assert not queue['active_workloads'] and a.terminal_chunk is None and a.terminal_exit_code is None
else:
    assert a.terminal_chunk not in [None, '', 'pending', 'unknown'] and a.terminal_exit_code is not None
    assert c['recorded_owned_processes_terminal'] and not live
    assert queue['active_workload_sessions'] == {'normal_gui': a.session}
    assert queue['current_native_catalog_gui_active']['case'] == str(case)
    if a.terminal_exit_code == 0:
        assert c['status'] == 'runtime_exited_original_zero_independent_GUI_qualification_pending'
        assert all(row['exit_code'] == 0 and not row.get('terminated_during_failure_cleanup') for row in c['processes'])
    else:
        assert c['status'] == 'failed_preserved'
before = read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
out = case / ('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
record = dict(status='normal_GUI_case_running' if a.mode == 'active' else 'normal_GUI_original_runtime_terminal_independent_qualification_pending' if a.terminal_exit_code == 0 else 'normal_GUI_original_failure_preserved',
    case=str(case), original_session_id=a.session, source_manifest_sha256=c['source_manifest_sha256'],
    profile=c['profile'], scenario=c['scenario'], pipeline=c['pipeline'], owned_processes=c['processes'],
    automation_scope=c.get('automation_scope', 'full-visual'), visual_capture_qualification_granted=False,
    process_observation=actual, recorded_original_identities=len(rows), original_identities_terminal=a.mode == 'terminal',
    terminal_tool_chunk=a.terminal_chunk, observed_actual_exit_code=a.terminal_exit_code,
    GUI_verified=False, physical_input_verified=False, whole_goal_complete=False)
work = ('正常GUI原'+str(a.session)+'：'+c['profile']+'／'+c['scenario']+
        ('沿同一隔離批次以Sky操作，正常來源與對應QA固定；待實際輸入／封包／角色狀態／畫面及原始終止核對。' if a.mode == 'active' else
         '已實際終止，所有記錄程序身分均終止；原始exit '+str(a.terminal_exit_code)+'，保留原始結果，尚待獨立GUI範圍資格。')+
        '當版兩版廣域與完整library預設資格保留，忽略本文未執行。GUI全流程、IME／多DPI／音效裝置及其餘G01–G12未完成；先自行測試確認才交人工，不請使用者登入，相鄰專案只讀。')
staged = {}
date = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        assert value['current_product_identity']['source_manifest_sha256'] == c['source_manifest_sha256']
        value['current_native_catalog_gui_active'] = record
        value.update(active_workloads={'normal_gui':record} if a.mode == 'active' else {},
                     active_workload_sessions={'normal_gui':a.session} if a.mode == 'active' else {},
                     source_and_corresponding_QA_frozen=a.mode == 'active', source_freeze_while_Cargo=False,
                     next_task=work, next_focus=work, last_goal_turn_classification='progress',
                     last_goal_turn_evidence=str(out / 'evidence.json'), goal_complete=False, whole_goal_complete=False)
        value['current_source_goal_validation'].setdefault('G08-player-gui', {}).update(complete=False, current_case=record)
        for item in value.get('items', []):
            if item.get('id') == 'G08-player-gui':
                item.update(state='current_normal_GUI_case_running' if a.mode == 'active' else 'current_normal_GUI_case_terminal_scope_qualification_pending',
                            complete=False, current_source_manifest_sha256=c['source_manifest_sha256'], current_evidence=record)
        value['current_focus'] = dict(item='normal_gui_full_flow_automation', work=work, applies_to_current_source=True, goal_complete=False)
        if name.endswith('next-validation-action.json'): value['next_action'] = work
        staged[name] = data(value)
    else:
        first, separator, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert separator
        datetime.strptime(first[:10], '%Y-%m-%d')
        staged[name] = (date+'：'+work+'\n\n'+rest).encode('utf-8')
assert set(staged) == set(before)
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for name in before: z.write(R / name, name)
with zipfile.ZipFile(out / 'before.zip') as z:
    assert all(hashlib.sha256(z.read(name)).hexdigest().upper() == digest for name, digest in before.items())
(out / 'evidence.json').write_bytes(data(record))
(out / 'checkpoint.py').write_bytes(Path(__file__).read_bytes())
(out / 'owned-CIM.json').write_bytes(result.stdout)
assert all(sha(R / name) == digest for name, digest in before.items())
for name, payload in staged.items(): (R / name).write_bytes(payload)
after = {name: hashlib.sha256(payload).hexdigest().upper() for name, payload in staged.items()}
assert all(sha(R / name) == digest for name, digest in after.items())
(out / 'readback.json').write_bytes(data(dict(before_files=before, after_files=after,
    before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json'))))
(case.parent / 'current-continuation.json').write_bytes(data(dict(**record,
    latest_checkpoint=str(out), latest_readback=str(out / 'readback.json'),
    must_poll_original_session_before_restarting=a.mode == 'active')))
print(json.dumps(dict(checkpoint=str(out), case_status=record['status'], original_session_id=a.session), ensure_ascii=False))
