"""Qualify five real scene opt-ins and preserve the sixth catalog failure."""
from pathlib import Path
import hashlib, json, re, sys, zipfile

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL, DEBUG, RELEASE = [Path(x).resolve(strict=True) for x in sys.argv[1:4]]
assert all(p.is_relative_to(APP) for p in [NORMAL,DEBUG,RELEASE])
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()

normal = read(NORMAL/'qualified.json')
scope = read(NORMAL/'scoped-qualified.json')
pins = read(NORMAL/'source-inputs.json')
assert normal['status'] == 'qualified_unique_entry_floor_port_normal_builds' and len(pins) == 1005
assert scope['status'] == 'qualified_floor_port_Debug120_Release120_and_normal_mains'
for name,value in pins.items():assert sha(ROOT/name) == value,name
root = Path(r'D:\lineage開發\0-客戶端')
assert not (root/'TW13081901.pak').exists() and (root/'launcher/TW13081901.pak').is_file()
failed_name = 'modern_lito::world_scene::tests::real_disintegrate_layers_share_world_depth_and_survive_target_removal'
profiles = {}
for profile,folder,log_path in [('Debug',DEBUG,DEBUG/'Debug/stdout.log'),('Release',RELEASE,RELEASE/'stdout.log')]:
    a = read(folder/'archive-readback.json')
    assert a['archive_readback_verified'] and sha(folder/'raw-evidence.zip') == a['archive_sha256']
    with zipfile.ZipFile(folder/'raw-evidence.zip') as z:
        assert set(z.namelist()) == set(a['files'])
        for name,value in a['files'].items():assert hashlib.sha256(z.read(name)).hexdigest().upper() == sha(folder/name) == value,name
    log = log_path.read_text(encoding='utf-8-sig')
    entries = re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',log,re.M)
    assert len(entries) == 6 and {n for n,v in entries} == set(scope['ignored_test_names'][profile])
    assert [(n,v) for n,v in entries if v == 'FAILED'] == [(failed_name,'FAILED')]
    assert re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;',log) == [('FAILED','5','1','0','0')]
    assert 'world_scene.rs:4061:68' in log and '(os error 2)' in log and 'shared resource I/O' in log
    run = Path(scope['runs'][profile][0]['path'])
    product = read(run/'products.json')['lito_modern']
    assert sha(product['path']) == product['sha256']
    invocation = read(DEBUG/'Debug/invocation.json') if profile == 'Debug' else read(RELEASE/'invocation.json')
    assert invocation['test_EXE_sha256'] == product['sha256']
    assert invocation['arguments'] == [product['path'],'world_scene::tests::','--ignored','--test-threads=1']
    if profile == 'Debug':
        r = read(DEBUG/'receipt.json')
        assert r['status'] == 'failed_preserved' and r['owned_process'] is None and r['results']['Debug']['exit_code'] == 101
    else:
        assert invocation['actual_test_exit_code'] == 101 and invocation['owned_pid'] is None
        assert invocation['root_catalog_absent'] and sha(root/'launcher/TW13081901.pak') == invocation['launcher_catalog_sha256']
    assert sha(normal['profiles'][profile]['path']) == normal['profiles'][profile]['sha256']
    profiles[profile] = {'passed':5,'failed':1,'ignored':0,'actual_test_executables':1,
        'test_EXE_sha256':product['sha256'],'passed_test_names':[n for n,v in entries if v == 'ok'],
        'failed_test_name':failed_name,'failed_before_scene_render':True,'raw_archive_sha256':a['archive_sha256']}
scene = (ROOT/'LiTo381-Modern/src/modern_lito/world_scene.rs').read_text(encoding='utf-8-sig').splitlines()
assert 'DisintegrateAssets::load(&root, &cancel).unwrap()' in scene[4060]
assets = (ROOT/'LiTo381-Modern/src/modern_lito/disintegrate_assets.rs').read_text(encoding='utf-8-sig')
assert 'SharedTwActionCatalog::load_selected(root, &IDS.into(), cancel)?' in assets
resources = (ROOT/'LiTo381-Modern/src/modern_lito/resources.rs').read_text(encoding='utf-8-sig')
assert re.search(r'fn read_shared_tw_action_pack\([^}]+?root,\s*&\[\],\s*"TW13081901.pak"',resources,re.S)
result = {'status':'independently_verified_five_scene_tests_per_profile_and_one_missing_root_catalog_boundary',
    'source_files':1005,'source_manifest_sha256':sha(NORMAL/'source-inputs.json'),
    'normal_mains':{p:m['sha256'] for p,m in normal['profiles'].items()},'profiles':profiles,
    'all_six_optins_passed':False,'failed_case_qualified':False,
    'catalog_boundary':'Disintegrate legacy loader explicitly requires root/TW13081901.pak; current resources only have launcher/TW13081901.pak. No source substitution or fallback applied.',
    'normal_source_policy_fixed':False,'raw_archives_readback_verified':True,
    'full_library_regression_verified':False,'normal_GUI_verified':False,'whole_goal_complete':False,
    'qualifier_sha256':sha(Path(__file__))}
dest = RELEASE/'five-scene-and-catalog-boundary-qualified.json'
assert not dest.exists()
dest.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert read(dest) == result
print(json.dumps({'path':str(dest),'five_passed_each_profile':True,'sixth_missing_catalog_not_PASS':True},ensure_ascii=False))
