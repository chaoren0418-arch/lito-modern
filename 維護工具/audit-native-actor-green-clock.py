"""Retain a static callback/elapsed-time discrepancy; never build or execute code."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, struct, zipfile

ROOT = Path(__file__).resolve().parents[1]
ORIGINAL = Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收')
NORMAL = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
digest = lambda b: hashlib.sha256(b).hexdigest().upper()
encode = lambda v: json.dumps(v, ensure_ascii=False, indent=2) + '\n'
pins = read(NORMAL / 'source-inputs.json')
assert len(pins) == 1019
names = ['LiTo381-Modern/src/modern_lito/native_actor_states.rs', 'LiTo381-Modern/src/modern_lito/native_world_scene.rs']
sources = {name: (ROOT / name).read_bytes() for name in names}
for name, content in sources.items():
    assert digest(content) == pins[name], name
actor = sources[names[0]].decode('utf-8-sig')
scene = sources[names[1]].decode('utf-8-sig')
assert '((elapsed_ns / 50_000_000) % 24) as u8' in actor
assert 'Style::Green((12u8.abs_diff(phase) + 16) / 2)' in actor
assert 'actor.draw_state.style(!observer && actor.id == local_id, elapsed_ns)' in scene

pe = (ORIGINAL / 'runtime/LiTo.unpacked-oep-current.exe').read_bytes()
asm = (ORIGINAL / 'ida-client-oep-v2/full-disassembly.asm').read_bytes()
assert digest(pe) == '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert digest(asm) == '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
e = struct.unpack_from('<I', pe, 60)[0]
assert pe[e:e + 4] == b'PE\0\0'
sections = struct.unpack_from('<H', pe, e + 6)[0]
optional_size = struct.unpack_from('<H', pe, e + 20)[0]
base = struct.unpack_from('<I', pe, e + 24 + 28)[0]
assert base == 0x400000

def raw(va, count):
    rv = va - base
    for i in range(sections):
        o = e + 24 + optional_size + 40 * i
        virtual_size, start, size, offset = struct.unpack_from('<IIII', pe, o + 8)
        if start <= rv and rv + count <= start + min(virtual_size, size):
            return pe[offset + rv - start:offset + rv - start + count]
    raise ValueError(hex(va))

callback = raw(0x4878f0, 31)
assert callback.hex().upper() == '8B0D40AC75006A0041BAF0784800890D40AC7500B932000000E8E2EC0400C3'
assert 0x4878f0 + 30 + struct.unpack_from('<i', callback, 26)[0] == 0x4d65f0
assert struct.unpack('<I', raw(0x75ac40, 4))[0] == 0
lines = asm.decode('utf-8-sig').split('\n')  # Preserve original LF numbering despite CRCRLF.
selected = []
for lo, hi in [(269156, 269164), (287831, 287843), (290614, 290635)]:
    selected.extend(f'{i + 1}:{lines[i].rstrip()}' for i in range(lo - 1, hi))
references = [{'original_LF_line': i + 1, 'text': line.rstrip()}
    for i, line in enumerate(lines) if 'dword_75AC40' in line]
assert len(references) == 4

# Conditional scalar comparisons only. Delivered callback counts are supplied
# explicitly; these rows do not assert a complete world-entry/retirement trace.
cases = []
for elapsed, initial, delivered in [(0, 0, 1), (1000, 0, 2), (1200, 0, 2),
        (0, 23, 1), (600, 11, 1), (0, 0xffffffff, 1)]:
    counter = (initial + delivered) & 0xffffffff
    pulse = abs(12 - counter % 24) + 16
    native_step = pulse // 2
    modern_step = (abs(12 - ((elapsed * 1_000_000 // 50_000_000) % 24)) + 16) // 2
    cases.append(dict(elapsed_ms=elapsed, initial_native_counter=initial,
        delivered_callbacks=delivered, resulting_native_counter=counter,
        native_selector_pulse=pulse, native_RGB555_table_step=native_step,
        current_elapsed_table_step=modern_step, differs=native_step != modern_step))
assert sum(c['differs'] for c in cases) == 3
out = ROOT / '驗證紀錄/current/native-actor-green-clock' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True, exist_ok=False)
(out / 'observer.py').write_bytes(Path(__file__).read_bytes())
(out / 'callback-original.bin').write_bytes(callback)
(out / 'selected-assembly.txt').write_text('\n'.join(selected) + '\n', encoding='utf-8', newline='\n')
(out / 'phase-references.json').write_text(encode(references), encoding='utf-8', newline='\n')
(out / 'conditional-counterexamples.json').write_text(encode(cases), encoding='utf-8', newline='\n')
for name, content in sources.items():
    (out / Path(name).name).write_bytes(content)
report = dict(status='static_original_green_callback_and_current_elapsed_discrepancy_retained',
    path=str(out), source_files=1019, source_inputs={name: digest(content) for name, content in sources.items()},
    original_PE_sha256=digest(pe), original_ASM_sha256=digest(asm), callback_original_bytes=31,
    callback_increments_before_schedule=True, schedule_delay_ms=50, raw_initial_counter=0,
    counter_is_wrapping_u32=True, selector_modulo=24, conditional_cases=6, differing_cases=3,
    product_source_modified=False, original_callback_executed=False,
    complete_clock_lifecycle_verified=False, indirect_counter_writes_ruled_out=False,
    full_framebuffer_oracle_verified=False, game_sessions=0, whole_goal_complete=False)
(out / 'observation.json').write_text(encode(report), encoding='utf-8', newline='\n')
files = {p.name: p for p in out.iterdir() if p.is_file()}
hashes = {name: digest(p.read_bytes()) for name, p in files.items()}
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for name, p in files.items():
        z.write(p, name)
with zipfile.ZipFile(out / 'raw.zip') as z:
    assert set(z.namelist()) == set(hashes)
    for name, h in hashes.items():
        assert digest(z.read(name)) == h, name
(out / 'archive-readback.json').write_text(encode(dict(files=hashes,
    raw_archive_sha256=digest((out / 'raw.zip').read_bytes()), verified=True)), encoding='utf-8', newline='\n')
print(encode(report))
