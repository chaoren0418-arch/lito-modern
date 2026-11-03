"""Execute a coherent candidate resource/Effects probe after current baseline gates."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,tomllib,zipfile,time
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
ap=argparse.ArgumentParser();ap.add_argument('--candidate',type=Path,
    default=ROOT/'維護工具/candidates/native-selector-nine-integration-shared-pool')
ap.add_argument('--library-run',type=Path,required=True)
ap.add_argument('--resource-qualification',type=Path,required=True)
ap.add_argument('--visibility-qualification',type=Path,required=True)
ap.add_argument('--bank-qualification',type=Path,required=True)
ap.add_argument('--probe-directory',type=Path,default=ROOT/'維護工具/candidates/native-selector-nine-runtime-followup')
ap.add_argument('--callback-qualification',type=Path,required=True)
ap.add_argument('--action-profile',choices=['selector-semantic'],default='selector-semantic')
ap.add_argument('--reuse-live-run',type=Path)
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
 'bank':options.bank_qualification.resolve(strict=True),
 'callbacks':options.callback_qualification.resolve(strict=True),
}
expected={
 'library':'independently_qualified_current_full_library_both_profiles_and_terminal_original_runner',
 'resources':'independently_readback_qualified_current_ten_unique_original_resource_bodies_both_profiles',
 'visibility':'independently_qualified_current_normal_visibility_both_profiles',
 'bank':'independently_qualified_candidate_resource_bank_and_observed_Effects_boundary',
 'callbacks':'independently_qualified_actual_candidate_all_nine_finite_callback_lifecycle',
}
for name,p in gates.items():
 assert p.resolve(strict=True).is_relative_to(ROOT/'驗證紀錄/current')
 q=read(p);assert q['status']==expected[name]
 assert q['source_manifest_sha256']==m['base_source_manifest_sha256']
 if name=='library':assert q['observed_terminal_exit_code']==0
assert read(options.library_run/'compiled-inputs.json')==pins
bank_receipt=read(options.bank_qualification.parent.parent/'receipt.json')
assert bank_receipt['candidate_after']==m['after']
assert read(options.bank_qualification)['observed_terminal_exit_code']==0
assert read(options.callback_qualification)['candidate_metadata_sha256']==sha(CANDIDATE/'candidate.json')
assert not read(ROOT/'工作佇列.json').get('active_workload_sessions'), 'Finish and checkpoint all current timing-sensitive jobs first'
probe_directory=options.probe_directory.resolve(strict=True)
assert probe_directory.is_relative_to(ROOT/'維護工具/candidates')
probe=probe_directory/'probe.rs'
preparation=read(probe.parent/'preparation.json')
assert preparation['probe_sha256']==sha(probe)
assert preparation['candidate_metadata_sha256']==sha(CANDIDATE/'candidate.json')
fixture_path=probe_directory/'current-world-packets.json'
assert sha(fixture_path)==preparation['fixture_sha256']
fixture_pins={'current-world-packets.json':sha(fixture_path),'fixture-readback.json':sha(probe_directory/'fixture-readback.json')}
candidate_pins={str(CANDIDATE/'sources'/Path(n).name):h for n,h in m['after'].items()}
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-selector-semantic-live'/stamp;out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-selector-nine-live'/stamp;runtime.mkdir(parents=True)
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
for name in fixture_pins:(out/name).write_bytes((probe_directory/name).read_bytes())

harness=probe_directory/'NativeSelectorHarness.java';assert sha(harness)==preparation['candidate_harness_sha256']
(out/'NativeSelectorHarness.java').write_bytes(harness.read_bytes())
formal=Path(r'D:/lineage開發/3-lito開發/lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')
jar=PROJECT/'target/current/server-reference/formal-server-C02AAAE8.jar'
catalog=PROJECT/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(formal)==sha(jar)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
jdk=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin')
classes=runtime/'oracle';classes.mkdir()

probe_digest=sha(probe)
with libroot.open('a',encoding='utf-8',newline='\n') as f:
 f.write('#[path="'+(out/'probe.rs').as_posix()+'"]\nmod candidate_resource_probe;\n')
 f.write('fn main()->anyhow::Result<()>{candidate_resource_probe::run()}\n')
output=runtime/'candidate-selector-live.exe'
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
 assert sha(harness)==sha(out/'NativeSelectorHarness.java')==preparation['candidate_harness_sha256']
 assert all(sha(p)==h for p,h in state.get('harness_class_pins',{}).items())
 assert sha(formal)==sha(jar)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
 assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
 assert all(sha(ROOT/n)==h for n,h in pins.items())
 assert all(sha(p)==h for p,h in candidate_pins.items())
 assert all(sha(staged_paths[n])==v['staged_sha256'] for n,v in path_adaptations.items())
 assert all(sha(v['path'])==v['sha256'] for v in state['libraries'].values())
 assert sha(probe)==sha(out/'probe.rs')==probe_digest
 assert all(sha(probe_directory/n)==sha(out/n)==h for n,h in fixture_pins.items())
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
 stable()
 reuse_receipt=None
 if options.reuse_live_run is None:
  state['status']='compiling_coherent_candidate_presentation_probe';save(out/'receipt.json',state)
  assert child(args,'build')==0, 'Preserve compile failure before repairing this isolated candidate/probe'
 else:
  reuse=options.reuse_live_run.resolve(strict=True);assert reuse.is_relative_to(ROOT/'驗證紀錄/current/native-selector-nine-live')
  reuse_q_path=reuse/'qualified/qualified.json';reuse_q=read(reuse_q_path);reuse_receipt=read(reuse/'receipt.json')
  assert reuse_q['status']=='independently_qualified_actual_C02_candidate_nine_selector_requests_scope' and reuse_q['profile']=='selector-nine' and reuse_q['observed_terminal_exit_code']==0
  assert reuse_q['candidate_metadata_sha256']==sha(CANDIDATE/'candidate.json') and reuse_receipt['candidate_after']==m['after']
  assert read(reuse/'compiled-inputs.json')==pins and reuse_receipt['probe_sha256']==probe_digest
  assert sha(reuse/'NativeSelectorHarness.java')==sha(harness)
  old_product=Path(reuse_receipt['product']);assert sha(old_product)==reuse_q['actual_product_sha256']
  import shutil
  shutil.copyfile(old_product,output);assert sha(output)==reuse_q['actual_product_sha256']
  old_runtime=old_product.parent
  for old_path,digest in reuse_receipt['harness_class_pins'].items():
   old_class=Path(old_path);assert sha(old_class)==digest
   target=classes/old_class.relative_to(old_runtime/'oracle');target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(old_class,target);assert sha(target)==digest
  state['reused_compile_qualification']={'path':str(reuse_q_path),'sha256':sha(reuse_q_path),'original_run':str(reuse),'original_compiler_processes':reuse_receipt['process_history'][:2]}
  state['current_staged_module_paths_not_compiled']=state['module_paths'];state['module_paths']=reuse_receipt['module_paths']
  state['compiler_executed_this_run']=False;state['compiler_product_reused']=True
 stable();state.update(status='executing_coherent_candidate_presentation_probe',product=str(output),product_sha256=sha(output));save(out/'receipt.json',state)

 if reuse_receipt is None:
  state['status']='compiling_isolated_C02_harness';save(out/'receipt.json',state)
  assert child([str(jdk/'javac.exe'),'-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(out/'NativeSelectorHarness.java')],'javac')==0
 else:state['harness_classes_reused']=True
 state['harness_class_pins']={str(p):sha(p) for p in classes.rglob('*.class')};assert len(state['harness_class_pins'])==2
 stable()
 state.update(status='executing_candidate_selector_requests',resource_profile='tw13081901-v1',extended_item_details=True,reference_jar=str(jar),reference_jar_sha256=sha(jar),common_catalog=str(catalog),common_catalog_sha256=sha(catalog));save(out/'receipt.json',state)
 started=[]
 def launch(arguments,label,child_env):
  folder=out/label;folder.mkdir();stdout=(folder/'stdout.log').open('wb');stderr=(folder/'stderr.log').open('wb')
  process=subprocess.Popen(arguments,cwd=runtime,env=child_env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
  k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
  times=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(wintypes.HANDLE(int(process._handle)),*[ctypes.byref(t) for t in times]);ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
  row=dict(pid=process.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=arguments,output=str(folder),role=label)
  state['process_history'].append(row);entry=(process,row,stdout,stderr);started.append(entry);state['owned_processes']=[e[1] for e in started if e[0].poll() is None];save(out/'receipt.json',state);return entry
 def poll_entry(entry):
  process,row,stdout,stderr=entry;code=process.poll()
  if code is not None:row['exit_code']=code;stdout.close();stderr.close()
  state['owned_processes']=[e[1] for e in started if e[0].poll() is None];save(out/'receipt.json',state);return code
 java_env=dict(env,JAVA_TOOL_OPTIONS='-Dfile.encoding=UTF-8');java_env.pop('_JAVA_OPTIONS',None);java_env.pop('JDK_JAVA_OPTIONS',None)
 try:
  server=launch([str(jdk/'java.exe'),'-Dfile.encoding=UTF-8','-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',str(classes)+';'+str(jar),'local.lito381.server.NativeSelectorHarness',str(runtime),options.action_profile],'server',java_env)
  until=time.monotonic()+15;ready=None
  while ready is None:
   assert poll_entry(server) is None,'Server exited before ready'
   assert time.monotonic()<until,'Existing15s server readiness bound exceeded'
   if (runtime/'ready.json').exists():
    try:ready=read(runtime/'ready.json')
    except json.JSONDecodeError:pass
   if ready is None:time.sleep(.02)
  assert ready['jar_sha256'].upper()==sha(jar)
  clients=[launch([str(output),'127.0.0.1:'+str(ready['port']),str(runtime),role],role,env) for role in ['a','b','c']]
  until=time.monotonic()+225
  while any(poll_entry(e) is None for e in clients):
   for e in clients:assert e[0].poll() in [None,0],f'Client {e[1]["role"]} failed; preserve before repair'
   assert time.monotonic()<until,'Existing225s peer wrapper bound exceeded'
   assert poll_entry(server) in [None,0],'Server failed while clients active'
   time.sleep(.05)
  assert all(poll_entry(e)==0 for e in clients)
  server[0].wait(timeout=15);assert poll_entry(server)==0,'Isolated server failed cleanup'
 finally:
  for e in reversed(started):
   process,row,stdout,stderr=e
   if process.poll() is None:
    command='Get-CimInstance Win32_Process -Filter "ProcessId='+str(process.pid)+'"|Select-Object ProcessId,CommandLine,@{n="created";e={$_.CreationDate.ToUniversalTime().ToString("o")}}|ConvertTo-Json -Compress'
    observed=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert observed.returncode==0 and not observed.stderr
    if not observed.stdout.strip():
     # The original process handle remains authoritative if CIM raced exit.
     process.wait(timeout=3);poll_entry(e);continue
    identity=json.loads(observed.stdout.decode('utf-8-sig'));assert identity['ProcessId']==process.pid and str(runtime).lower() in identity['CommandLine'].lower()
    assert abs((datetime.fromisoformat(identity['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001
    if process.poll() is None:
     row['cleanup_stop_identity']=identity;process.terminate();process.wait(timeout=3)
   poll_entry(e)
 stable()
 result=read(runtime/'result.json');a_client=read(runtime/'a-client.json');b_client=read(runtime/'b-client.json');c_client=read(runtime/'c-client.json')
 assert result['status']=='sessions_closed' and result['actual_client_sessions']==6 and result['authority_mutations']==0 and not result['sql_access'] and not result['state_persistent']
 assert all(v['worker_quiescent'] and v['shutdown']=='closed' for v in [a_client,b_client,c_client])
 requests=[]
 for capture in sorted((runtime/'captures').rglob('*.jsonl')):
  for line in capture.read_text(encoding='utf-8-sig').splitlines():
   packet=json.loads(line)
   if packet.get('direction')=='client_to_server':
    if re.fullmatch('[a-fA-F0-9 ]*',packet.get('plain','')) is None:continue # redacted login capture
    body=bytes.fromhex(packet['plain'])
    if body[:1]==b'\x14':requests.append(body.hex())
 assert sorted(requests)==sorted(['141905']*4+['140600']),requests
 accepted=sum(row['feedback']=='CastAction' for row in b_client['completed'])
 expected_accepts=4
 observation=dict(status='executed_candidate_live_selector_semantics',profile=options.action_profile,actual_new_sessions=6,actual_new_requests=len(requests),actual_new_casts=accepted,expected_accepted_casts=expected_accepts,full_Model_Presentation_executed=True,native_window_refresh_executed=False,physical_audio_opened=False,GUI_verified=False,whole_goal_complete=False,checks=dict(five_semantic_wire_requests_observed=True,server_sessions_closed=True,client_workers_quiescent=True,expected_cast_acceptance_observed=accepted==expected_accepts))
 save(out/'probe-observation.json',observation);state.update(observation_sha256=sha(out/'probe-observation.json'),full_Model_or_Presentation_executed=True,actual_new_sessions=6,actual_new_casts=accepted,actual_new_requests=len(requests))
 assert all(observation['checks'].values()),'Preserve authoritative cast acceptance mismatch before repair'
 state['status']='executed_coherent_candidate_live_selector_semantics'
except BaseException as e:error=e;state.update(status='failed_preserved',error=repr(e))
finally:save(out/'receipt.json',state)
files={p.relative_to(out).as_posix():p for p in out.rglob('*') if p.is_file()}
for item in runtime.rglob('*'):
 if item.is_file() and (item.suffix in ['.json','.jsonl','.png','.properties','.class'] or item.name.startswith('b-skill')):
  files['live-evidence/'+item.relative_to(runtime).as_posix()]=item
files.update({'candidate-sources/'+Path(n).name:CANDIDATE/'sources'/Path(n).name for n in m['after']})
files['candidate-before.zip']=CANDIDATE/'before.zip'
if output.exists():files['candidate-presentation-probe.exe']=output
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
 assert set(z.namelist())==set(hashes)
 for n,h in hashes.items():
  with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(status=state['status'],source_manifest_sha256=m['base_source_manifest_sha256'],
 raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,product_source_changed=False,
 normal_product_build=False,actual_new_sessions=state.get('actual_new_sessions',0),actual_new_casts=state.get('actual_new_casts',0),full_Model_or_Presentation_executed=state.get('full_Model_or_Presentation_executed',False),
 GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False))
print(json.dumps(dict(output=str(out),status=state['status'],raw_archive_sha256=sha(out/'raw.zip')),ensure_ascii=False))
if error:raise error
