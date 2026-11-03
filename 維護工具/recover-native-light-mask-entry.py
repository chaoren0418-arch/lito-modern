"""Recover only entry observation after a strict encoding error; retain test results."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import zipfile

ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-light-mask')
OUT=run/'entry-recovery';OUT.mkdir(exist_ok=False)
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def archive(p,files):
    pins={n:sha(f) for n,f in files.items()}
    with zipfile.ZipFile(p,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,f in files.items():z.write(f,n)
    with zipfile.ZipFile(p) as z:
        assert set(z.namelist())==set(pins)
        for n,h in pins.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
    return {'path':str(p),'sha256':sha(p),'file_pins':pins,'readback_verified':True}
r=read(run/'receipt.json')
assert r['status']=='failed' and r['phase']=='Debug-entry' and 'UnicodeDecodeError' in r['error']
assert r['Debug-entry_exit_code']==0 and r['active_pid'] is None
assert set(r['profiles'])=={'Debug','Release'}
for profile,record in r['profiles'].items():
    assert r[profile+'-compile_exit_code']==0
    for name,count in [('native_light_mask',5),('native_light_spell',7)]:
        t=record['tests'][name]
        assert (t['passed'],t['failed'],t['ignored'])==(count,0,0)
        assert r[profile+'-'+name+'_exit_code']==0
        text=(run/(profile+'-'+name+'.stdout')).read_text(encoding='utf-8-sig')
        assert re.search(r'test result: ok\. '+str(count)+r' passed; 0 failed; 0 ignored;',text)
pins=read(run/'source-inputs.json');assert len(pins)==974
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
stable()
decoded=(run/'Debug-entry.stdout').read_bytes().decode('cp950')
assert decoded.strip()==str(ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe')
(OUT/'first-Debug-entry-console-cp950-decoded.txt').write_text(decoded,encoding='utf-8')
(OUT/'first-Debug-entry-build.log').write_bytes((ROOT/'驗證紀錄/current/build.log').read_bytes())
(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
state={'status':'running','original_failed_receipt_sha256':sha(run/'receipt.json'),
    'original_raw_archive_sha256':read(run/'raw-retention.json')['sha256'],
    'actual_tests_rerun':False,'first_Debug_entry_was_successful':True,'first_stdout_encoding':'cp950',
    'source_manifest_sha256':sha(run/'source-inputs.json'),'profiles':{},'active_pid':None,'whole_goal_complete':False}
env=os.environ.copy();tools=Path(r'C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin')
env.update(RUSTC=str(tools/'rustc.exe'),RUSTDOC=str(tools/'rustdoc.exe'),
    TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),
    PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8');env['PATH']=str(tools)+os.pathsep+env['PATH']
for profile in ['Debug','Release']:
    stable()
    script=str(ROOT/'啟動現代客戶端.ps1').replace("'","''")
    command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); $OutputEncoding=[Text.UTF8Encoding]::new($false); & '"+script+"' -BuildOnly -BuildProfile "+profile
    args=['pwsh','-NoProfile','-EncodedCommand',base64.b64encode(command.encode('utf-16le')).decode('ascii')]
    save(OUT/(profile+'-entry-command.json'),{'args':args,'decoded_command':command,'entry':str(ROOT/'啟動現代客戶端.ps1'),'BuildOnly':True,'BuildProfile':profile,'forced_console_encoding':'utf-8'})
    with (OUT/(profile+'-entry.stdout')).open('wb') as out,(OUT/(profile+'-entry.stderr')).open('wb') as err:
        child=subprocess.Popen(args,cwd=ROOT/'LiTo381-Modern',env=env,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW)
        state['active_pid']=child.pid;state['active_profile']=profile;save(OUT/'receipt.json',state)
        code=child.wait()
    state['active_pid']=None;assert code==0,(profile,code)
    text=(OUT/(profile+'-entry.stdout')).read_text(encoding='utf-8-sig')
    image=ROOT/f'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/{profile.lower()}/LiToModern.exe'
    assert text.strip()==str(image)
    log=ROOT/('驗證紀錄/current/build-release.log' if profile=='Release' else '驗證紀錄/current/build.log')
    (OUT/(profile+'-entry-build.log')).write_bytes(log.read_bytes())
    state['profiles'][profile]={'path':str(image),'sha256':sha(image),'unique_entry':True,'features':[],
        'machine':'AMD64','optional_header':'PE32+','GUI_revalidated':False,'entry_exit_code':code,
        'entry_build_log_sha256':sha(OUT/(profile+'-entry-build.log'))}
    stable();save(OUT/'receipt.json',state)
state['status']='completed_unique_entry_mains_after_observation_encoding_recovery';state['active_profile']=None
state['normal_archive']=archive(OUT/'normal-mains.zip',{p+'/LiToModern.exe':Path(v['path']) for p,v in state['profiles'].items()})
save(OUT/'receipt.json',state)
retained=archive(OUT/'raw-evidence.zip',{p.name:p for p in OUT.iterdir() if p.is_file() and p.suffix!='.zip'})
save(OUT/'qualified.json',{**state,'raw_archive':retained})
combined={**r,'status':'completed_mask_and_light_authority_scoped_checks_and_unique_entry_mains','phase':None,
    'source_test_pipeline_original_status':'failed_on_entry_stdout_encoding_after_successful_tests_and_Debug_build',
    'normal_mains':state['profiles'],'normal_archive':state['normal_archive'],'raw_archive':read(run/'raw-retention.json'),
    'entry_recovery':str(OUT/'qualified.json'),'entry_recovery_sha256':sha(OUT/'qualified.json'),
    'actual_tests_rerun':False,'executed_original_mask_cases_per_profile':330,
    'original_native_mask_oracle':'驗證紀錄/current/native-lighting-mask-compositor/qualified.json',
    'product_feature_scope':'Original source.lmp loader and clipped max mask library; scene selection, ambient and framebuffer pass remain unconnected.',
    'full_default_integration_regression_verified':False,'full_library_regression_verified':False,'full_current_TCP_regression_verified':False}
save(run/'qualified.json',combined)
print(json.dumps({'status':combined['status'],'run':str(run),'normal_mains':{p:v['sha256'] for p,v in state['profiles'].items()},'tests_rerun':False},ensure_ascii=False))
