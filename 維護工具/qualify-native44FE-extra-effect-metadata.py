"""Seal metadata-only whole-crate typechecking, never resource/runtime qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('build',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
build=a.build.resolve(strict=True);assert build.is_relative_to(V/'native44FE-extra-finite-effects')and a.session>0 and a.terminal_chunk not in ['','pending','unknown']
b=read(build/'receipt.json');assert b['status']=='typechecked_isolated_lazy_finite_effect_variant_metadata_only'and b['metadata_only']and b['products']=={}and b['owned_process']is None
assert b['ability_request_candidate_included']and b['source_overlays_used']and not b['normal_product_modified']and b['actual_sessions']==b['actual_casts']==0
assert b['base_source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
assert sorted(b['runtime_overrides'])==sorted(['native_skill_effects.rs','native_additional_finite_effects.rs','native_skills.rs','native_skill_selection.rs','native_world.rs','native_ability_buffs.rs'])
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
assert sha(build/'candidate-inputs.json')==b['candidate_manifest_sha256']
for n,h in read(build/'candidate-inputs.json')['candidate_files'].items():assert sha(build/n)==h,n
assert sha(b['diagnostic_library'])==b['diagnostic_library_sha256']and Path(b['diagnostic_library']).suffix=='.rmeta'
assert [p.suffix for p in Path(b['runtime']).iterdir()if p.is_file()]==['.rmeta']
assert len(b['process_history'])==1 and b['process_history'][0]['exit_code']==0 and '--emit=metadata'in b['process_history'][0]['command']
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.CloseHandle.argtypes=[wintypes.HANDLE]
for row in [b['original_runner']]+b['process_history']:
    handle=k.OpenProcess(0x1000|0x100000,False,row['pid'])
    if not handle:assert ctypes.get_last_error()==87;continue
    try:
        ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
        created=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat();assert created!=row['creation_utc']or k.WaitForSingleObject(handle,0)==0
    finally:k.CloseHandle(handle)
out=build/'metadata-qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files={p.name:p for p in build.iterdir()if p.is_file()};pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
result=dict(status='isolated_whole_crate_metadata_typecheck_qualified_only',source_manifest_sha256=b['base_source_manifest_sha256'],candidate_manifest_sha256=b['candidate_manifest_sha256'],
    original_session=a.session,observed_terminal_chunk=a.terminal_chunk,observed_terminal_exit_code=0,recorded_original_owned_identities=2,recorded_original_owned_processes_terminal=True,
    runtime_overrides=b['runtime_overrides'],raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),machine_code_generated=False,tests_executed=False,SPR_PCM_verified=False,actual_sessions=0,actual_casts=0,normal_product_modified=False,whole_goal_complete=False)
save(out/'qualified.json',result)
candidate=ROOT/'維護工具/candidates/native44FE-ability-buff-requests';prep=read(candidate/'preparation.json');assert sha(candidate/'preparation.json')==sha(build/'ability_preparation.json')
prep.update(world_integration_typecheck_verified=True,world_integration_metadata_only=True,world_integration_typecheck=str(out/'qualified.json'),world_integration_typecheck_sha256=sha(out/'qualified.json'),world_integration_executed=False)
save(candidate/'preparation.json',prep)
print(json.dumps(dict(qualification=str(out/'qualified.json'),whole_crate_typecheck=True,machine_code=False,tests_executed=False),ensure_ascii=False))
