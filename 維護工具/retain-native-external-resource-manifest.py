"""Stream full file identities for declared external game asset namespaces."""
from pathlib import Path
from datetime import datetime,timezone
from ctypes import wintypes
import ctypes,hashlib,json,msvcrt,os,time

ROOT=Path(__file__).resolve().parents[1]
RESOURCE=Path(r'D:\lineage開發\0-客戶端')
EXTENSIONS={'.idx','.pak','.s32','.wav','.tbt','.img','.spr','.til','.lmp'}
NAMESPACES=['LiTo','map','sound']
def regular(path):
    st=path.stat(follow_symlinks=False)
    assert not getattr(st,'st_file_attributes',0)&0x400,path
    return st
def scan():
    regular(RESOURCE)
    found={}
    def add(path):
        st=regular(path);assert path.is_file() and 0<=st.st_size<=16*1024**3
        found[path.relative_to(RESOURCE).as_posix()]=(st.st_size,st.st_mtime_ns,st.st_ino)
    for p in RESOURCE.iterdir():
        if p.suffix.lower() in {'.idx','.pak'}:
            add(p)
    tw=RESOURCE/'launcher/TW13081901.pak'
    regular(tw.parent);add(tw)
    for namespace in NAMESPACES:
        base=RESOURCE/namespace
        if not base.exists():continue
        regular(base);pending=[base]
        while pending:
            folder=pending.pop()
            with os.scandir(folder) as entries:
                for entry in entries:
                    st=entry.stat(follow_symlinks=False)
                    assert not getattr(st,'st_file_attributes',0)&0x400,entry.path
                    if entry.is_dir(follow_symlinks=False):pending.append(Path(entry.path))
                    elif entry.is_file(follow_symlinks=False) and Path(entry.name).suffix.lower() in EXTENSIONS:add(Path(entry.path))
    return dict(sorted(found.items()))
files=scan();assert files and len(files)<100000
out=ROOT/'驗證紀錄/current/native-release-resource-manifest'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
save(out/'observed-before.json',files);(out/'producer.py').write_bytes(Path(__file__).read_bytes())
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.CreateFileW.argtypes=[wintypes.LPCWSTR,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,wintypes.HANDLE];k.CreateFileW.restype=wintypes.HANDLE
rows={};total=sum(v[0] for v in files.values());started=time.monotonic();last=started;processed=0
state=dict(status='hashing_declared_external_asset_files',output=str(out),resource_root=str(RESOURCE),files=len(files),bytes=total,selection=dict(top_level=['*.idx','*.pak'],fixed_files=['launcher/TW13081901.pak'],recursive_namespaces=NAMESPACES,extensions=sorted(EXTENSIONS)),resource_copies_created=0,resource_functionality_qualified=False,all_resource_formats_qualified=False,whole_goal_complete=False)
save(out/'progress.json',state);print(json.dumps({key:state[key] for key in ['status','output','files','bytes']},ensure_ascii=False),flush=True)
for name,before in files.items():
    path=RESOURCE/name;st=regular(path);assert (st.st_size,st.st_mtime_ns,st.st_ino)==before
    h=k.CreateFileW(str(path),0x80000000,1,None,3,0x00200000,None)
    if h in [None,ctypes.c_void_p(-1).value]:raise ctypes.WinError(ctypes.get_last_error())
    fd=msvcrt.open_osfhandle(h,os.O_RDONLY|os.O_BINARY)
    with os.fdopen(fd,'rb') as f:
        opened=os.fstat(f.fileno());assert (opened.st_size,opened.st_mtime_ns,opened.st_ino)==before
        digest=hashlib.file_digest(f,'sha256').hexdigest().upper()
        after=os.fstat(f.fileno());assert (after.st_size,after.st_mtime_ns,after.st_ino)==before
    rows[name]=dict(bytes=before[0],sha256=digest);processed+=before[0]
    if time.monotonic()-last>=20:
        state.update(completed_files=len(rows),completed_bytes=processed);save(out/'progress.json',state);print(json.dumps(dict(completed_files=len(rows),files=len(files),completed_bytes=processed,total_bytes=total)),flush=True);last=time.monotonic()
assert scan()==files
save(out/'resource-files.json',rows)
digest=hashlib.sha256((out/'resource-files.json').read_bytes()).hexdigest().upper()
assert json.loads((out/'resource-files.json').read_text(encoding='utf-8'))==rows
state.update(status='retained_full_SHA256_for_declared_external_asset_namespaces',completed_files=len(rows),completed_bytes=processed,manifest_sha256=digest,elapsed_seconds=time.monotonic()-started,metadata_and_file_set_unchanged_before_after=True,whole_file_SHA256=True,previous_Sprite_PAK_metadata_only_scope_not_relabelled=True,missing_root_Disint_catalog=not (RESOURCE/'TW13081901.pak').exists(),native_map303_source_present=(RESOURCE/'LiTo/LTmap/303').is_dir(),finished_utc=datetime.now(timezone.utc).isoformat())
save(out/'progress.json',state);print(json.dumps({key:state[key] for key in ['status','output','files','bytes','manifest_sha256','elapsed_seconds']},ensure_ascii=False),flush=True)
