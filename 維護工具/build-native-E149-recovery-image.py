"""Link an isolated candidate rlib and formal-case PE; never execute outputs."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,stat,subprocess,time,tomllib,traceback
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,owned,sha,encoded,validate_plan,check_pins

def digest(path):
 with Path(path).open('rb') as file:return hashlib.file_digest(file,'sha256').hexdigest().upper()

def save(path,value):
 path=owned(path);temp=path.with_name(path.name+'.writing')
 with temp.open('xb') as file:file.write(encoded(value))
 os.replace(temp,path)

def plain(path):
 path=Path(path).absolute()
 for part in [path,*path.parents]:
  if part.exists():assert not part.lstat().st_file_attributes&stat.FILE_ATTRIBUTE_REPARSE_POINT,part
 return path

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--plan',type=Path,required=True);a=ap.parse_args()
 plan_path=owned(a.plan.resolve(strict=True));p=validate_plan(plan_path);profile=p['profile']
 w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
 # Compilation is permitted in either phase; never compile beside an active
 # timed case, and never interpret building as permission to execute outputs.
 qpath=Path(w['current_development_typecheck_qualification']['qualification']);q=read(qpath)
 assert digest(qpath)==w['current_development_typecheck_qualification']['qualification_sha256']
 original=Path(q['run']);cohort=read(original/'cohort.json')
 assert q['source_manifest_sha256']==SOURCE==p['source_manifest_sha256']
 assert all(q['candidates'][key]['sha256']==row['sha256'] for key,row in w['current_isolated_development']['candidates'].items())
 # Integrated overlay only replaces three QA paths and adds Java bootstrap.
 # All Runtime paths and their fully qualified generated module graph remain
 # exactly those reviewed by the original twelve metadata compiler runs.
 expected_overrides={'LiTo381-Modern/tests/native_teleport_failure.rs',
  'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java',
  'LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java',
  'LiTo381-Modern/tests/oracle/NativeRecoveryProductionBootstrap.java'}
 assert q['candidate_overlay_files']==13
 old_files={}
 for row in w['current_isolated_development']['candidates'].values():
  capsule=read(row['path']);assert digest(row['path'])==row['sha256']
  for name,h in capsule['candidate_files'].items():assert name not in old_files;old_files[name]=h
 assert {n for n in p['effective_overlay_files'] if old_files.get(n)!=p['effective_overlay_files'][n]}==expected_overrides
 rows=[row for row in cohort['process_history'] if Path(row['output']).parent.name==profile]
 library_rows=[row for row in rows if Path(row['output']).name=='library-compile']
 test_rows=[row for row in rows if Path(row['output']).name=='teleport-recovery-formal-code-compile']
 assert len(library_rows)==len(test_rows)==1
 rustc=Path(library_rows[0]['command'][0]);assert rustc==Path(test_rows[0]['command'][0])
 toolchain=rustc.parent.parent;external_pins={str(rustc):digest(rustc)}
 # Hash existing compiler/standard-library inputs in place; never copy them.
 for folder,patterns in [(toolchain/'lib/rustlib/x86_64-pc-windows-msvc/lib',['*.rlib']),
  (toolchain/'lib/rustlib/x86_64-pc-windows-msvc/codegen-backends',['*.dll']),
  (toolchain/'bin',['rustc_driver*.dll','LLVM*.dll'])]:
  if folder.exists():
   for pattern in patterns:
    for path in folder.glob(pattern):external_pins[str(plain(path))]=digest(path)
 normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run'])
 cargo_log=normal/profile/'test-build/stdout.log';messages=[json.loads(line) for line in cargo_log.read_text('utf-8-sig').splitlines() if line.startswith('{')]
 native=sorted({value for row in messages if row.get('reason')=='build-script-executed' for value in row.get('linked_paths',[])})
 pins={str(R/name):value for name,value in read(p['base_compiled_inputs']).items()}
 pins.update(p['input_pins']);pins.update({p['effective_overlay_paths'][n]:h for n,h in p['effective_overlay_files'].items()})
 for name,h in cohort['input_pins'].items():
  if Path(name).is_relative_to(R):pins[name]=h
  else:assert Path(name)==rustc and h==external_pins[str(rustc)]
 for path in [plan_path,qpath,original/'cohort.json',original/'module-map.json',cargo_log,Path(p['base_compiled_inputs']),Path(__file__).resolve()]:pins[str(path)]=digest(path)
 for row in messages:
  if row.get('reason')!='compiler-artifact':continue
  for name in row.get('filenames',[]):
   path=Path(name)
   if path.suffix.lower() in ['.rlib','.rmeta','.dll','.lib'] and path.is_file():
    assert path.is_relative_to(P/'target/current');pins[str(plain(path))]=digest(path)
 for flag in native:
  name=flag.split('=',1)[-1];folder=plain(name)
  assert folder.is_relative_to(R) or folder.is_relative_to(Path(os.environ['USERPROFILE'])/'.cargo/registry'),folder
  assert folder.is_dir()
  for path in folder.iterdir():
   if path.is_file() and path.suffix.lower() in ['.lib','.a']:
    plain(path)
    (pins if path.is_relative_to(R) else external_pins)[str(path)]=digest(path)
 configuration=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))
 release=configuration['profile']['release'];assert tuple(release[k] for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
 def guard():
  check_pins(pins)
  assert all(digest(path)==h for path,h in external_pins.items())
  assert all(digest(path)==h for path,h in p['normal_product_pins'].items())
  assert digest(w['current_formal_reference_identity']['formal'])==FORMAL
 guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+profile.lower()
 out=R/'驗證紀錄/current/native-E149-recovery-linked-build'/stamp;out.mkdir(parents=True)
 products=P/'target/current/verification/native-E149-recovery-linked-images'/stamp;products.mkdir(parents=True)
 (products/'scratch').mkdir();(out/'builder.py').write_bytes(Path(__file__).read_bytes())
 library=products/'liblito_modern.rlib';product=products/'native_teleport_failure.exe'
 kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetCurrentProcess.restype=wintypes.HANDLE
 kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
 def identity(handle,pid):
  times=[wintypes.FILETIME() for _ in range(4)];assert kernel.GetProcessTimes(handle,*[ctypes.byref(t) for t in times])
  ticks=times[0].dwLowDateTime|times[0].dwHighDateTime<<32
  return dict(pid=pid,creation_filetime_ticks=ticks,start_epoch_millis=(ticks-116444736000000000)//10000,
   creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat())
 state=dict(schema='E149-recovery-linked-build-v1',status='preparing_link_commands',profile=profile,plan=str(plan_path),plan_sha256=digest(plan_path),
  source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,effective_overlay_files=p['effective_overlay_files'],
  effective_overlay_paths=p['effective_overlay_paths'],base_compiled_inputs=p['base_compiled_inputs'],
  input_pins=pins,external_toolchain_dependency_pins=external_pins,normal_product_pins=p['normal_product_pins'],
  original_metadata_qualification=str(qpath),original_metadata_qualification_sha256=digest(qpath),
  reused_full_module_graph=str(original/'module-map.json'),native_link_search_paths=native,
  product=str(product),library=str(library),compiler_runs=[],owned_process=None,
  pipeline=identity(kernel.GetCurrentProcess(),os.getpid()),
  tests_executed=0,game_or_fixture_executions=0,SQL_executed=False,database_started=False,desktop_inputs=0,
  outputs_executed=False,normal_source_modified=False,normal_products_modified=False,candidate_applied=False,whole_goal_complete=False)
 save(out/'cohort.json',state);print(json.dumps(dict(run=str(out),profile=profile,products=str(products),mode='link_only_no_output_execution'),ensure_ascii=False),flush=True)
 env={k:v for k,v in os.environ.items() if not k.startswith('LITO_') and k not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']}
 env.update(TEMP=str(products/'scratch'),TMP=str(products/'scratch'),CARGO_MANIFEST_DIR=str(P),
  CARGO_PKG_VERSION=configuration['package']['version'],CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1')
 primary=None
 try:
  for kind,old,output in [('library',library_rows[0],library),('formal_test',test_rows[0],product)]:
   guard();command=list(old['command']);assert command.count('--emit=metadata')==1
   command[command.index('--emit=metadata')]='--emit=link';command[command.index('-o')+1]=str(output)
   if kind=='formal_test':
    before=str(original/'generated/LiTo381-Modern/tests/native_teleport_failure.rs');assert command.count(before)==1
    command[command.index(before)]=p['effective_overlay_paths']['LiTo381-Modern/tests/native_teleport_failure.rs']
    before='lito_modern='+str(original/profile/'library.rmeta');assert command.count(before)==1
    command[command.index(before)]='lito_modern='+str(library)
   for flag in native:command+=['-L',flag]
   assert Path(command[0])==rustc and '--emit=link' in command
   folder=out/kind;folder.mkdir();state['status']='linking_'+kind
   with(folder/'stdout.log').open('xb') as stdout,(folder/'stderr.log').open('xb') as stderr:
    child=subprocess.Popen(command,cwd=products,env=env,stdout=stdout,stderr=stderr,
     creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
    row=dict(identity(int(child._handle),child.pid),kind=kind,command=command,output=str(output),exit_code=None,
     compiler_sha256=external_pins[str(rustc)],wait_confirmed=False,started_utc=datetime.now(timezone.utc).isoformat())
    state['compiler_runs'].append(row);state['owned_process']=row;save(out/'cohort.json',state)
    deadline=time.monotonic()+1200
    try:
     while True:
      try:child.wait(timeout=20);break
      except subprocess.TimeoutExpired:
       assert child.poll() is None
       state['last_live_observation_utc']=datetime.now(timezone.utc).isoformat();save(out/'cohort.json',state)
       print(json.dumps(dict(profile=profile,phase=kind,pid=child.pid,original_handle_live=True)),flush=True)
       if time.monotonic()>=deadline:raise TimeoutError('owned compiler exceeded 1200 seconds; original failure retained')
     row['wait_confirmed']=True
    except BaseException:
     if child.poll() is None:
      child.terminate();child.wait(timeout=30);row['exact_original_handle_cleanup']=True
     raise
    finally:
     row.update(exit_code=child.poll(),finished_utc=datetime.now(timezone.utc).isoformat(),
      logs={str(folder/n):digest(folder/n) for n in ['stdout.log','stderr.log']})
     state['owned_process']=None;save(out/'cohort.json',state)
   assert row['exit_code']==0,(kind,str(folder))
   assert output.is_file() and output.stat().st_size>0
   row.update(output_sha256=digest(output),output_bytes=output.stat().st_size);pins[str(output)]=digest(output);guard()
   save(out/'cohort.json',state)
  state.update(status='linked_compilers_exited',product_sha256=digest(product),library_sha256=digest(library),
   output_files={str(path):digest(path) for path in products.iterdir() if path.is_file()})
 except BaseException as error:
  primary=error;state.update(status='link_failure_preserved',error=repr(error))
  (out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8')
 finally:
  try:guard()
  except BaseException as error:
   state.update(input_guard_failure=repr(error),status='link_or_input_failure_preserved')
   if primary is None:primary=error
  state['finished_utc']=datetime.now(timezone.utc).isoformat();save(out/'cohort.json',state)
 print(json.dumps(dict(run=str(out),status=state['status'],profile=profile,
  compiler_exit_codes=[x['exit_code'] for x in state['compiler_runs']],tests_executed=0,outputs_executed=False),ensure_ascii=False),flush=True)
 if primary is not None:raise primary

if __name__=='__main__':main()
