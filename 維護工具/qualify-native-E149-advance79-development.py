"""Independently read compiler-only evidence, then checkpoint eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-advance79-metadata-compile');c=read(run/'cohort.json')
 require(c['status']=='metadata_compilers_complete_no_tests' and c['owned_process'] is None,'compiler pipeline not terminal')
 require(c['source_manifest_sha256']==SOURCE and c['formal_target_sha256']==FORMAL and c['tests_executed']==0 and not c['PEs_executed'] and not c['PEs_linked'],'scope differs')
 expected=[(p,k) for p in ['Debug','Release'] for k in ['library','library-test','blind20','slow29','advance79']]
 require([(x['profile'],x['kind']) for x in c['compiler_runs']]==expected and all(x['exit_code']==0 for x in c['compiler_runs']),'ten exact compiler successes required')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'toolchain changed')
 cp=Path(c['candidate']);candidate=read(cp);require(sha(cp)==c['candidate_sha256'],'candidate changed');check_pins({str(cp.parent/n):h for n,h in candidate['candidate_files'].items()})
 require(len(candidate['candidate_files'])==7 and len(candidate['changed_runtime_files'])==4,'candidate count differs')
 require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');contract=read(candidate['contract']);require(contract['actual_E149_definition']['skillId']==79 and contract['actual_E149_definition']['effectGfx']==3935,'formal definition differs')
 predecessor=Path(candidate['predecessor_slow29_candidate']);require(sha(predecessor)==candidate['predecessor_slow29_candidate_sha256'],'slow predecessor changed')
 previous=read(predecessor)
 for n in ['LiTo381-Modern/tests/native_blind.rs','LiTo381-Modern/tests/native_slow29.rs','LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs']:
  require(candidate['candidate_files'][n]==previous['candidate_files'][n],'retained blind/slow file changed')
 original=Path(c['original_mod']).read_text('utf-8-sig');mod=original
 for n,row in c['original_runtime_sources'].items():
  require(sha(row['path'])==row['sha256']==candidate['base_files'][n] and mod.count(row['path'])==1,'original runtime path changed')
  mod=mod.replace(row['path'],(cp.parent/n).as_posix())
 require(mod==Path(c['generated_mod']).read_text('utf-8-sig') and sorted(c['runtime_overrides'])==sorted(candidate['changed_runtime_files']) and len(re.findall(r'#\[path = r#"',mod))==362,'complete graph differs beyond four runtime paths')
 lib=Path(c['original_root']).read_text('utf-8-sig').replace(c['original_mod'],Path(c['generated_mod']).as_posix());require(lib==Path(c['generated_root']).read_text('utf-8-sig'),'root wrapper differs')
 fixture=P/'tests/fixtures/native-world-current.json'
 for kind,n in [('blind20','LiTo381-Modern/tests/native_blind.rs'),('slow29','LiTo381-Modern/tests/native_slow29.rs'),('advance79','LiTo381-Modern/tests/native_advance79.rs')]:
  s=(cp.parent/n).read_text('utf-8-sig');needle='include_bytes!("fixtures/native-world-current.json")';require(s.count(needle)==1 and Path(c['paired_tests'][kind]).read_text('utf-8-sig')==s.replace(needle,'include_bytes!(r#"'+fixture.as_posix()+'"#)'),'paired QA differs beyond fixture path')
  require(len(re.findall(r'#\[test\]\s*fn\s+\w+',s))==6,'six functions per QA required')
 plan=read(c['original_component_plan']);require(sha(c['original_component_plan'])==c['original_component_plan_sha256'],'original plan changed')
 libraries={p:next(x['product'] for x in c['compiler_runs'] if x['profile']==p and x['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['finished_utc'] and not row.get('exact_original_compiler_cleanup'),'compiler did not finish normally');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-advance79-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'output changed')
  command=list(plan['compiler_templates'][row['profile']]);kind=row['kind'];command[command.index(plan['roots']['candidate'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind=='library':command.remove('--test');command+=['--crate-type=rlib'];require(row['product_bytes']>0 and product.name.startswith('lib'),'library metadata absent')
  elif kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(row['command']==command and '--emit=metadata' in command and '--emit=link' not in command,'actual command differs')
 audio_path=Path(contract['audio_inspection']);require(sha(audio_path)==contract['audio_inspection_sha256'],'audio proof changed');audio=read(audio_path)
 require(sha(audio['original_PE'])==audio['original_PE_sha256'] and sha(audio['sound']['path'])==audio['sound']['sha256'],'readonly primary changed')
 require(audio['PE_executions']==audio['WAV_playbacks']==audio['tests_executed']==0 and audio['sound']['format_tag']==1,'static proof scope differs')
 for n,row in audio['code_ranges'].items():require(sha(audio_path.parent/(n+'.bytes'))==row['sha256'],'source bytes changed')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'another workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/target changed')
 inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal inputs changed')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['inspect-native-advance79-audio-source.py','prepare-native-E149-advance79-candidate.py','compile-native-E149-advance79-candidate.py','qualify-native-E149-advance79-development.py']]
 for path in tools:compile(ast.parse(path.read_text('utf-8'),filename=str(path)),str(path),'exec')
 rows=[c['pipeline'],*[x['original_process'] for x in c['compiler_runs']]];ids=','.join(str(x['pid']) for x in rows)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in rows:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler/pipeline still live')
 out=owned(V/'native-E149-advance79-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(observed)
 record=dict(status='unselected_advance79_request_menu_selector_audio_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,successful_compiler_exit_codes=[0]*10,
  metadata_products={x['profile']+'-'+x['kind']:dict(path=x['product'],sha256=x['product_sha256'],bytes=x['product_bytes']) for x in c['compiler_runs']},resolved_module_path_attributes=362,runtime_override_files=c['runtime_overrides'],new_tests_written=6,retained_blind20_tests_written=6,retained_slow29_tests_written=6,actual_E149_definition=contract['actual_E149_definition'],original_effect3935_frames=19,
  audio_source_correction=contract['correction'],audio_inspection=str(audio_path),audio_inspection_sha256=sha(audio_path),AST_compiled_tools=4,source_pins={str(p):sha(p) for p in tools+[T/'templates/native_advance79.rs',T/'native-E149-advance79-development.md']},
  original_compiler_identities_terminal=True,registered_overlay_selected=False,normal_sources_unchanged=True,normal_PEs_unchanged=True,current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],tests_executed=0,PEs_executed=False,PEs_linked=False,new_sessions=0,new_casts=0,desktop_inputs=0,WAV_playbacks=0,production_applied=False,GUI_verified=False,full_G06_qualified=False,whole_goal_complete=False,
  original_PE_path_guard_failure=str(V/'native-advance79-original-audio-source/path-guard-failure-20261024-005528-768383/failure.json'),original_PE_path_guard_failure_sha256=sha(V/'native-advance79-original-audio-source/path-guard-failure-20261024-005528-768383/failure.json'),limitations=['metadata only; all cases, resource playback, actual E149 skill, sound device and GUI pending','seven-file extension remains unselected; original13 registry and G06 source qualifications unchanged'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'audio-inspection.json':audio_path,'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 raw.update({Path(n).name:Path(n) for n in record['source_pins']})
 for x in c['compiler_runs']:
  for n in x['logs']:raw[x['profile']+'-'+x['kind']+'/'+Path(n).name]=Path(n)
 for n in candidate['candidate_files']:raw['candidate/'+n]=cp.parent/n
 failure_path=Path(record['original_PE_path_guard_failure']);raw['original-path-guard/failure.json']=failure_path;raw['original-path-guard/inspector-failed.py']=failure_path.with_name('inspector-failed.py')
 for p in audio_path.parent.iterdir():
  if p.is_file():raw['original-code/'+p.name]=p
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in raw.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work='新增未選入的#79靈魂昇華候選：沿#20／#29補C14 09/06、自用或可見目標、原畫面菱形、已學選單及3935原19有限影格。更正舊報告：Effect(1356)是原<可重疊音效，非子sprite；固定原OEP及1356 WAV只讀核對，沿既有PreparedCue退役與一次性路徑。MP／最大HPMP／1200秒均由E149權威回覆；新六案加保留十二案全部未跑。完整362模組只覆蓋4Runtime，兩版各library／library-test／#20／#29／#79共10項metadata退出0；無PE連結或執行、無聲音／GUI。正常1049輸入／兩EXE、現行13檔registry、G06綁定與16案均不變；測試及桌面繼續暫緩。恢復後依原G06計畫，核對技能候選再合併，最後新當版完整library兩版／Release回歸／完整GUI；G01–G12未完成。'
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_native_advance79_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='native_advance79_unselected_request_selector_audio_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
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
 print(json.dumps(dict(evidence=str(out/'evidence.json'),metadata_successes=10,tests_executed=0,selected=False,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();publish(a.run,a.terminal_chunk)
