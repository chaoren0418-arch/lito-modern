"""Full original64ms offset callback; no original scheduler or client execution."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,csv,hashlib,json,os,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
PE=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
CURRENT=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
assert sha(PE)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pins=read(CURRENT/'compiled-inputs.json');assert len(pins)==1025
def stable():
    for name,h in pins.items():assert sha(ROOT/name)==h
stable()
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-background-offset-callback'/stamp
runtime=PROJECT/'target/current/native-background-offset-callback'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
(out/'oracle.cpp').write_bytes((ROOT/'維護工具/oracle/NativeBackgroundOffsetCallback.cpp').read_bytes())
(out/'compiled-inputs.json').write_bytes((CURRENT/'compiled-inputs.json').read_bytes())
data=PE.read_bytes();u16=lambda n:struct.unpack_from('<H',data,n)[0];u32=lambda n:struct.unpack_from('<I',data,n)[0]
h=u32(60);optional=h+24;base=u32(optional+28);sections=[]
for i in range(u16(h+6)):
    at=optional+u16(h+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
start=0x4878c0;cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
instructions=[];relocations=[]
for ins in cs.disasm(get(start,0x30),start):
    assert ins.mnemonic in ['mov','inc','push','call','ret'],(ins.mnemonic,ins.op_str)
    instructions.append(ins)
    for op in ins.operands:
        if op.type==capstone.x86.X86_OP_MEM:
            assert op.mem.base==op.mem.index==0 and op.mem.disp in [0x62f4c0,0x62f4c4] and ins.disp_size==4
            relocations.append(dict(kind='memory',offset=ins.address-start+ins.disp_offset,target=op.mem.disp,next=0))
        if op.type==capstone.x86.X86_OP_IMM and ins.mnemonic!='call' and op.imm==start:
            assert ins.imm_size==4;relocations.append(dict(kind='self',offset=ins.address-start+ins.imm_offset,target=start,next=0))
    if ins.mnemonic=='call':
        assert ins.operands[0].type==capstone.x86.X86_OP_IMM and ins.operands[0].imm==0x4d65f0 and ins.imm_size==4
        relocations.append(dict(kind='relative',offset=ins.address-start+ins.imm_offset,target=0x4d65f0,next=ins.address-start+ins.size))
    if ins.mnemonic=='ret':break
assert instructions[-1].mnemonic=='ret' and len(relocations)==6
raw=get(start,instructions[-1].address+instructions[-1].size-start)
(out/'callback-original.bin').write_bytes(raw)
(out/'relocations.tsv').write_text(''.join(f"{r['kind']} {r['offset']} {r['target']} {r['next']}\n" for r in relocations),encoding='ascii')
meta=dict(start=start,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest().upper(),relocations=relocations,
    instructions=[dict(address=i.address,mnemonic=i.mnemonic,operands=i.op_str,bytes=i.bytes.hex()) for i in instructions])
save(out/'callback.json',meta)
command=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args']
command=[s.replace(str(ROOT/'維護工具/oracle/NativeLightingPrimitiveOracle.cpp'),str(out/'oracle.cpp')).replace(str(PROJECT/'target/current/native-lighting-source-oracle'),str(runtime)) for s in command]
assert str(out/'oracle.cpp') in command;save(out/'build-command.json',command)
state=dict(status='prepared_private_original_background_offset_callback',output=str(out),runtime=str(runtime),
    source_manifest_sha256=sha(CURRENT/'source-inputs.json'),original_PE=str(PE),original_PE_sha256=sha(PE),callback=meta,
    scheduler_body_stubbed=True,real_clock_measured=False,product_source_changed=False,modern_scene_binding_verified=False,
    actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False,processes=[],owned_process=None)
persist=lambda:save(out/'receipt.json',state)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb') as stdout,(out/(label+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['processes'].append(item);state['owned_process']=item;persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(label,code,'Preserve failure before repair')
persist();print(json.dumps(dict(output=str(out),original_code_bytes=len(raw)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_private_offset_callback_oracle';persist();execute(command,'compile')
    state['status']='executing_original_offset_callback';persist();execute([str(runtime/'oracle.exe'),str(out)],'native')
    result=read(out/'native-stdout.log');assert result['cases']==57 and result['guard_failures']==0
    addresses={k:int(v) for k,v in (line.split() for line in (out/'addresses.tsv').read_text().splitlines())}
    rows=list(csv.DictReader((out/'vectors.csv').open(encoding='ascii',newline='')));domain=set();sequence=[]
    for row in rows:
        seq,i,x,y,ax,ay,ox,oy,delay,owner,callback,calls=map(int,row.values())
        assert (ax,ay)==((x+1)&0xffffffff,(y+1)&0xffffffff) and (ox,oy)==(ax,ay)
        assert (delay,owner,callback,calls)==(64,0,addresses['callback'],1)
        if seq==0:assert (x,y) not in domain;domain.add((x,y))
        else:assert seq==1 and i==len(sequence);sequence.append((x,y,ax,ay))
    values=[0,1,65484,65534,65535,0x7fffffff,0xffffffff];assert domain=={(x,y) for x in values for y in values}
    assert sequence==[(65484+i,65534+i,65485+i,65535+i) for i in range(8)]
    original=(out/'callback-original.bin').read_bytes();relocated=(out/'callback-relocated.bin').read_bytes()
    allowed={i for r in relocations for i in range(r['offset'],r['offset']+4)}
    assert len(original)==len(relocated) and all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,relocated)))
    for item in relocations:
        expected=addresses['offset_x' if item['target']==0x62f4c0 else 'offset_y'] if item['kind']=='memory' else addresses['callback'] if item['kind']=='self' else (addresses['scheduler']-addresses['callback']-item['next'])&0xffffffff
        assert struct.unpack_from('<I',relocated,item['offset'])[0]==expected
    stable();state.update(status='original_background_offset_callback_compared',cases=57,guard_failures=0,increment_observed_before_schedule=True,helper_sha256=sha(runtime/'oracle.exe'));persist()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if (runtime/'oracle.exe').exists():files['private-x86-oracle.exe']=runtime/'oracle.exe'
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,h in hashes.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status'],cases=state.get('cases')),ensure_ascii=False),flush=True)
