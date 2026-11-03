"""Read original Java compiler evidence; checkpoint unselected morph QA sources."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def calls(text):
 return {m.group(1) for line in text.splitlines() if (m:=re.search(r'// (?:InterfaceMethod|Method) (local/lito381/server/[^\s]+)',line))}

def publish(run,inspection,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-recovery-bootstrap-compile');c=read(run/'cohort.json')
 require(c['status']=='integrated_fixture_compilation_only_complete' and len(c['process_history'])==1 and c['process_history'][0]['exit_code']==0 and len(c['classes'])==6 and c['metadata']=={},'six Java classes and original terminal compiler required')
 require(c['source_manifest_sha256']==SOURCE and c['formal_target_sha256']==FORMAL and c['tests_executed']==c['classes_instantiated']==0 and c['database_started'] is False,'scope differs')
 external={n:h for n,h in c['input_pins'].items() if not Path(n).is_relative_to(R)}
 javac=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javac.exe');require(set(external)=={str(javac)} and sha(javac)==external[str(javac)] and c['process_history'][0]['command'][0]==str(javac),'compiler identity differs')
 check_pins({n:h for n,h in c['input_pins'].items() if n not in external});check_pins(c['classes']);check_pins(c['process_history'][0]['logs'])
 cp=Path(c['candidate']);candidate=read(cp);require(sha(cp)==c['candidate_sha256'] and candidate['status']=='retired_bound_morph_denial_source_not_active_not_tested','candidate differs')
 require(candidate['morph_methods']==['setItem','setItemId'] and candidate['planned_morph_attempts']==8 and candidate['planned_aggregate_retired_setter_attempts']==40 and candidate['Rust_files_changed']==candidate['Runtime_files_changed']==0,'planned scope differs')
 check_pins({str(cp.parent/n):h for n,h in candidate['candidate_files'].items()});check_pins(candidate['morph_fragments']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed')
 prior=Path(candidate['predecessor_candidate']);require(sha(prior)==candidate['predecessor_candidate_sha256'],'canonical predecessor changed');old=read(prior)
 require(len(candidate['candidate_files'])==4 and {n for n,h in candidate['candidate_files'].items() if h!=old['candidate_files'][n]}==set(candidate['changed_files']) and len(candidate['changed_files'])==2 and all(n.endswith('.java') for n in candidate['changed_files']),'unexpected delta')
 for key in ['morph_API_inspection','original_bound_API_inspection']:
  require(sha(candidate[key])==candidate[key+'_sha256'],'original API changed');api=read(candidate[key]);require(api['formal_target_sha256']==FORMAL and all(x['exit_code']==0 for x in api['commands']),'API target differs');check_pins(api['outputs'])
 inspection=owned(inspection.resolve(strict=True),run);proof=read(inspection);require(proof['compile_cohort_sha256']==sha(run/'cohort.json') and all(x['exit_code']==0 for x in proof['commands']),'compiled inspection differs');check_pins(proof['outputs'])
 observer=(inspection.parent/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig');harness=(inspection.parent/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig')
 require(not re.search(r'\bputfield\b|java/lang/reflect/Field\.(?:set:|setInt:|setLong:|setBoolean:)',observer+harness) and 'RuntimeInventory.items:' not in observer,'QA writes game fields or refreshes observer state')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/target differs')
 require(w['current_E149_fixture_binding']==candidate['predecessor_fixture_binding'],'canonical fixture binding changed')
 prior_q_ref=w['current_E149_mutation_denial_development'];require(sha(prior_q_ref['qualification'])==prior_q_ref['qualification_sha256'],'prior mutation qualification changed');pq=read(prior_q_ref['qualification']);require(pq['candidate']==str(prior),'new candidate not based on qualified canonical eight setters')
 check_pins(pq['source_pins']);directory=Path(pq['bytecode_inspection']).parent
 require(sha(run/'compiler.py')==sha(T/'compile-native-E149-recovery-bootstrap.py')==pq['source_pins'][str(T/'compile-native-E149-recovery-bootstrap.py')],'original compiler control differs')
 java=[cp.parent/n for n in candidate['candidate_files'] if n.endswith('.java')]
 expected_command=[str(javac),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(run/'scratch'),'-encoding','UTF-8','-proc:none','--release','17','-classpath',candidate['formal_reference'],'-d',str(run/'classes'),str(P/'tests/oracle/NativePeerHarness.java'),*[str(p) for p in java]]
 require(len(java)==3 and c['process_history'][0]['command']==expected_command,'actual javac command differs')
 require(calls(observer)==calls((directory/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig')) and len(calls(observer))==18,'original18 observer record accessors differ')
 old_calls=calls((directory/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig'));require(old_calls<=calls(harness),'original harness game calls removed')
 delta={n for n in calls(harness)-old_calls if not n.startswith('local/lito381/server/NativeRecoveryReadonlySnapshot.')}
 require(delta=={'local/lito381/server/NativeDeWorldKinds$Item.setItem:(Llocal/lito381/server/NativeDeItemTemplate;)V','local/lito381/server/NativeDeWorldKinds$Item.setItemId:(I)V'},'game-call delta differs from two reviewed morph endpoints')
 source=(cp.parent/'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java').read_text('utf-8');observer_source=(cp.parent/'LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java').read_text('utf-8')
 require((T/'templates/NativeRecoveryMutationDenials.fragment.java').read_text('utf-8') in source and (T/'templates/NativeRecoveryMutationDenialObserver.fragment.java').read_text('utf-8') in observer_source,'original eight setters/observer changed')
 require(source.index('retiredOwnerGuards(ownerProofs,ownerReferences)')<source.index('retiredMutationDenials(mutationProofs,mutationTargets)')<source.index('retiredMorphDenials(morphProofs,morphTargets)')<source.index('flowComplete=true')<source.index('app.close()'),'Future/retirement/mutation/hub-close order differs')
 require('original session futures not terminal before retirement guards' in source and 'targets.size()==4' in source and 'attempts==8' in source,'original admission/barriers missing')
 fragment=(T/'templates/NativeRecoveryMorphDenials.fragment.java').read_text('utf-8');require(fragment.index('proofs.flush();attempts++;')<fragment.index('if(failure!=null&&'),'wrong mutation outcome is rejected before preserving receipt')
 require('new NativeDe' not in fragment and '.setAccessible(' not in fragment and '.adopt(' not in fragment,'fake game object or reflective write in morph probe')
 tools=[T/n for n in ['inspect-native-E149-morph-denial-contract.py','prepare-native-E149-morph-denial-candidate.py','native_E149_morph_denials.py','qualify-native-E149-morph-denial-development.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8'),filename=str(p)),str(p),'exec')
 inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal inputs changed');check_pins(c['normal_product_pins'])
 component=w['current_E149_component_linked_images'];require(sha(component['qualification'])==component['qualification_sha256'],'component qualification changed')
 for row in component['images'].values():require(sha(row['product'])==row['sha256'],'component PE changed')
 first=V/'native-E149-recovery-bootstrap-compile/20261024-010630-558508/cohort.json';fc=read(first);require(fc['process_history'][0]['exit_code']==0 and len(fc['classes'])==6 and fc['tests_executed']==0,'original first-draft compilation differs');check_pins(fc['classes']);check_pins({n:h for n,h in fc['input_pins'].items() if Path(n).is_relative_to(R)})
 identities=[c['pipeline'],*c['process_history'],fc['pipeline'],*fc['process_history']];ids=','.join(str(x['pid']) for x in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'same original compiler/pipeline still live')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 out=owned(V/'native-E149-morph-denial-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(observed)
 record=dict(status='retired_morph_probes_Java_compilation_and_reader_source_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,Java_compiler_exit_code=0,Java_classes=c['classes'],bytecode_inspection=str(inspection),bytecode_inspection_sha256=sha(inspection),
  first_draft_compile_cohort=str(first),first_draft_compile_cohort_sha256=sha(first),first_draft_actual_tool_chunk='f2b633',first_draft_actual_exit_code=0,first_draft_superseded_for_failure_receipt_preservation=True,
  original_harness_game_method_call_delta=sorted(delta),observer_original18_record_accessors_unchanged=True,original_eight_setter_source_unchanged=True,planned_owner_targets=4,planned_additional_morph_attempts=8,planned_aggregate_retired_setter_attempts=40,original_compiler_identities_terminal=True,
  AST_compiled_tools=4,source_pins={str(p):sha(p) for p in tools+[Path(n) for n in candidate['morph_fragments']]+[T/'native-E149-morph-denial-development.md',T/'native_E149_mutation_denials.py',T/'native_E149_item_attributes.py',T/'compile-native-E149-recovery-bootstrap.py',T/'inspect-native-E149-owner-guard-candidate.py']},
  original_fixture_binding_unchanged=w['current_E149_fixture_binding'],original_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_planned_setter_attempts=32,canonical_fixture_or_plans_selected=False,integrated_into_active_fixture=False,integrated_into_producer_reader=False,pure_receipt_reader_executed=False,
  normal_sources_unchanged=True,normal_PEs_unchanged=True,component_PEs_unchanged=True,Rust_files_changed=0,Rust_compilers_invoked=0,tests_executed=0,classes_instantiated=0,original_morph_setters_invoked=False,original_initializer_invoked=False,SQL_executed=False,database_started=False,desktop_inputs=0,production_applied=False,full_native_owner_qualified=False,full_attribute_matrix_qualified=False,full_Main_process_qualified=False,whole_goal_complete=False,
  limitations=['Java compilation and reader AST only; no original guards/setters/reader/game classes were invoked','new 8 receipts are not bound into canonical32 producer/reader or original16 plans','four two-template inventories and exact runtime rejection unverified','extended attributes, isolated source/account checks, leases/natural expiry and Main still pending'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw={Path(n).name:Path(n) for n in record['source_pins']};raw.update({'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'bytecode-inspection.json':inspection,'first-draft-cohort.json':first,'original-terminal-CIM.json':out/'original-terminal-CIM.json'})
 for n in candidate['candidate_files']:raw['candidate/'+n]=cp.parent/n
 for n in proof['outputs']:raw['bytecode/'+Path(n).name]=Path(n)
 for n in c['process_history'][0]['logs']:raw['compiler/'+Path(n).name]=Path(n)
 for n in fc['input_pins']:
  if Path(n).suffix=='.java':raw['first-draft/'+Path(n).name]=Path(n)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in raw.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work='G06新增未選入的真實模板／ID退役拒絕候選：四個原owner各保留兩個不同模板的實際附著BoundItem，新增setItem／setItemId各一次，共規劃8次；保留原八setter32次，擴充候選合計40次。先寫實際結果及前後觀測再判斷，驗證錯誤／意外接受也保留原receipt。新Java '+terminal_chunk+' actual exit0／6class，兩類javap exit0，原observer18accessor與原八setter片段不變；4個Python工具僅AST，純reader／guard／setter均未執行。現行canonical32與16案仍不變，新八筆未接入producer／reader／計畫。正常1049輸入／兩EXE、四元件PE與#79候選不變；下一步整合新binding／雙trace讀回，再補非預設屬性、source/account及timer前提。測試／桌面依使用者要求暫緩，完整G06／G01–G12未完成。'
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_morph_denial_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='G06_3d_retired_actual_template_ID_denial_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
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
 print(json.dumps(dict(evidence=str(out/'evidence.json'),Java_classes=6,planned_additional_morph_attempts=8,actual_morph_attempts=0,canonical_plans_unchanged=True,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--inspection',type=Path,required=True);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();publish(a.run,a.inspection,a.terminal_chunk)
