"""Build a complete isolated effect-bank variant only after the original broad job is sealed."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,os,re,subprocess,time,tomllib
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],required=True);ap.add_argument('--with-ability-buffs',action='store_true');ap.add_argument('--check-only',action='store_true');a=ap.parse_args()
queue=read(ROOT/'工作佇列.json');broad=V/'native44FE-default112/20261018-174151-282798'
if a.check_only:
    from native44FE_broad_gate import require_idle_runtime
    require_idle_runtime(ROOT);completion=read(broad/'cohort.json')
else:
    assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen'],'original broad/library workload must finish before candidate machine-code build'
    completion=read(broad/'completion-observation.json');assert completion['observed_terminal_exit_code']==0 and completion['recorded_original_owned_processes_all_terminal']
normal=V/'native-model-generation-normal/validation-20261018-160708-195207';c=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json')
assert len(inputs)==1041 and c['source_manifest_sha256']==completion['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
normal_q=next(normal.glob(a.profile+'-qualified-*/qualified.json'));nq=read(normal_q)
assert nq['passed']==23 and nq['failed']==nq['ignored']==0 and nq['recorded_owned_all_terminal']
candidate=ROOT/'維護工具/candidates/native44FE-extra-skill-effects-integration';prep=read(candidate/'preparation.json');prep_sha=sha(candidate/'preparation.json')
assert not prep['applied_to_normal_product']and not prep['normal_source_modified']
assert prep['base_runtime_sha256']==sha(P/'src/modern_lito/native_skill_effects.rs')and prep['base_module_sha256']==sha(P/'src/modern_lito/mod.rs')
assert sha(candidate/'resource_probe.rs')==prep['resource_probe_sha256']
ability=ROOT/'維護工具/candidates/native44FE-ability-buff-requests';ability_prep=None;ability_sha=None
if a.with_ability_buffs:
    ability_prep=read(ability/'preparation.json');ability_sha=sha(ability/'preparation.json')
    assert ability_prep['typecheck_verified']and ability_prep['packet_guard_component_verified']and ability_prep['live_probe_normal_API_typecheck_verified']
    assert not ability_prep['candidate_applied']and not ability_prep['normal_source_modified']and not ability_prep['live_probe_executed']
    component=Path(ability_prep['component']);cp=read(component/'receipt.json')
    assert cp['status']=='isolated_ability_request_and_selection_component_passed'and cp['passed']==5 and cp['failed']==cp['ignored']==0
    assert sha(component/'readback.json')==ability_prep['component_readback_sha256']
def guard():
    for name,h in inputs.items():assert sha(ROOT/name)==h,name
    for info in c['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
    assert sha(candidate/'preparation.json')==prep_sha
    for name,h in prep['files'].items():assert sha(candidate/name)==h,name
    assert sha(candidate/'resource_probe.rs')==prep['resource_probe_sha256']
    if 'base_tower_sha256'in prep:assert sha(P/'src/modern_lito/tower_assets.rs')==prep['base_tower_sha256']
    if ability_prep:
        assert sha(ability/'preparation.json')==ability_sha
        for name,h in ability_prep['files'].items():assert sha(ability/name)==h,name
        for name,h in ability_prep['base_files'].items():assert sha(P/'src/modern_lito'/name)==h,name
        assert sha(ability/'live_probe.rs')==ability_prep['live_probe_sha256']
        for name,h in ability_prep['world_files'].items():assert sha(ability/name)==h,name
        assert sha(P/'src/modern_lito/native_world.rs')==ability_prep['base_world_sha256']
guard();assert not {k:v for k,v in os.environ.items()if v and(k.startswith('CARGO_PROFILE_')or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'])}
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')+'-'+a.profile.lower()+('-ability'if ability_prep else'')+('-metadata'if a.check_only else'');out=V/'native44FE-extra-finite-effects'/('compile-'+stamp);out.mkdir(parents=True)
work=P/'target/current/native44FE-extra-finite-effects'/stamp;work.mkdir(parents=True)
for name in ['mod.rs','native_skill_effects.rs','native_additional_finite_effects.rs','resource_probe.rs','preparation.json']:(out/name).write_bytes((candidate/name).read_bytes())
runtime_overrides=['native_skill_effects.rs','native_additional_finite_effects.rs']
if 'tower_assets.rs'in prep['files']:
    (out/'tower_assets.rs').write_bytes((candidate/'tower_assets.rs').read_bytes());runtime_overrides.append('tower_assets.rs')
if ability_prep:
    for name in ['native_skills.rs','native_skill_selection.rs','native_world.rs','native_ability_buffs.rs','ability_world_probe.rs','live_probe.rs']:(out/name).write_bytes((ability/name).read_bytes())
    (out/'ability_preparation.json').write_bytes((ability/'preparation.json').read_bytes())
    (out/'ability_buff_component.rs').write_bytes((ROOT/'維護工具/candidates/native44FE-skill-book-audit/ability_buff_component.rs').read_bytes())
    (out/'ability-codecs.json').write_bytes((Path(ability_prep['current_codec_evidence'])/'codec.stdout').read_bytes())
    (out/'mod.rs').write_bytes((out/'mod.rs').read_bytes()+b'\npub mod native_ability_buffs;\n')
    runtime_overrides+=['native_skills.rs','native_skill_selection.rs','native_world.rs','native_ability_buffs.rs']
(out/'runner.py').write_bytes(Path(__file__).read_bytes());(out/'compiled-inputs.json').write_bytes((normal/'compiled-inputs.json').read_bytes())
lines=[];previous_path=False;overrides=[]
for line in (out/'mod.rs').read_text('utf-8-sig').splitlines():
    path=re.match(r'\s*#\[path\s*=\s*"([^"\n]+)"\]\s*$',line)
    if path:
        resolved=(P/'src/modern_lito'/path[1]).resolve(strict=True);lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']');previous_path=True;continue
    declaration=re.match(r'^(?:(?:pub(?:\([^)]*\))?)\s+)?mod\s+(\w+);\s*$',line)
    if declaration:
        name=declaration[1]+'.rs'
        if not previous_path:
            if name in runtime_overrides:resolved=out/name;overrides.append(name)
            else:resolved=P/'src/modern_lito'/name
            if not resolved.exists():resolved=P/'src/modern_lito'/declaration[1]/'mod.rs'
            assert resolved.is_file(),str(resolved)
            lines.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']')
        previous_path=False
    lines.append(line)
assert sorted(overrides)==sorted(runtime_overrides)
(out/'resolved_mod.rs').write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')
wrapper='#[path = '+json.dumps((out/'resolved_mod.rs').as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n'
wrapper+='#[path = '+json.dumps((P/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\n'
(out/'lib_source.rs').write_text(wrapper,encoding='utf-8',newline='\n')
libraries={};native=set();development=[]
for line in(normal/a.profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    row=json.loads(line)
    if row.get('reason')=='build-script-executed':native.update(row.get('linked_paths',[]))
    if row.get('reason')=='compiler-artifact':
        if row['target']['name']=='lito_modern'and not row['profile']['test']:development.append(row['profile'])
        for path in row['filenames']:
            if path.endswith('.rlib'):
                name=row['target']['name'].replace('-','_')
                if name not in libraries or '/x86_64-pc-windows-msvc/'in path.replace('\\','/'):libraries[name]=path
assert {'lito_modern','anyhow','serde_json','png'}<=libraries.keys();libraries.pop('lito_modern')
manifest=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'));release=manifest['profile']['release']
assert tuple(release[k]for k in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
common=['--edition=2021','--target','x86_64-pc-windows-msvc']
for path in [P/('target/current/x86_64-pc-windows-msvc/'+a.profile.lower()+'/deps'),P/('target/current/'+a.profile.lower()+'/deps')]:common+=['-L','dependency='+str(path)]
for name,path in sorted(libraries.items()):common+=['--extern',name+'='+path]
for path in sorted(native):common+=['-L',path]
if a.profile=='Release':
    for key in ['opt-level','lto','codegen-units','strip']:common+=['-C',key+'='+str(release[key])]
else:
    assert development and all(p['opt_level']=='0'and p['debuginfo']==1 and p['debug_assertions']and p['overflow_checks']for p in development)
    common+=['-C','opt-level=0','-C','debuginfo=1','-C','debug-assertions=yes','-C','overflow-checks=yes']
rust=Path(os.environ['USERPROFILE'])/'.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe';rlib=work/('liblito_extra_finite_effects.rmeta'if a.check_only else'liblito_extra_finite_effects.rlib')
products={name:dict(path=str(work/(name+'.exe')))for name in ['scene23','additional11']}
test_sources=[('scene23',P/'tests/native_world_lighting_scene.rs'),('additional11',out/'resource_probe.rs')]
if ability_prep:
    products.update({name:dict(path=str(work/(name+'.exe')))for name in ['ability5','ability_world2','ability_live']})
    test_sources+=[('ability5',out/'ability_buff_component.rs'),('ability_world2',out/'ability_world_probe.rs'),('ability_live',out/'live_probe.rs')]
commands=[('library',[str(rust),str(out/'lib_source.rs'),'--crate-name','lito_modern','--crate-type','rlib','-C','metadata=normal44FE_extra_finite11','-o',str(rlib)]+common)]
if a.check_only:commands[0][1].append('--emit=metadata');products={}
for name,source in ([]if a.check_only else test_sources):
    commands.append((name,[str(rust),str(source),'--crate-name','native_extra_finite_'+name,'--test','--extern','lito_modern='+str(rlib),'-o',products[name]['path']]+common))
k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=wintypes.HANDLE;k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(handle):
    ts=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(handle,*[ctypes.byref(t)for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
identity=dict(base_source_manifest_sha256=c['source_manifest_sha256'],candidate_files={name:sha(out/name)for name in runtime_overrides+['mod.rs','resource_probe.rs']+(['live_probe.rs','ability_buff_component.rs','ability_world_probe.rs','ability-codecs.json']if ability_prep else[])})
save(out/'candidate-inputs.json',identity)
state=dict(status='compiling_isolated_lazy_finite_effect_variant',profile=a.profile,base_source_manifest_sha256=c['source_manifest_sha256'],candidate_manifest_sha256=sha(out/'candidate-inputs.json'),normal_qualification=str(normal_q),normal_qualification_sha256=sha(normal_q),normal_mains={p:r['normal']['sha256']for p,r in c['profiles'].items()},
    original_runner=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_process=None,products=products,diagnostic_library=str(rlib),runtime=str(work),release_configuration=release,
    source_overlays_used=True,overlaid_runtime_module='native_skill_effects.rs',new_module='native_additional_finite_effects.rs',ability_request_candidate_included=bool(ability_prep),runtime_overrides=runtime_overrides,metadata_only=a.check_only,normal_product_modified=False,actual_sessions=0,actual_casts=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'receipt.json',state);env=dict(os.environ,CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=manifest['package']['version'],CARGO_PKG_NAME=manifest['package']['name'],TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
print('EXTRA_FINITE_COMPILE='+str(out),flush=True)
try:
    for label,command in commands:
        guard()
        with(out/(label+'.stdout')).open('wb')as stdout,(out/(label+'.stderr')).open('wb')as stderr:
            child=subprocess.Popen(command,cwd=P,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            job=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),label=label,command=command);state['process_history'].append(job);state['owned_process']=job;save(out/'receipt.json',state)
            job['exit_code']=child.wait();state['owned_process']=None;save(out/'receipt.json',state)
        assert job['exit_code']==0,label
    guard()
    for product in products.values():product['sha256']=sha(product['path'])
    state.update(status='typechecked_isolated_lazy_finite_effect_variant_metadata_only'if a.check_only else'compiled_isolated_lazy_finite_effect_variant',diagnostic_library_sha256=sha(rlib))
except BaseException as error:state.update(status='failed_preserved',error=repr(error));raise
finally:save(out/'receipt.json',state)
print(json.dumps(dict(compile=str(out),profile=a.profile,normal_product_modified=False),ensure_ascii=False))
