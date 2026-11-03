"""Read selected current75 methods only; never modify a product or runtime authority."""
from pathlib import Path
import hashlib,json,os,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'
def sha(data):return hashlib.sha256(data).hexdigest().upper()
assert sha(jar.read_bytes())=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
out=ROOT/'驗證紀錄/current/native33A3-contract-followup';out.mkdir(exist_ok=True)
env=dict(os.environ,TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
command=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+env['TMP'],'-p','-c','-classpath',str(jar),'local.lito381.server.ClientSession']
result=subprocess.run(command,capture_output=True,env=env,creationflags=subprocess.CREATE_NO_WINDOW,timeout=45)
assert result.returncode==0,result.stderr.decode('utf-8','replace')
lines=result.stdout.decode('utf-8').splitlines()
heads=[i for i,s in enumerate(lines)if re.match(r'^  (?:public|private|protected) .*\(.*\)(?: throws [^;]+)?;$',s)]
blocks={lines[i].strip():'\n'.join(lines[i:heads[n+1]if n+1<len(heads)else len(lines)])for n,i in enumerate(heads)}
names=['rentalItemSelectionContext','selectRentalItemSet','enterChannel','beginGameBootstrap','joinRentalRoom','enterRentalTown','resetRentalBattleLoadout','completeGameBootstrap','autoSelectPendingChannelPortHandoff','autoSelectPendingRentalBattleHandoff','redirectToChannelPort','joinRentalTown','activatePendingRentalBattleHandoff']
selected={h:b for h,b in blocks.items()if any(re.search(r'\b'+name+r'\(',h)for name in names)}
assert len(selected)>=4
with zipfile.ZipFile(jar)as z:class_sha=sha(z.read('local/lito381/server/ClientSession.class'))
record=dict(status='static_current75_contract_methods_only_no_new_sessions',reference_jar_sha256=sha(jar.read_bytes()),ClientSession_class_sha256=class_sha,command=command,javap_exit_code=result.returncode,stdout_encoding='utf-8',methods=selected,related_method_headers=[h for h in blocks if re.search('rental|channel|bootstrap',h,re.I)],runtime_authority_modified=False,normal_source_modified=False,actual_sessions=0,whole_goal_complete=False)
(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
(out/'ClientSession-selected-methods.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'initial-inspection-decode-failure.json').write_text(json.dumps(dict(tool_chunk='a87747',actual_exit_code=1,error='UnicodeDecodeError: UTF-8 decode at384709 with javap default file encoding',prior_stdout_not_retained=True,normal_source_modified=False,new_sessions=0,retried_only_read_only_javap_with_explicit_UTF8=True),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'header-inspection-failure.json').write_text(json.dumps(dict(tool_chunk='ab8268',actual_exit_code=1,error='Method header parser excluded declarations with throws clauses; fewer than4 selected methods',runtime_changed=False,method_scope_not_reduced=True,new_sessions=0),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(selected_methods={h:len(b.splitlines())for h,b in selected.items()},related_method_headers=record['related_method_headers'][:25]),ensure_ascii=False))
