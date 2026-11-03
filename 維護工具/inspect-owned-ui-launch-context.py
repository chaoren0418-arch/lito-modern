"""Read only this test launcher desktop context; never switch desktops or send input."""
from pathlib import Path
from datetime import datetime,timezone
from ctypes import wintypes
import ctypes,json,os
R=Path(__file__).resolve().parents[1]
u=ctypes.WinDLL('user32',use_last_error=True);k=ctypes.WinDLL('kernel32',use_last_error=True)
u.GetProcessWindowStation.restype=wintypes.HANDLE
u.GetThreadDesktop.argtypes=[wintypes.DWORD];u.GetThreadDesktop.restype=wintypes.HANDLE
u.GetUserObjectInformationW.argtypes=[wintypes.HANDLE,ctypes.c_int,ctypes.c_void_p,wintypes.DWORD,ctypes.POINTER(wintypes.DWORD)]
u.OpenInputDesktop.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];u.OpenInputDesktop.restype=wintypes.HANDLE
u.CloseDesktop.argtypes=[wintypes.HANDLE]
def name(handle):
    needed=wintypes.DWORD();u.GetUserObjectInformationW(handle,2,None,0,ctypes.byref(needed))
    assert needed.value>0;buffer=ctypes.create_unicode_buffer(needed.value//2+1)
    assert u.GetUserObjectInformationW(handle,2,buffer,ctypes.sizeof(buffer),ctypes.byref(needed))
    return buffer.value
result=dict(status='read_only_own_launcher_context',pid=os.getpid(),window_station=name(u.GetProcessWindowStation()),
    own_desktop=name(u.GetThreadDesktop(k.GetCurrentThreadId())),desktop_switch_performed=False,UI_input_sent=False,security_or_privacy_setting_changed=False)
handle=u.OpenInputDesktop(0,False,1)
if handle:
    try:result['read_only_input_desktop_name']=name(handle)
    finally:u.CloseDesktop(handle)
else:
    result['input_desktop_name_not_read']=ctypes.get_last_error()
    result['lock_state_inferred']=False
out=R/'驗證紀錄/current/native-keyboard-focus-sky-gui'/('launcher-context-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir();(out/'inspector.py').write_bytes(Path(__file__).read_bytes());(out/'readback.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(out/'readback.json'),**result),ensure_ascii=False))
