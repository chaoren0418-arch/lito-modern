"""Future owned G06 case producer. Execute only after user resumes tests.

No retry, UI, deployment, clock edits or player SQL. Results remain unreviewed
until the separate reader examines actual wire, snapshots and process exits.
"""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,contextlib,hashlib,importlib.util,io,json,os,secrets,time,traceback,zipfile
from native_E149_recovery_contract import R,P,FORMAL,SCENARIOS,owned,read,encoded,validate_plan,require,check_pins,client_image_binding
from native_E149_owned_process import HeldProcess,OwnedJob,identity,kernel,sha

def atomic(path,value):
 path=owned(path);temp=path.with_name(path.name+'.writing')
 with temp.open('xb') as file:file.write(encoded(value))
 os.replace(temp,path)

def authorize(run=None,pipeline=None):
 w=read(R/'工作佇列.json')
 require(not w['current_user_requested_goal'].get('test_execution_deferred_by_user',True),'user deferred tests; execution is disabled')
 active=w['active_workloads']
 if active:
  require(run is not None and set(active)=={'native_E149_recovery_case'},'another workload is active')
  row=active['native_E149_recovery_case']
  require(row['run']==str(run) and row['pipeline']['pid']==pipeline['pid']
   and row['pipeline']['creation_filetime_ticks']==pipeline['creation_filetime_ticks'],'active case belongs to another original pipeline')
 else:require(not w['source_and_corresponding_QA_frozen'],'source/QA freeze belongs to another operation')
 reference=w['current_formal_reference_identity'];require(reference['formal_sha256']==FORMAL,'formal reference declaration changed')
 if pipeline is None:require(sha(reference['formal'])==FORMAL,'formal target changed')
 else:
  current=Path(reference['formal']).stat()
  require(pipeline['formal_file_metadata']==dict(bytes=current.st_size,mtime_ns=current.st_mtime_ns),'formal target metadata changed during case')
 return w

def load_database_tool(path):
 specification=importlib.util.spec_from_file_location('_lito_owned_database',path)
 module=importlib.util.module_from_spec(specification);specification.loader.exec_module(module);return module

