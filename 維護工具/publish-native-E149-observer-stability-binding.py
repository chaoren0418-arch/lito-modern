"""Select compiled observer sources and prepare plans, never run test workloads."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,OWNER_IDENTITY_CONTRACT,read,sha,encoded,owned,require,check_pins,effective_overlay,client_image_binding,fixture_context
from native_E149_recovery_suite import prepare_suite,validate_suite
T=R/'維護工具';V=R/'驗證紀錄/current'
NAMES=['native_E149_recovery_contract.py','prepare-native-E149-recovery-case.py','verify-native-E149-recovery-case.py','native_E149_owned_process.py','run-native-E149-recovery-case.py','native_E149_item_attributes.py','native_E149_owner_guards.py','native_E149_recovery_suite.py','native_E149_mutation_denials.py','native_E149_morph_denials.py']
DOCS=['native-E149-recovery-case-contract.md','native-E149-extended-item-observation.md','native-E149-owner-guard-readback.md','native-E149-recovery-suite.md','native-E149-morph-fixture-binding.md','native-E149-observer-stability-binding.md']

def publish(out):
 out=owned(out.resolve(strict=True),V/'native-E149-observer-stability-binding-development');prior=read(out/'before-inputs.json');w=read(R/'工作佇列.json')
 require(sha(prior['source_before_archive'])==prior['source_before_archive_sha256'],'source archive changed')
 with zipfile.ZipFile(prior['source_before_archive']) as z:require(all(hashlib.sha256(z.read(Path(n).relative_to(R).as_posix())).hexdigest().upper()==h for n,h in prior['source_before_pins'].items()),'archived source differs')
 require(w['last_goal_turn_evidence']==prior['previous_checkpoint'] and w['current_E149_fixture_binding']==prior['previous_fixture_binding'] and w['current_E149_recovery_suite_development']==prior['previous_suite_development'],'checkpoint/binding changed')
 require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'workload/deferral changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 stability_ref=prior['stability_source_review'];require(stability_ref==w['current_E149_observer_stability_development'] and sha(stability_ref['qualification'])==stability_ref['qualification_sha256'],'observer qualification changed');iq=read(stability_ref['qualification']);check_pins(iq['source_pins']);check_pins(iq['Java_classes'])
 require(iq['Java_compiler_exit_code']==0 and len(iq['Java_classes'])==6 and iq['Java_classes_changed']==['NativeRecoveryReadonlySnapshot.class'] and iq['tests_executed']==0 and not iq['observer_executed'],'observer scope differs')
 oldref=prior['previous_fixture_binding'];require(sha(oldref['qualification'])==oldref['qualification_sha256'],'old binding changed');old=read(oldref['qualification'])
 cp=Path(iq['candidate']);candidate=read(cp);require(sha(cp)==iq['candidate_sha256'] and candidate['predecessor_candidate']==old['fixture_candidate'] and candidate['predecessor_candidate_sha256']==old['fixture_candidate_sha256'],'observer source ancestry differs')
 cohort_path=Path(iq['compile_run'])/'cohort.json';require(sha(cohort_path)==iq['compile_cohort_sha256'],'actual javac cohort changed');cohort=read(cohort_path);java_pins={n:h for n,h in cohort['input_pins'].items() if n.endswith('.java')};require(len(java_pins)==4,'actual Java input count differs');check_pins(java_pins)
 files=dict(old['current_effective_overlay_files']);paths=dict(old['current_effective_overlay_paths'])
 for n,h in candidate['candidate_files'].items():files[n]=h;paths[n]=str(cp.parent/n)
 observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java';require(len(files)==14 and [n for n in files if files[n]!=old['current_effective_overlay_files'][n]]==[observer],'effective overlay changed beyond observer')
 linked={};clients={}
 for profile,ref in w['current_E149_recovery_linked_image_qualifications'].items():
  require(sha(ref['qualification'])==ref['qualification_sha256'],'linked image qualification changed');image=read(ref['qualification']);require(sha(image['product'])==image['product_sha256'],'linked controller changed');check_pins(read(image['compile_cohort'])['input_pins']);linked[profile]=ref;clients[profile]=client_image_binding(image,files,paths)
  require(clients[profile]['changed_Java_context']==old['client_bindings'][profile]['changed_Java_context'],'linked client context changed outside original Java paths')
 binding=dict(old,fixture_qualification=stability_ref['qualification'],fixture_qualification_sha256=stability_ref['qualification_sha256'],fixture_candidate=str(cp),fixture_candidate_sha256=sha(cp),Java_compile_cohort=str(cohort_path),Java_compile_cohort_sha256=sha(cohort_path),Java_source_pins=java_pins,Java_classes=iq['Java_classes'],current_effective_overlay_files=files,current_effective_overlay_paths=paths,linked_client_qualifications=linked,client_bindings=clients,predecessor_fixture_binding=oldref,
  observer_stability_extension=dict(schema='E149-observer-stability-fixture-extension-v1',base_morph_qualification=dict(qualification=old['fixture_qualification'],qualification_sha256=old['fixture_qualification_sha256']),base_fixture_binding=oldref,changed_Java_sources=[observer],changed_classes=iq['Java_classes_changed'],added_stability_groups=iq['added_stability_groups']),Java_classes_executed=False,Java_runtime_compatibility_qualified=False,tests_executed=0,recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'fixture-binding.json').write_bytes(encoded(binding));fref=dict(qualification=str(out/'fixture-binding.json'),qualification_sha256=sha(out/'fixture-binding.json'))
 fixture_context(w,fref);actual_files,actual_paths=effective_overlay(w,fref);require(actual_files==files and actual_paths==paths,'effective overlay reconstruction differs')
 for n in NAMES:compile(ast.parse((T/n).read_text('utf-8-sig'),filename=str(T/n)),str(T/n),'exec')
 compile(ast.parse(Path(__file__).read_text('utf-8-sig')),str(Path(__file__)),'exec')
 source_pins={str(T/n):sha(T/n) for n in NAMES+DOCS}|{str(OWNER_IDENTITY_CONTRACT):sha(OWNER_IDENTITY_CONTRACT),str(Path(__file__).resolve()):sha(__file__)}
 oldsource=read(prior['previous_source_review']['qualification']);record=dict(oldsource,status='observer_stability_selected_fixture_and_reader_source_AST_only',source_pins=source_pins,source_before_archive=prior['source_before_archive'],source_before_archive_sha256=prior['source_before_archive_sha256'],source_before_pins=prior['source_before_pins'],previous_source_review=prior['previous_source_review'],previous_fixture_binding=oldref,previous_suite_development=prior['previous_suite_development'],
  fixture_source_binding=fref['qualification'],fixture_source_binding_sha256=fref['qualification_sha256'],fixture_qualification=stability_ref['qualification'],fixture_qualification_sha256=stability_ref['qualification_sha256'],client_bindings=clients,observer_stability_source_review=stability_ref,observer_stability_source_integrated=True,observer_stability_runtime_qualified=False,changed_Java_source_files=1,Java_and_Rust_sources_changed_in_this_revision=0,existing_linked_Rust_bytes_unchanged=11,total_AST_compiled_modules=10,
  source_schema_versions=dict(fixture_binding=3,observer_stability_extension=1,case_plan=4,execution=4,suite_plan=3),planned_aggregate_retired_setter_attempts=640,tests_executed=0,workload_processes_started=0,process_job_objects_created=0,SQL_executed=False,database_started=False,desktop_inputs=0,reader_runtime_qualified=False,runtime_producer_runtime_qualified=False,whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));review=dict(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'))
 suite_path=prepare_suite(review,fref);suite=validate_suite(suite_path,full_inputs=True);require(len(suite['members'])==16 and suite['planned_aggregate_retired_setter_attempts']==640,'prepared suite scope differs')
 record.update(**review,prepared_suite_plan=str(suite_path),prepared_suite_plan_sha256=sha(suite_path),previous_goal_turn_evidence=prior['previous_checkpoint'])
 baseline=read(w['current_development_typecheck_qualification']['qualification']);inputs=read(Path(baseline['run'])/'base-compiled-inputs.json');require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed');check_pins(baseline['normal_product_pins'])
 for row in w['current_E149_component_linked_images']['images'].values():require(sha(row['product'])==row['sha256'],'component image changed')
 for row in w['current_E149_integrated_client_development']['images'].values():require(sha(row['path'])==row['sha256'],'integrated GUI image changed')
 before=prior['before_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'eight-file checkpoint changed')
 archive={'source-before.zip':out/'source-before.zip','before-inputs.json':out/'before-inputs.json','fixture-binding.json':out/'fixture-binding.json','qualified-development.json':out/'qualified-development.json','suite-plan.json':suite_path}
 archive.update({'tools/'+Path(n).name:Path(n) for n in source_pins})
 for row in suite['members']:archive['members/'+row['profile']+'-'+row['scenario']+'.json']=Path(row['plan'])
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('G06觀測補強已選入新的fixture來源綁定：六組帳號／退役／application欄前後核對與同份輸出；沿原b56d33 javac六class，僅observer改變，未重編相同來源。'
  '新增嚴格observer extension：核對原morph→八setter祖源、四Java輸入、其餘五class不變及原11Rust；舊綁定保留原檢查。'
  'binding v3＋extension v1／case及execution v4／suite v3，已重綁兩版八分支16案，仍每案32＋8、合640次全未執行。'
  '十個Python控制模組僅AST及純檔案準備，沒有producer／reader／suite執行、Job、DB、SQL、guard或setter。舊來源與16案已封存，歷史資格不移記新版。'
  'StateCopy前提observer未合併；正常1049來源／兩正常PE、兩controller、四元件與兩隔離GUI PE不變。'
  '測試與桌面仍暫緩；恢復後先元件，再新選定G06十六案及技能、當版完整library兩版／Release／完整GUI，確認後才交人工。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_fixture_binding']=fref;value['current_E149_observer_stability_integration_development']=dict(**review,fixture_qualification=stability_ref['qualification'],fixture_qualification_sha256=stability_ref['qualification_sha256'],source_integrated=True,tests_executed=0,previous_fixture_binding=oldref,previous_suite_development=prior['previous_suite_development'])
   value['current_E149_recovery_execution_development']=record;value['current_E149_recovery_case_reader_development']=record
   value['current_E149_recovery_suite_development']=dict(**review,prepared_suite_plan=str(suite_path),prepared_suite_plan_sha256=sha(suite_path),selected_cases=16,source_complete=True,runtime_executed=False,tests_executed=0,automatic_retries=0,full_library_qualified=False,whole_goal_complete=False)
   for key in ['current_E149_owner_guard_integration_development','current_E149_mutation_denial_integration_development','current_E149_morph_denial_integration_development']:
    previous=value[key];value[key]=dict(previous,**review,fixture_qualification=stability_ref['qualification'],fixture_qualification_sha256=stability_ref['qualification_sha256'],source_integrated=True,tests_executed=0,previous_fixture_binding=oldref,previous_suite_development=prior['previous_suite_development'])
   value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='G06_observer_stability_fixture_source_selection',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'checkpoint before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()),'concurrent checkpoint writer');check_pins(source_pins);check_pins(baseline['normal_product_pins'])
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),fixture_binding=fref['qualification'],prepared_cases=16,planned_attempts=640,actual_attempts=0,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('workspace',type=Path);a=p.parse_args();publish(a.workspace)
