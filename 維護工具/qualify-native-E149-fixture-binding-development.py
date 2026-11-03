"""Publish static source bindings and prepare v2 plans. No runtime/test APIs.

The original linked-image qualification and Java-only compiler receipts stay
immutable. A binding compares source bytes; it grants no runtime compatibility.
"""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import ast,hashlib,json,zipfile
from native_E149_recovery_contract import (R,P,FORMAL,SOURCE,read,sha,encoded,check_pins,
 client_image_binding,effective_overlay,make_plan,OWNER_IDENTITY_CONTRACT)
T=R/'維護工具';V=R/'驗證紀錄/current'
names=['native_E149_recovery_contract.py','prepare-native-E149-recovery-case.py',
 'verify-native-E149-recovery-case.py','native_E149_owned_process.py',
 'run-native-E149-recovery-case.py','native_E149_item_attributes.py','native_E149_owner_guards.py','native_E149_recovery_suite.py']
sources={n:(T/n).read_text('utf-8') for n in names};trees={}
for name,source in sources.items():
 tree=ast.parse(source,filename=name);compile(tree,name,'exec');trees[name]=tree
 for node in tree.body:
  if isinstance(node,ast.If):assert ast.unparse(node.test)=="__name__ == '__main__'"
execute=next(n for n in trees['run-native-E149-recovery-case.py'].body if isinstance(n,ast.FunctionDef) and n.name=='execute')
assert ast.unparse(execute.body[0])=='authorize()'
suite_execute=next(n for n in trees['native_E149_recovery_suite.py'].body if isinstance(n,ast.FunctionDef) and n.name=='execute_suite')
assert ast.unparse(suite_execute.body[0])=='authorize_suite()'
suite_source=sources['native_E149_recovery_suite.py']
assert 'producer.execute(' in suite_source and 'reader.verify_execution(' in suite_source
assert suite_source.index('execution=producer.execute(')<suite_source.index('result=reader.verify_execution(')
assert 'automatic_retries=0' in suite_source and "'qualified_selected_case_matrix'" in suite_source
assert 'full_library_qualified=False' in suite_source and 'GUI_verified=False' in suite_source
authorize=next(n for n in trees['run-native-E149-recovery-case.py'].body if isinstance(n,ast.FunctionDef) and n.name=='authorize')
assert "get('test_execution_deferred_by_user', True)" in ast.unparse(authorize)
job=next(n for n in trees['native_E149_owned_process.py'].body if isinstance(n,ast.ClassDef) and n.name=='OwnedJob')
spawn=ast.unparse(next(n for n in job.body if isinstance(n,ast.FunctionDef) and n.name=='spawn_suspended'))
markers=['_winapi.CreateProcess(','.AssignProcessToJobObject(',"row['job_assigned_before_resume'] = True",'on_created(held)','.ResumeThread(']
positions=[spawn.index(n) for n in markers];assert positions==sorted(positions)
assert 'CREATE_SUSPENDED' in sources['native_E149_owned_process.py'] and 'CREATE_NO_WINDOW' in spawn
assert any(isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='flags' for t in n.targets)
 and any(isinstance(v,ast.Constant) and type(v.value) is int and v.value==4 for v in ast.walk(n.value)) for n in ast.walk(job))
assert '0x00002000' in sources['native_E149_owned_process.py'] and '0x00000800' not in sources['native_E149_owned_process.py']
assert 'taskkill' not in sources['run-native-E149-recovery-case.py']+sources['native_E149_owned_process.py']
assert 'subprocess.Popen(' not in sources['run-native-E149-recovery-case.py']
for marker in ['client_image_binding','fixture_source_binding','shared_client_resources_initialized','native_item_attribute_observer_kind']:
 assert marker in sources['run-native-E149-recovery-case.py'] and marker in sources['verify-native-E149-recovery-case.py']
