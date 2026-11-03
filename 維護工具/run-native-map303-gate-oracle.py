"""Run isolated gate writers only after current1002 GUI and owned load exit."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,array,csv,hashlib,json,os,struct,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'actual-tests-20261016-084451-903086/normal-qualified'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('static',type=Path);args=ap.parse_args()
staged=args.static.resolve(strict=True);assert staged.is_relative_to(ROOT/'驗證紀錄/current/native-map303-gate-static')
q=read(staged/'qualified-static.json');assert q['status']=='readonly_map303_gate_provenance_statically_retained' and q['archive_readback_verified']
assert sha(staged/'raw-evidence.zip')==q['archive_sha256']
with zipfile.ZipFile(staged/'raw-evidence.zip') as z:
    external={'auditor.py':ROOT/'維護工具/retain-native-map303-gate-static.py','source-inputs.json':NORMAL/'source-inputs.json','normal-qualified.json':NORMAL/'qualified.json'}
    for n,h in q['file_pins'].items():assert digest(z.read(n))==sha(external.get(n,staged/n))==h,n
pins=read(NORMAL/'source-inputs.json');assert len(pins)==1002
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for v in read(NORMAL/'qualified.json')['profiles'].values():assert sha(Path(v['path']))==v['sha256']
def idle():
    g=read(NORMAL/'GUI-current.json');end=read(NORMAL/'GUI-process-completion-observation.json')
    assert g['status']=='qualified_current_lighting_TW14_GUI_cases' and not g['source_and_corresponding_tests_frozen']
    assert end['actual_terminal_tool_observed'] and end['terminal_exit_code']==0 and end['original_pipeline_identity_absent'] and end['owned_workloads']==[]
    assert read(NORMAL/'GUI-completion-observation.json')['status']=='independently_readback_verified_current1002_TW14_GUI_and_terminal_process'
    command="$r='"+str(ROOT).replace("'","''")+"';$p=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -in @('cargo.exe','rustc.exe')) -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|native_|qualified-client|actual-client)' -or $_.ExecutablePath -match '\\\\deps\\\\.*\\.exe$'))});if($p.Count){exit 1}"
    child=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert child.returncode==0,'Owned GUI/Java/compiler/test still live'
idle();stable()
out=ROOT/'驗證紀錄/current/native-map303-gate-original'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True,exist_ok=False)
for n in ['rle-original.bin','right-original.bin','left-original.bin','extracted-code.json','qualified-static.json','selected-assembly.json']:(out/n).write_bytes((staged/n).read_bytes())
(out/'oracle.cpp').write_bytes((ROOT/'維護工具/oracle/NativeMap303GateOracle.cpp').read_bytes())
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
world=read(out/'selected-assembly.json')['sub_488860']['assembly']
assert 'lea     eax, [ebx-8]' in world and 'lea     eax, [ebx-4]' in world
command=read(ROOT/'驗證紀錄/current/native-lighting-source/build-command.json')['args']
command=[s.replace(str(ROOT/'維護工具/oracle/NativeLightingPrimitiveOracle.cpp'),str(out/'oracle.cpp')).replace(str(ROOT/'LiTo381-Modern/target/current/native-lighting-source-oracle'),str(out)) for s in command]
assert str(out/'oracle.cpp') in command and ('/Fe'+str(out/'oracle.exe')) in command
save(out/'build-command.json',{'args':command,'cwd':str(out)})
env=os.environ.copy();env.update(TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
receipt={'status':'building_private_map303_gate_oracle','owned_process':None,'native_execution':False,'source_files':1002,'game_sessions':0,'whole_goal_complete':False}
persist=lambda:save(out/'receipt.json',receipt)
def run(command,name):
    with (out/name).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:
        c=subprocess.Popen(command,cwd=out,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        receipt['owned_process']={'pid':c.pid,'args':command};persist();code=c.wait()
        receipt['owned_process']=None;receipt[name+'_exit_code']=code;persist()
    assert code==0,(name,code)
def packed(kind):
    x,y=[(0,0),(-7,3),(5,-4)][kind];b=bytearray([x&255,y&255,0,4])
    for spans in [[(3,2),(4,5),(0,0)],[],[(0,12)],[(8,1),(7,3)]]:
        b.append(len(spans))
        for skip,count in spans:
            b.extend([skip,count])
            for _ in range(count):b.extend(struct.pack('<H',0x8001+len(b)%0x7000))
    return b,x,y
persist()
try:
    run(command,'build.log');stable();idle()
    data=(out/'oracle.exe').read_bytes();eh=int.from_bytes(data[60:64],'little')
    assert data[:2]==b'MZ' and data[eh+4:eh+6]==b'\x4c\x01' and data[eh+24:eh+26]==b'\x0b\x01'
    receipt.update(status='executing_private_map303_gate_oracle',native_execution=True);persist()
    run([str(out/'oracle.exe'),str(out)],'oracle.json')
    native=read(out/'oracle.json');assert native['cases']==36 and native['guard_failures']==0 and not native['whole_client_executed']
    addresses={int(a,16):int(b,16) for a,b in (line.split() for line in (out/'addresses.tsv').read_text().splitlines())}
    for n,m in read(out/'extracted-code.json').items():
        original=(out/(n+'-original.bin')).read_bytes();actual=(out/(n+'-relocated.bin')).read_bytes()
        assert digest(original)==m['sha256'] and len(original)==len(actual)==m['bytes']
        allowed={i for r in m['relocations'] for i in range(r['offset'],r['offset']+4)}
        for i,(a,b) in enumerate(zip(original,actual)):assert a==b or i in allowed
        for r in m['relocations']:assert struct.unpack_from('<I',actual,r['offset'])[0]==addresses[r['original_address']]
    cases=list(csv.DictReader((out/'cases.csv').open(newline='',encoding='ascii')));assert len(cases)==36
    counts={};total=0
    for row in cases:
        name=row['function'];pitch=int(row['pitch']);enabled=int(row['enabled']);seed=int(row['seed']);start=int(row['start_pixel']);kind=int(row['kind'])
        assert int(row['reverse'])==0 and int(row['unlock_calls'])==(0 if name=='rle' else 1)
        expected=array.array('H',[0x7777])*(pitch*552)
        if name=='rle':
            data,x,y=packed(kind);at=4
            for rownum in range(4):
                target=start+x+(y+rownum)*pitch;spans=data[at];at+=1
                for _ in range(spans):
                    skip,count=data[at:at+2];at+=2;target+=skip//2
                    for _ in range(count):expected[target]=struct.unpack_from('<H',data,at)[0];at+=2;target+=1
        else:
            source=0;target=start+(22 if name=='right' else 0);stride=pitch-(4 if name=='right' else 2);width=2
            for _ in range(12):
                for j in range(width):expected[target+j]=0x8001+source+j
                source+=width;target+=width+stride;stride-=2;width+=2
            target+=4 if name=='right' else 0;stride+=8 if name=='right' else 4;width=22
            for _ in range(11):
                for j in range(width):expected[target+j]=0x8001+source+j
                source+=width;target+=width+stride;stride+=2;width-=2
            assert source==288
        actual=array.array('H');actual.frombytes((out/(row['id']+'-frame.bin')).read_bytes());assert actual==expected,('colors',row)
        gate=bytearray([seed])*(744*552)
        if enabled:
            for i,v in enumerate(expected):
                if v!=0x7777:y,x=divmod(i,pitch);assert x<744;gate[y*744+x]=1
        assert (out/(row['id']+'-gate.bin')).read_bytes()==gate,('gate',row)
        key=(name,pitch,enabled,seed);assert key not in counts;counts[key]=True;total+=len(expected)+len(gate)
    assert len(counts)==36 and total==native['color_and_gate_cells_checked'];stable();idle()
    receipt.update(status='qualified_isolated_original_map303_gate_writers',scalar_color_and_gate_cells=total,all_original_relocations_verified=True,source_inputs_unchanged=True);persist()
except Exception as e:
    receipt.update(status='failed_preserved',error=repr(e));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name!='raw-evidence.zip'};hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(out/'raw-evidence.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw-evidence.zip') as z:
        assert set(z.namelist())==set(hashes)
        for n,h in hashes.items():assert digest(z.read(n))==h,n
    save(out/'archive-readback.json',{'files':hashes,'sha256':sha(out/'raw-evidence.zip'),'verified':True,'status':receipt['status']})
    save(APP/'map303-gate-original-current.json',{'path':str(out),'receipt':str(out/'receipt.json'),'status':receipt['status'],'map303_product_complete':False})
print(json.dumps({'status':receipt['status'],'path':str(out),'cases':36,'scalar_cells':total}))
