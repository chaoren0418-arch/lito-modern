"""Inspect original definitions needed by missing requests using the pinned normal reader."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
base=V/'native44FE-skill-book-audit/20261018-175440-043382';matrix=read(base/'matrix.json')
shape=base/'request-shapes-effects-20261018-180850-512952';q=read(shape/'readback.json');assert sha(shape/'computed-shapes-effects.json')==q['data_sha256']
data=read(shape/'computed-shapes-effects.json');ids=set(matrix['missing_request_ids'])
graphics=read(ROOT/'維護工具/candidates/native44FE-extra-skill-finite-effects/preparation.json')['graphics']
assert graphics and data['source_manifest_sha256']==matrix['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
build=V/'native44FE-world-tree/compile-20261018-173453-542713';b=read(build/'receipt.json');assert b['status']=='compiled_current_production_Presentation_test'and not b['source_overlays_used']and not b['diagnostic_API_extension']
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
for r in b['libraries'].values():assert sha(r['path'])==r['sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=base/('finite-SPR2251-guard-'+stamp);out.mkdir()
work=P/'target/current/native44FE-skill-assets'/stamp;work.mkdir(parents=True)
probe=ROOT/'維護工具/candidates/native44FE-skill-book-audit/inspect_finite_SPRs_guard.rs';(out/probe.name).write_bytes(probe.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
candidate=ROOT/'維護工具/candidates/native44FE-extra-skill-effects-integration/native_additional_finite_effects.rs'
(out/candidate.name).write_bytes(candidate.read_bytes());candidate_sha=sha(candidate)
exe=work/'inspect-effect-definitions.exe';rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
args=[str(rust),str(out/probe.name),'--edition=2021','--crate-name','native44FE_readonly_effect_definitions','--target','x86_64-pc-windows-msvc','-C','debuginfo=1','-C','opt-level=0','-o',str(exe)]
for d in ['target/current/x86_64-pc-windows-msvc/debug/deps','target/current/debug/deps']:args+=['-L','dependency='+str(P/d)]
for n,r in b['libraries'].items():args+=['--extern',n+'='+r['path']]
for n in ['-L']:
    command=b['compiler']['command']
    for i,word in enumerate(command[:-1]):
        if word==n and not command[i+1].startswith('dependency='):args += [n,command[i+1]]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(h):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='reading_original_effect_definitions',source_manifest_sha256=matrix['source_manifest_sha256'],graphics=graphics,shape_evidence=str(shape),shape_evidence_sha256=sha(shape/'computed-shapes-effects.json'),normal_source_modified=False,source_overlays_used=False,actual_sessions=0,actual_casts=0,process_history=[])
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
def execute(command,label):
    with(out/(label+'.stdout')).open('wb')as stdout,(out/(label+'.stderr')).open('wb')as stderr:
        child=subprocess.Popen(command,env=env,cwd=work,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=label);state['process_history'].append(job);save(out/'receipt.json',state)
        job['exit_code']=child.wait();save(out/'receipt.json',state)
    if job['exit_code']!=0:
        state.update(status='failed_metadata_'+label,error='Readonly metadata helper failed; no gameplay scope inferred');save(out/'receipt.json',state)
    assert job['exit_code']==0,label
execute(args,'compile');state['product_sha256']=sha(exe)
execute([str(exe),r'D:\lineage開發\0-客戶端',','.join(map(str,graphics))],'inspect')
rows=read(out/'inspect.stdout');assert rows['candidate_guard_positive']==1 and rows['candidate_guard_rejections']==5 and sha(candidate)==candidate_sha
assert rows['only_readonly_SPR_metadata']and not rows['SPR_or_PCM_playback_verified']
assert {r['graphics']for r in rows['rows']}==set(graphics)
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
state.update(status='original_effect_SPR_metadata_readback_only',catalogue_sha256=bytes(rows['catalogue_sha256']).hex().upper(),rows=len(rows['rows']),actual_new_sessions=0,actual_new_casts=0,whole_goal_complete=False);save(out/'receipt.json',state)
files={p.name:p for p in out.iterdir()if p.is_file()};pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),data_sha256=sha(out/'inspect.stdout'),metadata_only=True))
print(json.dumps(dict(evidence=str(out),graphics=len(graphics),missing_definitions=sum(not r['present']for r in rows['rows']),only_metadata=True),ensure_ascii=False))
