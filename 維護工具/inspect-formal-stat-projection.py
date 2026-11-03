"""Read formal75 directory projection bytecode only; execute no server class."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';P=R/'LiTo381-Modern'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';expected='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371';assert sha(jar)==expected
assert not {n:os.environ.get(n)for n in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS']if os.environ.get(n)}
names=['NativeCharacterStats','NativeCharacterStats$DirectoryProjection','ExtendedCharacterStatsBridge','MemoryAccountRepository']
with zipfile.ZipFile(jar)as z:
 for n in names:assert 'local/lito381/server/'+n+'.class'in z.namelist()
out=V/'native-stat-projection-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(h):
 ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(x)for x in ts])
 ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
 return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
records=[];files={}
for name in names:
 command=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-p','-c','-classpath',str(jar),'local.lito381.server.'+name]
 with(out/(name+'.stdout.txt')).open('wb')as stdout,(out/(name+'.stderr.txt')).open('wb')as stderr:
  child=subprocess.Popen(command,cwd=out,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
  row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,exit_code=child.wait(),original_retained_handle_terminal=child.poll()is not None);records.append(row)
  assert row['exit_code']==0
 for suffix in ['.stdout.txt','.stderr.txt']:files[name+suffix]=sha(out/(name+suffix))
assert sha(jar)==expected
record=dict(status='formal75_projection_static_bytecode_only',formal_jar=str(jar),formal_jar_sha256=expected,classes=names,raw_files=files,
 original_javap_processes=records,server_class_executed=False,actual_network_sessions=0,SQL_access=False,stat_continuity_verified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'readback.json',record)
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n in [*files,'inspector.py','readback.json']:z.write(out/n,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(out/n)for n in z.namelist())
print(json.dumps(dict(contract=str(out),classes=names,server_or_network_started=False),ensure_ascii=False))
