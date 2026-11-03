"""Execute the pinned self-tick codec; never claim a cast or native clock binding."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
JDK=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--inspection-run',required=True,type=Path);a=ap.parse_args()
out=a.inspection_run.resolve(strict=True)
assert out.is_relative_to(ROOT/'驗證紀錄/current/native-self-tick-contract')
assert not (out/'codec-raw.zip').exists() and not (out/'codec-receipt.json').exists()
jar=PROJECT/'target/current/server-reference/formal-server-C02AAAE8.jar'
catalog=PROJECT/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
jar_sha='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
catalog_sha='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert sha(jar)==jar_sha and sha(catalog)==catalog_sha
source=ROOT/'維護工具/oracle/NativeSelfTickContractOracle.java';source_sha=sha(source)
(out/source.name).write_bytes(source.read_bytes());(out/'codec-runner.py').write_bytes(Path(__file__).read_bytes())
classes=PROJECT/'target/current/native-self-tick-contract'/out.name/'classes';classes.mkdir(parents=True)
env={k:v for k,v in os.environ.items() if not k.startswith('LITO_') and k not in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS']}
scratch=str(PROJECT/'target/current/scratch');env.update(TEMP=scratch,TMP=scratch)
calls=[];state=dict(status='prepared_self_tick_codec_only',jar_sha256=jar_sha,catalog_sha256=catalog_sha,
    source_sha256=source_sha,calls=calls,actual_game_sessions=0,actual_casts=0,product_changed=False,
    native_clock_source_verified=False,whole_goal_complete=False)
def execute(command,label):
    with (out/(label+'.stdout.txt')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
        p=subprocess.run(command,cwd=classes,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    calls.append(dict(arguments=command,exit_code=p.returncode));save(out/'codec-receipt.json',state)
    assert p.returncode==0,(label,p.returncode,'Retain failure before repair')
try:
    execute([str(JDK/'javac.exe'),'-J-Dfile.encoding=UTF-8','-J-XX:ActiveProcessorCount=1','-J-Xmx128m',
        '-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/source.name)],'compile')
    for profile in ['lito-legacy','tw13081901-v1']:
        command=[str(JDK/'java.exe'),'-Dfile.encoding=UTF-8','-XX:ActiveProcessorCount=1','-Xmx128m',
            '-Djava.io.tmpdir='+scratch,'-Dlito.resourceProfile='+profile,
            '-Dlito.extendedItemDetails='+('true' if profile=='tw13081901-v1' else 'false')]
        if profile=='tw13081901-v1':command+=['-Dlito.commonCatalog='+str(catalog)]
        command+=['-cp',str(classes)+';'+str(jar),'local.lito381.server.NativeSelfTickContractOracle',str(jar)]
        execute(command,profile)
        result=json.loads((out/(profile+'.stdout.txt')).read_text(encoding='utf-8-sig'))
        assert result['profile']==profile and len(result['positive'])==6 and result['negative_rejections']==13
        assert not result['native_clock_source_verified'] and result['actual_game_sessions']==result['actual_casts']==0
    assert sha(jar)==jar_sha and sha(catalog)==catalog_sha and sha(source)==source_sha
    state['status']='executed_self_tick_codec_both_profiles_no_sessions'
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
    save(out/'codec-receipt.json',state)
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['codec-raw.zip','codec-archive-readback.json']}
    for p in classes.rglob('*.class'):files['classes/'+p.relative_to(classes).as_posix()]=p
    hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(out/'codec-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'codec-raw.zip') as z:
        assert set(z.namelist())==set(hashes)
        for n,h in hashes.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    save(out/'codec-archive-readback.json',dict(archive_sha256=sha(out/'codec-raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status'],positive=12,negative_rejections=26,actual_casts=0),ensure_ascii=False))
