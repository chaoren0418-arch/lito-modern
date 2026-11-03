"""Read source, javac and javap evidence; no observer or game execution."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,collections,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def qualify(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-recovery-bootstrap-compile');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='integrated_fixture_compilation_only_complete' and c['compilation_scope']=='Java-only; unchanged Rust qualifications retained' and len(c['process_history'])==1 and c['process_history'][0]['exit_code']==0 and len(c['classes'])==6,'Java-only compilation incomplete')
 require(c['tests_executed']==0 and c['classes_instantiated']==0 and not c['SQL_executed'] and not c['database_started'] and not c['original_factories_invoked'],'compilation scope differs')
 require(sha(cp)==c['candidate_sha256'] and candidate['formal_target_sha256']==FORMAL and candidate['base_source_manifest_sha256']==SOURCE,'candidate identity changed')
 require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'workload/deferral changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and candidate['predecessor_fixture_binding']==w['current_E149_fixture_binding'],'formal/binding changed')
 for p,h in c['input_pins'].items():require(sha(p)==h,'compiler input changed: '+p)
 check_pins(c['normal_product_pins']);check_pins(c['classes']);check_pins(c['process_history'][0]['logs'])
 inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 prior=read(candidate['predecessor_candidate']);require(sha(candidate['predecessor_candidate'])==candidate['predecessor_candidate_sha256'],'predecessor changed')
 observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java';require(candidate['changed_files']==[observer] and set(candidate['candidate_files'])==set(prior['candidate_files']),'candidate file projection differs')
 for n,h in candidate['candidate_files'].items():
  require(sha(cp.parent/n)==h,'candidate file changed')
  require((h!=prior['candidate_files'][n])==(n==observer),'another fixture source changed')
 original=(Path(candidate['predecessor_candidate']).parent/observer).read_text('utf-8-sig');text=(cp.parent/observer).read_text('utf-8-sig');anchor='    static String observe(ClientSession c,GameHub hub)'
 require(original.split(anchor)[0]==text.split(anchor)[0] and text.count('synchronized(')==original.count('synchronized('),'observer changed outside observe or added synchronization')
 for marker in ['"test_account",account,','"active",ownerActive,','"session_matches",ownerSession==c','"source_item_disconnected",disconnected,','application!=null&&Boolean.FALSE.equals(applicationClosed)',
  'Objects.equals(account,field(c,"account"))','Objects.equals(disconnected,field(c,"sourceItemDisconnected"))','Objects.equals(ownerActive,field(owner,"active"))','ownerSession==((WeakReference<?>)field(owner,"session")).get()',
  'application==field(hub,"sourceItemApplication")','Objects.equals(applicationClosed,field(application,"closed"))']:
  require(text.count(marker)==1,'captured output or added stability condition differs: '+marker)
 contract=read(candidate['stability_contract']);require(sha(candidate['stability_contract'])==candidate['stability_contract_sha256'] and len(contract['added_stability_groups'])==6 and not contract['snapshot_is_atomic'] and not contract['ABA_excluded'],'source stability contract differs')
 require(sha(contract['original_owner_bytecode'])==contract['original_owner_bytecode_sha256'],'original owner field contract changed')
 binding=read(w['current_E149_fixture_binding']['qualification']);oldclasses={Path(p).name:(p,h) for p,h in binding['Java_classes'].items()};newclasses={Path(p).name:(p,h) for p,h in c['classes'].items()}
 require(len(oldclasses)==len(newclasses)==6 and oldclasses.keys()==newclasses.keys(),'class set differs')
 changed=sorted(n for n in oldclasses if oldclasses[n][1]!=newclasses[n][1]);require(changed==['NativeRecoveryReadonlySnapshot.class'],'another fixture class changed')
 for p,h in oldclasses.values():require(sha(p)==h,'original Java class changed')
 out=owned(V/'native-E149-observer-stability-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe');require(sha(javap)=='09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A','javap changed')
 inspections=[]
 for tag,classes in [('original',oldclasses),('candidate',newclasses)]:
  path=Path(classes['NativeRecoveryReadonlySnapshot.class'][0]);cmd=[str(javap),'-J-Dfile.encoding=UTF-8','-p','-c',str(path)]
  with (out/(tag+'.stdout.txt')).open('xb') as stdout,(out/(tag+'.stderr.txt')).open('xb') as stderr:process=subprocess.run(cmd,stdout=stdout,stderr=stderr,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
  row=dict(tag=tag,command=cmd,exit_code=process.returncode,outputs={str(out/(tag+suffix)):sha(out/(tag+suffix)) for suffix in ['.stdout.txt','.stderr.txt']});inspections.append(row);(out/'javap-commands.json').write_bytes(encoded(inspections));require(process.returncode==0,'javap failed; original diagnostics retained')
 def calls(s):return collections.Counter(re.findall(r'// (?:InterfaceMethod|Method) (local/lito381/server/[^\n]+)',s))
 oldcode=(out/'original.stdout.txt').read_text('utf-8-sig');newcode=(out/'candidate.stdout.txt').read_text('utf-8-sig');require(calls(oldcode) and calls(oldcode)==calls(newcode),'new original game method call introduced')
 require(not re.search(r'java/lang/reflect/Field[.]set(?:Boolean|Byte|Char|Short|Int|Long|Float|Double)?:',newcode) and not re.search(r'(?m)^\s*\d+:\s*putfield\s+[^\n]*// Field local/lito381/server/',newcode),'reflection or direct game field write found')
 identities=[c['pipeline'],*[{k:r[k] for k in ['pid','creation_utc']} for r in c['process_history']]];ids=','.join(str(r['pid']) for r in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress";raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original javac/pipeline still live')
 (out/'original-terminal-CIM.json').write_bytes(raw);before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['prepare-native-E149-observer-stability.py','qualify-native-E149-observer-stability.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 record=dict(status='unselected_observer_stability_Java_and_bytecode_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,Java_compiler_exit_code=0,Java_classes=c['classes'],Java_classes_changed=changed,javap_exit_codes=[r['exit_code'] for r in inspections],javap_inspections=inspections,original_game_method_calls_unchanged=True,
  added_stability_groups=contract['added_stability_groups'],captured_output_fields=contract['captured_output_fields'],application_active_semantics=contract['application_active_semantics'],atomic_snapshot_claimed=False,ABA_excluded=False,Rust_files_changed=0,Runtime_files_changed=0,
  source_pins={str(p):sha(p) for p in [*tools,T/'compile-native-E149-recovery-bootstrap.py',T/'native-E149-observer-stability.md']},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],previous_state_copy_observer_unchanged=w['current_E149_state_copy_observer_development'],
  tests_executed=0,observer_executed=False,reader_executed=False,guard_or_setter_executed=False,SQL_executed=False,server_started=False,GUI_verified=False,desktop_inputs=0,canonical_fixture_selected=False,production_applied=False,full_native_owner_qualified=False,whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'stability-contract.json':Path(candidate['stability_contract']),'observer.diff':cp.parent/'observer.diff','javap-commands.json':out/'javap-commands.json','original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'tools/'+Path(p).name:Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in inspections:
  for p in row['outputs']:archive[Path(p).name]=Path(p)
 for p in c['process_history'][0]['logs']:archive['Java/'+Path(p).name]=Path(p)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('G06唯讀observer新增六組前後比對：account、sourceItemDisconnected、owner.active／session referent、sourceItemApplication參照／closed；輸出使用同一份捕捉值。'
  '更正application_active由非空改為非空且closed=false，仍不代表life.admits／requireActive。只補一份Java觀測器，不寫遊戲欄位、不加鎖、不呼叫遊戲guard／setter。'
  '原Java-only實際退出0／六class，僅observer class改變；兩份javap退出0且原遊戲方法呼叫集合不變。只做來源／編譯／位元碼，observer與reader及所有runtime都未執行，非原子快照、不排除ABA。'
  '候選未選入，與StateCopy前提observer是獨立分支；正常1049輸入／所有PE、G06 binding／16案640次均不變。'
  '下一次fixture來源整合先納入此修正並重新綁定class／SHA；測試與桌面暫緩維持，恢復後依原元件／G06及當版技能、library兩版／Release／完整GUI驗證，再交人工。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_observer_stability_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='G06_readonly_observer_account_and_retirement_stability_source',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'checkpoint archive differs')
 require(all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items()),'concurrent writer');check_pins(c['normal_product_pins'])
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),Java_exit_code=0,classes_changed=changed,javap_exit_codes=[r['exit_code'] for r in inspections],tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();qualify(a.run,a.terminal_chunk)
