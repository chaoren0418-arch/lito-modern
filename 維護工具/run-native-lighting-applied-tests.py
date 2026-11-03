"""Retain and execute lighting and related world lifecycle targets."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, re, subprocess, sys, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
TOOLS = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
APP = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
now = lambda: datetime.now(timezone.utc).isoformat()
save = lambda p, v: p.write_text(json.dumps(v, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
pins = read(APP / 'repair-check-1/compiled-inputs.json')
# These are the explicit inputs changed for the first scene mask pass. All
# remaining baseline pins must still match, before any compilation begins.
for name in ['LiTo381-Modern/tests/native_light_lifecycle.rs',
             'LiTo381-Modern/src/modern_lito/mod.rs',
             'LiTo381-Modern/src/modern_lito/native_login.rs',
             'LiTo381-Modern/src/modern_lito/native_environment.rs',
             'LiTo381-Modern/src/modern_lito/native_render_environment.rs',
             'LiTo381-Modern/src/modern_lito/compositor.rs',
             'LiTo381-Modern/tests/native_render_environment.rs',
             'LiTo381-Modern/tests/native_environment.rs',
             'LiTo381-Modern/src/modern_lito/native_water_clock.rs',
             'LiTo381-Modern/tests/native_water_clock.rs',
             'LiTo381-Modern/tests/fixtures/native-water-clock-original.bin',
             'LiTo381-Modern/src/modern_lito/native_actor_relations.rs',
             'LiTo381-Modern/src/modern_lito/native_light_sources.rs',
             'LiTo381-Modern/tests/native_actor_relations.rs',
             'LiTo381-Modern/src/modern_lito/native_world_scene.rs',
             'LiTo381-Modern/src/modern_lito/native_world_view.rs',
             'LiTo381-Modern/src/modern_lito/native_world.rs',
             'LiTo381-Modern/tests/native_motion.rs',
             'LiTo381-Modern/src/modern_lito/native_world_lighting.rs',
             'LiTo381-Modern/tests/native_world_lighting.rs',
             'LiTo381-Modern/tests/native_world_lighting_scene.rs',
             'LiTo381-Modern/src/modern_lito/native_water_light.rs',
             'LiTo381-Modern/tests/native_water_light.rs',
             'LiTo381-Modern/tests/fixtures/native-water-original.bin']:
    pins[name] = sha(ROOT / name)
assert len(pins) == 1002
def check():
    for name, value in pins.items():
        assert sha(ROOT / name) == value, name
check()
output = APP / ('actual-tests-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
output.mkdir()
save(output / 'compiled-inputs.json', pins)
(output / 'runner.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(output / 'compiled-source.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name in pins: z.write(ROOT / name, name)
with zipfile.ZipFile(output / 'compiled-source.zip') as z:
    assert set(z.namelist()) == set(pins)
    for name, value in pins.items(): assert hashlib.sha256(z.read(name)).hexdigest().upper() == value
suites = ['native_light_color', 'native_light_sources', 'native_render_environment', 'native_light_lifecycle', 'native_world_lighting',
          'native_world', 'native_motion', 'native_actor_states', 'native_environment','native_world_lighting_scene','native_water_light','native_actor_relations','native_water_clock']
if '--scene-only' in sys.argv[1:]: suites=['native_world_lighting_scene']
if '--relation-labels-only' in sys.argv[1:]:
    suites=['native_world_metadata','native_label_updates','native_world_labels','native_clan_markers','native_emblems']
if '--include-relation-labels' in sys.argv[1:]:
    suites+=['native_world_metadata','native_label_updates','native_world_labels','native_clan_markers','native_emblems']
profile='Release' if '--release' in sys.argv[1:] else 'Debug'
receipt = {'status':'compiling_actual_applied_lighting_tests', 'started_utc':now(),
           'compiled_inputs':len(pins), 'normal_main_build_scope':'Cargo may also build bin targets; retain them separately without normal-main qualification', 'scene_pixel_pass_verified':False,
           'whole_goal_complete':False, 'profile':profile, 'suites':suites, 'results':{}, 'owned_process':None}
persist = lambda: save(output / 'receipt.json', receipt)
persist(); save(APP / 'actual-tests-current.json', {'output':str(output)})
env = os.environ.copy()
env.update(RUSTC=str(TOOLS/'rustc.exe'), RUSTDOC=str(TOOLS/'rustdoc.exe'),
           TEMP=str(PROJECT/'target/current/scratch'), TMP=str(PROJECT/'target/current/scratch'))
env['PATH'] = str(TOOLS)+os.pathsep+env['PATH']
def run(command, folder):
    folder.mkdir()
    with (folder/'stdout.log').open('wb') as out, (folder/'stderr.log').open('wb') as err:
        child = subprocess.Popen(command, cwd=PROJECT, env=env, stdout=out, stderr=err,
                                 creationflags=subprocess.CREATE_NO_WINDOW)
        receipt['owned_process']={'pid':child.pid,'arguments':command,'observed_started_utc':now()}; persist()
        code = child.wait()
        receipt['owned_process']=None; persist()
    return code
try:
    command = [str(TOOLS/'cargo.exe'),'test','--no-run','--locked','--offline','-j1',
               '--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json']
    if profile=='Release': command.append('--release')
    for name in suites: command += ['--test',name]
    receipt['build_arguments']=command; persist()
    code=run(command,output/'build'); receipt['compile_exit_code']=code; persist()
    assert code == 0, 'Actual target compilation failed; original source and logs retained'
    check()
    products={}; incidental={}
    for line in (output/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
        if not line.startswith('{'): continue
        row=json.loads(line)
        if row.get('reason')=='compiler-artifact' and row.get('executable'):
            name=row['target']['name']; assert row['features']==[]
            path=Path(row['executable']).resolve(strict=True)
            assert path.is_relative_to(PROJECT/'target/current')
            data=path.read_bytes(); pe=int.from_bytes(data[60:64],'little')
            assert data[:2]==b'MZ' and data[pe:pe+4]==b'PE\0\0' and data[pe+4:pe+6]==b'\x64\x86'
            if row['profile']['test']:
                assert name in suites
                products[name]={'path':str(path),'sha256':sha(path)}
            else:
                incidental[name]={'path':str(path),'sha256':sha(path),'normal_main_qualified':False}
    assert set(products)==set(suites)
    save(output/'products.json',products)
    save(output/'incidental-bin-products.json',incidental)
    with zipfile.ZipFile(output/'incidental-bin-products.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for name,p in incidental.items(): z.write(p['path'],name+'.exe')
    with zipfile.ZipFile(output/'incidental-bin-products.zip') as z:
        for name,p in incidental.items(): assert hashlib.sha256(z.read(name+'.exe')).hexdigest().upper()==p['sha256']
    with zipfile.ZipFile(output/'test-products.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for name,p in products.items(): z.write(p['path'],name+'.exe')
    with zipfile.ZipFile(output/'test-products.zip') as z:
        for name,p in products.items(): assert hashlib.sha256(z.read(name+'.exe')).hexdigest().upper()==p['sha256']
    for name,p in products.items():
        receipt.update(status='running_actual_applied_tests',active_suite=name); persist()
        code=run([p['path'],'--test-threads=1'],output/name)
        log=(output/name/'stdout.log').read_text(encoding='utf-8-sig')
        result=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
        assert len(result)==1
        result=result[0]
        receipt['results'][name]={'exit_code':code,'passed':int(result[1]),'failed':int(result[2]),'ignored':int(result[3])}; persist()
        assert code==0 and result[0]=='ok' and all(int(result[i])==0 for i in [2,4,5])
    check()
    receipt.update(status='actual_applied_lighting_and_related_targets_passed',finished_utc=now(),source_inputs_unchanged=True,
                   passed=sum(v['passed'] for v in receipt['results'].values()),failed=0,ignored=sum(v['ignored'] for v in receipt['results'].values()))
    persist()
    if '--build-normal' in sys.argv[1:]:
        normal={}
        for normal_profile in ['Debug','Release']:
            check();receipt.update(status='building_normal_via_unique_entry',active_profile=normal_profile);persist()
            code=run(['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',normal_profile],output/('normal-'+normal_profile))
            assert code==0
            binary=PROJECT/'target/current/x86_64-pc-windows-msvc'/normal_profile.lower()/'LiToModern.exe'
            data=binary.read_bytes();pe=int.from_bytes(data[60:64],'little')
            assert data[:2]==b'MZ' and data[pe:pe+4]==b'PE\0\0' and data[pe+4:pe+6]==b'\x64\x86'
            normal[normal_profile]={'path':str(binary),'sha256':sha(binary),'unique_entry_build_exit_code':code,'normal_GUI_verified':False}
            log=ROOT/'驗證紀錄/current'/('build-release.log' if normal_profile=='Release' else 'build.log')
            (output/('normal-'+normal_profile)/'entry-build.log').write_bytes(log.read_bytes())
            with zipfile.ZipFile(output/('normal-'+normal_profile)/'product.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:z.write(binary,'LiToModern.exe')
            with zipfile.ZipFile(output/('normal-'+normal_profile)/'product.zip') as z:assert hashlib.sha256(z.read('LiToModern.exe')).hexdigest().upper()==normal[normal_profile]['sha256']
            save(output/'normal-products.json',normal)
        check();receipt.update(status='actual_lighting_tests_and_unique_entry_normal_builds_passed',normal_products=normal,source_inputs_unchanged=True,finished_utc=now())
except BaseException as error:
    receipt.update(status='failed_preserved',finished_utc=now(),error=repr(error)); persist(); raise
persist()
print(json.dumps({'output':str(output),'status':receipt['status'],'passed':receipt['passed']}))
