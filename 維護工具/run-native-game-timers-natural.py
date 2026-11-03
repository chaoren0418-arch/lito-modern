"""Run exact original callback intervals on a retained x64 test image and C02."""
from datetime import datetime,timezone
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time
import zipfile

ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-game-timers')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
scope=read(run/'receipt.json');profile=a.profile
assert profile in scope['profiles'] and scope['source_files']==976
assert scope['profiles'][profile]['tests']['native_game_timers']['passed']==3
assert scope['profiles'][profile]['tests']['native_game_timers']['ignored']==2
pins=read(run/'source-inputs.json')
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
stable()
OUT=run/'natural'/profile;OUT.mkdir(parents=True,exist_ok=False)
RUNTIME=ROOT/'LiTo381-Modern/target/current/native-game-timers-harness'/profile
RUNTIME.mkdir(parents=True,exist_ok=True)
JDK=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin')
JAR=ROOT/'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'
assert sha(JAR)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
source=ROOT/'維護工具/oracle/NativeGameTimerHarness.java'
(OUT/'harness.java').write_bytes(source.read_bytes());(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(scope['profiles'][profile]['products']['path']) as z:
    raw=z.read('native_game_timers.exe')
assert hashlib.sha256(raw).hexdigest().upper()==scope['profiles'][profile]['tests']['native_game_timers']['exe_sha256']
image=OUT/'timer-tests.exe';image.write_bytes(raw)
env=os.environ.copy();env.update(PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',
    TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'))
state={'status':'initializing','profile':profile,'source_files':976,'source_manifest_sha256':sha(run/'source-inputs.json'),
    'test_exe_sha256':sha(image),'reference_sha256':sha(JAR),'source_must_remain_frozen':True,
    'intervals_modified':False,'whole_goal_complete':False,'GUI_verified':False,'children':{},'out':str(OUT)}
handles=[];streams=[]
def save():
    (OUT/'receipt.json').write_text(json.dumps(state,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def launch(name,args,cwd,child_env):
    out=(OUT/(name+'.stdout')).open('wb');err=(OUT/(name+'.stderr')).open('wb');streams.extend([out,err])
    p=subprocess.Popen(args,cwd=cwd,env=child_env,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW)
    handles.append(p);state['children'][name]={'pid':p.pid,'started_utc':datetime.now(timezone.utc).isoformat(),'args':args,'cwd':str(cwd),'exit_code':None};save();return p
def complete(name,p):
    code=p.poll();state['children'][name]['exit_code']=code;save();assert code==0,(name,code)
save()
try:
    compile=launch('javac',[str(JDK/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(JAR),'-d',str(RUNTIME),str(source)],RUNTIME,env)
    compile.wait(timeout=40);complete('javac',compile)
    server_root=OUT/'formal-server';server_root.mkdir()
    catalog=ROOT/'LiTo381-Modern/target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
    assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
    server=launch('formal-server',[str(JDK/'java.exe'),'-Dfile.encoding=UTF-8','-Xverify:all','-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',str(RUNTIME)+os.pathsep+str(JAR),'local.lito381.server.NativeGameTimerHarness',str(server_root)],server_root,env)
    deadline=time.monotonic()+20
    while not (server_root/'ready.json').exists():
        assert server.poll() is None,'Owned fixture failed before ready'
        assert time.monotonic()<deadline,'Fixture ready timeout'
        time.sleep(.05)
    ready=read(server_root/'ready.json');assert ready['jar_sha256'].upper()==sha(JAR)
    formal=OUT/'formal-client';formal.mkdir()
    formal_env=env.copy();formal_env.update(LITO_GAME_TIMER_LOGIN='127.0.0.1:'+str(ready['port']),LITO_GAME_TIMER_OUTPUT=str(formal))
    formal_test=launch('formal-client',[str(image),'--ignored','--exact','formal_character_admission_natural_callbacks_cancel_and_reauthenticate','--nocapture','--test-threads=1'],ROOT/'LiTo381-Modern',formal_env)
    stress_test=launch('backpressure-client',[str(image),'--ignored','--exact','natural_callbacks_continue_with_a_full_inbound_event_queue_and_no_UI_polling','--nocapture','--test-threads=1'],ROOT/'LiTo381-Modern',env)
    state['status']='running_original_intervals_formal_and_backpressure';save()
    deadline=time.monotonic()+470
    while formal_test.poll() is None or stress_test.poll() is None:
        assert server.poll() is None or formal_test.poll()==0,'Formal server ended while client still active'
        for name,p in [('formal-client',formal_test),('backpressure-client',stress_test)]:
            code=p.poll()
            if code is not None:assert code==0,(name,code)
            state['children'][name]['exit_code']=code
        state['observed_utc']=datetime.now(timezone.utc).isoformat();save()
        assert time.monotonic()<deadline,'Natural callback test deadline'
        time.sleep(.25)
    complete('formal-client',formal_test);complete('backpressure-client',stress_test)
    server.wait(timeout=12);complete('formal-server',server)
    for f in streams:f.flush()
    for name in ['formal-client','backpressure-client']:
        text=(OUT/(name+'.stdout')).read_text(encoding='utf-8-sig')
        assert re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored;',text),name
    sr=read(server_root/'result.json');assert sr['actual_client_sessions']==4 and sr['memberships_empty'] and sr['authority_mutations']==0
    cr=read(formal/'client-result.json');assert cr['quiescent'] and len(cr['rounds'])==2
    first,second=cr['rounds'];assert first['unadmitted_ms']>=61000 and first['admitted_ms']>=300000
    assert (first['timers']['alive_sent'],first['timers']['save_sent'])==(6,1)
    assert second['epoch']>first['epoch'] and second['admitted_ms']>=60000
    assert (second['timers']['alive_sent'],second['timers']['save_sent'])==(2,0)
    stable();state['status']='completed_original_intervals_formal4_sessions_and_backpressure_no_poll';state['source_must_remain_frozen']=False;save()
except BaseException as error:
    state['status']='failed';state['error']=repr(error);save();raise
finally:
    # These Popen objects retain exact process handles. No PID/name-wide cleanup.
    for p in handles:
        if p.poll() is None:p.terminate()
    for p in handles:
        if p.poll() is None:p.wait(timeout=10)
    for f in streams:f.close()
    files={p.relative_to(OUT).as_posix():p for p in OUT.rglob('*') if p.is_file() and p.suffix!='.zip'}
    assert len(files)<512
    file_pins={n:sha(p) for n,p in files.items()};packed=OUT/'raw-evidence.zip'
    with zipfile.ZipFile(packed,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(packed) as z:
        assert set(z.namelist())==set(file_pins)
        for n,h in file_pins.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
    retained={'archive_sha256':sha(packed),'files':file_pins,'readback_verified':True}
    (OUT/'raw-retention.json').write_text(json.dumps(retained,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
if state['status'].startswith('completed'):
    (OUT/'qualified.json').write_text(json.dumps({**state,'raw':retained,'actual_formal_sessions':4,'optin_tests_passed':2,'new_full_GUI_verified':False,'whole_goal_complete':False},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':state['status'],'out':str(OUT),'profile':profile},ensure_ascii=False))
