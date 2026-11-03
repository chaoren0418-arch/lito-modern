from r491_common import *
import subprocess,time,os,sys,shutil,ctypes,socket
from ctypes import wintypes
sys.dont_write_bytecode=True
v=read(L/'r508c-java-inputs.json');W=Path(v['work']);R=Path(v['runtime']);assert R.parent==Path(r'D:\lito-dev-runtime\20261003-supervision\modern') and R.name.startswith('nw-')
J=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin');pwsh=Path('C:/Users/CHAOREN/.cache/codex-runtimes/codex-primary-runtime/dependencies/native/powershell/pwsh.exe')
env={k:os.environ[k] for k in ['SystemRoot','WINDIR'] if k in os.environ};env.update(TEMP=str(R),TMP=str(R),PATH=str(Path(os.environ['SystemRoot'])/'System32'))
cp=';'.join(str(q) for q in [W/'test-classes',W/'candidate/LiTo381-Server-letter-auth-wire-candidate.jar',*sorted((W/'candidate/lib').glob('*.jar'))])
shutil.copyfile(P/'tools/R508NativeFixtureMain.java',W/'source/R508NativeFixtureMain.java')
sources=[W/'source'/n for n in ['ModernSettlementMysqlFixture.java','NativeLetterAuthenticatedWireMain.java','R508NativeFixtureMain.java']]
q=subprocess.run([str(J/'javac.exe'),'-J-Djava.io.tmpdir='+str(R),'-g','--release','17','-encoding','UTF-8','-sourcepath','','-cp',cp,'-d',str(W/'test-classes'),*[str(p) for p in sources]],cwd=W,env=env,capture_output=True,timeout=40)
(L/'r508c-final-java-compile.stdout.log').write_bytes(q.stdout);(L/'r508c-final-java-compile.stderr.log').write_bytes(q.stderr);assert q.returncode==0,q.stderr.decode('utf-8','replace')
pins=[pin(p) for d in ['source','candidate','test-classes','ddl','db'] for p in sorted((W/d).rglob('*')) if p.is_file()]
candidate=read(L/'r508c-consumer-candidate.json')
for e in candidate.values():
 if isinstance(e,dict) and 'sha256' in e:assert pin(e['path'])['sha256']==e['sha256']
snapshot=read(candidate['production_snapshot']['path'])
for e in snapshot['source_pins']:assert pin(e['path'])['sha256']==e['sha256']==pin(e['snapshot'])['sha256']
save(L/'r508c-java-execution-inputs.json',dict(files=pins,consumer=candidate,fixture_argv=[str(J/'java.exe'),'-Djava.io.tmpdir='+str(R),'-cp','<pinned copied testclasses/JAR/libs>','local.lito381.server.R508NativeFixtureMain',str(R),str(W)],all_explicit_writes_under_owned_modern=True,successful_cleanup_disabled=True))
def wait_file(p,child,seconds):
 deadline=time.monotonic()+seconds
 while not p.exists():
  assert child.poll() is None,('owned child exited',child.returncode,p)
  assert time.monotonic()<deadline,('owned wait deadline',p)
  time.sleep(.05)
 return read(p)
def ft(pid):
 k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE,*([ctypes.POINTER(wintypes.FILETIME)]*4)];k.CloseHandle.argtypes=[wintypes.HANDLE]
 h=k.OpenProcess(0x1000,False,pid);assert h
 try:
  values=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(x) for x in values]);return str((values[0].dwHighDateTime<<32)|values[0].dwLowDateTime)
 finally:k.CloseHandle(h)
def probe(name,ids,ports):
 # Observe only exact owned handles and the allocated loopback ports. No
 # PID/name-derived stop authority is granted by this diagnostic.
 script="$ErrorActionPreference='Stop'; $ids=@("+','.join(map(str,ids))+"); $ports=@("+','.join(map(str,ports))+"); $owners=@(Get-CimInstance Win32_Process | Where-Object{$_.ProcessId -in $ids} | Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine,CreationDate); $listeners=@(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object{$_.LocalPort -in $ports} | Select-Object LocalAddress,LocalPort,OwningProcess); @{owners=$owners;listeners=$listeners} | ConvertTo-Json -Depth 8"
 q=subprocess.run([str(pwsh),'-NoProfile','-Command',script],capture_output=True,timeout=30,env=env)
 assert q.returncode==0,q.stderr.decode('utf-8','replace');d=json.loads(q.stdout.decode('utf-8-sig'));save(R/name,d);return d
