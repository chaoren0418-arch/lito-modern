"""Build and exercise the Presentation source-owner correction, then the unique normals."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,zipfile,time,tomllib

ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
now=lambda:datetime.now(timezone.utc).isoformat()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def retain(path,files):
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,h in hashes.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    return dict(path=str(path),sha256=sha(path),files=hashes,readback_verified=True)


application=Path(read(ROOT/'驗證紀錄/current/native-selector-integration/active-application.json')['path']).resolve(strict=True)
assert application.is_relative_to(ROOT/'驗證紀錄/current/native-selector-integration')
app=read(application/'application.json');assert app['status']=='applied_selector_integration_pending_current_product_validation' and app['applied']
pins=read(application/'applied-source-inputs.json');inputs=read(application/'applied-compiled-inputs.json');assert len(pins)==1028 and len(inputs)==1029 and sha(application/'applied-source-inputs.json')==app['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release'];assert (release['opt-level'],release['lto'],release['codegen-units'],release['strip'])==('z','thin',16,'symbols')
targets=['native_selector_only','native_skills','native_skill_selection','native_attack_skills','native_lightning','native_chill_touch','native_ice_arrow','native_fire_arrow','native_wind_cutter','native_stalac_layers','native_vampiric_touch','native_ground_skills','native_reagent_skills','native_ranged','native_audio_device','native_effect_lifecycle','native_world_effects','native_animation','native_animation_resources','native_resource_loading','native_world_lighting_scene','native_render_environment','native_light_lifecycle','native_painted_input','native_terrain_residence','native_startup','native_login','native_world_entry']
assert len(targets)==len(set(targets))==28
for n in targets:assert (PROJECT/'tests'/(n+'.rs')).is_file(),n
inventory_cohort=read(ROOT/'驗證紀錄/current/native-default110/20261017-063056-912429/cohort.json');baseline_inventory={}
for profile in ['Debug','Release']:
 p=Path(inventory_cohort['profiles'][profile]['qualification']);assert sha(p)==inventory_cohort['profiles'][profile]['qualification_sha256'];q=read(p);assert q['source_manifest_sha256']==app['base_source_manifest_sha256'];baseline_inventory[profile]=dict(path=str(p),sha256=sha(p),names=q['all_test_names'])
new_tests=['nine_server_selectors_ignore_hover_and_never_spend_local_authority','selector_learning_death_and_mana_guards_do_not_forge_requests','counter72_actual_s39_acks_cast_without_s37_or_local_resource_damage','counter72_malformed_capture_is_atomic_and_unknown_profiles_stay_quarantined']
out=ROOT/'驗證紀錄/current/native-selector-integration'/('validation-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'source-inputs.json',pins);save(out/'compiled-inputs.json',inputs)
source=retain(out/'compiled-source.zip',{name:ROOT/name for name in inputs});entry_hash=sha(ROOT/'啟動現代客戶端.ps1');reference=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';reference_hash=sha(reference);assert reference_hash==app['reference_jar_sha256']
state=dict(status='prepared_current_selector_related_validation',output=str(out),application=str(application),application_sha256=sha(application/'application.json'),source_files=len(pins),compiled_input_files=len(inputs),source_manifest_sha256=sha(out/'source-inputs.json'),compiled_source_archive=source,targets=targets,expected_new_tests=new_tests,old_qualified_inventory_only={p:{k:v for k,v in row.items() if k!='names'} for p,row in baseline_inventory.items()},entry_sha256=entry_hash,reference_jar_sha256=reference_hash,profiles={},owned_process=None,process_history=[],observed_release_compilers=[],actual_formal_sessions=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,started_utc=now())
persist=lambda:save(out/'cohort.json',state)
env=os.environ.copy();env.update(RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'));env['PATH']=str(RUST)+os.pathsep+env['PATH']
def check():
 for n,h in inputs.items():assert sha(ROOT/n)==h,n
 assert sha(ROOT/'啟動現代客戶端.ps1')==entry_hash and sha(reference)==reference_hash

def observe_release_compiler(parent,folder):
 command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter \"ParentProcessId="+str(parent)+" AND Name='rustc.exe'\"|Select-Object ProcessId,ParentProcessId,CommandLine,@{n='CreationUtc';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 p=subprocess.run(['pwsh.exe','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
 rows=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];rows=[rows] if isinstance(rows,dict) else rows
 for row in rows:
  assert row['ParentProcessId']==parent
  row['observed_utc']=now();row['parent_job_output']=str(folder)
  if not any((v['ProcessId'],v['CreationUtc'])==(row['ProcessId'],row['CreationUtc']) for v in state['observed_release_compilers']):state['observed_release_compilers'].append(row)
 save(out/'release-compiler-observations.json',state['observed_release_compilers']);persist()

def execute(command,folder):
 folder.mkdir()
 with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
  p=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);times=[wintypes.FILETIME() for _ in range(4)];get=ctypes.WinDLL('kernel32',use_last_error=True).GetProcessTimes;get.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;get.restype=wintypes.BOOL;assert get(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(v) for v in times]);ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
  item=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(),arguments=command,output=str(folder));state['owned_process']=item;state['process_history'].append(item);persist();last_observation=time.monotonic()
  while p.poll() is None:
   if state.get('active_profile')=='Release' and folder.name=='build' and time.monotonic()-last_observation>=15:observe_release_compiler(p.pid,folder);last_observation=time.monotonic()
   time.sleep(.25)
  code=p.wait();item.update(exit_code=code,finished_utc=now());state['owned_process']=None;persist()
 return code
persist();print(json.dumps(dict(output=str(out),source_manifest_sha256=state['source_manifest_sha256']),ensure_ascii=False),flush=True)
try:
 for profile in ['Debug','Release']:
  check();d=out/profile;d.mkdir();state.update(status='compiling_current_selector_related_targets',active_profile=profile);info=dict(tests={},products={});state['profiles'][profile]=info;persist()
  command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--bin','LiToModern']
  if profile=='Release':command.append('--release')
  for target in targets:command+=['--test',target]
  code=execute(command,d/'build');info['compile_exit_code']=code;persist();assert code==0,'Preserve this compile failure before repair'
  messages=[json.loads(l) for l in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')];artifacts=[m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']];assert {m['target']['name'] for m in artifacts}==set(targets)|{'LiToModern'} and len(artifacts)==len(targets)+1
  for m in artifacts:
   assert m['features']==[];p=Path(m['executable']).resolve(strict=True);assert p.is_relative_to(PROJECT/'target/current');info['products'][m['target']['name']]=dict(path=str(p),sha256=sha(p),compiler_profile=m['profile'])
  info['test_products_archive']=retain(d/'test-products.zip',{name+'.exe':Path(v['path']) for name,v in info['products'].items()});state['status']='running_current_selector_related_targets';persist()
  for name,product in info['products'].items():
   code=execute([product['path'],'--test-threads=1'],d/('test-'+name));log=(d/('test-'+name)/'stdout.log').read_text(encoding='utf-8-sig');matches=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log);assert len(matches)==1
   status,passed,failed,ignored,measured,filtered=matches[0];rows=re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log,re.M);names=dict(rows);expected=dict(all=new_tests,ignored=[]) if name=='native_selector_only' else baseline_inventory[profile]['names']['main_unit' if name=='LiToModern' else name]
   info['tests'][name]=dict(exit_code=code,status=status,passed=int(passed),failed=int(failed),ignored=int(ignored),measured=int(measured),filtered=int(filtered),rows=names,expected_names=expected);persist()
   assert len(rows)==len(names) and set(names)==set(expected['all']),('registered test inventory changed',name)
   assert {n for n,s in names.items() if s=='ignored'}==set(expected['ignored']),('ignored test inventory changed',name)
   assert code==0 and status=='ok' and int(failed)==int(measured)==int(filtered)==0,('Preserve test failure before repair',name)
  info['actual_test_executables']=len(targets)+1;info['counts']={k:sum(v[k] for v in info['tests'].values()) for k in ['passed','failed','ignored']};persist();check();state['status']='building_unique_entry_normal';persist()
  assert execute(['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile],d/'normal-build')==0
  (d/'entry-build.log').write_bytes((ROOT/'驗證紀錄/current'/('build-release.log' if profile=='Release' else 'build.log')).read_bytes());exe=PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe';info['normal']=dict(path=str(exe),sha256=sha(exe),unique_entry_exit_code=0,archive=retain(d/'normal-product.zip',{'LiToModern.exe':exe}));persist();assert execute([str(exe),'--version'],d/'version')==0;info['version_exit_code']=0;persist();check()
  print(json.dumps(dict(profile=profile,counts=info['counts'],normal=info['normal']['sha256']),ensure_ascii=False),flush=True)
 state.update(status='executed_current_selector_related_tests_and_unique_normals',active_profile=None)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:state['finished_utc']=now();persist()
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
