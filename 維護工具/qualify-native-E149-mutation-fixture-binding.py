"""Publish mutation fixture bindings and pure plans; never invoke runtime APIs."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,zipfile
from native_E149_recovery_contract import (R,P,SOURCE,FORMAL,RETIRED_MUTATION_KIND,RETIRED_MUTATION_METHODS,
 OWNER_IDENTITY_CONTRACT,read,sha,encoded,owned,require,check_pins,effective_overlay,client_image_binding,make_plan,validate_plan)
T=R/'維護工具';V=R/'驗證紀錄/current'
NAMES=['native_E149_recovery_contract.py','prepare-native-E149-recovery-case.py','verify-native-E149-recovery-case.py',
 'native_E149_owned_process.py','run-native-E149-recovery-case.py','native_E149_item_attributes.py',
 'native_E149_owner_guards.py','native_E149_recovery_suite.py','native_E149_mutation_denials.py']

def publish(out):
 out=owned(out.resolve(strict=True),V/'native-E149-mutation-fixture-binding-development');prior=read(out/'before-inputs.json')
 require(sha(out/'source-before.zip')==prior['source_before_archive_sha256'],'original source archive changed')
 with zipfile.ZipFile(out/'source-before.zip') as z:
  require(len(z.namelist())==len(prior['source_before_pins']) and all(hashlib.sha256(z.read(str(Path(name).relative_to(R)).replace('\\','/'))).hexdigest().upper()==h
   for name,h in prior['source_before_pins'].items()),'original source archive readback differs')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/target changed')
 require(w['last_goal_turn_evidence']==prior['previous_checkpoint'] and w['current_E149_fixture_binding']==prior['previous_fixture_binding'],'original progress/binding changed')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/name)==h for name,h in before.items()),'eight checkpoint files changed')
 require(sha(prior['previous_source_qualification'])==prior['previous_source_qualification_sha256'],'previous source qualification changed')
 sources={name:(T/name).read_text('utf-8-sig') for name in NAMES};trees={}
 for name,source in sources.items():
  tree=ast.parse(source,filename=name);compile(tree,name,'exec');trees[name]=tree
  for node in tree.body:
   if isinstance(node,ast.If):require(ast.unparse(node.test)=="__name__ == '__main__'",'unreviewed top-level branch')
 compile(ast.parse(Path(__file__).read_text('utf-8'),filename=str(__file__)),str(__file__),'exec')
 for name,function,first in [('run-native-E149-recovery-case.py','execute','authorize()'),('native_E149_recovery_suite.py','execute_suite','authorize_suite()')]:
  body=next(node for node in trees[name].body if isinstance(node,ast.FunctionDef) and node.name==function)
  require(ast.unparse(body.body[0])==first,'execution does not begin with current user consent')
 require('subprocess.Popen(' not in sources['run-native-E149-recovery-case.py'] and 'taskkill' not in sources['native_E149_owned_process.py'],'producer bypasses original held process helper')
 helper=sources['native_E149_owned_process.py'];require(helper.index('self.k.AssignProcessToJobObject(self.handle,process)')<helper.index('on_created(held)')<helper.index('self.k.ResumeThread(thread)'),
  'original private Job enrollment/identity barrier changed')
 for name in ['run-native-E149-recovery-case.py','verify-native-E149-recovery-case.py']:
  require('item-mutation-denial-trace.jsonl' in sources[name] and 'retired_item_mutation_probe_kind' in sources[name],'mutation context/raw not wired')
 reader=trees['verify-native-E149-recovery-case.py']
 require(any(isinstance(node,ast.ImportFrom) and node.module=='native_E149_mutation_denials' for node in reader.body)
  and 'verify_retired_mutations(' in sources['verify-native-E149-recovery-case.py'],'pure mutation reader not called by outer reader source')
 require("'item-mutation-denial-trace.jsonl'" in sources['verify-native-E149-recovery-case.py'] and '67_108_864' in sources['verify-native-E149-recovery-case.py'],'bounded pinned mutation trace missing')
 pure=trees['native_E149_mutation_denials.py']
 require(all(all(alias.name=='json' for alias in node.names) for node in pure.body if isinstance(node,ast.Import)), 'pure reader imports process/game APIs')
 require(all(node.module=='native_E149_item_attributes' for node in pure.body if isinstance(node,ast.ImportFrom)),'pure reader imports unreviewed helper')
 ext=w['current_E149_mutation_denial_development'];require(sha(ext['qualification'])==ext['qualification_sha256'],'new Java qualification changed')
 iq=read(ext['qualification']);check_pins(iq['source_pins']);check_pins(iq['Java_classes'])
 require(iq['Java_compiler_exit_code']==0 and len(iq['Java_classes'])==6 and iq['original_compiler_identities_terminal']
  and iq['tests_executed']==0 and iq['original_mutation_setters_invoked'] is False and iq['pure_receipt_reader_executed'] is False,'new Java scope differs')
 require(iq['prior_fixture_binding']==prior['previous_fixture_binding'],'new Java was not derived from this original binding')
 require(sha(prior['previous_fixture_binding']['qualification'])==prior['previous_fixture_binding']['qualification_sha256'],'previous fixture binding changed')
 old_binding=read(prior['previous_fixture_binding']['qualification']);candidate=read(iq['candidate'])
 require(candidate['predecessor_candidate']==old_binding['fixture_candidate'] and candidate['predecessor_candidate_sha256']==old_binding['fixture_candidate_sha256'],'new Java predecessor differs')
 guard=w['current_E149_owner_guard_development'];require(guard['qualification']==old_binding['fixture_qualification'] and guard['qualification_sha256']==old_binding['fixture_qualification_sha256'],'original guard qualification differs')
 cohort_path=Path(iq['compile_run'])/'cohort.json';cohort=read(cohort_path);require(sha(cohort_path)==iq['compile_cohort_sha256'],'actual Java compiler cohort changed')
 java_pins={name:h for name,h in cohort['input_pins'].items() if name.endswith('.java')};require(len(java_pins)==4,'Java input scope differs');check_pins(java_pins)
 files=dict(old_binding['current_effective_overlay_files']);paths=dict(old_binding['current_effective_overlay_paths'])
 for name,h in candidate['candidate_files'].items():files[name]=h;paths[name]=str(Path(iq['candidate']).parent/name)
 require(len(files)==14 and sum(name.endswith('.rs') for name in files)==11,'effective overlay scope differs')
 linked={};client_bindings={}
 for profile,ref in w['current_E149_recovery_linked_image_qualifications'].items():
  require(sha(ref['qualification'])==ref['qualification_sha256'],'original linked qualification changed')
  image=read(ref['qualification']);require(image['profile']==profile and sha(image['product'])==image['product_sha256'],'linked controller changed')
  require(sha(image['compile_cohort'])==image['compile_cohort_sha256'],'original build cohort changed');check_pins(read(image['compile_cohort'])['input_pins'])
  linked[profile]=ref;client_bindings[profile]=client_image_binding(image,files,paths)
  require(client_bindings[profile]['changed_Java_context']==['LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java','LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'],
   'context differs outside the two reviewed Java files')
 binding=dict(schema='E149-fixture-source-binding-v2',status='source_binding_only_no_runtime',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  fixture_qualification=ext['qualification'],fixture_qualification_sha256=ext['qualification_sha256'],
  fixture_candidate=iq['candidate'],fixture_candidate_sha256=iq['candidate_sha256'],base_owner_guard_qualification=dict(qualification=guard['qualification'],qualification_sha256=guard['qualification_sha256']),
  owner_guard_probe_kind='G06_E149_owner_guard_v1',retired_item_mutation_probe_kind=RETIRED_MUTATION_KIND,retired_item_mutation_methods=RETIRED_MUTATION_METHODS,
  retired_item_mutation_targets=4,retired_item_mutation_attempts=32,Java_compile_cohort=str(cohort_path),Java_compile_cohort_sha256=sha(cohort_path),
  Java_source_pins=java_pins,Java_classes=iq['Java_classes'],current_effective_overlay_files=files,current_effective_overlay_paths=paths,
  linked_client_qualifications=linked,client_bindings=client_bindings,predecessor_fixture_binding=prior['previous_fixture_binding'],
  selected_for_future_prepared_cases_only=True,original_linked_image_qualifications_modified=False,client_images_relinked=False,
  Java_classes_executed=False,Java_runtime_compatibility_qualified=False,tests_executed=0,full_native_owner_qualified=False,whole_goal_complete=False,
  recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'fixture-binding.json').write_bytes(encoded(binding));fixture_ref=dict(qualification=str(out/'fixture-binding.json'),qualification_sha256=sha(out/'fixture-binding.json'))
 actual_files,actual_paths=effective_overlay(w,fixture_ref);require(actual_files==files and actual_paths==paths,'source overlay reconstruction differs')
 metadata=read(w['current_development_typecheck_qualification']['qualification']);inputs=read(Path(metadata['run'])/'base-compiled-inputs.json')
 require(len(inputs)==1049 and all(sha(R/name)==h for name,h in inputs.items()),'normal compiled inputs changed');check_pins(metadata['normal_product_pins'])
 component=w['current_E149_component_linked_images'];require(sha(component['qualification'])==component['qualification_sha256'],'component qualification changed')
 for image in component['images'].values():require(sha(image['product'])==image['sha256'],'component PE changed')
 docs=['native-E149-recovery-case-contract.md','native-E149-extended-item-observation.md','native-E149-owner-guard-readback.md','native-E149-recovery-suite.md']
 source_pins={str(T/name):sha(T/name) for name in NAMES+docs}|{str(OWNER_IDENTITY_CONTRACT):sha(OWNER_IDENTITY_CONTRACT),str(Path(__file__).resolve()):sha(__file__)}
 record=dict(read(prior['previous_source_qualification']),status='v3_selected_recovery_mutation_fixture_and_reader_source_AST_only',source_pins=source_pins,
  predecessor_producer_development=prior['previous_source_qualification'],predecessor_producer_development_sha256=prior['previous_source_qualification_sha256'],
  fixture_source_binding=fixture_ref['qualification'],fixture_source_binding_sha256=fixture_ref['qualification_sha256'],
  fixture_qualification=ext['qualification'],fixture_qualification_sha256=ext['qualification_sha256'],client_bindings=client_bindings,
  guard_fixture_qualification=guard['qualification'],guard_fixture_qualification_sha256=guard['qualification_sha256'],
  retired_item_mutation_probe_kind=RETIRED_MUTATION_KIND,retired_item_mutation_methods=RETIRED_MUTATION_METHODS,
  retired_item_mutation_targets=4,retired_item_mutation_attempts=32,retired_item_mutation_reader_integrated=True,
  retired_item_mutation_reader_functions_executed=False,original_mutation_setters_invoked=False,mutation_AST_compiled_modules=1,total_AST_compiled_modules=9,
  Java_and_Rust_sources_changed_in_this_revision=0,existing_linked_Rust_bytes_unchanged=11,changed_Java_source_files=2,client_images_relinked=False,
  source_schema_versions=dict(fixture_binding=2,case_plan=3,execution=3,suite_plan=2),publisher_AST_compiled=True,
  runtime_producer_source_complete=True,runtime_producer_implementation_pending=False,passive_reader_source_complete=True,fixture_binding_source_complete=True,
  runtime_producer_runtime_qualified=False,reader_executed=False,reader_module_imported=False,reader_runtime_qualified=False,
  owner_guard_reader_functions_executed=False,original_owner_guards_invoked=False,owner_identity_helper_executed=False,
  suite_orchestrator_source_complete=True,runtime_suite_function_executed=False,suite_runtime_qualified=False,suite_selected_cases=16,
  process_job_objects_created=0,handle_operations_executed=0,workload_processes_started=0,tests_executed=0,desktop_inputs=0,
  SQL_executed=False,database_started=False,original_factories_invoked=False,original_initializer_invoked=False,
  full_attribute_matrix_qualified=False,full_Main_process_qualified=False,full_native_owner_qualified=False,whole_goal_complete=False,
  static_source_findings=['v3 cases require a v2 mutation fixture binding and separately compiled Java classes; original eleven Rust inputs remain unchanged',
   'producer pins the full original mutation JSONL; outer reader matches scope then joins each of 32 actual setter receipts to original server trace and owner roles',
   'sixteen selected cases are serial, first failure stops, no retries; 512 setter attempts are planned, zero observed'],
  limitations=['AST/file-data preparation only; no runtime producer, reader, mutation parser, suite, Job, DB, SQL, guard, setter or GUI ran',
   'Compiled Java and raw source bindings do not qualify runtime compatibility or successful captures/rejections',
   'Each future fixture requires all 32 exact setter denials; full extended-attribute/source-account/timer/Main/G06/library/GUI qualification remains pending'],
  test_execution_deferred_by_user=True,recorded_utc=datetime.now(timezone.utc).isoformat())
 for key in ['plan','plan_sha256','prepared_case_plan','prepared_case_plan_sha256','prepared_case_plans','prepared_case_suite_plan','prepared_case_suite_plan_sha256']:
  record.pop(key,None)
 (out/'qualified-development.json').write_bytes(encoded(record));review=dict(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'))
 plans={}
 for profile in ['Debug','Release']:
  path=make_plan(profile,'cancel-repeat',review,fixture_ref);plan=validate_plan(path)
  require(plan['retired_item_mutation_attempts']==32 and plan['schema']=='E149-recovery-case-plan-v3' and plan['test_execution_deferred_at_preparation'],'new plan scope differs')
  plans[profile]=dict(path=str(path),sha256=sha(path))
 from native_E149_recovery_suite import prepare_suite
 suite_path=prepare_suite(review,fixture_ref,plans);suite=read(suite_path)
 require(suite['schema']=='E149-selected-recovery-suite-plan-v2' and len(suite['members'])==16,'new suite scope differs')
 for row in suite['members']:
  member=read(row['plan']);require(member['retired_item_mutation_attempts']==32 and member['fixture_source_binding_sha256']==fixture_ref['qualification_sha256'],'suite mutation context differs')
 record.update(**review,prepared_case_plan=plans['Debug']['path'],prepared_case_plan_sha256=plans['Debug']['sha256'],prepared_case_plans=plans,
  prepared_case_suite_plan=str(suite_path),prepared_case_suite_plan_sha256=sha(suite_path),suite_plan_preparation_function_executed=True,
  preparation_mode='pure file/plan preparation; no runtime producer/reader/parser/Job functions',previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw={Path(name).name:Path(name) for name in source_pins};raw.update({'source-before.zip':out/'source-before.zip','before-inputs.json':out/'before-inputs.json',
  'fixture-binding.json':out/'fixture-binding.json','qualified-development.json':out/'qualified-development.json','suite-plan.json':suite_path})
 for row in suite['members']:raw['member-'+row['profile']+'-'+row['scenario']+'.json']=Path(row['plan'])
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name,path in raw.items():z.write(owned(path),name)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in raw.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work='G06新退役BoundItem候選已接入fixture binding v2、case plan／execution v3、suite plan v2與獨立reader來源；九模組僅AST。原Java a4a9be exit0／6class沿原編譯資格，11個Rust輸入／原兩版controller及四元件PE不變。每案固定8方法×4原owner＝32次；16案計畫共512次，全部未執行。producer需新ready marker並保存mutation JSONL／SHA，reader逐筆對回原trace／owner／session／actor／不同值／精確stale例外與前後觀測。舊來源／綁定／計畫封存不改寫；本輪只準備16份新計畫，未呼叫producer／reader／parser／suite執行入口或建立Job／DB，正常1049輸入／兩EXE不變。使用者測試／桌面暫緩維持；完整擴充屬性／source-account／timer／Main／G06與G01–G12未完成。'
 staged={}
 for name in before:
  path=R/name
  if path.suffix=='.json':
   value=read(path);value['current_E149_fixture_binding']=fixture_ref
   value['current_E149_recovery_execution_development']=record;value['current_E149_recovery_case_reader_development']=record
   value['current_E149_mutation_denial_integration_development']=dict(**review,fixture_qualification=ext['qualification'],fixture_qualification_sha256=ext['qualification_sha256'],
    source_integrated=True,planned_attempts_per_case=32,planned_suite_attempts=512,tests_executed=0,original_setters_invoked=False,reader_functions_executed=False)
   value['current_E149_owner_guard_integration_development']=dict(**review,source_integrated=True,tests_executed=0,original_guards_invoked=False,reader_functions_executed=False)
   value['current_E149_recovery_suite_development']=dict(**review,prepared_suite_plan=str(suite_path),prepared_suite_plan_sha256=sha(suite_path),selected_cases=16,
    source_complete=True,runtime_executed=False,tests_executed=0,automatic_retries=0,full_library_qualified=False,whole_goal_complete=False)
   value['current_E149_owner_identity_development']=dict(**review,owner_identity_contract=str(OWNER_IDENTITY_CONTRACT),owner_identity_contract_sha256=sha(OWNER_IDENTITY_CONTRACT),
    source_only=True,tests_executed=0,helper_executed=False,full_native_owner_qualified=False)
   value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
   value['current_focus']=dict(item='G06_3d_mutation_denial_fixture_source_integration',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if name.endswith('next-validation-action.json'):value['next_action']=work
   staged[name]=encoded(value)
  else:
   _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs')
   staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name in before:z.write(R/name,name)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items()),'before archive differs')
 require(all(sha(R/name)==h for name,h in before.items()) and all(sha(R/name)==h for name,h in inputs.items()),'concurrent source/checkpoint writer');check_pins(source_pins);check_pins(metadata['normal_product_pins'])
 for name,blob in staged.items():(R/name).write_bytes(blob)
 require(all((R/name).read_bytes()==blob for name,blob in staged.items()),'progress readback differs')
 (out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={name:sha(R/name) for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),fixture_binding=fixture_ref['qualification'],AST_modules=9,prepared_case_plans=16,planned_suite_setter_attempts=512,
  actual_setter_attempts=0,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('workspace',type=Path);args=ap.parse_args();publish(args.workspace)
if __name__=='__main__':main()
