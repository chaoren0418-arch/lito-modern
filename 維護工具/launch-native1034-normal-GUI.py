"""Launch one normal client through the unique entry; perform no UI automation."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,time
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--controller-run',type=Path,required=True);ap.add_argument('--role',choices=['a','b'],required=True);a=ap.parse_args()
out=a.controller_run.resolve(strict=True);assert out.is_relative_to(ROOT/'驗證紀錄/current/native1034-interactive-teleport')
receipt=read(out/'receipt.json');assert receipt['status']=='ready_for_interactive_OS_SendInput' and receipt['owned_process'] is not None
normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950';c=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json');profile=receipt['profile']
assert len(inputs)==1035 and c['source_manifest_sha256']==receipt['source_manifest_sha256'];product=c['profiles'][profile]['normal'];exe=Path(product['path'])
assert sha(exe)==receipt['main_sha256']==product['sha256'];assert sha(ROOT/'啟動現代客戶端.ps1')==c['entry_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h
ready=read(out/'ready.json');assert ready['jar_sha256'].upper()==receipt['jar_sha256'];endpoint='127.0.0.1:'+str(int(ready['port']));assert 0<int(ready['port'])<65536
server=receipt['owned_process'];server_pid=int(server['pid']);assert server_pid>0
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$p=Get-CimInstance Win32_Process -Filter 'ProcessId="+str(server_pid)+"';[pscustomobject]@{server=$(if($p){$p|Select-Object ProcessId,CommandLine,@{n='creation_utc';e={$_.CreationDate.ToUniversalTime().ToString('o')}}});listeners=@(Get-NetTCPConnection -State Listen -LocalAddress 127.0.0.1 -LocalPort "+str(int(ready['port']))+"|Select-Object OwningProcess)}|ConvertTo-Json -Compress -Depth 4"
live=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',ps],creationflags=subprocess.CREATE_NO_WINDOW).decode('utf-8-sig'));assert live['server']['ProcessId']==server_pid
assert abs((datetime.fromisoformat(live['server']['creation_utc'].replace('Z','+00:00'))-datetime.fromisoformat(server['creation_utc'])).total_seconds())<.001
assert live['server']['CommandLine'].strip()==subprocess.list2cmdline(server['args']) and any(v['OwningProcess']==server_pid for v in live['listeners'])
def processes():
    ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter \"Name='LiToModern.exe'\"|Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine,@{n='creation_utc';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
    rows=subprocess.check_output(['pwsh','-NoProfile','-Command',ps],creationflags=subprocess.CREATE_NO_WINDOW);j=json.loads(rows.decode('utf-8-sig')) if rows.strip() else [];return [j] if isinstance(j,dict) else j
before=processes();folder=out/('normal-launch-'+a.role);folder.mkdir()
state=dict(status='starting_unique_entry',role=a.role,profile=profile,source_manifest_sha256=c['source_manifest_sha256'],normal_product=str(exe),normal_product_sha256=sha(exe),endpoint=endpoint,owned_server_live_confirmation=live,preexisting_processes=before,UI_automation_performed=False,OS_SendInput_verified=False,GUI_acceptance=False,whole_goal_complete=False)
save(folder/'receipt.json',state);(folder/'launcher.py').write_bytes(Path(__file__).read_bytes())
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin';env=dict(os.environ,RUSTC=str(rust/'rustc.exe'),RUSTDOC=str(rust/'rustdoc.exe'),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'));env['PATH']=str(rust)+os.pathsep+env['PATH']
assert not {k:v for k,v in env.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
command=['pwsh','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildProfile',profile,'-NativeLoginEndpoint',endpoint,'-ResourceRoot',r'D:\lineage開發\0-客戶端']
with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
    child=subprocess.Popen(command,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    state['entry_process']=dict(pid=child.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),command=command);save(folder/'receipt.json',state)
    code=child.wait();state['entry_process']['exit_code']=code;save(folder/'receipt.json',state);assert code==0
deadline=time.monotonic()+5
while True:
    rows=[j for j in processes() if j['ParentProcessId']==child.pid and j['ExecutablePath'] and Path(j['ExecutablePath']).resolve()==exe.resolve() and not any(j['ProcessId']==b['ProcessId'] and j['creation_utc']==b['creation_utc'] for b in before)]
    if rows:break
    assert time.monotonic()<deadline,'no uniquely owned normal app discovered; preserve startup logs';time.sleep(.05)
assert len(rows)==1;app=rows[0];assert endpoint in app['CommandLine'] and '--native-login' in app['CommandLine'];assert datetime.fromisoformat(app['creation_utc'].replace('Z','+00:00'))>=datetime.fromisoformat(state['entry_process']['creation_utc'])
for n,h in inputs.items():assert sha(ROOT/n)==h
assert sha(exe)==product['sha256'];state.update(status='normal_GUI_process_started_not_yet_accepted',normal_process=app,unique_entry_sha256=sha(ROOT/'啟動現代客戶端.ps1'));save(folder/'receipt.json',state)
print(json.dumps(dict(receipt=str(folder/'receipt.json'),normal_process=app,GUI_acceptance=False),ensure_ascii=False),flush=True)
