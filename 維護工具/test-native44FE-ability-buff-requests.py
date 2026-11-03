"""Small isolated request/selection component; no scene load or live server workload."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
candidate=ROOT/'維護工具/candidates/native44FE-ability-buff-requests';prep=read(candidate/'preparation.json');assert not prep['candidate_applied']and not prep['normal_source_modified']
build=V/'native44FE-world-tree/compile-20261018-173453-542713';b=read(build/'receipt.json');assert b['status']=='compiled_current_production_Presentation_test'and not b['source_overlays_used']
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041
def guard():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    for n,h in prep['files'].items():assert sha(candidate/n)==h,n
    for n,h in prep['base_files'].items():assert sha(P/'src/modern_lito'/n)==h,n
    for row in b['libraries'].values():assert sha(row['path'])==row['sha256']
    assert sha(prep['handler'])==prep['handler_sha256']and sha(prep['computed_shapes'])==prep['computed_shapes_sha256']
guard();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=V/'native44FE-ability-buff-requests'/('component-'+stamp);out.mkdir(parents=True)
work=P/'target/current/native44FE-ability-buff-requests'/stamp;work.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'preparation-before.json').write_bytes((candidate/'preparation.json').read_bytes())
for n in prep['files']:(out/n).write_bytes((candidate/n).read_bytes())
probe=ROOT/'維護工具/candidates/native44FE-skill-book-audit/ability_buff_component.rs';(out/probe.name).write_bytes(probe.read_bytes())
live=candidate/'live_probe.rs';(out/live.name).write_bytes(live.read_bytes());live_sha=sha(live)
(out/'native_ability_buffs.rs').write_bytes((candidate/'native_ability_buffs.rs').read_bytes())
(out/'ability-codecs.json').write_bytes((Path(prep['current_codec_evidence'])/'codec.stdout').read_bytes())
(out/'useAbilityBuffSpell.javap.txt').write_bytes(Path(prep['handler']).read_bytes())
exe=work/'ability-buff-component.exe';rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
args=[str(rust),str(out/probe.name),'--edition=2021','--crate-name','native44FE_ability_buff_component','--test','--target','x86_64-pc-windows-msvc','-C','debuginfo=1','-C','opt-level=0','-o',str(exe)]
for d in ['target/current/x86_64-pc-windows-msvc/debug/deps','target/current/debug/deps']:args+=['-L','dependency='+str(P/d)]
for n,row in b['libraries'].items():args+=['--extern',n+'='+row['path']]
command=b['compiler']['command']
for i,word in enumerate(command[:-1]):
    if word=='-L'and not command[i+1].startswith('dependency='):args+=['-L',command[i+1]]
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='compiling_isolated_ability_buff_component',source_manifest_sha256=prep['source_manifest_sha256'],candidate_files=prep['files'],normal_source_modified=False,candidate_applied=False,
    runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,actual_sessions=0,actual_casts=0,GUI_verified=False,SPR_decode_verified=False,PCM_decode_verified=False,whole_goal_complete=False)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
save(out/'receipt.json',state);print('ABILITY_BUFF_COMPONENT='+str(out),flush=True)
try:
    live_args=list(args);live_args[1]=str(out/live.name);live_args[live_args.index('--crate-name')+1]='native44FE_ability_live_API_typecheck';live_args[live_args.index('-o')+1]=str(work/'ability-live-API.exe')
    for label,command in [('compile',args),('compile_live_API_only',live_args),('component',[str(exe),'--nocapture','--test-threads=1'])]:
        with(out/(label+'.stdout')).open('wb')as stdout,(out/(label+'.stderr')).open('wb')as stderr:
            child=subprocess.Popen(command,cwd=work,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,label=label);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            try:job['exit_code']=child.wait(timeout=60)
            except subprocess.TimeoutExpired:
                assert birth(wintypes.HANDLE(int(child._handle)))==job['creation_utc'];job['cleanup_stopped_original_owned_handle']=True;child.terminate();job['exit_code']=child.wait(timeout=3);raise
            finally:state['owned_process']=None;save(out/'receipt.json',state)
        assert job['exit_code']==0,label
    text=(out/'component.stdout').read_text('utf-8-sig')
    assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('5','0','0','0','0')]
    assert len(re.findall(r'^test \S+ \.\.\. ok$',text,re.M))==5
    assert sha(live)==live_sha
    state.update(status='isolated_ability_request_and_selection_component_passed',passed=5,failed=0,ignored=0,product=str(exe),product_sha256=sha(exe),live_probe_normal_API_typecheck_verified=True,live_probe_executed=False,live_probe_sha256=live_sha)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:guard();save(out/'receipt.json',state)
files={p.name:p for p in out.iterdir()if p.is_file()};pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),component_only=True,actual_casts=0,normal_product_modified=False))
prep.update(typecheck_verified=True,packet_guard_component_verified=True,component=str(out),component_readback_sha256=sha(out/'readback.json'),actual_sessions=0,actual_casts=0,live_probe_normal_API_typecheck_verified=True,live_probe_executed=False,live_probe_sha256=live_sha)
prep['component_passed']=5
if prep.get('world_integration_typecheck_verified'):
    metadata=read(prep['world_integration_typecheck'])
    assert sha(prep['world_integration_typecheck'])==prep['world_integration_typecheck_sha256']
    for n in ['native_world.rs','native_ability_buffs.rs','native_skills.rs','native_skill_selection.rs']:
        assert sha(candidate/n)==metadata['raw_files'][n],n
    prep['world_integration_metadata_applies_to_runtime_modules_only']=True
save(candidate/'preparation.json',prep)
print(json.dumps(dict(evidence=str(out),passed=5,normal_modified=False,actual_casts=0),ensure_ascii=False))
