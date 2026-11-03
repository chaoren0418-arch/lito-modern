"""Exact environment scope evidence for GUI; no full-default assertion."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest().upper()

def load_scope(normal_path, rust_path, profile):
    normal_path=Path(normal_path).resolve(strict=True)
    rust_path=Path(rust_path).resolve(strict=True)
    assert normal_path.is_relative_to(ROOT/'驗證紀錄/current/native-environment')
    assert rust_path.is_relative_to(ROOT/'驗證紀錄/current/native-environment')
    normal=read(normal_path);pins=read(normal_path.parent/'source-inputs.json');rust=read(rust_path)
    assert normal['features']==[] and len(pins)==normal['source_files']==978
    assert normal['source_manifest_sha256']==sha(normal_path.parent/'source-inputs.json')
    assert rust['status']=='independently_qualified_GAME_environment_and_nine_related_targets'
    assert rust['source_files']==978 and rust['fixture_derivation_rechecked']
    assert rust['source_manifest_sha256']==normal['source_manifest_sha256']
    assert rust['counts'][profile]=={'passed':48,'failed':0,'ignored':3}
    assert rust['test_products_per_profile']==9 and rust['all_archives_readback_verified']
    assert not rust['full_default_regression_verified'] and not rust['full_library_regression_verified']
    assert not rust['full_current_TCP_regression_verified']
    assert sha(ROOT/'維護工具/qualify-native-environment-scoped.py')==rust['qualifier_sha256']
    run=Path(rust['run']).resolve(strict=True)
    assert run.is_relative_to(ROOT/'驗證紀錄/current/native-environment')
    assert sha(run/'qualified.json')==rust['run_qualification_sha256']
    scope=read(run/'qualified.json');receipt=read(run/'receipt.json')
    assert scope['status']==receipt['status']=='completed_GAME_environment_related_scoped_checks_and_unique_entry_mains'
    assert not receipt['active_pid'] and receipt['phase'] is None and receipt['features']==[]
    assert scope['source_manifest_sha256']==rust['source_manifest_sha256']
    assert scope['normal_mains'][profile]==normal['profiles'][profile]
    assert rust['normal_mains'][profile]==normal['profiles'][profile]['sha256']
    checks=scope['profiles'][profile]
    assert len(checks['tests'])==9
    for name,item in checks['tests'].items():
        assert item['failed']==0 and receipt[profile+'-'+name+'_exit_code']==0
        assert checks['products']['file_pins'][name+'.exe']==item['exe_sha256']
        for suffix in ['.stdout','.stderr','-command.json']:
            filename=profile+'-'+name+suffix
            assert sha(run/filename)==scope['raw_archive']['file_pins'][filename]
    formal_path=run/'formal-independent-qualified.json';formal=read(formal_path)
    assert formal['status']=='independently_qualified_both_profiles_actual_C02_environment_scope'
    assert formal['source_manifest_sha256']==rust['source_manifest_sha256']
    assert formal['normal_mains']==rust['normal_mains'] and formal['actual_formal_sessions']==8
    assert formal['all_raw_archives_readback_verified']
    assert sha(ROOT/'維護工具/qualify-native-environment-formal.py')==formal['qualifier_sha256']
    fixtures=read(run/'fixture-inputs.json')
    assert fixtures==rust['fixture_inputs'] and len(fixtures)==4 and not set(fixtures)&set(pins)
    for name,h in {**pins,**fixtures}.items():assert sha(ROOT/name)==h,name
    return normal,pins,rust,fixtures,formal_path

if __name__=='__main__':
    import argparse
    ap=argparse.ArgumentParser();ap.add_argument('normal',type=Path);ap.add_argument('scope',type=Path);a=ap.parse_args()
    for profile in ['Debug','Release']:
        n,p,s,f,actual=load_scope(a.normal,a.scope,profile)
        assert sha(n['profiles'][profile]['path'])==n['profiles'][profile]['sha256']
    print(json.dumps({'status':'exact_environment_GUI_prerequisites_verified','source_files':978,
        'additional_compile_fixtures':4,'scoped_tests_per_profile':48,'actual_prior_C02_sessions':8,
        'full_default_regression_verified':False,'whole_goal_complete':False}))
