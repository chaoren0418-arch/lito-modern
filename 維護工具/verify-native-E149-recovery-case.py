"""Future selected-case evidence reader. Never starts fixtures, DBs or tests.

No existing historical case is accepted. A v2 plan binds the original linked
Rust image and the separately compiled Java fixture; source readiness is not runtime qualification.
"""
from pathlib import Path
from datetime import datetime,timezone
import argparse,collections,json,re,struct,subprocess
from native_E149_recovery_contract import (R,P,FORMAL,SQL,CATALOG,SCENARIOS,ITEM_FIELDS,
 require,owned,read,sha,write_new,check_pins,validate_plan,client_image_binding)
from native_E149_item_attributes import observed_attributes,compare_staged_adoption,nondefault_coverage,PERMANENT_FIELDS,DIAGNOSTIC_FIELDS
from native_E149_owner_guards import verify_owner_guards
from native_E149_morph_denials import verify_retired_morphs

def jsonlines(path):
 return [json.loads(line) for line in owned(path).read_text('utf-8-sig').splitlines() if line.strip()]

def by_id(rows):
 require(isinstance(rows,list),'inventory rows absent')
 result={row['id']:row for row in rows};require(len(result)==len(rows),'duplicate item IDs')
 for row in rows:
  require(set(row)==ITEM_FIELDS,'pinned RuntimeInventory observation field set differs')
  require(row['count']>0 and row['bound_actor_present'] is True,'live native actor/count missing')
 return result

def without_ids(rows):
 return collections.Counter(json.dumps({k:v for k,v in row.items() if k!='id'},sort_keys=True,ensure_ascii=False) for row in rows)

def wire_inventory(body):
 require(len(body)>=2 and body[0]==0x7c,'invalid S7C')
 at=2;items={}
 for _ in range(body[1]):
  require(at+14<=len(body),'truncated S7C item header')
  oid,use,charge,icon,bless,count,identified=struct.unpack_from('<IBBHBIB',body,at);at+=14
  require(oid not in items and count>0 and bless<=3 and identified in [0,1],'invalid S7C fields')
  end=body.find(b'\0',at);require(end>=at,'unterminated S7C name')
  name=body[at:end].decode('cp950','strict');at=end+1
  require(at<len(body),'missing S7C detail length');size=body[at];at+=1
  require(at+size<=len(body),'truncated S7C details');detail=body[at:at+size];at+=size
  items[oid]=dict(id=oid,use_type=use,charge=charge,icon=icon,blessing=bless,count=count,
   identified=identified,name=name,detail_hex=detail.hex())
 require(at==len(body),'S7C trailing bytes');return items

def wire_signature(rows):
 return collections.Counter(json.dumps({k:v for k,v in row.items() if k!='id'},sort_keys=True,ensure_ascii=False) for row in rows)

def consistent_batch(batches,ids,start,end,exact=False):
 matches=[(i,rows) for i,rows in batches if start<i<end and (set(rows)==ids if exact else ids.issubset(rows))]
 require(matches,'no matching actual inventory wire batch')
 selected=[{oid:rows[oid] for oid in ids} for _,rows in matches]
 require(all(row==selected[0] for row in selected),'same item IDs changed wire fields across candidate batches')
 require(all(v['detail_hex'] for v in selected[0].values()),'extended item details absent')
 return matches[0][0],selected[0]

def source_identity(source,name):
 # GameStateStore.sourceCharacter trims Java UTF-16 code units <= U+0020.
 # Python str.strip() has a wider Unicode set and would change this contract.
 require(type(source) is str and type(name) is str,'source identity components absent')
 java_trim_characters=''.join(chr(n) for n in range(33))
 source,name=source.strip(java_trim_characters),name.strip(java_trim_characters)
 require(source and name and '\x1f' not in source and '\x1f' not in name,'invalid source-qualified identity')
 return source+'\x1f'+name

