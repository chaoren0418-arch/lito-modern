"""Exact source1019 object-gate Rust/main binding for the existing GUI controller."""
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
    assert normal['status']=='qualified_unique_entry_object_gate_normal_builds'
    assert normal['features']==[] and len(pins)==normal['source_files']==1019
    assert normal['source_manifest_sha256']==sha(normal_path.parent/'source-inputs.json')
    assert scope['status']=='qualified_object_gate_Debug135_Release135_and_normal_mains'
    assert scope['source_manifest_sha256']==normal['source_manifest_sha256']
    assert scope['counts'][profile]=={'passed':135,'failed':0,'ignored':6,'actual_test_executables':23}
    assert scope['integration_targets']==22 and scope['runtime_groups_per_profile']==24
    assert scope['selected_library_groups']==['world_raster::tests::','world_scene::tests::']
    assert len(scope['ignored_test_names'][profile])==6 and not scope['floor_gate_only']
    assert scope['supported_first_pass_object_gate_integrated'] and scope['full_object_dispatch_original_cases']==263423
    assert not scope['map303_gate_complete'] and not scope['map303_overlay_and_background_complete']
    assert scope['all_archives_readback_verified']
    assert not scope['full_default_regression_verified'] and not scope['full_library_regression_verified'] and not scope['full_current_TCP_regression_verified']
    assert scope['normal_mains'][profile]==normal['profiles'][profile]['sha256']
    main=normal['profiles'][profile]
    assert main['features']==[] and main['unique_entry'] and main['machine']=='AMD64' and main['optional_header']=='PE32+'
    assert sha(main['path'])==main['sha256']
    for run in scope['runs'][profile]:
        directory=Path(run['path']).resolve(strict=True);assert directory.is_relative_to(ROOT/'驗證紀錄/current/native-lighting-candidate')
        for name,key in [('receipt.json','receipt_sha256'),('compiled-source.zip','compiled_source_sha256'),
            ('test-products.zip','test_products_sha256'),('profile-qualified.json','profile_qualification_sha256')]:
            assert sha(directory/name)==run[key]
    for name,h in pins.items():assert sha(ROOT/name)==h,name
    return normal,pins,scope,{},rust_path

if __name__=='__main__':
    import argparse
    ap=argparse.ArgumentParser();ap.add_argument('normal',type=Path);ap.add_argument('scope',type=Path);a=ap.parse_args()
    for profile in ['Debug','Release']:normal,pins,scope,_,_=load_scope(a.normal,a.scope,profile)
    print(json.dumps({'status':'exact_object_gate_GUI_prerequisites_verified','compiled_input_files':len(pins),
        'scoped_tests':scope['counts'],'whole_goal_complete':False}))
