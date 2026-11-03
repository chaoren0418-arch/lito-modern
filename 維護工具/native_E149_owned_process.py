"""Original Windows process handles for owned CLI fixtures; no UI APIs.

Importing this module starts nothing. Kernel calls are lazy. A PID alone is
never an authority to terminate: adopted handles require exact recorded birth,
image/hash and command, constrained by the caller's reviewed case namespace.
"""
from pathlib import Path
from ctypes import wintypes
from datetime import datetime,timezone,timedelta
import ctypes,hashlib,json,os,subprocess

def require(ok,message):
 if not ok:raise RuntimeError(message)

def sha(path):
 with Path(path).open('rb') as file:return hashlib.file_digest(file,'sha256').hexdigest().upper()

def kernel():
 k=ctypes.WinDLL('kernel32',use_last_error=True)
 k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
 k.QueryFullProcessImageNameW.argtypes=[wintypes.HANDLE,wintypes.DWORD,wintypes.LPWSTR,ctypes.POINTER(wintypes.DWORD)];k.QueryFullProcessImageNameW.restype=wintypes.BOOL
 k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE
 k.CloseHandle.argtypes=[wintypes.HANDLE];k.CloseHandle.restype=wintypes.BOOL
 k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.WaitForSingleObject.restype=wintypes.DWORD
 k.GetExitCodeProcess.argtypes=[wintypes.HANDLE,ctypes.POINTER(wintypes.DWORD)];k.GetExitCodeProcess.restype=wintypes.BOOL
 k.TerminateProcess.argtypes=[wintypes.HANDLE,wintypes.UINT];k.TerminateProcess.restype=wintypes.BOOL
 k.GetCurrentProcess.restype=wintypes.HANDLE
 return k

def identity(handle,pid):
 k=kernel();times=[wintypes.FILETIME() for _ in range(4)]
 require(k.GetProcessTimes(handle,*[ctypes.byref(t) for t in times]),'process birth unavailable')
 ticks=times[0].dwLowDateTime|times[0].dwHighDateTime<<32
 length=wintypes.DWORD(32768);buffer=ctypes.create_unicode_buffer(length.value)
 require(k.QueryFullProcessImageNameW(handle,0,buffer,ctypes.byref(length)),'process image unavailable')
 executable=Path(buffer.value).resolve()
 return dict(pid=int(pid),creation_filetime_ticks=ticks,start_epoch_millis=(ticks-116444736000000000)//10000,
  creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),
  executable=str(executable),executable_sha256=sha(executable))

def command_line(pid):
 pid=int(pid);require(pid>0,'invalid process PID')
 code="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object CommandLine|ConvertTo-Json -Compress"
 result=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',code],capture_output=True,timeout=8,creationflags=subprocess.CREATE_NO_WINDOW)
 require(result.returncode==0 and result.stdout.strip(),'original command line unavailable')
 command=json.loads(result.stdout.decode('utf-8-sig'))['CommandLine']
 shell=ctypes.WinDLL('shell32',use_last_error=True)
 shell.CommandLineToArgvW.argtypes=[wintypes.LPCWSTR,ctypes.POINTER(ctypes.c_int)]
 shell.CommandLineToArgvW.restype=ctypes.POINTER(wintypes.LPWSTR)
 count=ctypes.c_int();pointer=shell.CommandLineToArgvW(command,ctypes.byref(count));require(pointer,'command parse failed')
 k=kernel();k.LocalFree.argtypes=[ctypes.c_void_p];k.LocalFree.restype=ctypes.c_void_p
 try:return [pointer[i] for i in range(count.value)]
 finally:k.LocalFree(ctypes.cast(pointer,ctypes.c_void_p))

