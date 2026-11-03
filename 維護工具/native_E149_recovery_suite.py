"""Prepare or, after explicit test resumption, run the selected E149 case matrix.

This is 8 recovery cases x 2 isolated controller profiles, not the complete
library/GUI/G06 suite. No retry. The original producer owns each case Job;
the separate reader must qualify that case before another starts.
"""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,importlib.util,json,os,traceback,zipfile
from native_E149_recovery_contract import (R,P,FORMAL,SOURCE,SCENARIOS,read,sha,encoded,
 owned,require,write_new,check_pins,make_plan,validate_plan)
T=R/'維護工具';V=R/'驗證紀錄/current'

def authorize_suite():
 w=read(R/'工作佇列.json')
 require(not w['current_user_requested_goal'].get('test_execution_deferred_by_user',True),'user deferred tests; suite execution disabled')
 require(not w['active_workloads'] and not w.get('active_workload_sessions') and not w['source_and_corresponding_QA_frozen'],'another original workload/session/freeze is active')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 return w

def prepare_suite(producer_review=None,fixture_binding=None,reusable=None):
 # Pure file/plan preparation; imports no producer, reader or process helper.
 reusable=reusable or {};members=[]
 for profile in ['Debug','Release']:
  for scenario in SCENARIOS:
   if scenario=='cancel-repeat' and profile in reusable:
    row=reusable[profile];path=Path(row['path']);require(sha(path)==row['sha256'],'reusable plan changed')
   else:path=make_plan(profile,scenario,producer_review,fixture_binding)
   p=read(path);require(p['profile']==profile and p['scenario']==scenario,'member plan scope differs')
   members.append(dict(profile=profile,scenario=scenario,plan=str(path),plan_sha256=sha(path)))
 first=read(members[0]['plan'])
 out=T/'candidates'/('native-E149-recovery-suite-plan-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
 p=dict(schema='E149-selected-recovery-suite-plan-v3',status='development_plan_only',
  source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,members=members,
  fixture_source_binding=first['fixture_source_binding'],fixture_source_binding_sha256=first['fixture_source_binding_sha256'],
  runtime_producer_review=first['runtime_producer_review'],runtime_producer_review_sha256=first['runtime_producer_review_sha256'],
  runtime_producer_pins=first['runtime_producer_pins'],
  retired_item_mutation_probe_kind=first['retired_item_mutation_probe_kind'],retired_item_mutation_methods=first['retired_item_mutation_methods'],
  retired_item_mutation_targets=first['retired_item_mutation_targets'],retired_item_mutation_attempts=first['retired_item_mutation_attempts'],
  retired_item_morph_probe_kind=first['retired_item_morph_probe_kind'],retired_item_morph_methods=first['retired_item_morph_methods'],
  retired_item_morph_targets=first['retired_item_morph_targets'],retired_item_morph_attempts=first['retired_item_morph_attempts'],retired_item_aggregate_attempts=first['retired_item_aggregate_attempts'],
  planned_aggregate_retired_setter_attempts=first['retired_item_aggregate_attempts']*len(members),
  input_pins={str(Path(__file__).resolve()):sha(__file__),first['fixture_source_binding']:first['fixture_source_binding_sha256'],
   first['runtime_producer_review']:first['runtime_producer_review_sha256']},
  test_execution_deferred_at_preparation=read(R/'工作佇列.json')['current_user_requested_goal']['test_execution_deferred_by_user'],
  tests_executed=0,reader_functions_executed=False,workload_processes_started=0,desktop_inputs=0,
  full_library_qualified=False,GUI_verified=False,full_native_owner_qualified=False,whole_goal_complete=False)
 write_new(out/'plan.json',p);validate_suite(out/'plan.json');return out/'plan.json'

def validate_suite(path,full_inputs=False):
 p=read(owned(path,T/'candidates'))
 require(p['schema']=='E149-selected-recovery-suite-plan-v3' and p['status']=='development_plan_only','not a selected recovery suite plan')
 require(p['source_manifest_sha256']==SOURCE and p['formal_target_sha256']==FORMAL,'suite source/target differs')
 check_pins(p['input_pins']);check_pins(p['runtime_producer_pins'])
 expected=[(profile,scenario) for profile in ['Debug','Release'] for scenario in SCENARIOS]
 require([(row['profile'],row['scenario']) for row in p['members']]==expected,'suite selection/order differs')
 require(len({row['plan'] for row in p['members']})==16,'duplicate member plan path')
 require(p['retired_item_aggregate_attempts']==p['retired_item_mutation_attempts']+p['retired_item_morph_attempts']==40
  and p['planned_aggregate_retired_setter_attempts']==640,'suite planned morph/aggregate counts differ')
 for row in p['members']:
  require(sha(owned(row['plan']))==row['plan_sha256'],'suite member plan changed')
  member=validate_plan(row['plan']) if full_inputs else read(row['plan'])
  require(member['profile']==row['profile'] and member['scenario']==row['scenario'],'member selection differs')
  for key in ['fixture_source_binding','fixture_source_binding_sha256','runtime_producer_review','runtime_producer_review_sha256','runtime_producer_pins',
   'retired_item_mutation_probe_kind','retired_item_mutation_methods','retired_item_mutation_targets','retired_item_mutation_attempts',
   'retired_item_morph_probe_kind','retired_item_morph_methods','retired_item_morph_targets','retired_item_morph_attempts','retired_item_aggregate_attempts']:
   require(member[key]==p[key],'member source binding differs: '+key)
 return p

def load_source(name,path):
 specification=importlib.util.spec_from_file_location(name,path)
 module=importlib.util.module_from_spec(specification);specification.loader.exec_module(module);return module

def save(path,value):
 temporary=path.with_name(path.name+'.writing')
 with temporary.open('xb') as file:file.write(encoded(value))
 os.replace(temporary,path)

def checkpoint_terminal(out,record):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],'same original case is not released; preserve suite audit without touching progress')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
 require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'progress checkpoint changed')
 final=dict(record,previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 write_new(out/'evidence.json',final)
 work=('E149兩版各八個精確恢復案例原結果均經獨立reader核對，僅此16案矩陣範圍；'
  if final['status']=='qualified_selected_case_matrix' else 'E149精確恢復矩陣沿原案例停止並保留失敗／已核對前綴，未重試；')
 work+='完整library／GUI、完整owner／Main／G01–G12仍待，不移記整體資格。'
 staged={}
 for name in before:
  path=R/name
  if path.suffix=='.json':
   j=read(path);j['current_E149_recovery_suite_execution']=dict(execution=str(out/'suite.json'),evidence=str(out/'evidence.json'),
    status=final['status'],qualified_selected_cases=len([x for x in final['results'] if x['status']=='qualified']),whole_goal_complete=False)
   j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
   if name.endswith('next-validation-action.json'):j['next_action']=work
   staged[name]=encoded(j)
  else:
   _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs')
   staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name in before:z.write(R/name,name)
 with zipfile.ZipFile(out/'before.zip') as z:
  require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'suite checkpoint archive differs')
 require(all(sha(R/n)==h for n,h in before.items()),'concurrent progress update')
 for name,blob in staged.items():(R/name).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'suite checkpoint readback differs')
 write_new(out/'readback.json',dict(before_files=before,after_files={n:sha(R/n) for n in before},
  before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))

