"""Execute the original producer/classifier graph with supplied private tables."""
import hashlib
import itertools
import json
import os
from pathlib import Path
import struct
import subprocess
import zipfile
from capstone import Cs,CS_ARCH_X86,CS_MODE_32,CS_GRP_CALL,CS_GRP_JUMP
from capstone.x86 import X86_OP_MEM,X86_OP_IMM

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-lighting-mask-producer'
RUNTIME=ROOT/'LiTo381-Modern/target/current/native-lighting-producer-oracle'
PE=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe')
ASM=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
assert sha(PE)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(ASM)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
# Run this bounded CPU-only oracle during the verified Release compilation,
# never alongside the subsequent live GUI cohort.
pipeline=read(ROOT/'驗證紀錄/current/native-game-timers/20261016-004010-201637/current-regression/current-followup.json')
phase=read(ROOT/'驗證紀錄/current/default-native-regression/20261016-010030-601-Release/receipt.json')
assert pipeline['status']=='running' and pipeline['phase']=='default93_both_profiles'
assert phase['status']=='compiling_default_targets' and phase['owned_process']
ps=subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',
    '[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); Get-CimInstance Win32_Process -Filter "ProcessId=57368" | ForEach-Object { [pscustomobject]@{pid=$_.ProcessId;name=$_.Name;exe=$_.ExecutablePath;command=$_.CommandLine;creation=$_.CreationDate.ToUniversalTime().ToString("o")} } | ConvertTo-Json -Compress'],capture_output=True,check=True)
live=json.loads(ps.stdout.decode('utf-8-sig'));assert live['name']=='cargo.exe' and '--release' in live['command'] and '--target-dir target/current' in live['command']
assert Path(live['exe']).resolve()==Path(phase['owned_process']['arguments'][0]).resolve()
from datetime import datetime
assert abs((datetime.fromisoformat(live['creation'].replace('Z','+00:00'))-datetime.fromisoformat(phase['owned_process']['creation_utc'])).total_seconds())<.01
normal_dir=ROOT/'驗證紀錄/current/native-game-timers/20261016-004010-201637/normal'
normal_qualification=read(normal_dir/'qualified.json');product_pins=read(normal_dir/'source-inputs.json')
def stable():
    assert len(product_pins)==976
    for n,h in product_pins.items():assert sha(ROOT/n)==h,n
    for v in normal_qualification['profiles'].values():assert sha(Path(v['path']))==v['sha256']
stable()
OUT.mkdir(exist_ok=False);RUNTIME.mkdir(exist_ok=True)
raw=PE.read_bytes();eh=struct.unpack_from('<I',raw,60)[0];opt=eh+24;sz=struct.unpack_from('<H',raw,eh+20)[0];base=struct.unpack_from('<I',raw,opt+28)[0]
sections=[struct.unpack_from('<IIII',raw,opt+sz+40*i+8) for i in range(struct.unpack_from('<H',raw,eh+6)[0])]
def at(address,count):
    for vs,va,rs,rp in sections:
        rva=address-base
        if va<=rva and rva+count<=va+rs:return raw[rp+rva-va:rp+rva-va+count]
    raise AssertionError(hex(address))
functions={'producer':(0x485a40,728),'project':(0x4b8fc0,None),'frame_count':(0x4e8ea0,None),
    'classify':(0x4e89f0,214),'predicate':(0x4e8bd0,38),'compare':(0x4d3a80,93),
    'remap':(0x4bd150,85),'blend':(0x4bb170,184),'clear':(0x4bb230,19)}
decoder=Cs(CS_ARCH_X86,CS_MODE_32);decoder.detail=True
for key,(address,count) in list(functions.items()):
    if count is None:
        instructions=[]
        for i in decoder.disasm(at(address,256),address):
            instructions.append(i)
            if i.mnemonic=='ret':break
        count=sum(i.size for i in instructions);functions[key]=(address,count)
