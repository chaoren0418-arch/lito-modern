"""Read back actual six-test scene opt-in runs, independently of their runner."""
from pathlib import Path
import hashlib, json, re, sys, zipfile

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
OUT = Path(sys.argv[1]).resolve(strict=True)
assert OUT.is_relative_to(APP)
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f,'sha256').hexdigest().upper()

receipt = read(OUT/'receipt.json')
assert receipt['status'] == 'actual_both_profiles_six_optin_scene_tests_passed'
assert receipt['owned_process'] is None and receipt['source_inputs_unchanged'] and receipt['normal_mains_unchanged']
normal_path = Path(receipt['normal_qualification']).resolve(strict=True)
assert normal_path.is_relative_to(APP) and sha(normal_path) == receipt['normal_qualification_sha256']
normal = read(normal_path)
scope = read(normal_path.parent/'scoped-qualified.json')
pins = read(normal_path.parent/'source-inputs.json')
assert normal['status'] == 'qualified_unique_entry_floor_port_normal_builds'
assert scope['status'] == 'qualified_floor_port_Debug120_Release120_and_normal_mains'
assert len(pins) == receipt['source_files'] == 1005
assert sha(normal_path.parent/'source-inputs.json') == receipt['source_manifest_sha256'] == normal['source_manifest_sha256']
for name,value in pins.items():assert sha(ROOT/name) == value,name
archive = read(OUT/'archive-readback.json')
assert archive['archive_readback_verified'] and archive['status'] == receipt['status']
assert sha(OUT/'raw-evidence.zip') == archive['archive_sha256']
with zipfile.ZipFile(OUT/'raw-evidence.zip') as z:
    assert set(z.namelist()) == set(archive['files'])
    for name,value in archive['files'].items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == sha(OUT/name) == value,name
tests = {}
for profile in ['Debug','Release']:
    folder = OUT/profile
    invocation = read(folder/'invocation.json')
    qpath = Path(invocation['original_profile_qualification']).resolve(strict=True)
    assert qpath.is_relative_to(APP) and sha(qpath) == invocation['original_profile_qualification_sha256']
    q = read(qpath)
    assert q['profile'] == profile and q['source_manifest_sha256'] == receipt['source_manifest_sha256']
    product = read(qpath.parent/'products.json')['lito_modern']
    assert sha(product['path']) == product['sha256'] == invocation['test_EXE_sha256'] == receipt['results'][profile]['test_EXE_sha256']
    assert invocation['arguments'] == [product['path'],'world_scene::tests::','--ignored','--test-threads=1']
    assert invocation['capture_and_reference_write_environment_disabled']
    assert invocation['read_only_resource_root'] == r'D:\lineage開發\0-客戶端'
    log = (folder/'stdout.log').read_text(encoding='utf-8-sig')
    entries = re.findall(r'^test (\S+) \.\.\. (\S+)',log,re.M)
    assert len(entries) == 6 and all(result == 'ok' for name,result in entries)
    assert {name for name,result in entries} == set(scope['ignored_test_names'][profile]) == set(invocation['expected_test_names'])
    assert re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;',log) == [('ok','6','0','0','0')]
    assert receipt['results'][profile]['exit_code'] == 0
    assert sha(normal['profiles'][profile]['path']) == normal['profiles'][profile]['sha256'] == receipt['normal_mains'][profile]
    tests[profile] = {'passed':6,'failed':0,'ignored':0,'actual_test_executables':1,
                      'test_EXE_sha256':product['sha256'],'test_names':[name for name,result in entries]}
result = {'status':'independently_qualified_both_profiles_six_real_resource_scene_optins',
          'source_files':1005,'source_manifest_sha256':receipt['source_manifest_sha256'],
          'normal_mains':receipt['normal_mains'],'profiles':tests,'raw_archive_sha256':archive['archive_sha256'],
          'raw_archive_readback_verified':True,'default_scope_still_120_passed_6_ignored_per_profile':True,
          'full_library_regression_verified':False,'normal_GUI_verified':False,'whole_goal_complete':False}
dest = OUT/'qualified.json'
assert not dest.exists()
dest.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert read(dest) == result
print(json.dumps({'qualification':str(dest),'status':result['status'],'passed_per_profile':6},ensure_ascii=False))