def execute_suite(plan_path):
 authorize_suite() # first: no runtime imports/Job/process before current consent
 plan_path=owned(plan_path.resolve(strict=True),T/'candidates');p=validate_suite(plan_path,full_inputs=True)
 w=authorize_suite()
 require(w['current_E149_fixture_binding']['qualification_sha256']==p['fixture_source_binding_sha256'],'suite uses an earlier fixture binding')
 producer=load_source('_lito_E149_suite_producer',T/'run-native-E149-recovery-case.py')
 reader=load_source('_lito_E149_suite_reader',T/'verify-native-E149-recovery-case.py')
 out=V/'native-E149-recovery-suite-execution'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 record=dict(schema='E149-selected-recovery-suite-execution-v2',status='executing_selected_case_matrix',
  plan=str(plan_path),plan_sha256=sha(plan_path),results=[],current_member=None,
  source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,runtime_source_pins=p['runtime_producer_pins'],
  automatic_retries=0,desktop_inputs=0,full_library_qualified=False,GUI_verified=False,
  full_native_owner_qualified=False,full_Main_process_qualified=False,whole_goal_complete=False,
  started_utc=datetime.now(timezone.utc).isoformat())
 save(out/'suite.json',record);problem=None
 try:
  for selected in p['members']:
   authorize_suite();check_pins(p['runtime_producer_pins'])
   row=dict(selected,status='producer_running',started_utc=datetime.now(timezone.utc).isoformat())
   record['results'].append(row);record['current_member']=selected;save(out/'suite.json',record)
   # Same Python pipeline invokes the original producer; every workload child
   # is still suspended/enrolled in that case's Job by the producer itself.
   execution=producer.execute(Path(selected['plan']))
   row.update(execution=str(execution),execution_sha256=sha(execution),status='awaiting_independent_readback');save(out/'suite.json',record)
   authorize_suite();result=reader.verify_execution(execution)
   require(result['status']=='qualified_selected_E149_recovery_case_observations_and_wire'
    and result['profile']==selected['profile'] and result['scenario']==selected['scenario'],'independent case reader scope differs')
   qualification=execution.with_name('qualified-selected-case.json');write_new(qualification,result)
   row.update(qualification=str(qualification),qualification_sha256=sha(qualification),status='qualified',finished_utc=datetime.now(timezone.utc).isoformat())
   record['current_member']=None
   save(out/'suite.json',record)
   print(json.dumps(dict(suite=str(out),qualified=len(record['results']),total=16,last_profile=selected['profile'],last_scenario=selected['scenario'])),flush=True)
  require(len(record['results'])==16 and all(x['status']=='qualified' for x in record['results']),'suite matrix incomplete')
  validate_suite(plan_path);require(sha(read(R/'工作佇列.json')['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed before matrix readback')
  record['status']='qualified_selected_case_matrix';record['current_member']=None
 except BaseException as error:
  problem=error;record.update(status='stopped_preserved_no_retry',error=repr(error))
  if record['results']:
   row=record['results'][-1]
   if row['status']!='qualified':
    row.update(status='failed_or_unreviewed_preserved',error=repr(error),owned_case_output=getattr(error,'owned_case_output',None))
  (out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8')
 finally:
  record['finished_utc']=datetime.now(timezone.utc).isoformat();save(out/'suite.json',record)
  try:checkpoint_terminal(out,record)
  except BaseException as error:
   record['checkpoint_failure']=repr(error);save(out/'suite.json',record)
   problem=problem or error
 print(json.dumps(dict(suite=str(out),status=record['status'],automatic_retries=0,whole_goal_complete=False)),flush=True)
 if problem is not None:raise problem
 return out/'suite.json'

def main():
 ap=argparse.ArgumentParser(description=__doc__);group=ap.add_mutually_exclusive_group(required=True)
 group.add_argument('--prepare',action='store_true');group.add_argument('--plan',type=Path)
 ap.add_argument('--execute-owned-tests',action='store_true');a=ap.parse_args()
 if a.prepare:
  require(not a.execute_owned_tests,'preparation does not execute tests');path=prepare_suite()
  print(json.dumps(dict(suite_plan=str(path),members=16,tests_executed=0),ensure_ascii=False))
 else:
  require(a.execute_owned_tests,'explicit execution flag required');execute_suite(a.plan)
if __name__=='__main__':main()
