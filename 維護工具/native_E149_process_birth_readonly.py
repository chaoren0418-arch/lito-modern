"""Read original creation time before any image lookup; never control a process."""
import ctypes,json,subprocess,hashlib
from ctypes import wintypes
from native_E149_owned_process import kernel
from native_E149_recovery_contract import require
def original_terminal(row):
 k=kernel();pid=int(row['pid']);require(pid>0,'invalid original PID');handle=k.OpenProcess(0x1000|0x100000,False,pid)
 if not handle:
  error=ctypes.get_last_error()
  if error==87:return dict(pid=pid,original_not_live=True,reason='PID absent',query_method='native_birth_first')
  code="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';$taskRow=Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"';if($null -eq $taskRow){[pscustomobject]@{Pid="+str(pid)+";Present=$false}|ConvertTo-Json -Compress}else{[pscustomobject]@{Pid="+str(pid)+";Present=$true;CreationFileTime=$taskRow.CreationDate.ToUniversalTime().ToFileTimeUtc().ToString()}|ConvertTo-Json -Compress}"
  result=subprocess.run(['pwsh','-NoLogo','-NoProfile','-NonInteractive','-Command',code],capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);require(result.returncode==0 and result.stdout.strip(),'original CIM observation unavailable; no terminal claim');value=json.loads(result.stdout.decode('utf-8-sig'));require(value['Pid']==pid,'CIM PID differs');ticks=int(value['CreationFileTime']) if value['Present'] else None
  require(ticks is None or ticks//10!=row['creation_filetime_ticks']//10,'same-microsecond original remains unqueryable; no terminal claim')
  return dict(pid=pid,original_not_live=True,reason='PID reused with different CIM birth' if ticks is not None else 'PID absent from successful CIM query',query_method='CIM_birth_fallback',native_open_error=error,original_creation_filetime_ticks=row['creation_filetime_ticks'],actual_CIM_creation_filetime_ticks=ticks,query_exit_code=0,query_stdout_sha256=hashlib.sha256(result.stdout).hexdigest().upper(),query_result=value)
 try:
  times=[wintypes.FILETIME() for _ in range(4)];require(k.GetProcessTimes(handle,*[ctypes.byref(v) for v in times]),'original creation time unavailable; no terminal claim');ticks=times[0].dwLowDateTime|times[0].dwHighDateTime<<32
  if ticks!=row['creation_filetime_ticks']:return dict(pid=pid,original_not_live=True,reason='PID reused with different birth',original_creation_filetime_ticks=row['creation_filetime_ticks'],actual_creation_filetime_ticks=ticks,query_method='native_birth_first')
  wait=k.WaitForSingleObject(handle,0);require(wait==0,'same-birth original is not signaled; no restart or terminal claim');return dict(pid=pid,original_not_live=True,reason='same-birth original handle signaled',creation_filetime_ticks=ticks,wait_result=wait,query_method='native_birth_first')
 finally:require(k.CloseHandle(handle),'original query handle close failed')
