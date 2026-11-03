"""Publish only the independently read back source1019 GUI scope."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL = APP / 'object-gate-tests-20261016-132623-487623/normal-qualified'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
encode = lambda v: json.dumps(v, ensure_ascii=False, indent=2) + '\n'
g = read(NORMAL / 'GUI-current.json')
o = read(NORMAL / 'GUI-completion-observation.json')
p = read(NORMAL / 'GUI-process-completion-observation.json')
n = read(NORMAL / 'qualified.json')
pins = read(NORMAL / 'source-inputs.json')
assert o['status'] == 'independently_readback_verified_current1019_TW14_GUI_and_terminal_process'
assert g['status'] == 'qualified_current_object_gate_TW14_GUI_cases'
assert not g['source_and_corresponding_tests_frozen']
assert p['session_id'] == o['session_id'] == 12012 and p['terminal_exit_code'] == 0
assert p['actual_terminal_tool_observed'] and p['original_pipeline_identity_absent'] and p['owned_workloads'] == []
assert len(pins) == o['source_files'] == 1019
for name, h in pins.items():
    assert sha(ROOT / name) == h, name
for profile in ['Debug', 'Release']:
    main = n['profiles'][profile]
    assert sha(Path(main['path'])) == main['sha256'] == o['normal_mains'][profile]
    visual = read(NORMAL / ('GUI-visual-' + profile + '.json'))
    assert visual['source_files'] == 1019 and visual['main_sha256'] == main['sha256']
    assert sha(Path(visual['image'])) == visual['image_sha256']
cohort = Path(o['cohort'])
assert sha(cohort / 'qualified.json') == o['qualification_sha256']
assert o['counts'] == dict(scenarios=14, actual_formal_sessions=112, LOGIN_sessions=56,
    GAME_sessions=56, exact_C_MOVE=220, extra_C_MOVE=0, original_world_PNGs=80,
    original_glyph_matches=56, stale_GUI_notifications=128)

tracking = ['工作佇列.json', '監製進度.json', '驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json']
docs = ['README.md', 'docs/ARCHITECTURE.zh-TW.md', 'docs/HISTORY.md', 'docs/NATIVE-FLOOR-GATE.zh-TW.md']
archive = NORMAL / 'progress-before-GUI-completion.zip'
before = {name: (ROOT / name).read_bytes() for name in tracking + docs}
if not archive.exists():
    with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
        for name, content in before.items():
            z.writestr(name, content)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist()) == set(before)
    for name, content in before.items():
        assert z.read(name) == content

scope = dict(o, evidence=str(cohort / 'qualified.json'), completion_observation=str(NORMAL / 'GUI-completion-observation.json'),
    physical_input_verified=False, multi_DPI_verified=False, same_endpoint_verified=False,
    whole_G06_complete=False, whole_goal_complete=False)
status = 'source1019_both135_and_normal_mains_and_TW14_GUI_independently_verified'
next_task = ('source1019正常05F325C8/CD4F29A8及兩版限定135/0/6、TW+extended十四案已獨立讀回；'
    '原12012實際exit0、PID121588原身份及全部自有workload已退出，850份raw ZIP及兩版目視紀錄已核對。'
    '下一步用prepared_object_gate_default108的runner依序執行Debug與Release，各108整合目標加main；'
    '須以各實際退出、原產物與log獨立qualify，未跑前不記完整default通過。'
    '整庫/TCP、同端點、實體輸入/多DPI、raw物件完整gate、原map303資源/背景camera/clock、可攜及穩定性與完整G01–G12仍待。登入器保持唯讀。')
outputs = {}
for name in tracking:
    v = json.loads(before[name].decode('utf-8-sig'))
    work = v['native_object_gate_product_current_work']
    work.update(status=status, source_frozen_for_current_GUI=False, current_GUI_verified=True,
        GUI_terminal_exit_code=0, GUI_evidence=str(cohort / 'qualified.json'),
        GUI_completion_observation=str(NORMAL / 'GUI-completion-observation.json'), GUI_counts=o['counts'],
        GUI_raw_files=o['raw_files'], GUI_raw_archive_sha256=o['raw_archive_sha256'])
    v['current_product_identity'].update(state='qualified_source1019_normal_mains_and_defined_TW14_GUI',
        working_tree_matches_recorded_source=True, whole_goal_complete=False)
    v['current_product_validation'].update(status=status, current_TW14_GUI_verified=True,
        full_default_library_TCP_regression_verified=False, whole_goal_complete=False)
    v.setdefault('retained_current14_GUI_before_object_gate', v.get('current14_teleport_GUI'))
    v.setdefault('retained_TW14_qualified_before_object_gate', v.get('current_TW14_GUI_qualified'))
    v.update(current14_teleport_GUI=scope, current_TW14_GUI_qualified=scope,
        current_object_gate_TW14_GUI=scope, current_GUI_all14_complete=True,
        source_freeze_for_active_GUI=False, pending_current_GUI_session=None,
        completed_current_GUI_session=12012, current_GUI_cohort=str(cohort),
        next_task=next_task, current_working_tree_verified=False,
        working_tree_modified_since_current_normal_qualification=False, whole_goal_complete=False,
        last_goal_turn_classification='progress', last_goal_turn_evidence=scope)
    a = v['working_tree_native_lighting_application']
    a.update(status=status, GUI='source1019_defined_TW14_independently_verified',
        GUI_terminal_exit_code=0, GUI_source_freeze_released=True, GUI_evidence=str(cohort / 'qualified.json'),
        GUI_raw_files=o['raw_files'], GUI_raw_archive_sha256=o['raw_archive_sha256'],
        GUI_counts=o['counts'], GUI_completion_observation=str(NORMAL / 'GUI-completion-observation.json'))
    if 'source_edits_frozen_until_Cargo_terminal' in v:
        v['source_edits_frozen_until_Cargo_terminal'] = False
    if name.endswith('continuation.json'):
        v['next'] = next_task
    for item in v.get('items', []):
        if item.get('id') == 'G06-room-service-branches':
            item['state'] = 'source1019_TW14_defined_teleport_recovery_verified_other_service_branches_pending'
            item['current1019_G06_3d_defined_scope'] = scope
            item['complete'] = False
            for check in item.get('checks', []):
                if check.get('id') == 'G06.3':
                    check['remaining'] = [
                        '普通世界／其他服務交接及同端點；跨端點與六種恢復的192DPI控件訊息子集已驗，實體輸入／多DPI仍待'
                        if x == '普通世界／其他服務交接、跨端點取消／過期／同端點及GUI' else x
                        for x in check.get('remaining', [])]
    outputs[ROOT / name] = encode(v)
outputs[APP / 'object-gate-normal-current.json'] = encode(work)

rel = (cohort / 'qualified.json').relative_to(ROOT).as_posix()
normal_rel = (NORMAL / 'GUI-completion-observation.json').relative_to(ROOT).as_posix()
paragraph = ('2026-10-16來源1019：唯一入口正常Debug **05F325C8**／Release **CD4F29A8**，'
    '各23實際x64測試EXE、24群135過0敗6既有opt-in忽略已獨立核對。當版TW＋extended GUI十四案全部完成：'
    '112實際sessions（56 LOGIN／56 GAME）、220次移動零多送、80原世界PNG、56原glyph及128舊通知；'
    '850份raw封存讀回，兩版新戰場PNG已目視。原12012實際exit0，原程序與自有workload均已退出。'
    '僅192DPI私有桌面控件訊息；實體輸入／多DPI、同端點、最新完整default/lib/TCP與完整G01–G12仍待。'
    'floor及普通RLE物件寫入來源已接Raster，raw物件預覽仍未具完整coverage資格；map303背景、viewport及clock尚未整合。')
text = before['README.md'].decode('utf-8-sig').replace('\r\n', '\n')
parts = text.split('\n\n')
assert parts[1].startswith('2026-10-16來源1019：')
parts[1] = paragraph + f' 見[十四案資格]({rel})與[終止後獨立讀回]({normal_rel})。'
if parts[2].startswith('2026-10-16來源1019進行中：'):
    del parts[2]
parts.insert(2, '下一步執行本版108個預設整合目標與main unit的雙版本回歸；目前僅準備好腳本，尚無結果。下文各階段收據按其所列版本保留，不移記為本版驗收。')
outputs[ROOT / 'README.md'] = '\n\n'.join(parts)
for name in docs[1:]:
    outputs[ROOT / name] = before[name].decode('utf-8-sig').rstrip() + '\n\n## 2026-10-16 source1019 TW14完成及範圍\n\n' + paragraph + '\n\n' + f'[十四案資格](../{rel})；[終止後獨立讀回](../{normal_rel})。完整default108＋main雙版本回歸已準備但未執行；整庫與完整TCP不包含在本次資格。\n'
for path, content in outputs.items():
    content = content.replace('\r\n', '\n')
    path.write_text(content, encoding='utf-8', newline='\n')
    assert path.read_text(encoding='utf-8') == content
receipt = {'status': status, 'recorded_utc': datetime.now(timezone.utc).isoformat(),
    'GUI_completion_sha256': sha(NORMAL / 'GUI-completion-observation.json'),
    'before_archive': str(archive), 'before_archive_sha256': sha(archive),
    'outputs': {str(path): sha(path) for path in outputs}, 'whole_goal_complete': False}
(NORMAL / 'progress-GUI-completion.json').write_text(encode(receipt), encoding='utf-8')
print(encode({'status': status, 'raw_files': o['raw_files'], 'counts': o['counts'], 'whole_goal_complete': False}))