reader_tree=trees['verify-native-E149-recovery-case.py']
assert any(isinstance(n,ast.ImportFrom) and n.module=='native_E149_item_attributes' for n in reader_tree.body)
for marker in ['observed_attributes(', 'compare_staged_adoption(', 'nondefault_coverage(', 'full_attribute_matrix_qualified=False']:
 assert marker in sources['verify-native-E149-recovery-case.py']
assert any(isinstance(n,ast.ImportFrom) and n.module=='native_E149_owner_guards' for n in reader_tree.body)
assert 'verify_owner_guards(' in sources['verify-native-E149-recovery-case.py']
assert 'owner-guard-trace.jsonl' in sources['run-native-E149-recovery-case.py'] and 'owner-guard-trace.jsonl' in sources['verify-native-E149-recovery-case.py']
assert all(n.module=='json' for n in trees['native_E149_owner_guards.py'].body if isinstance(n,ast.ImportFrom))
assert all(all(alias.name=='json' for alias in n.names) for n in trees['native_E149_owner_guards.py'].body if isinstance(n,ast.Import))
identity_function=next(n for n in reader_tree.body if isinstance(n,ast.FunctionDef) and n.name=='source_identity')
identity_source=ast.unparse(identity_function)
assert '.strip(java_trim_characters)' in identity_source and 'range(33)' in identity_source
assert '.lower(' not in identity_source and '.casefold(' not in identity_source
live_owner=ast.unparse(next(n for n in reader_tree.body if isinstance(n,ast.FunctionDef) and n.name=='live_owner'))
assert "owner['account'] == source_identity(row['source'], account)" in live_owner
assert "owner['source'] == source_identity(row['source'], row['character'])" in live_owner
owner_contract=read(OWNER_IDENTITY_CONTRACT)
assert owner_contract['schema']=='E149-owner-source-identity-contract-v1' and owner_contract['formal_target_sha256']==FORMAL
assert owner_contract['separator_code_point']==31 and not owner_contract['case_folding_at_owner_construction']
assert owner_contract['component_normalization']=='java_String_trim_UTF16_code_units_0000_through_0020'
check_pins(owner_contract['source_pins']);check_pins(owner_contract['excerpt_pins'])
w=read(R/'工作佇列.json')
assert not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
assert sha(w['current_formal_reference_identity']['formal'])==FORMAL
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
ext=w['current_E149_owner_guard_development'];assert sha(ext['qualification'])==ext['qualification_sha256']
eq=read(ext['qualification']);assert eq['Java_compiler_exit_code']==0 and eq['observed_actual_exit_code']==0
assert eq['Rust_files_changed']==eq['Rust_compilers_invoked']==eq['tests_executed']==0
assert eq['same_live_owner_positive_control_before_negative'] and eq['retired_checks_before_hub_shutdown']
assert not eq['game_field_writes_in_candidate'] and not eq['original_owner_guards_invoked'] and not eq['original_initializer_invoked']
check_pins(eq['source_pins'])
base=read(eq['prior_fixture_development']);assert sha(eq['prior_fixture_development'])==eq['prior_fixture_development_sha256']
assert base['common_initializer_call_explicitly_retained'] and base['common_initializer_before_sockets_and_config']
assert sha(base['Python_parser'])==base['Python_parser_sha256']
guard_harness=Path(eq['candidate']).parent/'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'
guard_source=guard_harness.read_text('utf-8')
assert guard_source.index('  initializeSharedProfile();')<guard_source.index('new ServerSocket(')
assert guard_source.count('\\"owner_guard_probe_kind\\":\\"G06_E149_owner_guard_v1\\"')==2
cohort_path=Path(eq['compile_run'])/'cohort.json';assert sha(cohort_path)==eq['compile_cohort_sha256']
cohort=read(cohort_path);assert cohort['metadata']=={} and len(cohort['process_history'])==1
javac=cohort['process_history'][0];assert javac['name']=='Java' and javac['exit_code']==0 and javac['finished_utc']
assert Path(javac['command'][0]).name=='javac.exe' and '-proc:none' in javac['command']
java_pins={n:h for n,h in cohort['input_pins'].items() if n.endswith('.java')}
assert len(java_pins)==4 and all(n in javac['command'] for n in java_pins)
check_pins(java_pins);check_pins(javac['logs']);check_pins(eq['Java_classes']);assert len(eq['Java_classes'])==6
assert sha(eq['candidate'])==eq['candidate_sha256'];candidate=read(eq['candidate'])
assert len(candidate['candidate_files'])==4
check_pins({str(Path(eq['candidate']).parent/n):h for n,h in candidate['candidate_files'].items()})
images=w['current_E149_recovery_linked_image_qualifications'];assert set(images)=={'Debug','Release'}
client_images={};linked_refs={}
for profile,row in images.items():
 assert sha(row['qualification'])==row['qualification_sha256'];image=read(row['qualification'])
 assert image['profile']==profile and image['compiler_exit_codes']==[0,0] and image['recorded_original_compilers_terminal']
 assert image['full_source_and_overlay_pins_verified'] and image['compiler_control_versions_archived'] and not image['outputs_executed']
 assert sha(image['compile_cohort'])==image['compile_cohort_sha256'] and sha(image['product'])==image['product_sha256']
 build=read(image['compile_cohort']);check_pins(build['input_pins'])
 assert build['effective_overlay_files']==image['effective_overlay_files']
 assert build['effective_overlay_paths']==image['effective_overlay_paths']
 client_images[profile]=image;linked_refs[profile]=dict(qualification=row['qualification'],qualification_sha256=row['qualification_sha256'])
