"""Run the source1021 shared green/water/controller/render binding scope."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, ctypes, hashlib, json, os, re, subprocess, zipfile
from ctypes import wintypes

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
NORMAL = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
TARGETS = ['native_green_clock','native_water_clock','native_actor_states','native_environment','native_render_environment']
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
digest = lambda b: hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()
encode = lambda v: json.dumps(v, ensure_ascii=False, indent=2) + '\n'
ap = argparse.ArgumentParser()
ap.add_argument('--profile', choices=['Debug','Release'], required=True)
ap.add_argument('--target',choices=TARGETS,action='append')
a = ap.parse_args()
if a.target:
    TARGETS=list(dict.fromkeys(a.target))
names = set(read(NORMAL / 'source-inputs.json')) | {
    'LiTo381-Modern/tests/native_green_clock.rs',
    'LiTo381-Modern/tests/fixtures/native-green-clock-original.json'}
pins = {name: sha(ROOT / name) for name in sorted(names)}
assert len(pins) == 1021
out = ROOT / '驗證紀錄/current/native-actor-green-clock' / ('scoped-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f') + '-' + a.profile)
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
receipt = dict(status='compiling_source1021_green_scope', profile=a.profile, source_files=1021,
    targets=TARGETS, output=str(out), owned_process=None, results={},
    retained_source1019_normal_mains_before=mains(), compiled_source_sha256=sha(out / 'compiled-source.zip'),
    current_normal_mains_qualified=False, current_GUI_qualified=False, whole_goal_complete=False)
save = lambda: (out / 'receipt.json').write_text(encode(receipt), encoding='utf-8', newline='\n')
save()
env = os.environ.copy()
env.update(RUSTC=str(RUST / 'rustc.exe'), RUSTDOC=str(RUST / 'rustdoc.exe'),
    TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'),
    PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8')
env['PATH'] = str(RUST) + os.pathsep + env['PATH']
def execute(args, label):
    with (out / (label + '.stdout.log')).open('wb') as stdout, (out / (label + '.stderr.log')).open('wb') as stderr:
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
    command = [str(RUST / 'cargo.exe'),'test','--no-run','--locked','--offline','-j1',
        '--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json']
    if a.profile == 'Release':
        command.append('--release')
    for name in TARGETS:
        command += ['--test', name]
    receipt['build_arguments'] = command
    print(encode(dict(output=str(out), profile=a.profile, source_files=1021)), flush=True)
    receipt['compile_exit_code'] = execute(command, 'build')
    save()
    assert receipt['compile_exit_code'] == 0, 'Retain compilation failure.'
    messages = [json.loads(line) for line in (out / 'build.stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
    products = {}
    for m in messages:
        if m.get('reason') != 'compiler-artifact' or not m.get('executable'):
            continue
        name = m['target']['name']
        if not m.get('profile',{}).get('test'):
            continue
        assert name in TARGETS and name not in products and m['features'] == []
        p = Path(m['executable']).resolve(strict=True)
        assert p.is_relative_to(PROJECT / 'target/current')
        with p.open('rb') as f:
            h = f.read(64)
            assert h[:2] == b'MZ'
            f.seek(int.from_bytes(h[60:64], 'little'))
            pe = f.read(26)
        assert pe[:4] == b'PE\0\0' and int.from_bytes(pe[4:6],'little') == 0x8664 and int.from_bytes(pe[24:26],'little') == 0x20b
        products[name] = dict(path=str(p),sha256=sha(p))
    assert set(products) == set(TARGETS)
    (out / 'products.json').write_text(encode(products), encoding='utf-8', newline='\n')
    with zipfile.ZipFile(out / 'test-products.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        for name, item in products.items():
            z.write(item['path'],name+'.exe')
    with zipfile.ZipFile(out / 'test-products.zip') as z:
        for name, item in products.items():
            assert digest(z.read(name+'.exe')) == item['sha256']
    for name in TARGETS:
        receipt.update(status='running_source1021_green_scope',active_target=name)
        save()
        item = products[name]
        assert sha(item['path']) == item['sha256']
        inventory_code = execute([item['path'],'--list'],name+'-inventory')
        assert inventory_code == 0
        inventory = re.findall(r'^(.+): test$',(out/(name+'-inventory.stdout.log')).read_text(encoding='utf-8-sig'),re.M)
        code = execute([item['path'],'--test-threads=1'],name+'-runtime')
        log = out / (name+'-runtime.stdout.log')
        found = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log.read_text(encoding='utf-8-sig'))
        assert len(found) == 1
        terminal, passed, failed, ignored, measured, filtered = found[0]
        passed, failed, ignored, measured, filtered = map(int,[passed,failed,ignored,measured,filtered])
        assert filtered == measured == 0 and passed+failed+ignored == len(inventory)
        receipt['results'][name] = dict(passed=passed,failed=failed,ignored=ignored,exit_code=code,
            test_sha256=item['sha256'],runtime_log_sha256=sha(log),inventory_count=len(inventory))
        save()
        print(f'{a.profile} {name}: {passed} passed/{failed} failed/{ignored} ignored',flush=True)
    for name, h in pins.items():
        assert sha(ROOT/name) == h, name
    counts = {key:sum(item[key] for item in receipt['results'].values()) for key in ['passed','failed','ignored']}
    receipt.update(**counts,sources_unchanged=True,active_target=None,incidental_normal_mains_after=mains())
    assert receipt['failed'] == 0 and all(v['exit_code'] == 0 for v in receipt['results'].values()), 'Retain runtime failures without relaxing assertions.'
    receipt['status'] = 'executed_source1021_green_shared_clock_scope'
except BaseException as error:
    receipt.update(status='failed_preserved',error=repr(error))
    raise
finally:
    receipt['finished_utc'] = datetime.now(timezone.utc).isoformat()
    save()
    files = {p.name:p for p in out.iterdir() if p.is_file() and p.suffix.lower() != '.zip'}
    hashes = {name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for name,p in files.items():
            z.write(p,name)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert set(z.namelist()) == set(hashes)
        for name,h in hashes.items():
            assert digest(z.read(name)) == h
    (out/'archive-readback.json').write_text(encode(dict(files=hashes,raw_archive_sha256=sha(out/'raw.zip'),verified=True)),encoding='utf-8',newline='\n')
print(encode(dict(status=receipt['status'],output=str(out),results=receipt['results'])),flush=True)
