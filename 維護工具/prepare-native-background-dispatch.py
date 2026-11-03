"""Retain original first-pass selector and exhaustive flag combinations; no execution."""
from pathlib import Path
from datetime import datetime,timezone
import csv,hashlib,itertools,json,struct,zipfile
import capstone
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
ORIGINAL=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收')
CURRENT=Path(json.loads((APP/'background-bank-Debug-tests-current.json').read_text(encoding='utf-8'))['path'])
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(CURRENT/'compiled-inputs.json');assert len(pins)==1015
for n,h in pins.items():assert digest((ROOT/n).read_bytes())==h,n
pe=(ORIGINAL/'runtime/LiTo.unpacked-oep-current.exe').read_bytes();assert digest(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
asm=(ORIGINAL/'ida-client-oep-v2/full-disassembly.asm').read_bytes();assert digest(asm)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
e=struct.unpack_from('<I',pe,60)[0];o=e+24;base=struct.unpack_from('<I',pe,o+28)[0];start=0x488d0b;end=0x488e90
for i in range(struct.unpack_from('<H',pe,e+6)[0]):
    _,va,size,off=struct.unpack_from('<IIII',pe,o+struct.unpack_from('<H',pe,e+20)[0]+40*i+8)
    if va<=start-base and end-base<=va+size:code=pe[off+start-base-va:off+end-base-va];break
assert len(code)==389
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
ins=list(cs.disasm(code,start));assert sum(i.size for i in ins)==389
keys={0x768894,0x8300e0,0x768898,0x768890,0x76888c,0x8300e4,0x8300ec,0x75ac40,0x503524,0x62f324}
relocations=[];seen=set()
for i in ins:
    if i.mnemonic.startswith('j'):
        target=i.operands[0].imm
        if not start<=target<=end:
            assert target==0x48af4b and i.imm_size==4;relocations.append(('J',i.address-start+i.imm_offset,target))
    if i.mnemonic=='call' and i.operands[0].type==capstone.x86.X86_OP_IMM:
        assert i.operands[0].imm==0x4be000 and i.imm_size==4;relocations.append(('C',i.address-start+i.imm_offset,0x4be000))
    for operand in i.operands:
        if operand.type==capstone.x86.X86_OP_MEM and operand.mem.base==operand.mem.index==0:
            key=operand.mem.disp&0xffffffff;assert key in keys and i.disp_size==4
            relocations.append(('D',i.address-start+i.disp_offset,key));seen.add(key)
assert seen==keys and sum(k=='J' for k,a,b in relocations)==2 and sum(k=='C' for k,a,b in relocations)==1
assert len({a for k,a,b in relocations})==len(relocations)
out=ROOT/'驗證紀錄/current/native-background-dispatch-original'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'selector-original.bin').write_bytes(code)
(out/'oracle.cpp').write_bytes((ROOT/'維護工具/oracle/NativeBackgroundDispatchOracle.cpp').read_bytes())
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
with (out/'relocations.csv').open('w',encoding='ascii',newline='') as f:csv.writer(f).writerows(relocations)
cases=[]
for options,mapid,flags in itertools.product([0,31],[302,303],range(65536)):
    cases.append([len(cases),flags,mapid,options,0,15,0])
for flags,header,options in itertools.product([0,0x40,0x400,0x20,0x200,1,8,0x80,0x10,2,0xc00,0x101,0xf000],range(9),[0,1,2,4,8,16,31]):
    cases.append([len(cases),flags,303,options,header,15,0])
for flags,shade,options in itertools.product([0x400,0x800,0xc00],[-128,-1,0,15,31,32,127],[0,31]):
    cases.append([len(cases),flags,303,options,0,shade,0])
for mapid,counter in itertools.product([302,303],[*range(48),0xffffffff]):
    cases.append([len(cases),0x20,mapid,31,0,15,counter])
for options,flags in itertools.product(range(32),[0xf000,0xf001,0xf008,0xf010,0xf020,0xf040,0xf080,0xf100,0xf200,0xfc00]):
    cases.append([len(cases),flags,303,options,0,15,0])
with (out/'cases.csv').open('w',encoding='ascii',newline='') as f:
    w=csv.writer(f);w.writerow(['case','flags','map','options','header','shade','counter']);w.writerows(cases)
old=ROOT/'驗證紀錄/current/native-background-real-original/20261016-123005-086741'
cmd=read(old/'build-command.json');cmd['args']=[s.replace(str(old),str(out)) for s in cmd['args']];cmd['cwd']=str(out);save(out/'build-command.json',cmd)
save(out/'compiled-inputs.json',pins)
proof={'status':'prepared_original_first_pass_selector_not_built_or_executed','path':str(out),
    'source_files':1015,'source_manifest_sha256':digest((CURRENT/'compiled-inputs.json').read_bytes()),
    'original_bytes':389,'appended_private_return_bytes':1,'relocations':relocations,'cases':len(cases),
    'exhaustive_flag_cases':262144,'object_pixels_executed':False,'game_sessions':0,'whole_goal_complete':False}
save(out/'preparation.json',proof)
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'preparation-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'preparation-raw.zip') as z:
    for n,h in files.items():assert digest(z.read(n))==h
save(out/'preparation-readback.json',{**proof,'files':files,'archive_sha256':digest((out/'preparation-raw.zip').read_bytes())})
save(APP/'background-dispatch-preparation-current.json',{'path':str(out)})
print(json.dumps({'path':str(out),'prepared_cases':len(cases),'original_bytes':389,'relocations':len(relocations)},ensure_ascii=False))
