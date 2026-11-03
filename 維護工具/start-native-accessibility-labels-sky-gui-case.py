"""Own the fixture and normal GUI processes; Windows inputs are exclusively Sky."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, socket, subprocess, time

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
P = R / 'LiTo381-Modern'
JDK = Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
def read(path):
    for attempt in range(10):
        try: return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt == 9: raise
            time.sleep(.05)
def sha(path):
    with Path(path).open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def save(path, obj): Path(path).write_text(json.dumps(obj, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')
k = ctypes.WinDLL('kernel32', use_last_error=True)
k.GetCurrentProcess.restype = wintypes.HANDLE
k.GetProcessTimes.argtypes = [wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts = [wintypes.FILETIME() for _ in range(4)]
    assert k.GetProcessTimes(handle, *[ctypes.byref(t) for t in ts])
    ticks = ts[0].dwLowDateTime | ts[0].dwHighDateTime << 32
    return (datetime(1601, 1, 1, tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()

ap = argparse.ArgumentParser()
ap.add_argument('prepared', type=Path)
ap.add_argument('--profile', required=True, choices=['Debug', 'Release'])
ap.add_argument('--automation-scope', choices=['full-visual', 'accessibility-keyboard'], default='full-visual')
ap.add_argument('--scenario', required=True, choices=['capture-probe', 'rental-entry', 'cross-endpoint', 'same-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-late', 'cancel-expiry', 'rental-hotel', 'rental-hotel-entry', 'rental-hotel-middle-entry', 'rental-hotel-share'])
a = ap.parse_args()
prepared = a.prepared.resolve(strict=True)
assert prepared.is_relative_to(V / 'native-accessibility-labels-sky-gui')
p = read(prepared / 'prepared.json')
queue = read(R / '工作佇列.json')
assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
assert queue['current_full_default_library_verified']
for name, digest in p['compiled_inputs'].items(): assert sha(R / name) == digest, name
for name, digest in p['fixture_sources'].items(): assert sha(name) == digest, name
for name, digest in p['fixture_origins'].items(): assert sha(name) == digest, name
for name, digest in p['class_files'].items(): assert sha(Path(p['classes']) / name) == digest, name
assert sha(p['jar']) == p['jar_sha256'] and sha(p['catalog']) == p['catalog_sha256']
out = V / 'native-accessibility-labels-sky-gui' / ('case-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-'+a.scenario)
out.mkdir()
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
state = dict(status='preparing_owned_server', source_manifest_sha256=p['source_manifest_sha256'],
             prepared=str(prepared), prepared_sha256=sha(prepared / 'prepared.json'), profile=a.profile, scenario=a.scenario,
             automation_scope=a.automation_scope, visual_capture_qualification_granted=False,
             launcher_sha256=sha(out / 'runner.py'),
             operator_wait_policy=p['operator_wait_policy'], performance_qualification_granted=False,
             pipeline=dict(pid=os.getpid(), creation_utc=birth(k.GetCurrentProcess())), processes=[],
             GUI_verified=False, physical_input_verified=False, whole_goal_complete=False, started_utc=datetime.now(timezone.utc).isoformat())
persist = lambda: save(out / 'cohort.json', state)
env = os.environ.copy()
env.update(TEMP=str(P / 'target/current/scratch'), TMP=str(P / 'target/current/scratch'))
for key in list(env):
    if key.startswith('LITO_'): del env[key]
children = []
probe_listener = None
def launch(command, role):
    stdout = (out / (role+'.stdout.log')).open('wb')
    stderr = (out / (role+'.stderr.log')).open('wb')
    process = subprocess.Popen(command, cwd=out, env=env, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
    row = dict(role=role, pid=process.pid, creation_utc=birth(wintypes.HANDLE(int(process._handle))), command=command, exit_code=None)
    children.append((process, row, stdout, stderr))
    state['processes'].append(row)
    persist()
    return process
persist()
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
try:
    transfer = a.scenario in ['cross-endpoint', 'same-endpoint']
    hotel = a.scenario.startswith('rental-')
    probe = a.scenario == 'capture-probe'
    harness = 'local.lito381.server.' + ('NativePeerHarness' if hotel else 'NativeTeleportHarness' if transfer else 'NativeTeleportFailureHarness')
    mode = 'gui-'+a.scenario
    command = [str(JDK / 'java.exe'), '-Dfile.encoding=UTF-8', '-Xverify:all', '-Dlito.resourceProfile=tw13081901-v1',
               '-Dlito.extendedItemDetails=true', '-Dlito.commonCatalog='+p['catalog'],
               '-cp', p['classes']+';'+p['jar'], harness, str(out), mode]
    if probe:
        probe_listener = socket.socket()
        probe_listener.bind(('127.0.0.1', 0))
        probe_listener.listen(1)
        save(out / 'ready.json', dict(port=probe_listener.getsockname()[1], jar_sha256=p['jar_sha256'],
             capture_only=True, acceptor_started=False, actual_formal_server=False))
    else:
        server = launch(command, 'server')
        deadline = time.monotonic()+8
        while not (out / 'ready.json').exists():
            assert server.poll() is None, 'Owned fixture exited before ready'
            assert time.monotonic() < deadline, 'Owned fixture ready exceeded 8 seconds'
            time.sleep(.05)
    ready = read(out / 'ready.json')
    assert ready['jar_sha256'].upper() == p['jar_sha256']
    endpoint = '127.0.0.1:'+str(ready['port'])
    for role in (['a'] if probe else ['a', 'b']):
        product = p['products'][a.profile][role]
        assert sha(product['path']) == product['sha256']
        launch([product['path'], '--native-login', endpoint, str(R.parent.parent / '0-客戶端')], role)
    policy=p['operator_wait_policy']
    assert policy['accept_seconds']==900 and policy['cleanup_seconds']==30
    assert not policy['application_protocol_deadlines_changed'] and not policy['gameplay_expiry_clocks_changed']
    fixture_budget = 120 if probe else policy['rental_overall_seconds'] if hotel else policy['transfer_failure_overall_seconds']
    state.update(status='normal_windows_running_for_Sky_inputs', ready=ready, endpoint=endpoint,
                 normal_products=p['products'][a.profile], runtime_deadline_seconds=fixture_budget, allowed_cleanup_seconds=30,
                 expected_formal_sessions=0 if probe else 4 if hotel else 6 if transfer else 10 if a.scenario=='wrong-service' else 8,
                 source_selection='測試乙', source_index=1 if hotel else 0)
    persist()
    print(json.dumps(dict(output=str(out), endpoint=endpoint, status=state['status'], normal_products=state['normal_products']), ensure_ascii=False), flush=True)
    deadline = time.monotonic()+fixture_budget+30
    while any(process.poll() is None for process, *_ in children):
        changed=False
        for process,row,*_ in children:
            if process.poll() is not None and row['exit_code'] is None:
                row.update(exit_code=process.returncode,finished_utc=datetime.now(timezone.utc).isoformat())
                changed=True
        if changed:persist()
        assert time.monotonic() < deadline, 'Owned GUI case did not close within fixture and cleanup budgets'
        time.sleep(.1)
    for process, row, *_ in children:
        row['exit_code'] = process.returncode
        row.setdefault('finished_utc',datetime.now(timezone.utc).isoformat())
        assert process.returncode == 0, row
    result = dict(memberships_empty=True, sql_access=False, authority_mutations=0, actual_client_sessions=0,
                  scope='capture-only inert owned loopback listener; no LOGIN or GAME tested') if probe else read(out / 'result.json')
    assert result['memberships_empty'] and not result['sql_access'] and result['authority_mutations'] == 0
    assert result['actual_client_sessions'] == state['expected_formal_sessions']
    state.update(status='runtime_exited_original_zero_independent_GUI_qualification_pending', server_result=result)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    if probe_listener is not None: probe_listener.close()
    for process, row, stdout, stderr in children:
        if process.poll() is None:
            # Only exact children whose original process handles are retained.
            assert birth(wintypes.HANDLE(int(process._handle))) == row['creation_utc']
            process.kill()
            process.wait()
            row['terminated_during_failure_cleanup'] = True
        row['exit_code'] = process.returncode
        stdout.close()
        stderr.close()
    state['finished_utc'] = datetime.now(timezone.utc).isoformat()
    state['recorded_owned_processes_terminal'] = all(x[0].poll() is not None for x in children)
    persist()
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
