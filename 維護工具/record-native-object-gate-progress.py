"""Record current gate work without applying old product/GUI results to it."""
from pathlib import Path
from datetime import datetime,timezone
import json,sys
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
run=Path(read(APP/'object-gate-tests-current.json')['output']);assert run.is_relative_to(APP)
r=read(run/'receipt.json');session=int(sys.argv[1]);assert r['compiled_inputs']==1019
work={'status':'source1019_RLE_gate_header_and_hidden_repair_tests_running','compiled_inputs':1019,
    'run':str(run),'tool_session':session,'profile':r['profile'],'source_frozen_for_current_tests':True,
    'actual_run_status':r['status'],'owned_process':r['owned_process'],
    'review_boundary':'Exact TIL headers retained; raw object previews count as unqualified fragments; Hidden submissions are skipped before tone paint.',
    'failed_initial_run':str(APP/'object-gate-tests-20261016-130245-955826/failed-scope-readback.json'),
    'normal_mains_rebuilt':False,'current_GUI_verified':False,'whole_goal_complete':False}
debug_current=APP/'object-gate-Debug-tests-current.json'
debug_qualification=None
if debug_current.exists():
    debug_run=Path(read(debug_current)['output'])
    if (debug_run/'profile-qualified.json').exists():
        debug_qualification=read(debug_run/'profile-qualified.json')
        assert debug_qualification['source_files']==1019 and debug_qualification['passed']==135 and debug_qualification['failed']==0
        work.update(Debug_qualification=str(debug_run/'profile-qualified.json'),Debug_counts={'passed':135,'failed':0,'ignored':6,'actual_test_executables':23},
            Debug_terminal_session=58895,Debug_terminal_exit_code=0,source_manifest_sha256=debug_qualification['source_manifest_sha256'])
if r['profile']=='Release':
    assert debug_qualification
    work['status']='source1019_Debug135_0_6_verified_Release_and_unique_entry_builds_running'
next_task=('來源1019 pure Rust selector與RLE/floor write provenance正沿同'+str(session)+'驗22整合及兩library群，來源/對應QA凍結。'
    '首輪104整合過、raster10過1 Hidden tone失敗已封存；已修Hidden前置略過，保留TIL真header，raw物件明示unqualified fragments，不當完整背景gate。'
    '正在重跑，預期新增raw來源測試後135/0/6但未宣稱通過；後續Release、唯一入口正常兩主與當版TW14。'
    '原263423案selector、真IMG背景24原frame與1015兩版各12僅保留原範圍資格。'
    'map303原生目錄仍不存在，不增BASE map fallback；camera/viewport/clock、水下wire/GUI、OS輸入、多DPI、可攜穩定性與完整G01–G12待。登入器只讀、產物獨立。')
if r['profile']=='Release':
    next_task=('修正版source1019 Debug23實際EXE/24群135過0敗6既有opt-in忽略已獨立核對，同58895 exit0。'
        'Release及唯一入口兩正常主正沿同'+str(session)+'執行，須等該handle terminal，來源與相關QA保持凍結。'
        '完成後先qualify-native-object-gate-tests.py核對Release，再qualify-native-object-gate-normal.py以ReleaseRun和DebugRun兩個參數核對兩正常主；'
        '接著用已準備object-gate GUI工具做當版TW14及獨立cohort/raw核對。不得移記1005 GUI。'
        'RLE物件typed路由/真檔頭/覆蓋已接；raw物件明示unqualified fragments、完整背景仍未接，不能用預覽替原RLE。'
        '首輪104整合過、raster10過1 Hidden tone失敗已封存，修正版不降低條件。原生303缺目錄、camera/clock/水下/OS輸入/多DPI/可攜穩定性及G01–G12仍待，登入器只讀。')
for n in ['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json']:
    p=ROOT/n;v=read(p)
    v.setdefault('retained_background_bank_source1015',v.get('native_background_product_current_work'))
    v.update(native_object_gate_product_current_work=work,next_task=next_task,current_working_tree_verified=False,
        working_tree_modified_since_current_normal_qualification=True,whole_goal_complete=False)
    v['current_product_validation']={'status':work['status'],'source_files':1019,'current_test_run':str(run),
        'normal_mains_rebuilt':False,'current_TW14_GUI_verified':False,'whole_goal_complete':False}
    if debug_qualification:v['current_product_validation'].update(Debug_qualification=work['Debug_qualification'],defined_Debug_scope=work['Debug_counts'])
    a=v.get('working_tree_native_lighting_application',{});a.update(status=work['status'],compiled_inputs=1019,current_test_session=session,
        source_frozen_for_current_tests=True,current_test_terminal_exit_code=None,current_Debug_qualification=None,
        current_Release_qualification=None,normal_mains_rebuilt_for_background_port=False,GUI='retained1005_not_current1019')
    if debug_qualification:a.update(current_Debug_qualification=work['Debug_qualification'])
    v['working_tree_native_lighting_application']=a
    if 'current_product_identity' in v:
        v['current_product_identity']['state']='last_qualified_source1005_normal_and_TW14_retained_working_source1019_tests_running'
    if 'source_edits_frozen_until_Cargo_terminal' in v:v['source_edits_frozen_until_Cargo_terminal']=True
    if n.endswith('continuation.json'):v['next']=next_task
    save(p,v);assert read(p)==v
save(APP/'object-gate-product-current.json',work)
save(APP/('object-gate-'+r['profile']+'-tests-current.json'),{'output':str(run)})
print(json.dumps({'status':work['status'],'source_files':1019,'run':str(run),'profile':r['profile']},ensure_ascii=False))
