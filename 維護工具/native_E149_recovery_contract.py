"""Shared data-only contract for future E149 recovery execution and readback."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,stat,sys
R=Path(__file__).resolve().parents[1]
P=R/'LiTo381-Modern'
FORMAL='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
SQL='C1E29B0479FE40476AD53B4E52498451C3A1E469A27483EABE67FA880EBCA7DD'
CATALOG='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
SOURCE='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
OWNER_IDENTITY_CONTRACT=R/'維護工具/resources/E149-owner-identity-contract-E1498871/contract.json'
RETIRED_MUTATION_KIND='G06_E149_retired_bound_item_mutation_v1'
RETIRED_MORPH_KIND='G06_E149_retired_bound_morph_v1'
RETIRED_MORPH_METHODS=['setItem','setItemId']
RETIRED_MUTATION_METHODS=['setCount','setEquipped','setIdentified','setSealed','setEnchantLevel','set_durability','setBless','setChargeCount']
SCENARIOS={
 'timeout':('destination_timeout_retires_old_authority_and_allows_fresh_login',False,False),
 'refused':('refused_destination_retires_authority_and_allows_fresh_login',False,False),
 'wrong-service':('wrong_service_destination_retires_authority_and_allows_fresh_login',False,False),
 'cancel-retry':('cancellation_then_immediate_retry_consumes_staged_character_once',False,False),
 'cancel-expiry':('cancellation_retires_destination_and_expired_handoff_requires_fresh_selection',True,False),
 'cancel-repeat':('repeated_cancellation_preserves_only_one_authenticated_recovery',False,True),
 'refused-cancel-repeat':('failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery',False,True),
 'cancel-expiry-source':('expired_stage_and_source_handoff_require_the_requested_source_directory',True,False),
}
ITEM_FIELDS={'id','count','catalog_id','name','equipped','durability','enchant_level','identified',
 'cached_packet_bless','raw_native_bless','charges','sealed','source_bound','tournament_reward',
 'expires_at','lito_owned','bound_actor_present'}
def require(ok,message):
 if not ok:raise RuntimeError(message)
def owned(path,root=R):
 path=Path(os.path.abspath(path));require(path.is_relative_to(root),'path outside owned root')
 for part in [path,*path.parents]:
  if part.exists():require(not part.lstat().st_file_attributes&stat.FILE_ATTRIBUTE_REPARSE_POINT,'reparse ancestor: '+str(part))
 require(path.resolve().is_relative_to(root.resolve()),'resolved path outside owned root')
 return path
def read(path):return json.loads(owned(path).read_text('utf-8-sig'))
def sha(path):
 with Path(path).open('rb') as file:return hashlib.file_digest(file,'sha256').hexdigest().upper()
def encoded(value):return (json.dumps(value,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
def write_new(path,value):
 with owned(path).open('xb') as file:file.write(encoded(value))
def check_pins(pins):
 for name,digest in pins.items():require(sha(owned(name))==digest,'pinned input changed: '+name)
def fixture_context(w,override=None):
 ref=override or w.get('current_E149_fixture_binding');require(ref is not None,'new fixture source binding required')
 require(sha(owned(ref['qualification']))==ref['qualification_sha256'],'fixture binding changed')
 binding=read(ref['qualification'])
 require(binding['schema']=='E149-fixture-source-binding-v3' and binding['status']=='source_binding_only_no_runtime','not a current mutation fixture source binding')
 require(binding['source_manifest_sha256']==SOURCE and binding['formal_target_sha256']==FORMAL,'fixture binding target differs')
 require(sha(owned(binding['fixture_qualification']))==binding['fixture_qualification_sha256'],'Java fixture qualification changed')
 iq=read(binding['fixture_qualification'])
 require(iq['source_manifest_sha256']==SOURCE and iq['formal_target_sha256']==FORMAL,'compiled Java target differs')
 require(iq['Java_compiler_exit_code']==0 and len(iq['Java_classes'])==6 and iq['Rust_files_changed']==0,'new Java compilation scope differs')
 require(binding['Java_classes']==iq['Java_classes'] and binding['fixture_candidate']==iq['candidate']
  and binding['fixture_candidate_sha256']==iq['candidate_sha256'],'fixture source/classes differ from actual compilation')
 cohort_path=Path(iq['compile_run'])/'cohort.json'
 require(binding['Java_compile_cohort']==str(cohort_path) and sha(owned(cohort_path))==binding['Java_compile_cohort_sha256']==iq['compile_cohort_sha256'],'actual Java compiler cohort differs')
 cohort=read(cohort_path)
 require(binding['Java_source_pins']=={n:h for n,h in cohort['input_pins'].items() if n.endswith('.java')},'Java sources differ from actual compiler inputs')
 require(cohort['classes']==iq['Java_classes'],'Java class outputs differ from actual compiler receipt')
 check_pins(binding['Java_source_pins']);check_pins(iq['Java_classes'])
 guard_ref=binding['base_owner_guard_qualification'];require(sha(owned(guard_ref['qualification']))==guard_ref['qualification_sha256'],'base owner guard qualification changed')
 guard=read(guard_ref['qualification'])
 require(binding['owner_guard_probe_kind']=='G06_E149_owner_guard_v1' and guard['same_live_owner_positive_control_before_negative'] is True
  and guard['retired_checks_before_hub_shutdown'] is True,'original owner guard candidate binding required')
 base_ref=binding['base_mutation_qualification'];require(sha(owned(base_ref['qualification']))==base_ref['qualification_sha256'],'base mutation qualification changed')
 base=read(base_ref['qualification'])
 extension=binding.get('observer_stability_extension');morph_iq=iq
 if extension is not None:
  require(extension['schema']=='E149-observer-stability-fixture-extension-v1','unreviewed observer extension')
  parent=extension['base_morph_qualification'];require(sha(owned(parent['qualification']))==parent['qualification_sha256'],'parent morph qualification changed')
  morph_iq=read(parent['qualification'])
  require(iq['status']=='unselected_observer_stability_Java_and_bytecode_only' and iq['Java_classes_changed']==['NativeRecoveryReadonlySnapshot.class']
   and iq['javap_exit_codes']==[0,0] and iq['original_game_method_calls_unchanged'] is True
   and len(iq['added_stability_groups'])==6 and iq['tests_executed']==0 and iq['observer_executed'] is False
   and iq['canonical_fixture_selected'] is False,'observer compilation/source scope differs')
  require(extension['base_fixture_binding']==iq['current_fixture_binding_unchanged'],'observer predecessor context differs')
  require(sha(owned(extension['base_fixture_binding']['qualification']))==extension['base_fixture_binding']['qualification_sha256'],'observer base binding changed')
  previous=read(extension['base_fixture_binding']['qualification'])
  require(previous['fixture_qualification']==parent['qualification'] and previous['fixture_qualification_sha256']==parent['qualification_sha256'],'observer parent is not the selected morph fixture')
  require(sha(owned(morph_iq['candidate']))==morph_iq['candidate_sha256'],'parent morph candidate changed')
  before=read(morph_iq['candidate']);after=read(iq['candidate']);observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java'
  require(after['predecessor_candidate']==morph_iq['candidate'] and after['predecessor_candidate_sha256']==morph_iq['candidate_sha256']
   and after['changed_files']==[observer] and set(after['candidate_files'])==set(before['candidate_files']),'observer source ancestry differs')
  require([n for n in after['candidate_files'] if after['candidate_files'][n]!=before['candidate_files'][n]]==[observer],'observer changed another fixture source')
  old_classes={Path(n).name:h for n,h in morph_iq['Java_classes'].items()};new_classes={Path(n).name:h for n,h in iq['Java_classes'].items()}
  require(len(old_classes)==len(new_classes)==6 and old_classes.keys()==new_classes.keys()
   and sorted(n for n in old_classes if old_classes[n]!=new_classes[n])==['NativeRecoveryReadonlySnapshot.class'],'observer changed another fixture class')
  check_pins(morph_iq['Java_classes']);check_pins(morph_iq['source_pins'])
 require(binding['retired_item_mutation_probe_kind']==RETIRED_MUTATION_KIND and binding['retired_item_mutation_methods']==RETIRED_MUTATION_METHODS
  and binding['retired_item_mutation_targets']==base['planned_owner_targets']==morph_iq['planned_owner_targets']==4
  and binding['retired_item_mutation_attempts']==base['planned_retired_setter_attempts']==32,'original core mutation scope differs')
 require(base['status']=='retired_bound_setter_probes_Java_compilation_and_reader_source_only'
  and base['pure_receipt_reader_source_complete'] is True and base['original_mutation_setters_invoked'] is False,'reviewed core mutation source required')
 require(morph_iq['status']=='retired_morph_probes_Java_compilation_and_reader_source_only' and morph_iq['original_eight_setter_source_unchanged'] is True
  and morph_iq['pure_receipt_reader_executed'] is False and morph_iq['original_morph_setters_invoked'] is False,'reviewed morph fixture source required')
 require(binding['retired_item_morph_probe_kind']==RETIRED_MORPH_KIND and binding['retired_item_morph_methods']==RETIRED_MORPH_METHODS
  and binding['retired_item_morph_targets']==4 and binding['retired_item_morph_attempts']==morph_iq['planned_additional_morph_attempts']==8
  and binding['retired_item_aggregate_attempts']==morph_iq['planned_aggregate_retired_setter_attempts']==40,'morph scope differs')
 require(sha(owned(binding['fixture_candidate']))==binding['fixture_candidate_sha256'],'morph fixture candidate changed')
 candidate=read(binding['fixture_candidate']);require(candidate['mutation_methods']==RETIRED_MUTATION_METHODS and candidate['morph_methods']==RETIRED_MORPH_METHODS,'compiled mutation/morph method sets differ')
 morph_candidate=read(morph_iq['candidate'])
 require(morph_candidate['predecessor_candidate']==base['candidate'] and morph_candidate['predecessor_candidate_sha256']==base['candidate_sha256'],'morph did not extend original core mutation candidate')
 check_pins(iq['source_pins']);check_pins(base['source_pins']);check_pins(guard['source_pins'])
 return ref,binding,iq

def effective_overlay(w,fixture_binding=None):
 files={};source_paths={}
 for row in w['current_isolated_development']['candidates'].values():
  # Registry uses path/sha256. Do not infer candidates from directory names.
  path=Path(row['path']);require(sha(path)==row['sha256'],'registered candidate changed')
  c=read(path)
  for name,digest in c['candidate_files'].items():
   require(name not in files,'original candidate overlap')
   require(sha(path.parent/name)==digest,'candidate source changed')
   files[name]=digest;source_paths[name]=str(path.parent/name)
 require(len(files)==13,'unexpected original overlay count')
 _,binding,_=fixture_context(w,fixture_binding);path=Path(binding['fixture_candidate'])
 require(sha(path)==binding['fixture_candidate_sha256'],'integrated fixture candidate changed')
 integrated=read(path);overlaps=set(files)&set(integrated['candidate_files'])
 require(overlaps=={'LiTo381-Modern/tests/native_teleport_failure.rs',
  'LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java',
  'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'},'unexpected integrated overlap')
 for name,digest in integrated['candidate_files'].items():
  require(sha(path.parent/name)==digest,'integrated source changed')
  files[name]=digest;source_paths[name]=str(path.parent/name)
 require(len(files)==14,'effective overlay must deduplicate to fourteen files')
 return files,source_paths

def client_image_binding(image,files,paths):
 require(image['status']=='qualified_linked_isolated_E149_recovery_controller','linked image qualification required')
 require(image['source_manifest_sha256']==SOURCE and image['formal_target_sha256']==FORMAL,'client image target differs')
 original=image['effective_overlay_files'];require(set(original)==set(files) and len(files)==14,'client overlay scope differs')
 rust={n:h for n,h in files.items() if n.endswith('.rs')};require(len(rust)==11,'unexpected Rust source scope')
 require(all(original[n]==h and sha(owned(paths[n]))==h for n,h in rust.items()),'current Rust bytes differ from linked image')
 require(all(sha(owned(image['effective_overlay_paths'][n]))==h for n,h in rust.items()),'original linked Rust source bytes changed')
 changed=sorted(n for n in files if original[n]!=files[n])
 java={'LiTo381-Modern/tests/oracle/'+n+'.java' for n in ['NativeTeleportFailureHarness','NativeRecoveryReadonlySnapshot','NativeRecoveryProductionBootstrap']}
 require(set(changed)<=java,'non-Java context changed')
 return dict(schema='E149-client-Rust-byte-binding-v1',original_linked_overlay_files=original,
  current_case_overlay_files=files,Rust_source_files_unchanged=11,changed_Java_context=changed,
  client_image_relinked=False,Java_runtime_compatibility_qualified=False)
def validate_plan(path):
 path=owned(path,R/'維護工具/candidates');p=read(path)
 require(p['schema']=='E149-recovery-case-plan-v4' and p['status']=='development_plan_only','not a current development case plan')
 require(p['scenario'] in SCENARIOS and p['profile'] in ['Debug','Release'],'case scope invalid')
 require(p['test_name']==SCENARIOS[p['scenario']][0],'selected Rust case differs')
 require(p['source_manifest_sha256']==SOURCE and p['formal_target_sha256']==FORMAL,'source/formal differs')
 check_pins(p['input_pins'])
 require(Path(p['owner_identity_contract'])==OWNER_IDENTITY_CONTRACT and sha(owned(OWNER_IDENTITY_CONTRACT))==p['owner_identity_contract_sha256'],'owner identity contract changed')
 identity_contract=read(OWNER_IDENTITY_CONTRACT)
 require(identity_contract['schema']=='E149-owner-source-identity-contract-v1' and identity_contract['formal_target_sha256']==FORMAL
  and identity_contract['separator_code_point']==31 and identity_contract['component_normalization']=='java_String_trim_UTF16_code_units_0000_through_0020'
  and identity_contract['case_folding_at_owner_construction'] is False,'owner identity semantics differ')
 check_pins(identity_contract['source_pins']);check_pins(identity_contract['excerpt_pins'])
 _,binding,iq=fixture_context({},dict(qualification=p['fixture_source_binding'],qualification_sha256=p['fixture_source_binding_sha256']))
 require(binding['current_effective_overlay_files']==p['effective_overlay_files'] and binding['current_effective_overlay_paths']==p['effective_overlay_paths'],'fixture source context differs')
 require(p['fixture_qualification']==binding['fixture_qualification'] and p['fixture_qualification_sha256']==binding['fixture_qualification_sha256'],'fixture qualification differs')
 require(p['Java_source_pins']==binding['Java_source_pins'] and p['Java_classes']==iq['Java_classes'],'Java source/class pins differ')
 require(p['fixture_classes']==str(Path(iq['compile_run'])/'classes'),'Java class directory differs')
 require(p['shared_client_resources_required'] is True and p['native_item_attribute_observer_kind']=='G06_E149_native_item_fields_v1','new initializer/attribute observation required')
 require(p['owner_guard_probe_kind']==binding['owner_guard_probe_kind']=='G06_E149_owner_guard_v1','owner guard scope differs')
 for key in ['retired_item_mutation_probe_kind','retired_item_mutation_methods','retired_item_mutation_targets','retired_item_mutation_attempts',
  'retired_item_morph_probe_kind','retired_item_morph_methods','retired_item_morph_targets','retired_item_morph_attempts','retired_item_aggregate_attempts']:
  require(p[key]==binding[key],'retired mutation plan scope differs: '+key)
 java=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe')
 require(Path(p['java'])==java and sha(java)==p['java_sha256'],'read-only JDK changed')
 require(Path(p['python'])==Path(sys.executable).resolve() and sha(p['python'])==p['python_sha256'],'Python tool identity changed')
 require(len(p['effective_overlay_files'])==14,'unexpected merged source scope')
 for name,digest in p['effective_overlay_files'].items():require(sha(owned(p['effective_overlay_paths'][name]))==digest,'merged source changed')
 inputs=read(p['base_compiled_inputs']);require(len(inputs)==1049,'baseline input count differs')
 for name,digest in inputs.items():require(sha(owned(R/name))==digest,'normal source changed')
 check_pins(p['normal_product_pins'])
 if p['linked_image_qualified']:
  require(sha(owned(p['linked_image_qualification']))==p['linked_image_qualification_sha256'],'linked image qualification changed')
  image=read(p['linked_image_qualification'])
  require(image['profile']==p['profile'] and client_image_binding(image,p['effective_overlay_files'],p['effective_overlay_paths'])==p['client_source_binding'],'linked Rust byte binding differs')
 return p
def make_plan(profile,scenario,producer_review=None,fixture_binding=None):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],'active workload/freeze')
 # Preparation does not authorize execution, whether tests are deferred or
 # have later been resumed. The producer independently checks current consent.
 ref=w['current_formal_reference_identity'];require(sha(ref['formal'])==sha(ref['cached_reference'])==FORMAL,'formal target changed')
 fixture_ref,binding,iq=fixture_context(w,fixture_binding)
 files,source_paths=effective_overlay(w,fixture_binding)
 require(binding['current_effective_overlay_files']==files,'registered fixture context differs')
 iqpath=Path(binding['fixture_qualification']);qpath=Path(w['current_development_typecheck_qualification']['qualification'])
 require(sha(qpath)==w['current_development_typecheck_qualification']['qualification_sha256'],'library metadata qualification changed')
 iq=read(iqpath);q=read(qpath)
 require(iq['Java_compiler_exit_code']==0 and len(iq['Java_classes'])==6,'integrated fixture compilation required')
 db=w['current_E149_owned_mysql_development'];bundle=read(db['bundle_manifest']);static=read(db['static_manifest'])
 catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin';require(sha(catalog)==CATALOG,'catalog differs')
 dependencies=[Path(__file__).resolve(),R/'維護工具/verify-native-E149-recovery-case.py',
  R/'維護工具/native-E149-owned-mysql.py',Path(ref['cached_reference']),Path(bundle['JDBC']['cached']),
  Path(db['bundle_manifest']),Path(db['static_manifest']),Path(static['SQL']),catalog,
  iqpath,Path(fixture_ref['qualification']),Path(w['current_development_typecheck_qualification']['qualification']),
  R/'維護工具/native-E149-recovery-case-contract.md',R/'維護工具/native_E149_mutation_denials.py',R/'維護工具/native_E149_morph_denials.py',OWNER_IDENTITY_CONTRACT]
 dependencies += [Path(q['run'])/'base-compiled-inputs.json',Path(binding['fixture_candidate'])]
 dependencies += [Path(n) for n in binding['Java_source_pins']]
 dependencies += [Path(row['path']) for row in w['current_isolated_development']['candidates'].values()]
 linked=w.get('current_E149_recovery_linked_image_qualifications',{}).get(profile)
 if linked:
  require(sha(linked['qualification'])==linked['qualification_sha256'],'linked qualification changed')
  lq=read(linked['qualification']);require(lq['profile']==profile,'linked profile differs')
  client_binding=client_image_binding(lq,files,source_paths)
  require(sha(lq['product'])==lq['product_sha256'],'linked product changed')
  dependencies += [Path(linked['qualification']),Path(lq['compile_cohort']),Path(lq['product'])]
 review=producer_review or w.get('current_E149_recovery_execution_development');reviewed=False;producer_pins={}
 if review:
  require(sha(review['qualification'])==review['qualification_sha256'],'producer review changed')
  pr=read(review['qualification']);producer_pins=pr['source_pins'];check_pins(producer_pins)
  reviewed=pr['runtime_producer_source_complete']
  dependencies += [Path(review['qualification']),*[Path(n) for n in producer_pins]]
 out=R/'維護工具/candidates'/('native-E149-recovery-case-plan-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
 p=dict(schema='E149-recovery-case-plan-v4',status='development_plan_only',profile=profile,scenario=scenario,test_name=SCENARIOS[scenario][0],
  source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,input_pins={str(p):sha(p) for p in dependencies}|iq['Java_classes'],
  effective_overlay_files=files,effective_overlay_paths=source_paths,base_compiled_inputs=str(Path(q['run'])/'base-compiled-inputs.json'),
  fixture_source_binding=fixture_ref['qualification'],fixture_source_binding_sha256=fixture_ref['qualification_sha256'],
  fixture_qualification=str(iqpath),fixture_qualification_sha256=binding['fixture_qualification_sha256'],
  Java_source_pins=binding['Java_source_pins'],Java_classes=iq['Java_classes'],client_source_binding=client_binding if linked else None,
  shared_client_resources_required=True,native_item_attribute_observer_kind='G06_E149_native_item_fields_v1',
  owner_guard_probe_kind=binding['owner_guard_probe_kind'],
  retired_item_mutation_probe_kind=binding['retired_item_mutation_probe_kind'],retired_item_mutation_methods=binding['retired_item_mutation_methods'],
  retired_item_mutation_targets=binding['retired_item_mutation_targets'],retired_item_mutation_attempts=binding['retired_item_mutation_attempts'],
  retired_item_morph_probe_kind=binding['retired_item_morph_probe_kind'],retired_item_morph_methods=binding['retired_item_morph_methods'],
  retired_item_morph_targets=binding['retired_item_morph_targets'],retired_item_morph_attempts=binding['retired_item_morph_attempts'],retired_item_aggregate_attempts=binding['retired_item_aggregate_attempts'],
  owner_identity_contract=str(OWNER_IDENTITY_CONTRACT),owner_identity_contract_sha256=sha(OWNER_IDENTITY_CONTRACT),
  normal_product_pins=q['normal_product_pins'],fixture_classes=str(Path(iq['compile_run'])/'classes'),
  formal_reference=ref['cached_reference'],JDBC=bundle['JDBC']['cached'],catalog=str(catalog),SQL=static['SQL'],
  database_bundle=db['bundle_manifest'],static_manifest=db['static_manifest'],
  java=r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe',
  java_sha256=sha(Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe')),
  python=str(Path(sys.executable).resolve()),python_sha256=sha(sys.executable),
  future_linked_image_qualification_required=True,linked_image_qualified=bool(linked),
  linked_image_qualification=linked['qualification'] if linked else None,linked_image_qualification_sha256=linked['qualification_sha256'] if linked else None,
  runtime_producer_review=review['qualification'] if reviewed else None,runtime_producer_review_sha256=review['qualification_sha256'] if reviewed else None,
  runtime_producer_pins=producer_pins,runtime_producer_implementation_pending=not reviewed,
  test_execution_deferred_at_preparation=w['current_user_requested_goal']['test_execution_deferred_by_user'],
  database_started=False,SQL_executed=False,tests_executed=0,
  desktop_inputs=0,full_Main_process_qualified=False,full_native_owner_qualified=False,whole_goal_complete=False)
 write_new(out/'plan.json',p);validate_plan(out/'plan.json');return out/'plan.json'