starts={v[0] for v in functions.values()};image=bytearray(0x100000);relocations=[];details={}
for key,(address,count) in functions.items():
    code=at(address,count);ins=list(decoder.disasm(code,address));assert sum(i.size for i in ins)==count and ins[-1].mnemonic=='ret',key
    image[address-0x480000:address-0x480000+count]=code
    if key=='blend':image[0x80000:0x80000+count]=code
    for i in ins:
        if i.group(CS_GRP_CALL):assert i.operands[0].type==X86_OP_IMM and i.operands[0].imm in starts,(key,hex(i.address),i.op_str)
        if i.group(CS_GRP_JUMP):assert i.operands[0].type==X86_OP_IMM and address<=i.operands[0].imm<address+count,(key,hex(i.address),i.op_str)
        for op in i.operands:
            if op.type==X86_OP_MEM and op.mem.disp>=0x500000:
                assert i.disp_size==4
                position=i.address-0x480000+i.disp_offset
                assert struct.unpack_from('<I',image,position)[0]==op.mem.disp
                relocations.append((position,op.mem.disp))
            elif op.type==X86_OP_IMM and op.imm in {0x6348a8,0x634aa8,0x83009c}:
                assert i.imm_size==4
                position=i.address-0x480000+i.imm_offset
                assert struct.unpack_from('<I',image,position)[0]==op.imm
                relocations.append((position,op.imm))
    details[key]={'address':hex(address),'bytes':count,'sha256':hashlib.sha256(code).hexdigest().upper(),
        'instructions':[{'address':hex(i.address),'mnemonic':i.mnemonic,'operands':i.op_str,'bytes':bytes(i.bytes).hex()} for i in ins]}
(OUT/'graph-original.bin').write_bytes(image);(OUT/'relocations.bin').write_bytes(b''.join(struct.pack('<II',*r) for r in relocations))
(OUT/'original-functions.json').write_text(json.dumps({'functions':details,'data_relocations':[(hex(a),hex(b)) for a,b in relocations]},indent=2)+'\n',encoding='utf-8')
(OUT/'offset-tables.bin').write_bytes(at(0x62e7e8,1920))
for name in ['bank-normal.bin','bank-alternate.bin']:(OUT/name).write_bytes((ROOT/'驗證紀錄/current/native-lighting-original-tables'/name).read_bytes())
lmp=(ROOT/'驗證紀錄/current/native-lighting-source-lmp/source.lmp').read_bytes();assert hashlib.sha256(lmp).hexdigest().upper()=='30CF051FE516A053AC494B8054E5A320BAB1F3F3DD7B0010E0CF0C74AACCE5AA';(OUT/'source.lmp').write_bytes(lmp)
slots=[]
for i in range(16):
    off=struct.unpack_from('<I',lmp,i*4)[0];w,h=struct.unpack_from('<ii',lmp,64+i*8);slots.append((off,w,h,lmp[off:off+w*h]))
def actor(**kw):
    v=dict(u=1000,v=1000,light=5,action=0,walk=0,heading=0,frame=0,frames=4,type=1,subtype=0,flag1f=0,flag50=0,state23=0,owner=0);v.update(kw);return list(v.values())
def case(actors,ambient=5,water=0,blind=0,camera=(-320,-198),cached=-1,seed=11):
    return {'header':[ambient,water,blind,1000,1000,*camera,seed,cached,7,len(actors)],'actors':actors}
cases=[]
for ambient,water,blind in itertools.product(range(17),range(2),range(3)):
    cases.append(case([actor(light=14),actor(u=1006,v=1003,light=3,type=5),actor(u=994,v=999,light=0,type=10,owner=1)],ambient,water,blind))
for light,water,blind in itertools.product([0,1,4,5,14,15,16,127,128,255],range(2),range(3)):
    cases.append(case([actor(light=light)],water=water,blind=blind))
for typ,action,flag1f,flag50 in itertools.product(range(16),[0,8,28,37,255],range(2),range(2)):
    cases.append(case([actor(light=0),actor(u=1002,light=3,type=typ,action=action,flag1f=flag1f,flag50=flag50)],water=1))
for owner,subtype,state in itertools.product([-1,0,1,2],[0,2],[0,2]):
    cases.append(case([actor(light=0),actor(u=1003,light=2,type=10,owner=owner,subtype=subtype,state23=state)],water=1))
for count in [12,8,6,4,3]:
    for heading in range(8):
        for frame in range(count if count in [12,8,6] else 5):
            cases.append(case([actor(u=1000+(heading%2),frames=count,heading=heading,frame=frame)]))
