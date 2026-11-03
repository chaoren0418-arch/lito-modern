"""Run the two current default Rust profiles only after the retained GUI cohort ends."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parent.parent
ENV = ROOT / '驗證紀錄/current/native-environment/20261016-023507-851373'
STAGE = ENV / 'current-regression'
NORMAL = ENV / 'normal/qualified.json'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
now = lambda: datetime.now(timezone.utc).isoformat()


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1 << 20), b''):
            h.update(block)
    return h.hexdigest().upper()


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def created(handle):
    times = [wintypes.FILETIME() for _ in range(4)]
    call = ctypes.WinDLL('kernel32', use_last_error=True).GetProcessTimes
    call.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    call.restype = wintypes.BOOL
    assert call(wintypes.HANDLE(handle), *[ctypes.byref(t) for t in times])
    ticks = (times[0].dwHighDateTime << 32) | times[0].dwLowDateTime
    return (datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)).isoformat()


def main():
    gui = read(ENV / 'GUI-current/current14-followup.json')
    assert gui['status'] == 'qualified_current_environment_TW14_GUI_cases'
    assert not gui['source_and_corresponding_tests_frozen']
    assert Path(gui['evidence']).is_file()
    script = r"""
    $root='D:\lineage開發\3-lito開發\lito重建版'
    $live=@(Get-CimInstance Win32_Process|Where-Object {
      $_.Name -in @('cargo.exe','rustc.exe') -or
      ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($root)) -or
      ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root,[StringComparison]::OrdinalIgnoreCase) -and
        ($_.Name -match '^(LiToModern|native_|qualified-client|actual-client)' -or $_.ExecutablePath -match '\\deps\\.*\.exe$'))
    })
    if($live.Count){$live|Select-Object ProcessId,CreationDate,CommandLine|ConvertTo-Json -Compress;exit 1}
    """
    encoded = __import__('base64').b64encode(script.encode('utf-16-le')).decode('ascii')
    guard = subprocess.run(['powershell.exe', '-NoProfile', '-EncodedCommand', encoded],
                           capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
    assert guard.returncode == 0, guard.stdout.decode('utf-8-sig', errors='replace')
    STAGE.mkdir(exist_ok=False)
    get_current = ctypes.WinDLL('kernel32').GetCurrentProcess
    get_current.restype = wintypes.HANDLE
    state = {'status': 'running', 'pipeline_pid': os.getpid(),
             'pipeline_creation_utc': created(get_current()), 'started_utc': now(),
             'prior_GUI_evidence': gui['evidence'], 'prior_GUI_evidence_sha256': sha(gui['evidence']),
             'normal_qualification_sha256': sha(NORMAL), 'profiles': {},
             'source_files': 978, 'additional_compile_fixtures': 4,
             'compiled_input_files': 982, 'source_and_corresponding_tests_frozen': True,
             'full_library_unit_regression_verified': False,
             'full_current_TCP_regression_verified': False, 'whole_goal_complete': False}
    state_path = STAGE / 'default94-followup.json'
    persist = lambda: save(state_path, state)
    persist()
    env = os.environ.copy()
    env.update(PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8',
               TEMP=str(ROOT / 'LiTo381-Modern/target/current/scratch'),
               TMP=str(ROOT / 'LiTo381-Modern/target/current/scratch'))

    def execute(command, filename):
        with (STAGE / filename).open('wb') as log:
            child = subprocess.Popen(command, cwd=ROOT, env=env, stdout=log,
                                     stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
            state['owned_child'] = {'pid': child.pid, 'creation_utc': created(int(child._handle)),
                                    'arguments': command}
            persist()
            code = child.wait()
        state['owned_child'] = None
        persist()
        assert code == 0, (filename, code)

    try:
        for profile in ('Debug', 'Release'):
            state['active_profile'] = profile
            state['phase'] = 'build_and_run_default94_and_main'
            persist()
            print(profile + ': starting current default94 and main tests', flush=True)
            execute([sys.executable, '-X', 'utf8', str(ROOT / '維護工具/run-native-environment-default-regression.py'),
                     '--profile', profile, '--normal-qualification', str(NORMAL)], profile + '-runner-console.txt')
            last = (STAGE / (profile + '-runner-console.txt')).read_text('utf-8-sig').splitlines()[-1]
            summary = json.loads(last)
            run = Path(summary['output']).resolve(strict=True)
            assert run.is_relative_to(ROOT / '驗證紀錄/current/default-native-regression')
            state['profiles'][profile] = {'run': str(run), 'runner': summary}
            state['phase'] = 'independent_qualification'
            persist()
            qualification = STAGE / (profile + '-default94-qualified.json')
            execute([sys.executable, '-X', 'utf8', str(ROOT / '維護工具/qualify-native-environment-default-regression.py'),
                     str(run), '--output', str(qualification), '--normal-qualification', str(NORMAL)],
                    profile + '-qualification-console.txt')
            # Raw logs/reports are separate from the already read-back products/source ZIPs.
            raw_files = sorted(p for p in run.rglob('*') if p.is_file() and p.suffix.lower() != '.zip')
            pins = {p.relative_to(run).as_posix(): sha(p) for p in raw_files}
            archive = run / 'raw-evidence.zip'
            with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
                for p in raw_files:
                    z.write(p, p.relative_to(run).as_posix())
            with zipfile.ZipFile(archive) as z:
                assert set(z.namelist()) == set(pins)
                for name, expected in pins.items():
                    h = hashlib.sha256()
                    with z.open(name) as f:
                        for block in iter(lambda: f.read(1 << 20), b''):
                            h.update(block)
                    assert h.hexdigest().upper() == expected, name
            archive_receipt = {'status': 'raw_archive_readback_verified', 'files': pins,
                               'archive_sha256': sha(archive), 'qualification_sha256': sha(qualification)}
            save(run / 'raw-archive-readback.json', archive_receipt)
            q = read(qualification)
            state['profiles'][profile].update(qualification=str(qualification),
                qualification_sha256=sha(qualification), counts={k: q[k] for k in ('passed', 'failed', 'ignored')},
                raw_archive=str(archive), raw_archive_sha256=archive_receipt['archive_sha256'])
            persist()
            print(profile + ': independently qualified ' + json.dumps(state['profiles'][profile]['counts']), flush=True)
        state.update(status='qualified_current_environment_default94_and_main_both_profiles',
                     active_profile=None, phase=None, source_and_corresponding_tests_frozen=False)
    except BaseException as error:
        state.update(status='failed', error=f'{type(error).__name__}: {error}')
        raise
    finally:
        state['last_observed_utc'] = now()
        persist()
    print(json.dumps(state, ensure_ascii=False), flush=True)


if __name__ == '__main__':
    main()