class Case:
 def __init__(self,p,plan_path,out):
  self.p,self.plan_path,self.out=p,plan_path,out
  self.audit=R/'驗證紀錄/current/native-E149-recovery-case-execution'/out.name;self.audit.mkdir(parents=True)
  self.pipeline=identity(kernel().GetCurrentProcess(),os.getpid())
  metadata=Path(read(R/'工作佇列.json')['current_formal_reference_identity']['formal']).stat()
  self.pipeline['formal_file_metadata']=dict(bytes=metadata.st_size,mtime_ns=metadata.st_mtime_ns)
  self.job=None;self.held={};self.records={};self.pending={}
  self.runtime=self.db_plan=self.db=None;self.registered=False
  self.e=dict(schema='E149-recovery-execution-v4',status='preparing_owned_case',plan=str(plan_path),plan_sha256=sha(plan_path),
   scenario=p['scenario'],profile=p['profile'],pipeline=self.pipeline,processes=[],runtime_producer_source_reviewed=True,
   runtime_producer_pins=p['runtime_producer_pins'],linked_image_qualification=p['linked_image_qualification'],
   linked_image_qualification_sha256=p['linked_image_qualification_sha256'],inputs_unchanged=False,forced_cleanup_used=False,
   fixture_source_binding=p['fixture_source_binding'],fixture_source_binding_sha256=p['fixture_source_binding_sha256'],
   fixture_qualification=p['fixture_qualification'],fixture_qualification_sha256=p['fixture_qualification_sha256'],
   Java_source_pins=p['Java_source_pins'],Java_classes=p['Java_classes'],client_source_binding=p['client_source_binding'],
   owner_identity_contract=p['owner_identity_contract'],owner_identity_contract_sha256=p['owner_identity_contract_sha256'],
   owner_guard_probe_kind=p['owner_guard_probe_kind'],
   retired_item_mutation_probe_kind=p['retired_item_mutation_probe_kind'],retired_item_mutation_methods=p['retired_item_mutation_methods'],
   retired_item_mutation_targets=p['retired_item_mutation_targets'],retired_item_mutation_attempts=p['retired_item_mutation_attempts'],
   retired_item_morph_probe_kind=p['retired_item_morph_probe_kind'],retired_item_morph_methods=p['retired_item_morph_methods'],
   retired_item_morph_targets=p['retired_item_morph_targets'],retired_item_morph_attempts=p['retired_item_morph_attempts'],retired_item_aggregate_attempts=p['retired_item_aggregate_attempts'],
   database_original_handle_signalled=False,desktop_inputs=0,selected_test_processes_started=0,
   full_Main_process_qualified=False,full_native_owner_qualified=False,whole_goal_complete=False,started_utc=datetime.now(timezone.utc).isoformat())
  self.last_notice=0;self.save()

 def save(self):
  self.e['processes']=list(self.records.values());atomic(self.out/'execution.json',self.e)

 def checkpoint(self,active):
  w=read(R/'工作佇列.json');previous=Path(w['last_goal_turn_evidence']).with_name('readback.json')
  before=read(previous)['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'progress files changed before checkpoint')
  if active:require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],'workload appeared during private DB preparation')
  else:
   require((self.registered and set(w['active_workloads'])=={'native_E149_recovery_case'} and w['active_workloads']['native_E149_recovery_case']['run']==str(self.out))
    or (not self.registered and not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']),'cannot release another workload')
  folder=self.audit/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));folder.mkdir()
  record=dict(run=str(self.out),execution=str(self.out/'execution.json'),pipeline=self.pipeline,
   status='owned_DB_ready_starting_native_fixture' if active else self.e['status'],
   original_handles_and_job=self.e.get('job_accounting'),source_and_corresponding_QA_frozen=active,
   queue_authorization_observed_before_start=True,whole_goal_complete=False)
  (folder/'evidence.json').write_bytes(encoded(record))
  work=('E149隔離案例已完成真實靜態DB匯入，沿原pipeline接續原factory／精確controller案例；來源及對應QA凍結，僅跟隨原身分，不自動重啟。'
   if active else 'E149原案例程序已退出，保留原始execution／日誌／proof／wire供獨立reader核對；'+self.e['status']+'，未授予完整Main／owner／GUI或整體資格。')
  staged={}
  for name in before:
   path=R/name
   if path.suffix=='.json':
    j=read(path)
    if active:j['active_workloads']={'native_E149_recovery_case':record}
    else:j['active_workloads'].pop('native_E149_recovery_case',None)
    j['source_and_corresponding_QA_frozen']=active;j['current_E149_recovery_case_execution']=record
    j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(folder/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
    if name.endswith('next-validation-action.json'):j['next_action']=work
    staged[name]=encoded(j)
   else:
    _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs')
    staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
  with zipfile.ZipFile(folder/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
   for name in before:z.write(R/name,name)
  with zipfile.ZipFile(folder/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'checkpoint archive differs')
  require(all(sha(R/n)==h for n,h in before.items()),'concurrent progress update')
  for name,blob in staged.items():(R/name).write_bytes(blob)
  require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs')
  (folder/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},
   before_archive_sha256=sha(folder/'before.zip'),evidence_sha256=sha(folder/'evidence.json'))))
  self.registered=active

 def spawn(self,kind,command,environment,expected_sha,extra=None):
  authorize(self.out,self.pipeline)
  def created(process):
   key=(process.row['pid'],process.row['creation_filetime_ticks']);process.row.update(extra or {})
   self.held[key]=process;self.records[key]=process.row;self.save()
  return self.job.spawn_suspended(command,environment,self.runtime,self.out/(kind+'.stdout.log'),
   self.out/(kind+'.stderr.log'),expected_sha,kind,created)

 def collect_database(self):
  path=self.runtime/'process-history.json'
  if not path.exists():return
  bundle=read(self.p['database_bundle']);base=Path(bundle['bundle_root'])
  allowed={str((base/'bin'/name).resolve()):bundle['files']['bin/'+name] for name in ['mysqld.exe','mysql.exe','mysqladmin.exe']}
  for source in read(path):
   record=source['identity'];command=source['command'];exe=Path(command[0]).resolve()
   require(str(exe) in allowed and record['executable_sha256']==allowed[str(exe)],'DB producer recorded an unreviewed executable')
   kind='database_bootstrap' if source['name']=='bootstrap' else 'database_daemon' if source['name']=='daemon' else 'mysql_client'
   if kind in ['database_bootstrap','database_daemon']:
    require(command[1]=='--no-defaults','DB producer inherited configuration')
    dirs=[v.split('=',1)[1] for v in command if v.startswith('--datadir=')]
    require(len(dirs)==1 and owned(dirs[0],self.runtime)==self.runtime/'data','DB record escaped this case')
   else:require(command[1]=='--defaults-file='+(self.runtime/'root-client.ini').as_posix(),'client escaped this owned DB configuration')
   key=(record['pid'],record['creation_filetime_ticks'])
   row=self.records.setdefault(key,dict(record,kind=kind,command=command,wait_confirmed=False,exit_code=None,DB_history_name=source['name']))
   if key in self.held:
    held=self.held[key]
    if held.terminal():held.collect_exit();row.update(held.row)
   elif source.get('exit_code') is None:
    adopted=None
    try:
     held=HeldProcess.adopt(record,command,{Path(n):h for n,h in allowed.items()});adopted=held;self.job.require_member(held)
     held.row.update(kind=kind,DB_history_name=source['name']);self.held[key]=held;self.records[key]=held.row;self.pending.pop(key,None)
    except Exception as error:
     if adopted is not None:adopted.close()
     # A short-lived client may finish while being observed. Keep the pending
     # observation until the original producer reports its actual Popen wait.
     self.pending[key]=type(error).__name__+': '+str(error)
   if source.get('exit_code') is not None:
    require(source.get('finished_utc'),'DB producer did not record completion')
    if key in self.held:
     require(self.held[key].terminal(),'DB receipt contradicts original live handle');self.held[key].collect_exit()
    row=self.records[key];row.update(wait_confirmed=True,exit_code=source['exit_code'],finished_utc=source['finished_utc'],exit_observation='original_DB_producer_Popen_wait')
    self.pending.pop(key,None)
  self.save()

 def tick(self,authorized=True):
  if authorized:authorize(self.out,self.pipeline)
  self.collect_database()
  for key,process in self.held.items():
   if process.handle and process.terminal():self.records[key].update(process.collect_exit())
  self.e['job_accounting']=self.job.accounting();self.save()
  if time.monotonic()-self.last_notice>=20:
   print(json.dumps(dict(run=str(self.out),status=self.e['status'],owned_job_active=self.e['job_accounting']['active_processes'])),flush=True);self.last_notice=time.monotonic()

 def wait(self,process,seconds,authorized=True):
  end=time.monotonic()+seconds
  while not process.terminal():
   self.tick(authorized);require(time.monotonic()<end,'owned phase deadline exceeded; same original process retained');time.sleep(.1)
  row=process.collect_exit();self.records[(row['pid'],row['creation_filetime_ticks'])].update(row);self.save()
  require(row['exit_code']==0,'owned process failed: '+row['kind']);return row

 def wait_json(self,path,process,seconds,condition):
  end=time.monotonic()+seconds
  while True:
   self.tick();require(not process.terminal(),'owned producer ended before readiness')
   if path.exists():
    try:row=read(path)
    except json.JSONDecodeError:row=None # same original writer may still be closing ready.json
    if row is not None and condition(row):return row
   require(time.monotonic()<end,'owned readiness deadline exceeded; no restart');time.sleep(.1)

 def stop_database(self,producer):
  self.collect_database();owner_path=self.runtime/'owned-process.json'
  if not producer.terminal() and owner_path.exists():
   owner=read(owner_path);key=(owner['pid'],owner['identity']['creation_filetime_ticks'])
   require(key in self.held,'original daemon handle not held; do not infer PID ownership')
   self.job.require_member(self.held[key])
   request=dict(nonce=read(self.db_plan)['nonce'],process=owner['identity'])
   destination=self.runtime/'stop-request.json'
   if destination.exists():require(read(destination)==request,'different DB stop request')
   else:atomic(destination,request)
  self.wait(producer,60,authorized=False);self.collect_database()
  daemon=[process for process in self.held.values() if process.row.get('kind')=='database_daemon']
  require(len(daemon)==1 and daemon[0].terminal(),'original daemon is not terminal')
  daemon[0].collect_exit();self.e['database_original_handle_signalled']=True

