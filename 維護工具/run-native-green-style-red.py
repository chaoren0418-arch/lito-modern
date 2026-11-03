"""Preserve the real source1020 red test before changing runtime clock semantics."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import ctypes, hashlib, json, os, re, subprocess, zipfile
from ctypes import wintypes

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
NORMAL = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
digest = lambda b: hashlib.sha256(b).hexdigest().upper()
sha = lambda p: digest(Path(p).read_bytes())
encode = lambda v: json.dumps(v, ensure_ascii=False, indent=2) + '\n'
old = read(NORMAL / 'source-inputs.json')
assert len(old) == 1019
for name, h in old.items():
    assert sha(ROOT / name) == h, name
pins = old | {'LiTo381-Modern/tests/native_green_clock.rs': sha(PROJECT / 'tests/native_green_clock.rs')}
assert len(pins) == 1020
out = ROOT / '驗證紀錄/current/native-actor-green-clock' / ('red-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True, exist_ok=False)
(out / 'source-inputs.json').write_text(encode(pins), encoding='utf-8', newline='\n')
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out / 'compiled-source.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, h in pins.items():
        assert sha(ROOT / name) == h
        z.write(ROOT / name, name)
with zipfile.ZipFile(out / 'compiled-source.zip') as z:
    assert set(z.namelist()) == set(pins)
    for name, h in pins.items():
        assert digest(z.read(name)) == h
normal = read(NORMAL / 'qualified.json')
mains = lambda: {p: sha(v['path']) for p, v in normal['profiles'].items()}
receipt = dict(status='compiling_red_native_counter_contract', source_files=1020,
    normal_mains_before=mains(), output=str(out), owned_process=None,
    compiled_source_sha256=sha(out / 'compiled-source.zip'), whole_goal_complete=False)
save = lambda: (out / 'receipt.json').write_text(encode(receipt), encoding='utf-8', newline='\n')
save()
env = os.environ.copy()
env.update(RUSTC=str(RUST / 'rustc.exe'), RUSTDOC=str(RUST / 'rustdoc.exe'),
    TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'),
    PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8')
env['PATH'] = str(RUST) + os.pathsep + env['PATH']
def execute(args, name):
    with (out / (name + '.stdout.log')).open('wb') as stdout, (out / (name + '.stderr.log')).open('wb') as stderr:
        child = subprocess.Popen(args, cwd=PROJECT, env=env, stdout=stdout, stderr=stderr,
            creationflags=subprocess.CREATE_NO_WINDOW)
        times = [wintypes.FILETIME() for _ in range(4)]
        get = ctypes.WinDLL('kernel32', use_last_error=True).GetProcessTimes
        get.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        get.restype = wintypes.BOOL
        assert get(wintypes.HANDLE(int(child._handle)), *[ctypes.byref(v) for v in times])
        ticks = (times[0].dwHighDateTime << 32) | times[0].dwLowDateTime
        created = datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)
        receipt['owned_process'] = dict(pid=child.pid, creation_utc=created.isoformat(), args=args)
        save()
        code = child.wait()
        receipt['owned_process'] = None
        save()
    return code
try:
    command = [str(RUST / 'cargo.exe'), 'test', '--no-run', '--locked', '--offline', '-j1',
        '--target', 'x86_64-pc-windows-msvc', '--target-dir', 'target/current',
        '--message-format=json', '--test', 'native_green_clock']
    receipt['build_arguments'] = command
    print(encode(dict(output=str(out), expected='one runtime assertion failure, not a compilation error')), flush=True)
    receipt['compile_exit_code'] = execute(command, 'build')
    save()
    assert receipt['compile_exit_code'] == 0
    messages = [json.loads(line) for line in (out / 'build.stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
    products = [x for x in messages if x.get('reason') == 'compiler-artifact' and x.get('executable')
        and x.get('profile', {}).get('test') and x['target']['name'] == 'native_green_clock']
    assert len(products) == 1 and products[0]['features'] == []
    exe = Path(products[0]['executable']).resolve(strict=True)
    assert exe.is_relative_to(PROJECT / 'target/current')
    data = exe.read_bytes()
    assert data[:2] == b'MZ'
    offset = int.from_bytes(data[60:64], 'little')
    assert data[offset:offset + 4] == b'PE\0\0' and int.from_bytes(data[offset + 4:offset + 6], 'little') == 0x8664 and int.from_bytes(data[offset + 24:offset + 26], 'little') == 0x20b
    receipt.update(status='executing_red_native_counter_contract', test_executable=str(exe), test_sha256=digest(data))
    with zipfile.ZipFile(out / 'test-product.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        z.writestr('native_green_clock.exe', data)
    with zipfile.ZipFile(out / 'test-product.zip') as z:
        assert digest(z.read('native_green_clock.exe')) == receipt['test_sha256']
    receipt['test_exit_code'] = execute([str(exe), '--test-threads=1'], 'runtime')
    text = (out / 'runtime.stdout.log').read_text(encoding='utf-8-sig')
    assert receipt['test_exit_code'] == 101 and 'Green(14)' in text and 'Green(13)' in text
    assert re.search(r'test result: FAILED\. 0 passed; 1 failed; 0 ignored;', text)
    for name, h in pins.items():
        assert sha(ROOT / name) == h, name
    receipt.update(status='observed_current_elapsed_style_fails_native_counter_contract',
        passed=0, failed=1, ignored=0, sources_unchanged=True,
        normal_mains_after=mains(), test_product_archive_sha256=sha(out / 'test-product.zip'))
except BaseException as error:
    receipt.update(status='failed_preserved', error=repr(error))
    raise
finally:
    receipt['finished_utc'] = datetime.now(timezone.utc).isoformat()
    save()
    files = {p.name: p for p in out.iterdir() if p.is_file() and p.suffix.lower() != '.zip'}
    hashes = {name: sha(p) for name, p in files.items()}
    with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
        for name, p in files.items():
            z.write(p, name)
    with zipfile.ZipFile(out / 'raw.zip') as z:
        assert set(z.namelist()) == set(hashes)
        for name, h in hashes.items():
            assert digest(z.read(name)) == h
    (out / 'archive-readback.json').write_text(encode(dict(files=hashes,
        raw_archive_sha256=sha(out / 'raw.zip'), verified=True)), encoding='utf-8', newline='\n')
print(encode(dict(status=receipt['status'], output=str(out), test_exit_code=receipt['test_exit_code'])), flush=True)