helper=None;console=None;status={'runtime':str(R),'historical_PASS_reruns':0,'GUI':0};native_pid=None;ports=[]
try:
 with (R/'r508-helper.stdout.log').open('wb') as out,(R/'r508-helper.stderr.log').open('wb') as err:
  helper=subprocess.Popen([str(J/'java.exe'),'-Djava.io.tmpdir='+str(R),'-cp',cp,'local.lito381.server.R508NativeFixtureMain',str(R),str(W)],cwd=R,env=env,stdout=out,stderr=err)
  status['helper']={'pid':helper.pid,'filetime':ft(helper.pid)};save(L/'r508c-integration-live-status.json',status)
  ready=wait_file(R/'r508-ready.json',helper,100);ports=[ready['port'],ready['sql_port']]
  q=subprocess.run([sys.executable,'-B',str(W/'source/R507OwnedConsoleServer.py'),'--datadir-id',ready['physical_datadir']],capture_output=True,env=env,timeout=10);assert q.returncode==0;qid=q.stdout.decode().strip();save(R/'physical-datadir-id.json',{'datadir':ready['physical_datadir'],'win32_volume_file_index_creation':qid})
  # A successful bind confirms both Native listen addresses are currently idle.
  probes=[]
  for host in ['127.0.0.1','127.0.0.2']:
   s=socket.socket();s.bind((host,ready['port']));probes.append(s)
  for s in probes:s.close()
  save(R/'idle-native-port-proof.json',{'port':ready['port'],'addresses':['127.0.0.1','127.0.0.2'],'bind_succeeded_before_owned_server':True})
  phase=R/'generation-1'
  with (phase/'launcher.stdout.log').open('wb') as lo,(phase/'launcher.stderr.log').open('wb') as le:
   console=subprocess.Popen([sys.executable,'-B',str(W/'source/R507OwnedConsoleServer.py'),str(phase),str(W/'candidate/LiTo381-Server-letter-auth-wire-candidate.jar'),str(R/'native-server.properties')],cwd=R,env=env,stdout=lo,stderr=le)
   owner=wait_file(phase/'server-console-owner.json',console,20);native_pid=owner['pid'];assert ft(native_pid)==owner['creation_filetime_utc']
   deadline=time.monotonic()+50
   while '[LiTo] ready' not in (phase/'server.stdout.log').read_text(encoding='utf-8',errors='replace'):
    assert console.poll() is None;assert time.monotonic()<deadline;time.sleep(.1)
   ids=[helper.pid,ready['mysql_pid'],console.pid,native_pid];live=probe('r508-owned-live.json',ids,ports)
   assert len(live['owners'])==4 and len(live['listeners'])==3
   status.update(state='READY_OWNED_JAVA_NATIVE_MAIN',ready=ready,owned_pids=ids);save(L/'r508c-integration-live-status.json',status)
   with (L/'r508c-consumer.stdout.log').open('wb') as co,(L/'r508c-consumer.stderr.log').open('wb') as ce:
    t=time.monotonic();q=subprocess.run([candidate['executable']['path'],str(P),str(R)],cwd=P,env=env,stdout=co,stderr=ce,timeout=120)
   status.update(consumer_exit=q.returncode,consumer_seconds=time.monotonic()-t);save(L/'r508c-integration-live-status.json',status)
finally:
 if console is not None and console.poll() is None:
  (R/'generation-1/normal-stop.request').write_text('owned new integration complete',encoding='ascii')
  try:console.wait(timeout=45)
  except subprocess.TimeoutExpired:
   # Never stop other servers by PID/name. The owning launcher retains its
   # own child handle and has a finite failure-cleanup deadline.
   status['console_retirement_pending']=True
 if helper is not None and helper.poll() is None:
  (R/'r508-stop.request').write_text('owned stop',encoding='ascii')
  try:helper.wait(timeout=20)
  except subprocess.TimeoutExpired:
   assert ft(helper.pid)==status['helper']['filetime'];helper.terminate();helper.wait(timeout=10);status['helper_forced_failure_retirement']=True
 status.update(helper_exit=helper.returncode if helper else None,console_exit=console.returncode if console else None)
 if ports:
  retirement=probe('r508-owned-retirement.json',status.get('owned_pids',[helper.pid] if helper else []),ports);status['all_owned_retired']=not retirement['owners'];status['all_owned_listeners_released']=not retirement['listeners']
 if (R/'r508-sql-retirement.json').exists():status['SQL_retirement']=read(R/'r508-sql-retirement.json')
 if (R/'generation-1/normal-stop-result.json').exists():status['native_normal_stop']=read(R/'generation-1/normal-stop-result.json')
 status['stdout']=pin(L/'r508c-consumer.stdout.log') if (L/'r508c-consumer.stdout.log').exists() else None;status['stderr']=pin(L/'r508c-consumer.stderr.log') if (L/'r508c-consumer.stderr.log').exists() else None
 save(L/'r508c-integration-exit.json',status)
 for e in pins:assert pin(e['path'])['sha256']==e['sha256'],e['path']
 for e in snapshot['source_pins']:assert pin(e['path'])['sha256']==e['sha256']==pin(e['snapshot'])['sha256']
assert status.get('consumer_exit')==0 and status.get('all_owned_retired') and status.get('all_owned_listeners_released') and status.get('native_normal_stop',{}).get('forced_termination') is False,status
print(json.dumps({'consumer_exit':status['consumer_exit'],'runtime':str(R),'owned_retired':True,'listeners_released':True,'full_native_login_channel_path':False}))