def live_owner(row,inventory_key,account):
 require(row['observer_kind']=='G06_E149_direct_fields_v1' and row['observer_no_gameplay_getters'] is True,'wrong observer')
 require(row['stable_observed'] is True and row['source_item_application_active'] is True,'unstable/missing native application')
 require(row['test_account']==account and row['source']=='測試乙','account/source changed')
 owner=row['source_item_owner'];require(isinstance(owner,dict),'active native owner absent')
 require(owner['active'] is True and owner['session_matches'] is True,'owner stale or different session')
 require(owner['account']==source_identity(row['source'],account)
  and owner['source']==source_identity(row['source'],row['character']) and owner['selected']==row['character']
  and owner['object_id']==row['object_id'],'owner character/source/object/account binding differs')
 inventory=row[inventory_key+'_state'];require(inventory['exists'] and inventory['stable_observed'],'inventory observation unstable')
 require(inventory['rows']==row[inventory_key],'root/state inventory projection differs')
 require(inventory['backing']=='local.lito381.server.NativeDeItemInventory','original native backing absent')
 require(inventory['native_owner_id']==owner['observer_owner_id']>0,'inventory belongs to another owner')
 require(inventory['rental_character'] is None and inventory['rental_account'] is None,'staged capsule granted as live inventory')
 require(row['full_native_owner_qualification_inferred'] is False,'observer asserted a qualification')
 return owner

def retired(row):
 owner=row['source_item_owner']
 return row['stable_observed'] is True and row['source_item_disconnected'] is True and (owner is None or owner['active'] is False)

