"""Bind actual lighting test products and unique-entry mains to GUI evidence."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def load_scope(normal_path,rust_path,profile):
    normal_path=Path(normal_path).resolve(strict=True);rust_path=Path(rust_path).resolve(strict=True)
    for p in [normal_path,rust_path]:assert p.is_relative_to(ROOT/'驗證紀錄/current/native-lighting-candidate')
    normal=read(normal_path);pins=read(normal_path.parent/'source-inputs.json');scope=read(rust_path)
    floor=normal['status']=='qualified_unique_entry_floor_port_normal_builds'
    port=normal['status']=='qualified_unique_entry_water_port_normal_builds'
    water=floor or port or normal['status']=='qualified_unique_entry_water_source_normal_builds'
    assert water or normal['status']=='qualified_unique_entry_lighting_normal_builds'
    assert normal['features']==[] and len(pins)==normal['source_files']==(1005 if floor else 1002 if port else 999 if water else 994)
    assert normal['source_manifest_sha256']==sha(normal_path.parent/'source-inputs.json')
    assert scope['status']==('qualified_floor_port_Debug120_Release120_and_normal_mains' if floor else 'qualified_water_port_Debug92_Release92_and_normal_mains' if port else 'qualified_water_source_producer_Debug81_Release65_and_normal_mains' if water else 'qualified_applied_lighting_tests_both_profiles')
    assert scope['source_manifest_sha256']==normal['source_manifest_sha256']
    expected={'passed':120,'failed':0,'ignored':6,'actual_test_executables':19} if floor else {'passed':92,'failed':0,'ignored':0,'actual_test_executables':18} if port else {'passed':81 if profile=='Debug' else 65,'failed':0,'ignored':0,
              'actual_test_executables':17 if profile=='Debug' else 12} if water else {'passed':43,'failed':0,'ignored':0}
    assert scope['counts'][profile]==expected
    if floor:
        assert scope['integration_targets']==18 and scope['runtime_groups_per_profile']==20
        assert scope['selected_library_groups']==['world_raster::tests::','world_scene::tests::']
        assert len(scope['ignored_test_names'][profile])==6 and scope['floor_gate_only']
        assert scope['map303_overlay_and_background_complete'] is False
    if not water:assert scope['test_products_per_profile']==10
    assert scope['all_archives_readback_verified']
    assert not scope['full_default_regression_verified'] and not scope['full_library_regression_verified']
    assert not scope['full_current_TCP_regression_verified']
    assert scope['normal_mains'][profile]==normal['profiles'][profile]['sha256']
    main=normal['profiles'][profile]
    assert main['features']==[] and main['unique_entry'] and main['machine']=='AMD64' and main['optional_header']=='PE32+'
    assert sha(main['path'])==main['sha256']
    for run in scope['runs'][profile]:
        directory=Path(run['path']).resolve(strict=True)
        assert directory.is_relative_to(ROOT/'驗證紀錄/current/native-lighting-candidate')
        assert sha(directory/'receipt.json')==run['receipt_sha256']
        assert sha(directory/'compiled-source.zip')==run['compiled_source_sha256']
        assert sha(directory/'test-products.zip')==run['test_products_sha256']
        if floor:assert sha(directory/'profile-qualified.json')==run['profile_qualification_sha256']
    for name,h in pins.items():assert sha(ROOT/name)==h,name
    # Compile fixtures belong to the complete, version-specific manifest.
    # This final path names current lighting scope, not prior formal sessions.
    return normal,pins,scope,{},rust_path

if __name__=='__main__':
    import argparse
    ap=argparse.ArgumentParser();ap.add_argument('normal',type=Path);ap.add_argument('scope',type=Path);a=ap.parse_args()
    for profile in ['Debug','Release']:normal,pins,scope,_,_=load_scope(a.normal,a.scope,profile)
    print(json.dumps({'status':'exact_lighting_GUI_prerequisites_verified','compiled_input_files':len(pins),
                      'scoped_tests':scope['counts'],'whole_goal_complete':False}))
