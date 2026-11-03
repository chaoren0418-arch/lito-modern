"""Future exact component producer. First action checks current test consent.

No GUI, SQL, fixture server, automatic retry or ignored-test execution. Results
stay unreviewed until the separate complete matrix reader consumes them.
"""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,importlib.util,json,os,time,traceback,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_owned_process import OwnedJob,identity,kernel
from native_E149_component_diagnostics import DiagnosticStore
from native_E149_libtest_readback import inventory,selected_result
T=R/'維護工具';V=R/'驗證紀錄/current';KEY='E149_component_matrix'

def authorize(run=None,pipeline=None,cleanup=False):
 w=read(R/'工作佇列.json')
 if not cleanup:require(not w['current_user_requested_goal'].get('test_execution_deferred_by_user',True),'user deferred tests; component execution disabled')
 if run is None:
  require(not w['active_workloads'] and not w.get('active_workload_sessions') and not w['source_and_corresponding_QA_frozen'],'another original workload/session/freeze active')
  require(not w.get('current_E149_component_matrix_execution',{}).get('diagnostic_recovery_pending',False),'recover the previous preserved diagnostic transaction before another matrix')
  require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 else:
  require(set(w['active_workloads'])=={KEY} and w['source_and_corresponding_QA_frozen'],'this original component workload is not registered')
  row=w['active_workloads'][KEY]
  require(row['run']==str(run) and row['pipeline']['pid']==pipeline['pid']
   and row['pipeline']['creation_filetime_ticks']==pipeline['creation_filetime_ticks'],'component workload belongs to another original pipeline')
 return w

def atomic(path,value):
 path=owned(path);temporary=path.with_name(path.name+'.writing')
 with temporary.open('xb') as file:file.write(encoded(value))
 os.replace(temporary,path)

def load_reader():
 specification=importlib.util.spec_from_file_location('_lito_component_reader',T/'verify-native-E149-component-matrix.py')
 module=importlib.util.module_from_spec(specification);specification.loader.exec_module(module);return module

