"""Record pending/qualified source1012 scope without borrowing old main/GUI results."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,sys
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
run=Path(read(APP/'background-geometry-tests-current.json')['path'])
assert run.is_relative_to(APP)
finished=sys.argv[1]=='qualified'
assert finished or sys.argv[1]=='pending'
receipt=read(run/'receipt.json')
q=read(run/'qualified.json') if finished else None
if finished:
    assert receipt['owned_process'] is None and q['source_files']==1012 and q['passed']==10
    assert q['failed']==q['ignored']==0 and q['actual_test_executables']==3
    assert q['all_archives_readback_verified']
    pins=read(run/'compiled-inputs.json')
    for n,h in pins.items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest().upper()==h,n
boundary=read(APP/'background-scene-boundary-current.json')
assert boundary['background_after_first_floor_and_object_pass'] and not boundary['BASE_map_fallback_established']
status='source1012_Debug10_0_0_background_geometry_pixels_floor_verified_scene_pending' if finished else 'source1012_geometry_pixels_floor_Debug_tests_running'
work={'status':status,'compiled_inputs':1012,'test_session':96668,'run':str(run),
    'terminal_exit_code':0 if finished else None,'source_frozen_for_tests':not finished,
    'Debug_qualification':str(run/'qualified.json') if finished else None,
    'Debug_counts':{'passed':10,'failed':0,'ignored':0,'actual_test_executables':3} if finished else None,
    'source_manifest_sha256':q['source_manifest_sha256'] if finished else None,
    'original_geometry_cases':2688,'original_geometry_calls':17472,'native_channel_words':135168,'native_pixel_cases':1584,
    'scene_boundary_observation':str(Path(boundary['path'])/'observation.json'),
    'normal_mains_rebuilt_for_this_port':False,'Release_tested':False,'background_renderer_integrated':False,
    'map303_overlay_gate_complete':False,'current_GUI_verified':False,'whole_goal_complete':False}
next_task=('來源1012背景geometry＋pixel＋floor已Debug三實際EXE10過0敗0忽略核對；' if finished else '來源1012背景geometry＋pixel＋floor正沿同96668執行Debug驗證，來源與對應QA凍結；')+(
    '原背景在第一輪地板及物件清單之後，撤回before-all-bodies假設；先驗各物件分派/gate provenance，再接真IMG800/803與原half-cell/camera到modern viewport。'
    '已觀察原map loader明示LiTo/LTMap s32失敗再seg，現有LiTo/LTmap/303缺目錄，BASE map/303的30S32不是已證明的fallback，不能自行替代或宣稱303正常GUI。'
    '真IMG native優先BASE Sprite路由與地圖路由分開。最新Release/唯一入口正常主/GUI及廣域default/lib/TCP另待，1005正常AFF5AF96/C7F36CC4與TW14僅保留原版資格。'
    '水下wire/GUI、OS輸入/多DPI、可攜穩定性與完整G01–G12仍待；重建版與登入器產物獨立，相鄰來源只讀。')
targets=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json']
for n in targets:
    p=ROOT/n;v=read(p)
    v.setdefault('retained_background_pixel_source1009',v.get('native_background_product_current_work'))
    v.update(native_background_product_current_work=work,next_task=next_task,current_working_tree_verified=False,
        working_tree_modified_since_current_normal_qualification=True,whole_goal_complete=False)
    v['current_product_validation']={'status':status,'source_files':1012,'defined_Debug_scope':work['Debug_counts'],
        'Debug_qualification':work['Debug_qualification'],'current_TW14_GUI_verified':False,'normal_mains_rebuilt':False,
        'Release_tested':False,'background_renderer_integrated':False,'full_default_library_TCP_regression_verified':False,'whole_goal_complete':False}
    a=v.get('working_tree_native_lighting_application',{})
    a.update(status=status,compiled_inputs=1012,current_test_session=96668,source_frozen_for_current_tests=not finished,
        current_test_terminal_exit_code=0 if finished else None,current_Debug_qualification=work['Debug_qualification'],
        normal_mains_rebuilt_for_background_port=False,current_Release_qualification=None,
        GUI='retained1005_not_current1012',actual_tests=work['Debug_qualification'])
    v['working_tree_native_lighting_application']=a
    if 'current_product_identity' in v:
        v['current_product_identity']['state']='last_qualified_source1005_normal_mains_and_TW14_retained_working_source1012_'+('Debug_geometry_pixels_floor_verified' if finished else 'tests_running')
    b=v.get('native_map303_background_current_work',{})
    b.update(status=status,product_geometry_core_Debug_verified=finished,product_geometry_qualification=work['Debug_qualification'],
        Rust_pixel_core_Debug_qualification=work['Debug_qualification'],
        background_after_first_floor_and_object_pass_static=True,before_all_bodies_assumption_valid=False,
        scene_boundary_observation=work['scene_boundary_observation'],native_map303_directory_exists=False,
        BASE_map303_s32_files=30,BASE_map_fallback_established=False,background_renderer_integrated=False)
    v['native_map303_background_current_work']=b
    if 'source_edits_frozen_until_Cargo_terminal' in v:v['source_edits_frozen_until_Cargo_terminal']=not finished
    if n.endswith('continuation.json'):v['next']=next_task
    save(p,v);assert read(p)==v
save(APP/'background-geometry-product-current.json',work)
print(json.dumps({'status':status,'trackers_updated':len(targets),'run':str(run),'source_frozen':not finished},ensure_ascii=False))
