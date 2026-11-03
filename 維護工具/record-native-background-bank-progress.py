"""Record source1015 scope, preserving old geometry and normal-main qualifications."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,sys
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
phase=sys.argv[1];assert phase in ['Debug','Release','qualified']
session=int(sys.argv[2]);profiles={}
for profile in ['Debug','Release']:
    current=APP/('background-bank-'+profile+'-tests-current.json')
    if not current.exists():continue
    run=Path(read(current)['path']);assert run.is_relative_to(APP)
    r=read(run/'receipt.json');assert r['compiled_inputs']==1015
    q=read(run/'qualified.json') if (run/'qualified.json').exists() else None
    if q:assert q['source_files']==1015 and q['passed']==12 and q['failed']==q['ignored']==0 and q['actual_test_executables']==4
    profiles[profile]={'run':str(run),'status':q['status'] if q else r['status'],
        'qualification':str(run/'qualified.json') if q else None,
        'counts':{'passed':12,'failed':0,'ignored':0,'actual_test_executables':4} if q else None,
        'source_manifest_sha256':q['source_manifest_sha256'] if q else None}
finished=phase=='qualified'
if finished:
    assert set(profiles)=={'Debug','Release'} and all(v['qualification'] for v in profiles.values())
    assert profiles['Debug']['source_manifest_sha256']==profiles['Release']['source_manifest_sha256']
    pins=read(Path(profiles['Debug']['run'])/'compiled-inputs.json')
    for n,h in pins.items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest().upper()==h,n
status='source1015_Debug_and_Release_each12_0_0_real_IMG_bank_and_core_verified_scene_pending' if finished else 'source1015_'+phase+'_bank_and_core_tests_running'
work={'status':status,'compiled_inputs':1015,'test_session':session,'terminal_exit_code':0 if finished else None,
    'source_frozen_for_tests':not finished,'profiles':profiles,'native_real_IMG_original_qualification':str(ROOT/'驗證紀錄/current/native-background-real-original/20261016-123005-086741/qualified.json'),
    'original_real_IMG_frames':24,'original_real_IMG_pixel_calls':132,'Rust_map303_real_IMG_frames_per_profile':12,
    'normal_mains_rebuilt_for_this_port':False,'background_renderer_integrated':False,'map303_overlay_gate_complete':False,
    'modern_viewport_adapter_verified':False,'current_GUI_verified':False,'whole_goal_complete':False}
if finished:
    dispatch=read(APP/'background-dispatch-qualified-current.json')
    assert dispatch['actual_cases']==263423 and dispatch['all_archives_readback_verified']
    work.update(test_sessions={'Debug':43971,'Release':85319},test_terminal_exit_codes={'Debug':0,'Release':0},
        original_dispatch_qualification=str(Path(dispatch['path'])/'qualified.json'),original_dispatch_cases=263423,
        background_clock_static_observation=str(Path(read(APP/'background-clock-lifecycle-current.json')['path'])/'observation.json'))
next_task=('來源1015真IMG背景bank與原geometry/pixel/floor已Debug/Release各四實際EXE12過0敗0忽略核對；' if finished else '來源1015真IMG背景bank與core正沿同'+str(session)+'驗'+phase+'，來源與對應QA凍結；')+(
    '原init/geometry/RGB555共4439bytes已配五真IMG隔離24完整平面/132calls核對，Rust依native優先/BASE Sprite缺項路由讀800/803並對map303十二整frame比對。'
    '原389byte物件分派已隔離263423案核對（全16bit flags 262144＋1279針對案），普通RLE才傳map303 gate低byte，roof權重/其他混色不寫此gate；完整scene gate仍未接。'
    '下一步移植typed route/provenance並核對actor/attachment/TIL producer，再接Raster與原half-cell/camera到modern viewport。回呼有取消/立即重啟路徑，只做static保留，不能逕用GAME歸零或elapsed補步。'
    'LiTo/LTmap/303缺目錄，BASE map/303的30S32不能當已證明fallback；原IMG路由與map路由分開。'
    '最新唯一入口正常主/GUI與廣域default/lib/TCP另待，1005正常AFF5AF96/C7F36CC4與TW14只留原版資格。'
    '水下wire/GUI、OS輸入/多DPI、可攜穩定性及完整G01–G12仍待；登入器只讀、產物獨立。')
targets=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json']
for n in targets:
    p=ROOT/n;v=read(p)
    v.setdefault('retained_background_geometry_source1012',v.get('native_background_product_current_work'))
    v.update(native_background_product_current_work=work,next_task=next_task,current_working_tree_verified=False,
        working_tree_modified_since_current_normal_qualification=True,whole_goal_complete=False)
    validation={'status':status,'source_files':1015,'profiles':profiles,'current_TW14_GUI_verified':False,
        'normal_mains_rebuilt':False,'background_renderer_integrated':False,'full_default_library_TCP_regression_verified':False,'whole_goal_complete':False}
    v['current_product_validation']=validation
    a=v.get('working_tree_native_lighting_application',{})
    a.update(status=status,compiled_inputs=1015,current_test_session=session,source_frozen_for_current_tests=not finished,
        current_test_terminal_exit_code=0 if finished else None,current_Debug_qualification=profiles.get('Debug',{}).get('qualification'),
        current_Release_qualification=profiles.get('Release',{}).get('qualification'),
        normal_mains_rebuilt_for_background_port=False,GUI='retained1005_not_current1015',
        actual_tests=str(APP/'background-bank-product-current.json'))
    v['working_tree_native_lighting_application']=a
    if 'current_product_identity' in v:v['current_product_identity']['state']='last_qualified_source1005_normal_mains_and_TW14_retained_working_source1015_'+('both_profiles_bank_core_verified' if finished else phase+'_tests_running')
    b=v.get('native_map303_background_current_work',{})
    b.update(status=status,real_IMG_original_composition_qualification=work['native_real_IMG_original_qualification'],
        original_geometry_and_pixels_with_real_IMG_verified=True,product_real_IMG_bank_verified=finished,
        product_bank_qualifications=profiles,background_renderer_integrated=False)
    if finished:
        b.update(background_native_execution_verified=True,background_real_pixel_kernel_executed=True,
            original_execution_scope='private original initializer/geometry/RGB555 with real warm IMG; original loader, scene gate, viewport and GUI unverified',
            Rust_pixel_core_Debug_qualification=profiles['Debug']['qualification'],
            product_geometry_qualification=profiles['Debug']['qualification'],
            original_dispatch_qualification=work['original_dispatch_qualification'],
            background_clock_static_observation=work['background_clock_static_observation'])
    v['native_map303_background_current_work']=b
    if 'source_edits_frozen_until_Cargo_terminal' in v:v['source_edits_frozen_until_Cargo_terminal']=not finished
    if n.endswith('continuation.json'):v['next']=next_task
    save(p,v);assert read(p)==v
save(APP/'background-bank-product-current.json',work)
print(json.dumps({'status':status,'profiles':profiles,'source_frozen':not finished},ensure_ascii=False))