def check_terminal_processes(rows):
 require(rows and all(x['wait_confirmed'] is True and isinstance(x['exit_code'],int) for x in rows),'actual process exit waits missing')
 require(len({(x['pid'],x['creation_filetime_ticks']) for x in rows})==len(rows),'duplicate original process identities')
 ids=','.join(str(int(x['pid'])) for x in rows)
 command=['pwsh','-NoProfile','-NonInteractive','-Command',
  "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"]
 observed=subprocess.run(command,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
 require(observed.returncode==0,'terminal identity OS observation failed')
 actual=json.loads(observed.stdout.decode('utf-8-sig')) if observed.stdout.strip() else []
 actual=[actual] if isinstance(actual,dict) else actual
 for row in rows:
  millis=(row['creation_filetime_ticks']-116444736000000000)//10000
  require(row['start_epoch_millis']==millis,'process birth precision changed')
  for other in actual:
   if other['ProcessId']==row['pid']:
    started=datetime.fromisoformat(other['created'].replace('Z','+00:00'))
    epoch=datetime(1970,1,1,tzinfo=timezone.utc);delta=started-epoch
    current_millis=(delta.days*86400+delta.seconds)*1000+delta.microseconds//1000
    require(current_millis!=millis,'original process is still live')
 return observed.stdout.decode('utf-8-sig')

def verify_execution(execution_path):
 execution_path=owned(execution_path,P/'target/current/verification/native-E149-recovery-cases')
 e=read(execution_path);require(e['schema']=='E149-recovery-execution-v4','execution schema differs')
 require(e['status']=='owned_processes_exited_results_unreviewed','not a completed raw execution')
 require(sha(owned(e['plan']))==e['plan_sha256'],'case plan changed');p=validate_plan(e['plan'])
 require(not p['runtime_producer_implementation_pending'],'current plan is preparation only; no reviewed runtime producer')
 require(e['scenario']==p['scenario'] and e['profile']==p['profile'],'execution scope differs')
 require(e['inputs_unchanged'] is True and e['forced_cleanup_used'] is False,'inputs drifted or cleanup was forced')
 require(e['owned_job_kill_on_close'] is True and e['job_accounting_final']['active_processes']==0,'owned process job not terminal')
 check_pins(e['evidence_pins']);check_pins(e['runtime_producer_pins'])
 require(e['runtime_producer_source_reviewed'] is True,'producer source review absent')
 for key in ['fixture_source_binding','fixture_source_binding_sha256','fixture_qualification','fixture_qualification_sha256',
  'Java_source_pins','Java_classes','client_source_binding','owner_identity_contract','owner_identity_contract_sha256',
  'linked_image_qualification','linked_image_qualification_sha256','owner_guard_probe_kind',
  'retired_item_mutation_probe_kind','retired_item_mutation_methods','retired_item_mutation_targets','retired_item_mutation_attempts',
  'retired_item_morph_probe_kind','retired_item_morph_methods','retired_item_morph_targets','retired_item_morph_attempts','retired_item_aggregate_attempts']:
  require(e[key]==p[key],'executed fixture/client context differs: '+key)
 check_pins(e['Java_source_pins']);check_pins(e['Java_classes'])
 image=read(e['linked_image_qualification']);require(sha(e['linked_image_qualification'])==e['linked_image_qualification_sha256'],'image qualification changed')
 require(image['status']=='qualified_linked_isolated_E149_recovery_controller','actual linked image qualification missing')
 require(client_image_binding(image,p['effective_overlay_files'],p['effective_overlay_paths'])==p['client_source_binding'] and image['profile']==p['profile'],'linked Rust source/profile differs')
 require(image['source_manifest_sha256']==p['source_manifest_sha256'] and image['full_source_and_overlay_pins_verified'] is True,'full source identity missing')
 require(image['compiler_exit_codes']==[0,0] and image['recorded_original_compilers_terminal'] is True,'actual linking evidence missing')
 build=read(image['compile_cohort']);require(sha(image['compile_cohort'])==image['compile_cohort_sha256'],'raw linking cohort changed')
 require(build['schema']=='E149-recovery-linked-build-v1' and build['status']=='linked_compilers_exited','raw linked build scope differs')
 require(build['effective_overlay_files']==image['effective_overlay_files'] and build['effective_overlay_paths']==image['effective_overlay_paths']
  and build['base_compiled_inputs']==p['base_compiled_inputs'],'original raw linked context differs')
 check_pins(build['input_pins'])
 require(all(build['input_pins'].get(build['effective_overlay_paths'][n])==h for n,h in image['effective_overlay_files'].items()),'raw build omitted original merged source pins')
 require(len(build['compiler_runs'])==2 and [x['kind'] for x in build['compiler_runs']]==['library','formal_test'],'actual library/test linking commands missing')
 for row in build['compiler_runs']:
  require(row['exit_code']==0 and '--emit=link' in row['command'] and Path(row['command'][0]).name=='rustc.exe','compiler did not link successfully')
  check_pins(row['logs'])
 formal_source=build['effective_overlay_paths']['LiTo381-Modern/tests/native_teleport_failure.rs']
 require(formal_source in build['compiler_runs'][1]['command'] and sha(owned(formal_source))==p['effective_overlay_files']['LiTo381-Modern/tests/native_teleport_failure.rs'],'linked wrong formal Rust case bytes')
 product=owned(image['product'],P/'target/current');require(sha(product)==image['product_sha256'],'linked PE changed')
 pe=product.read_bytes();require(pe[:2]==b'MZ' and len(pe)>256,'not a PE')
 offset=struct.unpack_from('<I',pe,0x3c)[0];require(pe[offset:offset+4]==b'PE\0\0' and struct.unpack_from('<H',pe,offset+4)[0]==0x8664,'not x64 PE')
 require(e['controller_product_sha256']==image['product_sha256'],'executed controller differs')
 runtime=owned(e['runtime'],P/'target/current/verification/native-E149-owned-mysql')
 require(runtime.is_dir(),'owned runtime absent');ready=read(runtime/'ready.json');result=read(runtime/'result.json');db=read(runtime/'state.json')
 scenario=p['scenario'];test_name,expired,repeated=SCENARIOS[scenario];expected_sessions=10 if scenario=='wrong-service' else 8
 require(ready['observer_kind']=='G06_E149_resources_direct_fields_v2' and ready['trace_observer_kind']=='G06_E149_direct_fields_v1','wrong ready observer scope')
 for row in [ready,result]:
  require(row['owner_guard_probe_kind']==p['owner_guard_probe_kind'],'original owner guard scope differs')
  require(row['retired_item_mutation_probe_kind']==p['retired_item_mutation_probe_kind']
   and row['retired_item_mutation_methods']==len(p['retired_item_mutation_methods'])
   and row['retired_item_mutation_targets']==p['retired_item_mutation_targets'],'retired setter probe scope differs')
  require(row['retired_item_morph_probe_kind']==p['retired_item_morph_probe_kind']
   and row['retired_item_morph_methods']==len(p['retired_item_morph_methods'])
   and row['retired_item_morph_targets']==p['retired_item_morph_targets'],'retired morph probe scope differs')
  require(row['shared_client_resources_initialized'] is True and row['native_item_attribute_observer_kind']==p['native_item_attribute_observer_kind'],'shared initializer/extended observer scope differs')
  require(row['jar_sha256'].upper()==FORMAL and row['original_resource_factories_initialized'] is True
   and row['native_server_lifetime_installed'] is True and row['static_sql_access'] is True
   and row['player_sql_access'] is False and row['full_main_process_qualified'] is False,'original resource/SQL scope differs')
 require(ready['requested_source']=='測試乙' and ready['default_source']=='測試甲' and ready['scenario']==scenario,'source/scenario differs')
 require(ready['static_tables']==19 and ready['static_rows']==6277,'static dataset differs')
 for name,key in [('database-owner.properties','owner_proof_sha256'),('static-import.properties','import_proof_sha256')]:
  file=runtime/name;require(sha(file)==ready[key]==db['proof_pins'][str(file)],'actual DB proof differs')
 require(result['status']=='sessions_closed' and result['actual_client_sessions']==expected_sessions
  and result['memberships_empty'] is True and result['staged_loadouts_after_reentry']==0
  and result['original_futures_completed'] is True and result['native_cleanup_completed'] is True
  and result['state_persistent'] is False and result['authority_mutations']==0,'session/native cleanup barrier differs')
 require(result['native_owner_runtime_qualified'] is False,'fixture must not grant full owner qualification')
 require(db['status']=='stopped' and db['owned_daemon_terminal'] is True and not db['cleanup_errors']
  and db['bootstrap_exit_code']==db['import_exit_code']==0 and db['SQL_executed'] is True
  and db['static_SQL_access'] is True and db['player_SQL_access'] is False,'actual private DB/import/shutdown incomplete')
 actual_tables={k:v['rows'] for k,v in read(p['static_manifest'])['selected_tables'].items()}
 require(db['actual_table_rows']==actual_tables and sum(actual_tables.values())==6277,'actual imported table rows differ')
 rows=e['processes'];kinds={row['kind'] for row in rows}
 require(all(row['exit_code']==0 for row in rows),'a recorded owned process failed')
 require({'database_producer','database_bootstrap','database_daemon','fixture','controller','mysql_client'}.issubset(kinds),'nested process history incomplete')
 for kind in ['database_producer','database_bootstrap','database_daemon','fixture','controller']:
  matches=[x for x in rows if x['kind']==kind];require(len(matches)==1 and matches[0]['exit_code']==0,'original process failed: '+kind)
 controller=next(x for x in rows if x['kind']=='controller');fixture=next(x for x in rows if x['kind']=='fixture')
 require(all(x['created_suspended'] and x['job_assigned_before_resume'] for x in rows if x['kind'] in ['database_producer','fixture','controller']),
  'a workload child ran before owned job enrollment')
 bundle=read(p['database_bundle']);base=owned(bundle['bundle_root'],P/'target/current/owned-database-toolchains')
 for row in rows:
  executable=Path(row['command'][0]);kind=row['kind']
  if kind=='controller':expected={product}
  elif kind=='fixture':expected={Path(p['java'])}
  elif kind=='database_producer':expected={Path(p['python'])}
  elif kind in ['database_bootstrap','database_daemon']:expected={base/'bin/mysqld.exe'}
  elif kind=='mysql_client':expected={base/'bin/mysql.exe',base/'bin/mysqladmin.exe'}
  else:raise RuntimeError('unreviewed process kind: '+kind)
  require(executable in expected and row['executable_sha256']==sha(executable),'executed process image differs')
  if kind in ['database_bootstrap','database_daemon']:
   args=row['command'];require(args[1]=='--no-defaults','database inherited global configuration')
   dirs=[v.split('=',1)[1] for v in args if v.startswith('--datadir=')]
   require(len(dirs)==1 and owned(dirs[0],runtime)==runtime/'data','database data directory differs')
   if kind=='database_bootstrap':require('--bootstrap' in args and '--skip-networking' in args,'bootstrap bind differs')
   else:
    require('--bind-address=127.0.0.1' in args and '--bootstrap' not in args,'daemon binding differs')
    require(row['pid']==db['process']['pid'] and row['creation_filetime_ticks']==db['process']['identity']['creation_filetime_ticks'],'import belonged to another daemon')
 require(controller['command']==[str(product),test_name,'--exact','--ignored','--nocapture'],'executed test selection differs')
 require(controller['environment']=={'LITO_TELEPORT_SCENARIO':scenario,'LITO_TELEPORT_ROOT':str(runtime),'LITO_TELEPORT_REFERENCE_SHA':FORMAL.lower()},'controller environment differs')
 classpath=p['fixture_classes']+';'+p['formal_reference']+';'+p['JDBC']
 expected_java=[p['java'],'-Dfile.encoding=UTF-8','-Djava.io.tmpdir='+str(runtime/'tmp'),'-XX:-UsePerfData',
  '-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+p['catalog'],
  '-Xverify:all','-classpath',classpath,'local.lito381.server.NativeTeleportFailureHarness',str(runtime),scenario,
  str(runtime/'database-owner.properties'),ready['owner_proof_sha256'],str(runtime/'static-import.properties'),ready['import_proof_sha256'],p['SQL'],'--execute-owned-tests']
 require(fixture['command']==expected_java,'JDBC/classpath/resource profile/exact fixture command differs')
 require(fixture['inherited_java_options_removed'] is True,'inherited Java options not excluded')
 require(e['database_original_handle_signalled'] is True,'daemon original handle was not observed terminal')
 terminal_observation=check_terminal_processes(rows)
 test_log=owned(e['controller_stdout'],execution_path.parent).read_text('utf-8-sig')
 require(re.search(r'^test '+re.escape(test_name)+r' \.\.\. ok\s*$',test_log,re.M),'selected test did not pass')
 require(re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out;',test_log),'case inventory/result count differs')

 captures=list((runtime/'captures').glob('*/frames.jsonl'));require(len(captures)==1,'ambiguous actual packet capture')
 required_files=[runtime/n for n in ['ready.json','result.json','state.json','database-owner.properties','static-import.properties',
  'server-trace.jsonl','staged-trace.jsonl','owner-guard-trace.jsonl','item-mutation-denial-trace.jsonl','item-morph-denial-trace.jsonl','a-failure.json','b-failure.json']]+[captures[0],Path(e['controller_stdout']),Path(e['linked_image_qualification']),Path(image['compile_cohort']),
  Path(p['fixture_source_binding']),Path(p['fixture_qualification'])]
 required_files += [runtime/(role+suffix) for role in ['a','b'] for suffix in ['-continuation-observe-request','-continuation-observed.json']]
 for file in required_files:require(e['evidence_pins'].get(str(file))==sha(owned(file)),'consumed evidence is not pinned: '+str(file))
 frames=jsonlines(captures[0]);packets=[];batches=[];teleports={}
 require(sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==expected_sessions,'actual login count differs')
 for i,row in enumerate(frames):
  if row['plain']=='MASKED_C_LOGIN':continue
  body=bytes.fromhex(row['plain']);require(body and row['wire_length']==len(body)+2,'capture frame length differs')
  packets.append((i,row,body))
  if row['direction']=='server_to_client' and body[0]==0x7c:batches.append((i,wire_inventory(body)))
  if row['direction']=='server_to_client' and body[0]==0xf4 and 'rental-teleport' in row['note']:
   require(body[1] not in teleports,'duplicate redirect route');teleports[body[1]]=i
 require(set(teleports)=={1,2},'actual destination routes differ')
 outgoing=[b for _,row,b in packets if row['direction']=='client_to_server']
 c8=[b for b in outgoing if b[0]==0xc8];expected_c8=8 if scenario=='cancel-expiry-source' else 6
 require(len(c8)==expected_c8 and all(b==b'\xc8'+'測試乙'.encode('cp950')+b'\0' for b in c8),'fresh source request count/value differs')
 require(sum(b[0]==0xc9 for b in outgoing)==(4 if expired else 2),'fresh versus staged selection count differs')
 require(outgoing.count(b'\xd9')==6,'room entry/exit count differs')
 trace=jsonlines(runtime/'server-trace.jsonl');stages=jsonlines(runtime/'staged-trace.jsonl');checks=[];guard_roles=[]
 for role,account,slot,count in [('a','PeerFixture',1,14),('b','PeerFixture2',2,11)]:
  data=read(runtime/(role+'-failure.json'));before,failed,recovery,continued,reentered=[data[k] for k in ['before','failed','recovery','continued','reentered']]
  require(data['scenario']==scenario and data['full_game_reentry_verified'] is True,'role result/scope differs')
  require(data['requested_source']=='測試乙' and data['default_source']=='測試甲','role source contract differs')
  require([data['failed_epoch'],data['recovery_epoch'],data['reentered_epoch']]==[data['old_epoch']+n for n in [1,2,3]],'connection epochs differ')
  require(failed['world']['actors']=={} and failed['inventory']['owner'] is None and failed['merchant']['owner'] is None
   and failed['selected_character'] is None and failed['joined_room'] is None,'previous client authority survived')
  require(recovery['phase']=='sources_ready' and recovery['world']['actors']=={} and recovery['inventory']['owner'] is None,'fresh login published old authority')
  cancelled=scenario in ['cancel-retry','cancel-repeat','cancel-expiry','cancel-expiry-source']
  require(failed['phase']==('closed' if cancelled else 'failed'),'failure/cancel state differs')
  if cancelled:require(failed['error'] is None and data['elapsed_ms']<5000 and data['closed_samples']>0,'cancel became a timeout')
  elif scenario in ['refused','refused-cancel-repeat']:require(data['elapsed_ms']<5000 and 'native TCP connect' in failed['error'],'actual refusal missing')
  elif scenario=='wrong-service':require(data['elapsed_ms']<15000 and failed['error'],'wrong service did not reject')
  else:require(5000<=data['elapsed_ms']<15000 and failed['error'],'handshake timeout changed')
  if repeated:require(data['cancellation_calls']>=2,'repeated cancel not exercised')
  require(data['manual_selection'] is expired,'fresh/manual admission differs')
  if expired:require(data['expiry_wait_ms']>=33000 and data['closed_samples']>=100,'real staged expiry not observed')
  if scenario=='cancel-expiry-source':require(data['source_handoff_wait_ms']>=31000,'real source expiry not awaited')
  require(before['selected_character']==continued['selected_character']==reentered['selected_character'],'character changed')
  require(continued['phase']=='character_selected' and continued['selected_source']=='測試乙'
   and continued['inventory']['owner']['connection']==data['reentered_epoch'],'recovery source/authority differs')
  require(reentered['phase']=='battle_pending' and reentered['world']['map_id']==500
   and reentered['world']['generation']>before['world']['generation'],'world did not retire/reenter')
  own=reentered['world']['actors'][str(reentered['world']['status']['object_id'])]
  require(data['recovery_move']['before']==[own['x'],own['y']],'recovery move starts from another actor/position')
  observed=[x for x in trace if x['test_account']==account]
  old=next((x for x in observed if x['rental_town'] and x['phase']=='BATTLE' and x['stable_observed'] and isinstance(x['source_item_owner'],dict)
   and x['source_item_owner']['active'] and len(x['rental_inventory'])==count),None)
  require(old is not None,'initial active rental trace absent');old_owner=live_owner(old,'rental_inventory',account)
  new=[x for x in observed if x['session_identity']!=old['session_identity'] and x['phase']=='BATTLE' and x['rental_town'] and len(x['rental_inventory'])==count]
  moved=next((x for x in new if x['stable_observed'] and isinstance(x['source_item_owner'],dict) and x['source_item_owner']['active']
   and [x['x'],x['y']]==data['recovery_move']['after']),None);require(moved is not None,'authoritative recovery move absent')
  new_owner=live_owner(moved,'rental_inventory',account);require(new_owner['observer_owner_id']!=old_owner['observer_owner_id'],'old owner reused')
  selected=read(runtime/(role+'-continuation-observed.json'))
  require(selected in observed and selected['session_identity']==moved['session_identity'] and selected['phase']=='LOBBY'
   and selected['native_inventory'],'acknowledged continuation trace absent/different session');live_owner(selected,'native_inventory',account)
  guard_roles.append(dict(account=account,old_owner_id=old_owner['observer_owner_id'],new_owner_id=new_owner['observer_owner_id'],
   old_session_id=old['session_identity'],new_session_id=moved['session_identity'],selected_observation=selected))
  request=(runtime/(role+'-continuation-observe-request')).read_text('utf-8')
  require(request==account+'\n'+selected['character']+'\n測試乙','continuation observation request differs')
  for session in [old['session_identity'],moved['session_identity']]:
   last=[x for x in trace if x['session_identity']==session][-1];require(retired(last),'disconnected source owner not observed retired')
  old_items=by_id(old['rental_inventory']);granted=by_id(selected['native_inventory']);fresh=by_id(moved['rental_inventory'])
  require(len(old_items)==len(fresh)==count and set(old_items).isdisjoint(fresh),'fresh rental IDs not regenerated')
  require(without_ids(old_items.values())==without_ids(fresh.values()),'complete fresh rental item fields changed')
  require(granted==by_id(moved['native_inventory']),'native inventory changed during reentry/move')
  attribute_states={key:observed_attributes(row[inventory+'_state']) for key,row,inventory in [
   ('old',old,'rental_inventory'),('selected',selected,'native_inventory'),
   ('moved_native',moved,'native_inventory'),('fresh',moved,'rental_inventory')]}
  for key,rows in [('old',old_items),('selected',granted),('moved_native',granted),('fresh',fresh)]:
   require(set(attribute_states[key])==set(rows),'native attribute IDs differ from actual inventory: '+key)
  attribute_adoption=None
  client=continued['inventory']['items'];require(set(map(int,client))==set(granted),'client/server recovered item IDs differ')
  require(all(client[str(oid)]['count']==row['count'] for oid,row in granted.items()),'client/server recovered item quantities differ')
  staged=[(x['observed_now_nanos'],v) for x in stages for v in x['staged'] if set(v['inventory_ids'])==set(old_items)]
  require(staged and len({v['expires_at_nanos'] for _,v in staged})==1,'original staged inventory/expiry absent')
  start,first=staged[0];require(not first['expired'] and 0<first['expires_at_nanos']-start<=30_000_000_000,'stage TTL changed')
  if expired:
   require(any(v['expired'] and now>=v['expires_at_nanos'] for now,v in staged),'passive natural stage expiry absent')
   require(set(old_items).isdisjoint(granted),'expired staged item IDs regranted')
  else:
   require(all(not v['expired'] and now<v['expires_at_nanos'] for now,v in staged),'stage unexpectedly expired')
   require(granted==old_items,'complete staged item fields changed on adoption')
   attribute_adoption=compare_staged_adoption(old['rental_inventory_state'],selected['native_inventory_state'])
  old_i,old_wire=consistent_batch(batches,set(old_items),-1,teleports[slot])
  new_i,new_wire=consistent_batch(batches,set(granted),teleports[slot],len(frames),True)
  fresh_i,fresh_wire=consistent_batch(batches,set(fresh),new_i,len(frames))
  require(wire_signature(old_wire.values())==wire_signature(fresh_wire.values()),'new rental wire fields/details changed')
  if not expired:require(old_wire==new_wire,'staged wire fields/details changed')
  else:require(all(set(old_items).isdisjoint(rows) for i,rows in batches if i>new_i),'expired IDs appeared after fresh admission')
  checks.append(dict(role=role,account=account,source='測試乙',rental_items=count,observed_item_field_names=sorted(ITEM_FIELDS),
   owner_source_qualified_character=source_identity('測試乙',selected['character']),
   owner_source_qualified_account=source_identity('測試乙',account),
   native_attribute_observer_kind=p['native_item_attribute_observer_kind'],permanent_attribute_fields=sorted(PERMANENT_FIELDS),
   diagnostic_attribute_fields=sorted(DIAGNOSTIC_FIELDS),native_attribute_adoption=attribute_adoption,
   nondefault_attribute_coverage={key:nondefault_coverage(value.values()) for key,value in attribute_states.items()},
   expired_native_IDs_not_regranted=expired,diagnostics_compared_as_invariants=False,ability_recomputation_requested=False,
   full_attribute_matrix_qualified=False,
   observed_old_owner=old_owner['observer_owner_id'],observed_new_owner=new_owner['observer_owner_id'],
   staged_expired=expired,natural_stage_wait_ms=data['expiry_wait_ms'],natural_source_wait_ms=data['source_handoff_wait_ms'],
   wire_rows=[old_i,new_i,fresh_i],role_result_sha256=sha(runtime/(role+'-failure.json'))))
 guard_checks=verify_owner_guards(jsonlines(runtime/'owner-guard-trace.jsonl'),trace,guard_roles,source_identity)
 mutation_trace=owned(runtime/'item-mutation-denial-trace.jsonl',runtime)
 require(mutation_trace.stat().st_size<=67_108_864,'retired mutation trace exceeds reviewed readback bound')
 morph_trace=owned(runtime/'item-morph-denial-trace.jsonl',runtime)
 require(morph_trace.stat().st_size<=67_108_864,'retired morph trace exceeds reviewed readback bound')
 morph_checks=verify_retired_morphs(jsonlines(morph_trace),jsonlines(mutation_trace),trace,guard_roles,source_identity)
 mutation_checks=morph_checks['core_mutations']
 require(morph_checks['actual_morph_attempts']==p['retired_item_morph_attempts'] and morph_checks['aggregate_retired_setter_attempts']==p['retired_item_aggregate_attempts'],'actual morph/aggregate counts differ from plan')
 require(mutation_checks['actual_mutation_attempts']==p['retired_item_mutation_attempts'],'actual retired mutation count differs from plan')
 check_pins(e['evidence_pins']);validate_plan(e['plan'])
 return dict(status='qualified_selected_E149_recovery_case_observations_and_wire',scenario=scenario,profile=p['profile'],
  execution=str(execution_path),execution_sha256=sha(execution_path),plan_sha256=e['plan_sha256'],controller_sha256=image['product_sha256'],
  source_manifest_sha256=p['source_manifest_sha256'],effective_overlay_files=p['effective_overlay_files'],formal_target_sha256=FORMAL,
  client_source_binding=p['client_source_binding'],fixture_source_binding=p['fixture_source_binding'],fixture_source_binding_sha256=p['fixture_source_binding_sha256'],
  fixture_qualification=p['fixture_qualification'],fixture_qualification_sha256=p['fixture_qualification_sha256'],
  owner_identity_contract=p['owner_identity_contract'],owner_identity_contract_sha256=p['owner_identity_contract_sha256'],
  owner_guard_probe_kind=p['owner_guard_probe_kind'],owner_guard_trace_sha256=sha(runtime/'owner-guard-trace.jsonl'),owner_guard_checks=guard_checks,
  retired_item_mutation_probe_kind=p['retired_item_mutation_probe_kind'],retired_item_mutation_trace_sha256=sha(runtime/'item-mutation-denial-trace.jsonl'),
  retired_item_mutation_checks=mutation_checks,
  retired_item_morph_probe_kind=p['retired_item_morph_probe_kind'],retired_item_morph_trace_sha256=sha(morph_trace),retired_item_morph_checks=morph_checks,
  capture_sha256=sha(captures[0]),checks=checks,original_processes_terminal=True,terminal_OS_observation=terminal_observation,
  static_SQL_access=True,player_SQL_access=False,new_test_executions_by_reader=0,GUI_verified=False,
  full_attribute_matrix_qualified=False,full_Main_process_qualified=False,full_native_owner_qualified=False,whole_goal_complete=False,
  limitations=['Only this selected case, observed owner bindings/retirement and inventory/wire continuity are covered',
   'Stable repeated reads are not atomic gameplay authority grants',
   'CaptureLog has no per-session wire ID; source request counts/values are aggregate, combined with per-account server/client snapshots',
   'Eight permanent native fields are compared only on unexpired same-ID staged adoption; cached abilities and fourteen diagnostics are observed without recomputation',
   'Fresh/expired inventories are not assumed to retain timer timestamps/generations; nondefault observations alone do not qualify expiry/mutation-denial/effect matrices',
   'Owner guard receipts,32 core and8 morph attempts cover only ten pinned BoundItem entries on these four retired admissions',
   'Extended attributes, isolated account/source branches, staged/detached denials, natural timer expiry, full G06/library/GUI/Main remain outside this qualification'],
  validator_sha256=sha(__file__),recorded_utc=datetime.now(timezone.utc).isoformat())

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('execution',type=Path);a=ap.parse_args()
 result=verify_execution(a.execution);output=owned(a.execution).with_name('qualified-selected-case.json');write_new(output,result)
 print(json.dumps(dict(qualification=str(output),scenario=result['scenario'],profile=result['profile'],new_test_executions=0,whole_goal_complete=False),ensure_ascii=False))
if __name__=='__main__':main()
