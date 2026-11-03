"""Check original channel-entry arithmetic and original remap tables/selector."""
import base64
import csv
import hashlib
import itertools
import json
import os
from pathlib import Path
import struct
import subprocess
import zipfile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_CALL, CS_GRP_JUMP
from capstone.x86 import X86_OP_MEM, X86_REG_EBP, X86_REG_ECX, X86_REG_EAX

ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
BASE=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收')
OUT=ROOT/'驗證紀錄/current/native-lighting-original-tables'
RUNTIME=PROJECT/'target/current/native-lighting-table-oracle'
CPP=ROOT/'維護工具/oracle/NativeLightingTableOracle.cpp'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
env=os.environ.copy()
env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1')

def no_gui():
    script="$p=Get-CimInstance Win32_Process|Where-Object {$_.Name -like 'LiToModern*' -and $_.ExecutablePath -and $_.ExecutablePath.StartsWith('"+str(ROOT)+"',[StringComparison]::OrdinalIgnoreCase)};if($p){exit 1};$q=Get-Content -LiteralPath '"+str(ROOT/'驗證紀錄/current/native-light-spell/current-regression/TW14-after-default.json')+"' -Raw|ConvertFrom-Json;if($q.status -ne 'waiting_on_verified_current_default_parent'){exit 2}"
    command=['powershell.exe','-NoProfile','-EncodedCommand',base64.b64encode(script.encode('utf-16-le')).decode('ascii')]
    result=subprocess.run(command,cwd=PROJECT,env=env,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=15)
    assert result.returncode==0,'Defer independent table oracle until current GUI cohort finishes'

no_gui()
pe=BASE/'runtime/LiTo.unpacked-oep-current.exe'
assert sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
normal_path=ROOT/'驗證紀錄/current/native-light-spell/normal/qualified.json'
normal=read(normal_path);pins=read(normal_path.parent/'source-inputs.json');assert len(pins)==970
for name,expected in pins.items():assert sha(ROOT/name)==expected,name
raw=pe.read_bytes();eh=struct.unpack_from('<I',raw,60)[0];optional=eh+24;size=struct.unpack_from('<H',raw,eh+20)[0]
base=struct.unpack_from('<I',raw,optional+28)[0]
sections=[struct.unpack_from('<IIII',raw,optional+size+40*i+8) for i in range(struct.unpack_from('<H',raw,eh+6)[0])]
def read_va(address,count):
    rva=address-base
    for vs,va,rs,rp in sections:
        if va<=rva and rva+count<=va+rs:return raw[rp+rva-va:rp+rva-va+count]
    raise AssertionError(hex(address))
metadata={f['name']:f for f in read(BASE/'ida-client-oep-v2/functions.json')}
init=metadata['sub_4C18E0'];decoder=Cs(CS_ARCH_X86,CS_MODE_32);decoder.detail=True
instructions=list(decoder.disasm(read_va(init['address'],init['size']),init['address']))
begin=None
for index,instruction in enumerate(instructions[:-1]):
    operands=instruction.operands
    if instruction.mnemonic=='mov' and len(operands)==2 and operands[0].reg==X86_REG_ECX and operands[1].type==X86_OP_MEM and operands[1].mem.base==X86_REG_EBP and operands[1].mem.disp==-0x1c:
        following=instructions[index+1]
        if following.mnemonic=='mov' and following.operands[0].reg==X86_REG_EAX and following.operands[1].imm==0x84210843:
            begin=index;break
assert begin is not None
end=next(i for i in range(begin,len(instructions)) if instructions[i].mnemonic=='mov' and instructions[i].operands[0].type==X86_OP_MEM and instructions[i].operands[0].mem.disp==0x80e09c)
entry_instructions=instructions[begin:end+1]
assert not any(i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP) for i in entry_instructions)
entry_start=entry_instructions[0].address
entry=read_va(entry_start,sum(i.size for i in entry_instructions))
selector_meta=metadata['sub_4BD150'];selector=read_va(selector_meta['address'],selector_meta['size'])
assert not any(i.group(CS_GRP_CALL) for i in decoder.disasm(selector,selector_meta['address']))
fragments={}
for name,code,addresses in [('entry',entry,[(0x81109c,1),(0x80d89c,1),(0x80e09c,1),(0x80e89c,1)]),('selector',selector,[(0x6348a4,2),(0x6348a8,1),(0x634aa8,1),(0x83009c,2)])]:
    relocations=[]
    for address,count in addresses:
        positions=[i for i in range(len(code)-3) if code[i:i+4]==struct.pack('<I',address)]
        assert len(positions)==count,(name,hex(address))
        relocations.extend({'offset':i,'original_address':hex(address)} for i in positions)
    fragments[name]={'bytes':len(code),'start':hex(entry_start if name=='entry' else selector_meta['address']),'relocations':relocations}
