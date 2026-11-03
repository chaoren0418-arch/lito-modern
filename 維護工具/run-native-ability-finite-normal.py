"""Unique normal builds, scene23, ability7 and one opted-in original resource case per profile."""
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
app=Path(read(ROOT/'驗證紀錄/current/native-ability-finite-normal/active-application.json')['path']);a=read(app/'application.json')
assert a['status']=='applied_ability_finite_pending_normal_validation' and a['applied']
inputs=read(app/'applied-compiled-inputs.json');source=read(app/'applied-source-inputs.json');assert len(inputs)==1048 and len(source)==1047
assert sha(app/'applied-source-inputs.json')==a['source_manifest_sha256']
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'})}
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release'];assert tuple(release[k] for k in ['opt-level','lto','strip','codegen-units'])==('z','thin','symbols',16)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-ability-finite-normal'/('validation-'+stamp);out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'compiled-inputs.json',inputs);(out/'source-inputs.json').write_bytes((app/'applied-source-inputs.json').read_bytes())
original=ROOT/'驗證紀錄/current/native-model-generation-normal/validation-20261018-160708-195207';prior=read(original/'cohort.json')
state=dict(status='prepared_ability_finite_normals',output=str(out),application=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=a['source_manifest_sha256'],source_files=1047,compiled_input_files=1048,
    release_configuration=release,normal_entry=str(ROOT/'啟動現代客戶端.ps1'),entry_sha256=sha(ROOT/'啟動現代客戶端.ps1'),profiles={},owned_process=None,process_history=[],observed_compilers=[],
    previous_source_manifest_sha256=a['before_source_manifest_sha256'],qualification_scope='unique-entry normal builds; scene23, ability7 default cases and one original-resource opt-in covering11graphics per profile; live executable compiled/ignored only; prior broad/library retained for44FE',actual_server_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
state['compiled_source_archive']=retain(out/'compiled-source.zip',{n:ROOT/n for n in inputs})
kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;kernel.GetCurrentProcess.restype=wintypes.HANDLE
def created(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert kernel.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
save(out/'original-runner-process.json',dict(pid=os.getpid(),parent_pid=os.getppid(),creation_utc=created(kernel.GetCurrentProcess())))
env=dict(os.environ,RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'));env['PATH']=str(RUST)+os.pathsep+env['PATH'];env['LITO_SHARED_RESOURCE_ROOT']=str(Path('D:/lineage開發/0-客戶端'))
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
targets=['native_world_lighting_scene','native_ability_buffs','native_ability_world','native_additional_finite_resources','native_ability_live']
expected_inventory=dict(zip(targets,[23,5,2,1,1]));expected_ignored=dict(zip(targets,[0,0,0,1,1]))
new_names=set()
room_prior={'Debug':ROOT/'驗證紀錄/current/default-native-regression/20261018-032357-157-Debug','Release':ROOT/'驗證紀錄/current/default-native-regression/20261018-033903-037-Release'}
unit_groups={'pending_join':'modern_lito::native_login::pending_join_departure_tests::','wire_spatial':'modern_lito::wire_spatial::tests::','catalog':'modern_lito::shared_action_catalog::tests::','light_arrow':'modern_lito::light_arrow_effect::tests::'}
protected={}
patterns=['native-layer-*.png','native-layer-reference.json','native-world-effects-*.png','native-world-effects-reference.json','native-bolt-*.png','native-bolt.json','native-skills-*.png','native-skills.json','native-TW11722-original-frame-metadata.json','native-shared-basic-heal-frames.json','native-shared-higher-heal-frames.json','native-legacy-higher-heal-original-definitions.json','native-legacy-higher-heal-frames.json']
verification=PROJECT/'target/current/verification'
def selected_outputs():return {p.name:p for pattern in patterns for p in verification.glob(pattern) if p.is_file()}
def protect(profile):
    d=out/profile/'protected-outputs';d.mkdir(parents=True);files=selected_outputs();pins={n:sha(p)for n,p in files.items()}
    with zipfile.ZipFile(d/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(d/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
    protected[profile]=(d,pins);save(d/'before.json',dict(files=pins,archive_sha256=sha(d/'before.zip')))
def restore(profile):
    if profile not in protected:return
    d,pins=protected.pop(profile);files=selected_outputs();generated={n:sha(p)for n,p in files.items()}
    with zipfile.ZipFile(d/'generated.zip','x',zipfile.ZIP_DEFLATED)as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(d/'generated.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in generated.items())
    with zipfile.ZipFile(d/'before.zip')as z:
        for n,h in pins.items():(verification/n).write_bytes(z.read(n));assert sha(verification/n)==h
    save(d/'restoration.json',dict(before_files=pins,generated_files=generated,generated_archive_sha256=sha(d/'generated.zip'),existing_files_restored=True))
print(json.dumps(dict(output=str(out),source_manifest_sha256=state['source_manifest_sha256']),ensure_ascii=False),flush=True);save(out/'cohort.json',state)
try:
    for profile in ['Debug','Release']:
        state.update(status='building_ability_finite_normal',active_profile=profile);info={};state['profiles'][profile]=info;save(out/'cohort.json',state)
        execute(['pwsh','-NoProfile','-File',state['normal_entry'],'-BuildOnly','-BuildProfile',profile],profile+'/normal-build');exe=PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe'
        info['normal']=dict(path=str(exe),sha256=sha(exe),archive=retain(out/(profile+'-normal.zip'),{'LiToModern.exe':exe}),unique_entry_exit_code=0);execute([str(exe),'--version'],profile+'/version');info['normal']['version_exit_code']=0;save(out/'cohort.json',state)
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json']
        for target in targets:command+=['--test',target]
        if profile=='Release':command+=['--release']
        d=execute(command,profile+'/test-build');rows=[json.loads(x) for x in (d/'stdout.log').read_text(encoding='utf-8-sig').splitlines() if x.startswith('{')];products={v['target']['name']:v for v in rows if v.get('reason')=='compiler-artifact' and v.get('executable') and v['profile']['test'] and v['target']['name'] in targets};assert set(products)==set(targets)
        info['targets']={};protect(profile)
        resource_work=PROJECT/'target/current/native44FE-extra-finite-effects'/('normal-'+out.name+'-'+profile.lower());resource_work.mkdir(parents=True)
        rq=Path(a['candidate_qualifications'][profile]['resources']).parent.parent
        for name in ['original-effect-definitions.json','candidate-graphics.json']:(resource_work/name).write_bytes((rq/name).read_bytes())
        env['LITO_EXTRA_FINITE_RENDER_OUT']=str(resource_work);info['resource_output']=str(resource_work)
        for target in targets:
            image=Path(products[target]['executable']);item=dict(path=str(image),sha256=sha(image),archive=retain(out/(profile+'-'+target+'.zip'),{target+'.exe':image}));info['targets'][target]=item
            execute([str(image),'--list','--format','terse'],profile+'/'+target+'/inventory');execute([str(image),'--list','--ignored','--format','terse'],profile+'/'+target+'/ignored-inventory');names=inventory(profile+'/'+target+'/inventory');ignored=inventory(profile+'/'+target+'/ignored-inventory');assert len(names)==expected_inventory[target] and len(ignored)==expected_ignored[target] and ignored<=names
            if target in prior['profiles'][profile]['targets']:
                expected=set(prior['profiles'][profile]['targets'][target]['test_names']);assert names==expected
            if target in ['native_room','native_room_departures','native_room_force']:
                old_inventory=room_prior[profile]/'cases'/target/'inventory/stdout.log';expected=set(re.findall(r'^(.+): test$',old_inventory.read_text('utf-8-sig'),re.M));assert names==expected
            if target=='native_cone_cold':assert len(names)==3
            if target=='native_animation':assert 'hurt_queued_after_motion::eight_directions_hurt_waits_for_walk_without_camera_rewind_and_old_life_replay' in names
            item['test_names']=sorted(names);item['counts']=counts(execute([str(image),'--test-threads=1'],profile+'/'+target+'/tests'));assert item['counts']==dict(passed=len(names)-len(ignored),failed=0,ignored=len(ignored),measured=0,filtered_out=0);assert sha(image)==item['sha256'];save(out/'cohort.json',state)
        resource_target=info['targets']['native_additional_finite_resources'];resource_target['opted_in_counts']=counts(execute([resource_target['path'],'--ignored','--test-threads=1','--nocapture'],profile+'/finite-original-resources'))
        assert resource_target['opted_in_counts']==dict(passed=1,failed=0,ignored=0,measured=0,filtered_out=0)
        resource_result=read(resource_work/'resource-result.json');assert resource_result['status']=='candidate_resource_render_and_source_event_progression_passed'and len(resource_result['cases'])==11
        info['resource_result_sha256']=sha(resource_work/'resource-result.json')
        restore(profile)
        print(json.dumps(dict(profile=profile,normal_sha256=info['normal']['sha256'],passed=sum(v['counts']['passed'] for v in info['targets'].values())),ensure_ascii=False),flush=True)
    state.update(status='executed_ability_finite_normals_related',active_profile=None)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    for protected_profile in list(protected):restore(protected_profile)
    save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
