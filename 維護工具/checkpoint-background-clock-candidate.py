"""Publish isolated background-clock evidence without changing product sources."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import zipfile

ROOT = Path(__file__).resolve().parents[1]
def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))
def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()
def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

native = ROOT/'驗證紀錄/current/native-background-offset-callback/20261016-233627-359668/qualified.json'
candidate = ROOT/'驗證紀錄/current/native-background-clock-rust-candidate/20261016-234332-623096/independent-readback.json'
queue_native = ROOT/'驗證紀錄/current/native-background-queue-lifecycle/20261017-000426-324007/qualified.json'
queue_rust = ROOT/'驗證紀錄/current/native-background-queue-rust-candidate/20261017-000455-725985/independent-readback.json'
binding_path = ROOT/'驗證紀錄/current/native-background-lifecycle-binding/20261017-000314-420982/observation.json'
resource_path = ROOT/'驗證紀錄/current/native-resource-optin/current1024-ten-bodies-20261017-001048-618593/qualified.json'
n, c = read(native), read(candidate)
qn, qr = read(queue_native), read(queue_rust)
assert qn['status'] == 'independently_qualified_original_background_queue_and_selected_lifecycle_slices'
assert qr['status'] == 'independently_read_back_unapplied_background_queue_candidate_against_original_lifecycle'
assert (qr['snapshots'], qr['queue_rows']) == (18, 90)
assert qr['distinct_ready_reuse_filter_executed'] and qr['ready_reuse_filter_removes_background']
assert qr['candidate_sha256'] == c['candidate_files']['native_water_clock.rs']
assert qr['native_qualification_sha256'] == sha(queue_native)
assert n['status'] == 'independently_qualified_original_background_offset_callback'
assert c['status'] == 'independently_read_back_unapplied_shared_background_clock_candidate'
assert n['cases'] == c['native_callback_cases'] == 57
assert len(c['synthetic_queue_scenarios']) == 7
assert n['observed_terminal_exit_code'] == c['observed_terminal_exit_code'] == 0
assert not c['candidate_applied_to_product'] and not c['automatic_scene_binding_added']
base = ROOT/'LiTo381-Modern/src/modern_lito/native_water_clock.rs'
assert sha(base) == c['candidate_files']['base-native_water_clock.rs']
cohort = ROOT/'驗證紀錄/current/native-default110/20261016-231456-842850/cohort.json'
live = read(cohort)
assert live['source_manifest_sha256'] == c['source_manifest_sha256']
assert live['source_and_corresponding_QA_frozen']
for name, digest in live['validation_pins'].items():
    assert sha(ROOT/'維護工具'/name) == digest
files = ['工作佇列.json', '監製進度.json',
         '驗證紀錄/current/native-teleport-wire/progress.json',
         '驗證紀錄/current/native-terrain-residence/continuation.json',
         'README.md', 'docs/HISTORY.md']
out = queue_rust.parent/('tracker-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {name: sha(ROOT/name) for name in files}
with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in files:
        archive.write(ROOT/name, name)
with zipfile.ZipFile(out/'before.zip') as archive:
    assert set(archive.namelist()) == set(files)
    for name, digest in before.items():
        assert hashlib.sha256(archive.read(name)).hexdigest().upper() == digest
evidence = dict(status=c['status'], qualification=str(candidate), sha256=sha(candidate),
    native_callback_qualification=str(native), native_callback_qualification_sha256=sha(native),
    native_callback_cases=57, synthetic_queue_scenarios=7, callback_delay_ms=64,
    callback_counter_bits=32, callback_owner=0, candidate_path='維護工具/candidates/native-background-clock/native_water_clock.rs',
    candidate_sha256=c['candidate_files']['native_water_clock.rs'],
    existing_water_green_and_map_methods_preserved=True,
    automatic_scene_binding_added=False, candidate_applied_to_product=False,
    native_scheduler_body_executed=False, real_clock_or_GUI_timing_verified=False,
    full_start_cancel_rearm_binding_verified=False, whole_goal_complete=False)
queue_evidence = dict(status=qr['status'], qualification=str(queue_rust), sha256=sha(queue_rust),
    native_qualification=str(queue_native), native_qualification_sha256=sha(queue_native),
    original_native_session_id=16155, original_Rust_exec_chunk_id='5ce4db', observed_terminal_exit_code=0,
    original_code_bytes=1336, original_functions_and_slices=15, appended_return_instructions=3,
    state_snapshots=18, queue_rows=90, complete_state_and_queue_fields_equal=True,
    native_scheduler_cancel_dispatch_and_filters_executed=True,
    D2_packet_table_and_selected_rearm_slice_verified=True,
    background_survives_generic_map_and_scene_filters=True,
    distinct_ready_reuse_filter_executed=True, ready_reuse_filter_removes_background=True,
    ready_filter_candidate_sha256=qr['ready_filter_candidate_sha256'],
    clock_source_stubbed=True, CRT_memory_operations_stubbed=True, audio_dispatch_tail_stubbed=True,
    entire_entry_cleanup_or_packet_handler_executed=False, modern_scene_binding_verified=False,
    candidate_applied_to_product=False, GUI_or_real_clock_timing_verified=False,
    full_start_cancel_rearm_binding_verified=False, whole_goal_complete=False)
binding = read(binding_path)
assert binding['result3_is_not_directory_opcode3'] and binding['reuse_filter_removes_background_green_water_and_aux']
binding_evidence = dict(status=binding['status'], evidence=str(binding_path), sha256=sha(binding_path),
    result3_is_not_directory_opcode3=True, primary_D1_is_named_participant_readiness=True,
    ready_full_or_reuse_requires_self_name_and_state1=True,
    ready_full_or_reuse_selector_globals_unmapped=True, modern_controller_binding_verified=False,
    product_source_changed=False, whole_goal_complete=False)
resource=read(resource_path)
assert resource['status']=='independently_readback_qualified_current_ten_unique_original_resource_bodies_both_profiles'
assert resource['source_manifest_sha256']==qr['source_manifest_sha256']
assert resource['actual_test_invocations']==20 and resource['unique_previously_ignored_bodies']==10
assert all(p['passed_optin_bodies']==10 and p['failed']==0 for p in resource['profiles'].values())
resource_evidence=dict(status=resource['status'],qualification=str(resource_path),sha256=sha(resource_path),
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=resource['source_manifest_sha256'],
    profiles=resource['profiles'],unique_previously_ignored_bodies=10,actual_test_invocations=20,actual_test_executables=2,
    original_sessions=[5566,6573],observed_terminal_exit_codes=[0,0],recorded_test_processes=20,
    recorded_test_processes_terminal=True,unchanged_default_library_ignored_count=473,
    other_library_ignored_bodies_not_qualified_by_these_scopes=463,sixth_scene_test_qualified=False,
    actual_formal_sessions=0,GUI_verified=False,audio_device_verified=False,performance_accepted=False,
    all_resources_or_ignored_bodies_verified=False,whole_goal_complete=False)
note = ('背景生命週期新核對：結果處理器case3不是S03角色目錄，D1是具名準備通知，自身state1另分完整/重用初始化。'
        '重用路徑4D64F0只留兩種回呼，會移除背景/green/water/aux，與保留slot6的4D63A0/4D6470不同。'
        '追加原filter後合1336原bytes/15項，16155actualexit0與純Rust候選5ce4dbactualexit0已獨立讀回18狀態快照/90佇列資料逐欄符合。'
        '原時鐘與CRT/audio尾端供應隔離stub，3片段各附RET，沒有執行完整初始化/cleanup/封包handler，scene/GUI接線仍待；候選及產品來源未改。'
        '當版4C7DE771廣域Debug487/0/21已核對，Release沿原24147執行，產品來源與四QA維持固定。'
        '當版完整library之10項原資源opt-in本文兩版各10過0敗，5566/6573actualexit0、20invocations/2映像及20測試程序退出已獨立核對；'
        '473預設ignored不改，其餘463不在本次scope，舊scene Disintegrate缺pack條件不推定為正常native玩法結果。'
        'G11原預算及必要ignored、GUI/實體輸入/IME/多DPI、clean host與完整G01–G12仍待。')
for name in files[:4]:
    path = ROOT/name
    value = read(path)
    previous_resource=value.get('current_original_resource_optins')
    if previous_resource and previous_resource.get('qualification')!=str(resource_path):
        value.setdefault('retained_original_resource_optins_before_current4C7_ten',previous_resource)
    value.update(current_background_shared_clock_candidate=evidence,
                 current_background_queue_lifecycle_candidate=queue_evidence,
                 current_background_lifecycle_binding_audit=binding_evidence,
                 current_original_resource_optins=resource_evidence,
                 completed_current_resource_optin_sessions=[5566,6573],
                 last_goal_turn_classification='progress',
                 last_goal_turn_evidence=[str(queue_native), str(queue_rust), str(binding_path), str(resource_path), str(cohort)],
                 progress_updated_utc=datetime.now(timezone.utc).isoformat(),
                 source_freeze_for_live_default110_job=True,
                 current_working_tree_verified=False, goal_complete=False)
    save(path, value)
for name in files[4:]:
    path = ROOT/name
    header, separator, body = path.read_text(encoding='utf-8-sig').partition('\n')
    path.write_text(header+'\n\n2026-10-17：'+note+'\n'+body, encoding='utf-8', newline='\n')
after = {name: sha(ROOT/name) for name in files}
for name in files[:4]:
    assert read(ROOT/name)['current_background_shared_clock_candidate'] == evidence
assert sha(base) == c['candidate_files']['base-native_water_clock.rs']
for name, digest in live['validation_pins'].items():
    assert sha(ROOT/'維護工具'/name) == digest
save(out/'record.json', dict(before=before, before_archive_sha256=sha(out/'before.zip'),
    after=after, evidence=evidence, queue_evidence=queue_evidence, binding_evidence=binding_evidence,
    resource_evidence=resource_evidence,
    product_source_changed=False, whole_goal_complete=False))
(out/'checkpoint-tool.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(checkpoint=str(out), status='scoped_background_candidate_trackers_updated'), ensure_ascii=False))
