"""Retain complete original initializer/kernels and bounded numeric cases."""
from pathlib import Path
from datetime import datetime,timezone
import csv,hashlib,itertools,json,struct,zipfile
import capstone

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'floor-tests-20261016-102632-227125/normal-qualified'
BASE=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收')
GB=0x630000;GE=0x832000
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,o:p.write_text(json.dumps(o,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(NORMAL/'source-inputs.json');assert len(pins)==1005
for n,h in pins.items():assert digest((ROOT/n).read_bytes())==h,n
normal=read(NORMAL/'qualified.json')
for v in normal['profiles'].values():assert digest(Path(v['path']).read_bytes())==v['sha256']
crt=read(APP/'background-CRT-layout-static-current.json')
assert crt['matching_trampolines']==57 and crt['within_candidate_module_unmatched']==0
assert digest(Path(crt['host_x86_DLL']).read_bytes())==crt['host_DLL_sha256']
raw=(BASE/'runtime/LiTo.unpacked-oep-current.exe').read_bytes();assert digest(raw)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assembly=(BASE/'ida-client-oep-v2/full-disassembly.asm').read_bytes();assert digest(assembly)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
e=struct.unpack_from('<I',raw,60)[0];o=e+24;base=struct.unpack_from('<I',raw,o+28)[0]
sections=[struct.unpack_from('<IIII',raw,o+struct.unpack_from('<H',raw,e+20)[0]+40*i+8) for i in range(struct.unpack_from('<H',raw,e+6)[0])]
def at(address,size):
    for _,va,n,off in sections:
        if va<=address-base and address-base+size<=va+n:return raw[off+address-base-va:off+address-base-va+size]
    raise AssertionError(hex(address))
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
functions={}
for name,start,size,ret_count in [('init',0x4c18e0,2030,2),('rgb565',0x4bf0b0,1101,1),('rgb555',0x4bf500,1115,2)]:
    code=at(start,size);ins=list(cs.disasm(code,start));assert sum(i.size for i in ins)==size and sum(i.mnemonic=='ret' for i in ins)==ret_count
    assert ins[-1].mnemonic=='ret';reloc=[]
    for i in ins:
        if i.mnemonic.startswith('j'):assert i.operands[0].type==capstone.x86.X86_OP_IMM and start<=i.operands[0].imm<start+size
        if i.mnemonic=='call' and i.operands[0].type==capstone.x86.X86_OP_IMM:
            expected=0x4f1592 if name=='init' else 0x5091863
            assert i.operands[0].imm==expected and i.imm_size==4
            reloc.append(('F' if name=='init' else 'E',i.address-start+i.imm_offset,expected))
        for operand in i.operands:
            if operand.type==capstone.x86.X86_OP_MEM and 0x400000<=operand.mem.disp<0xb00000:
                key=operand.mem.disp;assert (GB<=key<GE or 0x4fa328<=key<0x4fa338) and i.disp_size==4
                reloc.append(('D',i.address-start+i.disp_offset,key))
            if i.mnemonic not in ['call'] and not i.mnemonic.startswith('j') and operand.type==capstone.x86.X86_OP_IMM and GB<=operand.imm<GE:
                assert i.imm_size==4;reloc.append(('I',i.address-start+i.imm_offset,operand.imm))
    assert len({offset for kind,offset,key in reloc})==len(reloc)
    assert sum(k in ['F','E'] for k,o,v in reloc)==1
    functions[name]={'address':start,'bytes':size,'sha256':digest(code),'relocations':reloc,
        'returns':ret_count,'instructions':[{'address':i.address,'mnemonic':i.mnemonic,'operands':i.op_str} for i in ins]}
out=ROOT/'驗證紀錄/current/native-background-blend-original'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,v in functions.items():
    (out/(name+'-original.bin')).write_bytes(at(v['address'],v['bytes']))
    with (out/(name+'-relocations.csv')).open('w',encoding='ascii',newline='') as f:csv.writer(f).writerows(v['relocations'])
(out/'data-initial.bin').write_bytes(at(GB,GE-GB));(out/'screen-constants.bin').write_bytes(at(0x4fa328,16))
assert (out/'screen-constants.bin').read_bytes().hex()=='841042082184a03f0000000000003f40'
(out/'oracle.cpp').write_bytes((ROOT/'維護工具/oracle/NativeBackgroundBlendOracle.cpp').read_bytes())
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
lines=assembly.decode('utf-8',errors='replace').split('\n');selected={}
for original_name in ['sub_4C18E0','sub_4BF0B0','sub_4BF500']:
    a=next(i for i,l in enumerate(lines) if l.startswith(original_name+' proc near'));b=next(i for i in range(a+1,len(lines)) if lines[i].startswith(original_name+' endp'))
    selected[original_name]={'first_original_LF_line':a+1,'last_original_LF_line':b+1,'assembly':'\n'.join(lines[a:b+1])}
save(out/'selected-assembly.json',selected);save(out/'extraction.json',functions)
header=['case','format','pitch','strength','blend_mode','gate','offset_x','offset_y','dest_x','dest_y','left','top','right','bottom','fixture_version']
offsets=[(0,0),(0xffcc,0xfffe),(32,16),(33,31)];rects=[(0,0,32,32),(3,2,29,30),(11,7,20,10)];cases=[]
for mode,strength,blend,gate,pitch,offset in itertools.product([0,1],list(range(10))+[32],[0,1],[0,1,7],[640,744,800],range(4)):
    number=len(cases);rectangle=rects[(number//12+strength+mode)%3];dest=[(0,0),(min(pitch,744)-32,520),(5,550)][(number//4+blend+mode)%3]
    cases.append([number,mode,pitch,strength,blend,gate,*offsets[offset],*dest,*rectangle,1])
assert len(cases)==1584
with (out/'cases.csv').open('w',encoding='ascii',newline='') as f:w=csv.writer(f);w.writerow(header);w.writerows(cases)
old=ROOT/'驗證紀錄/current/native-map303-gate-original/20261016-093419-178126'
command=read(old/'build-command.json')['args'];command=[arg.replace(str(old),str(out)) for arg in command]
save(out/'build-command.json',{'args':command,'cwd':str(out)})
proof={'status':'prepared_complete_background_initializer_and_pixel_kernels_not_executed','path':str(out),
    'source_files_unchanged':1005,'source_manifest_sha256':normal['source_manifest_sha256'],'normal_mains':{k:v['sha256'] for k,v in normal['profiles'].items()},
    'original_PE_sha256':digest(raw),'original_ASM_sha256':digest(assembly),'original_function_bytes':sum(v['bytes'] for v in functions.values()),
    'functions':{k:{q:v[q] for q in ['address','bytes','sha256','returns']} for k,v in functions.items()},
    'cases':1584,'host_x86_CRT':crt['host_x86_DLL'],'host_CRT_sha256':crt['host_DLL_sha256'],'host_dependency':'msvcrt export _ftol; historical DLL image identity remains unverified',
    'native_execution_verified':False,'historical_CRT_DLL_identity_verified':False,
    'product_source_changed':False,'game_sessions':0,'whole_goal_complete':False}
save(out/'preparation.json',proof)
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'preparation-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'preparation-raw.zip') as z:
    for n,h in files.items():assert digest(z.read(n))==h,n
result={**proof,'files':files,'raw_archive_sha256':digest((out/'preparation-raw.zip').read_bytes()),'raw_archive_readback_verified':True}
save(out/'preparation-readback.json',result);save(APP/'background-blend-preparation-current.json',result)
print(json.dumps({k:result[k] for k in ['status','path','original_function_bytes','cases','raw_archive_readback_verified']},ensure_ascii=False))
