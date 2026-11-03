"""Unique normal builds and the three directly affected native request targets."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,time,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern';RUST=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def retain(path,files):
    hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(path) as z:
        for n,h in hashes.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    return dict(path=str(path),sha256=sha(path),files=hashes)
app=Path(read(ROOT/'驗證紀錄/current/native77-request-integration/active-application.json')['path']);a=read(app/'application.json')
assert a['status']=='applied_native77_request_pending_normal_and_live_validation' and a['applied']
inputs=read(app/'applied-compiled-inputs.json');source=read(app/'applied-source-inputs.json');assert len(inputs)==1031 and len(source)==1030
assert sha(app/'applied-source-inputs.json')==a['source_manifest_sha256']
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'})}
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release'];assert tuple(release[k] for k in ['opt-level','lto','strip','codegen-units'])==('z','thin','symbols',16)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native77-request-integration'/('validation-'+stamp);out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',inputs);save(out/'source-inputs.json',source)
original=ROOT/'驗證紀錄/current/native77-scene-integration/validation-20261017-183727-278843';prior=read(original/'cohort.json')
state=dict(status='prepared_native77_request_normals',output=str(out),application=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=a['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,
    release_configuration=release,normal_entry=str(ROOT/'啟動現代客戶端.ps1'),entry_sha256=sha(ROOT/'啟動現代客戶端.ps1'),profiles={},owned_process=None,process_history=[],observed_compilers=[],
    previous_C831_archive_qualification=a['previous_normal_qualification'],qualification_scope='normal builds and directly affected native request/selection/attack tests only',actual_server_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
state['compiled_source_archive']=retain(out/'compiled-source.zip',{n:ROOT/n for n in inputs})
kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;kernel.GetCurrentProcess.restype=wintypes.HANDLE
def created(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert kernel.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'original-runner-process.json',dict(pid=os.getpid(),parent_pid=os.getppid(),creation_utc=created(kernel.GetCurrentProcess())))
env=dict(os.environ,RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'));env['PATH']=str(RUST)+os.pathsep+env['PATH']
def check():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    assert sha(ROOT/'啟動現代客戶端.ps1')==state['entry_sha256']
def observe(top):
    ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Select-Object ProcessId,ParentProcessId,Name,CommandLine,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
    rows=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',ps]).decode('utf-8-sig'));parents={top}
    for _ in range(8):parents|={v['ProcessId'] for v in rows if v['ParentProcessId'] in parents}
    for row in rows:
        if row['ProcessId'] in parents and row['Name']=='rustc.exe' and not any((v['ProcessId'],v['created'])==(row['ProcessId'],row['created']) for v in state['observed_compilers']):state['observed_compilers'].append(row)
def execute(command,label):
    check();d=out/label;d.mkdir(parents=True)
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        child=subprocess.Popen(command,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);job=dict(pid=child.pid,creation_utc=created(wintypes.HANDLE(int(child._handle))),command=command,label=label)
        state['owned_process']=job;state['process_history'].append(job);save(out/'cohort.json',state);last=time.monotonic()
        while child.poll() is None:
            if state['active_profile']=='Release' and label.endswith(('normal-build','test-build')) and time.monotonic()-last>=20:observe(child.pid);save(out/'cohort.json',state);last=time.monotonic()
            time.sleep(.25)
        job['exit_code']=child.wait();state['owned_process']=None;save(out/'cohort.json',state)
    check();assert job['exit_code']==0,label+' failed; preserve before repair'
    return d
def inventory(label):return set(re.findall(r'^(.+): test$',(out/label/'stdout.log').read_text(encoding='utf-8-sig'),re.M))
def counts(d):
    row=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',(d/'stdout.log').read_text(encoding='utf-8-sig'));assert len(row)==1 and row[0][0]=='ok'
    return dict(zip(['passed','failed','ignored','measured','filtered_out'],map(int,row[0][1:])))
targets=['native_skills','native_skill_selection','native_attack_skills'];new_names={'disintegrate_request_contract::'+n for n in ['current_c02_exact_object_coordinates_and_readonly_authority','range_ten_boundary_and_target_changes_use_current_received_coordinates','invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests','mp_and_lawful_are_server_permission_inputs_and_never_locally_spent','selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection']}
print(json.dumps(dict(output=str(out),source_manifest_sha256=state['source_manifest_sha256']),ensure_ascii=False),flush=True);save(out/'cohort.json',state)
try:
    for profile in ['Debug','Release']:
        state.update(status='building_native77_request_normal',active_profile=profile);info={};state['profiles'][profile]=info;save(out/'cohort.json',state)
        execute(['pwsh','-NoProfile','-File',state['normal_entry'],'-BuildOnly','-BuildProfile',profile],profile+'/normal-build');exe=PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe'
        info['normal']=dict(path=str(exe),sha256=sha(exe),archive=retain(out/(profile+'-normal.zip'),{'LiToModern.exe':exe}),unique_entry_exit_code=0);execute([str(exe),'--version'],profile+'/version');info['normal']['version_exit_code']=0;save(out/'cohort.json',state)
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json']
        for target in targets:command+=['--test',target]
        if profile=='Release':command+=['--release']
        d=execute(command,profile+'/test-build');rows=[json.loads(x) for x in (d/'stdout.log').read_text(encoding='utf-8-sig').splitlines() if x.startswith('{')];products={v['target']['name']:v for v in rows if v.get('reason')=='compiler-artifact' and v.get('executable') and v['profile']['test'] and v['target']['name'] in targets};assert set(products)==set(targets)
        info['targets']={}
        for target in targets:
            image=Path(products[target]['executable']);item=dict(path=str(image),sha256=sha(image),archive=retain(out/(profile+'-'+target+'.zip'),{target+'.exe':image}));info['targets'][target]=item
            execute([str(image),'--list','--format','terse'],profile+'/'+target+'/inventory');execute([str(image),'--list','--ignored','--format','terse'],profile+'/'+target+'/ignored-inventory');names=inventory(profile+'/'+target+'/inventory');ignored=inventory(profile+'/'+target+'/ignored-inventory');assert names and not ignored
            if target in prior['profiles'][profile]['integration_targets']:
                expected=set(prior['profiles'][profile]['integration_targets'][target]['registered_tests'])|(new_names if target=='native_attack_skills' else set());assert names==expected
            item['test_names']=sorted(names);item['counts']=counts(execute([str(image),'--test-threads=1'],profile+'/'+target+'/tests'));assert item['counts']==dict(passed=len(names),failed=0,ignored=0,measured=0,filtered_out=0);assert sha(image)==item['sha256'];save(out/'cohort.json',state)
        print(json.dumps(dict(profile=profile,normal_sha256=info['normal']['sha256'],passed=sum(v['counts']['passed'] for v in info['targets'].values())),ensure_ascii=False),flush=True)
    state.update(status='executed_native77_request_normals_related',active_profile=None)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
