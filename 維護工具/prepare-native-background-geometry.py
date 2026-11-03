"""Prepare real x86 geometry and an independent scalar; never build or execute."""
from pathlib import Path
from datetime import datetime,timezone
import csv, hashlib, itertools, json, struct, zipfile
import capstone

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'floor-tests-20261016-102632-227125/normal-qualified'
BASE=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(NORMAL/'source-inputs.json');assert len(pins)==1005
for n,h in pins.items():assert digest((ROOT/n).read_bytes())==h,n
nq=read(NORMAL/'qualified.json')
for v in nq['profiles'].values():assert digest(Path(v['path']).read_bytes())==v['sha256']
pe=(BASE/'runtime/LiTo.unpacked-oep-current.exe').read_bytes()
asm=(BASE/'ida-client-oep-v2/full-disassembly.asm').read_bytes()
assert digest(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert digest(asm)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
e=struct.unpack_from('<I',pe,60)[0];optional=e+24;base=struct.unpack_from('<I',pe,optional+28)[0]
assert base==0x400000
sections=[struct.unpack_from('<IIII',pe,optional+struct.unpack_from('<H',pe,e+20)[0]+40*i+8) for i in range(struct.unpack_from('<H',pe,e+6)[0])]
start=0x487e10
for _,va,size,off in sections:
    if va<=start-base<va+size:
        raw=pe[off+start-base-va:off+start-base-va+0x520];break
else:raise AssertionError('function section')
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
ins=[]
for i in cs.disasm(raw,start):
    ins.append(i)
    if i.mnemonic.startswith('ret'):break
assert ins[-1].mnemonic=='ret' and ins[-1].address+ins[-1].size==0x48831e
size=sum(i.size for i in ins);assert size==1294
code=raw[:size];relocs=[]
keys={0x4f7430,0x62f324,0x62f4c0,0x62f4c4,0x6476d0,0x75a998,0x75a99c,0x768878,0x8316e4,0x8316e8,0x8316ec,0x834bd4,0x834ee4}
seen=set()
for i in ins:
    if i.mnemonic.startswith('j'):
        assert i.operands[0].type==capstone.x86.X86_OP_IMM and start<=i.operands[0].imm<start+size
    if i.mnemonic=='call' and i.operands[0].type==capstone.x86.X86_OP_IMM:
        assert i.operands[0].imm==0x497c60 and i.imm_size==4
        relocs.append(('C',i.address-start+i.imm_offset,0x497c60))
    for operand in i.operands:
        if operand.type==capstone.x86.X86_OP_MEM and operand.mem.base==operand.mem.index==0:
            key=operand.mem.disp&0xffffffff
            assert key in keys and i.disp_size==4
            relocs.append(('D',i.address-start+i.disp_offset,key));seen.add(key)
assert seen==keys and sum(k=='C' for k,o,v in relocs)==15
assert len({o for k,o,v in relocs})==len(relocs)
lines=asm.decode('utf-8',errors='replace').split('\n')
a=next(i for i,l in enumerate(lines) if l.startswith('sub_487E10 proc near'))
b=next(i for i in range(a+1,len(lines)) if lines[i].startswith('sub_487E10 endp'))
out=ROOT/'驗證紀錄/current/native-background-geometry-original'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out/'geometry-original.bin').write_bytes(code)
(out/'selected-assembly.txt').write_text('\n'.join(lines[a:b+1]),encoding='utf-8')
(out/'oracle.cpp').write_bytes((ROOT/'維護工具/oracle/NativeBackgroundGeometryOracle.cpp').read_bytes())
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
with (out/'relocations.csv').open('w',encoding='ascii',newline='') as f:csv.writer(f).writerows(relocs)
def trunc(v,d):return (abs(v)//d)*(-1 if v<0 else 1)
def remainder(v,d):return v-trunc(v,d)*d
def scalar(c):
    case,mapid,level,klass,cellx,celly,camx,camy,ox,oy=c
    half=trunc(cellx,2)
    px=trunc((celly+half)*24,6)+trunc((48-camx)*5,6)
    py=trunc((celly-half)*12,6)+trunc((48-camy)*5,6)
    xrem=px%384;yrem=py%384;quot=trunc(px,384)
    strength=32 if level>=15 or 1<=klass<=3 else trunc(level*9,16)
    result=[]
    for col in range(3):
        ys=(yrem+192)%384 if remainder(col+quot,2)==1 else yrem
        for row in range(3):
            x=col*384-xrem;y=row*384-ys;left=top=0;width=height=384
            if x<0:left=-x;width=x+384;x=0
            if x+384>744:width+=360-x
            if y<0:top=-y;height=y+384;y=0
            if y+384>552:height+=138-y
            if width<=0 or height<=0:continue
            result.append([case,len(result),800 if mapid==303 else 922,803 if mapid==303 else 923,left,top,left+width,top+height,x,y,ox,oy,strength,0 if mapid==303 else 1])
    return result
poses=[(0,0),(1,1),(-1,-1),(32724,32784),(-32725,32784),(32724,-32785),(123,-456),(-555,256)]
offsets=[(0xffcc,0xfffe),(0,0),(-1,-2),(384,768)]
cameras=[(48,48),(-25,99),(100,-50)]
cases=[];calls=[];counts=[]
for mapid,level,klass,pose,offset in itertools.product([302,303],[0,1,7,14,15,31],[-1,0,1,2,3,4,6],range(8),range(4)):
    c=[len(cases),mapid,level,klass,*poses[pose],*cameras[pose%3],*offsets[offset]]
    result=scalar(c);cases.append(c);calls+=result;counts.append([c[0],len(result)])
assert len(cases)==2688 and all(1<=v[1]<=9 for v in counts)
for name,header,rows in [
    ('cases.csv',['case','map','level','class','cell_x','cell_y','camera_x','camera_y','offset_x','offset_y'],cases),
    ('calls-expected.csv',['case','ordinal','base_id','overlay_id','left','top','right','bottom','x','y','offset_x','offset_y','strength','mode'],calls),
    ('counts-expected.csv',['case','calls'],counts)]:
    with (out/name).open('w',encoding='ascii',newline='') as f:w=csv.writer(f);w.writerow(header);w.writerows(rows)
old=ROOT/'驗證紀錄/current/native-map303-gate-original/20261016-093419-178126'
command=read(old/'build-command.json')['args']
command=[arg.replace(str(old),str(out)) for arg in command]
save(out/'build-command.json',{'args':command,'cwd':str(out)})
save(out/'extraction.json',{'original_start':start,'original_bytes':size,'original_code_sha256':digest(code),'data_globals':sorted(keys),'relocations':relocs,'instructions':[{'address':i.address,'mnemonic':i.mnemonic,'operands':i.op_str} for i in ins],
    'first_original_LF_line':a+1,'last_original_LF_line':b+1,'error_string_immediates_unchanged':True,
    'forbidden_error_and_loader_branches_expected_unexecuted':True})
proof={'status':'prepared_original_background_geometry_not_built_or_executed','path':str(out),
    'original_ASM_sha256':digest(asm),'original_PE_sha256':digest(pe),'original_function_bytes':size,
    'prepared_cases':len(cases),'scalar_expected_calls':len(calls),'absolute_data_relocations':sum(k=='D' for k,o,v in relocs),'forbidden_error_call_relocations':15,
    'source_files_unchanged':1005,'source_manifest_sha256':nq['source_manifest_sha256'],
    'normal_mains':{k:v['sha256'] for k,v in nq['profiles'].items()},
    'scope':'Original geometry with warm private image objects and recording pixel stub. Resource loading, real pixel/blend kernels, full world, timers and GUI are unverified.',
    'GUI_must_finish_before_any_helper_build_or_execution':True,'native_execution_verified':False,'game_sessions':0,'whole_goal_complete':False}
save(out/'preparation.json',proof)
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'preparation-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in files:z.write(out/name,name)
with zipfile.ZipFile(out/'preparation-raw.zip') as z:
    for name,h in files.items():assert digest(z.read(name))==h
result={**proof,'files':files,'raw_archive_sha256':digest((out/'preparation-raw.zip').read_bytes()),'raw_archive_readback_verified':True}
save(out/'preparation-readback.json',result);save(APP/'background-geometry-preparation-current.json',result)
print(json.dumps({k:result[k] for k in ['status','path','prepared_cases','scalar_expected_calls','absolute_data_relocations','raw_archive_readback_verified']},ensure_ascii=False))
