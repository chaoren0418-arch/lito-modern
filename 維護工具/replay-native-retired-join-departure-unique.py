"""Compile the unchanged controller plus a shadow-state-only ordering probe."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,tomllib,uuid
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';SRC=P/'src/modern_lito'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=ROOT/'驗證紀錄/current/native-cone-cold-normal/validation-20261018-055501-340560'
ap=argparse.ArgumentParser();ap.add_argument('--candidate',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],default='Debug');a=ap.parse_args()
c=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads')
completion=read(normal/'completion-observation.json');assert completion['observed_terminal_exit_code']==0 and completion['recorded_original_owned_processes_all_terminal'] and completion['source_manifest_sha256']==c['source_manifest_sha256']
assert queue['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+'-'+uuid.uuid4().hex[:8]
out=ROOT/'驗證紀錄/current/native-retired-join-order'/stamp;out.mkdir(parents=True)
runtime=P/'target/current/native-retired-join-order'/stamp;runtime.mkdir(parents=True)
candidate=a.candidate.resolve(strict=True)if a.candidate else None
if candidate:
    assert candidate.is_relative_to(ROOT/'驗證紀錄/current/native-retired-join-order')
    proposal=read(candidate/'proposal.json')
    for n,h in proposal['sources'].items():assert sha(candidate/n)==h
    assert sha(candidate/'probe-addon.rs')==proposal['probe_addon_sha256']
original=((candidate if candidate else SRC)/'native_login.rs').read_text('utf-8-sig')
probe=(ROOT/'維護工具/candidates/native-retired-join/unchanged_probe.rs').read_text('utf-8-sig')
if candidate:probe=(candidate/'probe-addon.rs').read_text('utf-8-sig')
(out/'native_login.rs').write_text(original+'\n'+probe,encoding='utf-8')
lines=[];previous_path=False
for line in (SRC/'mod.rs').read_text('utf-8-sig').splitlines():
    match=re.match(r'\s*#\[path\s*=\s*"([^"\n]+)"\]\s*$',line)
    if match:
        resolved=(SRC/match.group(1)).resolve(strict=True)
        lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']');previous_path=True;continue
    declaration=re.match(r'^(?:(?:pub(?:\([^)]*\))?)\s+)?mod\s+(\w+);\s*$',line)
    if declaration:
        name=declaration.group(1)+'.rs'
        if not previous_path:
            resolved=out/name if name=='native_login.rs' else (candidate/name if candidate and name in proposal['sources'] else SRC/name)
            if not resolved.is_file():resolved=SRC/declaration.group(1)/'mod.rs'
            assert resolved.is_file();lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']')
        previous_path=False
    lines.append(line)
resolved_mod=out/'resolved_mod.rs';resolved_mod.write_text('\n'.join(lines)+'\n',encoding='utf-8')
source=out/'probe.rs'
source.write_text('#[path = '+json.dumps(resolved_mod.as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n#[path = '+json.dumps((P/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\nfn main()->anyhow::Result<()>{let output=std::env::args().nth(1).expect("output");let value=modern_lito::native_login::pending_join_departure_replay()?;std::fs::write(output,serde_json::to_vec_pretty(&value)?)?;Ok(())}\n',encoding='utf-8')
libraries={};native=set()
for line in (normal/a.profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        for path in row['filenames']:
            if path.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/'in path.replace('\\','/'):libraries[name]=path
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe'
exe=runtime/'pending-join-replay.exe'
command=[str(rust),str(source),'--edition=2021','--crate-name','native_pending_join_replay','--crate-type','bin','--target','x86_64-pc-windows-msvc','-L','dependency='+str(P/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps')),'-L','dependency='+str(P/('target/current/'+a.profile.lower()+'/deps')),'-o',str(exe)]
release=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['profile']['release'];assert tuple(release[k]for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
if a.profile=='Release':
    for key in ['opt-level','lto','codegen-units','strip']:command+=['-C',key+'='+str(release[key])]
for n,p in sorted(libraries.items()):command+=['--extern',n+'='+p]
for p in sorted(native):command+=['-L',p]
env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;k.GetCurrentProcess.restype=wintypes.HANDLE
def identity(handle,pid):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return dict(pid=pid,creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat())
state=dict(status='compiling_controller_shadow_replay',profile=a.profile,candidate=str(candidate)if candidate else None,candidate_proposal_sha256=sha(candidate/'proposal.json')if candidate else None,candidate_sources=proposal['sources']if candidate else {},output=str(out),runtime=str(runtime),base_source_manifest_sha256=c['source_manifest_sha256'],normal_product_modified=False,source_controller_sha256=sha(SRC/'native_login.rs'),diagnostic_controller_sha256=sha(out/'native_login.rs'),libraries={n:dict(path=p,sha256=sha(p))for n,p in libraries.items()},process_history=[],original_runner=identity(k.GetCurrentProcess(),os.getpid()),new_network_sessions=0,GUI_verified=False,whole_goal_complete=False)
save(out/'compiled-inputs.json',inputs);(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'receipt.json',state)
print('PENDING_JOIN_REPLAY='+str(out),flush=True)
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb')as stdout,(out/(label+'-stderr.log')).open('wb')as stderr:
        p=subprocess.Popen(args,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(**identity(wintypes.HANDLE(int(p._handle)),p.pid),role=label);state['process_history'].append(job);save(out/'receipt.json',state)
        code=p.wait();job['exit_code']=code;save(out/'receipt.json',state);assert code==0,(label,code)
execute(command,'compile');state['product_sha256']=sha(exe);state['product']=str(exe)
execute([str(exe),str(out/'replay.json')],'replay')
for n,h in inputs.items():assert sha(ROOT/n)==h
for pin in state['libraries'].values():assert sha(pin['path'])==pin['sha256']
expected='candidate_retired_peer_before_pending_join_contracts_passed'if candidate else 'unchanged_current_controller_retired_D0_intercept_reproduced'
assert read(out/'replay.json')['status']==expected
state['status']='executed_candidate_controller_shadow_contracts'if candidate else 'executed_unchanged_controller_shadow_replay_red_reproduced';save(out/'receipt.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],new_network_sessions=0,normal_product_modified=False),ensure_ascii=False),flush=True)
