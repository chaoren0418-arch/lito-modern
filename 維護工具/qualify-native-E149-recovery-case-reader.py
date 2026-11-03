"""Publish reader/contract development only. Do not run the result reader."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('plan',type=Path);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
plan_path=a.plan.resolve(strict=True);assert plan_path.is_relative_to(T/'candidates');p=read(plan_path)
assert p['schema']=='E149-recovery-case-plan-v1' and p['status']=='development_plan_only'
assert p['scenario']=='cancel-repeat' and p['profile']=='Debug' and p['runtime_producer_implementation_pending']
assert not p['linked_image_qualified'] and p['tests_executed']==p['desktop_inputs']==0
assert not p['database_started'] and not p['SQL_executed']
assert all(sha(n)==h for n,h in p['input_pins'].items())
assert sha(p['java'])==p['java_sha256'] and sha(p['python'])==p['python_sha256']
sources=[T/'native_E149_recovery_contract.py',T/'prepare-native-E149-recovery-case.py',T/'verify-native-E149-recovery-case.py']
trees={}
for path in sources:
 tree=ast.parse(path.read_text('utf-8'),filename=str(path));compile(tree,str(path),'exec');trees[path.name]=tree
 # Main guards are the only module-level conditional calls.
 for node in tree.body:
  if isinstance(node,ast.If):assert ast.unparse(node.test)=="__name__ == '__main__'"
reader=sources[2].read_text('utf-8');contract=sources[0].read_text('utf-8')
assert 'subprocess.Popen' not in reader and 'subprocess.Popen' not in contract
functions={n.name:n for n in trees[sources[2].name].body if isinstance(n,ast.FunctionDef)}
body=ast.unparse(functions['verify_execution'])
assert body.index('runtime_producer_implementation_pending')<body.index('check_terminal_processes(rows)')
for marker in ['effective_overlay_files','compile_cohort_sha256','--emit=link','creation_filetime_ticks',
 'database_original_handle_signalled','static_SQL_access','player_SQL_access','continuation-observed.json',
 'source_item_disconnected','source_item_owner','raw_native_bless','detail_hex']:
 assert marker in reader+contract,marker
assigned={target.id:node.value for node in trees[sources[0].name].body if isinstance(node,ast.Assign) for target in node.targets if isinstance(target,ast.Name)}
assert len(ast.literal_eval(assigned['SCENARIOS']))==8 and len(ast.literal_eval(assigned['ITEM_FIELDS']))==17
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
ref=w['current_formal_reference_identity'];assert sha(ref['formal'])==ref['formal_sha256']==p['formal_target_sha256']
expected={};paths={}
for row in w['current_isolated_development']['candidates'].values():
 cp=Path(row['path']);assert sha(cp)==row['sha256']
 for name,h in read(cp)['candidate_files'].items():
  assert name not in expected and sha(cp.parent/name)==h;expected[name]=h;paths[name]=str(cp.parent/name)
assert len(expected)==13
iq=w['current_E149_integrated_fixture_development'];cp=Path(iq['candidate']);c=read(cp)
assert sha(cp)==iq['candidate_sha256'] and iq['continuation_observation_barrier_source_complete']
assert iq['server_continuation_phase']=='LOBBY' and iq['observed_original_tool_chunk']=='99fd68'
overlap=set(expected)&set(c['candidate_files']);assert len(overlap)==3
for name,h in c['candidate_files'].items():expected[name]=h;paths[name]=str(cp.parent/name)
assert len(expected)==14 and expected==p['effective_overlay_files'] and paths==p['effective_overlay_paths']
assert all(sha(path)==expected[name] for name,path in paths.items())
inputs=read(p['base_compiled_inputs']);assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(n)==h for n,h in p['normal_product_pins'].items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=V/'native-E149-recovery-case-reader-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
record=dict(status='case_contract_plan_preparation_and_passive_reader_AST_development_only',
 source_manifest_sha256=p['source_manifest_sha256'],formal_target_sha256=p['formal_target_sha256'],
 source_pins={str(path):sha(path) for path in sources+[T/'native-E149-recovery-case-contract.md']},
 plan=str(plan_path),plan_sha256=sha(plan_path),observed_original_preparation_tool_chunk=a.terminal_chunk,observed_preparation_actual_exit_code=0,
 AST_compiled_modules=len(trees),reader_module_imported=False,reader_executed=False,plan_preparer_executed=True,
 original_overlay_files=13,integration_files=4,integration_overlap_files=3,effective_overlay_files=14,
 source_overlay_mapping_independently_reconstructed=True,case_scenarios=8,observed_item_field_names=17,
 reader_scope=['fixed source/14-file overlay, real linked-image/raw compiler evidence, exact Java/JDBC/test commands',
  'actual 19-table/6277-row private import and static-only SQL scope',
  'original PID/FILETIME/image, actual waits and fresh OS terminal observations',
  'explicit continued-inventory observation acknowledgement before next rental',
  'per-account owner/source/character/inventory binding, generation retirement and authoritative movement',
  '17 existing item observations and exact S7C fields/opaque extended detail bytes',
  'natural stage/source waits, unused staged versus fresh selection, disconnected owners observed retired'],
 integrated_fixture_qualification=iq['qualification'],integrated_fixture_qualification_sha256=iq['qualification_sha256'],
 continuation_barrier_compiler_original_chunk='99fd68',continuation_barrier_compiler_actual_exit_code=0,
 linked_image_builder_pending=True,isolated_linked_image_qualified=False,runtime_producer_implementation_pending=True,
 passive_reader_source_complete=True,reader_runtime_qualified=False,tests_executed=0,SQL_executed=False,database_started=False,
 classes_instantiated=0,desktop_inputs=0,original_factories_invoked=False,normal_sources_unchanged=True,normal_PEs_unchanged=True,
 current_thirteen_file_overlay_unchanged=True,candidate_applied=False,full_Main_process_qualified=False,full_native_owner_qualified=False,
 limitations=['No actual execution receipt exists and no result reader/negative gate test was run',
  'Current plans deliberately keep pending producer and linked-image gates and cannot qualify a case',
  'Seventeen observed fields do not cover all native elemental/card/magic-stone internals or item UI',
  'CaptureLog has no per-session wire identifier; aggregate C8 evidence remains explicitly limited',
  'No DB/factory/session/actual owner/wire/cleanup/library/GUI runtime evidence is inferred from AST or compiler success'],
 test_execution_deferred_by_user=True,whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
prior=T/'candidates/native-E149-recovery-case-plan-20261023-210813-863715'
assert sha(prior/'contract-before-strengthened-input-pins.py')==read(prior/'plan.json')['input_pins'][str(sources[0])]
record.update(superseded_preparation_plan=str(prior/'plan.json'),superseded_preparation_plan_sha256=sha(prior/'plan.json'),
 superseded_reason='Added explicit baseline-manifest and candidate-manifest pins; older prepared dependency version retained, no runtime execution')
(out/'qualified-development.json').write_bytes(data(record))
raw={path.name:path for path in sources+[T/'native-E149-recovery-case-contract.md']}
raw.update({'reader.py':Path(__file__),'plan.json':plan_path,'qualified-development.json':out/'qualified-development.json',
 'prior-plan.json':prior/'plan.json','prior-contract.py':prior/'contract-before-strengthened-input-pins.py',
 'integrated-fixture-qualification.json':Path(iq['qualification']),'report-updater-before.py':T/'update-master-report-text-index-order.py'})
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name,path in raw.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in raw.items())
record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
(out/'evidence.json').write_bytes(data(record))
work='E149驗證接續已補LOBBY實際觀測確認點，原99fd68之Java6class／兩版正式metadata通過，均未執行。新增被動結果reader與8案契約，核對17項既有物品觀察欄、完整S7C資料、靜態SQL／禁止玩家SQL、owner綁定與退役、自然33／31秒、原程序實際退出；3個Python僅AST編譯，reader本身未執行。原'+a.terminal_chunk+' exit0只產生準備計畫；13檔＋4檔依3重疊獨立重建14檔有效覆蓋，均未套用。正常1049輸入／兩EXE不變；依使用者要求全部DB／factory／功能／GUI測試暫緩。接續開發隔離rlib／正式案例PE builder及獨立連結核對、owned case orchestrator，再於恢復測試後實跑反例／14案／完整library／Release／GUI，整體未完成。'
staged={}
for name in before:
 path=R/name
 if path.suffix=='.json':
  j=read(path);j['current_E149_recovery_case_reader_development']=record
  j['current_E149_integrated_fixture_development'].update(passive_result_reader_development=record['qualification'],passive_result_reader_development_sha256=record['qualification_sha256'],future_owned_case_orchestrator_pending=True)
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_3d_E149_passive_reader_and_case_contract_development',work=work,applies_to_current_source=False,goal_complete=False)
  if name.endswith('next-validation-action.json'):j['next_action']=work
  staged[name]=data(j)
 else:
  _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name in before:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(n)==h for n,h in p['normal_product_pins'].items()) and sha(ref['formal'])==ref['formal_sha256']
for name,blob in staged.items():(R/name).write_bytes(blob)
assert all((R/name).read_bytes()==blob for name,blob in staged.items())
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),AST_only=True,reader_executed=False,effective_overlay_files=14,database_started=False,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))
