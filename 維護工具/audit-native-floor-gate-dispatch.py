"""Execute original floor selector with real isolated writers and bounded TILs."""
from pathlib import Path
from datetime import datetime,timezone
import array,capstone,csv,hashlib,json,os,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'actual-tests-20261016-084451-903086/normal-qualified'
STATIC=ROOT/'驗證紀錄/current/native-map303-gate-static/20261016-090752-113308'
PE=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(NORMAL/'source-inputs.json');assert len(pins)==1002
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for v in read(NORMAL/'qualified.json')['profiles'].values():assert sha(Path(v['path']))==v['sha256']
def idle():
    ps="$r='"+str(ROOT).replace("'","''")+"';$p=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -in @('cargo.exe','rustc.exe')) -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|native_|oracle)' -or $_.ExecutablePath -match '\\\\deps\\\\.*\\.exe$'))});if($p.Count){exit 1}"
    result=subprocess.run(['powershell.exe','-NoProfile','-Command',ps],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert result.returncode==0,'Owned GUI/compiler/server/oracle still live'
stable();idle();raw=PE.read_bytes();assert digest(raw)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
eh=struct.unpack_from('<I',raw,60)[0];opt=eh+24;base=struct.unpack_from('<I',raw,opt+28)[0]
sections=[struct.unpack_from('<IIII',raw,opt+struct.unpack_from('<H',raw,eh+20)[0]+40*i+8) for i in range(struct.unpack_from('<H',raw,eh+6)[0])]
def at(va,n):
    rva=va-base
    for _,start,length,offset in sections:
        if start<=rva and rva+n<=start+length:return raw[offset+rva-start:offset+rva-start+n]
    raise ValueError(hex(va))
out=ROOT/'驗證紀錄/current/native-floor-gate-original'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True,exist_ok=False)
(out/'oracle.cpp').write_bytes((ROOT/'維護工具/oracle/NativeFloorGateOracle.cpp').read_bytes());(out/'auditor.py').write_bytes(Path(__file__).read_bytes());(out/'source-inputs.json').write_bytes((NORMAL/'source-inputs.json').read_bytes());(out/'normal-qualified.json').write_bytes((NORMAL/'qualified.json').read_bytes())
functions=read(STATIC/'extracted-code.json');relocs=[]
for name,m in functions.items():
    code=at(m['start'],m['bytes']);assert digest(code)==m['sha256']==sha(STATIC/(name+'-original.bin'))
    (out/(name+'-original.bin')).write_bytes(code)
    for r in m['relocations']:relocs.append((name,'absolute',r['offset'],r['original_address'],0))
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
start,end=0x488c5f,0x488cd7;code=at(start,end-start);instructions=list(cs.disasm(code,start));assert sum(i.size for i in instructions)==120
for i in instructions:
    if i.mnemonic.startswith('j'):assert start<=i.operands[0].imm<=end
    if i.mnemonic=='call':
        assert i.operands[0].type==capstone.x86.X86_OP_IMM and i.operands[0].imm in [0x4be000,0x4c14c0,0x4c1670] and i.imm_size==4
        relocs.append(('floor','relative',i.address-start+i.imm_offset,i.operands[0].imm,i.address-start+i.size))
    for op in i.operands:
        if op.type==capstone.x86.X86_OP_MEM and op.mem.base==0 and op.mem.index==0:
            assert op.mem.disp==0x62f324 and i.disp_size==4
            relocs.append(('floor','absolute',i.address-start+i.disp_offset,op.mem.disp,0))
assert len(relocs)==12
(out/'floor-original.bin').write_bytes(code);functions['floor']={'start':start,'bytes':120,'sha256':digest(code),'appended_private_return_byte':'C3','instructions':[{'address':i.address,'mnemonic':i.mnemonic,'operands':i.op_str} for i in instructions]}
(out/'relocations.tsv').write_text(''.join(f'{n} {kind} {o} {t} {next}\n' for n,kind,o,t,next in relocs),encoding='ascii');save(out/'extracted-code.json',functions)
command=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args'];command=[s.replace(str(ROOT/'維護工具/oracle/NativeLightingPrimitiveOracle.cpp'),str(out/'oracle.cpp')).replace(str(PROJECT/'target/current/native-lighting-source-oracle'),str(out)) for s in command];assert '/Fe'+str(out/'oracle.exe') in command
save(out/'build-command.json',{'args':command,'cwd':str(out)})
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
receipt={'status':'building_original_floor_dispatch_oracle','source_files':1002,'owned_process':None,'native_execution':False,'game_sessions':0,'whole_goal_complete':False};persist=lambda:save(out/'receipt.json',receipt)
def run(command,name):
    with (out/name).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:
        child=subprocess.Popen(command,cwd=out,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);receipt['owned_process']={'pid':child.pid,'args':command};persist();code=child.wait();receipt['owned_process']=None;receipt[name+'_exit_code']=code;persist()
    assert code==0,(name,code)
