"""Keep the pinned reader; use CIM birth only for an access-denied reused PID."""
from pathlib import Path
import argparse,importlib.util,json,subprocess,hashlib
from native_E149_recovery_contract import R,read,sha,encoded,require
T=R/'維護工具'
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def verify(qp,master,chunk):
 q=read(qp);path=T/'readback-native-E149-resurrection61.py';require(sha(path)==q['source_pins'][str(path)],'pinned reader differs');m=load('immutable_resurrection_reader',path);original_loader=m.module;fallbacks=[]
 def module(name,path):
  result=original_loader(name,path)
  if name=='birth_first_original_reader':
   native=result.birth_first
   def birth(row):
    try:return native(row)
    except RuntimeError as error:
     pid=int(row['pid']);require(pid>0,'invalid original PID');code="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';$taskRow=Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"';if($null -eq $taskRow){[pscustomobject]@{Pid="+str(pid)+";Present=$false}|ConvertTo-Json -Compress}else{[pscustomobject]@{Pid="+str(pid)+";Present=$true;CreationFileTime=$taskRow.CreationDate.ToUniversalTime().ToFileTimeUtc().ToString()}|ConvertTo-Json -Compress}"
     child=subprocess.run(['pwsh','-NoLogo','-NoProfile','-NonInteractive','-Command',code],capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);require(child.returncode==0 and child.stdout.strip(),'original PID CIM observation unavailable; no terminal claim');value=json.loads(child.stdout.decode('utf-8-sig'));require(value['Pid']==pid,'CIM PID differs')
     if value['Present']:
      ticks=int(value['CreationFileTime']);require(ticks//10!=row['creation_filetime_ticks']//10,'same-microsecond original PID remains unqueryable; no terminal claim');reason='PID reused with different CIM birth at microsecond precision'
     else:ticks=None;reason='original PID absent from successful CIM observation'
     observed=dict(pid=pid,original_not_live=True,reason=reason,original_creation_filetime_ticks=row['creation_filetime_ticks'],actual_CIM_creation_filetime_ticks=ticks,query_method='CIM_birth_fallback',original_query_error=str(error),query_exit_code=child.returncode,query_stdout_sha256=hashlib.sha256(child.stdout).hexdigest().upper(),query_result=value);fallbacks.append(observed);return observed
   result.birth_first=birth
  return result
 m.module=module;m.verify(qp,master,chunk);final=read(qp.with_name('final-readback.json'));require(final['original_processes_checked']==21 and all(r['original_not_live'] for r in final['original_process_observations']),'original process proof incomplete');result=dict(qualification=str(qp),qualification_sha256=sha(qp),pinned_reader=str(path),pinned_reader_sha256=sha(path),fallback_reader=str(Path(__file__).resolve()),fallback_reader_sha256=sha(Path(__file__).resolve()),original_reader_failure=str(qp.with_name('original-process-readback-failure.json')),original_reader_failure_sha256=sha(qp.with_name('original-process-readback-failure.json')),master_readback=str(master),observations=final['original_process_observations'],fallbacks=fallbacks,original_processes_checked=21,process_control_calls=0,restarts=0,tests_executed=0,whole_goal_complete=False);qp.with_name('cim-birth-process-readback.json').write_bytes(encoded(result));print(json.dumps({k:v for k,v in result.items() if k not in ['observations','fallbacks']},ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('qualification',type=Path);p.add_argument('--master-readback',type=Path,required=True);p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();verify(a.qualification,a.master_readback,a.observed_original_tool_chunk)
