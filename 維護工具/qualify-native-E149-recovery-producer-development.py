"""Static source custody only; never instantiate a Job or execute a case."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import ast,hashlib,json,sys,zipfile
from native_E149_recovery_contract import R,P,FORMAL,SOURCE,owned,read,sha,encoded,check_pins,make_plan
T=R/'維護工具';V=R/'驗證紀錄/current'
names=['native_E149_recovery_contract.py','prepare-native-E149-recovery-case.py','verify-native-E149-recovery-case.py',
 'native_E149_owned_process.py','run-native-E149-recovery-case.py']
sources={name:(T/name).read_text('utf-8') for name in names};trees={}
for name,source in sources.items():
 tree=ast.parse(source,filename=name);compile(tree,name,'exec');trees[name]=tree
 for node in tree.body:
  if isinstance(node,ast.If):assert ast.unparse(node.test)=="__name__ == '__main__'"
builder=ast.parse((T/'build-native-E149-recovery-image.py').read_text('utf-8'));compile(builder,'build-native-E149-recovery-image.py','exec')
producer=sources['run-native-E149-recovery-case.py'];process=sources['native_E149_owned_process.py'];reader=sources['verify-native-E149-recovery-case.py']
execute=next(node for node in trees['run-native-E149-recovery-case.py'].body if isinstance(node,ast.FunctionDef) and node.name=='execute')
assert ast.unparse(execute.body[0])=='authorize()'
authorize=next(node for node in trees['run-native-E149-recovery-case.py'].body if isinstance(node,ast.FunctionDef) and node.name=='authorize')
assert "get('test_execution_deferred_by_user', True)" in ast.unparse(authorize)
job=next(node for node in trees['native_E149_owned_process.py'].body if isinstance(node,ast.ClassDef) and node.name=='OwnedJob')
spawn=ast.unparse(next(node for node in job.body if isinstance(node,ast.FunctionDef) and node.name=='spawn_suspended'))
ordered=['_winapi.CreateProcess(','.AssignProcessToJobObject(',"row['job_assigned_before_resume'] = True",'on_created(held)', '.ResumeThread(']
positions=[spawn.index(marker) for marker in ordered];assert positions==sorted(positions)
assert 'CREATE_SUSPENDED' in process and '0x00002000' in process and '0x00000800' not in process
assert "{'handle_list': handles}" in spawn and 'CREATE_NO_WINDOW' in spawn
for marker in ['QueryInformationJobObject','IsProcessInJob','TerminateJobObject','creation_filetime_ticks','GetExitCodeProcess']:
 assert marker in process
for marker in ['300,lambda row:row[\'status\']==\'serving_static_only\'','case.checkpoint(True)','case.wait(controller,lease)',
 'case.wait(fixture,30)','case.stop_database(producer)','job_accounting_final','runtime_producer_source_reviewed','pending_process_observations']:
 assert marker in producer,marker
for marker in ['owned_job_kill_on_close','job_assigned_before_resume','job_accounting_final']:
 assert marker in reader
assert 'subprocess.Popen(' not in producer and 'taskkill' not in producer+process and 'rmtree' not in producer+process
assert 'mysqladmin' not in producer.split('def execute(',1)[1] # SQL/admin execution remains in pinned DB producer
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
assert sha(w['current_formal_reference_identity']['formal'])==FORMAL
old_reader=w['current_E149_recovery_case_reader_development'];oldq=read(old_reader['qualification'])
assert sha(old_reader['qualification'])==old_reader['qualification_sha256']
images=w['current_E149_recovery_linked_image_qualifications'];assert set(images)=={'Debug','Release'}
for profile,row in images.items():
 assert sha(row['qualification'])==row['qualification_sha256'];q=read(row['qualification'])
 assert q['compiler_control_versions_archived'] and not q['outputs_executed'] and q['compiler_exit_codes']==[0,0]
 check_pins(read(q['compile_cohort'])['input_pins']);assert sha(q['product'])==q['product_sha256']
inputs=read(Path(w['current_development_typecheck_qualification']['run'])/'base-compiled-inputs.json') if 'run' in w['current_development_typecheck_qualification'] else read(Path(read(w['current_development_typecheck_qualification']['qualification'])['run'])/'base-compiled-inputs.json')
assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
normal=read(w['current_development_typecheck_qualification']['qualification'])['normal_product_pins'];check_pins(normal)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=V/'native-E149-recovery-producer-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
source_pins={str(T/name):sha(T/name) for name in names};source_pins[str(T/'native-E149-recovery-case-contract.md')]=sha(T/'native-E149-recovery-case-contract.md')
record=dict(oldq,status='owned_case_producer_job_handles_and_reader_source_AST_only',source_pins=source_pins,
 predecessor_reader_development=old_reader['qualification'],predecessor_reader_development_sha256=old_reader['qualification_sha256'],
 source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,AST_compiled_modules=3,producer_AST_compiled_modules=2,total_AST_compiled_modules=5,
 builder_source_AST_compiled=True,builder_current_source=str(T/'build-native-E149-recovery-image.py'),builder_current_source_sha256=sha(T/'build-native-E149-recovery-image.py'),
 runtime_producer_source_complete=True,runtime_producer_implementation_pending=False,runtime_producer_runtime_qualified=False,
 passive_reader_source_complete=True,reader_runtime_qualified=False,reader_executed=False,reader_module_imported=False,
 linked_image_builder_pending=False,isolated_linked_image_qualified=True,both_profiles_linked=True,
 linked_profiles={profile:dict(qualification=row['qualification'],sha256=row['qualification_sha256']) for profile,row in images.items()},
 static_source_findings=['execute checks current defer before plan/job/child',
  'direct CreateProcess preserves primary-thread handle; suspended child is assigned to private noninheritable unnamed Job and journalled before resume',
  'no breakaway; actual DB datadir/image/command plus Job membership required when adopting daemon handles',
  'DB import precedes native source freeze, then exact pinned Java/JDBC factory/fixture and one exact controller case',
  'original handles, bounded leases, current-user-defer polling, stop request and Job active-process-zero barrier; no retry',
  'failure metadata and all raw outputs retained; reader remains a separate explicit readback step'],
 process_job_objects_created=0,workload_processes_started=0,handle_operations_executed=0,SQL_executed=False,database_started=False,
 original_factories_invoked=False,tests_executed=0,desktop_inputs=0,outputs_executed=False,
 full_Main_process_qualified=False,full_native_owner_qualified=False,whole_goal_complete=False,
 local_CPython_source=str(Path(sys.executable).parent/'Lib/subprocess.py'),
 references=['https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject',
  'https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information',
  'https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information'],
 limitations=['Static AST/source review only; no Job constructor, CreateProcess, handle observation, DB/Java/controller process or negative execution-gate test was invoked',
  'Windows Job nesting/assignment/inheritance/resume, actual PID/JDBC/SQL readiness, cancellation/failure cleanup and all result-reader assertions still require real authorized cases',
  'Preparation source-ready flags do not authorize tests; current user defer remains true',
  'Existing seventeen item observations omit additional native elemental/card/magic-stone internals; complete owner/Main/game/UI qualification remains pending'],
 recorded_utc=datetime.now(timezone.utc).isoformat(),test_execution_deferred_by_user=True)
record['local_CPython_source_sha256']=sha(record['local_CPython_source'])
# Drop inherited status fields which described only the superseded prepared plan.
for key in ['plan','plan_sha256','observed_original_preparation_tool_chunk','observed_preparation_actual_exit_code','superseded_preparation_plan','superseded_preparation_plan_sha256','superseded_reason']:
 record.pop(key,None)
(out/'qualified-development.json').write_bytes(encoded(record))
review=dict(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'))
plan=make_plan('Debug','cancel-repeat',producer_review=review);prepared=read(plan)
assert prepared['linked_image_qualified'] and not prepared['runtime_producer_implementation_pending'] and prepared['test_execution_deferred_at_preparation']
record.update(qualification=review['qualification'],qualification_sha256=review['qualification_sha256'],prepared_case_plan=str(plan),prepared_case_plan_sha256=sha(plan),
 preparation_mode='in-process pure plan function, no runtime Job or process helper invocation',previous_goal_turn_evidence=w['last_goal_turn_evidence'])
raw={Path(name).name:Path(name) for name in source_pins}
raw.update({'qualified-development.json':out/'qualified-development.json','reader.py':Path(__file__),'prepared-plan.json':plan,
 'builder-current.py':T/'build-native-E149-recovery-image.py','report-updater-before.py':T/'update-master-report-text-index-order.py'})
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name,path in raw.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in raw.items())
record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'))
(out/'evidence.json').write_bytes(encoded(record))
work='E149自動案例producer與Windows Job／原handle回收已完成來源初稿及5個Python AST檢查，未建立Job／程序或測試負向gate。先暫停建立、核對PID／FILETIME／映像、加入私有Job及保存紀錄才放行；原DB→factory→精確controller及期限／退出零程序屏障均有原始碼，尚未實跑。新Debug計畫已固定producer／reader版本與既有連結映像；source-ready不是測試授權，使用者暫緩仍true。正常1049輸入／兩EXE及14檔Runtime／QA候選均不變，未套用。接續補全NativeDe元素／卡片／魔石的唯讀觀測及完整物品／owner核對；使用者恢復測試後才驗Job與SQL／JDBC、原版6反例／候選14案、33／31秒、最終library／Release／完整GUI，整體未完成。'
staged={}
for name in before:
 path=R/name
 if path.suffix=='.json':
  j=read(path);j['current_E149_recovery_execution_development']=record;j['current_E149_recovery_case_reader_development']=record
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_3d_E149_owned_job_producer_source_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
  if name.endswith('next-validation-action.json'):j['next_action']=work
  staged[name]=encoded(j)
 else:
  _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name in before:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items());check_pins(normal)
for name,blob in staged.items():(R/name).write_bytes(blob)
assert all((R/name).read_bytes()==blob for name,blob in staged.items())
(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),prepared_plan=str(plan),Python_AST_modules=5,Job_objects_created=0,workload_processes_started=0,tests_executed=0,test_execution_deferred=True,eight_files_read_back=True),ensure_ascii=False))
