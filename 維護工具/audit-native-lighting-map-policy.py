"""Extract and check the original map-change policy, not pixel illumination."""
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
BASE = Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收')
PE = BASE / 'runtime/LiTo.unpacked-oep-current.exe'
ASM = BASE / 'ida-client-oep-v2/full-disassembly.asm'
OUT = ROOT / '驗證紀錄/current/native-lighting-map-policy'
RUNTIME = PROJECT / 'target/current/native-lighting-map-policy-oracle'
CPP = ROOT / '維護工具/oracle/NativeLightingMapPolicyOracle.cpp'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
assert sha(PE) == '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(ASM) == '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
normal_path = ROOT / '驗證紀錄/current/native-light-spell/normal/qualified.json'
normal = read(normal_path)
pins = read(normal_path.parent / 'source-inputs.json')
assert len(pins) == 970
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
OUT.mkdir(exist_ok=False)
RUNTIME.mkdir(exist_ok=True)
raw = PE.read_bytes()
eh = struct.unpack_from('<I', raw, 60)[0]
assert raw[eh:eh+4] == b'PE\0\0'
count, optional_size = struct.unpack_from('<H', raw, eh+6)[0], struct.unpack_from('<H', raw, eh+20)[0]
optional = eh + 24
assert struct.unpack_from('<H', raw, optional)[0] == 0x10b
imagebase = struct.unpack_from('<I', raw, optional+28)[0]
sections = [struct.unpack_from('<IIII', raw, optional+optional_size+40*i+8) for i in range(count)]

def read_va(address, size):
    rva = address-imagebase
    for virtual_size, va, raw_size, raw_offset in sections:
        if va <= rva and rva+size <= va+raw_size:
            return raw[raw_offset+rva-va:raw_offset+rva-va+size]
    raise AssertionError(hex(address))

fragment = read_va(0x48562f, 49)
expected_fragment = bytes.fromhex('81fe00400000893524f362007d1cc60560aa750000b868ef62003930740c83c0043d1cf3620072f2eb07c60560aa750001')
assert fragment == expected_fragment
table_bytes = read_va(0x62ef68, 0x62f31c-0x62ef68)
table = list(struct.unpack('<237I', table_bytes))
assert len(set(table)) == len(table)
text = ASM.read_text(encoding='utf-8')
lines = text.split('\n')
start = next(i for i, line in enumerate(lines) if line.startswith('dword_62EF68 dd '))
end = next(i for i in range(start+1, len(lines)) if lines[i].startswith('dword_62F31C dd '))
asm_values = []
for line in lines[start:end]:
    if 'dd ' in line:
        values = line.split('dd ', 1)[1].split(';', 1)[0]
        asm_values.extend(int(value.strip()[:-1], 16) if value.strip().endswith('h') else int(value.strip())
                          for value in values.split(','))
assert asm_values == table
(OUT / 'map-policy-original.bin').write_bytes(fragment)
(OUT / 'map-policy-table.bin').write_bytes(table_bytes)
selected = {}
for function in ['sub_4855A0', 'sub_4D0470', 'sub_4ACD70', 'sub_488860']:
    a = next(i for i, line in enumerate(lines) if line.startswith(function+' proc'))
    b = next(i for i in range(a, len(lines)) if lines[i].startswith(function+' endp'))
    selected[function] = {'first_newline_line': a+1, 'last_newline_line': b+1, 'assembly': '\n'.join(lines[a:b+1])}
