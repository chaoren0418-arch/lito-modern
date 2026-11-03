"""Run the six explicit real-resource scene tests from already qualified EXEs."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, re, subprocess, sys, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
APP = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL = Path(sys.argv[1]).resolve(strict=True)
assert NORMAL.is_relative_to(APP)
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
save = lambda p, v: p.write_text(json.dumps(v, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
now = lambda: datetime.now(timezone.utc).isoformat()

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

nq = read(NORMAL/'qualified.json')
scope = read(NORMAL/'scoped-qualified.json')
pins = read(NORMAL/'source-inputs.json')
assert nq['status'] == 'qualified_unique_entry_floor_port_normal_builds'
assert scope['status'] == 'qualified_floor_port_Debug120_Release120_and_normal_mains'
assert len(pins) == 1005 and sha(NORMAL/'source-inputs.json') == nq['source_manifest_sha256']

def check():
    for name, value in pins.items():
        assert sha(ROOT/name) == value, name
    for main in nq['profiles'].values():
        assert sha(main['path']) == main['sha256']

check()
output = APP/('floor-scene-optin-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
output.mkdir()
(output/'runner.py').write_bytes(Path(__file__).read_bytes())
receipt = {'status': 'running_explicit_real_resource_scene_tests', 'started_utc': now(),
           'normal_qualification': str(NORMAL/'qualified.json'),
           'normal_qualification_sha256': sha(NORMAL/'qualified.json'),
           'source_manifest_sha256': nq['source_manifest_sha256'], 'source_files': 1005,
           'normal_mains': {p: m['sha256'] for p, m in nq['profiles'].items()},
           'default_ignored_counts_remain_six_per_profile': True,
           'results': {}, 'owned_process': None, 'whole_goal_complete': False}
save(output/'receipt.json', receipt)
save(APP/'floor-scene-optin-current.json', {'output': str(output)})
env = os.environ.copy()
for key in list(env):
    if key.upper().startswith('LITO_'):
        del env[key]
env.update(LITO_SHARED_RESOURCE_ROOT=r'D:\lineage開發\0-客戶端',
           TEMP=str(PROJECT/'target/current/scratch'), TMP=str(PROJECT/'target/current/scratch'),
           PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8')
try:
    for profile in ['Debug', 'Release']:
        check()
        run = Path(scope['runs'][profile][0]['path'])
        q = read(run/'profile-qualified.json')
        assert q['source_manifest_sha256'] == nq['source_manifest_sha256']
        assert sha(run/'test-products.zip') == q['test_products_archive_sha256']
        product = read(run/'products.json')['lito_modern']
        exe = Path(product['path']).resolve(strict=True)
        assert exe.is_relative_to(PROJECT/'target/current') and sha(exe) == product['sha256']
        with zipfile.ZipFile(run/'test-products.zip') as z:
            assert hashlib.sha256(z.read('lito_modern.exe')).hexdigest().upper() == product['sha256']
        folder = output/profile
        folder.mkdir()
        command = [str(exe), 'world_scene::tests::', '--ignored', '--test-threads=1']
        save(folder/'invocation.json', {'arguments': command, 'test_EXE_sha256': product['sha256'],
             'original_profile_qualification': str(run/'profile-qualified.json'),
             'original_profile_qualification_sha256': sha(run/'profile-qualified.json'),
             'expected_test_names': scope['ignored_test_names'][profile],
             'read_only_resource_root': env['LITO_SHARED_RESOURCE_ROOT'],
             'capture_and_reference_write_environment_disabled': True})
        with (folder/'stdout.log').open('wb') as out, (folder/'stderr.log').open('wb') as err:
            child = subprocess.Popen(command, cwd=PROJECT, env=env, stdout=out, stderr=err,
                                     creationflags=subprocess.CREATE_NO_WINDOW)
            receipt['owned_process'] = {'pid': child.pid, 'arguments': command, 'observed_started_utc': now()}
            receipt['active_profile'] = profile
            save(output/'receipt.json', receipt)
            code = child.wait()
            receipt['owned_process'] = None
        text = (folder/'stdout.log').read_text(encoding='utf-8-sig')
        names = re.findall(r'^test (\S+) \.\.\. (?:ok|FAILED)', text, re.M)
        result = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', text)
        receipt['results'][profile] = {'exit_code': code, 'test_EXE_sha256': product['sha256'],
                                       'test_names': names, 'summary': result}
        save(output/'receipt.json', receipt)
        assert code == 0 and set(names) == set(scope['ignored_test_names'][profile]) and len(names) == 6
        assert len(result) == 1 and result[0][:5] == ('ok', '6', '0', '0', '0'), result
        assert sha(exe) == product['sha256']
    check()
    receipt.update(status='actual_both_profiles_six_optin_scene_tests_passed', source_inputs_unchanged=True,
                   normal_mains_unchanged=True, active_profile=None)
except BaseException as error:
    receipt.update(status='failed_preserved', error=repr(error))
    raise
finally:
    receipt['finished_utc'] = now()
    save(output/'receipt.json', receipt)
    files = {str(p.relative_to(output)).replace('\\', '/'): sha(p)
             for p in output.rglob('*') if p.is_file()}
    archive = output/'raw-evidence.zip'
    with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
        for name in files:
            z.write(output/name, name)
    with zipfile.ZipFile(archive) as z:
        assert set(z.namelist()) == set(files)
        for name, value in files.items():
            assert hashlib.sha256(z.read(name)).hexdigest().upper() == value
    save(output/'archive-readback.json', {'status': receipt['status'], 'files': files,
         'archive_sha256': sha(archive), 'archive_readback_verified': True,
         'full_library_regression_verified': False, 'normal_GUI_verified': False,
         'whole_goal_complete': False})
print(json.dumps({'output': str(output), 'status': receipt['status']}, ensure_ascii=False))
