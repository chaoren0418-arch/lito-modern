"""Own x86 oracle for one bounded increment branch of the original walking callback."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,csv,hashlib,json,os,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
TABLES=ROOT/'驗證紀錄/current/native-background-camera-tables/20261016-224656-376628'
CURRENT=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
q=read(TABLES/'qualified.json');assert q['status']=='independently_qualified_original_camera_initializer_tables'
prior=read(TABLES/'receipt.json');PE=Path(prior['original_PE']);assert sha(PE)==q['original_PE_sha256']
pins=read(CURRENT/'compiled-inputs.json')
def stable():
    for name,h in pins.items():assert sha(ROOT/name)==h,name
stable()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-background-camera-selector'/stamp
runtime=PROJECT/'target/current/native-background-camera-selector'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
(out/'oracle.cpp').write_bytes((ROOT/'維護工具/oracle/NativeBackgroundCameraSelector.cpp').read_bytes())
(out/'compiled-inputs.json').write_bytes((CURRENT/'compiled-inputs.json').read_bytes())
functions=read(TABLES/'functions.json')
for name,info in functions.items():
    raw=(TABLES/(name+'-original.bin')).read_bytes();assert hashlib.sha256(raw).hexdigest().upper()==info['sha256']
    (out/(name+'-original.bin')).write_bytes(raw)
    info['relocations']=[dict(kind='memory',offset=v['offset'],target=v['target'],next=0) for v in info['relocations']]
    (out/(name+'-relocations.tsv')).write_text(''.join(f"{v['kind']} {v['offset']} {v['target']} {v['next']}\n" for v in info['relocations']),encoding='ascii')
data=PE.read_bytes();u16=lambda n:struct.unpack_from('<H',data,n)[0];u32=lambda n:struct.unpack_from('<I',data,n)[0]
header=u32(60);optional=header+24;base=u32(optional+28);sections=[]
for i in range(u16(header+6)):
    at=optional+u16(header+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
scan=list(cs.disasm(get(0x4e2150,0x4e2415-0x4e2150),0x4e2150))
starts=[i.address for i in scan if i.mnemonic=='cmp' and i.op_str=='ebx, 0xc' and i.address<0x4e228a]
assert len(starts)==1
start=starts[0];end=0x4e2415;raw=get(start,end-start);instructions=list(cs.disasm(raw,start));relocations=[]
assert sum(i.size for i in instructions)==len(raw)
for ins in instructions:
    assert ins.mnemonic in ['mov','movsx','lea','add','neg','cmp','test','shl','call','jmp','je','jne'],(hex(ins.address),ins.mnemonic,ins.op_str)
    for op in ins.operands:
        if op.type==capstone.x86.X86_OP_MEM and (0x834c68<=op.mem.disp<0x834da8 or op.mem.disp in [0x75a998,0x75a99c,0x834ee4]):
            target=op.mem.disp
            assert 0x834c68<=target<0x834da8 or target in [0x75a998,0x75a99c,0x834ee4]
            assert ins.disp_size==4
            relocations.append(dict(kind='memory',offset=ins.address-start+ins.disp_offset,target=target,next=0))
        elif op.type==capstone.x86.X86_OP_MEM:
            assert op.mem.base not in [capstone.x86.X86_REG_EBP,capstone.x86.X86_REG_ESP]
            assert op.mem.base!=0 and -128<=op.mem.disp<=128,'Unmapped absolute memory operand'
    if ins.group(capstone.CS_GRP_JUMP) or ins.mnemonic=='call':
        assert len(ins.operands)==1 and ins.operands[0].type==capstone.x86.X86_OP_IMM
        target=ins.operands[0].imm&0xffffffff
        if not start<=target<end:
            assert (ins.mnemonic=='call' and target in [0x4874d0,0x4e8e30]) or (ins.mnemonic!='call' and target==0x4e2d07)
            assert ins.imm_size==4
            relocations.append(dict(kind='relative',offset=ins.address-start+ins.imm_offset,target=target,next=ins.address-start+ins.size))
functions['selector']=dict(start=start,end=end,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest().upper(),relocations=relocations,
    instructions=[dict(address=i.address,mnemonic=i.mnemonic,operands=i.op_str,bytes=i.bytes.hex()) for i in instructions])
(out/'selector-original.bin').write_bytes(raw)
(out/'selector-relocations.tsv').write_text(''.join(f"{v['kind']} {v['offset']} {v['target']} {v['next']}\n" for v in relocations),encoding='ascii')
save(out/'functions.json',functions)
command=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args']
command=[s.replace(str(ROOT/'維護工具/oracle/NativeLightingPrimitiveOracle.cpp'),str(out/'oracle.cpp')).replace(str(PROJECT/'target/current/native-lighting-source-oracle'),str(runtime)) for s in command]
assert str(out/'oracle.cpp') in command
save(out/'build-command.json',command)
state=dict(status='prepared_private_original_camera_increment_selector',output=str(out),runtime=str(runtime),
    source_manifest_sha256=q['source_manifest_sha256'],original_PE=str(PE),original_PE_sha256=sha(PE),
    initializer_qualification=str(TABLES/'qualified.json'),initializer_qualification_sha256=sha(TABLES/'qualified.json'),functions=functions,
    source_scope='Only noninitial increment-selector branch; appended boundary RET; original scroll tail stubbed to record arguments. Initialization/finish paths and full walking callback not executed.',
    product_source_changed=False,whole_callback_executed=False,modern_Pose_binding_verified=False,normal_scene_map303_enabled=False,
    actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False,processes=[],owned_process=None)
persist=lambda:save(out/'receipt.json',state)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb') as stdout,(out/(label+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['owned_process']=item;state['processes'].append(item);persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(label,code,'Preserve failure before repair')
persist();print(json.dumps(dict(output=str(out),selector_start=hex(start),selector_bytes=len(raw)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_isolated_x86_camera_selector';persist();execute(command,'compile')
    state['status']='executing_isolated_camera_increment_selector';persist();execute([str(runtime/'oracle.exe'),str(out)],'native')
    result=read(out/'native-stdout.log');assert result['cases']==192 and result['guard_failures']==0
    rows=list(csv.DictReader((out/'vectors.csv').open(encoding='ascii',newline='')));seen=set()
    for row in rows:
        n,h,phase,bx,by,ax,ay,sx,sy,calls,fallback=map(int,row.values());key=(n,h,phase,bx,by);assert key not in seen;seen.add(key)
        table='table_834D68' if n==4 else 'table_834CE8' if n==6 else ('table_834CA8' if phase&1 else 'table_834C68') if n==8 else 'table_834D28'
        dx,dy=q['initializer_tables'][table][h]
        assert (ax,ay,sx,sy,calls,fallback)==(bx+dx,by+dy,-dx,-dy,1,0)
    assert seen=={(n,h,p,x,y) for n in [4,6,8,12] for h in range(8) for p in [1,2] for x,y in [(48,48),(0,0),(-4096,4096)]}
    stable();state.update(status='isolated_original_camera_increment_selector_compared',cases=192,guard_failures=0,helper_sha256=sha(runtime/'oracle.exe'));persist()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if (runtime/'oracle.exe').exists():files['isolated-x86-oracle.exe']=runtime/'oracle.exe'
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,h in hashes.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status'],cases=state.get('cases')),ensure_ascii=False),flush=True)