OUT.mkdir(exist_ok=False);RUNTIME.mkdir(exist_ok=True)
(OUT/'entry-original.bin').write_bytes(entry);(OUT/'selector-original.bin').write_bytes(selector)
banks=[read_va(0x6348a8,512),read_va(0x634aa8,512)]
assert all(0<=v<=31 for bank in banks for v in bank)
assert banks[0][:32]==bytes(range(32))
(OUT/'bank-normal.bin').write_bytes(banks[0]);(OUT/'bank-alternate.bin').write_bytes(banks[1])
(OUT/'extracted-code.json').write_text(json.dumps(fragments,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
args=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args']
args=[a.replace('NativeLightingPrimitiveOracle.cpp',CPP.name).replace('native-lighting-source-oracle',RUNTIME.name) for a in args]
def execute(command,filename):
    result=subprocess.run(command,cwd=RUNTIME,env=env,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=30)
    (OUT/filename).write_bytes(result.stdout);(OUT/(filename+'.stderr')).write_bytes(result.stderr)
    assert result.returncode==0,(filename,result.returncode)
    return result
execute(args,'build.log');no_gui()
helper=RUNTIME/'oracle.exe';result=execute([str(helper),str(OUT)],'oracle.json');native=json.loads(result.stdout.decode('utf-8'))
assert (native['channel_entry_cases'],native['channel_outputs'],native['selector_calls'],native['remap_bytes_checked'])==(2048,6144,104,3328)
for name,v in fragments.items():
    original=(OUT/(name+'-original.bin')).read_bytes();relocated=(OUT/(name+'-relocated.bin')).read_bytes()
    allowed={i for r in v['relocations'] for i in range(r['offset'],r['offset']+4)}
    assert len(original)==len(relocated) and all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,relocated)))
channels=list(csv.DictReader((OUT/'channel-vectors.csv').read_text(encoding='utf-8-sig').splitlines()));assert len(channels)==2048
for row in channels:
    mode,color,level,b,g,r=(int(row[k]) for k in ['mode','color','level','blue','green','red']);value=color*level//31
    assert (b,g,r)==(value,value<<(6 if mode else 5),value<<(11 if mode else 10))
expected_tables=b''.join(struct.pack('<1024H',*[((c*l//31)<<shift) for c in range(32) for l in range(32)]) for mode in range(2) for shift in [0,6 if mode else 5,11 if mode else 10])
assert (OUT/'channel-tables.bin').read_bytes()==expected_tables
rows=list(csv.DictReader((OUT/'remap-vectors.csv').read_text(encoding='utf-8-sig').splitlines()));assert len(rows)==3328
previous=None
for call,group in itertools.groupby(rows,key=lambda r:int(r['call'])):
    group=list(group);assert [int(r['index']) for r in group]==list(range(32))
    first=group[0];requested=int(first['requested']);alternate=int(first['alternate']);before=int(first['cached_before']);level=min(15,requested)
    if before==-1:previous=bytes([0xee]*32)
    expected=previous if before==level else banks[int(alternate!=0)][level*32:(level+1)*32]
    assert bytes(int(r['actual']) for r in group)==expected
    assert all(int(r['cached_after'])==level for r in group)
    previous=expected
proof={'status':native['status'],'original_PE_sha256':sha(pe),**native,
    'Python_channel_entries_rechecked':2048,'Python_remap_bytes_rechecked':3328,
    'channel_entry_formula':'floor(color5*level5/31), blue shift0; RGB555 green5/red10; RGB565 green6/red11',
    'channel_entry_fragment_scope':'Entry arithmetic/stores with supplied product, mode and destination index; original outer initializer/FPU paths were not executed',
    'original_remap_banks_bytes':1024,'cache_semantics':'selector clamps above15; same cached level retains row even if alternate changes',
    'nonnegative_selector_inputs_only':True,'original_instructions_branches_constants_unchanged':True,
    'only_absolute_private_data_relocations':True,'entry_appended_harness_ret':True,
    'full_original_initializer_executed':False,'complete_light_mask_generation_verified':False,
    'full_world_framebuffer_verified':False,'client_source_files_unchanged':970,'product_source_changes':0,'whole_goal_complete':False}
(OUT/'audit.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(OUT/'build-command.json').write_text(json.dumps({'args':args,'cwd':str(RUNTIME)},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
files={p.name:p for p in OUT.iterdir() if p.is_file()};files.update({'oracle.cpp':CPP,'auditor.py':Path(__file__),'native-x86-oracle.exe':helper,'normal-qualified.json':normal_path})
fp={n:sha(p) for n,p in files.items()};archive=OUT/'original-table-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:
    for n,h in fp.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
for name,expected in pins.items():assert sha(ROOT/name)==expected,name
for image in normal['profiles'].values():assert sha(Path(image['path']))==image['sha256']
(OUT/'qualified.json').write_text(json.dumps({**proof,'archive_sha256':sha(archive),'file_pins':fp,'archive_readback_verified':True},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:proof[k] for k in ['status','channel_entry_cases','channel_outputs','selector_calls','remap_bytes_checked','full_original_initializer_executed','game_sessions']},ensure_ascii=False))