files=dict(client_images['Debug']['effective_overlay_files']);paths=dict(client_images['Debug']['effective_overlay_paths'])
for name,digest in candidate['candidate_files'].items():
 files[name]=digest;paths[name]=str(Path(eq['candidate']).parent/name)
assert len(files)==14
client_bindings={profile:client_image_binding(image,files,paths) for profile,image in client_images.items()}
expected_changes=['LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java','LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java']
assert all(b['changed_Java_context']==expected_changes and b['Rust_source_files_unchanged']==11 for b in client_bindings.values())
metadata=read(w['current_development_typecheck_qualification']['qualification'])
inputs=read(Path(metadata['run'])/'base-compiled-inputs.json')
assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items());check_pins(metadata['normal_product_pins'])
old=w['current_E149_recovery_execution_development'];assert sha(old['qualification'])==old['qualification_sha256']
out=V/'native-E149-fixture-binding-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
binding=dict(schema='E149-fixture-source-binding-v1',status='source_binding_only_no_runtime',
 source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
 fixture_qualification=ext['qualification'],fixture_qualification_sha256=ext['qualification_sha256'],
 owner_guard_probe_kind='G06_E149_owner_guard_v1',
 fixture_candidate=eq['candidate'],fixture_candidate_sha256=eq['candidate_sha256'],
 Java_compile_cohort=str(cohort_path),Java_compile_cohort_sha256=sha(cohort_path),Java_source_pins=java_pins,Java_classes=eq['Java_classes'],
 current_effective_overlay_files=files,current_effective_overlay_paths=paths,
 linked_client_qualifications=linked_refs,client_bindings=client_bindings,
 selected_for_future_prepared_cases_only=True,original_linked_image_qualifications_modified=False,
 client_images_relinked=False,Java_classes_executed=False,tests_executed=0,
 Java_runtime_compatibility_qualified=False,full_native_owner_qualified=False,whole_goal_complete=False,
 predecessor_integrated_fixture=w['current_E149_integrated_fixture_development']['qualification'],
 predecessor_integrated_fixture_sha256=w['current_E149_integrated_fixture_development']['qualification_sha256'],
 recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'fixture-binding.json').write_bytes(encoded(binding))
