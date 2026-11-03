"""Read actual baseline compiler receipts and source custody; no test execution."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-E149-component-matrix-compile')
c=read(run/'cohort.json');plan=Path(c['plan']);p=read(plan)
assert sha(plan)==c['plan_sha256'] and c['status']=='baseline_metadata_complete_no_tests' and c['metadata_only'] is True
assert c['tests_executed']==0 and not c['PEs_executed'] and c['owned_process'] is None
assert p['source_manifest_sha256']==SOURCE and p['formal_target_sha256']==FORMAL
check_pins(p['input_pins']);check_pins(p['normal_product_pins'])
assert all(sha(n)==h for n,h in p['external_toolchain_dependency_pins'].items())
assert [(x['profile'],x['variant']) for x in c['compiler_runs']]==[('Debug','baseline'),('Release','baseline')]
for row in c['compiler_runs']:
 assert row['exit_code']==0 and row['finished_utc'] and '--test' in row['command'] and '--emit=metadata' in row['command']
 assert '--emit=link' not in row['command'] and row['command'].count(p['roots']['baseline'])==1
 assert Path(row['product']).suffix=='.rmeta' and sha(row['product'])==row['product_sha256']
 assert not row.get('exact_original_compiler_cleanup',False);check_pins(row['logs'])
 commands=p['compiler_templates'][row['profile']]
 expected=list(commands);expected[expected.index(p['roots']['candidate'])]=p['roots']['baseline'];expected[expected.index('-o')+1]=row['product']
 assert row['command']==expected
for item in p['source_rewrites']:
 text=Path(item['origin']).read_text('utf-8-sig')
 for replacement in item['replacements']:
  assert replacement['before'] in text
  assert re.fullmatch(r'#\[path\s*=\s*"[^"\n]+"\]|(?:include_bytes|include_str|include)!\("[^"\n]+"\)',replacement['before'])
  text=text.replace(replacement['before'],replacement['after'],1)
 assert Path(item['generated']).read_text('utf-8')==text
base_root=Path(p['roots']['baseline']);candidate_root=Path(p['roots']['candidate'])
assert base_root.read_text('utf-8')==candidate_root.read_text('utf-8').replace(str(candidate_root.parent/'modern_lito.rs'),str(base_root.parent/'modern_lito.rs'))
before='#[path = r#"'+p['root_replacement_only']['original_native_login']+'"#]'
after='#[path = r#"'+p['root_replacement_only']['baseline_native_login']+'"#]'
original_mod=(candidate_root.parent/'modern_lito.rs').read_text('utf-8');assert original_mod.count(before)==1
assert (base_root.parent/'modern_lito.rs').read_text('utf-8')==original_mod.replace(before,after)
old_map=read(p['module_maps']['candidate']);new_map=read(p['module_maps']['baseline'])
assert len(old_map)==len(new_map)==362
assert [row['module'] for row in old_map]==[row['module'] for row in new_map]
assert [i for i,(old,new) in enumerate(zip(old_map,new_map)) if old!=new]==[next(i for i,row in enumerate(old_map) if row['module']=='native_login')]
assert len(p['cases']['baseline'])==23 and len(p['cases']['candidate'])==28
assert sum(x['existing_control'] for x in p['cases']['baseline'])==sum(x['existing_control'] for x in p['cases']['candidate'])==17
assert sum(x['expected']=='specific_assertion_failure' for x in p['cases']['baseline'])==6
assert sum(not x['existing_control'] for x in p['cases']['candidate'])==11 and p['new_formal_cases_separate']==3
assert all(x['expected']=='pass' for x in p['cases']['candidate'])
assert all(x['panic_contains'] for x in p['cases']['baseline'] if x['expected']=='specific_assertion_failure')
q=read(p['original_metadata_qualification']);assert sha(p['original_metadata_qualification'])==p['original_metadata_qualification_sha256']
assert set(q['profiles'])=={'Debug','Release'} and all(q['profiles'][profile]['library-test-code']['compiler_exit_code']==0 for profile in q['profiles'])
inputs=read(p['base_compiled_inputs']);assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL
before_files=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before_files)==8 and all(sha(R/n)==h for n,h in before_files.items())
failure=V/'native-E149-component-matrix-tooling-failures/20261023-231716-491955/failure.json'
f=read(failure);assert f['actual_exit_code']==1 and f['compilers_started']==0
tools=[T/'prepare-native-E149-component-matrix.py',T/'compile-native-E149-component-matrix.py',Path(__file__)]
for path in tools:compile(ast.parse(path.read_text('utf-8'),filename=str(path)),str(path),'exec')
identities=[c['pipeline'],*[row['original_process'] for row in c['compiler_runs']]];ids=','.join(str(row['pid']) for row in identities)
query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW)
actual=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];actual=[actual] if isinstance(actual,dict) else actual
epoch=datetime(1970,1,1,tzinfo=timezone.utc)
for row in identities:
 for other in actual:
  if row['pid']==other['ProcessId']:
   value=datetime.fromisoformat(other['created'].replace('Z','+00:00'))-epoch
   millis=(value.days*86400+value.seconds)*1000+value.microseconds//1000
   assert millis!=row['start_epoch_millis'],'same original compiler/pipeline still live'
out=V/'native-E149-component-matrix-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'owned-CIM.json').write_bytes(observed)
record=dict(status='baseline_both_profile_metadata_and_component_matrix_source_only',plan=str(plan),plan_sha256=sha(plan),
 compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,
 baseline_metadata_compiler_exit_codes=[0,0],compiler_identities_terminal=True,metadata_products={x['product']:dict(sha256=x['product_sha256'],bytes=x['product_bytes']) for x in c['compiler_runs']},
 original_candidate_metadata_qualification=p['original_metadata_qualification'],original_candidate_metadata_qualification_sha256=p['original_metadata_qualification_sha256'],
 source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,full_module_declarations=362,normal_compiled_inputs=1049,
 per_profile_cases=p['expected_per_profile'],baseline_selected_cases=23,candidate_selected_cases=28,new_candidate_components=11,new_formal_cases_separate=3,
 only_baseline_native_login_Runtime_replaced=True,other_independent_candidates_identical=True,source_rewrites_are_paths_only=True,
 candidate_linked_images_pending=True,baseline_linked_images_pending=True,test_harness_inventory_executed=False,
 source_pins={str(path):sha(path) for path in tools+[T/'native-E149-component-matrix.md']},AST_compiled_tools=3,
 tooling_failure=str(failure),tooling_failure_sha256=sha(failure),
 tests_executed=0,baseline_counterexamples_observed=False,candidate_tests_passed=False,PEs_executed=False,GUI_inputs=0,
 normal_sources_unchanged=True,normal_PEs_unchanged=True,production_applied=False,
 full_library_qualified=False,full_G06_qualified=False,whole_goal_complete=False,test_execution_deferred_by_user=True,
 limitations=['Baseline compiler exit0 does not demonstrate six failures or create executable test evidence; empty test rmeta is only a marker',
  'Source-derived 23/28 selections still require real libtest inventory after test resumption',
  'Need four independently qualified linked PE and exact process/exit/assertion readback; crash/build failure cannot qualify expected-red',
  'Seventeen original controls and eleven new components are separate from three new formal cases and the 16-case formal suite'],recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified-development.json').write_bytes(encoded(record))
record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
raw={Path(n).name:Path(n) for n in record['source_pins']}
raw.update({'qualified-development.json':out/'qualified-development.json','plan.json':plan,'compile-cohort.json':run/'cohort.json','owned-CIM.json':out/'owned-CIM.json'})
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name,path in raw.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(path) for n,path in raw.items())
record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'))
(out/'evidence.json').write_bytes(encoded(record))
work='G06元件對照已建立原版／候選全362模組隔離來源與逐名清單：每版原版17原有控制＋6指定預期反例＝23案，候選17原有＋11新元件＝28案；另3新正式案才合計原提案14案。原版僅native_login Runtime換回正常3E，其他T鍵／#20候選相同，配對QA避免呼叫候選私有方法。原版Debug／Release --emit=metadata實際兩命令exit0，原程序終止核對；候選原全圖metadata保留原資格。未連結元件PE、未執行--list或案例；空test rmeta不算測試。後續需四份PE獨立連結資格及精確執行／斷言reader：原17控制先過，6反例需指定assert panic，編譯錯／crash不算重現。正常1049輸入／兩EXE不變，原16案正式suite與owner契約不變；全部測試／桌面仍暫緩，完整library／Release／GUI／G01–G12未完成。'
staged={}
for name in before_files:
 path=R/name
 if path.suffix=='.json':
  j=read(path);j['current_E149_component_matrix_development']=record
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_3d_original_candidate_component_matrix_metadata',work=work,applies_to_normal_EXE=False,goal_complete=False)
  if name.endswith('next-validation-action.json'):j['next_action']=work
  staged[name]=encoded(j)
 else:
  _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name in before_files:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before_files.items())
assert all(sha(R/n)==h for n,h in before_files.items()) and all(sha(R/n)==h for n,h in inputs.items());check_pins(p['normal_product_pins'])
for name,blob in staged.items():(R/name).write_bytes(blob)
assert all((R/n).read_bytes()==blob for n,blob in staged.items())
(out/'readback.json').write_bytes(encoded(dict(before_files=before_files,after_files={n:sha(R/n) for n in before_files},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),baseline_metadata_compiler_exits=[0,0],baseline_cases=23,candidate_cases=28,
 tests_executed=0,baseline_failures_observed=False,eight_files_read_back=True),ensure_ascii=False))
