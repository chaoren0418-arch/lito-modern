"""Read current catalogue shapes/S37 codecs; no game sessions or source modification."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('audit',type=Path);a=ap.parse_args();base=a.audit.resolve(strict=True)
assert base.is_relative_to(ROOT/'驗證紀錄/current/native44FE-skill-book-audit')
matrix=read(base/'matrix.json');proof=read(base/'readback.json');assert sha(base/'matrix.json')==proof['matrix_sha256']
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar';catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(jar)==matrix['reference_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
pairs=sorted({str(row['skill_id'])+':'+definition['factory']for row in matrix['matrix']for definition in row['definitions']});assert len(pairs)==74
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=base/('request-shapes-effects-'+stamp);out.mkdir()
work=P/'target/current/native44FE-skill-contract'/stamp;work.mkdir(parents=True);classes=work/'classes';classes.mkdir()
source=ROOT/'維護工具/candidates/native44FE-skill-book-audit/CurrentSkillRequestShapeOracle.java';(out/source.name).write_bytes(source.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
jdk=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin');env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
assert not any(env.get(n)for n in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'])
commands=[]
def execute(command,label):
    result=subprocess.run(command,env=env,cwd=work,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    (out/(label+'.stdout')).write_bytes(result.stdout);(out/(label+'.stderr')).write_bytes(result.stderr)
    commands.append(dict(label=label,command=command,exit_code=result.returncode));save(out/'commands.json',commands)
    assert result.returncode==0,(label,result.stderr.decode('utf-8',errors='replace'))
    return result.stdout
execute([str(jdk/'javac.exe'),'-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/source.name)],'javac')
results={}
for profile in ['lito-legacy','tw13081901-v1']:
    raw=execute([str(jdk/'java.exe'),'-XX:-UsePerfData','-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+env['TMP'],'-Dlito.resourceProfile='+profile,'-Dlito.commonCatalog='+str(catalog),'-cp',str(classes)+os.pathsep+str(jar),'local.lito381.server.CurrentSkillRequestShapeOracle',str(jar),','.join(pairs)],profile)
    rows=[json.loads(x)for x in raw.decode('utf-8').splitlines()];assert len(rows)==74
    assert all(row['actual_sessions']==0 and not row['runtime_activation_verified']and len(row['codec_examples'])==20 for row in rows)
    results[profile]=rows
record=dict(status='current75_computed_shapes_and_effect_codecs_only',source_manifest_sha256=matrix['source_manifest_sha256'],reference_jar_sha256=sha(jar),common_catalog_sha256=sha(catalog),rows_per_profile=74,codec_examples_per_row=20,
    actual_new_sessions=0,actual_new_casts=0,phase_guard_or_target_selection_verified=False,normal_source_modified=False,whole_goal_complete=False,results=results)
save(out/'computed-shapes-effects.json',record)
files={p.name:p for p in out.iterdir()if p.is_file()};pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),data_sha256=sha(out/'computed-shapes-effects.json')))
print(json.dumps(dict(evidence=str(out),skills_per_profile=74,codec_examples=2960,actual_sessions=0,actual_casts=0),ensure_ascii=False))
