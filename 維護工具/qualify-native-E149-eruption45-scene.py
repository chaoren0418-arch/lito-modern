"""Publish exact S23/129 source and compiler scope after the original tool exits."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
CHECKPOINTS=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
NOTE='#45接收呈現已接候選：S23 wire0／gfx129依資料走class7；原排程同deadline FIFO已靜態綁定。Model shared資源、caster Hit、固定wire source／同incarnation root endpoint、獨立節點History／raw ring與book淘汰、source／generation退役、Over＋Shadow整份預檢、C6最後提交及typed音效occurrence已接；caster／target離開不砍已開始射線，child不接ABA。共33檔十九Runtime／十一Rust QA／三Java，完整365module與原六nested；兩版library／library-test四metadata退出0，新10（八純＋二原資源opt-in）＋前165共175案全未跑。child Hit僅精確診斷：原相對gfx hurt及入口資格／ResourceDatabase、可選速度倍率／timer rollover未解；本機#45請求尚未登記，識別59／剩121。正常1049輸入／PE、registry及G06 binding／16案640次不變，桌面／測試／播放仍暫緩。'
NEXT='續綁#45原type6 hurt完整排除／LT相對gfx與初始化、正式目標資格／ResourceDatabase覆寫，補本機請求與選單；保留已接S23呈現與來源／incarnation守衛。原action0倍率及unsigned32長uptime邊界須核對，不猜gfx base；其餘技能／G06及完整library雙版／Release／GUI依總報告。測試與桌面等使用者明確恢復。'

def validate(cohort):
 c=read(cohort);k=read(c['contract']);require(sha(c['contract'])==c['contract_sha256'],'contract differs');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'readonly input differs')
 require(c['status']=='four_metadata_compilers_complete_no_tests' and len(c['compiler_runs'])==4 and [x['exit_code'] for x in c['compiler_runs']]==[0]*4 and c['owned_process'] is None and c['module_count']==365 and c['scene_integrated'],'compiler scope incomplete')
 require(c['tests_executed']==0 and not any(c[x] for x in ['PEs_linked','outputs_executed','whole_goal_complete']),'execution boundary differs')
 require([(x['profile'],x['kind']) for x in c['compiler_runs']]==[(p,kind) for p in ['Debug','Release'] for kind in ['library','library-test']],'compiler order differs')
 old=read(c['predecessor_cohort']);require(sha(c['predecessor_cohort'])==c['predecessor_cohort_sha256'] and old['module_count']==364,'predecessor differs');modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for name,row in k['runtime_overrides'].items():
  source=Path(row['predecessor']).read_text('utf-8-sig');require(sha(row['predecessor'])==row['predecessor_sha256'],'wrapper predecessor changed')
  for a,b in row['changes']:require(source.count(a)==1,'wrapper source delta ambiguous');source=source.replace(a,b)
  require(source==Path(row['path']).read_text('utf-8-sig') and sha(row['path'])==row['sha256'],'wrapper delta differs')
  needles=list(dict.fromkeys([row['predecessor'],Path(row['predecessor']).as_posix()]));found=[n for n in needles if modern.count(n)==1];require(len(found)==1 and sum(modern.count(n) for n in needles)==1,'wrapper module mapping differs');modern=modern.replace(found[0],Path(row['path']).as_posix())
 for name,row in k['modules'].items():modern+='\n#[path = r#"'+Path(row['path']).as_posix()+'"#]\npub(crate) mod '+name+';\n'
 require(modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==365,'generated full graph differs')
 oldroot=Path(old['generated_root']).read_text('utf-8-sig');require(Path(c['generated_root']).read_text('utf-8-sig')==oldroot.replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix()),'root differs')
 require(c['original_nested_runtime_pairs']==old['original_nested_runtime_pairs'],'original nested pair differs')
 for pair in c['original_nested_runtime_pairs'].values():require(sha(pair['path'])==pair['sha256'] and sha(pair['source'])==pair['source_sha256'],'nested runtime source differs')
 paths=k['effective_overlay_paths'];files=k['effective_overlay_files'];require(len(files)==len(paths)==33,'overlay count differs');check_pins({paths[n]:h for n,h in files.items()})
 adapter=Path(k['modules']['native_eruption45_scene']['path']).read_text('utf-8-sig');require(adapter.count('#[test]')==10 and adapter.count('#[ignore=')==2 and 'unreachable!' not in adapter,'new QA/source incomplete')
 failure=k['original_compile_failure'];check_pins(failure['source_pins']);failed=read(failure['cohort']);require(failed['status']=='original_compile_failure_preserved' and [r['exit_code'] for r in failed['compiler_runs']]==[0,1] and failure['observed_original_tool_chunk']=='c12ec5' and failure['observed_actual_exit_code']==1 and not failure['prior_partial_library_success_transferred'],'original compiler failure differs')
 correction=failure['corrected_only_QA_path_type'];text=Path(correction['original']).read_text('utf-8-sig');require(text.count(correction['before'])==1 and text.replace(correction['before'],correction['after'])==adapter and sha(correction['original'])==correction['original_sha256'] and sha(correction['replacement'])==correction['replacement_sha256'],'QA path correction differs')
 proof=read(k['inspection']);require(sha(k['inspection'])==k['inspection_sha256'] and proof['S23_wire_flight_type']==0 and proof['native_effect_class']==7 and proof['factory_membership']=={n:n=='attackSpell' for n in proof['factory_membership']} and len(proof['javap_commands'])==2 and all(x['exit_code']==0 for x in proof['javap_commands']),'primary evidence differs')
 products={}
 for row in c['compiler_runs']:
  require(sha(row['product'])==row['product_sha256'] and Path(row['product']).stat().st_size==row['product_bytes'],'metadata differs');check_pins(row['logs'])
  template=next(x['command'] for x in old['compiler_runs'] if x['profile']==row['profile'] and x['kind']=='library');expected=list(template);expected[expected.index(old['generated_root'])]=c['generated_root'];expected[expected.index('-o')+1]=row['product']
  if row['kind']=='library-test':expected.remove('--crate-type=rlib');expected+=['--test']
  require(row['command']==expected and '--emit=metadata' in expected,'compiler command differs');products[row['profile']+'-'+row['kind']]=dict(path=row['product'],sha256=row['product_sha256'],bytes=row['product_bytes'])
 return c,k,proof,products

def publish(cohort,chunk):
 c,k,proof,products=validate(cohort);w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'phase differs')
 prior=w['current_E149_eruption45_core_development'];require(prior['qualification']==k['predecessor_qualification'] and prior['qualification_sha256']==k['predecessor_qualification_sha256'],'current core differs');require('current_E149_eruption45_scene_development' not in w,'stage already published')
 previous=read(prior['qualification']);before_checkpoint=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before_checkpoint)==8 and all(sha(R/n)==h for n,h in before_checkpoint.items()),'checkpoint changed')
 out=owned(V/'native-E149-eruption45-scene-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);before={n:sha(R/n) for n in CHECKPOINTS}
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in CHECKPOINTS:z.write(R/n,n)
 pins=dict(c['input_pins']);pins[str(cohort)]=sha(cohort)
 for p in [Path(__file__).resolve(),T/'readback-native-E149-eruption45-scene.py',T/'readback-native-E149-eruption45-core.py',T/'prepare-native-E149-eruption45-scene-tools.py']:pins[str(p)]=sha(p)
 q=dict(schema='E149-eruption45-scene-development-v1',status='unselected_S23_129_class7_scene_and_audio_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,
  candidate=c['contract'],candidate_sha256=c['contract_sha256'],contract=c['contract'],contract_sha256=c['contract_sha256'],compile_run=str(cohort.parent),compile_cohort_sha256=sha(cohort),observed_original_tool_chunk=chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*4,metadata_products=products,
  predecessor_qualification=prior['qualification'],predecessor_qualification_sha256=prior['qualification_sha256'],effective_overlay_files=33,runtime_modules=19,Rust_QA_files=11,Java_files=3,resolved_module_path_attributes=365,original_nested_paths_preserved=6,
  new_tests_written=10,new_opt_in_original_resource_cases=2,retained_skill_tests_written=165,current_source_coverage=k['current_source_coverage'],
  current_fixture_binding_unchanged=previous['current_fixture_binding_unchanged'],current_formal_suite_unchanged=previous['current_formal_suite_unchanged'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],
  modules_integrated_into_scene=True,request_registered=False,native_scheduler_FIFO_static_binding=True,child_hurt_dispatched=False,relative_graphics_hurt_binding_complete=False,ResourceDatabase_GAME_observed=False,optional_action0_speed_and_uint32_rollover_qualified=False,
  active_history_includes_eruption_sequences=True,independent_crust_lifetime_not_actor_lifetime=True,root_and_child_target_incarnation_guard=True,wire_source_and_ray_endpoint_frozen=True,whole_cast_dual_layer_preflight=True,final_child_draw_committed_by_exact_occurrence=True,shared_64_cast_512_fragment_4096_audio_caps_unchanged=True,
  tests_executed=0,new_sessions=0,desktop_inputs=0,WAV_playbacks=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,GUI_verified=False,old_source_success_transferred=False,whole_goal_complete=False,
  pending=k['pending'],original_compile_failure_preserved=k['original_compile_failure'],normal_product_pins=c['normal_product_pins'],recorded_utc=datetime.now(timezone.utc).isoformat())
 raw={str(Path(c['contract'])):sha(c['contract']),str(cohort):sha(cohort),k['inspection']:sha(k['inspection'])}
 raw.update({p:h for p,h in proof['source_pins'].items() if Path(p).is_relative_to(Path(k['inspection']).parent)})
 for row in c['compiler_runs']:raw.update(row['logs'])
 raw.update(k['original_compile_failure']['source_pins'])
 for row in k['modules'].values():raw[row['path']]=row['sha256']
 for row in k['runtime_overrides'].values():raw[row['path']]=row['sha256']
 for p in [Path(__file__).resolve(),T/'readback-native-E149-eruption45-scene.py',T/'compile-native-E149-eruption45-scene.py',T/'prepare-native-E149-eruption45-scene.py',T/'prepare-native-E149-eruption45-scene-tools.py',T/'native-E149-eruption45-scene.md',V/'native-E149-eruption45-scene-inspection/20261024-091638-487209/failed-inspector.py',V/'native-E149-eruption45-scene-inspection/20261024-091638-487209/original-failure.txt',V/'native-E149-eruption45-scene-tooling-notes/20261024-092330/failed-prepare.py',V/'native-E149-eruption45-scene-tooling-notes/20261024-092330/original-failure.txt']:raw[str(p)]=sha(p)
 for p in [T/'compile-native-E149-eruption45-scene-root.py',T/'prepare-native-E149-eruption45-scene-root.py']:raw[str(p)]=sha(p)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for p,h in raw.items():require(sha(p)==h,'raw input changed');z.write(p,Path(p).relative_to(R).as_posix())
 with zipfile.ZipFile(out/'raw.zip') as z:require(len(z.namelist())==len(raw) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in raw.items()),'raw archive differs')
 q.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_archive_files=raw)
 qp=out/'qualified-development.json';qp.write_bytes(encoded(q));ref=dict(qualification=str(qp),qualification_sha256=sha(qp),status=q['status'],compile_run=q['compile_run'],compiler_exit_codes=[0]*4,effective_overlay_files=33,scene_integrated=True,request_registered=False,new_tests_written=10,tests_executed=0,request_recognition_count=59,missing_request_count=121,whole_goal_complete=False)
 ep=out/'evidence.json';ep.write_bytes(encoded(dict(qualification=ref,previous_goal_turn_evidence=w['last_goal_turn_evidence'],actual_progress='received S23/129 integrated source, FIFO binding and four metadata compilers; no tests or complete-skill acceptance',whole_goal_complete=False)))
 for n in CHECKPOINTS:
  p=R/n;require(sha(p)==before[n],'concurrent checkpoint writer')
  if n.endswith('.json'):
   j=read(p);j['current_E149_eruption45_scene_development']=ref;j['last_goal_turn_evidence']=str(ep);j['last_goal_turn_classification']='progress';j['next_development_action']=NEXT
   if 'whole_goal_complete' in j:j['whole_goal_complete']=False
   if 'goal_complete' in j:j['goal_complete']=False
   if n in ['工作佇列.json','監製進度.json']:j['next_task']=NEXT;j['current_focus']=dict(item='eruption45_received_scene_hurt_and_local_entry_pending',work=NOTE,next=NEXT,evidence=str(qp));j['progress_updated_utc']=q['recorded_utc']
   p.write_bytes(encoded(j))
  else:p.write_text('2026-10-24：'+NOTE+' '+NEXT+' [來源與編譯](<'+qp.as_posix()+'>)。\n\n'+p.read_text('utf-8-sig'),encoding='utf-8',newline='\n')
 after={n:sha(R/n) for n in CHECKPOINTS};require(all(sha(R/n)==h for n,h in after.items()),'checkpoint differs')
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 (out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files=after,before_archive_sha256=sha(out/'before.zip'),qualification=str(qp),qualification_sha256=sha(qp),tests_executed=0,whole_goal_complete=False)))
 print(json.dumps(dict(qualification=str(qp),sha256=sha(qp),checkpoint=str(out/'readback.json'),compiler_exit_codes=[0]*4,tests_executed=0,scene_integrated=True),ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('cohort',type=Path);p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();publish(a.cohort,a.observed_original_tool_chunk)
