"""Qualify Java compilation and source/bytecode only; publish no runtime pass."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
METHODS=['setCount','setEquipped','setIdentified','setSealed','setEnchantLevel','set_durability','setBless','setChargeCount']

def qualify(run,inspection_path,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-recovery-bootstrap-compile');c=read(run/'cohort.json')
 require(c['status']=='integrated_fixture_compilation_only_complete' and c['source_manifest_sha256']==SOURCE
  and c['formal_target_sha256']==FORMAL and len(c['process_history'])==1 and c['process_history'][0]['name']=='Java'
  and c['process_history'][0]['exit_code']==0 and len(c['classes'])==6 and c['metadata']=={},'original Java-only compilation incomplete')
 require(c['tests_executed']==c['classes_instantiated']==0 and c['SQL_executed'] is False and c['database_started'] is False,'runtime work present in compile receipt')
 external={name:h for name,h in c['input_pins'].items() if not Path(name).is_relative_to(R)}
 javac=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javac.exe')
 require(set(external)=={str(javac)} and sha(javac)==external[str(javac)] and c['process_history'][0]['command'][0]==str(javac),'read-only compiler identity differs')
 check_pins({name:h for name,h in c['input_pins'].items() if name not in external});check_pins(c['classes']);check_pins(c['process_history'][0]['logs'])
 cp=Path(c['candidate']);candidate=read(cp);require(sha(cp)==c['candidate_sha256'] and candidate['status']=='retired_bound_item_mutation_denial_source_not_active_not_tested','candidate identity differs')
 require(candidate['mutation_methods']==METHODS and candidate['planned_retired_setter_attempts']==32
  and candidate['Runtime_files_changed']==candidate['Rust_files_changed']==0 and candidate['all_Rust_inputs_byte_identical'] is True,'mutation/Rust scope differs')
 check_pins({str(cp.parent/name):h for name,h in candidate['candidate_files'].items()});check_pins(candidate['mutation_fragments'])
 require(sha(candidate['predecessor_candidate'])==candidate['predecessor_candidate_sha256'],'original canonical source changed')
 old=read(candidate['predecessor_candidate']);require(len(candidate['candidate_files'])==4
  and {name for name,h in candidate['candidate_files'].items() if h!=old['candidate_files'][name]}==set(candidate['changed_files'])
  and len(candidate['changed_files'])==2 and all(name.endswith('.java') for name in candidate['changed_files']),'unexpected candidate delta')
 require(sha(candidate['mutation_API_inspection'])==candidate['mutation_API_inspection_sha256'],'original setter contract changed')
 api=read(candidate['mutation_API_inspection']);check_pins(api['outputs'])
 require(api['formal_target_sha256']==FORMAL and all(row['exit_code']==0 for row in api['commands']),'original setter bytecode target differs')
 inspection_path=owned(inspection_path.resolve(strict=True),run);inspection=read(inspection_path)
 require(inspection['compile_cohort_sha256']==sha(run/'cohort.json') and all(row['exit_code']==0 for row in inspection['commands']),'compiled bytecode inspection differs')
 check_pins(inspection['outputs']);directory=inspection_path.parent
 observer=(directory/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig');harness=(directory/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig')
 require(not re.search(r'\bputfield\b|java/lang/reflect/Field\.(?:set:|setInt:|setLong:|setBoolean:)',observer+harness),'direct/reflective game field writes in QA classes')
 require('RuntimeInventory.items:' not in observer,'observer refreshes live inventory views')
 calls=lambda text:{match.group(1) for line in text.splitlines() if (match:=re.search(r'// (?:InterfaceMethod|Method) (local/lito381/server/[^\s]+)',line))}
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'user deferral or formal identity changed')
 prior=w['current_E149_owner_guard_development'];require(sha(prior['qualification'])==prior['qualification_sha256'],'prior guard qualification changed')
 prior_q=read(prior['qualification']);prior_directory=Path(prior_q['bytecode_inspection']).parent
 require(sha(directory/'inspector.py')==prior_q['source_pins'][str(T/'inspect-native-E149-owner-guard-candidate.py')]
  and sha(run/'compiler.py')==sha(T/'compile-native-E149-recovery-bootstrap.py'),'reused compiler/disassembler source differs')
 require(calls(observer)==calls((prior_directory/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig')) and len(calls(observer))==18,'original 18 record accessors changed')
 old_calls=calls((prior_directory/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig'))
 require(old_calls<=calls(harness),'previous guard/bootstrap game calls were removed')
 delta=calls(harness)-old_calls
 game_delta={name for name in delta if not name.startswith('local/lito381/server/NativeRecoveryReadonlySnapshot.')}
 expected={'local/lito381/server/NativeDeWorldKinds$Item.'+method+':('+('J' if method=='setCount' else 'Z' if method in ['setEquipped','setIdentified','setSealed'] else 'I')+')V' for method in METHODS}
 require(game_delta==expected,'compiled harness game method delta is not the eight reviewed setters')
 source=(cp.parent/'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java').read_text('utf-8')
 require(source.index('retiredOwnerGuards(ownerProofs,ownerReferences)')<source.index('retiredMutationDenials(mutationProofs,mutationTargets)')<source.index('flowComplete=true')<source.index('app.close()'),
  'retired mutations are outside original Futures/guard/hub-close barrier')
 require('original session futures not terminal before retirement guards' in source and 'targets.size()==4' in source and 'attempts==32' in source,'original admission/count barriers absent')
 tools=[T/name for name in ['inspect-native-E149-mutation-denial-contract.py','prepare-native-E149-mutation-denial-candidate.py',
  'native_E149_mutation_denials.py','qualify-native-E149-mutation-denial-development.py']]
 for path in tools:compile(ast.parse(path.read_text('utf-8'),filename=str(path)),str(path),'exec')
 inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/name)==h for name,h in inputs.items()),'normal source inputs changed');check_pins(c['normal_product_pins'])
 component=w['current_E149_component_linked_images'];require(sha(component['qualification'])==component['qualification_sha256'],'four-image qualification changed')
 for image in component['images'].values():require(sha(image['product'])==image['sha256'],'existing component PE changed')
 identities=[c['pipeline'],*c['process_history']];ids=','.join(str(row['pid']) for row in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW)
 live=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:
  require(not any(item['ProcessId']==row['pid'] and abs((datetime.fromisoformat(item['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for item in live),'original Java compiler/pipeline still live')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/name)==h for name,h in before.items()),'progress checkpoint changed')
 out=owned(V/'native-E149-mutation-denial-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 (out/'owned-CIM.json').write_bytes(observed)
 record=dict(status='retired_bound_setter_probes_Java_compilation_and_reader_source_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),
  observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,Java_compiler_exit_code=0,Java_classes=c['classes'],
  bytecode_inspection=str(inspection_path),bytecode_inspection_sha256=sha(inspection_path),
  original_setter_API_inspection=candidate['mutation_API_inspection'],original_setter_API_inspection_sha256=candidate['mutation_API_inspection_sha256'],
  original_harness_game_method_call_delta=sorted(game_delta),observer_original_18_record_accessors_unchanged=True,
  direct_game_field_writes=False,no_fake_game_objects=True,planned_owner_targets=4,planned_retired_setter_attempts=32,original_compiler_identities_terminal=True,
  AST_compiled_tools=4,source_pins={str(path):sha(path) for path in tools+[Path(name) for name in candidate['mutation_fragments']]
   +[T/'native-E149-mutation-denial-development.md',T/'compile-native-E149-recovery-bootstrap.py',T/'inspect-native-E149-owner-guard-candidate.py']},
  pure_receipt_reader_source_complete=True,pure_receipt_reader_executed=False,normal_sources_unchanged=True,normal_PEs_unchanged=True,
  prior_fixture_binding=w['current_E149_fixture_binding'],component_images_qualification=component['qualification'],component_images_qualification_sha256=component['qualification_sha256'],
  integrated_into_active_fixture=False,integrated_into_producer_reader=False,client_image_bindings_updated=False,
  Rust_compilers_invoked=0,Rust_files_changed=0,production_applied=False,tests_executed=0,classes_instantiated=0,original_mutation_setters_invoked=False,
  original_initializer_invoked=False,SQL_executed=False,database_started=False,desktop_inputs=0,
  full_native_owner_qualified=False,full_attribute_matrix_qualified=False,full_Main_process_qualified=False,whole_goal_complete=False,
  test_execution_deferred_by_user=True,limitations=['No functional parser/guard/setter/fixture/DB/GUI invocation; 32 is planned, not observed',
   'New Java candidate/reader are isolated and not bound into canonical producer or 16-case suite',
   'Capture of four real nonempty inventories, retained attachment and exact rejection/output remain unverified',
   'Eight core BoundItem setter entries do not qualify extended attributes, source/account isolation, timers or full G06'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw={Path(name).name:Path(name) for name in record['source_pins']}
 raw.update({'qualified-development.json':out/'qualified-development.json','compile-cohort.json':run/'cohort.json','candidate.json':cp,
  'contract.json':Path(candidate['contract']),'bytecode-inspection.json':inspection_path,'owned-CIM.json':out/'owned-CIM.json'})
 for name in candidate['candidate_files']:raw['candidate/'+name]=cp.parent/name
 for name in inspection['outputs']:raw['bytecode/'+Path(name).name]=Path(name)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name,path in raw.items():z.write(owned(path),name)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in raw.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work='G06新增真實退役BoundItem八setter拒絕候選及純reader來源：四個原admission各保留一實際actor，原Future與既有退役guard完成、hub close前，計畫32次不同值呼叫；只接受精確stale-owner例外與前後完整觀測一致，detached／staged／參數錯誤不算。原Java '+terminal_chunk+' actual exit0／6class，兩類javap exit0，原observer18個record accessor不變，harness僅新增八個原setter入口；4個Python模組僅AST，reader／setter／guard／案例均未呼叫。新候選與reader未接入canonical fixture／正式producer／16案計畫，原Rust及四元件PE／正常1049輸入／兩EXE不變。下一步完成新binding及外層reader整合；所有案例／桌面維持使用者暫緩，完整source/account／非預設屬性／timer／Main／G06與G01–G12未完成。'
 staged={}
 for name in before:
  path=R/name
  if path.suffix=='.json':
   value=read(path);value['current_E149_mutation_denial_development']=record
   value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
   value['current_focus']=dict(item='G06_3d_retired_native_setter_denial_source_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if name.endswith('next-validation-action.json'):value['next_action']=work
   staged[name]=encoded(value)
  else:
   _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs')
   staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name in before:z.write(R/name,name)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items()),'before archive differs')
 require(all(sha(R/name)==h for name,h in before.items()) and all(sha(R/name)==h for name,h in inputs.items()),'concurrent source/checkpoint writer');check_pins(c['normal_product_pins'])
 for name,blob in staged.items():(R/name).write_bytes(blob)
 require(all((R/name).read_bytes()==blob for name,blob in staged.items()),'progress readback differs')
 (out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={name:sha(R/name) for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),Java_classes=6,planned_setter_attempts=32,actual_setter_attempts=0,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('run',type=Path);ap.add_argument('--inspection',type=Path,required=True);ap.add_argument('--terminal-chunk',required=True)
 a=ap.parse_args();qualify(a.run,a.inspection,a.terminal_chunk)
if __name__=='__main__':main()
