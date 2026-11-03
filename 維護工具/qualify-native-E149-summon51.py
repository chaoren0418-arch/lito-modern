"""Publish only reviewed source and eighteen metadata commands for 51 selection UI and controller."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
CHECKPOINTS=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
NOTE='#51召喚術候選已接獨立種類下拉選單及typed SummonTarget／controller：C14 14 06 02＋selection共七bytes，不用actor ID。正式31種選項經兩個helper與七欄constructor展開，特殊容量含+7；controlAbility5時依選項，否則由server依等級決定。顯式選取與背包revision／完整owner、connection／generation／actor／control interruption／map守衛保留；reuse1000基值、30MP／HP、三寶石、容量與1800秒皆server權威。353／743召喚拒絕可晚於action，新增scoped pending回饋；2510／755非成功ACK。四Runtime差分仍33檔／365module，原五controller＋一rental共六nested重建同target；兩版library／library-test／七skill整合18metadata退出0；原初版action即清pending缺口封存，修正版於既有兩秒窗保留跨poll拒絕關聯，其他技能原終止規則不變。新12純QA＋前214共226案全未跑；入口64／缺116，不是玩法／GUI資格。正常1049輸入／PE、正式JAR、registry與G06 binding／16案640次不變，桌面／測試／播放暫緩。'
NEXT='依當版#55／#56定義續接剩餘玩法；#51完整companion指令、server控制能力／模板／材料／拒絕、自然1800秒與owner／傳送退役須恢復測試後驗。續綁45／46／50原hurt完整guard／LT相對gfx與初始化、action0倍率／uint32 rollover，不猜base或造hurt。完整G06、library Debug／Release、Release與GUI仍依總報告，等使用者明確恢復測試。'


def validate(cohort):
 c=read(cohort);k=read(c['contract']);require(sha(c['contract'])==c['contract_sha256'] and k['base_source_manifest_sha256']==SOURCE and k['formal_target_sha256']==FORMAL,'source/formal differs');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'readonly dependency differs')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'];require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and [(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,kind) for p in ['Debug','Release'] for kind in kinds] and [r['exit_code'] for r in c['compiler_runs']]==[0]*18 and c['owned_process'] is None,'cohort incomplete')
 require(c['tests_executed']==0 and not any(c[x] for x in ['PEs_linked','outputs_executed','whole_goal_complete']),'execution boundary differs')
 old=read(c['predecessor_cohort']);require(sha(c['predecessor_cohort'])==c['predecessor_cohort_sha256'] and c['paired_tests']==old['paired_tests'],'predecessor/seven integrations differ');modern=Path(old['generated_mod']).read_text('utf-8-sig');case_count=0;ignored_count=0
 for name,row in k['runtime_overrides'].items():
  s=Path(row['predecessor']).read_text('utf-8-sig');require(sha(row['predecessor'])==row['predecessor_sha256'],'predecessor raw differs')
  for a,b in row['changes']:require(s.count(a)==1,'source delta ambiguous');s=s.replace(a,b)
  for template in row['supplements']:
   require(sha(template['path'])==template['sha256'],'source supplement differs');extra=Path(template['path']).read_text('utf-8-sig');s+='\n'+extra;case_count+=extra.count('#[test]');ignored_count+=extra.count('#[ignore=')
  if row['generated_suffix']:s+='\n'+row['generated_suffix']
  require(s==Path(row['path']).read_text('utf-8-sig') and sha(row['path'])==row['sha256'],'source reconstruction differs')
  previous=row['predecessor'];replacement=row['path']
  if name in old['original_nested_runtime_pairs']:
   pair=old['original_nested_runtime_pairs'][name];current=c['original_nested_runtime_pairs'][name];require(current['source']==row['path'] and current['source_sha256']==row['sha256'] and current['rewrites']==pair['rewrites'] and current['predecessor']==pair,'nested pair identity differs');wrapped=s
   for rewrite in pair['rewrites']:
    require(wrapped.count(rewrite['original_attribute'])==1 and sha(rewrite['resolved_path'])==c['input_pins'][rewrite['resolved_path']],'nested target differs');wrapped=wrapped.replace(rewrite['original_attribute'],rewrite['compiled_attribute'])
   require(wrapped==Path(current['path']).read_text('utf-8-sig') and sha(current['path'])==current['sha256'],'nested wrapper differs');previous=pair['path'];replacement=current['path']
  needles=list(dict.fromkeys([previous,Path(previous).as_posix()]));found=[n for n in needles if modern.count(n)==1];require(len(found)==1 and sum(modern.count(n) for n in needles)==1,'runtime mapping differs');modern=modern.replace(found[0],Path(replacement).as_posix())
 require(len(k['runtime_overrides'])==4 and case_count==12 and ignored_count==0 and k['modules']=={} and modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==c['module_count']==365,'module/case inventory differs')
 require(Path(old['generated_root']).read_text('utf-8-sig').replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig') and set(c['original_nested_runtime_pairs'])==set(old['original_nested_runtime_pairs']) and sum(len(r['rewrites']) for r in c['original_nested_runtime_pairs'].values())==6,'root/nested pairing differs')
 for pair in c['original_nested_runtime_pairs'].values():require(sha(pair['path'])==pair['sha256'] and sha(pair['source'])==pair['source_sha256'],'nested source differs')
 prior=read(k['predecessor_qualification']);require(sha(k['predecessor_qualification'])==k['predecessor_qualification_sha256'],'prior qualification differs');base=read(prior['candidate']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths'])
 for n,row in k['runtime_overrides'].items():files[n]=row['sha256'];paths[n]=row['path']
 require(len(files)==33 and files==k['effective_overlay_files'] and paths==k['effective_overlay_paths'],'projection differs');check_pins({paths[n]:h for n,h in files.items()})
 request=Path(paths['LiTo381-Modern/src/modern_lito/native_skills.rs']);s=request.read_text('utf-8-sig');b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'request parser differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 m=re.search(r'pub fn summon_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',s);require(m is not None,'summon source recognition absent');ids.update(map(int,re.findall(r'\d+',m[1])))
 previous=prior['current_source_coverage'];require(previous['recognized_count']==63 and ids==set(previous['recognized_ids'])|{51} and len(ids)==64,'recognition changes exceed51');coverage=dict(previous);coverage.update(request_source=str(request),request_source_sha256=sha(request),recognized_ids=sorted(ids),recognized_count=64,missing_request_ids=sorted(set(previous['missing_request_ids'])-{51}),missing_request_count=116);require(coverage==k['current_source_coverage'],'coverage differs')
 proof=read(k['inspection']);require(sha(k['inspection'])==k['inspection_sha256'] and proof['definitions']==k['definitions'] and len(proof['choices'])==31 and proof['control_ability']==5 and proof['special_capacity_adjustment']==7 and proof['material_before_action'] and proof['action_before_map_level_capacity_failure'] and not proof['PE_executed'],'primary/profile scope differs');check_pins(proof['source_pins']);require(all(sha(p)==h for p,h in proof['external_readonly_pins'].items()),'original primary differs')
 expected='pub const SUMMON_SELECTIONS:&[SummonSelection]=&[\n'+''.join('    SummonSelection{selection:'+str(row['selection'])+',label:'+json.dumps(row['base_label'],ensure_ascii=False)+',required_level:'+str(row['required_level'])+'},\n' for row in proof['choices'])+'];\n';require(expected==k['generated_summon_selections'] and k['runtime_overrides']['LiTo381-Modern/src/modern_lito/native_skills.rs']['generated_suffix']==expected,'generated choices differ')
 initial=read(k['original_metadata_cohort']);require(sha(k['original_metadata_cohort'])==k['original_metadata_cohort_sha256'] and sha(k['original_development_candidate'])==k['original_development_candidate_sha256'] and initial['contract_sha256']==k['original_development_candidate_sha256'] and initial['status']=='eighteen_metadata_compilers_complete_no_tests' and [r['exit_code'] for r in initial['compiler_runs']]==[0]*18 and k['original_metadata_actual_exit_code']==0 and k['summon_pending_after_cast_action_bounded_seconds']==2 and k['original_issue_was_static_review_not_executed_test'],'initial source/metadata boundary differs');check_pins(initial['input_pins'])
 libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']};products={}
 for row in c['compiler_runs']:
  require(sha(row['product'])==row['product_sha256'] and Path(row['product']).stat().st_size==row['product_bytes'] and not row.get('original_handle_cleanup'),'metadata product differs');check_pins(row['logs']);kind=row['kind'];command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command,'metadata command differs');products[row['profile']+'-'+kind]=dict(path=row['product'],sha256=row['product_sha256'],bytes=row['product_bytes'])
 return c,k,prior,coverage,proof,products

def publish(cohort,chunk):
 c,k,prior,coverage,proof,products=validate(cohort);w=read(R/'工作佇列.json');ref=w['current_E149_freeze50_development'];require(ref['qualification']==k['predecessor_qualification'] and ref['qualification_sha256']==k['predecessor_qualification_sha256'] and w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and 'current_E149_summon51_development' not in w,'current phase differs')
 checkpoint=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(checkpoint)==8 and all(sha(R/n)==h for n,h in checkpoint.items()),'checkpoint differs');out=owned(V/'native-E149-summon51-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);before={n:sha(R/n) for n in CHECKPOINTS}
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in CHECKPOINTS:z.write(R/n,n)
 pins=dict(c['input_pins']);pins[str(cohort)]=sha(cohort);initial=read(k['original_metadata_cohort']);pins.update(initial['input_pins'])
 for p in [Path(__file__).resolve(),T/'readback-native-E149-summon51.py',T/'readback-native-E149-eruption45-core.py']:pins[str(p)]=sha(p)
 pins[str(T/'fix-summon51-publication-newline.py')]=sha(T/'fix-summon51-publication-newline.py')
 pins['D:\\lineage開發\\3-lito開發\\lito重建版\\驗證紀錄\\current\\native-E149-summon51-tooling-notes\\20261024-110614-200727\\receipt.json']=sha('D:\\lineage開發\\3-lito開發\\lito重建版\\驗證紀錄\\current\\native-E149-summon51-tooling-notes\\20261024-110614-200727\\receipt.json')
 q=dict(schema='E149-summon51-development-v1',status='unselected_summon51_selection_UI_controller_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,candidate=c['contract'],candidate_sha256=c['contract_sha256'],contract=c['contract'],contract_sha256=c['contract_sha256'],compile_run=str(cohort.parent),compile_cohort_sha256=sha(cohort),observed_original_tool_chunk=chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,metadata_products=products,predecessor_qualification=k['predecessor_qualification'],predecessor_qualification_sha256=k['predecessor_qualification_sha256'],changed_runtime_files=4,effective_overlay_files=33,runtime_modules=19,Rust_QA_files=11,Java_files=3,resolved_module_path_attributes=365,original_nested_paths_preserved=6,new_tests_written=12,new_opt_in_original_resource_cases=0,retained_skill_tests_written=214,current_source_coverage=coverage,definitions=k['definitions'],next_definition_ids=k['next_definition_ids'],next_definitions=k['next_definitions'],
  original_development_candidate=k['original_development_candidate'],original_development_candidate_sha256=k['original_development_candidate_sha256'],original_metadata_cohort=k['original_metadata_cohort'],original_metadata_cohort_sha256=k['original_metadata_cohort_sha256'],original_metadata_observed_tool_chunk=k['original_metadata_observed_tool_chunk'],original_source_issue=k['original_source_issue'],summon_pending_after_cast_action_bounded_seconds=2,
  summon_selection_count=31,typed_summon_domain=True,explicit_nonmodal_selection_UI=True,full_inventory_owner_and_InputStamp_fence=True,server_controlled_selection_or_direct_grade=True,scoped_map_and_level_failure_after_cast_action=True,summon_effects_are_not_ACK=True,original_six_nested_targets_preserved=True,no_local_cost_damage_or_status_prediction=True,shared_audio_and_fragment_caps_unchanged=True,companion_commands_and_natural_lifetime_qualified=False,
  current_fixture_binding_unchanged=prior['current_fixture_binding_unchanged'],current_formal_suite_unchanged=prior['current_formal_suite_unchanged'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],request_registered_ids=[51],relative_graphics_hurt_binding_complete=False,optional_action0_speed_and_uint32_rollover_qualified=False,actual_GAME_overrides_observed=False,
  tests_executed=0,new_sessions=0,desktop_inputs=0,WAV_playbacks=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,GUI_verified=False,old_source_success_transferred=False,whole_goal_complete=False,normal_product_pins=c['normal_product_pins'],recorded_utc=datetime.now(timezone.utc).isoformat())
 raw={str(cohort):sha(cohort),c['contract']:sha(c['contract']),k['inspection']:sha(k['inspection'])};raw.update({p:h for p,h in proof['source_pins'].items() if Path(p).is_relative_to(Path(k['inspection']).parent)})
 for row in c['compiler_runs']:raw.update(row['logs'])
 for row in k['runtime_overrides'].values():raw[row['path']]=row['sha256'];raw.update({t['path']:t['sha256'] for t in row['supplements']})
 for p in [Path(__file__).resolve(),T/'readback-native-E149-summon51.py',T/'prepare-native-E149-summon51.py',T/'prepare-native-E149-summon51-tools.py',T/'compile-native-E149-summon51.py',T/'native-E149-summon51.md']:raw[str(p)]=sha(p)
 for path in [Path(k['original_development_candidate']),Path(k['original_metadata_cohort']),Path(initial['generated_root']),Path(initial['generated_mod']),T/'compile-native-E149-summon51-hold.py',T/'prepare-native-E149-summon51-hold.py',T/'prepare-native-E149-summon51-hold-tools.py',T/'prepare-native-E149-summon51-hold-publication.py',T/'native-E149-summon51-hold.md']:raw[str(path)]=sha(path)
 for row in read(k['original_development_candidate'])['runtime_overrides'].values():raw[row['path']]=row['sha256']
 for row in initial['compiler_runs']:raw.update(row['logs'])
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for p,h in raw.items():require(sha(p)==h,'raw input changed');z.write(p,Path(p).relative_to(R).as_posix())
 with zipfile.ZipFile(out/'raw.zip') as z:require(len(z.namelist())==len(raw) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in raw.items()),'raw archive differs')
 q.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_archive_files=raw);qp=out/'qualified-development.json';qp.write_bytes(encoded(q));published=dict(qualification=str(qp),qualification_sha256=sha(qp),status=q['status'],compile_run=q['compile_run'],compiler_exit_codes=[0]*18,effective_overlay_files=33,new_tests_written=12,tests_executed=0,request_recognition_count=64,missing_request_count=116,whole_goal_complete=False)
 ep=out/'evidence.json';ep.write_bytes(encoded(dict(qualification=published,previous_goal_turn_evidence=w['last_goal_turn_evidence'],actual_progress='51 typed selection UI and controller, original31 helpers, scoped late refusal; eighteen metadata commands only',whole_goal_complete=False)))
 for n in CHECKPOINTS:
  p=R/n;require(sha(p)==before[n],'concurrent checkpoint writer')
  if n.endswith('.json'):
   j=read(p);j['current_E149_summon51_development']=published;j['last_goal_turn_evidence']=str(ep);j['last_goal_turn_classification']='progress';j['next_development_action']=NEXT
   if 'whole_goal_complete' in j:j['whole_goal_complete']=False
   if 'goal_complete' in j:j['goal_complete']=False
   if n in ['工作佇列.json','監製進度.json']:j['next_task']=NEXT;j['current_focus']=dict(item='summon51_typed_selection_UI_and_controller',work=NOTE,next=NEXT,evidence=str(qp));j['progress_updated_utc']=q['recorded_utc']
   p.write_bytes(encoded(j))
  else:p.write_text('2026-10-24：'+NOTE+' '+NEXT+' [來源與編譯](<'+qp.as_posix()+'>)。\n\n'+p.read_text('utf-8-sig'),encoding='utf-8',newline='\n')
 after={n:sha(R/n) for n in CHECKPOINTS};require(all(sha(R/n)==h for n,h in after.items()),'checkpoint differs')
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 (out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files=after,before_archive_sha256=sha(out/'before.zip'),qualification=str(qp),qualification_sha256=sha(qp),tests_executed=0,whole_goal_complete=False)));print(json.dumps(dict(qualification=str(qp),sha256=sha(qp),checkpoint=str(out/'readback.json'),metadata_commands=18,tests_executed=0,recognized_requests=64,remaining=116),ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('cohort',type=Path);p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();publish(a.cohort,a.observed_original_tool_chunk)
