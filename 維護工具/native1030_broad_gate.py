"""Candidate work may use broad compilation time or an observed terminal cohort."""
import json,time,ctypes
from ctypes import wintypes
from datetime import datetime,timezone,timedelta
def read(path):
    for i in range(10):
        try:return json.loads(path.read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def require_idle_runtime(root):
    library=root/'驗證紀錄/current/native1030-library-default/current-continuation.json'
    if library.is_file():
        reservation=read(library)
        if reservation.get('candidate_compilation_reserved_until_pipeline_terminal'):
            k=ctypes.WinDLL('kernel32',use_last_error=True)
            k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE
            k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
            k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.CloseHandle.argtypes=[wintypes.HANDLE]
            handle=k.OpenProcess(0x100000|0x1000,False,reservation['pipeline_pid'])
            if handle:
                try:
                    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts])
                    ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
                    created=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
                    same=abs((created-datetime.fromisoformat(reservation['pipeline_creation_utc'])).total_seconds())<.001
                    assert not same or k.WaitForSingleObject(handle,0)==0,'full-library original pipeline still active; no candidate compiler/runtime beside it'
                finally:k.CloseHandle(handle)
            else:
                assert ctypes.get_last_error()==87,'library PID cannot be inspected; check birth via CIM before releasing reservation'
    child=root/'驗證紀錄/current/default-native-regression/20261017-221209-558-Release/receipt.json'
    status=read(child)['status']
    if status=='compiling_default_targets':return 'broad_compilation'
    base=root/'驗證紀錄/current/native1030-default111/20261017-221208-451052'
    assert status=='verified_current_default111_integration_targets_and_main_units', 'no candidate work beside broad runtime/readback'
    cohort=read(base/'cohort.json');observed=read(base/'completion-observation.json')
    assert cohort['status']=='independently_qualified_current_default111_both_profiles'
    assert observed['observed_terminal_exit_code']==0 and observed['recorded_original_owned_processes_all_terminal']
    return 'observed_terminal_broad_cohort'
