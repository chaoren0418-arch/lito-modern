"""Compare original x86 mask code with an independent clipped max compositor."""
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import zipfile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_CALL, CS_GRP_JUMP
from capstone.x86 import X86_OP_MEM, X86_OP_IMM

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
OUT = ROOT / '驗證紀錄/current/native-lighting-mask-compositor'
RUNTIME = PROJECT / 'target/current/native-lighting-mask-oracle'
CPP = ROOT / '維護工具/oracle/NativeLightingMaskOracle.cpp'
PE = Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe')
LMP = ROOT / '驗證紀錄/current/native-lighting-source-lmp/source.lmp'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
assert sha(PE) == '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(LMP) == '30CF051FE516A053AC494B8054E5A320BAB1F3F3DD7B0010E0CF0C74AACCE5AA'
completion = read(ROOT / '驗證紀錄/current/native-light-spell/current-regression/current14-completion-observation.json')
assert completion['scenarios'] == 14 and not completion['owned_GUI_or_test_processes_observed']
# Recheck actual live state, rather than relying solely on the previous receipt.
ps = subprocess.run(['powershell.exe', '-NoProfile', '-NonInteractive', '-Command',
    '[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); Get-CimInstance Win32_Process | Select-Object Name,ExecutablePath,CommandLine | ConvertTo-Json -Compress'],
    capture_output=True, check=True)
for p in json.loads(ps.stdout.decode('utf-8-sig')):
    exe, command = (p.get('ExecutablePath') or '').casefold(), (p.get('CommandLine') or '').casefold()
    if str(ROOT).casefold() in exe or (p['Name'].casefold() in ('java.exe','cargo.exe','rustc.exe') and str(ROOT).casefold() in command):
        assert not p['Name'].casefold().startswith(('litomodern','qualified-client','native_','java','cargo','rustc')), p
OUT.mkdir(exist_ok=False)
RUNTIME.mkdir(exist_ok=True)
raw = PE.read_bytes()
eh = struct.unpack_from('<I', raw, 60)[0]
opt = eh + 24
opt_size = struct.unpack_from('<H', raw, eh + 20)[0]
base = struct.unpack_from('<I', raw, opt + 28)[0]
sections = [struct.unpack_from('<IIII', raw, opt + opt_size + 40*i + 8)
    for i in range(struct.unpack_from('<H', raw, eh + 6)[0])]
def at(address, count):
    for vs, va, rs, rp in sections:
        rva = address - base
        if va <= rva and rva + count <= va + rs:
            return raw[rp+rva-va:rp+rva-va+count]
    raise AssertionError(hex(address))

decoder = Cs(CS_ARCH_X86, CS_MODE_32)
decoder.detail = True
extracted = {}
for name, address, size in [('blend', 0x4bb170, 184), ('clear', 0x4bb230, 19)]:
    code = at(address, size)
    instructions = list(decoder.disasm(code, address))
    assert sum(i.size for i in instructions) == size
    assert instructions[-1].mnemonic == 'ret' and not instructions[-1].op_str
    assert not any(i.group(CS_GRP_CALL) for i in instructions)
    for i in instructions:
        if i.group(CS_GRP_JUMP):
            assert i.operands[0].type == X86_OP_IMM
            assert address <= i.operands[0].imm < address + size
        for operand in i.operands:
            if operand.type == X86_OP_MEM:
                assert operand.mem.base or operand.mem.index, 'No original global data references expected'
    (OUT / (name + '-original.bin')).write_bytes(code)
    extracted[name] = {'address': hex(address), 'bytes': size, 'relocations': 0,
        'instructions': [{'address': hex(i.address), 'bytes': bytes(i.bytes).hex(), 'mnemonic': i.mnemonic, 'operands': i.op_str} for i in instructions]}
(OUT / 'extracted-code.json').write_text(json.dumps(extracted, indent=2)+'\n', encoding='utf-8')

lmp = LMP.read_bytes()
slots = []
for i in range(16):
    offset = struct.unpack_from('<I', lmp, i * 4)[0]
    w,h = struct.unpack_from('<ii', lmp, 64 + i * 8)
    assert 0 <= w <= 320 and 0 <= h <= 204 and offset + w*h <= len(lmp)
    slots.append((w,h,lmp[offset:offset+w*h]))
cases = []
positions = [(0,0),(1,1),(37,22),(160,102),(319,203),(320,204),(-1,-1),(-7,3),
    (13,-9),(-400,-300),(1000,1000),(315,-3),(-3,199),(159,203),(319,101)]
for slot in range(16):
    w,h,_ = slots[slot]
    for x,y in positions + [(-w,-h),(-w+1,-h+1),(320-w,204-h)]:
        cases.append({'seed': slot*11, 'clear': True, 'ops': [{'slot':slot,'x':x,'y':y}]})
for seed in range(16):
    ops = [{'slot': (seed+j)%15, 'x': 117-j*7, 'y': 85-j*3} for j in range(16)]
    cases.append({'seed':seed,'clear':False,'ops':ops})
    cases.append({'seed':seed,'clear':True,'ops':list(reversed(ops))})
