"""Independent file/process/artifact readback. Does not execute a test body."""
from pathlib import Path
import argparse, ctypes, hashlib, json, zipfile
from native_E149_recovery_contract import R,read,sha,encoded,require,check_pins
from native_E149_owned_process import kernel,identity
import importlib.util

def original_terminal(row):
 k=kernel();handle=k.OpenProcess(0x1000|0x100000,False,row['pid'])
 if not handle:
  error=ctypes.get_last_error();require(error==87,'original process unqueryable, not proved absent');return dict(pid=row['pid'],original_not_live=True,reason='PID absent')
 try:
  actual=identity(handle,row['pid'])
  if actual['creation_filetime_ticks']!=row['creation_filetime_ticks']:return dict(pid=row['pid'],original_not_live=True,reason='PID reused with different birth')
  require(actual==row and k.WaitForSingleObject(handle,0)==0,'original process still live or image changed')
  return dict(pid=row['pid'],original_not_live=True,reason='original handle signaled')
 finally:require(k.CloseHandle(handle),'query handle close failed')

def verify(qp,master):
 q=read(qp);require(q['status']=='unselected_eruption45_core_and_shared_assets_metadata_only','stage differs');check_pins(q['source_pins']);check_pins(q['normal_product_pins'])
 spec=importlib.util.spec_from_file_location('eruption45_source_qualifier',Path(__file__).with_name('qualify-native-E149-eruption45-core.py'));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
 c,contract,products=module.validate(Path(q['compile_run'])/'cohort.json')
 require(q['observed_actual_exit_code']==0 and q['observed_original_tool_chunk']=='eb5dd7' and q['compiler_exit_codes']==[0]*4 and q['metadata_products']==products,'compiler receipt differs')
 processes=[original_terminal(row) for row in [c['pipeline']]+[r['original_process'] for r in c['compiler_runs']]]
 require(q['new_tests_written']==10 and q['retained_skill_tests_written']==155 and q['tests_executed']==0 and not q['modules_integrated_into_scene'] and not q['request_registered'] and not q['native_scheduler_tie_order_qualified'],'scope differs')
 require(q['current_source_coverage']['recognized_count']==59 and q['current_source_coverage']['missing_request_count']==121,'recognition scope differs')
 require(sha(q['raw_archive'])==q['raw_archive_sha256'],'raw archive changed')
 with zipfile.ZipFile(q['raw_archive']) as z:
  require(len(z.namelist())==len(q['raw_archive_files']) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in q['raw_archive_files'].items()),'raw archive entries differ')
 rb=read(qp.with_name('readback.json'));require(rb['qualification']==str(qp) and rb['qualification_sha256']==sha(qp) and len(rb['after_files'])==8 and all(sha(R/n)==h for n,h in rb['after_files'].items()),'eight checkpoints differ')
 with zipfile.ZipFile(qp.with_name('before.zip')) as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in rb['before_files'].items()),'before checkpoint differs')
 w=read(R/'工作佇列.json');ref=w['current_E149_eruption45_core_development'];require(ref['qualification']==str(qp) and ref['qualification_sha256']==sha(qp),'current stage pointer differs')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase/workload changed')
 previous=read(contract['predecessor_qualification']);require(q['current_fixture_binding_unchanged']==previous['current_fixture_binding_unchanged'] and q['current_formal_suite_unchanged']==previous['current_formal_suite_unchanged'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'G06/canonical registry differs')
 report=R.parent/'LiTo381-總開發報告.md';m=read(master);require(sha(report)==m['after_sha256'] and master.parent.joinpath('after.md').read_bytes()==report.read_bytes(),'master report differs')
 text=report.read_text('utf-8-sig');require('#45地裂術' in text and '165' in text and '尚未接入場景' in text and '測試與桌面操作暫緩' in text,'master report lacks scope/continuation')
 result=dict(qualification=str(qp),qualification_sha256=sha(qp),compiler_originals_not_live=processes,metadata_commands=4,tests_executed=0,normal_inputs_checked=1049,unchanged_registry_and_G06=True,eight_checkpoint_readback=str(qp.with_name('readback.json')),master_readback=str(master),master_report_sha256=sha(report),modules_integrated_into_scene=False,whole_goal_complete=False)
 qp.with_name('final-readback.json').write_bytes(encoded(result));print(json.dumps({k:v for k,v in result.items() if k!='compiler_originals_not_live'},ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('qualification',type=Path);p.add_argument('--master-readback',type=Path,required=True);a=p.parse_args();verify(a.qualification,a.master_readback)
