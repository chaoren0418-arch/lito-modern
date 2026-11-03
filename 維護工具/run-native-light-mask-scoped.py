"""Retain exact sources/products and execute the mask and light authority checks."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
TOOLS = Path(r'C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin')
OUT = ROOT / '驗證紀錄/current/native-light-mask' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
OUT.mkdir(parents=True, exist_ok=False)
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
def sha(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def save(path,value):
    path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def archive(path,files):
    pins={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for name,p in files.items():
            z.write(p,name)
    with zipfile.ZipFile(path) as z:
        assert set(z.namelist())==set(pins)
        for name,value in pins.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value,name
    return {'path':str(path),'sha256':sha(path),'file_pins':pins,'readback_verified':True}

old=read(ROOT/'驗證紀錄/current/native-light-spell/normal/source-inputs.json')
new_names=['LiTo381-Modern/src/modern_lito/native_light_mask.rs','LiTo381-Modern/tests/native_light_mask.rs',
    'LiTo381-Modern/tests/fixtures/native-light-source.lmp','LiTo381-Modern/tests/fixtures/native-light-mask-original.json']
names=sorted(set(old)|set(new_names))
pins={n:sha(ROOT/n) for n in names}
assert len(pins)==974
assert [n for n in old if old[n]!=pins[n]]==['LiTo381-Modern/src/modern_lito/mod.rs']
save(OUT/'source-inputs.json',pins)
(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
state={'status':'starting','output':str(OUT),'source_files':len(pins),'source_manifest_sha256':sha(OUT/'source-inputs.json'),
    'features':[],'tests':['native_light_mask','native_light_spell'],'profiles':{},'normal_mains':{},
    'whole_goal_complete':False,'GUI_verified':False,'night_world_rendering_integrated':False,'active_pid':None}
def persist(): save(OUT/'receipt.json',state)
def stable():
    for n,v in pins.items(): assert sha(ROOT/n)==v,('Source drift',n)
state['source_archive']=archive(OUT/'compiled-source.zip',{n:ROOT/n for n in pins})
env=os.environ.copy()
env.update(RUSTC=str(TOOLS/'rustc.exe'),RUSTDOC=str(TOOLS/'rustdoc.exe'),
    TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),
    PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
env['PATH']=str(TOOLS)+os.pathsep+env['PATH']
def execute(args,stem,decode_text=True):
    stable()
    save(OUT/(stem+'-command.json'),{'args':args,'cwd':str(PROJECT)})
    with (OUT/(stem+'.stdout')).open('wb') as out,(OUT/(stem+'.stderr')).open('wb') as err:
        child=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW)
        state['active_pid']=child.pid;state['phase']=stem;persist()
        code=child.wait()
    state['active_pid']=None;state[stem+'_exit_code']=code;persist()
    stable()
    assert code==0,(stem,code)
    raw=(OUT/(stem+'.stdout')).read_bytes()
    return raw.decode('utf-8-sig') if decode_text else raw
def pe64(path):
    raw=path.read_bytes();at=struct.unpack_from('<I',raw,60)[0]
    assert raw[at:at+4]==b'PE\0\0' and struct.unpack_from('<H',raw,at+4)[0]==0x8664 and struct.unpack_from('<H',raw,at+24)[0]==0x20b

persist()
try:
    for profile in ['Debug','Release']:
        command=[str(TOOLS/'cargo.exe'),'test','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc',
            '--target-dir','target/current','--no-run','--message-format=json','--test','native_light_mask','--test','native_light_spell']
        if profile=='Release':command.append('--release')
        text=execute(command,profile+'-compile')
        artifacts=[json.loads(line) for line in text.splitlines() if line.startswith('{')]
        images={m['target']['name']:Path(m['executable']) for m in artifacts if m.get('reason')=='compiler-artifact'
            and m.get('executable') and m['profile']['test'] and m['target']['name'] in state['tests']}
        assert set(images)==set(state['tests'])
        for path in images.values(): pe64(path)
        products=archive(OUT/(profile+'-test-products.zip'),{n+'.exe':p for n,p in images.items()})
        result={}
        for name,path in images.items():
            text=execute([str(path),'--test-threads=1','--nocapture'],profile+'-'+name)
            matches=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
            assert len(matches)==1
            passed,failed,ignored=map(int,matches[0]);assert failed==ignored==0
            assert passed=={'native_light_mask':5,'native_light_spell':7}[name]
            result[name]={'passed':passed,'failed':failed,'ignored':ignored,'exe_sha256':sha(path)}
        state['profiles'][profile]={'tests':result,'products':products};persist()
    for profile in ['Debug','Release']:
        execute(['pwsh','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile],profile+'-entry',decode_text=False)
        log=ROOT/('驗證紀錄/current/build-release.log' if profile=='Release' else '驗證紀錄/current/build.log')
        (OUT/(profile+'-entry-build.log')).write_bytes(log.read_bytes())
        image=PROJECT/f'target/current/x86_64-pc-windows-msvc/{profile.lower()}/LiToModern.exe'
        pe64(image)
        state['normal_mains'][profile]={'path':str(image),'sha256':sha(image),
            'entry_build_log_sha256':sha(OUT/(profile+'-entry-build.log')),'unique_entry':True,
            'machine':'AMD64','optional_header':'PE32+','features':[],'GUI_revalidated':False}
        persist()
    state['normal_archive']=archive(OUT/'normal-mains.zip',{profile+'/LiToModern.exe':Path(v['path']) for profile,v in state['normal_mains'].items()})
    stable();state['phase']=None;state['status']='completed_mask_and_light_authority_scoped_checks_and_unique_entry_mains';persist()
except BaseException as exc:
    state['status']='failed';state['error']=repr(exc);persist();raise
finally:
    # Preserve failures too; the exact source archive is already retained.
    raw_files={p.relative_to(OUT).as_posix():p for p in OUT.iterdir() if p.is_file() and p.suffix!='.zip'}
    retained=archive(OUT/'raw-evidence.zip',raw_files)
    save(OUT/'raw-retention.json',retained)
if state['status'].startswith('completed'):
    q={**state,'raw_archive':retained,'executed_original_mask_cases_per_profile':330,
        'original_native_mask_oracle':'驗證紀錄/current/native-lighting-mask-compositor/qualified.json',
        'product_feature_scope':'Original source.lmp loader and reusable clipped max mask library; scene/ambient selection and framebuffer pass not connected.',
        'full_default_integration_regression_verified':False,'full_library_regression_verified':False,'full_current_TCP_regression_verified':False}
    save(OUT/'qualified.json',q)
print(json.dumps({'status':state['status'],'output':str(OUT),'profiles':list(state['profiles'])},ensure_ascii=False))
