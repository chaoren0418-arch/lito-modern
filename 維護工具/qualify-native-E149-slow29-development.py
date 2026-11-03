"""Qualify eight actual metadata results and publish the unselected extension."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-slow29-metadata-compile');c=read(run/'cohort.json')
 require(c['status']=='metadata_compilers_complete_no_tests' and c['owned_process'] is None,'metadata pipeline not terminal')
 require(c['source_manifest_sha256']==SOURCE and c['formal_target_sha256']==FORMAL and c['tests_executed']==0
  and c['PEs_executed'] is False and c['PEs_linked'] is False,'metadata source/scope differs')
 expected=[(profile,kind) for profile in ['Debug','Release'] for kind in ['library','library-test','blind20','slow29']]
 require([(row['profile'],row['kind']) for row in c['compiler_runs']]==expected and all(row['exit_code']==0 for row in c['compiler_runs']),'eight exact successful compiler results required')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins'])
 require(all(sha(name)==h for name,h in c['external_toolchain_dependency_pins'].items()),'read-only toolchain changed')
 old_path=Path(c['previous_original_cohort']);old=read(old_path);require(sha(old_path)==c['previous_original_cohort_sha256'],'original failed cohort changed')
 require(c['reused_original_successful_compilers']==2 and c['new_compilers_started']==6 and old['compiler_runs'][-1]==c['previous_failed_compiler']
  and c['previous_failed_compiler']['exit_code']==1,'original failure/reuse scope differs')
 for index,row in enumerate(c['compiler_runs'][:2]):
  require({key:row[key] for key in old['compiler_runs'][index]}==old['compiler_runs'][index],'reused successful result was relabeled')
 copy=c['dependency_library_copy'];require(sha(copy['copy'])==copy['sha256']==sha(copy['original'])==old['compiler_runs'][0]['product_sha256'],'reused library copy changed')
 require(sha(c['previous_control_snapshot'])==c['previous_control_snapshot_sha256'],'original compiler snapshot changed')
 check_pins(c['previous_failed_compiler']['logs'])
 cp=Path(c['candidate']);candidate=read(cp);require(sha(cp)==c['candidate_sha256'],'candidate changed');check_pins({str(cp.parent/name):h for name,h in candidate['candidate_files'].items()})
 require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');contract=read(candidate['contract'])
 require(contract['actual_E149_definition']['skillId']==29 and contract['actual_E149_definition']['maximumRange']==8
  and contract['actual_E149_definition']['reuseDelayMillis']==500,'slow contract differs')
 original_mod=Path(c['original_mod']).read_text('utf-8-sig');mod=original_mod
 for name,source in c['original_runtime_sources'].items():
  require(sha(source['path'])==source['sha256']==candidate['base_files'][name],'original runtime copy changed')
  require(mod.count(source['path'])==1,'original graph path not unique');mod=mod.replace(source['path'],(cp.parent/name).as_posix())
 require(Path(c['generated_mod']).read_text('utf-8-sig')==mod and len(c['runtime_overrides'])==3,'complete graph was changed beyond three runtime paths')
 lib=Path(c['original_root']).read_text('utf-8-sig').replace(str(Path(c['original_mod'])),Path(c['generated_mod']).as_posix())
 require(Path(c['generated_root']).read_text('utf-8-sig')==lib,'library graph wrapper differs')
 fixture=P/'tests/fixtures/native-world-current.json'
 for kind,name in [('blind20','LiTo381-Modern/tests/native_blind.rs'),('slow29','LiTo381-Modern/tests/native_slow29.rs')]:
  source=(cp.parent/name).read_text('utf-8-sig');needle='include_bytes!("fixtures/native-world-current.json")'
  require(source.count(needle)==1 and Path(c['paired_tests'][kind]).read_text('utf-8-sig')==source.replace(needle,'include_bytes!(r#"'+fixture.as_posix()+'"#)'), 'paired QA differs beyond fixture path')
  require(len(re.findall(r'#\[test\]\s*fn\s+\w+',source))==6,'six original/new test functions required')
 plan=read(c['original_component_plan']);require(sha(c['original_component_plan'])==c['original_component_plan_sha256'],'original command plan changed')
 libraries={profile:next(row for row in c['compiler_runs'] if row['profile']==profile and row['kind']=='library') for profile in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(not row.get('exact_original_compiler_cleanup') and row['finished_utc'],'compiler did not finish normally');check_pins(row['logs'])
  output=owned(row['product'],P/'target/current/verification/native-E149-slow29-metadata');require(sha(output)==row['product_sha256'] and output.stat().st_size==row['product_bytes'],'metadata product changed')
  command=list(plan['compiler_templates'][row['profile']]);kind=row['kind']
  command[command.index(plan['roots']['candidate'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind]
  command[command.index('-o')+1]=row['product']
  if kind=='library':command.remove('--test');command+=['--crate-type=rlib'];require(row['product_bytes']>0,'library metadata absent')
  elif kind in c['paired_tests']:
   library=copy['copy'] if row['profile']=='Debug' else libraries['Release']['product']
   command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+library]
  require(row['command']==command and '--emit=metadata' in command and '--emit=link' not in command,'actual metadata command differs')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'another workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/target changed')
 require(w['current_isolated_development']['candidates']['blind20']['path']==candidate['predecessor_blind20_candidate']
  and w['current_isolated_development']['candidates']['blind20']['sha256']==candidate['predecessor_blind20_candidate_sha256'],'selected #20 registry changed')
 inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/name)==h for name,h in inputs.items()),'normal inputs changed')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/name)==h for name,h in before.items()),'checkpoint changed')
 tools=[T/name for name in ['inspect-native-E149-slow29-advance79.py','prepare-native-E149-slow29-candidate.py','compile-native-E149-slow29-candidate.py',
  'continue-native-E149-slow29-metadata.py','qualify-native-E149-slow29-development.py']]
 for path in tools:compile(ast.parse(path.read_text('utf-8'),filename=str(path)),str(path),'exec')
 rows=[c['pipeline'],old['pipeline'],*[row['original_process'] for row in c['compiler_runs']],c['previous_failed_compiler']['original_process']]
 ids=','.join(str(row['pid']) for row in rows)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW)
 live=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in rows:require(not any(item['ProcessId']==row['pid'] and abs((datetime.fromisoformat(item['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for item in live),'same original compiler/pipeline remains live')
 out=owned(V/'native-E149-slow29-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(observed)
 record=dict(status='unselected_slow29_request_menu_and_finite_frames_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),
  observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,successful_compiler_exit_codes=[0]*8,reused_original_successful_compilers=2,new_compilers_started=6,
  original_failed_cohort=str(old_path),original_failed_cohort_sha256=sha(old_path),original_failed_compiler_exit_code=1,preparation_failure=c['preparation_failure'],
  metadata_products={row['profile']+'-'+row['kind']:dict(path=row['product'],sha256=row['product_sha256'],bytes=row['product_bytes']) for row in c['compiler_runs']},
  full_original_module_path_inventory_preserved=True,resolved_module_path_attributes=len(re.findall(r'#\[path = r#"',mod)),runtime_override_files=c['runtime_overrides'],
  new_tests_written=6,retained_blind20_tests_written=6,original_effect752_frames=16,original_effect752_sound_events=0,
  actual_E149_definition=contract['actual_E149_definition'],advance79_remaining=contract['advance79_remaining'],
  AST_compiled_tools=5,source_pins={str(path):sha(path) for path in tools+[T/'templates/native_slow29.rs',T/'native-E149-slow29-development.md']},
  original_compiler_identities_terminal=True,registered_overlay_selected=False,normal_sources_unchanged=True,normal_PEs_unchanged=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],
  tests_executed=0,PEs_executed=False,PEs_linked=False,resource_or_graphics_probe_executed=False,new_sessions=0,new_casts=0,desktop_inputs=0,
  production_applied=False,GUI_verified=False,full_G06_qualified=False,whole_goal_complete=False,
  limitations=['Metadata compilation only; no case, SPR playback, runtime dispatch, material/probability, GUI or source-counterexample result is claimed',
   'Current selected #20/cancel/keyboard registry, G06 bindings/suite and all old PE qualifications remain scoped to their original bytes',
   '79 child effect/control semantics are still unimplemented; normal 38-ID recognition is unchanged'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw={Path(name).name:Path(name) for name in record['source_pins']};raw.update({'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json',
  'original-failed-cohort.json':old_path,'original-compiler.py':Path(c['previous_control_snapshot']),'candidate.json':cp,'contract.json':Path(candidate['contract']),
  'preparation-failure.json':Path(c['preparation_failure']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'})
 for row in c['compiler_runs']:
  for name in row['logs']:raw[row['profile']+'-'+row['kind']+'/'+Path(name).name]=Path(name)
 for name in c['previous_failed_compiler']['logs']:raw['original-failed-integration/'+Path(name).name]=Path(name)
 for name in candidate['candidate_files']:raw['candidate/'+name]=cp.parent/name
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name,path in raw.items():z.write(owned(path),name)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in raw.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work='新增未選入的原生#29緩速候選：沿#20補C14 03/04、另一個可見目標目前座標、八格與500ms；MP／材料／機率／64秒狀態由E149原server決定。新增精確752原16有限影格／兩輪子影格／無音效；六個新案均未跑。完整模組圖只換3Runtime，兩版library／library-test／原#20與新#29整合共8個成功metadata命令；沿原Debug兩項結果，只接續6個compiler。原路徑準備與metadata命名失敗保留，非產品通過。無PE連結／執行，正常1049輸入／兩EXE、現行13檔registry、G06綁定與16案均不變；#79的3935子特效1356與attr109尚待原繪製語意。測試／桌面繼續暫緩，下一步補完整技能與非預設狀態前提；恢復測試後先依G06原計畫，再核對新候選、合併重建與完整library兩版／Release／GUI，G01–G12未完成。'
 staged={}
 for name in before:
  path=R/name
  if path.suffix=='.json':
   value=read(path);value['current_native_slow29_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
   value['current_focus']=dict(item='native_slow29_unselected_request_and_finite_effect_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if name.endswith('next-validation-action.json'):value['next_action']=work
   staged[name]=encoded(value)
  else:
   _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs');staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name in before:z.write(R/name,name)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items()),'before archive differs')
 require(all(sha(R/name)==h for name,h in before.items()) and all(sha(R/name)==h for name,h in inputs.items()),'concurrent checkpoint/source writer');check_pins(c['normal_product_pins'])
 for name,blob in staged.items():(R/name).write_bytes(blob)
 require(all((R/name).read_bytes()==blob for name,blob in staged.items()),'progress readback differs')
 (out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={name:sha(R/name) for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),metadata_successes=8,new_compilers=6,reused_compilers=2,tests_executed=0,selected=False,eight_files_read_back=True),ensure_ascii=False))
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);args=ap.parse_args();publish(args.run,args.terminal_chunk)
if __name__=='__main__':main()
