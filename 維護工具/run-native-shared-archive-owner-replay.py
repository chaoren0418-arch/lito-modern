"""Compile both probes first, then time each captured town replay without a compiler."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,subprocess,shutil
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
BASE=ROOT/'驗證紀錄/current/native-shared-archive-owner'
ap=argparse.ArgumentParser();ap.add_argument('--related-run',required=True,type=Path);a=ap.parse_args()
PAIR=a.related_run.resolve(strict=True);assert PAIR.is_relative_to(BASE)
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
prerequisite=read(PAIR/'completion-observation.json');assert prerequisite['observed_terminal_exit_code']==0 and prerequisite['recorded_owned_processes_all_terminal']
assert prerequisite['status']=='independently_qualified_shared_archive_owner_both_profiles'
pins=read(PAIR/'compiled-inputs.json');assert len(pins)==1025
source=ROOT/'維護工具/oracle/NativeTownLoadReplay.rs'
fixture=ROOT/'驗證紀錄/current/native-G06-controller/20261017-034558-407944/town-remote-actor-replay-20261017-040540-289846/fixture.json'
f=read(fixture);assert sha(f['baseline_fixture'])==f['baseline_fixture_sha256'] and sha(f['additional_capture'])==f['additional_capture_sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=BASE/('replay-'+stamp);out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'probe.rs').write_bytes(source.read_bytes());(out/'fixture.json').write_bytes(fixture.read_bytes())
runtime=PROJECT/'target/current/native-shared-archive-owner-replay'/stamp;runtime.mkdir(parents=True)
state=dict(status='preparing_same_source_offline_town_replay',source_manifest_sha256=prerequisite['source_manifest_sha256'],fixture_source_manifest_sha256=f['source_manifest_sha256'],related_qualification=str(PAIR/'completion-observation.json'),related_qualification_sha256=sha(PAIR/'completion-observation.json'),output=str(out),runtime=str(runtime),
 source_sha256=sha(source),fixture_sha256=sha(fixture),actual_new_sessions=0,product_source_changed_during_diagnostic=False,OS_cache_not_flushed=True,other_host_load_not_excluded=True,performance_accepted=False,
 GUI_verified=False,whole_goal_complete=False,profiles={},process_history=[],owned_process=None)
def stable():
 for n,h in pins.items():assert sha(ROOT/n)==h,n
 assert sha(source)==state['source_sha256'] and sha(fixture)==state['fixture_sha256']
def persist():save(out/'cohort.json',state)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(PROJECT),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),
 PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,folder):
 stable();folder.mkdir()
 with (folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
  p=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
  times=[wintypes.FILETIME() for _ in range(4)];k=ctypes.WinDLL('kernel32',use_last_error=True)
  k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetProcessTimes.restype=wintypes.BOOL
  assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)),*[ctypes.byref(t) for t in times]);tick=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
  item=dict(pid=p.pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=tick//10)).isoformat(),arguments=args)
  state['owned_process']=item;state['process_history'].append(item);persist();code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
 stable();return code
stable();persist();print(json.dumps(dict(output=str(out),runtime=str(runtime)),ensure_ascii=False),flush=True)
try:
 for profile in ['Debug','Release']:
  folder=out/profile;folder.mkdir();target=runtime/profile;target.mkdir();(target/'fixture.json').write_bytes(fixture.read_bytes())
  log=PAIR/profile/'build/stdout.log'
  deps=PROJECT/('target/current/x86_64-pc-windows-msvc/'+profile.lower()+'/deps');libraries={};native=set()
  for line in log.read_text(encoding='utf-8-sig').splitlines():
   if not line.startswith('{'):continue
   m=json.loads(line)
   if m.get('reason')=='build-script-executed':native.update(m.get('linked_paths',[]))
   if m.get('reason')=='compiler-artifact' and m['target']['name'] in ['lito_modern','serde_json']:
    paths=[p for p in m['filenames'] if p.endswith('.rlib')]
    if paths:libraries[m['target']['name']]=paths[0]
  assert set(libraries)=={'lito_modern','serde_json'}
  exe=target/'town-load-replay.exe';args=[str(RUST/'rustc.exe'),str(out/'probe.rs'),'--edition=2021','--crate-name','native_town_load_replay',
   '--target','x86_64-pc-windows-msvc','-L','dependency='+str(deps),'-L','dependency='+str(PROJECT/('target/current/'+profile.lower()+'/deps')),'-o',str(exe)]
  args+=['-C','opt-level=0'] if profile=='Debug' else ['-C','opt-level=z','-C','lto=thin','-C','codegen-units=16','-C','strip=symbols']
  for n,p in libraries.items():args+=['--extern',n+'='+p]
  for p in sorted(native):args+=['-L',p]
  state.update(status='compiling_offline_town_probe',active_profile=profile)
  state['profiles'][profile]=dict(executable=str(exe),libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},compiler_arguments=args)
  persist();assert execute(args,folder/'build')==0,'Preserve diagnostic compile failure before repair'
  state['profiles'][profile]['executable_sha256']=sha(exe);persist()
 for profile in ['Debug','Release']:
  state.update(status='executing_offline_captured_town_replay',active_profile=profile);persist();info=state['profiles'][profile]
  assert execute([info['executable'],r'D:\lineage開發\0-客戶端',str(runtime/profile)],out/profile/'run')==0,'Preserve actual diagnostic failure'
  result=runtime/profile/'results.json';shutil.copyfile(result,out/profile/'results.json');v=read(result)
  assert [r['actors'] for r in v['records']]==[14,15]
  assert v['records'][0]['load_profile_us']['shared_archive_sources_reused'] is False
  assert v['records'][1]['load_profile_us']['shared_archive_sources_reused'] is True
  info.update(results_sha256=sha(result),status=v['status']);persist()
  print(json.dumps(dict(profile=profile,records=[{k:r[k] for k in ['role','load_us','first_draw_us','load_profile_us']} for r in v['records']]),ensure_ascii=False),flush=True)
 state.update(status='executed_both_offline_captured_town_load_diagnostics',active_profile=None)
except BaseException as e:state.update(status='failed_preserved',error=repr(e));raise
finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();persist()
