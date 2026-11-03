"""Execute a coherent candidate resource/Effects probe after current baseline gates."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
ap=argparse.ArgumentParser();ap.add_argument('--candidate',type=Path,
    default=ROOT/'維護工具/candidates/native-selector-nine-integration-shared-pool')
ap.add_argument('--library-run',type=Path,required=True)
ap.add_argument('--resource-qualification',type=Path,required=True)
ap.add_argument('--visibility-qualification',type=Path,required=True)
ap.add_argument('--probe-directory',type=Path,default=ROOT/'維護工具/candidates/native-selector-nine-runtime-followup')
options=ap.parse_args()
CANDIDATE=options.candidate.resolve(strict=True)
assert CANDIDATE.is_relative_to(ROOT/'維護工具/candidates')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
m=read(CANDIDATE/'candidate.json');base=Path(m['base_related_run']);pins=read(base/'compiled-inputs.json')
assert len(pins)==1025 and not m['applied']
# These are baseline qualifications, never candidate or GUI acceptance.
gates={
 'library':options.library_run.resolve(strict=True)/'completion-observation.json',
 'resources':options.resource_qualification.resolve(strict=True),
 'visibility':options.visibility_qualification.resolve(strict=True),
}
expected={
 'library':'independently_qualified_current_full_library_both_profiles_and_terminal_original_runner',
 'resources':'independently_readback_qualified_current_ten_unique_original_resource_bodies_both_profiles',
 'visibility':'independently_qualified_current_normal_visibility_both_profiles',
}
for name,p in gates.items():
 assert p.resolve(strict=True).is_relative_to(ROOT/'驗證紀錄/current')
 q=read(p);assert q['status']==expected[name]
 assert q['source_manifest_sha256']==m['base_source_manifest_sha256']
 if name=='library':assert q['observed_terminal_exit_code']==0
assert read(options.library_run/'compiled-inputs.json')==pins
assert not read(ROOT/'工作佇列.json').get('active_workload_sessions'), 'Finish and checkpoint all current timing-sensitive jobs first'
probe_directory=options.probe_directory.resolve(strict=True)
assert probe_directory.is_relative_to(ROOT/'維護工具/candidates')
probe=probe_directory/'probe.rs'
preparation=read(probe.parent/'preparation.json')
assert preparation['probe_sha256']==sha(probe)
assert preparation['candidate_metadata_sha256']==sha(CANDIDATE/'candidate.json')
candidate_pins={str(CANDIDATE/'sources'/Path(n).name):h for n,h in m['after'].items()}
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-selector-nine-integration-resource-probe'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-selector-nine-integration-resource-probe'/stamp;runtime.mkdir(parents=True)
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

# Retain the reviewed probe next to its generated coherent module graph.
(out/'probe.rs').write_bytes(probe.read_bytes())
probe_digest=sha(probe)
with libroot.open('a',encoding='utf-8',newline='\n') as f:
 f.write('#[path="'+(out/'probe.rs').as_posix()+'"]\nmod candidate_resource_probe;\n')
 f.write('fn main()->anyhow::Result<()>{candidate_resource_probe::run(std::path::Path::new(r"D:/lineage開發/0-客戶端"),std::path::Path::new(r"'+(out/'probe-observation.json').as_posix()+'"))}\n')
output=runtime/'candidate-resource-probe.exe'
args=[str(RUST/'rustc.exe'),str(libroot),'--edition=2021','--crate-name','lito_selector_resource_probe',
 '--crate-type','bin','--target','x86_64-pc-windows-msvc',
 '-L','dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),
 '-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(output)]
for name,p in sorted(libraries.items()):args+=['--extern',name+'='+p]
for p in sorted(native):args+=['-L',p]
state=dict(status='preparing_candidate_resource_probe',candidate=str(CANDIDATE),
 source_manifest_sha256=m['base_source_manifest_sha256'],candidate_after=m['after'],
 module_paths=module_paths,path_adaptations=path_adaptations,
 libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},
 baseline_gates={n:dict(path=str(p),sha256=sha(p)) for n,p in gates.items()},
 probe_sha256=probe_digest,owned_process=None,process_history=[],product_source_changed=False,
 normal_product_build=False,actual_new_sessions=0,actual_new_casts=0,audio_device_opened=False,
 full_Model_or_Presentation_executed=False,GUI_verified=False,whole_goal_complete=False)
save(out/'receipt.json',state);save(out/'compiled-inputs.json',pins)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'candidate.json').write_bytes((CANDIDATE/'candidate.json').read_bytes())
def stable():
 assert all(sha(ROOT/n)==h for n,h in pins.items())
 assert all(sha(p)==h for p,h in candidate_pins.items())
 assert all(sha(staged_paths[n])==v['staged_sha256'] for n,v in path_adaptations.items())
 assert all(sha(v['path'])==v['sha256'] for v in state['libraries'].values())
 assert sha(probe)==sha(out/'probe.rs')==probe_digest
 assert all(sha(v['path'])==v['sha256'] for v in state['baseline_gates'].values())
def child(arguments,label):
 folder=out/label;folder.mkdir()
 with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
  p=subprocess.Popen(arguments,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
  k=ctypes.WinDLL('kernel32',use_last_error=True)
  k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
  times=[wintypes.FILETIME() for _ in range(4)]
  assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times])
  ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
  row=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=arguments,output=str(folder))
  state['owned_process']=row;state['process_history'].append(row);save(out/'receipt.json',state)
  code=p.wait();row['exit_code']=code;state['owned_process']=None;save(out/'receipt.json',state)
  return code
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
error=None
try:
 stable();state['status']='compiling_coherent_candidate_resource_probe';save(out/'receipt.json',state)
 assert child(args,'build')==0, 'Preserve compile failure before repairing this isolated candidate/probe'
 stable();state.update(status='executing_coherent_candidate_resource_probe',product=str(output),product_sha256=sha(output));save(out/'receipt.json',state)
 assert child([str(output)],'runtime')==0, 'Preserve actual candidate resource execution failure before repair'
 stable();observation=read(out/'probe-observation.json')
 assert len(observation['frames'])==128 and observation['actual_new_sessions']==observation['actual_new_casts']==0
 assert not observation['audio_device_opened'] and not observation['full_Model_or_Presentation_executed']
 state.update(status='executed_coherent_candidate_resource_and_Effects_component_probe',observation_sha256=sha(out/'probe-observation.json'))
except BaseException as e:error=e;state.update(status='failed_preserved',error=repr(e))
finally:save(out/'receipt.json',state)
files={p.relative_to(out).as_posix():p for p in out.rglob('*') if p.is_file()}
files.update({'candidate-sources/'+Path(n).name:CANDIDATE/'sources'/Path(n).name for n in m['after']})
files['candidate-before.zip']=CANDIDATE/'before.zip'
if output.exists():files['candidate-resource-probe.exe']=output
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
 assert set(z.namelist())==set(hashes)
 for n,h in hashes.items():
  with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(status=state['status'],source_manifest_sha256=m['base_source_manifest_sha256'],
 raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,product_source_changed=False,
 normal_product_build=False,actual_new_sessions=0,actual_new_casts=0,full_Model_or_Presentation_executed=False,
 GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False))
print(json.dumps(dict(output=str(out),status=state['status'],raw_archive_sha256=sha(out/'raw.zip')),ensure_ascii=False))
if error:raise error
