from r491_common import *
import shutil,uuid
S=Path(r'D:\lineage開發\3-lito開發\lito伺服器\LiTo381-Server')
h=read(S.parent/'Native-書信加密登入選角雙來源端到端持久化交接.json')
f=Path(h['frozen_root']);assert pin(f/'manifest.json')['sha256']==h['manifest_sha256']
m=read(f/'manifest.json');selected=[]
for e in m['files']:
 if e['path'].startswith(('candidate/','ddl/','db/')) or e['path'] in ('source/ModernSettlementMysqlFixture.java','source/NativeLetterAuthenticatedWireMain.java'):
  q=f/e['path'];assert pin(q)['bytes']==e['bytes'] and pin(q)['sha256']==e['sha256'];selected.append(q)
assert any(q.name=='ModernSettlementMysqlFixture.java' for q in selected)
w=P/'target'/('r507-java-'+uuid.uuid4().hex);w.mkdir()
for q in selected:
 d=w/q.relative_to(f);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(q,d)
helper=S/'tools/NativeLetterOwnedConsoleServer.py';d=w/'source/NativeLetterOwnedConsoleServer.py';shutil.copyfile(helper,d)
# Modern owns this new launcher. JVM and diagnostic temporary paths are explicit.
own=w/'source/R507OwnedConsoleServer.py'
own.write_text(d.read_text(encoding='utf-8').replace("'-Dfile.encoding=UTF-8'","'-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+str(runtime)"),encoding='utf-8')
r=Path(r'D:\lito-dev-runtime\20261003-supervision\modern')/('nw-'+uuid.uuid4().hex);r.mkdir()
shutil.copytree(w/'db',r/'db');(r/'generation-1').mkdir()
save(L/'r507-java-inputs.json',dict(work=str(w),runtime=str(r),parent_handoff=pin(S.parent/'Native-書信加密登入選角雙來源端到端持久化交接.json'),parent_manifest=pin(f/'manifest.json'),inputs=[dict(original=pin(q),snapshot=pin(w/q.relative_to(f))) for q in selected],owned_console_original=pin(helper),owned_console_snapshot=pin(d),owned_console_candidate=pin(own),writes={'compiler':str(w/'test-classes'),'runtime_and_subprocess_cwd':str(r),'java.io.tmpdir':str(r),'TEMP_TMP':str(r),'mysql':'<runtime>/lito-modern-mysql-fixtures/mysql-5.5-*/{data,tmp,logs,pid,native-database.properties}','native_server':'<runtime>/{native.state,native-captures,native-*.properties,generation-1/*}'},retention='Do not call mysql.successful(); no recursive delete or retry of refused cleanup.',historical_PASS_reruns=0,GUI=0))
v=read(P.parent/'監製進度.json');v.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R507_modern_owned_Java_integration_preparing',current_task='modern ASCII隔離目錄已授權且Java main可接受；R506封存保留，新增Java整合、host Back與S_FB邊界核對中')
v['r507']={'state':'preparing_owned_java','scope_pending_withdrawn':True,'evidence':str(L/'r507-java-inputs.json'),'GUI':0,'historical_PASS_reruns':0};save(P.parent/'監製進度.json',v)
print(json.dumps({'work':str(w),'runtime':str(r),'copied_pins':len(selected)},ensure_ascii=False))