fixture_ref=dict(qualification=str(out/'fixture-binding.json'),qualification_sha256=sha(out/'fixture-binding.json'))
actual_files,actual_paths=effective_overlay(w,fixture_ref);assert actual_files==files and actual_paths==paths
source_pins={str(T/n):sha(T/n) for n in names+['native-E149-recovery-case-contract.md','native-E149-extended-item-observation.md','native-E149-owner-guard-readback.md','native-E149-recovery-suite.md']}
source_pins[str(OWNER_IDENTITY_CONTRACT)]=sha(OWNER_IDENTITY_CONTRACT)
record=dict(read(old['qualification']),status='v2_selected_recovery_case_suite_and_owner_guard_reader_source_AST_only',
 source_pins=source_pins,source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
 predecessor_producer_development=old['qualification'],predecessor_producer_development_sha256=old['qualification_sha256'],
 fixture_source_binding=fixture_ref['qualification'],fixture_source_binding_sha256=fixture_ref['qualification_sha256'],
 fixture_qualification=ext['qualification'],fixture_qualification_sha256=ext['qualification_sha256'],client_bindings=client_bindings,
 owner_guard_probe_kind='G06_E149_owner_guard_v1',owner_guard_trace_reader_integrated=True,
 owner_guard_reader_functions_executed=False,original_owner_guards_invoked=False,
 guard_fixture_qualification=ext['qualification'],guard_fixture_qualification_sha256=ext['qualification_sha256'],
 owner_identity_contract=str(OWNER_IDENTITY_CONTRACT),owner_identity_contract_sha256=sha(OWNER_IDENTITY_CONTRACT),
 owner_identity_comparison_corrected_from_bare_to_source_qualified=True,owner_identity_helper_executed=False,
 nondefault_gameplay_route_proven=False,nondefault_setter_scan_classes=2554,
 gameplay_reachability_inspection=owner_contract['source_reachability_inspection'],
 gameplay_reachability_inspection_sha256=owner_contract['source_reachability_inspection_sha256'],
 Java_and_Rust_sources_changed_in_this_revision=0,
 AST_compiled_modules=3,producer_AST_compiled_modules=2,item_attribute_AST_compiled_modules=1,owner_guard_AST_compiled_modules=1,
 suite_AST_compiled_modules=1,total_AST_compiled_modules=8,suite_orchestrator_source_complete=True,
 runtime_suite_function_executed=False,suite_runtime_qualified=False,suite_selected_cases=16,suite_automatic_retries=0,
 runtime_producer_source_complete=True,runtime_producer_implementation_pending=False,runtime_producer_runtime_qualified=False,
 passive_reader_source_complete=True,reader_executed=False,reader_module_imported=False,reader_runtime_qualified=False,
 fixture_binding_source_complete=True,extended_item_parser_called_by_reader_source=True,extended_item_parser_functions_executed=False,
 existing_linked_Rust_bytes_unchanged=11,changed_Java_source_files=2,client_images_relinked=False,
 process_job_objects_created=0,handle_operations_executed=0,workload_processes_started=0,tests_executed=0,desktop_inputs=0,
 SQL_executed=False,database_started=False,original_factories_invoked=False,original_initializer_invoked=False,
 full_attribute_matrix_qualified=False,full_Main_process_qualified=False,full_native_owner_qualified=False,whole_goal_complete=False,
 static_source_findings=['v2 plan/execution pin original client qualification and separately compiled Java fixture',
  'eleven current Rust sources are byte-identical to original linked inputs; only two Java context sources differ',
  'original linked maps/cohorts/commands remain immutable; equivalent copied Rust paths compare by SHA',
  'current fixture binding is required by producer after its first defer authorization check',
  'owner guard marker is fixed in plan/execution/ready/result and raw trace is pinned before independent readback',
  'pure reader joins two live controls, one/two foreign inventory rejections and four actual retirements to original observer IDs/trace/continuation ACKs',
  'suite first checks current consent, then serializes eight exact cases per profile through original producer and separate reader; first failure stops without retry',
  'suite keeps each original case Job rather than adding a nested Job; terminal checkpoint never clears another active original workload',
  'ready/result require original shared initializer marker and native field schema',
  'reader consumes native sidecars, compares eight permanent fields on unexpired staged IDs, and separately reports nondefault coverage',
  'original owner source/account fields are namespaced source+U001F+character/account, not bare values; Java trim bounds and case are preserved',
  'original constructor/accessor/sourceCharacter/concat bootstrap evidence is pinned into both future plans',
  'nondefault normal-GAME generation remains unproven; constant-pool caller absence does not exclude reflection or external routing',
  'cached abilities and fourteen diagnostics do not trigger recomputation or cross-owner timer/generation invariants'],
 limitations=['AST/source and file/data custody only; no runtime suite/producer/reader/attribute/guard-readback function, original owner guard, Job, process, DB, SQL/JDBC, initializer or selected case was executed',
  'New Java/client runtime compatibility and process ownership/cleanup assertions still require authorized actual cases',
  'Source binding and prepared-case flags do not authorize tests; user defer remains true',
  'Default/empty attributes do not qualify nondefault gameplay/effect/mutation-denial/expiry matrix, dark-item payload or timer leases',
  'Guard-only receipts do not qualify item mutation, independently isolated account/source branches or timer/expiry-denial matrix',
  'Sixteen selected case plans are not executed results, baseline counterexamples, full library, GUI or whole G06',
  'Repeated stable observations are not atomic authority; complete owner/Main/G06/library/Release/GUI remain pending'],
 test_execution_deferred_by_user=True,recorded_utc=datetime.now(timezone.utc).isoformat())
