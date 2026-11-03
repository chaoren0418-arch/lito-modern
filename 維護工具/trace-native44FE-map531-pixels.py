"""Inspect a few source pixels through the pinned normal library, no game flow or source overlay."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
normal=V/'native44FE-world-tree/compile-20261018-173453-542713';b=read(normal/'receipt.json');assert not b['source_overlays_used']and not b['diagnostic_API_extension']
qualified=V/'native44FE-library-default/20261018-190758-247440/Debug/full-default-qualified/qualified.json';q=read(qualified)
assert q['passed']==1969 and q['failed']==0 and q['source_manifest_sha256']==b['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
image=P/'target/current/scratch/native-multisector-batch/run-37120-0/cpu-531-seam-after.png';assert sha(image)==q['raw_files']['owned-output/native-multisector-batch/'+image.name]
manifest=P/'logs/continuation-20261003/r481-map-binding.json';inputs=read(normal/'compiled-inputs.json')
data=read(manifest);binding=next(x for x in data['bindings']if x['map_id']==531);authority_path=Path(data['catalog_source']['path']);assert sha(authority_path)==data['catalog_source']['sha256']
authority=next(x for x in read(authority_path)['rows']if x['logical_map']==531)
original_sources={}
for n,pin in zip(['LTtile.idx','LTtile.pak'],binding['archives'][:2]):
    path=Path(r'D:\lineage開發\0-客戶端\LiTo')/n;assert sha(path)==pin['sha256'];original_sources[str(path)]=pin['sha256']
native_sector=next(x for x in authority['native_sector_files']if Path(x['path']).name.lower()=='80008000.s32');path=Path(native_sector['path']);assert sha(path)==native_sector['sha256'];original_sources[str(path)]=native_sector['sha256']
owned_sector=next(x for x in binding['sectors']if x['x']==32768 and x['y']==32768);assert owned_sector['pin']['sha256']==native_sector['sha256']
def guard():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for row in b['libraries'].values():assert sha(row['path'])==row['sha256']
    assert sha(image)==q['raw_files']['owned-output/native-multisector-batch/'+image.name]
    for path,h in original_sources.items():assert sha(path)==h,path
guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native44FE-map531-provenance'/stamp;out.mkdir(parents=True)
work=P/'target/current/native44FE-pixel-provenance'/stamp;work.mkdir(parents=True)
source=ROOT/'維護工具/candidates/native44FE-skill-book-audit/trace_531_pixels.rs';(out/source.name).write_bytes(source.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
exe=work/'trace-531.exe';rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
args=[str(rust),str(out/source.name),'--edition=2021','--crate-name','native44FE_pixel_source','--target','x86_64-pc-windows-msvc','-C','debuginfo=1','-C','opt-level=0','-o',str(exe)]
for d in ['target/current/x86_64-pc-windows-msvc/debug/deps','target/current/debug/deps']:args+=['-L','dependency='+str(P/d)]
for name,row in b['libraries'].items():args+=['--extern',name+'='+row['path']]
command=b['compiler']['command']
for i,word in enumerate(command[:-1]):
    if word=='-L'and not command[i+1].startswith('dependency='):args+=['-L',command[i+1]]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='inspecting_normal_API_pixel_sources',source_manifest_sha256=b['source_manifest_sha256'],normal_library=str(normal),reference_qualification=str(qualified),reference_qualification_sha256=sha(qualified),reference_PNG_sha256=sha(image),manifest=str(manifest),manifest_sha256=sha(manifest),
    runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,normal_source_modified=False,source_overlays_used=False,inspector_plain_Native_old28_index=True,original_native_source_pins=original_sources,actual_sessions=0,GUI_verified=False,whole_goal_complete=False)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('PIXEL_PROVENANCE='+str(out),flush=True)
try:
    for label,command in [('compile',args),('inspect',[str(exe),str(manifest),str(image),str(work/'result.json')])]:
        with(out/(label+'.stdout')).open('wb')as stdout,(out/(label+'.stderr')).open('wb')as stderr:
            child=subprocess.Popen(command,cwd=work,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,command=command);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=60)
            except subprocess.TimeoutExpired:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];job['cleanup_stopped_original_owned_handle']=True;child.terminate();job['exit_code']=child.wait(timeout=3);raise
            finally:state['owned_process']=None;save(out/'receipt.json',state)
        assert job['exit_code']==0,label
    result=read(work/'result.json');assert result['full_RGBA_match']and len(result['points'])==5
    (out/'result.json').write_bytes((work/'result.json').read_bytes());state.update(status='normal_API_pixel_source_inspection_completed',product=str(exe),product_sha256=sha(exe),result_sha256=sha(out/'result.json'))
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:guard();save(out/'receipt.json',state)
files={p.name:p for p in out.iterdir()if p.is_file()};pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),normal_API_only=True,normal_source_modified=False))
print(json.dumps(dict(evidence=str(out),points=[dict(position=p['position'],RGBA=p['RGBA'],writers=len(p['writes']),last_writer=p['writes'][-1])for p in result['points']]),ensure_ascii=False))
