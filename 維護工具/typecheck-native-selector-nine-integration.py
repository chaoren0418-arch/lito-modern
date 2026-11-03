"""Compile one coherent candidate module graph without modifying the product."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
ap=argparse.ArgumentParser();ap.add_argument('--candidate',type=Path,
    default=ROOT/'維護工具/candidates/native-selector-nine-integration-shared-pool')
CANDIDATE=ap.parse_args().candidate.resolve(strict=True)
assert CANDIDATE.is_relative_to(ROOT/'維護工具/candidates')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
m=read(CANDIDATE/'candidate.json');base=Path(m['base_related_run']);pins=read(base/'compiled-inputs.json')
assert len(pins)==1025 and not m['applied']
candidate_pins={str(CANDIDATE/'sources'/Path(n).name):h for n,h in m['after'].items()}
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-selector-nine-integration-typecheck'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-selector-nine-integration-typecheck'/stamp;runtime.mkdir(parents=True)
staged=out/'candidate-source-paths';staged.mkdir()
staged_paths={};path_adaptations={}
for name,digest in m['after'].items():
    source=CANDIDATE/'sources'/Path(name).name
    text=source.read_text(encoding='utf-8-sig');changes=[]
    if source.name!='mod.rs':
        def absolute_child(match):
            child=Path(match[1])
            if child.is_absolute():return match[0]
            target=(PROJECT/'src/modern_lito'/child).resolve(strict=True)
            changes.append(dict(relative=match[1],absolute=str(target)))
            return '#[path="'+target.as_posix()+'"]'
        text=re.sub(r'#\[path\s*=\s*"([^"]+)"\]',absolute_child,text)
    target=staged/source.name;target.write_text(text,encoding='utf-8',newline='\n')
    staged_paths[source.name]=target
    path_adaptations[source.name]=dict(original_candidate_sha256=digest,staged_sha256=sha(target),child_paths=changes)
lines=[];pending=None;module_paths={}
for line in (CANDIDATE/'sources/mod.rs').read_text(encoding='utf-8-sig').splitlines():
    attr=re.fullmatch(r'\s*#\[path\s*=\s*"([^"]+)"\]\s*',line)
    if attr:assert pending is None;pending=attr[1];continue
    module=re.fullmatch(r'(\s*)((?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;)(?:\s*//.*)?',line)
    if module:
        name=module[3]
        selected=staged_paths.get(name+'.rs')
        if selected is not None:p=selected
        elif pending:p=PROJECT/'src/modern_lito'/pending
        else:
            p=PROJECT/'src/modern_lito'/(name+'.rs')
            if not p.exists():p=PROJECT/'src/modern_lito'/name/'mod.rs'
        p=p.resolve(strict=True);pending=None;module_paths[name]=str(p)
        lines.extend([module[1]+'#[path="'+p.as_posix()+'"]',module[1]+module[2]])
    else:lines.append(line)
assert pending is None
modroot=out/'isolated_mod.rs';modroot.write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')
libroot=out/'isolated_lib.rs'
libroot.write_text('#[path="'+modroot.as_posix()+'"]\npub mod modern_lito;\n#[path="'+(PROJECT/'src/shared_v1/morph_auth.rs').as_posix()+'"]\npub mod morph_auth;\n',encoding='utf-8',newline='\n')
libraries,native,buildenv={},{},{}
for line in (base/'Debug/build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':
        for p in row.get('linked_paths',[]):native[p]=True
        if 'lito-modern' in row.get('package_id',''):
            buildenv.update(row.get('env',[]));buildenv['OUT_DIR']=row['out_dir']
    if row.get('reason')=='compiler-artifact' and row['target']['name']!='lito_modern':
        paths=[p for p in row['filenames'] if p.endswith('.rlib')]
        if paths:
            name=row['target']['name'].replace('-','_');p=paths[0]
            if name not in libraries or '/x86_64-pc-windows-msvc/' in p.replace('\\','/'):
                libraries[name]=p
assert {'anyhow','serde','serde_json','sha2','windows','rustls','roxmltree'}<=set(libraries)
assert 'OUT_DIR' in buildenv
package=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['package']
version=package['version'];parts=(version.split('.')+['0','0'])[:3]
env=dict(os.environ,CARGO_MANIFEST_DIR=str(PROJECT),CARGO_PKG_NAME=package['name'],CARGO_PKG_VERSION=version,
    CARGO_PKG_VERSION_MAJOR=parts[0],CARGO_PKG_VERSION_MINOR=parts[1],CARGO_PKG_VERSION_PATCH=parts[2],
    CARGO_PKG_VERSION_PRE='',CARGO_PKG_AUTHORS=':'.join(package.get('authors',[])),
    CARGO_PKG_DESCRIPTION=package.get('description',''),CARGO_PKG_REPOSITORY=package.get('repository',''),
    CARGO_PKG_HOMEPAGE=package.get('homepage',''),CARGO_PKG_LICENSE=package.get('license',''),
    TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),
    PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',**buildenv)
output=runtime/'liblito_selector_candidate.rmeta'
args=[str(RUST/'rustc.exe'),str(libroot),'--edition=2021','--crate-name','lito_selector_candidate',
    '--crate-type','rlib','--emit=metadata','--target','x86_64-pc-windows-msvc',
    '-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),
    '-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(output)]
for name,p in sorted(libraries.items()):args+=['--extern',name+'='+p]
for p in sorted(native):args+=['-L',p]
state=dict(status='preparing_coherent_candidate_typecheck',base_related_run=str(base),
    source_manifest_sha256=m['base_source_manifest_sha256'],candidate=str(CANDIDATE),candidate_after=m['after'],
    module_paths=module_paths,path_adaptations=path_adaptations,libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},
    product_source_changed=False,applied=False,normal_product_build=False,owned_process=None,process_history=[],
    actual_new_sessions=0,actual_new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(out/'receipt.json',state);save(out/'compiled-inputs.json',pins)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'candidate.json').write_bytes((CANDIDATE/'candidate.json').read_bytes())
def stable():
    assert all(sha(ROOT/n)==h for n,h in pins.items())
    assert all(sha(p)==h for p,h in candidate_pins.items())
    assert all(sha(staged_paths[n])==v['staged_sha256'] for n,v in path_adaptations.items())
    assert all(sha(v['path'])==v['sha256'] for v in state['libraries'].values())
print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
error=None
try:
    stable();(out/'build').mkdir()
    with (out/'build/stdout.log').open('wb') as stdout,(out/'build/stderr.log').open('wb') as stderr:
        p=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        k=ctypes.WinDLL('kernel32',use_last_error=True)
        k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
        times=[wintypes.FILETIME() for _ in range(4)]
        assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
        ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        item=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=args)
        state.update(status='typechecking_coherent_candidate_library_graph',owned_process=item);state['process_history'].append(item);save(out/'receipt.json',state)
        code=p.wait();item['exit_code']=code;state['owned_process']=None;save(out/'receipt.json',state)
    stable();assert code==0,'Preserve the candidate typecheck failure before repair'
    state.update(status='typechecked_coherent_candidate_library_graph',metadata=str(output),metadata_sha256=sha(output))
except BaseException as e:error=e;state.update(status='failed_preserved',error=repr(e))
finally:save(out/'receipt.json',state)
files={p.relative_to(out).as_posix():p for p in out.rglob('*') if p.is_file()}
files.update({'candidate-sources/'+Path(n).name:CANDIDATE/'sources'/Path(n).name for n in m['after']})
files['candidate-before.zip']=CANDIDATE/'before.zip'
if output.exists():files['candidate.rmeta']=output
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    for n,h in hashes.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(status=state['status'],source_manifest_sha256=m['base_source_manifest_sha256'],
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,product_source_changed=False,applied=False,
    actual_new_sessions=0,actual_new_casts=0,normal_product_build=False,GUI_verified=False,whole_goal_complete=False))
print(json.dumps(dict(output=str(out),status=state['status'],raw_archive_sha256=sha(out/'raw.zip')),ensure_ascii=False))
if error:raise error
