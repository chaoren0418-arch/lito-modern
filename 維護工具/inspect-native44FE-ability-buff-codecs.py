"""Current formal ability codec bytes and schema gate, without sessions."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';assert sha(jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native44FE-ability-buff-requests'/('codecs-'+stamp);out.mkdir(parents=True)
work=P/'target/current/native44FE-ability-buff-requests'/('codecs-'+stamp);work.mkdir(parents=True);classes=work/'classes';classes.mkdir()
source=ROOT/'維護工具/candidates/native44FE-skill-book-audit/AbilityBuffWireOracle.java';(out/source.name).write_bytes(source.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
jdk=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin');env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
assert not any(env.get(n)for n in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'])
commands=[('compile',[str(jdk/'javac.exe'),'-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/source.name)]),('codec',[str(jdk/'java.exe'),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+env['TMP'],'-cp',str(classes)+os.pathsep+str(jar),'local.lito381.server.AbilityBuffWireOracle',str(jar)])];results=[]
for label,command in commands:
    result=subprocess.run(command,cwd=work,env=env,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)
    (out/(label+'.stdout')).write_bytes(result.stdout);(out/(label+'.stderr')).write_bytes(result.stderr);results.append(dict(label=label,command=command,exit_code=result.returncode));save(out/'commands.json',results);assert result.returncode==0,label
data=json.loads((out/'codec.stdout').read_text('utf-8-sig'));assert len(data['cases'])==6 and data['actual_sessions']==data['actual_casts']==0
files={p.name:p for p in out.iterdir()if p.is_file()};pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(reference_jar_sha256=sha(jar),raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),codec_only=True,actual_sessions=0))
print(json.dumps(dict(evidence=str(out),data=data),ensure_ascii=False))
