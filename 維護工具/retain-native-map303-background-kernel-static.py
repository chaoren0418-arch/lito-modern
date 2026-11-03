"""Retain original background dispatch/clock/caller evidence without executing it."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re, zipfile

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
RUN = APP/'floor-tests-20261016-102632-227125'
ASM = Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
digest = lambda b: hashlib.sha256(b).hexdigest().upper()
save = lambda p,v: p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins = read(RUN/'compiled-inputs.json')
assert len(pins) == 1005
for n,h in pins.items():
    assert digest((ROOT/n).read_bytes()) == h,n
raw = ASM.read_bytes()
assert digest(raw) == '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
# Preserve authoritative LF indexing; the original file has CRCRLF sequences.
lines = raw.decode('utf-8',errors='replace').split('\n')
names = ['sub_4878C0','sub_4878F0','sub_487E10','sub_488860','sub_4C18E0','sub_4BF0B0','sub_4BF500']
selected = {}
for name in names:
    a = next(i for i,l in enumerate(lines) if l.startswith(name+' proc near'))
    b = next(i for i in range(a+1,len(lines)) if lines[i].startswith(name+' endp'))
    selected[name] = {'first_newline_line':a+1,'last_newline_line':b+1,'assembly':'\n'.join(lines[a:b+1])}
init = selected['sub_4C18E0']['assembly']
nonzero,zero = init.split('loc_4C1FBC:',1)
pattern = r'mov\s+dword_([0-9A-F]+), offset (sub_[0-9A-F]+)'
assert dict(re.findall(pattern,nonzero))['768878'] == 'sub_4BF0B0'
assert dict(re.findall(pattern,zero))['768878'] == 'sub_4BF500'
assert 'mov     al, byte_641298' in init and 'jz      loc_4C1FBC' in init
for name in ['sub_4BF0B0','sub_4BF500']:
    body = selected[name]['assembly']
    gate_register = 'ecx' if name == 'sub_4BF0B0' else 'edx'
    assert 'dword_6476C4' in body and f'cmp     byte ptr [{gate_register}], 0' in body
    assert 'jnz     loc_' in body and 'retn    20h' in body
    assert 'word_82B09C' in body and 'word_81C09C' in body
cloud = selected['sub_4878C0']['assembly']
assert all(s in cloud for s in ['inc     edx','inc     ecx','mov     ecx, 40h','call    sub_4D65F0'])
counter = selected['sub_4878F0']['assembly']
assert all(s in counter for s in ['inc     ecx','dword_75AC40','mov     ecx, 32h','call    sub_4D65F0'])
background = selected['sub_487E10']['assembly']
assert all(s in background for s in ['call    dword_768878','cmp     dword_62F324, 12Fh','push    0','push    1','dword_62F4C0','dword_62F4C4'])
world = selected['sub_488860']['assembly']
sites = []
for i,line in enumerate(world.split('\n')):
    if 'call    sub_487E10' in line:
        w = world.split('\n')
        sites.append({'function_relative_LF_line':i+1,'assembly':'\n'.join(w[max(0,i-25):i+26])})
assert len(sites) == 1
globals_ = [{'newline_line':i+1,'assembly':line} for i,line in enumerate(lines)
            if re.match(r'dword_(62F4C0|62F4C4|75AC40) dd ',line)]
assert len(globals_) == 3
out = ROOT/'驗證紀錄/current/native-map303-background-kernel-static'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
save(out/'selected-assembly.json',selected)
save(out/'world-call-sites.json',sites)
save(out/'initial-global-declarations.json',globals_)
proof = {'status':'background_kernel_dispatch_and_callback_dataflow_retained_static_only',
    'original_ASM_sha256':digest(raw),'compiled_source_files_unchanged':1005,
    'source_manifest_sha256':digest((RUN/'compiled-inputs.json').read_bytes()),
    'dispatch_slot':'dword_768878','RGB565_nonzero_flag_kernel':'sub_4BF0B0',
    'RGB555_zero_flag_kernel':'sub_4BF500','mode_flag':'byte_641298',
    'gate_observation':'Both readers skip nonzero terrain gate bytes before background pixel writes.',
    'cloud_offset_callback':{'function':'sub_4878C0','delay_ms':64,'effect':'increments both raw 32-bit offset globals then queues itself'},
    'additional_counter_callback':{'function':'sub_4878F0','delay_ms':50,'effect':'increments dword_75AC40 then queues itself'},
    'initial_global_values':'Retained verbatim; no signed16 reinterpretation or reset assumption.',
    'blend_tables_scope':'Original table addresses and initializer retained; full FPU/table initializer and background blend not executed.',
    'background_native_execution_verified':False,'complete_background_geometry_verified':False,
    'product_background_renderer_complete':False,'new_game_sessions':0,'whole_goal_complete':False}
save(out/'audit.json',proof)
files = {n:out/n for n in ['selected-assembly.json','world-call-sites.json','initial-global-declarations.json','audit.json']}
files['auditor.py'] = Path(__file__)
hashes = {n:digest(p.read_bytes()) for n,p in files.items()}
with zipfile.ZipFile(out/'raw-static-evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw-static-evidence.zip') as z:
    assert set(z.namelist()) == set(files)
    for n,h in hashes.items():assert digest(z.read(n)) == h
result = {**proof,'path':str(out),'files':hashes,'archive_sha256':digest((out/'raw-static-evidence.zip').read_bytes()),'archive_readback_verified':True}
save(out/'retained-static.json',result)
save(APP/'map303-background-kernel-static-current.json',result)
print(json.dumps({'path':str(out),'status':result['status'],'archive_readback_verified':True},ensure_ascii=False))
