"""Publish an isolated static queue prototype; resource-ready and scene gates remain open."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,hashlib,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_process_birth_readonly import original_terminal
T=R/'維護工具';V=R/'驗證紀錄/current';KEY='current_E149_static2941_queue_development'
CP=V/'native-E149-static2941-queue-metadata/20261024-203626-169258/cohort.json'
CHECKPOINTS=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
NOTE='97隔離專用queue已接Controller：區分decode／dispatch／候選constructor／fresh enqueue四時刻，zero-delay先排隊、同deadline穩定順序、poll批次後固定sample逐筆pop再退役；不由paint推時鐘。World快照只共享不可復活的Arc／Atomic存活token，ring淘汰、connection／generation換代、五處world清除、state reset與Queue drop均退役；seen改單一watermark，不無限累積。這仍是不可套用正常來源的中間原型：資源就緒先於enqueue未接，與原4E5C00先SPR再4D65F0不同；Scene／History及原共享callback／重入繪圖也未接，不能宣告97完成或無像素。44檔30 Runtime／11 Rust QA／3 Java、372module／原六nested，20metadata均0；新8純QA＋前463共471案全未跑，88／剩92不變。正常1049／PE／JAR／registry／G06640次不變，未套用／選入／PE連結／測試／桌面／解碼資源／播放。'
NEXT='先把97的資源就緒gate接在實際enqueue之前：原4E5C00載靜態SPR後才4D65F0取fresh clock；目前專用queue原型先排程，不能當原先後已等價或直接套用。保留controller-owned token與observer撤銷，串接static bank／Scene／History及cached／late-worker的live-token檢查，補原共享callback／重入繪圖與巢狀drain，不固定一幀或借32秒期限。97完成再99／100及剩92、4183與G06物品／source/account／timer、LT hurt/base。測試與桌面待明確恢復，完整G06640次、library Debug／Release、Release與租裝等GUI依總報告，自驗後交人工。'

def main(chunk):
 w=read(R/'工作佇列.json');require(KEY not in w and w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'source-only phase differs');c=read(CP);require(c['status']=='twenty_metadata_complete_no_execution' and c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compile scope differs');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency differs');k=read(c['candidate']);require(sha(c['candidate'])==c['candidate_sha256'],'candidate differs');require((k['overlay_count'],k['runtime_files'],k['Rust_QA_files'],k['Java_files'],k['module_count'],k['original_nested_paths_preserved'],k['new_tests_written'],k['previous_metadata_cases'])==(44,30,11,3,372,6,8,463),'inventory differs');require(k['network_decode_to_event_to_login_to_world_source_bound'] and k['static2941_cue_receipt_bound'] and not k['native_PE_clock_equivalence_qualified'] and not k['scene_history_and_retirement_queue_integrated'],'receipt boundary differs')
 tests=0
 for name,row in k['runtime_overrides'].items():
  require(sha(row['predecessor'])==row['predecessor_sha256'],'runtime predecessor differs');s=Path(row['predecessor']).read_text('utf-8-sig')
  for a,b in row['changes']:require(s.count(a)==row.get('repeated_replacement_counts',{}).get(a,1),'source delta ambiguous');s=s.replace(a,b)
  for extra in row['supplements']:require(sha(extra['path'])==extra['sha256'],'source supplement differs');v=Path(extra['path']).read_text('utf-8-sig');s+='\n'+v;tests+=v.count('#[test]')
  require(s==Path(row['path']).read_text('utf-8-sig') and sha(row['path'])==row['sha256'],'source reconstruction differs')
  if name in c['original_nested_runtime_pairs']:
   pair=c['original_nested_runtime_pairs'][name];require(pair['source']==row['path'] and pair['source_sha256']==row['sha256'],'paired source differs')
   for rw in pair['rewrites']:require(s.count(rw['original_attribute'])==1 and sha(rw['resolved_path'])==c['input_pins'][rw['resolved_path']],'nested path differs');s=s.replace(rw['original_attribute'],rw['compiled_attribute'])
   require(s==Path(pair['path']).read_text('utf-8-sig') and sha(pair['path'])==pair['sha256'],'paired runtime differs')
 require(tests==8 and len(k['effective_overlay_files'])==44 and all(sha(k['effective_overlay_paths'][n])==h for n,h in k['effective_overlay_files'].items()) and sum(len(v['rewrites'])for v in c['original_nested_runtime_pairs'].values())==6,'effective source differs')
 for m in k['modules'].values():require(sha(m['path'])==m['sha256'] and sha(m['template'])==m['template_sha256'] and Path(m['path']).read_bytes()==Path(m['template']).read_bytes(),'new queue module differs')
 require(k['controller_queue_and_fresh_enqueue_sample_bound'] and k['lease_revocation_shared_with_snapshots'] and not k['scene_and_history_consumers_bound'] and not k['resource_ready_before_enqueue_qualified'] and not k['all_native_global_callbacks_qualified'],'prototype gates differ')
 kinds=['library','library-test',*c['paired_tests']];require(len(kinds)==10 and [(r['profile'],r['kind'])for r in c['compiler_runs']]==[(p,n)for p in ['Debug','Release']for n in kinds],'command inventory differs')
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['original_handle_signaled'] and '--emit=metadata'in row['command'] and ('--test'in row['command'])==(row['kind']!='library') and sha(row['product'])==row['product_sha256'] and all(sha(p)==h for p,h in row['logs'].items()),'actual metadata product/log differs')
  if row['kind']in c['paired_tests']:
   extern=next(n for n in row['command']if n.startswith('lito_modern='));require(Path(extern.split('=',1)[1]).name.startswith('liblito_modern-'),'extern library basename differs')
 identities=[c['pipeline']]+[r['original_process']for r in c['compiler_runs']];require(len({(v['pid'],v['creation_filetime_ticks'])for v in identities})==21,'original process inventory differs');observations=[original_terminal(v)for v in identities]
 ref=w['current_E149_static_effect_ingress_development'];require(k['predecessor_source']==ref and sha(ref['evidence'])==ref['evidence_sha256'],'previous development differs');previous=read(ref['evidence']);fullref=previous['previous_qualified_18_metadata'];fullq=read(fullref['qualification']);require(sha(fullref['qualification'])==fullref['qualification_sha256'],'prior full metadata qualification differs');base=read(k['predecessor_candidate']);require(sha(k['predecessor_candidate'])==k['predecessor_candidate_sha256'],'prior candidate differs')
 for n in ['LiTo381-Modern/src/modern_lito/native_skills.rs','LiTo381-Modern/src/modern_lito/native_skill_selection.rs']:require(k['effective_overlay_files'][n]==base['effective_overlay_files'][n],'request/menu source changed')
 cov=fullq['current_source_coverage'];require(cov['recognized_count']==88 and cov['missing_request_count']==92,'coverage differs');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049,'normal count differs');check_pins({str(R/n):h for n,h in inputs.items()});require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and previous['canonical_registry_unchanged']==w['current_isolated_development']['candidates'] and previous['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and previous['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'],'formal/G06/registry differs')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(set(before)==set(CHECKPOINTS) and all(sha(R/n)==h for n,h in before.items()),'checkpoint differs');out=V/'native-E149-static2941-queue-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
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
 for p in [Path(c['candidate']),Path(__file__).resolve(),T/'update-master-report-E149-static2941-queue.py']:raw[str(p)]=sha(p)
 for m in k['modules'].values():raw[m['path']]=m['sha256'];raw[m['template']]=m['template_sha256']
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
  for p,h in raw.items():require(sha(p)==h,'raw source changed');z.write(p,Path(p).relative_to(R).as_posix())
 with zipfile.ZipFile(out/'raw.zip')as z:require(len(z.namelist())==len(raw) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in raw.items()),'raw archive differs')
 record=dict(schema='E149-static2941-queue-prototype-development-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,base_compiled_inputs=c['base_compiled_inputs'],base_compiled_inputs_sha256=sha(c['base_compiled_inputs']),compile_cohort=str(CP),compile_cohort_sha256=sha(CP),candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],normal_product_pins=c['normal_product_pins'],previous_development=ref,previous_qualified_18_metadata=fullref,current_source_coverage=cov,compiler_exit_codes=[0]*20,new_metadata_compilers=20,observed_original_tool_chunk=chunk,observed_actual_exit_code=0,original_process_observations=observations,normal_inputs_checked=1049,effective_overlay_files=44,runtime_files=30,Rust_QA_files=11,Java_files=3,module_count=372,new_tests_written=8,previous_metadata_cases=463,total_written_unrun_cases=471,tests_executed=0,desktop_inputs=0,controller_queue_and_fresh_enqueue_sample_bound=True,lease_revocation_shared_with_snapshots=True,source_ready_precedes_enqueue_qualified=False,all_native_global_callbacks_qualified=False,Scene_and_History_bound=False,normal_apply_eligible=False,assets_decoded=False,production_applied=False,canonical_registry_selected=False,PEs_linked=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_archive_files=raw,current_fixture_binding_unchanged=previous['current_fixture_binding_unchanged'],current_formal_suite_unchanged=previous['current_formal_suite_unchanged'],canonical_registry_unchanged=previous['canonical_registry_unchanged'],note=NOTE,next_action=NEXT,whole_goal_complete=False);ep=out/'evidence.json';ep.write_bytes(encoded(record));published=dict(evidence=str(ep),evidence_sha256=sha(ep),candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],compile_cohort=str(CP),compile_cohort_sha256=sha(CP),status='isolated_static_queue_prototype_resource_order_and_scene_pending',metadata_exit_codes=[0]*20,tests_executed=0,whole_goal_complete=False)
 for n in CHECKPOINTS:
  p=R/n;require(sha(p)==before[n],'concurrent checkpoint writer')
  if n.endswith('.json'):
   j=read(p);j[KEY]=published;j['last_goal_turn_evidence']=str(ep);j['last_goal_turn_classification']='progress';j['next_development_action']=NEXT
   if n in ['工作佇列.json','監製進度.json']:j['current_focus']=dict(item='static2941_controller_queue_prototype_resource_gate_pending',work=NOTE,next=NEXT,evidence=str(ep));j['next_task']=NEXT;j['progress_updated_utc']=datetime.now(timezone.utc).isoformat()
   p.write_bytes(encoded(j))
  else:p.write_text('2026-10-25：'+NOTE+' '+NEXT+' [解碼收據來源與20metadata](<'+ep.as_posix()+'>)。\n\n'+p.read_text('utf-8-sig'),encoding='utf-8',newline='\n')
 after={n:sha(R/n)for n in CHECKPOINTS};require(all(sha(R/n)==h for n,h in after.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files=after,evidence=str(ep),evidence_sha256=sha(ep),before_archive_sha256=sha(out/'before.zip'),tests_executed=0,whole_goal_complete=False)));print(json.dumps(dict(evidence=str(ep),sha256=sha(ep),readback=str(out/'readback.json'),compiler_exit_codes=[0]*20,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();main(a.observed_original_tool_chunk)