class HeldProcess:
 def __init__(self,handle,row,close_handle,popen=None):
  self.handle,self.row,self.close_handle,self.popen=handle,row,close_handle,popen

 @classmethod
 def spawned(cls,proc,command,expected_image_sha,kind):
  handle=int(proc._handle);actual=identity(handle,proc.pid)
  require(Path(actual['executable'])==Path(command[0]).resolve() and actual['executable_sha256']==expected_image_sha,'spawned image differs')
  row=dict(actual,kind=kind,command=list(command),wait_confirmed=False,exit_code=None)
  return cls(handle,row,False,proc)

 @classmethod
 def adopt(cls,record,command,allowed_images):
  # The caller first validates the original DB producer's durable record,
  # executable whitelist and datadir/loopback namespace; none are inferred.
  require(Path(record['executable']).resolve() in allowed_images,'adopted image is outside reviewed whitelist')
  require(record['executable_sha256']==allowed_images[Path(record['executable']).resolve()],'adopted image digest differs')
  k=kernel();handle=k.OpenProcess(0x1000|0x100000|0x0001,False,int(record['pid']))
  require(handle,'recorded original process is absent or unqueryable')
  try:
   actual=identity(handle,record['pid'])
   for key in ['pid','creation_filetime_ticks','start_epoch_millis','executable_sha256']:
    require(actual[key]==record[key],'recorded PID was reused or image changed')
   require(Path(actual['executable'])==Path(record['executable']).resolve(),'recorded image path differs')
   require(command_line(record['pid'])==command,'recorded process command/namespace differs')
   again=identity(handle,record['pid']);require(again==actual,'identity changed during observation')
   return cls(handle,dict(actual,command=list(command),wait_confirmed=False,exit_code=None),True)
  except BaseException:
   k.CloseHandle(handle);raise

 def terminal(self):
  code=kernel().WaitForSingleObject(self.handle,0)
  require(code in [0,258],'original handle wait observation failed')
  return code==0

 def collect_exit(self):
  require(self.terminal(),'original process is still live')
  code=wintypes.DWORD();require(kernel().GetExitCodeProcess(self.handle,ctypes.byref(code)),'original exit code unavailable')
  if self.popen is not None:self.popen.wait(timeout=1)
  self.row.update(wait_confirmed=True,exit_code=int(code.value));return self.row

 def wait(self,seconds):
  code=kernel().WaitForSingleObject(self.handle,max(0,int(seconds*1000)))
  require(code in [0,258],'original handle wait failed')
  if code==0:return self.collect_exit()
  raise TimeoutError('same original process is still live')

 def terminate_original(self,reason):
  if not self.terminal():
   # A held kernel object cannot turn into a new process when a PID is reused.
   # Its ownership was established before this method became available.
   require(kernel().TerminateProcess(self.handle,1),'exact owned handle termination failed')
   self.row.update(forced_cleanup=True,forced_cleanup_reason=reason)
  return self.wait(15)

 def close(self):
  if self.close_handle and self.handle:
   require(kernel().CloseHandle(self.handle),'owned query handle close failed');self.handle=None

