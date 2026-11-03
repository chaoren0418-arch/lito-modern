"""Execute only five original table initializers in an owned x86 numerical oracle."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,os,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
PE=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
ASM=ROOT.parent/'lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm'
CURRENT=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
assert sha(PE)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(ASM)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
q=read(CURRENT/'completion-observation.json');pins=read(CURRENT/'compiled-inputs.json')
assert q['status']=='independently_qualified_retained_archive_both_profiles' and len(pins)==1025
def stable():
    for name,h in pins.items():assert sha(ROOT/name)==h,name
stable()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-background-camera-tables'/stamp
runtime=PROJECT/'target/current/native-background-camera-tables'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
(out/'oracle.cpp').write_bytes((ROOT/'維護工具/oracle/NativeBackgroundCameraTables.cpp').read_bytes())
data=PE.read_bytes();u16=lambda at:struct.unpack_from('<H',data,at)[0];u32=lambda at:struct.unpack_from('<I',data,at)[0]
pe=u32(60);optional=pe+24;imagebase=u32(optional+28);assert imagebase==0x400000 and u16(optional)==0x10b
sections=[]
for i in range(u16(pe+6)):
    at=optional+u16(pe+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
def get(va,size):
    rva=va-imagebase
    for start,length,offset in sections:
        if start<=rva and rva+size<=start+length:return data[offset+rva-start:offset+rva-start+size]
    raise AssertionError(hex(va))
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
functions={}
for start,table in [(0x4e1000,0x834d68),(0x4e10a0,0x834d28),(0x4e1140,0x834ce8),(0x4e11e0,0x834ca8),(0x4e1280,0x834c68)]:
    name=f'table_{table:06X}';instructions=[];relocations=[]
    for ins in cs.disasm(get(start,0x90),start):
        assert ins.mnemonic in ['mov','xor','ret'],(ins.mnemonic,ins.op_str)
        instructions.append(ins)
        for op in ins.operands:
            if op.type==capstone.x86.X86_OP_MEM:
                assert op.mem.base==op.mem.index==0 and table<=op.mem.disp<table+64 and op.mem.disp%4==0
                assert ins.disp_size==4
                relocations.append(dict(offset=ins.address-start+ins.disp_offset,target=op.mem.disp))
        if ins.mnemonic=='ret':break
    assert instructions[-1].mnemonic=='ret' and len(relocations)==16
    assert {r['target'] for r in relocations}==set(range(table,table+64,4))
    raw=get(start,instructions[-1].address+instructions[-1].size-start)
    (out/(name+'-original.bin')).write_bytes(raw)
    (out/(name+'-relocations.tsv')).write_text(''.join(f"{r['offset']} {r['target']}\n" for r in relocations),encoding='ascii')
    functions[name]=dict(start=start,table=table,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest().upper(),relocations=relocations,
        instructions=[dict(address=i.address,mnemonic=i.mnemonic,operands=i.op_str,bytes=i.bytes.hex()) for i in instructions])
save(out/'functions.json',functions)
command=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args']
command=[part.replace(str(ROOT/'維護工具/oracle/NativeLightingPrimitiveOracle.cpp'),str(out/'oracle.cpp')).replace(str(PROJECT/'target/current/native-lighting-source-oracle'),str(runtime)) for part in command]
assert str(out/'oracle.cpp') in command
save(out/'build-command.json',command)
state=dict(status='prepared_private_original_camera_initializers',source_manifest_sha256=q['source_manifest_sha256'],compiled_inputs=1025,
    original_PE=str(PE),original_PE_sha256=sha(PE),original_ASM_sha256=sha(ASM),functions=functions,
    product_source_changed=False,normal_scene_map303_enabled=False,whole_callback_executed=False,
    modern_Pose_to_raw_camera_verified=False,actual_formal_sessions=0,whole_goal_complete=False,processes=[],owned_process=None)
persist=lambda:save(out/'receipt.json',state)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,name):
    with (out/(name+'-stdout.log')).open('wb') as stdout,(out/(name+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['processes'].append(item);state['owned_process']=item;persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(name,code,'Retain failure before repair')
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_private_x86_camera_initializer_oracle';persist();execute(command,'compile')
    helper=runtime/'oracle.exe';binary=helper.read_bytes();header=int.from_bytes(binary[60:64],'little')
    assert binary[:2]==b'MZ' and binary[header+4:header+6]==b'\x4c\x01' and binary[header+24:header+26]==b'\x0b\x01'
    state['status']='executing_private_original_camera_initializers';persist();execute([str(helper),str(out)],'native')
    result=read(out/'native-stdout.log');assert result['initializer_functions']==5 and result['heading_vectors']==40 and result['guard_failures']==0
    addresses={name:int(value) for name,value in [line.split() for line in (out/'relocation-addresses.tsv').read_text().splitlines()]}
    expected={'table_834D68':[[-6,-3],[0,-6],[6,-3],[12,0],[6,3],[0,6],[-6,3],[-12,0]],
        'table_834D28':[[-2,-1],[0,-2],[2,-1],[4,0],[2,1],[0,2],[-2,1],[-4,0]],
        'table_834CE8':[[-4,-2],[0,-4],[4,-2],[8,0],[4,2],[0,4],[-4,2],[-8,0]],
        'table_834CA8':[[-3,-2],[0,-3],[3,-2],[6,0],[3,2],[0,3],[-3,2],[-6,0]],
        'table_834C68':[[-3,-1],[0,-3],[3,-1],[6,0],[3,1],[0,3],[-3,1],[-6,0]]}
    assert {row['name']:row['values'] for row in result['tables']}==expected
    for name,info in functions.items():
        original=(out/(name+'-original.bin')).read_bytes();relocated=(out/(name+'-relocated.bin')).read_bytes()
        allowed={i for r in info['relocations'] for i in range(r['offset'],r['offset']+4)}
        assert len(original)==len(relocated) and all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,relocated)))
        for r in info['relocations']:assert struct.unpack_from('<I',relocated,r['offset'])[0]==addresses['table_data']+r['target']-0x834c68
    stable();state.update(status='original_camera_initializer_tables_independently_compared',initializer_functions=5,heading_vectors=40,guard_failures=0,helper_sha256=sha(helper));persist()
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
print(json.dumps(dict(output=str(out),status=state['status'],heading_vectors=40),ensure_ascii=False),flush=True)
