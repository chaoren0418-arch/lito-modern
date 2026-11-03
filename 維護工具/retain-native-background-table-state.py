"""Read original background table state and conversion trampoline, never run."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,zipfile
import capstone

ROOT=Path(__file__).resolve().parents[1]
BASE=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收')
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'floor-tests-20261016-102632-227125/normal-qualified'
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(NORMAL/'source-inputs.json');assert len(pins)==1005
for n,h in pins.items():assert digest((ROOT/n).read_bytes())==h,n
pe=(BASE/'runtime/LiTo.unpacked-oep-current.exe').read_bytes()
asm=(BASE/'ida-client-oep-v2/full-disassembly.asm').read_bytes()
assert digest(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert digest(asm)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
e=struct.unpack_from('<I',pe,60)[0];o=e+24;base=struct.unpack_from('<I',pe,o+28)[0]
sections=[struct.unpack_from('<IIII',pe,o+struct.unpack_from('<H',pe,e+20)[0]+40*i+8) for i in range(struct.unpack_from('<H',pe,e+6)[0])]
def at(address,count):
    for _,va,size,off in sections:
        if va<=address-base and address-base+count<=va+size:return pe[off+address-base-va:off+address-base-va+count]
    raise AssertionError(hex(address))
lines=asm.decode('utf-8',errors='replace').split('\n')
names=['sub_4C18E0','sub_4F1592'];selected={}
for name in names:
    a=next(i for i,l in enumerate(lines) if l.startswith(name+' proc near'))
    b=next(i for i in range(a+1,len(lines)) if lines[i].startswith(name+' endp'))
    selected[name]={'first_original_LF_line':a+1,'last_original_LF_line':b+1,'assembly':'\n'.join(lines[a:b+1])}
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
helper=at(0x4f1592,6);ins=list(cs.disasm(helper,0x4f1592))
assert len(ins)==2 and ins[0].mnemonic=='nop' and ins[1].mnemonic=='jmp' and ins[1].operands[0].imm==0x7741ac20
constants=at(0x4fa328,16);values=struct.unpack('<2d',constants)
assert constants.hex()=='841042082184a03f0000000000003f40' and values==(1.0/31.0,31.0)
out=ROOT/'驗證紀錄/current/native-background-table-static'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
table_ranges=[]
for address,count in [(0x81089c,2048),(0x80d89c,2048),(0x81209c,0x5000),(0x82b09c,0x5000)]:
    raw=at(address,count);assert not any(raw)
    name=f'table-{address:08X}.bin';(out/name).write_bytes(raw)
    table_ranges.append({'original_address':address,'bytes':count,'nonzero_bytes':0,'sha256':digest(raw),'file':name})
(out/'conversion-trampoline.bin').write_bytes(helper)
(out/'screen-constants.bin').write_bytes(constants)
(out/'observer.py').write_bytes(Path(__file__).read_bytes())
save(out/'selected-assembly.json',selected)
proof={'status':'original_background_uninitialized_table_state_and_external_conversion_target_retained_static_only',
    'original_PE_sha256':digest(pe),'original_ASM_sha256':digest(asm),'source_files_unchanged':1005,
    'source_manifest_sha256':digest((NORMAL/'source-inputs.json').read_bytes()),'table_ranges':table_ranges,
    'table_state':'All sampled table regions are zero in this analysis PE; these bytes cannot serve as initialized background blend goldens.',
    'initializer':'sub_4C18E0','external_conversion_trampoline':{'address':0x4f1592,'bytes':6,'sha256':digest(helper),'target':0x7741ac20,'resolved_DLL_or_symbol':None},
    'screen_FPU_constants':list(values),'full_initializer_executed':False,
    'conversion_semantics_verified':False,'background_pixels_verified':False,
    'game_sessions':0,'whole_goal_complete':False}
save(out/'audit.json',proof)
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw-static.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'raw-static.zip') as z:
    for n,h in files.items():assert digest(z.read(n))==h,n
result={**proof,'path':str(out),'files':files,'raw_archive_sha256':digest((out/'raw-static.zip').read_bytes()),'raw_archive_readback_verified':True}
save(out/'retained-static.json',result);save(APP/'background-table-static-current.json',result)
print(json.dumps({'status':proof['status'],'path':str(out),'zero_ranges':len(table_ranges),'full_initializer_executed':False},ensure_ascii=False))
