"""Preserve an actual pre-cast room-join failure without granting a pass."""
from pathlib import Path
from datetime import datetime, timezone
import ctypes, hashlib, json, zipfile
from ctypes import wintypes
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
run=ROOT/'驗證紀錄/current/native-cone-cold-class2-regression/20261018-053532-621374'
r=read(run/'receipt.json');runtime=Path(r['runtime'])
assert r['status']=='failed_preserved' and r['all_started_processes_terminal'] and not r['owned_processes']
assert r['asset_metadata_unchanged']
jobs=r['process_history']+[read(run/'original-runner-process.json')]
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
k.CloseHandle.argtypes=[wintypes.HANDLE]
for j in jobs:
    h=k.OpenProcess(0x1000,False,j['pid'])
    if not h:assert ctypes.get_last_error()==87;continue
    try:
        ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t)for t in ts])
        ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
        original=int((datetime.fromisoformat(j['creation_utc'])-datetime(1601,1,1,tzinfo=timezone.utc)).total_seconds()*10_000_000)
        assert abs(ticks-original)>10000,'recorded original identity still exists'
    finally:k.CloseHandle(h)
assert 'departure without current or known retired room' in (run/'b/stderr.log').read_text('utf-8-sig')
assert not any(bytes.fromhex(v.get('plain',''))[:1]==b'\x14'
    for p in (runtime/'captures').rglob('frames.jsonl')
    for s in p.read_text('utf-8-sig').splitlines() for v in [json.loads(s)]
    if v.get('opcode')=='0x14')
out=run/'failure-boundary';out.mkdir()
files={p.relative_to(runtime).as_posix():p for p in runtime.rglob('*')if p.is_file()}
files.update({'driver/'+p.relative_to(run).as_posix():p for p in run.rglob('*')if p.is_file()and not p.is_relative_to(out)})
files['preserver.py']=Path(__file__);pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
result=dict(status='preserved_actual_precast_join_failure_not_qualified',run=str(run),
    original_session=37509,observed_actual_exit_code=1,terminal_tool_chunk='144f44',
    failure='departure without current or known retired room during RoomJoining',
    actual_C14_requests=0,recorded_original_identities=len(jobs),recorded_original_identities_all_terminal=True,
    raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),
    evidence_inference='D0 ArenaBot broadcast raced with pending CE acceptance; shared server capture interleaves peers, per-peer order still requires deterministic controller replay',
    product_source_modified=False,normal_product_registered=False,GUI_verified=False,whole_goal_complete=False)
(out/'boundary.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert read(out/'boundary.json')==result
print(json.dumps(dict(boundary=str(out/'boundary.json'),recorded_original_identities=len(jobs),actual_C14_requests=0),ensure_ascii=False))
