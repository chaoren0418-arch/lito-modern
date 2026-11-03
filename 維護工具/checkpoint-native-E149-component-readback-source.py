"""Publish AST/source custody for future linked-image and component readers."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import ast,hashlib,json,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
names=['native_E149_libtest_readback.py','qualify-native-E149-component-images.py','verify-native-E149-component-matrix.py',
 'native_E149_component_diagnostics.py','run-native-E149-component-matrix.py','native_E149_owned_process.py']
compile(ast.parse(Path(__file__).read_text('utf-8'),filename=str(__file__)),str(__file__),'exec')
trees={}
for name in names:
 source=(T/name).read_text('utf-8');tree=ast.parse(source,filename=name);compile(tree,name,'exec');trees[name]=tree
 for node in tree.body:
  if isinstance(node,ast.If):assert ast.unparse(node.test)=="__name__ == '__main__'"
parser=trees[names[0]]
assert all(all(alias.name=='re' for alias in node.names) for node in parser.body if isinstance(node,ast.Import))
assert not any(isinstance(node,ast.ImportFrom) for node in parser.body)
source=(T/names[0]).read_text('utf-8')
for marker in ['101','total-1',"'specific_assertion_failure'",'panic_site','panic_contains']:
 assert marker in source
for name in names[1:3]:
 source=(T/name).read_text('utf-8')
 assert 'taskkill' not in source and 'TerminateProcess' not in source and 'subprocess.Popen' not in source
calls=[node for node in ast.walk(trees[names[1]]) if isinstance(node,ast.Call) and isinstance(node.func,ast.Attribute)
 and isinstance(node.func.value,ast.Name) and node.func.value.id=='subprocess']
assert len(calls)==1 and calls[0].func.attr=='check_output'
calls=[node for node in ast.walk(trees[names[2]]) if isinstance(node,ast.Call) and isinstance(node.func,ast.Attribute)
 and isinstance(node.func.value,ast.Name) and node.func.value.id=='subprocess']
assert len(calls)==1 and calls[0].func.attr=='run'
producer=trees['run-native-E149-component-matrix.py']
execute=next(node for node in producer.body if isinstance(node,ast.FunctionDef) and node.name=='execute')
assert ast.unparse(execute.body[0])=='authorize()'
source=(T/'run-native-E149-component-matrix.py').read_text('utf-8')
for marker in ['spawn_suspended','matrix.checkpoint(True)','matrix.job=OwnedJob()',
 'DiagnosticStore(matrix.out,matrix.cleanup_authorized)','collect_exit()',"['active_processes']==0",'capture_attempted',
 'inherited_test_options_removed=True','matrix.checkpoint(False)']:
 assert marker in source
assert source.index('matrix.checkpoint(True)')<source.index('matrix.job=OwnedJob()')<source.index('DiagnosticStore(matrix.out,matrix.cleanup_authorized)')
for name in names[3:5]:
 source=(T/name).read_text('utf-8')
 assert all(marker not in source for marker in ['taskkill','subprocess.Popen','shutil.rmtree','os.system(','shell=True'])
diagnostics=trees['native_E149_component_diagnostics.py']
assert not any(isinstance(node,(ast.Import,ast.ImportFrom)) and any(alias.name in ['subprocess','ctypes'] for alias in node.names) for node in diagnostics.body)
source=(T/'native_E149_component_diagnostics.py').read_text('utf-8')
for marker in ['8_388_608','33_554_432','st_nlink==1','capture_attempted',"archive(folder/'diagnostics-after.zip',after)",
 'stable(after,scan(profile))','os.replace(temporary,target)','target.unlink()','self.require_restored(profile)']:
 assert marker in source
assert source.index("archive(folder/'diagnostics-after.zip',after)")<source.index('os.replace(temporary,target)')
source=(T/'native_E149_owned_process.py').read_text('utf-8')
assert source.index('self.k.AssignProcessToJobObject(self.handle,process)')<source.index('on_created(held)')<source.index('self.k.ResumeThread(thread)')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL
prior=w['current_E149_component_matrix_development'];assert sha(prior['qualification'])==prior['qualification_sha256']
pq=read(prior['qualification']);assert pq['baseline_metadata_compiler_exit_codes']==[0,0] and pq['tests_executed']==0
check_pins(pq['source_pins']);plan=read(pq['plan']);assert sha(pq['plan'])==pq['plan_sha256']
predecessor=w.get('current_E149_component_execution_development')
if predecessor:
 assert sha(predecessor['qualification'])==predecessor['qualification_sha256']
 assert sha(predecessor['raw_archive'])==predecessor['raw_archive_sha256']
inputs=read(plan['base_compiled_inputs']);assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items());check_pins(plan['normal_product_pins'])
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
reference=Path(w['current_native_catalog_library']['run'])/'Debug/inventory-audit/all.stdout.txt'
text=reference.read_text('utf-8-sig');assert ': test\n' in text and '2463 tests, 0 benchmarks' in text
out=V/'native-E149-component-readback-source'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
record=dict(status='component_producer_diagnostic_preservation_and_readers_source_AST_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
 source_pins={str(T/n):sha(T/n) for n in names+['native-E149-component-readback-contract.md','checkpoint-native-E149-component-readback-source.py']},
 AST_compiled_modules=6,publisher_AST_compiled=True,
 previous_component_source_qualification=predecessor['qualification'] if predecessor else None,
 previous_component_source_qualification_sha256=predecessor['qualification_sha256'] if predecessor else None,
 baseline_metadata_qualification=prior['qualification'],baseline_metadata_qualification_sha256=prior['qualification_sha256'],
 baseline_metadata_compiler_exit_codes=[0,0],
 libtest_format_reference=str(reference),libtest_format_reference_sha256=sha(reference),historical_format_only_no_qualification_transferred=True,
 component_image_qualifier_source_complete=True,component_result_reader_source_complete=True,component_producer_source_complete=True,
 diagnostic_preservation_source_complete=True,four_linked_component_images_pending=True,diagnostic_preservation_producer_pending=False,
 exact_inventory_required=True,controls_must_precede_expected_red=True,designated_panic_source_line_required=True,
 baseline_case_count_per_profile=23,candidate_case_count_per_profile=28,planned_case_executions=102,planned_inventory_processes=4,
 tests_executed=0,PEs_executed=False,parser_functions_executed=False,image_qualifier_executed=False,matrix_reader_executed=False,
 Job_objects_created=0,workload_processes_started=0,desktop_inputs=0,normal_sources_unchanged=True,normal_PEs_unchanged=True,
 component_producer_executed=False,diagnostic_store_constructed=False,diagnostic_restoration_executed=False,
 full_library_qualified=False,full_G06_qualified=False,whole_goal_complete=False,test_execution_deferred_by_user=True,
 limitations=['AST and source review only; no parser assertion vector, PE header qualification, runtime test inventory, component case or negative execution gate was run',
  'Producer and diagnostic preservation sources are complete; actual 4 linked images, original-handle Job execution, diagnostic archive/restoration and all reader checks remain pending',
  'Designated panic-site/message matching must be checked against actual raw Rust output after tests resume; source intent is not a failure reproduction',
  '102 planned selected cases are not full library, formal authentication, natural TTL, GUI or complete G06'],recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified-source.json').write_bytes(encoded(record))
record.update(qualification=str(out/'qualified-source.json'),qualification_sha256=sha(out/'qualified-source.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
raw={Path(n).name:Path(n) for n in record['source_pins']};raw['qualified-source.json']=out/'qualified-source.json'
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name,path in raw.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in raw.items())
record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'))
(out/'evidence.json').write_bytes(encoded(record))
work='G06元件producer、固定診斷JSON保護及獨立reader已完成來源，6模組AST核對，未呼叫parser／producer／診斷回復或建立Job。未來先暫停建立原程序、加入私有Job及保存PID／FILETIME才放行；每案先封存新輸出再回復原bytes／mtime，部分回復失敗保留archive並阻止新批次。reader獨立核對前後archive、精確變更、回復receipt與最終檔案。四元件PE未連結、4次inventory／102精確案均未執行，6原版指定反例未重現；原17控制先過才承認指定assert panic。正常1049輸入／兩EXE與原正式suite工具不變。下一步四映像連結與獨立資格；所有案例／桌面依使用者要求仍暫緩，完整library／Release／GUI／G01–G12未完成。'
staged={}
for name in before:
 path=R/name
 if path.suffix=='.json':
  j=read(path);j['current_E149_component_execution_development']=record
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_component_owned_producer_diagnostic_preservation_source_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
  if name.endswith('next-validation-action.json'):j['next_action']=work
  staged[name]=encoded(j)
 else:
  _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name in before:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items());check_pins(plan['normal_product_pins'])
for name,blob in staged.items():(R/name).write_bytes(blob)
assert all((R/name).read_bytes()==blob for name,blob in staged.items())
(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),AST_modules=6,tests_executed=0,parser_executed=False,component_producer_source_complete=True,eight_files_read_back=True),ensure_ascii=False))