for key in ['plan','plan_sha256','prepared_case_plan','prepared_case_plan_sha256','superseded_preparation_plan','superseded_preparation_plan_sha256',
 'observed_original_preparation_tool_chunk','observed_preparation_actual_exit_code','superseded_reason']:
 record.pop(key,None)
(out/'qualified-development.json').write_bytes(encoded(record))
review=dict(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'))
plans={}
for profile in ['Debug','Release']:
 plan=make_plan(profile,'cancel-repeat',producer_review=review,fixture_binding=fixture_ref);prepared=read(plan)
 assert prepared['schema']=='E149-recovery-case-plan-v2' and prepared['linked_image_qualified']
 assert not prepared['runtime_producer_implementation_pending'] and prepared['test_execution_deferred_at_preparation']
 assert prepared['fixture_classes']==str(Path(eq['compile_run'])/'classes')
 assert prepared['owner_identity_contract_sha256']==sha(OWNER_IDENTITY_CONTRACT)
 assert prepared['owner_guard_probe_kind']=='G06_E149_owner_guard_v1'
 plans[profile]=dict(path=str(plan),sha256=sha(plan))
from native_E149_recovery_suite import prepare_suite
suite_plan=prepare_suite(producer_review=review,fixture_binding=fixture_ref,reusable=plans)
suite_prepared=read(suite_plan);assert len(suite_prepared['members'])==16 and suite_prepared['tests_executed']==0
record.update(**review,prepared_case_plan=plans['Debug']['path'],prepared_case_plan_sha256=plans['Debug']['sha256'],prepared_case_plans=plans,
 prepared_case_suite_plan=str(suite_plan),prepared_case_suite_plan_sha256=sha(suite_plan),suite_plan_preparation_function_executed=True,
 preparation_mode='pure file/data plan preparation, without producer/reader/Job/attribute-function execution',
 previous_goal_turn_evidence=w['last_goal_turn_evidence'])
raw={Path(n).name:Path(n) for n in source_pins}
raw.update({'publisher.py':Path(__file__),'fixture-binding.json':out/'fixture-binding.json',
 'qualified-development.json':out/'qualified-development.json','Debug-plan.json':Path(plans['Debug']['path']),
 'Release-plan.json':Path(plans['Release']['path']),'suite-plan.json':suite_plan,'report-updater-before.py':T/'update-master-report-text-index-order.py'})
raw.update({'suite-member-'+row['profile']+'-'+row['scenario']+'.json':Path(row['plan']) for row in suite_prepared['members']})
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name,path in raw.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:
 assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items())