for typ,blind in itertools.product(range(16),[1,2]):
    cases.append(case([actor(light=0),actor(u=1008,light=255,type=typ)],blind=blind))
for du,dv in [(35,15),(36,15),(35,16),(-35,-15),(-36,-15),(-35,-16)]:
    cases.append(case([actor(light=0),actor(u=1000+du,v=1000+dv,light=14)]))
for camera in [(-3,-3),(0,0),(300,300),(-640,-408),(-320,-198)]:cases.append(case([actor(light=14)],camera=camera))
for light in [1,14,15]:cases.append(case([actor(light=light) for _ in range(128)]))
for cached,water,blind in itertools.product([-1,0,3,5,15],range(2),[0,1]):cases.append(case([actor(light=5)],water=water,blind=blind,cached=cached))
for ambient,water,blind in itertools.product([5,15],range(2),range(3)):cases.append(case([],ambient,water,blind))
(OUT/'cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
commands=bytearray(struct.pack('<I',len(cases)))
for c in cases:
    commands.extend(struct.pack('<11i',*c['header']))
    for a in c['actors']:commands.extend(struct.pack('<14i',*a))
(OUT/'commands.bin').write_bytes(commands)
cpp=ROOT/'維護工具/oracle/NativeLightingProducerOracle.cpp';(OUT/'oracle.cpp').write_bytes(cpp.read_bytes());(OUT/'audit.py').write_bytes(Path(__file__).read_bytes())
args=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args'];args=[s.replace('NativeLightingPrimitiveOracle.cpp',cpp.name).replace('native-lighting-source-oracle',RUNTIME.name) for s in args]
env=os.environ.copy();env.update(PYTHONDONTWRITEBYTECODE='1',TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'))
def execute(command,name):
    result=subprocess.run(command,cwd=RUNTIME,env=env,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=50)
    (OUT/name).write_bytes(result.stdout);(OUT/(name+'.stderr')).write_bytes(result.stderr)
    assert result.returncode==0,(name,result.returncode)
    return result
proof=None
try:
    execute(args,'build.log');(OUT/'oracle.exe').write_bytes((RUNTIME/'oracle.exe').read_bytes())
    native=json.loads(execute([str(RUNTIME/'oracle.exe'),str(OUT)],'native.json').stdout.decode('utf-8'))
    data=(OUT/'native-results.bin').read_bytes();position=0;blit_total=0
    offsets=(OUT/'offset-tables.bin').read_bytes();normal=(OUT/'bank-normal.bin').read_bytes();alternate=(OUT/'bank-alternate.bin').read_bytes()
    def signed(n):return n if n<128 else n-256
    def classify(a):
        typ=a[8];act=signed(a[3]);force=a[10];dead=a[11]
        if typ==11:return 1 if act>=37 or act==28 else 11
        if force:return 3 if dead or act==8 else 5
        if typ==10:
            if dead:return 3
            if act==8:return 2
            if a[13] in [1,2]:return 6 if a[13]==1 else 5
            return 1 if a[9]==2 and a[12]==2 else 10
        if typ in [0,4,7,8,9,13,14,15]:return typ
        if typ in [5,6,12]:return 3 if dead or act==8 else typ
        return 1
    vectors=[]
    for index,c in enumerate(cases):
        ambient,water,blind,ou,ov,cx,cy,seed,cached,row_seed,count=c['header'];aa=c['actors'];effective=3 if water else ambient
        expected=bytearray((i*13+seed)&255 for i in range(65280));row=bytes((i*7+row_seed)&31 for i in range(32));trace=[];total=0
        if blind or effective<15:
            level=0 if blind else min(15,effective)
            if cached!=level:row=(alternate if water else normal)[level*32:(level+1)*32];cached=level
            expected=bytearray(65280)
            for i,a in enumerate(aa):
                if abs(aa[0][0]-a[0])>35 or abs(aa[0][1]-a[1])>15:continue
                if blind:
                    if i!=0 and (blind!=2 or a[8] not in [5,6,10,12]):continue
                    light=5
                elif water and classify(a) in [5,3]:light=14 if classify(a)==5 else 5
                else:
                    light=min(15,signed(a[2]))
                    if light<=0:continue
                u,v=a[0]-ou,a[1]-ov
                px=12*(((u+1)&~1)+2*v-2)-cx+24;py=6*(2*v-(u&~1))-cy+6
                if signed(a[3])==signed(a[4]):
                    frames=a[7];stride=frames if frames in [12,8,6] else 4;phase=a[6] if stride!=4 else min(a[6],3)
                    address={12:0x62e8e8,8:0x62ed68,6:0x62ebe8,4:0x62e7e8}[stride]
                    dx,dy=struct.unpack_from('<ii',offsets,address-0x62e7e8+(a[5]*stride+phase)*8);px+=dx;py+=dy
                off,w,h,pixels=slots[light]
                half=lambda n:(-1 if n<0 else 1)*(abs(n)//2)
                x=half(px-w)&~3;y=half(py-h);trace.append((x,y,w,h,off));total+=light
                for yy in range(max(0,y),min(204,y+h)):
                    for xx in range(max(0,x),min(320,x+w)):
                        dest=yy*320+xx;expected[dest]=max(expected[dest],pixels[(yy-y)*w+xx-x])
                if total>=700:break
        mask=data[position:position+65280];position+=65280;observed_row=data[position:position+32];position+=32
        observed_cached,n=struct.unpack_from('<iI',data,position);position+=8
        actual_trace=[struct.unpack_from('<5i',data,position+j*20) for j in range(n)];position+=n*20
        actual_probes=list(data[position:position+count*3]);position+=count*3
        expected_probes=[v for a in aa for v in [classify(a),int(a[8] in [5,6,10,12]),a[7]]]
        assert mask==expected and observed_row==row and observed_cached==cached and actual_trace==trace and actual_probes==expected_probes,('producer mismatch',index,actual_trace,trace,actual_probes,expected_probes)
        blit_total+=n;vectors.append({'case':index,'mask_sha256':hashlib.sha256(mask).hexdigest().upper(),'remap':list(row),'cached':cached,'blits':trace,'actor_queries':actual_probes})
    assert position==len(data) and native['cases']==len(cases) and native['blits']==blit_total
    (OUT/'verified-vectors.json').write_text(json.dumps(vectors,indent=2)+'\n',encoding='utf-8')
    proof={'status':'original_private_mask_producer_classifier_projection_and_compositor_graph_qualified_with_fixture_tables',
        **native,'original_PE_sha256':sha(PE),'original_ASM_sha256':sha(ASM),'functions_executed':list(functions),
        'source_lmp_sha256':sha(OUT/'source.lmp'),'Python_output_masks_and_query_results_rechecked':len(cases),
        'producer_instructions_and_branches_preserved_except_private_data_operands':True,
        'blit_dependency_entry_uses_counter_wrapper_then_unmodified_original_copy':True,
        'ASCII_casefold_and_catalog_tables_supplied_as_fixtures':True,'actual_SPR_attribute102_and108_loader_execution_verified':False,
        'packet_to_native_coordinate_and_actor_flag_mapping_verified':False,'full_world_framebuffer_verified':False,
        'client_product_source_changed':False,'new_game_sessions':0,'whole_goal_complete':False}
except BaseException as error:
    (OUT/'failure.json').write_text(json.dumps({'status':'failed_independent_oracle','error':repr(error),'new_game_sessions':0},indent=2)+'\n',encoding='utf-8');raise
finally:
    files={p.name:sha(p) for p in OUT.iterdir() if p.is_file() and p.suffix!='.zip'};archive=OUT/'producer-evidence.zip'
    with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n in files:z.write(OUT/n,n)
    with zipfile.ZipFile(archive) as z:
        assert z.testzip() is None
        for n,h in files.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    retained={'archive_sha256':sha(archive),'files':files,'readback_verified':True}
    (OUT/'retention.json').write_text(json.dumps(retained,indent=2)+'\n',encoding='utf-8')
if proof:
    stable();proof.update(retained);proof['current_product_source_files_unchanged']=976;proof['current_product_source_manifest_sha256']=normal_qualification['source_manifest_sha256'];(OUT/'qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:v for k,v in proof.items() if k!='files'},ensure_ascii=False))
