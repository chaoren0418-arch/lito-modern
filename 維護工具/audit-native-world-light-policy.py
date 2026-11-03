"""Run after current GUI cleanup; original map-to-mask argument only."""
from pathlib import Path
import hashlib
import json
import os
import struct
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
OUT = ROOT / '驗證紀錄/current/native-world-light-policy'
RUNTIME = PROJECT / 'target/current/native-world-light-policy-oracle'
CPP = ROOT / '維護工具/oracle/NativeWorldLightPolicyOracle.cpp'
NORMAL = ROOT / '驗證紀錄/current/native-game-timers/20261016-004010-201637/normal'
STAGE = NORMAL.parent / 'current-regression'
PE = Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe')

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))

def write(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

# A receipt alone does not permit a workload to overlap GUI/native tests.
state = read(STAGE / 'current-followup.json')
assert state['status'] == 'completed_current_default93_and_TW14_defined_scopes'
assert not state['source_and_GUI_QA_must_remain_frozen']
guard_command = r"""
$root='D:\lineage開發\3-lito開發\lito重建版'
$live=@(Get-CimInstance Win32_Process | Where-Object {
 $_.ProcessId -eq 56764 -or $_.Name -in @('cargo.exe','rustc.exe') -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root,[StringComparison]::OrdinalIgnoreCase) -and
 ($_.Name -match '^(LiToModern|native_|qualified-client)' -or $_.ExecutablePath -match '\\deps\\.*\.exe$')) -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($root))
})
if($live.Count){throw 'Prior owned pipeline/GUI/Java/native workload still live'}
Write-Output 'prior_pipeline_and_owned_workloads_absent'
"""
guard = subprocess.run(['powershell.exe', '-NoProfile', '-NonInteractive', '-Command', guard_command],
                       capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW, timeout=30)
assert guard.returncode == 0, guard.stderr.decode('utf-8', errors='replace')
assert not OUT.exists(), 'Preserve existing original run'
OUT.mkdir()
RUNTIME.mkdir(exist_ok=True)
(OUT / 'live-workload-guard.stdout').write_bytes(guard.stdout)
(OUT / 'live-workload-guard.stderr').write_bytes(guard.stderr)
normal = read(NORMAL / 'qualified.json')
pins = read(NORMAL / 'source-inputs.json')
assert len(pins) == 976
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
for item in normal['profiles'].values():
    assert sha(Path(item['path'])) == item['sha256']
assert sha(PE) == '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
raw = PE.read_bytes()
header = struct.unpack_from('<I', raw, 60)[0]
section_count = struct.unpack_from('<H', raw, header + 6)[0]
optional_size = struct.unpack_from('<H', raw, header + 20)[0]
optional = header + 24
imagebase = struct.unpack_from('<I', raw, optional + 28)[0]
sections = [struct.unpack_from('<IIII', raw, optional + optional_size + 40*i + 8)
            for i in range(section_count)]

def read_va(address, size):
    rva = address - imagebase
    for virtual_size, va, raw_size, offset in sections:
        if va <= rva and rva + size <= va + raw_size:
            return raw[offset+rva-va:offset+rva-va+size]
    raise AssertionError(hex(address))

fragment = read_va(0x488AE0, 0x488BA7 - 0x488AE0)
assert len(fragment) == 199 and fragment[:6] == bytes.fromhex('8a0d60aa7500')
assert read_va(0x488BA7, 1) == b'\xe8', 'Original call boundary'
(OUT / 'map-argument-original.bin').write_bytes(fragment)
(OUT / 'oracle.cpp').write_bytes(CPP.read_bytes())
(OUT / 'auditor.py').write_bytes(Path(__file__).read_bytes())
write(OUT / 'normal-qualification.json', normal)
write(OUT / 'source-inputs.json', pins)
arguments = read(ROOT / '驗證紀錄/current/native-lighting-source/build-command.json')['args']
arguments = [arg.replace('NativeLightingPrimitiveOracle.cpp', CPP.name)
             .replace('native-lighting-source-oracle', RUNTIME.name) for arg in arguments]
write(OUT / 'build-command.json', {'args': arguments, 'cwd': str(RUNTIME)})
env = os.environ.copy()
env.update(PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8',
           TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'))

def execute(command, filename):
    done = subprocess.run(command, cwd=RUNTIME, env=env, capture_output=True,
                          creationflags=subprocess.CREATE_NO_WINDOW, timeout=45)
    (OUT / filename).write_bytes(done.stdout)
    (OUT / (filename + '.stderr')).write_bytes(done.stderr)
    assert done.returncode == 0, (filename, done.returncode)
    return done

try:
    execute(arguments, 'build.log')
    (OUT / 'oracle.exe').write_bytes((RUNTIME / 'oracle.exe').read_bytes())
    native = json.loads(execute([str(RUNTIME / 'oracle.exe'), str(OUT)], 'native.json').stdout)
    values = (OUT / 'native-map-levels.bin').read_bytes()
    assert len(values) == 262144
    overrides = {**dict.fromkeys(range(72, 75), 4), **dict.fromkeys(range(75, 78), 11),
                 **dict.fromkeys([307, 308, 309, 310, 320, 330, 511, 527], 2)}
    for index, flag in enumerate([0, 1, 128, 255]):
        for map_id in range(65536):
            expected = overrides.get(map_id, 1) if flag else (8 if 547 <= map_id <= 550 else 15)
            assert values[index*65536+map_id] == expected, (flag, map_id)
    relocated = (OUT / 'map-argument-relocated.bin').read_bytes()
    assert len(relocated) == len(fragment)
    assert all(a == b or 2 <= index < 6 for index, (a, b) in enumerate(zip(fragment, relocated)))
    for name, digest in pins.items():
        assert sha(ROOT / name) == digest, name
    for item in normal['profiles'].values():
        assert sha(Path(item['path'])) == item['sha256']
except BaseException as error:
    write(OUT / 'failure.json', {'status': 'failed_retained', 'error': f'{type(error).__name__}: {error}',
                                'world_paint_executed': False, 'whole_goal_complete': False})
    raise
finally:
    file_pins = {p.name: sha(p) for p in OUT.iterdir() if p.is_file()}
    packed = OUT / 'world-light-policy-evidence.zip'
    with zipfile.ZipFile(packed, 'x', zipfile.ZIP_DEFLATED) as archive:
        for name in file_pins:
            archive.write(OUT / name, name)
    with zipfile.ZipFile(packed) as archive:
        for name, digest in file_pins.items():
            assert hashlib.sha256(archive.read(name)).hexdigest().upper() == digest
    write(OUT / 'retention.json', {'files': file_pins, 'archive_sha256': sha(packed),
                                  'readback_verified': True})
proof = {**native, 'status': 'original_map_to_mask_argument_fragment_native_and_scalar_verified',
         'original_PE_sha256': sha(PE), 'original_fragment_sha256': sha(OUT / 'map-argument-original.bin'),
         'flag_inputs_executed': [0, 1, 128, 255], 'Python_map_cases_rechecked': len(values),
         'addresses_only_relocated': True, 'all_original_internal_branches_unchanged': True,
         'observation_tail': 'mov eax,ecx; ret appended before original485A40 call',
         'source_files_unchanged': len(pins), 'source_manifest_sha256': normal['source_manifest_sha256'],
         'normal_mains': {p: v['sha256'] for p, v in normal['profiles'].items()},
         'complete_world_framebuffer_verified': False, 'daylight_to_framebuffer_use_resolved': False,
         'archive_sha256': sha(packed), 'archive_readback_verified': True, 'whole_goal_complete': False}
write(OUT / 'qualified.json', proof)
print(json.dumps(proof, ensure_ascii=False))