record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'))
(out/'evidence.json').write_bytes(encoded(record))
work='E149新增八個精確恢復案×Debug／Release的序列工具來源與16份固定member計畫；八個Python模組僅AST／來源核對，沒有執行suite／producer／reader。execute_suite第一步檢查現行暫緩旗標，每案沿原producer私有Job與原程序回收，再由獨立reader核對，首個失敗／來源變更／重新暫緩即停止並保留原始結果，不重試、不跳過；沒有外層巢狀Job。16案即使全核對也僅授此矩陣，不算完整library／GUI／G06／owner。owner guard／wire／屬性及來源限定識別契約維持；沿用原580956的6class、11個相同Rust輸入與兩版映像，正常1049來源／兩EXE不變，候選未套用。本輪只做純計畫準備，Job／DB／SQL／initializer／原guard／案例／桌面均未操作，使用者暫緩測試仍true。接續補原版6反例／候選14元件案與完整物品前提；恢復測試後才驗實際序列、guard時序、Job／真實DB／JDBC回收與自然33／31秒，再做新當版完整library兩版／Release／完整GUI，G01–G12未完成。'
staged={}
for name in before:
 path=R/name
 if path.suffix=='.json':
  j=read(path);j['current_E149_fixture_binding']=fixture_ref
  j['current_E149_recovery_execution_development']=record;j['current_E149_recovery_case_reader_development']=record
  j['current_E149_owner_guard_integration_development']=dict(qualification=review['qualification'],qualification_sha256=review['qualification_sha256'],
   fixture_qualification=ext['qualification'],fixture_qualification_sha256=ext['qualification_sha256'],
   source_integrated=True,selected_for_future_prepared_cases_only=True,tests_executed=0,original_guards_invoked=False,reader_functions_executed=False)
  j['current_E149_recovery_suite_development']=dict(qualification=review['qualification'],qualification_sha256=review['qualification_sha256'],
   prepared_suite_plan=str(suite_plan),prepared_suite_plan_sha256=sha(suite_plan),selected_cases=16,
   source_complete=True,runtime_executed=False,tests_executed=0,automatic_retries=0,full_library_qualified=False,whole_goal_complete=False)
  j['current_E149_owner_identity_development']=dict(qualification=review['qualification'],qualification_sha256=review['qualification_sha256'],
   owner_identity_contract=str(OWNER_IDENTITY_CONTRACT),owner_identity_contract_sha256=sha(OWNER_IDENTITY_CONTRACT),
   source_only=True,tests_executed=0,helper_executed=False,full_native_owner_qualified=False)
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_3d_E149_selected_case_suite_source_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
  if name.endswith('next-validation-action.json'):j['next_action']=work
  staged[name]=encoded(j)
 else:
  _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name in before:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip') as z:
 assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items());check_pins(metadata['normal_product_pins']);check_pins(source_pins)
for name,blob in staged.items():(R/name).write_bytes(blob)
assert all((R/n).read_bytes()==blob for n,blob in staged.items())
(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},
 before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),fixture_binding=fixture_ref['qualification'],prepared_plans=plans,
 Python_AST_modules=8,prepared_suite_plan=str(suite_plan),selected_case_plans=16,unchanged_linked_Rust_sources=11,changed_Java_context_sources=2,
 Job_objects_created=0,workload_processes_started=0,tests_executed=0,test_execution_deferred=True,eight_files_read_back=True),ensure_ascii=False))
