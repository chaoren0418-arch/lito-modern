"""Publish actual compiler-only images and preserve the current user test deferral."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(qualification,terminal_chunk,original_session):
 qualification=owned(qualification.resolve(strict=True));q=read(qualification)
 require(q['schema']=='E149-component-images-qualified-v1' and q['status']=='qualified_linked_selected_component_matrix_images',
  'four independently qualified actual images required')
 require(q['source_manifest_sha256']==SOURCE and q['formal_target_sha256']==FORMAL
  and q['original_compiler_exit_codes']==[0,0,0,0] and q['original_compilers_terminal'] is True
  and q['compiler_controls_archived'] is True,'compiler identity/scope differs')
 require(q['observed_original_tool_chunk']==terminal_chunk and q['observed_actual_exit_code']==0
  and type(original_session) is int and original_session>0,'actual terminal tool receipt differs')
 require(q['tests_executed']==0 and q['PEs_executed'] is False and q['test_harness_inventory_executed'] is False,
  'not a compiler-only qualification')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions']
  and not w['source_and_corresponding_QA_frozen'],'another original workload/session is active')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 source_ref=w['current_E149_component_execution_development'];require(sha(source_ref['qualification'])==source_ref['qualification_sha256'],'component source review changed')
 source=read(source_ref['qualification']);check_pins(source['source_pins'])
 require(source['AST_compiled_modules']==6 and source['component_producer_source_complete'] is True
  and source['diagnostic_preservation_source_complete'] is True,'component tool source incomplete')
 require(q['qualifier_sha256']==source['source_pins'][str(T/'qualify-native-E149-component-images.py')],'executed image qualifier source differs')
 plan=read(q['plan']);require(sha(q['plan'])==q['plan_sha256'],'component plan changed')
 build=read(q['compile_cohort']);require(sha(q['compile_cohort'])==q['compile_cohort_sha256'],'sealed compiler cohort changed')
 check_pins(build['input_pins']);check_pins(build['normal_product_pins'])
 require(sha(q['original_observed_cohort'])==q['original_observed_cohort_sha256'],'original compiler cohort changed')
 session_path=Path(q['original_observed_cohort']).with_name('original-tool-session.json');session=read(session_path)
 require(session['original_tool_session_id']==original_session and session['pipeline']==build['pipeline']
  and session['cohort']==q['original_observed_cohort'] and session['compiler_only'] is True
  and session['tests_executed']==0 and session['PEs_executed'] is False,'original compiler session journal differs')
 inputs=read(plan['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal inputs changed')
 require(set(q['images'])=={profile+'-'+variant for profile in ['Debug','Release'] for variant in ['baseline','candidate']},'image set differs')
 for name,row in q['images'].items():
  file=owned(row['product'],P/'target/current/verification/native-E149-component-matrix')
  require(sha(file)==row['sha256'] and file.stat().st_size==row['bytes'] and row['bytes']>0,'qualified image changed: '+name)
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
 require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'progress checkpoint changed')
 compile(ast.parse(Path(__file__).read_text('utf-8'),filename=str(__file__)),str(__file__),'exec')
 out=V/'native-E149-component-linked-images-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 (out/'publisher.py').write_bytes(Path(__file__).read_bytes())
 raw={'qualified-images.json':qualification,'qualified-build.json':Path(q['compile_cohort']),
  'original-cohort.json':Path(q['original_observed_cohort']),'component-source-review.json':Path(source_ref['qualification']),
  'plan.json':Path(q['plan']),'qualifier.py':qualification.parent/'qualifier.py',
  'owned-CIM.json':qualification.parent/'owned-CIM.json','publisher.py':out/'publisher.py',
  'original-tool-session.json':session_path}
 for row in build['compiler_runs']:
  for file in row['logs']:raw[row['profile']+'-'+row['variant']+'/'+Path(file).name]=Path(file)
 for row in q['compiler_control_snapshots'].values():raw['control-inputs/'+Path(row['snapshot']).name]=Path(row['snapshot'])
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name,path in raw.items():z.write(owned(path),name)
 with zipfile.ZipFile(out/'raw.zip') as z:
  require(len(z.namelist())==len(raw) and all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in raw.items()),'raw source/compiler archive differs')
 record=dict(status='four_component_library_test_images_linked_qualified_not_executed',
  qualification=str(qualification),qualification_sha256=sha(qualification),source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  images=q['images'],compile_cohort=q['compile_cohort'],compile_cohort_sha256=q['compile_cohort_sha256'],
  original_observed_cohort=q['original_observed_cohort'],original_observed_cohort_sha256=q['original_observed_cohort_sha256'],
  original_tool_session_id=original_session,observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,
  original_compiler_exit_codes=[0,0,0,0],original_compilers_terminal=True,compiler_controls_archived=True,
  tool_source_qualification=source_ref['qualification'],tool_source_qualification_sha256=source_ref['qualification_sha256'],
  plan=q['plan'],plan_sha256=q['plan_sha256'],raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),
  source_pins=source['source_pins'],normal_inputs_unchanged=True,normal_products_unchanged=True,
  tests_executed=0,PEs_executed=False,test_inventory_executed=False,component_matrix_reader_executed=False,
  diagnostic_store_constructed=False,desktop_inputs=0,full_library_qualified=False,full_G06_qualified=False,whole_goal_complete=False,
  test_execution_deferred_by_user=w['current_user_requested_goal']['test_execution_deferred_by_user'],
  desktop_automation_deferred_by_user=w['current_user_requested_goal']['desktop_automation_deferred_by_user'],
  previous_goal_turn_evidence=w['last_goal_turn_evidence'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'evidence.json').write_bytes(encoded(record))
 work='G06原版／候選四份元件library-test PE已實際連結，Debug／Release各baseline與candidate原rustc均exit0；獨立核對原命令、來源／依賴、x64 PE32+／SHA、compiler終止及控制檔封存。正常1049輸入／兩EXE不變。這只是編譯資格：四PE皆未執行，--list／102精確案／6指定反例／診斷回復／正式suite／GUI尚未實跑。元件producer與資料保護來源沿前次6模組AST資格；下一步待使用者恢復測試後續原計畫實際inventory及對照矩陣，再驗正式16案與完整owner／期限／library／Release／GUI。使用者測試與桌面暫緩旗標維持，完整G01–G12未完成。'
 staged={}
 for name in before:
  path=R/name
  if path.suffix=='.json':
   j=read(path);j['current_E149_component_linked_images']=record
   j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
   j['current_focus']=dict(item='G06_component_four_actual_linked_images_not_executed',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if name.endswith('next-validation-action.json'):j['next_action']=work
   staged[name]=encoded(j)
  else:
   _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs')
   staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name in before:z.write(R/name,name)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items()),'concurrent checkpoint/input writer')
 check_pins(build['normal_product_pins']);check_pins(source['source_pins'])
 for name,blob in staged.items():(R/name).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'progress readback differs')
 (out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),linked_images=4,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('qualification',type=Path)
 ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--original-session',required=True,type=int)
 a=ap.parse_args();publish(a.qualification,a.terminal_chunk,a.original_session)
if __name__=='__main__':main()
