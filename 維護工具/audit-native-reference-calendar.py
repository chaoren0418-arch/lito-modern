"""Export and independently check C02's declared calendar after GUI cleanup."""
from pathlib import Path
import bisect
import csv
import datetime
import hashlib
import json
import os
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
OUT = ROOT / '驗證紀錄/current/native-reference-calendar'
RUNTIME = PROJECT / 'target/current/native-reference-calendar-oracle'
JDK = Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot')
JAR = PROJECT / 'target/current/server-reference/formal-server-C02AAAE8.jar'
SOURCE = ROOT / '維護工具/oracle/NativeReferenceCalendarOracle.java'
NORMAL = ROOT / '驗證紀錄/current/native-game-timers/20261016-004010-201637/normal'
STAGE = NORMAL.parent / 'current-regression'

def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))

def write(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

state = read(STAGE / 'current-followup.json')
assert state['status'] == 'completed_current_default93_and_TW14_defined_scopes'
assert not state['source_and_GUI_QA_must_remain_frozen']
guard = subprocess.run(['powershell.exe', '-NoProfile', '-NonInteractive', '-Command', r"""
$root='D:\lineage開發\3-lito開發\lito重建版'
$live=@(Get-CimInstance Win32_Process | Where-Object {
 $_.ProcessId -eq 56764 -or $_.Name -in @('cargo.exe','rustc.exe') -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root,[StringComparison]::OrdinalIgnoreCase) -and
 ($_.Name -match '^(LiToModern|native_|qualified-client)' -or $_.ExecutablePath -match '\\deps\\.*\.exe$')) -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($root))
})
if($live.Count){throw 'Prior owned workload is still live'}
Write-Output 'prior_pipeline_and_owned_workloads_absent'
"""], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW, timeout=30)
assert guard.returncode == 0, guard.stderr.decode('utf-8', errors='replace')
assert not OUT.exists(), 'Keep previous runs immutable'
OUT.mkdir()
RUNTIME.mkdir(exist_ok=True)
(OUT / 'guard.stdout').write_bytes(guard.stdout)
(OUT / 'guard.stderr').write_bytes(guard.stderr)
(OUT / 'oracle.java').write_bytes(SOURCE.read_bytes())
(OUT / 'auditor.py').write_bytes(Path(__file__).read_bytes())
env = os.environ.copy()
env.update(PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8',
           TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'))

def execute(arguments, name):
    write(OUT / (name + '.arguments.json'), arguments)
    done = subprocess.run(arguments, cwd=PROJECT, env=env, capture_output=True,
                          creationflags=subprocess.CREATE_NO_WINDOW, timeout=45)
    (OUT / (name + '.stdout')).write_bytes(done.stdout)
    (OUT / (name + '.stderr')).write_bytes(done.stderr)
    assert done.returncode == 0, (name, done.returncode)
    return done

try:
    normal = read(NORMAL / 'qualified.json')
    pins = read(NORMAL / 'source-inputs.json')
    assert len(pins) == 976
    for name, digest in pins.items():
        assert sha(ROOT / name) == digest, name
    for item in normal['profiles'].values():
        assert sha(Path(item['path'])) == item['sha256']
    assert sha(JAR) == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
    write(OUT / 'normal-qualification.json', normal)
    write(OUT / 'source-inputs.json', pins)
    (OUT / 'tzdb.dat').write_bytes((JDK / 'lib/tzdb.dat').read_bytes())
    execute([str(JDK / 'bin/javac.exe'), '-J-Dfile.encoding=UTF-8', '-encoding', 'UTF-8',
             '-cp', str(JAR), '-d', str(RUNTIME), str(SOURCE)], 'javac')
    result = execute([str(JDK / 'bin/java.exe'), '-Dfile.encoding=UTF-8', '-Xmx96m', '-Xverify:all',
                      '-cp', str(RUNTIME) + os.pathsep + str(JAR),
                      'local.lito381.server.NativeReferenceCalendarOracle', str(JAR), str(OUT)], 'oracle')
    native = json.loads(result.stdout)
    assert native['zone'] == 'Asia/Taipei' and native['epoch_zero_offset'] == 28800
    assert native['jar_sha256'] == sha(JAR)
    (OUT / 'NativeReferenceCalendarOracle.class').write_bytes(
        (RUNTIME / 'local/lito381/server/NativeReferenceCalendarOracle.class').read_bytes())
    transitions = []
    previous_offset = native['epoch_zero_offset']
    with (OUT / 'transitions.csv').open(encoding='utf-8', newline='') as stream:
        for row in csv.DictReader(stream):
            raw, before, after = (int(row[k]) for k in ['raw', 'offset_before', 'offset_after'])
            assert 0 <= raw <= 2147483647 and before == previous_offset
            assert not transitions or transitions[-1][0] < raw
            transitions.append([raw, after])
            previous_offset = after
    assert len(transitions) == native['transitions']
    starts = [row[0] for row in transitions]
    samples = 0
    years = set()
    observed_offsets = set()
    with (OUT / 'calendar.csv').open(encoding='utf-8', newline='') as stream:
        for row in csv.DictReader(stream):
            row = {k: int(v) for k, v in row.items()}
            raw = row['raw']
            index = bisect.bisect_right(starts, raw) - 1
            offset = transitions[index][1] if index >= 0 else native['epoch_zero_offset']
            actual = datetime.datetime.fromtimestamp(raw + offset, datetime.timezone.utc)
            expected = [actual.year, actual.timetuple().tm_yday - 1, actual.month, actual.day,
                        actual.hour, actual.minute, actual.second]
            assert row['offset'] == offset
            assert [row[k] for k in ['year', 'yday0', 'month1', 'day', 'hour', 'minute', 'second']] == expected
            assert row['reference_second'] == actual.hour*3600 + actual.minute*60 + actual.second
            years.add(actual.year)
            observed_offsets.add(offset)
            samples += 1
    assert samples == native['calendar_samples'] and samples > 200000
    assert min(years) == 1970 and max(years) == 2038
    assert observed_offsets == {28800, 32400}
    policy = {'reference_jar_sha256': sha(JAR), 'reference_declared_zone': 'Asia/Taipei',
              'jdk_tzdb_version': native['tzdb_version'], 'jdk_tzdb_sha256': sha(OUT / 'tzdb.dat'),
              'raw_domain_inclusive': [0, 2147483647], 'initial_offset_seconds': native['epoch_zero_offset'],
              'offset_transitions_raw_and_after_seconds': transitions,
              'original_client_calendar_API_UTC_or_local_resolved': False}
    write(OUT / 'reference-policy.json', policy)
    for name, digest in pins.items():
        assert sha(ROOT / name) == digest, name
    for item in normal['profiles'].values():
        assert sha(Path(item['path'])) == item['sha256']
except BaseException as error:
    write(OUT / 'failure.json', {'status': 'failed_retained', 'error': f'{type(error).__name__}: {error}',
                                'actual_game_sessions': 0, 'whole_goal_complete': False})
    raise
finally:
    file_pins = {p.name: sha(p) for p in OUT.iterdir() if p.is_file()}
    packed = OUT / 'reference-calendar-evidence.zip'
    with zipfile.ZipFile(packed, 'x', zipfile.ZIP_DEFLATED) as archive:
        for name in file_pins:
            archive.write(OUT / name, name)
    with zipfile.ZipFile(packed) as archive:
        for name, digest in file_pins.items():
            assert hashlib.sha256(archive.read(name)).hexdigest().upper() == digest
    write(OUT / 'retention.json', {'files': file_pins, 'archive_sha256': sha(packed),
                                  'readback_verified': True})
proof = {**native, 'status': 'formal_C02_calendar_policy_and_historical_DST_vectors_verified',
         'Python_calendar_samples_verified': samples, 'years_inclusive': [min(years), max(years)],
         'offset_seconds_observed': sorted(observed_offsets),
         'source_files_unchanged': len(pins), 'source_manifest_sha256': normal['source_manifest_sha256'],
         'normal_mains': {p: v['sha256'] for p, v in normal['profiles'].items()},
         'original_client_calendar_UTC_or_local_resolved': False,
         'local_six_times_clock_prediction_implemented': False,
         'framebuffer_or_gameplay_verified': False, 'whole_goal_complete': False,
         'archive_sha256': sha(packed), 'archive_readback_verified': True}
write(OUT / 'qualified.json', proof)
print(json.dumps(proof, ensure_ascii=False))
