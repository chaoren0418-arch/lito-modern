"""Full-graph isolated metadata compilation. Never execute a test, game or GUI."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,os,re,subprocess,tomllib
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
save=lambda p,v:Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
source=w['current_product_identity']['source_manifest_sha256'];assert source=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
registry=w['current_isolated_development']['candidates'];assert set(registry)=={'keyboard','blind20','cancel-recovery'}
normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run']);inputs=read(normal/'compiled-inputs.json')
assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
overlay={};pins={}
for key,entry in registry.items():
    cp=Path(entry['path']);c=read(cp);assert sha(cp)==entry['sha256'] and c['base_source_manifest_sha256']==source
    pins[str(cp)]=sha(cp)
    for n,h in c['base_files'].items():assert sha(R/n)==h
    for n in c.get('new_files',[]):assert not(R/n).exists()
    for n,h in c['candidate_files'].items():
        assert n not in overlay;file=cp.parent/n;assert sha(file)==h;overlay[n]=file;pins[str(file)]=h
assert len(overlay)==sum(entry['files'] for entry in registry.values())
out=V/'native-development-typecheck'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'scratch').mkdir();(out/'generated').mkdir();(out/'runner.py').write_bytes(Path(__file__).read_bytes());save(out/'base-compiled-inputs.json',inputs)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t) for t in ts])
    ticks=ts[0].dwLowDateTime|ts[0].dwHighDateTime<<32
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
state=dict(status='preparing_isolated_metadata_compilation',source_manifest_sha256=source,
    candidates=registry,pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),
    process_history=[],owned_process=None,profiles={},input_pins=pins,source_rewrites=[],
    normal_source_modified=False,normal_products_modified=False,candidate_applied=False,
    tests_executed=0,game_or_fixture_executions=0,GUI_inputs=0,whole_goal_complete=False)
save(out/'cohort.json',state);print(json.dumps(dict(output=str(out),mode='compile_metadata_only_no_execution'),ensure_ascii=False),flush=True)
normal_products={profile:P/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe' for profile in ['Debug','Release']}
state['normal_product_pins']={str(p):sha(p) for p in normal_products.values()}
def guard():
    assert all(sha(R/n)==h for n,h in inputs.items())
    assert all(sha(p)==h for p,h in pins.items())
    assert all(sha(p)==h for p,h in state['normal_product_pins'].items())
def selected(path):
    name=path.relative_to(R).as_posix();return overlay.get(name,path)
def raw_path(path):return 'r#"'+str(path)+'"#'
generated={}
try:
    # Only module/include path strings are rewritten for isolation. Original
    # fixtures, cfgs, assertions and full function bodies are not changed.
    for name,file in overlay.items():
        text=file.read_text('utf-8-sig');origin=R/name;rewrites=[]
        def path_attr(match):
            original=match.group(0);base=(origin.parent/match.group(1)).resolve(strict=True)
            path=selected(base);replacement='#[path = '+raw_path(path)+']';rewrites.append(dict(before=original,after=replacement))
            return replacement
        text=re.sub(r'#\[path\s*=\s*"([^"\n]+)"\]',path_attr,text)
        def include_literal(match):
            base=(origin.parent/match.group(2)).resolve(strict=True)
            replacement=match.group(1)+'!('+raw_path(base)+')';rewrites.append(dict(before=match.group(0),after=replacement));return replacement
        text=re.sub(r'(include_bytes|include_str|include)!\("([^"\n]+)"\)',include_literal,text)
        target=out/'generated'/name;assert not target.exists();target.parent.mkdir(parents=True,exist_ok=True)
        target.write_text(text,encoding='utf-8',newline='\n')
        generated[name]=target;pins[str(target)]=sha(target)
        state['source_rewrites'].append(dict(origin=str(file),generated=str(target),rewrites=rewrites))
    # A copied candidate controller must see its copied test module, not the
    # pristine file; update only the earlier generated absolute path attributes.
    for name,target in generated.items():
        text=target.read_text('utf-8');changes=[]
        for other,original_candidate in overlay.items():
            before='#[path = '+raw_path(original_candidate)+']';after='#[path = '+raw_path(generated[other])+']'
            if before in text:text=text.replace(before,after);changes.append(dict(before=before,after=after))
        target.write_text(text,encoding='utf-8',newline='\n');pins[str(target)]=sha(target)
        if changes:state['source_rewrites'].append(dict(generated=str(target),generated_path_redirects=changes))
    root=P/'src/modern_lito/mod.rs';lines=[];modules=[]
    for line in root.read_text('utf-8-sig').splitlines():
        match=re.fullmatch(r'\s*(?:pub(?:\(crate\))?\s+)?mod\s+(\w+);\s*',line)
        if not match:lines.append(line);continue
        name=match.group(1);path=root.parent/(name+'.rs')
        if lines and re.fullmatch(r'#\[path\s*=\s*"([^"\n]+)"\]',lines[-1]):
            path=root.parent/re.fullmatch(r'#\[path\s*=\s*"([^"\n]+)"\]',lines.pop()).group(1)
        assert path.is_file(),path
        relative=path.relative_to(R).as_posix();path=generated.get(relative,path)
        lines.extend(['#[path = '+raw_path(path)+']',line]);modules.append(dict(module=name,path=str(path)))
    root_copy=out/'generated/modern_lito.rs';root_copy.write_text('\n'.join(lines)+'\n',encoding='utf-8');pins[str(root_copy)]=sha(root_copy)
    lib=P/'src/lib.rs';lib_text=lib.read_text('utf-8-sig');assert lib_text.count('pub mod modern_lito;')==1
    lib_text=lib_text.replace('pub mod modern_lito;','#[path = '+raw_path(root_copy)+']\npub mod modern_lito;')
    old='#[path = "shared_v1/morph_auth.rs"]';assert lib_text.count(old)==1
    lib_text=lib_text.replace(old,'#[path = '+raw_path(P/'src/shared_v1/morph_auth.rs')+']')
    wrapper=out/'generated/lib.rs';wrapper.write_text(lib_text,encoding='utf-8');pins[str(wrapper)]=sha(wrapper)
    save(out/'module-map.json',modules);state['module_map']=str(out/'module-map.json');state['module_map_sha256']=sha(out/'module-map.json')
    state['input_pins']=pins;save(out/'cohort.json',state)
    package=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))
    rustc=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';pins[str(rustc)]=sha(rustc)
    env={n:v for n,v in os.environ.items() if not n.startswith('LITO_')};env.update(TEMP=str(out/'scratch'),TMP=str(out/'scratch'),CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=package['package']['version'])
    def execute(command,folder):
        assert Path(command[0])==rustc and '--emit=metadata' in command
        assert command[command.index('-o')+1].endswith('.rmeta')
        guard();folder.mkdir(parents=True)
        with(folder/'stdout.log').open('wb') as stdout,(folder/'stderr.log').open('wb') as stderr:
            child=subprocess.Popen(command,cwd=folder,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,output=str(folder),exit_code=None)
            state['process_history'].append(row);state['owned_process']=row;save(out/'cohort.json',state)
            try:child.wait(timeout=600)
            except BaseException:
                if child.poll() is None:
                    assert birth(wintypes.HANDLE(int(child._handle)))==row['creation_utc'];child.kill();child.wait();row['exact_owned_cleanup']=True
                raise
            finally:
                row.update(exit_code=child.returncode,finished_utc=datetime.now(timezone.utc).isoformat());state['owned_process']=None;save(out/'cohort.json',state)
        guard();assert child.returncode==0,(str(folder),child.returncode)
    for profile in ['Debug','Release']:
        messages=[json.loads(l) for l in(normal/profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines() if l.startswith('{')]
        deps=P/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'deps';extern={}
        for dep in package['dependencies']:
            name=dep.replace('-','_');paths={str(Path(f).resolve(strict=True)) for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']==name for f in m['filenames'] if f.endswith('.rlib') and Path(f).parent==deps}
            assert len(paths)==1,(profile,dep,paths);extern[name]=paths.pop()
        for p in extern.values():pins[p]=sha(p)
        work=out/profile;work.mkdir();state['profiles'][profile]=dict(extern={n:dict(path=p,sha256=sha(p)) for n,p in extern.items()},checks={})
        base=[str(rustc),'--edition=2021','--target','x86_64-pc-windows-msvc','--emit=metadata','-L','dependency='+str(deps),'-L','dependency='+str(P/'target/current'/profile.lower()/'deps')]
        base+=['-C','opt-level=0','-C','debuginfo=2'] if profile=='Debug' else ['-C','opt-level=z','-C','lto=thin','-C','codegen-units=16','-C','strip=symbols']
        for name,p in extern.items():base+=['--extern',name+'='+p]
        tasks=[('library',wrapper,['--crate-type=rlib','--crate-name','lito_modern']),
               ('library-test-code',wrapper,['--test','--crate-name','lito_modern']),
               ('normal-entry-code',P/'src/main.rs',['--crate-type=bin','--crate-name','LiToModern']),
               ('keyboard-test-code',generated['LiTo381-Modern/tests/native_skill_selection.rs'],['--test','--crate-name','native_skill_selection']),
               ('blind20-test-code',generated['LiTo381-Modern/tests/native_blind.rs'],['--test','--crate-name','native_blind'])]
        formal_test=generated.get('LiTo381-Modern/tests/native_teleport_failure.rs')
        if formal_test is not None:
            tasks.append(('teleport-recovery-formal-code',formal_test,['--test','--crate-name','native_teleport_failure']))
        library_metadata=work/'library.rmeta'
        for label,code,flags in tasks:
            output=work/(label+'.rmeta');command=[*base,str(code),*flags,'-o',str(output)]
            if label not in ['library','library-test-code']:command+=['--extern','lito_modern='+str(library_metadata)]
            state.update(status='compiling_metadata_only',active_profile=profile,active_check=label);save(out/'cohort.json',state)
            execute(command,work/(label+'-compile'));assert output.is_file()
            state['profiles'][profile]['checks'][label]=dict(metadata=str(output),metadata_sha256=sha(output),compiler_exit_code=0,test_execution=False)
            save(out/'cohort.json',state);print(json.dumps(dict(profile=profile,check=label,compiler_exit_code=0,test_execution=False)),flush=True)
    state.update(status='metadata_compilation_both_profiles_complete_no_tests_executed')
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:
    state.update(input_pins=pins,finished_utc=datetime.now(timezone.utc).isoformat());guard();save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status'],tests_executed=0,normal_products_modified=False),ensure_ascii=False),flush=True)
