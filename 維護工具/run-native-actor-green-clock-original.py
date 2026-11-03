"""Run the prepared private x86 oracle only after current default108 is terminal."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, ctypes, hashlib, json, os, subprocess, zipfile
from ctypes import wintypes

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
NORMAL = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified'
OLD = ROOT / '驗證紀錄/current/native-water-clock-original/20261016-074852-110050'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
digest = lambda b: hashlib.sha256(b).hexdigest().upper()
sha = lambda p: digest(p.read_bytes())
encode = lambda v: json.dumps(v, ensure_ascii=False, indent=2) + '\n'
ap = argparse.ArgumentParser()
ap.add_argument('directory', type=Path)
a = ap.parse_args()
out = a.directory.resolve(strict=True)
assert out.is_relative_to(ROOT / '驗證紀錄/current/native-actor-green-clock-original')
completion = read(NORMAL / 'default108-completion-observation.json')
assert completion['status'] == 'independently_readback_verified_source1019_default108_both_profiles_and_terminal_process'
assert completion['session_id'] == 29540 and completion['terminal_exit_code'] == 0
assert completion['source_files'] == 1019 and completion['owned_workloads'] == []
root_literal = str(ROOT).replace("'", "''")
command = """[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot='ROOT_LITERAL'
@(Get-CimInstance Win32_Process|Where-Object {
 $_.Name -in @('cargo.exe','rustc.exe','cl.exe','link.exe') -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($taskRoot)) -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -and
 ($_.Name -match '^(LiToModern|lito_modern-|native_|qualified-client|actual-client|oracle)' -or
 ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))
}|Select-Object ProcessId,Name,ExecutablePath,CommandLine)|ConvertTo-Json -Compress
""".replace('ROOT_LITERAL', root_literal)
idle = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command],
    capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert idle.returncode == 0 and not idle.stderr
assert not idle.stdout.strip() or json.loads(idle.stdout.decode('utf-8-sig')) == [], 'Wait for the original owned workload; do not compile concurrently.'
assert not (out / 'run-receipt.json').exists(), 'Retain the existing run; do not restart it.'
prepared = read(out / 'prepared.json')
pins = read(out / 'source-inputs.json')
normal = read(NORMAL / 'qualified.json')
assert prepared['source_files'] == len(pins) == 1019
assert sha(out / 'source-inputs.json') == prepared['source_manifest_sha256'] == normal['source_manifest_sha256']
assert normal['status'] == 'qualified_unique_entry_object_gate_normal_builds'
def check_sources():
    for name, h in pins.items():
        assert sha(ROOT / name) == h, name
    return {profile: sha(Path(value['path'])) for profile, value in normal['profiles'].items()}
before = check_sources()
assert before == {p: v['sha256'] for p, v in normal['profiles'].items()}
archive = read(out / 'preparation-readback.json')
assert sha(out / 'prepared-inputs.zip') == archive['archive_sha256']
with zipfile.ZipFile(out / 'prepared-inputs.zip') as z:
    assert set(z.namelist()) == set(archive['files'])
    for name, h in archive['files'].items():
        assert digest(z.read(name)) == h == sha(out / name), name
runtime = PROJECT / 'target/current/native-actor-green-clock-oracle' / out.name
runtime.mkdir(parents=True, exist_ok=False)
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
receipt = dict(status='building_private_original_green_callback_oracle', source_files=1019,
    normal_qualification=str(NORMAL / 'qualified.json'), normal_mains_before=before,
    runtime=str(runtime), source=str(out / 'oracle.cpp'), owned_process=None,
    product_source_modified=False, whole_goal_complete=False)
save = lambda: (out / 'run-receipt.json').write_text(encode(receipt), encoding='utf-8', newline='\n')
save()
env = os.environ.copy()
env.update(TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'),
    PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8')
def execute(args, stdout_path, timeout=60):
    with stdout_path.open('wb') as stdout, stdout_path.with_suffix(stdout_path.suffix + '.stderr').open('wb') as stderr:
        child = subprocess.Popen(args, cwd=runtime, env=env, stdout=stdout, stderr=stderr,
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
        try:
            code = child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            child.kill()  # Retained Popen handle: never a name or reused PID.
            child.wait(timeout=10)
            raise
        finally:
            receipt['owned_process'] = None
            save()
    return code
try:
    old = read(OLD / 'build-command.json')
    args = [str(out / 'oracle.cpp') if x.endswith('oracle.cpp') else
        '/Fo' + str(runtime / 'oracle.obj') if x.startswith('/Fo') else
        '/Fe' + str(runtime / 'oracle.exe') if x.startswith('/Fe') else x for x in old['args']]
    assert Path(args[0]).name.lower() == 'cl.exe' and 'x86' in Path(args[0]).parts
    assert sum(x == str(out / 'oracle.cpp') for x in args) == 1
    (out / 'build-command.json').write_text(encode(dict(args=args, cwd=str(runtime),
        compiler_sha256=sha(Path(args[0])))), encoding='utf-8', newline='\n')
    receipt['build_exit_code'] = execute(args, out / 'build.log')
    save()
    assert receipt['build_exit_code'] == 0, 'Retain the build failure.'
    for name in ['oracle.exe', 'oracle.obj']:
        (out / name).write_bytes((runtime / name).read_bytes())
        assert sha(out / name) == sha(runtime / name)
    receipt.update(status='executing_private_original_green_callback_oracle', helper_sha256=sha(out / 'oracle.exe'))
    save()
    receipt['native_exit_code'] = execute([str(runtime / 'oracle.exe'), str(out)], out / 'oracle.json')
    save()
    assert receipt['native_exit_code'] == 0, 'Retain the native failure.'
    receipt['normal_mains_after'] = check_sources()
    assert receipt['normal_mains_after'] == before
    receipt.update(status='executed_original_green_callback_private_queue', product_sources_unchanged=True)
except BaseException as error:
    receipt.update(status='failed_preserved', error=repr(error))
    raise
finally:
    receipt['finished_utc'] = datetime.now(timezone.utc).isoformat()
    save()
    files = {p.name: p for p in out.iterdir() if p.is_file() and p.name != 'prepared-inputs.zip'}
    hashes = {name: sha(p) for name, p in files.items()}
    with zipfile.ZipFile(out / 'run-raw.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        for name, p in files.items():
            z.write(p, name)
    with zipfile.ZipFile(out / 'run-raw.zip') as z:
        assert set(z.namelist()) == set(hashes)
        for name, h in hashes.items():
            assert digest(z.read(name)) == h, name
    (out / 'run-archive-readback.json').write_text(encode(dict(files=hashes,
        archive_sha256=sha(out / 'run-raw.zip'), verified=True)), encoding='utf-8', newline='\n')
print(encode(dict(status=receipt['status'], path=str(out))))
