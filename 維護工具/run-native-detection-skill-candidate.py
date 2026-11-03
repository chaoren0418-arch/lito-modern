"""Compile request/menu/effect proposals with unchanged scoped loader dependency."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
BUILD=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
CANDIDATE=ROOT/'維護工具/candidates/native-detection-skill'
PROBE=ROOT/'維護工具/oracle/NativeDetectionSkillCandidateProbe.rs'
RUSTC=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rustc.exe')
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
meta=read(CANDIDATE/'metadata.json');pins=read(BUILD/'compiled-inputs.json');assert len(pins)==1025
candidate_files={}
for name,info in meta['files'].items():
    assert sha(CANDIDATE/('base-'+name))==sha(PROJECT/'src/modern_lito'/name)==info['base_sha256']
    assert sha(CANDIDATE/name)==info['candidate_sha256']
    candidate_files[name]=info['candidate_sha256'];candidate_files['base-'+name]=info['base_sha256']
candidate_files['metadata.json']=sha(CANDIDATE/'metadata.json')
deps=PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps';libraries={};native_paths=set()
for line in (BUILD/'Debug/build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    m=json.loads(line)
    if m.get('reason')=='build-script-executed':native_paths.update(m.get('linked_paths',[]))
    if m.get('reason')=='compiler-artifact' and m['target']['name'] in ['lito_modern','serde','serde_json','sha2','anyhow','png']:
        paths=[Path(s) for s in m['filenames'] if s.endswith('.rlib') and Path(s).parent==deps]
        if paths:assert len(paths)==1;libraries[m['target']['name']]=paths[0]
assert set(libraries)=={'lito_modern','serde','serde_json','sha2','anyhow','png'}
probe_sha=sha(PROBE)
def stable():
    for name,value in pins.items():assert sha(ROOT/name)==value
    for name,value in candidate_files.items():assert sha(CANDIDATE/name)==value
    assert sha(PROBE)==probe_sha
stable();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-detection-skill-candidate'/stamp
runtime=PROJECT/'target/current/native-detection-skill-candidate'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
for name in candidate_files:(out/name).write_bytes((CANDIDATE/name).read_bytes())
for name,path in [('runner.py',Path(__file__)),('probe-template.rs',PROBE),('compiled-inputs.json',BUILD/'compiled-inputs.json'),
    ('world-fixture.json',PROJECT/'tests/fixtures/native-world-current.json'),('skills-fixture.json',PROJECT/'tests/fixtures/native-skills-current.json'),
    ('C02-selector-codec.json',ROOT/'驗證紀錄/current/native-current-skill-contract-audit/tw13081901-v1-selector-codec.stdout.jsonl'),
    ('C02-selector-codec-qualified.json',ROOT/'驗證紀錄/current/native-current-skill-contract-audit/selector-codec-qualified.json'),
    ('tower_assets.rs',PROJECT/'src/modern_lito/tower_assets.rs'),
    ('shared_action_catalog.rs',PROJECT/'src/modern_lito/shared_action_catalog.rs')]:
    (out/name).write_bytes(path.read_bytes())
request=(CANDIDATE/'native_skills.rs').read_text(encoding='utf-8-sig')
start=request.index('fn can_cast(');end=request.index('#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]',start)
extracted=request[start:end]
assert extracted.count('pub fn ')==5 and 'pub enum Feedback' not in extracted
(out/'request-method-extract.rs').write_text(extracted,encoding='utf-8',newline='\n')
effect_source=(out/'native_skill_effects.rs').read_text(encoding='utf-8-sig')
tw_start=effect_source.index('        // S37\'s explicit11722..11725 selects the reviewed TW data namespace.')
tw_end=effect_source.index('    }\n    pub fn draw(',tw_start)
tw_omitted=effect_source[tw_start:tw_end]
assert 'load_selected_from_launcher(' in tw_omitted and 'self.tw_heal_error' in tw_omitted
(out/'effect-method-extract.rs').write_text(effect_source[:tw_start]+effect_source[tw_end:],encoding='utf-8',newline='\n')
save(out/'method-extraction.json',dict(candidate_sha256=candidate_files['native_skills.rs'],first_function='can_cast',
    exact_contiguous_slice_end='before Feedback derive',extracted_sha256=sha(out/'request-method-extract.rs'),
    public_method_count=5,existing_Effect_type_reexported_to_preserve_World_type_identity=True,
    full_candidate_selection_compiled=True,generic_skill_effect_loader_and_draw_compiled=True,
    unrelated_private_TW_prepare_branch_omitted=True,omitted_TW_branch_sha256=hashlib.sha256(tw_omitted.encode()).hexdigest().upper(),
    effect_method_extract_sha256=sha(out/'effect-method-extract.rs'),
    unchanged_tower_loader_and_shared_catalog_copied_for_crate_visibility=True,
    loader_source_sha256=sha(out/'tower_assets.rs'),shared_catalog_source_sha256=sha(out/'shared_action_catalog.rs'),product_source_changed=False))
source=PROBE.read_text(encoding='utf-8-sig')
for token,name in [('@REQUEST_EXTRACT@','request-method-extract.rs'),('@SELECTION@','native_skill_selection.rs'),
                   ('@EFFECTS@','effect-method-extract.rs'),('@TOWER@','tower_assets.rs'),('@SHARED_CATALOG@','shared_action_catalog.rs')]:
    assert source.count(token)==1;source=source.replace(token,(out/name).as_posix())
(out/'probe.rs').write_text(source,encoding='utf-8',newline='\n')
exe=runtime/'detection-candidate.exe'
command=[str(RUSTC),str(out/'probe.rs'),'--edition=2021','--crate-name','native_detection_candidate','--target','x86_64-pc-windows-msvc',
    '-L','dependency='+str(deps),'-L','dependency='+str(PROJECT/'target/current/debug/deps'),'-o',str(exe)]
for name,p in libraries.items():command+=['--extern',name+'='+str(p)]
for p in sorted(native_paths):command+=['-L',p]
state=dict(status='prepared_unapplied_native_detection_candidate',output=str(out),runtime=str(runtime),
    source_manifest_sha256=sha(BUILD/'source-inputs.json'),candidate_files=candidate_files,probe_sha256=probe_sha,
    compiler_arguments=command,libraries={n:dict(path=str(p),sha256=sha(p)) for n,p in libraries.items()},
    candidate_applied_to_product=False,product_source_changed=False,actual_formal_sessions=0,actual_casts=0,
    GUI_verified=False,whole_goal_complete=False,processes=[],owned_process=None)
def persist():save(out/'receipt.json',state)
env={k:v for k,v in os.environ.items() if not k.startswith('LITO_')}
env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8')
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb') as stdout,(out/(label+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['processes'].append(item);state['owned_process']=item;persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(label,code,'Preserve failure before repair')
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_unapplied_detection_candidate';persist();execute(command,'compile')
    binary=exe.read_bytes();h=int.from_bytes(binary[60:64],'little')
    assert binary[:2]==b'MZ' and binary[h+4:h+6]==b'\x64\x86' and binary[h+24:h+26]==b'\x0b\x02'
    state['status']='executing_detection_candidate_with_original_resources';persist()
    execute([str(exe),r'D:\lineage開發\0-客戶端',str(out)],'candidate')
    result=read(out/'candidate-results.json');assert result['graphics']==749 and result['actual_casts']==0
    stable();state.update(status='unapplied_detection_candidate_passed_pending_independent_readback',diagnostic_sha256=sha(exe),
        source_frames=result['source_frames'],request_guard_rejections=len(result['request_guard_rejections']));persist()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if exe.exists():files['Rust-x64-probe.exe']=exe
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,value in hashes.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
    save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