class OwnedJob:
 """Noninheritable, unnamed job. Every work child is enrolled before resume.

 Structures/flags follow Microsoft JOBOBJECT_* documentation. CreateProcess
 arguments follow the installed CPython subprocess implementation. No job or
 process is created by importing this class.
 """
 def __init__(self):
  k=kernel();k.CreateJobObjectW.argtypes=[ctypes.c_void_p,wintypes.LPCWSTR];k.CreateJobObjectW.restype=wintypes.HANDLE
  k.SetInformationJobObject.argtypes=[wintypes.HANDLE,ctypes.c_int,ctypes.c_void_p,wintypes.DWORD];k.SetInformationJobObject.restype=wintypes.BOOL
  k.AssignProcessToJobObject.argtypes=[wintypes.HANDLE,wintypes.HANDLE];k.AssignProcessToJobObject.restype=wintypes.BOOL
  k.QueryInformationJobObject.argtypes=[wintypes.HANDLE,ctypes.c_int,ctypes.c_void_p,wintypes.DWORD,ctypes.POINTER(wintypes.DWORD)];k.QueryInformationJobObject.restype=wintypes.BOOL
  k.TerminateJobObject.argtypes=[wintypes.HANDLE,wintypes.UINT];k.TerminateJobObject.restype=wintypes.BOOL
  k.IsProcessInJob.argtypes=[wintypes.HANDLE,wintypes.HANDLE,ctypes.POINTER(wintypes.BOOL)];k.IsProcessInJob.restype=wintypes.BOOL
  k.ResumeThread.argtypes=[wintypes.HANDLE];k.ResumeThread.restype=wintypes.DWORD
  class Basic(ctypes.Structure):
   _fields_=[('process_time',ctypes.c_longlong),('job_time',ctypes.c_longlong),('flags',wintypes.DWORD),
    ('working_min',ctypes.c_size_t),('working_max',ctypes.c_size_t),('active_limit',wintypes.DWORD),
    ('affinity',ctypes.c_size_t),('priority',wintypes.DWORD),('scheduling',wintypes.DWORD)]
  class IO(ctypes.Structure):_fields_=[('c'+str(i),ctypes.c_ulonglong) for i in range(6)]
  class Extended(ctypes.Structure):
   _fields_=[('basic',Basic),('io',IO),('process_memory',ctypes.c_size_t),('job_memory',ctypes.c_size_t),
    ('peak_process_memory',ctypes.c_size_t),('peak_job_memory',ctypes.c_size_t)]
  require(ctypes.sizeof(ctypes.c_void_p)==8 and ctypes.sizeof(Extended)==144,'expected x64 job layout')
  handle=k.CreateJobObjectW(None,None);require(handle,'could not create owned job')
  try:
   info=Extended();info.basic.flags=0x00002000 # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE; no breakaway
   require(k.SetInformationJobObject(handle,9,ctypes.byref(info),ctypes.sizeof(info)),'could not install owned job close barrier')
  except BaseException:k.CloseHandle(handle);raise
  self.handle,self.k,self.forced=handle,k,False

 def spawn_suspended(self,command,environment,cwd,stdout_path,stderr_path,expected_sha,kind,on_created):
  # Direct _winapi retains both handles. Popen closes the primary thread
  # handle, which would prevent atomic job enrollment before ResumeThread.
  import _winapi,msvcrt
  command=list(map(str,command));cwd=str(cwd)
  require(sha(command[0])==expected_sha,'planned executable changed')
  process=thread=None;held=None
  with open(stdout_path,'xb') as out,open(stderr_path,'xb') as err,open(os.devnull,'rb') as null:
   handles=[msvcrt.get_osfhandle(file.fileno()) for file in [null,out,err]]
   previous=[os.get_handle_inheritable(h) for h in handles]
   startup=subprocess.STARTUPINFO();startup.dwFlags=subprocess.STARTF_USESTDHANDLES
   startup.hStdInput,startup.hStdOutput,startup.hStdError=handles
   startup.lpAttributeList={'handle_list':handles}
   try:
    for handle in handles:os.set_handle_inheritable(handle,True)
    flags=0x00000004|subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS # CREATE_SUSPENDED
    process,thread,pid,tid=_winapi.CreateProcess(command[0],subprocess.list2cmdline(command),None,None,True,flags,environment,cwd,startup)
    actual=identity(process,pid)
    require(actual['executable_sha256']==expected_sha and Path(actual['executable'])==Path(command[0]).resolve(),'created image differs')
    row=dict(actual,kind=kind,command=command,wait_confirmed=False,exit_code=None,
     created_suspended=True,job_assigned_before_resume=False,primary_thread_id=tid)
    held=HeldProcess(process,row,True)
    require(self.k.AssignProcessToJobObject(self.handle,process),'owned job assignment failed; suspended child will be retired')
    row['job_assigned_before_resume']=True
    on_created(held) # durably preserve the original identity before code runs
    require(self.k.ResumeThread(thread)==1,'original primary thread resume failed')
    return held
   except BaseException:
    if process:
     if held is not None:held.terminate_original('creation/enrollment failed');held.close()
     else:
      self.k.TerminateProcess(process,1);self.k.WaitForSingleObject(process,15000);self.k.CloseHandle(process)
    raise
   finally:
    if thread:self.k.CloseHandle(thread)
    for handle,value in zip(handles,previous):os.set_handle_inheritable(handle,value)

 def accounting(self):
  class Accounting(ctypes.Structure):
   _fields_=[('time'+str(i),ctypes.c_longlong) for i in range(4)]+[(n,wintypes.DWORD) for n in ['page_faults','total_processes','active_processes','terminated_processes']]
  info=Accounting();require(self.k.QueryInformationJobObject(self.handle,1,ctypes.byref(info),ctypes.sizeof(info),None),'owned job accounting unavailable')
  return dict(total_processes=int(info.total_processes),active_processes=int(info.active_processes),terminated_processes=int(info.terminated_processes))

 def terminate_members(self):
  require(self.k.TerminateJobObject(self.handle,1),'owned job termination failed');self.forced=True

 def require_member(self,process):
  member=wintypes.BOOL()
  require(self.k.IsProcessInJob(process.handle,self.handle,ctypes.byref(member)) and member.value,'recorded process is not in this owned job')

 def close(self):
  if self.handle:require(self.k.CloseHandle(self.handle),'owned job handle close failed');self.handle=None
