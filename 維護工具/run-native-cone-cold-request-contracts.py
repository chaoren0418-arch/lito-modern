"""Run the three staged request/packet/selection contracts on both scene overlays."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current';P=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
builds={
 'Debug':V/'native-cone-cold-scene/compile-20261018-054938-654394-debug-2fd5c225',
 'Release':V/'native-cone-cold-scene/compile-20261018-054938-663322-release-0b977157'}
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native-cone-cold-request-contracts'/stamp;out.mkdir(parents=True)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def identity(handle,pid):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return dict(pid=pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat())
state=dict(status='executing_candidate_request_contracts',original_runner=identity(k.GetCurrentProcess(),os.getpid()),profiles={},normal_product_modified=False,new_network_sessions=0,GUI_verified=False,whole_goal_complete=False)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'receipt.json',state)
try:
    for profile,d in builds.items():
        b=read(d/'receipt.json');assert b['status']=='compiled_isolated_cold_scene_diagnostic'and b['probe_kind']=='core'and b['profile']==profile
        assert sha(b['product'])==b['product_sha256']and sha(b['normal_product'])==b['normal_product_sha256']
        for n,h in read(d/'compiled-inputs.json').items():assert sha(ROOT/n)==h
        for pin in b['candidate_inputs'].values():assert sha(pin['path'])==pin['sha256']
        result=out/(profile+'-result.json');command=[b['product'],r'D:\lineage開發\0-客戶端',str(result)]
        with (out/(profile+'-stdout.log')).open('wb')as stdout,(out/(profile+'-stderr.log')).open('wb')as stderr:
            p=subprocess.Popen(command,cwd=P,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            job=dict(**identity(wintypes.HANDLE(int(p._handle)),p.pid),command=command)
            state['profiles'][profile]=dict(build=str(d),build_receipt_sha256=sha(d/'receipt.json'),product_sha256=b['product_sha256'],process=job);save(out/'receipt.json',state)
            job['exit_code']=p.wait();save(out/'receipt.json',state);assert job['exit_code']==0,(profile,'preserve first contract failure')
        report=read(result);assert report['status']=='candidate_cold_request_S23_selector_contracts_passed'and report['cases']==3
        assert report['network_sessions']==0 and not report['asset_resources_opened']and not report['normal_product_registered']
    assert read(out/'Debug-result.json')==read(out/'Release-result.json')
    state['status']='executed_both_candidate_request_contracts'
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],cases_each=3,new_network_sessions=0),ensure_ascii=False))
