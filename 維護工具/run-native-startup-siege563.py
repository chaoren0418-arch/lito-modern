"""Reuse qualified current x64 test images for real thirty-player map563 runs.

Both-profile mode requires the completed original default110 pipeline. A single
already-qualified profile may run in its private output tree while the other
profile is compiling. No compilation or product-source edits occur here.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, subprocess, sys, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
now = lambda: datetime.now(timezone.utc).isoformat()

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, v):
    p.write_text(json.dumps(v, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('default_cohort', type=Path)
ap.add_argument('--profile', choices=['Debug', 'Release', 'both'], default='both')
ap.add_argument('--input-kind', choices=['default110','current-default110','retained-archive-related','retained-geometry-related'], default='default110')
a = ap.parse_args()
base = a.default_cohort.resolve(strict=True)
c = read(base / 'cohort.json')
selected_profiles = ['Debug', 'Release'] if a.profile == 'both' else [a.profile]
completion_path = base / 'completion-observation.json'
if a.input_kind in ['default110','current-default110']:
    assert base.is_relative_to(ROOT / '驗證紀錄/current/native-default110')
    if a.profile == 'both':
        completion = read(completion_path)
        assert c['status'] == 'independently_qualified_current_default110_both_profiles'
        assert completion['observed_terminal_exit_code'] == 0
        assert completion['recorded_original_owned_processes_all_terminal']
    for profile in selected_profiles:
        assert c['profiles'][profile]['qualified'] and c['profiles'][profile]['actual_runner_exit_code'] == 0
    if a.input_kind=='default110':
        assert c['source_manifest_sha256']=='A9E802DB33BE2FF06EAA155D9D764A2DCB9018B9B2CC912E2D0A6C3EBF849FF7'
    else:
        assert a.profile=='both' and c['source_manifest_sha256']==completion['source_manifest_sha256']
        for profile in selected_profiles:
            bound=read(c['profiles'][profile]['qualification'])
            assert bound['source_manifest_sha256']==c['source_manifest_sha256'] and bound['actual_test_executables']==111 and bound['counts']['failed']==0
    normal_path = Path(c['normal_qualification'])
    n = read(normal_path)
    normal_root=normal_path.parent.parent if normal_path.parent.name=='qualified' else normal_path.parent
    normal = read(normal_root / 'cohort.json')
    pins = read(normal_root / 'source-inputs.json')
    qualifications = {p:dict(path=c['profiles'][p]['qualification'],sha256=c['profiles'][p]['qualification_sha256']) for p in selected_profiles}
else:
    assert base.is_relative_to(ROOT / '驗證紀錄/current' / ('native-retained-geometry' if a.input_kind=='retained-geometry-related' else 'native-terrain-archive-retention'))
    completion = read(completion_path)
    assert completion['status'] == 'independently_qualified_'+('retained_geometry' if a.input_kind=='retained-geometry-related' else 'retained_archive')+'_both_profiles'
    assert completion['recorded_owned_processes_all_terminal'] and completion['recorded_owned_identities'] == (26 if a.input_kind=='retained-geometry-related' else 20)
    assert c['status'] == 'executed_'+('retained_geometry' if a.input_kind=='retained-geometry-related' else 'retained_archive')+'_related_tests_and_unique_normals'
    assert c['owned_process'] is None and all(p['exit_code'] == 0 for p in c['process_history'])
    assert c['source_manifest_sha256'] == completion['source_manifest_sha256']
    normal_path = completion_path
    n = completion
    normal = c
    pins = read(base / 'source-inputs.json')
    qualifications = {p:dict(path=completion['profiles'][p]['qualification'],sha256=completion['profiles'][p]['qualification_sha256']) for p in selected_profiles}
    for profile in selected_profiles:
        q = read(qualifications[profile]['path'])
        assert sha(qualifications[profile]['path']) == qualifications[profile]['sha256']
        assert q['status'] == 'independently_qualified_'+('retained_geometry' if a.input_kind=='retained-geometry-related' else 'retained_archive')+'_related_profile'
        assert q['source_manifest_sha256'] == c['source_manifest_sha256'] and q['actual_test_executables'] == (10 if a.input_kind=='retained-geometry-related' else 7)
        assert q['counts']['failed'] == 0
assert len(pins) == 1024
jar = PROJECT / 'target/current/server-reference/formal-server-C02AAAE8.jar'
catalog = PROJECT / 'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(jar) == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(catalog) == 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
tools = ['run-native-arena-portal.ps1', 'verify-native-arena-portal.py', 'verify-native-terrain-arena.py']
qa = {name: sha(ROOT / '維護工具' / name) for name in tools}
out = ROOT / '驗證紀錄/current/native-startup-siege563' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
for name in tools:
    (out / name).write_bytes((ROOT / '維護工具' / name).read_bytes())
save(out / 'source-inputs.json', pins)
kernel=ctypes.WinDLL('kernel32',use_last_error=True)
kernel.GetCurrentProcess.restype=wintypes.HANDLE
kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
kernel.GetProcessTimes.restype=wintypes.BOOL
own_times=[wintypes.FILETIME() for _ in range(4)]
assert kernel.GetProcessTimes(kernel.GetCurrentProcess(),*[ctypes.byref(v) for v in own_times])
own_ticks=own_times[0].dwLowDateTime|(own_times[0].dwHighDateTime<<32)
runner_identity=dict(pid=os.getpid(),creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=own_ticks//10)).isoformat())
state = dict(status='prepared_current_map563_thirty_player_TW_extended', output=str(out),
    input_kind=a.input_kind, input_cohort=str(base), runner_identity=runner_identity,
    default_cohort=str(base) if a.input_kind in ['default110','current-default110'] else None,
    input_completion_sha256=sha(completion_path) if completion_path.exists() else None,
    default_completion_sha256=sha(completion_path) if a.input_kind in ['default110','current-default110'] and completion_path.exists() else None,
    selected_profiles=selected_profiles, original_default_pipeline_complete=c['status']=='independently_qualified_current_default110_both_profiles',
    related_seven_target_validation_only=a.input_kind == 'retained-archive-related',
    related_ten_target_validation_only=a.input_kind == 'retained-geometry-related',
    profile_qualifications=qualifications,
    source_files=1024, compiled_input_files=1025, source_manifest_sha256=c['source_manifest_sha256'],
    normal_qualification=str(normal_path), normal_qualification_sha256=sha(normal_path),
    normal_mains={p: v['normal_sha256'] for p, v in n['profiles'].items()},
    jar_sha256=sha(jar), common_catalog_sha256=sha(catalog), resource_profile='tw13081901-v1',
    extended_item_details=True, validation_pins=qa, profiles={}, process_history=[], owned_process=None,
    normal_EXE_runtime_verified=False, GUI_verified=False, physical_input_verified=False,
    whole_G06_complete=False, whole_goal_complete=False, started_utc=now())
persist = lambda: save(out / 'cohort.json', state)
env = os.environ.copy()
env.update(PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8',
    TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'))
env['JAVA_TOOL_OPTIONS'] = '-Dfile.encoding=UTF-8 -Dlito.resourceProfile=tw13081901-v1 -Dlito.extendedItemDetails=true -Dlito.commonCatalog=' + str(catalog)
state['JAVA_TOOL_OPTIONS_for_owned_children_only'] = env['JAVA_TOOL_OPTIONS']

def check():
    for name, digest in pins.items():
        assert sha(ROOT / name) == digest, name
    for name, digest in qa.items():
        assert sha(ROOT / '維護工具' / name) == digest, name
    for profile, info in normal['profiles'].items():
        assert sha(info['normal']['path']) == n['profiles'][profile]['normal_sha256']
    assert sha(jar) == state['jar_sha256'] and sha(catalog) == state['common_catalog_sha256']

def execute(command, folder):
    folder.mkdir()
    with (folder / 'stdout.log').open('wb') as stdout, (folder / 'stderr.log').open('wb') as stderr:
        child = subprocess.Popen(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr,
            creationflags=subprocess.CREATE_NO_WINDOW)
        times = [wintypes.FILETIME() for _ in range(4)]
        call = ctypes.WinDLL('kernel32', use_last_error=True).GetProcessTimes
        call.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        call.restype = wintypes.BOOL
        assert call(wintypes.HANDLE(int(child._handle)), *[ctypes.byref(t) for t in times])
        ticks = times[0].dwLowDateTime | times[0].dwHighDateTime << 32
        created = datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)
        item = dict(pid=child.pid, creation_utc=created.isoformat(), command=command, output=str(folder))
        state['process_history'].append(item)
        state['owned_process'] = item
        persist()
        code = child.wait()
        item.update(exit_code=code, finished_utc=now())
        state['owned_process'] = None
        persist()
    return code

persist()
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
try:
    for profile in selected_profiles:
        check()
        prior = c['profiles'][profile]
        run = Path(prior['run']) if a.input_kind in ['default110','current-default110'] else base
        qualification = qualifications[profile]
        q = read(qualification['path'])
        assert sha(qualification['path']) == qualification['sha256'] and q['counts']['failed'] == 0
        inputs = read(run / 'compiled-inputs.json')
        assert len(inputs) == 1025 and all(sha(ROOT / name) == h for name, h in inputs.items())
        if a.input_kind in ['default110','current-default110']:
            products = read(run / 'product-retention.json')
        else:
            products = dict(products=prior['products'], archive=prior['test_products_archive']['path'], archive_sha256=prior['test_products_archive']['sha256'])
        product = products['products']['native_arena_portal']
        assert sha(product['path']) == product['sha256']
        assert sha(products['archive']) == q['product_archive_sha256' if a.input_kind in ['default110','current-default110'] else 'test_products_archive_sha256']
        with zipfile.ZipFile(products['archive']) as z:
            with z.open('native_arena_portal.exe') as stream:
                assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == product['sha256']
        d = out / profile
        d.mkdir()
        manifest = d / 'build-inputs.json'
        save(manifest, dict(status='completed_build', profile=profile, files=inputs,
            products={'native_arena_portal': product}, original_compilation=str(run),
            original_qualification=qualification['path'], original_qualification_sha256=qualification['sha256'],
            qualification_input_kind=a.input_kind, broad_default_regression_claimed=a.input_kind in ['default110','current-default110'],
            compiled_source_archive=str(run / 'compiled-source.zip'),
            compiled_source_archive_sha256=q['compiled_source_archive_sha256'],
            test_products_archive=products['archive'], test_products_archive_sha256=products['archive_sha256'],
            no_new_compilation=True))
        state.update(status='running_current_map563_thirty_player_TW_extended', active_profile=profile)
        state['profiles'][profile] = dict(directory=str(d), test_binary_sha256=product['sha256'],
            retained_build_manifest_sha256=sha(manifest))
        persist()
        code = execute(['pwsh.exe', '-NoProfile', '-File', str(ROOT / '維護工具/run-native-arena-portal.ps1'),
            '-TestBinary', product['path'], '-ReferenceJar', str(jar), '-BuildProfile', profile,
            '-MapId', '563', '-BuildInputManifest', str(manifest)], d / 'runtime')
        state['profiles'][profile]['runtime_exit_code'] = code
        persist()
        assert code == 0, 'Retain original failure; do not retry automatically.'
        raw = (d / 'runtime/stdout.log').read_bytes()
        try:
            text = raw.decode('utf-8-sig')
        except UnicodeDecodeError:
            text = raw.decode('cp950')
            assert text.encode('cp950') == raw
        cases = []
        for line in text.splitlines():
            path = Path(line.strip())
            if path.is_absolute() and path.is_dir() and (path / 'receipt.json').exists():
                cases.append(path.resolve())
        assert len(cases) == 1
        case = cases[0]
        assert case.is_relative_to(ROOT / '驗證紀錄/current/native-arena-portal')
        r = read(case / 'receipt.json')
        assert r['map_id'] == 563 and r['test_binary_sha256'] == product['sha256'] and r['build_profile'] == profile
        state['profiles'][profile]['case'] = str(case)
        persist()
        code = execute([sys.executable, '-X', 'utf8', str(ROOT / '維護工具/verify-native-terrain-arena.py'), str(case)], d / 'readback')
        state['profiles'][profile]['comparison_exit_code'] = code
        persist()
        assert code == 0, 'Preserve independent-reader failure before repairing.'
        check()
        print(json.dumps(dict(profile=profile, map=563, actual_sessions=60, case=str(case)), ensure_ascii=False), flush=True)
    state.update(status='executed_current_selected_map563_thirty_player_TW_extended', active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    state['finished_utc'] = now()
    persist()
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
