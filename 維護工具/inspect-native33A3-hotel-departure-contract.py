"""Read current formal hotel departure paths; no sessions or authority writes."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'
def sha(path):
    with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
assert sha(jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
out=V/'native33A3-hotel-departure-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
command=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+env['TMP'],'-p','-c','-classpath',str(jar),'local.lito381.server.ClientSession']
result=subprocess.run(command,capture_output=True,env=env,creationflags=subprocess.CREATE_NO_WINDOW,timeout=45)
assert result.returncode==0 and not result.stderr,result.stderr.decode('utf-8','replace')
lines=result.stdout.decode('utf-8').splitlines();heads=[i for i,s in enumerate(lines) if re.match(r'^  (?:public|private|protected) .*\(.*\)(?: throws [^;]+)?;$',s)]
blocks={lines[i].strip():'\n'.join(lines[i:heads[n+1] if n+1<len(heads) else len(lines)]) for n,i in enumerate(heads)}
names=['enterRentalTownPortal','moveRentalTown','prepareRentalHotelDeparture','beginRentalTownMapTransition','completeRentalTownPortal','handleRentalTownHotelAction','serverRentalHotelLeaseInvalidated']
selected={h:b for h,b in blocks.items() if any(re.search(r'\b'+name+r'\(',h) for name in names)}
assert len(selected)==len(names)
with zipfile.ZipFile(jar) as z:class_sha=hashlib.sha256(z.read('local/lito381/server/ClientSession.class')).hexdigest().upper()
record=dict(status='current75_hotel_departure_static_methods_only',reference_jar=str(jar),reference_jar_sha256=sha(jar),ClientSession_class_sha256=class_sha,command=command,javap_exit_code=result.returncode,stdout_encoding='utf-8',methods=selected,actual_sessions=0,runtime_authority_modified=False,normal_source_modified=False,departure_runtime_verified=False,whole_goal_complete=False)
(out/'inspector.py').write_bytes(Path(__file__).read_bytes());(out/'selected-methods.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(output=str(out),selected_methods={h:len(b.splitlines())for h,b in selected.items()},runtime_verified=False),ensure_ascii=False))