(OUT / 'selected-assembly.json').write_text(json.dumps(selected, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
args = read(ROOT / '驗證紀錄/current/native-lighting-source/build-command.json')['args']
args = [arg.replace('NativeLightingPrimitiveOracle.cpp', CPP.name).replace('native-lighting-source-oracle', RUNTIME.name) for arg in args]
env = os.environ.copy()
env.update(TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'), PYTHONDONTWRITEBYTECODE='1')

def execute(command, filename):
    result = subprocess.run(command, cwd=RUNTIME, env=env, capture_output=True,
                            creationflags=subprocess.CREATE_NO_WINDOW, timeout=30)
    (OUT / filename).write_bytes(result.stdout)
    (OUT / (filename+'.stderr')).write_bytes(result.stderr)
    assert result.returncode == 0, (filename, result.returncode)
    return result

execute(args, 'build.log')
helper = RUNTIME / 'oracle.exe'
helper_bytes = helper.read_bytes()
helper_pe = struct.unpack_from('<I', helper_bytes, 60)[0]
assert struct.unpack_from('<H', helper_bytes, helper_pe+4)[0] == 0x14c
result = execute([str(helper), str(OUT / 'map-policy-original.bin'), str(OUT / 'map-policy-table.bin'),
                  str(OUT / 'map-policy-relocated.bin'), str(OUT / 'map16-flags.bin')], 'oracle.json')
native = json.loads(result.stdout.decode('utf-8'))
assert native['map16_cases'] == 65536 and native['signed_boundaries'] == 4
flags = (OUT / 'map16-flags.bin').read_bytes()
assert len(flags) == 65536 and set(flags) == {0, 1}
for map_id, flag in enumerate(flags):
    assert flag == int(map_id >= 0x4000 or map_id in table), map_id
relocated = (OUT / 'map-policy-relocated.bin').read_bytes()
offsets = [8, 16, 22, 34, 44]
allowed = {i for offset in offsets for i in range(offset, offset+4)}
assert len(relocated) == 49
assert all(a == b or i in allowed for i, (a, b) in enumerate(zip(fragment, relocated)))
address = lambda at: struct.unpack_from('<I', relocated, at)[0]
assert address(16) == address(44) and address(34)-address(22) == 237*4
scene_maps = {str(map_id): bool(flags[map_id]) for map_id in [4, 101, 102, 500, 501, 502, 503, 510, 511, 513, 514, 527, 553, 563, 564, 573, 16383, 16384]}
proof = {'status': 'original_map_change_policy_fragment_and_table_verified_only',
    'original_PE_sha256': sha(PE), 'original_ASM_sha256': sha(ASM),
    'fragment_start': '0x0048562F', 'fragment_end_exclusive': '0x00485660',
    'original_bytes': 49, 'original_table_entries': 237, 'original_table': table,
    'native_map16_cases': 65536, 'Python_map16_rechecked': 65536, 'signed_boundaries': 4,
    'original_map_policy_flag_by_selected_map': scene_maps,
    'map_change_policy': 'signed map>=0x4000 or present in original237-entry table sets flag1; otherwise0',
    'map_initial_ambient_selection': 'flag1 passes1 to original setter; flag0 passes original4D02F0 curve result',
    'meaning_of_flag1_as_fullbright_or_dungeon_verified': False,
    'relocations': {'absolute_operand_offsets': offsets, 'addresses_only_changed': True,
                    'relative_branches_and_constants_unchanged': True,
                    'return_tail': 'harness-only movzx eax, private flag; ret appended at original fallthrough'},
    'original_client_calendar_resolved': False, 'RGB_pixel_shading_verified': False,
    'spatial_light_falloff_verified': False, 'game_sessions': 0,
    'client_source_files_unchanged': 970, 'client_remains_AMD64': True, 'whole_goal_complete': False}
(OUT / 'audit.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
(OUT / 'build-command.json').write_text(json.dumps({'args': args, 'cwd': str(RUNTIME)}, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
files = {p.name: p for p in OUT.iterdir() if p.is_file()}
files.update({'oracle.cpp': CPP, 'auditor.py': Path(__file__), 'native-x86-oracle.exe': helper, 'normal-qualified.json': normal_path})
file_pins = {name: sha(path) for name, path in files.items()}
archive = OUT / 'map-policy-evidence.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
    for name, path in files.items():
        z.write(path, name)
with zipfile.ZipFile(archive) as z:
    for name, expected in file_pins.items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
for image in normal['profiles'].values():
    assert sha(Path(image['path'])) == image['sha256']
(OUT / 'qualified.json').write_text(json.dumps({**proof, 'archive_sha256': sha(archive), 'file_pins': file_pins,
    'archive_readback_verified': True}, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps({key: proof[key] for key in ['status', 'native_map16_cases', 'Python_map16_rechecked', 'signed_boundaries', 'game_sessions', 'client_source_files_unchanged']}, ensure_ascii=False))
