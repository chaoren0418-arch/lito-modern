"""Independent readback of source/metadata/checkpoints, not a test execution."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,zipfile
from native_E149_recovery_contract import R,read,sha,encoded,require,check_pins
T=R/'維護工具'
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def verify(qp,master,chunk):
 q=read(qp);check_pins(q['source_pins']);check_pins(q['normal_product_pins']);require(q['status']=='unselected_S23_129_class7_scene_and_audio_metadata_only','stage differs')
 checker=module('eruption_scene_source_checker',T/'qualify-native-E149-eruption45-scene.py');c,k,proof,products=checker.validate(Path(q['compile_run'])/'cohort.json')
 require(q['observed_actual_exit_code']==0 and q['observed_original_tool_chunk']==chunk and q['compiler_exit_codes']==[0]*4 and q['metadata_products']==products,'original compiler receipt differs')
 processes=module('eruption_core_process_reader',T/'readback-native-E149-eruption45-core.py')
 observations=[processes.original_terminal(row) for row in [c['pipeline']]+[r['original_process'] for r in c['compiler_runs']]]
 failure=read(k['original_compile_failure']['cohort']);observations.extend(processes.original_terminal(row) for row in [failure['pipeline']]+[r['original_process'] for r in failure['compiler_runs']])
 require((q['effective_overlay_files'],q['resolved_module_path_attributes'],q['original_nested_paths_preserved'],q['new_tests_written'],q['retained_skill_tests_written'],q['new_opt_in_original_resource_cases'])==(33,365,6,10,165,2),'source inventory differs')
 require(q['tests_executed']==0 and q['modules_integrated_into_scene'] and q['native_scheduler_FIFO_static_binding'] and not q['request_registered'] and not q['child_hurt_dispatched'] and not q['ResourceDatabase_GAME_observed'] and not q['optional_action0_speed_and_uint32_rollover_qualified'],'source/runtime scope differs')
 require(q['current_source_coverage']['recognized_count']==59 and q['current_source_coverage']['missing_request_count']==121,'request scope differs')
 require(sha(q['raw_archive'])==q['raw_archive_sha256'],'raw archive changed')
 with zipfile.ZipFile(q['raw_archive']) as z:require(len(z.namelist())==len(q['raw_archive_files']) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in q['raw_archive_files'].items()),'raw archive differs')
 rb=read(qp.with_name('readback.json'));require(rb['qualification']==str(qp) and rb['qualification_sha256']==sha(qp) and len(rb['after_files'])==8 and all(sha(R/n)==h for n,h in rb['after_files'].items()),'eight checkpoints differ')
 with zipfile.ZipFile(qp.with_name('before.zip')) as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in rb['before_files'].items()),'before checkpoint archive differs')
 w=read(R/'工作佇列.json');ref=w['current_E149_eruption45_scene_development'];require(ref['qualification']==str(qp) and ref['qualification_sha256']==sha(qp) and w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase/current pointer differs')
 previous=read(k['predecessor_qualification']);require(q['current_fixture_binding_unchanged']==previous['current_fixture_binding_unchanged'] and q['current_formal_suite_unchanged']==previous['current_formal_suite_unchanged'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'G06/registry changed')
 report=R.parent/'LiTo381-總開發報告.md';m=read(master);require(sha(report)==m['after_sha256'] and report.read_bytes()==master.parent.joinpath('after.md').read_bytes(),'master report differs')
 text=report.read_text('utf-8-sig');require(all(s in text for s in ['#45地裂術接收呈現','175','33檔','尚未登記','hurt','測試與桌面操作暫緩']),'master report lacks current scope')
 result=dict(qualification=str(qp),qualification_sha256=sha(qp),original_process_observations=observations,metadata_commands=4,new_cases_written=10,tests_executed=0,normal_inputs_checked=1049,unchanged_registry_and_G06=True,
  eight_checkpoint_readback=str(qp.with_name('readback.json')),master_readback=str(master),master_report_sha256=sha(report),scene_source_integrated=True,request_registered=False,child_hurt_dispatched=False,whole_goal_complete=False)
 qp.with_name('final-readback.json').write_bytes(encoded(result));print(json.dumps({k:v for k,v in result.items() if k!='original_process_observations'},ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('qualification',type=Path);p.add_argument('--master-readback',type=Path,required=True);p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();verify(a.qualification,a.master_readback,a.observed_original_tool_chunk)