cases.extend([
    {'seed':0,'clear':False,'ops':[{'kind':'pairs','x':0,'y':0,'w':320,'h':204}]},
    {'seed':0,'clear':False,'ops':[{'kind':'constant','value':255,'x':0,'y':0,'w':320,'h':204}]},
])
for w,h in [(0,0),(0,7),(7,0),(-1,0),(0,-1),(-1,-1),(12,-1),(-1,12)]:
    cases.append({'seed':3,'clear':False,'ops':[{'kind':'constant','value':121,'x':0,'y':0,'w':w,'h':h}]})

def source(op):
    if 'slot' in op:
        return slots[op['slot']]
    w,h = op['w'],op['h']
    n = max(0,w)*max(0,h)
    data = bytes(i//256 for i in range(n)) if op['kind']=='pairs' else bytes([op['value']])*n
    return w,h,data

stream = bytearray(struct.pack('<I', len(cases)))
for c in cases:
    stream.extend(struct.pack('<III',c['seed'],int(c['clear']),len(c['ops'])))
    for op in c['ops']:
        w,h,data = source(op)
        stream.extend(struct.pack('<iiiiI',op['x'],op['y'],w,h,len(data)))
        stream.extend(data)
(OUT / 'commands.bin').write_bytes(stream)
args = read(ROOT / '驗證紀錄/current/native-lighting-source/build-command.json')['args']
args = [arg.replace('NativeLightingPrimitiveOracle.cpp', CPP.name).replace('native-lighting-source-oracle', RUNTIME.name) for arg in args]
env = os.environ.copy()
env.update(PYTHONDONTWRITEBYTECODE='1',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
def execute(args, name):
    result = subprocess.run(args, cwd=RUNTIME, env=env, capture_output=True, timeout=50, creationflags=subprocess.CREATE_NO_WINDOW)
    (OUT / name).write_bytes(result.stdout)
    (OUT / (name+'.stderr')).write_bytes(result.stderr)
    assert result.returncode == 0, (name,result.returncode)
    return result
(OUT/'build-command.json').write_text(json.dumps({'args':args,'cwd':str(RUNTIME)},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
execute(args, 'build.log')
helper = RUNTIME / 'oracle.exe'
native = json.loads(execute([str(helper),str(OUT/'blend-original.bin'),str(OUT/'clear-original.bin'),str(OUT/'commands.bin'),str(OUT/'actual-masks.bin')],'native.json').stdout.decode('utf-8'))
actual = (OUT/'actual-masks.bin').read_bytes()
assert native['cases'] == len(cases) and len(actual) == len(cases)*320*204
for i,c in enumerate(cases):
    expected = bytearray(320*204) if c['clear'] else bytearray((j*17+c['seed'])&255 for j in range(320*204))
    for op in c['ops']:
        w,h,data = source(op)
        x,y = op['x'],op['y']
        for yy in range(max(0,y), min(204,y+h)):
            for xx in range(max(0,x), min(320,x+w)):
                d=yy*320+xx
                expected[d] = max(expected[d], data[(yy-y)*w+xx-x])
    observed = actual[i*320*204:(i+1)*320*204]
    assert observed == expected, i
    c['original_native_sha256'] = hashlib.sha256(observed).hexdigest().upper()
(OUT/'vectors.json').write_text(json.dumps({'mask_size':[320,204], 'source_lmp_sha256':sha(LMP), 'cases':cases},indent=2)+'\n',encoding='utf-8')
(OUT/'source.lmp').write_bytes(lmp)
(OUT/'oracle.cpp').write_bytes(CPP.read_bytes())
(OUT/'oracle.exe').write_bytes(helper.read_bytes())
(OUT/'audit.py').write_bytes(Path(__file__).read_bytes())
proof = {'status':'original_x86_mask_clear_and_clipped_unsigned_max_qualified',
    'source_PE_sha256':sha(PE),'source_lmp_sha256':sha(LMP),'original_code_relocations':0,
    **native,'independent_python_mask_bytes_checked':len(actual),
    'exhaustive_unsigned_byte_pairs':65536,'vectors_sha256':sha(OUT/'vectors.json'),
    'full_mask_producer_projection_and_selection_verified':False,'full_world_framebuffer_verified':False,
    'product_integration_verified':False,'game_sessions':0,'whole_goal_complete':False}
pins={p.name:sha(p) for p in OUT.iterdir() if p.is_file()}
archive=OUT/'mask-compositor-evidence.zip'
with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name in pins:
        z.write(OUT/name,name)
with zipfile.ZipFile(archive) as z:
    assert z.testzip() is None
    for name,value in pins.items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper()==value
proof.update(archive_sha256=sha(archive),archive_files=pins,archive_readback_verified=True)
(OUT/'qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:v for k,v in proof.items() if k not in ('archive_files',)},ensure_ascii=False))
