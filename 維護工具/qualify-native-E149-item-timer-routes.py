"""Qualify static Code findings and unresolved fixture prerequisites only."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from collections import Counter
import argparse,ast,hashlib,json,re,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(inspection):
 inspection=owned(inspection.resolve(strict=True),V/'native-E149-item-timer-routes');data=read(inspection);require(data['status']=='decoded_original_Code_timer_writers_and_candidate_routes_only' and data['source_manifest_sha256']==SOURCE and data['formal_target_sha256']==FORMAL,'static target differs');check_pins(data['outputs'])
 require(data['tests_executed']==data['classes_instantiated']==0 and not data['timer_or_game_methods_invoked'] and all(x['exit_code']==0 for x in data['commands']),'static scope differs')
 require(sha(data['jar'])==FORMAL,'original JAR changed');directory=inspection.parent;inventory=read(directory/'class_inventory.json');writes=read(directory/'field_writes.json');calls=read(directory/'lifetime_calls.json');schedules=read(directory/'schedule_calls.json');handles=read(directory/'method_handles.json');reflections=read(directory/'reflective_writes.json')
 require(len(inventory)==data['scanned_server_classes']==2554 and sum(x['methods_with_code'] for x in inventory)==data['scanned_methods_with_code']==31461 and sum(x['instructions'] for x in inventory)==data['decoded_instructions']==929761,'full Code inventory differs')
 with zipfile.ZipFile(data['jar']) as jar:
  expected=[n for n in jar.namelist() if n.startswith('local/lito381/server/') and n.endswith('.class')];require(len(set(expected))==len(expected) and {x['path'] for x in inventory}==set(expected),'JAR class inventory differs')
  require(all(hashlib.sha256(jar.read(x['path'])).hexdigest().upper()==x['class_sha256'] for x in inventory),'class bytes changed')
 require(len(writes)==data['field_write_instructions']==13 and len(calls)==5 and len(schedules)==74 and not handles,'observed instruction scope differs')
 # Separate primary javap enumeration, without importing/executing the scanner.
 actual=[]
 for command in data['commands']:
  path=next(Path(n) for n in command['logs'] if n.endswith('.stdout.txt'));method=None
  for line in path.read_text('utf-8-sig').splitlines():
   if line.startswith('  ') and not line.startswith('   ') and '(' in line and line.rstrip().endswith(';'):
    name=line.split('(',1)[0].split()[-1];method='<init>' if name==command['class_name'].replace('/','.') else name
   match=re.search(r'^\s*(\d+):\s+(?:putfield|putstatic)\b.*// Field (.+):',line)
   if match:
    field=match[2].rsplit('.',1)[-1]
    if field in data['target_fields']:actual.append((command['class_name'],method,int(match[1]),field))
 require(Counter(actual)==Counter((x['class_name'],x['method'],x['offset'],x['member']['name']) for x in writes),'independent javap writer inventory differs')
 actor='local/lito381/server/NativeDeWorldKinds$Item';require({x['class_name'] for x in writes}=={actor},'unexpected direct writer class')
 null_fields=['_magicStoneExpireFuture','_equipmentTimerFuture','_activeEquipmentTimer'];zero_fields=['_magicStoneExpirySubmissionInFlight','_magicStoneExpiryRetryScheduled']
 for field in null_fields:
  rows=[x for x in writes if x['member']['name']==field];require(rows and all(x['immediately_preceding_constant'] and x['constant_value'] is None and x['context'][-2]['opcode']==1 for x in rows),'non-null direct writer discovered: '+field)
 for field in zero_fields:
  rows=[x for x in writes if x['member']['name']==field];require(rows and all(x['immediately_preceding_constant'] and type(x['constant_value']) is int and x['constant_value']==0 and x['context'][-2]['opcode']==3 for x in rows),'true direct flag writer discovered: '+field)
 blocked_call_names=['applyMagicStoneExpiryIfNeeded','terminalizeEquipmentLocked','stopEquipmentTimerSerialized'];require(not any(x['member']['name'] in blocked_call_names or x['member']['name']=='<init>' and 'EquipmentTimerLease' in x['member']['owner'] for x in calls),'new direct timer entry discovered')
 require(not any(x['class_name']==actor for x in schedules),'Item scheduling entry discovered')
 tools=[T/n for n in ['native_E149_classfile_scan.py','inspect-native-E149-item-timer-routes.py','qualify-native-E149-item-timer-routes.py']]
 for path in tools:compile(ast.parse(path.read_text('utf-8'),filename=str(path)),str(path),'exec')
 for name in ['native_E149_classfile_scan.py','inspect-native-E149-item-timer-routes.py']:require(sha(T/name)==sha(directory/name),'original scan source snapshot differs')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'activity/deferral changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed');baseline=read(w['current_development_typecheck_qualification']['qualification']);inputs=read(Path(baseline['run'])/'base-compiled-inputs.json');require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal inputs changed');check_pins(baseline['normal_product_pins'])
 check_pins(read(w['current_E149_morph_denial_integration_development']['qualification'])['source_pins']);check_pins(read(w['current_E149_state_copy_observer_development']['qualification'])['source_pins'])
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 out=owned(V/'native-E149-item-timer-route-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 prerequisites=dict(schema='E149-item-timer-normal-GAME-prerequisites-v1',formal_target_sha256=FORMAL,source_inspection=str(inspection),source_inspection_sha256=sha(inspection),scope='static_explicit_Code_routes_only',fields=[dict(field=n,explicit_writes=sum(x['member']['name']==n for x in writes),normal_GAME_creation_proven=False,actual_timer_scenario_executed=False) for n in sorted(data['target_fields'])],
  missing=['authorized normal GAME packet/method path to an actual nondefault timed item','original item/template/resource provenance and positive owner admission','actual Future/lease/pending state before intended copy refusal','natural timer cleanup/expiry and original owner retirement evidence'],generic_reflection_and_external_paths_excluded=False,fixture_execution_authorized=False,full_native_owner_qualified=False)
 (out/'normal-GAME-prerequisites.json').write_bytes(encoded(prerequisites))
 record=dict(status='static_item_timer_Code_routes_and_unresolved_GAME_prerequisites',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,inspection=str(inspection),inspection_sha256=sha(inspection),observed_original_scan_tool_chunk='23dfcb',observed_actual_exit_code=0,scanned_server_classes=2554,scanned_methods_with_code=31461,decoded_instructions=929761,independent_javap_writer_inventory_equal=True,explicit_timer_field_writes=13,schedule_instructions=74,item_schedule_instructions=0,matching_method_handles=0,unresolved_generic_reflective_write_instructions=len(reflections),null_only_explicit_fields=null_fields,false_only_explicit_fields=zero_fields,no_direct_callers_found_for=blocked_call_names,normal_GAME_prerequisites=str(out/'normal-GAME-prerequisites.json'),normal_GAME_prerequisites_sha256=sha(out/'normal-GAME-prerequisites.json'),normal_GAME_creation_proven=False,source_pins={str(p):sha(p) for p in tools+[T/'native-E149-item-timer-routes.md']},AST_compiled_tools=3,original_fixture_binding_unchanged=w['current_E149_fixture_binding'],original_formal_suite_unchanged=w['current_E149_recovery_suite_development'],tests_executed=0,classes_instantiated=0,timer_or_copy_methods_invoked=False,DB_or_SQL_executed=False,desktop_inputs=0,normal_sources_unchanged=True,normal_PEs_unchanged=True,production_applied=False,full_native_owner_qualified=False,whole_goal_complete=False,
  limitations=['Static Code facts and javap agreement do not establish a successful normal GAME route','Generic reflection/dynamic loading/external callers are not excluded','No fake state is supplied and no unavailable scenario is counted as passed','Canonical40-attempt fixture/16 plans and unselected StateCopy observer unchanged'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw={Path(n).name:Path(n) for n in record['source_pins']};raw.update({'qualified-development.json':out/'qualified-development.json','normal-GAME-prerequisites.json':out/'normal-GAME-prerequisites.json','inspection.json':inspection})
 for n in data['outputs']:raw['original-scan/'+Path(n).name]=Path(n)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in raw.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work='G06計時物品入口已做全正式JAR實際Code掃描：2554類／31461有Code方法／929761指令，13處條件欄寫入逐method/offset與javap一致。兩Future與active lease直接寫入只清空、兩pending旗標只寫false，Item無schedule，未找到EquipmentTimerLease constructor與三個計時私方法的直接caller；74個其他排程不能移記為Item入口。通用反射／動態／外部仍未排除，七項正常GAME正向建立前提全維持未證明，沒有造Future／lease或把缺前提算過。三個Python工具僅AST／資料掃描，無類別初始化、timer／copy／案例／桌面。現行640次16案與StateCopy候選不變，正常1049輸入／兩EXE不變。下一步另查正常物品來源／反射路徑與可驗證的原授權入口，並繼續獨立剩餘玩法；測試暫緩，完整owner／Main／G06／G01–G12未完成。'
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_item_timer_route_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='G06_3d_original_timer_Code_route_prerequisites',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'progress format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items()),'concurrent writer');check_pins(baseline['normal_product_pins'])
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),static_Code_instructions=929761,javap_writer_inventory_equal=True,normal_GAME_timer_creation_proven=False,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('inspection',type=Path);a=p.parse_args();publish(a.inspection)
