"""Publish compilation/static custody only. No owner guards or cases execute."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,FORMAL,read,sha,encoded,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-E149-recovery-bootstrap-compile')
c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp)
assert sha(cp)==c['candidate_sha256'] and c['status']=='integrated_fixture_compilation_only_complete'
assert len(c['process_history'])==1 and c['process_history'][0]['name']=='Java' and c['process_history'][0]['exit_code']==0
assert c['metadata']=={} and len(c['classes'])==6 and c['tests_executed']==c['classes_instantiated']==0
external={n:h for n,h in c['input_pins'].items() if not Path(n).is_relative_to(R)}
compiler=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javac.exe')
assert set(external)=={str(compiler)} and c['process_history'][0]['command'][0]==str(compiler)
assert sha(compiler)==external[str(compiler)] # read-only toolchain identity, no write outside R
check_pins({n:h for n,h in c['input_pins'].items() if n not in external});check_pins(c['classes']);check_pins(c['process_history'][0]['logs'])
assert candidate['all_Rust_inputs_byte_identical'] and candidate['Runtime_files_changed']==candidate['Rust_files_changed']==0
assert not candidate['original_owner_guards_invoked'] and not candidate['integrated_into_active_fixture']
old=read(candidate['predecessor_candidate']);assert sha(candidate['predecessor_candidate'])==candidate['predecessor_candidate_sha256']
assert len(candidate['candidate_files'])==4
assert set(candidate['changed_files'])=={n for n,h in candidate['candidate_files'].items() if h!=old['candidate_files'][n]}
assert len(candidate['changed_files'])==2 and all(n.endswith('.java') for n in candidate['changed_files'])
check_pins({str(cp.parent/n):h for n,h in candidate['candidate_files'].items()})
fragment=Path(candidate['guard_fragment']);assert sha(fragment)==candidate['guard_fragment_sha256']
source=fragment.read_text('utf-8');harness=(cp.parent/'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java').read_text('utf-8')
assert 'references.size()<=16' in source and 'references.size()==4' in source
assert source.count('owner.requireActive(inventory);')==3
assert 'owner.matches(foreignSession)' in source and 'foreign.account.equals(owner.account)' in source
assert 'source item owner is stale' in source and 'source item inventory is staged or inactive' in source
assert 'new NativeDeItemApplication.Owner' not in source and '.adopt(' not in source and '.retire(' not in source
assert '.set(' not in source and 'owner.active=' not in source and 'System.currentTimeMillis' not in source
assert harness.index('continuedOwnerGuards(current,row,ownerProofs,ownerReferences)')<harness.index('NativePeerHarness.write(observed,row)')
assert harness.index('retiredOwnerGuards(ownerProofs,ownerReferences)')<harness.index('flowComplete=true')<harness.index('app.close()')
assert 'original session futures not terminal before retirement guards' in harness
inspection_path=next(run.glob('owner-guard-bytecode-*/inspection.json'));inspection=read(inspection_path)
assert inspection['compile_cohort_sha256']==sha(run/'cohort.json') and all(row['exit_code']==0 for row in inspection['commands'])
check_pins(inspection['outputs']);directory=inspection_path.parent
observer=(directory/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig')
compiled_harness=(directory/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig')
assert not re.search(r'\bputfield\b',observer+compiled_harness)
assert not re.search(r'java/lang/reflect/Field\.(?:set:|setInt:|setLong:|setBoolean:)',observer+compiled_harness)
assert 'RuntimeInventory.items:' not in observer
calls=lambda text:{m.group(1) for line in text.splitlines() if (m:=re.search(r'// (?:InterfaceMethod|Method) (local/lito381/server/[^\s]+)',line))}
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
prior=w['current_E149_item_attribute_observer_development'];prior_q=read(prior['qualification'])
assert sha(prior['qualification'])==prior['qualification_sha256']
prior_directory=Path(prior_q['bytecode_inspection']).parent
assert calls(observer)==calls((prior_directory/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig')) and len(calls(observer))==18
delta=calls(compiled_harness)-calls((prior_directory/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig'))
own_prefix='local/lito381/server/NativeRecoveryReadonlySnapshot.'
original_delta={name for name in delta if not name.startswith(own_prefix)}
assert original_delta=={
 'local/lito381/server/NativeDeItemApplication$Owner.requireActive:(Llocal/lito381/server/RuntimeInventory;)V',
 'local/lito381/server/NativeDeItemApplication$Owner.matches:(Llocal/lito381/server/ClientSession;)Z'},original_delta
api=read(candidate['guard_API_bytecode_inspection']);assert sha(candidate['guard_API_bytecode_inspection'])==candidate['guard_API_bytecode_inspection_sha256']
assert api['formal_target_sha256']==FORMAL and all(row['exit_code']==0 for row in api['commands']);check_pins(api['outputs'])
assert sha(candidate['guard_owner_bytecode'])==candidate['guard_owner_bytecode_sha256']
client_bytecode=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt'
client_text=client_bytecode.read_text('utf-8-sig');start=client_text.index('  local.lito381.server.RuntimeInventory nativeDeInventory(int);')
getter=client_text[start:client_text.index('\n  void nativeDeSetInventory',start)]
assert 'tableswitch' in getter and getter.count('getfield')==4 and 'putfield' not in getter
failure=V/'native-E149-owner-guard-development-failures/20261023-225514-052970/failure.json'
failed=read(failure);assert failed['actual_exit_code']==1 and failed['observed_original_tool_chunk']=='db78dd'
inputs=read(c['base_compiled_inputs']);assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items());check_pins(c['normal_product_pins'])
assert sha(w['current_formal_reference_identity']['formal'])==FORMAL
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
tools=[T/'prepare-native-E149-owner-guard-candidate.py',T/'inspect-native-E149-owner-guard-candidate.py',Path(__file__)]
for path in tools:compile(ast.parse(path.read_text('utf-8'),filename=str(path)),str(path),'exec')
identities=[c['pipeline'],*c['process_history']];ids=','.join(str(row['pid']) for row in identities)
query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW)
actual=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];actual=[actual] if isinstance(actual,dict) else actual
for row in identities:
 assert not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)
out=V/'native-E149-owner-guard-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'owned-CIM.json').write_bytes(observed)
record=dict(status='original_owner_guard_probe_Java_compilation_and_bytecode_only',candidate=str(cp),candidate_sha256=sha(cp),
 compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,
 Java_compiler_exit_code=0,Java_classes=c['classes'],Rust_compilers_invoked=0,Rust_files_changed=0,
 formal_target_sha256=FORMAL,source_manifest_sha256=c['source_manifest_sha256'],
 bytecode_inspection=str(inspection_path),bytecode_inspection_sha256=sha(inspection_path),
 guard_API_bytecode_inspection=candidate['guard_API_bytecode_inspection'],guard_API_bytecode_inspection_sha256=candidate['guard_API_bytecode_inspection_sha256'],
 guard_fragment=str(fragment),guard_fragment_sha256=sha(fragment),original_harness_game_method_call_delta=sorted(original_delta),
 client_inventory_getter_bytecode=str(client_bytecode),client_inventory_getter_bytecode_sha256=sha(client_bytecode),
 input_boundary_tooling_failure=str(failure),input_boundary_tooling_failure_sha256=sha(failure),
 observer_original_18_record_accessors_unchanged=True,game_field_writes_in_candidate=False,
 same_live_owner_positive_control_before_negative=True,retired_checks_before_hub_shutdown=True,
 actual_original_session_references_only=True,maximum_retained_owner_references=16,expected_selected_owner_admissions=4,
 guard_scope=candidate['guard_scope'],original_compiler_identities_terminal=True,AST_compiled_tools=3,
 source_pins={str(p):sha(p) for p in tools+[fragment,T/'native-E149-owner-guard-development.md']},
 prior_fixture_development=prior['qualification'],prior_fixture_development_sha256=prior['qualification_sha256'],
 prior_current_fixture_binding=w['current_E149_fixture_binding'],
 integrated_into_active_fixture=False,integrated_into_producer_reader=False,client_image_bindings_updated=False,
 normal_sources_unchanged=True,normal_PEs_unchanged=True,production_applied=False,
 tests_executed=0,classes_instantiated=0,original_owner_guards_invoked=False,original_initializer_invoked=False,
 SQL_executed=False,database_started=False,desktop_inputs=0,
 full_Main_process_qualified=False,full_native_owner_qualified=False,full_attribute_matrix_qualified=False,whole_goal_complete=False,
 limitations=['Compilation/static bytecode only; original owner requireActive/matches and QA probes were not invoked',
  'New candidate is not canonical; current producer/reader/plans keep the preceding fixture context',
  'Actual positive/foreign-inventory/retired results, sampling races, observer identity/Future/cleanup linkage remain unverified',
  'Guard-only rejection is not an attempted item mutation or full independent source/account/timer/expiry-denial matrix'],
 test_execution_deferred_by_user=True,recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified-development.json').write_bytes(encoded(record))
record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),
 previous_goal_turn_evidence=w['last_goal_turn_evidence'])
raw={Path(n).name:Path(n) for n in record['source_pins']}
raw.update({'qualified-development.json':out/'qualified-development.json','compile-cohort.json':run/'cohort.json',
 'candidate.json':cp,'bytecode-inspection.json':inspection_path,'owned-CIM.json':out/'owned-CIM.json'})
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name,path in raw.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items())
record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'))
(out/'evidence.json').write_bytes(encoded(record))
work='E149新增獨立owner guard候選：真實接續owner先通過requireActive才ACK；另一帳號實際inventory／session拒絕前後仍接受自身inventory；四個選角GAME原Future退出後、hub shutdown前核對retired stale拒絕。只取實際owner／session／inventory參考，不造owner或改active／物品／時鐘，guard-only不算完整mutation拒絕矩陣。原580956 actual exit0編譯6class；javap兩類exit0、原observer18accessor不變，harness只新增原requireActive／matches兩方法路徑，未呼叫guard。Rust／正常1049輸入及兩EXE不變。新候選未接入active fixture／producer／reader或兩版計畫；下步固定guard trace並逐角色讀回，仍分開綁原Rust映像及新Java資格。全部測試／桌面操作維持使用者暫緩；Job／DB／SQL／initializer／案例均未跑，完整owner／Main／G01–G12未完成。'
staged={}
for name in before:
 path=R/name
 if path.suffix=='.json':
  j=read(path);j['current_E149_owner_guard_development']=record
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_3d_E149_original_owner_guard_source_candidate',work=work,applies_to_normal_EXE=False,goal_complete=False)
  if name.endswith('next-validation-action.json'):j['next_action']=work
  staged[name]=encoded(j)
 else:
  _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name in before:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items());check_pins(c['normal_product_pins'])
for name,blob in staged.items():(R/name).write_bytes(blob)
assert all((R/n).read_bytes()==blob for n,blob in staged.items())
(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},
 before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),Java_classes=6,original_guards_invoked=False,
 tests_executed=0,active_fixture_changed=False,eight_files_read_back=True),ensure_ascii=False))
