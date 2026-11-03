"""Bind type1/1811 wire anchor and bounded lifecycle; never execute the old PE."""
from pathlib import Path
from datetime import datetime, timezone
import capstone, hashlib, json, struct, zipfile

ROOT = Path(__file__).resolve().parents[1]
V = ROOT / '驗證紀錄/current'

def read(path):
    return json.loads(Path(path).read_text('utf-8-sig'))

def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

def save(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

normal = V / 'native-retired-join-normal/validation-20261018-063154-317211'
inputs = read(normal / 'compiled-inputs.json')
assert len(inputs) == 1041
for name, digest in inputs.items():
    assert sha(ROOT / name) == digest
parent = V / 'native-eruption-wire-binding/20261018-081317-270930/binding.json'
binding = read(parent)
assets = V / 'native-sunburst-assets/load-20261018-084907-363031/qualified-20261018-084958-694248/qualified.json'
assetq = read(assets)
assert assetq['current_TW_typed_parser_executed'] and assetq['SPR_frames'] == 19
assert assetq['source_manifest_sha256'] == read(normal / 'cohort.json')['source_manifest_sha256']
pe = Path(binding['primary_PE']['path']).read_bytes()
assert hashlib.sha256(pe).hexdigest().upper() == binding['primary_PE']['sha256']
offset = struct.unpack_from('<I', pe, 60)[0]
count = struct.unpack_from('<H', pe, offset + 6)[0]
sections = offset + 24 + struct.unpack_from('<H', pe, offset + 20)[0]

def raw(address, length):
    for index in range(count):
        size, start, physical, at = struct.unpack_from('<IIII', pe, sections + index * 40 + 8)
        if start <= address - 0x400000 and address - 0x400000 + length <= start + min(size, physical):
            return pe[at + address - 0x400000 - start:at + address - 0x400000 - start + length]
    raise ValueError(hex(address))

decoder = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
decoder.detail = True
ranges = {'wire_tail': (0x4a5f4d, 0x4a5fe0), 'factory': (0x4a6234, 0x4a6288),
          'constructor': (0x44ede0, 0x44eeb3), 'entry': (0x44f350, 0x450032),
          'advance': (0x44ef80, 0x44f322), 'timer': (0x44f330, 0x44f341)}
decoded = {name: list(decoder.disasm(raw(lo, hi-lo), lo)) for name, (lo, hi) in ranges.items()}
ins = {i.address: i for rows in decoded.values() for i in rows}
expected = {
    0x4a5f9a: ('lea', 'ecx, [eax + eax - 0x8000]'),
    0x4a5fa4: ('lea', 'ebx, [edx + edx - 0x8000]'),
    0x4a5fcf: ('dec', 'eax'), 0x4a5fd9: ('jmp', 'dword ptr [eax*4 + 0x4a63c4]'),
    0x4a6261: ('push', 'esi'), 0x4a6268: ('push', 'ebx'),
    0x4a6281: ('call', '0x44ede0'),
    0x44ee09: ('mov', 'ecx, dword ptr [ebp + 0x1c]'),
    0x44ee0f: ('mov', 'eax, dword ptr [ebp + 0x18]'),
    0x44ee18: ('mov', 'dword ptr [esi + 0x2c], eax'),
    0x44ee1b: ('mov', 'dword ptr [esi + 0x30], ecx'),
    0x44ee1e: ('mov', 'dword ptr [esi + 0x3c], eax'),
    0x44ee24: ('mov', 'dword ptr [esi + 0x40], ecx'),
    0x44ee2f: ('mov', 'cl, byte ptr [ebp + 0x14]'),
    0x44ee4e: ('mov', 'byte ptr [esi + 0x12], cl'),
    0x44ee5b: ('mov', 'edi, dword ptr [ebp + 0x20]'),
    0x44ee84: ('call', '0x4ea8b0'), 0x44ee89: ('mov', 'dword ptr [esi + 0x68], eax'),
    0x45001c: ('call', '0x4e78b0'), 0x450023: ('call', '0x44ef80'),
    0x44efb6: ('call', '0x4e5b50'),
    0x44efbf: ('mov', 'byte ptr [esi + 0x13c], 1'),
    0x44f0df: ('mov', 'ecx, dword ptr [ebp - 0x14]'),
    0x44f0e3: ('mov', 'edx, 0x44f330'), 0x44f0e8: ('call', '0x4d65f0'),
    0x44f17b: ('mov', 'al, byte ptr [esi + 0x12]'),
    0x44f238: ('test', 'byte ptr [ecx + eax*4 + 0x12], 1'),
    0x44f249: ('call', '0x4e8160'),
    0x44f333: ('call', '0x4e01a0'), 0x44f33a: ('call', '0x44ef80')}
for address, operation in expected.items():
    assert (ins[address].mnemonic, ins[address].op_str) == operation, hex(address)
assert struct.unpack('<I', raw(0x4a63c4, 4))[0] == 0x4a6234
assert struct.unpack('<I', raw(0x4f84c4, 4))[0] == 0x44f350

# Follow the known graphics-only selector. Fail if an unmodelled operation appears.
pc, equal, compared, selected = 0x44f374, None, [], []
while pc != 0x45001a:
    i = ins[pc]
    selected.append(dict(address=f'0x{pc:08X}', mnemonic=i.mnemonic, operands=i.op_str))
    if i.mnemonic == 'cmp' and i.op_str.startswith('ax, '):
        value = int(i.op_str.split(', ')[1], 0)
        compared.append(value)
        equal = 1811 == value
        pc += i.size
    elif i.mnemonic in ('je', 'jne'):
        assert equal is not None
        take = equal if i.mnemonic == 'je' else not equal
        pc = int(i.op_str, 0) if take else pc + i.size
    else:
        raise AssertionError((hex(pc), i.mnemonic, i.op_str))
assert compared == [2483, 2484, 758, 1819, 757, 2261]

# No local writes to the anchor in ordinary advancement/timer, including partial stores.
anchor_writes = []
for name in ('advance', 'timer'):
    for i in decoded[name]:
        for op in i.operands:
            if op.type == capstone.CS_OP_MEM and op.access & capstone.CS_AC_WRITE:
                if op.mem.base == capstone.x86.X86_REG_ESI and 0x2c <= op.mem.disp < 0x34:
                    anchor_writes.append((hex(i.address), i.op_str))
assert not anchor_writes
out = V / 'native-sunburst-anchor-binding' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
for name, (lo, hi) in ranges.items():
    (out / (name+'.bin')).write_bytes(raw(lo, hi-lo))
    (out / (name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+'\n', encoding='utf-8')
record = dict(status='primary_type1_1811_wire_destination_anchor_and_bounded_lifecycle_bound_only',
    source_manifest_sha256=assetq['source_manifest_sha256'], primary_PE=binding['primary_PE'],
    parent_wire_binding=dict(path=str(parent), sha256=sha(parent)),
    current_TW_assets=dict(path=str(assets), sha256=sha(assets)), graphics=1811, effect_type=1,
    selector=selected, factory_branch='0x4A6234', constructor='0x44EDE0', entry='0x44F350',
    anchor='S23 destination native X=2*u16(to.x)-32768; Y=u16(to.y), copied at construction',
    findings=[
        'Current typed1811 declares effect type1. Factory table slot0 calls44EDE0 with destination EBX/ESI, unlike the out-of-range default branch which uses source coordinates.',
        'The1811 entry selector reaches45001A directly, registers then advances. No special direction offset or current-target position lookup is selected.',
        'The constructor copies these coordinates into2C/30 and3C/40. Advance/timer have no direct writes to2C/30; shared/transitive callees remain separately scoped.',
        'The timer unlinks through4E01A0 then advances. Source wrap sets13C but schedules the returned source dwell before the following callback retires.',
        'Byte12 is controlled by wire flight_type, not by effect type1. Actual formal skill46 flight_type remains to be captured; no ambient-off claim is granted.',
        'A nonzero target argument creates a target list; the selected Hit source bit gates a current-actor lookup and guarded hurt dispatch. Unlike null-target cold impact, Sunburst cannot be assumed to have no hurt.',
        'Current typed1811 source11 has Hit; this is a target-event association, not runtime permission to alter HP.'],
    local_anchor_writes=anchor_writes, transitive_anchor_mutation_fully_audited=False,
    external_scheduler_time_units_qualified=False, complete_hurt_guards_qualified=False,
    actual_formal_flight_type_captured=False, renderer_qualified=False, normal_product_source_modified=False,
    corresponding_frozen_QA_modified=False, request_registered=False, new_sessions=0, new_casts=0,
    new_renders=0, GUI_verified=False, whole_goal_complete=False)
save(out / 'binding.json', record)
(out / 'auditor.py').write_bytes(Path(__file__).read_bytes())
pins = {p.name: sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out / 'evidence.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for name in pins:
        z.write(out / name, name)
with zipfile.ZipFile(out / 'evidence.zip') as z:
    assert all(hashlib.sha256(z.read(name)).hexdigest().upper() == digest for name, digest in pins.items())
save(out / 'readback.json', dict(files=pins, archive_sha256=sha(out / 'evidence.zip')))
for name, digest in inputs.items():
    assert sha(ROOT / name) == digest
print(json.dumps(dict(binding=str(out / 'binding.json'), selector_comparisons=len(compared), renderer_qualified=False), ensure_ascii=False))
