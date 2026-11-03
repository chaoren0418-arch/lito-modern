"""Check the pinned reference clock against independently calculated boundaries."""
import csv
import hashlib
import json
import os
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
OUT = ROOT / '驗證紀錄/current/native-lighting-clock/reference-clock'
RUNTIME = PROJECT / 'target/current/native-lighting-clock-oracle'
JDK = Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
JAR = PROJECT / 'target/current/server-reference/formal-server-C02AAAE8.jar'
SOURCE = ROOT / '維護工具/oracle/NativeLightingClockOracle.java'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
assert sha(JAR) == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
normal_path = ROOT / '驗證紀錄/current/native-light-spell/normal/qualified.json'
normal = read(normal_path)
pins = read(normal_path.parent / 'source-inputs.json')
assert len(pins) == 970
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
OUT.mkdir(exist_ok=False)
RUNTIME.mkdir(exist_ok=True)
env = os.environ.copy()
env.update(TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'),
           PYTHONDONTWRITEBYTECODE='1')

def execute(command, filename):
    result = subprocess.run(command, cwd=PROJECT, env=env, capture_output=True,
                            creationflags=subprocess.CREATE_NO_WINDOW, timeout=30)
    (OUT / filename).write_bytes(result.stdout)
    (OUT / (filename + '.stderr')).write_bytes(result.stderr)
    assert result.returncode == 0, (filename, result.returncode)

execute([str(JDK / 'javac.exe'), '-J-Dfile.encoding=UTF-8', '-encoding', 'UTF-8',
         '-cp', str(JAR), '-d', str(RUNTIME), str(SOURCE)], 'javac.log')
execute([str(JDK / 'java.exe'), '-Dfile.encoding=UTF-8', '-Xmx32m', '-Xverify:all', '-cp',
         str(RUNTIME) + os.pathsep + str(JAR), 'local.lito381.server.NativeLightingClockOracle', str(JAR)],
        'vectors.csv')
rows = list(csv.DictReader((OUT / 'vectors.csv').read_text(encoding='utf-8-sig').splitlines()))
epoch_rows = [r for r in rows if r['kind'] == 'epoch']
raw_rows = [r for r in rows if r['kind'] == 'raw']
assert len(epoch_rows) == 2014 and len(raw_rows) == 11
for row in epoch_rows:
    elapsed = int(row['input']) - 1735689600000
    if elapsed < 0:
        expected = 'ERROR:IllegalArgumentException'
    elif elapsed * 6 > 2**63 - 1:
        expected = 'ERROR:ArithmeticException'
    else:
        accelerated = elapsed * 6 // 1000
        aligned = accelerated - accelerated % 3
        expected = 'ERROR:IllegalArgumentException' if aligned > 2**31 - 1 else str(aligned)
    assert row['result'] == expected, row
for row in raw_rows:
    raw = int(row['input'])
    if raw < 0:
        assert row['result'] == 'ERROR:IllegalArgumentException'
        continue
    assert int(row['utc_second']) == raw % 86400
    assert int(row['taipei_second']) == (raw + int(row['offset_seconds'])) % 86400
example = next(r for r in raw_rows if r['input'] == '338466069')
assert int(example['taipei_second']) == 66069 and int(example['utc_second']) == 37269
proof = {'status': 'reference_C02_clock_scale_alignment_and_declared_zone_verified_only',
         'reference_jar_sha256': sha(JAR), 'actual_original_clock_class_code_source_verified': True,
         'epoch_base_millis': 1735689600000, 'real_to_game_speed': 6,
         'alignment_game_seconds': 3, 'epoch_boundary_vectors': len(epoch_rows),
         'raw_calendar_vectors': len(raw_rows), 'reference_declared_zone': 'Asia/Taipei',
         'raw338466069_Taipei_second': 66069, 'raw338466069_UTC_second': 37269,
         'original_client_calendar_UTC_or_local_resolved': False,
         'original_client_local_clock_advance_verified': False,
         'night_illumination_or_pixel_shading_verified': False,
         'client_source_files_unchanged': 970, 'new_game_sessions': 0,
         'whole_goal_complete': False}
(OUT / 'audit.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
files = {p.name: p for p in OUT.iterdir() if p.is_file()}
files['oracle.java'] = SOURCE
files['auditor.py'] = Path(__file__)
files['NativeLightingClockOracle.class'] = RUNTIME / 'local/lito381/server/NativeLightingClockOracle.class'
file_pins = {name: sha(path) for name, path in files.items()}
archive = OUT / 'clock-evidence.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
    for name, path in files.items():
        z.write(path, name)
with zipfile.ZipFile(archive) as z:
    for name, expected in file_pins.items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
(OUT / 'qualified.json').write_text(json.dumps({**proof, 'archive_sha256': sha(archive),
    'file_pins': file_pins, 'archive_readback_verified': True}, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(proof, ensure_ascii=False))
