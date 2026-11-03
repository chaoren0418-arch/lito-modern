"""Short actual C02 environment proof using the exact already-built Cargo rlib."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, os, shutil, struct, subprocess, time, zipfile

ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
TOOLS=Path(r'C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin')
JDK=Path(r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],required=True);args=ap.parse_args()
run=args.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-environment')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def write(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
qualification=read(run/'independent-qualified.json');assert qualification['all_archives_readback_verified']
assert read(run/'receipt.json')['status']=='completed_GAME_environment_related_scoped_checks_and_unique_entry_mains'
pins={**read(run/'source-inputs.json'),**read(run/'fixture-inputs.json')}
def stable():
    for name,h in pins.items():assert sha(ROOT/name)==h,name
    for v in read(run/'normal/qualified.json')['profiles'].values():assert sha(Path(v['path']))==v['sha256']
stable()
guard=subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',r"""
$root='D:\lineage開發\3-lito開發\lito重建版'
$live=@(Get-CimInstance Win32_Process|Where-Object {$_.Name -in @('cargo.exe','rustc.exe') -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($root)) -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root,[StringComparison]::OrdinalIgnoreCase) -and $_.Name -match '^(LiToModern|native_|qualified-client)')})
if($live.Count){throw 'Prior owned native/compiler/Java workload live'}
"""],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=30)
assert guard.returncode==0,guard.stderr
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=run/'formal'/(stamp+'-'+args.profile);out.mkdir(parents=True)
runtime=PROJECT/'target/current/native-environment-formal'/args.profile;runtime.mkdir(parents=True,exist_ok=True)
jar=PROJECT/'target/current/server-reference/formal-server-C02AAAE8.jar'
catalog=PROJECT/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(jar)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
env=os.environ.copy();env.update(PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',
    TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PATH=str(TOOLS)+os.pathsep+env['PATH'])
rust_source=ROOT/'維護工具/oracle/native_environment_probe.rs'
java_source=ROOT/'維護工具/oracle/NativeEnvironmentHarness.java'
for src,name in [(rust_source,'client.rs'),(java_source,'harness.java'),(Path(__file__),'runner.py')]:
    (out/name).write_bytes(src.read_bytes())
state={'status':'starting','profile':args.profile,'output':str(out),'source_files':978,
       'source_manifest_sha256':qualification['source_manifest_sha256'],'scoped_qualification_sha256':sha(run/'independent-qualified.json'),
       'normal_mains':qualification['normal_mains'],'actual_formal_sessions':0,'whole_goal_complete':False,'owned_processes':{}}
children=[];handles=[]
def persist():write(out/'receipt.json',state)
def launch(name,command,cwd):
    stable();write(out/(name+'-command.json'),{'args':list(map(str,command)),'cwd':str(cwd)})
    stdout=(out/(name+'.stdout')).open('wb');stderr=(out/(name+'.stderr')).open('wb');handles.extend([stdout,stderr])
    p=subprocess.Popen(command,cwd=cwd,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    children.append((name,p));state['owned_processes'][name]={'pid':p.pid,'started_utc':datetime.now(timezone.utc).isoformat(),'identity':'retained Popen handle','status':'running'};persist();return p
def finish(name,p,timeout):
    code=p.wait(timeout=timeout);state['owned_processes'][name].update(status='exited',exit_code=code);persist();assert code==0,(name,code)
persist()
try:
    messages=[]
    for line in (run/(args.profile+'-compile.stdout')).read_text('utf-8-sig').splitlines():
        if line.startswith('{'):messages.append(json.loads(line))
    libraries={}
    for name in ['lito_modern','serde_json']:
        paths={Path(p) for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']==name
               for p in m['filenames'] if p.endswith('.rlib')}
        assert len(paths)==1,(name,paths);path=paths.pop();assert path.exists();libraries[name]=path
    write(out/'linked-libraries.json',{name:{'path':str(p),'sha256':sha(p)} for name,p in libraries.items()})
    exe=runtime/'client.exe'
    command=[str(TOOLS/'rustc.exe'),'--edition=2021','--crate-name','native_environment_probe',str(rust_source),
             '--target','x86_64-pc-windows-msvc','-L','dependency='+str(libraries['lito_modern'].parent),
             '-L','dependency='+str(PROJECT/'target/current'/args.profile.lower()/'deps'),
             '--extern','lito_modern='+str(libraries['lito_modern']),'--extern','serde_json='+str(libraries['serde_json']),
             '-C','debuginfo=0','-C','opt-level='+('z' if args.profile=='Release' else '0'),'-o',str(exe)]
    if args.profile=='Release':command.extend(['-C','lto=thin','-C','codegen-units=16','-C','strip=symbols'])
    native_paths=sorted({p for m in messages if m.get('reason')=='build-script-executed' for p in m.get('linked_paths',[])})
    for path in native_paths:command.extend(['-L',path])
    child=launch('rustc',command,runtime);finish('rustc',child,180 if args.profile=='Release' else 60)
    actual=out/'actual-client.exe';actual.write_bytes(exe.read_bytes());b=actual.read_bytes();pe=struct.unpack_from('<I',b,60)[0]
    assert struct.unpack_from('<H',b,pe+4)[0]==0x8664 and struct.unpack_from('<H',b,pe+24)[0]==0x20b
    state['actual_probe_sha256']=sha(actual);persist()
    classes=runtime/'classes';classes.mkdir(exist_ok=True)
    child=launch('javac',[str(JDK/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(jar),'-d',str(classes),str(java_source)],runtime)
    finish('javac',child,40)
    for p in classes.rglob('*.class'):
        target=out/'actual-harness-classes'/p.relative_to(classes);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(p.read_bytes())
    server_root=out/'formal-server';server_root.mkdir()
    server=launch('server',[str(JDK/'java.exe'),'-Dfile.encoding=UTF-8','-Xverify:all','-Dlito.resourceProfile=tw13081901-v1',
        '-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',str(classes)+os.pathsep+str(jar),
        'local.lito381.server.NativeEnvironmentHarness',str(server_root)],server_root)
    until=time.monotonic()+15
    while not (server_root/'ready.json').exists():
        assert server.poll() is None,'Server terminated before readiness';assert time.monotonic()<until;time.sleep(.03)
    ready=read(server_root/'ready.json');assert ready['jar_sha256'].upper()==sha(jar)
    client_root=out/'client';client_root.mkdir()
    client=launch('client',[str(actual),'127.0.0.1:'+str(ready['port']),str(client_root),str(ROOT)],PROJECT)
    finish('client',client,90);finish('server',server,20)
    result=read(server_root/'result.json');client_result=read(client_root/'client.json')
    assert result['actual_client_sessions']==4 and result['LOGIN_sessions']==result['GAME_sessions']==2
    assert result['memberships_empty'] and not result['authority_mutations'] and not result['sql_access'] and not result['state_persistent']
    assert client_result['quiescent'] and not client_result['calendar_locally_advanced'] and not client_result['synthetic_packet_injection']
    assert len(client_result['rounds'])==2 and client_result['rounds'][0]['epoch']<client_result['rounds'][1]['epoch']
    for v in client_result['rounds']:
        assert v['before_admission']['calendar'] is None and v['mapped']['epoch']==v['epoch']
        assert v['mapped']['map']==500 and v['mapped']['map_mask_argument']==15
        assert v['mapped']['status_time_packets']>=1 and v['mapped']['time_packets']>=2
        assert v['after_cancel']['epoch'] is None and v['after_cancel']['calendar'] is None and v['after_cancel']['map'] is None
    groups=[]
    for capture in (server_root/'captures').rglob('frames.jsonl'):
        for line in capture.read_text('utf-8-sig').splitlines():
            frame=json.loads(line)
            if frame['direction']=='server_to_client' and frame.get('note')=='S_KEY':groups.append([])
            assert groups;groups[-1].append(frame)
    assert len(groups)==4
    wire_rounds=[]
    for index,group in enumerate(groups):
        timer=[f for f in group if f['direction']=='client_to_server' and f['opcode'] in ['0x20','0x1D']]
        if index in [0,2]:assert not timer;continue
        round=(index-1)//2;v=client_result['rounds'][round];times=[]
        for f in group:
            if f['direction']!='server_to_client' or f['opcode'] not in ['0x0C','0x21']:continue
            body=bytes.fromhex(f['plain'])
            if f['opcode']=='0x0C':assert len(body)==37 and f['wire_length']==39;raw=int.from_bytes(body[25:29],'little');assert raw==0
            else:
                assert len(body)==5 and f['wire_length']==7;raw=int.from_bytes(body[1:],'little')
                millis=int(datetime.fromisoformat(f['time'].replace('Z','+00:00')).timestamp()*1000)
                expected=(millis-1735689600000)*6//1000;expected-=expected%3
                assert raw%3==0 and abs(raw-expected)<=3
            times.append({'opcode':f['opcode'],'raw':raw,'time':f['time']})
        assert times[0]['opcode']=='0x21' and times[-1]['opcode']=='0x0C'
        assert times[0]['raw']==v['after_admission']['calendar']['raw']
        assert times[-1]['raw']==v['mapped']['calendar']['raw']==0
        assert sum(t['opcode']=='0x21' for t in times)==v['mapped']['time_packets']
        assert sum(t['opcode']=='0x0C' for t in times)==v['mapped']['status_time_packets']
        for f in timer:assert bytes.fromhex(f['plain'])==bytes([int(f['opcode'],16)]) and f['wire_length']==3
        wire_rounds.append({'round':round,'epoch':v['epoch'],'time_updates_in_wire_order':times,'timer_requests':len(timer)})
    write(out/'wire-time-order.json',wire_rounds)
    stable();state.update(status='actual_original_C02_GAME_environment_world_time_and_retirement_verified',actual_formal_sessions=4,
        LOGIN_sessions=2,GAME_sessions=2,resource_profile='tw13081901-v1',extended_item_details=True,
        jar_sha256=sha(jar),catalog_sha256=sha(catalog),calendar_locally_predicted=False,synthetic_packet_injection=False,
        actual_blind_weather_live_scenarios_executed=False,GUI_verified=False,full_current_TCP_verified=False)
except BaseException as error:
    state.update(status='failed_retained',error=f'{type(error).__name__}: {error}');raise
finally:
    for name,p in children:
        if p.poll() is None:
            p.terminate()
            try:p.wait(timeout=10)
            except subprocess.TimeoutExpired:p.kill();p.wait(timeout=10)
            state['owned_processes'][name].update(status='terminated_by_retained_owner_handle',exit_code=p.returncode)
    for handle in handles:handle.close()
    state['all_owned_children_terminal']=all(p.poll() is not None for _,p in children);persist()
    files={p.relative_to(out).as_posix():p for p in out.rglob('*') if p.is_file() and p.suffix!='.zip'}
    file_pins={name:sha(p) for name,p in files.items()};packed=out/'raw-evidence.zip'
    with zipfile.ZipFile(packed,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(packed) as z:
        for name,h in file_pins.items():
            with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,name
    retention={'path':str(packed),'sha256':sha(packed),'files':file_pins,'readback_verified':True};write(out/'raw-retention.json',retention)
if state['status'].startswith('actual_original'):
    write(out/'qualified.json',{**state,'raw':retention,'scoped_evidence':str(run/'independent-qualified.json'),
        'controller_probe_is_normal_GUI_executable':False,'world_framebuffer_verified':False,'whole_goal_complete':False})
print(json.dumps({k:state[k] for k in ['status','output','profile','actual_formal_sessions','all_owned_children_terminal']},ensure_ascii=False))
