"""Prepare ten private x86 fragments; never build, run, or alter client inputs."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, struct, zipfile
import capstone

ROOT = Path(__file__).resolve().parents[1]
NORMAL = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified'
OLD = ROOT / '驗證紀錄/current/native-water-clock-original/20261016-074852-110050'
STATIC = ROOT / '驗證紀錄/current/native-actor-green-clock/20261016-144529-431115'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
digest = lambda b: hashlib.sha256(b).hexdigest().upper()
encode = lambda v: json.dumps(v, ensure_ascii=False, indent=2) + '\n'
pins = read(NORMAL / 'source-inputs.json')
assert len(pins) == 1019
for name, h in pins.items():
    assert digest((ROOT / name).read_bytes()) == h, name
old_q = read(OLD / 'qualified.json')
old_a = read(OLD / 'archive-readback.json')
assert old_q['status'] == 'original_clock_fragments_independently_verified'
assert digest((OLD / 'raw.zip').read_bytes()) == old_q['archive_sha256'] == old_a['archive_sha256']
with zipfile.ZipFile(OLD / 'raw.zip') as z:
    for name, h in old_a['files'].items():
        assert digest(z.read(name)) == h, name
    base = z.read('oracle.cpp').decode('utf-8-sig').replace('\r\n', '\n')
    functions = z.read('functions.tsv').decode('utf-8-sig').replace('\r\n', '\n')
    relocations = z.read('relocations.tsv').decode('utf-8-sig').replace('\r\n', '\n')
    original_files = {row.split()[0] + '-original.bin': z.read(row.split()[0] + '-original.bin')
        for row in functions.splitlines() if row.strip()}
    original_files['keep-original.bin'] = z.read('keep-original.bin')
assert len(original_files) == 9
callback = (STATIC / 'callback-original.bin').read_bytes()
assert len(callback) == 31 and callback.hex().upper() == '8B0D40AC75006A0041BAF0784800890D40AC7500B932000000E8E2EC0400C3'
assert read(STATIC / 'observation.json')['product_source_modified'] is False
assert (0x4878f0).to_bytes(4, 'little') not in original_files['keep-original.bin']
prefix = base.split('int wmain(', 1)[0]

def replace_once(old, new):
    global prefix
    assert prefix.count(old) == 1, old
    prefix = prefix.replace(old, new, 1)

replace_once(',move_slot;', ',move_slot,green_counter;')
replace_once('std::array<Slot*,11>', 'std::array<Slot*,12>')
replace_once('&move_slot};', '&move_slot,&green_counter};')
replace_once('f==address(code.at("phase"))||', 'f==address(code.at("green"))||f==address(code.at("phase"))||')
replace_once('    if(callback==address(code.at("phase")))return "phase";',
    '    if(callback==address(code.at("green")))return "green";\n    if(callback==address(code.at("phase")))return "phase";')
replace_once('std::map<U,U> targets{{0x64bc2d,',
    'std::map<U,U> targets{{0x75ac40,address(&green_counter.value)},{0x64bc2d,')
replace_once('require(code.size()==8,"Fragment inventory mismatch")',
    'require(code.size()==10,"Fragment inventory mismatch")')
cases_path = ROOT / '維護工具/oracle/NativeActorGreenClockCases.cpp.inc'
cases = cases_path.read_text(encoding='utf-8-sig')
assert 'scalar_vectors' in cases and 'queue_snapshots' in cases
functions = functions.rstrip() + '\ngreen 4878f0 31\nscene_cleanup 4d6470 122\n'
relocations = relocations.rstrip() + '\n' + '\n'.join([
    'green absolute 2 75ac40 0', 'green absolute 10 4878f0 0',
    'green absolute 16 75ac40 0', 'green relative 26 4d65f0 30']) + '\n'
original_files['green-original.bin'] = callback
pe = Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe').read_bytes()
assert digest(pe) == '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
e = struct.unpack_from('<I', pe, 60)[0]
o = e + 24
base = struct.unpack_from('<I', pe, o + 28)[0]
cleanup = None
for i in range(struct.unpack_from('<H', pe, e + 6)[0]):
    _, va, size, offset = struct.unpack_from('<IIII', pe, o + struct.unpack_from('<H', pe, e + 20)[0] + 40 * i + 8)
    if va <= 0x4d6470 - base and 0x4d64ea - base <= va + size:
        cleanup = pe[offset + 0x4d6470 - base - va:offset + 0x4d64ea - base - va]
assert cleanup is not None and len(cleanup) == 122
cs = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
cs.detail = True
instructions = list(cs.disasm(cleanup, 0x4d6470))
assert sum(i.size for i in instructions) == 122 and instructions[-1].mnemonic == 'ret'
allowed = {0x834c00, 0x834c08, 0x6365e0, 0x6365fc, 0x77409100}
cleanup_relocations = []
for instruction in instructions:
    for operand in instruction.operands:
        if operand.type == capstone.x86.X86_OP_MEM and operand.mem.base == 0 and operand.mem.index == 0 and operand.mem.disp >= 0x400000:
            target = operand.mem.disp
            assert target in allowed and instruction.disp_size == 4
            cleanup_relocations.append(('absolute', instruction.address - 0x4d6470 + instruction.disp_offset, target, 0))
        elif operand.type == capstone.x86.X86_OP_IMM and operand.imm >= 0x400000:
            target = operand.imm
            if instruction.mnemonic.startswith('j'):
                assert 0x4d6470 <= target < 0x4d64ea
                continue
            assert target in allowed and instruction.imm_size == 4
            kind = 'relative' if instruction.mnemonic == 'call' else 'absolute'
            cleanup_relocations.append((kind, instruction.address - 0x4d6470 + instruction.imm_offset, target,
                instruction.address - 0x4d6470 + instruction.size if kind == 'relative' else 0))
assert len(cleanup_relocations) == 8
relocations += ''.join(f'scene_cleanup {kind} {offset} {target:x} {next_offset}\n'
    for kind, offset, target, next_offset in cleanup_relocations)
original_files['scene_cleanup-original.bin'] = cleanup
out = ROOT / '驗證紀錄/current/native-actor-green-clock-original' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True, exist_ok=False)
for name, content in original_files.items():
    (out / name).write_bytes(content)
(out / 'oracle.cpp').write_text(prefix + cases, encoding='utf-8', newline='\n')
(out / 'functions.tsv').write_text(functions, encoding='utf-8', newline='\n')
(out / 'relocations.tsv').write_text(relocations, encoding='utf-8', newline='\n')
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
(out / 'cases.cpp.inc').write_bytes(cases_path.read_bytes())
(out / 'source-inputs.json').write_bytes((NORMAL / 'source-inputs.json').read_bytes())
report = dict(status='prepared_original_green_callback_and_queue_not_built_or_executed',
    path=str(out), source_files=1019, source_manifest_sha256=digest((NORMAL / 'source-inputs.json').read_bytes()),
    original_functions=10, original_function_bytes=sum(len(v) for k, v in original_files.items() if k != 'keep-original.bin'),
    scalar_vectors_expected=532, queue_snapshots_expected=16,
    original_queue_reference=str(OLD / 'qualified.json'), original_queue_reference_sha256=digest((OLD / 'qualified.json').read_bytes()),
    original_green_static_reference=str(STATIC / 'observation.json'),
    product_source_modified=False, helper_built=False, original_callback_executed=False,
    whole_client_or_all_entry_paths_verified=False, whole_goal_complete=False)
(out / 'prepared.json').write_text(encode(report), encoding='utf-8', newline='\n')
files = {p.name: p for p in out.iterdir() if p.is_file()}
hashes = {name: digest(p.read_bytes()) for name, p in files.items()}
with zipfile.ZipFile(out / 'prepared-inputs.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for name, p in files.items():
        z.write(p, name)
with zipfile.ZipFile(out / 'prepared-inputs.zip') as z:
    assert set(z.namelist()) == set(hashes)
    for name, h in hashes.items():
        assert digest(z.read(name)) == h, name
(out / 'preparation-readback.json').write_text(encode(dict(files=hashes,
    archive_sha256=digest((out / 'prepared-inputs.zip').read_bytes()), verified=True)), encoding='utf-8', newline='\n')
print(encode(report))
