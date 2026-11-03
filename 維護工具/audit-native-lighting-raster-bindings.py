"""Resolve original pixel-kernel dispatch and retain the next data-flow nodes."""
import hashlib
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).resolve().parents[1]
BASE = Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2')
OUT = ROOT / '驗證紀錄/current/native-lighting-raster-bindings'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
assembly = BASE / 'full-disassembly.asm'
assert sha(assembly) == '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines = assembly.read_text(encoding='utf-8').split('\n')
functions = {}
current = None
for index, line in enumerate(lines):
    match = re.match(r'^(\w+) proc\b', line)
    if match:
        current = (match.group(1), index)
    if current and line.startswith(current[0]+' endp'):
        functions[current[0]] = (current[1], index)
        current = None

def body(name):
    first, last = functions[name]
    return '\n'.join(lines[first:last+1])

initializer = body('sub_4C18E0')
assert 'mov     al, byte_641298' in initializer
assert 'jnz     short loc_4C1912' in initializer
assert 'mov     [ebp+var_C], 0Ah' in initializer and 'mov     [ebp+var_8], 5' in initializer
nonzero, zero = initializer.split('loc_4C1FBC:', 1)
pattern = r'mov\s+dword_([0-9A-F]+), offset (sub_[0-9A-F]+)'
nonzero_bindings = dict(re.findall(pattern, nonzero))
zero_bindings = dict(re.findall(pattern, zero))
assert set(nonzero_bindings) == set(zero_bindings) and len(nonzero_bindings) == 27
assert nonzero_bindings['768888'] == 'sub_4BDE70' and zero_bindings['768888'] == 'sub_4BDCE0'
assert nonzero_bindings['768880'] == 'sub_4BDB30' and zero_bindings['768880'] == 'sub_4BD970'
for name in ['sub_4BDE70', 'sub_4BDCE0']:
    kernel = body(name)
    for instruction in ['byte_83009C[ebx]', 'word_80D89C[ecx*2]', 'word_80E09C', 'word_80E89C',
                        'mov     [ebp+var_4], 198h', 'mov     [ebp+var_C], 140h',
                        'dword_6476C4', 'cmp     bl, 1Fh', 'shr     ebx, 3']:
        assert instruction in kernel, (name, instruction)
assert 'shr     eax, 6' in body('sub_4BDE70') and 'shr     edi, 0Bh' in body('sub_4BDE70')
assert 'shr     eax, 0Ah' in body('sub_4BDCE0') and 'shr     edi, 5' in body('sub_4BDCE0')
world = body('sub_488860')
assert 'mov     edx, dword_75ABE8' in world and 'call    dword_768888' in world
selected_names = ['sub_4C18E0', 'sub_4BDCE0', 'sub_4BDE70', 'sub_4BD970', 'sub_4BDB30',
                  'sub_4BB250', 'sub_4BD150', 'sub_4831B0', 'sub_483330', 'sub_485A40',
                  'sub_487230', 'sub_483770', 'sub_4CD9D0', 'sub_488860']
selected = {name: {'first_newline_line': functions[name][0]+1, 'last_newline_line': functions[name][1]+1,
                   'assembly': body(name)} for name in selected_names}
normal_path = ROOT / '驗證紀錄/current/native-light-spell/normal/qualified.json'
normal = read(normal_path)
pins = read(normal_path.parent / 'source-inputs.json')
assert len(pins) == 970
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
proof = {'status': 'original_raster_dispatch_and_mask_lookup_nodes_identified_static_only',
    'original_ASM_sha256': sha(assembly), 'initializer': 'sub_4C18E0',
    'display_mode_flag': 'byte_641298', 'dispatch_slots_per_mode': 27,
    'RGB565_nonzero_bindings': nonzero_bindings, 'RGB555_zero_bindings': zero_bindings,
    'mask_consumer_slot': 'dword_768888', 'mask_source_global_at_world_call': 'dword_75ABE8',
    'mask_kernels': {'RGB565': 'sub_4BDE70', 'RGB555': 'sub_4BDCE0'},
    'source_observations': {'row_counter': 408, 'samples_per_row': 320,
        'input_byte_quantization_shift': 3, 'remap_table': 'byte_83009C',
        'remapped31_skips_color_transform': True,
        'channel_tables': ['word_80D89C', 'word_80E09C', 'word_80E89C'],
        'optional_byte_gate_global': 'dword_6476C4'},
    'next_data_flow_nodes': {'mask_buffer_lifecycle': ['sub_4831B0', 'sub_483330'],
        'mask_producer_candidate': 'sub_485A40', 'remap_table_writers': ['sub_4BB250', 'sub_4BD150'],
        'channel_table_initializer': 'sub_4C18E0'},
    'callback487230_observation': 'reads a rectangle, clamps it to640x480, invokes a surface method, returns1; API names remain unresolved',
    'native_kernel_execution_verified': False, 'channel_table_generation_formula_verified': False,
    'full_light_mask_generation_verified': False, 'pixel_framebuffer_verified': False,
    'client_source_files_unchanged': 970, 'product_source_changes': 0, 'game_sessions': 0,
    'whole_goal_complete': False}
OUT.mkdir(exist_ok=False)
(OUT / 'audit.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
(OUT / 'selected-assembly.json').write_text(json.dumps(selected, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
files = {'auditor.py': Path(__file__), 'audit.json': OUT / 'audit.json',
         'selected-assembly.json': OUT / 'selected-assembly.json', 'normal-qualified.json': normal_path}
file_pins = {name: sha(path) for name, path in files.items()}
archive = OUT / 'raster-source-evidence.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
    for name, path in files.items():
        z.write(path, name)
with zipfile.ZipFile(archive) as z:
    for name, expected in file_pins.items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected
(OUT / 'qualified.json').write_text(json.dumps({**proof, 'archive_sha256': sha(archive), 'file_pins': file_pins,
    'archive_readback_verified': True}, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps({key: proof[key] for key in ['status', 'dispatch_slots_per_mode', 'mask_kernels', 'native_kernel_execution_verified', 'game_sessions']}, ensure_ascii=False))
