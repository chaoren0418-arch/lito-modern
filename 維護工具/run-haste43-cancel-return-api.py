"""Build the opt-in target and run original formal75 memory fixtures serially.

Requires current normal19 and focused return10 qualifications. This
runner grants no qualification and never modifies game authority or clocks.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, re, subprocess, time, zipfile
R = Path(__file__).resolve().parents[1]
P = R / 'LiTo381-Modern'
V = R / '驗證紀錄/current'
T = R / '維護工具'
SOURCE = 'E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
RUST = Path(os.environ['USERPROFILE']) / '.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
JDK = Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
TEST = 'rental_entry_rejects_ordinary_channel_before_wire_and_remains_usable_after_return'

def read(path):
    for attempt in range(10):
        try: return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt == 9: raise
            time.sleep(.05)

def sha(path):
    with Path(path).open('rb') as stream: return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

def save(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

now = lambda: datetime.now(timezone.utc).isoformat()
parser = argparse.ArgumentParser()
parser.add_argument('normal',type=Path)
args=parser.parse_args();base=args.normal.resolve(strict=True)
assert base.is_relative_to(V/'native-installed-resource-profile-normal')
w=read(R/'工作佇列.json');assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
assert w['current_product_identity']['source_manifest_sha256']==SOURCE
normal_path=base/'qualified/qualified.json';normal=read(normal_path)
assert normal['source_manifest_sha256']==SOURCE and normal['status']=='independently_qualified_installed_resource_profile_normal_related_both_profiles'
assert normal['passed_per_profile']==19 and normal['observed_terminal_exit_code']==0 and normal['recorded_owned_processes_all_terminal']
assert w['current_native_installed_resource_profile_normal_qualification']['run']==str(base)
focused=w['current_native_rental_return_reconnect_focused'];assert focused['source_manifest_sha256']==SOURCE and focused['qualified']
assert sha(focused['qualification'])==focused['qualification_sha256']and focused['profiles']['Debug']['passed']==11
from haste43_cancel_runtime_gate import require_idle_runtime
require_idle_runtime(R)
cohort=read(base/'cohort.json');inputs=read(base/'compiled-inputs.json')
assert len(inputs)==1049 and len(read(base/'source-inputs.json'))==1048
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';formal=R.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin';oracle=P/'tests/oracle/NativePeerHarness.java'
assert sha(jar)==sha(formal)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
qa = {name: sha(T / name) for name in ['run-haste43-cancel-return-api.py',
    'qualify-haste43-cancel-return-api.py', 'checkpoint-haste43-cancel-return-api.py','haste43_cancel_runtime_gate.py']}
assert not {name: value for name, value in os.environ.items() if value and
    (name.startswith('CARGO_PROFILE_') or name in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'])}
stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out = V / 'native-rental-return-reconnect-api' / stamp
out.mkdir(parents=True)
work = P / 'target/current/native-rental-return-reconnect-api' / stamp
work.mkdir(parents=True)
classes = work / 'classes'
classes.mkdir()
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
(out / 'source-inputs.json').write_bytes((base / 'source-inputs.json').read_bytes())
(out / 'compiled-inputs.json').write_bytes((base / 'compiled-inputs.json').read_bytes())
oracle_text=oracle.read_text('utf-8-sig')
peer_before='int peerCount=profile.equals("siege-native")?30:profile.equals("lightning-multi")?3:2;'
peer_after='int peerCount=profile.equals("rental-return")?4:profile.equals("siege-native")?30:profile.equals("lightning-multi")?3:2;'
assert oracle_text.count(peer_before)==1
(out/'NativePeerHarness.java').write_text(oracle_text.replace(peer_before,peer_after),encoding='utf-8',newline='\n')
k = ctypes.WinDLL('kernel32', use_last_error=True)
k.GetCurrentProcess.restype = wintypes.HANDLE
k.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4

def birth(handle):
    values = [wintypes.FILETIME() for _ in range(4)]
    assert k.GetProcessTimes(handle, *[ctypes.byref(value) for value in values])
    ticks = values[0].dwLowDateTime | values[0].dwHighDateTime << 32
    return (datetime(1601,1,1,tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)).isoformat()

state = dict(status='prepared_current_rental_entry_guard', source_manifest_sha256=SOURCE,
    source_files=1048, compiled_input_files=1049, normal_run=str(base), focused_qualification=focused['qualification'],focused_qualification_sha256=focused['qualification_sha256'],
    normal_qualification=str(normal_path),normal_qualification_sha256=sha(normal_path),
    expected_sessions_per_profile=8,full_library_in_this_scope=False,derived_oracle_sha256=sha(out/'NativePeerHarness.java'),fixture_derivation=dict(before=peer_before,after=peer_after,counts_only=True,gameplay_clocks_changed=False,authority_or_constructor_changes=False),
    runtime=str(work), jar=str(jar), jar_sha256=sha(jar), catalog=str(catalog), catalog_sha256=sha(catalog),
    oracle=str(oracle), oracle_sha256=sha(oracle), validation_pins=qa,
    runner=dict(pid=os.getpid(), creation_utc=birth(k.GetCurrentProcess())), profiles={},
    process_history=[], owned_processes=[], actual_sessions=0, SQL_access=False, authority_mutations=False,
    gameplay_clocks_changed=False, GUI_verified=False, qualification_granted=False, whole_goal_complete=False,
    started_utc=now())
env = {name: value for name, value in os.environ.items() if not name.startswith('LITO_')}
env.update(TEMP=str(P / 'target/current/scratch'), TMP=str(P / 'target/current/scratch'),
    RUSTC=str(RUST / 'rustc.exe'), RUSTDOC=str(RUST / 'rustdoc.exe'), PYTHONDONTWRITEBYTECODE='1')
env['PATH'] = str(RUST) + os.pathsep + env['PATH']
children = []

def guard():
    assert all(sha(R / name) == digest for name, digest in inputs.items())
    assert all(sha(T / name) == digest for name, digest in qa.items())
    assert sha(jar) == sha(formal) == state['jar_sha256'] and sha(catalog) == state['catalog_sha256']
    assert sha(oracle) == state['oracle_sha256']
    assert sha(normal_path)==state['normal_qualification_sha256']
    assert sha(focused['qualification'])==state['focused_qualification_sha256']
    assert sha(out/'NativePeerHarness.java')==state['derived_oracle_sha256']
    for profile, info in normal['profiles'].items():
        assert sha(info['normal']['path']) == info['normal']['sha256']

def refresh():
    for process, row, stdout, stderr in children:
        if process.poll() is not None and row['exit_code'] is None:
            row.update(exit_code=process.returncode, finished_utc=now())
            stdout.close(); stderr.close()
    state['owned_processes'] = [row for process, row, _, _ in children if process.poll() is None]
    save(out / 'cohort.json', state)

def launch(command, label, folder, cwd, child_env=None):
    folder.mkdir(exist_ok=True)
    stdout = (folder / (label + '.stdout')).open('wb')
    stderr = (folder / (label + '.stderr')).open('wb')
    process = subprocess.Popen(command, cwd=cwd, env=child_env or env,
        stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
    row = dict(pid=process.pid, creation_utc=birth(wintypes.HANDLE(int(process._handle))),
        label=label, command=command, output=str(folder), exit_code=None)
    state['process_history'].append(row)
    children.append((process,row,stdout,stderr)); refresh()
    return process

guard(); refresh(); print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
try:
    # Finish both compilations before starting either timed fixture.
    for profile in ['Debug','Release']:
        guard(); state.update(status='compiling_current_rental_entry_guard', active_profile=profile); refresh()
        d = out / profile; d.mkdir(); state['profiles'][profile] = dict(directory=str(d))
        command = [str(RUST / 'cargo.exe'),'test','--no-run','--locked','--offline','-j1',
            '--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json',
            '--test','native_rental_entry_guard'] + (['--release'] if profile == 'Release' else [])
        compiler = launch(command, 'cargo', d, P)
        assert compiler.wait() == 0, 'Preserve original target compilation failure'
        refresh(); guard()
        messages = [json.loads(line) for line in (d / 'cargo.stdout').read_text('utf-8-sig').splitlines() if line.startswith('{')]
        artifacts = [row for row in messages if row.get('reason') == 'compiler-artifact' and row.get('executable') and row['profile']['test']]
        assert len(artifacts) == 1 and artifacts[0]['target']['name'] == 'native_rental_entry_guard' and artifacts[0]['features'] == []
        product = Path(artifacts[0]['executable']).resolve(strict=True)
        assert product.is_relative_to(P / 'target/current')
        run = work / profile; run.mkdir(); immutable = run / 'rental-entry-guard.exe'
        immutable.write_bytes(product.read_bytes()); assert sha(immutable) == sha(product)
        state['profiles'][profile].update(runtime=str(run), product=str(immutable), product_sha256=sha(immutable),
            original_test_product=str(product), compiler_profile=artifacts[0]['profile'])
        with zipfile.ZipFile(d / 'test-product.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as archive:
            archive.write(immutable, 'native_rental_entry_guard.exe')
        state['profiles'][profile]['test_product_archive_sha256'] = sha(d / 'test-product.zip'); refresh()
    compiler = launch([str(JDK / 'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8',
        '-cp',str(jar),'-d',str(classes),str(out / 'NativePeerHarness.java')], 'javac', out, work)
    assert compiler.wait(timeout=60) == 0; refresh()
    state['class_files'] = {str(path.relative_to(classes)):sha(path) for path in classes.rglob('*.class')}
    assert state['class_files']; refresh()
    for profile in ['Debug','Release']:
        guard(); d = out / profile; row = state['profiles'][profile]; run = Path(row['runtime'])
        state.update(status='executing_current_rental_entry_guard', active_profile=profile); refresh()
        server = launch([str(JDK / 'java.exe'),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Xverify:all',
            '-Djava.io.tmpdir=' + env['TMP'],'-Dlito.resourceProfile=tw13081901-v1',
            '-Dlito.extendedItemDetails=true','-Dlito.commonCatalog=' + str(catalog),
            '-cp',str(classes) + os.pathsep + str(jar),'local.lito381.server.NativePeerHarness',str(run),'rental-return'],
            'server', d, run)
        deadline = time.monotonic() + 8
        while not (run / 'ready.json').exists():
            refresh(); assert server.poll() is None and time.monotonic() < deadline, 'Original server 8s readiness bound'
            time.sleep(.02)
        assert read(run / 'ready.json')['jar_sha256'].upper() == state['jar_sha256']
        client_env = dict(env, LITO_RENTAL_ENTRY_GUARD_ROOT=str(run))
        client = launch([row['product'],TEST,'--exact','--ignored','--nocapture','--test-threads=1'],
            'client', d, P, client_env)
        deadline = time.monotonic() + 120
        while client.poll() is None:
            refresh(); assert server.poll() in [None,0] and time.monotonic() < deadline, 'Original 120s outer fixture bound'
            time.sleep(.1)
        refresh(); assert client.returncode == 0, 'Preserve actual guard/return failure'
        assert server.wait(timeout=8) == 0; refresh(); guard()
        result = read(run / 'result.json')
        assert result['actual_client_sessions'] == 8 and result['game_sessions'] == 4 and result['memberships_empty']
        assert not result['sql_access'] and not result['state_persistent'] and result['authority_mutations'] == 0
        totals = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',
            (d / 'client.stdout').read_text('utf-8-sig'))
        assert totals == [('1','0','0','0','0')]
        for role in ['a','b']: assert (run / (role + '-rental-entry-guard.json')).is_file()
        row.update(actual_sessions=8, passed=1, failed=0, ignored=0, result_sha256=sha(run / 'result.json'))
        state['actual_sessions'] += 8; refresh()
        print(json.dumps(dict(profile=profile, passed=1, actual_sessions=8, independent_readback_pending=True)), flush=True)
    state.update(status='executed_both_current_rental_entry_guard_profiles_pending_independent_readback', active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error)); raise
finally:
    for process, row, _, _ in reversed(children):
        if process.poll() is None:
            assert birth(wintypes.HANDLE(int(process._handle))) == row['creation_utc']
            row['cleanup_stopped_original_owned_handle'] = True
            process.terminate(); process.wait(timeout=3)
    refresh(); guard(); state['finished_utc'] = now(); refresh()
print(json.dumps(dict(output=str(out), status=state['status'], qualification_granted=False), ensure_ascii=False), flush=True)