def execute(plan_path):
 authorize() # before plan creation, job allocation, socket or child execution
 plan_path=owned(plan_path.resolve(strict=True));p=validate_plan(plan_path)
 current=read(R/'工作佇列.json').get('current_E149_fixture_binding',{})
 require(current.get('qualification')==p['fixture_source_binding'] and current.get('qualification_sha256')==p['fixture_source_binding_sha256'],'plan is not the current prepared fixture binding')
 require(not p['runtime_producer_implementation_pending'] and p['linked_image_qualified'],'reviewed producer and linked image required')
 check_pins(p['runtime_producer_pins']);review=read(p['runtime_producer_review'])
 require(sha(p['runtime_producer_review'])==p['runtime_producer_review_sha256'] and review['runtime_producer_source_complete'],'producer source review missing')
 image=read(p['linked_image_qualification']);require(sha(p['linked_image_qualification'])==p['linked_image_qualification_sha256'],'linked image qualification changed')
 require(client_image_binding(image,p['effective_overlay_files'],p['effective_overlay_paths'])==p['client_source_binding'] and image['profile']==p['profile'],'linked Rust source/profile differs')
 build=read(image['compile_cohort']);check_pins(build['input_pins']);require(sha(image['product'])==image['product_sha256'],'linked PE changed')
 out=P/'target/current/verification/native-E149-recovery-cases'/('case-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+secrets.token_hex(6))
 out.mkdir(parents=True);(out/'tmp').mkdir();case=Case(p,plan_path,out);producer=fixture=controller=None;problem=None;cleanup=[]
 try:
  case.db=load_database_tool(R/'維護工具/native-E149-owned-mysql.py')
  capture=io.StringIO()
  with contextlib.redirect_stdout(capture):case.db.plan(Path(p['database_bundle']),Path(p['static_manifest']),900)
  case.db_plan=Path(json.loads(capture.getvalue())['plan']);case.runtime=Path(read(case.db_plan)['case_root'])
  case.e.update(runtime=str(case.runtime),database_plan=str(case.db_plan),database_plan_sha256=sha(case.db_plan))
  bundle=read(p['database_bundle']);check_pins({str(Path(bundle['bundle_root'])/n):h for n,h in bundle['files'].items()})
  env={k:v for k,v in os.environ.items() if not k.startswith('LITO_') and k not in ['JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS']}
  env.update(TEMP=str(out/'tmp'),TMP=str(out/'tmp'))
  case.job=OwnedJob();case.e['owned_job_kill_on_close']=True
  producer=case.spawn('database_producer',[p['python'],'-B','-X','utf8',str(R/'維護工具/native-E149-owned-mysql.py'),'serve','--plan',str(case.db_plan),'--execute'],env,p['python_sha256'])
  case.e['status']='waiting_actual_static_import';case.save()
  state=case.wait_json(case.runtime/'state.json',producer,300,lambda row:row['status']=='serving_static_only')
  observe_end=time.monotonic()+12
  while not any(h.row.get('kind')=='database_daemon' for h in case.held.values()):
   case.tick();require(not producer.terminal() and time.monotonic()<observe_end,'same original daemon could not be observed before deadline');time.sleep(.1)
  validate_plan(plan_path);case.checkpoint(True)
  owner=case.runtime/'database-owner.properties';imported=case.runtime/'static-import.properties'
  command=[p['java'],'-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+str(case.runtime/'tmp'),'-XX:-UsePerfData',
   '-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+p['catalog'],
   '-Xverify:all','-classpath',p['fixture_classes']+';'+p['formal_reference']+';'+p['JDBC'],
   'local.lito381.server.NativeTeleportFailureHarness',str(case.runtime),p['scenario'],str(owner),sha(owner),str(imported),sha(imported),p['SQL'],'--execute-owned-tests']
  fixture=case.spawn('fixture',command,env,p['java_sha256'],dict(inherited_java_options_removed=True));case.e['status']='waiting_original_resource_factories';case.save()
  ready=case.wait_json(case.runtime/'ready.json',fixture,180,lambda row:row['observer_kind']=='G06_E149_resources_direct_fields_v2')
  require(ready['original_resource_factories_initialized'] and ready['player_sql_access'] is False,'resource/SQL readiness differs')
  require(ready['shared_client_resources_initialized'] is True and ready['native_item_attribute_observer_kind']==p['native_item_attribute_observer_kind'],'original shared initializer/extended observer readiness absent')
  require(ready['owner_guard_probe_kind']==p['owner_guard_probe_kind'],'original owner guard fixture required')
  require(ready['retired_item_mutation_probe_kind']==p['retired_item_mutation_probe_kind']
   and ready['retired_item_mutation_methods']==len(p['retired_item_mutation_methods'])
   and ready['retired_item_mutation_targets']==p['retired_item_mutation_targets'],'retired setter probe fixture required')
  require(ready['retired_item_morph_probe_kind']==p['retired_item_morph_probe_kind']
   and ready['retired_item_morph_methods']==len(p['retired_item_morph_methods'])
   and ready['retired_item_morph_targets']==p['retired_item_morph_targets'],'retired morph fixture required')
  require(ready['owner_proof_sha256']==sha(owner) and ready['import_proof_sha256']==sha(imported),'factory proof changed')
  overrides={'LITO_TELEPORT_SCENARIO':p['scenario'],'LITO_TELEPORT_ROOT':str(case.runtime),'LITO_TELEPORT_REFERENCE_SHA':FORMAL.lower()}
  controller=case.spawn('controller',[image['product'],p['test_name'],'--exact','--ignored','--nocapture'],env|overrides,image['product_sha256'],dict(environment=overrides))
  case.e.update(status='executing_single_owned_recovery_case',controller_product_sha256=image['product_sha256'],selected_test_processes_started=1,
   controller_stdout=str(out/'controller.stdout.log'));case.save()
  lease=500 if p['scenario']=='cancel-expiry-source' else 380 if p['scenario']=='cancel-expiry' else 260
  case.wait(controller,lease);case.wait(fixture,30);case.e['status']='stopping_private_database';case.save();case.stop_database(producer)
 except BaseException as error:
  problem=error;case.e.update(status='failed_preserved',error=repr(error));(out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8')
 finally:
  if case.job is not None:
   if problem is not None:
    for process in [controller,fixture]:
     if process is not None and not process.terminal():
      try:process.terminate_original('original case failed');case.e['forced_cleanup_used']=True
      except BaseException as error:cleanup.append(repr(error))
    if producer is not None:
     try:case.stop_database(producer)
     except BaseException as error:cleanup.append(repr(error))
   try:
    if case.job.accounting()['active_processes']:
     case.job.terminate_members();case.e['forced_cleanup_used']=True
    end=time.monotonic()+15
    while case.job.accounting()['active_processes'] and time.monotonic()<end:time.sleep(.1)
    case.e['job_accounting_final']=case.job.accounting();require(case.e['job_accounting_final']['active_processes']==0,'owned job has remaining members')
    case.collect_database()
    for process in case.held.values():
     if process.handle:
      require(process.terminal(),'owned original handle still live');case.records[(process.row['pid'],process.row['creation_filetime_ticks'])].update(process.collect_exit())
     else:require(process.row['wait_confirmed'],'closed original handle has no completed wait')
   except BaseException as error:cleanup.append(repr(error))
   finally:
    for process in case.held.values():
     try:process.close()
     except BaseException as error:cleanup.append(repr(error))
    try:case.job.close()
    except BaseException as error:cleanup.append(repr(error))
  try:
   validate_plan(plan_path);check_pins(p['runtime_producer_pins'])
   require(sha(read(R/'工作佇列.json')['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed before terminal guard')
   case.e['inputs_unchanged']=True
  except BaseException as error:cleanup.append('input guard: '+repr(error))
  if problem is None and (case.pending or any(not row['wait_confirmed'] or row['exit_code']!=0 for row in case.records.values())):
   cleanup.append('original process identities/waits did not all finish successfully')
  case.e.update(cleanup_errors=cleanup,finished_utc=datetime.now(timezone.utc).isoformat())
  case.e['forced_cleanup_used']|=any(row.get('forced_cleanup',False) for row in case.records.values())
  case.e['pending_process_observations']=[dict(pid=k[0],creation_filetime_ticks=k[1],error=v) for k,v in case.pending.items()]
  if problem is None and not cleanup and not case.e['forced_cleanup_used']:
   case.e['status']='owned_processes_exited_results_unreviewed'
   evidence=[case.runtime/n for n in ['ready.json','result.json','state.json','database-owner.properties','static-import.properties','process-history.json',
    'server-trace.jsonl','staged-trace.jsonl','owner-guard-trace.jsonl','item-mutation-denial-trace.jsonl','item-morph-denial-trace.jsonl','a-failure.json','b-failure.json']]
   evidence += [case.runtime/(role+suffix) for role in ['a','b'] for suffix in ['-continuation-observe-request','-continuation-observed.json']]
   evidence += list((case.runtime/'captures').glob('*/frames.jsonl'))+[out/(kind+suffix) for kind in ['database_producer','fixture','controller'] for suffix in ['.stdout.log','.stderr.log']]
   evidence += [Path(p[n]) for n in ['linked_image_qualification','fixture_source_binding','fixture_qualification']]+[Path(image['compile_cohort'])]
   try:case.e['evidence_pins']={str(path):sha(owned(path)) for path in evidence}
   except BaseException as error:
    cleanup.append('evidence collection: '+repr(error));problem=error;case.e['status']='evidence_incomplete_preserved'
    (out/'evidence-collection-failure.txt').write_text(traceback.format_exc(),encoding='utf-8')
  elif problem is None:problem=RuntimeError('cleanup/input failure; original outputs retained');case.e['status']='cleanup_failure_preserved'
  case.save()
  if case.job is None or case.e.get('job_accounting_final',{}).get('active_processes')==0:
   try:case.checkpoint(False)
   except BaseException as error:cleanup.append('progress release: '+repr(error));case.save();problem=problem or error
 print(json.dumps(dict(run=str(out),status=case.e['status'],owned_job_active=case.e.get('job_accounting_final',{}).get('active_processes'),independent_reader_required=True),ensure_ascii=False),flush=True)
 if problem is not None:
  problem.owned_case_output=str(out);raise problem
 return out/'execution.json'

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--plan',type=Path,required=True)
 ap.add_argument('--execute-owned-tests',action='store_true',required=True);a=ap.parse_args();execute(a.plan)
if __name__=='__main__':main()
