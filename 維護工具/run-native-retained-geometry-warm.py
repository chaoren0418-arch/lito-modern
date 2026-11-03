"""Run real Model timings only after completed retained-geometry related validation and quiet checks."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,os,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
RUSTC=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\rustc.exe')
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--related-completion',required=True,type=Path);a=ap.parse_args()
related_path=a.related_completion.resolve(strict=True);broad=read(related_path)
assert related_path.is_relative_to(ROOT/'驗證紀錄/current/native-retained-geometry')
assert broad['status']=='independently_qualified_retained_geometry_both_profiles'
assert broad['observed_terminal_exit_code']==0 and broad['recorded_owned_processes_all_terminal']
assert set(broad['profiles'])=={'Debug','Release'} and all(v['counts']['failed']==0 for v in broad['profiles'].values())
BUILD=related_path.parent
pins=read(BUILD/'compiled-inputs.json');assert len(pins)==1025
assert broad['source_manifest_sha256']==sha(BUILD/'source-inputs.json')
budget=ROOT/'驗證紀錄/current/native-performance/acceptance-budget-v1.json'
assert sha(budget)=='98F17500C627D9A5F18767329A7F4913CEB42F09B1226DFA6F8B3735E8CF3CBA'
deps=PROJECT/'target/current/x86_64-pc-windows-msvc/release/deps';libraries={};native_paths=set()
for line in (BUILD/'Release/build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):continue
    m=json.loads(line)
    if m.get('reason')=='build-script-executed':native_paths.update(m.get('linked_paths',[]))
    if m.get('reason')=='compiler-artifact' and m['target']['name'] in ['lito_modern','serde_json']:
        paths=[Path(s) for s in m['filenames'] if s.endswith('.rlib') and Path(s).parent==deps]
        if paths:assert len(paths)==1;libraries[m['target']['name']]=paths[0]
assert set(libraries)=={'lito_modern','serde_json'}
source=ROOT/'維護工具/oracle/NativeRetainedGeometryWarmDiagnostic.rs';source_sha=sha(source)
def stable():
    for name,value in pins.items():assert sha(ROOT/name)==value
    assert sha(source)==source_sha
    for profile,info in broad['profiles'].items():
        value=info['normal_sha256']
        assert sha(PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe')==value
def quiet():
    escaped=str(ROOT).replace("'","''")
    command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$r='"+escaped+"';$heavy=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -in @('cargo.exe','rustc.exe')) -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and $_.Name -like '*.exe' -and $_.Name -ne 'python.exe')});$os=Get-CimInstance Win32_OperatingSystem;$cpu=Get-CimInstance Win32_Processor;[pscustomobject]@{heavy=$heavy|Select-Object ProcessId,CreationDate,Name,CommandLine;free_physical_KiB=$os.FreePhysicalMemory;CPU_load_percent=$cpu.LoadPercentage;observed_utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 4 -Compress"
    p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert p.returncode==0 and not p.stderr;value=json.loads(p.stdout.decode('utf-8-sig'))
    assert not value['heavy'],'Owned/global compiler or owned executable validation still active; do not benchmark.'
    return value
stable();before=quiet();stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-performance'/('retained-geometry-warm-'+stamp)
runtime=PROJECT/'target/current/native-warm-residence-diagnostic'/stamp
out.mkdir();runtime.mkdir(parents=True)
for name,path in [('runner.py',Path(__file__)),('probe.rs',source),('compiled-inputs.json',BUILD/'compiled-inputs.json'),
    ('world-fixture.json',PROJECT/'tests/fixtures/native-world-current.json'),('related-completion.json',related_path),('acceptance-budget-v1.json',budget)]:
    (out/name).write_bytes(path.read_bytes())
save(out/'quiet-before-compile.json',before);exe=runtime/'warm-residence-diagnostic.exe'
command=[str(RUSTC),str(out/'probe.rs'),'--edition=2021','--crate-name','native_warm_residence_diagnostic','--target','x86_64-pc-windows-msvc',
    '-C','opt-level=z','-C','lto=thin','-C','codegen-units=16','-C','strip=symbols','-L','dependency='+str(deps),
    '-L','dependency='+str(PROJECT/'target/current/release/deps'),'-o',str(exe)]
for name,path in libraries.items():command+=['--extern',name+'='+str(path)]
for path in sorted(native_paths):command+=['-L',path]
state=dict(status='prepared_quiet_Model_warm_residence_diagnostic',output=str(out),runtime=str(runtime),
    source_manifest_sha256=broad['source_manifest_sha256'],related_completion=str(related_path),related_completion_sha256=sha(related_path),full_current_regression_verified=False,
    budget_sha256=sha(budget),compiler_arguments=command,libraries={n:dict(path=str(p),sha256=sha(p)) for n,p in libraries.items()},
    synthetic_authority_fixture=True,actual_formal_sessions=0,GUI_verified=False,performance_accepted=False,whole_goal_complete=False,
    processes=[],owned_process=None)
def persist():save(out/'receipt.json',state)
env={k:v for k,v in os.environ.items() if not k.startswith('LITO_')};env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
def execute(args,label):
    with (out/(label+'-stdout.log')).open('wb') as stdout,(out/(label+'-stderr.log')).open('wb') as stderr:
        p=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        item=dict(pid=p.pid,arguments=args,started_utc=datetime.now(timezone.utc).isoformat());state['processes'].append(item);state['owned_process']=item;persist()
        code=p.wait();item['exit_code']=code;state['owned_process']=None;persist()
    assert code==0,(label,code,'Preserve failure before repair')
persist();print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
try:
    state['status']='compiling_diagnostic_Release_wrapper';persist();execute(command,'compile')
    save(out/'quiet-before-measurement.json',quiet());stable()
    state['status']='measuring_quiet_synthetic_Model_warm_residence';persist();execute([str(exe),r'D:\lineage開發\0-客戶端',str(out)],'diagnostic')
    save(out/'quiet-after-measurement.json',quiet());stable();result=read(out/'diagnostic-results.json')
    assert result['measured_reloads']==40 and result['warmup_pairs']==5
    limits=read(budget)['budgets']['native_scene_load'];timing=result['wall_us']
    state.update(status='quiet_Model_warm_residence_diagnostic_complete',diagnostic_sha256=sha(exe),wall_us=timing,
        synthetic_warm_distribution_within_original_budget=timing['p95_nearest_rank']<=limits['warm_residency_reload_p95_ms_max']*1000
            and timing['maximum']<=limits['warm_residency_reload_single_ms_max']*1000);persist()
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));persist();raise
finally:
    files={p.name:p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip','archive-readback.json']}
    if exe.exists():files['Release-x64-diagnostic.exe']=exe
    hashes={name:sha(p) for name,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for name,p in files.items():z.write(p,name)
    with zipfile.ZipFile(out/'raw.zip') as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for name,value in hashes.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
    save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
print(json.dumps(dict(output=str(out),status=state['status'],wall_us=state.get('wall_us'),performance_accepted=False),ensure_ascii=False),flush=True)
