"""Read-only primary binding; do not infer a daylight pixel consumer from a setter."""
from pathlib import Path
from datetime import datetime, timezone
import capstone, hashlib, json, struct, zipfile

ROOT = Path(__file__).resolve().parents[1]
ORIGINAL = ROOT.parent / 'lito登入器/舊版分析與驗收'

def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()

def read(p):
    return json.loads(Path(p).read_text(encoding='utf-8-sig'))

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

pe_path = ORIGINAL / 'runtime/LiTo.unpacked-oep-current.exe'
asm_path = ORIGINAL / 'ida-client-oep-v2/full-disassembly.asm'
assert sha(pe_path) == '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(asm_path) == '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
pe = pe_path.read_bytes()
header = struct.unpack_from('<I', pe, 60)[0]
section_count = struct.unpack_from('<H', pe, header + 6)[0]
section_start = header + 24 + struct.unpack_from('<H', pe, header + 20)[0]
image_base = struct.unpack_from('<I', pe, header + 52)[0]
assert image_base == 0x400000
sections = [struct.unpack_from('<IIII', pe, section_start + i * 40 + 8) for i in range(section_count)]

def raw(address, length):
    rva = address - image_base
    for size, start, physical, at in sections:
        if start <= rva and rva + length <= start + min(size, physical):
            return pe[at + rva - start:at + rva - start + length]
    raise ValueError(hex(address))

def file_va(offset):
    for size, start, physical, at in sections:
        if at <= offset < at + physical:
            return hex(image_base + start + offset - at)
    return None

def references(address):
    needle = struct.pack('<I', address)
    result, start = [], 0
    while (offset := pe.find(needle, start)) >= 0:
        result.append(dict(raw_offset=offset, operand_byte_va=file_va(offset)))
        start = offset + 1
    return result

ranges = {
    'static_map_mask_argument': (0x488AE0, 0x488BB0),
    'mask_producer': (0x485A40, 0x485D20),
    'mask_pixel_gate': (0x489113, 0x489205),
    'encoded_integer_setter': (0x4C3E90, 0x4C3EE9),
    'encoded_integer_random_helper': (0x4C3CA0, 0x4C3CC0),
    'class2_ambient_entry_restore': (0x44EBD0, 0x44ED80),
    'map_scene_ambient_reset': (0x485734, 0x48576C),
}
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
decoded = {name: list(md.disasm(raw(lo, hi - lo), lo)) for name, (lo, hi) in ranges.items()}
assert any(i.mnemonic == 'call' and i.op_str == '0x485a40' for i in decoded['static_map_mask_argument'])
assert any(i.mnemonic == 'mov' and i.op_str == 'ecx, dword ptr [ebp - 8]' for i in decoded['static_map_mask_argument'])
assert any(i.mnemonic == 'cmp' and i.op_str == 'eax, 0xf' for i in decoded['mask_producer'])
assert any('0x831690' in i.op_str for i in decoded['mask_producer'])
assert any(i.mnemonic == 'cmp' and i.op_str == 'dword ptr [ebp - 8], 0xf' for i in decoded['mask_pixel_gate'])
for name in ['static_map_mask_argument', 'mask_producer', 'mask_pixel_gate']:
    assert not any(any(hex(address) in i.op_str for address in [0x8316A0, 0x8316A4, 0x8316A8]) for i in decoded[name])
refs = {hex(address): references(address) for address in [0x8316A0, 0x8316A4, 0x8316A8]}
assert len(refs['0x8316a0']) == 11

run = ROOT / '驗證紀錄/current/native-class2-live/20261018-001311-469195'
qpath = run / 'qualified-20261018-001510-643949/qualified.json'
q = read(qpath)
assert q['non15_ambient_lifecycle_verified'] and not q['non15_brightness_change_verified']
runtime = Path(read(run / 'receipt.json')['runtime'])
client = read(runtime / 'b-client.json')
initial = client['initial']['render']['receipt']
hit = read(runtime / 'b-gfx11736-hit.json')['receipt']
observed = {}
for name, receipt in [('initial', initial), ('hit', hit)]:
    light = receipt['world_lighting']
    assert not light['active'] and light['reason'] == 'native_full_bright_mask_bypass'
    observed[name] = light

stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out = ROOT / '驗證紀錄/current/native-class2-light-consumer' / stamp
out.mkdir(parents=True)
for name, (lo, hi) in ranges.items():
    (out / (name + '.bin')).write_bytes(raw(lo, hi - lo))
    (out / (name + '.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex():<24} {i.mnemonic} {i.op_str}' for i in decoded[name]) + '\n', encoding='utf-8')
facts = dict(
    status='primary_static_mask_binding_qualified_dynamic_ambient_pixel_consumer_unresolved',
    primary_PE=dict(path=str(pe_path), sha256=sha(pe_path)),
    primary_ASM=dict(path=str(asm_path), sha256=sha(asm_path)),
    encoded_ambient_direct_operand_occurrences=refs,
    direct_operand_search_is_not_indirect_alias_proof=True,
    observed_primary=dict(map_policy_argument='488AE0..488BA2 computes static map argument in local -8; ECX passed to485A40',
        producer='485A40 consults blind831690 and underwater64BC2D; dry unblinded argument>=15 returns',
        pixel_gate='489140..489156 uses the same local map argument threshold15',
        dynamic_ambient='8316A0 state changes found in class1/class2, map load, weather and time; no consumer established in inspected render ranges'),
    current_candidate=dict(qualification=str(qpath), qualification_sha256=sha(qpath), actual_runtime=str(runtime),
        controlled_local_S21_replay=True, real_C02_night_clock=False, observed_render_receipts=observed,
        ambient_lifecycle_verified=True, pixel_brightness_verified=False),
    decision='Retain separate static mask_argument and dynamic ambient_level. Do not substitute ambient_level into the qualified mask pass without primary consumer evidence.',
    source_modified=False, new_network_sessions=0, new_renders=0, GUI_verified=False, whole_goal_complete=False)
save(out / 'binding.json', facts)
(out / 'auditor.py').write_bytes(Path(__file__).read_bytes())
pins = {p.name: sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out / 'evidence.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in pins:
        archive.write(out / name, name)
with zipfile.ZipFile(out / 'evidence.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in pins.items())
save(out / 'archive-readback.json', dict(files=pins, archive_sha256=sha(out / 'evidence.zip'), verified=True))
print(json.dumps(dict(binding=str(out / 'binding.json'), direct_ambient_operands=len(refs['0x8316a0']), dynamic_pixel_consumer_resolved=False, source_modified=False), ensure_ascii=False))
