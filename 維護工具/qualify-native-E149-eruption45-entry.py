"""Read back #45 request/menu source and 18 metadata commands, then checkpoint."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
CHECKPOINTS=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
NOTE='#45地裂術本機入口已補候選：C14 14 05 04＋角色ID／x/y共11bytes、其他可見活體／八格／reuse基值100ms與實際選單；沿Target與Owner、controller互斥／pending、滑鼠與T鍵守衛。原classfile BootstrapMethods綁定classic及實際repair020 battle篩選，不用同名方法猜caller；ResourceDatabase先CombatDatabase、再snapshot／CombatRules之Code鏈已釐清，未讀DB／執行getter，MP20等僅基值且由server決定。33檔／365module／原六nested不變，兩版library／library-test／七skill整合18metadata退出0，新10（九純＋一原資源opt-in）＋前175共185案全未跑。請求來源識別60／剩120，非技能驗收；已接S23 class7呈現沿前輪，child hurt相對gfx／guard、可選倍率／timer rollover仍待。正常1049輸入／所有PE、registry及G06 binding／16案640次不變，桌面／測試／播放暫緩。'
NEXT='繼續完成#45原type6 hurt／LT相對gfx與初始化、速度及長uptime界線，同時依當版#46／#47定義補剩餘入口與原資源呈現；不把60個來源入口當玩法完成。G06、完整library Debug／Release、Release與GUI依總報告，測試／桌面等使用者明確恢復。'

def coverage(source,previous):
 s=Path(source).read_text('utf-8-sig');b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];recognized=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):recognized.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'request parser differs');recognized.update(map(int,re.findall(r'\d+',m[1])))
 require(previous['recognized_count']==59 and recognized==set(previous['recognized_ids'])|{45} and len(recognized)==60,'request changes exceed45')
 result=dict(previous);result.update(request_source=str(source),request_source_sha256=sha(source),recognized_ids=sorted(recognized),recognized_count=60,missing_request_ids=sorted(set(previous['missing_request_ids'])-{45}),missing_request_count=120);return result

def validate(cohort):
 c=read(cohort);k=read(c['contract']);require(sha(c['contract'])==c['contract_sha256'] and k['base_source_manifest_sha256']==SOURCE and k['formal_target_sha256']==FORMAL,'identity differs');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'readonly input differs')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'];require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and [(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,kind) for p in ['Debug','Release'] for kind in kinds] and [r['exit_code'] for r in c['compiler_runs']]==[0]*18 and c['owned_process'] is None,'compiler cohort incomplete')
 require(c['tests_executed']==0 and not any(c[x] for x in ['PEs_linked','outputs_executed','whole_goal_complete']),'execution differs')
 old=read(c['predecessor_cohort']);require(sha(c['predecessor_cohort'])==c['predecessor_cohort_sha256'] and old['module_count']==365,'predecessor differs');require(sha(k['integration_predecessor_cohort'])==k['integration_predecessor_cohort_sha256'] and c['paired_tests']==read(k['integration_predecessor_cohort'])['paired_tests'],'seven integrations differ')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for name,row in k['runtime_overrides'].items():
  text=Path(row['predecessor']).read_text('utf-8-sig');require(sha(row['predecessor'])==row['predecessor_sha256'],'runtime predecessor differs')
  for a,b in row['changes']:require(text.count(a)==1,'runtime delta ambiguous');text=text.replace(a,b)
  if row['QA_template'] is not None:
   template=row['QA_template'];require(sha(template['path'])==template['sha256'],'QA template differs');qa=Path(template['path']).read_text('utf-8-sig');require(qa.count('#[test]')==10 and qa.count('#[ignore=')==1,'new cases differ');text+=qa
  require(text==Path(row['path']).read_text('utf-8-sig') and sha(row['path'])==row['sha256'],'runtime delta differs')
  needles=list(dict.fromkeys([row['predecessor'],Path(row['predecessor']).as_posix()]));found=[n for n in needles if modern.count(n)==1];require(len(found)==1 and sum(modern.count(n) for n in needles)==1,'runtime mapping differs');modern=modern.replace(found[0],Path(row['path']).as_posix())
 require(k['modules']=={} and modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==c['module_count']==365,'complete graph differs')
 root=Path(old['generated_root']).read_text('utf-8-sig');require(root.replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig'),'root differs');require(c['original_nested_runtime_pairs']==old['original_nested_runtime_pairs'] and sum(len(r['rewrites']) for r in c['original_nested_runtime_pairs'].values())==6,'nested pair differs')
 for pair in c['original_nested_runtime_pairs'].values():require(sha(pair['path'])==pair['sha256'] and sha(pair['source'])==pair['source_sha256'],'nested source differs')
 prior=read(k['predecessor_qualification']);previous=read(prior['candidate']);files=dict(previous['effective_overlay_files']);paths=dict(previous['effective_overlay_paths'])
 for name,row in k['runtime_overrides'].items():files[name]=row['sha256'];paths[name]=row['path']
 require(k['effective_overlay_files']==files and k['effective_overlay_paths']==paths and len(files)==33,'merged projection differs');check_pins({paths[n]:h for n,h in files.items()})
 cov=coverage(paths['LiTo381-Modern/src/modern_lito/native_skills.rs'],prior['current_source_coverage']);require(cov==k['current_source_coverage'],'coverage differs')
 require(sha(k['override_chain'])==k['override_chain_sha256'],'override chain differs');db=read(k['override_chain']);require(set(db['methods'])=={'ResourceDatabase','CombatDatabase','CombatRules'} and not db['observed_GAME_values'] and not db['load_or_JDBC_invoked'],'override observation scope differs')
 failure=c['original_tooling_failure'];check_pins(failure['source_pins']);failed=read(failure['cohort']);require(failed['status']=='original_compile_failure_preserved' and [r['exit_code'] for r in failed['compiler_runs']]==[0,0,1] and failure['observed_original_tool_chunk']=='180a76' and failure['observed_actual_exit_code']==1 and not failure['source_or_QA_changed'] and not failure['prior_partial_success_transferred'],'original naming failure differs')
 require(Path(failed['generated_mod']).read_bytes()==Path(c['generated_mod']).read_bytes() and all(failed['input_pins'].get(row['path'])==row['sha256'] for row in k['runtime_overrides'].values()) and failed['input_pins'].get(str(T/'templates/native_eruption45_entry_tests.rs'))==sha(T/'templates/native_eruption45_entry_tests.rs'),'Rust/QA changed after original filename failure')
 products={};libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(sha(row['product'])==row['product_sha256'] and Path(row['product']).stat().st_size==row['product_bytes'] and not row.get('original_handle_cleanup'),'metadata product differs');check_pins(row['logs'])
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command,'compiler command differs');products[row['profile']+'-'+kind]=dict(path=row['product'],sha256=row['product_sha256'],bytes=row['product_bytes'])
 return c,k,prior,cov,products

def publish(cohort,chunk):
 c,k,prior,cov,products=validate(cohort);w=read(R/'工作佇列.json');ref=w['current_E149_eruption45_scene_development'];require(ref['qualification']==k['predecessor_qualification'] and ref['qualification_sha256']==k['predecessor_qualification_sha256'],'current scene differs');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and 'current_E149_eruption45_entry_development' not in w,'phase/publication differs')
 before_checkpoint=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before_checkpoint)==8 and all(sha(R/n)==h for n,h in before_checkpoint.items()),'checkpoint differs')
 out=owned(V/'native-E149-eruption45-entry-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);before={n:sha(R/n) for n in CHECKPOINTS}
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in CHECKPOINTS:z.write(R/n,n)
 pins=dict(c['input_pins']);pins[str(cohort)]=sha(cohort)
 for p in [Path(__file__).resolve(),T/'readback-native-E149-eruption45-entry.py',T/'readback-native-E149-eruption45-core.py']:pins[str(p)]=sha(p)
 q=dict(schema='E149-eruption45-entry-development-v1',status='unselected_eruption45_request_menu_and_received_scene_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,candidate=c['contract'],candidate_sha256=c['contract_sha256'],contract=c['contract'],contract_sha256=c['contract_sha256'],compile_run=str(cohort.parent),compile_cohort_sha256=sha(cohort),observed_original_tool_chunk=chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,metadata_products=products,
  predecessor_qualification=k['predecessor_qualification'],predecessor_qualification_sha256=k['predecessor_qualification_sha256'],effective_overlay_files=33,runtime_modules=19,Rust_QA_files=11,Java_files=3,resolved_module_path_attributes=365,original_nested_paths_preserved=6,new_tests_written=10,new_opt_in_original_resource_cases=1,retained_skill_tests_written=175,current_source_coverage=cov,
  wire_prefix=[20,5,4],wire_bytes=11,base_range=8,base_reuse_millis=100,literal_definition=k['literal_definition'],next_definition_ids=[46,47],next_definitions=k['next_definitions'],formal_target_bootstraps_bound=True,override_chain_source_bound=True,actual_GAME_overrides_observed=False,
  current_fixture_binding_unchanged=prior['current_fixture_binding_unchanged'],current_formal_suite_unchanged=prior['current_formal_suite_unchanged'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],
  modules_integrated_into_scene=True,request_registered=True,local_cost_or_damage_predicted=False,retained_received_scene_sources_unchanged=True,child_hurt_dispatched=False,relative_graphics_hurt_binding_complete=False,optional_action0_speed_and_uint32_rollover_qualified=False,original_naming_failure_preserved=c['original_tooling_failure'],
  tests_executed=0,new_sessions=0,desktop_inputs=0,WAV_playbacks=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,GUI_verified=False,old_source_success_transferred=False,whole_goal_complete=False,normal_product_pins=c['normal_product_pins'],recorded_utc=datetime.now(timezone.utc).isoformat())
 raw={str(cohort):sha(cohort),c['contract']:sha(c['contract']),k['inspection']:sha(k['inspection']),k['override_chain']:sha(k['override_chain'])};proof=read(k['inspection']);raw.update({p:h for p,h in proof['source_pins'].items() if Path(p).is_relative_to(Path(k['inspection']).parent)})
 for row in c['compiler_runs']:raw.update(row['logs'])
 raw.update(c['original_tooling_failure']['source_pins'])
 for row in k['runtime_overrides'].values():raw[row['path']]=row['sha256']
 for p in [Path(__file__).resolve(),T/'readback-native-E149-eruption45-entry.py',T/'prepare-native-E149-eruption45-entry.py',T/'prepare-native-E149-eruption45-entry-tools.py',T/'prepare-native-E149-eruption45-entry-libname.py',T/'compile-native-E149-eruption45-entry.py',T/'compile-native-E149-eruption45-entry-libname.py',T/'templates/native_eruption45_entry_tests.rs',T/'native_E149_classfile_bootstraps.py',T/'native-E149-eruption45-entry.md']:raw[str(p)]=sha(p)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for p,h in raw.items():require(sha(p)==h,'raw input changed');z.write(p,Path(p).relative_to(R).as_posix())
 with zipfile.ZipFile(out/'raw.zip') as z:require(len(z.namelist())==len(raw) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in raw.items()),'raw archive differs')
 q.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_archive_files=raw)
 qp=out/'qualified-development.json';qp.write_bytes(encoded(q));published=dict(qualification=str(qp),qualification_sha256=sha(qp),status=q['status'],compile_run=q['compile_run'],compiler_exit_codes=[0]*18,effective_overlay_files=33,request_registered=True,new_tests_written=10,tests_executed=0,request_recognition_count=60,missing_request_count=120,whole_goal_complete=False)
 ep=out/'evidence.json';ep.write_bytes(encoded(dict(qualification=published,previous_goal_turn_evidence=w['last_goal_turn_evidence'],actual_progress='local45 actor request/menu, bootstrap+override static binding and eighteen metadata commands; no tests or full-skill acceptance',whole_goal_complete=False)))
 for n in CHECKPOINTS:
  p=R/n;require(sha(p)==before[n],'concurrent checkpoint writer')
  if n.endswith('.json'):
   j=read(p);j['current_E149_eruption45_entry_development']=published;j['last_goal_turn_evidence']=str(ep);j['last_goal_turn_classification']='progress';j['next_development_action']=NEXT
   if 'whole_goal_complete' in j:j['whole_goal_complete']=False
   if 'goal_complete' in j:j['goal_complete']=False
   if n in ['工作佇列.json','監製進度.json']:j['next_task']=NEXT;j['current_focus']=dict(item='eruption45_local_entry_hurt_still_pending',work=NOTE,next=NEXT,evidence=str(qp));j['progress_updated_utc']=q['recorded_utc']
   p.write_bytes(encoded(j))
  else:p.write_text('2026-10-24：'+NOTE+' '+NEXT+' [來源與編譯](<'+qp.as_posix()+'>)。\n\n'+p.read_text('utf-8-sig'),encoding='utf-8',newline='\n')
 after={n:sha(R/n) for n in CHECKPOINTS};require(all(sha(R/n)==h for n,h in after.items()),'checkpoint readback differs')
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 (out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files=after,before_archive_sha256=sha(out/'before.zip'),qualification=str(qp),qualification_sha256=sha(qp),tests_executed=0,whole_goal_complete=False)))
 print(json.dumps(dict(qualification=str(qp),sha256=sha(qp),checkpoint=str(out/'readback.json'),metadata_commands=18,tests_executed=0,recognized_requests=60,remaining=120),ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('cohort',type=Path);p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();publish(a.cohort,a.observed_original_tool_chunk)
