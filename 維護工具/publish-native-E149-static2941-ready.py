"""Publish an isolated static queue prototype; resource-ready and scene gates remain open."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,hashlib,zipfile,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_process_birth_readonly import original_terminal
T=R/'維護工具';V=R/'驗證紀錄/current';KEY='current_E149_static2941_ready_development'
CP=V/'native-E149-static2941-ready-metadata/20261024-210441-110315/cohort.json'
CHECKPOINTS=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
NOTE='97已改為receive→等待SPR→實際bank證明→fresh enqueue，尚未Ready不建立deadline；proof持有已核對bank，非布林或假時間。Scene背景載入／原fragments與Over提交、FiniteHistory、view callback交付、controller queue、cached／late-frame及paint／input token守衛來源已接；retire token不因resource取消／root ABA／history恢復而復活，source錯誤同scope latch。原共享callback／重入繪圖與native clock仍未資格，97未完整、不可套用。44檔30 Runtime／11 Rust QA／3 Java，372module／原六nested，20metadata均0。原8案改寫＋新增4共12（8純／4原資源opt-in）；全體475案全未跑，88／剩92不變。candidate新資源案欄位0為舊描述值，依實際QA獨立盤點新1／重分類原3，不修改已編譯contract或移記資格。準備工具get預設值誤索引新增module的9e69f3失敗在編譯前，原source／partial目錄保留；v2只修候選／正常清單取值。正常1049／PE／JAR／registry／G06640次不變；未套用／選入／連結PE／測試／載入SPR／桌面／播放。'
NEXT='先補原共享callback與重入繪圖producer／4D6570固定sample交錯，確認2941在合法window可提交、callback後不可再發佈，不能用固定一幀或永遠不可見代替；native clock／rollover與原像素oracle仍未驗。97來源與原資源／GUI完成後再99／100及剩92，並續4183、G06物品／source/account／timer、LT hurt/base。使用者明確恢復測試後先跑12 gate案（4個需實際原資源）、當版完整library Debug／Release及G06精確640次／Release回歸，再租裝等完整GUI，自行核對後交人工。'

def main(chunk):
 w=read(R/'工作佇列.json');require(KEY not in w and w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'source-only phase differs');c=read(CP);require(c['status']=='twenty_metadata_complete_no_execution' and c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compile scope differs');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency differs');k=read(c['candidate']);require(sha(c['candidate'])==c['candidate_sha256'],'candidate differs');require((k['overlay_count'],k['runtime_files'],k['Rust_QA_files'],k['Java_files'],k['module_count'],k['original_nested_paths_preserved'],k['new_tests_written'],k['previous_metadata_cases'])==(44,30,11,3,372,6,4,471),'inventory differs');require(k['network_decode_to_event_to_login_to_world_source_bound'] and k['static2941_cue_receipt_bound'] and not k['native_PE_clock_equivalence_qualified'] and not k['scene_history_and_retirement_queue_integrated'],'receipt boundary differs')
 tests=0
 for name,row in k['runtime_overrides'].items():
  require(sha(row['predecessor'])==row['predecessor_sha256'],'runtime predecessor differs');s=Path(row['predecessor']).read_text('utf-8-sig')
  if row.get('replacement_template'):
   rt=row['replacement_template'];require(sha(rt['path'])==rt['sha256'],'replacement module template differs');s=Path(rt['path']).read_text('utf-8-sig')
  for rt in row.get('template_replacements',[]):
   require(sha(rt['before'])==rt['before_sha256'] and sha(rt['after'])==rt['after_sha256'],'QA template differs');before_text=Path(rt['before']).read_text('utf-8-sig');after_text=Path(rt['after']).read_text('utf-8-sig');require(s.count(before_text)==1,'QA replacement ambiguous');s=s.replace(before_text,after_text)
  for a,b in row['changes']:require(s.count(a)==row.get('repeated_replacement_counts',{}).get(a,1),'source delta ambiguous');s=s.replace(a,b)
  for extra in row['supplements']:require(sha(extra['path'])==extra['sha256'],'source supplement differs');v=Path(extra['path']).read_text('utf-8-sig');s+='\n'+v;tests+=v.count('#[test]')
  require(s==Path(row['path']).read_text('utf-8-sig') and sha(row['path'])==row['sha256'],'source reconstruction differs')
  if name in c['original_nested_runtime_pairs']:
   pair=c['original_nested_runtime_pairs'][name];require(pair['source']==row['path'] and pair['source_sha256']==row['sha256'],'paired source differs')
   for rw in pair['rewrites']:require(s.count(rw['original_attribute'])==1 and sha(rw['resolved_path'])==c['input_pins'][rw['resolved_path']],'nested path differs');s=s.replace(rw['original_attribute'],rw['compiled_attribute'])
   require(s==Path(pair['path']).read_text('utf-8-sig') and sha(pair['path'])==pair['sha256'],'paired runtime differs')
 require(tests==0 and len(k['effective_overlay_files'])==44 and all(sha(k['effective_overlay_paths'][n])==h for n,h in k['effective_overlay_files'].items()) and sum(len(v['rewrites'])for v in c['original_nested_runtime_pairs'].values())==6,'effective source differs')
 for m in k['modules'].values():require(sha(m['path'])==m['sha256'] and sha(m['template'])==m['template_sha256'] and Path(m['path']).read_bytes()==Path(m['template']).read_bytes(),'new queue module differs')
 require(k['controller_queue_and_fresh_enqueue_sample_bound'] and k['lease_revocation_shared_with_snapshots'] and k['scene_and_history_consumers_bound'] and k['resource_ready_before_enqueue_source_bound'] and k['cached_and_late_frame_token_checks_source_bound'] and not k['all_native_global_callbacks_qualified'],'ready source gates differ')
 qa_row=k['runtime_overrides']['LiTo381-Modern/src/modern_lito/native_connection.rs']['template_replacements'][0];before_qa=Path(qa_row['before']).read_text('utf-8-sig');after_qa=Path(qa_row['after']).read_text('utf-8-sig');pattern=r'#\[test\]\s*(?:#\[ignore[^\n]*?\]\s*)?fn\s+(\w+)';old_ids=set(re.findall(pattern,before_qa));new_ids=set(re.findall(pattern,after_qa));require(len(old_ids)==8 and len(new_ids)==12 and old_ids<=new_ids and len(new_ids-old_ids)==4 and after_qa.count('#[ignore=')==4,'actual QA classification differs');pf=read(k['preparation_refinement']);require(sha(k['preparation_refinement'])==k['preparation_refinement_sha256'] and pf['observed_original_tool_chunk']=='9e69f3' and pf['observed_actual_exit_code']==1 and pf['Rust_compilers_started']==0 and sha(pf['original_source'])==pf['original_source_sha256'],'preparation failure differs')
 kinds=['library','library-test',*c['paired_tests']];require(len(kinds)==10 and [(r['profile'],r['kind'])for r in c['compiler_runs']]==[(p,n)for p in ['Debug','Release']for n in kinds],'command inventory differs')
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['original_handle_signaled'] and '--emit=metadata'in row['command'] and ('--test'in row['command'])==(row['kind']!='library') and sha(row['product'])==row['product_sha256'] and all(sha(p)==h for p,h in row['logs'].items()),'actual metadata product/log differs')
  if row['kind']in c['paired_tests']:
   extern=next(n for n in row['command']if n.startswith('lito_modern='));require(Path(extern.split('=',1)[1]).name.startswith('liblito_modern-'),'extern library basename differs')
 identities=[c['pipeline']]+[r['original_process']for r in c['compiler_runs']];require(len({(v['pid'],v['creation_filetime_ticks'])for v in identities})==21,'original process inventory differs');observations=[original_terminal(v)for v in identities]
 ref=w['current_E149_static2941_queue_development'];require(k['predecessor_source']==ref and sha(ref['evidence'])==ref['evidence_sha256'],'previous development differs');previous=read(ref['evidence']);fullref=previous['previous_qualified_18_metadata'];fullq=read(fullref['qualification']);require(sha(fullref['qualification'])==fullref['qualification_sha256'],'prior full metadata qualification differs');base=read(k['predecessor_candidate']);require(sha(k['predecessor_candidate'])==k['predecessor_candidate_sha256'],'prior candidate differs')
 for n in ['LiTo381-Modern/src/modern_lito/native_skills.rs','LiTo381-Modern/src/modern_lito/native_skill_selection.rs']:require(k['effective_overlay_files'][n]==base['effective_overlay_files'][n],'request/menu source changed')
 cov=fullq['current_source_coverage'];require(cov['recognized_count']==88 and cov['missing_request_count']==92,'coverage differs');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049,'normal count differs');check_pins({str(R/n):h for n,h in inputs.items()});require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and previous['canonical_registry_unchanged']==w['current_isolated_development']['candidates'] and previous['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and previous['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'],'formal/G06/registry differs')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(set(before)==set(CHECKPOINTS) and all(sha(R/n)==h for n,h in before.items()),'checkpoint differs');out=V/'native-E149-static2941-ready-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
  for n in CHECKPOINTS:z.write(R/n,n)
 raw={}
 for folder in [CP.parent]:
  for p in folder.iterdir():
   if p.is_file():raw[str(p)]=sha(p)
 for row in c['compiler_runs']:
  for p,h in row['logs'].items():raw[p]=h
  if row['exit_code']==0:raw[row['product']]=row['product_sha256']
 for row in k['runtime_overrides'].values():
  raw[row['path']]=row['sha256']
  for v in row['supplements']:raw[v['path']]=v['sha256']
 for row in k['runtime_overrides'].values():
  for rt in row.get('template_replacements',[]):raw[rt['before']]=rt['before_sha256'];raw[rt['after']]=rt['after_sha256']
  if row.get('replacement_template'):rt=row['replacement_template'];raw[rt['path']]=rt['sha256']
 for p in [Path(c['candidate']),Path(k['preparation_refinement']),Path(pf['original_source']),Path(__file__).resolve(),T/'update-master-report-E149-static2941-ready.py']:raw[str(p)]=sha(p)
 for m in k['modules'].values():raw[m['path']]=m['sha256'];raw[m['template']]=m['template_sha256']
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
  for p,h in raw.items():require(sha(p)==h,'raw source changed');z.write(p,Path(p).relative_to(R).as_posix())
 with zipfile.ZipFile(out/'raw.zip')as z:require(len(z.namelist())==len(raw) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in raw.items()),'raw archive differs')
 record=dict(schema='E149-static2941-ready-source-development-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,base_compiled_inputs=c['base_compiled_inputs'],base_compiled_inputs_sha256=sha(c['base_compiled_inputs']),compile_cohort=str(CP),compile_cohort_sha256=sha(CP),candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],normal_product_pins=c['normal_product_pins'],previous_development=ref,previous_qualified_18_metadata=fullref,current_source_coverage=cov,compiler_exit_codes=[0]*20,new_metadata_compilers=20,observed_original_tool_chunk=chunk,observed_actual_exit_code=0,original_process_observations=observations,normal_inputs_checked=1049,effective_overlay_files=44,runtime_files=30,Rust_QA_files=11,Java_files=3,module_count=372,new_tests_written=4,retained_cases_rewritten=8,previous_metadata_cases=471,total_written_unrun_cases=475,ready_gate_group_cases=12,ready_gate_group_resource_opt_in=4,new_resource_opt_in_cases=1,preparation_descriptor_new_resource_count=k['new_resource_opt_in_cases'],classification_derived_from_actual_QA=True,preparation_refinement=k['preparation_refinement'],preparation_refinement_sha256=k['preparation_refinement_sha256'],tests_executed=0,desktop_inputs=0,controller_queue_and_fresh_enqueue_sample_bound=True,source_ready_precedes_enqueue_source_bound=True,Scene_and_History_source_bound=True,cached_late_and_paint_token_checks_source_bound=True,all_native_global_callbacks_qualified=False,normal_apply_eligible=False,assets_decoded=False,production_applied=False,canonical_registry_selected=False,PEs_linked=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_archive_files=raw,current_fixture_binding_unchanged=previous['current_fixture_binding_unchanged'],current_formal_suite_unchanged=previous['current_formal_suite_unchanged'],canonical_registry_unchanged=previous['canonical_registry_unchanged'],note=NOTE,next_action=NEXT,whole_goal_complete=False);ep=out/'evidence.json';ep.write_bytes(encoded(record));published=dict(evidence=str(ep),evidence_sha256=sha(ep),candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],compile_cohort=str(CP),compile_cohort_sha256=sha(CP),status='static_ready_scene_history_source_bound_native_global_and_runtime_unqualified',metadata_exit_codes=[0]*20,tests_executed=0,whole_goal_complete=False)
 for n in CHECKPOINTS:
  p=R/n;require(sha(p)==before[n],'concurrent checkpoint writer')
  if n.endswith('.json'):
   j=read(p);j[KEY]=published;j['last_goal_turn_evidence']=str(ep);j['last_goal_turn_classification']='progress';j['next_development_action']=NEXT
   if n in ['工作佇列.json','監製進度.json']:j['current_focus']=dict(item='static2941_ready_gate_scene_history_bound_global_callbacks_pending',work=NOTE,next=NEXT,evidence=str(ep));j['next_task']=NEXT;j['progress_updated_utc']=datetime.now(timezone.utc).isoformat()
   p.write_bytes(encoded(j))
  else:p.write_text('2026-10-25：'+NOTE+' '+NEXT+' [解碼收據來源與20metadata](<'+ep.as_posix()+'>)。\n\n'+p.read_text('utf-8-sig'),encoding='utf-8',newline='\n')
 after={n:sha(R/n)for n in CHECKPOINTS};require(all(sha(R/n)==h for n,h in after.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files=after,evidence=str(ep),evidence_sha256=sha(ep),before_archive_sha256=sha(out/'before.zip'),tests_executed=0,whole_goal_complete=False)));print(json.dumps(dict(evidence=str(ep),sha256=sha(ep),readback=str(out/'readback.json'),compiler_exit_codes=[0]*20,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();main(a.observed_original_tool_chunk)
