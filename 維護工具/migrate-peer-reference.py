"""Promote the tested BBC59308 candidate after regenerating all wire fixtures."""
from pathlib import Path
import datetime, hashlib, json, os, shutil, subprocess
root=Path(__file__).resolve().parent.parent
project=root/'LiTo381-Modern';out=root/'驗證紀錄/current/native-real-peers/reference-BBC59308'
out.mkdir(parents=True,exist_ok=True)
def read(p): return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
report=read(root/'驗證紀錄/current/native-real-peers/formal-drift.json')
candidate=Path(report['snapshot']);expected=report['sha256']
assert sha(candidate)==sha(Path(report['source']))==expected
pinpath=root/'驗證紀錄/current/reference-snapshot.json';pin=read(pinpath)
snapshot=Path(pin['snapshot']);old=sha(snapshot)
assert old==pin['sha256']==report['previous_sha256'], 'One-time migration; baseline changed'
evidence=read(root/'驗證紀錄/current/native-real-peers/20261013-055923-658/authority-comparison.json')
assert evidence['jar_sha256']==expected and evidence['status']=='verified_cross_client_and_server_authority'
scopes=[('login','NativeLoginOracle'),('game','NativeGameOracle'),('lobby','NativeLobbyOracle'),
 ('room','NativeRoomOracle'),('world','NativeWorldOracle'),('motion','NativeMotionOracle'),
 ('navigation','NativeNavigationOracle'),('combat','NativeCombatOracle'),('ranged','NativeRangedOracle'),
 ('skills','NativeSkillOracle'),('attack-skills','NativeAttackSkillOracle'),('ground-skills','NativeGroundSkillOracle'),
 ('inventory','NativeInventoryOracle'),('item-actions','NativeItemActionsOracle'),('reagent-skills','NativeReagentSkillOracle'),
 ('ground-pickup','NativeGroundPickupOracle'),('label-updates','NativeLabelUpdatesOracle'),('emblems','NativeEmblemOracle'),
 ('version','FormalCipherOracle'),('cipher','FormalCipherOracle'),('force','NativeForceOracle')]
jdk=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin')
classes=project/'target/current/native-real-peers/oracle-candidate';classes.mkdir(parents=True,exist_ok=True)
env=dict(os.environ,TEMP=str(project/'target/current/scratch'),TMP=str(project/'target/current/scratch'))
def run(args):return subprocess.run(args,cwd=project,env=env,creationflags=subprocess.CREATE_NO_WINDOW,
    capture_output=True,text=True,encoding='utf-8',check=True).stdout
sources=sorted({str(project/'tests/oracle'/f'{klass}.java') for _,klass in scopes})
run([str(jdk/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(candidate),'-d',str(classes),*sources])
rows=[]
for name,klass in scopes:
    args=[str(jdk/'java.exe'),'-Dfile.encoding=UTF-8','-cp',str(classes)+';'+str(candidate)]
    args+=([klass,str(candidate)]+(['--version-contract'] if name=='version' else [])) if name in ['version','cipher'] else ['local.lito381.server.'+klass]
    text=run(args);fresh=json.loads(text)
    fixture=project/'tests/fixtures'/('formal-cipher-0.6.42.json' if name=='cipher' else f'native-{name}-current.json')
    before=read(fixture)
    assert fresh['jar_sha256'].upper()==expected
    a=dict(fresh);b=dict(before);a.pop('jar_sha256');b.pop('jar_sha256')
    assert a==b, f'{name}: semantic fixture drift requires investigation'
    (out/f'candidate-{name}.json').write_text(text.strip()+'\n',encoding='utf-8')
    rows.append(dict(scope=name,fixture=str(fixture.relative_to(root)),old_sha256=sha(fixture),equal_except_jar=True))
assert sha(Path(report['source']))==expected, 'Source drift during validation'
(out/'reference-snapshot-before.json').write_bytes(pinpath.read_bytes())
archive=snapshot.with_name('candidate-'+old[:8]+'.jar')
if not archive.exists():shutil.copyfile(snapshot,archive)
assert sha(archive)==old
for row in rows:
    fixture=root/row['fixture'];(out/('before-'+row['scope']+'.json')).write_bytes(fixture.read_bytes())
    fixture.write_bytes((out/('candidate-'+row['scope']+'.json')).read_bytes())
code=project/'src/modern_lito/rebuild_verification.rs';text=code.read_text(encoding='utf-8')
assert old.lower() in text
code.write_text(text.replace(old.lower(),expected.lower()),encoding='utf-8')
shutil.copyfile(candidate,snapshot);assert sha(snapshot)==expected
pin.update(sha256=expected,bytes=snapshot.stat().st_size,previous_snapshot=str(archive),previous_sha256=old,
           utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
pinpath.write_text(json.dumps(pin,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'comparison.json').write_text(json.dumps(dict(candidate_sha256=expected,previous_sha256=old,
    fixtures=rows,all_equal_except_jar=True,full_compatibility_verified=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Promoted BBC59308 snapshot; 21 regenerated fixtures equal except JAR identity; full regression still pending.')
