"""Promote only independently qualified source1019 mains; GUI remains a separate scope."""
from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'object-gate-tests-20261016-132623-487623/normal-qualified'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
n=read(NORMAL/'qualified.json');s=read(NORMAL/'scoped-qualified.json');pins=read(NORMAL/'source-inputs.json')
assert n['status']=='qualified_unique_entry_object_gate_normal_builds' and len(pins)==1019
assert s['status']=='qualified_object_gate_Debug135_Release135_and_normal_mains'
for name,h in pins.items():assert sha(ROOT/name)==h,name
for p in ['Debug','Release']:
    assert s['counts'][p]=={'passed':135,'failed':0,'ignored':6,'actual_test_executables':23}
    assert sha(Path(n['profiles'][p]['path']))==s['normal_mains'][p]
session=int(sys.argv[1]) if len(sys.argv)>1 else None
g=read(NORMAL/'GUI-current.json') if (NORMAL/'GUI-current.json').exists() else None
if session:assert g and g['source_and_corresponding_tests_frozen']
status='source1019_both135_0_6_and_normal_mains_qualified_GUI_running' if g else 'source1019_both135_0_6_and_normal_mains_qualified_GUI_preparing'
work={'status':status,'compiled_inputs':1019,'source_manifest_sha256':n['source_manifest_sha256'],
    'Rust_qualification':str(NORMAL/'scoped-qualified.json'),'normal_qualification':str(NORMAL/'qualified.json'),
    'normal_mains':s['normal_mains'],'counts':s['counts'],'Rust_terminal_sessions':{'Debug':58895,'Release_and_normal':94643},
    'Rust_terminal_exit_codes':{'Debug':0,'Release_and_normal':0},'source_frozen_for_current_GUI':True,
    'GUI_session':session,'GUI_state':str(NORMAL/'GUI-current.json') if g else None,
    'GUI_cohort':g['cohort'] if g else None,'GUI_pipeline_pid':g['pipeline_pid'] if g else None,
    'current_GUI_verified':False,'supported_RLE_gate_integrated':True,'raw_object_coverage_complete':False,
    'background_renderer_integrated':False,'whole_goal_complete':False}
next_task=('source1019 Debug/Release各23實際EXE、24群135過0敗6既有opt-in忽略與唯一入口正常05F325C8/CD4F29A8已獨立核對；同94643已實際exit0。'
    +('當版TW14 GUI正沿同'+str(session)+'執行，須續等原handle並檢查normal-qualified/GUI-current.json，不重啟、不改來源或相關QA。' if session else '當版TW14 GUI即將啟動，使用run-native-object-gate-TW14.ps1及已核對normal/scoped-qualified；來源與相關QA凍結。')
    +'完成後以observe-native-object-gate-GUI-terminal.py記錄實際tool退出及CIM，再以qualify-native-object-gate-GUI-completion.py讀回cohort/raw，並目視兩版新PNG。'
    'raw物件仍unqualified、原生303缺目錄，不加BASE map fallback；背景camera/clock、水下wire/GUI、OS輸入/多DPI、可攜穩定性與完整G01–G12仍待。登入器只讀。')
for filename in ['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json']:
    p=ROOT/filename;v=read(p)
    v.setdefault('retained_normal_identity_before_object_gate',v.get('current_product_identity'))
    a=v.get('working_tree_native_lighting_application',{})
    v.setdefault('retained_GUI_before_object_gate',{k:a[k] for k in a if k=='GUI' or k.startswith('GUI_')})
    v.update(native_object_gate_product_current_work=work,next_task=next_task,current_working_tree_verified=False,
        working_tree_modified_since_current_normal_qualification=False,whole_goal_complete=False)
    v['current_product_identity']={'state':'qualified_source1019_normal_mains_GUI_pending','source_files':1019,
        'source_manifest_sha256':n['source_manifest_sha256'],'normal_mains':s['normal_mains'],
        'qualification':str(NORMAL/'qualified.json'),'working_tree_matches_recorded_source':True,'whole_goal_complete':False}
    v['current_product_validation']={'status':status,'source_files':1019,'defined_Rust_scope':s['counts'],
        'normal_mains_qualified':True,'normal_qualification':str(NORMAL/'qualified.json'),
        'current_TW14_GUI_verified':False,'full_default_library_TCP_regression_verified':False,'whole_goal_complete':False}
    a.update(status=status,compiled_inputs=1019,current_test_session=None,source_frozen_for_current_tests=False,
        current_test_terminal_exit_code=0,normal_mains_rebuilt=True,normal_mains_rebuilt_for_background_port=True,
        normal_qualification=str(NORMAL/'qualified.json'),scoped_qualification=str(NORMAL/'scoped-qualified.json'),
        GUI='source1019_running' if g else 'source1019_preparing',GUI_session=session,
        GUI_pipeline_pid=g['pipeline_pid'] if g else None,GUI_creation_utc=g['pipeline_creation_utc'] if g else None,
        GUI_terminal_exit_code=None,GUI_source_freeze_released=False,GUI_evidence=None,GUI_raw_files=None,
        GUI_raw_archive_sha256=None,GUI_source_files=1019 if g else None)
    v['working_tree_native_lighting_application']=a
    if 'source_edits_frozen_until_Cargo_terminal' in v:v['source_edits_frozen_until_Cargo_terminal']=True
    if filename.endswith('continuation.json'):v['next']=next_task
    save(p,v);assert read(p)==v
save(APP/'object-gate-normal-current.json',work)
print(json.dumps({'status':status,'normal_mains':s['normal_mains'],'GUI_session':session},ensure_ascii=False))
