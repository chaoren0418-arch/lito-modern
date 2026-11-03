"""Read original compiles, preserve recipe failure, publish source-only ingress progress."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,hashlib,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_process_birth_readonly import original_terminal
T=R/'維護工具';V=R/'驗證紀錄/current';KEY='current_E149_static_effect_ingress_development'
CP=V/'native-E149-static-effect-ingress-metadata-continuation/20261024-201510-822889/cohort.json'
CHECKPOINTS=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
NOTE='97的網路解碼完成clock已沿Event→LOGIN目前epoch／rx_counter檢查→World靜態S37 cue傳遞。IngressReceipt在worker完整解密後、channel backpressure前取Instant；World只為有效2941保存generation／sequence／target incarnation／position綁定，ring淘汰與world退役清除，無時間的accept不造clock，JSON略過本機時鐘。這是decode receipt，不是原PE clock或callback排程期限；constructor dispatch及fresh enqueue sample／Scene／History／全域queue尚未接，97仍未完成。候選43檔29 Runtime／11 Rust QA／3 Java、371module與原六nested；兩版library／library-test／八整合共20metadata皆0，其中原Debug兩成功保留，修正extern庫檔名前綴後只補18項。原三程序0／0／1與工具62a862失敗完整保留，不改QA以掩蓋cascading錯誤，未重跑已成功Debug兩項。新4純QA＋前459共463案全未跑，88／剩92不變。正常1049／既有PE／正式JAR／registry／G06640次未變；未套用／選入／PE連結／測試／資源解碼／桌面／播放。'
NEXT='先接97的controller dispatch／constructor時刻與實際fresh enqueue clock，再接零delay全域queue／Scene／History／owner退役，串起static bank／Over提交橋；不能把decode receipt、worker載入或paint時間當4D65F0 sample。保留同deadline順序與重入繪圖、兩suffix上游／4857B0巢狀drain；不固定一幀或借32秒期限。97完成後續99／100及剩92、4183與G06物品／source/account／timer、LT hurt/base。測試與桌面待明確恢復，之後先G06精確640次與當版完整library Debug／Release、Release回歸，再租裝等GUI，自行核對後交人工。'

def main(chunk):
 w=read(R/'工作佇列.json');require(KEY not in w and w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'source-only phase differs');c=read(CP);require(c['status']=='twenty_metadata_complete_two_retained_eighteen_new_no_execution' and c['retained_original_successes']==2 and c['remaining_compilers_started']==18 and c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compile scope differs');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency differs');k=read(c['candidate']);require(sha(c['candidate'])==c['candidate_sha256'],'candidate differs');require((k['overlay_count'],k['runtime_files'],k['Rust_QA_files'],k['Java_files'],k['module_count'],k['original_nested_paths_preserved'],k['new_tests_written'],k['previous_metadata_cases'])==(43,29,11,3,371,6,4,459),'inventory differs');require(k['network_decode_to_event_to_login_to_world_source_bound'] and k['static2941_cue_receipt_bound'] and not k['native_PE_clock_equivalence_qualified'] and not k['scene_history_and_retirement_queue_integrated'],'receipt boundary differs')
 tests=0
 for name,row in k['runtime_overrides'].items():
  require(sha(row['predecessor'])==row['predecessor_sha256'],'runtime predecessor differs');s=Path(row['predecessor']).read_text('utf-8-sig')
  for a,b in row['changes']:require(s.count(a)==1,'source delta ambiguous');s=s.replace(a,b)
  for extra in row['supplements']:require(sha(extra['path'])==extra['sha256'],'source supplement differs');v=Path(extra['path']).read_text('utf-8-sig');s+='\n'+v;tests+=v.count('#[test]')
  require(s==Path(row['path']).read_text('utf-8-sig') and sha(row['path'])==row['sha256'],'source reconstruction differs')
  if name in c['original_nested_runtime_pairs']:
   pair=c['original_nested_runtime_pairs'][name];require(pair['source']==row['path'] and pair['source_sha256']==row['sha256'],'paired source differs')
   for rw in pair['rewrites']:require(s.count(rw['original_attribute'])==1 and sha(rw['resolved_path'])==c['input_pins'][rw['resolved_path']],'nested path differs');s=s.replace(rw['original_attribute'],rw['compiled_attribute'])
   require(s==Path(pair['path']).read_text('utf-8-sig') and sha(pair['path'])==pair['sha256'],'paired runtime differs')
 require(tests==4 and len(k['effective_overlay_files'])==43 and all(sha(k['effective_overlay_paths'][n])==h for n,h in k['effective_overlay_files'].items()) and sum(len(v['rewrites'])for v in c['original_nested_runtime_pairs'].values())==6,'effective source differs')
 kinds=['library','library-test',*c['paired_tests']];require(len(kinds)==10 and [(r['profile'],r['kind'])for r in c['compiler_runs']]==[(p,n)for p in ['Debug','Release']for n in kinds],'command inventory differs')
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['original_handle_signaled'] and '--emit=metadata'in row['command'] and ('--test'in row['command'])==(row['kind']!='library') and sha(row['product'])==row['product_sha256'] and all(sha(p)==h for p,h in row['logs'].items()),'actual metadata product/log differs')
  if row['kind']in c['paired_tests']:
   extern=next(n for n in row['command']if n.startswith('lito_modern='));require(Path(extern.split('=',1)[1]).name.startswith('liblito_modern-'),'extern library basename differs')
 f=read(c['original_recipe_failure']);first=read(f['original_cohort']);require(sha(c['original_recipe_failure'])==c['original_recipe_failure_sha256'] and sha(f['original_cohort'])==f['original_cohort_sha256'] and f['observed_original_tool_chunk']=='62a862' and f['observed_actual_exit_code']==1 and f['source_and_QA_changes']==0 and [r['exit_code']for r in first['compiler_runs']]==[0,0,1],'original failure differs');require(c['compiler_runs'][:2][0]['command']==first['compiler_runs'][0]['command'] and c['compiler_runs'][1]['command']==first['compiler_runs'][1]['command'] and sha(c['retained_Debug_library_alias'])==c['retained_Debug_library_alias_sha256']==first['compiler_runs'][0]['product_sha256'],'retained compiles differ')
 identities=[c['pipeline']]+[r['original_process']for r in c['compiler_runs']]+[first['pipeline'],first['compiler_runs'][2]['original_process']];require(len({(v['pid'],v['creation_filetime_ticks'])for v in identities})==23,'original process inventory differs');observations=[original_terminal(v)for v in identities]
 ref=w['current_E149_hide97_static_draw_bridge_development'];require(k['predecessor_source']==ref and sha(ref['evidence'])==ref['evidence_sha256'],'previous development differs');previous=read(ref['evidence']);fullref=previous['previous_qualified_18_metadata'];fullq=read(fullref['qualification']);require(sha(fullref['qualification'])==fullref['qualification_sha256'],'prior full metadata qualification differs');base=read(k['predecessor_candidate']);require(sha(k['predecessor_candidate'])==k['predecessor_candidate_sha256'],'prior candidate differs')
 for n in ['LiTo381-Modern/src/modern_lito/native_skills.rs','LiTo381-Modern/src/modern_lito/native_skill_selection.rs']:require(k['effective_overlay_files'][n]==base['effective_overlay_files'][n],'request/menu source changed')
 cov=fullq['current_source_coverage'];require(cov['recognized_count']==88 and cov['missing_request_count']==92,'coverage differs');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049,'normal count differs');check_pins({str(R/n):h for n,h in inputs.items()});require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and previous['canonical_registry_unchanged']==w['current_isolated_development']['candidates'] and previous['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and previous['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'],'formal/G06/registry differs')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(set(before)==set(CHECKPOINTS) and all(sha(R/n)==h for n,h in before.items()),'checkpoint differs');out=V/'native-E149-static-effect-ingress-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
  for n in CHECKPOINTS:z.write(R/n,n)
 raw={}
 for folder in [CP.parent,Path(f['original_cohort']).parent]:
  for p in folder.iterdir():
   if p.is_file():raw[str(p)]=sha(p)
 for row in c['compiler_runs']+first['compiler_runs']:
  for p,h in row['logs'].items():raw[p]=h
  if row['exit_code']==0:raw[row['product']]=row['product_sha256']
 for row in k['runtime_overrides'].values():
  raw[row['path']]=row['sha256']
  for v in row['supplements']:raw[v['path']]=v['sha256']
 for p in [Path(c['candidate']),Path(c['retained_Debug_library_alias']),Path(__file__).resolve(),T/'update-master-report-E149-static-effect-ingress.py']:raw[str(p)]=sha(p)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
  for p,h in raw.items():require(sha(p)==h,'raw source changed');z.write(p,Path(p).relative_to(R).as_posix())
 with zipfile.ZipFile(out/'raw.zip')as z:require(len(z.namelist())==len(raw) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in raw.items()),'raw archive differs')
 record=dict(schema='E149-static-effect-ingress-source-development-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,base_compiled_inputs=c['base_compiled_inputs'],base_compiled_inputs_sha256=sha(c['base_compiled_inputs']),compile_cohort=str(CP),compile_cohort_sha256=sha(CP),candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],normal_product_pins=c['normal_product_pins'],previous_development=ref,previous_qualified_18_metadata=fullref,current_source_coverage=cov,compiler_exit_codes=[0]*20,retained_original_debug_compiles=2,new_metadata_compilers=18,observed_original_tool_chunk=chunk,observed_actual_exit_code=0,original_recipe_failure=c['original_recipe_failure'],original_recipe_failure_sha256=c['original_recipe_failure_sha256'],original_process_observations=observations,normal_inputs_checked=1049,effective_overlay_files=43,runtime_files=29,Rust_QA_files=11,Java_files=3,module_count=371,new_tests_written=4,previous_metadata_cases=459,total_written_unrun_cases=463,tests_executed=0,desktop_inputs=0,decoded_clock_forwarding_source_bound=True,decode_receipt_is_not_enqueue_clock=True,constructor_dispatch_and_fresh_enqueue_sample_pending=True,S37_scene_history_and_retirement_queue_integrated=False,assets_decoded=False,production_applied=False,canonical_registry_selected=False,PEs_linked=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_archive_files=raw,current_fixture_binding_unchanged=previous['current_fixture_binding_unchanged'],current_formal_suite_unchanged=previous['current_formal_suite_unchanged'],canonical_registry_unchanged=previous['canonical_registry_unchanged'],note=NOTE,next_action=NEXT,whole_goal_complete=False);ep=out/'evidence.json';ep.write_bytes(encoded(record));published=dict(evidence=str(ep),evidence_sha256=sha(ep),candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],compile_cohort=str(CP),compile_cohort_sha256=sha(CP),status='worker_decode_receipt_bound_twenty_metadata_no_runtime',metadata_exit_codes=[0]*20,tests_executed=0,whole_goal_complete=False)
 for n in CHECKPOINTS:
  p=R/n;require(sha(p)==before[n],'concurrent checkpoint writer')
  if n.endswith('.json'):
   j=read(p);j[KEY]=published;j['last_goal_turn_evidence']=str(ep);j['last_goal_turn_classification']='progress';j['next_development_action']=NEXT
   if n in ['工作佇列.json','監製進度.json']:j['current_focus']=dict(item='static2941_worker_receipt_bound_dispatch_queue_pending',work=NOTE,next=NEXT,evidence=str(ep));j['next_task']=NEXT;j['progress_updated_utc']=datetime.now(timezone.utc).isoformat()
   p.write_bytes(encoded(j))
  else:p.write_text('2026-10-25：'+NOTE+' '+NEXT+' [解碼收據來源與20metadata](<'+ep.as_posix()+'>)。\n\n'+p.read_text('utf-8-sig'),encoding='utf-8',newline='\n')
 after={n:sha(R/n)for n in CHECKPOINTS};require(all(sha(R/n)==h for n,h in after.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files=after,evidence=str(ep),evidence_sha256=sha(ep),before_archive_sha256=sha(out/'before.zip'),tests_executed=0,whole_goal_complete=False)));print(json.dumps(dict(evidence=str(ep),sha256=sha(ep),readback=str(out/'readback.json'),compiler_exit_codes=[0]*20,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();main(a.observed_original_tool_chunk)
