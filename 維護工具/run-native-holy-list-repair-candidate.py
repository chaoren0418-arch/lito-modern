"""Check the one-list test correction against the unchanged current library."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, re, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT/'LiTo381-Modern'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/default-native-regression')
failure_dir = run/'failure-case-native_holy_weapon'
failure = read(failure_dir/'readback.json')
assert failure['original_Rust_exit_code'] == 101 and failure['recorded_case_processes_all_terminal']
assert sha(failure_dir/'raw.zip') == failure['raw_archive_sha256']
with zipfile.ZipFile(failure_dir/'raw.zip') as z:
    assert set(z.namelist()) == set(failure['raw_files'])
    for name, digest in failure['raw_files'].items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
pins = read(run/'compiled-inputs.json')
assert len(pins) == 1025
source = PROJECT/'tests/native_holy_weapon.rs'
before = source.read_bytes()
old = b'[221,744,745,2510,829,830,831,761,2165,871,754]'
new = b'[221,744,745,749,2510,829,830,831,761,2165,871,754]'
assert before.count(old) == 1 and old not in new
after = before.replace(old, new)
assert after.replace(new, old) == before
candidate = ROOT/'維護工具/candidates/native-holy-qualification-list-repair'
candidate.mkdir()
probe = candidate/'native_holy_weapon.rs'
probe.write_bytes(after)
(candidate/'fixtures').mkdir()
fixture = PROJECT/'tests/fixtures/native-world-current.json'
(candidate/'fixtures/native-world-current.json').write_bytes(fixture.read_bytes())
stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out = run/('holy-list-candidate-'+stamp)
out.mkdir()
runtime = PROJECT/'target/current/native-holy-list-candidate'/stamp
runtime.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
(out/'before-test.rs').write_bytes(before)
(out/'candidate-test.rs').write_bytes(after)
libraries, native = {}, set()
for line in (run/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):
        continue
    m = json.loads(line)
    if m.get('reason') == 'build-script-executed':
        native.update(m.get('linked_paths', []))
    if m.get('reason') == 'compiler-artifact' and m['target']['name'] in ['lito_modern', 'serde_json']:
        paths = [p for p in m['filenames'] if p.endswith('.rlib')]
        if paths:
            libraries[m['target']['name']] = paths[0]
assert set(libraries) == {'lito_modern', 'serde_json'}
state = dict(status='preparing_isolated_holy_list_candidate', source_manifest_sha256=failure['source_manifest_sha256'],
    original_failure=str(failure_dir/'readback.json'), original_failure_archive_sha256=failure['raw_archive_sha256'],
    original_Rust_exit_code=101, original_test_source_sha256=sha(source), candidate_source=str(probe),
    candidate_source_sha256=sha(probe), fixture_sha256=sha(fixture),
    correction='Only add verified detection effect749 to the exact aggregate list; preserve every holy-specific assertion.',
    libraries={n:dict(path=p, sha256=sha(p)) for n,p in libraries.items()},
    product_source_changed=False, whole_broad_batch_pass_inferred=False, process_history=[], owned_process=None,
    GUI_verified=False, whole_goal_complete=False)
save(candidate/'candidate.json', {k:v for k,v in state.items() if k not in ['process_history', 'owned_process']})

def stable():
    assert all(sha(ROOT/n) == h for n,h in pins.items())
    assert sha(source) == state['original_test_source_sha256'] and sha(probe) == state['candidate_source_sha256']
    assert sha(candidate/'fixtures/native-world-current.json') == state['fixture_sha256']
    assert all(sha(info['path']) == info['sha256'] for info in state['libraries'].values())

env = dict(os.environ, CARGO_MANIFEST_DIR=str(PROJECT), PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8',
    TEMP=str(PROJECT/'target/current/scratch'), TMP=str(PROJECT/'target/current/scratch'))

def execute(args, folder):
    stable()
    folder.mkdir()
    with (folder/'stdout.log').open('wb') as stdout, (folder/'stderr.log').open('wb') as stderr:
        p = subprocess.Popen(args, cwd=PROJECT, env=env, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
        times = [wintypes.FILETIME() for _ in range(4)]
        k = ctypes.WinDLL('kernel32', use_last_error=True)
        k.GetProcessTimes.argtypes = [wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
        k.GetProcessTimes.restype = wintypes.BOOL
        assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)), *[ctypes.byref(t) for t in times])
        ticks = times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        item = dict(pid=p.pid, creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(), arguments=args)
        state['process_history'].append(item)
        state['owned_process'] = item
        save(out/'receipt.json', state)
        code = p.wait()
        item['exit_code'] = code
        state['owned_process'] = None
        save(out/'receipt.json', state)
    stable()
    return code

exe = runtime/'native-holy-list-candidate.exe'
args = [str(RUST/'rustc.exe'), '--test', str(probe), '--edition=2021', '--crate-name', 'native_holy_list_candidate',
    '--target', 'x86_64-pc-windows-msvc', '-L', 'dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),
    '-L', 'dependency='+str(PROJECT/'target/current/debug/deps'), '-o', str(exe)]
for name,p in libraries.items():
    args += ['--extern', name+'='+p]
for p in sorted(native):
    args += ['-L', p]
save(out/'receipt.json', state)
print(json.dumps(dict(output=str(out)), ensure_ascii=False), flush=True)
try:
    state['status'] = 'compiling_isolated_holy_list_candidate'
    assert execute(args, out/'build') == 0, 'Preserve candidate compile failure before repair'
    state.update(executable=str(exe), executable_sha256=sha(exe), status='executing_isolated_holy_list_candidate')
    assert execute([str(exe), '--list'], out/'inventory') == 0
    names = re.findall(r'^(.+): test$', (out/'inventory/stdout.log').read_text(encoding='utf-8-sig'), re.M)
    assert len(names) == len(set(names)) == 3
    assert execute([str(exe), '--nocapture', '--test-threads=1'], out/'runtime') == 0
    text = (out/'runtime/stdout.log').read_text(encoding='utf-8-sig')
    assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', text) == [('3', '0', '0')]
    assert set(re.findall(r'^test (.+?) \.\.\. ok$', text, re.M)) == set(names)
    assert not (out/'runtime/stderr.log').read_bytes().strip()
    state.update(status='passed_isolated_holy_list_candidate_on_unchanged_library', passed=3, failed=0, ignored=0)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    save(out/'receipt.json', state)
files = {p.relative_to(out).as_posix():p for p in out.rglob('*') if p.is_file()}
files.update({'candidate.exe':exe, 'fixture.json':fixture, 'candidate-metadata.json':candidate/'candidate.json'})
hashes = {name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(out/'raw.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name,p in files.items():
        z.write(p, name)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist()) == set(hashes)
    for name, digest in hashes.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
save(out/'readback.json', dict(status=state['status'], source_manifest_sha256=state['source_manifest_sha256'],
    candidate_source_sha256=state['candidate_source_sha256'], original_failure_archive_sha256=failure['raw_archive_sha256'],
    actual_Rust_exit_code=0, passed=3, failed=0, ignored=0, product_source_changed=False,
    whole_broad_batch_pass_inferred=False, raw_archive_sha256=sha(out/'raw.zip'), raw_files=hashes, whole_goal_complete=False))
print(json.dumps(dict(status=state['status'], passed=3, failed=0, ignored=0, product_source_changed=False), ensure_ascii=False))
