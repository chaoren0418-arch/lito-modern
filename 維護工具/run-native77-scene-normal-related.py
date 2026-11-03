"""Unique normal builds and actual native77 scene-related library selections."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,time,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUST=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def retain(path,files):
    hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(path) as z:
        assert set(z.namelist())==set(hashes)
        for n,h in hashes.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    return dict(path=str(path),sha256=sha(path),files=hashes)
application=Path(read(ROOT/'驗證紀錄/current/native77-scene-integration/active-application.json')['path']).resolve(strict=True)
a=read(application/'application.json');assert a['status']=='applied_native77_scene_pending_normal_validation' and a['applied']
pins=read(application/'applied-source-inputs.json');inputs=read(application/'applied-compiled-inputs.json');assert len(pins)==1030 and len(inputs)==1031
assert sha(application/'applied-source-inputs.json')==a['source_manifest_sha256']
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'})}
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert (release['opt-level'],release['lto'],release['codegen-units'],release['strip'])==('z','thin',16,'symbols')
out=ROOT/'驗證紀錄/current/native77-scene-integration'/('validation-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'source-inputs.json',pins);save(out/'compiled-inputs.json',inputs)
source=retain(out/'compiled-source.zip',{n:ROOT/n for n in inputs})
new_test='modern_lito::sprite::tests::reviewed544_marker_preserves_slots_and_rejects_corruption'
new_asset_test='modern_lito::disintegrate_assets::tests::current_tw_namespace_requires_exact_profile_and_cancels_before_io'
new_probe_names=set(re.findall(r'fn ([A-Za-z0-9_]+)\(', (PROJECT/'src/modern_lito/native_disintegrate_tests.rs').read_text(encoding='utf-8')))
new_probe_names={n for n in new_probe_names if n.startswith(('actual_','capture_','cancellation_','canonical_'))}
assert len(new_probe_names)==14
new_tests={new_asset_test}|{'modern_lito::native_disintegrate_tests::native77_probe::'+n for n in new_probe_names}
new_ignored={n for n in new_tests if '::actual_' in n};assert len(new_ignored)==10
original=ROOT/'驗證紀錄/current/native-startup-library-default/20261017-150542-781788'
state=dict(status='prepared_current_native77_scene_normal_validation',output=str(out),application=str(application),application_sha256=sha(application/'application.json'),
    source_manifest_sha256=a['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,compiled_source_archive=source,release_configuration=release,
    normal_entry=str(ROOT/'啟動現代客戶端.ps1'),entry_sha256=sha(ROOT/'啟動現代客戶端.ps1'),original_inventory_cohort=str(original),
    reference_jar_sha256=a['reference_jar_sha256'],profiles={},owned_process=None,process_history=[],observed_compilers=[],actual_formal_sessions=0,new_casts=0,
    full_library_bodies_executed=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
kernel=ctypes.WinDLL('kernel32',use_last_error=True)
kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;kernel.GetProcessTimes.restype=wintypes.BOOL
def created(handle):
    times=[wintypes.FILETIME() for _ in range(4)];assert kernel.GetProcessTimes(handle,*[ctypes.byref(t) for t in times])
    ticks=times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
    return (datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
kernel.GetCurrentProcess.restype=wintypes.HANDLE
save(out/'original-runner-process.json',dict(pid=os.getpid(),parent_pid=os.getppid(),creation_utc=created(kernel.GetCurrentProcess()),arguments=__import__('sys').argv))
env=dict(os.environ,RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
env['PATH']=str(RUST)+os.pathsep+env['PATH']
def check():
    for n,h in inputs.items():assert sha(ROOT/n)==h,n
    assert sha(ROOT/'啟動現代客戶端.ps1')==state['entry_sha256']
def observe(top):
    ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Select-Object ProcessId,ParentProcessId,Name,CommandLine,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
    raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);rows=json.loads(raw.decode('utf-8-sig'));parents={top};changed=True
    while changed:
        more={x['ProcessId'] for x in rows if x['ParentProcessId'] in parents};changed=not more<=parents;parents|=more
    for row in rows:
        if row['ProcessId'] in parents and row['Name']=='rustc.exe' and not any((x['ProcessId'],x['created'])==(row['ProcessId'],row['created']) for x in state['observed_compilers']):state['observed_compilers'].append(row)
def execute(command,label,child_env=None):
    check();d=out/label;d.mkdir(parents=True)
    with (d/'stdout.log').open('wb') as stdout,(d/'stderr.log').open('wb') as stderr:
        p=subprocess.Popen(command,cwd=PROJECT,env=child_env or env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        job=dict(pid=p.pid,creation_utc=created(wintypes.HANDLE(int(p._handle))),command=command,label=label)
        state['owned_process']=job;state['process_history'].append(job);save(out/'cohort.json',state);last=time.monotonic()
        while p.poll() is None:
            if state.get('active_profile')=='Release' and label.endswith(('normal-build','lib-build')) and time.monotonic()-last>=20:observe(p.pid);save(out/'cohort.json',state);last=time.monotonic()
            time.sleep(.25)
        job['exit_code']=p.wait();state['owned_process']=None;save(out/'cohort.json',state)
    check();assert job['exit_code']==0,label+' failed; preserve before repair'
    return d
def names(label):return set(re.findall(r'^(.+): test$',(out/label/'stdout.log').read_text(encoding='utf-8-sig'),re.M))
def counts(directory):
    text=(directory/'stdout.log').read_text(encoding='utf-8-sig');row=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text);assert len(row)==1 and row[0][0]=='ok'
    return dict(zip(['passed','failed','ignored','measured','filtered_out'],map(int,row[0][1:])))
resource_reference=read(ROOT/'驗證紀錄/current/native77-current-assets/20261017-175445-506890/receipt.json')
for name,h in resource_reference['original_input_pins'].items():assert sha(name)==h
for name,m in resource_reference['original_PAK_metadata'].items():assert (Path(name).stat().st_size,Path(name).stat().st_mtime_ns)==(m['bytes'],m['mtime_ns'])
save(out/'resource-inputs-before-tests.json',dict(observed_utc=datetime.now(timezone.utc).isoformat(),before_actual_asset_jobs=True,original_input_pins=resource_reference['original_input_pins'],original_PAK_metadata=resource_reference['original_PAK_metadata'],observed_input_mtimes={n:Path(n).stat().st_mtime_ns for n in resource_reference['original_input_pins']},pipeline_history_at_pin=[]))
print(json.dumps(dict(output=str(out),source_manifest_sha256=state['source_manifest_sha256']),ensure_ascii=False),flush=True);save(out/'cohort.json',state)
try:
    for profile in ['Debug','Release']:
        state.update(active_profile=profile,status='building_unique_current_normal');info={};state['profiles'][profile]=info;save(out/'cohort.json',state)
        execute(['pwsh.exe','-NoProfile','-File',state['normal_entry'],'-BuildOnly','-BuildProfile',profile],profile+'/normal-build')
        exe=PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe'
        info['normal']=dict(path=str(exe),sha256=sha(exe),unique_entry_exit_code=0,archive=retain(out/(profile+'-normal.zip'),{'LiToModern.exe':exe}));save(out/'cohort.json',state)
        execute([str(exe),'--version'],profile+'/version');info['normal']['version_exit_code']=0
        state['status']='compiling_current_library_native77_test_image';save(out/'cohort.json',state)
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json','--lib']
        if profile=='Release':command.append('--release')
        d=execute(command,profile+'/lib-build');messages=[json.loads(x) for x in (d/'stdout.log').read_text(encoding='utf-8-sig').splitlines() if x.startswith('{')]
        products=[x for x in messages if x.get('reason')=='compiler-artifact' and x.get('executable') and x['profile']['test']];assert len(products)==1 and products[0]['target']['name']=='lito_modern' and products[0]['features']==[]
        test=Path(products[0]['executable']);info['test']=dict(path=str(test),sha256=sha(test),archive=retain(out/(profile+'-test.zip'),{'lito_modern.exe':test}))
        execute([str(test),'--list','--format','terse'],profile+'/inventory');execute([str(test),'--list','--ignored','--format','terse'],profile+'/ignored-inventory')
        old=read(original/profile/'inventory-audit/inventory.json');assert old['total_tests']==2410 and old['ignored_tests']==473
        actual=names(profile+'/inventory');ignored=names(profile+'/ignored-inventory');assert actual==set(old['all_test_names'])|{new_test}|new_tests and ignored==set(old['ignored_test_names'])|new_ignored
        info['registered_total']=len(actual);info['unchanged_registered_ignored']=len(ignored)
        state['status']='executing_current_source_native77_related_tests';save(out/'cohort.json',state)
        info['selections']={}
        for prefix in ['modern_lito::native_disintegrate_tests::','modern_lito::disintegrate_assets::tests::','modern_lito::native_world::tests::','modern_lito::native_audio_delivery::','modern_lito::native_combat_animation::','modern_lito::native_effect_timing::','modern_lito::world_raster::','modern_lito::compositor::tests::','modern_lito::native_world_animation::','modern_lito::sprite::tests::']:
            expected={n for n in actual if prefix in n}
            if not expected:
                info.setdefault('absent_library_test_modules',[]).append(prefix);continue
            label=prefix.replace('modern_lito::','').replace('::','-').strip('-')
            d=execute([str(test),prefix,'--test-threads=1'],profile+'/'+label);result=counts(d)
            assert result['passed']==len(expected-ignored) and result['ignored']==len(expected&ignored) and result['failed']==0 and result['filtered_out']==len(actual)-len(expected)
            info['selections'][prefix]=dict(counts=result,test_names=sorted(expected));save(out/'cohort.json',state)
        prefix='modern_lito::native_disintegrate_tests::native77_probe::actual_'
        result=counts(execute([str(test),prefix,'--ignored','--test-threads=1'],profile+'/native77-actual-assets'))
        assert result==dict(passed=10,failed=0,ignored=0,measured=0,filtered_out=len(actual)-10)
        info['actual_asset_lifecycle']=result
        targets=['native_world','native_combat','native_attack_skills','native_animation','native_effect_lifecycle','native_world_effects','native_world_layers','native_lifecycle','native_skill_selection','native_chill_touch','native_selector_only','native_world_lighting_scene']
        command=[str(RUST/'cargo.exe'),'test','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json']
        for target in targets:command+=['--test',target]
        if profile=='Release':command.append('--release')
        d=execute(command,profile+'/integration-build');messages=[json.loads(x) for x in (d/'stdout.log').read_text(encoding='utf-8-sig').splitlines() if x.startswith('{')]
        products={x['target']['name']:x for x in messages if x.get('reason')=='compiler-artifact' and x.get('executable') and x['profile']['test'] and x['target']['name'] in targets};assert set(products)==set(targets)
        info['integration_targets']={}
        for target in targets:
            image=Path(products[target]['executable']);item=dict(path=str(image),sha256=sha(image));info['integration_targets'][target]=item
            item['archive']=retain(out/(profile+'-'+target+'.zip'),{target+'.exe':image})
            execute([str(image),'--list','--format','terse'],profile+'/'+target+'/inventory');execute([str(image),'--list','--ignored','--format','terse'],profile+'/'+target+'/ignored-inventory')
            all_names=names(profile+'/'+target+'/inventory');ignore_names=names(profile+'/'+target+'/ignored-inventory');assert all_names and ignore_names<=all_names
            item['registered_tests']=sorted(all_names);item['registered_ignored']=sorted(ignore_names)
            result=counts(execute([str(image),'--test-threads=1'],profile+'/'+target+'/tests'));assert result==dict(passed=len(all_names-ignore_names),failed=0,ignored=len(ignore_names),measured=0,filtered_out=0)
            item['counts']=result;assert sha(image)==item['sha256'];save(out/'cohort.json',state)
        assert sha(exe)==info['normal']['sha256'] and sha(test)==info['test']['sha256'];save(out/'cohort.json',state)
        print(json.dumps(dict(profile=profile,normal_sha256=info['normal']['sha256'],related_passed=sum(v['counts']['passed'] for v in info['selections'].values()),actual_asset_lifecycle=info['actual_asset_lifecycle']),ensure_ascii=False),flush=True)
    state.update(status='executed_current_native77_normals_and_related_scopes',active_profile=None)
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
