"""Execute original pixel kernels against supplied synthetic tables only."""
import csv
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import zipfile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_CALL, CS_GRP_JUMP
from capstone.x86 import X86_OP_IMM

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
PE = Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe')
OUT = ROOT / '驗證紀錄/current/native-lighting-raster-kernels'
RUNTIME = PROJECT / 'target/current/native-lighting-raster-oracle'
CPP = ROOT / '維護工具/oracle/NativeLightingRasterOracle.cpp'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
assert sha(PE) == '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
normal_path = ROOT / '驗證紀錄/current/native-light-spell/normal/qualified.json'
normal = read(normal_path)
pins = read(normal_path.parent / 'source-inputs.json')
assert len(pins) == 970
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
run = ROOT / '驗證紀錄/current/default-native-regression/20261015-223525-056-Release'
phase = read(run / 'receipt.json')
assert phase['status'] == 'compiling_default_targets', 'Defer independent CPU oracle until GUI cohort completes'
messages = [json.loads(line) for line in (run / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
images = [m for m in messages if m.get('reason') == 'compiler-artifact' and m.get('executable') and m['profile']['test']]
assert len(images) < 85, 'Do not overlap the approaching GUI cohort'
OUT.mkdir(exist_ok=False)
RUNTIME.mkdir(exist_ok=True)
raw = PE.read_bytes()
eh = struct.unpack_from('<I', raw, 60)[0]
optional = eh+24
size = struct.unpack_from('<H', raw, eh+20)[0]
base = struct.unpack_from('<I', raw, optional+28)[0]
sections = [struct.unpack_from('<IIII', raw, optional+size+40*i+8)
            for i in range(struct.unpack_from('<H', raw, eh+6)[0])]

def read_va(address, count):
    rva = address-base
    for vs, va, rs, rp in sections:
        if va <= rva and rva+count <= va+rs:
            return raw[rp+rva-va:rp+rva-va+count]
    raise AssertionError(hex(address))

decoder = Cs(CS_ARCH_X86, CS_MODE_32)
decoder.detail = True
functions = {}
for mode, start, count in [('555', 0x4bdce0, 399), ('565', 0x4bde70, 393)]:
    code = read_va(start, count)
    instructions = list(decoder.disasm(code, start))
    assert sum(i.size for i in instructions) == len(code)
    assert not any(i.group(CS_GRP_CALL) for i in instructions)
    for instruction in instructions:
        if instruction.group(CS_GRP_JUMP):
            assert instruction.operands[0].type == X86_OP_IMM
            assert start <= instruction.operands[0].imm < start+count
    assert instructions[-1].mnemonic == 'ret' and instructions[-1].op_str == '0x10'
    relocations = []
    for address, expected_count in [(0x6476c4,1),(0x83009c,1),(0x80d89c,2),(0x80e09c,2),(0x80e89c,2)]:
        needle = struct.pack('<I', address)
        offsets = [i for i in range(len(code)-3) if code[i:i+4] == needle]
        assert len(offsets) == expected_count
        relocations.extend({'offset': i, 'original_address': hex(address)} for i in offsets)
    (OUT / f'kernel{mode}-original.bin').write_bytes(code)
    functions[mode] = {'start': hex(start), 'bytes': count, 'relocations': relocations,
        'instructions': [{'address': hex(i.address), 'bytes': bytes(i.bytes).hex(), 'mnemonic': i.mnemonic, 'operands': i.op_str} for i in instructions]}
(OUT / 'extracted-code.json').write_text(json.dumps(functions, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
args = read(ROOT / '驗證紀錄/current/native-lighting-source/build-command.json')['args']
args = [arg.replace('NativeLightingPrimitiveOracle.cpp', CPP.name).replace('native-lighting-source-oracle', RUNTIME.name) for arg in args]
env = os.environ.copy()
env.update(TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'), PYTHONDONTWRITEBYTECODE='1')

def execute(command, filename):
    result = subprocess.run(command, cwd=RUNTIME, env=env, capture_output=True,
                            creationflags=subprocess.CREATE_NO_WINDOW, timeout=45)
    (OUT / filename).write_bytes(result.stdout)
    (OUT / (filename+'.stderr')).write_bytes(result.stderr)
    assert result.returncode == 0, (filename, result.returncode)
    return result

execute(args, 'build.log')
helper = RUNTIME / 'oracle.exe'
result = execute([str(helper), str(OUT/'kernel555-original.bin'), str(OUT/'kernel565-original.bin'),
    str(OUT/'kernel555-relocated.bin'), str(OUT/'kernel565-relocated.bin'), str(OUT/'vectors.csv')], 'oracle.json')
native = json.loads(result.stdout.decode('utf-8'))
assert native['native_calls'] == 134 and native['exhaustive_color_raw_pairs'] == 2*65536*256
assert native['viewport_pixels_checked'] == 134*640*408
assert native['guard_pixels_checked'] == 134*(800*414-640*408)
for mode, function in functions.items():
    original = (OUT/f'kernel{mode}-original.bin').read_bytes()
    relocated = (OUT/f'kernel{mode}-relocated.bin').read_bytes()
    allowed = {i for item in function['relocations'] for i in range(item['offset'],item['offset']+4)}
    assert len(original) == len(relocated)
    assert all(a == b or i in allowed for i, (a,b) in enumerate(zip(original,relocated)))
    for address in {r['original_address'] for r in function['relocations']}:
        values = {struct.unpack_from('<I',relocated,r['offset'])[0] for r in function['relocations'] if r['original_address'] == address}
        assert len(values) == 1
rows = list(csv.DictReader((OUT/'vectors.csv').read_text(encoding='utf-8-sig').splitlines()))
assert len(rows) == native['retained_vectors'] == 11968
for row in rows:
    mode,phase,raw_value,color,gate,mapped,actual = (int(row[key]) for key in ['mode','pass','raw','color','gate','mapped','actual'])
    level = ((raw_value//8)*13+7)%32 if phase else raw_value//8
    assert mapped == level
    if level == 31 or phase and gate == 0:
        expected = color
    else:
        components = [color&31, (color>>(6 if mode else 5))&31, (color>>(11 if mode else 10))&31]
        values = [((c*17+level*7+salt)%32 if phase else c*level//31) for c,salt in zip(components,[3,11,19])]
        expected = values[0] | values[1]<<(6 if mode else 5) | values[2]<<(11 if mode else 10)
    assert actual == expected, row
proof = {'status': native['status'], 'original_PE_sha256': sha(PE), **native,
    'Python_vectors_rechecked': len(rows), 'data_only_relocations_per_kernel': 8,
    'original_instructions_relative_branches_constants_unchanged': True,
    'external_original_calls': 0, 'fixture_extent': {'viewport': [640,408], 'mask': [320,204],
        'pixel_pitch_words': 800, 'offset_pixels': [33,3], 'optional_gate_stride': 744},
    'tables_scope': 'Synthetic declared LUT/remap fixtures: multiply and adversarial; not original table-generation qualification',
    'original_channel_table_generation_verified': False, 'complete_mask_generation_verified': False,
    'client_source_files_unchanged': 970, 'product_source_changes': 0, 'whole_goal_complete': False}
(OUT/'audit.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
(OUT/'build-command.json').write_text(json.dumps({'args': args, 'cwd': str(RUNTIME)}, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
files = {p.name:p for p in OUT.iterdir() if p.is_file()}
files.update({'oracle.cpp': CPP, 'auditor.py': Path(__file__), 'native-x86-oracle.exe': helper, 'normal-qualified.json': normal_path})
file_pins = {name:sha(path) for name,path in files.items()}
archive = OUT/'raster-kernel-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
    for name,path in files.items():z.write(path,name)
with zipfile.ZipFile(archive) as z:
    for name,expected in file_pins.items():assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected
for name,expected in pins.items():assert sha(ROOT/name) == expected,name
for image in normal['profiles'].values():assert sha(Path(image['path'])) == image['sha256']
(OUT/'qualified.json').write_text(json.dumps({**proof,'archive_sha256':sha(archive),'file_pins':file_pins,
    'archive_readback_verified':True}, ensure_ascii=False, indent=2)+'\n',encoding='utf-8')
print(json.dumps({key:proof[key] for key in ['status','native_calls','exhaustive_color_raw_pairs','guard_pixels_checked','Python_vectors_rechecked','original_channel_table_generation_verified','game_sessions']},ensure_ascii=False))
