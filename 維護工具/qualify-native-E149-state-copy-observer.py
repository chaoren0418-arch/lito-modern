"""Qualify a passive observer's compilation; never execute its classifier."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def calls(text):return {m.group(1) for line in text.splitlines() if (m:=re.search(r'// (?:InterfaceMethod|Method) (local/lito381/server/[^\s]+)',line))}

def publish(run,inspection,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-recovery-bootstrap-compile');c=read(run/'cohort.json');require(c['status']=='integrated_fixture_compilation_only_complete' and len(c['process_history'])==1 and c['process_history'][0]['exit_code']==0 and len(c['classes'])==6 and c['metadata']=={},'original Java terminal/scope differs')
 require(c['source_manifest_sha256']==SOURCE and c['formal_target_sha256']==FORMAL and c['tests_executed']==c['classes_instantiated']==0 and c['database_started'] is False,'execution scope differs')
 external={n:h for n,h in c['input_pins'].items() if not Path(n).is_relative_to(R)};javac=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javac.exe');require(set(external)=={str(javac)} and sha(javac)==external[str(javac)],'compiler changed')
 check_pins({n:h for n,h in c['input_pins'].items() if n not in external});check_pins(c['classes']);check_pins(c['process_history'][0]['logs'])
 cp=Path(c['candidate']);candidate=read(cp);require(sha(cp)==c['candidate_sha256'] and candidate['status']=='state_copy_precondition_observer_source_not_selected_not_tested','candidate changed');check_pins({str(cp.parent/n):h for n,h in candidate['candidate_files'].items()})
 require(sha(candidate['contract'])==candidate['contract_sha256'] and sha(candidate['state_copy_contract'])==candidate['state_copy_contract_sha256'] and sha(candidate['observer_fragment'])==candidate['observer_fragment_sha256'],'contract/fragment changed')
 contract=read(candidate['state_copy_contract']);require(contract['formal_target_sha256']==FORMAL and len(contract['declared_instance_fields'])==62 and contract['pending_generation_inclusive_minimum']==0 and contract['snapshot_is_copy_authorization'] is False,'pre-copy contract differs')
 for key in ['original_API_inspection','original_item_fields_source','original_copy_source']:require(sha(contract[key])==contract[key+'_sha256'],'original bytecode changed')
 api=read(contract['original_API_inspection']);check_pins(api['outputs']);require(all(x['exit_code']==0 for x in api['commands']),'original inspection failed')
 require(candidate['changed_files']==['LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java'] and candidate['Rust_files_changed']==candidate['Runtime_files_changed']==0,'source scope differs')
 prior=Path(candidate['predecessor_candidate']);require(sha(prior)==candidate['predecessor_candidate_sha256'],'canonical candidate changed');old=read(prior);require({n for n,h in candidate['candidate_files'].items() if h!=old['candidate_files'][n]}==set(candidate['changed_files']),'unexpected source delta')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'activity/deferral changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and w['current_E149_fixture_binding']==candidate['predecessor_fixture_binding'],'formal/binding changed')
 fb=read(w['current_E149_fixture_binding']['qualification']);require(fb['fixture_candidate']==str(prior),'canonical source binding differs');prior_q=read(fb['fixture_qualification']);require(sha(fb['fixture_qualification'])==fb['fixture_qualification_sha256'],'canonical Java qualification changed');check_pins(prior_q['source_pins']);check_pins(prior_q['Java_classes'])
 require(sha(run/'compiler.py')==sha(T/'compile-native-E149-recovery-bootstrap.py')==prior_q['source_pins'][str(T/'compile-native-E149-recovery-bootstrap.py')],'compiler control changed')
 java=[str(cp.parent/n) for n in candidate['candidate_files'] if n.endswith('.java')];expected=[str(javac),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(run/'scratch'),'-encoding','UTF-8','-proc:none','--release','17','-classpath',candidate['formal_reference'],'-d',str(run/'classes'),str(P/'tests/oracle/NativePeerHarness.java'),*java];require(c['process_history'][0]['command']==expected,'actual compiler command differs')
 old_classes={Path(n).name:h for n,h in prior_q['Java_classes'].items()};classes={Path(n).name:h for n,h in c['classes'].items()};require(set(old_classes)==set(classes) and {n for n in classes if old_classes[n]!=classes[n]}=={'NativeRecoveryReadonlySnapshot.class'},'class delta exceeds one observer')
 inspection=owned(inspection.resolve(strict=True),run);proof=read(inspection);require(proof['compile_cohort_sha256']==sha(run/'cohort.json') and all(x['exit_code']==0 for x in proof['commands']),'compiled inspection differs');check_pins(proof['outputs'])
 directory=Path(prior_q['bytecode_inspection']).parent;observer=(inspection.parent/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig');harness=(inspection.parent/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig')
 require(calls(observer)==calls((directory/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig')) and len(calls(observer))==18 and calls(harness)==calls((directory/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig')),'game method calls changed')
 require(not re.search(r'\bputfield\b|java/lang/reflect/Field\.(?:set:|setInt:|setLong:|setBoolean:)',observer),'observer writes game fields')
 require('ScheduledFuture.isDone:' not in observer and 'ScheduledFuture.isCancelled:' not in observer and 'ScheduledFuture.cancel:' not in observer and 'NativeDeItemStateCopy.copy:' not in observer,'observer invokes timer/copy methods')
 start=observer.index(' stateCopyPreconditions(');end=observer.index('  static long observationId(',start);body=observer[start:end]
 require('Class.getDeclaredFields:' in body and 'Field.get:' in body and 'class java/util/concurrent/ScheduledFuture' in body and 'Class.isAssignableFrom:' in body,'complete passive scan missing')
 require(all('// String '+n in body for n in contract['declared_instance_fields']),'compiled exact field manifest missing')
 tools=[T/n for n in ['prepare-native-E149-state-copy-observer.py','native_E149_state_copy_preconditions.py','qualify-native-E149-state-copy-observer.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8'),filename=str(p)),str(p),'exec')
 pure=ast.parse((T/'native_E149_state_copy_preconditions.py').read_text('utf-8'));require(all(isinstance(x,ast.FunctionDef) or isinstance(x,ast.Expr) and isinstance(x.value,ast.Constant) for x in pure.body),'classifier has executable top-level code/imports')
 inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal inputs changed');check_pins(c['normal_product_pins']);component=w['current_E149_component_linked_images']
 for row in component['images'].values():require(sha(row['product'])==row['sha256'],'component PE changed')
 ids=','.join(str(x['pid']) for x in [c['pipeline'],*c['process_history']]);query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress";observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in [c['pipeline'],*c['process_history']]:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'same original compiler remains live')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 out=owned(V/'native-E149-state-copy-observer-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(observed)
 record=dict(status='unselected_state_copy_precondition_observer_Java_and_AST_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,Java_compiler_exit_code=0,Java_classes=c['classes'],changed_classes=['NativeRecoveryReadonlySnapshot.class'],bytecode_inspection=str(inspection),bytecode_inspection_sha256=sha(inspection),state_copy_contract=candidate['state_copy_contract'],state_copy_contract_sha256=candidate['state_copy_contract_sha256'],declared_instance_fields=62,known_condition_entries=7,observer_original18_record_accessors_unchanged=True,no_new_game_method_calls=True,no_timer_status_or_cancel_calls=True,AST_compiled_tools=3,original_compiler_identities_terminal=True,
  source_pins={str(p):sha(p) for p in tools+[Path(candidate['observer_fragment']),T/'native-E149-state-copy-preconditions.md',T/'compile-native-E149-recovery-bootstrap.py',T/'inspect-native-E149-owner-guard-candidate.py']},original_fixture_binding_unchanged=w['current_E149_fixture_binding'],original_formal_suite_unchanged=w['current_E149_recovery_suite_development'],normal_sources_unchanged=True,normal_PEs_unchanged=True,component_PEs_unchanged=True,Rust_files_changed=0,Rust_compilers_invoked=0,tests_executed=0,classes_instantiated=0,classifier_executed=False,original_state_copy_invoked=False,original_timer_methods_invoked=False,SQL_executed=False,database_started=False,desktop_inputs=0,production_applied=False,canonical_fixture_or_plans_selected=False,integrated_into_active_fixture=False,integrated_into_producer_reader=False,full_native_owner_qualified=False,whole_goal_complete=False,
  findings=['Any retained ScheduledFuture instance blocks copy; completion/cancellation is not consulted','Nonnull declared EquipmentTimerLease blocks copy','Either pending flag true or generation>=0, including zero, blocks copy','All62 nonstatic declared fields are scanned; only predicate entries are emitted; no blockers never grants copy authority'],limitations=['No observer/classifier/timer/copy/test ran','Current canonical40-attempt fixture and16 plans unchanged','Normal GAME timer/lease creation, actual copy denial, natural expiry and full owner/Main/G06 remain pending'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw={Path(n).name:Path(n) for n in record['source_pins']};raw.update({'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'state-copy-contract.json':Path(candidate['state_copy_contract']),'compiled-inspection.json':inspection,'original-terminal-CIM.json':out/'original-terminal-CIM.json'})
 for n in candidate['candidate_files']:raw['candidate/'+n]=cp.parent/n
 for n in proof['outputs']:raw['bytecode/'+Path(n).name]=Path(n)
 for n in c['process_history'][0]['logs']:raw['compiler/'+Path(n).name]=Path(n)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in raw.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work='G06新增未選入的StateCopy前提唯讀observer：精確掃描原Item62非靜態欄，輸出七個已知條件與任一其他實際Future。更正active簡稱：任何非空ScheduledFuture都阻擋，不看done／cancelled；非空EquipmentTimerLease、兩pending旗標true或generation>=0（含0）也阻擋。原'+terminal_chunk+' actual exit0／Java六class，僅observer class改變，原18record accessor與harness遊戲呼叫不變；三個Python工具僅AST，純classifier／observer／timer／copy皆未執行，無阻擋也不授權copy。未選入canonical／16案；原32＋8、640次計畫及11Rust／兩正常EXE／四元件PE不變。下一步查正常GAME計時物品／真實lease與pending產生／清理，再補拒絕與自然到期矩陣及新觀測綁定。測試／桌面繼續暫緩，完整owner／Main／G06／G01–G12未完成。'
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_state_copy_observer_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='G06_3d_StateCopy_exact_preconditions_observer',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'progress format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items()),'concurrent writer');check_pins(c['normal_product_pins'])
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),Java_classes=6,changed_classes=1,declared_instance_fields=62,tests_executed=0,classifier_executed=False,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--inspection',type=Path,required=True);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();publish(a.run,a.inspection,a.terminal_chunk)