class Matrix:
 def __init__(self,iqpath,iq,p,review,review_ref):
  self.iq,self.p,self.review=iq,p,review;self.job=None;self.held=[];self.data=None;self.registered=False
  self.out=P/'target/current/verification/native-E149-component-matrix-execution'/('case-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));self.out.mkdir(parents=True)
  self.audit=V/'native-E149-component-matrix-execution'/self.out.name;self.audit.mkdir(parents=True)
  self.pipeline=identity(kernel().GetCurrentProcess(),os.getpid())
  self.e=dict(schema='E149-component-matrix-execution-v1',status='preparing_owned_component_matrix',
   images_qualification=str(iqpath),images_qualification_sha256=sha(iqpath),plan=iq['plan'],plan_sha256=iq['plan_sha256'],
   tool_source_review=review_ref['qualification'],tool_source_review_sha256=review_ref['qualification_sha256'],tool_source_pins=review['source_pins'],
   source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,pipeline=self.pipeline,inventories=[],cases=[],
   source_inputs_unchanged=False,normal_products_unchanged=False,forced_cleanup_used=False,diagnostic_roots_restored=False,
   diagnostics_before={},automatic_retries=0,desktop_inputs=0,full_library_qualified=False,full_G06_qualified=False,
   whole_goal_complete=False,started_utc=datetime.now(timezone.utc).isoformat())
  self.last_notice=0;self.save()

 def save(self):atomic(self.out/'execution.json',self.e)

 def checkpoint(self,active):
  w=read(R/'工作佇列.json');before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
  require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'progress checkpoint changed')
  if active:require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],'another operation appeared')
  else:authorize(self.out,self.pipeline,cleanup=True)
  folder=self.audit/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));folder.mkdir()
  record=dict(run=str(self.out),execution=str(self.out/'execution.json'),pipeline=self.pipeline,status=self.e['status'],
   diagnostic_recovery_pending=not active and self.data is not None and not self.e['diagnostic_roots_restored'],
   source_and_corresponding_QA_frozen=active,whole_goal_complete=False)
  (folder/'evidence.json').write_bytes(encoded(record));staged={}
  work=('元件原pipeline開始精確inventory／案例，來源與對應QA凍結，僅跟隨原handle／Job，不重啟。' if active
   else '元件原工作程序與Job均已終止，保留原execution／日誌／診斷archive供獨立reader；'+self.e['status']+'；完整G06／library／GUI未授資格。')
  for name in before:
   path=R/name
   if path.suffix=='.json':
    j=read(path)
    if active:j['active_workloads']={KEY:record}
    else:j['active_workloads'].pop(KEY,None)
    j['source_and_corresponding_QA_frozen']=active;j['current_E149_component_matrix_execution']=record
    j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(folder/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
    if name.endswith('next-validation-action.json'):j['next_action']=work
    staged[name]=encoded(j)
   else:
    _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs')
    staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
  with zipfile.ZipFile(folder/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
   for name in before:z.write(R/name,name)
  with zipfile.ZipFile(folder/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'progress before archive differs')
  require(all(sha(R/n)==h for n,h in before.items()),'concurrent progress writer')
  for name,blob in staged.items():(R/name).write_bytes(blob)
  require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'progress readback differs')
  (folder/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(folder/'before.zip'),evidence_sha256=sha(folder/'evidence.json'))))
  self.registered=active

 def cleanup_authorized(self):authorize(self.out,self.pipeline,cleanup=True)

 def inputs(self):
  check_pins(self.review['source_pins']);build=read(self.iq['compile_cohort']);check_pins(build['input_pins']);check_pins(build['normal_product_pins'])
  require(sha(self.iq['compile_cohort'])==self.iq['compile_cohort_sha256'] and sha(self.iq['plan'])==self.iq['plan_sha256'],'linked build/plan changed')
  inputs=read(self.p['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal compiled inputs changed')
  require(sha(read(R/'工作佇列.json')['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')

 def invoke(self,profile,variant,case=None):
  authorize(self.out,self.pipeline);check_pins(self.review['source_pins'])
  image=self.iq['images'][profile+'-'+variant];folder=self.out/('process-'+str(len(self.held)).zfill(3)+'-'+profile+'-'+variant);folder.mkdir();(folder/'tmp').mkdir()
  command=[image['product'],'--list'] if case is None else [image['product'],case['test_name'],'--exact','--nocapture','--test-threads=1']
  subset={'RUST_BACKTRACE':'0','RUST_LIB_BACKTRACE':'0','RUST_TEST_THREADS':'1'}
  environment={k:v for k,v in os.environ.items() if not k.startswith(('LITO_','RUST_'))}
  environment.update(subset,TEMP=str(folder/'tmp'),TMP=str(folder/'tmp'))
  row=dict(profile=profile,variant=variant,environment=subset,inherited_test_options_removed=True,
   stdout=str(folder/'stdout.log'),stderr=str(folder/'stderr.log'),diagnostic_restore_verified=False)
  if case is not None:row['case']=case
  self.e['inventories' if case is None else 'cases'].append(row);self.data.begin(profile,folder);self.save()
  def created(process):
   self.held.append(process);row['process']=process.row;self.e['current_process']=process.row;self.save()
  child=self.job.spawn_suspended(command,environment,folder,Path(row['stdout']),Path(row['stderr']),image['sha256'],
   'inventory' if case is None else 'component_case',created)
  end=time.monotonic()+(30 if case is None else 120)
  while not child.terminal():
   authorize(self.out,self.pipeline);require(time.monotonic()<end,'same original component process exceeded tooling lease; no restart')
   if time.monotonic()-self.last_notice>=20:
    print(json.dumps(dict(run=str(self.out),cases_started=len(self.e['cases']),current_pid=child.row['pid'])),flush=True);self.last_notice=time.monotonic()
   time.sleep(.1)
  row['process']=child.collect_exit();require(self.job.accounting()['active_processes']==0,'unexpected live descendant; preserve before diagnostic restoration')
  require(self.job.accounting()['total_processes']==len(self.held),'unrecorded descendant process detected')
  row.update(self.data.preserve_restore());self.e['current_process']=None;self.save()
  return row

def execute(images_path):
 authorize() # before linked-image lookup, runtime reader, Job or diagnostics
 images_path=owned(images_path.resolve(strict=True));iq=read(images_path)
 require(iq['schema']=='E149-component-images-qualified-v1' and iq['status']=='qualified_linked_selected_component_matrix_images','independent four-image qualification required')
 require(iq['source_manifest_sha256']==SOURCE and iq['formal_target_sha256']==FORMAL and iq['original_compiler_exit_codes']==[0,0,0,0]
  and iq['original_compilers_terminal'] is True and iq['compiler_controls_archived'] is True,'linked image identity differs')
 p=read(iq['plan']);require(sha(iq['plan'])==iq['plan_sha256'],'linked plan changed')
 require(set(iq['images'])=={profile+'-'+variant for profile in ['Debug','Release'] for variant in ['baseline','candidate']},'four linked images missing')
 w=authorize();review_ref=w['current_E149_component_execution_development'];review=read(review_ref['qualification'])
 require(sha(review_ref['qualification'])==review_ref['qualification_sha256'] and review['component_producer_source_complete'] is True,'reviewed component producer source required')
 check_pins(review['source_pins']);reader=load_reader();matrix=Matrix(images_path,iq,p,review,review_ref);problem=None
 try:
  matrix.inputs();matrix.checkpoint(True);matrix.job=OwnedJob();matrix.e['owned_job_kill_on_close']=True
  matrix.data=DiagnosticStore(matrix.out,matrix.cleanup_authorized);matrix.e['diagnostics_before']=matrix.data.descriptors;matrix.save()
  inventories={}
  for profile in ['Debug','Release']:
   for variant in ['baseline','candidate']:
    row=matrix.invoke(profile,variant)
    require(not row['diagnostic_changes'],'--list unexpectedly changed synthetic diagnostic files')
    names=inventory(Path(row['stdout']).read_text('utf-8-sig'),Path(row['stderr']).read_text('utf-8-sig'),row['process']['exit_code'])
    prefixes=('modern_lito::native_login::rental_return_tests::','modern_lito::native_login::teleport_wire_tests::')
    require({name for name in names if name.startswith(prefixes)}=={case['test_name'] for case in p['cases'][variant]},'actual module inventory differs from planned cases')
    inventories[profile+'-'+variant]=names
  for profile in ['Debug','Release']:
   base=set(inventories[profile+'-baseline']);candidate=set(inventories[profile+'-candidate'])
   require(not base-candidate and candidate-base=={x['test_name'] for x in p['cases']['candidate']}-{x['test_name'] for x in p['cases']['baseline']},'unrelated test inventory changed')
   for variant in ['baseline','candidate']:
    matrix.inputs();cases=p['cases'][variant]
    ordered=[case for case in cases if case['existing_control']]+[case for case in cases if not case['existing_control']]
    for case in ordered:
     row=matrix.invoke(profile,variant,case)
     site=reader.assertion_site(p,case) if case['expected']=='specific_assertion_failure' else None
     selected_result(Path(row['stdout']).read_text('utf-8-sig'),Path(row['stderr']).read_text('utf-8-sig'),row['process']['exit_code'],
      case['test_name'],len(inventories[profile+'-'+variant]),case['expected'],site,case['panic_contains'])
  matrix.e['status']='original_matrix_processes_terminal_results_unreviewed'
 except BaseException as error:
  problem=error;matrix.e.update(status='failed_preserved_no_retry',error=repr(error));(matrix.out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8')
 finally:
  errors=[]
  if matrix.job is not None:
   try:
    if matrix.job.accounting()['active_processes']:
     matrix.job.terminate_members();matrix.e['forced_cleanup_used']=True
     end=time.monotonic()+15
     while matrix.job.accounting()['active_processes'] and time.monotonic()<end:time.sleep(.1)
    matrix.e['job_accounting_final']=matrix.job.accounting();require(matrix.e['job_accounting_final']['active_processes']==0,'original Job not empty; do not restore diagnostics yet')
    for child in matrix.held:
     if child.handle:require(child.terminal(),'original process not terminal');child.collect_exit()
     else:require(child.row['wait_confirmed'],'closed original handle lacks exit receipt')
    if matrix.data is not None:
     if matrix.data.pending is not None:
      # Clean up an interrupted process once, without running any case again.
      # A partial preservation/restoration never receives a second attempt.
      require(not matrix.data.capture_attempted,'partial diagnostic preservation retained; explicit recovery required')
      matrix.e['cleanup_diagnostic_receipt']=matrix.data.preserve_restore()
     matrix.data.all_restored();matrix.e['diagnostic_roots_restored']=True
   except BaseException as error:errors.append(repr(error))
   finally:
    for child in matrix.held:
     try:child.close()
     except BaseException as error:errors.append(repr(error))
    try:matrix.job.close()
    except BaseException as error:errors.append(repr(error))
  try:matrix.inputs();matrix.e['source_inputs_unchanged']=True;matrix.e['normal_products_unchanged']=True
  except BaseException as error:errors.append('source guard: '+repr(error))
  matrix.e.update(cleanup_errors=errors,finished_utc=datetime.now(timezone.utc).isoformat())
  if errors or matrix.e['forced_cleanup_used'] or not matrix.e['diagnostic_roots_restored']:
   matrix.e['status']='cleanup_or_input_failure_preserved';problem=problem or RuntimeError('component cleanup/input guard failed; preserve original artifacts')
  if problem is None:
   try:
    files=[images_path,Path(iq['plan']),Path(iq['compile_cohort']),Path(review_ref['qualification'])]
    files += [path for path in matrix.out.rglob('*') if path.is_file() and path.name not in ['execution.json','execution.json.writing']]
    matrix.e['evidence_pins']={str(owned(path)):sha(path) for path in files}
   except BaseException as error:
    errors.append('evidence preservation: '+repr(error));matrix.e['status']='evidence_preservation_failure';problem=error
  matrix.save()
  if matrix.registered and (matrix.job is None or matrix.e.get('job_accounting_final',{}).get('active_processes')==0):
   try:matrix.checkpoint(False)
   except BaseException as error:matrix.e['progress_release_error']=repr(error);matrix.save();problem=problem or error
 print(json.dumps(dict(run=str(matrix.out),status=matrix.e['status'],independent_reader_required=True,automatic_retries=0),ensure_ascii=False),flush=True)
 if problem is not None:problem.owned_case_output=str(matrix.out);raise problem
 return matrix.out/'execution.json'

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--images',type=Path,required=True)
 ap.add_argument('--execute-owned-tests',action='store_true',required=True);a=ap.parse_args();execute(a.images)
if __name__=='__main__':main()