persist()
try:
    run(command,'build.log');stable();idle();receipt.update(status='executing_original_floor_dispatch_oracle',native_execution=True);persist();run([str(out/'oracle.exe'),str(out)],'oracle.json')
    native=read(out/'oracle.json');assert native=={'status':'original_floor_dispatch_and_gate_executed','cases':64,'color_and_gate_cells':52568064,'surface_Unlock_stub_calls':16,'guard_failures':0,'whole_client_executed':False}
    address={int(a):int(b) for a,b in (l.split() for l in (out/'addresses.tsv').read_text().splitlines())}
    for name,m in functions.items():
        original=(out/(name+'-original.bin')).read_bytes();actual=(out/(name+'-relocated.bin')).read_bytes();assert len(actual)==len(original)+(name=='floor')
        if name=='floor':assert actual[-1]==0xc3
        selected=[r for r in relocs if r[0]==name];allowed={p for r in selected for p in range(r[2],r[2]+4)};assert all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,actual)))
        for _,kind,o,t,next in selected:
            expected=address[t] if kind=='absolute' else (address[t]-address[m['start']]-next)&0xffffffff
            assert struct.unpack_from('<I',actual,o)[0]==expected
    records=list(csv.DictReader((out/'cases.csv').open(newline='',encoding='ascii')));assert len(records)==64
    actual_cases=set();vectors=[]
    for row in records:
        index,kind,parity,map_id,pitch,origin,unlocks=map(int,[row[k] for k in ['id','type','parity','map','pitch','start','unlock_calls']]);assert pitch==744 and origin==250*744+180
        key=(kind,parity,map_id);assert key not in actual_cases;actual_cases.add(key)
        source=(out/f'source-{kind}.bin').read_bytes();assert source[0]==kind
        frame=array.array('H',[0x7777])*(744*552);gate=bytearray(744*552);writes=[]
        if kind&8:assert unlocks==0
        elif kind&2:
            assert unlocks==0;x,y,w,h=source[1:5];assert x+w<=24 and y+h<=24;p=5
            for dy in range(h):
                count=source[p];p+=1;dx=x
                for _ in range(count):
                    skip,n=source[p:p+2];p+=2;assert skip%2==0;dx+=skip//2;assert dx+n<=x+w
                    for j in range(n):writes.append((origin+(y+dy)*744+dx+j,struct.unpack_from('<H',source,p)[0]));p+=2
                    dx+=n
            assert p==len(source)
        else:
            assert unlocks==1;p=1
            for dy in range(24):
                n=2*(dy+1 if dy<12 else 23-dy);dx=24-n if parity==0 else 0
                for j in range(n):writes.append((origin+dy*744+dx+j,struct.unpack_from('<H',source,p)[0]));p+=2
            assert p==len(source)==577
        for i,value in writes:frame[i]=value;gate[i]=int(map_id==303)
        actual=(out/f'{index}-frame.bin').read_bytes();assert actual==frame.tobytes(),('colors',row)
        assert (out/f'{index}-gate.bin').read_bytes()==gate,('gate',row)
        vectors.append({**{k:int(v) for k,v in row.items()},'source_sha256':sha(out/f'source-{kind}.bin'),'frame_sha256':digest(actual),'gate_sha256':digest(gate),'written_pixels':len(writes)})
    assert actual_cases=={(t,p,m) for t in [0,1,2,3,6,7,8,9,16,17,18,19,22,23,34,35] for p in [0,1] for m in [302,303]}
    save(out/'vectors.json',vectors);stable();idle();receipt.update(status='qualified_original_floor_dispatch64_and_gate',source_inputs_unchanged=True,original_bytes=1225,reviewed_relocations=12,cases=64,color_and_gate_cells=52568064,surface_Unlock_stubbed=True,product_changes=0);persist()
except BaseException as error:receipt.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name!='raw-evidence.zip'};hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(out/'raw-evidence.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw-evidence.zip') as z:
        assert set(z.namelist())==set(hashes)
        for n,h in hashes.items():assert digest(z.read(n))==h,n
    save(out/'archive-readback.json',{'status':receipt['status'],'files':hashes,'sha256':sha(out/'raw-evidence.zip'),'verified':True});save(APP/'floor-gate-original-current.json',{'status':receipt['status'],'path':str(out),'whole_goal_complete':False})
print(json.dumps({'status':receipt['status'],'path':str(out),'cases':64,'color_and_gate_cells':52568064}))
