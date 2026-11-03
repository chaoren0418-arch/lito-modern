"""Independently read compiler-only evidence, then checkpoint eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-item-spells-metadata-compile');c=read(run/'cohort.json')
 require(c['status']=='metadata_compilers_complete_no_tests' and c['owned_process'] is None,'compiler pipeline not terminal')
 require(c['source_manifest_sha256']==SOURCE and c['formal_target_sha256']==FORMAL and c['tests_executed']==0 and not c['PEs_executed'] and not c['PEs_linked'],'scope differs')
 expected=[(p,k) for p in ['Debug','Release'] for k in ['library','library-test','blind20','slow29','advance79','item_spells']]
 require([(x['profile'],x['kind']) for x in c['compiler_runs']]==expected and all(x['exit_code']==0 for x in c['compiler_runs']),'twelve exact compiler successes required')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'toolchain changed')
 cp=Path(c['candidate']);candidate=read(cp);require(sha(cp)==c['candidate_sha256'],'candidate changed');check_pins({str(cp.parent/n):h for n,h in candidate['candidate_files'].items()})
 require(len(candidate['candidate_files'])==10 and len(candidate['changed_runtime_files'])==6,'candidate count differs')
 require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');contract=read(candidate['contract']);require(set(contract['original_definitions'])=={'12','21'} and contract['original_definitions']['12']['effectGfx']==747 and contract['original_definitions']['21']['effectGfx']==748,'formal definition differs')
 predecessor=Path(candidate['predecessor_advance79_candidate']);require(sha(predecessor)==candidate['predecessor_advance79_candidate_sha256'],'advance predecessor changed')
 previous=read(predecessor)
 for n in ['LiTo381-Modern/tests/native_blind.rs','LiTo381-Modern/tests/native_slow29.rs','LiTo381-Modern/tests/native_advance79.rs']:
  require(candidate['candidate_files'][n]==previous['candidate_files'][n],'retained blind/slow file changed')
 original=Path(c['original_mod']).read_text('utf-8-sig');mod=original
 for n,row in c['original_runtime_sources'].items():
  require(sha(row['path'])==row['sha256'] and sha(row['original_candidate_or_normal'])==candidate['base_files'][n] and mod.count(row['path'])==1,'original runtime path changed')
  destination=cp.parent/n
  if n in c['paired_runtime_sources']:
   pair=c['paired_runtime_sources'][n];destination=Path(pair['path']);require(sha(destination)==pair['sha256'] and sha(pair['source'])==pair['source_sha256']==candidate['candidate_files'][n],'paired runtime changed')
   source=(cp.parent/n).read_text('utf-8-sig');old_graph=Path(row['path']).read_text('utf-8-sig')
   for rewrite in pair['rewrites']:
    require(source.count(rewrite['original_attribute'])==1 and old_graph.count(rewrite['compiled_attribute'])==1,'nested path rewrite not unique')
    source=source.replace(rewrite['original_attribute'],rewrite['compiled_attribute']);old_graph=old_graph.replace(rewrite['compiled_attribute'],rewrite['original_attribute'])
    require(Path(rewrite['resolved_path']).is_file(),'original nested module absent')
   require(source==destination.read_text('utf-8-sig') and old_graph==Path(row['original_candidate_or_normal']).read_text('utf-8-sig'),'paired runtime differs beyond original nested path attributes')
  else:require(row['sha256']==candidate['base_files'][n],'nonpaired base bytes differ')
  mod=mod.replace(row['path'],destination.as_posix())
 require(mod==Path(c['generated_mod']).read_text('utf-8-sig') and sorted(c['runtime_overrides'])==sorted(candidate['changed_runtime_files']) and len(re.findall(r'#\[path = r#"',mod))==362 and len(c['paired_runtime_sources'])==2 and sum(len(p['rewrites']) for p in c['paired_runtime_sources'].values())==6,'complete graph differs beyond six runtime paths')
 lib=Path(c['original_root']).read_text('utf-8-sig').replace(c['original_mod'],Path(c['generated_mod']).as_posix());require(lib==Path(c['generated_root']).read_text('utf-8-sig'),'root wrapper differs')
 fixture=P/'tests/fixtures/native-world-current.json'
 for kind,n in [('blind20','LiTo381-Modern/tests/native_blind.rs'),('slow29','LiTo381-Modern/tests/native_slow29.rs'),('advance79','LiTo381-Modern/tests/native_advance79.rs'),('item_spells','LiTo381-Modern/tests/native_item_spells.rs')]:
  s=(cp.parent/n).read_text('utf-8-sig');needle='include_bytes!("fixtures/native-world-current.json")';require(s.count(needle)==1 and Path(c['paired_tests'][kind]).read_text('utf-8-sig')==s.replace(needle,'include_bytes!(r#"'+fixture.as_posix()+'"#)'),'paired QA differs beyond fixture path')
  require(len(re.findall(r'#\[test\]\s*fn\s+\w+',s))==(8 if kind=='item_spells' else 6),'QA function count differs')
 plan=read(c['original_component_plan']);require(sha(c['original_component_plan'])==c['original_component_plan_sha256'],'original plan changed')
 libraries={p:next(x['product'] for x in c['compiler_runs'] if x['profile']==p and x['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['finished_utc'] and not row.get('exact_original_compiler_cleanup'),'compiler did not finish normally');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-item-spells-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'output changed')
  command=list(plan['compiler_templates'][row['profile']]);kind=row['kind'];command[command.index(plan['roots']['candidate'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind=='library':command.remove('--test');command+=['--crate-type=rlib'];require(row['product_bytes']>0 and product.name.startswith('lib'),'library metadata absent')
  elif kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(row['command']==command and '--emit=metadata' in command and '--emit=link' not in command,'actual command differs')
 require(sha(contract['API_inspection'])==contract['API_inspection_sha256'],'original item spell API changed');api=read(contract['API_inspection']);check_pins(api['outputs']);require(all(x['exit_code']==0 for x in api['commands']),'original API inspection failed')
 for key in ['original_ClientSession','original_catalog','original_list']:require(sha(contract[key])==contract[key+'_sha256'],'original source metadata changed')
 require(contract['original_list_sha256']=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127' and [x['graphics'] for x in contract['effects']]==[747,748],'original sprite source differs')
 require(all(x['mpCost']==20 and x['reuseDelayMillis']==10 and x['durationSeconds']==1800 and x['invalidItemMessage']==79 and x['successMessage']==161 and x['expirationMessage']==308 for x in contract['original_definitions'].values()),'cost/effect/rejection contract differs')
 for registry,name in [('cancel-recovery','native_login.rs'),('keyboard','native_login_window.rs')]:
  n='LiTo381-Modern/src/modern_lito/'+name;source=Path(candidate['extra_graph_base_sources'][n]);ref=read(R/'工作佇列.json')['current_isolated_development']['candidates'][registry];require(sha(ref['path'])==ref['sha256'] and source==Path(ref['path']).parent/n and sha(source)==candidate['base_files'][n],'current original registry predecessor differs')
 old_run=V/'native-E149-item-spells-metadata-compile/20261024-015623-685195';failed=read(old_run/'cohort.json');require(failed['status']=='compiler_failure_preserved' and [x['exit_code'] for x in failed['compiler_runs']]==[0,0,0,0,0,1],'original failed QA compiler result changed')
 for x in failed['compiler_runs']:check_pins(x['logs'])
 require('private' in Path(next(n for n in failed['compiler_runs'][-1]['logs'] if n.endswith('stderr.log'))).read_text('utf-8'),'original private-module diagnostic absent')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'another workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/target changed')
 inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal inputs changed')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['inspect-native-E149-item-spell-contract.py','prepare-native-E149-item-spells-candidate.py','compile-native-E149-item-spells-candidate.py','qualify-native-E149-item-spells-development.py']]
 for path in tools:compile(ast.parse(path.read_text('utf-8'),filename=str(path)),str(path),'exec')
 rows=[c['pipeline'],failed['pipeline'],*[x['original_process'] for x in c['compiler_runs']],*[x['original_process'] for x in failed['compiler_runs']]];ids=','.join(str(x['pid']) for x in rows)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in rows:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler/pipeline still live')
 out=owned(V/'native-E149-item-spells-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(observed)
 record=dict(status='unselected_inventory_spell_UI_controller_and_effects_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,successful_compiler_exit_codes=[0]*12,
  metadata_products={x['profile']+'-'+x['kind']:dict(path=x['product'],sha256=x['product_sha256'],bytes=x['product_bytes']) for x in c['compiler_runs']},resolved_module_path_attributes=362,runtime_override_files=c['runtime_overrides'],paired_runtime_sources=c['paired_runtime_sources'],original_nested_paths_preserved=6,new_tests_written=8,retained_skill_tests_written=18,original_definitions=contract['original_definitions'],original_effect_frames={'747':19,'748':20},original_sounds={'747':113,'748':137},AST_compiled_tools=4,
  original_failed_cohort=str(old_run/'cohort.json'),original_failed_cohort_sha256=sha(old_run/'cohort.json'),original_failed_tool_chunk='b78f7e',original_failed_actual_exit_code=1,original_failed_compiler_exit_codes=[0,0,0,0,0,1],
  source_pins={str(p):sha(p) for p in tools+[T/'templates/native_item_spells.rs',T/'templates/native_item_spell_target.rs',T/'native-E149-item-spells-development.md']},original_compiler_identities_terminal=True,registered_overlay_selected=False,normal_sources_unchanged=True,normal_PEs_unchanged=True,current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],tests_executed=0,PEs_executed=False,PEs_linked=False,new_sessions=0,new_casts=0,desktop_inputs=0,WAV_playbacks=0,production_applied=False,GUI_verified=False,full_G06_qualified=False,whole_goal_complete=False,
  limitations=['metadata only; original resource frames/audio, actual E149 cast/item state and GUI all pending','six-runtime extension remains unselected; original13 registry and G06 source/image qualifications unchanged','original failed QA compiler and preparation guards preserved; no success transferred across changed source'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-failed-cohort.json':old_run/'cohort.json','original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 raw.update({Path(n).name:Path(n) for n in record['source_pins']})
 for x in c['compiler_runs']:
  for n in x['logs']:raw[x['profile']+'-'+x['kind']+'/'+Path(n).name]=Path(n)
 for n in candidate['candidate_files']:raw['candidate/'+n]=cp.parent/n
 failure_dir=V/'native-E149-item-spells-preparation-failures/20261024-015244-131927'
 for p in failure_dir.iterdir():
  if p.is_file():raw['preparation-failures/'+p.name]=p
 graph_failure=V/'native-E149-item-spells-metadata-compile/20261024-015453-009908'
 for p in graph_failure.iterdir():
  if p.is_file():raw['graph-preparation-failure/'+p.name]=p
 for row in failed['compiler_runs']:
  for n in row['logs']:raw['original-failed/'+row['profile']+'-'+row['kind']+'/'+Path(n).name]=Path(n)
 old_cp=Path(failed['candidate']);old_c=read(old_cp)
 for n in old_c['candidate_files']:raw['original-failed-candidate/'+n]=old_cp.parent/n
 raw['original-failed-compiler.py']=old_run/'compiler.py'
 raw['current-compiler.py']=run/'compiler.py'
 for n,pair in c['paired_runtime_sources'].items():raw['compiled-runtime/'+Path(n).name]=Path(pair['path'])
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in raw.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work='新增未選入#12武器／#21防具的背包施法全程候選：typed inventory target綁owner／完整Item／revision／map／InputStamp，原C14物品ID分域、明確row選擇才授權，refresh不重授權，更新／ABA／傳送／中斷即失效；背包世界頁顯示選單及施法鈕並避開清單，controller送前重驗與互斥gate，79拒絕只限item Pending。MP／資格／物品效果／1800秒由E149決定；747原19frame/Sound113及748原20frame/Sound137接既有finite bank。完整362模組覆蓋6Runtime且保留6個nested模組參照，兩版library／library-test／四skill整合共12metadata退出0；新增8案及保留18案全未跑。原QA private module失敗及準備路徑錯誤封存，未改原結果。候選未選入／套用，無PE連結／執行或GUI；正常1049輸入／兩EXE、13檔registry、G06綁定／16案不變。下一步獨立來源審查／合併前提與剩餘玩法，恢復後先原G06，再驗物品技能／原資源／低MP／錯物品／舊選擇／取消／期限及GUI；新當版完整library兩版／Release／完整GUI仍待，測試／桌面暫緩維持。'
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_native_item_spells_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='native_inventory_spell_typed_target_and_GUI_source_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items()),'concurrent writer');check_pins(c['normal_product_pins'])
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),metadata_successes=12,tests_executed=0,selected=False,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();publish(a.run,a.terminal_chunk)
