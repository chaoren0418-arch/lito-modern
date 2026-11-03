"""Build a diagnostic against pinned current Debug rlibs, retain actual RGBA/PNG."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,struct,subprocess,zipfile,zlib
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
CORE=APP/'actual-tests-20261016-082957-233653'
TOOLS=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(CORE/'compiled-inputs.json');assert len(pins)==1002 and read(CORE/'qualified.json')['passed']==76
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
stable();deps=PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps';libs={};native_paths=set()
for line in (CORE/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    v=json.loads(line)
    if v.get('reason')=='build-script-executed':native_paths.update(v.get('linked_paths',[]))
    if v.get('reason')=='compiler-artifact' and v['target']['name'] in ['lito_modern','serde_json']:
        matches=[Path(n) for n in v['filenames'] if n.endswith('.rlib') and Path(n).parent==deps]
        if matches:assert len(matches)==1;libs[v['target']['name']]=matches[0]
assert set(libs)=={'lito_modern','serde_json'}
out=ROOT/'驗證紀錄/current/native-water-scene-preview'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-water-scene-preview';runtime.mkdir(exist_ok=True)
source=ROOT/'維護工具/oracle/NativeWaterScenePreview.rs';(out/'preview.rs').write_bytes(source.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',pins)
binary=runtime/'water-preview.exe';command=[str(TOOLS/'rustc.exe'),str(out/'preview.rs'),'--edition=2021','--crate-name','water_scene_preview','--target','x86_64-pc-windows-msvc','-L','dependency='+str(deps),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(binary)]
for name,p in libs.items():command+=['--extern',name+'='+str(p)]
for path in sorted(native_paths):command+=['-L',path]
env=os.environ.copy();env.update(RUSTC=str(TOOLS/'rustc.exe'),RUSTDOC=str(TOOLS/'rustdoc.exe'),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
receipt={'status':'compiling_current_library_diagnostic','owned_process':None,'source_inputs':1002,'libraries':{n:{'path':str(p),'sha256':sha(p)} for n,p in libs.items()},'normal_main_GUI':False,'game_sessions':0,'whole_goal_complete':False}
def execute(args,name):
    save(out/(name+'-command.json'),{'args':args})
    with (out/(name+'.stdout')).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:
        child=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        receipt['owned_process']={'pid':child.pid,'args':args};save(out/'receipt.json',receipt)
        code=child.wait();receipt['owned_process']=None;receipt[name+'_exit_code']=code;save(out/'receipt.json',receipt)
    assert code==0,(name,code)
def png(raw,width,height):
    def chunk(name,data):return struct.pack('>I',len(data))+name+data+struct.pack('>I',zlib.crc32(name+data)&0xffffffff)
    scan=b''.join(b'\0'+raw[y*width*4:(y+1)*width*4] for y in range(height))
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(scan))+chunk(b'IEND',b'')
try:
    execute(command,'build');receipt['status']='rendering_actual_product_library';save(out/'receipt.json',receipt)
    execute([str(binary),str(Path(r'D:\lineage開發\0-客戶端')),str(out),str(PROJECT/'tests/fixtures/native-world-current.json')],'render')
    outputs={}
    for name in ['dry','water-phase0','water-phase1']:
        raw=(out/(name+'.rgba')).read_bytes();assert len(raw)==800*600*4
        frame=read(out/(name+'.json'));assert hashlib.sha256(raw).hexdigest().lower()==frame['rgba_sha256'].lower()
        (out/(name+'.png')).write_bytes(png(raw,800,600));outputs[name]={'raw_sha256':sha(out/(name+'.rgba')),'png_sha256':sha(out/(name+'.png'))}
    assert len({v['raw_sha256'] for v in outputs.values()})==3
    stable();receipt.update(status='actual_product_library_water_images_retained',outputs=outputs,exe_sha256=sha(binary));save(out/'receipt.json',receipt)
except BaseException as error:
    receipt.update(status='failed_preserved',error=repr(error));save(out/'receipt.json',receipt);raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if binary.exists() and receipt.get('build_exit_code')==0:files['diagnostic.exe']=binary
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw.zip') as z:
        for n,p in files.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p),n
    save(out/'archive-readback.json',{'status':'raw_archive_readback_verified','archive_sha256':sha(out/'raw.zip'),'files':{n:sha(p) for n,p in files.items()}})
save(APP/'water-scene-preview-current.json',{'output':str(out),'status':receipt['status'],'normal_main_GUI':False,'whole_goal_complete':False})
print(json.dumps({'output':str(out),'status':receipt['status']}))
