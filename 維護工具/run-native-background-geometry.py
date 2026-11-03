"""Build/run retained geometry only after current1005 GUI independently ends."""
from pathlib import Path
from datetime import datetime,timezone
import csv,hashlib,json,os,struct,subprocess,sys,zipfile

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'floor-tests-20261016-102632-227125/normal-qualified'
OUT=Path(sys.argv[1]).resolve(strict=True)
assert OUT.is_relative_to(ROOT/'驗證紀錄/current/native-background-geometry-original')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
now=lambda:datetime.now(timezone.utc).isoformat()

def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()

prepared=read(OUT/'preparation-readback.json')
assert prepared['status']=='prepared_original_background_geometry_not_built_or_executed'
assert sha(OUT/'preparation-raw.zip')==prepared['raw_archive_sha256']
with zipfile.ZipFile(OUT/'preparation-raw.zip') as z:
    assert set(z.namelist())==set(prepared['files'])
    for n,h in prepared['files'].items():assert digest(z.read(n))==sha(OUT/n)==h,n
pins=read(NORMAL/'source-inputs.json');assert len(pins)==1005
nq=read(NORMAL/'qualified.json')

def stable():
    assert sha(NORMAL/'source-inputs.json')==prepared['source_manifest_sha256']
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for p,v in nq['profiles'].items():assert sha(v['path'])==v['sha256']==prepared['normal_mains'][p]

def idle():
    g=read(NORMAL/'GUI-current.json');terminal=read(NORMAL/'GUI-process-completion-observation.json')
    q=read(NORMAL/'GUI-completion-observation.json')
    assert g['status']=='qualified_current_lighting_TW14_GUI_cases' and not g['source_and_corresponding_tests_frozen']
    assert terminal['actual_terminal_tool_observed'] and terminal['terminal_exit_code']==0
    assert terminal['original_pipeline_identity_absent'] and terminal['owned_workloads']==[]
    assert q['status']=='independently_readback_verified_current1005_TW14_GUI_and_terminal_process'
    command="$r='"+str(ROOT).replace("'","''")+"';$p=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -in @('cargo.exe','rustc.exe')) -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|lito_modern-|native_|qualified-client|actual-client)' -or ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))});if($p.Count){exit 1}"
    result=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert result.returncode==0,'Owned GUI/server/compiler/test load remains live'

# Guards precede mutations or process creation. A live GUI must not be disturbed.
stable();idle()
assert not (OUT/'run-receipt.json').exists()
(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
receipt={'status':'building_private_background_geometry','started_utc':now(),'owned_process':None,
    'original_function_bytes':1294,'source_files':1005,'native_execution_started':False,
    'real_pixel_kernel_executed':False,'game_sessions':0,'whole_goal_complete':False}
persist=lambda:save(OUT/'run-receipt.json',receipt)
env=os.environ.copy();env.update(TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')

def run(command,name):
    with (OUT/name).open('wb') as stdout,(OUT/(name+'.stderr')).open('wb') as stderr:
        child=subprocess.Popen(command,cwd=OUT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        receipt['owned_process']={'pid':child.pid,'arguments':command,'observed_started_utc':now()};persist()
        code=child.wait();receipt['owned_process']=None;receipt[name+'_exit_code']=code;persist()
    assert code==0,(name,code)

def table(name):
    with (OUT/name).open(encoding='ascii',newline='') as f:return [{k:int(v) for k,v in row.items()} for row in csv.DictReader(f)]

persist()
try:
    run(read(OUT/'build-command.json')['args'],'build.log');stable();idle()
    b=(OUT/'oracle.exe').read_bytes();e=int.from_bytes(b[60:64],'little')
    assert b[:2]==b'MZ' and b[e:e+4]==b'PE\0\0' and b[e+4:e+6]==b'\x4c\x01' and b[e+24:e+26]==b'\x0b\x01'
    receipt.update(status='executing_original_geometry_with_recording_kernel',native_execution_started=True);persist()
    run([str(OUT/'oracle.exe'),str(OUT)],'native-summary.json')
    native=read(OUT/'native-summary.json')
    assert native=={'status':'original_geometry_ran_with_recording_pixel_stub','cases':2688,'recorded_calls':17472,'data_relocations':31,'error_call_relocations':15,'unexpected_loader_or_error_calls':0,'guards_unchanged':True}
    assert table('calls-native.csv')==table('calls-expected.csv')
    assert table('counts-native.csv')==table('counts-expected.csv')
    extraction=read(OUT/'extraction.json');original=(OUT/'geometry-original.bin').read_bytes();actual=(OUT/'geometry-relocated.bin').read_bytes()
    assert digest(original)==extraction['original_code_sha256'] and len(original)==len(actual)==1294
    allowed={i for kind,o,key in extraction['relocations'] for i in range(o,o+4)}
    assert all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,actual)))
    with (OUT/'private-addresses.csv').open(encoding='ascii',newline='') as f:
        addresses={(r['kind'],int(r['original'])):int(r['address']) for r in csv.DictReader(f)}
    assert len(addresses)==15
    for kind,offset,key in extraction['relocations']:
        value=struct.unpack_from('<I',actual,offset)[0]
        if kind=='D':assert value==addresses[(kind,key)]
        else:assert (addresses[('B',0x487e10)]+offset+4+value)&0xffffffff==addresses[(kind,key)]
    stable();idle()
    receipt.update(status='original_background_geometry_matches_independent_scalar',cases=2688,recorded_calls=17472,all_allowed_relocations_verified=True,source_inputs_unchanged=True,normal_mains_unchanged=True)
except BaseException as error:
    receipt.update(status='failed_preserved',error=repr(error));raise
finally:
    receipt['finished_utc']=now();persist()
    files={p.name:sha(p) for p in OUT.iterdir() if p.is_file() and p.name not in ['preparation-raw.zip']}
    with zipfile.ZipFile(OUT/'run-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for n in files:z.write(OUT/n,n)
    with zipfile.ZipFile(OUT/'run-raw.zip') as z:
        assert set(z.namelist())==set(files)
        for n,h in files.items():assert digest(z.read(n))==h,n
    save(OUT/'run-readback.json',{'status':receipt['status'],'files':files,'raw_archive_sha256':sha(OUT/'run-raw.zip'),'archive_readback_verified':True,'real_pixel_kernel_executed':False,'full_background_renderer_verified':False,'whole_goal_complete':False})
print(json.dumps({'path':str(OUT),'status':receipt['status'],'cases':2688,'recorded_calls':17472},ensure_ascii=False))
