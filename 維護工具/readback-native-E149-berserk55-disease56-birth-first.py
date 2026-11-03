"""Read original PID/FILETIME before image query; no process control or rerun."""
from pathlib import Path
import argparse,ctypes,importlib.util,json
from ctypes import wintypes
from native_E149_recovery_contract import R,read,sha,encoded,require
from native_E149_owned_process import kernel
T=R/'維護工具'
def birth_first(row):
 k=kernel();handle=k.OpenProcess(0x1000|0x100000,False,row['pid'])
 if not handle:
  error=ctypes.get_last_error();require(error==87,'PID unqueryable; absence not proved');return dict(pid=row['pid'],original_not_live=True,reason='PID absent',query_method='birth_first')
 try:
  times=[wintypes.FILETIME() for _ in range(4)];require(k.GetProcessTimes(handle,*[ctypes.byref(v) for v in times]),'process FILETIME unavailable; no terminal claim');ticks=times[0].dwLowDateTime|times[0].dwHighDateTime<<32
  if ticks!=row['creation_filetime_ticks']:return dict(pid=row['pid'],original_not_live=True,reason='PID reused with different birth',actual_creation_filetime_ticks=ticks,original_creation_filetime_ticks=row['creation_filetime_ticks'],query_method='birth_first')
  wait=k.WaitForSingleObject(handle,0);require(wait==0,'same-birth original process not signaled; no restart or terminal claim');return dict(pid=row['pid'],original_not_live=True,reason='same PID and FILETIME original handle signaled',creation_filetime_ticks=ticks,wait_result=wait,query_method='birth_first')
 finally:require(k.CloseHandle(handle),'query handle close failed')
def verify(qp,master,chunk):
 q=read(qp);path=T/'readback-native-E149-berserk55-disease56.py';require(sha(path)==q['source_pins'][str(path)],'pinned stage reader differs');spec=importlib.util.spec_from_file_location('immutable_55_56_stage_reader',path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);original_loader=m.module
 def load(name,path):
  result=original_loader(name,path)
  if name=='sun_weak_original_process_reader':require(sha(path)==q['source_pins'][str(path)],'original process reader changed');result.original_terminal=birth_first
  return result
 m.module=load;m.verify(qp,master,chunk)
 final=read(qp.with_name('final-readback.json'));require(len(final['original_process_observations'])==22 and all(v['original_not_live'] for v in final['original_process_observations']),'original processes not all proved terminal')
 result=dict(qualification=str(qp),qualification_sha256=sha(qp),immutable_stage_reader=str(path),immutable_stage_reader_sha256=sha(path),birth_first_reader=str(Path(__file__).resolve()),birth_first_reader_sha256=sha(Path(__file__).resolve()),master_readback=str(master),original_processes_checked=22,observations=final['original_process_observations'],previous_image_query_failures=[dict(tool_chunk='16aa3c',actual_exit_code=1),dict(tool_chunk='cd46fd',actual_exit_code=1)],reason='Compare original creation FILETIME before any image access; a reused PID cannot require querying an unrelated image',process_control_calls=0,restarts=0,tests_executed=0,whole_goal_complete=False)
 qp.with_name('birth-first-process-readback.json').write_bytes(encoded(result));print(json.dumps({k:v for k,v in result.items() if k!='observations'},ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('qualification',type=Path);p.add_argument('--master-readback',type=Path,required=True);p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();verify(a.qualification,a.master_readback,a.observed_original_tool_chunk)
